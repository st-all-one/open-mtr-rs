use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Data civil do MTR (sem fuso horário), serializada como string `dd-MM-yyyy`.
///
/// Formato canônico do wrapper: `NaiveDate` interno, `15-03-2025` em JSON e em `Display`.
/// [`Self::parse`] aceita somente `dd-MM-yyyy`; datas ISO 8601 e timestamps em ms são
/// aceitos apenas como entrada de [`crate::openmtr::domain::mtr::Periodo`].
#[derive(Clone, Debug)]
pub struct DataMtr(pub NaiveDate);

impl DataMtr {
    /// Envolve uma data civil já construída, sem conversão de fuso.
    #[must_use]
    pub fn new(d: NaiveDate) -> Self {
        Self(d)
    }

    /// Interpreta uma data no formato canônico `dd-MM-yyyy`.
    ///
    /// Falha para ISO 8601 (`2025-03-15`) e para outros separadores; o erro descreve o
    /// formato esperado.
    pub fn parse(s: &str) -> Result<Self, String> {
        NaiveDate::parse_from_str(s, "%d-%m-%Y")
            .map(Self)
            .map_err(|e| format!("data inválida (use dd-MM-yyyy): {e}"))
    }
}

/// Escreve a data civil no formato canônico `dd-MM-yyyy`.
impl fmt::Display for DataMtr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.format("%d-%m-%Y"))
    }
}

/// Serializa a data civil como string `dd-MM-yyyy` (não ISO 8601).
impl Serialize for DataMtr {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

/// Desserializa uma data do formato `dd-MM-yyyy`, rejeitando outros formatos.
impl<'de> Deserialize<'de> for DataMtr {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::parse(&s).map_err(serde::de::Error::custom)
    }
}

/// Instante em milissegundos desde o Unix epoch (UTC), serializado como número `i64`.
///
/// Usado em campos de auditoria/resposta; o inteiro é o valor de tempo, sem fuso anexo.
#[derive(Clone, Debug)]
pub struct TimestampMs(pub i64);

impl TimestampMs {
    /// Captura o instante atual do relógio UTC shared (fonte injetável em testes).
    #[must_use]
    pub fn now() -> Self {
        Self(crate::openmtr::time_util::utc_now().timestamp_millis())
    }

    /// Converte uma data-hora ingênua interpretada como UTC em milissegundos de epoch.
    #[must_use]
    pub fn from_datetime(dt: NaiveDateTime) -> Self {
        Self(dt.and_utc().timestamp_millis())
    }
}

/// Serializa o instante como número inteiro de milissegundos desde o epoch.
impl Serialize for TimestampMs {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

/// Desserializa o instante de um número inteiro de milissegundos desde o epoch.
impl<'de> Deserialize<'de> for TimestampMs {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        i64::deserialize(deserializer).map(Self)
    }
}
