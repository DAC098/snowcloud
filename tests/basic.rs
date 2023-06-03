const START_TIME: u64 = 1679587200000;

#[test]
fn sanity_check() {
    type MyFlake = snowcloud::flake::i64::SingleIdFlake<43, 16, 4>;
    type MyCloud = snowcloud::cloud::Generator<MyFlake>;

    let mut gen = MyCloud::new(START_TIME, 1).unwrap();

    println!("{}", gen.ids());

    for _ in 0..(MyFlake::MAX_SEQUENCE * 3) {
        let Some(result) = snowcloud::cloud::wait::blocking_next_id_mut(&mut gen, 2) else {
            panic!("ran out of attempts to get a new snowflake");
        };

        let flake = result.expect("failed to generate snowflake");

        println!("{}", flake.id());
    }
}

#[test]
fn threaded_sanity_check() {
    type MyFlake = snowcloud::flake::u64::DualIdFlake<44, 8, 8, 4>;
    type MyCloud = snowcloud::cloud::sync::MutexGenerator<MyFlake>;

    let gen = MyCloud::new(START_TIME, (1, 1))
        .expect("failed to create mutex generator");

    let mut threads = Vec::with_capacity(4);

    for _ in 0..threads.capacity() {
        let local_gen = gen.clone();

        threads.push(std::thread::spawn(move || {
            for _ in 0..(MyFlake::MAX_SEQUENCE * 3) {
                let Some(result) = snowcloud::cloud::wait::blocking_next_id(&local_gen, 3) else {
                    panic!("ran out of attempts to get a new snowflake");
                };

                result.expect("failed to generate snowflake");
            }
        }));
    }

    for joiner in threads {
        joiner.join().expect("thread paniced");
    }
}
