use std::time::Duration;
use std::hash::Hasher;

#[cfg(feature = "serde")]
use std::fmt;
#[cfg(feature = "serde")]
use serde::{de, ser};

use crate::error;
use crate::traits;
use crate::Segments;

/// u64 Snowflake with 1 id segment
///
/// the format is as follows with a 44 bit timestamp, 8 bit primary id, and 12
/// bit sequence:
///
/// ```text
///  11111111111111111111111111111111111111111111 - 11111111 - 111111111111
///  |                                          |   |      |   |          |
/// 64                                         21  20     13  12          1
///                                     timestamp          |              |
///                                               primary id              |
///                                                                sequence
/// ```
///
/// bit values for each segment can be specified by `TS`, `PID`, and `SEQ`.
/// the total amount of bits should equal 64.
///
/// Note: there is currently no way to ensure that the values provided are
/// valid. `generic_const_exprs` would help with this but is unstable currently
///
/// # Timestamp
///
/// timestamp is in milliseconds with a bit value specified by the `TS` const.
/// the snowflake holds the duration value of when the snowflake was created
/// and the timestamp will be pulled from that.
///
/// Note: when creating a snowflake outside of a generator the duration will
/// only be as accurate as the provided ts.
///
/// # Primary Id
///
/// specified by the `PID` const. used to help differentiate ids outside of the
/// timestamp and sequence values. an example representation could be different
/// server ids if being used across multiple machines in a web server.
///
/// # Sequence
///
/// specified by the `SEQ` const. indicates the count of when the snowflake was
/// generated in the same millisecond.
///
/// # De/Serialize
///
/// with the `serde` feature you can de/serialize a snowflake to and from an
/// [`u64`](core::primitive::u64) by default
///
/// ```rust
/// use serde::{Serialize, Deserialize};
///
/// type MyFlake = snowcloud::u64::SingleIdFlake<43, 8, 12>;
///
/// #[derive(Serialize, Deserialize)]
/// pub struct MyStruct {
///     id: MyFlake
/// }
///
/// let my_struct = MyStruct {
///     id: MyFlake::from_parts(1, 1, 1).unwrap(),
/// };
///
/// let json_string = serde_json::to_string(&my_struct).unwrap();
///
/// println!("{}", json_string);
/// ```
///
/// if you want more options check out [`serde_ext`](crate::serde_ext)
///
/// # Example Usage
///
/// ```rust
/// type MyFlake = snowcloud::u64::SingleIdFlake<43, 8, 12>;
/// type MyCloud = snowcloud::Generator<MyFlake>;
///
/// const START_TIME: u64 = 1679587200000;
///
/// let mut cloud = MyCloud::new(START_TIME, 1u64)
///     .expect("failed to create MyCloud");
/// let flake: MyFlake = cloud.next_id()
///     .expect("failed to create snowflake");
///
/// let id: u64 = flake.into();
/// println!("{}", id);
///
/// let and_back: MyFlake = id.try_into()
///     .expect("invalid u64 was provided");
/// println!("{:?}", and_back);
/// ```
#[derive(Eq, Clone)]
pub struct SingleIdFlake<const TS: u8, const PID: u8, const SEQ: u8> {
    pub(crate) ts: Duration,
    pub(crate) tsm: u64,
    pub(crate) pid: u64,
    pub(crate) seq: u64,
}

impl<const TS: u8, const PID: u8, const SEQ: u8> SingleIdFlake<TS, PID, SEQ> {
    /// max value that a timestamp can be. `(1 << TS as u64) - 1`
    pub const MAX_TIMESTAMP: u64 = (1 << TS as u64) - 1;
    /// max value that a primary id can be. `(1 << PID as u64) - 1`
    pub const MAX_PRIMARY_ID: u64 = (1 << PID as u64) - 1;
    /// max value a sequence can be. `(1 << SEQ as u64) - 1`
    pub const MAX_SEQUENCE: u64 = (1 << SEQ as u64) - 1;

