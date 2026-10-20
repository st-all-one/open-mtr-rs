#![allow(
    clippy::too_many_lines,
    reason = "cliente HTTP legado; será dividido durante a revitalização"
)]

use serde::de::DeserializeOwned;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use tracing::{info, instrument, warn};

use futures::channel::oneshot;
use futures::lock::Mutex as AsyncMutex;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

use crate::openmtr::client::token::TokenManager;
use crate::openmtr::config::Config;
use crate::openmtr::error::OpenMTRError;
use crate::openmtr::transport::{HttpRequest, HttpResponse, HttpTransport};

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
                return Err(format!(
                    "Content-Type inesperado: {ct} (esperado {expected})"
                ));
            }
            if expected == "xml" && !ct_lower.contains("xml") && !ct_lower.contains("octet-stream")
            {
                return Err(format!(
                    "Content-Type inesperado: {ct} (esperado application/xml)"
                ));
            }
        }

        let magic_ok = match expected {
            "pdf" => self.bytes.starts_with(b"%PDF"),
            "xml" => self.bytes.starts_with(b"<?xml") || self.bytes.starts_with(b"<"),
            _ => true,
        };
        if !magic_ok {
            return Err(format!(
                "Arquivo inválido: magic bytes não correspondem ao formato {expected}"
            ));
        }

        if let Some(cl) = self.content_length
            && cl != u64::try_from(self.bytes.len()).unwrap_or(u64::MAX)
        {
            return Err(format!(
                "Content-Length ({cl}) difere dos bytes recebidos ({})",
                self.bytes.len()
            ));
        }

        Ok(())
    }
}

/// Semáforo assíncrono simples usado para limitar a concorrência de rede.
struct ConcurrencyLimiter {
    limit: usize,
    state: Mutex<LimiterState>,
}

struct LimiterState {
    active: usize,
    waiters: VecDeque<oneshot::Sender<()>>,
}

struct Permit<'a> {
    limiter: &'a ConcurrencyLimiter,
}

impl ConcurrencyLimiter {
    fn new(limit: usize) -> Self {
        Self {
            limit: limit.max(1),
            state: Mutex::new(LimiterState {
                active: 0,
                waiters: VecDeque::new(),
            }),
        }
    }

    async fn acquire(&self) -> Permit<'_> {
        loop {
            let receiver = {
                let mut state = self
                    .state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if state.active < self.limit {
                    state.active = state.active.saturating_add(1);
                    None
                } else {
                    let (tx, rx) = oneshot::channel();
                    state.waiters.push_back(tx);
                    Some(rx)
                }
            };

            match receiver {
                None => return Permit { limiter: self },
                Some(rx) => {
                    if rx.await.is_ok() {
                        return Permit { limiter: self };
                    }
                }
            }
        }
    }
}

