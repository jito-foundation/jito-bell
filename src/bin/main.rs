use std::{collections::HashMap, env, path::PathBuf};

use anyhow::Context;
use clap::{Parser, ValueEnum};
use futures::{sink::SinkExt, stream::StreamExt};
use jito_bell::{parser::JitoTransactionParser, JitoBellHandler};
use log::{error, info};
use maplit::hashmap;
use tonic::transport::channel::ClientTlsConfig;
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::prelude::{
    subscribe_update::UpdateOneof, CommitmentLevel, SubscribeRequest,
    SubscribeRequestFilterTransactions,
};

#[derive(Debug, Clone, Parser)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long, default_value_t = String::from("http://127.0.0.1:10000"))]
    /// Service endpoint
    endpoint: String,

    #[clap(long)]
    x_token: Option<String>,

    /// Commitment level: processed, confirmed or finalized
    #[clap(long)]
    commitment: Option<ArgsCommitment>,

    /// Filter vote transactions
    #[clap(long)]
    vote: Option<bool>,

    /// Filter failed transactions
    #[clap(long)]
    failed: Option<bool>,

    /// Filter by transaction signature
    #[clap(long)]
    signature: Option<String>,

    /// Filter included account in transactions
    #[clap(long)]
    account_include: Vec<String>,

    /// Filter excluded account in transactions
    #[clap(long)]
    account_exclude: Vec<String>,

    /// Filter required account in transactions
    #[clap(long)]
    account_required: Vec<String>,

    #[clap(long)]
    config_file: PathBuf,
}

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
enum ArgsCommitment {
    #[default]
    Processed,
    Confirmed,
    Finalized,
}

impl From<ArgsCommitment> for CommitmentLevel {
    fn from(commitment: ArgsCommitment) -> Self {
        match commitment {
            ArgsCommitment::Processed => CommitmentLevel::Processed,
            ArgsCommitment::Confirmed => CommitmentLevel::Confirmed,
            ArgsCommitment::Finalized => CommitmentLevel::Finalized,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env::set_var(
        env_logger::DEFAULT_FILTER_ENV,
        env::var_os(env_logger::DEFAULT_FILTER_ENV).unwrap_or_else(|| "info".into()),
    );
    env_logger::init();

    let args = Args::parse();
    let handler = JitoBellHandler::new(args.config_file);

    let mut client = GeyserGrpcClient::build_from_shared(args.endpoint)?
        .x_token(args.x_token)?
        .tls_config(ClientTlsConfig::new().with_native_roots())?
        .connect()
        .await?;
    let (mut subscribe_tx, mut stream) = client.subscribe().await?;

    let commitment: CommitmentLevel = args.commitment.unwrap_or_default().into();
    subscribe_tx
        .send(SubscribeRequest {
            slots: HashMap::new(),
            accounts: HashMap::new(),
            transactions: hashmap! { "".to_owned() => SubscribeRequestFilterTransactions {
                vote: args.vote,
                failed: args.failed,
                signature: args.signature.clone(),
                account_include: args.account_include,
                account_exclude: args.account_exclude,
                account_required: args.account_required,
            } },
            transactions_status: HashMap::new(),
            entry: HashMap::new(),
            blocks: HashMap::new(),
            blocks_meta: HashMap::new(),
            commitment: Some(commitment as i32),
            accounts_data_slice: vec![],
            ping: None,
            from_slot: None,
        })
        .await?;

    while let Some(message) = stream.next().await {
        match message {
            Ok(msg) => {
                if let Some(UpdateOneof::Transaction(transaction)) = msg.update_oneof {
                    let parser = JitoTransactionParser::new(transaction);

                    info!("Instruction: {:?}", parser.programs);

                    handler
                        .send_notification(&parser)
                        .await
                        .map_err(|e| anyhow::anyhow!(e))?;
                    // let sig = Signature::try_from(tx.signature.as_slice())
                    //     .expect("valid signature from transaction")
                    //     .to_string();
                    // if let Some(timestamp) = entry.0 {
                    //     info!("received txn {} at {}", sig, timestamp);
                    // } else {
                    //     entry.1.push(sig);
                    // }
                    // Some(UpdateOneof::TransactionStatus(tx)) => {
                    //     let entry = messages.entry(tx.slot).or_default();
                    //     tx.
                    //     let sig = Signature::try_from(tx.signature.as_slice())
                    //         .expect("valid signature from transaction")
                    //         .to_string();
                    //     if let Some(timestamp) = entry.0 {
                    //         info!("received txn {} at {}", sig, timestamp);
                    //     } else {
                    //         entry.1.push(sig);
                    //     }
                    // }
                    // Some(UpdateOneof::BlockMeta(block)) => {
                    //     let entry = messages.entry(block.slot).or_default();
                    //     entry.0 = block.block_time.map(|obj| {
                    //         DateTime::from_timestamp(obj.timestamp, 0)
                    //             .expect("invalid or out-of-range datetime")
                    //     });
                    //     if let Some(timestamp) = entry.0 {
                    //         for sig in &entry.1 {
                    //             info!("received txn {} at {}", sig, timestamp);
                    //         }
                    //     }

                    //     // remove outdated
                    //     while let Some(slot) = messages.keys().next().cloned() {
                    //         if slot < block.slot - 20 {
                    //             messages.remove(&slot);
                    //         } else {
                    //             break;
                    //         }
                    //     }
                    // }
                }
            }
            Err(error) => {
                error!("stream error: {error:?}");
                break;
            }
        }
    }

    Ok(())
}
