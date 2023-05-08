use criterion::*;

use ark_bw6_761::{fr::Fr, g1::G1Projective as G1, g2::G2Projective as G2, BW6_761};
use ark_ec::{pairing::Pairing, scalar_mul::variable_base::VariableBaseMSM, CurveGroup, Group};
use ark_ff::PrimeField;
use ark_std::UniformRand;

mod g1 {
	use super::*;

	fn arithmetic(c: &mut Criterion) {
		let name = format!("{}::{}", stringify!(BW6_761), stringify!(G1));

		const SAMPLES: usize = 1000;
		let mut rng = ark_std::test_rng();
		let mut arithmetic = c.benchmark_group(format!("Arithmetic for {name}"));
		let group_elements_left = (0..SAMPLES).map(|_| <G1>::rand(&mut rng)).collect::<Vec<_>>();
		let group_elements_right = (0..SAMPLES).map(|_| <G1>::rand(&mut rng)).collect::<Vec<_>>();
		let worst_case_scalar = [u8::MAX; 64];
		let scalars = (0..SAMPLES)
			.map(|_| Fr::from_be_bytes_mod_order(&worst_case_scalar))
			.collect::<Vec<_>>();
		arithmetic.bench_function("Addition", |b| {
			let mut i = 0;
			b.iter(|| {
				i = (i + 1) % SAMPLES;
				group_elements_left[i] + group_elements_right[i]
			})
		});
		arithmetic.bench_function("Scalar Multiplication(worst-case)", |b| {
			let mut i = 0;
			b.iter(|| {
				i = (i + 1) % SAMPLES;
				group_elements_left[i] * scalars[i]
			})
		});
	}

	fn msm(samples: usize, c: &mut Criterion) {
		let name = format!("{}::{}", stringify!(BW6_761), stringify!(G1));

		let mut rng = ark_std::test_rng();

		let v: Vec<_> = (0..samples).map(|_| <G1>::rand(&mut rng)).collect();
		let v = <G1>::normalize_batch(&v);
		let worst_case_scalar = [u8::MAX; 64];
		let scalars: Vec<_> = (0..samples)
			.map(|_| Fr::from_be_bytes_mod_order(&worst_case_scalar).into_bigint())
			.collect();
		c.bench_function(&format!("MSM-{samples} for {name}"), |b| {
			b.iter(|| {
				let result: G1 = VariableBaseMSM::msm_bigint(&v, &scalars);
				result
			})
		});
	}

	pub fn benches() {
		let mut criterion: Criterion<_> = (Criterion::default()).configure_from_args();
		arithmetic(&mut criterion);
		let _ = (1..129).map(|i| msm(i, &mut criterion)).collect::<Vec<_>>();
	}
}

mod g2 {
	use super::*;

	fn arithmetic(c: &mut Criterion) {
		let name = format!("{}::{}", stringify!(BW6_761), stringify!(G2));

		const SAMPLES: usize = 1000;
		let mut rng = ark_std::test_rng();
		let mut arithmetic = c.benchmark_group(format!("Arithmetic for {name}"));
		let group_elements_left = (0..SAMPLES).map(|_| <G2>::rand(&mut rng)).collect::<Vec<_>>();
		let group_elements_right = (0..SAMPLES).map(|_| <G2>::rand(&mut rng)).collect::<Vec<_>>();
		let worst_case_scalar = [u8::MAX; 64];
		let scalars = (0..SAMPLES)
			.map(|_| Fr::from_be_bytes_mod_order(&worst_case_scalar))
			.collect::<Vec<_>>();
		arithmetic.bench_function("Addition", |b| {
			let mut i = 0;
			b.iter(|| {
				i = (i + 1) % SAMPLES;
				group_elements_left[i] + group_elements_right[i]
			})
		});
		arithmetic.bench_function("Scalar Multiplication(worst-case)", |b| {
			let mut i = 0;
			b.iter(|| {
				i = (i + 1) % SAMPLES;
				group_elements_left[i] * scalars[i]
			})
		});
	}

	fn msm(samples: usize, c: &mut Criterion) {
		let name = format!("{}::{}", stringify!(BW6_761), stringify!(G2));

		let mut rng = ark_std::test_rng();

		let v: Vec<_> = (0..samples).map(|_| <G2>::rand(&mut rng)).collect();
		let v = <G2>::normalize_batch(&v);
		let worst_case_scalar = [u8::MAX; 64];
		let scalars: Vec<_> = (0..samples)
			.map(|_| Fr::from_be_bytes_mod_order(&worst_case_scalar).into_bigint())
			.collect();
		c.bench_function(&format!("MSM-{samples} for {name}"), |b| {
			b.iter(|| {
				let result: G2 = VariableBaseMSM::msm_bigint(&v, &scalars);
				result
			})
		});
	}

	pub fn benches() {
		let mut criterion: Criterion<_> = (Criterion::default()).configure_from_args();
		arithmetic(&mut criterion);
		let _ = (1..129).map(|i| msm(i, &mut criterion)).collect::<Vec<_>>();
	}
}

mod pairing {
	use super::*;

	fn pairing(c: &mut Criterion) {
		type G1Prepared = <BW6_761 as Pairing>::G1Prepared;
		type G2Prepared = <BW6_761 as Pairing>::G2Prepared;

		const SAMPLES: usize = 1000;

		let mut rng = ark_std::test_rng();

		let g1s = (0..SAMPLES).map(|_| G1::rand(&mut rng)).collect::<Vec<_>>();
		let g2s = (0..SAMPLES).map(|_| G2::rand(&mut rng)).collect::<Vec<_>>();
		let g1s = G1::normalize_batch(&g1s);
		let g2s = G2::normalize_batch(&g2s);
		let (prepared_1, prepared_2): (Vec<G1Prepared>, Vec<G2Prepared>) = g1s
			.iter()
			.zip(&g2s)
			.map(|(g1, g2)| {
				let g1: G1Prepared = g1.into();
				let g2: G2Prepared = g2.into();
				(g1, g2)
			})
			.unzip();
		let miller_loop_outputs = prepared_1
			.iter()
			.cloned()
			.zip(prepared_2.iter().cloned())
			.map(|(g1, g2)| BW6_761::multi_miller_loop([g1], [g2]))
			.collect::<Vec<_>>();
		let mut i = 0;
		let mut pairing = c.benchmark_group(format!("Pairing for {}", stringify!(BW6_761)));
		pairing.bench_function(stringify!(Miller Loop), |b| {
			b.iter(|| {
				i = (i + 1) % SAMPLES;
				BW6_761::multi_miller_loop([prepared_1[i].clone()], [prepared_2[i].clone()])
			})
		});
		pairing.bench_function(stringify!(Final Exponentiation), |b| {
			b.iter(|| {
				i = (i + 1) % SAMPLES;
				BW6_761::final_exponentiation(miller_loop_outputs[i])
			})
		});
		pairing.bench_function(stringify!(Full Pairing), |b| {
			b.iter(|| {
				i = (i + 1) % SAMPLES;
				BW6_761::multi_pairing([g1s[i]], [g2s[i]])
			})
		});
	}

	criterion_group!(benches, pairing);
}

fn main() {
	g1::benches();
	g2::benches();
	pairing::benches();
	Criterion::default().configure_from_args().final_summary();
}
