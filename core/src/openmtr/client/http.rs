#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc, clippy::too_many_lines)]

use std::sync::{Arc, RwLock};
use std::time::Duration;
use serde::de::DeserializeOwned;
use tracing::{info, warn, instrument};

#[cfg(target_arch = "wasm32")]
use web_time::Instant;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

use crate::openmtr::config::Config;
use crate::openmtr::client::token::TokenManager;
use crate::openmtr::transport::{HttpRequest, HttpTransport};
use crate::openmtr::error::OpenMTRError;

#[cfg(target_arch = "wasm32")]
pub(crate) fn now_unix_secs() -> u64 {
    (js_sys::Date::now() / 1000.0) as u64
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Debug, Clone)]
pub struct RawResponse {
    pub bytes: Vec<u8>,
    pub content_type: Option<String>,
    pub content_length: Option<u64>,
}

impl RawResponse {
    pub fn validate(&self, expected: &str) -> Result<(), String> {
        if let Some(ref ct) = self.content_type {
            let ct_lower = ct.to_lowercase();
            if ct_lower.contains("text/html") || ct_lower.contains("text/plain") {
                return Err(format!("Content-Type inesperado: {ct} (esperado {expected})"));
            }
            if expected == "xml" && !ct_lower.contains("xml") && !ct_lower.contains("octet-stream") {
                return Err(format!("Content-Type inesperado: {ct} (esperado application/xml)"));
            }
        }

        let magic_ok = match expected {
            "pdf" => self.bytes.starts_with(b"%PDF"),
            "xml" => {
                self.bytes.starts_with(b"<?xml")
                    || self.bytes.starts_with(b"<")
            }
            _ => true,
        };
        if !magic_ok {
            return Err(format!(
                "Arquivo inválido: magic bytes não correspondem ao formato {expected}"
            ));
        }

        if let Some(cl) = self.content_length
            && cl != self.bytes.len() as u64 {
                return Err(format!(
                    "Content-Length ({cl}) difere dos bytes recebidos ({})",
                    self.bytes.len()
                ));
            }

        Ok(())
    }
}

pub struct MtrHttpClient {
    transport: Arc<dyn HttpTransport>,
    config: Config,
    token_mgr: Arc<RwLock<TokenManager>>,
}

impl MtrHttpClient {
    pub fn new(config: Config, transport: impl HttpTransport + 'static) -> Result<Self, OpenMTRError> {
        let integration_token = config.integration_token.clone().unwrap_or_default();

        let token_mgr = {
            let now_secs = now_unix_secs();
            let ttl = TokenManager::extract_expiry(&integration_token)
                .map_or(3600, |exp| exp.saturating_sub(now_secs));
            let safe = ttl.saturating_sub(60);
            let expires = Instant::now()
                + std::time::Duration::from_secs(safe);
            Arc::new(RwLock::new(TokenManager::from_cached(
                integration_token,
                expires,
            )))
        };

        Ok(Self {
            transport: Arc::new(transport),
            config,
            token_mgr,
        })
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn token_mgr(&self) -> &Arc<RwLock<TokenManager>> {
        &self.token_mgr
    }

    pub async fn get<T_: DeserializeOwned>(
        &self,
        path: &str,
        op: &str,
    ) -> Result<T_, OpenMTRError> {
        self.request::<T_, ()>("GET", path, None::<&()>, op).await
    }

    pub async fn post<T_: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
        op: &str,
    ) -> Result<T_, OpenMTRError> {
        self.request("POST", path, Some(body), op).await
    }

    pub async fn post_bytes(
        &self,
        path: &str,
        op: &str,
    ) -> Result<RawResponse, OpenMTRError> {
        self.request_raw("POST", path, op).await
    }

    #[allow(dead_code)]
    pub async fn get_bytes(
        &self,
        path: &str,
        op: &str,
    ) -> Result<RawResponse, OpenMTRError> {
        self.request_raw("GET", path, op).await
    }

