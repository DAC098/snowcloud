//! provides u64 based snowflakes;

mod single;
mod dual;

pub use single::SingleIdFlake;
pub use dual::DualIdFlake;
