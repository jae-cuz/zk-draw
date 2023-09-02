use halo2_proofs::{
    plonk::{ConstraintSystem, Error, Column, Advice, Selector, Instance, Expression, Circuit, create_proof, keygen_vk, keygen_pk, ProvingKey, VerifyingKey, verify_proof, SingleVerifier},
    circuit::{Layouter, Value, SimpleFloorPlanner, AssignedCell},
    poly::{Rotation, commitment::Params}, pasta::{Fp, group::ff::PrimeField, EqAffine},transcript::{Blake2bWrite, Challenge255, Blake2bRead}, 
};
use rand_core::OsRng;
struct ACell (AssignedCell<Fp,Fp>);

#[derive(Clone, Debug)]
pub struct GachaConfig<const MULTIPLIER: u64, const ADDER: u64> {
    adv: [Column<Advice>; 2],
    divisor: Column<Advice>,
    inst: Column<Instance>,
    selector: Selector,
}

impl<const MULTIPLIER: u64, const ADDER: u64> GachaConfig<MULTIPLIER, ADDER> {
    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self {
        let adv_0 = meta.advice_column();
        let adv_1 = meta. advice_column();
        let divisor = meta.advice_column();
        let selector = meta.selector();
        let inst = meta.instance_column();

        meta.enable_equality(adv_0);
        meta.enable_equality(adv_1);
        meta.enable_equality(inst);

        meta.create_gate("linear congruence", |meta| {
            let x = meta.query_advice(adv_0, Rotation::cur());
            let b = meta.query_advice(adv_1, Rotation::cur());
            let d = meta.query_advice(divisor, Rotation::cur());

            let a = Expression::Constant(Fp::from(MULTIPLIER));
            let m = Expression::Constant(Fp::from(65536));
            let c = Expression::Constant(Fp::from(ADDER));

            let s = meta.query_selector(selector);

            vec![s * (a * x + c - b - m * d)]
        });

        GachaConfig {
            adv: [adv_0, adv_1],
            divisor: divisor, 
            inst: inst,
            selector: selector,
        }
    }


    fn assign_first_row(
        &self,
        mut layouter: impl Layouter<Fp>,
        prev: Value<Fp>,
    ) -> Result<ACell, Error> {
        layouter.assign_region(|| "linear operation", |mut region| {
            let offset = 0;

            self.selector.enable(&mut region, offset)?;

            let next_val = prev.map(|a| {
                a * Fp::from(MULTIPLIER) + Fp::from(ADDER)
            });

            let rem_val = next_val.map(rem);
            let quot_val = next_val.map(quot);      

            region.assign_advice(|| "seed", self.adv[0], offset, || prev).map(ACell)?;
            let next_cell = region.assign_advice(|| "next value mod m", self.adv[1], offset, || rem_val).map(ACell)?;
            region.assign_advice(|| "quotient", self.divisor, offset, || quot_val).map(ACell)?;

            Ok(next_cell)
        })
    }

    fn assign_next_row (
        &self,
        mut layouter: impl Layouter<Fp>,
        prev: &ACell,
    ) -> Result<ACell, Error> {
        layouter.assign_region(|| "linear operation", |mut region| {
            let offset = 0;

            self.selector.enable(&mut region, offset)?;

            let prev_val = prev.0.value().copied();
            let next_val = prev_val.map(|a| {
                a * Fp::from(MULTIPLIER) + Fp::from(ADDER)
            }); 
            let rem_val = next_val.map(rem);
            let quot_val = next_val.map(quot);      

            prev.0.copy_advice(|| "prev", &mut region, self.adv[0], offset)?;
            let next_cell = region.assign_advice(|| "next value mod m", self.adv[1], offset, || rem_val).map(ACell)?;
            region.assign_advice(|| "quotient", self.divisor, offset, || quot_val).map(ACell)?;

            Ok(next_cell)
        })
    }
    
    fn expose_public(
        &self,
        mut layouter: impl Layouter<Fp>,
        cell: &ACell,
        row: usize,
    ) -> Result<(), Error> {
        layouter.constrain_instance(cell.0.cell(), self.inst, row)
    }
}

#[derive(Debug, Default)]
pub struct GachaCircuit<const MULTIPLIER: u64, const ADDER: u64> {
    seed: Value<Fp>,
    n: u64,
}

impl<const MULTIPLIER: u64, const ADDER: u64> Circuit<Fp> for GachaCircuit<MULTIPLIER, ADDER> {
    type Config = GachaConfig<MULTIPLIER, ADDER>;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        GachaConfig::configure(meta)
    }

    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<Fp>) -> Result<(), Error> {
        let mut prev = config.assign_first_row(layouter.namespace(|| "first row"), self.seed)?;

        for _i in 1..self.n {
            prev = config.assign_next_row(layouter.namespace(|| "next row"), &prev)?;
        }

        config.expose_public(layouter.namespace(|| "out"), &prev, 0)?;
        
        Ok(())
    }
}

