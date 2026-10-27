use crate::account::Account;
use crate::error::CliError;
use crate::normalize::DumpRow;
use chrono::NaiveDate;
use open_mtr_core::openmtr::domain::newtypes::DataMtr;
use open_mtr_core::openmtr::domain::{FiltroMtr, FiltroParte, Periodo, TipoParte};
use open_mtr_core::openmtr::open_mtr_client::OpenMtrClient;
use open_mtr_core::openmtr::system::System;
use std::collections::BTreeMap;

pub(super) async fn fetch_classes(client: &OpenMtrClient) -> BTreeMap<u32, String> {
    match client.listar_classes().await {
        Ok(resp) => resp
            .dados
            .into_iter()
            .map(|c| (c.codigo, c.descricao.to_uppercase()))
            .collect(),
        Err(e) => {
            tracing::warn!(error = %e, "dump: tabela de classes indisponível; usando códigos");
            BTreeMap::new()
        }
    }
}

pub(super) async fn fetch_window(
    client: &OpenMtrClient,
    account: &Account,
    start: NaiveDate,
    end: NaiveDate,
    classes: &BTreeMap<u32, String>,
) -> Result<(Vec<DumpRow>, Vec<String>), CliError> {
    // O endpoint `listaManifesto` filtra por `tipoParceiro`; sem os quatro
    // papéis o dump só traz os manifestos em que a conta é GERADORA (default 8)
    // e perde transportador/destinador/armazenador.
    let papeis: Vec<Option<TipoParte>> = if matches!(account.system, System::Sinir) {
        vec![
            Some(TipoParte::Gerador),
            Some(TipoParte::Transportador),
            Some(TipoParte::Destinador),
            Some(TipoParte::Armazenador),
        ]
    } else {
        vec![None]
    };

    let mut out = Vec::new();
    let mut item_errors = Vec::new();
    for papel in &papeis {
        let (mut rows, mut errors) =
            fetch_role(client, account, start, end, papel.clone(), classes).await?;
        out.append(&mut rows);
        item_errors.append(&mut errors);
    }
    Ok((out, item_errors))
}

async fn fetch_role(
    client: &OpenMtrClient,
    account: &Account,
    start: NaiveDate,
    end: NaiveDate,
    papel: Option<TipoParte>,
    classes: &BTreeMap<u32, String>,
) -> Result<(Vec<DumpRow>, Vec<String>), CliError> {
    let periodo =
        Periodo::new(DataMtr(start), DataMtr(end)).map_err(|e| CliError::Usage(e.to_string()))?;
    let parte = papel.map(|tipo| FiltroParte {
        tipo,
        ..FiltroParte::default()
    });
    let mut out = Vec::new();
    let mut item_errors = Vec::new();
    let system = account.system_slug().to_string();
    let mut page = 1u32;
    loop {
        let filtro = FiltroMtr {
            periodo: Some(periodo.clone()),
            parte: parte.clone(),
            detalhado: true,
            pagina: Some(page),
            tamanho: Some(500),
            ..FiltroMtr::default()
        };
        let resp = client
            .consultar_mtrs(&filtro)
            .await
            .map_err(|e| CliError::Network(e.to_string()))?;
        let len = resp.dados.len();
        let total = resp.total;
        item_errors.extend(resp.erros.iter().map(ToString::to_string));
        for mtr in resp.dados {
            out.push(DumpRow::from_mtr_with_classes(
                &account.id,
                &system,
                account.access_priority,
                mtr,
                classes,
            ));
        }
        let reached_total =
            total.is_some_and(|t| i64::try_from(out.len()).unwrap_or(i64::MAX) >= t);
        if len < 500 || !matches!(account.system, System::Sinir) || page >= 1000 || reached_total {
            break;
        }
        page += 1;
    }
    Ok((out, item_errors))
}
