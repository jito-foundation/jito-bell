use borsh::BorshDeserialize;
use jito_vault_sdk::instruction::VaultInstruction;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use super::instruction::ParsableInstruction;

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
        jito_vault_client::programs::JITO_VAULT_ID
    }

    /// Parse Jito Vault Program
    pub fn parse_jito_vault_program<T: ParsableInstruction>(
        instruction: &T,
        account_keys: &[Pubkey],
    ) -> Option<JitoVaultProgram> {
        let vault_ix = match VaultInstruction::try_from_slice(instruction.data()) {
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

    /// #[account(0, name = "config")]
    /// #[account(1, writable, name = "vault")]
    /// #[account(2, writable, name = "vrt_mint")]
    /// #[account(3, writable, signer, name = "depositor")]
    /// #[account(4, writable, name = "depositor_token_account")]
    /// #[account(5, writable, name = "vault_token_account")]
    /// #[account(6, writable, name = "depositor_vrt_token_account")]
    /// #[account(7, writable, name = "vault_fee_token_account")]
    /// #[account(8, name = "token_program")]
    /// #[account(9, signer, optional, name = "mint_signer", description = "Signer for minting")]
    pub fn parse_mint_to_ix<T: ParsableInstruction>(
        instruction: &T,
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

        Self::MintTo { ix, min_amount_out }
    }

    /// #[account(0, name = "config")]
    /// #[account(1, writable, name = "vault")]
    /// #[account(2, writable, name = "vault_staker_withdrawal_ticket")]
    /// #[account(3, writable, name = "vault_staker_withdrawal_ticket_token_account")]
    /// #[account(4, writable, signer, name = "staker")]
    /// #[account(5, writable, name = "staker_vrt_token_account")]
    /// #[account(6, signer, name = "base")]
    /// #[account(7, name = "token_program")]
    /// #[account(8, name = "system_program")]
    /// #[account(9, signer, optional, name = "burn_signer", description = "Signer for burning")]
    pub fn parse_enqueue_withdrawal_ix<T: ParsableInstruction>(
        instruction: &T,
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
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
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

        Self::EnqueueWithdrawal { ix, amount }
    }
}

#[cfg(test)]
mod tests {
    use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};
    use yellowstone_grpc_proto::prelude::CompiledInstruction;

    use crate::ix_parser::vault::JitoVaultProgram;

    fn create_test_pubkeys(count: usize) -> Vec<Pubkey> {
        (0..count).map(|_| Keypair::new().pubkey()).collect()
    }

    fn create_compiled_instruction(
        program_id_index: u32,
        accounts: Vec<u8>,
        data: Vec<u8>,
    ) -> CompiledInstruction {
        CompiledInstruction {
            program_id_index,
            accounts,
            data,
        }
    }

    #[test]
    fn test_mint_to() {
        let ix_number = 11;
        let num_account = 10;
        let amount_in: u64 = 5; // 5 SOL
        let min_amount_out: u64 = 5; // 5 SOL

        let account_keys = create_test_pubkeys(num_account);

        let mut data = vec![ix_number];
        data.extend_from_slice(&amount_in.to_le_bytes());
        data.extend_from_slice(&min_amount_out.to_le_bytes());

        // Create account indices
        let accounts = (0..num_account).map(|i| i as u8).collect();

        let instruction = create_compiled_instruction(1, accounts, data);

        // Parse the instruction
        let parsed = JitoVaultProgram::parse_jito_vault_program(&instruction, &account_keys);

        // Validate result
        assert!(parsed.is_some());
        if let Some(JitoVaultProgram::MintTo { min_amount_out, .. }) = parsed {
            assert_eq!(min_amount_out, 5); // Ensure parsed value matches expected value
        } else {
            panic!("Expected MintTo variant");
        }
    }

    #[test]
    fn test_enqueue_withdrawal() {
        let ix_number = 12;
        let num_account = 10;
        let amount: u64 = 5; // 5 SOL

        let account_keys = create_test_pubkeys(num_account);

        let mut data = vec![ix_number];
        data.extend_from_slice(&amount.to_le_bytes());

        // Create account indices
        let accounts = (0..num_account).map(|i| i as u8).collect();

        let instruction = create_compiled_instruction(1, accounts, data);

        // Parse the instruction
        let parsed = JitoVaultProgram::parse_jito_vault_program(&instruction, &account_keys);

        // Validate result
        assert!(parsed.is_some());
        if let Some(JitoVaultProgram::EnqueueWithdrawal { amount, .. }) = parsed {
            assert_eq!(amount, 5);
        } else {
            panic!("Expected MintTo variant");
        }
    }
}