fn rem(input: Fp) -> Fp {
    let repr = input.to_repr();
    let mut rem_repr: [u8; 32] = [0; 32];
    rem_repr[0] = repr[0];
    rem_repr[1] = repr[1];
    Fp::from_repr(rem_repr).unwrap()
}

fn quot(input: Fp) -> Fp {
    let mut repr = input.to_repr();
    for i in 0..30 {
        repr[i] = repr[i+2];
    }
    repr[30] = 0;
    repr[31] = 0;
    Fp::from_repr(repr).unwrap()
}

pub fn get_random(
    seed: u64,
    multiplier: u64,
    adder: u64,
    modulus: u64,
    number_of_iter: u64,
) -> u64 {
    let mut ret = seed;
    for _i in 0..number_of_iter {
        ret = (ret * multiplier + adder) % modulus;
    }
    ret
}


pub fn generate_setup_params(
    k: u32,
) -> Params<EqAffine> {
    Params::<EqAffine>::new(k)
}
pub fn generate_keys<const MULTIPLIER:u64, const ADDER:u64>(
    params: &Params<EqAffine>,
    circuit: &GachaCircuit<{MULTIPLIER},{ADDER}>,
) -> (ProvingKey<EqAffine>, VerifyingKey<EqAffine>) {
    // just to emphasize that for vk, pk we don't need to know the value of `x`
    let vk = keygen_vk(params, circuit).expect("vk should not fail");
    let pk = keygen_pk(params, vk.clone(), circuit).expect("pk should not fail");
    (pk, vk)
}

pub fn empty_circuit<const MULTIPLIER:u64, const ADDER:u64>(n : u64) -> GachaCircuit<{MULTIPLIER},{ADDER}> {
    GachaCircuit {
        seed: Value::unknown(),
        n,
    }
}

pub fn create_circuit<const MULTIPLIER:u64, const ADDER:u64>(seed:u64, n:u64) -> GachaCircuit<{MULTIPLIER},{ADDER}> {
    GachaCircuit {
        seed: Value::known(Fp::from(seed)),
        n,
    }
}

pub fn generate_proof<const MULTIPLIER:u64, const ADDER:u64>(
    params: &Params<EqAffine>,
    pk: &ProvingKey<EqAffine>,
    circuit: GachaCircuit<{MULTIPLIER},{ADDER}>,
    pub_input: &Vec<Fp>,
) -> Vec<u8> {
    println!("Generating proof...");
    let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
    create_proof(
        params, 
        pk, 
        &[circuit],
        &[&[pub_input]], 
        OsRng, 
        &mut transcript
    ).expect("Prover should not fail");
    transcript.finalize()
}

pub fn verify(
    params: &Params<EqAffine>,
    vk: &VerifyingKey<EqAffine>,
    pub_input: &Vec<Fp>,
    proof: Vec<u8>,
) -> Result<(), Error> {
    println!("Verifying proof...");
    let strategy = SingleVerifier::new(&params);
    let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);
    verify_proof(
        params, 
        vk, 
        strategy, 
        &[&[pub_input]], 
        &mut transcript,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use halo2_proofs::{
        dev::MockProver,
        pasta::Fp,
        circuit::Value,
    };

    #[test]
    fn test_rand() {
        let seed: u64 = 54352;

        const MULTIPLIER: u64 = 15;
        const ADDER: u64 = 3;
        let modulus: u64 = 1 << 16;

        let multiplier = MULTIPLIER;
        let adder = ADDER;

        for i in 1..30 {
            let number_of_iter: u64 = i;

            let circuit = GachaCircuit::<MULTIPLIER, ADDER> {
                seed: Value::known(Fp::from(seed)),
                n: number_of_iter,
            };
    
            let rand = get_random(seed, multiplier, adder, modulus, number_of_iter);
            println!("{}", rand);
            let public_input = vec![Fp::from(rand)];
            let prover = MockProver::run(10, &circuit, vec![public_input]).unwrap();
            prover.assert_satisfied();
        }
    }

    #[cfg(feature = "dev-graph")]
    #[test]
    fn print_test_rand() {
        use plotters::prelude::*;

        let seed: u64 = 54352;

        let number_of_iter: u64 = 30;
        const MULTIPLIER: u64 = 15;
        const ADDER: u64 = 3;
        let modulus: u64 = 1 << 16;

        let multiplier = MULTIPLIER;
        let adder = ADDER;

        let circuit = GachaCircuit::<MULTIPLIER, ADDER> {
            seed: Value::known(Fp::from(seed)),
            n: number_of_iter,
        };
    
        let root = BitMapBackend::new("rand.png", (1024, 3096)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root.titled("Rand Layout", ("sans-serif", 60)).unwrap();

        halo2_proofs::dev::CircuitLayout::default()
            .render(5, &circuit, &root)
            .unwrap();
    }
}
