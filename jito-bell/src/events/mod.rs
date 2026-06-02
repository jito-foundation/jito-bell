//! Event payload definitions emitted by supported on-chain programs.
//!
//! Program-specific modules define the Borsh-deserializable structs and
//! discriminators used by `event_parser` to turn transaction logs into typed
//! events. These types should mirror the on-chain event layouts closely so that
//! historical logs remain parseable.

pub mod jito_steward;
