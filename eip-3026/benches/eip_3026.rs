use criterion::Criterion;

use ark_bw6_761::{fr::Fr, g1::G1Projective as G1, g2::G2Projective as G2, BW6_761};
use ark_ec::{scalar_mul::variable_base::VariableBaseMSM, CurveGroup, Group};
use ark_ff::PrimeField;
use ark_std::UniformRand;

mod g1 {
	use super::*;

	fn rand(c: &mut Criterion) {
		let name = format!("{}::{}", stringify!(BW6_761), stringify!(G1));
		let mut rng = ark_std::test_rng();
		c.bench_function(&format!("Sample {name} elements"), |b| b.iter(|| G1::rand(&mut rng)));
	}

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
		arithmetic.bench_function("Double", |b| {
			let mut i = 0;
			b.iter(|| {
				i = (i + 1) % SAMPLES;
				group_elements_left[i].double()
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
		rand(&mut criterion);
		arithmetic(&mut criterion);
		let _ = (1..129).map(|i| msm(i, &mut criterion)).collect::<Vec<_>>();
	}
}

fn main() {
	g1::benches();
	// g2::benches();
	// pairing::benches();
	Criterion::default().configure_from_args().final_summary();
}