    /// total bits to shift the timestamp. `(PID as u64 + SEQ as u64)`
    pub const TIMESTAMP_SHIFT: u64 = (PID as u64 + SEQ as u64);
    /// total bits to shift the primary id. `SEQ as u64`
    pub const PRIMARY_ID_SHIFT: u64 = SEQ as u64;

    /// bit mask for timestamp. `Self::MAX_TIMESTAMP << Self::TIMESTAMP_SHIFT`
    pub const TIMESTAMP_MASK: u64 = Self::MAX_TIMESTAMP << Self::TIMESTAMP_SHIFT;
    /// bit mask for primary id. `Self::MAX_PRIMARY_ID << Self::PRIMARY_ID_SHIFT`
    pub const PRIMARY_ID_MASK: u64 = Self::MAX_PRIMARY_ID << Self::PRIMARY_ID_SHIFT;
    /// bit mask for sequence. `Self::MAX_SEQUENCE`
    pub const SEQUENCE_MASK: u64 = Self::MAX_SEQUENCE;

    const MAX_EPOCH: u64 = (1 << TS as u64) - 1;
    const MAX_U64_SEQUENCE: u64 = (1 << SEQ as u64) - 1;
    const MAX_DURATION: Duration = Duration::from_millis(Self::MAX_EPOCH);

    /// returns duration
    ///
    /// if the flake was created outside of a Snowcloud then this will have
    /// less precision
    pub fn duration(&self) -> &Duration {
        &self.ts
    }

    /// returns timestamp
    pub fn timestamp(&self) -> &u64 {
        &self.tsm
    }

    /// returns primary id reference
    pub fn primary_id(&self) -> &u64 {
        &self.pid
    }

    /// returns sequence reference
    pub fn sequence(&self) -> &u64 {
        &self.seq
    }

    /// generates a Snowflake from the provided parts
    ///
    /// checks will be performed on each part to ensure that they are
    /// valid for the given Snowflake. 
    /// [`IdSegInvalid`](crate::error::Error::IdSegInvalid) will be returned if
    /// the primary id is invalid
    pub fn from_parts(tsm: u64, pid: u64, seq: u64) -> error::Result<Self> {
        if tsm > Self::MAX_TIMESTAMP {
            return Err(error::Error::EpochInvalid);
        }

        if pid > Self::MAX_PRIMARY_ID {
            return Err(error::Error::IdSegInvalid);
        }

        if seq > Self::MAX_SEQUENCE {
            return Err(error::Error::SequenceInvalid);
        }

        let ts = Duration::from_millis(tsm as u64);

        Ok(Self { ts, tsm, pid, seq })
    }

    /// splits the current Snowflake into its individual parts
    pub fn into_parts(self) -> (u64, u64, u64) {
        (self.tsm, self.pid, self.seq)
    }

    /// generates the unique id
    pub fn id(&self) -> u64 {
        (self.tsm << Self::TIMESTAMP_SHIFT) | (self.pid << Self::PRIMARY_ID_SHIFT) | self.seq
    }

    /// attempts to generated a snowflake from the given u64
    pub fn try_from(id: &u64) -> error::Result<Self> {
        let millis = ((*id & Self::TIMESTAMP_MASK) >> Self::TIMESTAMP_SHIFT) as u64;

        Ok(Self {
            ts: Duration::from_millis(millis),
            tsm: (id & Self::TIMESTAMP_MASK) >> Self::TIMESTAMP_SHIFT,
            pid: (id & Self::PRIMARY_ID_MASK) >> Self::PRIMARY_ID_SHIFT,
            seq: id & Self::SEQUENCE_MASK,
        })
    }

}

impl<const TS: u8, const PID: u8, const SEQ: u8> traits::Id for SingleIdFlake<TS, PID, SEQ> {
    type BaseType = u64;

