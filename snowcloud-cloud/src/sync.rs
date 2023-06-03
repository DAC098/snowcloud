use std::sync::{Arc, Mutex};
use std::time::{SystemTime, Duration};

use snowcloud_core::traits::{IdGenerator, FromIdGenerator, IdBuilder};

use crate::error;
use crate::common::Counts;

/// thread safe snowflake generator
///
/// generates a given snowflake with the provided epoch and id value. epoch is
/// a specified date that can be in the future of
/// [`UNIX_EPOCH`](std::time::SystemTime::UNIX_EPOCH) but not in the future of
/// now. the sequence value will always start at 1 when created.
///
/// this guards the previous time and sequence count behind an
/// [`Arc`](std::sync::Arc) [`Mutex`](std::sync::Mutex). the critical section
/// is small and will not block if its unable to get a valid snowflake.
///
/// if you want to wait for the next available id without calling the function
/// again check out [`blocking_next_id`](crate::wait::blocking_next_id) or
/// other waiting methods depending on how you want to wait for the next 
/// available id.
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
/// println!("epoch: {:?}", cloud.epoch());
/// println!("ids: {}", cloud.ids());
///
/// println!("{:?}", cloud.next_id());
/// ```
pub struct MutexGenerator<F>
where
    F: FromIdGenerator
{
    ep: SystemTime,
    ids: F::IdSegType,
    counts: Arc<Mutex<Counts>>,
}

impl<F> Clone for MutexGenerator<F>
where
    F: FromIdGenerator,
    F::IdSegType: Clone
{
    fn clone(&self) -> Self {
        MutexGenerator {
            ep: self.ep,
            ids: self.ids.clone(),
            counts: Arc::clone(&self.counts),
        }
    }
}

impl<F> MutexGenerator<F>
where
    F: FromIdGenerator,
    F::Builder: IdBuilder,
{
    /// returns a new MutexGenerator
    ///
    /// will return an error if ids is invalid, the timestamp is invalid, it 
    /// fails to retrieve the current timestamp, or if the epoch is ahead of 
    /// the current timestamp
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

        Ok(MutexGenerator {
            ep: sys_time,
            ids,
            counts: Arc::new(Mutex::new(Counts {
                sequence: 1,
                prev_time,
            }))
        })
    }

    /// returns epoch
    pub fn epoch(&self) -> &SystemTime {
        &self.ep
    }

    /// returns ids
    ///
    /// type is determined by the provided snowflake
    pub fn ids(&self) -> &F::IdSegType {
        &self.ids
    }

    /// retrieves the next available id
    ///
    /// if the current timestamp reaches max, the max sequence value is
    /// reached, or if it fails to get the current timestamp this will
    /// return an error.
    pub fn next_id(&self) -> error::Result<<<F as FromIdGenerator>::Builder as IdBuilder>::Output> {
        let mut builder = F::builder(&self.ids);
        let ts: Duration;

        {
            // lock down counts for the current thread
            let Ok(mut counts) = self.counts.lock() else {
                return Err(error::Error::MutexError);
            };

            // since we do not know when the lock will be freed we
            // have to get the time once the lock is freed to have
            // an accurate timestamp
            ts = self.ep.elapsed()?;
            let ts_secs = ts.as_secs();
            let ts_nanos = ts.subsec_nanos();
            let ts_millis = ts_nanos / 1_000_000;

            if !builder.with_ts(ts_secs * 1_000 + ts_millis as u64) {
                return Err(error::Error::TimestampMaxReached);
            }

            let prev_secs = counts.prev_time.as_secs();
            let prev_millis = counts.prev_time.subsec_nanos() / 1_000_000;

            // if we are still on the previously recorded millisecond
            // then we increment the sequence. since the comparison of
            // durations includes nanoseconds we have to do a little
            // more work to only compare what we want
            if prev_secs == ts_secs && prev_millis == ts_millis {
                // before we increment, check to make sure that we
                // have not reached the maximum sequence value. if
                // we have then given an estimate to the next
                // millisecond so that then user can decided on
                // how to wait for the next available value
                if !builder.with_seq(counts.sequence) {
                    return Err(error::Error::SequenceMaxReached(
                        Duration::from_nanos((1_000_000 - (ts_nanos % 1_000_000)) as u64)
                    ));
                }

                // increment to the next sequence number
                counts.sequence += 1;
            } else {
                // we are not on the previousely recorded millisecond
                // so the sequence value will be set to one
                builder.with_seq(1);

                // set the previous time to now and prep for the next
                // available sequence number
                counts.prev_time = ts;
                counts.sequence = 2;
            }

        // counts_lock should be dropped and the mutext should now be
        // unlocked for the next 
        }

        builder.with_dur(ts);

        Ok(builder.build())
    }
}

