use super::*;

#[test]
fn test_new_error() {
    let err = OpenMTRError::new(400, "Bad Request", "detail");
    assert_eq!(err.status, 400);
    assert_eq!(err.title, "Bad Request");
}

#[test]
fn test_network_error() {
    let err = OpenMTRError::network("connection refused");
    assert_eq!(err.status, 503);
    assert_eq!(err.kind, ErrorKind::Network);
}

#[test]
fn test_timeout_error() {
    let err = OpenMTRError::timeout("crawled");
    assert_eq!(err.status, 504);
    assert_eq!(err.kind, ErrorKind::Timeout);
    assert!(err.is_timeout());
}

#[test]
fn test_is_timeout_is_not_substring_based() {
    let err = OpenMTRError::network("upstream timeout announced");
    assert!(!err.is_timeout());
}

#[test]
fn test_with_instance_is_deterministic() {
    let instance = Uuid::from_u128(42);
    let err = OpenMTRError::validation("x").with_instance(instance);
    assert_eq!(err.instance, instance);
}

#[test]
fn test_auth_error() {
    let err = OpenMTRError::auth("invalid token");
    assert_eq!(err.status, 401);
}

#[test]
fn test_config_error() {
    let err = OpenMTRError::config("missing base_url");
    assert_eq!(err.status, 500);
}

#[test]
fn test_validation_error() {
    let err = OpenMTRError::validation("invalid cpf");
    assert_eq!(err.status, 422);
}

#[test]
fn test_not_supported() {
    let err = OpenMTRError::not_supported("export");
    assert_eq!(err.status, 501);
    assert!(err.detail.contains("export"));
}

#[test]
fn test_error_types_are_valid_urls() {
    assert!(
        error_types::network()
            .as_str()
            .starts_with("urn:openmtr:error:")
    );
    assert!(
        error_types::timeout()
            .as_str()
            .starts_with("urn:openmtr:error:")
    );
    assert!(
        error_types::auth()
            .as_str()
            .starts_with("urn:openmtr:error:")
    );
    assert!(
        error_types::validation()
            .as_str()
            .starts_with("urn:openmtr:error:")
    );
    assert!(
        error_types::config()
            .as_str()
            .starts_with("urn:openmtr:error:")
    );
    assert!(
        error_types::not_found()
            .as_str()
            .starts_with("urn:openmtr:error:")
    );
    assert!(
        error_types::api()
            .as_str()
            .starts_with("urn:openmtr:error:")
    );
    assert!(
        error_types::not_supported()
            .as_str()
            .starts_with("urn:openmtr:error:")
    );
    assert!(
        error_types::internal()
            .as_str()
            .starts_with("urn:openmtr:error:")
    );
}

#[test]
fn test_display() {
    let err = OpenMTRError::new(400, "x", "y");
    let s = err.to_string();
    assert!(s.contains("400"));
    assert!(s.contains('x'));
    assert!(s.contains('y'));
}

#[cfg(feature = "native-http")]
#[tokio::test]
async fn test_from_reqwest_error() {
    let body = reqwest::Client::new()
        .get("http://127.0.0.1:1")
        .send()
        .await;
    if let Err(e) = body {
        let err: OpenMTRError = e.into();
        assert!(err.status == 503 || err.status == 504);
    }
}