    fn id(&self) -> Self::BaseType {
        SingleIdFlake::id(self)
    }
}

impl<const TS: u8, const PID: u8, const SEQ: u8> From<SingleIdFlake<TS, PID, SEQ>> for u64 {
    #[inline(always)]
    fn from(flake: SingleIdFlake<TS, PID, SEQ>) -> u64 {
        flake.id()
    }
}

impl<const TS: u8, const PID: u8, const SEQ: u8> From<&SingleIdFlake<TS, PID, SEQ>> for u64 {
    #[inline(always)]
    fn from(flake: &SingleIdFlake<TS, PID, SEQ>) -> u64 {
        flake.id()
    }
}

impl<const TS: u8, const PID: u8, const SEQ: u8> TryFrom<u64> for SingleIdFlake<TS, PID, SEQ> {
    type Error = error::Error;

    #[inline(always)]
    fn try_from(id: u64) -> Result<Self, Self::Error> {
        SingleIdFlake::try_from(&id)
    }
}

impl<const TS: u8, const PID: u8, const SEQ: u8> TryFrom<&u64> for SingleIdFlake<TS, PID, SEQ> {
    type Error = error::Error;

    #[inline(always)]
    fn try_from(id: &u64) -> Result<Self, Self::Error> {
        SingleIdFlake::try_from(id)
    }
}

impl<const TS: u8, const PID: u8, const SEQ: u8> std::cmp::PartialEq for SingleIdFlake<TS, PID, SEQ> {
    fn eq(&self, rhs: &Self) -> bool {
        self.tsm == rhs.tsm && self.pid == rhs.pid && self.seq == rhs.seq
    }
}

impl<const TS: u8, const PID: u8, const SEQ: u8> std::hash::Hash for SingleIdFlake<TS, PID, SEQ> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tsm.hash(state);
        self.pid.hash(state);
        self.seq.hash(state);
    }
}

impl<const TS: u8, const PID: u8, const SEQ: u8> std::fmt::Debug for SingleIdFlake<TS, PID, SEQ> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = self.id();

        f.debug_struct("SingleIdFlake")
            .field("id", &id)
            .field("ts", &self.ts)
            .field("tsm", &self.tsm)
            .field("pid", &self.pid)
            .field("seq", &self.seq)
            .finish()
    }
}

// current worry is that if not done properly values could get really weird
// if something happens in between checks unless create checks again to make
// sure that the values are correct
impl<const TS: u8, const PID: u8, const SEQ: u8> traits::FromIdGenerator for SingleIdFlake<TS, PID, SEQ> {
    type IdSegType = Segments<u64, 1>;

    fn valid_id(v: &Self::IdSegType) -> bool {
        *v.primary() > 0 && *v.primary() <= Self::MAX_PRIMARY_ID
    }

    fn valid_epoch(e: &u64) -> bool {
        *e <= Self::MAX_EPOCH
    }

    fn max_sequence(seq: &u64) -> bool {
        *seq > Self::MAX_U64_SEQUENCE
    }

    fn max_duration(ts: &Duration) -> bool {
        *ts > Self::MAX_DURATION
    }

    fn current_tick(ts: &Duration, prev: &Duration) -> bool {
        ts.as_millis() == prev.as_millis()
    }

    fn next_tick(ts: Duration) -> Duration {
        Duration::from_nanos((1_000_000 - (ts.subsec_nanos() % 1_000_000)) as u64)
    }

    fn create(ts: Duration, seq: u64, ids: &Self::IdSegType) -> Self {
        let tsm = ts.as_millis() as u64;

        Self {
            ts,
            tsm,
            pid: *ids.primary(),
            seq
        }
    }
}

#[cfg(feature = "serde")]
impl<const TS: u8, const PID: u8, const SEQ: u8> ser::Serialize for SingleIdFlake<TS, PID, SEQ> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer
    {
        let id = self.id();

        serializer.serialize_u64(id)
    }
}

