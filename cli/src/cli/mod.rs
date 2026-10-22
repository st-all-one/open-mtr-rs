use crate::error::CliError;
use crate::runtime::Runtime;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

mod commands;

#[derive(Parser, Debug)]
#[command(
    name = "openmtr",
    version,
    about = "CLI utilitária multi-conta do openMTR"
)]
pub struct Cli {
    #[arg(long, global = true, default_value = "open-mtr-config.toml")]
    pub config: PathBuf,
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
    #[arg(long)]
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
    Consultar {
        #[arg(long)]
        account: Option<String>,
        #[arg(long)]
        system: Option<String>,
        #[arg(long)]
        tag: Vec<String>,
        #[arg(long)]
        all_accounts: bool,
        #[arg(long)]
        params: Option<String>,
    },
    Criar {
        #[arg(long)]
        account: String,
        #[arg(long)]
        params: Option<String>,
    },
    Receber {
        #[arg(long)]
        account: String,
        #[arg(long)]
        params: Option<String>,
    },
    Cancelar {
        #[arg(long)]
        account: String,
        #[arg(long)]
        params: Option<String>,
    },
    Modificar {
        #[arg(long)]
        account: String,
        #[arg(long)]
        params: Option<String>,
    },
    Aceitar {
        #[arg(long)]
        account: String,
        #[arg(long)]
        params: Option<String>,
    },
    Download {
        #[arg(long)]
        account: String,
        #[arg(long)]
        params: Option<String>,
        #[arg(long)]
        out: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
pub enum CdfCmd {
    Consultar {
        #[arg(long)]
        account: Option<String>,
        #[arg(long)]
        params: Option<String>,
    },
    Gerar {
        #[arg(long)]
        account: String,
        #[arg(long)]
        params: Option<String>,
    },
    Download {
        #[arg(long)]
        account: String,
        #[arg(long)]
        params: Option<String>,
        #[arg(long)]
        out: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
pub enum RefCmd {
    Listar {
        #[arg(long)]
        account: Option<String>,
        #[arg(long)]
        table: String,
    },
}

impl Cli {
    pub async fn run(self) -> Result<(), CliError> {
        let fixed_time = commands::parse_fixed_time(self.fixed_time.as_deref())?;
        if let Command::SelfTool { sub } = &self.command {
            return run_self(sub, self.json).await;
        }
        let config_path = self.config.clone();
        let rt = Runtime::load(&self.config)?;
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
