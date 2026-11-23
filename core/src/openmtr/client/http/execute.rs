//! Execução de requisições: backoff, retry e conversão de respostas.

use serde::de::DeserializeOwned;
use std::time::Duration;
use tracing::{info, instrument, warn};

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

use crate::openmtr::error::OpenMTRError;
use crate::openmtr::transport::{HttpRequest, HttpResponse};

use super::{AuthMode, MtrHttpClient, RawResponse};

impl MtrHttpClient {
    /// Executa uma requisição autenticada com corpo opcional e desserializa o JSON.
    ///
    /// Quando `body` é `Some`, ele é serializado como JSON e o cabeçalho
    /// `Content-Type: application/json` é enviado.
    #[instrument(skip(self, body), fields(operation = %op, method = %method, path = %path))]
    pub async fn request<T_: DeserializeOwned, B: serde::Serialize + Sync>(
        &self,
        method: &str,
        path: &str,
        body: Option<&B>,
        op: &str,
    ) -> Result<T_, OpenMTRError> {
        let body_bytes = body
            .map(serde_json::to_vec)
            .transpose()
            .map_err(|e| OpenMTRError::parse_error(e.to_string()))?;
        let resp = self
            .execute(method, path, body_bytes, op, AuthMode::Bearer)
            .await?;
        resp.json()
    }

    /// Executa uma requisição autenticada sem corpo e devolve a resposta bruta.
    #[instrument(skip(self), fields(operation = %op, method = %method, path = %path))]
    pub async fn request_raw(
        &self,
        method: &str,
        path: &str,
        op: &str,
    ) -> Result<RawResponse, OpenMTRError> {
        let resp = self
            .execute(method, path, None, op, AuthMode::Bearer)
            .await?;
        Ok(RawResponse {
            bytes: resp.bytes(),
            content_type: resp.content_type,
            content_length: resp.content_length,
        })
    }

    async fn execute(
        &self,
        method: &str,
        path: &str,
        body: Option<Vec<u8>>,
        op: &str,
        auth: AuthMode,
    ) -> Result<HttpResponse, OpenMTRError> {
        let start = Instant::now();
        let url = self.config.endpoint(path);

        let max_attempts: u32 = 3;
        let mut attempt: u32 = 0;

        loop {
            attempt = attempt.saturating_add(1);
            let should_retry = attempt < max_attempts;
            let token = match auth {
                AuthMode::Bearer => Some(self.get_token().await?),
                AuthMode::None => None,
            };

            let mut headers = Vec::new();
            if let Some(token) = &token {
                headers.push(("Authorization".into(), format!("Bearer {token}")));
            }
            if body.is_some() {
                headers.push(("Content-Type".into(), "application/json".into()));
            }

            let req = HttpRequest {
                method: method.to_string(),
                url: url.clone(),
                headers,
                body: body.clone(),
            };

            let permit = self.limiter.acquire().await;
            let send_result = self.transport.send(req).await;
            drop(permit);

            match send_result {
                Ok(resp) => {
                    let status = resp.status;

                    if status == 401 {
                        if auth == AuthMode::Bearer && attempt == 1 {
                            let stale = token.as_deref().unwrap_or_default();
                            self.refresh_token_for(stale).await?;
                            continue;
                        }
                        let body_text = String::from_utf8_lossy(&resp.body);
                        return Err(OpenMTRError::auth(format!("Unauthorized: {body_text}")));
                    }

                    if resp.is_success() {
                        let elapsed = start.elapsed();
                        info!(
                            operation = %op,
                            status = %status,
                            duration_ms = %elapsed.as_millis(),
                            "Request succeeded"
                        );
                        return Ok(resp);
                    }

                    if should_retry && (status == 429 || status >= 500) {
                        let delay = Duration::from_secs(1 << (attempt.saturating_sub(1)));
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
                            let delay = Duration::from_secs(1 << (attempt.saturating_sub(1)));
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

    /// Executa um `POST` sem autenticação e desserializa a resposta como JSON.
    ///
    /// Timeouts do transporte são convertidos em [`OpenMTRError::timeout`] com
    /// mensagem identificando `op`.
    pub async fn post_public<B: serde::Serialize + Sync>(
        &self,
        path: &str,
        body: &B,
        op: &str,
    ) -> Result<serde_json::Value, OpenMTRError> {
        let body_bytes =
            serde_json::to_vec(body).map_err(|e| OpenMTRError::parse_error(e.to_string()))?;
        let resp = self
            .execute("POST", path, Some(body_bytes), op, AuthMode::None)
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    OpenMTRError::timeout(format!("{op} timed out"))
                } else {
                    e
                }
            })?;
        resp.json()
    }
}

pub(super) fn map_http_error(status: u16, detail: &str) -> OpenMTRError {
    match status {
        401 => OpenMTRError::auth(format!("Unauthorized: {detail}")),
        403 => OpenMTRError::http_error(403, "Forbidden", detail),
        404 => OpenMTRError::not_found(detail),
        422 => OpenMTRError::validation(detail),
        429 => OpenMTRError::http_error(429, "Too Many Requests", detail),
        s if s >= 500 => OpenMTRError::http_error(s, "Server Error", detail),
        _ => OpenMTRError::http_error(status, "HTTP Error", detail),
    }
}

#[cfg(feature = "native-http")]
pub(super) async fn sleep(d: Duration) {
    tokio::time::sleep(d).await;
}

#[cfg(not(feature = "native-http"))]
pub(super) async fn sleep(_d: Duration) {
    // Sem `native-http` não há timer de runtime disponível. Em WASM o retry é
    // imediato; em hosts nativos a feature `native-http` deve ser habilitada
    // para obter backoff real sem bloquear o executor.
}
