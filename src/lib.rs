use std::{
    sync::{
        atomic::{
            Ordering,
            AtomicI64,
        },
        Arc,
    },
    time::{
        Instant,
        SystemTime,
    },
};

pub mod error;

/// generates Snowcloud and Snowflake for the given bit sizes
///
/// on the chance that the bit sizes are not what you want you can generate
/// your own with different sizes without having to modify the library
#[macro_export]
macro_rules! gen_code {
    ($ts_bits:expr, $mid_bits:expr, $seq_bits:expr) => {
        pub const TIMESTAMP_BITS: i64 = $ts_bits;
        pub const MACHINE_ID_BITS: i64 = $mid_bits;
        pub const SEQUENCE_BITS: i64 = $seq_bits;

        pub const MAX_TIMESTAMP: i64 = (1 << TIMESTAMP_BITS) - 1;
        pub const MAX_MACHINE_ID: i64 = (1 << MACHINE_ID_BITS) - 1;
        pub const MAX_SEQUENCE: i64 = (1 << SEQUENCE_BITS) - 1;

        pub const TIMESTAMP_SHIFT: i64 = MACHINE_ID_BITS + SEQUENCE_BITS;
        pub const MACHINE_ID_SHIFT: i64 = SEQUENCE_BITS;

        pub const TIMESTAMP_MASK: i64 = MAX_TIMESTAMP << TIMESTAMP_SHIFT;
        pub const MACHINE_ID_MASK: i64 = MAX_MACHINE_ID << MACHINE_ID_SHIFT;
        pub const SEQUENCE_MASK: i64 = MAX_SEQUENCE;

        /// id generated from a Snowcloud
        #[derive(Debug)]
        #[cfg_attr(test, derive(Clone))]
        pub struct Snowflake {
            ts: i64,
            mid: i64,
            seq: i64,
        }

        impl Snowflake {
            pub fn timestamp(&self) -> &i64 {
                &self.ts
            }

            pub fn machine_id(&self) -> &i64 {
                &self.mid
            }

            pub fn sequence(&self) -> &i64 {
                &self.seq
            }
        }

        impl From<Snowflake> for i64 {
            fn from(id: Snowflake) -> i64 {
                (id.ts << TIMESTAMP_SHIFT) | (id.mid << MACHINE_ID_SHIFT) | id.seq
            }
        }

        impl TryFrom<i64> for Snowflake {
            type Error = error::Error;

            fn try_from(id: i64) -> error::Result<Snowflake> {
                if id < 0 {
                    return Err(error::Error::InvalidId);
                }

                Ok(Snowflake {
                    ts: (id & TIMESTAMP_MASK) >> TIMESTAMP_SHIFT,
                    mid: (id & MACHINE_ID_MASK) >> MACHINE_ID_SHIFT,
                    seq: id & SEQUENCE_MASK
                })
            }
        }

        /// generates Snowflakes from a given EPOCH and machine id
        ///
        /// uses atomics for the sequence and previous timestamp to help with
        /// performance and reduce the amount of waiting that needs to be done
        #[derive(Clone)]
        pub struct Snowcloud {
            pub epoch: i64,
            pub machine_id: i64,
            sequence: Arc<AtomicI64>,
            prev_time: Arc<AtomicI64>,
        }

        impl Snowcloud {

            /// returns a new Snowcloud
            ///
            /// will return an error if the machine id is invalid, the
            /// timestamp is invalid, it failes to retrieve the current
            /// timestamp, or if the epoch is ahead of the current timestamp
            pub fn new(machine_id: i64, epoch: i64) -> error::Result<Snowcloud> {
                if machine_id > MAX_MACHINE_ID {
                    return Err(error::Error::MachineIdTooLarge);
                }

                if epoch > MAX_TIMESTAMP {
                    return Err(error::Error::EpochTooLarge);
                }

                let now = ts_millis()?;

                if epoch > now {
                    return Err(error::Error::EpochInFuture);
                }

                Ok(Snowcloud {
                    epoch,
                    machine_id,
                    sequence: Arc::new(AtomicI64::new(1)),
                    prev_time: Arc::new(AtomicI64::new(now)),
                })
            }

            /// retrieves the next available id
            ///
            /// if the current timestamp reaches max, the max sequence value is
            /// reached, or if it fails to get the current timestamp this will
            /// return an error.
            pub fn next_id(&self) -> error::Result<Snowflake> {
                let now = ts_millis()? - self.epoch;
                let mut seq_value: i64 = 1;

                if now > MAX_TIMESTAMP {
                    return Err(error::Error::TimestampMaxReached);
                }

                if now == self.prev_time.load(Ordering::Relaxed) {
                    seq_value = self.sequence.fetch_add(1, Ordering::AcqRel);
                } else {
                    self.prev_time.store(now, Ordering::SeqCst);
                    self.sequence.store(2, Ordering::SeqCst);
                }

                if seq_value > MAX_SEQUENCE {
                    return Err(error::Error::SequenceMaxReached);
                }

                Ok(Snowflake {
                    ts: now,
                    mid: self.machine_id,
                    seq: seq_value,
                })
            }

            /// spins until a valid id is provided
            ///
            /// if the return value of next_id is max sequence error then this
            /// will spin_loop until the next millisecond and try again
            pub fn spin_next_id(&self) -> error::Result<Snowflake> {
                loop {
                    let start = Instant::now();

                    match self.next_id() {
                        Ok(sf) => {
                            return Ok(sf);
                        },
                        Err(err) => {
                            match err {
                                error::Error::SequenceMaxReached => {
                                    spin_one_milli(&start);
                                },
                                _ => {
                                    return Err(err);
                                }
                            }
                        }
                    }
                }
            }
        }
    };
}

