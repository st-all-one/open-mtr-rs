use super::single_account;
use crate::error::CliError;
use crate::runtime::Runtime;
use chrono::{DateTime, Utc};
use open_mtr_core::openmtr::open_mtr_client::OpenMtrClient;

use crate::cli::RefCmd;

pub(super) async fn run(
    rt: &Runtime,
    sub: RefCmd,
    fixed_time: Option<DateTime<Utc>>,
    json: bool,
) -> Result<(), CliError> {
    match sub {
        RefCmd::Listar { account, table } => {
            let ids: Vec<String> = account.into_iter().collect();
            let sel = rt.resolve(&ids, None, &[], false)?;
            let acc = single_account(&sel)?;
            let client = acc.build_client(&rt.config, fixed_time).await?;
            list_table(&client, &table, json).await
        }
    }
}

async fn list_table(client: &OpenMtrClient, table: &str, json: bool) -> Result<(), CliError> {
    let net = |e: open_mtr_core::openmtr::error::OpenMTRError| CliError::Network(e.to_string());
    macro_rules! emit {
        ($r:expr) => {{
            let r = $r.map_err(net)?;
            super::io::print_value(json, &r)
        }};
    }
    match table {
        "classe" | "classes" => emit!(client.listar_classes().await),
        "acondicionamento" | "acondicionamentos" => emit!(client.listar_acondicionamentos().await),
        "estado_fisico" | "estados_fisicos" => emit!(client.listar_estados_fisicos().await),
        "tratamento" | "tratamentos" => emit!(client.listar_tratamentos().await),
        "unidade" | "unidades" => emit!(client.listar_unidades().await),
        "residuo" | "residuos" => emit!(client.listar_residuos().await),
        _ => Err(CliError::Usage(format!("tabela desconhecida: {table}"))),
    }
}
