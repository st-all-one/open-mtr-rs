use super::find_account;
use super::io::{print_one_or_many, write_downloads};
use crate::error::CliError;
use crate::params::{filtro_cdf_from_json, parse_items, parse_params};
use crate::runtime::Runtime;
use chrono::{DateTime, Utc};
use open_mtr_core::openmtr::domain::{DownloadCdf, GerarCdf};

use crate::cli::CdfCmd;

pub(super) async fn run(
    rt: &Runtime,
    sub: CdfCmd,
    fixed_time: Option<DateTime<Utc>>,
    json: bool,
) -> Result<(), CliError> {
    match sub {
        CdfCmd::Consultar { account, params } => {
            let ids: Vec<String> = account.into_iter().collect();
            let sel = rt.resolve(&ids, None, &[], false)?;
            if sel.is_empty() {
                return Err(CliError::Usage("escopo de conta obrigatório".into()));
            }
            let value = parse_params(params.as_deref())?;
            let filtro = filtro_cdf_from_json(&value)?;
            let mut results = Vec::new();
            for acc in sel {
                let client = acc.build_client(&rt.config, fixed_time).await?;
                let res = client
                    .consultar_cdfs(&filtro)
                    .await
                    .map_err(|e| CliError::Network(e.to_string()))?;
                let data =
                    serde_json::to_value(res).map_err(|e| CliError::Internal(e.to_string()))?;
                results.push(serde_json::json!({
                    "account": acc.id,
                    "system": acc.system_slug(),
                    "data": data,
                }));
            }
            print_one_or_many(json, results)
        }
        CdfCmd::Gerar { account, params } => gerar(rt, &account, params, fixed_time, json).await,
        CdfCmd::Download {
            account,
            params,
            out,
        } => download(rt, &account, params, fixed_time, &out, json).await,
    }
}

async fn gerar(
    rt: &Runtime,
    account_id: &str,
    params: Option<String>,
    fixed_time: Option<DateTime<Utc>>,
    json: bool,
) -> Result<(), CliError> {
    let account = find_account(rt, account_id)?;
    let value = parse_params(params.as_deref())?;
    let items: Vec<GerarCdf> = parse_items(&value)?;
    let client = account.build_client(&rt.config, fixed_time).await?;
    let res = client
        .gerar_cdfs(items)
        .await
        .map_err(|e| CliError::Network(e.to_string()))?;
    super::io::print_value(json, &res)
}

async fn download(
    rt: &Runtime,
    account_id: &str,
    params: Option<String>,
    fixed_time: Option<DateTime<Utc>>,
    out: &std::path::Path,
    json: bool,
) -> Result<(), CliError> {
    let account = find_account(rt, account_id)?;
    let value = parse_params(params.as_deref())?;
    let items: Vec<DownloadCdf> = parse_items(&value)?;
    let client = account.build_client(&rt.config, fixed_time).await?;
    let resp = client
        .download_cdfs(items)
        .await
        .map_err(|e| CliError::Network(e.to_string()))?;
    write_downloads(resp.dados, out, json)
}
