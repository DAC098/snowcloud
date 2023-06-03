//! # Snowcloud
//!
//! a small library for implementing custom ids based on timestamps, static
//! ids, and sequence counters. the module provides 2 types of generators, a
//! thread safe and non thread safe version. they allow for different types of
//! waiting for ids if you want specific behavior. each generator is capable of
//! using different snowflake types to allow for different snowflake formats.
//!
//! examples of using snowflakes with `i64` base types
//!
//! ```rust
//! // 43 bit timestamp, 8 bit primary id, 12 bit sequence
//! type MyFlake = snowcloud::i64::SingleIdFlake<43, 8, 12>;
//! type MyCloud = snowcloud::Generator<MyFlake>;
//!
//! // 2023/03/23 9:00:00 in milliseconds, timestamps will start from this
//! // date
//! const START_TIME: u64 = 1679587200000;
//!
//! let mut cloud = MyCloud::new(START_TIME, 1)
//!     .expect("failed to create MyCloud");
//! let flake = cloud.next_id()
//!     .expect("failed to create snowflake");
//!
//! println!("{}", flake.id());
//! ```
//!
//! creating a snowflake with two id segments
//!
//! ```rust
//! // 43 bit timestamp, 4 bit primary id, 4 bit secondary id, 12 bit sequence
//! type MyFlake = snowcloud::i64::DualIdFlake<43, 4, 4, 12>;
//! type MyCloud = snowcloud::Generator<MyFlake>;
//!
//! // 2023/03/23 9:00:00 in milliseconds, timestamps will start from this
//! // date
//! const START_TIME: u64 = 1679587200000;
//!
//! let mut cloud = MyCloud::new(START_TIME, (1, 1))
//!     .expect("failed to create MyCloud");
//! let flake = cloud.next_id()
//!     .expect("failed to create snowflake");
//!
//! println!("{}", flake.id());
//! ```
//!
//! ## Behavior
//!
//! [`sync::MutexGenerator`] is a thread safe implementation for sharing
//! between threads on a system. it uses an [`Arc`](std::sync::Arc)
//! [`Mutex`](std::sync::Mutex) to handle sharing the sequence count and
//! prev_time. the only time it will block is when acquiring the mutex and will
//! not wait if a valid snowflake cannot be acquired. if a generator is unable
//! to create a snowflake because the max sequence number has been reached an
//! error will be returned providing an estimated duration to the next 
//! millisecond. how you want to wait can be decided by the user.
//!
//! [`Generator`] is similar in most aspects to `sync::MutexGenerator` expect 
//! next_id is a mutating call and sequence count with prev_time are not stored
//! in an Arc Mutext. THIS IS NOT THREAD SAFE.
//!
//! ## Traits
//!
//! to help with using a generator in other situations, traits are provided and
//! implemented for the base types (more can be added later if
//! necessary/desired).
//!
//! - [`IdGenerator`](crate::traits::IdGenerator) describes the basic layout of
//!   an id generator. requiring an Error, Id, and Output type to be specified
//!   along with the next_id method.
//! - [`IdGeneratorMut`](crate::traits::IdGeneratorMut) is similar to
//!   [`IdGenerator`](crate::traits::IdGenerator) except the next_id call
//!   allows for mutating the object
//! - [`NextAvailId`](crate::traits::NextAvailId) describes an object that is
//!   capable of returing a [`duraiton`](std::time::Duration) to the next 
//!   available millisecond. check 
//!   [`blocking_next_id`](crate::wait::blocking_next_id) for example 
//!   implementation.
//! - [`Id`](crate::traits::Id) describes base methods for what an Id requires.
//!   currently just handles turning a snowflake into its base type like an
//!   `i64`.
//!
//! ## Timestamps
//!
//! all snowflakes use timestamps of milliseconds with the generators using
//! [`SystemTime`](std::time::SystemTime) to track the provided epoch and
//! generate new timestamps for snowflakes. the start date must be in the 
//! future of [`UNIX_EPOCH`](std::time::SystemTime::UNIX_EPOCH) and cannot be a
//! date in the future, `now >= start_time >= UNIX_EPOCH`.
//!
//! ```rust
//! // the current example date is 2023/03/23 9:00:00.
//! const VALID_START_DATE: u64 =   1679587200000;
//!
//! // if a date that is after the current system time is provided the
//! // snowcloud will return an error. 2077/10/23 9:00:00
//! const INVALID_START_DATE: u64 = 3402205200000;
//! ```
//!
//! below is a table with various bit values and how many years you can get out
//! of a timestamp. you will probably get diminishing returns with lower bit
//! values if this is to be used over a long duration of time.
//!
//! | bits | max value | years |
//! | ---: | --------: | ----: |
//! | 43 | 8796093022207 | 278 |
//! | 42 | 4398046511103 | 139 |
//! | 41 | 2199023255551 | 69 |
//! | 40 | 1099511627775 | 34 |
//! | 39 | 549755813887 | 17 |
//! | 38 | 274877906943 | 8 |
//! | 37 | 137438953471 | 4 |
//! | 36 | 68719476735 | 2 |
//! | 35 | 34359738367 | 1 |
//!
//! ## De/Serialize
//!
//! snowflakes support serde [`Serialize`](serde::Serialize) and
//! [`Deserialize`](serde::Deserialize) to there internal types with an 
//! addtional option to de/serailize to a string. see 
//! [`serde_ext`](crate::serde_ext) for additional methods of de/serialization

pub use snowcloud_core::traits;
pub use snowcloud_flake as flake;
pub use snowcloud_cloud as cloud;
