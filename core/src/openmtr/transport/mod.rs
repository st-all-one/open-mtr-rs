#![allow(clippy::missing_errors_doc, clippy::return_self_not_must_use)]

use async_trait::async_trait;
use crate::openmtr::error::OpenMTRError;

#[cfg(feature = "native-http")]
pub mod reqwest;
#[cfg(feature = "wasm-http")]
pub mod fetch;

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}

impl HttpRequest {
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            method: "GET".into(),
            url: url.into(),
            headers: Vec::new(),
            body: None,
        }
    }

    pub fn post(url: impl Into<String>) -> Self {
        Self {
            method: "POST".into(),
            url: url.into(),
            headers: Vec::new(),
            body: None,
        }
    }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }

    pub fn json_body<T: serde::Serialize>(mut self, body: &T) -> Result<Self, OpenMTRError> {
        self.body = Some(serde_json::to_vec(body).map_err(|e| OpenMTRError::parse_error(e.to_string()))?);
        Ok(self)
    }
}

#[derive(Debug)]
pub struct HttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
    pub content_type: Option<String>,
    pub content_length: Option<u64>,
}

impl HttpResponse {
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, OpenMTRError> {
        serde_json::from_slice(&self.body).map_err(|e| OpenMTRError::parse_error(e.to_string()))
    }

    #[must_use]
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    #[must_use]
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    #[must_use]
    pub fn bytes(&self) -> Vec<u8> {
        self.body.clone()
    }
}

#[async_trait]
pub trait HttpTransport: Send + Sync {
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
            Self { responses: Mutex::new(Vec::new()) }
        }

        fn push(&self, status: u16, body: serde_json::Value) {
            self.responses.lock().unwrap().push(Ok(MockResponse { status, body }));
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
                        content_length: Some(len as u64),
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

        let resp = mock.send(HttpRequest::get("https://test/health")).await.unwrap();
        assert_eq!(resp.status, 200);
        assert!(resp.is_success());
    }

    #[tokio::test]
    async fn mock_transport_error() {
        let mock = MockTransport::new();
        mock.push_error(OpenMTRError::timeout("timeout"));

        let err = mock.send(HttpRequest::get("https://test/api")).await.unwrap_err();
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
            .json_body(&serde_json::json!({"key": "value"})).unwrap();
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
        assert!(HttpResponse { status: 200, body: vec![], content_type: None, content_length: None }.is_success());
        assert!(HttpResponse { status: 299, body: vec![], content_type: None, content_length: None }.is_success());
        assert!(!HttpResponse { status: 400, body: vec![], content_type: None, content_length: None }.is_success());
        assert!(!HttpResponse { status: 500, body: vec![], content_type: None, content_length: None }.is_success());
    }
}
