pub mod error;

#[cfg(features = "serde")]
pub mod serde_ext;

mod segments;

pub mod i64;
pub mod u64;
pub use segments::Segments;
