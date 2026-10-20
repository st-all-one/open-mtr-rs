use async_trait::async_trait;
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use open_mtr_core::openmtr::error::OpenMTRError;
use open_mtr_core::openmtr::transport::{HttpRequest, HttpResponse, HttpTransport};

pub(crate) struct JsFetchTransport;

// SAFETY: no ambiente WASM o transporte vive no único thread do runtime JS.
// Os métodos não compartilham estado mutável entre threads.
unsafe impl Send for JsFetchTransport {}
// SAFETY: idem; `JsFetchTransport` é zero-sized e imutável.
unsafe impl Sync for JsFetchTransport {}

impl JsFetchTransport {
    pub(crate) fn new() -> Self {
        Self
    }
}

async fn do_fetch(req: HttpRequest) -> Result<HttpResponse, OpenMTRError> {
    let global = js_sys::global();

    let fetch_val = js_sys::Reflect::get(&global, &"fetch".into())
        .map_err(|_| OpenMTRError::network("globalThis.fetch not available"))?;
    let fetch_fn = fetch_val
        .dyn_into::<js_sys::Function>()
        .map_err(|_| OpenMTRError::network("fetch is not a function"))?;

    let init = js_sys::Object::new();
    js_sys::Reflect::set(&init, &"method".into(), &req.method.clone().into()).ok();

    let headers = js_sys::Object::new();
    for (k, v) in &req.headers {
        js_sys::Reflect::set(&headers, &k.clone().into(), &v.clone().into()).ok();
    }
    js_sys::Reflect::set(&init, &"headers".into(), &headers).ok();

    if let Some(body_bytes) = &req.body {
        let array = Uint8Array::from(body_bytes.as_slice());
        js_sys::Reflect::set(&init, &"body".into(), &array).ok();
    }

    let url: JsValue = req.url.into();
    let init_js: JsValue = init.into();

    let promise_val = fetch_fn
        .call2(&JsValue::UNDEFINED, &url, &init_js)
        .map_err(|e| OpenMTRError::network(format!("fetch call: {e:?}")))?;

    let promise: js_sys::Promise = promise_val
        .dyn_into()
        .map_err(|_| OpenMTRError::network("fetch result not a Promise"))?;

    let resp = JsFuture::from(promise)
        .await
        .map_err(|e| OpenMTRError::network(format!("fetch: {e:?}")))?;

    let status = js_sys::Reflect::get(&resp, &"status".into())
        .ok()
        .and_then(|v| v.as_f64())
        .and_then(|v| v.round().to_string().parse::<u16>().ok())
        .unwrap_or(0);

    let headers_val = js_sys::Reflect::get(&resp, &"headers".into()).ok();

    let content_type = headers_val
        .as_ref()
        .and_then(|h| extract_header(h, "content-type"));
    let content_length = headers_val
        .as_ref()
        .and_then(|h| extract_header(h, "content-length"))
        .and_then(|s| s.parse::<u64>().ok());

    let body = read_response_body(&resp).await.unwrap_or_default();

    Ok(HttpResponse {
        status,
        body,
        content_type,
        content_length,
    })
}

fn extract_header(headers_val: &JsValue, name: &str) -> Option<String> {
    let get_fn = js_sys::Reflect::get(headers_val, &"get".into()).ok()?;
    let get = get_fn.dyn_into::<js_sys::Function>().ok()?;
    let result = get.call1(headers_val, &name.into()).ok()?;
    if result.is_null() || result.is_undefined() {
        None
    } else {
        result.as_string()
    }
}

async fn read_response_body(resp: &JsValue) -> Option<Vec<u8>> {
    let array_buf_fn = js_sys::Reflect::get(resp, &"arrayBuffer".into()).ok()?;
    let call_fn = array_buf_fn.dyn_into::<js_sys::Function>().ok()?;
    let promise_val = call_fn.call0(resp).ok()?;
    let promise: js_sys::Promise = promise_val.dyn_into().ok()?;
    let buf = JsFuture::from(promise).await.ok()?;
    Some(Uint8Array::new(&buf).to_vec())
}

#[async_trait]
impl HttpTransport for JsFetchTransport {
    async fn send(&self, req: HttpRequest) -> Result<HttpResponse, OpenMTRError> {
        let (tx, rx) = futures::channel::oneshot::channel();

        wasm_bindgen_futures::spawn_local(async move {
            let result = do_fetch(req).await;
            drop(tx.send(result));
        });

        rx.await
            .map_err(|_| OpenMTRError::network("fetch task cancelled"))?
    }
}
