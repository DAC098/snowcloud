//! base traits for implementing helper code
//!
//! good use case could be for implementing different waiter functions or ways
//! of getting ids from the base struct

use std::time::Duration;

/// basics of an id generator
///
/// describes what is needed to be considered an IdGenerator.
/// [`sync::MutexGenerator`](crate::sync::MutexGenerator) implements this
/// trait as an example
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
/// [`Generator`](crate::Generator) implements this trait as an example
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

pub trait IdBuilder {
    type Output;

    fn with_ts(&mut self, ts: u64) -> bool;
    fn with_seq(&mut self, seq: u64) -> bool;
    fn with_dur(&mut self, dur: Duration) -> () {}

    fn build(self) -> Self::Output;
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
    type Builder;

    /// validates a given IdSegType.
    fn valid_id(v: &Self::IdSegType) -> bool;

    /// validates a given epoch value
    fn valid_epoch(e: &u64) -> bool;

    fn builder(ids: &Self::IdSegType) -> Self::Builder;
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

