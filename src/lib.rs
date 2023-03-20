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
const NANO_IN_MILLI_U128: u128 = 1_000_000;

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
        #[derive(Eq, Hash, PartialEq, Clone, Debug)]
        pub struct Snowflake {
            ts: i64,
            mid: i64,
            seq: i64,
        }

        impl Snowflake {
            /// returns timestamp reference
            pub fn timestamp(&self) -> &i64 {
                &self.ts
            }

            /// returns machine id reference
            pub fn machine_id(&self) -> &i64 {
                &self.mid
            }

            /// returns sequence reference
            pub fn sequence(&self) -> &i64 {
                &self.seq
            }

            /// generates a Snowflake from the provided parts
            ///
            /// checks will be performed on each part to ensure that they are
            /// valid for the given Snowflake
            pub fn from_parts(ts: i64, mid: i64, seq: i64) -> error::Result<Snowflake> {
                if ts < 0 || ts > MAX_TIMESTAMP {
                    return Err(error::Error::EpochInvalid);
                }

                if mid < 0 || mid > MAX_MACHINE_ID {
                    return Err(error::Error::MachineIdInvalid);
                }

                if seq < 0 || seq > MAX_SEQUENCE {
                    return Err(error::Error::SequenceInvalid);
                }

                Ok(Snowflake { ts, mid, seq })
            }

            /// splits the current Snowflake into its individual parts
            pub fn into_parts(self) -> (i64, i64, i64) {
                (self.ts, self.mid, self.seq)
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

        /// stores sequence and prev_time for a Snowcloud
        ///
        /// is guarded by an Arc Mutex since this data is shared between
        /// threads
        struct Counts {
            sequence: i64,
            prev_time: i64,
        }

        /// generates Snowflakes from a given EPOCH and machine id
        pub struct Snowcloud {
            epoch: i64,
            machine_id: i64,
            counts: Arc<Mutex<Counts>>,
        }

        impl Clone for Snowcloud {
            fn clone(&self) -> Snowcloud {
                Snowcloud {
                    epoch: self.epoch.clone(),
                    machine_id: self.machine_id.clone(),
                    counts: Arc::clone(&self.counts),
                }
            }
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

                let (now, _) = ts_millis()?;

                if epoch > now {
                    return Err(error::Error::EpochInvalid);
                }

                Ok(Snowcloud {
                    epoch,
                    machine_id,
                    counts: Arc::new(Mutex::new(Counts {
                        sequence: 1,
                        prev_time: now,
                    }))
                })
            }

            /// returns Snowcloud epoch
            pub fn get_epoch(&self) -> &i64 {
                &self.epoch
            }

            /// returns Snowcloud machine_id
            pub fn get_machine_id(&self) -> &i64 {
                &self.machine_id
            }

            /// retrieves the next available id
            ///
            /// if the current timestamp reaches max, the max sequence value is
            /// reached, or if it fails to get the current timestamp this will
            /// return an error.
            pub fn next_id(&self) -> error::Result<Snowflake> {
                let seq_value: i64;
                let now: i64;

                {
                    // lock down counts for the current thread
                    let Ok(mut counts_lock) = self.counts.lock() else {
                        return Err(error::Error::MutexError);
                    };

                    // since we do not know when the lock will be freed we
                    // have to get the time once the lock is freed to have
                    // an accurate timestamp
                    let (secs, nanos) = ts_millis()?;
                    now = secs - self.epoch;

                    if now > MAX_TIMESTAMP {
                        return Err(error::Error::TimestampMaxReached);
                    }

                    // if we are still on the previously recorded millisecond
                    // then we increment the sequence
                    if now == counts_lock.prev_time {
                        seq_value = counts_lock.sequence;

                        // before we increment, check to make sure that we
                        // have not reached the maximum sequence value. if
                        // we have then given an estimate to the next
                        // millisecond so that then user can decided on
                        // how to wait for the next available value
                        if seq_value > MAX_SEQUENCE {
                            return Err(error::Error::SequenceMaxReached(NANO_IN_MILLI - nanos));
                        }

                        // increment to the next sequence number
                        counts_lock.sequence += 1;
                    } else {
                        // we are not on the previousely recorded millisecond
                        // so the sequence value will be set to one
                        seq_value = 1;

                        // set the previous time to now and prep for the next
                        // available sequence number
                        counts_lock.prev_time = now.clone();
                        counts_lock.sequence = 2;
                    }

                // counts_lock should be dropped and the mutext should now be
                // unlocked for the next 
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

gen_code!(43, 8, 12);

/// returns current UNIX EPOCH in milliseconds
fn ts_millis() -> error::Result<(i64,u32)> {
    let now = SystemTime::now();

    let Ok(duration) = now.duration_since(SystemTime::UNIX_EPOCH) else {
        return Err(error::Error::TimestampError);
    };

    let as_nanos = duration.as_nanos();
    let millis = as_nanos / NANO_IN_MILLI_U128;
    let nanos = (as_nanos % NANO_IN_MILLI_U128) as u32;

    let Ok(cast) = i64::try_from(millis) else {
        return Err(error::Error::TimestampError);
    };

    Ok((cast, nanos))
}

/// busy spins for one millisecond from the given start instant
fn spin_one_milli(to_next_milli: u32) -> () {
    let start = Instant::now();

    loop {
        let duration = start.elapsed().subsec_nanos();

        let Some(_diff) = to_next_milli.checked_sub(duration) else {
            break;
        };

        std::hint::spin_loop();
    }
}

#[cfg(test)]
mod test;
