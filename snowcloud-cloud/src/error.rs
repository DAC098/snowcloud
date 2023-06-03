use std::time::Duration;

use snowcloud_core::traits;

/// possible errors for generators
///
/// since the errors are not very complex no additional information is provided
/// except for SequenceMaxReached that provides the duration to the next
/// millisecond.
///
/// this error implements [`NextAvailId`](crate::traits::NextAvailId) if being
/// used in a generic way.
///
/// ```rust
/// use snowcloud::Error::SequenceMaxReached;
/// type MyFlake = snowcloud::i64::SingleIdFlake<43, 8, 12>;
/// type MyCloud = snowcloud::Generator<MyFlake>;
///
/// const START_TIME: u64 = 1679587200000;
///
/// let mut cloud = MyCloud::new(START_TIME, 1)
///     .expect("failed to create MyCloud");
///
/// match cloud.next_id() {
///     Ok(flake) => {
///         println!("{}", flake.id());
///     },
///     Err(err) => {
///         match err {
///             SequenceMaxReached(dur) => {
///                 // we can wait for the specified duration to try again
///             },
///             _ => {
///                 println!("{}", err);
///             }
///         }
///     }
/// }
/// ```
#[derive(Debug)]
pub enum Error {

    /// a provided id seg is invalid.
    IdSegInvalid,

    /// a provided epoch is invalid
    EpochInvalid,

    /// a provided sequence is less than 0 or greater than the max value
    /// specified by a Snowflake
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
    MutexError
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IdSegInvalid => write!(
                f, "id seg invalid"
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
