use crate::account::Account;
use crate::cli::RefTable;
use crate::download::safe_path;
use crate::error::CliError;
use crate::params::{
    filtro_cdf_from_json, filtro_mtr_from_json, parse_items, periodo_from_json, string_array,
};
use crate::runtime::Runtime;
use chrono::{DateTime, Utc};
use open_mtr_core::openmtr::domain::{
    AceitarAlteracao, CancelarMtr, CriarMtr, DownloadCdf, DownloadItem, GerarCdf, ModificarMtr,
    ReceberMtr,
};
use open_mtr_core::openmtr::open_mtr_client::OpenMtrClient;
use serde::Deserialize;
use serde_json::{Value, json};
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct BatchOp {
    pub op: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub account: Option<String>,
    #[serde(default)]
    pub params: Value,
    #[serde(default)]
    pub out: Option<PathBuf>,
    #[serde(default)]
    pub dry_run: bool,
}

pub struct BatchOptions {
    pub file: Option<PathBuf>,
    pub continue_on_error: bool,
    pub dry_run: bool,
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

pub async fn run(rt: &Runtime, opts: &BatchOptions) -> Result<(), CliError> {
    let mut out = std::io::stdout();
    run_to(rt, opts, &mut out).await
}

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

fn error_code(e: &CliError) -> &'static str {
    match e {
        CliError::Config(_) => "config",
        CliError::Auth(_) => "auth",
        CliError::NotFound(_) => "not_found",
        CliError::Conflict(_) => "conflict",
        CliError::Network(_) => "network",
        CliError::Usage(_) => "usage",
        CliError::Partial(_) => "partial",
        CliError::Internal(_) => "internal",
    }
}

fn read_input(file: Option<&Path>) -> Result<String, CliError> {
    let path = file.ok_or_else(|| CliError::Usage("batch exige --file <JSONL|->".into()))?;
    if path == Path::new("-") {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| CliError::Usage(e.to_string()))?;
        Ok(buf)
    } else {
        std::fs::read_to_string(path)
            .map_err(|e| CliError::Config(format!("ler {}: {e}", path.display())))
    }
}

async fn execute(rt: &Runtime, op: &BatchOp, opts: &BatchOptions) -> Result<Value, CliError> {
    if rt.is_login() && op.account.is_some() {
        return Err(CliError::Usage(
            "--login não combina com account na linha do batch".into(),
        ));
    }
    if op.op == "health" {
        return Ok(json!({ "ok": true }));
    }
    let account = resolve_account(rt, op.account.as_deref())?;
    if opts.dry_run || op.dry_run {
        return Ok(
            json!({ "dry_run": true, "op": op.op, "account": account.id, "params": op.params }),
        );
    }
    let client = account
        .build_client(&rt.config, opts.fixed_time, &rt.cache)
        .await?;
    let net = |e: open_mtr_core::openmtr::error::OpenMTRError| CliError::Network(e.to_string());
    match op.op.as_str() {
        "account.test" => Ok(json!({ "ok": true, "account": account.id })),
        "mtr.consultar" => {
            let filtro = filtro_mtr_from_json(&op.params)?;
            to_value(client.consultar_mtrs(&filtro).await.map_err(net)?)
        }
        "mtr.criar" => {
            let items: Vec<CriarMtr> = parse_items(&op.params)?;
            to_value(client.criar_mtrs(items).await.map_err(net)?)
        }
        "mtr.receber" => {
            let items: Vec<ReceberMtr> = parse_items(&op.params)?;
            to_value(client.receber_mtrs(items).await.map_err(net)?)
        }
        "mtr.cancelar" => {
            let items: Vec<CancelarMtr> = parse_items(&op.params)?;
            to_value(client.cancelar_mtrs(items).await.map_err(net)?)
        }
        "mtr.modificar" => {
            let items: Vec<ModificarMtr> = parse_items(&op.params)?;
            to_value(client.modificar_mtrs(items).await.map_err(net)?)
        }
        "mtr.aceitar" => {
            let items: Vec<AceitarAlteracao> = parse_items(&op.params)?;
            to_value(client.aceitar_alteracoes(items).await.map_err(net)?)
        }
        "mtr.download" => download(&client, op).await,
        "cdf.consultar" => {
            let filtro = filtro_cdf_from_json(&op.params)?;
            to_value(client.consultar_cdfs(&filtro).await.map_err(net)?)
        }
        "cdf.gerar" => {
            let items: Vec<GerarCdf> = parse_items(&op.params)?;
            to_value(client.gerar_cdfs(items).await.map_err(net)?)
        }
        "cdf.download" => {
            let items: Vec<DownloadCdf> = parse_items(&op.params)?;
            let out = op
                .out
                .clone()
                .ok_or_else(|| CliError::Usage("cdf.download exige out".into()))?;
            let resp = client.download_cdfs(items).await.map_err(net)?;
            write_downloads(&out, resp.dados)
        }
        "ref.listar" => {
            let table = op
                .params
                .get("table")
                .and_then(Value::as_str)
                .ok_or_else(|| CliError::Usage("ref.listar exige params.table".into()))?;
            ref_table(&client, table).await
        }
        "ref.por-residuo" => {
            let codigo = op
                .params
                .get("codigo")
                .and_then(Value::as_str)
                .ok_or_else(|| CliError::Usage("ref.por-residuo exige params.codigo".into()))?;
            to_value(
                client
                    .listar_classes_por_residuo(codigo)
                    .await
                    .map_err(net)?,
            )
        }
        "ref.por-estado-fisico" => {
            let codigo = op
                .params
                .get("codigo")
                .and_then(Value::as_u64)
                .ok_or_else(|| {
                    CliError::Usage("ref.por-estado-fisico exige params.codigo inteiro".into())
                })?;
            let codigo = u32::try_from(codigo)
                .map_err(|_| CliError::Usage("params.codigo fora de u32".into()))?;
            to_value(
                client
                    .listar_acondicionamentos_por_estado_fisico(codigo)
                    .await
                    .map_err(net)?,
            )
        }
        other => Err(CliError::Usage(format!("op não suportada: {other}"))),
    }
}

