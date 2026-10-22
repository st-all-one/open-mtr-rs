#![allow(
    clippy::cargo_common_metadata,
    reason = "crate interna, publish = false"
)]
use clap::Parser;
use open_mtr_cli::cli::Cli;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    init_tracing(cli.quiet, cli.log_level.as_deref());
    match cli.run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("erro: {e}");
            ExitCode::from(e.exit_code() as u8)
        }
    }
}

fn init_tracing(quiet: bool, level: Option<&str>) {
    use tracing_subscriber::EnvFilter;
    if quiet {
        return;
    }
    let filter = match level {
        Some(level) => EnvFilter::new(level),
        None => EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
    };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();
}
