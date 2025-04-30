use std::str::FromStr;

use ::borsh::BorshDeserialize;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    native_token::lamports_to_sol,
    pubkey::Pubkey,
};
use spl_stake_pool::instruction::StakePoolInstruction;

use super::instruction::ParsableInstruction;

/// SPL Stake Pool Program
#[derive(Debug, PartialEq)]
pub enum SplStakePoolProgram {
    Initialize,
    AddValidatorToPool,
    RemoveValidatorFromPool,
    DecreaseValidatorStake,
    IncreaseValidatorStake {
        ix: Instruction,
        amount: f64,
    },
    SetPreferredValidator,
    UpdateValidatorListBalance,
    UpdateStakePoolBalance,
    CleanupRemovedValidatorEntries,
    DepositStake {
        ix: Instruction,
    },
    WithdrawStake {
        ix: Instruction,
        minimum_lamports_out: f64,
    },
    SetManager,
    SetFee,
    SetStaker,
    DepositSol {
        ix: Instruction,
        amount: f64,
    },
    SetFundingAuthority,
    WithdrawSol {
        ix: Instruction,
        amount: f64,
    },
    CreateTokenMetadata,
    UpdateTokenMetadata,
    IncreaseAdditionalValidatorStake,
    DecreaseAdditionalValidatorStake,
    DecreaseValidatorStakeWithReserve {
        ix: Instruction,
        amount: f64,
    },
    Redelegate,
    DepositStakeWithSlippage,
    WithdrawStakeWithSlippage,
    DepositSolWithSlippage,
    WithdrawSolWithSlippage,
}

impl std::fmt::Display for SplStakePoolProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SplStakePoolProgram::Initialize => write!(f, "initialize"),
            SplStakePoolProgram::AddValidatorToPool => write!(f, "add_validator_pool"),
            SplStakePoolProgram::RemoveValidatorFromPool => write!(f, "remove_validator_from_pool"),
            SplStakePoolProgram::DecreaseValidatorStake => write!(f, "decrease_validator_stake"),
            SplStakePoolProgram::IncreaseValidatorStake { ix: _, amount: _ } => {
                write!(f, "increase_validator_stake")
            }
            SplStakePoolProgram::SetPreferredValidator => write!(f, "set_preferred_validator"),
            SplStakePoolProgram::UpdateValidatorListBalance => {
                write!(f, "update_validator_list_balance")
            }
            SplStakePoolProgram::UpdateStakePoolBalance => write!(f, "update_stake_pool_balance"),
            SplStakePoolProgram::CleanupRemovedValidatorEntries => {
                write!(f, "cleanup_removed_validator_entries")
            }
            SplStakePoolProgram::DepositStake { ix: _ } => write!(f, "deposit_stake"),
            SplStakePoolProgram::WithdrawStake {
                ix: _,
                minimum_lamports_out: _,
            } => write!(f, "withdraw_stake"),
            SplStakePoolProgram::SetManager => write!(f, "set_manager"),
            SplStakePoolProgram::SetFee => write!(f, "set_fee"),
            SplStakePoolProgram::SetStaker => write!(f, "set_staker"),
            SplStakePoolProgram::DepositSol { ix: _, amount: _ } => write!(f, "deposit_sol"),
            SplStakePoolProgram::SetFundingAuthority => write!(f, "set_funding_authority"),
            SplStakePoolProgram::WithdrawSol { ix: _, amount: _ } => write!(f, "withdraw_sol"),
            SplStakePoolProgram::CreateTokenMetadata => write!(f, "create_token_metadata"),
            SplStakePoolProgram::UpdateTokenMetadata => write!(f, "update_token_metadata"),
            SplStakePoolProgram::IncreaseAdditionalValidatorStake => {
                write!(f, "increase_additional_validator_stake")
            }
            SplStakePoolProgram::DecreaseAdditionalValidatorStake => {
                write!(f, "decrease_additional_validator_stake")
            }
            SplStakePoolProgram::DecreaseValidatorStakeWithReserve { ix: _, amount: _ } => {
                write!(f, "decrease_validator_stake_with_reserve")
            }
            SplStakePoolProgram::Redelegate => write!(f, "redelegate"),
            SplStakePoolProgram::DepositStakeWithSlippage => {
                write!(f, "deposit_stake_with_slippage")
            }
            SplStakePoolProgram::WithdrawStakeWithSlippage => {
                write!(f, "withdraw_stake_with_slippage")
            }
            SplStakePoolProgram::DepositSolWithSlippage => write!(f, "deposit_sol_with_slippage"),
            SplStakePoolProgram::WithdrawSolWithSlippage => write!(f, "withdraw_sol_with_slippage"),
        }
    }
}

