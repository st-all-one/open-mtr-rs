#![allow(clippy::module_name_repetitions)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelarMtr {
    pub numero: String,
    pub justificativa: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtrCancelado {
    pub numero: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_cancelamento: Option<String>,
}
