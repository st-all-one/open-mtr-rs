use super::single_account;
use crate::cli::{RefCmd, RefTable};
use crate::error::CliError;
use crate::runtime::Runtime;
use chrono::{DateTime, Utc};
use open_mtr_core::openmtr::error::OpenMTRError;
use open_mtr_core::openmtr::open_mtr_client::OpenMtrClient;

pub(super) async fn run(
    rt: &Runtime,
    sub: RefCmd,
    fixed_time: Option<DateTime<Utc>>,
    json: bool,
) -> Result<(), CliError> {
    match sub {
        RefCmd::Listar { table, account } => {
            let client = client_for(rt, account, fixed_time).await?;
            list_table(&client, table, json).await
        }
        RefCmd::PorResiduo { codigo, account } => {
            let client = client_for(rt, account, fixed_time).await?;
            let res = client
                .listar_classes_por_residuo(&codigo)
                .await
                .map_err(net)?;
            super::io::print_value(json, &res)
        }
        RefCmd::PorEstadoFisico { codigo, account } => {
            let codigo = codigo.parse::<u32>().map_err(|_| {
                CliError::Usage("CODIGO do estado físico deve ser um inteiro".into())
            })?;
            let client = client_for(rt, account, fixed_time).await?;
            let res = client
                .listar_acondicionamentos_por_estado_fisico(codigo)
                .await
                .map_err(net)?;
            super::io::print_value(json, &res)
        }
    }
}

async fn client_for(
    rt: &Runtime,
    account: Option<String>,
    fixed_time: Option<DateTime<Utc>>,
) -> Result<OpenMtrClient, CliError> {
    let ids: Vec<String> = account.into_iter().collect();
    let sel = rt.select(&ids, None, &[], false)?;
    let acc = single_account(&sel)?;
    acc.build_client(&rt.config, fixed_time, &rt.cache).await
}

fn net(error: OpenMTRError) -> CliError {
    CliError::Network(error.to_string())
}

async fn list_table(client: &OpenMtrClient, table: RefTable, json: bool) -> Result<(), CliError> {
    macro_rules! emit {
        ($future:expr) => {{
            let res = $future.await.map_err(net)?;
            super::io::print_value(json, &res)
        }};
    }
    match table {
        RefTable::Residuo => emit!(client.listar_residuos()),
        RefTable::Classe => emit!(client.listar_classes()),
        RefTable::Unidade => emit!(client.listar_unidades()),
        RefTable::Acondicionamento => emit!(client.listar_acondicionamentos()),
        RefTable::EstadoFisico => emit!(client.listar_estados_fisicos()),
        RefTable::Tratamento => emit!(client.listar_tratamentos()),
    }
}
