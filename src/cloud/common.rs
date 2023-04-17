use std::time::Duration;

/// stores sequence and prev_time for a generator
#[derive(Clone)]
pub struct Counts {
    pub sequence: u64,
    pub prev_time: Duration,
}