    #[instrument(skip(self, body), fields(operation = %op, method = %method, path = %path))]
    pub async fn request<T_: DeserializeOwned, B: serde::Serialize>(
        &self,
        method: &str,
        path: &str,
        body: Option<&B>,
        op: &str,
    ) -> Result<T_, OpenMTRError> {
        let start = Instant::now();
        let url = format!("{}{}", self.config.base_url, path);

        let max_attempts: u32 = 3;
        let mut attempt: u32 = 0;

        loop {
            attempt += 1;
            let should_retry = attempt < max_attempts;
            let token = self.get_token().await?;

            let mut req = HttpRequest {
                method: method.to_string(),
                url: url.clone(),
                headers: vec![("Authorization".into(), format!("Bearer {token}"))],
                body: None,
            };

            if let Some(b) = body {
                req.body = Some(
                    serde_json::to_vec(b)
                        .map_err(|e| OpenMTRError::parse_error(e.to_string()))?,
                );
                req.headers.push(("Content-Type".into(), "application/json".into()));
            }

            match self.transport.send(req).await {
                Ok(resp) => {
                    let status = resp.status;

                    if status == 401 {
                        if attempt == 1 {
                            self.refresh_token().await?;
                            let new_token = self.get_token().await?;
                            let mut retry_req = HttpRequest {
                                method: method.to_string(),
                                url: url.clone(),
                                headers: vec![("Authorization".into(), format!("Bearer {new_token}"))],
                                body: body.map(|b| serde_json::to_vec(b).unwrap_or_default()),
                            };
                            if retry_req.body.is_some() {
                                retry_req.headers.push(("Content-Type".into(), "application/json".into()));
                            }
                            let retry_resp = self.transport.send(retry_req).await?;
                            if retry_resp.is_success() {
                                let elapsed = start.elapsed();
                                info!(
                                    operation = %op,
                                    status = %retry_resp.status,
                                    duration_ms = %elapsed.as_millis(),
                                    "Request succeeded after token refresh"
                                );
                                return retry_resp.json();
                            }
                            let body_text = String::from_utf8_lossy(&retry_resp.body);
                            return Err(map_http_error(
                                retry_resp.status,
                                &body_text,
                            ));
                        }
                        let body_text = String::from_utf8_lossy(&resp.body);
                        return Err(OpenMTRError::auth(format!(
                            "Unauthorized: {body_text}"
                        )));
                    }

                    if resp.is_success() {
                        let elapsed = start.elapsed();
                        info!(
                            operation = %op,
                            status = %status,
                            duration_ms = %elapsed.as_millis(),
                            "Request succeeded"
                        );
                        return resp.json();
                    }

                    if should_retry && (status == 429 || status >= 500) {
                        let delay = Duration::from_secs(1 << (attempt - 1));
                        warn!(
                            operation = %op,
                            status = %status,
                            attempt = %attempt,
                            retry_delay_ms = %delay.as_millis(),
                            "Retrying after error status"
                        );
                        sleep(delay).await;
                        continue;
                    }

                    let body_text = String::from_utf8_lossy(&resp.body);
                    return Err(map_http_error(status, &body_text));
                }
                Err(e) => {
                    if e.is_timeout() {
                        if should_retry {
                            let delay = Duration::from_secs(1 << (attempt - 1));
                            warn!(
                                operation = %op,
                                attempt = %attempt,
                                retry_delay_ms = %delay.as_millis(),
                                "Timeout, retrying"
                            );
                            sleep(delay).await;
                            continue;
                        }
                        return Err(e);
                    }
                    return Err(e);
                }
            }
        }
    }

