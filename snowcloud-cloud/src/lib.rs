use std::time::{SystemTime, Duration};

use snowcloud_core::traits::{IdGeneratorMut, FromIdGenerator, IdBuilder};

pub mod error;
pub mod wait;
mod common;
pub mod sync;

use common::Counts;

/// simple snowflake generator
///
/// generates a given snowflake with the provided epoch and id value. epoch is
/// a specified date that can be in the future of
/// [`UNIX_EPOCH`](std::time::SystemTime::UNIX_EPOCH) but not in the future of
/// now. the sequence value will always start at 1 when created.
///
/// if you want to wait for the next available id without calling the function
/// again check out [`blocking_next_id_mut`](crate::wait::blocking_next_id_mut)
/// or other waiting methods depending on how you want to wait for the next
/// available id.
///
/// ```rust
/// type MyFlake = snowcloud::i64::SingleIdFlake<43, 8, 12>;
/// type MyCloud = snowcloud::Generator<MyFlake>;
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
pub struct Generator<F>
where
    F: FromIdGenerator
{
    ep: SystemTime,
    ids: F::IdSegType,
    counts: Counts,
}

impl<F> Generator<F>
where
    F: FromIdGenerator,
    F::Builder: IdBuilder,
{
    /// returns a new Generator
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

        let Some(sys_time) = SystemTime::UNIX_EPOCH.checked_add(Duration::from_millis(epoch)) else {
            return Err(error::Error::TimestampError);
        };
        let prev_time = sys_time.elapsed()?;

        Ok(Generator {
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
    pub fn next_id(&mut self) -> error::Result<<<F as FromIdGenerator>::Builder as IdBuilder>::Output> {
        let mut builder = F::builder(&self.ids);

        let ts = self.ep.elapsed()?;
        let ts_secs = ts.as_secs();
        let ts_nanos = ts.subsec_nanos();
        let ts_millis = ts_nanos / 1_000_000;

        if !builder.with_ts(ts_secs * 1_000 + ts_millis as u64) {
            return Err(error::Error::TimestampMaxReached);
        }

        let prev_secs = self.counts.prev_time.as_secs();
        let prev_millis = self.counts.prev_time.subsec_nanos() / 1_000_000;

        if prev_secs == ts_secs && prev_millis == ts_millis {
            if !builder.with_seq(self.counts.sequence) {
                return Err(error::Error::SequenceMaxReached(
                    Duration::from_nanos((1_000_000 - (ts_nanos % 1_000_000)) as u64)
                ));
            }

            self.counts.sequence += 1;
        } else {
            builder.with_seq(1);

            self.counts.prev_time = ts;
            self.counts.sequence = 2;
        }

        builder.with_dur(ts);

        Ok(builder.build())
    }
}

impl<F> IdGeneratorMut for Generator<F>
where
    F: FromIdGenerator,
    F::Builder: IdBuilder,
{
    type Error = error::Error;
    type Id = <<F as FromIdGenerator>::Builder as IdBuilder>::Output;
    type Output = Result<Self::Id, Self::Error>;

    fn next_id(&mut self) -> Self::Output {
        Generator::next_id(self)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::io::Write as _;

    use snowcloud_flake::i64::SingleIdFlake;

    use super::*;

    const START_TIME: u64 = 1679082337000;
    const MACHINE_ID: i64 = 1;

    type TestSnowflake = SingleIdFlake<43, 8, 12>;
    type TestSnowcloud = Generator<TestSnowflake>;

    #[test]
    fn unique_ids() -> () {
        let mut cloud = TestSnowcloud::new(START_TIME, MACHINE_ID).unwrap();
        let mut found_dups = false;
        let mut total_found: usize = 0;
        let mut unique_ids: HashMap<i64, Vec<(usize, TestSnowflake)>> = HashMap::new();
        let mut generated: Vec<TestSnowflake> = Vec::with_capacity(TestSnowflake::MAX_SEQUENCE as usize);

        for i in 0..generated.capacity() {
            match cloud.next_id() {
                Ok(res) => {
                    generated.push(res);
                },
                Err(err) => {
                    panic!("failed next_id: {:?} {} {} {}", err, i, generated.capacity(), TestSnowflake::MAX_SEQUENCE);
                }
            }
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
            .open("Generator_unique_id.debug.txt")
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
                        let dur = dup.1.duration().unwrap();

                        debug_output.write_fmt(format_args!(
                            "index: {:index_width$} {} {} {:seq_width$} | {}.{}\n",
                            dup.0,
                            dup.1.timestamp(),
                            dup.1.primary_id(),
                            dup.1.sequence(),
                            dur.as_secs(),
                            dur.subsec_nanos(),
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

            let dur = generated[index].duration().unwrap();

            debug_output.write_fmt(format_args!(
                "{:index_width$} {} {} {:seq_width$} | {}.{} {}\n",
                index,
                generated[index].timestamp(),
                generated[index].primary_id(),
                generated[index].sequence(),
                dur.as_secs(),
                dur.subsec_nanos(),
                if is_dup { 'd' } else { ' ' },
                index_width = index_width,
                seq_width = seq_width,
            )).unwrap();
        }

        panic!("encountered duplidate ids. check Generator_unique_id.debug.txt for details"); 
    }
}
