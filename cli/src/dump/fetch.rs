use crate::account::Account;
use crate::error::CliError;
use crate::normalize::DumpRow;
use chrono::NaiveDate;
use open_mtr_core::openmtr::domain::newtypes::DataMtr;
use open_mtr_core::openmtr::domain::{FiltroMtr, Periodo};
use open_mtr_core::openmtr::open_mtr_client::OpenMtrClient;

pub(super) async fn fetch_window(
    client: &OpenMtrClient,
    account: &Account,
    start: NaiveDate,
    end: NaiveDate,
) -> Result<(Vec<DumpRow>, Vec<String>), CliError> {
    let periodo =
        Periodo::new(DataMtr(start), DataMtr(end)).map_err(|e| CliError::Usage(e.to_string()))?;
    let mut out = Vec::new();
    let mut item_errors = Vec::new();
    let system = account.system_slug().to_string();
    let mut page = 1u32;
    loop {
        let filtro = FiltroMtr {
            periodo: Some(periodo.clone()),
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
            out.push(DumpRow::from_mtr(
                &account.id,
                &system,
                account.access_priority,
                mtr,
            ));
        }
        let reached_total =
            total.is_some_and(|t| i64::try_from(out.len()).unwrap_or(i64::MAX) >= t);
        if len < 500
            || !matches!(
                account.system,
                open_mtr_core::openmtr::system::System::Sinir
            )
            || page >= 1000
            || reached_total
        {
            break;
        }
        page += 1;
    }
    Ok((out, item_errors))
}
