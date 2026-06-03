//! Event log parsing for supported Jito Bell programs.
//!
//! Parser modules decode Solana program log records, match event
//! discriminators, and deserialize payloads from `events` into the shared
//! `EventParser` enum consumed by transaction handling and notification
//! routing.

use crate::event_parser::jito_steward::JitoStewardEvent;

pub mod jito_steward;

#[derive(Debug)]
pub enum EventParser {
    JitoSteward(JitoStewardEvent),
}
