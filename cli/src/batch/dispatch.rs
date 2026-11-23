use super::io::{serialize_value, write_downloads};
use super::ops::{download, mtr_write_op, ref_table};
use super::{BatchOp, BatchOptions};
use crate::account::Account;
use crate::error::CliError;
use crate::params::{filtro_cdf_from_json, parse_items};
use crate::runtime::Runtime;
use open_mtr_core::openmtr::domain::{DownloadCdf, GerarCdf};
use open_mtr_core::openmtr::open_mtr_client::OpenMtrClient;
use serde_json::{Value, json};

pub(super) async fn execute(
    rt: &Runtime,
    op: &BatchOp,
    opts: &BatchOptions,
) -> Result<Value, CliError> {
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
    dispatch(&client, op, &account.id).await
}

async fn dispatch(
    client: &OpenMtrClient,
    op: &BatchOp,
    account_id: &str,
) -> Result<Value, CliError> {
    let net = |e: open_mtr_core::openmtr::error::OpenMTRError| CliError::Network(e.to_string());
    match op.op.as_str() {
        "account.test" => Ok(json!({ "ok": true, "account": account_id })),
        "mtr.consultar" | "mtr.criar" | "mtr.receber" | "mtr.cancelar" | "mtr.modificar"
        | "mtr.aceitar" => mtr_write_op(client, op).await,
        "mtr.download" => download(client, op).await,
        "cdf.consultar" => {
            let filtro = filtro_cdf_from_json(&op.params)?;
            Ok(serialize_value(
                client.consultar_cdfs(&filtro).await.map_err(net)?,
            ))
        }
        "cdf.gerar" => {
            let items: Vec<GerarCdf> = parse_items(&op.params)?;
            Ok(serialize_value(
                client.gerar_cdfs(items).await.map_err(net)?,
            ))
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
            ref_table(client, table).await
        }
        "ref.por-residuo" => {
            let codigo = op
                .params
                .get("codigo")
                .and_then(Value::as_str)
                .ok_or_else(|| CliError::Usage("ref.por-residuo exige params.codigo".into()))?;
            Ok(serialize_value(
                client
                    .listar_classes_por_residuo(codigo)
                    .await
                    .map_err(net)?,
            ))
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
            Ok(serialize_value(
                client
                    .listar_acondicionamentos_por_estado_fisico(codigo)
                    .await
                    .map_err(net)?,
            ))
        }
        other => Err(CliError::Usage(format!("op não suportada: {other}"))),
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
