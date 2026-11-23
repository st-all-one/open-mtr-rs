use super::*;
use crate::openmtr::error::OpenMTRError;
use crate::openmtr::transport::{HttpRequest, HttpResponse, HttpTransport};
use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::execute::map_http_error;

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
