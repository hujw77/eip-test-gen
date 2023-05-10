use criterion::*;
use parity_crypto::publickey::{recover as ec_recover, sign, Generator, Message, Random};

fn bench_ecrevocer(c: &mut Criterion) {
	const SAMPLES: usize = 100000;

	let keypairs = (0..SAMPLES).map(|_| Random.generate());
	let message = Message::zero();
	let signatures = keypairs
		.map(|keypair| sign(keypair.secret(), &Message::zero()).unwrap())
		.collect::<Vec<_>>();

	let mut group = c.benchmark_group("ECRECOVER");
	group.sample_size(1000);
	group.bench_function(&format!("ECRECOVER for {SAMPLES} samples"), |b| {
		let mut i = 0;
		b.iter(|| {
			i = (i + 1) % SAMPLES;
			ec_recover(&signatures[i], &message).unwrap()
		})
	});
}

criterion_group! {
	name = ecrecover;
	config = Criterion::default();
	targets = bench_ecrevocer
}

criterion_main!(ecrecover);