fn to_value<T: serde::Serialize>(value: T) -> Result<Value, CliError> {
    serde_json::to_value(value).map_err(|e| CliError::Internal(e.to_string()))
}

fn write_downloads(out: &Path, items: Vec<DownloadItem>) -> Result<Value, CliError> {
    std::fs::create_dir_all(out).map_err(|e| CliError::Config(e.to_string()))?;
    let mut files = Vec::new();
    for item in items {
        let path = safe_path(out, &item.filename);
        std::fs::write(&path, &item.buffer).map_err(|e| CliError::Config(e.to_string()))?;
        files.push(path.display().to_string());
    }
    Ok(json!({ "files": files }))
}

async fn download(client: &OpenMtrClient, op: &BatchOp) -> Result<Value, CliError> {
    let numeros = string_array(&op.params, "numeros")?;
    let out = op
        .out
        .clone()
        .ok_or_else(|| CliError::Usage("mtr.download exige out".into()))?;
    let resp = client
        .download_mtrs(numeros)
        .await
        .map_err(|e| CliError::Network(e.to_string()))?;
    write_downloads(&out, resp.dados)
}

async fn ref_table(client: &OpenMtrClient, table: &str) -> Result<Value, CliError> {
    use clap::ValueEnum;

    fn to_value<T: serde::Serialize>(v: T) -> Result<Value, CliError> {
        serde_json::to_value(v).map_err(|e| CliError::Internal(e.to_string()))
    }
    let net = |e: open_mtr_core::openmtr::error::OpenMTRError| CliError::Network(e.to_string());
    let table = RefTable::from_str(table, true)
        .map_err(|_| CliError::Usage(format!("tabela desconhecida: {table}")))?;
    match table {
        RefTable::Classe => to_value(client.listar_classes().await.map_err(net)?),
        RefTable::Acondicionamento => {
            to_value(client.listar_acondicionamentos().await.map_err(net)?)
        }
        RefTable::EstadoFisico => to_value(client.listar_estados_fisicos().await.map_err(net)?),
        RefTable::Tratamento => to_value(client.listar_tratamentos().await.map_err(net)?),
        RefTable::Unidade => to_value(client.listar_unidades().await.map_err(net)?),
        RefTable::Residuo => to_value(client.listar_residuos().await.map_err(net)?),
    }
}

fn resolve_account<'a>(rt: &'a Runtime, id: Option<&str>) -> Result<&'a Account, CliError> {
    if let Some(id) = id {
        return rt
            .accounts
            .iter()
            .find(|a| a.id == id && a.enabled)
            .ok_or_else(|| CliError::NotFound(format!("conta não encontrada: {id}")));
    }
    let enabled: Vec<&Account> = rt.accounts.iter().filter(|a| a.enabled).collect();
    match enabled.as_slice() {
        [only] => Ok(*only),
        [] => Err(CliError::Config("nenhuma conta habilitada".into())),
        _ => Err(CliError::Usage(
            "linha sem account e múltiplas contas habilitadas".into(),
        )),
    }
}

#[allow(dead_code)]
fn _periodo(params: &Value) -> Result<Option<open_mtr_core::openmtr::domain::Periodo>, CliError> {
    periodo_from_json(params)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ref_table_accepts_cli_canonical_name() {
        use clap::ValueEnum;
        assert!(matches!(
            RefTable::from_str("estado-fisico", true),
            Ok(RefTable::EstadoFisico)
        ));
    }
}
