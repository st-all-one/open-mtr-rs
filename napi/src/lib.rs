//! Bindings napi-rs do open-mtr para Node.js.
//!
//! [`OpenMtrClient`] expõe ao JavaScript as operações do `open-mtr-core`:
//! entradas são JSON (strings ou vetores) e saídas são [`RespostaJson`], com
//! `dados` serializados em valores JS. Downloads trazem o arquivo em
//! `dados[].buffer`, codificado em Base64 padrão.
#![allow(
    clippy::as_conversions,
    clippy::disallowed_types,
    clippy::wildcard_imports,
    reason = "binding napi: macros geram Rc interno e o prelude é reexportado de propósito"
)]
#![allow(
    missing_docs,
    reason = "código gerado por `#[napi]` não pode receber doc; todo item escrito à mão está documentado"
)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use open_mtr_core::*;

mod json;
use json::{convert_download_resposta, convert_resposta, map_err};

/// Obtém o token de autenticação clássica (CPF + senha) para sistemas que o
/// exigem (ex.: MTR-PE, SIGOR). Não depende de um cliente instanciado.
///
/// `params_json`: `{"tipo":"cpf_senha_unidade","cpf":"...","senha":"...","unidade":123,"cnpj":"..."}`
///
/// Retorna `{ ok, token }` no campo `dados[0]` do envelope.
#[napi]
pub async fn obter_token(system: String, params_json: String) -> Result<String> {
    let system: System = system.parse().map_err(|e: String| Error::from_reason(e))?;
    let params: AuthParams =
        serde_json::from_str(&params_json).map_err(|e| Error::from_reason(e.to_string()))?;
    let token = OpenMtrClassicAuth::new(system)
        .params(params)
        .build()
        .await
        .map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(serde_json::json!({ "ok": true, "token": token }).to_string())
}

/// Envelope devolvido pelas operações do cliente ao JavaScript.
///
/// Espelha `Resposta<T>` do core: `ok` e `status` descrevem o resultado,
/// `dados` contém os objetos serializados pela operação e `erros` traz
/// problemas RFC 9457 (`type`, `title`, `status`, `detail`, `instance`, `kind`).
/// Nas operações de download, cada item de `dados` é
/// `{"filename": string, "buffer": string, "formato": string}`, com `buffer`
/// em Base64 padrão do conteúdo binário.
#[napi(object)]
pub struct RespostaJson {
    /// Identificador da requisição, formatado como UUID string.
    pub id: String,
    /// Indica se a operação foi concluída com sucesso.
    pub ok: bool,
    /// Código HTTP convertido para `u32` a partir do `u16` do core.
    pub status: u32,
    /// Mensagem de sucesso ou de falha devolvida pela operação.
    pub mensagem: String,
    /// Resultados da operação, serializados como valores JSON arbitrários.
    pub dados: Vec<serde_json::Value>,
    /// Problemas RFC 9457 serializados como objetos JSON.
    pub erros: Vec<serde_json::Value>,
}

/// Cliente MTR exportado para JavaScript por napi-rs.
///
/// Guarda `system`, `token` e `base_url` informados no construtor e inicializa
/// o cliente do core sob demanda, na primeira operação. As chamadas de rede
/// retornam uma Promise; [`Self::health`] é síncrona e não acessa a rede.
#[napi]
pub struct OpenMtrClient {
    inner: tokio::sync::OnceCell<open_mtr_core::OpenMtrClient>,
    system: String,
    token: String,
    base_url: Option<String>,
}

/// Operações do cliente expostas ao JavaScript.
#[napi]
impl OpenMtrClient {
    /// Cria o cliente com o sistema, o token e, opcionalmente, a URL base.
    ///
    /// `system` deve ser um identificador aceito por `System` (`sinir`, `mtr_pe`,
    /// `sigor`); `base_url` substitui o endereço padrão do adaptador. O cliente do
    /// core é construído apenas na primeira operação, não no construtor.
    #[napi(constructor)]
    pub fn new(system: String, token: String, base_url: Option<String>) -> Self {
        Self {
            inner: tokio::sync::OnceCell::new(),
            system,
            token,
            base_url,
        }
    }

    async fn get_client(&self) -> Result<&open_mtr_core::OpenMtrClient> {
        let system = self.system.clone();
        let token = self.token.clone();
        let base_url = self.base_url.clone();
        self.inner
            .get_or_try_init(|| async move {
                let parsed: System = system.parse().map_err(|e: String| Error::from_reason(e))?;
                let mut builder = open_mtr_core::OpenMtrClient::builder()
                    .base(parsed)
                    .token(token);
                if let Some(url) = base_url {
                    builder = builder.base_url(url);
                }
                builder
                    .build()
                    .await
                    .map_err(|e| Error::from_reason(e.to_string()))
            })
            .await
    }

    // ─── MTR ───────────────────────────────────────────────────────────

