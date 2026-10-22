use super::internal;
use crate::error::CliError;
use crate::normalize::{CSV_HEADER, DumpRow, csv_line, escape_csv_field};
use std::path::Path;

pub(super) fn write_jsonl(path: &Path, rows: &[DumpRow]) -> Result<(), CliError> {
    let mut body = String::new();
    for row in rows {
        body.push_str(&serde_json::to_string(row).map_err(internal)?);
        body.push('\n');
    }
    std::fs::write(path, body)
        .map_err(|e| CliError::Config(format!("escrever {}: {e}", path.display())))
}

pub(super) fn write_csv(path: &Path, rows: &[DumpRow]) -> Result<(), CliError> {
    let mut body = String::from(CSV_HEADER);
    body.push('\n');
    for row in rows {
        body.push_str(&csv_line(row));
        body.push('\n');
    }
    std::fs::write(path, body)
        .map_err(|e| CliError::Config(format!("escrever {}: {e}", path.display())))
}

const RESIDUOS_HEADER: &str = "account,system,numero,idx,codigo_ibama,descricao,quantidade,quantidade_recebida,unidade,estado_fisico,classe,acondicionamento,tratamento";

#[allow(
    clippy::excessive_nesting,
    reason = "linha -> resíduo -> campo; formato CSV fixo"
)]
pub(super) fn write_residuos_csv(path: &Path, rows: &[DumpRow]) -> Result<(), CliError> {
    let mut body = String::from(RESIDUOS_HEADER);
    body.push('\n');
    for row in rows {
        let residuos: Vec<serde_json::Value> =
            serde_json::from_str(&row.residuos_json).unwrap_or_default();
        for (idx, r) in residuos.iter().enumerate() {
            let field = |name: &str| -> String {
                match r.get(name) {
                    Some(serde_json::Value::String(s)) => s.clone(),
                    Some(serde_json::Value::Number(n)) => n.to_string(),
                    Some(serde_json::Value::Bool(b)) => b.to_string(),
                    _ => String::new(),
                }
            };
            let line = [
                row.account.clone(),
                row.system.clone(),
                row.numero.clone(),
                idx.to_string(),
                field("codigo_ibama"),
                field("descricao"),
                field("quantidade"),
                field("quantidade_recebida"),
                field("unidade"),
                field("estado_fisico"),
                field("classe"),
                field("acondicionamento"),
                field("tratamento"),
            ]
            .iter()
            .map(|f| escape_csv_field(f))
            .collect::<Vec<_>>()
            .join(",");
            body.push_str(&line);
            body.push('\n');
        }
    }
    std::fs::write(path, body)
        .map_err(|e| CliError::Config(format!("escrever {}: {e}", path.display())))
}
