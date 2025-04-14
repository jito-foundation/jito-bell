use solana_sdk::{pubkey::Pubkey, signature::Signature};
use stake_pool::SplStakePoolProgram;
use token_2022::SplToken2022Program;
use yellowstone_grpc_proto::geyser::SubscribeUpdateTransaction;

pub mod stake_pool;
pub mod token_2022;
pub mod vault;

#[derive(Debug)]
pub enum JitoBellProgram {
    SplStakePool(SplStakePoolProgram),
    SplToken2022(SplToken2022Program),
}

impl std::fmt::Display for JitoBellProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JitoBellProgram::SplStakePool(_) => write!(f, "spl_stake_pool"),
            JitoBellProgram::SplToken2022(_) => write!(f, "spl-token-2022"),
        }
    }
}

/// Parse Transaction
#[derive(Debug)]
pub struct JitoTransactionParser {
    /// Transaction signature
    pub transaction_signature: String,

    /// The array of programs related to Jito Network
    pub programs: Vec<JitoBellProgram>,
}

impl JitoTransactionParser {
    /// Initialize new parser
    pub fn new(transaction: SubscribeUpdateTransaction) -> Self {
        let mut transaction_signature = String::new();
        let mut programs = Vec::new();

        if let Some(tx) = transaction.transaction {
            if let Some(tx) = tx.transaction {
                let signature_slice = &tx.signatures[0];
                let mut slice = [0; 64];
                slice.copy_from_slice(&signature_slice[..64]);
                let tx_signature = Signature::from(slice);
                transaction_signature = tx_signature.to_string();

                if let Some(msg) = tx.message {
                    let pubkeys: Vec<Pubkey> = msg
                        .account_keys
                        .iter()
                        .map(|account_key| {
                            let mut slice = [0; 32];
                            slice.copy_from_slice(&account_key[..32]);
                            Pubkey::new_from_array(slice)
                        })
                        .collect();

                    for instruction in &msg.instructions {
                        let program_id = &pubkeys[instruction.program_id_index as usize];

                        match *program_id {
                            program_id if program_id.eq(&SplStakePoolProgram::program_id()) => {
                                if let Some(ix_info) =
                                    SplStakePoolProgram::parse_spl_stake_pool_program(
                                        instruction,
                                        &pubkeys,
                                    )
                                {
                                    programs.push(JitoBellProgram::SplStakePool(ix_info));
                                }
                            }
                            program_id if program_id.eq(&SplToken2022Program::program_id()) => {
                                if let Some(ix_info) =
                                    SplToken2022Program::parse_spl_token_2022_program(
                                        instruction,
                                        &pubkeys,
                                    )
                                {
                                    programs.push(JitoBellProgram::SplToken2022(ix_info));
                                }
                            }
                            _ => continue,
                        }
                    }
                }
            }
        }

        Self {
            transaction_signature,
            programs,
        }
    }
}
