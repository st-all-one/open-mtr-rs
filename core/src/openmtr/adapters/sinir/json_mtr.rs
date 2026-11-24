//! Conversões de JSON do SINIR para MTRs (listagem e detalhe).

use jiff::{Timestamp, civil::Date, fmt::strtime, tz::TimeZone};
use serde_json::Value;

use crate::openmtr::domain::{Mtr, Residuo};

use super::json_residuos::{json_parte, json_residuo, json_residuo_detalhe, sinir_status};

/// Converte o JSON detalhado de um manifesto em `Mtr`.
pub(super) fn json_mtr_detalhe(v: &Value) -> Mtr {
    let data_criacao = v
        .get("manData")
        .and_then(Value::as_i64)
        .and_then(data_de_millis);

    let data_expedicao = v
        .get("manDataExpedicao")
        .and_then(Value::as_i64)
        .and_then(data_de_millis);

    let status_str = v
        .get("situacaoManifesto")
        .and_then(|s| s.get("simDescricao"))
        .and_then(|s| s.as_str())
        .unwrap_or_default();

    let residuos: Vec<Residuo> = v
        .get("listaManifestoResiduo")
        .and_then(|a| a.as_array())
        .map_or_default(|arr| arr.iter().map(json_residuo_detalhe).collect());

    Mtr {
        numero: v
            .get("manNumero")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
        barcode: String::new(),
        status: sinir_status(status_str),
        gerador: v.get("parceiroGerador").map(json_parte),
        transportador: v.get("parceiroTransportador").map(json_parte),
        destinador: v.get("parceiroDestinador").map(json_parte),
        residuos,
        data_criacao,
        data_recebimento: v
            .get("dataRecebimentoAT")
            .and_then(|s| s.as_str())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                // Recebimento efetivo do manifestos vem em situacaoManifesto.simDataRecebimento
                v.get("situacaoManifesto")
                    .and_then(|s| s.get("simDataRecebimento"))
                    .and_then(|s| s.as_str())
            })
            .and_then(|s| {
                if s.is_empty() {
                    None
                } else {
                    strtime::parse("%d/%m/%Y", s).ok()?.to_date().ok()
                }
            }),
        observacao: v
            .get("manObservacao")
            .and_then(|s| s.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        motorista: v
            .get("manNomeMotorista")
            .and_then(|s| s.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        placa: v
            .get("manPlacaVeiculo")
            .and_then(|s| s.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        tipo_manifesto: v
            .get("tipoManifesto")
            .and_then(Value::as_i64)
            .map(|n| n as i32),
        estado_uf: v
            .get("estado")
            .and_then(|e| e.get("estAbreviacao"))
            .and_then(|s| s.as_str())
            .map(String::from),
        data_expedicao,
        armazenamento_temporario: v
            .get("possuiArmazenamentoTemporario")
            .and_then(Value::as_bool),
        hash: v
            .get("manHashCode")
            .and_then(|s| s.as_str())
            .map(String::from),
        tem_alteracao_pendente: v
            .get("temSolicitacaoAlteracaoRecebimento")
            .and_then(Value::as_bool),
        numero_estadual: v
            .get("manNumeroEstadual")
            .and_then(|s| s.as_str())
            .map(String::from)
            .filter(|s| !s.is_empty()),
        codigo_estadual: v.get("manCodigoEstadual").and_then(Value::as_i64),
    }
}

/// Converte o JSON resumido de um manifesto em `Mtr`.
pub(super) fn json_mtr(v: &Value) -> Mtr {
    let data_criacao = v
        .get("manData")
        .and_then(Value::as_i64)
        .and_then(data_de_millis);

    let data_expedicao = v
        .get("manDataExpedicao")
        .and_then(Value::as_i64)
        .and_then(data_de_millis);

    let status_str = v
        .get("manStatus")
        .and_then(|s| s.as_str())
        .or_else(|| {
            v.get("situacaoManifesto")
                .and_then(|s| s.get("simDescricao"))
                .and_then(|s| s.as_str())
        })
        .unwrap_or_default();

    let residuos: Vec<Residuo> = v
        .get("listaManifestoResiduo")
        .and_then(|a| a.as_array())
        .map_or_default(|arr| arr.iter().map(json_residuo).collect());

    Mtr {
        numero: v
            .get("manNumero")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
        barcode: String::new(),
        status: sinir_status(status_str),
        gerador: v.get("parceiroGerador").map(json_parte),
        transportador: v.get("parceiroTransportador").map(json_parte),
        destinador: v.get("parceiroDestinador").map(json_parte),
        residuos,
        data_criacao,
        data_recebimento: None,
        // --- GAP 3: campos adicionais ---
        observacao: v
            .get("manObservacao")
            .and_then(|s| s.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        motorista: v
            .get("manNomeMotorista")
            .and_then(|s| s.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        placa: v
            .get("manPlacaVeiculo")
            .and_then(|s| s.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        tipo_manifesto: v
            .get("tipoManifesto")
            .and_then(Value::as_i64)
            .map(|n| n as i32),
        estado_uf: v
            .get("estado")
            .and_then(|e| e.get("estAbreviacao"))
            .and_then(|s| s.as_str())
            .map(String::from),
        data_expedicao,
        armazenamento_temporario: v
            .get("possuiArmazenamentoTemporario")
            .and_then(Value::as_bool),
        hash: v
            .get("manHashCode")
            .and_then(|s| s.as_str())
            .map(String::from),
        tem_alteracao_pendente: v
            .get("temSolicitacaoAlteracaoRecebimento")
            .and_then(Value::as_bool),
        numero_estadual: v
            .get("manNumeroEstadual")
            .and_then(|s| s.as_str())
            .map(String::from)
            .filter(|s| !s.is_empty()),
        codigo_estadual: v.get("manCodigoEstadual").and_then(Value::as_i64),
    }
}

/// Converte epoch ms em data civil UTC (o contrato do SINIR não anexa fuso).
fn data_de_millis(ms: i64) -> Option<Date> {
    Timestamp::from_millisecond(ms)
        .ok()
        .map(|ts| ts.to_zoned_datetime(TimeZone::UTC).date())
}
