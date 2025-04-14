use solana_sdk::instruction::Instruction;

/// Jito Vault Program
#[derive(Debug)]
pub enum JitoVaultProgram {
    InitializeConfig,
    InitializeVault,
    InitializeVaultWithMint,
    InitializeVaultOperatorDelegation,
    InitializeVaultNcnTicket,
    InitializeVaultNcnSlasherOperatorTicket,
    InitializeVaultNcnSlasherTicket,
    WarmupVaultNcnTicket,
    CooldownVaultNcnTicket,
    WarmupVaultNcnSlasherTicket,
    CooldownVaultNcnSlasherTicket,
    MintTo {
        ix: Instruction,
        min_amount_out: u64,
    },
    EnqueueWithdrawal {
        ix: Instruction,
        amount: u64,
    },
    ChangeWithdrawalTicketOwner,
    BurnWithdrawalTicket,
    SetDepositCapacity,
    SetFees,
    SetProgramFee,
    SetProgramFeeWallet,
    SetIsPaused,
    DelegateTokenAccount,
    SetAdmin,
    SetSecondaryAdmin,
    AddDelegation,
    CooldownDelegation,
    UpdateVaultBalance,
    InitializeVaultUpdateStateTracker,
    CrankVaultUpdateStateTracker,
    CloseVaultUpdateStateTracker,
    CreateTokenMetadata,
    UpdateTokenMetadata,
    SetConfigAdmin,
}

impl std::fmt::Display for JitoVaultProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JitoVaultProgram::InitializeConfig => write!(f, "initialize_config"),
            JitoVaultProgram::InitializeVault => write!(f, "initialize_vault"),
            JitoVaultProgram::InitializeVaultWithMint => write!(f, "initialize_vault_with_mint"),
            JitoVaultProgram::InitializeVaultOperatorDelegation => {
                write!(f, "initialize_vault_operator_delegation")
            }
            JitoVaultProgram::InitializeVaultNcnTicket => write!(f, "initialize_vault_ncn_ticket"),
            JitoVaultProgram::InitializeVaultNcnSlasherOperatorTicket => {
                write!(f, "initialize_vault_ncn_slasher_operator_ticket")
            }
            JitoVaultProgram::InitializeVaultNcnSlasherTicket => {
                write!(f, "initialize_vault_ncn_slasher_ticket")
            }
            JitoVaultProgram::WarmupVaultNcnTicket => {
                write!(f, "warmup_vault_ncn_ticket")
            }
            JitoVaultProgram::CooldownVaultNcnTicket => {
                write!(f, "cooldown_vault_ncn_ticket")
            }

            JitoVaultProgram::WarmupVaultNcnSlasherTicket => {
                write!(f, "warmup_vault_ncn_slasher_ticket")
            }
            JitoVaultProgram::CooldownVaultNcnSlasherTicket => {
                write!(f, "cooldown_vault_ncn_slasher_ticket")
            }
            JitoVaultProgram::MintTo {
                ix: _,
                min_amount_out: _,
            } => {
                write!(f, "mint_to")
            }
            JitoVaultProgram::EnqueueWithdrawal { ix: _, amount: _ } => {
                write!(f, "enqueue_withdrawal")
            }
            JitoVaultProgram::ChangeWithdrawalTicketOwner => {
                write!(f, "change_withdrawal_ticket_owner")
            }
            JitoVaultProgram::BurnWithdrawalTicket => {
                write!(f, "burn_withdrawal_ticket")
            }
            JitoVaultProgram::SetDepositCapacity => {
                write!(f, "set_deposit_capacity")
            }
            JitoVaultProgram::SetFees => {
                write!(f, "set_fees")
            }
            JitoVaultProgram::SetProgramFee => {
                write!(f, "set_program_fee")
            }
            JitoVaultProgram::SetProgramFeeWallet => {
                write!(f, "set_program_fee_wallet")
            }
            JitoVaultProgram::SetIsPaused => {
                write!(f, "set_is_paused")
            }
            JitoVaultProgram::DelegateTokenAccount => {
                write!(f, "delegate_token_account")
            }
            JitoVaultProgram::SetAdmin => {
                write!(f, "set_admin")
            }
            JitoVaultProgram::SetSecondaryAdmin => {
                write!(f, "set_secondary_admin")
            }
            JitoVaultProgram::AddDelegation => {
                write!(f, "add_delegation")
            }
            JitoVaultProgram::CooldownDelegation => {
                write!(f, "cooldown_delegation")
            }

            JitoVaultProgram::UpdateVaultBalance => {
                write!(f, "update_vault_balance")
            }
            JitoVaultProgram::InitializeVaultUpdateStateTracker => {
                write!(f, "initialize_vault_update_state_tracker")
            }
            JitoVaultProgram::CrankVaultUpdateStateTracker => {
                write!(f, "crank_vault_update_state_tracker")
            }
            JitoVaultProgram::CloseVaultUpdateStateTracker => {
                write!(f, "close_vault_update_state_tracker")
            }
            JitoVaultProgram::CreateTokenMetadata => {
                write!(f, "create_token_metadata")
            }
            JitoVaultProgram::UpdateTokenMetadata => {
                write!(f, "update_token_metadata")
            }
            JitoVaultProgram::SetConfigAdmin => {
                write!(f, "set_config_admin")
            }
        }
    }
}
