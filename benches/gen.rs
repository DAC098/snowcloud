use criterion::{criterion_group, criterion_main, Criterion};

use snowcloud;

const START_TIME: i64 = 946684800000;
const MACHINE_ID: i64 = 1;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("gen MAX_SEQUENCE", |b| b.iter(|| {
        let cloud = snowcloud::Snowcloud::new(MACHINE_ID, START_TIME).unwrap();

        for _ in 0..snowcloud::MAX_SEQUENCE {
            cloud.next_id().expect("error generating id");
        }
    }));

    c.bench_function("gen 3xMAX_SEQUENCE", |b| b.iter(|| {
        let cloud = snowcloud::Snowcloud::new(MACHINE_ID, START_TIME).unwrap();

        for _ in 0..(snowcloud::MAX_SEQUENCE * 3) {
            cloud.spin_next_id().expect("error generating id");
        }
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
