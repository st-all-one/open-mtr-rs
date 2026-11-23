use super::BatchOp;
use super::io::{serialize_value, write_downloads};
use crate::cli::RefTable;
use crate::error::CliError;
use crate::params::{filtro_mtr_from_json, parse_items, string_array};
use open_mtr_core::openmtr::domain::{
    AceitarAlteracao, CancelarMtr, CriarMtr, ModificarMtr, ReceberMtr,
};
use open_mtr_core::openmtr::open_mtr_client::OpenMtrClient;
use serde_json::Value;

pub(super) async fn mtr_write_op(client: &OpenMtrClient, op: &BatchOp) -> Result<Value, CliError> {
    let net = |e: open_mtr_core::openmtr::error::OpenMTRError| CliError::Network(e.to_string());
    match op.op.as_str() {
        "mtr.consultar" => {
            let filtro = filtro_mtr_from_json(&op.params)?;
            Ok(serialize_value(
                client.consultar_mtrs(&filtro).await.map_err(net)?,
            ))
        }
        "mtr.criar" => {
            let items: Vec<CriarMtr> = parse_items(&op.params)?;
            Ok(serialize_value(
                client.criar_mtrs(items).await.map_err(net)?,
            ))
        }
        "mtr.receber" => {
            let items: Vec<ReceberMtr> = parse_items(&op.params)?;
            Ok(serialize_value(
                client.receber_mtrs(items).await.map_err(net)?,
            ))
        }
        "mtr.cancelar" => {
            let items: Vec<CancelarMtr> = parse_items(&op.params)?;
            Ok(serialize_value(
                client.cancelar_mtrs(items).await.map_err(net)?,
            ))
        }
        "mtr.modificar" => {
            let items: Vec<ModificarMtr> = parse_items(&op.params)?;
            Ok(serialize_value(
                client.modificar_mtrs(items).await.map_err(net)?,
            ))
        }
        "mtr.aceitar" => {
            let items: Vec<AceitarAlteracao> = parse_items(&op.params)?;
            Ok(serialize_value(
                client.aceitar_alteracoes(items).await.map_err(net)?,
            ))
        }
        other => Err(CliError::Usage(format!("op não suportada: {other}"))),
    }
}

pub(super) async fn download(client: &OpenMtrClient, op: &BatchOp) -> Result<Value, CliError> {
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

pub(super) async fn ref_table(client: &OpenMtrClient, table: &str) -> Result<Value, CliError> {
    use clap::ValueEnum;

    let net = |e: open_mtr_core::openmtr::error::OpenMTRError| CliError::Network(e.to_string());
    let table = RefTable::from_str(table, true)
        .map_err(|_| CliError::Usage(format!("tabela desconhecida: {table}")))?;
    match table {
        RefTable::Classe => Ok(serialize_value(client.listar_classes().await.map_err(net)?)),
        RefTable::Acondicionamento => Ok(serialize_value(
            client.listar_acondicionamentos().await.map_err(net)?,
        )),
        RefTable::EstadoFisico => Ok(serialize_value(
            client.listar_estados_fisicos().await.map_err(net)?,
        )),
        RefTable::Tratamento => Ok(serialize_value(
            client.listar_tratamentos().await.map_err(net)?,
        )),
        RefTable::Unidade => Ok(serialize_value(
            client.listar_unidades().await.map_err(net)?,
        )),
        RefTable::Residuo => Ok(serialize_value(
            client.listar_residuos().await.map_err(net)?,
        )),
    }
}
