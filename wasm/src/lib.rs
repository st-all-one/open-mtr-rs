//! Bindings wasm-bindgen do cliente open-mtr para Deno e para o navegador.
//!
//! [`OpenMtrWasm`] guarda o cliente em um `RefCell`: existe apenas um
//! empréstimo mutável por vez, de modo que `init()` substitui a conexão sem
//! concorrer com as operações. O runtime wasm é single-thread, portanto
//! `Send` e `Sync` não integram o contrato desta API.
//!
//! As entradas são JSON em `&str` desserializado para os tipos de domínio do
//! core; as saídas são o envelope `Resposta`, com `ok`, `status`, `mensagem`,
//! `dados` e `erros`. Falhas de parsing ou de inicialização viram `JsError`;
//! falhas da API MTR vêm no envelope.

#![allow(
    unsafe_code,
    clippy::await_holding_refcell_ref,
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::future_not_send,
    clippy::wildcard_imports,
    reason = "binding wasm single-thread: RefCell/Rc são o modelo imposto pelo wasm-bindgen; unsafe confinado a transport.rs"
)]

use std::cell::RefCell;

use wasm_bindgen::prelude::*;

use open_mtr_core::openmtr::adapters::MtrCtx;
use open_mtr_core::openmtr::client::http::MtrHttpClient;
use open_mtr_core::openmtr::config::Config;
use open_mtr_core::openmtr::domain::{
    CriarMtrComplementar, DownloadCdf, FiltroCdf, FiltroMtr, GerarCdf,
};
use open_mtr_core::openmtr::system::System;

mod consulta;
mod escrita;
mod json;
mod transport;
use json::lote_result_to_json;
use transport::JsFetchTransport;

/// Cliente MTR exportado para JavaScript por wasm-bindgen.
///
/// O cliente fica em um `RefCell`, com um único empréstimo mutável por vez:
/// [`Self::init`] instala a conexão e não pode concorrer com as operações.
/// Como o runtime wasm é single-thread, `Send` e `Sync` não se aplicam.
/// Exceto [`Self::new`], [`Self::init`] e [`Self::obter_token`], os métodos
/// exigem uma instância inicializada.
#[wasm_bindgen]
pub struct OpenMtrWasm {
    client: RefCell<Option<open_mtr_core::OpenMtrClient>>,
}

