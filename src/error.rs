use std::time::Duration;

use crate::traits;

/// possible errors for Snowclouds/Snowflakes
#[derive(Debug)]
pub enum Error {

    /// a provided primary id is less than 0 or greater than the max value
    /// specified by a Snowcloud
    PrimaryIdInvalid,

    /// a provided epoch is less than 0 or greater than the max value
    /// specified by a Snowcloud
    EpochInvalid,

    /// a provided sequence is less than 0 or greater than the max value
    /// specified by a Snowcloud
    SequenceInvalid,

    /// the max possible timestamp value has been reached when generating a
    /// new id
    TimestampMaxReached,

    /// the max possible sequence value has been reached when generating a
    /// new id. the returned duration is an estimate on how long to wait for 
    /// the next millisecond
    SequenceMaxReached(Duration),

    /// failed to get a valid UNIX EPOCH timestamp
    TimestampError,

    /// error when attempting to lock a mutex
    MutexError,

    /// the provided i64 is not a valid Snowflake
    InvalidId,
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::PrimaryIdInvalid => write!(
                f, "primary id invalid"
            ),
            Error::EpochInvalid => write!(
                f, "epoch invalid"
            ),
            Error::SequenceInvalid => write!(
                f, "sequence invalid"
            ),
            Error::TimestampMaxReached => write!(
                f, "timestamp max reached"
            ),
            Error::SequenceMaxReached(_) => write!(
                f, "sequence max reached"
            ),
            Error::TimestampError => write!(
                f, "timestamp error"
            ),
            Error::MutexError => write!(
                f, "mutex error"
            ),
            Error::InvalidId => write!(
                f, "invalid id"
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<std::time::SystemTimeError> for Error {
    fn from(_: std::time::SystemTimeError) -> Error {
        Error::TimestampError
    }
}

impl traits::NextAvailId for Error {
    fn next_avail_id(&self) -> Option<&Duration> {
        match self {
            Error::SequenceMaxReached(dur) => Some(dur),
            _ => None
        }
    }
}
