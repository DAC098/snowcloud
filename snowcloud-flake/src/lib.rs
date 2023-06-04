pub mod error;

#[cfg(feature = "serde")]
pub mod serde_ext;
#[cfg(feature = "postgres")]
mod pg;

mod segments;

pub mod i64;
pub mod u64;
pub use segments::Segments;
