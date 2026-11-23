//! Abstração de transporte HTTP usada pelo cliente do openMTR.
//!
//! O traço [`HttpTransport`](crate::openmtr::transport::HttpTransport) é o
//! ponto de extensão/teste: implementações concretas (como
//! `reqwest::ReqwestTransport` com a feature `native-http`, ou um transporte
//! WASM) podem ser injetadas via `build_with_transport`. Os tipos
//! [`HttpRequest`](crate::openmtr::transport::HttpRequest) e
//! [`HttpResponse`](crate::openmtr::transport::HttpResponse) são a troca de
//! dados independente de runtime.
use crate::openmtr::error::OpenMTRError;
use async_trait::async_trait;

#[cfg(feature = "native-http")]
/// Transporte HTTP baseado em `reqwest`, disponível com `native-http`.
pub mod reqwest;

#[derive(Debug, Clone)]
/// Requisição HTTP abstrata passada a um [`HttpTransport`].
pub struct HttpRequest {
    /// Método HTTP em maiúsculas (`GET`, `POST`, ...).
    pub method: String,
    /// URL absoluta de destino.
    pub url: String,
    /// Cabeçalhos como pares `(nome, valor)`.
    pub headers: Vec<(String, String)>,
    /// Corpo serializado, ou `None` para requisições sem corpo.
    pub body: Option<Vec<u8>>,
}

impl HttpRequest {
    /// Cria uma requisição `GET` sem cabeçalhos nem corpo.
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            method: "GET".into(),
            url: url.into(),
            headers: Vec::new(),
            body: None,
        }
    }

    /// Cria uma requisição `POST` sem cabeçalhos nem corpo.
    pub fn post(url: impl Into<String>) -> Self {
        Self {
            method: "POST".into(),
            url: url.into(),
            headers: Vec::new(),
            body: None,
        }
    }

    /// Acrescenta um cabeçalho à requisição.
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }

    /// Serializa `body` como JSON no corpo da requisição.
    ///
    /// Falha com [`OpenMTRError::parse_error`] se a serialização falhar.
    pub fn json_body<T: serde::Serialize>(mut self, body: &T) -> Result<Self, OpenMTRError> {
        self.body =
            Some(serde_json::to_vec(body).map_err(|e| OpenMTRError::parse_error(e.to_string()))?);
        Ok(self)
    }
}

/// Resposta HTTP abstrata devolvida por um [`HttpTransport`].
#[derive(Debug)]
pub struct HttpResponse {
    /// Código de status HTTP numérico.
    pub status: u16,
    /// Corpo bruto da resposta.
    pub body: Vec<u8>,
    /// Valor do cabeçalho `Content-Type`, quando informado.
    pub content_type: Option<String>,
    /// Valor do cabeçalho `Content-Length` em bytes, quando informado.
    pub content_length: Option<u64>,
}

impl HttpResponse {
    /// Desserializa o corpo como JSON.
    ///
    /// Falha com [`OpenMTRError::parse_error`] se o corpo não for JSON válido
    /// para `T`, independentemente do status HTTP.
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, OpenMTRError> {
        serde_json::from_slice(&self.body).map_err(|e| OpenMTRError::parse_error(e.to_string()))
    }

    /// Devolve o corpo como texto, substituindo bytes inválidos por U+FFFD.
    #[must_use]
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    /// Indica sucesso quando o status está entre 200 e 299.
    #[must_use]
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Devolve uma cópia do corpo em bytes.
    #[must_use]
    pub fn bytes(&self) -> Vec<u8> {
        self.body.clone()
    }
}

/// Transporte HTTP assíncrono, ponto de extensão e teste do cliente.
///
/// Exige `Send + Sync` para uso concorrente. Implementações próprias permitem
/// executar o mesmo fluxo em qualquer runtime (nativo via reqwest, WASM via
/// fetch) e são injetadas nos builders por `build_with_transport`.
#[async_trait]
pub trait HttpTransport: Send + Sync {
    /// Envia uma requisição e devolve a resposta, ou um erro de transporte.
    async fn send(&self, req: HttpRequest) -> Result<HttpResponse, OpenMTRError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockTransport {
        responses: Mutex<Vec<Result<MockResponse, OpenMTRError>>>,
    }

