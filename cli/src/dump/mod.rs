use crate::account::Account;
use crate::config::CliConfig;
use crate::error::CliError;
use crate::normalize::DumpRow;
use chrono::NaiveDate;
use serde::Serialize;
use serde_json::json;
use std::path::PathBuf;

mod dedup;
mod fetch;
mod output;
mod pdf;
mod state;

use dedup::deduplicate;
use fetch::{fetch_classes, fetch_window};
use output::{write_csv, write_jsonl, write_residuos_csv};
use pdf::download_pdfs;
use state::{State, load_previous_rows, load_state, save_state};

#[derive(Debug, Clone)]
pub struct DumpOptions {
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub window_days: u32,
    pub out: PathBuf,
    pub pdf: bool,
    pub no_dedup: bool,
    pub resume: bool,
    pub force: bool,
    pub format: String,
    pub concurrency: u32,
    pub fixed_time: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DumpError {
    pub account: String,
    pub window: Option<String>,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DedupRecord {
    pub key: String,
    pub kept_account: String,
    pub dropped_accounts: Vec<String>,
}

#[derive(Debug, Default, Serialize)]
struct Totals {
    fetched: usize,
    emitted: usize,
    deduplicated: usize,
    errors: usize,
}

#[allow(
    clippy::excessive_nesting,
    reason = "pipeline linear: conta -> janela -> fetch; extrair funções separaria passos coesos"
)]
pub async fn run(
    cfg: &CliConfig,
    accounts: &[&Account],
    opts: &DumpOptions,
) -> Result<(), CliError> {
    validate_period(opts)?;
    std::fs::create_dir_all(&opts.out)
        .map_err(|e| CliError::Config(format!("criar {}: {e}", opts.out.display())))?;

    let mut rows: Vec<DumpRow> = if opts.resume && !opts.force {
        load_previous_rows(&opts.out)?
    } else {
        Vec::new()
    };

    let state_path = opts.out.join(".openmtr-dump-state.json");
    let mut state = if opts.resume && !opts.force {
        load_state(&state_path, opts, accounts)?
    } else {
        State::new(opts, accounts)
    };

    let mut errors: Vec<DumpError> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();
    let windows = windows(opts.window_days, opts.from, opts.to);
    let mut fetched = rows.len();

    for account in accounts {
        if matches!(
            account.system,
            open_mtr_core::openmtr::system::System::Sigor
        ) {
            if accounts.len() == 1 {
                return Err(CliError::Usage(format!(
                    "dump: SIGOR não suporta listagem por período (conta {}); use `mtr consultar --numero` ou batch",
                    account.id
                )));
            }
            skipped.push(account.id.clone());
            continue;
        }
        let mut effective = (*account).clone();
        effective.max_concurrency = Some(opts.concurrency.max(1));
        let client = effective.build_client(cfg, opts.fixed_time).await?;
        let classes = fetch_classes(&client).await;
        for (start, end) in &windows {
            let key = window_key(*start, *end);
            if state.is_finished(&account.id, &key) {
                continue;
            }
            match fetch_window(&client, account, *start, *end, &classes).await {
                Ok((mut got, item_errors)) => {
                    fetched += got.len();
                    rows.append(&mut got);
                    for message in item_errors {
                        errors.push(DumpError {
                            account: account.id.clone(),
                            window: Some(key.clone()),
                            code: "item".to_string(),
                            message,
                        });
                    }
                    state.mark(&account.id, &key);
                    save_state(&state_path, &state)?;
                }
                Err(e) => {
                    errors.push(DumpError {
                        account: account.id.clone(),
                        window: Some(key),
                        code: "window".to_string(),
                        message: e.to_string(),
                    });
                }
            }
        }
    }

    let (rows, dedup) = if opts.no_dedup {
        (rows, Vec::new())
    } else {
        deduplicate(rows)
    };

    let mut rows = rows;
    rows.sort_by(|a, b| {
        (&a.account, &a.system, &a.numero, &a.barcode)
            .cmp(&(&b.account, &b.system, &b.numero, &b.barcode))
    });

    let mut files = Vec::new();
    if matches!(opts.format.as_str(), "jsonl" | "both") {
        let p = opts.out.join("mtrs.jsonl");
        write_jsonl(&p, &rows)?;
        files.push("mtrs.jsonl");
    }
    if matches!(opts.format.as_str(), "csv" | "both") {
        let p = opts.out.join("mtrs.csv");
        write_csv(&p, &rows)?;
        files.push("mtrs.csv");
        let rp = opts.out.join("residuos.csv");
        write_residuos_csv(&rp, &rows)?;
        files.push("residuos.csv");
    }

    if opts.pdf {
        download_pdfs(
            cfg,
            accounts,
            &rows,
            &opts.out,
            opts.fixed_time,
            &mut errors,
        )
        .await?;
    }

    let totals = Totals {
        fetched,
        emitted: rows.len(),
        deduplicated: dedup.len(),
        errors: errors.len(),
    };
    let report = json!({
        "config_version": cfg.config_version,
        "period": { "from": opts.from, "to": opts.to },
        "accounts": accounts.iter().map(|a| a.id.clone()).collect::<Vec<_>>(),
        "files": files,
        "totals": totals,
        "completeness": {
            "complete": errors.is_empty(),
            "errors": errors.len(),
        },
        "deduplicated": dedup,
        "skipped": skipped,
        "errors": errors,
    });
    let path = opts.out.join("relatorio.json");
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&report).map_err(internal)?,
    )
    .map_err(|e| CliError::Config(format!("escrever relatório: {e}")))?;

    eprintln!(
        "dump: {} MTR(s) emitido(s), {} deduplicado(s), {} erro(s) -> {}",
        totals.emitted,
        totals.deduplicated,
        totals.errors,
        opts.out.display()
    );
    if totals.errors > 0 {
        return Err(CliError::Partial(totals.errors));
    }
    Ok(())
}