    #[instrument(skip(self), fields(operation = %op, method = %method, path = %path))]
    pub async fn request_raw(
        &self,
        method: &str,
        path: &str,
        op: &str,
    ) -> Result<RawResponse, OpenMTRError> {
        let start = Instant::now();
        let url = format!("{}{}", self.config.base_url, path);

        let max_attempts: u32 = 3;
        let mut attempt: u32 = 0;

        loop {
            attempt += 1;
            let should_retry = attempt < max_attempts;
            let token = self.get_token().await?;

            let req = HttpRequest {
                method: method.to_string(),
                url: url.clone(),
                headers: vec![("Authorization".into(), format!("Bearer {token}"))],
                body: None,
            };

            match self.transport.send(req).await {
                Ok(resp) => {
                    let status = resp.status;

                    if status == 401 {
                        if attempt == 1 {
                            self.refresh_token().await?;
                            let new_token = self.get_token().await?;
                            let retry_req = HttpRequest {
                                method: method.to_string(),
                                url: url.clone(),
                                headers: vec![("Authorization".into(), format!("Bearer {new_token}"))],
                                body: None,
                            };
                            let retry_resp = self.transport.send(retry_req).await?;
                            if retry_resp.is_success() {
                                let elapsed = start.elapsed();
                                info!(
                                    operation = %op,
                                    status = %retry_resp.status,
                                    duration_ms = %elapsed.as_millis(),
                                    "Request succeeded after token refresh"
                                );
                                return Ok(RawResponse {
                                    bytes: retry_resp.bytes(),
                                    content_type: retry_resp.content_type,
                                    content_length: retry_resp.content_length,
                                });
                            }
                            let body_text = String::from_utf8_lossy(&retry_resp.body);
                            return Err(map_http_error(retry_resp.status, &body_text));
                        }
                        let body_text = String::from_utf8_lossy(&resp.body);
                        return Err(OpenMTRError::auth(format!(
                            "Unauthorized: {body_text}"
                        )));
                    }

                    if resp.is_success() {
                        let elapsed = start.elapsed();
                        info!(
                            operation = %op,
                            status = %status,
                            duration_ms = %elapsed.as_millis(),
                            "Request succeeded"
                        );
                        return Ok(RawResponse {
                            bytes: resp.bytes(),
                            content_type: resp.content_type,
                            content_length: resp.content_length,
                        });
                    }

                    if should_retry && (status == 429 || status >= 500) {
                        let delay = Duration::from_secs(1 << (attempt - 1));
                        warn!(
                            operation = %op,
                            status = %status,
                            attempt = %attempt,
                            retry_delay_ms = %delay.as_millis(),
                            "Retrying after error status"
                        );
                        sleep(delay).await;
                        continue;
                    }

                    let body_text = String::from_utf8_lossy(&resp.body);
                    return Err(map_http_error(status, &body_text));
                }
                Err(e) => {
                    if e.is_timeout() {
                        if should_retry {
                            let delay = Duration::from_secs(1 << (attempt - 1));
                            warn!(
                                operation = %op,
                                attempt = %attempt,
                                retry_delay_ms = %delay.as_millis(),
                                "Timeout, retrying"
                            );
                            sleep(delay).await;
                            continue;
                        }
                        return Err(e);
                    }
                    return Err(e);
                }
            }
        }
    }

    pub fn set_token_direct(&self, jwt: &str) {
        let now_secs = now_unix_secs();
        let ttl = TokenManager::extract_expiry(jwt)
            .map_or(3600, |exp| exp.saturating_sub(now_secs));
        let safe = ttl.saturating_sub(60);
        let expires = Instant::now() + std::time::Duration::from_secs(safe);
        let mut mgr = self.token_mgr.write().expect("lock poisoned");
        *mgr = TokenManager::from_cached(jwt.to_string(), expires);
    }

    pub fn set_refresh_url(&self, url: &str) {
        let mut mgr = self.token_mgr.write().expect("lock poisoned");
        mgr.set_refresh_url(url);
    }

    pub async fn refresh_token(&self) -> Result<(), OpenMTRError> {
        let refresh_req = {
            let mgr = self.token_mgr.read().expect("lock poisoned");
            mgr.force_refresh_data()
        };
        let (url, integration_token) = refresh_req.ok_or_else(|| {
            OpenMTRError::network("Refresh URL not set or token not needed")
        })?;

        let req = HttpRequest::post(&url)
            .header("Authorization", format!("Bearer {integration_token}"));
        let resp = self.transport.send(req).await?;

        if !resp.is_success() {
            return Err(OpenMTRError::http_error(resp.status, "Token refresh failed", ""));
        }

        let body: serde_json::Value = resp.json()?;
        if body["erro"].as_bool().unwrap_or(true) {
            return Err(OpenMTRError::http_error(
                resp.status,
                "Token refresh returned error",
                body["detail"].as_str().unwrap_or(""),
            ));
        }

        let objeto_resposta = body["objetoResposta"].as_str().ok_or_else(|| {
            OpenMTRError::parse_error("Missing objetoResposta in refresh response")
        })?;
        let token = objeto_resposta
            .strip_prefix("Bearer ")
            .ok_or_else(|| OpenMTRError::parse_error("objetoResposta does not start with 'Bearer '"))?;

        let mut mgr = self.token_mgr.write().expect("lock poisoned");
        mgr.set_cached_token(token.to_string());
        Ok(())
    }

