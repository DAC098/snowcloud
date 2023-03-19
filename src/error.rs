/// possible errors for Snowclouds/Snowflakes
#[derive(Debug)]
pub enum Error {
    MachineIdInvalid,
    EpochInvalid,
    TimestampMaxReached,
    SequenceMaxReached(u32),
    TimestampError,
    MutexError,
    InvalidId,
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MachineIdInvalid => write!(
                f, "machine id invalid"
            ),
            Error::EpochInvalid => write!(
                f, "epoch invalid"
            ),
            Error::TimestampMaxReached => write!(
                f, "max timestamp reached"
            ),
            Error::SequenceMaxReached(_) => write!(
                f, "max sequence reached"
            ),
            Error::TimestampError => write!(
                f, "timestamp error"
            ),
            Error::MutexError => write!(
                f, "mutex error"
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