    /// Consulta MTRs segundo um `FiltroMtr` serializado em `filtro_json`.
    ///
    /// Os MTRs encontrados vêm em `dados`; `detalhado: true` no filtro acrescenta
    /// os dados de detalhe de cada manifesto.
    #[napi]
    pub async fn consultar_mtrs(&self, filtro_json: String) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let filtro: FiltroMtr =
            serde_json::from_str(&filtro_json).map_err(|e| Error::from_reason(e.to_string()))?;
        let resposta = client.consultar_mtrs(&filtro).await.map_err(map_err)?;
        Ok(convert_resposta(resposta))
    }

    /// Cria MTRs em lote a partir de uma lista JSON de solicitações `CriarMtr`.
    #[napi]
    pub async fn criar_mtrs(&self, mtrs_json: String) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let mtrs: Vec<CriarMtr> =
            serde_json::from_str(&mtrs_json).map_err(|e| Error::from_reason(e.to_string()))?;
        let resposta = client.criar_mtrs(mtrs).await.map_err(map_err)?;
        Ok(convert_resposta(resposta))
    }

    /// Registra o recebimento de MTRs a partir de uma lista JSON de `ReceberMtr`.
    #[napi]
    pub async fn receber_mtrs(&self, lotes_json: String) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let lotes: Vec<ReceberMtr> =
            serde_json::from_str(&lotes_json).map_err(|e| Error::from_reason(e.to_string()))?;
        let resposta = client.receber_mtrs(lotes).await.map_err(map_err)?;
        Ok(convert_resposta(resposta))
    }

    /// Cancela MTRs em lote a partir de uma lista JSON de `CancelarMtr`.
    #[napi]
    pub async fn cancelar_mtrs(&self, cancelamentos_json: String) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let reqs: Vec<CancelarMtr> = serde_json::from_str(&cancelamentos_json)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        let resposta = client.cancelar_mtrs(reqs).await.map_err(map_err)?;
        Ok(convert_resposta(resposta))
    }

    /// Modifica MTRs em lote a partir de uma lista JSON de `ModificarMtr`.
    #[napi]
    pub async fn modificar_mtrs(&self, mods_json: String) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let mods: Vec<ModificarMtr> =
            serde_json::from_str(&mods_json).map_err(|e| Error::from_reason(e.to_string()))?;
        let resposta = client.modificar_mtrs(mods).await.map_err(map_err)?;
        Ok(convert_resposta(resposta))
    }

    /// Baixa os MTRs indicados pelos números em `nums`.
    ///
    /// Cada item de `dados` contém `filename`, `formato` e `buffer` com o arquivo
    /// codificado em Base64 padrão.
    #[napi]
    pub async fn download_mtrs(&self, nums: Vec<String>) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let resposta = client.download_mtrs(nums).await.map_err(map_err)?;
        Ok(convert_download_resposta(resposta))
    }

    // ─── CDF ───────────────────────────────────────────────────────────

    /// Emite CDFs a partir de uma lista JSON de solicitações `GerarCdf`.

    #[napi]
    pub async fn gerar_cdfs(&self, reqs_json: String) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let reqs: Vec<GerarCdf> =
            serde_json::from_str(&reqs_json).map_err(|e| Error::from_reason(e.to_string()))?;
        let resposta = client.gerar_cdfs(reqs).await.map_err(map_err)?;
        Ok(convert_resposta(resposta))
    }

    /// Consulta CDFs segundo um `FiltroCdf` serializado em `filtro_json`.
    #[napi]
    pub async fn consultar_cdfs(&self, filtro_json: String) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let filtro: FiltroCdf =
            serde_json::from_str(&filtro_json).map_err(|e| Error::from_reason(e.to_string()))?;
        let resposta = client.consultar_cdfs(&filtro).await.map_err(map_err)?;
        Ok(convert_resposta(resposta))
    }

    /// Baixa CDFs a partir de uma lista JSON de solicitações `DownloadCdf`.
    ///
    /// Cada item de `dados` contém `filename`, `formato` e `buffer` com o arquivo
    /// codificado em Base64 padrão.
    #[napi]
    pub async fn download_cdfs(&self, reqs_json: String) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let reqs: Vec<DownloadCdf> =
            serde_json::from_str(&reqs_json).map_err(|e| Error::from_reason(e.to_string()))?;
        let resposta = client.download_cdfs(reqs).await.map_err(map_err)?;
        Ok(convert_download_resposta(resposta))
    }

    // ─── Tabelas ───────────────────────────────────────────────────────

    /// Lista os resíduos do catálogo IBAMA em `dados`.

    #[napi]
    pub async fn listar_residuos(&self) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let resposta = client.listar_residuos().await.map_err(map_err)?;
        Ok(convert_resposta(resposta))
    }

    /// Lista as formas de acondicionamento disponíveis em `dados`.
    #[napi]
    pub async fn listar_acondicionamentos(&self) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let resposta = client.listar_acondicionamentos().await.map_err(map_err)?;
        Ok(convert_resposta(resposta))
    }

    /// Lista as classes de resíduos disponíveis em `dados`.
    #[napi]
    pub async fn listar_classes(&self) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let resposta = client.listar_classes().await.map_err(map_err)?;
        Ok(convert_resposta(resposta))
    }

    /// Lista os estados físicos dos resíduos em `dados`.
    #[napi]
    pub async fn listar_estados_fisicos(&self) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let resposta = client.listar_estados_fisicos().await.map_err(map_err)?;
        Ok(convert_resposta(resposta))
    }

    /// Lista os tratamentos de resíduos disponíveis em `dados`.
    #[napi]
    pub async fn listar_tratamentos(&self) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let resposta = client.listar_tratamentos().await.map_err(map_err)?;
        Ok(convert_resposta(resposta))
    }

    /// Lista as unidades de medida disponíveis em `dados`.
    #[napi]
    pub async fn listar_unidades(&self) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let resposta = client.listar_unidades().await.map_err(map_err)?;
        Ok(convert_resposta(resposta))
    }

    // ─── Health ────────────────────────────────────────────────────────

    /// Devolve o estado local do cliente, sem fazer requisições de rede.
    ///
    /// O envelope tem `ok: true`, `status: 200`, `mensagem` com o sistema
    /// configurado e `id`, `dados` e `erros` vazios.
    #[napi]
    pub fn health(&self) -> RespostaJson {
        RespostaJson {
            id: String::new(),
            ok: true,
            status: 200,
            mensagem: format!("Sistema: {}", self.system),
            dados: Vec::new(),
            erros: Vec::new(),
        }
    }
}
