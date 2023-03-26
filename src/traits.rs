//! base traits for implementing helper code
//!
//! good use case could be for implementing different waiter functions or ways
//! of getting ids from the base struct

/// basics of an id generator
pub trait IdGenerator {
    type Error;
    type Id;
    type Output;

    fn next_id(&self) -> Self::Output;
}

/// similar to IdGenerator but allows for mutating
pub trait IdGeneratorMut {
    type Error;
    type Id;
    type Output;

    fn next_id(&mut self) -> Self::Output;
}

/// for retrieving the duration of the next available id
pub trait NextAvailId {
    fn next_avail_id(&self) -> Option<&std::time::Duration>;
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

