use std::time::Duration;

/// stores sequence and prev_time for a generator
#[derive(Clone)]
pub struct Counts {
    pub sequence: i64,
    pub prev_time: Duration,
}

pub const NANOSECOND: u32 = 1_000_000;
pub const MILLI_IN_SECOND: i64 = 1_000;