impl SplStakePoolProgram {
    /// Retrieve Program ID of SPL Stake Pool Program
    pub fn program_id() -> Pubkey {
        Pubkey::from_str("SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy").unwrap()
    }

    /// Parse SPL Stake Pool program
    pub fn parse_spl_stake_pool_program<T: ParsableInstruction>(
        instruction: &T,
        account_keys: &[Pubkey],
    ) -> Option<SplStakePoolProgram> {
        let stake_pool_ix = match StakePoolInstruction::try_from_slice(&instruction.data()) {
            Ok(ix) => ix,
            Err(_) => return None,
        };

        match stake_pool_ix {
            StakePoolInstruction::IncreaseValidatorStake {
                lamports,
                transient_stake_seed: _,
            } => Some(Self::parse_increase_validator_stake_ix(
                instruction,
                account_keys,
                lamports,
            )),
            StakePoolInstruction::DepositStake => {
                Some(Self::parse_deposit_stake_ix(instruction, account_keys))
            }
            StakePoolInstruction::WithdrawStake(amount) => Some(Self::parse_withdraw_stake_ix(
                instruction,
                account_keys,
                amount,
            )),
            StakePoolInstruction::DepositSol(amount) => Some(Self::parse_deposit_sol_ix(
                instruction,
                account_keys,
                amount,
            )),
            StakePoolInstruction::WithdrawSol(amount) => Some(Self::parse_withdraw_sol_ix(
                instruction,
                account_keys,
                amount,
            )),
            StakePoolInstruction::DecreaseValidatorStakeWithReserve {
                lamports,
                transient_stake_seed: _,
            } => Some(Self::parse_decrease_validator_stake_with_reserve_ix(
                instruction,
                account_keys,
                lamports,
            )),
            _ => None,
        }
    }

