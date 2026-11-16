use crate::error::CliError;
use crate::runtime::Runtime;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

mod commands;

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

#[derive(Parser, Debug)]
#[command(
    name = "openmtr",
    version,
    about = "CLI utilitária multi-conta do openMTR",
    after_help = AFTER_HELP
)]
pub struct Cli {
    #[arg(long, global = true, default_value = "open-mtr-config.toml")]
    pub config: PathBuf,
    /// Credenciais explícitas por sistema (dispensa `--config`).
    #[arg(long, global = true, value_name = "JSON|@ARQUIVO|-")]
    pub login: Option<String>,
    #[arg(long, global = true)]
    pub json: bool,
    #[arg(long, global = true)]
    pub fixed_time: Option<String>,
    #[arg(long, global = true)]
    pub quiet: bool,
    #[arg(long, global = true)]
    pub log_level: Option<String>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Account {
        #[command(subcommand)]
        sub: AccountCmd,
    },
    Config {
        #[command(subcommand)]
        sub: ConfigCmd,
    },
    Mtr {
        #[command(subcommand)]
        sub: MtrCmd,
    },
    Cdf {
        #[command(subcommand)]
        sub: CdfCmd,
    },
    Ref {
        #[command(subcommand)]
        sub: RefCmd,
    },
    Health,
    Dump(DumpArgs),
    Batch(BatchArgs),
    #[command(name = "self")]
    SelfTool {
        #[command(subcommand)]
        sub: SelfCmd,
    },
}

#[derive(Args, Debug)]
pub struct DumpArgs {
    #[arg(long)]
    pub all_accounts: bool,
    #[arg(long)]
    pub account: Vec<String>,
    #[arg(long)]
    pub system: Option<String>,
    #[arg(long)]
    pub tag: Vec<String>,
    #[arg(long)]
    pub from: String,
    #[arg(long)]
    pub to: String,
    #[arg(long, default_value = "30")]
    pub window_days: u32,
    #[arg(long, default_value = "dump")]
    pub out: PathBuf,
    #[arg(long)]
    pub pdf: bool,
    #[arg(long)]
    pub no_dedup: bool,
    #[arg(long)]
    pub resume: bool,
    #[arg(long)]
    pub force: bool,
    #[arg(long, default_value = "both")]
    pub format: String,
    #[arg(long, default_value = "10")]
    pub concurrency: u32,
}

