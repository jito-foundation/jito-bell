use std::{env, io::Write, process::Command};

use clap::Parser;
use jito_bell::{
    cli_args::Args, multi_writer::MultiWriter, subscribe_option::SubscribeOption, JitoBellHandler,
};
use log::info;
use solana_metrics::set_host_id;
use solana_sdk::commitment_config::CommitmentConfig;
use yellowstone_grpc_proto::geyser::CommitmentLevel;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

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

    let hostname_cmd = Command::new("hostname")
        .output()
        .expect("Failed to execute hostname command");

    let hostname = String::from_utf8_lossy(&hostname_cmd.stdout)
        .trim()
        .to_string();

    set_host_id(format!("jito-bell_{hostname}"));

    let commitment: CommitmentLevel = args.commitment.unwrap_or_default().into();
    let subscribe_option = SubscribeOption::new(args.clone(), commitment);

    info!("Subscription configuration:\n{}", subscribe_option);

    let commitment = CommitmentConfig::confirmed();
    let mut handler =
        JitoBellHandler::new(args.endpoint.clone(), commitment, args.config_file).await?;

    info!("Jito Bell Config:\n{}", handler.config);

    info!("Starting heartbeat...");
    handler.heart_beat(&subscribe_option).await?;

    Ok(())
}
