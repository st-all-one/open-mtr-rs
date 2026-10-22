use crate::account::Account;
use crate::batch::{self, BatchOptions};
use crate::dump::{self, DumpOptions};
use crate::error::CliError;
use crate::runtime::Runtime;
use chrono::{DateTime, NaiveDate, TimeZone, Utc};

use super::{AccountCmd, BatchArgs, CdfCmd, ConfigCmd, DumpArgs, MtrCmd, RefCmd};

mod cdf;
mod io;
mod mtr;
mod ref_commands;

pub fn parse_fixed_time(raw: Option<&str>) -> Result<Option<DateTime<Utc>>, CliError> {
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

pub fn run_account(rt: &Runtime, sub: AccountCmd) -> Result<(), CliError> {
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
            Ok(())
        }
        AccountCmd::Test { id } => {
            let acc = find_account(rt, &id)?;
            println!("ok: {} ({})", id, acc.system_slug());
            Ok(())
        }
    }
}

pub fn run_config(
    config_path: &std::path::Path,
    rt: &Runtime,
    sub: ConfigCmd,
    json: bool,
) -> Result<(), CliError> {
    match sub {
        ConfigCmd::Validate => {
            println!("config ok: {} contas", rt.accounts.len());
            Ok(())
        }
        ConfigCmd::Migrate { dry_run } => {
            let out = crate::migrate::migrate(config_path, dry_run)?;
            if json {
                crate::output::print_json(&serde_json::json!({
                    "config": config_path.display().to_string(),
                    "dry_run": out.dry_run,
                    "changed": out.changed,
                    "from_version": out.from_version,
                    "to_version": out.to_version,
                    "backup": out.backup.as_ref().map(|p| p.display().to_string()),
                }));
            } else if out.changed {
                println!(
                    "migrado {}: config_version {} -> {}{}",
                    config_path.display(),
                    out.from_version.unwrap_or(0),
                    out.to_version,
                    out.backup
                        .as_ref()
                        .map(|p| format!(" (backup: {})", p.display()))
                        .unwrap_or_default()
                );
            } else {
                println!("config já está em config_version {}", out.to_version);
            }
            Ok(())
        }
    }
}

pub async fn run_mtr(
    rt: &Runtime,
    sub: MtrCmd,
    fixed_time: Option<DateTime<Utc>>,
    json: bool,
) -> Result<(), CliError> {
    mtr::run(rt, sub, fixed_time, json).await
}

pub async fn run_cdf(
    rt: &Runtime,
    sub: CdfCmd,
    fixed_time: Option<DateTime<Utc>>,
    json: bool,
) -> Result<(), CliError> {
    cdf::run(rt, sub, fixed_time, json).await
}

pub async fn run_ref(
    rt: &Runtime,
    sub: RefCmd,
    fixed_time: Option<DateTime<Utc>>,
    json: bool,
) -> Result<(), CliError> {
    ref_commands::run(rt, sub, fixed_time, json).await
}

pub async fn run_dump(
    rt: &Runtime,
    args: DumpArgs,
    fixed_time: Option<DateTime<Utc>>,
) -> Result<(), CliError> {
    let sel = rt.resolve(
        &args.account,
        args.system.as_deref(),
        &args.tag,
        args.all_accounts,
    )?;
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
    dump::run(&rt.config, &sel, &opts).await
}

pub async fn run_batch(
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

fn find_account<'a>(rt: &'a Runtime, id: &str) -> Result<&'a Account, CliError> {
    rt.accounts
        .iter()
        .find(|a| a.id == id && a.enabled)
        .ok_or_else(|| CliError::NotFound(format!("conta não encontrada: {id}")))
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
