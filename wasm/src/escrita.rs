//! Operações de escrita em lote (criar/receber/cancelar/modificar/aceitar).

use wasm_bindgen::prelude::*;

use open_mtr_core::openmtr::domain::{
    AceitarAlteracao, CancelarMtr, CriarMtr, ModificarMtr, ReceberMtr,
};

use super::OpenMtrWasm;
use super::json::lote_result_to_json;

#[wasm_bindgen]
impl OpenMtrWasm {
    /// Cria MTRs em lote a partir de uma lista JSON de solicitações `CriarMtr`.
    pub async fn criar_mtrs(&self, json: &str) -> Result<String, JsError> {
        let mtrs: Vec<CriarMtr> =
            serde_json::from_str(json).map_err(|e| JsError::new(&e.to_string()))?;
        let client = self.client()?;
        let result = client.criar_mtrs(mtrs).await;
        Ok(lote_result_to_json(&result))
    }

    /// Registra o recebimento de MTRs a partir de uma lista JSON de `ReceberMtr`.
    pub async fn receber_mtrs(&self, json: &str) -> Result<String, JsError> {
        let lotes: Vec<ReceberMtr> =
            serde_json::from_str(json).map_err(|e| JsError::new(&e.to_string()))?;
        let client = self.client()?;
        let result = client.receber_mtrs(lotes).await;
        Ok(lote_result_to_json(&result))
    }

    /// Cancela MTRs em lote a partir de uma lista JSON de `CancelarMtr`.
    pub async fn cancelar_mtrs(&self, json: &str) -> Result<String, JsError> {
        let reqs: Vec<CancelarMtr> =
            serde_json::from_str(json).map_err(|e| JsError::new(&e.to_string()))?;
        let client = self.client()?;
        let result = client.cancelar_mtrs(reqs).await;
        Ok(lote_result_to_json(&result))
    }

    /// Modifica MTRs em lote a partir de uma lista JSON de `ModificarMtr`.
    pub async fn modificar_mtrs(&self, json: &str) -> Result<String, JsError> {
        let reqs: Vec<ModificarMtr> =
            serde_json::from_str(json).map_err(|e| JsError::new(&e.to_string()))?;
        let client = self.client()?;
        let result = client.modificar_mtrs(reqs).await;
        Ok(lote_result_to_json(&result))
    }

    /// Aceita alterações de MTRs em lote a partir de uma lista JSON de `AceitarAlteracao`.
    pub async fn aceitar_alteracoes(&self, json: &str) -> Result<String, JsError> {
        let reqs: Vec<AceitarAlteracao> =
            serde_json::from_str(json).map_err(|e| JsError::new(&e.to_string()))?;
        let client = self.client()?;
        let result = client.aceitar_alteracoes(reqs).await;
        Ok(lote_result_to_json(&result))
    }
}
