//! Jito Steward instruction and event notification handling.
//!
//! This module handles configured Steward instruction alerts and converts parsed
//! Steward events into notification descriptions before applying event config
//! thresholds or simple destinations.

use log::debug;

use crate::{
    error::JitoBellError,
    event_parser::jito_steward::JitoStewardEvent,
    ix_parser::jito_steward::JitoStewardInstruction,
    program::{EventConfig, Instruction},
    tx_parser::JitoTransactionParser,
    JitoBellHandler,
};

/// Sends a notification for each matching `CopyDirectedStakeTargets` instruction
/// that includes notification metadata.
pub(crate) async fn handle_jito_steward_program(
    handler: &mut JitoBellHandler,
    parser: &JitoTransactionParser,
    jito_steward_instruction: &JitoStewardInstruction,
    instruction: &Instruction,
) -> Result<(), JitoBellError> {
    debug!("Jito Steward Instruction: {jito_steward_instruction}");

    if let JitoStewardInstruction::CopyDirectedStakeTargets {
        ix: _,
        vote_pubkey: _,
        total_target_lamports,
        validator_list_index: _,
    } = jito_steward_instruction
    {
        if let Some(ref notification_info) = instruction.notification_info {
            handler
                .dispatch_platform_notifications(
                    &notification_info.destinations,
                    &notification_info.description,
                    Some(*total_target_lamports as f64),
                    Some("lamports"),
                    &parser.transaction_signature,
                )
                .await?;
        }
    }

    Ok(())
}

pub(crate) async fn handle_jito_steward_event(
    handler: &mut JitoBellHandler,
    parser: &JitoTransactionParser,
    jito_steward_event: &JitoStewardEvent,
    event_config: EventConfig,
) -> Result<(), JitoBellError> {
    let (description, amount, unit) = match jito_steward_event {
        JitoStewardEvent::StateTransition(state_transition) => {
            let desc = format!(
                "Steward state transition occurred: {} → {}",
                state_transition.previous_state, state_transition.new_state
            );
            (desc, None, None)
        }
        JitoStewardEvent::Rebalance(rebalance) => {
            let (change_type, amount_lamports) = if rebalance.increase_lamports > 0 {
                ("Stake Increase", rebalance.increase_lamports)
            } else {
                (
                    "Stake Decrease",
                    rebalance.decrease_components.total_unstake_lamports,
                )
            };

            let amount_sol = amount_lamports as f64 / 1_000_000_000.0;
            let type_emoji = if rebalance.increase_lamports > 0 {
                "📈"
            } else {
                "📉"
            };
            let validator_full = rebalance.vote_account.to_string();
            let validator_url =
                format!("https://www.jito.network/stakenet/steward/{validator_full}/");

            let desc = format!(
                "{} *{}* | {:.2} SOL\n\
                    \n\
                    Validator: <{}|{}>\n\
                    Epoch: {} | Type: {:?}",
                type_emoji,
                change_type,
                amount_sol,
                validator_url,
                validator_full,
                rebalance.epoch,
                rebalance.rebalance_type_tag
            );

            (desc, Some(amount_sol), Some("SOL"))
        }
        JitoStewardEvent::DirectedRebalance(rebalance) => {
            let (change_type, amount_lamports) = if rebalance.increase_lamports > 0 {
                ("Stake Increase", rebalance.increase_lamports)
            } else {
                ("Stake Decrease", rebalance.decrease_lamports)
            };

            let amount_sol = amount_lamports as f64 / 1_000_000_000.0;
            let type_emoji = if rebalance.increase_lamports > 0 {
                "📈"
            } else {
                "📉"
            };

            let validator_full = rebalance.vote_account.to_string();
            let validator_url =
                format!("https://www.jito.network/stakenet/steward/{validator_full}/");

            let desc = format!(
                "{} *{}* | {:.2} SOL\n\
                    \n\
                    Validator: <{}|{}>\n\
                    Epoch: {} | Type: {:?}",
                type_emoji,
                change_type,
                amount_sol,
                validator_url,
                validator_full,
                rebalance.epoch,
                rebalance.rebalance_type_tag
            );

            (desc, Some(amount_sol), Some("SOL"))
        }
        _ => {
            debug!("Unhandled event type: {:?}", jito_steward_event);
            ("Unknown event".to_string(), None, None)
        }
    };

    match event_config {
        EventConfig::WithThresholds { thresholds } => {
            if let Some(amt) = amount {
                let matching_threshold =
                    thresholds.iter().filter(|t| amt >= t.value).max_by(|a, b| {
                        a.value
                            .partial_cmp(&b.value)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });

                if let Some(threshold) = matching_threshold {
                    let final_desc = if threshold.notification.description.is_empty() {
                        description.clone()
                    } else {
                        format!("{}\n\n{}", threshold.notification.description, description)
                    };
                    handler
                        .dispatch_platform_notifications(
                            &threshold.notification.destinations,
                            &final_desc,
                            Some(amt),
                            unit,
                            &parser.transaction_signature,
                        )
                        .await?;
                }
            }
        }
        EventConfig::Simple {
            destinations,
            description: config_desc,
        } => {
            // Use config description if provided, otherwise use generated description
            let final_desc = if config_desc.is_empty() {
                description
            } else {
                format!("{}\n\n{}", config_desc, description)
            };

            handler
                .dispatch_platform_notifications(
                    &destinations,
                    &final_desc,
                    amount,
                    unit,
                    &parser.transaction_signature,
                )
                .await?;
        }
    }

    Ok(())
}
