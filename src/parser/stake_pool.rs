use ::borsh::BorshDeserialize;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use spl_stake_pool::{instruction::StakePoolInstruction, state::StakePool};
use yellowstone_grpc_proto::prelude::CompiledInstruction;

pub enum JitoStakePool {
    DepositStakeWithSlippage {
        ix: Instruction,
        minimum_pool_tokens_out: u64,
    },
    WithdrawStake(Instruction),
    DepositSol(Instruction),
    WithdrawSol(Instruction),
}

impl JitoStakePool {
    pub fn parse_jito_stake_pool_ix(
        instruction: &CompiledInstruction,
        account_keys: &[Pubkey],
    ) -> Option<JitoStakePool> {
        let stake_pool_ix = StakePoolInstruction::try_from_slice(&instruction.data).unwrap();

        match stake_pool_ix {
            StakePoolInstruction::DepositStakeWithSlippage {
                minimum_pool_tokens_out,
            } => Some(Self::parse_deposit_stake_with_slippage_ix(
                instruction,
                account_keys,
                minimum_pool_tokens_out,
            )),
            _ => None,
        }
    }
    /// Parse Deposit Stake Instruction
    /// https://github.com/solana-program/stake-pool/blob/4ad88c05c567d47cbf4f3ea7e6cb765e15b336b9/program/src/instruction.rs#L1834C1-L1845C64
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
    pub fn parse_deposit_stake_with_slippage_ix(
        instruction: &CompiledInstruction,
        account_keys: &[Pubkey],
        minimum_pool_tokens_out: u64,
    ) -> JitoStakePool {
        let mut account_metas = [
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
        ];

        for (index, account) in instruction.accounts.iter().enumerate() {
            account_metas[index].pubkey = account_keys[*account as usize];
        }

        let ix = Instruction {
            program_id: Pubkey::new_unique(),
            accounts: account_metas.to_vec(),
            data: instruction.data.clone(),
        };

        JitoStakePool::DepositStakeWithSlippage {
            ix,
            minimum_pool_tokens_out,
        }
    }
}
