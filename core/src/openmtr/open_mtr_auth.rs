#![allow(clippy::missing_errors_doc, clippy::return_self_not_must_use)]

use std::time::Duration;
use tracing::info;

use crate::openmtr::error::OpenMTRError;
use crate::openmtr::system::System;

/// Utilitário de autenticação para sistemas legado (CPF + senha).
///
/// Sistemas modernos (SINIR, etc.) usam `integration_token` direto no
/// [`OpenMtrClient`](crate::openmtr::open_mtr_client::OpenMtrClient).
///
/// `OpenMtrAuth` existe apenas para sistemas que ainda exigem
/// CPF + senha + unidade para obter o token.
#[derive(Default)]
pub struct OpenMtrAuthBuilder {
    system: Option<System>,
    base_url: Option<String>,
    cpf: Option<String>,
    senha: Option<String>,
    unidade: Option<u32>,
    timeout: Option<Duration>,
    cnpj: Option<String>,
}

impl OpenMtrAuthBuilder {
    #[must_use]
    pub fn base(mut self, system: System) -> Self {
        self.system = Some(system);
        self
    }

    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    pub fn credenciais(mut self, cpf: impl Into<String>, senha: impl Into<String>, unidade: u32) -> Self {
        self.cpf = Some(cpf.into());
        self.senha = Some(senha.into());
        self.unidade = Some(unidade);
        self
    }

    pub fn cnpj(mut self, cnpj: impl Into<String>) -> Self {
        self.cnpj = Some(cnpj.into());
        self
    }

    #[must_use]
    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout = Some(Duration::from_secs(secs));
        self
    }

    /// Executa a autenticação e retorna o token JWT.
    ///
    /// Para sistemas modernos (SINIR, etc.), retorna erro "não suportado".
    #[cfg(feature = "native-http")]
    pub async fn build(self) -> Result<String, OpenMTRError> {
        let system = self.system.ok_or_else(|| {
            OpenMTRError::config("system é obrigatório — use .base(System::...)")
        })?;

        let base_url = self
            .base_url
            .unwrap_or_else(|| system.default_base_url().to_string());

        let (Some(cpf), Some(senha), Some(unidade)) = (self.cpf, self.senha, self.unidade) else { return Err(OpenMTRError::config(
                "credenciais (CPF, senha, unidade) são obrigatórias",
            )) };

        info!(
            system = %system.nome(),
            "Autenticando com credenciais legado (CPF + senha)"
        );

        let adapter = system.adapter()?;
        let mut config_builder = crate::openmtr::config::Config::builder()
            .system(system)
            .base_url(base_url)
            .timeout_secs(self.timeout.map_or(60, |d| d.as_secs()))
            .max_rps(10);
        if let Some(ref cnpj) = self.cnpj {
            config_builder = config_builder.cnpj(cnpj);
        }
        let config = config_builder.build()?;

        let transport = crate::openmtr::transport::reqwest::ReqwestTransport::new(
            config.timeout,
        )?;
        let http = crate::openmtr::client::http::MtrHttpClient::new(config.clone(), transport)?;

        let ctx = crate::openmtr::adapters::MtrCtx {
            http,
            config,
        };

        adapter.autenticar_credenciais(&ctx, &cpf, &senha, unidade).await
    }
}
