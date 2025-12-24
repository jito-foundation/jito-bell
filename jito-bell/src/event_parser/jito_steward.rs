use std::str::FromStr;

use base64::{engine::general_purpose::STANDARD, Engine};
use borsh::BorshDeserialize;
use log::error;
use solana_pubkey::Pubkey;

use crate::events::jito_steward::{
    AutoAddValidatorEvent, AutoRemoveValidatorEvent, DecreaseComponents, DirectedRebalanceEvent,
    EpochMaintenanceEvent, InstantUnstakeComponents, RebalanceEvent, ScoreComponents,
    StateTransition,
};

const PROGRAM_LOG: &str = "Program log: ";
const PROGRAM_DATA: &str = "Program data: ";

#[derive(Debug, Clone)]
pub enum JitoStewardEvent {
    AutoRemoveValidator(AutoRemoveValidatorEvent),
    AutoAddValidator(AutoAddValidatorEvent),
    EpochMaintenance(EpochMaintenanceEvent),
    StateTransition(StateTransition),
    Rebalance(RebalanceEvent),
    DirectedRebalance(DirectedRebalanceEvent),
    DecreaseComponents(DecreaseComponents),
    ScoreComponents(ScoreComponents),
    InstantUnstake(InstantUnstakeComponents),
}

impl std::fmt::Display for JitoStewardEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JitoStewardEvent::AutoRemoveValidator(_) => write!(f, "auto_remove_validator"),
            JitoStewardEvent::AutoAddValidator(_) => write!(f, "auto_add_validator"),
            JitoStewardEvent::EpochMaintenance(_) => write!(f, "epoch_maintenance"),
            JitoStewardEvent::StateTransition(_) => write!(f, "state_transition"),
            JitoStewardEvent::Rebalance(_) => write!(f, "rebalance"),
            JitoStewardEvent::DirectedRebalance(_) => write!(f, "directed_rebalance"),
            JitoStewardEvent::DecreaseComponents(_) => write!(f, "decrease_components"),
            JitoStewardEvent::ScoreComponents(_) => write!(f, "score_components"),
            JitoStewardEvent::InstantUnstake(_) => write!(f, "instant_unstake"),
        }
    }
}

impl JitoStewardEvent {
    /// Retrieve Program ID of Jito Steward Program
    pub fn program_id() -> Pubkey {
        Pubkey::from_str("Stewardf95sJbmtcZsyagb2dg4Mo8eVQho8gpECvLx8").unwrap()
    }