    pub async fn post_public<B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
        op: &str,
    ) -> Result<serde_json::Value, OpenMTRError> {
        let url = format!("{}{}", self.config.base_url, path);
        let req = HttpRequest {
            method: "POST".into(),
            url,
            headers: vec![("Content-Type".into(), "application/json".into())],
            body: Some(
                serde_json::to_vec(body)
                    .map_err(|e| OpenMTRError::parse_error(e.to_string()))?,
            ),
        };
        let resp = self.transport.send(req).await.map_err(|e| {
            if e.is_timeout() {
                OpenMTRError::timeout(format!("{op} timed out"))
            } else {
                e
            }
        })?;
        if resp.is_success() {
            resp.json()
        } else {
            let body_text = String::from_utf8_lossy(&resp.body);
            Err(map_http_error(resp.status, &body_text))
        }
    }

    pub async fn get_token(&self) -> Result<String, OpenMTRError> {
        {
            let guard = self.token_mgr.read().expect("lock poisoned");
            if let Some(t) = guard.token() {
                return Ok(t.to_string());
            }
        }

        self.refresh_token().await?;

        let guard = self.token_mgr.read().expect("lock poisoned");
        guard
            .token()
            .ok_or_else(|| OpenMTRError::auth("No token available after refresh"))
            .map(std::string::ToString::to_string)
    }
}

fn map_http_error(status: u16, detail: &str) -> OpenMTRError {
    match status {
        401 => OpenMTRError::auth(format!("Unauthorized: {detail}")),
        403 => OpenMTRError::new(403, "Forbidden", detail),
        404 => OpenMTRError::new(404, "Not Found", detail),
        422 => OpenMTRError::validation(detail),
        429 => OpenMTRError::new(429, "Too Many Requests", detail),
        s if s >= 500 => OpenMTRError::new(s, "Server Error", detail),
        _ => OpenMTRError::new(status, "HTTP Error", detail),
    }
}

#[cfg(feature = "native-http")]
async fn sleep(d: Duration) {
    tokio::time::sleep(d).await;
}

#[cfg(all(not(feature = "native-http"), target_arch = "wasm32"))]
async fn sleep(_d: Duration) {
    // No-op: single-threaded WASM — retry without delay
}

#[cfg(all(not(feature = "native-http"), not(target_arch = "wasm32")))]
async fn sleep(d: Duration) {
    std::thread::sleep(d);
}

#[cfg(all(not(feature = "native-http"), not(target_arch = "wasm32")))]
async fn sleep(d: Duration) {
    std::thread::sleep(d);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_http_error_401() {
        let err = map_http_error(401, "invalid token");
        assert_eq!(err.status, 401);
    }

    #[test]
    fn test_map_http_error_403() {
        let err = map_http_error(403, "forbidden");
        assert_eq!(err.status, 403);
    }

    #[test]
    fn test_map_http_error_404() {
        let err = map_http_error(404, "not found");
        assert_eq!(err.status, 404);
    }

    #[test]
    fn test_map_http_error_422() {
        let err = map_http_error(422, "validation error");
        assert_eq!(err.status, 422);
    }

    #[test]
    fn test_map_http_error_429() {
        let err = map_http_error(429, "rate limited");
        assert_eq!(err.status, 429);
    }

    #[test]
    fn test_map_http_error_500() {
        let err = map_http_error(500, "server error");
        assert_eq!(err.status, 500);
    }

    #[test]
    fn test_map_http_error_unknown() {
        let err = map_http_error(418, "teapot");
        assert_eq!(err.status, 418);
    }
}