gen_code!(43, 8, 12);

/// returns current UNIX EPOCH in milliseconds
fn ts_millis() -> error::Result<i64> {
    let now = SystemTime::now();

    let Ok(duration) = now.duration_since(SystemTime::UNIX_EPOCH) else {
        return Err(error::Error::TimestampError);
    };

    let sec: u64 = duration.as_secs() * 1000;
    let millis: u64 = duration.subsec_millis().into();

    i64::try_from(sec + millis).map_err(|_| error::Error::TimestampError)
}

/// busy spins for one millisecond from the given start instant
fn spin_one_milli(start: &Instant) -> () {
    loop {
        let check = Instant::now();
        let duration = check.duration_since(*start);

        if duration.subsec_millis() > 0 {
            break;
        }

        std::hint::spin_loop();
    }
}

#[cfg(test)]
mod test {
    use std::sync::{Arc, Barrier};
    use std::collections::HashSet;
    use std::thread;

    use super::*;

    const START_TIME: i64 = 946684800000;
    const MACHINE_ID: i64 = 1;

    #[test]
    fn unique_ids_single_thread() -> () {
        let cloud = Snowcloud::new(MACHINE_ID, START_TIME).unwrap();
        let mut unique_ids: HashSet<i64> = HashSet::new();
        let mut generated: Vec<Snowflake> = Vec::new();

        for _ in 0..MAX_SEQUENCE {
            let flake = cloud.spin_next_id().expect("failed spin_next_id");
            let id: i64 = flake.clone().into();

            assert!(
                unique_ids.insert(id), 
                "encountered an id that was already generated. id: {:#?}\ngenerated: {:#?}", 
                flake,
                generated
            );

            generated.push(flake.clone());
        }
    }

    #[test]
    fn unique_ids_across_threads() -> () {
        let barrier = Arc::new(Barrier::new(3));
        let mut handles = Vec::with_capacity(3);
        let cloud = Snowcloud::new(MACHINE_ID, START_TIME).unwrap();

        for _ in 0..handles.capacity() {
            let b = Arc::clone(&barrier);
            let c = cloud.clone();

            handles.push(thread::spawn(move || {
                let mut id_list = Vec::with_capacity(MAX_SEQUENCE as usize);
                b.wait();

                for _ in 0..MAX_SEQUENCE {
                    id_list.push(c.spin_next_id().expect("failed spin_next_id"));
                }

                id_list
            }));
        }

        let mut unique_ids: HashSet<i64> = HashSet::new();
        let mut thread: u8 = 0;

        for handle in handles {
            let list = handle.join().expect("thread paniced");
            let mut id_count: usize = 0;

            for flake in list {
                let id: i64 = flake.clone().into();

                assert!(unique_ids.insert(id), "encountered an id that was already generated. thread: {} index: {} id: {:#?}", thread, id_count, flake);

                id_count += 1;
            }

            thread += 1;
        }
    }
}
