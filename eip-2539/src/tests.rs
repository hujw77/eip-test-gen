use ark_bls12_377::{Fq, Fq2, Fr, G1Affine, G1Projective as G1, G2Affine, G2Projective as G2};
use ark_ec::{CurveGroup, Group};
use ark_ff::{Field, MontFp, PrimeField};
use ark_std::ops::{Mul, Neg};
use ark_std::test_rng;
use ark_std::UniformRand;
use ark_std::{One, Zero};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;

const NUM_TESTS: usize = 100;
const PREFIX: &str = "bls12377";
const FAIL_PREFIX: &str = "fail-bls12377";
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
    let mut file =
        File::create(FAIL_PREFIX.to_string() + name + ".json").expect("must create the file");
    file.write(serialized.as_bytes())
        .expect("must write vectors");
}

fn gen_fail_vectors(input_len: usize) -> Vec<VectorFail> {
    let mut vectors: Vec<VectorFail> = vec![];

    // invalid length: empty
    {
        let vector = VectorFail {
            input: String::from(""),
            expected_error: String::from("invalid input length"),
            name: format!("invalid_input_length_empty"),
        };
        vectors.push(vector);
    }

    // invalid length: short
    {
        let vector = VectorFail {
            input: String::from(""),
            expected_error: String::from("invalid input length"),
            name: format!("invalid_input_length_short"),
        };
        vectors.push(vector);
    }

    // invalid length: long
    {
        let input: String = hex::encode(vec![1u8; input_len + 1]);
        let vector = VectorFail {
            input,
            expected_error: String::from("invalid input length"),
            name: format!("invalid_input_length_large"),
        };
        vectors.push(vector);
    }

    // violate top zeros
    {
        let input: String = hex::encode(vec![1u8; input_len]);
        let vector = VectorFail {
            input,
            expected_error: String::from("invliad Fq"),
            name: format!("violate_top_zero_bytes"),
        };
        vectors.push(vector);
    }

    vectors
}

fn number_larger_than_modulus() -> Vec<u8> {
    hex::decode("01ae3a4617c510eac63b05c06ca1493b1a22d9f300f5138f1ef3622fba094800170b5d44300000008508c00000000002")
        .expect("must decode")
}

fn rand_g1_point_not_on_curve() -> G1 {
    let mut rng = test_rng();
    let x = Fq::rand(&mut rng);
    let y = Fq::rand(&mut rng);
    let p = G1Affine::new_unchecked(x, y);
    assert!(!p.is_on_curve());
    p.into()
}
fn rand_g2_point_not_on_curve() -> G2 {
    let mut rng = test_rng();
    let x = Fq2::rand(&mut rng);
    let y = Fq2::rand(&mut rng);
    let p = G2Affine::new_unchecked(x, y);
    assert!(!p.is_on_curve());
    p.into()
}

fn rand_g1_point_not_on_correct_subgroup() -> G1 {
    let mut rng = test_rng();

    loop {
        let x = Fq::rand(&mut rng);
        let mut y: Fq = x * x;
        y *= x;
        y += Fq::one();
        // y.sqrt().
        if let Some(y) = y.sqrt() {
            let p = G1Affine::new_unchecked(x, y);
            assert!(p.is_on_curve());
            assert!(!p.is_in_correct_subgroup_assuming_on_curve());
            return p.into();
        }
    }
}

