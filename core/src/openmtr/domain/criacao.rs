#![allow(clippy::module_name_repetitions)]

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use super::parte::Parte;
use super::residuo::ResiduoCriacao;
use super::tipo::TipoManifesto;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriarMtr {
    pub gerador: Parte,
    pub transportador: Parte,
    pub destinador: Parte,
    #[serde(default)]
    pub residuos: Vec<ResiduoCriacao>,
    pub responsavel: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observacoes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_expedicao: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placa_veiculo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motorista: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub armazenador_temporario: Option<Parte>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tipo_manifesto: Option<TipoManifesto>,
    pub cargo: Option<String>,
    pub codigo_referencia: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtrCriado {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numero: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codigo_gerado: Option<i64>,
    pub sucesso: bool,
    pub mensagem: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriarMtrComplementar {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motorista: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placa_veiculo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observacoes: Option<String>,
    pub transportador: Parte,
    pub manifestos: Vec<String>,
}
