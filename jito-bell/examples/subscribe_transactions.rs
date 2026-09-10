//! Subscribe to Yellowstone gRPC and print every transaction that touches one
//! of the programs jito-bell watches.
//!
//! Useful as a live check after a dependency bump: if transactions still parse
//! into recognised instructions here, the ingest path works end to end.
//!
//! ```sh
//! cargo run --example subscribe_transactions -- \
//!     --yellowstone-url "$YELLOWSTONE_URL" \
//!     --x-token "$X_TOKEN" \
//!     --commitment confirmed
//! ```

use std::collections::HashMap;

use anyhow::Context;
use clap::Parser;
use futures::stream::StreamExt;
use jito_bell::{
    cli_args::ArgsCommitment,
    ix_parser::{
        jito_steward::JitoStewardInstruction, squads_v3::SquadsV3Program,
        squads_v4::SquadsV4Program, stake_pool::SplStakePoolProgram,
        token_2022::SplToken2022Program, vault::JitoVaultProgram,
    },
    tx_parser::JitoTransactionParser,
};
use yellowstone_grpc_client::{ClientTlsConfig, GeyserGrpcClient};
use yellowstone_grpc_proto::{
    geyser::CommitmentLevel,
    prelude::{
        subscribe_update::UpdateOneof, SubscribeRequest, SubscribeRequestFilterTransactions,
    },
};

#[derive(Debug, Parser)]
#[clap(about = "Print transactions touching a program jito-bell watches")]
struct Args {
    /// Yellowstone gRPC endpoint URL
    #[clap(long, env = "YELLOWSTONE_URL", hide_env_values = true)]
    yellowstone_url: String,

    /// X-Token for the endpoint
    #[clap(long, env = "X_TOKEN", hide_env_values = true)]
    x_token: Option<String>,

    /// Commitment level: processed, confirmed or finalized
    #[clap(long, value_enum, default_value = "confirmed")]
    commitment: ArgsCommitment,

    /// Stop after printing this many matching transactions
    #[clap(long)]
    limit: Option<u64>,
}

/// The programs `JitoTransactionParser` knows how to decode. Passing them as
/// `account_include` keeps the server from sending traffic that could never match.
fn watched_programs() -> Vec<String> {
    [
        JitoStewardInstruction::program_id(),
        JitoVaultProgram::program_id(),
        SplStakePoolProgram::program_id(),
        SplToken2022Program::program_id(),
        SquadsV3Program::program_id(),
        SquadsV4Program::program_id(),
    ]
    .iter()
    .map(ToString::to_string)
    .collect()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let args = Args::parse();

    let mut client = GeyserGrpcClient::build_from_shared(args.yellowstone_url.clone())
        .context("invalid yellowstone url")?
        .x_token(args.x_token.clone())
        .context("invalid x-token")?
        .tls_config(ClientTlsConfig::new().with_native_roots())
        .context("failed to build tls config")?
        .connect()
        .await
        .context("failed to connect to yellowstone")?;

    let programs = watched_programs();
    println!(
        "watching {} programs at {:?} commitment",
        programs.len(),
        args.commitment
    );

    let request = SubscribeRequest {
        transactions: HashMap::from([(
            "txs".to_owned(),
            SubscribeRequestFilterTransactions {
                vote: Some(false),
                failed: Some(false),
                signature: None,
                account_include: programs,
                account_exclude: Vec::new(),
                account_required: Vec::new(),
                cuckoo_account_include: None,
                token_accounts: None,
            },
        )]),
        commitment: Some(CommitmentLevel::from(args.commitment) as i32),
        ..Default::default()
    };

    let mut stream = client
        .subscribe_once(request)
        .await
        .context("failed to subscribe")?;
    let mut printed = 0u64;

    while let Some(update) = stream.next().await {
        let update = update.context("stream error")?;
        let Some(UpdateOneof::Transaction(tx)) = update.update_oneof else {
            continue;
        };

        let parsed = JitoTransactionParser::new(tx);

        // The filter only narrows by account, so most transactions still carry
        // nothing the parser recognises. Only print the ones that matched.
        if parsed.instructions.is_empty() && parsed.events.is_empty() {
            continue;
        }

        println!("\ntx {}", parsed.transaction_signature);
        for instruction in &parsed.instructions {
            println!("  instruction [{instruction}] {instruction:?}");
        }
        for event in &parsed.events {
            println!("  event {event:?}");
        }

        printed += 1;
        if args.limit.is_some_and(|limit| printed >= limit) {
            break;
        }
    }

    Ok(())
}