impl Drop for Permit<'_> {
    fn drop(&mut self) {
        let mut state = self
            .limiter
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        while let Some(waiter) = state.waiters.pop_front() {
            if waiter.send(()).is_ok() {
                return;
            }
        }
        state.active = state.active.saturating_sub(1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuthMode {
    Bearer,
    None,
}

pub struct MtrHttpClient {
    transport: Arc<dyn HttpTransport>,
    config: Config,
    token_mgr: Arc<RwLock<TokenManager>>,
    refresh_lock: AsyncMutex<()>,
    limiter: ConcurrencyLimiter,
}

impl MtrHttpClient {
    pub fn new(config: Config, transport: impl HttpTransport + 'static) -> Self {
        let integration_token = config.integration_token.clone().unwrap_or_default();
        let limiter = ConcurrencyLimiter::new(config.max_concurrency());

        Self {
            transport: Arc::new(transport),
            token_mgr: Arc::new(RwLock::new(TokenManager::new(integration_token))),
            config,
            refresh_lock: AsyncMutex::new(()),
            limiter,
        }
    }

    #[allow(
        dead_code,
        reason = "campo/helper mantido para compatibilidade da API e testes"
    )]
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }

    #[allow(
        dead_code,
        reason = "campo/helper mantido para compatibilidade da API e testes"
    )]
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

    pub async fn post<T_: DeserializeOwned, B: serde::Serialize + Sync>(
        &self,
        path: &str,
        body: &B,
        op: &str,
    ) -> Result<T_, OpenMTRError> {
        self.request("POST", path, Some(body), op).await
    }

    pub async fn post_bytes(&self, path: &str, op: &str) -> Result<RawResponse, OpenMTRError> {
        self.request_raw("POST", path, op).await
    }

    #[allow(
        dead_code,
        reason = "campo/helper mantido para compatibilidade da API e testes"
    )]
    pub async fn get_bytes(&self, path: &str, op: &str) -> Result<RawResponse, OpenMTRError> {
        self.request_raw("GET", path, op).await
    }

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
            if let Some(ref token) = token {
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

    pub fn set_token_direct(&self, jwt: &str) {
        let mut mgr = self
            .token_mgr
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // Preserva o token de integração/refresh já configurado; apenas o access
        // token em cache é substituído.
        mgr.set_cached_token(jwt.to_string());
    }

    pub fn set_refresh_url(&self, url: &str) {
        let mut mgr = self
            .token_mgr
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        mgr.set_refresh_url(url);
    }

    pub async fn refresh_token(&self) -> Result<(), OpenMTRError> {
        let _guard = self.refresh_lock.lock().await;
        self.refresh_token_locked().await
    }

    async fn refresh_token_for(&self, stale_token: &str) -> Result<(), OpenMTRError> {
        let _guard = self.refresh_lock.lock().await;
        {
            let mgr = self
                .token_mgr
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(current) = mgr.token()
                && current != stale_token
            {
                // Outra requisição concorrente já renovou o access token.
                return Ok(());
            }
        }
        self.refresh_token_locked().await
    }

    async fn refresh_token_locked(&self) -> Result<(), OpenMTRError> {
        let refresh_req = {
            let mgr = self
                .token_mgr
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            mgr.force_refresh_data()
        };
        let (url, integration_token) = refresh_req
            .ok_or_else(|| OpenMTRError::network("Refresh URL not set or token not needed"))?;

        let req =
            HttpRequest::post(&url).header("Authorization", format!("Bearer {integration_token}"));
        let resp = self.transport.send(req).await?;

        if !resp.is_success() {
            return Err(OpenMTRError::http_error(
                resp.status,
                "Token refresh failed",
                "",
            ));
        }

        let body: serde_json::Value = resp.json()?;
        if body
            .get("erro")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true)
        {
            return Err(OpenMTRError::http_error(
                resp.status,
                "Token refresh returned error",
                body.get("detail")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(""),
            ));
        }

        let objeto_resposta = body
            .get("objetoResposta")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                OpenMTRError::parse_error("Missing objetoResposta in refresh response")
            })?;
        let token = objeto_resposta.strip_prefix("Bearer ").ok_or_else(|| {
            OpenMTRError::parse_error("objetoResposta does not start with 'Bearer '")
        })?;

        let mut mgr = self
            .token_mgr
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        mgr.set_cached_token(token.to_string());
        Ok(())
    }

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

    pub async fn get_token(&self) -> Result<String, OpenMTRError> {
        {
            let guard = self
                .token_mgr
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(t) = guard.token() {
                return Ok(t.to_string());
            }
        }

        self.refresh_token_if_needed().await?;

        let guard = self
            .token_mgr
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard
            .token()
            .ok_or_else(|| OpenMTRError::auth("No token available after refresh"))
            .map(ToString::to_string)
    }

    async fn refresh_token_if_needed(&self) -> Result<(), OpenMTRError> {
        let _guard = self.refresh_lock.lock().await;
        {
            let mgr = self
                .token_mgr
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if mgr.token().is_some() {
                return Ok(());
            }
        }
        self.refresh_token_locked().await
    }
}

fn map_http_error(status: u16, detail: &str) -> OpenMTRError {
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
async fn sleep(d: Duration) {
    tokio::time::sleep(d).await;
}

#[cfg(not(feature = "native-http"))]
async fn sleep(_d: Duration) {
    // Sem `native-http` não há timer de runtime disponível. Em WASM o retry é
    // imediato; em hosts nativos a feature `native-http` deve ser habilitada
    // para obter backoff real sem bloquear o executor.
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Clone)]
    struct CountingTransport {
        calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl HttpTransport for CountingTransport {
        async fn send(&self, _req: HttpRequest) -> Result<HttpResponse, OpenMTRError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let body = serde_json::to_vec(&serde_json::json!({
                "erro": false,
                "objetoResposta": "Bearer access-token"
            }))
            .map_err(|e| OpenMTRError::parse_error(e.to_string()))?;
            let len = u64::try_from(body.len()).unwrap_or(u64::MAX);
            Ok(HttpResponse {
                status: 200,
                body,
                content_type: Some("application/json".into()),
                content_length: Some(len),
            })
        }
    }

    #[tokio::test]
    async fn concurrent_get_token_refreshes_once() {
        let transport = CountingTransport {
            calls: Arc::new(AtomicUsize::new(0)),
        };
        let config = Config::builder()
            .base_url("https://example.com")
            .token("refresh-token")
            .build()
            .expect("config válida");
        let http = MtrHttpClient::new(config, transport.clone());
        http.set_refresh_url("https://example.com/token");

        let (first, second) = tokio::join!(http.get_token(), http.get_token());

        assert_eq!(first.expect("primeiro token"), "access-token");
        assert_eq!(second.expect("segundo token"), "access-token");
        assert_eq!(transport.calls.load(Ordering::SeqCst), 1);
    }

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
