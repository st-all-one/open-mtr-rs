#![allow(clippy::module_name_repetitions)]

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::mtr::Periodo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GerarCdf {
    #[serde(default)]
    pub manifestos: Vec<String>,
    pub data_emissao: NaiveDate,
    pub responsavel: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observacoes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cdf {
    pub codigo: String,
    pub numero: String,
    pub data_emissao: NaiveDate,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdfEmitido {
    pub codigo: String,
    pub numero: String,
    pub data_emissao: NaiveDate,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FiltroCdf {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codigos: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub periodo: Option<Periodo>,
}
