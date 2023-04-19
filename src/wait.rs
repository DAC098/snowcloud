//! methods for waiting on the next available id from a snowcloud
//!
//! currently contains only a blocking method but other could be added in
//! the future

use std::time::{Instant, Duration};

use crate::traits::{NextAvailId, IdGenerator, IdGeneratorMut};

/// blocks the current thread for the given duration by sleeping, yielding, or
/// spinning
fn block_duration(dur: &Duration) {
    let start = Instant::now();

    loop {
        let Some(diff) = dur.checked_sub(start.elapsed()) else {
            break;
        };

        let nanos = diff.subsec_nanos();

        if nanos > 500_000 {
            std::thread::sleep(diff);
        } else if nanos > 1_000 {
            std::thread::yield_now();
        } else {
            std::hint::spin_loop();
        }
    }
}

/// blocks the current thread for next available id with a given number of
/// attempts
///
/// if total attempts reaches 0 then the result will be none otherwise will be
/// some with whatever happened when generating the id
///
/// ```rust
/// type MyFlake = snowcloud::i64::SingleIdFlake<43, 8, 12>;
/// type MyCloud = snowcloud::sync::MutexGenerator<MyFlake>;
///
/// const START_TIME: u64 = 1679587200000;
///
/// let cloud = MyCloud::new(START_TIME, 1)
///     .expect("failed to create MyCloud");
///
/// // create more snowflakes than what is possible in a millisecond
/// for _ in 0..(MyFlake::MAX_SEQUENCE as usize * 2) {
///     let Some(result) = snowcloud::wait::blocking_next_id(&cloud, 2) else {
///         println!("ran out of attempts to get a new snowflake");
///         continue;
///     };
///
///     let flake = result.expect("failed to create snowflake");
///
///     println!("{}", flake.id());
/// }
/// ```
pub fn blocking_next_id<C>(cloud: &C, mut attempts: u8) -> Option<std::result::Result<C::Id, C::Error>> 
where
    C: IdGenerator,
    C::Error: NextAvailId,
    C::Output: Into<std::result::Result<C::Id, C::Error>>,
{
    while attempts != 0 {
        match cloud.next_id().into() {
            Ok(sf) => {
                return Some(Ok(sf))
            },
            Err(err) => {
                let Some(dur) = err.next_avail_id() else {
                    return Some(Err(err));
                };

                block_duration(dur);
            }
        }

        attempts -= 1;
    }

    None
}

/// mutable version of [`blocking_next_id`]
///
/// if total attempts reaches 0 then the result will be None otherwise will be
/// some with whatever happened when generating the id
///
/// ```rust
/// use snowcloud::Error;
/// type MyFlake = snowcloud::i64::SingleIdFlake<43, 8, 12>;
/// type MyCloud = snowcloud::Generator<MyFlake>;
///
/// const START_TIME: u64 = 1679587200000;
///
/// let mut cloud = MyCloud::new(START_TIME, 1)
///     .expect("failed to create MyCloud");
///
/// // create more snowflakes than what is possible in a millisecond
/// for _ in 0..(MyFlake::MAX_SEQUENCE as usize * 2) {
///     let Some(result) = snowcloud::wait::blocking_next_id_mut(&mut cloud, 2) else {
///         println!("ran out of attempts to get a new snowflake");
///         continue;
///     };
///
///     let flake = result.expect("failed to create snowflake");
///
///     println!("{}", flake.id());
/// }
/// ```
pub fn blocking_next_id_mut<C>(cloud: &mut C, mut attempts: u8) -> Option<std::result::Result<C::Id, C::Error>>
where
    C: IdGeneratorMut,
    C::Error: NextAvailId,
    C::Output: Into<std::result::Result<C::Id, C::Error>>,
{
    while attempts != 0 {
        match cloud.next_id().into() {
            Ok(sf) => {
                return Some(Ok(sf))
            },
            Err(err) => {
                let Some(dur) = err.next_avail_id() else {
                    return Some(Err(err));
                };

                block_duration(dur);
            }
        }

        attempts -= 1;
    }

    None
}

