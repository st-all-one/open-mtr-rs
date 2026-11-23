//! Binário `openmtr`: parseia os argumentos e delega ao módulo `cli`.

#![allow(
    clippy::cargo_common_metadata,
    reason = "crate interna, publish = false"
)]
#![allow(clippy::print_stderr, reason = "o binário reporta erros em stderr")]
use clap::Parser;
use open_mtr_cli::cli::Cli;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    if cli.quiet {
        init_tracing(LogInit::Silent);
    } else {
        init_tracing(LogInit::Filtered(cli.log_level.as_deref()));
    }
    match cli.run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("erro: {e}");
            ExitCode::from(u8::try_from(e.exit_code()).unwrap_or(1))
        }
    }
}

/// Como inicializar o `tracing` a partir das flags globais da CLI.
enum LogInit<'a> {
    /// `--quiet`: nenhuma saída de log.
    Silent,
    /// Filtro explícito (`--log-level`) ou `RUST_LOG`, com `warn` como padrão.
    Filtered(Option<&'a str>),
}

fn init_tracing(init: LogInit<'_>) {
    use tracing_subscriber::EnvFilter;
    let filter = match init {
        LogInit::Silent => return,
        LogInit::Filtered(Some(level)) => EnvFilter::new(level),
        LogInit::Filtered(None) => {
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"))
        }
    };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();
}
