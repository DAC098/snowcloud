# Snowcloud

[Documentation](https://docs.rs/snowcloud/) | [Crates.io](https://crates.io/crates/snowcloud)

a small library for implementing custom ids based on timestamps, static ids, and sequence counters. the module provides 2 types of generators, a thread safe and non thread safe version. they allow for different types of waiting for ids if you want specific behavior. each generator is capable of using different snowflake types to allow for different snowflake formats.

```rust
// 43 bit timestamp, 8 bit primary id, 12 bit sequence
type MyFlake = snowcloud::i64::SingleIdFlake<43, 8, 12>;
type MyCloud = snowcloud::Generator<MyFlake>;

// 2023/03/23 9:00:00 in milliseconds, timestamps will start from this
// date
const START_TIME: u64 = 1679587200000;

let mut cloud = MyCloud::new(START_TIME, 1)
    .expect("failed to create MyCloud");
let flake = cloud.next_id()
    .expect("failed to create snowflake");

println!("{}", flake.id());
```

check out the [docs](https://docs.rs/snowcloud) for more information

## Features

 - integer types: support for using i64 / u64 underlying integer types
 - id segments: support for different amount of id segments, 1 / 2 static ids in a snowflake with the timestamp and sequence
 - de/serialize: supports serializing and deserializing snowflakes into integers or strings using [serde](https://serde.rs)

## State

there are additions that can be added. 
 - using Atomics if possible (or sane).
 - other helper methods or structs if they are a common enough use case.
 - trying to get more performance so that the library is not the bottle neck
 - other things?

since the api is fairly minimal there probably wont be too much in terms of change but just as a precaution this will not have a major version until it is finalized (open to suggestions).

## Contributions

fixes, improvements, or suggestions are welcome.
