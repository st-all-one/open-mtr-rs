use crate::core::errors::{ErrorCategory, ErrorContext, MtrError};
use serde::Deserialize;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const DEFAULT_TOKEN_TTL_SECS: u64 = 28800;

#[derive(Deserialize)]
struct TokenApiResponse {
    erro: bool,
    mensagem: Option<String>,
    #[serde(rename = "objetoResposta")]
    objeto_resposta: Option<String>,
    #[serde(rename = "totalRecords")]
    total_records: Option<i32>,
}

struct CachedToken {
    access_token: String,
    expires_at: Instant,
}

pub struct TokenManager {
    integration_token: String,
    cached: Option<CachedToken>,
}

impl TokenManager {
    pub fn new(integration_token: impl Into<String>) -> Self {
        Self {
            integration_token: integration_token.into(),
            cached: None,
        }
    }

    pub fn token(&self) -> Option<&str> {
        self.cached.as_ref().and_then(|c| {
            if Instant::now() < c.expires_at {
                Some(c.access_token.as_str())
            } else {
                None
            }
        })
    }

    fn parse_token_from_response(raw: &str) -> &str {
        raw.strip_prefix("Bearer ").unwrap_or(raw)
    }

    /// Extrai `exp` claim de um JWT sem verificar assinatura
    fn extract_expiry(jwt: &str) -> Option<u64> {
        let payload_b64 = jwt.split('.').nth(1)?;
        use base64::Engine;
        let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(payload_b64)
            .ok()?;
        let json: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
        json.get("exp")?.as_u64()
    }

    pub async fn refresh(
        &mut self,
        http: &reqwest::Client,
        base_url: &str,
    ) -> Result<(), MtrError> {
        let resp = http
            .post(format!("{}/token", base_url))
            .header("Authorization", format!("Bearer {}", self.integration_token))
            .send()
            .await
            .map_err(MtrError::Http)?;

        let parsed: TokenApiResponse = resp.json().await.map_err(MtrError::Http)?;

        if parsed.erro {
            return Err(MtrError::api(
                ErrorCategory::AuthenticationFailed,
                401,
                parsed.mensagem.as_deref().unwrap_or("Erro desconhecido"),
                ErrorContext::new("token.refresh"),
            ));
        }

        let raw_token = parsed
            .objeto_resposta
            .as_deref()
            .ok_or_else(|| MtrError::api(
                ErrorCategory::ParseError,
                500,
                "Token response sem objetoResposta",
                ErrorContext::new("token.refresh"),
            ))?;

        let access_token = Self::parse_token_from_response(raw_token).to_string();

        // Tenta extrair exp do JWT; fallback para 8h
        let ttl_secs = Self::extract_expiry(&access_token)
            .map(|exp| exp.saturating_sub(std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()))
            .unwrap_or(DEFAULT_TOKEN_TTL_SECS);

        let margin = Duration::from_secs(60);
        let ttl = Duration::from_secs(ttl_secs).saturating_sub(margin);

        self.cached = Some(CachedToken {
            access_token,
            expires_at: Instant::now() + ttl,
        });

        Ok(())
    }
}