impl Default for OpenMtrWasm {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl OpenMtrWasm {
    /// Construtor JavaScript: cria uma instância sem cliente, aguardando [`Self::init`].
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            client: RefCell::new(None),
        }
    }

    /// Autentica no sistema escolhido e instala o cliente usado pelas operações.
    ///
    /// `system` aceita `sinir`, `sigor` ou um estado no formato `mtrpe`, `mtr_pe` /
    /// `mtr-pe` (e variantes análogas para CE, SP, RJ, MG e BA); `base_url` vazia
    /// usa a URL padrão do sistema. O transporte é `fetch` do JavaScript.
    ///
    /// Substitui o cliente no único empréstimo mutável do `RefCell`: chame uma vez
    /// antes das operações e não o concorra com elas, já que o runtime wasm é
    /// single-thread e `Send`/`Sync` não se aplicam.
    /// Retorna `{"ok":true,"mensagem":"inicializado"}`.
    pub async fn init(
        &self,
        system: String,
        token: String,
        base_url: String,
    ) -> Result<String, JsError> {
        let system: System = system
            .parse()
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
        let http = MtrHttpClient::new(config.clone(), transport);

        let adapter = system.adapter().map_err(|e| JsError::new(&e.to_string()))?;

        let ctx = MtrCtx::new(http, config);

        adapter
            .autenticar_token(&ctx)
            .await
            .map_err(|e| JsError::new(&e.to_string()))?;

        let inner = open_mtr_core::OpenMtrClient::from_parts(std::sync::Arc::new(ctx), adapter);

        *self.client.borrow_mut() = Some(inner);
        Ok(r#"{"ok":true,"mensagem":"inicializado"}"#.into())
    }

    /// Obtém o token de autenticação clássica (CPF + senha) para sistemas que
    /// o exigem (ex.: MTR-PE, SIGOR), **sem** depender de um cliente
    /// instanciado. Ideal para servidores Deno/SSR que precisam gerar o token
    /// e então chamar `init(system, token, "")`.
    ///
    /// `params_json`: `{"tipo":"cpf_senha_unidade","cpf":"...","senha":"...","unidade":123,"cnpj":"..."}`
    ///
    /// Retorna `{"ok":true,"token":"..."}` ou `{"ok":false,"mensagem":"..."}`.
    pub async fn obter_token(&self, system: String, params_json: &str) -> Result<String, JsError> {
        let system: System = system
            .parse()
            .map_err(|_| JsError::new(&format!("sistema inválido: {system}")))?;

        let params: open_mtr_core::openmtr::open_mtr_auth::AuthParams =
            serde_json::from_str(params_json)
                .map_err(|e| JsError::new(&format!("params_json inválido: {e}")))?;

        let auth = open_mtr_core::openmtr::open_mtr_auth::OpenMtrClassicAuth::new(system)
            .params(params)
            .timeout_secs(120);

        let token = auth
            .build_with_transport(JsFetchTransport::new())
            .await
            .map_err(|e| JsError::new(&format!("{} — {}", e.title, e.detail)))?;

        Ok(serde_json::to_string(&serde_json::json!({
            "ok": true,
            "token": token,
        }))
        .unwrap_or_default())
    }

    pub(crate) fn client(
        &self,
    ) -> Result<std::cell::Ref<'_, open_mtr_core::OpenMtrClient>, JsError> {
        let borrowed = self.client.borrow();
        if borrowed.is_none() {
            return Err(JsError::new(
                "cliente não inicializado — chame init() primeiro",
            ));
        }
        let Some(client) = std::cell::Ref::filter_map(borrowed, Option::as_ref).ok() else {
            return Err(JsError::new("cliente não inicializado"));
        };
        Ok(client)
    }

    /// Consulta MTRs com um `FiltroMtr` serializado em `filtro_json`.
    ///
    /// O envelope devolvido traz os MTRs encontrados em `dados`.
    pub async fn consultar_mtrs(&self, filtro_json: &str) -> Result<String, JsError> {
        let filtro: FiltroMtr =
            serde_json::from_str(filtro_json).map_err(|e| JsError::new(&e.to_string()))?;
        let client = self.client()?;
        let result = client.consultar_mtrs(&filtro).await;
        Ok(lote_result_to_json(&result))
    }

    /// Baixa os MTRs cujos números são passados como uma lista JSON de strings.
    ///
    /// O envelope devolvido traz os arquivos em `dados`, como `DownloadItem`.
    pub async fn download_mtrs(&self, json: &str) -> Result<String, JsError> {
        let nums: Vec<String> =
            serde_json::from_str(json).map_err(|e| JsError::new(&e.to_string()))?;
        let client = self.client()?;
        let result = client.download_mtrs(nums).await;
        Ok(lote_result_to_json(&result))
    }

    /// Emite CDFs a partir de uma lista JSON de solicitações `GerarCdf`.
    pub async fn gerar_cdfs(&self, json: &str) -> Result<String, JsError> {
        let reqs: Vec<GerarCdf> =
            serde_json::from_str(json).map_err(|e| JsError::new(&e.to_string()))?;
        let client = self.client()?;
        let result = client.gerar_cdfs(reqs).await;
        Ok(lote_result_to_json(&result))
    }

    /// Consulta CDFs com um `FiltroCdf` serializado em `filtro_json`.
    pub async fn consultar_cdfs(&self, json: &str) -> Result<String, JsError> {
        let filtro: FiltroCdf =
            serde_json::from_str(json).map_err(|e| JsError::new(&e.to_string()))?;
        let client = self.client()?;
        let result = client.consultar_cdfs(&filtro).await;
        Ok(lote_result_to_json(&result))
    }

    /// Baixa CDFs a partir de uma lista JSON de solicitações `DownloadCdf`.
    pub async fn download_cdfs(&self, json: &str) -> Result<String, JsError> {
        let reqs: Vec<DownloadCdf> =
            serde_json::from_str(json).map_err(|e| JsError::new(&e.to_string()))?;
        let client = self.client()?;
        let result = client.download_cdfs(reqs).await;
        Ok(lote_result_to_json(&result))
    }

    /// Lista os resíduos do catálogo IBAMA (`ResiduoIbama`).
    pub async fn listar_residuos(&self) -> Result<String, JsError> {
        let client = self.client()?;
        let result = client.listar_residuos().await;
        Ok(lote_result_to_json(&result))
    }

    /// Lista as formas de acondicionamento disponíveis (`Acondicionamento`).
    pub async fn listar_acondicionamentos(&self) -> Result<String, JsError> {
        let client = self.client()?;
        let result = client.listar_acondicionamentos().await;
        Ok(lote_result_to_json(&result))
    }

    /// Lista as classes de resíduos disponíveis (`ClasseResiduo`).
    pub async fn listar_classes(&self) -> Result<String, JsError> {
        let client = self.client()?;
        let result = client.listar_classes().await;
        Ok(lote_result_to_json(&result))
    }

    /// Lista os estados físicos dos resíduos (`EstadoFisico`).
    pub async fn listar_estados_fisicos(&self) -> Result<String, JsError> {
        let client = self.client()?;
        let result = client.listar_estados_fisicos().await;
        Ok(lote_result_to_json(&result))
    }

    /// Lista os tratamentos de resíduos disponíveis (`TratamentoResiduo`).
    pub async fn listar_tratamentos(&self) -> Result<String, JsError> {
        let client = self.client()?;
        let result = client.listar_tratamentos().await;
        Ok(lote_result_to_json(&result))
    }

    /// Lista as unidades de medida disponíveis (`UnidadeMedida`).
    pub async fn listar_unidades(&self) -> Result<String, JsError> {
        let client = self.client()?;
        let result = client.listar_unidades().await;
        Ok(lote_result_to_json(&result))
    }

    /// Lista as classes de resíduo associadas ao código IBAMA informado.
    pub async fn listar_classes_por_residuo(&self, codigo_ibama: &str) -> Result<String, JsError> {
        let client = self.client()?;
        let result = client.listar_classes_por_residuo(codigo_ibama).await;
        Ok(lote_result_to_json(&result))
    }

    /// Lista os acondicionamentos válidos para o código de estado físico informado.
    pub async fn listar_acondicionamentos_por_estado_fisico(
        &self,
        codigo_estado: u32,
    ) -> Result<String, JsError> {
        let client = self.client()?;
        let result = client
            .listar_acondicionamentos_por_estado_fisico(codigo_estado)
            .await;
        Ok(lote_result_to_json(&result))
    }

    /// Cria um manifesto complementar a partir de um `CriarMtrComplementar` em JSON.
    ///
    /// Recebe um único objeto, e não uma lista como os métodos de lote.
    pub async fn criar_manifesto_complementar(&self, json: &str) -> Result<String, JsError> {
        let req: CriarMtrComplementar =
            serde_json::from_str(json).map_err(|e| JsError::new(&e.to_string()))?;
        let client = self.client()?;
        let result = client.criar_manifesto_complementar(req).await;
        Ok(lote_result_to_json(&result))
    }
}
