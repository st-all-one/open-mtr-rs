#![allow(clippy::module_name_repetitions)]

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::fmt;

use super::newtypes::{CpfCnpj, DataMtr};
use super::parte::Parte;
use super::residuo::Residuo;
use super::tipo::TipoParte;

#[derive(Debug, Clone)]
pub struct Periodo {
    pub inicio: DataMtr,
    pub fim: DataMtr,
}

impl Periodo {
    pub fn new(inicio: DataMtr, fim: DataMtr) -> Result<Self, crate::openmtr::error::OpenMTRError> {
        if fim.0 < inicio.0 {
            return Err(crate::openmtr::error::OpenMTRError::validation(
                "data final deve ser maior ou igual à data inicial",
            ));
        }
        Ok(Self { inicio, fim })
    }

    pub fn ultimos_dias(dias: i64) -> Self {
        let hoje = chrono::Utc::now().date_naive();
        let inicio = hoje - chrono::Duration::days(dias);
        Self {
            inicio: DataMtr(inicio),
            fim: DataMtr(hoje),
        }
    }
}

impl Serialize for Periodo {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = s.serialize_struct("Periodo", 2)?;
        st.serialize_field("dataInicial", &self.inicio)?;
        st.serialize_field("dataFinal", &self.fim)?;
        st.end()
    }
}

impl<'de> Deserialize<'de> for Periodo {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Raw {
            #[serde(rename = "dataInicial")]
            data_inicial: DataMtr,
            #[serde(rename = "dataFinal")]
            data_final: DataMtr,
        }
        let raw = Raw::deserialize(d)?;
        Ok(Self {
            inicio: raw.data_inicial,
            fim: raw.data_final,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Mtr {
    pub numero: String,
    pub status: StatusMtr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gerador: Option<Parte>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transportador: Option<Parte>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destinador: Option<Parte>,
    #[serde(default)]
    pub residuos: Vec<Residuo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_criacao: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_recebimento: Option<NaiveDate>,

    // --- Campos adicionais (GAP 3) ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observacao: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motorista: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placa: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tipo_manifesto: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estado_uf: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_expedicao: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub armazenamento_temporario: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tem_alteracao_pendente: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numero_estadual: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codigo_estadual: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatusMtr {
    Ativo,
    EmTransporte,
    Recebido,
    Cancelado,
    Expirado,
    Outro(String),
}

impl Default for StatusMtr {
    fn default() -> Self {
        Self::Ativo
    }
}

impl fmt::Display for StatusMtr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ativo => write!(f, "Ativo"),
            Self::EmTransporte => write!(f, "Em Transporte"),
            Self::Recebido => write!(f, "Recebido"),
            Self::Cancelado => write!(f, "Cancelado"),
            Self::Expirado => write!(f, "Expirado"),
            Self::Outro(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FiltroMtr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numeros: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub periodo: Option<Periodo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parte: Option<FiltroParte>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagina: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tamanho: Option<u32>,
    /// Quando true, enriquece cada MTR com dados detalhados
    /// (resíduos, CNPJ, campos completos) via requisições paralelas
    /// a GET /retornaManifesto/{numero}.
    #[serde(default)]
    pub detalhado: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FiltroParte {
    pub tipo: TipoParte,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documento: Option<CpfCnpj>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unidade: Option<u32>,
}
