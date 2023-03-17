/// possible errors for Snowclouds/Snowflakes
#[derive(Debug)]
pub enum Error {
    MachineIdTooLarge,
    EpochTooLarge,
    EpochInFuture,
    TimestampMaxReached,
    SequenceMaxReached,
    TimestampError,
    InvalidId,
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MachineIdTooLarge => write!(
                f, "the given machine id is too large."
            ),
            Error::EpochTooLarge => write!(
                f, "the requested epoch is too large."
            ),
            Error::EpochInFuture => write!(
                f, "the requested epoch is in the future."
            ),
            Error::TimestampMaxReached => write!(
                f, "max timestamp reached"
            ),
            Error::SequenceMaxReached => write!(
                f, "max sequence reached"
            ),
            Error::TimestampError => write!(
                f, "timestamp error"
            ),
            Error::InvalidId => write!(
                f, "invalid id provided"
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

