use std::str::FromStr;

use solana_sdk::pubkey::Pubkey;

/// SPL Stake Pool Program
#[derive(Debug, PartialEq)]
pub enum JitoStewardInstruction {
    InitializeSteward,
    ReallocState,
    AutoAddValidatorToPool,
    AutoRemoveValidatorFromPool,
    InstantRemoveValidator,
    EpochMaintainance,
    ComputeScore,
    ComputeDelegations,
    Idle,
    ComputeInstantUnstake,
    Rebalance,
    SetNewAuthority,
    PauseSteward,
    ResumeSteward,
    AddValidatorsToBlacklist,
    RemoveValidatosFromBlacklist,
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
}

impl std::fmt::Display for JitoStewardInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JitoStewardInstruction::InitializeSteward => write!(f, "initiallize_steward"),
            JitoStewardInstruction::ReallocState => write!(f, "realloc_state"),
            JitoStewardInstruction::AutoAddValidatorToPool => write!(f, "auto_add_validator_pool"),
            JitoStewardInstruction::AutoRemoveValidatorFromPool => {
                write!(f, "auto_remove_validator_from_pool")
            }
            JitoStewardInstruction::InstantRemoveValidator => write!(f, "instant_remove_validator"),
            JitoStewardInstruction::EpochMaintainance => write!(f, "epoch_maintainance"),
            JitoStewardInstruction::ComputeScore => write!(f, "compute_score"),
            JitoStewardInstruction::ComputeDelegations => write!(f, "coompute_delegations"),
            JitoStewardInstruction::Idle => write!(f, "idle"),
            JitoStewardInstruction::ComputeInstantUnstake => write!(f, "compute_instant_unstake"),
            JitoStewardInstruction::Rebalance => write!(f, "rebalance"),
            JitoStewardInstruction::SetNewAuthority => write!(f, "set_new_authority"),
            JitoStewardInstruction::PauseSteward => write!(f, "pause_steward"),
            JitoStewardInstruction::ResumeSteward => write!(f, "resume_steward"),
            JitoStewardInstruction::AddValidatorsToBlacklist => {
                write!(f, "add_validators_to_blacklist")
            }
            JitoStewardInstruction::RemoveValidatosFromBlacklist => {
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
        }
    }
}

impl JitoStewardInstruction {
    /// Retrieve Program ID of SPL Stake Pool Program
    pub fn program_id() -> Pubkey {
        Pubkey::from_str("Stewardf95sJbmtcZsyagb2dg4Mo8eVQho8gpECvLx8").unwrap()
    }
}
