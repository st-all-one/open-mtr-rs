use std::cell::RefCell;

use serde::Serialize;
use wasm_bindgen::prelude::*;

use open_mtr_core::openmtr::system::System;
use open_mtr_core::openmtr::domain::{
    AceitarAlteracao, CancelarMtr, CriarMtr, CriarMtrComplementar,
    DownloadCdf, FiltroCdf, FiltroMtr, GerarCdf, ModificarMtr,
    ReceberMtr,
};
use open_mtr_core::openmtr::config::Config;
use open_mtr_core::openmtr::client::http::MtrHttpClient;
use open_mtr_core::openmtr::adapters::MtrCtx;

mod transport;
use transport::JsFetchTransport;

fn lote_result_to_json<T: Serialize>(
    result: &open_mtr_core::MtrResult<T>,
) -> String {
    match result {
        Ok(resp) => serde_json::to_string(resp)
            .unwrap_or_else(|e| format!(r#"{{"ok":false,"mensagem":"serialize error: {e}"}}"#)),
        Err(err) => serde_json::to_string(&serde_json::json!({
            "ok": false,
            "status": err.status,
            "mensagem": format!("{} — {}", err.title, err.detail),
        })).unwrap_or_default(),
    }
}

#[wasm_bindgen]
pub struct OpenMtrWasm {
    client: RefCell<Option<open_mtr_core::OpenMtrClient>>,
}

#[wasm_bindgen]
impl OpenMtrWasm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            client: RefCell::new(None),
        }
    }

    pub async fn init(
        &self,
        system: String,
        token: String,
        base_url: String,
    ) -> Result<String, JsError> {
        let system: System = system.parse()
            .map_err(|_| JsError::new(&format!("sistema inválido: {system}")))?;

        let base_url = if base_url.is_empty() {
            system.default_base_url().to_string()
        } else {
            base_url
        };

        let config = Config::builder()
            .system(system)
            .base_url(base_url)
            .token(&token)
            .timeout_secs(120)
            .max_rps(10)
            .build()
            .map_err(|e| JsError::new(&e.to_string()))?;

        let transport = JsFetchTransport::new();
        let http = MtrHttpClient::new(config.clone(), transport)
            .map_err(|e| JsError::new(&e.to_string()))?;

        let adapter = system.adapter()
            .map_err(|e| JsError::new(&e.to_string()))?;

        let ctx = MtrCtx { http, config };

        adapter.autenticar_token(&ctx).await
            .map_err(|e| JsError::new(&e.to_string()))?;

        let inner = open_mtr_core::OpenMtrClient::from_parts(
            std::sync::Arc::new(ctx),
            adapter,
        );

        *self.client.borrow_mut() = Some(inner);
        Ok(r#"{"ok":true,"mensagem":"inicializado"}"#.into())
    }

    fn client(&self) -> Result<std::cell::Ref<'_, open_mtr_core::OpenMtrClient>, JsError> {
        let borrowed = self.client.borrow();
        if borrowed.is_none() {
            return Err(JsError::new("cliente não inicializado — chame init() primeiro"));
        }
        Ok(std::cell::Ref::map(borrowed, |b| b.as_ref().unwrap()))
    }

    pub async fn health(&self) -> Result<String, JsError> {
        let client = self.client()?;
        match client.consultar_mtrs(&FiltroMtr::default()).await {
            Ok(_) => Ok(r#"{"ok":true,"mensagem":"conexão OK"}"#.into()),
            Err(e) => Ok(serde_json::to_string(&serde_json::json!({
                "ok": false,
                "mensagem": e.to_string(),
            })).unwrap_or_default()),
        }
    }

    pub async fn consultar_mtrs(&self, filtro_json: &str) -> Result<String, JsError> {
        let filtro: FiltroMtr = serde_json::from_str(filtro_json)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let client = self.client()?;
        let result = client.consultar_mtrs(&filtro).await;
        Ok(lote_result_to_json(&result))
    }

    pub async fn criar_mtrs(&self, json: &str) -> Result<String, JsError> {
        let mtrs: Vec<CriarMtr> = serde_json::from_str(json)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let client = self.client()?;
        let result = client.criar_mtrs(mtrs).await;
        Ok(lote_result_to_json(&result))
    }

    pub async fn receber_mtrs(&self, json: &str) -> Result<String, JsError> {
        let lotes: Vec<ReceberMtr> = serde_json::from_str(json)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let client = self.client()?;
        let result = client.receber_mtrs(lotes).await;
        Ok(lote_result_to_json(&result))
    }

    pub async fn cancelar_mtrs(&self, json: &str) -> Result<String, JsError> {
        let reqs: Vec<CancelarMtr> = serde_json::from_str(json)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let client = self.client()?;
        let result = client.cancelar_mtrs(reqs).await;
        Ok(lote_result_to_json(&result))
    }

    pub async fn modificar_mtrs(&self, json: &str) -> Result<String, JsError> {
        let reqs: Vec<ModificarMtr> = serde_json::from_str(json)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let client = self.client()?;
        let result = client.modificar_mtrs(reqs).await;
        Ok(lote_result_to_json(&result))
    }

    pub async fn aceitar_alteracoes(&self, json: &str) -> Result<String, JsError> {
        let reqs: Vec<AceitarAlteracao> = serde_json::from_str(json)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let client = self.client()?;
        let result = client.aceitar_alteracoes(reqs).await;
        Ok(lote_result_to_json(&result))
    }

    pub async fn download_mtrs(&self, json: &str) -> Result<String, JsError> {
        let nums: Vec<String> = serde_json::from_str(json)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let client = self.client()?;
        let result = client.download_mtrs(nums).await;
        Ok(lote_result_to_json(&result))
    }

    pub async fn gerar_cdfs(&self, json: &str) -> Result<String, JsError> {
        let reqs: Vec<GerarCdf> = serde_json::from_str(json)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let client = self.client()?;
        let result = client.gerar_cdfs(reqs).await;
        Ok(lote_result_to_json(&result))
    }

    pub async fn consultar_cdfs(&self, json: &str) -> Result<String, JsError> {
        let filtro: FiltroCdf = serde_json::from_str(json)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let client = self.client()?;
        let result = client.consultar_cdfs(&filtro).await;
        Ok(lote_result_to_json(&result))
    }

    pub async fn download_cdfs(&self, json: &str) -> Result<String, JsError> {
        let reqs: Vec<DownloadCdf> = serde_json::from_str(json)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let client = self.client()?;
        let result = client.download_cdfs(reqs).await;
        Ok(lote_result_to_json(&result))
    }

    pub async fn listar_residuos(&self) -> Result<String, JsError> {
        let client = self.client()?;
        let result = client.listar_residuos().await;
        Ok(lote_result_to_json(&result))
    }

    pub async fn listar_acondicionamentos(&self) -> Result<String, JsError> {
        let client = self.client()?;
        let result = client.listar_acondicionamentos().await;
        Ok(lote_result_to_json(&result))
    }

    pub async fn listar_classes(&self) -> Result<String, JsError> {
        let client = self.client()?;
        let result = client.listar_classes().await;
        Ok(lote_result_to_json(&result))
    }

    pub async fn listar_estados_fisicos(&self) -> Result<String, JsError> {
        let client = self.client()?;
        let result = client.listar_estados_fisicos().await;
        Ok(lote_result_to_json(&result))
    }

    pub async fn listar_tratamentos(&self) -> Result<String, JsError> {
        let client = self.client()?;
        let result = client.listar_tratamentos().await;
        Ok(lote_result_to_json(&result))
    }

    pub async fn listar_unidades(&self) -> Result<String, JsError> {
        let client = self.client()?;
        let result = client.listar_unidades().await;
        Ok(lote_result_to_json(&result))
    }

    pub async fn listar_classes_por_residuo(&self, codigo_ibama: &str) -> Result<String, JsError> {
        let client = self.client()?;
        let result = client.listar_classes_por_residuo(codigo_ibama).await;
        Ok(lote_result_to_json(&result))
    }

    pub async fn listar_acondicionamentos_por_estado_fisico(&self, codigo_estado: u32) -> Result<String, JsError> {
        let client = self.client()?;
        let result = client.listar_acondicionamentos_por_estado_fisico(codigo_estado).await;
        Ok(lote_result_to_json(&result))
    }

    pub async fn criar_manifesto_complementar(&self, json: &str) -> Result<String, JsError> {
        let req: CriarMtrComplementar = serde_json::from_str(json)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let client = self.client()?;
        let result = client.criar_manifesto_complementar(req).await;
        Ok(lote_result_to_json(&result))
    }
}