    /// Parse Increase Validator Stake Instruction
    /// https://github.com/solana-labs/solana-program-library/blob/b7dd8fee93815b486fce98d3d43d1d0934980226/stake-pool/program/src/instruction.rs#L163-L199
    ///
    ///  0. `[]` Stake pool
    ///  1. `[s]` Stake pool staker
    ///  2. `[]` Stake pool withdraw authority
    ///  3. `[w]` Validator list
    ///  4. `[w]` Stake pool reserve stake
    ///  5. `[w]` Transient stake account
    ///  6. `[]` Validator stake account
    ///  7. `[]` Validator vote account to delegate to
    ///  8. '[]' Clock sysvar
    ///  9. '[]' Rent sysvar
    /// 10. `[]` Stake History sysvar
    /// 11. `[]` Stake Config sysvar
    /// 12. `[]` System program
    /// 13. `[]` Stake program
    fn parse_increase_validator_stake_ix<T: ParsableInstruction>(
        instruction: &T,
        account_keys: &[Pubkey],
        lamports: u64,
    ) -> Self {
        // Initialize account_metas with default AccountMeta objects.
        // These will be replaced with actual values in the loop below.
        let mut account_metas = vec![
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), true),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
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
            program_id: SplStakePoolProgram::program_id(),
            accounts: account_metas.to_vec(),
            data: instruction.data().to_vec(),
        };

        SplStakePoolProgram::IncreaseValidatorStake {
            ix,
            amount: lamports_to_sol(lamports),
        }
    }

    /// Parse Deposit Stake Instruction
    /// https://github.com/solana-labs/solana-program-library/blob/b7dd8fee93815b486fce98d3d43d1d0934980226/stake-pool/program/src/instruction.rs#L271-L289
    ///
    ///   0. `[w]` Stake pool
    ///   1. `[w]` Validator stake list storage account
    ///   2. `[s]/[]` Stake pool deposit authority
    ///   3. `[]` Stake pool withdraw authority
    ///   4. `[w]` Stake account to join the pool (withdraw authority for the
    ///      stake account should be first set to the stake pool deposit
    ///      authority)
    ///   5. `[w]` Validator stake account for the stake account to be merged
    ///      with
    ///   6. `[w]` Reserve stake account, to withdraw rent exempt reserve
    ///   7. `[w]` User account to receive pool tokens
    ///   8. `[w]` Account to receive pool fee tokens
    ///   9. `[w]` Account to receive a portion of pool fee tokens as referral
    ///      fees
    ///   10. `[w]` Pool token mint account
    ///   11. '[]' Sysvar clock account
    ///   12. '[]' Sysvar stake history account
    ///   13. `[]` Pool token program id,
    ///   14. `[]` Stake program id,
    fn parse_deposit_stake_ix<T: ParsableInstruction>(
        instruction: &T,
        account_keys: &[Pubkey],
    ) -> Self {
        let mut account_metas = [
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), true),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
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
            program_id: SplStakePoolProgram::program_id(),
            accounts: account_metas.to_vec(),
            data: instruction.data().to_vec(),
        };

        SplStakePoolProgram::DepositStake { ix }
    }

    /// Parse Withdraw Stake Instruction
    /// https://github.com/solana-labs/solana-program-library/blob/b7dd8fee93815b486fce98d3d43d1d0934980226/stake-pool/program/src/instruction.rs#L313C1-L325C36
    ///
    ///   0. `[w]` Stake pool
    ///   1. `[w]` Validator stake list storage account
    ///   2. `[]` Stake pool withdraw authority
    ///   3. `[w]` Validator or reserve stake account to split
    ///   4. `[w]` Unitialized stake account to receive withdrawal
    ///   5. `[]` User account to set as a new withdraw authority
    ///   6. `[s]` User transfer authority, for pool token account
    ///   7. `[w]` User account with pool tokens to burn from
    ///   8. `[w]` Account to receive pool fee tokens
    ///   9. `[w]` Pool token mint account
    ///  10. `[]` Sysvar clock account (required)
    ///  11. `[]` Pool token program id
    ///  12. `[]` Stake program id,
    fn parse_withdraw_stake_ix<T: ParsableInstruction>(
        instruction: &T,
        account_keys: &[Pubkey],
        minimum_lamports_out: u64,
    ) -> SplStakePoolProgram {
        let mut account_metas = [
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), true),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
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
            program_id: SplStakePoolProgram::program_id(),
            accounts: account_metas.to_vec(),
            data: instruction.data().to_vec(),
        };

        SplStakePoolProgram::WithdrawStake {
            ix,
            minimum_lamports_out: lamports_to_sol(minimum_lamports_out),
        }
    }

    /// Parse Deposit SOL Instruction
    /// https://github.com/solana-labs/solana-program-library/blob/b7dd8fee93815b486fce98d3d43d1d0934980226/stake-pool/program/src/instruction.rs#L357C1-L367C64
    ///
    ///   0. `[w]` Stake pool
    ///   1. `[]` Stake pool withdraw authority
    ///   2. `[w]` Reserve stake account, to deposit SOL
    ///   3. `[s]` Account providing the lamports to be deposited into the pool
    ///   4. `[w]` User account to receive pool tokens
    ///   5. `[w]` Account to receive fee tokens
    ///   6. `[w]` Account to receive a portion of fee as referral fees
    ///   7. `[w]` Pool token mint account
    ///   8. `[]` System program account
    ///   9. `[]` Token program id
    ///  10. `[s]` (Optional) Stake pool sol deposit authority.
    fn parse_deposit_sol_ix<T: ParsableInstruction>(
        instruction: &T,
        account_keys: &[Pubkey],
        amount: u64,
    ) -> SplStakePoolProgram {
        let mut account_metas = [
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), true),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
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
            program_id: SplStakePoolProgram::program_id(),
            accounts: account_metas.to_vec(),
            data: instruction.data().to_vec(),
        };

        SplStakePoolProgram::DepositSol {
            ix,
            amount: lamports_to_sol(amount),
        }
    }

    /// Parse Withdraw SOL Instruction
    /// https://github.com/solana-labs/solana-program-library/blob/b7dd8fee93815b486fce98d3d43d1d0934980226/stake-pool/program/src/instruction.rs#L381C1-L394C64
    ///
    ///   0. `[w]` Stake pool
    ///   1. `[]` Stake pool withdraw authority
    ///   2. `[s]` User transfer authority, for pool token account
    ///   3. `[w]` User account to burn pool tokens
    ///   4. `[w]` Reserve stake account, to withdraw SOL
    ///   5. `[w]` Account receiving the lamports from the reserve, must be a
    ///      system account
    ///   6. `[w]` Account to receive pool fee tokens
    ///   7. `[w]` Pool token mint account
    ///   8. '[]' Clock sysvar
    ///   9. '[]' Stake history sysvar
    ///  10. `[]` Stake program account
    ///  11. `[]` Token program id
    ///  12. `[s]` (Optional) Stake pool sol withdraw authority
    fn parse_withdraw_sol_ix<T: ParsableInstruction>(
        instruction: &T,
        account_keys: &[Pubkey],
        amount: u64,
    ) -> SplStakePoolProgram {
        let mut account_metas = [
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), true),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
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
            program_id: SplStakePoolProgram::program_id(),
            accounts: account_metas.to_vec(),
            data: instruction.data().to_vec(),
        };

        SplStakePoolProgram::WithdrawSol {
            ix,
            amount: lamports_to_sol(amount),
        }
    }

    /// Parse Decrease Validator Stake With Reserve Instruction
    /// https://github.com/solana-labs/solana-program-library/blob/b7dd8fee93815b486fce98d3d43d1d0934980226/stake-pool/program/src/instruction.rs#L512-L541
    ///
    ///  0. `[]` Stake pool
    ///  1. `[s]` Stake pool staker
    ///  2. `[]` Stake pool withdraw authority
    ///  3. `[w]` Validator list
    ///  4. `[w]` Reserve stake account, to fund rent exempt reserve
    ///  5. `[w]` Canonical stake account to split from
    ///  6. `[w]` Transient stake account to receive split
    ///  7. `[]` Clock sysvar
    ///  8. '[]' Stake history sysvar
    ///  9. `[]` System program
    /// 10. `[]` Stake program
    fn parse_decrease_validator_stake_with_reserve_ix<T: ParsableInstruction>(
        instruction: &T,
        account_keys: &[Pubkey],
        lamports: u64,
    ) -> SplStakePoolProgram {
        let mut account_metas = [
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), true),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
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
            program_id: SplStakePoolProgram::program_id(),
            accounts: account_metas.to_vec(),
            data: instruction.data().to_vec(),
        };

        SplStakePoolProgram::DecreaseValidatorStakeWithReserve {
            ix,
            amount: lamports_to_sol(lamports),
        }
    }
}

