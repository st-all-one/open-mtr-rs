use async_trait::async_trait;

use super::{HttpRequest, HttpResponse, HttpTransport};
use crate::openmtr::error::OpenMTRError;

pub struct ReqwestTransport {
    client: reqwest::Client,
}

impl ReqwestTransport {
    pub fn new(timeout: std::time::Duration) -> Result<Self, OpenMTRError> {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| OpenMTRError::network(e.to_string()))?;
        Ok(Self { client })
    }
}

#[async_trait]
impl HttpTransport for ReqwestTransport {
    async fn send(&self, req: HttpRequest) -> Result<HttpResponse, OpenMTRError> {
        let method = match req.method.as_str() {
            "GET" => reqwest::Method::GET,
            "POST" => reqwest::Method::POST,
            "PUT" => reqwest::Method::PUT,
            "DELETE" => reqwest::Method::DELETE,
            "PATCH" => reqwest::Method::PATCH,
            other => {
                return Err(OpenMTRError::network(format!(
                    "unsupported HTTP method: {other}"
                )));
            }
        };

        let mut builder = self.client.request(method, req.url);

        for (key, value) in &req.headers {
            builder = builder.header(key.as_str(), value.as_str());
        }

        if let Some(body) = req.body {
            builder = builder.body(body);
        }

        let resp = builder.send().await.map_err(|e| {
            if e.is_timeout() {
                OpenMTRError::timeout("request timed out")
            } else {
                OpenMTRError::network(e.to_string())
            }
        })?;

        let status = resp.status().as_u16();
        let content_type = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(ToString::to_string);
        let content_length = resp
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());
        let body = resp
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| OpenMTRError::network(e.to_string()))?;

        Ok(HttpResponse {
            status,
            body,
            content_type,
            content_length,
        })
    }
}
