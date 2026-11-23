//! Serialização das respostas do core para o envelope JSON devolvido ao JS.

use serde::Serialize;

pub(super) fn lote_result_to_json<T: Serialize>(result: &open_mtr_core::MtrResult<T>) -> String {
    match result {
        Ok(resp) => serde_json::to_string(resp)
            .unwrap_or_else(|e| format!(r#"{{"ok":false,"mensagem":"serialize error: {e}"}}"#)),
        Err(err) => serde_json::to_string(&serde_json::json!({
            "ok": false,
            "status": err.status,
            "mensagem": format!("{} — {}", err.title, err.detail),
        }))
        .unwrap_or_default(),
    }
}
