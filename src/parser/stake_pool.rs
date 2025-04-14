use std::str::FromStr;

use ::borsh::BorshDeserialize;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    native_token::lamports_to_sol,
    pubkey::Pubkey,
};
use spl_stake_pool::instruction::StakePoolInstruction;
use yellowstone_grpc_proto::prelude::CompiledInstruction;

#[derive(Debug)]
pub enum SplStakePoolProgram {
    DepositStake {
        ix: Instruction,
    },
    WithdrawStake {
        ix: Instruction,
        minimum_lamports_out: f64,
    },
    DepositSol {
        ix: Instruction,
        amount: f64,
    },
    WithdrawSol {
        ix: Instruction,
        amount: f64,
    },
}

impl std::fmt::Display for SplStakePoolProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SplStakePoolProgram::DepositStake { ix: _ } => write!(f, "deposit_stake"),
            SplStakePoolProgram::WithdrawStake {
                ix: _,
                minimum_lamports_out: _,
            } => write!(f, "withdraw_stake"),
            SplStakePoolProgram::DepositSol { ix: _, amount: _ } => write!(f, "deposit_sol"),
            SplStakePoolProgram::WithdrawSol { ix: _, amount: _ } => write!(f, "withdraw_sol"),
        }
    }
}

impl SplStakePoolProgram {
    /// Retrieve Program ID of SPL Stake Pool Program
    pub fn program_id() -> Pubkey {
        Pubkey::from_str("SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy").unwrap()
    }

    /// Parse SPL Stake Pool program
    pub fn parse_spl_stake_pool_program(
        instruction: &CompiledInstruction,
        account_keys: &[Pubkey],
    ) -> Option<SplStakePoolProgram> {
        let stake_pool_ix = match StakePoolInstruction::try_from_slice(&instruction.data) {
            Ok(ix) => ix,
            Err(_) => return None,
        };

        match stake_pool_ix {
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
            _ => None,
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
    fn parse_deposit_stake_ix(
        instruction: &CompiledInstruction,
        account_keys: &[Pubkey],
    ) -> SplStakePoolProgram {
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

        for (index, account) in instruction.accounts.iter().enumerate() {
            account_metas[index].pubkey = account_keys[*account as usize];
        }

        let ix = Instruction {
            program_id: SplStakePoolProgram::program_id(),
            accounts: account_metas.to_vec(),
            data: instruction.data.clone(),
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
    fn parse_withdraw_stake_ix(
        instruction: &CompiledInstruction,
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

        for (index, account) in instruction.accounts.iter().enumerate() {
            account_metas[index].pubkey = account_keys[*account as usize];
        }

        let ix = Instruction {
            program_id: SplStakePoolProgram::program_id(),
            accounts: account_metas.to_vec(),
            data: instruction.data.clone(),
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
    fn parse_deposit_sol_ix(
        instruction: &CompiledInstruction,
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

        for (index, account) in instruction.accounts.iter().enumerate() {
            account_metas[index].pubkey = account_keys[*account as usize];
        }

        let ix = Instruction {
            program_id: SplStakePoolProgram::program_id(),
            accounts: account_metas.to_vec(),
            data: instruction.data.clone(),
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
    fn parse_withdraw_sol_ix(
        instruction: &CompiledInstruction,
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

        for (index, account) in instruction.accounts.iter().enumerate() {
            account_metas[index].pubkey = account_keys[*account as usize];
        }

        let ix = Instruction {
            program_id: SplStakePoolProgram::program_id(),
            accounts: account_metas.to_vec(),
            data: instruction.data.clone(),
        };

        SplStakePoolProgram::WithdrawSol {
            ix,
            amount: lamports_to_sol(amount),
        }
    }
}
