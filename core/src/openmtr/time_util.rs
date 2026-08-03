#[cfg(target_arch = "wasm32")]
pub(crate) fn utc_now() -> chrono::DateTime<chrono::Utc> {
    let ms = js_sys::Date::now() as i64;
    chrono::DateTime::from_timestamp_millis(ms).unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn utc_now() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}
