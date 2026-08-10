#![allow(clippy::missing_errors_doc, clippy::return_self_not_must_use)]

use std::time::Duration;

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::openmtr::error::OpenMTRError;
use crate::openmtr::system::System;

/// Parâmetros de credenciais para autenticação clássica (CPF + senha).
///
/// Cada sistema/unidade que exige esse fluxo define a variante correspondente.
/// A variante `CpfSenhaUnidade` cobre os sistemas que usam identificador
/// (CPF/CNPJ) + senha + código de unidade — ex.: MTR-PE e SIGOR.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
#[non_exhaustive]
pub enum AuthParams {
    CpfSenhaUnidade {
        cpf: String,
        senha: String,
        unidade: u32,
        /// CNPJ da empresa. Obrigatório no MTR-PE; ignorado em sistemas que
        /// não o exigem (ex.: SIGOR).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cnpj: Option<String>,
    },
}

impl AuthParams {
    #[must_use]
    pub fn cpf_senha_unidade(
        cpf: impl Into<String>,
        senha: impl Into<String>,
        unidade: u32,
    ) -> Self {
        Self::CpfSenhaUnidade {
            cpf: cpf.into(),
            senha: senha.into(),
            unidade,
            cnpj: None,
        }
    }
}

/// Autenticação clássica (CPF + senha) isolada do cliente.
///
/// Sistemas modernos (SINIR) usam `integration_token` direto no
/// [`OpenMtrClient`](crate::openmtr::open_mtr_client::OpenMtrClient) e não
/// precisam deste facade. Já os sistemas que exigem credenciais (MTR-PE, SIGOR)
/// usam `OpenMtrClassicAuth` para consumir as credenciais e gerar o token JWT.
///
/// Retorna **sempre o token** (`String`), pronto para ser passado ao cliente:
///
/// ```rust,ignore
/// let token = OpenMtrClassicAuth::new(System::MtrPe)
///     .params(AuthParams::CpfSenhaUnidade { cpf, senha, unidade, cnpj: Some(cnpj) })
///     .build()
///     .await?;
///
/// let mtr = OpenMtrClient::builder()
///     .base(System::MtrPe)
///     .token(&token)
///     .build()
///     .await?;
/// ```
#[derive(Default)]
pub struct OpenMtrClassicAuth {
    system: Option<System>,
    base_url: Option<String>,
    params: Option<AuthParams>,
    timeout: Option<Duration>,
}

impl OpenMtrClassicAuth {
    #[must_use]
    pub fn new(system: System) -> Self {
        Self {
            system: Some(system),
            ..Self::default()
        }
    }

    /// Atalho funcional: obtém o token de um sistema com as credenciais dadas.
    ///
    /// ```rust,ignore
    /// let token = OpenMtrClassicAuth::obter_token(
    ///     System::MtrPe,
    ///     &AuthParams::CpfSenhaUnidade { cpf, senha, unidade, cnpj },
    /// ).await?;
    /// ```
    #[cfg(feature = "native-http")]
    pub async fn obter_token(
        system: System,
        params: &AuthParams,
    ) -> Result<String, OpenMTRError> {
        Self::new(system).params(params.clone()).build().await
    }

    #[must_use]
    pub fn base(mut self, system: System) -> Self {
        self.system = Some(system);
        self
    }

    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    #[must_use]
    pub fn params(mut self, params: AuthParams) -> Self {
        self.params = Some(params);
        self
    }

    #[must_use]
    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout = Some(Duration::from_secs(secs));
        self
    }

    /// Executa a autenticação e retorna o token JWT.
    ///
    /// Para sistemas modernos (SINIR), retorna erro "não suportado".
    #[cfg(feature = "native-http")]
    pub async fn build(self) -> Result<String, OpenMTRError> {
        let transport = crate::openmtr::transport::reqwest::ReqwestTransport::new(
            self.timeout.unwrap_or_else(|| Duration::from_mins(1)),
        )?;
        self.build_with_transport(transport).await
    }

    /// Idem [`Self::build`], porém com transporte HTTP plugável.
    ///
    /// Permite usar o mesmo fluxo de credenciais em qualquer runtime — inclusive
    /// no WASM (via `JsFetchTransport`) para servidores Deno/SSR.
    pub async fn build_with_transport(
        self,
        transport: impl crate::openmtr::transport::HttpTransport + 'static,
    ) -> Result<String, OpenMTRError> {
        let system = self.system.ok_or_else(|| {
            OpenMTRError::config("system é obrigatório — use .base(System::...)")
        })?;

        let params = self.params.ok_or_else(|| {
            OpenMTRError::config("params são obrigatórios — use .params(AuthParams::...)")
        })?;

        let base_url = self
            .base_url
            .unwrap_or_else(|| system.default_base_url().to_string());

        info!(
            system = %system.nome(),
            "Autenticando com credenciais clássicas (CPF + senha)"
        );

        let adapter = system.adapter()?;
        let config = crate::openmtr::config::Config::builder()
            .system(system)
            .base_url(base_url)
            .timeout_secs(self.timeout.map_or(60, |d| d.as_secs()))
            .max_rps(10)
            .build()?;

        let http = crate::openmtr::client::http::MtrHttpClient::new(config.clone(), transport)?;

        let ctx = crate::openmtr::adapters::MtrCtx { http, config };

        adapter.autenticar_credenciais(&ctx, &params).await
    }
}

/// Alias de compatibilidade (nome anterior).
pub type OpenMtrAuthBuilder = OpenMtrClassicAuth;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_params_cpf_senha_unidade() {
        let params = AuthParams::cpf_senha_unidade("07800211428", "secret", 3615);
        #[allow(unreachable_patterns)] // defensivo: enum #[non_exhaustive]
        match params {
            AuthParams::CpfSenhaUnidade { cpf, senha, unidade, cnpj } => {
                assert_eq!(cpf, "07800211428");
                assert_eq!(senha, "secret");
                assert_eq!(unidade, 3615);
                assert!(cnpj.is_none());
            }
            _ => panic!("variante inesperada"),
        }
    }

    #[test]
    fn test_auth_params_serde_roundtrip() {
        let params = AuthParams::CpfSenhaUnidade {
            cpf: "07800211428".into(),
            senha: "secret".into(),
            unidade: 3615,
            cnpj: Some("11201870000226".into()),
        };
        let json = serde_json::to_string(&params).expect("serialize");
        assert!(json.contains(r#""tipo":"cpf_senha_unidade""#));
        let back: AuthParams = serde_json::from_str(&json).expect("deserialize");
        #[allow(unreachable_patterns)] // defensivo: enum #[non_exhaustive]
        match back {
            AuthParams::CpfSenhaUnidade { cnpj, unidade, .. } => {
                assert_eq!(cnpj.as_deref(), Some("11201870000226"));
                assert_eq!(unidade, 3615);
            }
            _ => panic!("variante inesperada"),
        }
    }

    #[test]
    fn test_builder_default_sem_system() {
        let auth = OpenMtrClassicAuth::default();
        assert!(auth.system.is_none());
        assert!(auth.params.is_none());
    }

    #[test]
    fn test_builder_new_system() {
        let auth = OpenMtrClassicAuth::new(System::MtrPe);
        assert!(matches!(auth.system, Some(System::MtrPe)));
    }

    #[test]
    fn test_alias_compat() {
        let auth = OpenMtrAuthBuilder::new(System::MtrPe);
        assert!(matches!(auth.system, Some(System::MtrPe)));
    }
}