#[cfg(feature = "serde")]
struct NumVisitor<const TS: u8, const PID: u8, const SEQ: u8> {}

#[cfg(feature = "serde")]
impl<'de, const TS: u8, const PID: u8, const SEQ: u8> de::Visitor<'de> for NumVisitor<TS, PID, SEQ> {
    type Value = SingleIdFlake<TS, PID, SEQ>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "integer from 0 to u64::MAX")
    }

    fn visit_i64<E>(self, i: i64) -> Result<Self::Value, E>
    where
        E: de::Error
    {
        let Ok(flake) = TryFrom::try_from(i as u64) else {
            return Err(E::invalid_value(de::Unexpected::Signed(i), &self));
        };

        Ok(flake)
    }

    fn visit_u64<E>(self, u: u64) -> Result<Self::Value, E>
    where
        E: de::Error
    {
        let Ok(flake) = TryFrom::try_from(u) else {
            return Err(E::invalid_value(de::Unexpected::Unsigned(u), &self));
        };

        Ok(flake)
    }
}

#[cfg(feature = "serde")]
impl<'de, const TS: u8, const PID: u8, const SEQ: u8> de::Deserialize<'de> for SingleIdFlake<TS, PID, SEQ> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_u64(NumVisitor {})
    }
}

#[cfg(test)]
mod test {
    use super::*;

    type TestSnowflake = SingleIdFlake<43, 8, 12>;

    #[test]
    fn properly_calculated_consts() {
        let max_timestamp: u64 = 0b1111111111111111111111111111111111111111111;
        let max_primary_id: u64 = 0b11111111;
        let max_sequence: u64 = 0b111111111111;

        let timestamp_shift: u64 = 8 + 12;
        let primary_id_shift: u64 = 12;

        let timestamp_mask: u64 =  0b0_1111111111111111111111111111111111111111111_00000000_000000000000;
        let primary_id_mask: u64 = 0b0_0000000000000000000000000000000000000000000_11111111_000000000000;
        let sequence_mask: u64 =   0b0_0000000000000000000000000000000000000000000_00000000_111111111111;

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

        let to_int: u64 = (&flake).into();
        let to_flake: TestSnowflake = (&to_int).try_into().unwrap();

        assert_eq!(to_flake, flake);
    }

    #[test]
    fn properly_shifted_integers() {
        let flake = TestSnowflake::from_parts(1, 1, 1).unwrap();

        let expected: u64 = 0b00000000000000000000000000000000000000000001_00000001_000000000001;

        assert_eq!(
            flake.id(),
            expected,
            "impropperly formatted snowflake.\n{:064b}\n{:064b}\n{:#?}",
            expected,
            flake.id(),
            flake
        );
    }

    #[cfg(feature = "serde")]
    mod serde_ext {
        use super::*;

        use serde::{Serialize, Deserialize};
        use serde_json;

        #[derive(Serialize, Deserialize)]
        struct IdFlake {
            id: TestSnowflake,
        }

        #[test]
        fn to_int() {
            let obj = IdFlake {
                id: TestSnowflake::from_parts(1, 1, 1).unwrap(),
            };

            match serde_json::to_string(&obj) {
                Ok(json_string) => {
                    assert_eq!(
                        json_string,
                        String::from("{\"id\":1052673}"),
                        "invalid json string"
                    );
                },
                Err(err) => {
                    panic!("failed to create json string. {:#?}", err);
                }
            }
        }

        #[test]
        fn from_int() {
            let json_str = "{\"id\":1052673}";

            match serde_json::from_str::<IdFlake>(json_str) {
                Ok(obj) => {
                    assert_eq!(
                        obj.id,
                        TestSnowflake::from_parts(1, 1, 1).unwrap(),
                        "invalid parsed id"
                    );
                },
                Err(err) => {
                    panic!("failed to parse json string. {:#?}", err);
                }
            }
        }
    }
}
