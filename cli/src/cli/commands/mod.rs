use crate::account::{Account, AccountScope};
use crate::batch::{self, BatchOptions};
use crate::dump::{self, DumpOptions};
use crate::error::CliError;
use crate::runtime::Runtime;
use chrono::{DateTime, NaiveDate, TimeZone, Utc};

use super::{AccountCmd, BatchArgs, CdfCmd, ConfigCmd, DumpArgs, MtrCmd, RefCmd};
use crate::output::OutputFormat;

mod cdf;
mod io;
mod mtr;
mod ref_commands;

pub(super) fn parse_fixed_time(raw: Option<&str>) -> Result<Option<DateTime<Utc>>, CliError> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    if let Ok(dt) = DateTime::parse_from_rfc3339(raw) {
        return Ok(Some(dt.with_timezone(&Utc)));
    }
    if let Ok(d) = NaiveDate::parse_from_str(raw, "%Y-%m-%d") {
        let dt = d
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| CliError::Usage("data inválida".into()))?;
        return Ok(Some(Utc.from_utc_datetime(&dt)));
    }
    Err(CliError::Usage(format!(
        "--fixed-time inválido: {raw} (use RFC 3339 ou YYYY-MM-DD)"
    )))
}

#[allow(clippy::print_stdout, reason = "CLI: a saída padrão é o produto")]
pub(super) fn run_account(rt: &Runtime, sub: AccountCmd) -> Result<(), CliError> {
    match sub {
        AccountCmd::List => {
            for a in &rt.accounts {
                println!(
                    "{}\t{}\t{}\t{}",
                    a.id,
                    a.system_slug(),
                    a.access_type,
                    a.tags.join(",")
                );
            }
        }
        AccountCmd::Test { id } => {
            let acc = find_account(rt, Some(&id))?;
            println!("ok: {id} ({})", acc.system_slug());
        }
    }
    Ok(())
}

#[allow(clippy::print_stdout, reason = "CLI: a saída padrão é o produto")]
pub(super) fn run_config(
    config_path: &std::path::Path,
    rt: &Runtime,
    sub: ConfigCmd,
    format: OutputFormat,
) -> Result<(), CliError> {
    match sub {
        ConfigCmd::Validate => {
            let count = rt.accounts.len();
            println!("config ok: {count} contas");
        }
        ConfigCmd::Migrate { dry_run } => {
            let mode = if dry_run {
                crate::migrate::WriteMode::DryRun
            } else {
                crate::migrate::WriteMode::Apply
            };
            let out = crate::migrate::migrate(config_path, mode)?;
            if matches!(format, OutputFormat::Json) {
                crate::output::print_json(&serde_json::json!({
                    "config": config_path.display().to_string(),
                    "dry_run": out.dry_run,
                    "changed": out.changed,
                    "from_version": out.from_version,
                    "to_version": out.to_version,
                    "backup": out.backup.as_ref().map(|p| p.display().to_string()),
                }));
            } else if out.changed {
                let path = config_path.display();
                let from = out.from_version.unwrap_or(0);
                let to = out.to_version;
                let backup = out
                    .backup
                    .as_ref()
                    .map_or_default(|p| format!(" (backup: {})", p.display()));
                println!("migrado {path}: config_version {from} -> {to}{backup}");
            } else {
                let version = out.to_version;
                println!("config já está em config_version {version}");
            }
        }
    }
    Ok(())
}

pub(super) async fn run_mtr(
    rt: &Runtime,
    sub: MtrCmd,
    fixed_time: Option<DateTime<Utc>>,
    format: OutputFormat,
) -> Result<(), CliError> {
    mtr::run(rt, sub, fixed_time, format).await
}

pub(super) async fn run_cdf(
    rt: &Runtime,
    sub: CdfCmd,
    fixed_time: Option<DateTime<Utc>>,
    format: OutputFormat,
) -> Result<(), CliError> {
    cdf::run(rt, sub, fixed_time, format).await
}

pub(super) async fn run_ref(
    rt: &Runtime,
    sub: RefCmd,
    fixed_time: Option<DateTime<Utc>>,
    format: OutputFormat,
) -> Result<(), CliError> {
    ref_commands::run(rt, sub, fixed_time, format).await
}

pub(super) async fn run_dump(
    rt: &Runtime,
    args: DumpArgs,
    fixed_time: Option<DateTime<Utc>>,
) -> Result<(), CliError> {
    let scope = if args.all_accounts {
        AccountScope::All
    } else if !args.account.is_empty() {
        AccountScope::Ids(&args.account)
    } else if let Some(system) = args.system.as_deref() {
        AccountScope::System(system)
    } else {
        AccountScope::Tags(&args.tag)
    };
    let sel = rt.select(scope)?;
    if sel.is_empty() {
        return Err(CliError::Usage("dump exige escopo de conta".into()));
    }
    let opts = DumpOptions {
        from: dump::parse_date(&args.from)?,
        to: dump::parse_date(&args.to)?,
        window_days: args.window_days,
        out: args.out,
        pdf: args.pdf,
        no_dedup: args.no_dedup,
        resume: args.resume,
        force: args.force,
        format: args.format,
        concurrency: args.concurrency,
        fixed_time,
    };
    dump::run(&rt.config, &rt.cache, &sel, &opts).await
}

pub(super) async fn run_batch(
    rt: &Runtime,
    args: BatchArgs,
    fixed_time: Option<DateTime<Utc>>,
) -> Result<(), CliError> {
    let opts = BatchOptions {
        file: args.file,
        continue_on_error: args.continue_on_error,
        dry_run: args.dry_run,
        fixed_time,
    };
    batch::run(rt, &opts).await
}

pub(super) fn find_account<'a>(rt: &'a Runtime, id: Option<&str>) -> Result<&'a Account, CliError> {
    if rt.is_login() && id.is_some() {
        return Err(CliError::Usage("--login não combina com --account".into()));
    }
    match id {
        Some(id) => rt
            .accounts
            .iter()
            .find(|a| a.id == id && a.enabled)
            .ok_or_else(|| CliError::NotFound(format!("conta não encontrada: {id}"))),
        None => rt.single(),
    }
}

fn single_account<'a>(sel: &[&'a Account]) -> Result<&'a Account, CliError> {
    match sel {
        [only] => Ok(*only),
        [] => Err(CliError::Usage("escopo de conta obrigatório".into())),
        _ => Err(CliError::Usage(
            "operação exige exatamente uma conta; use --account".into(),
        )),
    }
}
