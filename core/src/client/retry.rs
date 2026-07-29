use crate::core::errors::MtrError;
use std::time::Duration;
use tokio::time::sleep;

pub async fn with_retry<T, F, Fut>(f: F) -> Result<T, MtrError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, MtrError>>,
{
    let max_retries = 3;
    let mut last_err = None;

    for attempt in 0..=max_retries {
        match f().await {
            Ok(val) => return Ok(val),
            Err(e) => {
                if !e.is_retryable() || attempt == max_retries {
                    return Err(e);
                }
                last_err = Some(e);
                let delay = Duration::from_millis(1000 * (1 << attempt));
                sleep(delay).await;
            }
        }
    }

    Err(last_err.unwrap_or_else(|| MtrError::config("Retry exhausted")))
}
