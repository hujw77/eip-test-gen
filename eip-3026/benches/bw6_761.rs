use criterion::*;

use ark_bw6_761::{fr::Fr, g1::G1Projective as G1, g2::G2Projective as G2, BW6_761};
use ark_ec::{pairing::Pairing, scalar_mul::variable_base::VariableBaseMSM, CurveGroup};
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
		let id = BenchmarkId::new("Arithmetic", "Addition");
		arithmetic.sample_size(1000);
		arithmetic.bench_function(id, |b| {
			let mut i = 0;
			b.iter(|| {
				i = (i + 1) % SAMPLES;
				group_elements_left[i] + group_elements_right[i]
			})
		});
		let id = BenchmarkId::new("Arithmetic", "Scalar Multiplication(worst-case)");
		arithmetic.bench_function(id, |b| {
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

		let mut group = c.benchmark_group(format!("MSM for {name}"));
		(0..samples).for_each(|i| {
			let sample = i + 1;
			let v: Vec<_> = (0..sample).map(|_| <G1>::rand(&mut rng)).collect();
			let v = <G1>::normalize_batch(&v);
			let worst_case_scalar = [u8::MAX; 64];
			let scalars: Vec<_> = (0..sample)
				.map(|_| Fr::from_be_bytes_mod_order(&worst_case_scalar).into_bigint())
				.collect();
			let id = BenchmarkId::new("MSM", sample);
			group.sample_size(1000);
			group.bench_function(id, |b| {
				b.iter(|| {
					let result: G1 = VariableBaseMSM::msm_bigint(&v, &scalars);
					result
				})
			});
		});
	}

	pub fn benches() {
		let mut criterion: Criterion<_> = (Criterion::default()).configure_from_args();
		arithmetic(&mut criterion);
		msm(128, &mut criterion);
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
		let id = BenchmarkId::new("Arithmetic", "Addition");
		arithmetic.sample_size(1000);
		arithmetic.bench_function(id, |b| {
			let mut i = 0;
			b.iter(|| {
				i = (i + 1) % SAMPLES;
				group_elements_left[i] + group_elements_right[i]
			})
		});
		let id = BenchmarkId::new("Arithmetic", "Scalar Multiplication(worst-case)");
		arithmetic.bench_function(id, |b| {
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

		let mut group = c.benchmark_group(format!("MSM for {name}"));
		(0..samples).for_each(|i| {
			let sample = i + 1;
			let v: Vec<_> = (0..sample).map(|_| <G2>::rand(&mut rng)).collect();
			let v = <G2>::normalize_batch(&v);
			let worst_case_scalar = [u8::MAX; 64];
			let scalars: Vec<_> = (0..sample)
				.map(|_| Fr::from_be_bytes_mod_order(&worst_case_scalar).into_bigint())
				.collect();
			let id = BenchmarkId::new("MSM", sample);
			group.sample_size(1000);
			group.bench_function(id, |b| {
				b.iter(|| {
					let result: G2 = VariableBaseMSM::msm_bigint(&v, &scalars);
					result
				})
			});
		});
	}

	pub fn benches() {
		let mut criterion: Criterion<_> = (Criterion::default()).configure_from_args();
		arithmetic(&mut criterion);
		msm(128, &mut criterion);
	}
}

mod pairing {
	use super::*;

	fn pairing(c: &mut Criterion) {
		let pairs: [usize; 5] = [2, 4, 8, 12, 16];
		let mut rng = ark_std::test_rng();

		let mut group = c.benchmark_group(format!("Pairing for {}", stringify!(BW6_671)));
		for num_pair in pairs.iter() {
			let sample = *num_pair;
			let g1s = (0..sample).map(|_| G1::rand(&mut rng)).collect::<Vec<_>>();
			let g2s = (0..sample).map(|_| G2::rand(&mut rng)).collect::<Vec<_>>();
			let g1s = G1::normalize_batch(&g1s);
			let g2s = G2::normalize_batch(&g2s);
			let id = BenchmarkId::new("Pairing", sample);
			group.sample_size(1000);
			group.bench_with_input(id, &(g1s, g2s), |b, (g1s, g2s)| {
				b.iter(|| BW6_761::multi_pairing(black_box(g1s), black_box(g2s)))
			});
		}
	}

	criterion_group!(benches, pairing);
}

fn main() {
	g1::benches();
	g2::benches();
	pairing::benches();
	Criterion::default().configure_from_args().final_summary();
}
