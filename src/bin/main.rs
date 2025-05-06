use std::{env, io::Write, path::PathBuf};

use clap::{Parser, ValueEnum};
use jito_bell::{multi_writer::MultiWriter, subscribe_option::SubscribeOption, JitoBellHandler};
use log::info;
use solana_sdk::commitment_config::CommitmentConfig;
use yellowstone_grpc_proto::geyser::CommitmentLevel;

#[derive(Debug, Clone, Parser)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long, env = "ENDPOINT")]
    /// Service endpoint
    endpoint: String,

    #[clap(long, env = "X_TOKEN")]
    x_token: Option<String>,

    /// Commitment level: processed, confirmed or finalized
    #[clap(long, env)]
    commitment: Option<ArgsCommitment>,

    /// Filter vote transactions
    #[clap(long, env)]
    vote: Option<bool>,

    /// Filter failed transactions
    #[clap(long, env)]
    failed: Option<bool>,

    /// Filter by transaction signature
    #[clap(long, env)]
    signature: Option<String>,

    /// Filter included account in transactions
    #[clap(long, env = "ACCOUNT_INCLUDE", value_delimiter = ',')]
    account_include: Vec<String>,

    /// Filter excluded account in transactions
    #[clap(long, env)]
    account_exclude: Vec<String>,

    /// Filter required account in transactions
    #[clap(long, env)]
    account_required: Vec<String>,

    #[clap(long, env = "CONFIG_FILE")]
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
    dotenv::dotenv().ok();

    let log_path =
        env::var("LOG_FILE_PATH").unwrap_or_else(|_| "/var/log/jito-bell/app.log".to_string());

    if let Some(dir) = std::path::Path::new(&log_path).parent() {
        std::fs::create_dir_all(dir)?;
    }

    env::set_var(
        env_logger::DEFAULT_FILTER_ENV,
        env::var_os(env_logger::DEFAULT_FILTER_ENV).unwrap_or_else(|| "info".into()),
    );

    let env = env_logger::Env::default();
    env_logger::Builder::from_env(env)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] {}: {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                record.args()
            )
        })
        .write_style(env_logger::WriteStyle::Always)
        .target(env_logger::Target::Pipe(Box::new(MultiWriter::new())))
        .init();

    let args = Args::parse();

    info!("Starting Jito Bell with endpoint: {}", args.endpoint);

    let commitment: CommitmentLevel = args.commitment.unwrap_or_default().into();
    let subscribe_option = SubscribeOption::new(
        args.endpoint.clone(),
        args.x_token,
        commitment,
        args.vote,
        args.failed,
        args.signature,
        args.account_include,
        args.account_exclude,
        args.account_required,
    );

    info!("Subscription configuration:\n{}", subscribe_option);

    let commitment = CommitmentConfig::confirmed();
    let mut handler =
        JitoBellHandler::new(args.endpoint.clone(), commitment, args.config_file).await?;

    info!("Jito Bell Config:\n{}", handler.config);

    info!("Starting heartbeat...");
    handler.heart_beat(&subscribe_option).await?;

    Ok(())
}
