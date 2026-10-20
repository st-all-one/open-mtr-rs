use serde::{Deserialize, Serialize};

use super::tipo::FormatoDownload;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadItem {
    pub filename: String,
    pub buffer: Vec<u8>,
    pub formato: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadCdf {
    pub codigo: String,
    pub formato: FormatoDownload,
}
