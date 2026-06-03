//! Notification routing and shared helpers for parsed Jito Bell transactions.
//!
//! This module owns the lightweight dispatch layer between `JitoBellHandler` and
//! program-specific handler functions. Program modules should stay as free
//! functions and only depend on the handler for notification dispatch or explicit
//! runtime services such as RPC account reads.

mod jito_steward;
mod stake_pool;
mod vault;

use borsh::BorshDeserialize;
use log::debug;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use spl_token::state::Mint;

use crate::{
    error::JitoBellError, event_parser::EventParser, ix_parser::InstructionParser,
    program::ProgramName, threshold_config::ThresholdConfig, tx_parser::JitoTransactionParser,
    JitoBellHandler, DEFAULT_VRT_SYMBOL,
};

fn sort_thresholds(thresholds: &mut [ThresholdConfig]) {
    thresholds.sort_by(|a, b| {
        b.value
            .partial_cmp(&a.value)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

// TODO: Pure fn should not be async -- refactor fetch
async fn divisor(rpc_client: &RpcClient, vrt: &Pubkey) -> f64 {
    let decimals = match rpc_client.get_account(vrt).await {
        Ok(mint_acc) => match Mint::unpack(&mint_acc.data) {
            Ok(acc) => acc.decimals,
            Err(_) => 9,
        },
        Err(_e) => 9,
    };

    10_f64.powi(decimals as i32)
}

// TODO: Pure fn should not be async -- refactor fetch
async fn vrt_symbol(rpc_client: &RpcClient, vrt: &Pubkey) -> String {
    let meta_pubkey = jito_vault_sdk::inline_mpl_token_metadata::pda::find_metadata_account(vrt).0;
    let symbol = match rpc_client.get_account(&meta_pubkey).await {
        Ok(meta_acc) => {
            match jito_vault_client::log::metadata::Metadata::deserialize(
                &mut meta_acc.data.as_slice(),
            ) {
                Ok(meta) => meta.symbol,
                Err(_e) => DEFAULT_VRT_SYMBOL.to_string(),
            }
        }
        Err(_e) => DEFAULT_VRT_SYMBOL.to_string(),
    };

    symbol
}

/// Send notification
pub(crate) async fn send_notification(
    handler: &mut JitoBellHandler,
    parser: &JitoTransactionParser,
) -> Result<(), JitoBellError> {
    for program in &parser.instructions {
        match program {
            InstructionParser::SplToken2022(_) => {
                debug!("Token 2022");
            }
            InstructionParser::SplStakePool(spl_stake_program) => {
                debug!("SPL Stake Pool");

                if let Some(instruction) =
                    handler.get_instruction_config(ProgramName::SplStakePool, spl_stake_program)
                {
                    stake_pool::handle_spl_stake_pool_program(
                        handler,
                        parser,
                        spl_stake_program,
                        &instruction,
                    )
                    .await?;
                }
            }
            InstructionParser::JitoVault(jito_vault_program) => {
                debug!("Jito Vault");

                if let Some(instruction) =
                    handler.get_instruction_config(ProgramName::JitoVault, jito_vault_program)
                {
                    vault::handle_jito_vault_program(
                        handler,
                        parser,
                        jito_vault_program,
                        &instruction,
                    )
                    .await?;
                }
            }
            InstructionParser::JitoSteward(jito_steward_instruction) => {
                debug!("Jito Steward");

                if let Some(instruction) = handler
                    .get_instruction_config(ProgramName::JitoSteward, jito_steward_instruction)
                {
                    jito_steward::handle_jito_steward_program(
                        handler,
                        parser,
                        jito_steward_instruction,
                        &instruction,
                    )
                    .await?;
                }
            }
        }
    }

    for event in &parser.events {
        match event {
            EventParser::JitoSteward(jito_steward_event) => {
                if let Some(event_config) =
                    handler.get_event_config(ProgramName::JitoSteward, jito_steward_event)
                {
                    jito_steward::handle_jito_steward_event(
                        handler,
                        parser,
                        jito_steward_event,
                        event_config,
                    )
                    .await?;
                }
            }
        }
    }

    Ok(())
}
