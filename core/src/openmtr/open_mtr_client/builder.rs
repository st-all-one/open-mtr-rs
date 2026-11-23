use std::sync::Arc;
use std::time::Duration;

use crate::openmtr::adapters::MtrCtx;
use crate::openmtr::client::http::MtrHttpClient;
use crate::openmtr::config::Config;
use crate::openmtr::error::OpenMTRError;
use crate::openmtr::system::System;
use crate::openmtr::time_util::{Clock, IdGenerator};
use crate::openmtr::transport::HttpTransport;
use tracing::info;

use super::OpenMtrClient;

/// Builder de [`OpenMtrClient`], com pontos de injeção de tempo e identificadores.
#[derive(Default)]
pub struct OpenMtrClientBuilder {
    system: Option<System>,
    base_url: Option<String>,
    integration_token: Option<String>,
    access_token: Option<String>,
    timeout: Option<Duration>,
    max_rps: Option<u32>,
    clock: Option<Arc<dyn Clock>>,
    id_generator: Option<Arc<dyn IdGenerator>>,
}

impl OpenMtrClientBuilder {
    /// Define o sistema alvo; obrigatório e usado para resolver adaptador e URL base.
    #[must_use]
    pub fn base(mut self, system: System) -> Self {
        self.system = Some(system);
        self
    }

    /// Sobrescreve a URL base padrão do sistema (homologação ou proxy).
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Define o integration token usado na autenticação do adapter.
    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.integration_token = Some(token.into());
        self
    }

    /// Usa um access token já emitido, sem passar pela autenticação do adapter.
    ///
    /// É o caminho do cache de sessão: o token é instalado diretamente no
    /// cliente e a construção não faz `POST /token` nem login clássico.
    pub fn access_token(mut self, token: impl Into<String>) -> Self {
        self.access_token = Some(token.into());
        self
    }

    /// Define o timeout das requisições HTTP, em segundos.
    #[must_use]
    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout = Some(Duration::from_secs(secs));
        self
    }

    /// Limita a taxa de requisições (requisições por segundo).
    #[must_use]
    pub fn max_rps(mut self, rps: u32) -> Self {
        self.max_rps = Some(rps);
        self
    }

    /// Injeta um relógio determinístico; controla `today()` e os campos de data do domínio.
    #[must_use]
    pub fn clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = Some(clock);
        self
    }

    /// Injeta um gerador determinístico de IDs; controla o `id` do envelope [`crate::openmtr::resposta::Resposta`].
    #[must_use]
    pub fn id_generator(mut self, id_generator: Arc<dyn IdGenerator>) -> Self {
        self.id_generator = Some(id_generator);
        self
    }

    /// Constrói o cliente com transporte reqwest nativo (feature `native-http`).
    ///
    /// Em outros runtimes, use [`Self::build_with_transport`].
    #[cfg(feature = "native-http")]
    pub async fn build(self) -> Result<OpenMtrClient, OpenMTRError> {
        let transport = crate::openmtr::transport::reqwest::ReqwestTransport::new(
            self.timeout.unwrap_or(Duration::from_mins(1)),
        )?;
        self.build_with_transport(transport).await
    }

    /// Constrói o cliente com um transporte HTTP plugável.
    ///
    /// A URL base vem de [`System::default_base_url`] quando não informada.
    /// Os pontos de injeção [`Self::clock`] e [`Self::id_generator`] alimentam o
    /// `MtrCtx::with_determinism`: o relógio é a fonte de `today()` e o gerador
    /// produz o `id` do envelope [`crate::openmtr::resposta::Resposta`]. Sem injeção, usam
    /// `SystemClock` e `UuidV7Generator`.
    pub async fn build_with_transport(
        self,
        transport: impl HttpTransport + 'static,
    ) -> Result<OpenMtrClient, OpenMTRError> {
        let system = self
            .system
            .ok_or_else(|| OpenMTRError::config("system é obrigatório — use .base(System::...)"))?;

        let integration_token = self.integration_token;
        let access_token = self.access_token;
        if access_token.as_deref().is_some_and(|t| t.trim().is_empty()) {
            return Err(OpenMTRError::config("access_token vazio"));
        }
        let token = integration_token
            .or_else(|| access_token.clone())
            .ok_or_else(|| {
                OpenMTRError::config(
                    "token ou access_token é obrigatório — use .token(...) ou .access_token(...)",
                )
            })?;

        let base_url = self
            .base_url
            .unwrap_or_else(|| system.default_base_url().to_string());

        let config = Config::builder()
            .system(system)
            .base_url(base_url)
            .token(token)
            .timeout_secs(self.timeout.map_or(60, |d| d.as_secs()))
            .max_rps(self.max_rps.unwrap_or(10))
            .build()?;

        let http = MtrHttpClient::new(config.clone(), transport);

        let adapter = system.adapter()?;

        let clock = self
            .clock
            .unwrap_or_else(|| Arc::new(crate::openmtr::time_util::SystemClock));
        let id_generator = self
            .id_generator
            .unwrap_or_else(|| Arc::new(crate::openmtr::time_util::UuidV7Generator));
        let ctx = MtrCtx::with_determinism(http, config, clock, id_generator);

        if let Some(access) = access_token {
            ctx.http.set_token_direct(&access);
        } else {
            adapter.autenticar_token(&ctx).await.map_err(|mut error| {
                error.instance = ctx.new_id();
                error
            })?;
        }

        info!(
            system = %adapter.nome(),
            "openMTR client initialized"
        );

        Ok(OpenMtrClient {
            ctx: Arc::new(ctx),
            adapter,
        })
    }
}
