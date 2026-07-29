use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use reqwest::{Client, Method, StatusCode};
use serde::de::DeserializeOwned;
use tracing::{info, warn, instrument};

use crate::openmtr::config::Config;
use crate::openmtr::client::token::TokenManager;
use crate::openmtr::error::OpenMTRError;

pub(crate) struct MtrHttpClient {
    http: Client,
    config: Config,
    token_mgr: Arc<RwLock<TokenManager>>,
}

impl MtrHttpClient {
    pub fn new(config: Config) -> Result<Self, OpenMTRError> {
        let timeout = config.timeout;
        let http = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| OpenMTRError::network(e.to_string()))?;

        let token_mgr = Arc::new(RwLock::new(TokenManager::new(
            config.integration_token.clone().unwrap_or_default(),
        )));

        Ok(Self {
            http,
            config,
            token_mgr,
        })
    }

    #[allow(dead_code)]
    pub fn config(&self) -> &Config {
        &self.config
    }

    #[allow(dead_code)]
    pub fn token_mgr(&self) -> &Arc<RwLock<TokenManager>> {
        &self.token_mgr
    }

    pub async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        op: &str,
    ) -> Result<T, OpenMTRError> {
        self.request::<T, ()>(Method::GET, path, None::<&()>, op).await
    }

    pub async fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
        op: &str,
    ) -> Result<T, OpenMTRError> {
        self.request(Method::POST, path, Some(body), op).await
    }

    pub async fn post_bytes(
        &self,
        path: &str,
        op: &str,
    ) -> Result<Vec<u8>, OpenMTRError> {
        self.request_raw(Method::POST, path, op).await
    }

    #[allow(dead_code)]
    pub async fn get_bytes(
        &self,
        path: &str,
        op: &str,
    ) -> Result<Vec<u8>, OpenMTRError> {
        self.request_raw(Method::GET, path, op).await
    }

    #[instrument(skip(self, body), fields(operation = %op, method = %method, path = %path))]
    pub async fn request<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
        op: &str,
    ) -> Result<T, OpenMTRError> {
        let start = Instant::now();
        let url = format!("{}{}", self.config.base_url, path);

        let max_attempts: u32 = 3;
        let mut attempt: u32 = 0;

        loop {
            attempt += 1;
            let should_retry = attempt < max_attempts;
            let token = self.get_token().await?;

            let mut req = self
                .http
                .request(method.clone(), &url)
                .header("Authorization", format!("Bearer {token}"));

            if let Some(b) = body {
                req = req.json(b);
            }

            match req.send().await {
                Ok(resp) => {
                    let status = resp.status();

                    if status == StatusCode::UNAUTHORIZED {
                        if attempt == 1 {
                            self.refresh_token().await?;
                            let new_token = self.get_token().await?;
                            let mut retry_req = self
                                .http
                                .request(method.clone(), &url)
                                .header("Authorization", format!("Bearer {new_token}"));
                            if let Some(b) = body {
                                retry_req = retry_req.json(b);
                            }
                            let retry_resp = retry_req.send().await.map_err(|e| {
                                if e.is_timeout() {
                                    OpenMTRError::timeout("retry after 401 timeout")
                                } else {
                                    OpenMTRError::network(e.to_string())
                                }
                            })?;
                            if retry_resp.status().is_success() {
                                let elapsed = start.elapsed();
                                info!(
                                    operation = %op,
                                    status = %retry_resp.status(),
                                    duration_ms = %elapsed.as_millis(),
                                    "Request succeeded after token refresh"
                                );
                                return retry_resp
                                    .json::<T>()
                                    .await
                                    .map_err(|e| OpenMTRError::parse_error(e.to_string()));
                            }
                            let status_code = retry_resp.status();
                            let body_text = retry_resp.text().await.unwrap_or_default();
                            return Err(map_http_error(status_code, &body_text));
                        }
                        let body_text = resp.text().await.unwrap_or_default();
                        return Err(OpenMTRError::auth(format!("Unauthorized: {body_text}")));
                    }

                    if status.is_success() {
                        let elapsed = start.elapsed();
                        info!(
                            operation = %op,
                            status = %status,
                            duration_ms = %elapsed.as_millis(),
                            "Request succeeded"
                        );
                        return resp
                            .json::<T>()
                            .await
                            .map_err(|e| OpenMTRError::parse_error(e.to_string()));
                    }

                    if should_retry && (status.as_u16() == 429 || status.as_u16() >= 500) {
                        let delay = Duration::from_secs(1 << (attempt - 1));
                        warn!(
                            operation = %op,
                            status = %status,
                            attempt = %attempt,
                            retry_delay_ms = %delay.as_millis(),
                            "Retrying after error status"
                        );
                        tokio::time::sleep(delay).await;
                        continue;
                    }

                    let body_text = resp.text().await.unwrap_or_default();
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
                            tokio::time::sleep(delay).await;
                            continue;
                        }
                        return Err(OpenMTRError::timeout(format!("request timed out: {op}")));
                    }
                    return Err(OpenMTRError::network(e.to_string()));
                }
            }
        }
    }

    #[instrument(skip(self), fields(operation = %op, method = %method, path = %path))]
    pub async fn request_raw(
        &self,
        method: Method,
        path: &str,
        op: &str,
    ) -> Result<Vec<u8>, OpenMTRError> {
        let start = Instant::now();
        let url = format!("{}{}", self.config.base_url, path);

        let max_attempts: u32 = 3;
        let mut attempt: u32 = 0;

        loop {
            attempt += 1;
            let should_retry = attempt < max_attempts;
            let token = self.get_token().await?;

            let req = self
                .http
                .request(method.clone(), &url)
                .header("Authorization", format!("Bearer {token}"));

            match req.send().await {
                Ok(resp) => {
                    let status = resp.status();

                    if status == StatusCode::UNAUTHORIZED {
                        if attempt == 1 {
                            self.refresh_token().await?;
                            let new_token = self.get_token().await?;
                            let retry_req = self
                                .http
                                .request(method.clone(), &url)
                                .header("Authorization", format!("Bearer {new_token}"));
                            let retry_resp = retry_req.send().await.map_err(|e| {
                                if e.is_timeout() {
                                    OpenMTRError::timeout("retry after 401 timeout")
                                } else {
                                    OpenMTRError::network(e.to_string())
                                }
                            })?;
                            if retry_resp.status().is_success() {
                                let elapsed = start.elapsed();
                                info!(
                                    operation = %op,
                                    status = %retry_resp.status(),
                                    duration_ms = %elapsed.as_millis(),
                                    "Request succeeded after token refresh"
                                );
                                return retry_resp
                                    .bytes()
                                    .await
                                    .map(|b| b.to_vec())
                                    .map_err(|e| OpenMTRError::network(e.to_string()));
                            }
                            let status_code = retry_resp.status();
                            let body_text = retry_resp.text().await.unwrap_or_default();
                            return Err(map_http_error(status_code, &body_text));
                        }
                        let body_text = resp.text().await.unwrap_or_default();
                        return Err(OpenMTRError::auth(format!("Unauthorized: {body_text}")));
                    }

                    if status.is_success() {
                        let elapsed = start.elapsed();
                        info!(
                            operation = %op,
                            status = %status,
                            duration_ms = %elapsed.as_millis(),
                            "Request succeeded"
                        );
                        return resp
                            .bytes()
                            .await
                            .map(|b| b.to_vec())
                            .map_err(|e| OpenMTRError::network(e.to_string()));
                    }

                    if should_retry && (status.as_u16() == 429 || status.as_u16() >= 500) {
                        let delay = Duration::from_secs(1 << (attempt - 1));
                        warn!(
                            operation = %op,
                            status = %status,
                            attempt = %attempt,
                            retry_delay_ms = %delay.as_millis(),
                            "Retrying after error status"
                        );
                        tokio::time::sleep(delay).await;
                        continue;
                    }

                    let body_text = resp.text().await.unwrap_or_default();
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
                            tokio::time::sleep(delay).await;
                            continue;
                        }
                        return Err(OpenMTRError::timeout(format!("request timed out: {op}")));
                    }
                    return Err(OpenMTRError::network(e.to_string()));
                }
            }
        }
    }

    pub async fn set_refresh_url(&self, url: &str) {
        let mut mgr = self.token_mgr.write().await;
        mgr.set_refresh_url(url);
    }

    pub async fn refresh_token(&self) -> Result<(), OpenMTRError> {
        let mut mgr = self.token_mgr.write().await;
        mgr.refresh(&self.http).await
    }

    pub async fn get_token(&self) -> Result<String, OpenMTRError> {
        let guard = self.token_mgr.read().await;
        if let Some(t) = guard.token() {
            return Ok(t.to_string());
        }
        drop(guard);
        let mut guard = self.token_mgr.write().await;
        guard.refresh(&self.http).await?;
        guard
            .token()
            .ok_or_else(|| OpenMTRError::auth("No token available after refresh"))
            .map(|t| t.to_string())
    }
}