    /// Parse a log message and extract any events
    pub fn parse_log(log: &str) -> Option<JitoStewardEvent> {
        // Extract the base64 encoded data from the log
        let log_data = log
            .strip_prefix(PROGRAM_LOG)
            .or_else(|| log.strip_prefix(PROGRAM_DATA))?;

        // Decode base64
        let log_bytes = match STANDARD.decode(log_data) {
            Ok(bytes) => bytes,
            Err(e) => {
                error!("Could not base64 decode log: {log} - error: {e}");
                return None;
            }
        };

        // Need at least 8 bytes for discriminator
        if log_bytes.len() < 8 {
            return None;
        }

        let discriminator = &log_bytes[0..8];
        let event_data = &log_bytes[8..];

        // Try each event type by comparing discriminators

        // AutoRemoveValidatorEvent
        if discriminator == AutoRemoveValidatorEvent::DISCRIMINATOR {
            match AutoRemoveValidatorEvent::try_from_slice(event_data) {
                Ok(event) => return Some(JitoStewardEvent::AutoRemoveValidator(event)),
                Err(e) => error!("Failed to deserialize AutoRemoveValidatorEvent: {e}"),
            }
        }

        // AutoAddValidatorEvent
        if discriminator == AutoAddValidatorEvent::DISCRIMINATOR {
            match AutoAddValidatorEvent::try_from_slice(event_data) {
                Ok(event) => return Some(JitoStewardEvent::AutoAddValidator(event)),
                Err(e) => error!("Failed to deserialize AutoAddValidatorEvent: {e}"),
            }
        }

        // EpochMaintenanceEvent
        if discriminator == EpochMaintenanceEvent::DISCRIMINATOR {
            match EpochMaintenanceEvent::try_from_slice(event_data) {
                Ok(event) => return Some(JitoStewardEvent::EpochMaintenance(event)),
                Err(e) => error!("Failed to deserialize EpochMaintenanceEvent: {e}"),
            }
        }

        // StateTransition
        if discriminator == StateTransition::DISCRIMINATOR {
            match StateTransition::try_from_slice(event_data) {
                Ok(event) => return Some(JitoStewardEvent::StateTransition(event)),
                Err(e) => error!("Failed to deserialize StateTransition: {e}"),
            }
        }

        // RebalanceEvent
        if discriminator == RebalanceEvent::DISCRIMINATOR {
            match RebalanceEvent::try_from_slice(event_data) {
                Ok(event) => return Some(JitoStewardEvent::Rebalance(event)),
                Err(e) => error!("Failed to deserialize RebalanceEvent: {e}"),
            }
        }

        // RebalanceDirectedEvent
        if discriminator == DirectedRebalanceEvent::DISCRIMINATOR {
            match DirectedRebalanceEvent::try_from_slice(event_data) {
                Ok(event) => return Some(JitoStewardEvent::DirectedRebalance(event)),
                Err(e) => error!("Failed to deserialize DirectedRebalanceEvent: {e}"),
            }
        }

        // DecreaseComponents
        if discriminator == DecreaseComponents::DISCRIMINATOR {
            match DecreaseComponents::try_from_slice(event_data) {
                Ok(event) => return Some(JitoStewardEvent::DecreaseComponents(event)),
                Err(e) => error!("Failed to deserialize DecreaseComponents: {e}"),
            }
        }

        // ScoreComponents
        if discriminator == ScoreComponents::DISCRIMINATOR {
            match ScoreComponents::try_from_slice(event_data) {
                Ok(event) => return Some(JitoStewardEvent::ScoreComponents(event)),
                Err(e) => error!("Failed to deserialize ScoreComponents: {e}"),
            }
        }

        // InstantUnstakeComponents
        if discriminator == InstantUnstakeComponents::DISCRIMINATOR {
            match InstantUnstakeComponents::try_from_slice(event_data) {
                Ok(event) => return Some(JitoStewardEvent::InstantUnstake(event)),
                Err(e) => error!("Failed to deserialize InstantUnstakeComponents: {e}"),
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rebalance_event_log() {
        let log = "Program data: eBt162gqhEtMsYgkyyouvKpCwIQK+PQ/fXApJFO4g1PcCGJg3zGq0YMDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==";

        let event = JitoStewardEvent::parse_log(log);
        assert!(event.is_some());

        let event = event.unwrap();
        match event {
            JitoStewardEvent::Rebalance(_) => {
                println!("Successfully parsed RebalanceEvent");
            }
            _ => panic!("Expected RebalanceEvent, got {event:?}"),
        }
    }

    #[test]
    fn test_parse_state_transition_log() {
        let log = "Program data: agl496lqzumDAwAAAAAAAFUiLBcAAAAACQAAAFJlYmFsYW5jZQQAAABJZGxl";

        let event = JitoStewardEvent::parse_log(log);
        assert!(event.is_some());

        let event = event.unwrap();
        match event {
            JitoStewardEvent::StateTransition(st) => {
                println!("Successfully parsed StateTransition: {st:?}");
            }
            _ => panic!("Expected StateTransition, got {event:?}"),
        }
    }

    #[test]
    fn test_parse_directed_rebalance_log() {
        let log = "Program data: uz87SL9AcR380T2ote3ZsSVQXwXdXsmLELwWrxqrmm7aHBZ2nE7pQoMDAAAAAAAAAAAAAAAAAAAAAAA=";

        let event = JitoStewardEvent::parse_log(log);
        assert!(event.is_some());

        let event = event.unwrap();
        match event {
            JitoStewardEvent::DirectedRebalance(st) => {
                println!("Successfully parsed DirectedRebalance: {st:?}");
            }
            _ => panic!("Expected DirectedRebalance, got {event:?}"),
        }
    }
}
