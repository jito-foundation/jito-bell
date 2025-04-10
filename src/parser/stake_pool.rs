use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::{self, Pubkey},
};
use spl_stake_pool::solana_program::example_mocks::solana_sdk::instruction::account_meta;
use yellowstone_grpc_proto::prelude::CompiledInstruction;

pub enum JitoStakePool {
    DepositStake(Instruction),
    WithdrawStake(Instruction),
    DepositSol(Instruction),
    WithdrawSol(Instruction),
}

impl JitoStakePool {
    /// Parse Deposit Stake Instruction
    /// AccountMeta::new_readonly(*stake_pool_withdraw_authority, false),
    /// AccountMeta::new(*deposit_stake_address, false),
    /// AccountMeta::new(*validator_stake_account, false),
    /// AccountMeta::new(*reserve_stake_account, false),
    /// AccountMeta::new(*pool_tokens_to, false),
    /// AccountMeta::new(*manager_fee_account, false),
    /// AccountMeta::new(*referrer_pool_tokens_account, false),
    /// AccountMeta::new(*pool_mint, false),
    /// AccountMeta::new_readonly(sysvar::clock::id(), false),
    /// AccountMeta::new_readonly(sysvar::stake_history::id(), false),
    /// AccountMeta::new_readonly(*token_program_id, false),
    /// AccountMeta::new_readonly(stake::program::id(), false),
    pub fn parse_deposit_stake(
        instruction: &CompiledInstruction,
        account_keys: &[Pubkey],
    ) -> JitoStakePool {
        let mut account_metas = [
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

        JitoStakePool::DepositStake(ix)
    }
}