impl<F> IdGenerator for MutexGenerator<F>
where
    F: FromIdGenerator,
    F::Builder: IdBuilder
{
    type Error = error::Error;
    type Id = <<F as FromIdGenerator>::Builder as IdBuilder>::Output;
    type Output = Result<Self::Id, Self::Error>;

    fn next_id(&self) -> Self::Output {
        MutexGenerator::next_id(self)
    }
}

#[cfg(test)]
mod test {
    use std::sync::{Arc, Barrier};
    use std::collections::HashMap;
    use std::thread;
    use std::io::Write as _;

    use snowcloud_flake::i64::SingleIdFlake;

    use super::*;
    use crate::wait::blocking_next_id;

    const START_TIME: u64 = 1679082337000;
    const MACHINE_ID: i64 = 1;

    type TestSnowflake = SingleIdFlake<43, 8, 12>;
    type TestSnowcloud = MutexGenerator<TestSnowflake>;

    #[test]
    fn unique_ids() {
        let cloud = TestSnowcloud::new(START_TIME, MACHINE_ID).unwrap();
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
            .open("MutexGenerator_unique_id.debug.txt")
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

        panic!("encountered duplidate ids. check MutexGenerator_unique_id.debug.txt for details"); 
    }

    #[test]
    fn unique_ids_threaded() -> () {
        let start = std::time::Instant::now();
        let barrier = Arc::new(Barrier::new(3));
        let mut handles = Vec::with_capacity(3);
        let cloud = TestSnowcloud::new(START_TIME, MACHINE_ID).unwrap();

        for _ in 0..handles.capacity() {
            let t = start.clone();
            let b = Arc::clone(&barrier);
            let c = cloud.clone();

            handles.push(thread::spawn(move || {
                let mut id_list = Vec::with_capacity(TestSnowflake::MAX_SEQUENCE as usize);
                b.wait();

                for _ in 0..id_list.capacity() {
                    let Some(result) = blocking_next_id(&c, 2) else {
                        panic!("ran out of spin_next_id attempts");
                    };

                    id_list.push((
                        result.expect("failed spin_next_id"),
                        t.elapsed()
                    ));
                }

                id_list
            }));
        }

        let mut failed = false;
        let mut thread: usize = 0;
        let mut ordered_time_groups: Vec<std::time::Duration> = Vec::new();
        let mut time_groups: HashMap<std::time::Duration, Vec<Vec<usize>>> = HashMap::new();
        let mut unique_ids: HashMap<TestSnowflake, Vec<(usize, usize)>> = HashMap::new();
        let mut thread_list: Vec<Vec<(TestSnowflake, std::time::Duration)>> = Vec::with_capacity(handles.len());

        for handle in handles {
            let list = handle.join().expect("thread paniced");

            thread_list.push(list);

            for index in 0..thread_list[thread].len() {
                let (flake, dur) = &thread_list[thread][index];

                if let Some(groups) = time_groups.get_mut(dur) {
                    groups[thread].push(index);
                } else {
                    ordered_time_groups.push(dur.clone());

                    let mut group = Vec::with_capacity(thread_list.capacity());

                    for t in 0..group.capacity() {
                        let mut v = Vec::new();

                        if t == thread {
                            v.push(index);
                        }

                        group.push(v);
                    }

                    time_groups.insert(dur.clone(), group);
                }

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

        ordered_time_groups.sort();

        let mut debug_output = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("MutexGenerator_unique_id_threaded.debug.txt")
            .expect("failed to create debug_file");

        let mut joined_lists = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("MutexGenerator_unique_id_threaded_all.debug.txt")
            .expect("faled to create debug_file");

        let mut timing_groups = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("MutexGenerator_unique_id_threaded_time.debug.txt")
            .expect("failed to create debug_file");

        let max_seq_width = (TestSnowflake::MAX_SEQUENCE.checked_ilog10().unwrap_or(0) + 1) as usize;
        let max_duration = (ordered_time_groups.last().unwrap().as_nanos().checked_ilog10().unwrap_or(0) + 1) as usize;
        let mut max_ts_width = 0;

        for thread in 0..thread_list.len() {
            let decimals = (thread_list[thread].last().unwrap().0.timestamp().checked_ilog10().unwrap_or(0) + 1) as usize;

            if decimals > max_ts_width {
                max_ts_width = decimals;
            }
        }

        for dur in ordered_time_groups {
            timing_groups.write_fmt(format_args!(
                "{:width$} ",
                dur.as_nanos(),
                width = max_duration,
            )).unwrap();

            let mut first = true;
            let mut cntu = true;
            let mut iter_list = Vec::with_capacity(thread_list.len());

            for group in time_groups.get(&dur).unwrap() {
                iter_list.push(group.iter());
            }

            while cntu {
                cntu = false;
                let mut thread = 0;

                if !first {
                    timing_groups.write_fmt(format_args!(
                        "{:width$} ",
                        "",
                        width = max_duration,
                    )).unwrap();
                } else {
                    first = false;
                }

                for iter in iter_list.iter_mut() {
                    if let Some(index) = iter.next() {
                        timing_groups.write_fmt(format_args!(
                            " | {:ts_width$} {} {:seq_width$} {}",
                            thread_list[thread][*index].0.timestamp(),
                            thread_list[thread][*index].0.primary_id(),
                            thread_list[thread][*index].0.sequence(),
                            if unique_ids.get(&thread_list[thread][*index].0).unwrap().len() > 1 {
                                'd'
                            } else {
                                ' '
                            },
                            ts_width = max_ts_width,
                            seq_width = max_seq_width,
                        )).unwrap();

                        cntu = true;
                    } else {
                        timing_groups.write_fmt(format_args!(
                            " | {:ts_width$}   {:seq_width$}  ",
                            ' ',
                            ' ',
                            ts_width = max_ts_width,
                            seq_width = max_seq_width,
                        )).unwrap();
                    }

                    thread += 1;
                }

                timing_groups.write(b"\n").unwrap();
            }
        }

        for index in 0..(TestSnowflake::MAX_SEQUENCE as usize) {
            joined_lists.write_fmt(format_args!(
                "{:width$} ",
                index,
                width = 4,
            )).unwrap();

            for thread in 0..thread_list.len() {
                if thread > 0 {
                    joined_lists.write(b" | ").unwrap();
                }

                joined_lists.write_fmt(format_args!(
                    "{:ts_width$} {} {:seq_width$} {}",
                    thread_list[thread][index].0.timestamp(),
                    thread_list[thread][index].0.primary_id(),
                    thread_list[thread][index].0.sequence(),
                    if unique_ids.get(&thread_list[thread][index].0).unwrap().len() > 1 {
                        'd'
                    } else {
                        ' '
                    },
                    ts_width = max_ts_width,
                    seq_width = max_seq_width,
                )).unwrap();
            }

            joined_lists.write(b"\n").unwrap();
        }

        for (flake, dups) in unique_ids {
            if dups.len() > 1 {
                debug_output.write_fmt(format_args!("flake {} {} {}\n", flake.timestamp(), flake.primary_id(), flake.sequence())).unwrap();

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
                            "{:width$} {:ts_width$} {} {:seq_width$}\n", 
                            prev_index,
                            thread_list[thread][prev_index].0.timestamp(),
                            thread_list[thread][prev_index].0.primary_id(),
                            thread_list[thread][prev_index].0.sequence(),
                            width = index_decimals,
                            ts_width = max_ts_width,
                            seq_width = max_seq_width,
                        )).unwrap();
                    }

                    debug_output.write_fmt(format_args!(
                        "{:width$} {:ts_width$} {} {:seq_width$} dupliate\n",
                        index,
                        thread_list[thread][index].0.timestamp(),
                        thread_list[thread][index].0.primary_id(),
                        thread_list[thread][index].0.sequence(),
                        width = index_decimals,
                        ts_width = max_ts_width,
                        seq_width = max_seq_width,
                    )).unwrap();

                    if index != next {
                        for next_index in next..high {
                            debug_output.write_fmt(format_args!(
                                "{:width$} {:ts_width$} {} {:seq_width$}\n",
                                next_index,
                                thread_list[thread][next_index].0.timestamp(),
                                thread_list[thread][next_index].0.primary_id(),
                                thread_list[thread][next_index].0.sequence(),
                                width = index_decimals,
                                ts_width = max_ts_width,
                                seq_width = max_seq_width,
                            )).unwrap();
                        }
                    }
                }

                debug_output.write_fmt(format_args!("\n")).unwrap();
            }
        }

        panic!("encountered duplidate ids. check MutexGenerator_unique_id_threaded for output");
    }
}
