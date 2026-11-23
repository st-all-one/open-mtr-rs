//! Execução de lotes de operações descritas em JSONL.
//!
//! Cada linha não vazia (e não comentada com `#`) é uma [`BatchOp`](crate::batch::BatchOp); o
//! resultado de cada operação é emitido como um objeto JSON com `index`, `id`,
//! `op`, `account`, `ok` e `status` (o código de saída equivalente do
//! [`CliError`](crate::error::CliError) ou com `error` contendo `code`, `message` e `retryable`.
//! O lote termina com `CliError::Partial` se qualquer operação falhar;
//! `continue_on_error` controla se as linhas seguintes ainda são executadas.
use crate::error::CliError;
use crate::params::periodo_from_json;
use crate::runtime::Runtime;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::PathBuf;

mod dispatch;
mod io;
mod ops;

#[cfg(test)]
mod tests;

use dispatch::execute;
use io::{error_code, read_input};

/// Operação individual de um lote em formato JSONL.
#[derive(Debug, Deserialize)]
pub struct BatchOp {
    /// Nome da operação; veja a lista de operações aceitas em `run_to`.
    pub op: String,
    #[serde(default)]
    /// Identificador arbitrário do chamador, repetido no resultado.
    pub id: Option<String>,
    #[serde(default)]
    /// ID da conta a usar; ausente usa a única conta do runtime.
    pub account: Option<String>,
    #[serde(default)]
    /// Parâmetros da operação, interpretados conforme a `op`.
    pub params: Value,
    #[serde(default)]
    /// Diretório de saída para operações de download.
    pub out: Option<PathBuf>,
    #[serde(default)]
    /// Simula a operação apenas para esta linha, sem chamadas à API.
    pub dry_run: bool,
}

/// Configuração global de execução do lote.
pub struct BatchOptions {
    /// Arquivo JSONL de entrada, ou `-` para stdin.
    pub file: Option<PathBuf>,
    /// Continua processando linhas após uma falha.
    pub continue_on_error: bool,
    /// Simula todas as operações, sem chamadas à API.
    pub dry_run: bool,
    /// Relógio fixo para tornar determinísticas as operações com timestamp.
    pub fixed_time: Option<DateTime<Utc>>,
}

const KNOWN_OPS: &[&str] = &[
    "health",
    "account.test",
    "mtr.consultar",
    "mtr.criar",
    "mtr.receber",
    "mtr.cancelar",
    "mtr.modificar",
    "mtr.aceitar",
    "mtr.download",
    "cdf.consultar",
    "cdf.gerar",
    "cdf.download",
    "ref.listar",
    "ref.por-residuo",
    "ref.por-estado-fisico",
];

/// Executa o lote emitindo cada resultado na saída padrão.
pub async fn run(rt: &Runtime, opts: &BatchOptions) -> Result<(), CliError> {
    let mut out = std::io::stdout();
    run_to(rt, opts, &mut out).await
}

/// Executa o lote escrevendo os resultados em `out` (usado em testes).
pub async fn run_to<W: std::io::Write>(
    rt: &Runtime,
    opts: &BatchOptions,
    out: &mut W,
) -> Result<(), CliError> {
    let input = read_input(opts.file.as_deref())?;
    let mut had_error = false;
    for (idx, line) in input.lines().enumerate() {
        let index = idx + 1;
        let text = line.trim();
        if text.is_empty() || text.starts_with('#') {
            continue;
        }
        let op: BatchOp = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(e) => return Err(CliError::Usage(format!("linha {index}: {e}"))),
        };
        if !KNOWN_OPS.contains(&op.op.as_str()) {
            return Err(CliError::Usage(format!(
                "linha {index}: op desconhecida: {}",
                op.op
            )));
        }
        let result = execute(rt, &op, opts).await;
        let line_out = match result {
            Ok(data) => json!({
                "index": index,
                "id": op.id,
                "op": op.op,
                "account": op.account,
                "ok": true,
                "status": 200,
                "data": data,
                "warnings": [],
            }),
            Err(e) => {
                had_error = true;
                json!({
                    "index": index,
                    "id": op.id,
                    "op": op.op,
                    "account": op.account,
                    "ok": false,
                    "status": e.exit_code(),
                    "error": { "code": error_code(&e), "message": e.to_string(), "retryable": matches!(e, CliError::Network(_)) },
                })
            }
        };
        writeln!(out, "{line_out}").map_err(|e| CliError::Config(e.to_string()))?;
        out.flush().map_err(|e| CliError::Config(e.to_string()))?;
        if had_error && !opts.continue_on_error {
            break;
        }
    }
    if had_error {
        return Err(CliError::Partial(1));
    }
    Ok(())
}

#[allow(dead_code, reason = "utilitário mantido para insultos de teste")]
fn _periodo(params: &Value) -> Result<Option<open_mtr_core::openmtr::domain::Periodo>, CliError> {
    periodo_from_json(params)
}
