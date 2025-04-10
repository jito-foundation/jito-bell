use std::collections::HashMap;

use solana_sdk::pubkey::Pubkey;
use stake_pool::JitoStakePool;
use yellowstone_grpc_proto::{
    geyser::{SubscribeUpdateTransaction, SubscribeUpdateTransactionInfo},
    prelude::CompiledInstruction,
};

mod stake_pool;

pub enum JitoInstruction {
    JitoStakePool(JitoStakePool),
}

/// Parse Transaction
pub struct JitoTransactionParser {
    /// Transaction Hash
    pub transaction_hash: String,
    // Instructions
    // pub instructions: Vec<JitoInstruction>,
}

impl JitoTransactionParser {
    /// Construct new parser
    ///
    pub fn new(transaction: SubscribeUpdateTransaction) -> Self {
        let mut transaction_hash = String::new();

        if let Some(tx) = transaction.transaction {
            if let Some(tx) = tx.transaction {
                if let Some(msg) = tx.message {
                    // instructions.extend(msg.instructions);

                    for instruction in &msg.instructions {
                        // instruction.
                    }
                }
            }
        }

        Self { transaction_hash }
    }
}
