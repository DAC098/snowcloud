//! additional serde options for de/serializing a snowflake
//!
//! provides one module for converting a snowflake to a string if something
//! cannot handle working with 64 bit signed integers (javascript).
//!
//! ```rust
//! use serde::{Serialize, Deserialize};
//! use snowcloud::serde_ext::i64_string_id;
//!
//! type MyFlake = snowcloud::Snowflake<43, 8, 12>;
//!
//! #[derive(Serialize, Deserialize)]
//! pub struct MyStruct {
//!     #[serde(with = "i64_string_id")]
//!     id: MyFlake,
//! }
//!
//! let my_struct = MyStruct {
//!     id: MyFlake::from_parts(1, 1, 1).unwrap(),
//! };
//!
//! let json_string = serde_json::to_string(&my_struct).unwrap();
//!
//! println!("{}", json_string);
//! ```

use std::string::ToString;

use serde::ser;

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

/// de/serializes a snowflake to a string
///
/// structured to be used in `#[serde(with = "i64_string_id")]`. will assume
/// base 10 number strings
pub mod i64_string_id {
    use std::fmt;
    use std::marker::PhantomData;
    use core::convert::TryFrom;

    use serde::de;

    use crate::traits;

    pub use super::serialize;

    /// visitor for deserializing a string to a snowflake
    struct StringVisitor<F> {
        phantom: PhantomData<F>
    }

    impl<'de, F> de::Visitor<'de> for StringVisitor<F>
    where
        F: traits::Id + TryFrom<i64> + Sized
    {
        type Value = F;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "a number string from 0 to 9,223,372,036,854,775,807")
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let Ok(int) = i64::from_str_radix(s, 10) else {
                return Err(E::invalid_value(de::Unexpected::Str(s), &self));
            };

            let Ok(flake) = TryFrom::try_from(int) else {
                return Err(E::invalid_value(de::Unexpected::Str(s), &self));
            };

            Ok(flake)
        }
    }

    /// deserializes a given string to a snowflake
    pub fn deserialize<'de, F, D>(deserializer: D) -> Result<F, D::Error>
    where
        F: traits::Id + TryFrom<i64> + Sized,
        D: de::Deserializer<'de>
    {
        deserializer.deserialize_str(StringVisitor {
            phantom: PhantomData
        })
    }

    #[cfg(test)]
    mod test {
        use serde::{Serialize, Deserialize};
        use serde_json;

        use crate::serde_ext::i64_string_id;
        use crate::flake::Snowflake;

        type MyFlake = Snowflake<43, 8, 12>;

        #[derive(Serialize, Deserialize)]
        struct StringFlake {
            #[serde(with = "i64_string_id")]
            id: MyFlake,
        }

        #[test]
        fn to_string() {
            let obj = StringFlake {
                id: MyFlake::from_parts(1, 1, 1).unwrap()
            };

            match serde_json::to_string(&obj) {
                Ok(json_string) => {
                    assert_eq!(
                        json_string,
                        String::from("{\"id\":\"1052673\"}"),
                        "invalid json string"
                    );
                },
                Err(err) => {
                    panic!("failed to create json string. {:#?}", err);
                }
            }
        }

        #[test]
        fn from_string() {
            let json_str = "{\"id\":\"1052673\"}";

            match serde_json::from_str::<StringFlake>(json_str) {
                Ok(obj) => {
                    assert_eq!(
                        obj.id,
                        MyFlake::from_parts(1, 1, 1).unwrap(),
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
