use std::str::FromStr;

use solana_pubkey::Pubkey;
use solana_sdk::instruction::{AccountMeta, Instruction};

use crate::ix_parser::instruction::ParsableInstruction;

/// Jito Steward Instructions
#[derive(Debug, PartialEq)]
pub enum JitoStewardInstruction {
    InitializeSteward,
    ReallocState,
    AutoAddValidatorToPool,
    AutoRemoveValidatorFromPool,
    InstantRemoveValidator,
    EpochMaintenance,
    ComputeScore,
    ComputeDelegations,
    Idle,
    ComputeInstantUnstake,
    Rebalance,
    SetNewAuthority,
    PauseSteward,
    ResumeSteward,
    AddValidatorsToBlacklist,
    RemoveValidatorsFromBlacklist,
    UpdateParameters,
    ResetStewardState,
    AdminMarkForRemoval,
    ResetValidatorLamportBalances,
    CloseStewardAccounts,
    MigrateStateToV2,
    SetStaker,
    AddValidatorToPool,
    RemoveValidatorFromPool,
    SetPreferredValidator,
    IncreaseValidatorStake,
    DecreaseValidatorStake,
    IncreaseAdditionalValidatorStake,
    DecreaseAdditionalValidatorStake,
    UpdatePriorityFeeParameters,
    CopyDirectedStakeTargets {
        ix: Instruction,
        vote_pubkey: Pubkey,
        total_target_lamports: u64,
        validator_list_index: u32,
    },
}

impl std::fmt::Display for JitoStewardInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JitoStewardInstruction::InitializeSteward => write!(f, "initialize_steward"),
            JitoStewardInstruction::ReallocState => write!(f, "realloc_state"),
            JitoStewardInstruction::AutoAddValidatorToPool => write!(f, "auto_add_validator_pool"),
            JitoStewardInstruction::AutoRemoveValidatorFromPool => {
                write!(f, "auto_remove_validator_from_pool")
            }
            JitoStewardInstruction::InstantRemoveValidator => write!(f, "instant_remove_validator"),
            JitoStewardInstruction::EpochMaintenance => write!(f, "epoch_maintenance"),
            JitoStewardInstruction::ComputeScore => write!(f, "compute_score"),
            JitoStewardInstruction::ComputeDelegations => write!(f, "compute_delegations"),
            JitoStewardInstruction::Idle => write!(f, "idle"),
            JitoStewardInstruction::ComputeInstantUnstake => write!(f, "compute_instant_unstake"),
            JitoStewardInstruction::Rebalance => write!(f, "rebalance"),
            JitoStewardInstruction::SetNewAuthority => write!(f, "set_new_authority"),
            JitoStewardInstruction::PauseSteward => write!(f, "pause_steward"),
            JitoStewardInstruction::ResumeSteward => write!(f, "resume_steward"),
            JitoStewardInstruction::AddValidatorsToBlacklist => {
                write!(f, "add_validators_to_blacklist")
            }
            JitoStewardInstruction::RemoveValidatorsFromBlacklist => {
                write!(f, "remove_validators_from_blacklist")
            }
            JitoStewardInstruction::UpdateParameters => write!(f, "update_parameters"),
            JitoStewardInstruction::ResetStewardState => write!(f, "reset_steward_state"),
            JitoStewardInstruction::AdminMarkForRemoval => write!(f, "admin_mark_for_removal"),
            JitoStewardInstruction::ResetValidatorLamportBalances => {
                write!(f, "reset_validator_lamports_balances")
            }
            JitoStewardInstruction::CloseStewardAccounts => write!(f, "close_steward_accounts"),
            JitoStewardInstruction::MigrateStateToV2 => write!(f, "migrate_state_to_v2"),
            JitoStewardInstruction::SetStaker => write!(f, "set_staker"),
            JitoStewardInstruction::AddValidatorToPool => write!(f, "add_validator_to_pool"),
            JitoStewardInstruction::RemoveValidatorFromPool => {
                write!(f, "remove_validator_from_pool")
            }
            JitoStewardInstruction::SetPreferredValidator => write!(f, "set_preferred_validator"),
            JitoStewardInstruction::IncreaseValidatorStake => write!(f, "increase_validator_stake"),
            JitoStewardInstruction::DecreaseValidatorStake => write!(f, "decrease_validator_stake"),
            JitoStewardInstruction::IncreaseAdditionalValidatorStake => {
                write!(f, "increase_additional_validator_stake")
            }
            JitoStewardInstruction::DecreaseAdditionalValidatorStake => {
                write!(f, "decrease_additional_validator_stake")
            }
            JitoStewardInstruction::UpdatePriorityFeeParameters => {
                write!(f, "update_priority_fee_parameters")
            }
            JitoStewardInstruction::CopyDirectedStakeTargets {
                ix: _,
                vote_pubkey: _,
                total_target_lamports: _,
                validator_list_index: _,
            } => {
                write!(f, "copy_directed_stake_targets")
            }
        }
    }
}

