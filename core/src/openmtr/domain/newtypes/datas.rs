use jiff::civil::{Date, DateTime};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::openmtr::time_util::{Clock as _, SystemClock};

/// Data civil do MTR (sem fuso horário), serializada como string `dd-MM-yyyy`.
///
/// Formato canônico do wrapper: `jiff::civil::Date` interno, `15-03-2025` em JSON e em
/// `Display`. [`Self::parse`] aceita somente `dd-MM-yyyy`; datas ISO 8601 e timestamps em
/// ms são aceitos apenas como entrada de [`crate::openmtr::domain::mtr::Periodo`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DataMtr(pub Date);

impl DataMtr {
    /// Envolve uma data civil já construída, sem conversão de fuso.
    #[must_use]
    pub fn new(d: Date) -> Self {
        Self(d)
    }

    /// Interpreta uma data no formato canônico `dd-MM-yyyy`.
    ///
    /// Falha para ISO 8601 (`2025-03-15`) e para outros separadores; o erro descreve o
    /// formato esperado.
    pub fn parse(s: &str) -> Result<Self, String> {
        // `civil::Date` só faz parsing ISO por padrão; o formato brasileiro exige `strtime`.
        let parsed = jiff::fmt::strtime::parse("%d-%m-%Y", s)
            .map_err(|e| format!("data inválida (use dd-MM-yyyy): {e}"))?;
        let date = parsed
            .to_date()
            .map_err(|e| format!("data inválida (use dd-MM-yyyy): {e}"))?;
        Ok(Self(date))
    }
}

/// Escreve a data civil no formato canônico `dd-MM-yyyy`.
impl fmt::Display for DataMtr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format_dd_mm_yyyy(self.0))
    }
}

/// Formata uma data civil como `dd-MM-yyyy` (formato do contrato de bytes).
fn format_dd_mm_yyyy(d: Date) -> String {
    jiff::fmt::strtime::format("%d-%m-%Y", d).unwrap_or_else(|_| d.to_string())
}

/// Serializa a data civil como string `dd-MM-yyyy` (não ISO 8601).
impl Serialize for DataMtr {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format_dd_mm_yyyy(self.0))
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimestampMs(pub i64);

impl TimestampMs {
    /// Captura o instante atual do relógio UTC (fonte injetável em testes).
    #[must_use]
    pub fn now() -> Self {
        Self::from_timestamp(&SystemClock.now())
    }

    /// Converte um instante UTC em milissegundos de epoch.
    #[must_use]
    pub fn from_timestamp(ts: &jiff::Timestamp) -> Self {
        Self(ts.as_millisecond())
    }

    /// Converte uma data-hora civil interpretada como UTC em milissegundos de epoch.
    #[must_use]
    pub fn from_datetime(dt: DateTime) -> Self {
        match dt.to_zoned_datetime(jiff::tz::TimeZone::UTC) {
            Ok(zdt) => Self::from_timestamp(&zdt.timestamp()),
            Err(_) => Self(0),
        }
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
