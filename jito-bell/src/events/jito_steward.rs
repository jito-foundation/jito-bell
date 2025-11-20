use borsh::BorshDeserialize;
use solana_pubkey::Pubkey;

#[derive(Debug, Clone, BorshDeserialize)]
pub struct AutoRemoveValidatorEvent {
    pub validator_list_index: u64,
    pub vote_account: Pubkey,
    pub vote_account_closed: bool,
    pub stake_account_deactivated: bool,
    pub marked_for_immediate_removal: bool,
}

impl AutoRemoveValidatorEvent {
    pub const DISCRIMINATOR: [u8; 8] = [211, 46, 52, 163, 17, 38, 197, 186];
}

#[derive(Debug, Clone, BorshDeserialize)]
pub struct AutoAddValidatorEvent {
    pub validator_list_index: u64,
    pub vote_account: Pubkey,
}

impl AutoAddValidatorEvent {
    pub const DISCRIMINATOR: [u8; 8] = [123, 65, 239, 15, 82, 216, 206, 28];
}

#[derive(Debug, Clone, BorshDeserialize)]
pub struct EpochMaintenanceEvent {
    pub validator_index_to_remove: Option<u64>,
    pub validator_list_length: u64,
    pub num_pool_validators: u64,
    pub validators_to_remove: u64,
    pub validators_to_add: u64,
    pub maintenance_complete: bool,
}

impl EpochMaintenanceEvent {
    pub const DISCRIMINATOR: [u8; 8] = [255, 149, 70, 161, 199, 176, 9, 42];
}

#[derive(Debug, Clone, BorshDeserialize)]
pub struct StateTransition {
    pub epoch: u64,
    pub slot: u64,
    pub previous_state: String,
    pub new_state: String,
}

impl StateTransition {
    pub const DISCRIMINATOR: [u8; 8] = [106, 9, 120, 247, 169, 106, 206, 233];
}

#[derive(Default, Debug, Clone, PartialEq, Eq, BorshDeserialize)]
pub struct DecreaseComponents {
    pub scoring_unstake_lamports: u64,
    pub instant_unstake_lamports: u64,
    pub stake_deposit_unstake_lamports: u64,
    pub total_unstake_lamports: u64,
}

impl DecreaseComponents {
    pub const DISCRIMINATOR: [u8; 8] = [129, 8, 124, 12, 11, 140, 0, 8];
}

#[derive(Debug, Clone, BorshDeserialize)]
pub struct RebalanceEvent {
    pub vote_account: Pubkey,
    pub epoch: u16,
    pub rebalance_type_tag: RebalanceTypeTag,
    pub increase_lamports: u64,
    pub decrease_components: DecreaseComponents,
}

impl RebalanceEvent {
    pub const DISCRIMINATOR: [u8; 8] = [120, 27, 117, 235, 104, 42, 132, 75];
}

#[derive(Debug, Clone, BorshDeserialize)]
pub enum RebalanceTypeTag {
    None,
    Increase,
    Decrease,
}

#[derive(Debug, Clone, BorshDeserialize)]
pub struct DirectedRebalanceEvent {
    pub vote_account: Pubkey,
    pub epoch: u16,
    pub rebalance_type_tag: RebalanceTypeTag,
    pub increase_lamports: u64,
    pub decrease_lamports: u64,
}

impl DirectedRebalanceEvent {
    pub const DISCRIMINATOR: [u8; 8] = [187, 63, 59, 72, 191, 64, 113, 29];
}

/// Deprecated: This struct is no longer emitted but is kept to allow parsing of old events.
/// Because the event discriminator is based on struct name, it's important to rename the struct if
/// fields are changed.
#[derive(Debug, Clone, PartialEq, BorshDeserialize)]
pub struct ScoreComponents {
    pub score: f64,
    pub yield_score: f64,
    pub mev_commission_score: f64,
    pub blacklisted_score: f64,
    pub superminority_score: f64,
    pub delinquency_score: f64,
    pub running_jito_score: f64,
    pub commission_score: f64,
    pub historical_commission_score: f64,
    pub vote_credits_ratio: f64,
    pub vote_account: Pubkey,
    pub epoch: u16,
}

impl ScoreComponents {
    pub const DISCRIMINATOR: [u8; 8] = [218, 204, 53, 7, 22, 2, 217, 251];
}

/// Deprecated: This struct is no longer emitted but is kept to allow parsing of old events.
/// Because the event discriminator is based on struct name, it's important to rename the struct if
/// fields are changed.
#[derive(Debug, Clone, PartialEq, Eq, BorshDeserialize)]
pub struct InstantUnstakeComponents {
    pub instant_unstake: bool,
    pub delinquency_check: bool,
    pub commission_check: bool,
    pub mev_commission_check: bool,
    pub is_blacklisted: bool,
    pub vote_account: Pubkey,
    pub epoch: u16,
}

impl InstantUnstakeComponents {
    pub const DISCRIMINATOR: [u8; 8] = [217, 80, 196, 114, 226, 11, 127, 77];
}