#[cfg(test)]
mod tests {
    use solana_sdk::{
        native_token::lamports_to_sol, pubkey::Pubkey, signature::Keypair, signer::Signer,
    };
    use yellowstone_grpc_proto::prelude::CompiledInstruction;

    use crate::parser::stake_pool::SplStakePoolProgram;

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
    fn test_parse_increase_validator_stake() {
        let ix_number = 4;
        let num_account = 14;
        let lamports: u64 = 5_000_000_000; // 5 SOL
        let transient_stake_seed: u64 = 123;

        let account_keys = create_test_pubkeys(num_account);

        let mut data = vec![ix_number];
        data.extend_from_slice(&lamports.to_le_bytes());
        data.extend_from_slice(&transient_stake_seed.to_le_bytes());

        // Create account indices
        let accounts = (0..num_account).map(|i| i as u8).collect();

        let instruction = create_compiled_instruction(1, accounts, data);

        // Parse the instruction
        let parsed = SplStakePoolProgram::parse_spl_stake_pool_program(&instruction, &account_keys);

        // Validate result
        assert!(parsed.is_some());
        if let Some(SplStakePoolProgram::IncreaseValidatorStake { amount, .. }) = parsed {
            assert_eq!(amount, lamports_to_sol(lamports));
        } else {
            panic!("Expected IncreaseValidatorStake variant");
        }
    }

