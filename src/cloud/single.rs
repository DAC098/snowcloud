use std::time::{SystemTime, Duration};

use crate::traits;
use crate::error;
use crate::cloud::common::Counts;

/// non-thread safe generator
///
/// since the previous time and sequence count are not guarded, next id is
/// considered a mutating call and not thread safe. otherwise operates in
/// a similar fashion as [`MultiThread`](crate::MultiThread).
///
/// if you want to wait for the next available id without calling the function
/// again check out [`blocking_next_id_mut`](crate::wait::blocking_next_id_mut)
/// or other waiting methods depending on how you want to wait for the next
/// available id.
///
/// ```rust
/// type MyFlake = snowcloud::i64::SingleIdFlake<43, 8, 12>;
/// type MyCloud = snowcloud::SingleThread<MyFlake>;
///
/// const START_TIME: u64 = 1679587200000;
///
/// let mut cloud = MyCloud::new(START_TIME, 1)
///     .expect("failed to create MyCloud");
///
/// println!("epoch: {:?}", cloud.epoch());
/// println!("ids: {}", cloud.ids());
///
/// println!("{:?}", cloud.next_id());
/// ```
#[derive(Clone)]
pub struct SingleThread<F>
where
    F: traits::FromIdGenerator
{
    ep: SystemTime,
    ids: F::IdSegType,
    counts: Counts,
}

impl<F> SingleThread<F>
where
    F: traits::FromIdGenerator + Sized,
{
    /// returns a new SingleThread generator
    ///
    /// will return an error if the primary id is invalid, the timestamp is
    /// invalid, it failes to retrieve the current timestamp, or if the epoch
    /// is ahead of the current timestamp
    pub fn new<I>(epoch: u64, ids: I) -> error::Result<Self>
    where
        I: Into<F::IdSegType>
    {
        let ids = ids.into();

        if !F::valid_id(&ids) {
            return Err(error::Error::IdSegInvalid);
        }

        if !F::valid_epoch(&epoch) {
            return Err(error::Error::EpochInvalid);
        }

        let Some(sys_time) = SystemTime::UNIX_EPOCH.clone()
            .checked_add(Duration::from_millis(epoch)) else {
            return Err(error::Error::TimestampError);
        };
        let prev_time = sys_time.elapsed()?;

        Ok(SingleThread {
            ep: sys_time,
            ids,
            counts: Counts {
                sequence: 1,
                prev_time,
            }
        })
    }

    /// returns epoch
    pub fn epoch(&self) -> &SystemTime {
        &self.ep
    }

    /// returns ids.
    ///
    /// type is determined by the provided snowflake
    pub fn ids(&self) -> &F::IdSegType {
        &self.ids
    }

    /// retrieves the next available id
    ///
    /// if the current timestamp reaches max, the max sequence value is
    /// reached, or if it fails to get the current timestamp this will return
    /// an error
    pub fn next_id(&mut self) -> error::Result<F> {
        let seq: u64;

        let ts = self.ep.elapsed()?;

        if F::max_duration(&ts) {
            return Err(error::Error::TimestampMaxReached);
        }

        if F::current_tick(&ts, &self.counts.prev_time) {
            seq = self.counts.sequence;

            if F::max_sequence(&seq) {
                return Err(error::Error::SequenceMaxReached(F::next_tick(ts)));
            }

            self.counts.sequence += 1;
        } else {
            seq = 1;

            self.counts.prev_time = ts;
            self.counts.sequence = 2;
        }

        Ok(F::create(ts, seq, &self.ids))
    }
}

impl<F> traits::IdGeneratorMut for SingleThread<F>
where
    F: traits::FromIdGenerator
{
    type Error = error::Error;
    type Id = F;
    type Output = std::result::Result<Self::Id, Self::Error>;

    fn next_id(&mut self) -> Self::Output {
        SingleThread::next_id(self)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::io::Write as _;

    use super::*;
    use crate::flake::i64::SingleIdFlake;

    const START_TIME: u64 = 1679082337000;
    const MACHINE_ID: i64 = 1;

    type TestSnowflake = SingleIdFlake<43, 8, 12>;
    type TestSnowcloud = SingleThread<TestSnowflake>;

    #[test]
    fn unique_ids() -> () {
        let mut cloud = TestSnowcloud::new(START_TIME, MACHINE_ID).unwrap();
        let mut found_dups = false;
        let mut total_found: usize = 0;
        let mut unique_ids: HashMap<i64, Vec<(usize, TestSnowflake)>> = HashMap::new();
        let mut generated: Vec<TestSnowflake> = Vec::with_capacity(TestSnowflake::MAX_SEQUENCE as usize);

        for _ in 0..generated.capacity() {
            generated.push(cloud.next_id().expect("failed next_id"));
        }

        for i in 0..generated.len() {
            let flake = &generated[i];
            let id: i64 = flake.id();

            if let Some(dups) = unique_ids.get_mut(&id) {
                found_dups = true;
                total_found += 1;

                dups.push((i, flake.clone()));
            } else {
                let mut dups = Vec::with_capacity(1);
                dups.push((i, flake.clone()));

                unique_ids.insert(id, dups);
            }
        }

        if !found_dups {
            return;
        }

        let seq_width = (TestSnowflake::MAX_SEQUENCE.checked_ilog10().unwrap_or(0) + 1) as usize;
        let index_width = (generated.len().checked_ilog10().unwrap_or(0) + 1) as usize;
        let mut debug_output = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("SingleThread_unique_id.debug.txt")
            .expect("failed to create debug_file");

        debug_output.write_fmt(format_args!("total found: {} / {}\n", total_found, generated.len())).unwrap();

        for flake in &generated {
            let id = flake.id();

            if let Some(dups) = unique_ids.get(&id) {
                if dups.len() > 1 {
                    total_found += 1;

                    debug_output.write_fmt(format_args!(
                        "flake: {}\n",
                        id,
                    )).unwrap();

                    for dup in dups {
                        debug_output.write_fmt(format_args!(
                            "index: {:index_width$} {} {} {:seq_width$} | {}.{}\n",
                            dup.0,
                            dup.1.timestamp(),
                            dup.1.primary_id(),
                            dup.1.sequence(),
                            dup.1.duration().as_secs(),
                            dup.1.duration().subsec_nanos(),
                            index_width = index_width,
                            seq_width = seq_width,
                        )).unwrap();
                    }
                }
            }
        }

        debug_output.write(b"\n").unwrap();

        for index in 0..generated.len() {
            let mut is_dup = false;
            let id = generated[index].id();

            if let Some(dups) = unique_ids.get(&id) {
                is_dup = dups.len() > 1;
            }

            debug_output.write_fmt(format_args!(
                "{:index_width$} {} {} {:seq_width$} | {}.{} {}\n",
                index,
                generated[index].timestamp(),
                generated[index].primary_id(),
                generated[index].sequence(),
                generated[index].duration().as_secs(),
                generated[index].duration().subsec_nanos(),
                if is_dup { 'd' } else { ' ' },
                index_width = index_width,
                seq_width = seq_width,
            )).unwrap();
        }

        panic!("encountered duplidate ids. check SingleThread_unique_id.debug.txt for details"); 
    }
}