#[derive(Args, Debug)]
pub struct BatchArgs {
    /// JSONL de operações; use `-` para stdin.
    #[arg(long, value_name = "JSONL|-")]
    pub file: Option<PathBuf>,
    #[arg(long)]
    pub continue_on_error: bool,
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Subcommand, Debug)]
pub enum SelfCmd {
    Version,
    Update {
        #[arg(long)]
        check: bool,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        version: Option<String>,
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum AccountCmd {
    List,
    Test { id: String },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCmd {
    Validate,
    Migrate {
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum MtrCmd {
    /// Consulta MTRs por número (posicional) ou por filtro em `--params`.
    Consultar {
        /// Número do MTR; exclusivo com `--params`.
        #[arg(value_name = "NUMERO", conflicts_with = "params")]
        numero: Option<String>,
        #[arg(long)]
        account: Option<String>,
        #[arg(long)]
        system: Option<String>,
        #[arg(long)]
        tag: Vec<String>,
        #[arg(long)]
        all_accounts: bool,
        #[arg(long, value_name = "JSON|-")]
        params: Option<String>,
    },
    Criar {
        #[arg(long)]
        account: Option<String>,
        #[arg(long, value_name = "JSON|-", required = true)]
        params: String,
    },
    Receber {
        #[arg(long)]
        account: Option<String>,
        #[arg(long, value_name = "JSON|-", required = true)]
        params: String,
    },
    Cancelar {
        #[arg(long)]
        account: Option<String>,
        #[arg(long, value_name = "JSON|-", required = true)]
        params: String,
    },
    Modificar {
        #[arg(long)]
        account: Option<String>,
        #[arg(long, value_name = "JSON|-", required = true)]
        params: String,
    },
    Aceitar {
        #[arg(long)]
        account: Option<String>,
        #[arg(long, value_name = "JSON|-", required = true)]
        params: String,
    },
    /// Baixa PDFs de um ou mais números.
    Download {
        /// Número do MTR; exclusivo com `--params`.
        #[arg(value_name = "NUMERO", conflicts_with = "params")]
        numero: Option<String>,
        #[arg(long)]
        account: Option<String>,
        #[arg(long, value_name = "JSON|-")]
        params: Option<String>,
        #[arg(long)]
        out: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
pub enum CdfCmd {
    /// Consulta CDFs por código (posicional) ou por filtro em `--params`.
    Consultar {
        /// Código do CDF; exclusivo com `--params`.
        #[arg(value_name = "CODIGO", conflicts_with = "params")]
        codigo: Option<String>,
        #[arg(long)]
        account: Option<String>,
        #[arg(long, value_name = "JSON|-")]
        params: Option<String>,
    },
    Gerar {
        #[arg(long)]
        account: Option<String>,
        #[arg(long, value_name = "JSON|-", required = true)]
        params: String,
    },
    Download {
        #[arg(long)]
        account: Option<String>,
        #[arg(long, value_name = "JSON|-", required = true)]
        params: String,
        #[arg(long)]
        out: PathBuf,
    },
}

/// Tabelas de referência suportadas (conjunto fechado).
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum RefTable {
    #[value(alias = "residuos")]
    Residuo,
    #[value(alias = "classes")]
    Classe,
    #[value(alias = "unidades")]
    Unidade,
    #[value(alias = "acondicionamentos")]
    Acondicionamento,
    #[value(
        name = "estado-fisico",
        alias = "estado_fisico",
        alias = "estados_fisicos"
    )]
    EstadoFisico,
    #[value(alias = "tratamentos")]
    Tratamento,
}

#[derive(Subcommand, Debug)]
pub enum RefCmd {
    /// Lista uma tabela de referência.
    Listar {
        /// Tabela de referência.
        #[arg(value_name = "TABELA")]
        table: RefTable,
        #[arg(long)]
        account: Option<String>,
    },
    /// Classes compatíveis com um código IBAMA.
    PorResiduo {
        /// Código IBAMA do resíduo.
        #[arg(value_name = "CODIGO")]
        codigo: String,
        #[arg(long)]
        account: Option<String>,
    },
    /// Acondicionamentos de um estado físico.
    PorEstadoFisico {
        /// Código do estado físico.
        #[arg(value_name = "CODIGO")]
        codigo: String,
        #[arg(long)]
        account: Option<String>,
    },
}

impl Cli {
    pub async fn run(self) -> Result<(), CliError> {
        if self.login.as_deref() == Some("-") && command_reads_stdin(&self.command) {
            return Err(CliError::Usage(
                "--login - não combina com --params -/--file - (um único consumidor de stdin)"
                    .into(),
            ));
        }
        let fixed_time = commands::parse_fixed_time(self.fixed_time.as_deref())?;
        if self.login.is_some() && matches!(&self.command, Command::SelfTool { .. }) {
            return Err(CliError::Usage("--login não se aplica a `self`".into()));
        }
        if let Command::SelfTool { sub } = &self.command {
            return run_self(sub, self.json).await;
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
            let _ = self
                .login
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
            Command::Config { sub } => commands::run_config(&config_path, &rt, sub, self.json),
            Command::Mtr { sub } => commands::run_mtr(&rt, sub, fixed_time, self.json).await,
            Command::Cdf { sub } => commands::run_cdf(&rt, sub, fixed_time, self.json).await,
            Command::Ref { sub } => commands::run_ref(&rt, sub, fixed_time, self.json).await,
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

async fn run_self(sub: &SelfCmd, json: bool) -> Result<(), CliError> {
    match sub {
        SelfCmd::Version => {
            println!("openmtr {}", crate::selfupdate::current_version());
            Ok(())
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
            if json {
                crate::output::print_json(&plan);
            } else if opts.check {
                println!(
                    "current: {}\nlatest: {}\nup_to_date: {}",
                    plan.current, plan.latest, !plan.changed
                );
            } else if opts.dry_run {
                println!(
                    "target: {}\nasset: {}\nurl: {}\nchecksums: {}\nchanged: {}",
                    plan.target, plan.asset, plan.asset_url, plan.checksums_url, plan.changed
                );
            } else if plan.changed || opts.force {
                println!("atualizado {} -> {}", plan.current, plan.latest);
            } else {
                println!("já está na versão {}", plan.current);
            }
            Ok(())
        }
    }
}
