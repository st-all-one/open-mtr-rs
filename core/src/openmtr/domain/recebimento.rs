use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use super::residuo::ResiduoRecebido;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceberMtr {
    pub numero: String,
    pub data_recebimento: NaiveDateTime,
    pub responsavel: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observacoes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motorista: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placa_veiculo: Option<String>,
    #[serde(default)]
    pub residuos_recebidos: Vec<ResiduoRecebido>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtrRecebido {
    pub numero: String,
    pub recebido: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mensagem: Option<String>,
}