    struct MockResponse {
        status: u16,
        body: serde_json::Value,
    }

    impl MockTransport {
        fn new() -> Self {
            Self {
                responses: Mutex::new(Vec::new()),
            }
        }

        fn push(&self, status: u16, body: serde_json::Value) {
            self.responses
                .lock()
                .unwrap()
                .push(Ok(MockResponse { status, body }));
        }

        fn push_error(&self, err: OpenMTRError) {
            self.responses.lock().unwrap().push(Err(err));
        }
    }

    #[async_trait]
    impl HttpTransport for MockTransport {
        async fn send(&self, _req: HttpRequest) -> Result<HttpResponse, OpenMTRError> {
            let next = self.responses.lock().unwrap().remove(0);
            match next {
                Ok(mock) => {
                    let body = serde_json::to_vec(&mock.body).unwrap();
                    let len = body.len();
                    Ok(HttpResponse {
                        status: mock.status,
                        body,
                        content_type: Some("application/json".into()),
                        content_length: Some(u64::try_from(len).unwrap_or(u64::MAX)),
                    })
                }
                Err(e) => Err(e),
            }
        }
    }

    #[tokio::test]
    async fn mock_transport_ok() {
        let mock = MockTransport::new();
        mock.push(200, serde_json::json!({"ok": true}));

        let resp = mock
            .send(HttpRequest::get("https://test/health"))
            .await
            .unwrap();
        assert_eq!(resp.status, 200);
        assert!(resp.is_success());
    }

    #[tokio::test]
    async fn mock_transport_error() {
        let mock = MockTransport::new();
        mock.push_error(OpenMTRError::timeout("timeout"));

        let err = mock
            .send(HttpRequest::get("https://test/api"))
            .await
            .unwrap_err();
        assert!(err.is_timeout());
    }

    #[tokio::test]
    async fn http_request_builder_get() {
        let req = HttpRequest::get("https://api.example.com/v1/mtrs")
            .header("Authorization", "Bearer abc");
        assert_eq!(req.method, "GET");
        assert_eq!(req.url, "https://api.example.com/v1/mtrs");
        assert_eq!(req.headers.len(), 1);
    }

    #[tokio::test]
    async fn http_request_builder_post_json() {
        let req = HttpRequest::post("https://api.example.com/v1/mtrs")
            .json_body(&serde_json::json!({"key": "value"}))
            .unwrap();
        assert_eq!(req.method, "POST");
        assert!(req.body.is_some());
        let body: serde_json::Value = serde_json::from_slice(&req.body.unwrap()).unwrap();
        assert_eq!(body["key"], "value");
    }

    #[tokio::test]
    async fn http_response_json_parse() {
        let body = serde_json::to_vec(&serde_json::json!({"x": 1})).unwrap();
        let resp = HttpResponse {
            status: 200,
            body,
            content_type: None,
            content_length: None,
        };
        let parsed: serde_json::Value = resp.json().unwrap();
        assert_eq!(parsed["x"], 1);
    }

    #[tokio::test]
    async fn http_response_is_success() {
        assert!(
            HttpResponse {
                status: 200,
                body: vec![],
                content_type: None,
                content_length: None
            }
            .is_success()
        );
        assert!(
            HttpResponse {
                status: 299,
                body: vec![],
                content_type: None,
                content_length: None
            }
            .is_success()
        );
        assert!(
            !HttpResponse {
                status: 400,
                body: vec![],
                content_type: None,
                content_length: None
            }
            .is_success()
        );
        assert!(
            !HttpResponse {
                status: 500,
                body: vec![],
                content_type: None,
                content_length: None
            }
            .is_success()
        );
    }
}
