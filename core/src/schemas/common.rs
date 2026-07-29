use crate::core::errors::MtrError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ObjetoRespostaRest<T> {
    pub erro: bool,
    #[serde(rename = "objetoResposta")]
    pub objeto_resposta: Option<T>,
    pub mensagem: Option<String>,
    #[serde(rename = "totalRecords")]
    pub total_records: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct Paginacao<T> {
    pub content: Vec<T>,
    #[serde(rename = "totalPages")]
    pub total_pages: i32,
    #[serde(rename = "totalElements")]
    pub total_elements: i64,
    #[serde(rename = "number")]
    pub page: i32,
    #[serde(rename = "size")]
    pub page_size: i32,
}

#[derive(Debug, Clone)]
pub struct DataMtr(pub chrono::NaiveDate);

impl DataMtr {
    #[must_use]
    pub fn new(d: chrono::NaiveDate) -> Self {
        Self(d)
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        chrono::NaiveDate::parse_from_str(s, "%d-%m-%Y")
            .map(Self)
            .map_err(|e| format!("Data inválida '{s}': {e}"))
    }
}

impl std::fmt::Display for DataMtr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.format("%d-%m-%Y"))
    }
}

impl Serialize for DataMtr {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(&self.0.format("%d-%m-%Y"))
    }
}

impl<'de> Deserialize<'de> for DataMtr {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        chrono::NaiveDate::parse_from_str(&s, "%d-%m-%Y")
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Pagina(u32);

impl Pagina {
    pub fn new(val: u32) -> Result<Self, MtrError> {
        if val < 1 {
            Err(MtrError::validation("pagina deve ser >= 1"))
        } else {
            Ok(Pagina(val))
        }
    }
    pub fn get(&self) -> i32 {
        self.0 as i32
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TamanhoPagina(u32);

impl TamanhoPagina {
    pub fn new(val: u32) -> Result<Self, MtrError> {
        if val < 1 || val > 50 {
            Err(MtrError::validation(
                "tamanho da pagina deve estar entre 1 e 50",
            ))
        } else {
            Ok(TamanhoPagina(val))
        }
    }
    pub fn get(&self) -> i32 {
        self.0 as i32
    }
}
