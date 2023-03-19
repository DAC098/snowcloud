use std::{
    sync::{
        Arc,
        Mutex,
    },
    time::{
        Instant,
        SystemTime,
    },
};

pub mod error;

const NANO_IN_MILLI: u32 = 1_000_000;

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
        #[derive(Eq, Hash, PartialEq, Debug)]
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

        struct Counts {
            sequence: i64,
            prev_time: i64,
            prev_nanos: u32,
        }

        /// generates Snowflakes from a given EPOCH and machine id
        ///
        /// uses an Arc Mutex to store prev_time and sequence data
        #[derive(Clone)]
        pub struct Snowcloud {
            pub epoch: i64,
            pub machine_id: i64,
            counts: Arc<Mutex<Counts>>,
        }

        impl Snowcloud {

            /// returns a new Snowcloud
            ///
            /// will return an error if the machine id is invalid, the
            /// timestamp is invalid, it failes to retrieve the current
            /// timestamp, or if the epoch is ahead of the current timestamp
            pub fn new(machine_id: i64, epoch: i64) -> error::Result<Snowcloud> {
                if machine_id < 0 || machine_id > MAX_MACHINE_ID {
                    return Err(error::Error::MachineIdInvalid);
                }

                if epoch < 0 || epoch > MAX_TIMESTAMP {
                    return Err(error::Error::EpochInvalid);
                }

                let (now, nanos) = ts_millis()?;

                if epoch > now {
                    return Err(error::Error::EpochInvalid);
                }

                Ok(Snowcloud {
                    epoch,
                    machine_id,
                    counts: Arc::new(Mutex::new(Counts {
                        sequence: 1,
                        prev_time: now,
                        prev_nanos: nanos,
                    })),
                })
            }

            /// retrieves the next available id
            ///
            /// if the current timestamp reaches max, the max sequence value is
            /// reached, or if it fails to get the current timestamp this will
            /// return an error.
            pub fn next_id(&self) -> error::Result<Snowflake> {
                let (secs, nanos) = ts_millis()?;
                let now = secs - self.epoch;
                let mut seq_value: i64 = 1;

                if now > MAX_TIMESTAMP {
                    return Err(error::Error::TimestampMaxReached);
                }

                {
                    let Ok(mut counts_lock) = self.counts.lock() else {
                        return Err(error::Error::MutexError);
                    };

                    if now == counts_lock.prev_time {
                        seq_value = counts_lock.sequence;

                        if seq_value > MAX_SEQUENCE {
                            counts_lock.prev_nanos = nanos;

                            return Err(error::Error::SequenceMaxReached(NANO_IN_MILLI - nanos));
                        }

                        counts_lock.sequence += 1;
                        counts_lock.prev_nanos = nanos;
                    } else {
                        counts_lock.sequence = 2;
                        counts_lock.prev_time = now;
                        counts_lock.prev_nanos = nanos;
                    }
                }

                /* 
                // maybe this can work but right now I dont know what to do
                // in a multithreaded instance
                if now == self.prev_time.load(Ordering::Relaxed) {
                    seq_value = self.sequence.load(Ordering::Acquire);

                    if seq_value > MAX_SEQUENCE {
                        return Err(error::Error::SequenceMaxReached);
                    }

                    self.sequence.swap(seq_value + 1, Ordering::SeqCst);
                } else {
                    self.prev_time.swap(now, Ordering::SeqCst);
                    self.sequence.swap(2, Ordering::SeqCst);
                }
                */

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
                    match self.next_id() {
                        Ok(sf) => {
                            return Ok(sf);
                        },
                        Err(err) => {
                            match err {
                                error::Error::SequenceMaxReached(to_next_milli) => {
                                    spin_one_milli(to_next_milli);
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

gen_code!(42, 8, 13);

/// returns current UNIX EPOCH in milliseconds
fn ts_millis() -> error::Result<(i64,u32)> {
    let now = SystemTime::now();

    let Ok(duration) = now.duration_since(SystemTime::UNIX_EPOCH) else {
        return Err(error::Error::TimestampError);
    };

    let Ok(cast) = i64::try_from(duration.as_millis()) else {
        return Err(error::Error::TimestampError);
    };

    Ok((cast, duration.subsec_nanos()))
}

/// busy spins for one millisecond from the given start instant
fn spin_one_milli(to_next_milli: u32) -> () {
    let start = Instant::now();

    loop {
        let duration = start.elapsed().subsec_nanos();

        let Some(diff) = to_next_milli.checked_sub(duration) else {
            break;
        };

        std::hint::spin_loop();
    }
}

#[cfg(test)]
mod test {
    use std::sync::{Arc, Barrier};
    use std::collections::{HashMap, HashSet};
    use std::thread;
    use std::io::Write as _;

    use super::*;

    const START_TIME: i64 = 1679082337000;
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
    fn unique_ids_multi_threads() -> () {
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

        let mut failed = false;
        let mut thread: usize = 0;
        let mut unique_ids: HashMap<Snowflake, Vec<(usize, usize)>> = HashMap::new();
        let mut thread_list: Vec<Vec<Snowflake>> = Vec::with_capacity(handles.len());

        for handle in handles {
            let list = handle.join().expect("thread paniced");

            thread_list.push(list);

            for index in 0..thread_list[thread].len() {
                let flake = &thread_list[thread][index];

                if let Some(dups) = unique_ids.get_mut(flake) {
                    failed = true;
                    dups.push((thread, index));
                } else {
                    let mut dups = Vec::with_capacity(1);
                    dups.push((thread, index));

                    unique_ids.insert(flake.clone(), dups);
                }
            }

            thread += 1;
        }

        if !failed {
            return;
        }

        let mut debug_output = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("unique_id_multi_thread.debug.txt")
            .expect("failed to create debug_file");

        for (flake, dups) in unique_ids {
            if dups.len() > 1 {
                debug_output.write_fmt(format_args!("flake {} {}\n", flake.ts, flake.seq)).unwrap();

                for (thread, index) in dups {
                    debug_output.write_fmt(format_args!("thread {}\n", thread)).unwrap();

                    let (mut low, of) = index.overflowing_sub(3);
                    let mut next = index + 1;
                    let mut high = next + 3;

                    if of {
                        low = 0;
                    }

                    if next > thread_list[thread].len() {
                        next = thread_list[thread].len();
                        high = thread_list[thread].len();
                    } else if high > thread_list[thread].len() {
                        high = thread_list[thread].len();
                    }

                    let index_decimals = (high.checked_ilog10().unwrap_or(0) + 1) as usize;

                    for prev_index in low..index {
                        debug_output.write_fmt(format_args!(
                            "{:width$} {} {}\n", 
                            prev_index,
                            thread_list[thread][prev_index].ts,
                            thread_list[thread][prev_index].seq,
                            width = index_decimals,
                        )).unwrap();
                    }

                    debug_output.write_fmt(format_args!(
                        "{:width$} {} {} dupliate\n",
                        index,
                        thread_list[thread][index].ts,
                        thread_list[thread][index].seq,
                        width = index_decimals,
                    )).unwrap();

                    if index != next {
                        for next_index in next..high {
                            debug_output.write_fmt(format_args!(
                                "{:width$} {} {}\n",
                                next_index,
                                thread_list[thread][next_index].ts,
                                thread_list[thread][next_index].seq,
                                width = index_decimals,
                            )).unwrap();
                        }
                    }
                }

                debug_output.write_fmt(format_args!("\n")).unwrap();
            }
        }

        panic!("encountered duplidate ids. check unique_id_multi_thread.deubg.txt for output");
    }
}
