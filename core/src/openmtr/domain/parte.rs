use serde::{Deserialize, Serialize};

use super::newtypes::CpfCnpj;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parte {
    pub documento: CpfCnpj,
    pub razao_social: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endereco: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub municipio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uf: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unidade: Option<u32>,
}
