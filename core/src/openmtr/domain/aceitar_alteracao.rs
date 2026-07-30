#![allow(clippy::module_name_repetitions)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AceitarAlteracao {
    pub numero: String,
    pub aceitar: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codigo_gerador: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlteracaoAceita {
    pub numero: String,
    pub aceita: bool,
    pub mensagem: String,
}
