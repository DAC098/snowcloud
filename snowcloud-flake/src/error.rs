/// possible errors for Snowclouds/Snowflakes
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

    /// the provided i64 is not a valid Snowflake
    InvalidId,

    /// provided too many segments for creating a Snowflake
    TooManySegments
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
            Error::InvalidId => write!(
                f, "invalid id"
            ),
            Error::TooManySegments => write!(
                f, "too many segments"
            )
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
