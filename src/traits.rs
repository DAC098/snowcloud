//! base traits for implementing helper code
//!
//! good use case could be for implementing different waiter functions or ways
//! of getting ids from the base struct

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
    fn next_avail_id(&self) -> Option<&std::time::Duration>;
}

/// basic Snowflake structure
pub trait Id {
    /// what the id can be turned to and from
    type BaseType;

    /// creates the a value of BaseType from the id
    fn id(&self) -> Self::BaseType;
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