impl JitoStewardInstruction {
    /// Retrieve Program ID of Jito Steward Program
    pub fn program_id() -> Pubkey {
        Pubkey::from_str("Stewardf95sJbmtcZsyagb2dg4Mo8eVQho8gpECvLx8").unwrap()
    }

    /// Parse Jito Steward instruction
    pub fn parse<T: ParsableInstruction>(
        instruction: &T,
        account_keys: &[Pubkey],
    ) -> Option<JitoStewardInstruction> {
        let instruction_data = instruction.data();
        match instruction_data[0..8] {
            [135, 132, 9, 127, 189, 161, 14, 5] => {
                let vote_pubkey = {
                    let mut pubkey_array = [0; 32];
                    pubkey_array.copy_from_slice(&instruction_data[8..40]);
                    Pubkey::new_from_array(pubkey_array)
                };

                let total_target_lamports = {
                    let mut slice = [0; 8];
                    slice.copy_from_slice(&instruction_data[40..48]);
                    u64::from_le_bytes(slice)
                };

                let validator_list_index = {
                    let mut slice = [0; 4];
                    slice.copy_from_slice(&instruction_data[48..52]);
                    u32::from_le_bytes(slice)
                };

                Some(Self::parse_copy_directed_stake_targets_ix(
                    instruction,
                    account_keys,
                    vote_pubkey,
                    total_target_lamports,
                    validator_list_index,
                ))
            }
            _ => None,
        }
    }

    /// #[account(0, name = "config")]
    /// #[account(1, writable, name = "directed_stake_meta")]
    /// #[account(2, name = "clock")]
    /// #[account(3, writable, name = "validator_list")]
    /// #[account(4, writable, signer, name = "authority")]
    pub fn parse_copy_directed_stake_targets_ix<T: ParsableInstruction>(
        instruction: &T,
        account_keys: &[Pubkey],
        vote_pubkey: Pubkey,
        total_target_lamports: u64,
        validator_list_index: u32,
    ) -> Self {
        let mut account_metas = [
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), true),
        ];

        for (index, account) in instruction.accounts().iter().enumerate() {
            if let Some(account_meta) = account_metas.get_mut(index) {
                if let Some(account) = account_keys.get(*account as usize) {
                    account_meta.pubkey = *account;
                }
            }
        }

        let ix = Instruction {
            program_id: Self::program_id(),
            accounts: account_metas.to_vec(),
            data: instruction.data().to_vec(),
        };

        Self::CopyDirectedStakeTargets {
            ix,
            vote_pubkey,
            total_target_lamports,
            validator_list_index,
        }
    }
}

#[cfg(test)]
mod tests {
    use yellowstone_grpc_proto::prelude::CompiledInstruction;

    use crate::ix_parser::jito_steward::JitoStewardInstruction;

    #[test]
    fn test_parse_copy_directed_stake_targets() {
        let instruction = {
            let data =
             hex::decode(
                 "8784097fbda10e050595ae71c4811b808d99e14b3997c386dfb609da3a788b49bba5093fd17040ae000000000000000037000000"
             ).unwrap();
            CompiledInstruction {
                program_id_index: 0,
                accounts: vec![0],
                data,
            }
        };

        let account_keys = vec![];
        let jito_steward_instruction =
            JitoStewardInstruction::parse(&instruction, &account_keys).unwrap();

        match jito_steward_instruction {
            JitoStewardInstruction::CopyDirectedStakeTargets {
                ix: _,
                vote_pubkey: _,
                total_target_lamports: _,
                validator_list_index: _,
            } => {}
            _ => panic!("Wrong instruction"),
        }
    }
}
