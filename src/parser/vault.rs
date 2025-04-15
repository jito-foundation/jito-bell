use std::str::FromStr;

use borsh::BorshDeserialize;
use jito_vault_sdk::instruction::VaultInstruction;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    native_token::lamports_to_sol,
    pubkey::Pubkey,
};
use yellowstone_grpc_proto::prelude::CompiledInstruction;

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

impl JitoVaultProgram {
    pub fn program_id() -> Pubkey {
        Pubkey::from_str("Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8").unwrap()
    }

    /// Parse Jito Vault Program
    pub fn parse_jito_vault_program(
        instruction: &CompiledInstruction,
        account_keys: &[Pubkey],
    ) -> Option<JitoVaultProgram> {
        let vault_ix = match VaultInstruction::try_from_slice(&instruction.data) {
            Ok(ix) => ix,
            Err(_) => return None,
        };

        match vault_ix {
            VaultInstruction::MintTo {
                amount_in: _,
                min_amount_out,
            } => Some(Self::parse_mint_to_ix(
                instruction,
                account_keys,
                min_amount_out,
            )),
            VaultInstruction::EnqueueWithdrawal { amount } => Some(
                Self::parse_enqueue_withdrawal_ix(instruction, account_keys, amount),
            ),
            _ => None,
        }
    }

    // #[account(0, name = "config")]
    // #[account(1, writable, name = "vault")]
    // #[account(2, writable, name = "vrt_mint")]
    // #[account(3, writable, signer, name = "depositor")]
    // #[account(4, writable, name = "depositor_token_account")]
    // #[account(5, writable, name = "vault_token_account")]
    // #[account(6, writable, name = "depositor_vrt_token_account")]
    // #[account(7, writable, name = "vault_fee_token_account")]
    // #[account(8, name = "token_program")]
    // #[account(9, signer, optional, name = "mint_signer", description = "Signer for minting")]
    pub fn parse_mint_to_ix(
        instruction: &CompiledInstruction,
        account_keys: &[Pubkey],
        min_amount_out: u64,
    ) -> Self {
        let mut account_metas = [
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), true),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), true),
        ];

        for (index, account) in instruction.accounts.iter().enumerate() {
            account_metas[index].pubkey = account_keys[*account as usize];
        }

        let ix = Instruction {
            program_id: Self::program_id(),
            accounts: account_metas.to_vec(),
            data: instruction.data.clone(),
        };

        Self::MintTo {
            ix,
            min_amount_out: lamports_to_sol(min_amount_out) as u64,
        }
    }

    pub fn parse_enqueue_withdrawal_ix(
        instruction: &CompiledInstruction,
        account_keys: &[Pubkey],
        amount: u64,
    ) -> Self {
        let mut account_metas = [
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), true),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), true),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
        ];

        for (index, account) in instruction.accounts.iter().enumerate() {
            account_metas[index].pubkey = account_keys[*account as usize];
        }

        let ix = Instruction {
            program_id: Self::program_id(),
            accounts: account_metas.to_vec(),
            data: instruction.data.clone(),
        };

        Self::EnqueueWithdrawal {
            ix,
            amount: lamports_to_sol(amount) as u64,
        }
    }
}
