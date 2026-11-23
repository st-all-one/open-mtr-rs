//! Definição da interface de linha de comando e despacho dos subcomandos.

use crate::error::CliError;
use crate::runtime::Runtime;
use clap::Parser;
use std::path::PathBuf;

mod args;
mod command;
mod commands;
mod subcommands;

pub use args::{BatchArgs, DumpArgs};
pub use command::Command;
pub use subcommands::{AccountCmd, CdfCmd, ConfigCmd, MtrCmd, RefCmd, RefTable, SelfCmd};

const AFTER_HELP: &str = "\
ENTRADA
  --login <JSON|@ARQUIVO|->  credenciais explícitas por sistema (dispensa --config)
  --params <JSON|->          payload do comando
  --file <JSONL|->           lote do `batch`

POSICIONAIS
  Quando existir, o posicional é o ALVO primário do comando (NUMERO, CODIGO ou
  TABELA) e é mutuamente exclusivo com --params. Nenhum comando lê stdin
  implicitamente: use `-` explícito.

SAÍDA
  stdout = dados; stderr = logs. `--json` emite JSON de máquina.";

/// Executa um comando `openmtr` a partir dos argumentos de linha de comando.
#[derive(Parser, Debug)]
#[command(
    name = "openmtr",
    version,
    about = "CLI utilitária multi-conta do openMTR",
    after_help = AFTER_HELP
)]
pub struct Cli {
    /// Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`).
    #[arg(long, global = true, default_value = "open-mtr-config.toml")]
    pub config: PathBuf,
    /// Credenciais explícitas por sistema (dispensa `--config`).
    #[arg(long, global = true, value_name = "JSON|@ARQUIVO|-")]
    pub login: Option<String>,
    /// Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída.
    #[arg(long, global = true)]
    pub json: bool,
    /// Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`.
    #[arg(long, global = true)]
    pub fixed_time: Option<String>,
    /// Suprime a inicialização dos logs (a saída de dados permanece).
    #[arg(long, global = true)]
    pub quiet: bool,
    /// Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`).
    #[arg(long, global = true)]
    pub log_level: Option<String>,
    /// Grupo de comandos a executar.
    #[command(subcommand)]
    pub command: Command,
}

#[allow(clippy::print_stdout, reason = "CLI: a saída padrão é o produto")]
impl Cli {
    /// Executa o comando selecionado com o runtime configurado.
    pub async fn run(self) -> Result<(), CliError> {
        if self.login.as_deref() == Some("-") && command_reads_stdin(&self.command) {
            return Err(CliError::Usage(
                "--login - não combina com --params -/--file - (um único consumidor de stdin)"
                    .into(),
            ));
        }
        let fixed_time = commands::parse_fixed_time(self.fixed_time.as_deref())?;
        let format = if self.json {
            crate::output::OutputFormat::Json
        } else {
            crate::output::OutputFormat::Text
        };
        if self.login.is_some() && matches!(&self.command, Command::SelfTool { .. }) {
            return Err(CliError::Usage("--login não se aplica a `self`".into()));
        }
        if let Command::SelfTool { sub } = &self.command {
            return run_self(sub, format).await;
        }
        if self.login.is_some()
            && matches!(
                &self.command,
                Command::Config { .. } | Command::Account { .. }
            )
        {
            return Err(CliError::Usage(
                "--login não se aplica a `config`/`account`; use --config".into(),
            ));
        }
        if matches!(&self.command, Command::Health) {
            // `health` é um smoke test do binário: valida o login quando
            // fornecido, mas não exige config nem chama a API.
            self.login
                .as_deref()
                .map(crate::login::Login::parse)
                .transpose()?;
            println!("ok");
            return Ok(());
        }
        let rt = match &self.login {
            Some(raw) => Runtime::from_login(crate::login::Login::parse(raw)?)?,
            None => Runtime::load(&self.config)?,
        };
        let config_path = self.config.clone();
        match self.command {
            Command::Health => {
                println!("ok");
                Ok(())
            }
            Command::Account { sub } => commands::run_account(&rt, sub),
            Command::Config { sub } => commands::run_config(&config_path, &rt, sub, format),
            Command::Mtr { sub } => commands::run_mtr(&rt, sub, fixed_time, format).await,
            Command::Cdf { sub } => commands::run_cdf(&rt, sub, fixed_time, format).await,
            Command::Ref { sub } => commands::run_ref(&rt, sub, fixed_time, format).await,
            Command::Dump(args) => commands::run_dump(&rt, args, fixed_time).await,
            Command::Batch(args) => commands::run_batch(&rt, args, fixed_time).await,
            Command::SelfTool { .. } => Err(CliError::Internal(
                "comando self deveria ter sido tratado antes do runtime".into(),
            )),
        }
    }
}

/// `true` quando o comando consome stdin via `-`.
fn command_reads_stdin(command: &Command) -> bool {
    match command {
        Command::Batch(args) => args.file.as_deref() == Some(std::path::Path::new("-")),
        Command::Mtr { sub } => match sub {
            MtrCmd::Consultar { params, .. } | MtrCmd::Download { params, .. } => {
                params.as_deref() == Some("-")
            }
            MtrCmd::Criar { params, .. }
            | MtrCmd::Receber { params, .. }
            | MtrCmd::Cancelar { params, .. }
            | MtrCmd::Modificar { params, .. }
            | MtrCmd::Aceitar { params, .. } => params.as_str() == "-",
        },
        Command::Cdf { sub } => match sub {
            CdfCmd::Consultar { params, .. } => params.as_deref() == Some("-"),
            CdfCmd::Gerar { params, .. } | CdfCmd::Download { params, .. } => {
                params.as_str() == "-"
            }
        },
        _ => false,
    }
}

#[allow(clippy::print_stdout, reason = "CLI: a saída padrão é o produto")]
async fn run_self(sub: &SelfCmd, format: crate::output::OutputFormat) -> Result<(), CliError> {
    match sub {
        SelfCmd::Version => {
            let version = crate::selfupdate::current_version();
            println!("openmtr {version}");
        }
        SelfCmd::Update {
            check,
            dry_run,
            version,
            force,
        } => {
            let opts = crate::selfupdate::UpdateOptions {
                check: *check,
                dry_run: *dry_run,
                version: version.clone(),
                force: *force,
                ..Default::default()
            };
            let plan = crate::selfupdate::run(&opts).await?;
            if matches!(format, crate::output::OutputFormat::Json) {
                crate::output::print_json(&plan);
            } else if *check {
                let current = &plan.current;
                let latest = &plan.latest;
                let up_to_date = !plan.changed;
                println!("current: {current}\nlatest: {latest}\nup_to_date: {up_to_date}");
            } else if *dry_run {
                let target = &plan.target;
                let asset = &plan.asset;
                let url = &plan.asset_url;
                let checksums = &plan.checksums_url;
                let changed = plan.changed;
                println!(
                    "target: {target}\nasset: {asset}\nurl: {url}\nchecksums: {checksums}\nchanged: {changed}"
                );
            } else if plan.changed || *force {
                let current = &plan.current;
                let latest = &plan.latest;
                println!("atualizado {current} -> {latest}");
            } else {
                let current = &plan.current;
                println!("já está na versão {current}");
            }
        }
    }
    Ok(())
}
