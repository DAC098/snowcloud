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
    let start = std::time::Instant::now();
    let barrier = Arc::new(Barrier::new(3));
    let mut handles = Vec::with_capacity(3);
    let cloud = Snowcloud::new(MACHINE_ID, START_TIME).unwrap();

    for _ in 0..handles.capacity() {
        let t = start.clone();
        let b = Arc::clone(&barrier);
        let c = cloud.clone();

        handles.push(thread::spawn(move || {
            let mut id_list = Vec::with_capacity(MAX_SEQUENCE as usize);
            b.wait();

            for _ in 0..MAX_SEQUENCE {
                id_list.push((
                    c.spin_next_id().expect("failed spin_next_id"),
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
    let mut unique_ids: HashMap<Snowflake, Vec<(usize, usize)>> = HashMap::new();
    let mut thread_list: Vec<Vec<(Snowflake, std::time::Duration)>> = Vec::with_capacity(handles.len());

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
        .open("unique_id_multi_thread.debug.txt")
        .expect("failed to create debug_file");

    let mut joined_lists = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("unique_id_multi_thread_all.debug.txt")
        .expect("faled to create debug_file");

    let mut timing_groups = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("unique_id_multi_thread_time.debug.txt")
        .expect("failed to create debug_file");

    let max_seq_width = (MAX_SEQUENCE.checked_ilog10().unwrap_or(0) + 1) as usize;
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
                        thread_list[thread][*index].0.machine_id(),
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

    for index in 0..(MAX_SEQUENCE as usize) {
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
                thread_list[thread][index].0.machine_id(),
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
            debug_output.write_fmt(format_args!("flake {} {} {}\n", flake.timestamp(), flake.machine_id(), flake.sequence())).unwrap();

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
                        thread_list[thread][prev_index].0.machine_id(),
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
                    thread_list[thread][index].0.machine_id(),
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
                            thread_list[thread][next_index].0.machine_id(),
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

    panic!("encountered duplidate ids. check unique_id_multi_thread.deubg.txt for output");
}
