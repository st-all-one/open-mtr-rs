use super::OpenMTRError;

#[cfg(feature = "native-http")]
impl From<reqwest::Error> for OpenMTRError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::timeout(err.to_string())
        } else {
            Self::network(err.to_string())
        }
    }
}

impl From<serde_json::Error> for OpenMTRError {
    fn from(err: serde_json::Error) -> Self {
        Self::validation(err.to_string())
    }
}
