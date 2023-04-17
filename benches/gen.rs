use criterion::{criterion_group, criterion_main, Criterion, BatchSize};

use snowcloud::{SingleThread, MultiThread};
use snowcloud::i64::{SingleIdFlake, DualIdFlake};

type SID13 = SingleIdFlake<43, 7, 13>;
type SID12 = SingleIdFlake<43, 8, 12>;

type DID13 = DualIdFlake<43, 3, 4, 13>;
type DID12 = DualIdFlake<43, 4, 4, 12>;

const START_TIME: u64 = 946684800000;

pub fn single_thread_generator(c: &mut Criterion) {
    let mut gen_group = c.benchmark_group("SingleThread");

    gen_group.bench_function("SingleIdFlake 1", |b| b.iter_batched_ref(
        || SingleThread::<SID12>::new(START_TIME, 1).unwrap(),
        |cloud| {
            cloud.next_id().expect("error generating id");
        },
        BatchSize::SmallInput
    ));

    gen_group.bench_function("SingleIdFlake 4,095", |b| b.iter_batched_ref(
        || SingleThread::<SID12>::new(START_TIME, 1).unwrap(),
        |cloud| {
            for _ in 0..SID12::MAX_SEQUENCE {
                cloud.next_id().expect("error generating id");
            }
        },
        BatchSize::SmallInput
    ));

    gen_group.bench_function("SingleIdFlake 8,191", |b| b.iter_batched_ref(
        || SingleThread::<SID13>::new(START_TIME, 1).unwrap(),
        |cloud| {
            for _ in 0..SID13::MAX_SEQUENCE {
                cloud.next_id().expect("error generating id");
            }
        },
        BatchSize::SmallInput
    ));

    gen_group.bench_function("DualIdFlake 1", |b| b.iter_batched_ref(
        || SingleThread::<DID12>::new(START_TIME, (1, 1)).unwrap(),
        |cloud| {
            cloud.next_id().expect("error generating id");
        },
        BatchSize::SmallInput
    ));

    gen_group.bench_function("DualIdFlake 4,095", |b| b.iter_batched_ref(
        || SingleThread::<DID12>::new(START_TIME, (1, 1)).unwrap(),
        |cloud| {
            for _ in 0..DID12::MAX_SEQUENCE {
                cloud.next_id().expect("error generating id");
            }
        },
        BatchSize::SmallInput
    ));

    gen_group.bench_function("DualIdFlake 8,191", |b| b.iter_batched_ref(
        || SingleThread::<DID13>::new(START_TIME, (1,1)).unwrap(),
        |cloud| {
            for _ in 0..DID13::MAX_SEQUENCE {
                cloud.next_id().expect("error generating id");
            }
        },
        BatchSize::SmallInput
    ));

    gen_group.finish();
}

pub fn multi_thread_generator(c: &mut Criterion) {
    let mut gen_group = c.benchmark_group("MultiThread");

    gen_group.bench_function("SingleIdFlake 1", |b| b.iter_batched_ref(
        || MultiThread::<SID12>::new(START_TIME, 1).unwrap(),
        |cloud| {
            cloud.next_id().expect("error generating id");
        },
        BatchSize::SmallInput
    ));

    gen_group.bench_function("SingleIdFlake 4,095", |b| b.iter_batched_ref(
        || MultiThread::<SID12>::new(START_TIME, 1).unwrap(),
        |cloud| {
            for _ in 0..SID12::MAX_SEQUENCE {
                cloud.next_id().expect("error generating id");
            }
        },
        BatchSize::SmallInput
    ));

    gen_group.bench_function("SingleIdFlake 8,191", |b| b.iter_batched_ref(
        || MultiThread::<SID13>::new(START_TIME, 1).unwrap(),
        |cloud| {
            for _ in 0..SID13::MAX_SEQUENCE {
                cloud.next_id().expect("error generating id");
            }
        },
        BatchSize::SmallInput
    ));

    gen_group.bench_function("DualIdFlake 1", |b| b.iter_batched_ref(
        || MultiThread::<DID12>::new(START_TIME, (1, 1)).unwrap(),
        |cloud| {
            cloud.next_id().expect("error generating id");
        },
        BatchSize::SmallInput
    ));

    gen_group.bench_function("DualIdFlake 4,095", |b| b.iter_batched_ref(
        || MultiThread::<DID12>::new(START_TIME, (1, 1)).unwrap(),
        |cloud| {
            for _ in 0..DID12::MAX_SEQUENCE {
                cloud.next_id().expect("error generating id");
            }
        },
        BatchSize::SmallInput
    ));

    gen_group.bench_function("DualIdFlake 8,191", |b| b.iter_batched_ref(
        || MultiThread::<DID13>::new(START_TIME, (1, 1)).unwrap(),
        |cloud| {
            for _ in 0..DID13::MAX_SEQUENCE {
                cloud.next_id().expect("error generating id");
            }
        },
        BatchSize::SmallInput
    ));

    gen_group.finish();
}

criterion_group!(
    benches,
    single_thread_generator,
    multi_thread_generator
);
criterion_main!(benches);
