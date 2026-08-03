use async_trait::async_trait;
use js_sys::{Array, Uint8Array};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, Request, RequestInit, RequestMode, Response};

use super::{HttpRequest, HttpResponse, HttpTransport};
use crate::openmtr::error::OpenMTRError;

pub struct FetchTransport {
    base_timeout_ms: u64,
}

impl FetchTransport {
    pub fn new(timeout_secs: u64) -> Self {
        Self {
            base_timeout_ms: timeout_secs * 1000,
        }
    }
}

#[async_trait(?Send)]
impl HttpTransport for FetchTransport {
    async fn send(&self, req: HttpRequest) -> Result<HttpResponse, OpenMTRError> {
        let mut opts = RequestInit::new();
        opts.method(&req.method);
        opts.mode(RequestMode::Cors);

        let headers = Headers::new().map_err(|e| {
            OpenMTRError::network(format!("failed to create headers: {e:?}"))
        })?;
        for (key, value) in &req.headers {
            headers
                .set(key, value)
                .map_err(|e| OpenMTRError::network(format!("failed to set header: {e:?}")))?;
        }
        opts.headers(&headers);

        if let Some(body) = req.body {
            let array = Uint8Array::from(body.as_slice());
            opts.body(Some(&array.into()));
        }

        let request = Request::new_with_str_and_init(&req.url, &opts).map_err(|e| {
            OpenMTRError::network(format!("failed to build request: {e:?}"))
        })?;

        let window = web_sys::window()
            .ok_or_else(|| OpenMTRError::network("no window available"))?;

        let timeout_promise = wasm_bindgen_futures::js_sys::Promise::new(
            &mut |resolve, _reject| {
                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    &resolve,
                    self.base_timeout_ms as i32,
                );
            },
        );

        let fetch_promise = window.fetch_with_request(&request);

        let result = js_sys::Promise::race(&Array::of2(&fetch_promise, &timeout_promise));

        let val = JsFuture::from(result).await.map_err(|e| {
            let msg = format!("{e:?}");
            if msg.contains("timeout") || msg.contains("Timeout") {
                OpenMTRError::timeout("request timed out")
            } else {
                OpenMTRError::network(msg)
            }
        })?;

        if !val.is_instance_of::<Response>() {
            return Err(OpenMTRError::timeout("request timed out"));
        }

        let resp: Response = val.dyn_into().map_err(|_| {
            OpenMTRError::network("unexpected fetch result")
        })?;

        let status = resp.status();

        let content_type = resp.headers().get("content-type").ok().flatten();
        let content_length = resp
            .headers()
            .get("content-length")
            .ok()
            .flatten()
            .and_then(|s| s.parse::<u64>().ok());

        let array_buffer = JsFuture::from(resp.array_buffer().map_err(|e| {
            OpenMTRError::network(format!("failed to read body: {e:?}"))
        })?)
        .await
        .map_err(|e| OpenMTRError::network(format!("failed to read body: {e:?}")))?;

        let body = Uint8Array::new(&array_buffer).to_vec();

        Ok(HttpResponse {
            status,
            body,
            content_type,
            content_length,
        })
    }
}
