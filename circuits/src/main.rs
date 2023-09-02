#[cfg(not(target_family = "wasm"))]
fn main() {
    use halo2_proofs::pasta::Fp;
    use gacha::gacha::{GachaCircuit, get_random, generate_setup_params, generate_keys, empty_circuit, create_circuit, generate_proof, verify};

    // Size of the circuit. Circuit must fit within 2^k rows.
    let k = 6;

    const MULTIPLIER: u64 = 15;
    const ADDER: u64 = 3;
    let modulus: u64 = 1 << 16;

    let multiplier = MULTIPLIER;
    let adder = ADDER;

    let n = 30;

    // Input values to generate a proof with
    let seed = 54352;
    let random_value = get_random(seed, multiplier, adder, modulus, n);

    // Create circuit
    let gacha_circuit: GachaCircuit<MULTIPLIER, ADDER> = create_circuit(seed, n);
    
    // Generate setup params
    let params = generate_setup_params(k);

    // Generate proving and verifying keys
    let empty_circuit: GachaCircuit<MULTIPLIER, ADDER> = empty_circuit(n);
    let (pk, vk) = generate_keys(&params, &empty_circuit);

    // Generate proof
    let proof = generate_proof(&params, &pk, gacha_circuit, &vec![Fp::from(random_value)]);
    
    // Verify proof
    let verify = verify(&params, &vk, &vec![Fp::from(random_value)], proof);
    println!("Verify result: {:?}", verify);
}