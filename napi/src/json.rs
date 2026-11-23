//! Conversão das respostas do core para o envelope [`super::RespostaJson`] exposto ao JS.

use super::RespostaJson;
use base64::Engine as _;
use open_mtr_core::{DownloadItem, Resposta};

pub(super) fn convert_resposta<T: serde::Serialize>(resposta: Resposta<T>) -> RespostaJson {
    RespostaJson {
        id: resposta.id.to_string(),
        ok: resposta.ok,
        status: u32::from(resposta.status),
        mensagem: resposta.mensagem,
        dados: resposta
            .dados
            .into_iter()
            .map(|d| serde_json::to_value(d).unwrap_or(serde_json::Value::Null))
            .collect(),
        erros: resposta
            .erros
            .into_iter()
            .map(|e| serde_json::to_value(e).unwrap_or(serde_json::Value::Null))
            .collect(),
    }
}

pub(super) fn convert_download_resposta(resposta: Resposta<DownloadItem>) -> RespostaJson {
    let engine = base64::engine::general_purpose::STANDARD;
    RespostaJson {
        id: resposta.id.to_string(),
        ok: resposta.ok,
        status: u32::from(resposta.status),
        mensagem: resposta.mensagem,
        dados: resposta
            .dados
            .into_iter()
            .map(|d| {
                serde_json::json!({
                    "filename": d.filename,
                    "buffer": engine.encode(&d.buffer),
                    "formato": d.formato,
                })
            })
            .collect(),
        erros: resposta
            .erros
            .into_iter()
            .map(|e| serde_json::to_value(e).unwrap_or(serde_json::Value::Null))
            .collect(),
    }
}

/// Converte um erro do core em `napi::Error`, preservando a mensagem RFC 9457.
pub(super) fn map_err(e: open_mtr_core::openmtr::error::OpenMTRError) -> napi::Error {
    napi::Error::from_reason(e.to_string())
}
