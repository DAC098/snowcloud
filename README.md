# Snowcloud

a small librarry for implementing custom ids based on timestamps, static ids, and sequence counters. the current implementation is thread safe and supports different methods of waiting on the next available id. timestamps are milliseconds and begin from a specifed start date from UNIX_EPOCH. this is roughtly based from Twitter Snowflake but allows for custom bit lengths that are specified at compile time.

```rust
// 43 bit timestamp, 8 bit primary id, 12 bit sequence
type MyCloud = Snowcloud<43, 8, 12>;

// 2023/03/23 9:00:00 in milliseconds, timestamps will start from this
// date
const START_TIME: u64 = 1679587200000;
// primary id could be a machine/node id for example
const PRIMARY_ID: i64 = 1;

let cloud = MyCloud::new(PRIMARY_ID, START_TIME);
let flake = cloud.next_id().unwrap();

println!("{}", flake.id());
```

Note: as it currently stands the Snowflake is desined to fit in a 64 bit signed integer that will never be negative. unfortunately there is no way to verify that the bit values specified equal 63 bits since the sign bit cannot be used.

## Behavior

a snowcloud next id will only block when acquring the mutext that guards the previous time and current sequence number. if its is unable to create a snowflake because the sequence has maxed out for the current millisecond and error will be returned. the specific error will contain an estimate of how many nanoseconds to wait for until the next millisecond. how to wait can be decied by the user.

## Traits

to help with using a snowcloud in other situations two traits are provided (more can be added later if nncessary/desired).

 - IdGenerator
 - NextAvailId

current use case would be for allowing different typeso f waiting for the next available id. see blocking_next_id for example implementation.

## Timestamp

as stated previously, the timestamp is in millisconds and is based from a specific start date that you can specify. the start date must be in the future of UNIX_EPOCH and cannot be a date in the future. internally, a snowcloud will use SystemTime to get the timestamp and the convert to the necessary values.

```rust
// the current example date is 2023/03/23 9:00:00
const VALID_START_DATE: u64 = 1679587200000;

// if a date that is after the current system time is provided the snowcloud
// will return an error. 2077/10/23 9:00:00
const INVALID_START_DATE: u64 = 3402205200000
```

below is a table with various bit values and how many years you can get out of a timestamp.

| bits | max value | years |
| ---: | --------: | ----: |
| 43 | 8796093022207 | 278 |
| 42 | 4398046511103 | 139 |
| 41 | 2199023255551 | 69 |
| 40 | 1099511627775 | 34 |
| 39 | 549755813887 | 17 |
| 38 | 274877906943 | 8 |
| 37 | 137438953471 | 4 |
| 36 | 68719476735 | 2 |
| 35 | 34359738367 | 1 |