use crate::account::Account;
use crate::error::CliError;
use crate::params::{filtro_cdf_from_json, filtro_mtr_from_json, periodo_from_json};
use crate::runtime::Runtime;
use chrono::{DateTime, Utc};
use open_mtr_core::openmtr::open_mtr_client::OpenMtrClient;
use serde::Deserialize;
use serde_json::{Value, json};
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
    "mtr.download",
    "cdf.consultar",
    "ref.listar",
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
    match file {
        Some(path) => std::fs::read_to_string(path)
            .map_err(|e| CliError::Config(format!("ler {}: {e}", path.display()))),
        None => {
            use std::io::Read;
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| CliError::Usage(e.to_string()))?;
            Ok(buf)
        }
    }
}

async fn execute(rt: &Runtime, op: &BatchOp, opts: &BatchOptions) -> Result<Value, CliError> {
    if op.op == "health" {
        return Ok(json!({ "ok": true }));
    }
    let account = resolve_account(rt, op.account.as_deref())?;
    if opts.dry_run || op.dry_run {
        return Ok(
            json!({ "dry_run": true, "op": op.op, "account": account.id, "params": op.params }),
        );
    }
    let client = account.build_client(&rt.config, opts.fixed_time).await?;
    match op.op.as_str() {
        "account.test" => Ok(json!({ "ok": true, "account": account.id })),
        "mtr.consultar" => {
            let filtro = filtro_mtr_from_json(&op.params)?;
            let resp = client
                .consultar_mtrs(&filtro)
                .await
                .map_err(|e| CliError::Network(e.to_string()))?;
            Ok(serde_json::to_value(resp).map_err(|e| CliError::Internal(e.to_string()))?)
        }
        "mtr.download" => download(&client, op).await,
        "cdf.consultar" => {
            let filtro = filtro_cdf_from_json(&op.params)?;
            let resp = client
                .consultar_cdfs(&filtro)
                .await
                .map_err(|e| CliError::Network(e.to_string()))?;
            Ok(serde_json::to_value(resp).map_err(|e| CliError::Internal(e.to_string()))?)
        }
        "ref.listar" => {
            let table = op
                .params
                .get("table")
                .and_then(Value::as_str)
                .ok_or_else(|| CliError::Usage("ref.listar exige params.table".into()))?;
            ref_table(&client, table).await
        }
        other => Err(CliError::Usage(format!("op não suportada: {other}"))),
    }
}

async fn download(client: &OpenMtrClient, op: &BatchOp) -> Result<Value, CliError> {
    let numeros: Vec<String> = op
        .params
        .get("numeros")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .ok_or_else(|| CliError::Usage("mtr.download exige params.numeros".into()))?;
    let out = op
        .out
        .clone()
        .ok_or_else(|| CliError::Usage("mtr.download exige out".into()))?;
    std::fs::create_dir_all(&out).map_err(|e| CliError::Config(e.to_string()))?;
    let resp = client
        .download_mtrs(numeros)
        .await
        .map_err(|e| CliError::Network(e.to_string()))?;
    let mut files = Vec::new();
    for item in resp.dados {
        let path = out.join(&item.filename);
        std::fs::write(&path, &item.buffer).map_err(|e| CliError::Config(e.to_string()))?;
        files.push(path.display().to_string());
    }
    Ok(json!({ "files": files }))
}

async fn ref_table(client: &OpenMtrClient, table: &str) -> Result<Value, CliError> {
    fn to_value<T: serde::Serialize>(v: T) -> Result<Value, CliError> {
        serde_json::to_value(v).map_err(|e| CliError::Internal(e.to_string()))
    }
    let net = |e: open_mtr_core::openmtr::error::OpenMTRError| CliError::Network(e.to_string());
    match table {
        "classe" | "classes" => to_value(client.listar_classes().await.map_err(net)?),
        "acondicionamento" | "acondicionamentos" => {
            to_value(client.listar_acondicionamentos().await.map_err(net)?)
        }
        "estado_fisico" | "estados_fisicos" => {
            to_value(client.listar_estados_fisicos().await.map_err(net)?)
        }
        "tratamento" | "tratamentos" => to_value(client.listar_tratamentos().await.map_err(net)?),
        "unidade" | "unidades" => to_value(client.listar_unidades().await.map_err(net)?),
        "residuo" | "residuos" => to_value(client.listar_residuos().await.map_err(net)?),
        _ => Err(CliError::Usage(format!("tabela desconhecida: {table}"))),
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
