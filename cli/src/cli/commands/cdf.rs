use super::find_account;
use super::io::{print_one_or_many, write_downloads};
use crate::account::AccountScope;
use crate::error::CliError;
use crate::output::OutputFormat;
use crate::params::{filtro_cdf_from_json, parse_items, parse_params};
use crate::runtime::Runtime;
use jiff::Timestamp;
use open_mtr_core::openmtr::domain::{DownloadCdf, GerarCdf};

use crate::cli::CdfCmd;

pub(super) async fn run(
    rt: &Runtime,
    sub: CdfCmd,
    fixed_time: Option<Timestamp>,
    format: OutputFormat,
) -> Result<(), CliError> {
    match sub {
        CdfCmd::Consultar {
            codigo,
            account,
            params,
        } => {
            let value = match (codigo, params) {
                (Some(codigo), None) => serde_json::json!({ "codigos": [codigo] }),
                (None, Some(raw)) => parse_params(Some(&raw))?,
                (None, None) => {
                    return Err(CliError::Usage(
                        "cdf consultar exige <CODIGO> ou --params <JSON|->".into(),
                    ));
                }
                (Some(_), Some(_)) => {
                    return Err(CliError::Usage(
                        "use <CODIGO> ou --params, não ambos".into(),
                    ));
                }
            };
            let ids: Vec<String> = account.into_iter().collect();
            let sel = rt.select(AccountScope::Ids(&ids))?;
            if sel.is_empty() {
                return Err(CliError::Usage("escopo de conta obrigatório".into()));
            }
            let filtro = filtro_cdf_from_json(&value)?;
            let mut results = Vec::new();
            for acc in sel {
                let client = acc.build_client(&rt.config, fixed_time, &rt.cache).await?;
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
            print_one_or_many(format, results)
        }
        CdfCmd::Gerar { account, params } => {
            gerar(rt, account.as_deref(), &params, fixed_time, format).await
        }
        CdfCmd::Download {
            account,
            params,
            out,
        } => download(rt, account.as_deref(), &params, fixed_time, &out, format).await,
    }
}

async fn gerar(
    rt: &Runtime,
    account_id: Option<&str>,
    params: &str,
    fixed_time: Option<Timestamp>,
    format: OutputFormat,
) -> Result<(), CliError> {
    let account = find_account(rt, account_id)?;
    let value = parse_params(Some(params))?;
    let items: Vec<GerarCdf> = parse_items(&value)?;
    let client = account
        .build_client(&rt.config, fixed_time, &rt.cache)
        .await?;
    let res = client
        .gerar_cdfs(items)
        .await
        .map_err(|e| CliError::Network(e.to_string()))?;
    super::io::print_value(format, &res)
}

async fn download(
    rt: &Runtime,
    account_id: Option<&str>,
    params: &str,
    fixed_time: Option<Timestamp>,
    out: &std::path::Path,
    format: OutputFormat,
) -> Result<(), CliError> {
    let account = find_account(rt, account_id)?;
    let value = parse_params(Some(params))?;
    let items: Vec<DownloadCdf> = parse_items(&value)?;
    let client = account
        .build_client(&rt.config, fixed_time, &rt.cache)
        .await?;
    let resp = client
        .download_cdfs(items)
        .await
        .map_err(|e| CliError::Network(e.to_string()))?;
    write_downloads(resp.dados, out, format)
}