fn rand_g2_point_not_on_correct_subgroup() -> G2 {
    let mut rng = test_rng();

    loop {
        let x = Fq2::rand(&mut rng);
        let mut y: Fq2 = x * x;
        y *= x;
        y += Fq2::new(
			Fq::zero(),
			MontFp!("155198655607781456406391640216936120121836107652948796323930557600032281009004493664981332883744016074664192874906"),
		);
        if let Some(y) = y.sqrt() {
            let p = G2Affine::new_unchecked(x, y);
            assert!(p.is_on_curve());
            assert!(!p.is_in_correct_subgroup_assuming_on_curve());
            return p.into();
        }
    }
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

fn encode_g1(g1: G1) -> [u8; 128] {
    let g = g1.into_affine();
    let mut result = [0u8; 128];
    let x_bytes = encode_fq(g.x);
    result[0..64].copy_from_slice(&x_bytes[..]);
    let y_bytes = encode_fq(g.y);
    result[64..128].copy_from_slice(&y_bytes[..]);
    result
}

fn encode_g2(g2: G2) -> [u8; 256] {
    let g = g2.into_affine();
    let mut result = [0u8; 256];
    let x0_bytes = encode_fq(g.x.c0);
    result[0..64].copy_from_slice(&x0_bytes[..]);
    let x1_bytes = encode_fq(g.x.c1);
    result[64..128].copy_from_slice(&x1_bytes[..]);
    let y0_bytes = encode_fq(g.y.c0);
    result[128..192].copy_from_slice(&y0_bytes[..]);
    let y1_bytes = encode_fq(g.y.c1);
    result[192..256].copy_from_slice(&y1_bytes[..]);
    result
}

fn gen_g1_add_vectors() {
    let mut rng = test_rng();
    let mut vectors: Vec<VectorSuccess> = vec![];
    for i in 0..NUM_TESTS {
        let mut input_bytes: Vec<u8> = vec![];
        let a = G1::rand(&mut rng);
        let b = G1::rand(&mut rng);
        let a_bytes = encode_g1(a);
        let b_bytes = encode_g1(b);
        input_bytes.extend(a_bytes);
        input_bytes.extend(b_bytes);
        let input: String = hex::encode(input_bytes.clone());

        let r = a + b;
        let result_bytes: Vec<u8> = encode_g1(r).to_vec();
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

        let a = G1::rand(&mut rng);
        let e = Fr::rand(&mut rng);
        let a_bytes = encode_g1(a);
        let e_bytes = encode_fr(e);

        input_bytes.extend(a_bytes);
        input_bytes.extend(e_bytes);
        let input: String = hex::encode(input_bytes.clone());

        let r = a.mul(e);
        let result_bytes: Vec<u8> = encode_g1(r).to_vec();
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
            let a = G1::rand(&mut rng);
            let e = Fr::rand(&mut rng);
            let a_bytes = encode_g1(a);
            let e_bytes = encode_fr(e);

            input_bytes.extend(a_bytes);
            input_bytes.extend(e_bytes);

            acc += a.mul(e);
        }
        let input: String = hex::encode(input_bytes.clone());

        let result_bytes: Vec<u8> = encode_g1(acc).to_vec();
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

fn gen_g2_add_vectors() {
    let mut rng = test_rng();
    let mut vectors: Vec<VectorSuccess> = vec![];
    for i in 0..NUM_TESTS {
        let mut input_bytes: Vec<u8> = vec![];
        let a = G2::rand(&mut rng);
        let b = G2::rand(&mut rng);
        let a_bytes: Vec<u8> = encode_g2(a).to_vec();
        let b_bytes: Vec<u8> = encode_g2(b).to_vec();
        input_bytes.extend(a_bytes);
        input_bytes.extend(b_bytes);
        let input: String = hex::encode(input_bytes.clone());

        let r = a + b;
        let result_bytes: Vec<u8> = encode_g2(r).to_vec();
        let result: String = hex::encode(result_bytes);
        let vector = VectorSuccess {
            input,
            expected: result,
            name: format!("{}_{}", "g2_add", i + 1),
        };
        vectors.push(vector);
    }
    write_vectors(vectors, "G2Add");
}

fn gen_g2_mul_vectors() {
    let mut rng = test_rng();
    let mut vectors: Vec<VectorSuccess> = vec![];
    for i in 0..NUM_TESTS {
        let mut input_bytes: Vec<u8> = vec![];

        let a = G2::rand(&mut rng);
        let e = Fr::rand(&mut rng);
        let a_bytes = encode_g2(a);
        let e_bytes = encode_fr(e);

        input_bytes.extend(a_bytes);
        input_bytes.extend(e_bytes);
        let input: String = hex::encode(input_bytes.clone());

        let r = a.mul(e);
        let result_bytes: Vec<u8> = encode_g2(r).to_vec();
        let result: String = hex::encode(result_bytes);
        let vector = VectorSuccess {
            input,
            expected: result,
            name: format!("{}_{}", "g1_mul", i + 1),
        };
        vectors.push(vector);
    }
    write_vectors(vectors, "G2Mul");
}

fn gen_g2_multiexp_vectors() {
    let mut rng = test_rng();
    let mut vectors: Vec<VectorSuccess> = vec![];
    let mul_pair_size: usize = NUM_TESTS;
    for i in 1..mul_pair_size + 1 {
        let mut input_bytes: Vec<u8> = vec![];
        let mut acc = G2::zero();
        for _ in 0..i {
            let a = G2::rand(&mut rng);
            let e = Fr::rand(&mut rng);
            let a_bytes = encode_g2(a);
            let e_bytes = encode_fr(e);

            input_bytes.extend(a_bytes);
            input_bytes.extend(e_bytes);

            acc += a.mul(e);
        }
        let input: String = hex::encode(input_bytes.clone());

        let result_bytes: Vec<u8> = encode_g2(acc).to_vec();
        let result: String = hex::encode(result_bytes);
        let vector = VectorSuccess {
            input,
            expected: result,
            name: format!("{}_{}", "g1_multiexp", i + 1),
        };
        vectors.push(vector);
    }
    write_vectors(vectors, "G2MultiExp");
}

fn gen_pairing_vectors() {
    let mut rng = test_rng();
    let mut vectors: Vec<VectorSuccess> = vec![];
    let mut positive_result_bytes: Vec<u8> = vec![0u8; 32];
    positive_result_bytes[31] = 1u8;
    let negative_result_bytes: Vec<u8> = vec![0u8; 32];
    let g1_inf_encoded: Vec<u8> = vec![0u8; 128];
    let g2_inf_encoded: Vec<u8> = vec![0u8; 256];

    let g1 = G1::generator();
    let g2 = G2::generator();

    // expect true
    {
        // a. single pair
        {
            let mut input_bytes: Vec<u8> = vec![];

            let mut bytes_a1 = g1_inf_encoded.clone();
            let mut bytes_a2 = encode_g2(g2.clone()).to_vec();
            input_bytes.extend(bytes_a1);
            input_bytes.extend(bytes_a2);

            let input: String = hex::encode(input_bytes.clone());

            let vector = VectorSuccess {
                input,
                expected: hex::encode(positive_result_bytes.clone()),
                name: format!("{}", "g2_pairing_1"),
            };
            vectors.push(vector);

            input_bytes.clear();
            bytes_a1 = encode_g1(g1.clone()).to_vec();
            bytes_a2 = g2_inf_encoded.to_vec().clone();
            input_bytes.extend(bytes_a1);
            input_bytes.extend(bytes_a2);

            let input: String = hex::encode(input_bytes.clone());

            let vector = VectorSuccess {
                input,
                expected: hex::encode(positive_result_bytes.clone()),
                name: format!("{}", "g2_pairing_2"),
            };
            vectors.push(vector);
        }

        // b. multiple pair
        {
            for i in 0..NUM_TESTS {
                let mut acc: Fr = Fr::zero();
                let pair_size: usize = i + 2;
                let mut input_bytes: Vec<u8> = vec![];
                // n-1 pairs
                for _ in 0..pair_size - 1 {
                    let e1 = Fr::rand(&mut rng);
                    let e2 = Fr::rand(&mut rng);
                    let a1 = g1.mul(e1);
                    let a2 = g2.mul(e2);
                    let bytes_a1 = encode_g1(a1);
                    let bytes_a2 = encode_g2(a2);
                    input_bytes.extend(bytes_a1);
                    input_bytes.extend(bytes_a2);
                    // println!("e1\n{}", e1);
                    // println!("e2\n{}", e2);
                    // println!("acc\n{}", acc);
                    acc += e1 * e2;
                }
                // println!("acc\n{}", acc);
                // last pair
                let a1 = g1.mul(acc.neg());
                // println!("nacc\n{}", acc.neg());
                let a2 = g2;
                let bytes_a1 = encode_g1(a1);
                let bytes_a2 = encode_g2(a2);
                input_bytes.extend(bytes_a1);
                input_bytes.extend(bytes_a2);

                let input: String = hex::encode(input_bytes.clone());
                let result: String = hex::encode(positive_result_bytes.clone());

                let vector = VectorSuccess {
                    input,
                    expected: result,
                    name: format!("{}_{}", "pairing", i + 2),
                };
                vectors.push(vector);
            }
        }
    }

    // expect false
    {
        for i in 0..NUM_TESTS {
            let pair_size: usize = i + 1;
            let mut input_bytes: Vec<u8> = vec![];
            for _ in 0..pair_size {
                let e1 = Fr::rand(&mut rng);
                let e2 = Fr::rand(&mut rng);
                let a1 = g1.mul(e1);
                let a2 = g2.mul(e2);
                let bytes_a1 = encode_g1(a1);
                let bytes_a2 = encode_g2(a2);
                input_bytes.extend(bytes_a1);
                input_bytes.extend(bytes_a2);
            }

            let input: String = hex::encode(input_bytes.clone());
            let result: String = hex::encode(negative_result_bytes.clone());

            let vector = VectorSuccess {
                input,
                expected: result,
                name: format!("{}_{}", "pairing", NUM_TESTS + i + 2),
            };
            vectors.push(vector);
        }
    }

    write_vectors(vectors, "Pairing");
}
fn gen_fail_g1_add_vectors() {
    let mut rng = test_rng();
    let input_len = 4 * WORD_SIZE;
    let pad_zeros: Vec<u8> = vec![0u8; WORD_SIZE - FE_SIZE];

    let mut vectors: Vec<VectorFail> = gen_fail_vectors(input_len);

    // large modulus
    {
        let a = G1::rand(&mut rng);

        let mut input_bytes: Vec<u8> = vec![];
        let a_bytes = encode_g1(a);
        input_bytes.extend(a_bytes);
        input_bytes.extend(pad_zeros.clone());
        input_bytes.extend(number_larger_than_modulus());
        input_bytes.extend(vec![0u8; WORD_SIZE]);

        let input: String = hex::encode(input_bytes.clone());
        let vector = VectorFail {
            input,
            expected_error: String::from("invliad Fq"),
            name: format!("large_field_element"),
        };
        vectors.push(vector);
    }

    // not on curve
    {
        let a = G1::rand(&mut rng);
        let b = rand_g1_point_not_on_curve();

        let a_bytes = encode_g1(a.into());
        let e_bytes = encode_g1(b.into());

        let mut input_bytes: Vec<u8> = vec![];
        input_bytes.extend(a_bytes);
        input_bytes.extend(e_bytes);

        let input: String = hex::encode(input_bytes.clone());
        let vector = VectorFail {
            input,
            expected_error: String::from("point is not on curve"),
            name: format!("point_not_on_curve"),
        };
        vectors.push(vector);
    }
    write_vectors_fail(vectors, "G1Add");
}

fn gen_fail_g1_mul_vectors() {
    let input_len = 2 * WORD_SIZE + SCALAR_SIZE;
    let pad_zeros: Vec<u8> = vec![0u8; WORD_SIZE - FE_SIZE];
    let mut vectors: Vec<VectorFail> = gen_fail_vectors(input_len);

    // large modulus
    {
        let mut input_bytes: Vec<u8> = vec![];
        // x
        input_bytes.extend(pad_zeros.clone());
        input_bytes.extend(number_larger_than_modulus());
        // y
        input_bytes.extend(vec![0u8; WORD_SIZE]);
        // e
        input_bytes.extend(vec![0u8; SCALAR_SIZE]);

        let input: String = hex::encode(input_bytes.clone());
        let vector = VectorFail {
            input,
            expected_error: String::from("invliad Fq"),
            name: format!("large_field_element"),
        };
        vectors.push(vector);
    }

    // not on curve
    {
        let a: G1 = rand_g1_point_not_on_curve();
        let a_bytes = encode_g1(a);

        let mut input_bytes: Vec<u8> = vec![];
        input_bytes.extend(a_bytes);
        input_bytes.extend(vec![0u8; SCALAR_SIZE]);

        let input: String = hex::encode(input_bytes.clone());
        let vector = VectorFail {
            input,
            expected_error: String::from("point is not on curve"),
            name: format!("point_not_on_curve"),
        };
        vectors.push(vector);
    }
    write_vectors_fail(vectors, "G1Mul");
}

fn gen_fail_g1_multiexp_vectors() {
    let mut rng = test_rng();
    let input_len = 3 * (2 * WORD_SIZE + SCALAR_SIZE);
    let pad_zeros: Vec<u8> = vec![0u8; WORD_SIZE - FE_SIZE];
    let mut vectors: Vec<VectorFail> = gen_fail_vectors(input_len);

    // large modulus
    {
        let a = G1::rand(&mut rng);
        let b = G1::rand(&mut rng);
        let e1 = Fr::rand(&mut rng);
        let e2 = Fr::rand(&mut rng);

        let mut input_bytes: Vec<u8> = vec![];

        let a_bytes = encode_g1(a);
        let e1_bytes = encode_fr(e1);
        input_bytes.extend(a_bytes);
        input_bytes.extend(e1_bytes);

        let b_bytes = encode_g1(b);
        let e2_bytes = encode_fr(e2);
        input_bytes.extend(b_bytes);
        input_bytes.extend(e2_bytes);

        input_bytes.extend(pad_zeros.clone());
        input_bytes.extend(number_larger_than_modulus());
        // y
        input_bytes.extend(vec![0u8; WORD_SIZE]);
        // e
        input_bytes.extend(vec![0u8; SCALAR_SIZE]);

        let input: String = hex::encode(input_bytes.clone());
        let vector = VectorFail {
            input,
            expected_error: String::from("invliad Fq"),
            name: format!("large_field_element"),
        };
        vectors.push(vector);
    }

    // not on curve
    {
        let a = G1::rand(&mut rng);
        let b = G1::rand(&mut rng);
        let c = rand_g1_point_not_on_curve();
        let e1 = Fr::rand(&mut rng);
        let e2 = Fr::rand(&mut rng);
        let e3 = Fr::rand(&mut rng);

        let mut input_bytes: Vec<u8> = vec![];

        let a_bytes = encode_g1(a);
        let e1_bytes = encode_fr(e1);
        input_bytes.extend(a_bytes);
        input_bytes.extend(e1_bytes);

        let b_bytes = encode_g1(b);
        let e2_bytes = encode_fr(e2);
        input_bytes.extend(b_bytes);
        input_bytes.extend(e2_bytes);

        let c_bytes = encode_g1(c);
        let e3_bytes = encode_fr(e3);
        input_bytes.extend(c_bytes);
        input_bytes.extend(e3_bytes);

        let input: String = hex::encode(input_bytes.clone());
        let vector = VectorFail {
            input,
            expected_error: String::from("point is not on curve"),
            name: format!("point_not_on_curve"),
        };
        vectors.push(vector);
    }
    write_vectors_fail(vectors, "G1MultiExp");
}

fn gen_fail_g2_add_vectors() {
    let mut rng = test_rng();
    let input_len = 8 * WORD_SIZE;
    let pad_zeros: Vec<u8> = vec![0u8; WORD_SIZE - FE_SIZE];
    let mut vectors: Vec<VectorFail> = gen_fail_vectors(input_len);

    // large modulus
    {
        let a = G2::rand(&mut rng);
        let mut input_bytes: Vec<u8> = vec![];
        let a_bytes = encode_g2(a);
        input_bytes.extend(a_bytes);

        // x0
        input_bytes.extend(pad_zeros.clone());
        input_bytes.extend(number_larger_than_modulus());
        // x1, y0, y1
        input_bytes.extend(vec![0u8; WORD_SIZE]);
        input_bytes.extend(vec![0u8; WORD_SIZE]);
        input_bytes.extend(vec![0u8; WORD_SIZE]);

        let input: String = hex::encode(input_bytes.clone());
        let vector = VectorFail {
            input,
            expected_error: String::from("invalid Fq"),
            name: format!("large_field_element"),
        };
        vectors.push(vector);
    }

    // not on curve
    {
        let a = G2::rand(&mut rng);
        let b: G2 = rand_g2_point_not_on_curve();

        let a_bytes = encode_g2(a);
        let e_bytes = encode_g2(b);

        let mut input_bytes: Vec<u8> = vec![];
        input_bytes.extend(a_bytes);
        input_bytes.extend(e_bytes);

        let input: String = hex::encode(input_bytes.clone());
        let vector = VectorFail {
            input,
            expected_error: String::from("point is not on curve"),
            name: format!("point_not_on_curve"),
        };
        vectors.push(vector);
    }
    write_vectors_fail(vectors, "G2Add");
}
fn gen_fail_g2_mul_vectors() {
    let input_len = 2 * 2 * WORD_SIZE + SCALAR_SIZE;
    let pad_zeros: Vec<u8> = vec![0u8; WORD_SIZE - FE_SIZE];
    let mut vectors: Vec<VectorFail> = gen_fail_vectors(input_len);

    // large modulus
    {
        let mut input_bytes: Vec<u8> = vec![];

        // x0
        input_bytes.extend(pad_zeros.clone());
        input_bytes.extend(number_larger_than_modulus());
        // x1, y0, y1
        input_bytes.extend(vec![0u8; WORD_SIZE]);
        input_bytes.extend(vec![0u8; WORD_SIZE]);
        input_bytes.extend(vec![0u8; WORD_SIZE]);
        // e
        input_bytes.extend(vec![0u8; SCALAR_SIZE]);

        let input: String = hex::encode(input_bytes.clone());
        let vector = VectorFail {
            input,
            expected_error: String::from("invalid Fq"),
            name: format!("large_field_element"),
        };
        vectors.push(vector);
    }

    // not on curve
    {
        let a: G2 = rand_g2_point_not_on_curve();
        let a_bytes = encode_g2(a);

        let mut input_bytes: Vec<u8> = vec![];
        input_bytes.extend(a_bytes);
        input_bytes.extend(vec![0u8; SCALAR_SIZE]);

        let input: String = hex::encode(input_bytes.clone());
        let vector = VectorFail {
            input,
            expected_error: String::from("point is not on curve"),
            name: format!("point_not_on_curve"),
        };
        vectors.push(vector);
    }
    write_vectors_fail(vectors, "G2Mul_Fail");
}

fn gen_fail_g2_multiexp_vectors() {
    let mut rng = test_rng();
    let input_len = 3 * (2 * 2 * WORD_SIZE + SCALAR_SIZE);
    let pad_zeros: Vec<u8> = vec![0u8; WORD_SIZE - FE_SIZE];
    let mut vectors: Vec<VectorFail> = gen_fail_vectors(input_len);

    // large modulus
    {
        let a = G2::rand(&mut rng);
        let b = G2::rand(&mut rng);
        let e1 = Fr::rand(&mut rng);
        let e2 = Fr::rand(&mut rng);

        let mut input_bytes: Vec<u8> = vec![];

        let a_bytes = encode_g2(a);
        let e1_bytes = encode_fr(e1);
        input_bytes.extend(a_bytes);
        input_bytes.extend(e1_bytes);

        let b_bytes = encode_g2(b);
        let e2_bytes = encode_fr(e2);
        input_bytes.extend(b_bytes);
        input_bytes.extend(e2_bytes);

        // x0
        input_bytes.extend(pad_zeros.clone());
        input_bytes.extend(number_larger_than_modulus());
        // x1, y0, y1
        input_bytes.extend(vec![0u8; WORD_SIZE]);
        input_bytes.extend(vec![0u8; WORD_SIZE]);
        input_bytes.extend(vec![0u8; WORD_SIZE]);
        // e
        input_bytes.extend(vec![0u8; SCALAR_SIZE]);

        let input: String = hex::encode(input_bytes.clone());
        let vector = VectorFail {
            input,
            expected_error: String::from("invalid Fq"),
            name: format!("large_field_element"),
        };
        vectors.push(vector);
    }

    // not on curve
    {
        let a = G2::rand(&mut rng);
        let b = G2::rand(&mut rng);
        let c = rand_g2_point_not_on_curve();
        let e1 = Fr::rand(&mut rng);
        let e2 = Fr::rand(&mut rng);
        let e3 = Fr::rand(&mut rng);

        let mut input_bytes: Vec<u8> = vec![];

        let a_bytes = encode_g2(a);
        let e1_bytes = encode_fr(e1);
        input_bytes.extend(a_bytes);
        input_bytes.extend(e1_bytes);

        let b_bytes = encode_g2(b);
        let e2_bytes = encode_fr(e2);
        input_bytes.extend(b_bytes);
        input_bytes.extend(e2_bytes);

        let c_bytes = encode_g2(c);
        let e3_bytes = encode_fr(e3);
        input_bytes.extend(c_bytes);
        input_bytes.extend(e3_bytes);

        let input: String = hex::encode(input_bytes.clone());
        let vector = VectorFail {
            input,
            expected_error: String::from("point is not on curve"),
            name: format!("point_not_on_curve"),
        };
        vectors.push(vector);
    }
    write_vectors_fail(vectors, "G2MultiExp");
}
fn gen_fail_pairing() {
    let mut rng = test_rng();
    let input_len = 3 * 4 * WORD_SIZE;
    let mut vectors: Vec<VectorFail> = gen_fail_vectors(input_len);
    let pad_zeros: Vec<u8> = vec![0u8; WORD_SIZE - FE_SIZE];

    // large modulus
    {
        let mut input_bytes: Vec<u8> = vec![];

        let a1 = G1::rand(&mut rng);
        let a2 = G2::rand(&mut rng);
        let a1_bytes = encode_g1(a1);
        let a2_bytes = encode_g2(a2);
        input_bytes.extend(a1_bytes);
        input_bytes.extend(a2_bytes);

        let b1 = G1::rand(&mut rng);
        let b2 = G2::rand(&mut rng);
        let b1_bytes = encode_g1(b1);
        let b2_bytes = encode_g2(b2);

        input_bytes.extend(b1_bytes);
        input_bytes.extend(b2_bytes);

        // c1x
        input_bytes.extend(pad_zeros.clone());
        input_bytes.extend(number_larger_than_modulus());
        // c1y
        input_bytes.extend(vec![0u8; WORD_SIZE]);
        // c2
        input_bytes.extend(vec![0u8; 4 * WORD_SIZE]);

        let input: String = hex::encode(input_bytes.clone());
        let vector = VectorFail {
            input,
            expected_error: String::from("invalid Fq"),
            name: format!("large_field_element"),
        };
        vectors.push(vector);
    }

    // not on curve g1
    {
        let mut input_bytes: Vec<u8> = vec![];

        let a1 = G1::rand(&mut rng);
        let a2 = G2::rand(&mut rng);
        let a1_bytes = encode_g1(a1);
        let a2_bytes = encode_g2(a2);
        input_bytes.extend(a1_bytes);
        input_bytes.extend(a2_bytes);

        let b1 = G1::rand(&mut rng);
        let b2 = G2::rand(&mut rng);
        let b1_bytes = encode_g1(b1);
        let b2_bytes = encode_g2(b2);
        input_bytes.extend(b1_bytes);
        input_bytes.extend(b2_bytes);

        let c1: G1 = rand_g1_point_not_on_curve();
        let c2 = G2::rand(&mut rng);
        let c1_bytes = encode_g1(c1);
        let c2_bytes = encode_g2(c2);
        input_bytes.extend(c1_bytes);
        input_bytes.extend(c2_bytes);

        let input: String = hex::encode(input_bytes.clone());
        let vector = VectorFail {
            input,
            expected_error: String::from("point is not on curve"),
            name: format!("point_not_on_curve_g1"),
        };
        vectors.push(vector);
    }

    // not on curve g2
    {
        let mut input_bytes: Vec<u8> = vec![];

        let a1 = G1::rand(&mut rng);
        let a2 = G2::rand(&mut rng);
        let a1_bytes = encode_g1(a1);
        let a2_bytes = encode_g2(a2);
        input_bytes.extend(a1_bytes);
        input_bytes.extend(a2_bytes);

        let b1 = G1::rand(&mut rng);
        let b2 = G2::rand(&mut rng);
        let b1_bytes = encode_g1(b1);
        let b2_bytes = encode_g2(b2);
        input_bytes.extend(b1_bytes);
        input_bytes.extend(b2_bytes);

        let c1 = G1::rand(&mut rng);
        let c2: G2 = rand_g2_point_not_on_curve();
        let c1_bytes = encode_g1(c1);
        let c2_bytes = encode_g2(c2);
        input_bytes.extend(c1_bytes);
        input_bytes.extend(c2_bytes);

        let input: String = hex::encode(input_bytes.clone());
        let vector = VectorFail {
            input,
            expected_error: String::from("point is not on curve"),
            name: format!("point_not_on_curve_g2"),
        };
        vectors.push(vector);
    }

    // incorrect subgroup g1
    {
        let mut input_bytes: Vec<u8> = vec![];

        let a1 = G1::rand(&mut rng);
        let a2 = G2::rand(&mut rng);
        let a1_bytes = encode_g1(a1);
        let a2_bytes = encode_g2(a2);
        input_bytes.extend(a1_bytes);
        input_bytes.extend(a2_bytes);

        let b1 = G1::rand(&mut rng);
        let b2 = G2::rand(&mut rng);
        let b1_bytes = encode_g1(b1);
        let b2_bytes = encode_g2(b2);
        input_bytes.extend(b1_bytes);
        input_bytes.extend(b2_bytes);

        let c1: G1 = rand_g1_point_not_on_correct_subgroup();
        let c2 = G2::rand(&mut rng);
        let c1_bytes = encode_g1(c1);
        let c2_bytes = encode_g2(c2);
        input_bytes.extend(c1_bytes);
        input_bytes.extend(c2_bytes);

        let input: String = hex::encode(input_bytes.clone());
        let vector = VectorFail {
            input,
            expected_error: String::from("g1 point is not on correct subgroup"),
            name: format!("incorrect_subgroup_g1"),
        };
        vectors.push(vector);
    }

    // incorrect subgroup g2
    {
        let mut input_bytes: Vec<u8> = vec![];

        let a1 = G1::rand(&mut rng);
        let a2 = G2::rand(&mut rng);
        let a1_bytes = encode_g1(a1);
        let a2_bytes = encode_g2(a2);
        input_bytes.extend(a1_bytes);
        input_bytes.extend(a2_bytes);

        let b1 = G1::rand(&mut rng);
        let b2 = G2::rand(&mut rng);
        let b1_bytes = encode_g1(b1);
        let b2_bytes = encode_g2(b2);
        input_bytes.extend(b1_bytes);
        input_bytes.extend(b2_bytes);

        let c1 = G1::rand(&mut rng);
        let c2: G2 = rand_g2_point_not_on_correct_subgroup();
        let c1_bytes = encode_g1(c1);
        let c2_bytes = encode_g2(c2);
        input_bytes.extend(c1_bytes);
        input_bytes.extend(c2_bytes);

        let input: String = hex::encode(input_bytes.clone());
        let vector = VectorFail {
            input,
            expected_error: String::from("g2 point is not on correct subgroup"),
            name: format!("incorrect_subgroup_g2"),
        };
        vectors.push(vector);
    }

    write_vectors_fail(vectors, "Pairing");
}

#[test]
fn generate_test_vectors() {
    gen_g1_add_vectors();
    gen_g1_mul_vectors();
    gen_g1_multiexp_vectors();
    gen_g2_add_vectors();
    gen_g2_mul_vectors();
    gen_g2_multiexp_vectors();
    gen_pairing_vectors();
}

#[test]
fn generate_fail_test_vectors() {
    gen_fail_g1_add_vectors();
    gen_fail_g1_mul_vectors();
    gen_fail_g1_multiexp_vectors();
    gen_fail_g2_add_vectors();
    gen_fail_g2_mul_vectors();
    gen_fail_g2_multiexp_vectors();
    gen_fail_pairing();
}
