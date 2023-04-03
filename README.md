# Snowcloud

[Documentation](https://docs.rs/snowcloud/) | [Crates.io](https://crates.io/crates/snowcloud)

a small library for implementing custom ids based on timestamps, static ids, and sequence counters. the module provides 2 types of generators, a thread safe and non thread safe version. they allow for different types of waiting for ids if you want specific behavior. the snowflakes generated by a generator are composed of 3 sections and are described in the documentation. small example of how to create a generator and create a snowflake is shown below.

```rust
// 43 bit timestamp, 8 bit primary id, 12 bit sequence
type MyCloud = snowcloud::SingleThread<43, 8, 12>;

// 2023/03/23 9:00:00 in milliseconds, timestamps will start from this
// date
const START_TIME: u64 = 1679587200000;
// primary id could be a machine/node id for example
const PRIMARY_ID: i64 = 1;

let mut cloud = MyCloud::new(PRIMARY_ID, START_TIME)
    .expect("failed to create MyCloud");
let flake = cloud.next_id()
    .expect("failed to create snowflake");

println!("{}", flake.id());
```

check out the [docs](https://docs.rs/snowcloud) for more information

## Features

 - de/serialize: supports serializing and deserializing snowflakes into integers or strings using [serde](https://serde.rs)

## State

there are additions that can be added. 
 - support for other integer types besides i64.
 - using Atomics if possible (or sane).
 - other helper methods or structs if they are a common enough use case.
 - trying to get more performance so that the library is not the bottle neck
 - other things?

complexity is low and this is not trying to achieve much. just to do its job well and be efficient about it.

since the api is fairly minimal there probably wont be too much in terms of change but just as a precaution this will not have a major version until it is finalized (open to suggestions).

## Contributions

fixes, improvements, or suggestions are welcome.