    #[test]
    fn test_parse_deposit_stake() {
        let ix_number = 9;
        let num_account = 15;

        let account_keys = create_test_pubkeys(num_account);

        let data = vec![ix_number];

        let accounts = (0..num_account).map(|i| i as u8).collect();

        let instruction = create_compiled_instruction(14, accounts, data);

        // Parse the instruction
        let parsed = SplStakePoolProgram::parse_spl_stake_pool_program(&instruction, &account_keys);

        // Validate result
        assert!(parsed.is_some());
        if let Some(SplStakePoolProgram::DepositStake { ix: _ }) = parsed {
        } else {
            panic!("Expected DepositStake variant");
        }
    }

    #[test]
    fn test_parse_withdraw_stake() {
        let ix_number = 10;
        let num_account = 13;
        let lamports: u64 = 5_000_000_000; // 5 SOL

        let account_keys = create_test_pubkeys(num_account);

        let mut data = vec![ix_number];
        data.extend_from_slice(&lamports.to_le_bytes());

        let accounts = (0..num_account).map(|i| i as u8).collect();

        let instruction = create_compiled_instruction(1, accounts, data);

        // Parse the instruction
        let parsed = SplStakePoolProgram::parse_spl_stake_pool_program(&instruction, &account_keys);

        // Validate result
        assert!(parsed.is_some());
        if let Some(SplStakePoolProgram::WithdrawStake {
            ix: _,
            minimum_lamports_out,
        }) = parsed
        {
            assert_eq!(minimum_lamports_out, lamports_to_sol(lamports));
        } else {
            panic!("Expected WithdrawStake variant");
        }
    }

    #[test]
    fn test_parse_deposit_sol() {
        let ix_number = 14;
        let num_account = 11;
        let lamports: u64 = 5_000_000_000; // 5 SOL

        let account_keys = create_test_pubkeys(num_account);

        let mut data = vec![ix_number];
        data.extend_from_slice(&lamports.to_le_bytes());

        let accounts = (0..num_account).map(|i| i as u8).collect();

        let instruction = create_compiled_instruction(1, accounts, data);

        // Parse the instruction
        let parsed = SplStakePoolProgram::parse_spl_stake_pool_program(&instruction, &account_keys);

        // Validate result
        assert!(parsed.is_some());
        if let Some(SplStakePoolProgram::DepositSol { ix: _, amount }) = parsed {
            assert_eq!(amount, lamports_to_sol(lamports));
        } else {
            panic!("Expected DepositSol variant");
        }
    }

    #[test]
    fn test_parse_withdraw_sol() {
        let ix_number = 16;
        let num_account = 13;
        let lamports: u64 = 5_000_000_000; // 5 SOL

        let account_keys = create_test_pubkeys(num_account);

        let mut data = vec![ix_number];
        data.extend_from_slice(&lamports.to_le_bytes());

        let accounts = (0..num_account).map(|i| i as u8).collect();

        let instruction = create_compiled_instruction(1, accounts, data);

        // Parse the instruction
        let parsed = SplStakePoolProgram::parse_spl_stake_pool_program(&instruction, &account_keys);

        // Validate result
        assert!(parsed.is_some());
        if let Some(SplStakePoolProgram::WithdrawSol { ix: _, amount }) = parsed {
            assert_eq!(amount, lamports_to_sol(lamports));
        } else {
            panic!("Expected WithdrawSol variant");
        }
    }

    #[test]
    fn test_parse_decrease_validator_stake_with_reserve() {
        let ix_number = 21;
        let num_account = 11;
        let lamports: u64 = 6_000_000_000; // 6 SOL
        let transient_stake_seed: u64 = 123;

        let account_keys = create_test_pubkeys(num_account);

        let mut data = vec![ix_number];
        data.extend_from_slice(&lamports.to_le_bytes());
        data.extend_from_slice(&transient_stake_seed.to_le_bytes());

        let accounts = (0..num_account).map(|i| i as u8).collect();

        let instruction = create_compiled_instruction(1, accounts, data);

        // Parse the instruction
        let parsed = SplStakePoolProgram::parse_spl_stake_pool_program(&instruction, &account_keys);

        // Validate result
        assert!(parsed.is_some());
        if let Some(SplStakePoolProgram::DecreaseValidatorStakeWithReserve { ix: _, amount }) =
            parsed
        {
            assert_eq!(amount, lamports_to_sol(lamports));
        } else {
            panic!("Expected DecreaseValidatorStakeWithReserve variant");
        }
    }
}
