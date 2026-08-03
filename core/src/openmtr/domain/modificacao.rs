#![allow(clippy::module_name_repetitions)]

use serde::{Deserialize, Serialize};

use super::parte::Parte;
use super::residuo::ResiduoRecebido;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ModificacaoTipo {
    Recebimento,
    Titularidade,
    Residuos,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModificarMtr {
    pub numero: String,
    pub tipo: ModificacaoTipo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub justificativa: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub novos_residuos: Option<Vec<ResiduoRecebido>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub novo_titular: Option<Parte>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtrModificado {
    pub numero: String,
    pub modificado: bool,
    pub mensagem: String,
}
