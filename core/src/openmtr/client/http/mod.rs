//! Cliente HTTP do openMTR sobre um [`HttpTransport`](crate::openmtr::transport::HttpTransport) injetável.
//!
//! O módulo coordena autenticação Bearer, retry e limite de concorrência;
//! o módulo `execute` contém o loop de requisição, `token` a renovação de token e
//! `response` o tipo de resposta bruta usado por downloads.

mod execute;
mod response;
mod token;

use serde::de::DeserializeOwned;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, RwLock};

use futures::channel::oneshot;
use futures::lock::Mutex as AsyncMutex;

use crate::openmtr::client::token::TokenManager;
use crate::openmtr::config::Config;
use crate::openmtr::error::OpenMTRError;
use crate::openmtr::transport::HttpTransport;

pub use response::RawResponse;

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
pub(super) enum AuthMode {
    Bearer,
    None,
}

/// Cliente HTTP do openMTR construído sobre um [`HttpTransport`] injetável.
///
/// Centraliza autenticação Bearer, retry e limite de concorrência. Cada
/// requisição tenta no máximo 3 vezes, com backoff de 1s e 2s, em respostas
/// `429`, `5xx` e em timeouts do transporte; outros erros não são repetidos.
/// Um `401` na primeira tentativa dispara uma renovação de token protegida por
/// mutex assíncrono (single-flight): requisições concorrentes compartilham a
/// mesma renovação. A concorrência de rede é limitada por
/// [`Config::max_concurrency`].
pub struct MtrHttpClient {
    pub(super) transport: Arc<dyn HttpTransport>,
    pub(super) config: Config,
    pub(super) token_mgr: Arc<RwLock<TokenManager>>,
    pub(super) refresh_lock: AsyncMutex<()>,
    limiter: ConcurrencyLimiter,
}

impl MtrHttpClient {
    /// Cria o cliente com a configuração e o transporte HTTP fornecidos.
    ///
    /// O `integration_token` de [`Config`] alimenta o [`TokenManager`] como
    /// refresh token; use [`Self::set_refresh_url`] para habilitar a renovação.
    /// O transporte é injetado para permitir substituí-lo em testes.
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

    /// Devolve a configuração usada pelo cliente.
    #[allow(
        dead_code,
        reason = "campo/helper mantido para compatibilidade da API e testes"
    )]
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Devolve o gerenciador de tokens compartilhado, protegido por `RwLock`.
    #[allow(
        dead_code,
        reason = "campo/helper mantido para compatibilidade da API e testes"
    )]
    #[must_use]
    pub fn token_mgr(&self) -> &Arc<RwLock<TokenManager>> {
        &self.token_mgr
    }

    /// Executa um `GET` autenticado e desserializa a resposta como JSON.
    ///
    /// `path` é relativo a [`Config::base_url`] via [`Config::endpoint`]; `op`
    /// identifica a operação nos logs e spans de tracing.
    pub async fn get<T_: DeserializeOwned>(
        &self,
        path: &str,
        op: &str,
    ) -> Result<T_, OpenMTRError> {
        self.request::<T_, ()>("GET", path, None::<&()>, op).await
    }

    /// Executa um `POST` autenticado com corpo JSON e desserializa a resposta.
    pub async fn post<T_: DeserializeOwned, B: serde::Serialize + Sync>(
        &self,
        path: &str,
        body: &B,
        op: &str,
    ) -> Result<T_, OpenMTRError> {
        self.request("POST", path, Some(body), op).await
    }

    /// Executa um `POST` autenticado e devolve a resposta bruta, sem desserializar.
    ///
    /// Útil para downloads: aplique [`RawResponse::validate`] antes de usar os bytes.
    pub async fn post_bytes(&self, path: &str, op: &str) -> Result<RawResponse, OpenMTRError> {
        self.request_raw("POST", path, op).await
    }

    /// Executa um `GET` autenticado e devolve a resposta bruta, sem desserializar.
    #[allow(
        dead_code,
        reason = "campo/helper mantido para compatibilidade da API e testes"
    )]
    pub async fn get_bytes(&self, path: &str, op: &str) -> Result<RawResponse, OpenMTRError> {
        self.request_raw("GET", path, op).await
    }
}

#[cfg(test)]
mod tests;
