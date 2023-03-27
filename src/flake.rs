use std::time::Duration;
use std::hash::Hasher;

use crate::error;

/// id generated from a Snowcloud
///
/// since ts is a duration, it will only have full accuracy when generated. if
/// its created from an i64 or from_parts accuracy will be lost since the
/// timestamp will only be milliseconds not nanoseconds
///
/// the const values are specified when using a generator since they will be
/// passed down. if you need to explicitly store a snowflake type then you will
/// have to specify it before hand. make sure that the const values are
/// identical to that of the generator otherwise you will get compiler errors
///
/// ```rust
/// type MyFlake = snowcloud::Snowflake<43, 8, 12>;
/// type MyCloud = snowcloud::SingleThread<43, 8, 12>;
///
/// const START_TIME: u64 = 1679587200000;
/// const PRIMARY_ID: i64 = 1;
///
/// let mut cloud = MyCloud::new(PRIMARY_ID, START_TIME)
///     .expect("failed to create MyCloud");
/// let flake: MyFlake = cloud.next_id()
///     .expect("failed to create snowflake");
///
/// let id: i64 = flake.into();
/// println!("{}", id);
///
/// let and_back: MyFlake = id.try_into()
///     .expect("invalid i64 was provided");
/// println!("{:?}", and_back);
/// ```
#[derive(Eq, Clone)]
pub struct Snowflake<const TS: u8, const PID: u8, const SEQ: u8> {
    pub(crate) ts: Duration,
    pub(crate) tsm: i64,
    pub(crate) pid: i64,
    pub(crate) seq: i64,
}

impl<const TS: u8, const PID: u8, const SEQ: u8> Snowflake<TS, PID, SEQ> {
    /// max value that a timestamp can be
    pub const MAX_TIMESTAMP: i64 = (1 << TS as i64) - 1;
    /// max value that a primary id can be
    pub const MAX_PRIMARY_ID: i64 = (1 << PID as i64) - 1;
    /// max value a sequence can be
    pub const MAX_SEQUENCE: i64 = (1 << SEQ as i64) - 1;

    /// total bits to shift the timestamp
    pub const TIMESTAMP_SHIFT: i64 = (PID as i64 + SEQ as i64);
    /// total bits to shift the primary id
    pub const PRIMARY_ID_SHIFT: i64 = SEQ as i64;

    /// bit mask for timestamp
    pub const TIMESTAMP_MASK: i64 = Self::MAX_TIMESTAMP << Self::TIMESTAMP_SHIFT;
    /// bit mask for primary id
    pub const PRIMARY_ID_MASK: i64 = Self::MAX_PRIMARY_ID << Self::PRIMARY_ID_SHIFT;
    /// bit mask for sequence
    pub const SEQUENCE_MASK: i64 = Self::MAX_SEQUENCE;

    /// returns duration
    ///
    /// if the flake was created outside of a Snowcloud then this will have
    /// less precision
    pub fn duration(&self) -> &Duration {
        &self.ts
    }

    /// returns timestamp
    pub fn timestamp(&self) -> &i64 {
        &self.tsm
    }

    /// returns primary id reference
    pub fn primary_id(&self) -> &i64 {
        &self.pid
    }

    /// returns sequence reference
    pub fn sequence(&self) -> &i64 {
        &self.seq
    }

    /// generates a Snowflake from the provided parts
    ///
    /// checks will be performed on each part to ensure that they are
    /// valid for the given Snowflake
    pub fn from_parts(tsm: i64, pid: i64, seq: i64) -> error::Result<Self> {
        if tsm < 0 || tsm > Self::MAX_TIMESTAMP {
            return Err(error::Error::EpochInvalid);
        }

        if pid < 0 || pid > Self::MAX_PRIMARY_ID {
            return Err(error::Error::PrimaryIdInvalid);
        }

        if seq < 0 || seq > Self::MAX_SEQUENCE {
            return Err(error::Error::SequenceInvalid);
        }

        let ts = Duration::from_millis(tsm as u64);

        Ok(Snowflake { ts, tsm, pid, seq })
    }

    /// splits the current Snowflake into its individual parts
    pub fn into_parts(self) -> (i64, i64, i64) {
        (self.tsm, self.pid, self.seq)
    }

    /// generates the unique id
    pub fn id(&self) -> i64 {
        (self.tsm << Self::TIMESTAMP_SHIFT) | (self.pid << Self::PRIMARY_ID_SHIFT) | self.seq
    }
}

impl<const TS: u8, const PID: u8, const SEQ: u8> std::cmp::PartialEq for Snowflake<TS, PID, SEQ> {
    fn eq(&self, rhs: &Self) -> bool {
        self.tsm == rhs.tsm && self.pid == rhs.pid && self.seq == rhs.seq
    }
}

impl<const TS: u8, const PID: u8, const SEQ: u8> std::hash::Hash for Snowflake<TS, PID, SEQ> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tsm.hash(state);
        self.pid.hash(state);
        self.seq.hash(state);
    }
}

