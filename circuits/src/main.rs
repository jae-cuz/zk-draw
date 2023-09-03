#[cfg(not(target_family = "wasm"))]
fn main() {
    use halo2_proofs::pasta::Fp;
    use gacha::gacha::{GachaCircuit, get_random, generate_setup_params, generate_keys, empty_circuit, create_circuit, generate_proof, verify};

    // Size of the circuit. Circuit must fit within 2^k rows.
    let k = 6;

    const N: u64 = 30;

    // Input values to generate a proof with
    let seed = 54352;
    let random_value = get_random(seed, N);

    // Create circuit
    let gacha_circuit: GachaCircuit<N> = create_circuit(seed);
    
    // Generate setup params
    let params = generate_setup_params(k);

    // Generate proving and verifying keys
    let empty_circuit: GachaCircuit<N> = empty_circuit();
    let (pk, vk) = generate_keys(&params, &empty_circuit);

    // Generate proof
    let proof = generate_proof(&params, &pk, gacha_circuit, &vec![Fp::from(random_value)]);
    
    // Verify proof
    let verify = verify(&params, &vk, &vec![Fp::from(random_value)], proof);
    println!("Verify result: {:?}", verify);
}