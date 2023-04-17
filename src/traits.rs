//! base traits for implementing helper code
//!
//! good use case could be for implementing different waiter functions or ways
//! of getting ids from the base struct

use std::time::Duration;

/// basics of an id generator
///
/// describes what is needed to be considered an IdGenerator.
/// [`MultiThread`](crate::MultiThread) implements this trait as an example
pub trait IdGenerator {
    /// the potential error that could be returned from next_id
    type Error;

    /// the actual Id type that is returned from next_id
    type Id;

    /// to help with allowing for different situations, Output can
    /// what ever is needed. a [`Result`](std::result::Result) or if used in
    /// an async context then an impl of [`Future`](core::future::Future) 
    type Output;

    /// call to get the next available id
    fn next_id(&self) -> Self::Output;
}

/// similar to [`IdGenerator`](crate::traits::IdGenerator) but allows for 
/// mutating
///
/// describes what is needed to be considered an IdGeneratorMut.
/// [`SingleThread`](crate::SingleThread) implements this trait as an example
pub trait IdGeneratorMut {
    /// the potential error that could be returned from next_id
    type Error;

    /// the actual Id type that is returned from next_id
    type Id;

    /// to help with allowing for different situations, Output can bwhat ever
    /// is needed. a [`Result`](std::result::Result) or if used in an async
    /// context then an impl of [`Future`](core::future::Future)
    type Output;

    /// mutating call to get the next available id
    fn next_id(&mut self) -> Self::Output;
}

/// for retrieving the duration of the next available id
///
/// [`Error`](crate::Error) implements this trait as an example
pub trait NextAvailId {
    /// optional return to get the duration to the next available id
    fn next_avail_id(&self) -> Option<&Duration>;
}

/// basic Snowflake structure
pub trait Id {
    /// what the id can be turned to and from
    type BaseType;

    /// creates the a value of BaseType from the id
    fn id(&self) -> Self::BaseType;
}

/// defines how to generate self from an IdGenerator
///
/// to reduce the amount of duplicate logic in generators a structure can
/// implement this to provide the necessary information for generating it.
pub trait FromIdGenerator: Sized {
    /// the type for the id segements that the id can handle. example of this
    /// would be an `i64` if the id only holds a single segment or `(i64, i64)`
    /// if it can hold two segments
    type IdSegType;

    /// validates a given IdSegType.
    fn valid_id(v: &Self::IdSegType) -> bool;

    /// validates a given epoch value
    fn valid_epoch(e: &u64) -> bool;

    /// checks to make sure that a given sequence number is not greater than
    /// what the id can handle
    fn max_sequence(seq: &u64) -> bool;

    /// checks to make sure that a given duration of time is not greater than
    /// what the id can handle
    fn max_duration(ts: &Duration) -> bool;

    /// checks to see if the current timestamp is equal to that of the previous
    /// timestamp. this will allow for having different durations of time for
    /// an id timestamp. checking if they are on the same millisecond or same
    /// second.
    fn current_tick(ts: &Duration, prev: &Duration) -> bool;

    /// gets the duration to the next available tick to cause the sequence to
    /// reset
    fn next_tick(ts: Duration) -> Duration;

    /// creates the id from the current timestampe, sequence, and provided
    /// IdSegType
    fn create(ts: Duration, seq: u64, ids: &Self::IdSegType) -> Self;
}

// when generic_const_exprs is stable this will be used to check that the
// provided bit values equal to 63
/*
enum Assert<const CHECK: bool> {}

trait IsTrue {}
trait IsFalse {}

impl IsTrue for Assert<true> {}
impl IsFalse for Assert<false> {}
*/

