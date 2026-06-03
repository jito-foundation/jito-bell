//! Jito Vault notification handling.
//!
//! This module evaluates parsed vault instructions against configured VRT and
//! USD thresholds. It performs the RPC reads needed to resolve vault state, mint
//! decimals, and token metadata for notification amounts and symbols.

use std::str::FromStr;

use borsh::BorshDeserialize;
use defillama_rs::{
    models::{Chain, Token},
    DefiLlamaClient,
};
use jito_vault_client::accounts::Vault;
use log::debug;
use solana_sdk::pubkey::Pubkey;

use crate::{
    error::JitoBellError, ix_parser::vault::JitoVaultProgram, program::Instruction,
    tx_parser::JitoTransactionParser, JitoBellHandler,
};

use super::{divisor, sort_thresholds, vrt_symbol};

/// Handle Jito Vault Program
///
/// - Notify only once for the first matching threshold.
pub(crate) async fn handle_jito_vault_program(
    handler: &mut JitoBellHandler,
    parser: &JitoTransactionParser,
    jito_vault_program: &JitoVaultProgram,
    instruction: &Instruction,
) -> Result<(), JitoBellError> {
    debug!("Jito Vault Program: {}", jito_vault_program);

    match jito_vault_program {
        JitoVaultProgram::MintTo { ix, min_amount_out } => {
            let _config_info = &ix.accounts[0];
            let _vault_info = &ix.accounts[1];
            let vrt_mint_info = &ix.accounts[2];
            let _depositor_info = &ix.accounts[3];
            let _depositor_token_account = &ix.accounts[4];
            let _vault_token_account = &ix.accounts[5];
            let _depositor_vrt_token_account = &ix.accounts[6];
            let _vault_fee_token_account = &ix.accounts[7];

            if let Some(vrts) = instruction.vrts.clone() {
                if let Some((address, vrt_config)) =
                    vrts.get_key_value(&vrt_mint_info.pubkey.to_string())
                {
                    let vrt = Pubkey::from_str(address).unwrap();
                    let divisor = divisor(&handler.rpc_client, &vrt).await;
                    let symbol = vrt_symbol(&handler.rpc_client, &vrt).await;

                    let mut thresholds = vrt_config.thresholds.clone();
                    sort_thresholds(&mut thresholds);
                    for threshold in vrt_config.thresholds.iter() {
                        let min_amount_out = *min_amount_out as f64 / divisor;
                        if min_amount_out >= threshold.value {
                            handler
                                .dispatch_platform_notifications(
                                    &threshold.notification.destinations,
                                    &threshold.notification.description,
                                    Some(min_amount_out),
                                    Some(&symbol),
                                    &parser.transaction_signature,
                                )
                                .await?;
                            break;
                        }
                    }
                }
            }
        }
        JitoVaultProgram::EnqueueWithdrawal { ix, amount } => {
            let _config_info = &ix.accounts[0];
            let vault_info = &ix.accounts[1];
            let _vault_staker_withdrawal_ticket_info = &ix.accounts[2];
            let _vault_staker_withdrawal_ticket_token_account_info = &ix.accounts[3];
            let _staker_info = &ix.accounts[4];
            let _staker_vrt_token_account_info = &ix.accounts[5];
            let _base_info = &ix.accounts[6];

            let vault_acc = handler.rpc_client.get_account(&vault_info.pubkey).await?;
            let vault = Vault::deserialize(&mut vault_acc.data.as_slice())?;

            // VRT amount
            if let Some(ref vrts) = instruction.vrts {
                if let Some((address, vrt_config)) = vrts.get_key_value(&vault.vrt_mint.to_string())
                {
                    let vrt = Pubkey::from_str(address).unwrap();
                    let divisor = divisor(&handler.rpc_client, &vrt).await;
                    let symbol = vrt_symbol(&handler.rpc_client, &vrt).await;

                    let mut thresholds = vrt_config.thresholds.clone();
                    sort_thresholds(&mut thresholds);
                    for threshold in vrt_config.thresholds.iter() {
                        let amount = *amount as f64 / divisor;
                        if amount >= threshold.value {
                            handler
                                .dispatch_platform_notifications(
                                    &threshold.notification.destinations,
                                    &threshold.notification.description,
                                    Some(amount),
                                    Some(&symbol),
                                    &parser.transaction_signature,
                                )
                                .await?;
                            break;
                        }
                    }

                    // USD amount
                    if !vrt_config.usd_thresholds.is_empty() {
                        let client = DefiLlamaClient::new();
                        let vrt = Token::new(Chain::Solana, vrt.to_string());
                        let prices = client.get_price(&vrt).await?;

                        if let Some(usd_price) = prices.coins.values().last() {
                            let mut sorted_usd_thresholds = vrt_config.usd_thresholds.clone();
                            sorted_usd_thresholds.sort_by(|a, b| {
                                b.value
                                    .partial_cmp(&a.value)
                                    .unwrap_or(std::cmp::Ordering::Equal)
                            });

                            for usd_threshold in sorted_usd_thresholds.iter() {
                                let amount = *amount as f64 / 1_000_000_000_f64;
                                let amount = (amount * usd_price.price) as u64;

                                if amount >= usd_threshold.value {
                                    handler
                                        .dispatch_platform_notifications(
                                            &usd_threshold.notification.destinations,
                                            &usd_threshold.notification.description,
                                            Some(amount as f64),
                                            Some("USD"),
                                            &parser.transaction_signature,
                                        )
                                        .await?;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        JitoVaultProgram::InitializeConfig
        | JitoVaultProgram::InitializeVault
        | JitoVaultProgram::InitializeVaultWithMint
        | JitoVaultProgram::InitializeVaultOperatorDelegation
        | JitoVaultProgram::InitializeVaultNcnTicket
        | JitoVaultProgram::InitializeVaultNcnSlasherOperatorTicket
        | JitoVaultProgram::InitializeVaultNcnSlasherTicket
        | JitoVaultProgram::WarmupVaultNcnTicket
        | JitoVaultProgram::CooldownVaultNcnTicket
        | JitoVaultProgram::WarmupVaultNcnSlasherTicket
        | JitoVaultProgram::CooldownVaultNcnSlasherTicket
        | JitoVaultProgram::ChangeWithdrawalTicketOwner
        | JitoVaultProgram::BurnWithdrawalTicket
        | JitoVaultProgram::SetDepositCapacity
        | JitoVaultProgram::SetFees
        | JitoVaultProgram::SetProgramFee
        | JitoVaultProgram::SetProgramFeeWallet
        | JitoVaultProgram::SetIsPaused
        | JitoVaultProgram::DelegateTokenAccount
        | JitoVaultProgram::SetAdmin
        | JitoVaultProgram::SetSecondaryAdmin
        | JitoVaultProgram::AddDelegation
        | JitoVaultProgram::CooldownDelegation
        | JitoVaultProgram::UpdateVaultBalance
        | JitoVaultProgram::InitializeVaultUpdateStateTracker
        | JitoVaultProgram::CrankVaultUpdateStateTracker
        | JitoVaultProgram::CloseVaultUpdateStateTracker
        | JitoVaultProgram::CreateTokenMetadata
        | JitoVaultProgram::UpdateTokenMetadata
        | JitoVaultProgram::SetConfigAdmin => {
            unreachable!()
        }
    }

    Ok(())
}
