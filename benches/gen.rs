use criterion::{criterion_group, criterion_main, Criterion};

use snowcloud;

type BenchMultiThread = snowcloud::MultiThread<43, 8, 12>;
type BenchSingleThread = snowcloud::SingleThread<43, 8, 12>;

const START_TIME: u64 = 946684800000;
const MACHINE_ID: i64 = 1;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("gen MultiThread MAX_SEQUENCE", |b| b.iter(|| {
        let cloud = BenchMultiThread::new(MACHINE_ID, START_TIME).unwrap();

        for _ in 0..BenchMultiThread::MAX_SEQUENCE {
            cloud.next_id().expect("error generating id");
        }
    }));

    c.bench_function("gen SingleThread MAX_SEQUENCE", |b| b.iter(|| {
        let mut cloud = BenchSingleThread::new(MACHINE_ID, START_TIME).unwrap();

        for _ in 0..BenchSingleThread::MAX_SEQUENCE {
            cloud.next_id().expect("error generating id");
        }
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
