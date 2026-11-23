//! Operações de leitura (health e consulta de MTRs/CDFs).

use wasm_bindgen::prelude::*;

use open_mtr_core::openmtr::domain::FiltroMtr;

use super::OpenMtrWasm;

#[wasm_bindgen]
impl OpenMtrWasm {
    /// Verifica a conexão consultando MTRs com o filtro padrão.
    ///
    /// Uma falha de rede não gera `JsError`: vem como `{"ok":false,"mensagem":"..."}`.
    pub async fn health(&self) -> Result<String, JsError> {
        let client = self.client()?;
        match client.consultar_mtrs(&FiltroMtr::default()).await {
            Ok(_) => Ok(r#"{"ok":true,"mensagem":"conexão OK"}"#.into()),
            Err(e) => Ok(serde_json::to_string(&serde_json::json!({
                "ok": false,
                "mensagem": e.to_string(),
            }))
            .unwrap_or_default()),
        }
    }
}
