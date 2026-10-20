#![allow(
    clippy::as_conversions,
    clippy::disallowed_types,
    clippy::wildcard_imports,
    reason = "binding napi: macros geram Rc interno e o prelude é reexportado de propósito"
)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use open_mtr_core::*;

use base64::Engine as _;

fn convert_resposta<T: serde::Serialize>(resposta: Resposta<T>) -> RespostaJson {
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

fn convert_download_resposta(resposta: Resposta<DownloadItem>) -> RespostaJson {
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

fn map_err(e: OpenMTRError) -> Error {
    Error::from_reason(e.to_string())
}

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

#[napi(object)]
pub struct RespostaJson {
    pub id: String,
    pub ok: bool,
    pub status: u32,
    pub mensagem: String,
    pub dados: Vec<serde_json::Value>,
    pub erros: Vec<serde_json::Value>,
}

#[napi]
pub struct OpenMtrClient {
    inner: tokio::sync::OnceCell<open_mtr_core::OpenMtrClient>,
    system: String,
    token: String,
    base_url: Option<String>,
}

#[napi]
impl OpenMtrClient {
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

    #[napi]
    pub async fn consultar_mtrs(&self, filtro_json: String) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let filtro: FiltroMtr =
            serde_json::from_str(&filtro_json).map_err(|e| Error::from_reason(e.to_string()))?;
        let resposta = client.consultar_mtrs(&filtro).await.map_err(map_err)?;
        Ok(convert_resposta(resposta))
    }

    #[napi]
    pub async fn criar_mtrs(&self, mtrs_json: String) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let mtrs: Vec<CriarMtr> =
            serde_json::from_str(&mtrs_json).map_err(|e| Error::from_reason(e.to_string()))?;
        let resposta = client.criar_mtrs(mtrs).await.map_err(map_err)?;
        Ok(convert_resposta(resposta))
    }

    #[napi]
    pub async fn receber_mtrs(&self, lotes_json: String) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let lotes: Vec<ReceberMtr> =
            serde_json::from_str(&lotes_json).map_err(|e| Error::from_reason(e.to_string()))?;
        let resposta = client.receber_mtrs(lotes).await.map_err(map_err)?;
        Ok(convert_resposta(resposta))
    }

    #[napi]
    pub async fn cancelar_mtrs(&self, cancelamentos_json: String) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let reqs: Vec<CancelarMtr> = serde_json::from_str(&cancelamentos_json)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        let resposta = client.cancelar_mtrs(reqs).await.map_err(map_err)?;
        Ok(convert_resposta(resposta))
    }

    #[napi]
    pub async fn modificar_mtrs(&self, mods_json: String) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let mods: Vec<ModificarMtr> =
            serde_json::from_str(&mods_json).map_err(|e| Error::from_reason(e.to_string()))?;
        let resposta = client.modificar_mtrs(mods).await.map_err(map_err)?;
        Ok(convert_resposta(resposta))
    }

    #[napi]
    pub async fn download_mtrs(&self, nums: Vec<String>) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let resposta = client.download_mtrs(nums).await.map_err(map_err)?;
        Ok(convert_download_resposta(resposta))
    }

    // ─── CDF ───────────────────────────────────────────────────────────

    #[napi]
    pub async fn gerar_cdfs(&self, reqs_json: String) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let reqs: Vec<GerarCdf> =
            serde_json::from_str(&reqs_json).map_err(|e| Error::from_reason(e.to_string()))?;
        let resposta = client.gerar_cdfs(reqs).await.map_err(map_err)?;
        Ok(convert_resposta(resposta))
    }

    #[napi]
    pub async fn consultar_cdfs(&self, filtro_json: String) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let filtro: FiltroCdf =
            serde_json::from_str(&filtro_json).map_err(|e| Error::from_reason(e.to_string()))?;
        let resposta = client.consultar_cdfs(&filtro).await.map_err(map_err)?;
        Ok(convert_resposta(resposta))
    }

    #[napi]
    pub async fn download_cdfs(&self, reqs_json: String) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let reqs: Vec<DownloadCdf> =
            serde_json::from_str(&reqs_json).map_err(|e| Error::from_reason(e.to_string()))?;
        let resposta = client.download_cdfs(reqs).await.map_err(map_err)?;
        Ok(convert_download_resposta(resposta))
    }

    // ─── Tabelas ───────────────────────────────────────────────────────

    #[napi]
    pub async fn listar_residuos(&self) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let resposta = client.listar_residuos().await.map_err(map_err)?;
        Ok(convert_resposta(resposta))
    }

    #[napi]
    pub async fn listar_acondicionamentos(&self) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let resposta = client.listar_acondicionamentos().await.map_err(map_err)?;
        Ok(convert_resposta(resposta))
    }

    #[napi]
    pub async fn listar_classes(&self) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let resposta = client.listar_classes().await.map_err(map_err)?;
        Ok(convert_resposta(resposta))
    }

    #[napi]
    pub async fn listar_estados_fisicos(&self) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let resposta = client.listar_estados_fisicos().await.map_err(map_err)?;
        Ok(convert_resposta(resposta))
    }

    #[napi]
    pub async fn listar_tratamentos(&self) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let resposta = client.listar_tratamentos().await.map_err(map_err)?;
        Ok(convert_resposta(resposta))
    }

    #[napi]
    pub async fn listar_unidades(&self) -> Result<RespostaJson> {
        let client = self.get_client().await?;
        let resposta = client.listar_unidades().await.map_err(map_err)?;
        Ok(convert_resposta(resposta))
    }

    // ─── Health ────────────────────────────────────────────────────────

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
