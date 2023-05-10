use criterion::*;

use substrate_bn::{pairing_batch, Group, G1, G2};

fn bench_bn_pairing(c: &mut Criterion) {
	const SAMPLES: usize = 1000;

	let mut rng = rand::thread_rng();
	let g1s = (0..SAMPLES).map(|_| G1::random(&mut rng)).collect::<Vec<_>>();
	let g2s = (0..SAMPLES).map(|_| G2::random(&mut rng)).collect::<Vec<_>>();

	let mut group = c.benchmark_group("Bn");
	group.sample_size(1000);
	group.bench_function(&format!("Pairing for {SAMPLES} samples"), |b| {
		let mut i = 0;
		b.iter(|| {
			i = (i + 1) % SAMPLES;
			pairing_batch(&[(g1s[i], g2s[i])])
		})
	});
}

criterion_group! {
	name = bn_pairing;
	config = Criterion::default();
	targets = bench_bn_pairing
}

criterion_main!(bn_pairing);
