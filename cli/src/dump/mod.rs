//! Pipeline de dump determinístico de manifestos por conta e janela temporal.
//!
//! O período é dividido em janelas de `window_days` e processado conta a conta;
//! cada janela concluída é marcada em `.openmtr-dump-state.json`, permitindo
//! retomar com `--resume`. As linhas são deduplicadas pela chave
//! `sistema|número` (ou código de barras) mantendo a conta de maior prioridade,
//! e ordenadas por `(account, system, numero, barcode)` antes da emissão, que
//! grava `mtrs.jsonl`, `mtrs.csv`, `residuos.csv` (conforme `format`) e
//! `relatorio.json` com totais e completeness. Falhas em janelas são
//! coletadas e resultam em [`CliError::Partial`](crate::error::CliError::Partial).
use crate::account::Account;
use crate::config::CliConfig;
use crate::error::CliError;
use crate::normalize::DumpRow;
use crate::session::SessionCache;
use jiff::civil::Date;
use serde::Serialize;
use std::path::PathBuf;

mod dedup;
mod fetch;
mod output;
mod pdf;
mod pipeline;
mod state;
#[cfg(test)]
mod tests;
mod window;

pub use window::parse_date;
pub(crate) use window::window_key;

pub(crate) fn internal(e: serde_json::Error) -> CliError {
    CliError::Internal(e.to_string())
}

use dedup::deduplicate;
use pipeline::{emit_outputs, fetch_accounts};
use state::{State, load_previous_rows, load_state};
use window::{validate_period, windows};

/// Parâmetros do pipeline de dump.
#[allow(
    clippy::struct_excessive_bools,
    reason = "flags independentes da CLI: PDF, dedup, retomada e force"
)]
#[derive(Debug, Clone)]
pub struct DumpOptions {
    /// Data inicial do período, inclusiva.
    pub from: Date,
    /// Data final do período, inclusiva.
    pub to: Date,
    /// Tamanho das janelas em dias; o período é dividido em blocos contíguos.
    pub window_days: u32,
    /// Diretório de saída para JSONL/CSV/PDF/relatório/estado.
    pub out: PathBuf,
    /// Baixa também os PDFs dos manifestos.
    pub pdf: bool,
    /// Desliga a deduplicação entre contas.
    pub no_dedup: bool,
    /// Retoma janelas já concluídas e linhas do JSONL anterior.
    pub resume: bool,
    /// Ignora estado e saídas anteriores, refazendo tudo.
    pub force: bool,
    /// Formato de saída: `jsonl`, `csv`, `both` ou `none`.
    pub format: String,
    /// Concorrência de requisições por conta.
    pub concurrency: u32,
    /// Relógio fixo para determinismo de testes.
    pub fixed_time: Option<jiff::Timestamp>,
}

/// Falha de uma janela, conta ou PDF, registrada no relatório.
#[derive(Debug, Clone, Serialize)]
pub struct DumpError {
    /// ID da conta afetada.
    pub account: String,
    /// Janela no formato `inicio..fim`, quando aplicável.
    pub window: Option<String>,
    /// Categoria do erro (`item`, `window`, `pdf`, ...).
    pub code: String,
    /// Mensagem descritiva da falha.
    pub message: String,
}

/// Registro de uma linha removida pela deduplicação entre contas.
#[derive(Debug, Clone, Serialize)]
pub struct DedupRecord {
    /// Chave de deduplicação (`sistema|número` ou código de barras).
    pub key: String,
    /// Conta cuja linha foi mantida (maior prioridade).
    pub kept_account: String,
    /// Contas cujas linhas foram descartadas.
    pub dropped_accounts: Vec<String>,
}

#[derive(Debug, Default, Serialize)]
struct Totals {
    fetched: usize,
    emitted: usize,
    deduplicated: usize,
    errors: usize,
}

/// Executa o dump completo, retornando `Partial` se houver erros em janelas ou PDFs.
#[allow(
    clippy::excessive_nesting,
    clippy::print_stderr,
    reason = "pipeline linear: conta -> janela -> fetch; relatório de progresso vai para stderr"
)]
pub async fn run(
    cfg: &CliConfig,
    cache: &SessionCache,
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

    fetch_accounts(
        cfg,
        cache,
        accounts,
        opts,
        &windows,
        &state_path,
        &mut state,
        &mut rows,
        &mut errors,
        &mut skipped,
        &mut fetched,
    )
    .await?;

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

    let totals = emit_outputs(
        cfg,
        cache,
        accounts,
        opts,
        &rows,
        &dedup,
        &skipped,
        fetched,
        &mut errors,
    )
    .await?;

    if totals.errors > 0 {
        return Err(CliError::Partial(totals.errors));
    }
    Ok(())
}
