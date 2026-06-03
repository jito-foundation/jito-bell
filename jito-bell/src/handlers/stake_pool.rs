//! SPL Stake Pool notification handling.
//!
//! This module evaluates parsed stake pool instructions against configured LST
//! and stake-pool thresholds. It may inspect related Token 2022 instructions in
//! the same parsed transaction to determine the minted pool-token amount.

use log::debug;

use crate::{
    error::JitoBellError,
    ix_parser::{
        stake_pool::SplStakePoolProgram, token_2022::SplToken2022Program, InstructionParser,
    },
    program::InstructionConfig,
    tx_parser::JitoTransactionParser,
    JitoBellHandler,
};

use super::sort_thresholds;

/// Handle SPL Stake Pool Program
///
/// - Notify only once for the first matching threshold.
pub(crate) async fn handle_spl_stake_pool_program(
    handler: &mut JitoBellHandler,
    parser: &JitoTransactionParser,
    spl_stake_program: &SplStakePoolProgram,
    instruction: &InstructionConfig,
) -> Result<(), JitoBellError> {
    debug!("SPL Stake Program: {}", spl_stake_program);

    match spl_stake_program {
        SplStakePoolProgram::IncreaseValidatorStake { ix, amount } => {
            let stake_pool_info = &ix.accounts[0];
            let _staker_info = &ix.accounts[1];
            let _withdraw_authority_info = &ix.accounts[2];
            let _validator_list_info = &ix.accounts[3];
            let _reserve_stake_account_info = &ix.accounts[4];
            let _maybe_ephemeral_stake_account_info = &ix.accounts[5];
            let _validator_stake_account_info = &ix.accounts[6];
            let _validator_vote_account_info = &ix.accounts[7];
            let _clock_info = &ix.accounts[8];
            let _rent_info = &ix.accounts[9];
            let _stake_history_info = &ix.accounts[10];
            let _stake_config_info = &ix.accounts[11];
            let _system_program_info = &ix.accounts[12];
            let _stake_program_info = &ix.accounts[13];

            if let Some(mut stake_pools) = instruction.stake_pools.clone() {
                if let Some(alert_config) = stake_pools.get_mut(&stake_pool_info.pubkey.to_string())
                {
                    sort_thresholds(alert_config.thresholds.as_mut());
                    for threshold in alert_config.thresholds.iter() {
                        if *amount > threshold.value {
                            handler
                                .dispatch_platform_notifications(
                                    &threshold.notification.destinations,
                                    &threshold.notification.description,
                                    Some(*amount),
                                    Some("SOL"),
                                    &parser.transaction_signature,
                                )
                                .await?;
                            break;
                        }
                    }
                }
            }
        }
        SplStakePoolProgram::DepositStake { ix } => {
            let _stake_pool_info = &ix.accounts[0];
            let _validator_list_info = &ix.accounts[1];
            let _stake_deposit_authority_info = &ix.accounts[2];
            let withdraw_authority_info = &ix.accounts[3];
            let _stake_info = &ix.accounts[4];
            let _validator_stake_account_info = &ix.accounts[5];
            let _reserve_stake_account_info = &ix.accounts[6];
            let dest_user_pool_info = &ix.accounts[7];
            let _manager_fee_info = &ix.accounts[8];
            let _referrer_fee_info = &ix.accounts[9];
            let pool_mint_info = &ix.accounts[10];

            if let Some(mut lsts) = instruction.lsts.clone() {
                if let Some(alert_config) = lsts.get_mut(&pool_mint_info.pubkey.to_string()) {
                    for program in &parser.instructions {
                        if let InstructionParser::SplToken2022(program) = program {
                            match program {
                                SplToken2022Program::MintTo { ix, amount } => {
                                    let mint_info = &ix.accounts[0];
                                    let destination_account_info = &ix.accounts[1];
                                    let owner_info = &ix.accounts[2];

                                    if mint_info.pubkey.eq(&pool_mint_info.pubkey)
                                        && destination_account_info
                                            .pubkey
                                            .eq(&dest_user_pool_info.pubkey)
                                        && owner_info.pubkey.eq(&withdraw_authority_info.pubkey)
                                    {
                                        sort_thresholds(alert_config.thresholds.as_mut());
                                        for threshold in alert_config.thresholds.iter() {
                                            if *amount as f64 > threshold.value {
                                                handler
                                                    .dispatch_platform_notifications(
                                                        &threshold.notification.destinations,
                                                        &threshold.notification.description,
                                                        Some(*amount as f64),
                                                        Some("SOL"),
                                                        &parser.transaction_signature,
                                                    )
                                                    .await?;
                                                break;
                                            }
                                        }

                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        SplStakePoolProgram::WithdrawStake {
            ix,
            minimum_lamports_out,
        } => {
            let _stake_pool_info = &ix.accounts[0];
            let _validator_list_info = &ix.accounts[1];
            let _withdraw_authority_info = &ix.accounts[2];
            let _stake_split_from = &ix.accounts[3];
            let _stake_split_to = &ix.accounts[4];
            let _user_stake_authority_info = &ix.accounts[5];
            let _user_transfer_authority_info = &ix.accounts[6];
            let _burn_from_pool_info = &ix.accounts[7];
            let _manager_fee_info = &ix.accounts[8];
            let pool_mint_info = &ix.accounts[9];

            if let Some(mut lsts) = instruction.lsts.clone() {
                if let Some(alert_config) = lsts.get_mut(&pool_mint_info.pubkey.to_string()) {
                    sort_thresholds(alert_config.thresholds.as_mut());
                    for threshold in alert_config.thresholds.iter() {
                        if *minimum_lamports_out >= threshold.value {
                            handler
                                .dispatch_platform_notifications(
                                    &threshold.notification.destinations,
                                    &threshold.notification.description,
                                    Some(*minimum_lamports_out),
                                    Some("SOL"),
                                    &parser.transaction_signature,
                                )
                                .await?;
                            break;
                        }
                    }
                }
            }
        }
        SplStakePoolProgram::DepositSol { ix, amount } => {
            let _stake_pool_info = &ix.accounts[0];
            let _withdraw_authority_info = &ix.accounts[1];
            let _reserve_stake_account_info = &ix.accounts[2];
            let _from_user_lamports_info = &ix.accounts[3];
            let _dest_user_pool_info = &ix.accounts[4];
            let _manager_fee_info = &ix.accounts[5];
            let _referrer_fee_info = &ix.accounts[6];
            let pool_mint_info = &ix.accounts[7];

            if let Some(mut lsts) = instruction.lsts.clone() {
                if let Some(alert_config) = lsts.get_mut(&pool_mint_info.pubkey.to_string()) {
                    sort_thresholds(alert_config.thresholds.as_mut());
                    for threshold in alert_config.thresholds.iter() {
                        if *amount >= threshold.value {
                            handler
                                .dispatch_platform_notifications(
                                    &threshold.notification.destinations,
                                    &threshold.notification.description,
                                    Some(*amount),
                                    Some("SOL"),
                                    &parser.transaction_signature,
                                )
                                .await?;
                            break;
                        }
                    }
                }
            }
        }
        SplStakePoolProgram::WithdrawSol { ix, amount } => {
            let _stake_pool_info = &ix.accounts[0];
            let _withdraw_authority_info = &ix.accounts[1];
            let _user_transfer_authority_info = &ix.accounts[2];
            let _burn_from_pool_info = &ix.accounts[3];
            let _reserve_stake_info = &ix.accounts[4];
            let _destination_lamports_info = &ix.accounts[5];
            let _manager_fee_info = &ix.accounts[6];
            let pool_mint_info = &ix.accounts[7];

            if let Some(mut lsts) = instruction.lsts.clone() {
                if let Some(alert_config) = lsts.get_mut(&pool_mint_info.pubkey.to_string()) {
                    sort_thresholds(alert_config.thresholds.as_mut());
                    for threshold in alert_config.thresholds.iter() {
                        if *amount >= threshold.value {
                            handler
                                .dispatch_platform_notifications(
                                    &threshold.notification.destinations,
                                    &threshold.notification.description,
                                    Some(*amount),
                                    Some("SOL"),
                                    &parser.transaction_signature,
                                )
                                .await?;
                            break;
                        }
                    }
                }
            }
        }
        SplStakePoolProgram::DecreaseValidatorStakeWithReserve { ix, amount } => {
            let stake_pool_info = &ix.accounts[0];
            let _staker_info = &ix.accounts[1];
            let _stake_pool_withdraw_authority_info = &ix.accounts[2];
            let _validator_list_info = &ix.accounts[3];
            let _reserve_stake_account_info = &ix.accounts[4];
            let _validator_stake_info = &ix.accounts[5];
            let _transient_stake_info = &ix.accounts[6];
            let _clock_info = &ix.accounts[7];
            let _stake_history_info = &ix.accounts[8];
            let _system_program_info = &ix.accounts[9];
            let _stake_program_info = &ix.accounts[10];

            if let Some(mut stake_pools) = instruction.stake_pools.clone() {
                if let Some(alert_config) = stake_pools.get_mut(&stake_pool_info.pubkey.to_string())
                {
                    sort_thresholds(alert_config.thresholds.as_mut());
                    for threshold in alert_config.thresholds.iter() {
                        if *amount > threshold.value {
                            handler
                                .dispatch_platform_notifications(
                                    &threshold.notification.destinations,
                                    &threshold.notification.description,
                                    Some(*amount),
                                    Some("SOL"),
                                    &parser.transaction_signature,
                                )
                                .await?;
                            break;
                        }
                    }
                }
            }
        }
        SplStakePoolProgram::Initialize
        | SplStakePoolProgram::AddValidatorToPool
        | SplStakePoolProgram::RemoveValidatorFromPool
        | SplStakePoolProgram::DecreaseValidatorStake
        | SplStakePoolProgram::SetPreferredValidator
        | SplStakePoolProgram::UpdateValidatorListBalance
        | SplStakePoolProgram::UpdateStakePoolBalance
        | SplStakePoolProgram::CleanupRemovedValidatorEntries
        | SplStakePoolProgram::SetManager
        | SplStakePoolProgram::SetFee
        | SplStakePoolProgram::SetStaker
        | SplStakePoolProgram::SetFundingAuthority
        | SplStakePoolProgram::CreateTokenMetadata
        | SplStakePoolProgram::UpdateTokenMetadata
        | SplStakePoolProgram::IncreaseAdditionalValidatorStake
        | SplStakePoolProgram::DecreaseAdditionalValidatorStake
        | SplStakePoolProgram::Redelegate
        | SplStakePoolProgram::DepositStakeWithSlippage
        | SplStakePoolProgram::WithdrawStakeWithSlippage
        | SplStakePoolProgram::DepositSolWithSlippage
        | SplStakePoolProgram::WithdrawSolWithSlippage => {
            unreachable!()
        }
    }

    Ok(())
}
