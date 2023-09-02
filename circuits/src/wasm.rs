use std::io::BufReader;
use crate::gacha::{GachaCircuit, get_random, generate_setup_params, generate_keys, empty_circuit, create_circuit, generate_proof, verify}; //hammster::{calculate_hamming_distance, create_circuit, empty_circuit, generate_setup_params, generate_keys, generate_proof, verify};
use halo2_proofs::{
    poly::commitment::Params, 
    pasta::{Fp, EqAffine}, 
    plonk::keygen_vk
};
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

fn copy_vec_to_u8arr(v: &Vec<u8>) -> Uint8Array {
    let u8_arr = Uint8Array::new_with_length(v.len() as u32);
    u8_arr.copy_from(v);
    u8_arr
}

#[wasm_bindgen]
pub fn setup_params(k: u32) -> Uint8Array {
    log("running setup");
    
    // Generate setup params
    let params = generate_setup_params(k); 
    let mut buf = vec![];
    params.write(&mut buf).expect("Can write params");

    copy_vec_to_u8arr(&buf)
}

#[wasm_bindgen]
pub fn generate_random(seed: u64) -> u64 {
    const MULTIPLIER: u64 = 15;
    const ADDER: u64 = 3;
    let modulus: u64 = 1 << 16;

    let multiplier = MULTIPLIER;
    let adder = ADDER;

    let n = 30;

    get_random(seed, multiplier, adder, modulus, n)
}

#[wasm_bindgen]
pub fn proof_generate(
    seed: u64,
    params_bytes: &[u8],
) -> Uint8Array {
    log("proving...");
    const MULTIPLIER: u64 = 15;
    const ADDER: u64 = 3;
    let modulus: u64 = 1 << 16;

    let multiplier = MULTIPLIER;
    let adder = ADDER;

    let n = 30;

    let params = Params::<EqAffine>::read(&mut BufReader::new(params_bytes)).expect("params should not fail to read");

    let random_value = get_random(seed, multiplier, adder, modulus, n);

    // Generate proving key
    let empty_circuit : GachaCircuit<MULTIPLIER,ADDER> = empty_circuit(n);
    let (pk, _vk) = generate_keys(&params, &empty_circuit);
    
    // Generate proof
    let gacha_circuit : GachaCircuit<MULTIPLIER, ADDER> = create_circuit(seed, n);
    let proof = generate_proof(&params, &pk, gacha_circuit, &vec![Fp::from(random_value)]);
    
    copy_vec_to_u8arr(&proof)
}

#[wasm_bindgen]
pub fn proof_verify(
    params_bytes: &[u8], 
    random_value: u64,
    proof: &[u8]
) -> bool {
    log("verifying...");

    const MULTIPLIER: u64 = 15;
    const ADDER: u64 = 3;
    let n = 30;

    let params = Params::<EqAffine>::read(&mut BufReader::new(params_bytes)).expect("params should not fail to read");

    // Generate verifying key
    let empty_circuit: GachaCircuit<MULTIPLIER, ADDER> = empty_circuit(n);
    let vk = keygen_vk(&params, &empty_circuit).expect("vk should not fail to generate");

    // Transform params for verify function
    let random_value_fp = vec![Fp::from(random_value as u64)];
    let proof_vec = proof.to_vec();

    // Verify the proof and public input
    let ret_val = verify(&params, &vk, &random_value_fp, proof_vec);
    match ret_val {
        Err(_) => false,
        _ => true,
    }
}