impl<const TS: u8, const PID: u8, const SEQ: u8> std::fmt::Debug for Snowflake<TS, PID, SEQ> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = self.id();

        f.debug_struct("Snowflake")
            .field("id", &id)
            .field("ts", &self.ts)
            .field("tsm", &self.tsm)
            .field("pid", &self.pid)
            .field("seq", &self.seq)
            .finish()
    }
}

impl<const TS: u8, const PID: u8, const SEQ: u8> From<Snowflake<TS, PID, SEQ>> for i64 {
    fn from(flake: Snowflake<TS, PID, SEQ>) -> i64 {
        flake.id()
    }
}

impl<const TS: u8, const PID: u8, const SEQ: u8> From<&Snowflake<TS, PID, SEQ>> for i64 {
    fn from(flake: &Snowflake<TS, PID, SEQ>) -> i64 {
        flake.id()
    }
}

impl<const TS: u8, const PID: u8, const SEQ: u8> TryFrom<i64> for Snowflake<TS, PID, SEQ> {
    type Error = error::Error;

    fn try_from(id: i64) -> error::Result<Self> {
        if id < 0 {
            return Err(error::Error::InvalidId);
        }

        let millis = ((id & Self::TIMESTAMP_MASK) >> Self::TIMESTAMP_SHIFT) as u64;

        Ok(Snowflake {
            ts: Duration::from_millis(millis),
            tsm: (id & Self::TIMESTAMP_MASK) >> Self::TIMESTAMP_SHIFT,
            pid: (id & Self::PRIMARY_ID_MASK) >> Self::PRIMARY_ID_SHIFT,
            seq: id & Self::SEQUENCE_MASK
        })
    }
}

impl<const TS: u8, const PID: u8, const SEQ: u8> TryFrom<&i64> for Snowflake<TS, PID, SEQ> {
    type Error = error::Error;

    fn try_from(id: &i64) -> error::Result<Self> {
        if *id < 0 {
            return Err(error::Error::InvalidId);
        }

        let millis = ((*id & Self::TIMESTAMP_MASK) >> Self::TIMESTAMP_SHIFT) as u64;

        Ok(Snowflake {
            ts: Duration::from_millis(millis),
            tsm: (id & Self::TIMESTAMP_MASK) >> Self::TIMESTAMP_SHIFT,
            pid: (id & Self::PRIMARY_ID_MASK) >> Self::PRIMARY_ID_SHIFT,
            seq: id & Self::SEQUENCE_MASK,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    type TestSnowflake = Snowflake<43, 8, 12>;

    #[test]
    fn properly_calculated_consts() {
        let max_timestamp: i64 = 0b1111111111111111111111111111111111111111111;
        let max_primary_id: i64 = 0b11111111;
        let max_sequence: i64 = 0b111111111111;

        let timestamp_shift: i64 = 8 + 12;
        let primary_id_shift: i64 = 12;

        let timestamp_mask: i64 =  0b0_1111111111111111111111111111111111111111111_00000000_000000000000;
        let primary_id_mask: i64 = 0b0_0000000000000000000000000000000000000000000_11111111_000000000000;
        let sequence_mask: i64 =   0b0_0000000000000000000000000000000000000000000_00000000_111111111111;

        assert_eq!(TestSnowflake::MAX_TIMESTAMP, max_timestamp, "invalid max timestamp");
        assert_eq!(TestSnowflake::MAX_PRIMARY_ID, max_primary_id, "invalid max primary id");
        assert_eq!(TestSnowflake::MAX_SEQUENCE, max_sequence, "invalid max sequence");

        assert_eq!(TestSnowflake::TIMESTAMP_SHIFT, timestamp_shift, "invalid timestamp shift");
        assert_eq!(TestSnowflake::PRIMARY_ID_SHIFT, primary_id_shift, "invalid primary id shift");

        assert_eq!(TestSnowflake::TIMESTAMP_MASK, timestamp_mask, "invalid timestamp mask");
        assert_eq!(TestSnowflake::PRIMARY_ID_MASK, primary_id_mask, "invalid primary id mask");
        assert_eq!(TestSnowflake::SEQUENCE_MASK, sequence_mask, "invalid sequence mask");
    }

    #[test]
    fn to_int_and_back() {
        let flake = TestSnowflake::from_parts(1, 1, 1).unwrap();

        let to_int: i64 = (&flake).into();
        let to_flake: TestSnowflake = (&to_int).try_into().unwrap();

        assert_eq!(to_flake, flake);
    }

    #[test]
    fn properly_shifted_integers() {
        let flake = TestSnowflake::from_parts(1, 1, 1).unwrap();

        let expected: i64 = 0b00000000000000000000000000000000000000000001_00000001_000000000001;

        assert_eq!(
            flake.id(),
            expected,
            "impropperly formatted snowflake.\n{:064b}\n{:064b}\n{:#?}",
            expected,
            flake.id(),
            flake
        );
    }
}
