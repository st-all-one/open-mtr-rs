use super::fetch::{fetch_classes, fetch_window};
use super::output::{write_csv, write_jsonl, write_residuos_csv};
use super::pdf::download_pdfs;
use super::state::{State, save_state};
use super::{DedupRecord, DumpError, DumpOptions, Totals, internal, window_key};
use crate::account::Account;
use crate::config::CliConfig;
use crate::error::CliError;
use crate::normalize::DumpRow;
use crate::session::SessionCache;
use chrono::NaiveDate;
use serde_json::json;

#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "pipeline de fetch por conta: parâmetros refletem o estado da execução"
)]
pub(super) async fn fetch_accounts(
    cfg: &CliConfig,
    cache: &SessionCache,
    accounts: &[&Account],
    opts: &DumpOptions,
    windows: &[(NaiveDate, NaiveDate)],
    state_path: &std::path::Path,
    state: &mut State,
    rows: &mut Vec<DumpRow>,
    errors: &mut Vec<DumpError>,
    skipped: &mut Vec<String>,
    fetched: &mut usize,
) -> Result<(), CliError> {
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
        let client = effective.build_client(cfg, opts.fixed_time, cache).await?;
        let classes = fetch_classes(&client).await;
        for (start, end) in windows {
            let key = window_key(*start, *end);
            if state.is_finished(&account.id, &key) {
                continue;
            }
            match fetch_window(&client, account, *start, *end, &classes).await {
                Ok((mut got, item_errors)) => {
                    *fetched += got.len();
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
                    save_state(state_path, state)?;
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
    Ok(())
}

#[allow(
    clippy::too_many_arguments,
    clippy::print_stderr,
    reason = "saída do dump: parâmetros refletem o estado; o resumo vai para stderr"
)]
pub(super) async fn emit_outputs(
    cfg: &CliConfig,
    cache: &SessionCache,
    accounts: &[&Account],
    opts: &DumpOptions,
    rows: &[DumpRow],
    dedup: &[DedupRecord],
    skipped: &[String],
    fetched: usize,
    errors: &mut Vec<DumpError>,
) -> Result<Totals, CliError> {
    let mut files = Vec::new();
    if matches!(opts.format.as_str(), "jsonl" | "both") {
        let p = opts.out.join("mtrs.jsonl");
        write_jsonl(&p, rows)?;
        files.push("mtrs.jsonl");
    }
    if matches!(opts.format.as_str(), "csv" | "both") {
        let p = opts.out.join("mtrs.csv");
        write_csv(&p, rows)?;
        files.push("mtrs.csv");
        let rp = opts.out.join("residuos.csv");
        write_residuos_csv(&rp, rows)?;
        files.push("residuos.csv");
    }

    if opts.pdf {
        download_pdfs(
            cfg,
            cache,
            accounts,
            rows,
            &opts.out,
            opts.fixed_time,
            errors,
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
    Ok(totals)
}
