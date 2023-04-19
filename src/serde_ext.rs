//! additional serde options for de/serializing a snowflake
//!
//! provides one module for converting a snowflake to a string if something
//! cannot handle working with 64 bit signed integers (javascript).
//!
//! ```rust
//! use serde::{Serialize, Deserialize};
//! use snowcloud::serde_ext::string_id;
//!
//! type I64SID = snowcloud::i64::SingleIdFlake<43, 8, 12>;
//!
//! #[derive(Serialize, Deserialize)]
//! pub struct MyStruct {
//!     #[serde(with = "string_id")]
//!     id: I64SID,
//! }
//!
//! let my_struct = MyStruct {
//!     id: I64SID::from_parts(1, 1, 1).unwrap(),
//! };
//!
//! let json_string = serde_json::to_string(&my_struct).unwrap();
//!
//! println!("{}", json_string);
//! ```

pub trait FromStrRadix: Sized {
    type Error;

    fn from_str_radix(src: &str, radix: u32) -> Result<Self, Self::Error>;
}

macro_rules! from_str_radix {
    ($t:ty) => {
        impl FromStrRadix for $t {
            type Error = std::num::ParseIntError;

            #[inline(always)]
            fn from_str_radix(src: &str, radix: u32) -> Result<Self, Self::Error> {
                <$t>::from_str_radix(src, radix)
            }
        }
    };
}

from_str_radix!(i64);
from_str_radix!(u64);

/// de/serializes a snowflake to a string
///
/// structured to be used in `#[serde(with = "string_id")]`. will assume
/// base 10 number strings
pub mod string_id {
    use std::fmt;
    use std::marker::PhantomData;
    use core::convert::TryFrom;

    use serde::{ser, de};

    use super::FromStrRadix;
    use crate::traits;

    /// serializes a given snowflake to a string
    pub fn serialize<F, S>(flake: &F, serializer: S) -> Result<S::Ok, S::Error>
    where
        F: traits::Id,
        F::BaseType: ToString,
        S: ser::Serializer
    {
        let id_str = flake.id().to_string();

        serializer.serialize_str(id_str.as_str())
    }

    /// visitor for deserializing a string to a snowflake
    struct StringVisitor<F> {
        phantom: PhantomData<F>
    }

    impl<'de, F> de::Visitor<'de> for StringVisitor<F>
    where
        F: traits::Id + TryFrom<F::BaseType>,
        F::BaseType: FromStrRadix
    {
        type Value = F;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "non empty integer string within the valid range of the Id")
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let Ok(num) = FromStrRadix::from_str_radix(s, 10) else {
                return Err(E::invalid_value(de::Unexpected::Str(s), &self));
            };

            let Ok(flake) = TryFrom::try_from(num) else {
                return Err(E::invalid_value(de::Unexpected::Str(s), &self));
            };

            Ok(flake)
        }
    }

    /// deserializes a given string to a snowflake
    pub fn deserialize<'de, F, D>(deserializer: D) -> Result<F, D::Error>
    where
        F: traits::Id + TryFrom<F::BaseType>,
        F::BaseType: FromStrRadix,
        D: de::Deserializer<'de>
    {
        deserializer.deserialize_str(StringVisitor {
            phantom: PhantomData
        })
    }

    /// visitor for deserializ
    #[cfg(test)]
    mod test {
        use serde::{Serialize, Deserialize};
        use serde_json;

        use crate::serde_ext::string_id;
        use crate::flake;

        type I64SID = flake::i64::SingleIdFlake<43, 8, 12>;
        type I64DID = flake::i64::DualIdFlake<43, 4, 4, 12>;
        type U64SID = flake::u64::SingleIdFlake<44, 8, 12>;
        type U64DID = flake::u64::DualIdFlake<44, 4, 4, 12>;

        #[derive(Serialize, Deserialize)]
        struct I64SIDJson {
            #[serde(with = "string_id")]
            id: I64SID,
        }

        impl I64SIDJson {
            fn from_parts(ts: i64, seq: i64, pid: i64) -> Self {
                Self { id: I64SID::from_parts(ts, pid, seq).unwrap() }
            }
        }

        #[derive(Serialize, Deserialize)]
        struct I64DIDJson {
            #[serde(with = "string_id")]
            id: I64DID
        }

        impl I64DIDJson {
            fn from_parts(ts: i64, seq: i64, pid: i64, sid: i64) -> Self {
                Self { id: I64DID::from_parts(ts, pid, sid, seq).unwrap() }
            }
        }

        #[derive(Serialize, Deserialize)]
        struct U64SIDJson {
            #[serde(with = "string_id")]
            id: U64SID
        }

        impl U64SIDJson {
            fn from_parts(ts: u64, seq: u64, pid: u64) -> Self {
                Self { id: U64SID::from_parts(ts, pid, seq).unwrap() }
            }
        }

        #[derive(Serialize, Deserialize)]
        struct U64DIDJson {
            #[serde(with = "string_id")]
            id: U64DID
        }

        impl U64DIDJson {
            fn from_parts(ts: u64, seq: u64, pid: u64, sid: u64) -> Self {
                Self { id: U64DID::from_parts(ts, pid, sid, seq).unwrap() }
            }
        }

        macro_rules! string_test {
            ($to_string:ident, $from_string:ident, $type:path, $expected:literal, $ts:literal, $seq:literal, $($id:literal),+) => {
                #[test]
                fn $to_string() {
                    let obj = <$type>::from_parts($ts, $seq, $($id),+);

                    match serde_json::to_string(&obj) {
                        Ok(json_string) => {
                            assert_eq!(
                                json_string.as_str(),
                                $expected,
                                "invalid json string"
                            );
                        },
                        Err(err) => {
                            panic!("failed to create json string. {:#?}", err);
                        }
                    }
                }

                #[test]
                fn $from_string() {
                    match serde_json::from_str::<$type>($expected) {
                        Ok(obj) => {
                            assert_eq!(
                                obj.id,
                                <$type>::from_parts($ts, $seq, $($id),+).id,
                                "invalid parsed id"
                            );
                        },
                        Err(err) => {
                            panic!("failed to parse json string. {:#?}", err);
                        }
                    }
                }
            };
        }

        string_test!(
            to_string_i64_single_id_seg,
            from_string_i64_single_id_seg,
            I64SIDJson,
            "{\"id\":\"1052673\"}",
            1, 1, 1
        );

        string_test!(
            to_string_i64_dual_id_seg,
            from_string_i64_dual_id_seg,
            I64DIDJson,
            "{\"id\":\"1118209\"}",
            1, 1, 1, 1
        );

        string_test!(
            to_string_u64_single_id_seg,
            from_string_u64_single_id_seg,
            U64SIDJson,
            "{\"id\":\"1052673\"}",
            1, 1, 1
        );

        string_test!(
            to_string_u64_dual_id_seg,
            from_string_u64_dual_id_seg,
            U64DIDJson,
            "{\"id\":\"1118209\"}",
            1, 1, 1, 1
        );
    }
}
