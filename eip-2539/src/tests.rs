use ark_bls12_377::{Fq, Fr, G1Affine, G1Projective as G1, G2Affine, G2Projective as G2};
use ark_ec::CurveGroup;
use ark_ff::PrimeField;
use ark_std::ops::Mul;
use ark_std::test_rng;
use ark_std::UniformRand;
use ark_std::Zero;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;

const NUM_TESTS: usize = 100;
const PREFIX: &str = "bls12377";
const FE_SIZE: usize = 48;
const SCALAR_SIZE: usize = 32;
const WORD_SIZE: usize = 64;

#[derive(Serialize, Deserialize)]
struct VectorSuccess {
    input: String,
    expected: String,
    name: String,
}

#[derive(Serialize, Deserialize)]
struct VectorFail {
    input: String,
    expected_error: String,
    name: String,
}

fn write_vectors(vectors: Vec<VectorSuccess>, name: &str) {
    let serialized: String = serde_json::to_string(&vectors).unwrap();
    let mut file = File::create(PREFIX.to_string() + name + ".json").expect("must create the file");
    file.write(serialized.as_bytes())
        .expect("must write vectors");
}

fn write_vectors_fail(vectors: Vec<VectorFail>, name: &str) {
    let serialized: String = serde_json::to_string(&vectors).unwrap();
    let mut file = File::create(PREFIX.to_string() + name + ".json").expect("must create the file");
    file.write(serialized.as_bytes())
        .expect("must write vectors");
}

fn encode_fq(field: Fq) -> [u8; 64] {
    let mut result = [0u8; 64];
    let rep = field.into_bigint();

    result[16..24].copy_from_slice(&rep.0[5].to_be_bytes());
    result[24..32].copy_from_slice(&rep.0[4].to_be_bytes());
    result[32..40].copy_from_slice(&rep.0[3].to_be_bytes());
    result[40..48].copy_from_slice(&rep.0[2].to_be_bytes());
    result[48..56].copy_from_slice(&rep.0[1].to_be_bytes());
    result[56..64].copy_from_slice(&rep.0[0].to_be_bytes());

    result
}

fn encode_fr(r: Fr) -> [u8; 32] {
    let mut result = [0u8; 32];
    let rep = r.into_bigint();

    result[0..8].copy_from_slice(&rep.0[3].to_be_bytes());
    result[8..16].copy_from_slice(&rep.0[2].to_be_bytes());
    result[16..24].copy_from_slice(&rep.0[1].to_be_bytes());
    result[24..32].copy_from_slice(&rep.0[0].to_be_bytes());

    result
}

fn encode_g1(g1: G1Affine) -> [u8; 128] {
    let mut result = [0u8; 128];
    let x_bytes = encode_fq(g1.x);
    result[0..64].copy_from_slice(&x_bytes[..]);
    let y_bytes = encode_fq(g1.y);
    result[64..128].copy_from_slice(&y_bytes[..]);
    result
}

fn gen_g1_add_vectors() {
    let mut rng = test_rng();
    let mut vectors: Vec<VectorSuccess> = vec![];
    for i in 0..NUM_TESTS {
        let mut input_bytes: Vec<u8> = vec![];
        let a = G1::rand(&mut rng).into_affine();
        let b = G1::rand(&mut rng).into_affine();
        let a_bytes = encode_g1(a);
        let b_bytes = encode_g1(b);
        input_bytes.extend(a_bytes);
        input_bytes.extend(b_bytes);
        let input: String = hex::encode(input_bytes.clone());

        let r = a + b;
        let result_bytes: Vec<u8> = encode_g1(r.into_affine()).to_vec();
        let result: String = hex::encode(result_bytes);
        let vector = VectorSuccess {
            input,
            expected: result,
            name: format!("{}_{}", "g1_add", i + 1),
        };

        vectors.push(vector);
    }
    write_vectors(vectors, "G1Add");
}

fn gen_g1_mul_vectors() {
    let mut rng = test_rng();
    let mut vectors: Vec<VectorSuccess> = vec![];
    for i in 0..NUM_TESTS {
        let mut input_bytes: Vec<u8> = vec![];

        let a = G1::rand(&mut rng).into_affine();
        let e = Fr::rand(&mut rng);
        let a_bytes = encode_g1(a);
        let e_bytes = encode_fr(e);

        input_bytes.extend(a_bytes);
        input_bytes.extend(e_bytes);
        let input: String = hex::encode(input_bytes.clone());

        let r = a.mul(e);
        let result_bytes: Vec<u8> = encode_g1(r.into_affine()).to_vec();
        let result: String = hex::encode(result_bytes);
        let vector = VectorSuccess {
            input,
            expected: result,
            name: format!("{}_{}", "g1_mul", i + 1),
        };
        vectors.push(vector);
    }
    write_vectors(vectors, "G1Mul");
}

fn gen_g1_multiexp_vectors() {
    let mut rng = test_rng();
    let mut vectors: Vec<VectorSuccess> = vec![];
    let mul_pair_size: usize = NUM_TESTS;
    for i in 1..mul_pair_size + 1 {
        let mut input_bytes: Vec<u8> = vec![];
        let mut acc = G1::zero();
        for _ in 0..i {
            let a = G1::rand(&mut rng).into_affine();
            let e = Fr::rand(&mut rng);
            let a_bytes = encode_g1(a);
            let e_bytes = encode_fr(e);

            input_bytes.extend(a_bytes);
            input_bytes.extend(e_bytes);

            acc += a.mul(e);
        }
        let input: String = hex::encode(input_bytes.clone());

        let result_bytes: Vec<u8> = encode_g1(acc.into_affine()).to_vec();
        let result: String = hex::encode(result_bytes);
        let vector = VectorSuccess {
            input,
            expected: result,
            name: format!("{}_{}", "g1_multiexp", i + 1),
        };
        vectors.push(vector);
    }
    write_vectors(vectors, "G1MultiExp");
}

// fn gen_g2_add_vectors() {
//     let mut rng = test_rng();
//     let mut vectors: Vec<VectorSuccess> = vec![];
//     for i in 0..NUM_TESTS {
//         let mut input_bytes: Vec<u8> = vec![];
//         let mut a: G2Projective = rng.gen();
//         let b: G2Projective = rng.gen();
//         let a_bytes: Vec<u8> = encode_g2(a);
//         let b_bytes: Vec<u8> = encode_g2(b);
//         input_bytes.extend(a_bytes);
//         input_bytes.extend(b_bytes);
//         let input: String = hex::encode(input_bytes.clone());

//         a.add_assign(b);
//         let result_bytes: Vec<u8> = encode_g2(a);
//         let result: String = hex::encode(result_bytes);
//         let vector = VectorSuccess {
//             input,
//             expected: result,
//             name: format!("{}_{}", "g2_add", i + 1),
//         };
//         vectors.push(vector);
//     }
//     write_vectors(vectors, "_g2_add");
// }

#[test]
fn generate_test_vectors() {
    gen_g1_add_vectors();
    gen_g1_mul_vectors();
    gen_g1_multiexp_vectors();
    // gen_g2_add_vectors();
    // gen_g2_mul_vectors();
    // gen_g2_multiexp_vectors();
    // gen_pairing_vectors();
}

// #[test]
// fn generate_test_vectors() {
//     gen_fail_g1_add_vectors();
//     gen_fail_g1_mul_vectors();
//     gen_fail_g1_multiexp_vectors();
//     gen_fail_g2_add_vectors();
//     gen_fail_g2_mul_vectors();
//     gen_fail_g2_multiexp_vectors();
//     gen_fail_pairing();
// }
