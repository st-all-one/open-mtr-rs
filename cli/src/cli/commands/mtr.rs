use super::find_account;
use super::io::{print_one_or_many, write_downloads};
use crate::error::CliError;
use crate::params::{filtro_mtr_from_json, parse_items, parse_params, string_array};
use crate::runtime::Runtime;
use chrono::{DateTime, Utc};
use open_mtr_core::openmtr::domain::{
    AceitarAlteracao, CancelarMtr, CriarMtr, ModificarMtr, ReceberMtr,
};

use crate::cli::MtrCmd;

pub(super) async fn run(
    rt: &Runtime,
    sub: MtrCmd,
    fixed_time: Option<DateTime<Utc>>,
    json: bool,
) -> Result<(), CliError> {
    match sub {
        MtrCmd::Consultar {
            numero,
            account,
            system,
            tag,
            all_accounts,
            params,
        } => {
            consultar(
                rt,
                numero,
                account,
                system,
                tag,
                all_accounts,
                params,
                fixed_time,
                json,
            )
            .await
        }
        MtrCmd::Criar { account, params } => {
            write(
                rt,
                account.as_deref(),
                &params,
                fixed_time,
                WriteKind::Criar,
                json,
            )
            .await
        }
        MtrCmd::Receber { account, params } => {
            write(
                rt,
                account.as_deref(),
                &params,
                fixed_time,
                WriteKind::Receber,
                json,
            )
            .await
        }
        MtrCmd::Cancelar { account, params } => {
            write(
                rt,
                account.as_deref(),
                &params,
                fixed_time,
                WriteKind::Cancelar,
                json,
            )
            .await
        }
        MtrCmd::Modificar { account, params } => {
            write(
                rt,
                account.as_deref(),
                &params,
                fixed_time,
                WriteKind::Modificar,
                json,
            )
            .await
        }
        MtrCmd::Aceitar { account, params } => {
            write(
                rt,
                account.as_deref(),
                &params,
                fixed_time,
                WriteKind::Aceitar,
                json,
            )
            .await
        }
        MtrCmd::Download {
            numero,
            account,
            params,
            out,
        } => {
            download(
                rt,
                numero,
                account.as_deref(),
                params,
                fixed_time,
                &out,
                json,
            )
            .await
        }
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "espelha as flags do verbo consultar"
)]
async fn consultar(
    rt: &Runtime,
    numero: Option<String>,
    account: Option<String>,
    system: Option<String>,
    tag: Vec<String>,
    all_accounts: bool,
    params: Option<String>,
    fixed_time: Option<DateTime<Utc>>,
    json: bool,
) -> Result<(), CliError> {
    let value = match (numero, params) {
        (Some(numero), None) => serde_json::json!({ "numeros": [numero] }),
        (None, Some(raw)) => parse_params(Some(&raw))?,
        (None, None) => {
            return Err(CliError::Usage(
                "mtr consultar exige <NUMERO> ou --params <JSON|->".into(),
            ));
        }
        (Some(_), Some(_)) => {
            return Err(CliError::Usage(
                "use <NUMERO> ou --params, não ambos".into(),
            ));
        }
    };
    let ids: Vec<String> = account.into_iter().collect();
    let sel = rt.select(&ids, system.as_deref(), &tag, all_accounts)?;
    if sel.is_empty() {
        return Err(CliError::Usage("escopo de conta obrigatório".into()));
    }
    let filtro = filtro_mtr_from_json(&value)?;
    let mut results = Vec::new();
    for acc in sel {
        let client = acc.build_client(&rt.config, fixed_time, &rt.cache).await?;
        let res = client
            .consultar_mtrs(&filtro)
            .await
            .map_err(|e| CliError::Network(e.to_string()))?;
        let data = serde_json::to_value(res).map_err(|e| CliError::Internal(e.to_string()))?;
        results.push(serde_json::json!({
            "account": acc.id,
            "system": acc.system_slug(),
            "data": data,
        }));
    }
    print_one_or_many(json, results)
}

#[derive(Clone, Copy)]
enum WriteKind {
    Criar,
    Receber,
    Cancelar,
    Modificar,
    Aceitar,
}

async fn write(
    rt: &Runtime,
    account_id: Option<&str>,
    params: &str,
    fixed_time: Option<DateTime<Utc>>,
    kind: WriteKind,
    json: bool,
) -> Result<(), CliError> {
    let account = find_account(rt, account_id)?;
    let value = parse_params(Some(params))?;
    let client = account
        .build_client(&rt.config, fixed_time, &rt.cache)
        .await?;
    let net = |e: open_mtr_core::openmtr::error::OpenMTRError| CliError::Network(e.to_string());
    let result = match kind {
        WriteKind::Criar => {
            let items: Vec<CriarMtr> = parse_items(&value)?;
            serde_json::to_value(client.criar_mtrs(items).await.map_err(net)?)
        }
        WriteKind::Receber => {
            let items: Vec<ReceberMtr> = parse_items(&value)?;
            serde_json::to_value(client.receber_mtrs(items).await.map_err(net)?)
        }
        WriteKind::Cancelar => {
            let items: Vec<CancelarMtr> = parse_items(&value)?;
            serde_json::to_value(client.cancelar_mtrs(items).await.map_err(net)?)
        }
        WriteKind::Modificar => {
            let items: Vec<ModificarMtr> = parse_items(&value)?;
            serde_json::to_value(client.modificar_mtrs(items).await.map_err(net)?)
        }
        WriteKind::Aceitar => {
            let items: Vec<AceitarAlteracao> = parse_items(&value)?;
            serde_json::to_value(client.aceitar_alteracoes(items).await.map_err(net)?)
        }
    }
    .map_err(|e| CliError::Internal(e.to_string()))?;
    super::io::print_value(json, &result)
}

async fn download(
    rt: &Runtime,
    numero: Option<String>,
    account_id: Option<&str>,
    params: Option<String>,
    fixed_time: Option<DateTime<Utc>>,
    out: &std::path::Path,
    json: bool,
) -> Result<(), CliError> {
    let numeros = match (numero, params) {
        (Some(numero), None) => vec![numero],
        (None, Some(raw)) => string_array(&parse_params(Some(&raw))?, "numeros")?,
        (None, None) => {
            return Err(CliError::Usage(
                "mtr download exige <NUMERO> ou --params <JSON|-> com `numeros`".into(),
            ));
        }
        (Some(_), Some(_)) => {
            return Err(CliError::Usage(
                "use <NUMERO> ou --params, não ambos".into(),
            ));
        }
    };
    let account = find_account(rt, account_id)?;
    let client = account
        .build_client(&rt.config, fixed_time, &rt.cache)
        .await?;
    let resp = client
        .download_mtrs(numeros)
        .await
        .map_err(|e| CliError::Network(e.to_string()))?;
    write_downloads(resp.dados, out, json)
}