fn map_http_error(status: StatusCode, detail: &str) -> OpenMTRError {
    match status.as_u16() {
        401 => OpenMTRError::auth(format!("Unauthorized: {detail}")),
        403 => OpenMTRError::new(403, "Forbidden", detail),
        404 => OpenMTRError::new(404, "Not Found", detail),
        422 => OpenMTRError::validation(detail),
        429 => OpenMTRError::new(429, "Too Many Requests", detail),
        s if s >= 500 => OpenMTRError::new(s, "Server Error", detail),
        _ => OpenMTRError::new(status.as_u16(), "HTTP Error", detail),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_http_error_401() {
        let err = map_http_error(StatusCode::UNAUTHORIZED, "invalid token");
        assert_eq!(err.status, 401);
    }

    #[test]
    fn test_map_http_error_403() {
        let err = map_http_error(StatusCode::FORBIDDEN, "forbidden");
        assert_eq!(err.status, 403);
    }

    #[test]
    fn test_map_http_error_404() {
        let err = map_http_error(StatusCode::NOT_FOUND, "not found");
        assert_eq!(err.status, 404);
    }

    #[test]
    fn test_map_http_error_422() {
        let err = map_http_error(StatusCode::UNPROCESSABLE_ENTITY, "validation error");
        assert_eq!(err.status, 422);
    }

    #[test]
    fn test_map_http_error_429() {
        let err = map_http_error(StatusCode::TOO_MANY_REQUESTS, "rate limited");
        assert_eq!(err.status, 429);
    }

    #[test]
    fn test_map_http_error_500() {
        let err = map_http_error(StatusCode::INTERNAL_SERVER_ERROR, "server error");
        assert_eq!(err.status, 500);
    }

    #[test]
    fn test_map_http_error_unknown() {
        let err = map_http_error(StatusCode::from_u16(418).unwrap_or(StatusCode::IM_A_TEAPOT), "teapot");
        assert_eq!(err.status, 418);
    }
}
