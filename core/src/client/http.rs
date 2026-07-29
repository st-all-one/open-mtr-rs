use crate::client::token::TokenManager;
use crate::core::errors::{ErrorCategory, ErrorContext, MtrError};
use reqwest::{Client, Response, StatusCode};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{instrument, warn};

#[derive(Clone)]
pub struct MtrClient {
    http: Client,
    config: crate::core::config::MtrConfig,
    token_mgr: Arc<RwLock<TokenManager>>,
}

impl MtrClient {
    pub fn new(config: crate::core::config::MtrConfig) -> Result<Self, MtrError> {
        let http = Client::builder()
            .user_agent(concat!("mtr-sinir-rs/", env!("CARGO_PKG_VERSION")))
            .timeout(config.timeout)
            .build()
            .map_err(|e| MtrError::config(format!("falha ao criar HTTP client: {e}")))?;
        Ok(Self {
            http,
            token_mgr: Arc::new(RwLock::new(TokenManager::new(&config.integration_token))),
            config,
        })
    }

    pub fn config(&self) -> &crate::core::config::MtrConfig {
        &self.config
    }

    #[instrument(skip(self), fields(operation = %op))]
    pub async fn request<T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        op: &str,
    ) -> Result<T, MtrError> {
        self.request_with::<T, serde_json::Value>(method, path, None, None, op)
            .await
    }

    #[instrument(skip(self, body), fields(operation = %op))]
    pub async fn request_with_body<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: B,
        op: &str,
    ) -> Result<T, MtrError> {
        self.request_with::<T, B>(method, path, Some(body), None, op)
            .await
    }

    #[instrument(skip(self), fields(operation = %op))]
    pub async fn request_with_params<T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        params: Vec<(&str, String)>,
        op: &str,
    ) -> Result<T, MtrError> {
        self.request_with::<T, serde_json::Value>(method, path, None, Some(params), op)
            .await
    }

    async fn request_with<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<B>,
        query: Option<Vec<(&str, String)>>,
        op: &str,
    ) -> Result<T, MtrError> {
        let url = format!("{}{}", self.config.base_url, path);
        let token = self.get_token().await?;

        let mut req = self
            .http
            .request(method.clone(), &url)
            .header("Authorization", format!("Bearer {token}"));

        if let Some(ref b) = body {
            req = req.json(b);
        }
        if let Some(ref params) = query {
            req = req.query(params);
        }

        let resp = req.send().await.map_err(|e| {
            if e.is_timeout() {
                MtrError::timeout(self.config.timeout, op)
            } else {
                MtrError::Http(e)
            }
        })?;

        if resp.status() == StatusCode::UNAUTHORIZED {
            warn!("Token expirado — renovando...");
            let new_token = {
                let mut mgr = self.token_mgr.write().await;
                mgr.refresh(&self.http, &self.config.base_url).await?;
                mgr.token().expect("token após refresh").to_string()
            };

            let mut retry = self.http.request(method, &url);
            if let Some(ref b) = body {
                retry = retry.json(b);
            }
            retry = retry.header("Authorization", format!("Bearer {new_token}"));
            let resp = retry.send().await.map_err(MtrError::Http)?;
            return Self::handle_response(resp, op).await;
        }

        Self::handle_response(resp, op).await
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(
        resp: Response,
        op: &str,
    ) -> Result<T, MtrError> {
        let status = resp.status();
        let content_type = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        if content_type.contains("pdf") || content_type.contains("xml") {
            return Err(MtrError::api(
                ErrorCategory::ParseError,
                200,
                "Use download_raw() para PDF/XML",
                ErrorContext::new(op),
            ));
        }

        let json: serde_json::Value = resp.json().await.map_err(MtrError::Http)?;

        if !status.is_success() {
            let msg = json
                .get("mensagem")
                .and_then(|v| v.as_str())
                .unwrap_or("Erro desconhecido");
            return Err(MtrError::api(
                ErrorCategory::from_status(status.as_u16())
                    .unwrap_or(ErrorCategory::InternalError),
                status.as_u16(),
                format!("API error [{}]: {}", status.as_u16(), msg),
                ErrorContext::new(op),
            ));
        }

        serde_json::from_value(json).map_err(|e| MtrError::deserialize(e.to_string()))
    }

    #[instrument(skip(self), fields(operation = %"download"))]
    pub async fn download_raw(
        &self,
        method: reqwest::Method,
        path: &str,
    ) -> Result<Vec<u8>, MtrError> {
        let url = format!("{}{}", self.config.base_url, path);
        let token = self.get_token().await?;

        let resp = self
            .http
            .request(method, &url)
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .map_err(MtrError::Http)?;

        if !resp.status().is_success() {
            return Err(MtrError::api(
                ErrorCategory::from_status(resp.status().as_u16())
                    .unwrap_or(ErrorCategory::InternalError),
                resp.status().as_u16(),
                "Download falhou",
                ErrorContext::new("download"),
            ));
        }

        resp.bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(MtrError::Http)
    }

    pub async fn token_valido(&self) -> bool {
        self.token_mgr.read().await.token().is_some()
    }

    async fn get_token(&self) -> Result<String, MtrError> {
        let mgr = self.token_mgr.read().await;
        if let Some(t) = mgr.token() {
            return Ok(t.to_string());
        }
        drop(mgr);
        let mut mgr = self.token_mgr.write().await;
        mgr.refresh(&self.http, &self.config.base_url).await?;
        Ok(mgr.token().expect("token após refresh").to_string())
    }
}