fn validate_period(opts: &DumpOptions) -> Result<(), CliError> {
    if opts.from > opts.to {
        return Err(CliError::Usage("--from deve ser <= --to".into()));
    }
    if (opts.to - opts.from).num_days() > 3660 {
        return Err(CliError::Usage("período maior que 3660 dias".into()));
    }
    if opts.window_days == 0 {
        return Err(CliError::Usage("--window-days deve ser >= 1".into()));
    }
    match opts.format.as_str() {
        "jsonl" | "csv" | "both" | "none" => Ok(()),
        other => Err(CliError::Usage(format!("--format inválido: {other}"))),
    }
}

pub fn parse_date(s: &str) -> Result<NaiveDate, CliError> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .or_else(|_| NaiveDate::parse_from_str(s, "%d-%m-%Y"))
        .map_err(|_| CliError::Usage(format!("data inválida: {s} (use YYYY-MM-DD)")))
}

fn windows(days: u32, from: NaiveDate, to: NaiveDate) -> Vec<(NaiveDate, NaiveDate)> {
    let step = chrono::Duration::days(i64::from(days.max(1)) - 1);
    let mut out = Vec::new();
    let mut start = from;
    while start <= to {
        let end = (start + step).min(to);
        out.push((start, end));
        start = end + chrono::Duration::days(1);
    }
    out
}

pub(crate) fn window_key(start: NaiveDate, end: NaiveDate) -> String {
    format!("{start}..{end}")
}

pub(crate) fn internal(e: serde_json::Error) -> CliError {
    CliError::Internal(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_respect_bounds_and_step() {
        let from = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 5, 31).unwrap();
        let w = windows(30, from, to);
        assert_eq!(w.len(), 2);
        assert_eq!(w[0], (from, NaiveDate::from_ymd_opt(2026, 5, 30).unwrap()));
        assert_eq!(w[1], (NaiveDate::from_ymd_opt(2026, 5, 31).unwrap(), to));
    }

    #[test]
    fn parse_date_accepts_iso_and_brazilian() {
        let expected = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
        assert_eq!(parse_date("2026-05-01").unwrap(), expected);
        assert_eq!(parse_date("01-05-2026").unwrap(), expected);
    }
}
