use jiff::{Timestamp, civil::Date};
use serde::{Deserialize, Serialize};

use crate::openmtr::domain::newtypes::DataMtr;

/// Intervalo inclusivo de datas usado ao filtrar manifestos.
///
/// Serializado como `{"dataInicial":"dd-MM-yyyy","dataFinal":"dd-MM-yyyy"}`; a leitura também
/// aceita ISO 8601 e epoch ms. O atalho `{"dias": N}` é resolvido no instante da desserialização
/// pelo relógio do sistema, por [`Self::ultimos_dias`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Periodo {
    /// Data civil inicial do intervalo, serializada como `dd-MM-yyyy`.
    pub inicio: DataMtr,
    /// Data civil final do intervalo, inclusiva, serializada como `dd-MM-yyyy`.
    pub fim: DataMtr,
}

impl Periodo {
    /// Constrói o intervalo, exigindo que `fim` não seja anterior a `inicio`.
    ///
    /// Datas invertidas geram um erro de validação em vez de um intervalo vazio silencioso.
    pub fn new(inicio: DataMtr, fim: DataMtr) -> Result<Self, crate::openmtr::error::OpenMTRError> {
        if fim.0 < inicio.0 {
            return Err(crate::openmtr::error::OpenMTRError::validation(
                "data final deve ser maior ou igual à data inicial",
            ));
        }
        Ok(Self { inicio, fim })
    }
    /// Atalho relativo ao relógio do sistema, com `inicio = hoje - dias` e `fim = hoje`.
    ///
    /// `hoje` é lido de `utc_now()` (data UTC) no momento da chamada; é isso que resolve o atalho
    /// `{"dias": N}` ao desserializar [`Periodo`] ou [`super::FiltroMtr`]. Como esse relógio não é
    /// injetável aqui, golden tests e reprocessamentos devem usar [`Self::ultimos_dias_em`] com
    /// uma data obtida de [`crate::openmtr::time_util::Clock`], ou um intervalo via [`Self::new`].
    #[must_use]
    pub fn ultimos_dias(dias: i64) -> Self {
        Self::ultimos_dias_em(
            crate::openmtr::time_util::utc_now()
                .to_zoned_datetime(jiff::tz::TimeZone::UTC)
                .date(),
            dias,
        )
    }
    /// Últimos `dias` até `hoje`, sem consultar o relógio global.
    ///
    /// `inicio = hoje - dias` e `fim = hoje`; `hoje` normalmente vem de
    /// [`crate::openmtr::time_util::Clock::today`], o que mantém determinísticos snapshots e
    /// reprocessamentos.
    #[must_use]
    pub fn ultimos_dias_em(hoje: Date, dias: i64) -> Self {
        let inicio = hoje.checked_sub(jiff::ToSpan::days(dias)).unwrap_or(hoje);
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

/// Converte um valor de data recebido nos bindings em `Date` (calendário UTC).
///
/// Aceita (como o JS costuma enviar):
/// - String `dd-MM-yyyy` (formato canônico do wrapper);
/// - String ISO 8601 — ex.: o que `JSON.stringify(new Date())` produz,
///   `"2026-06-01T00:00:00.000Z"`, ou `"2026-06-01"`;
/// - Número (epoch ms desde Unix epoch, interpretado como UTC e reduzido à data civil).
fn data_flexivel(v: &serde_json::Value) -> Option<Date> {
    match v {
        serde_json::Value::String(s) => {
            if let Ok(d) = DataMtr::parse(s) {
                return Some(d.0);
            }
            if let Ok(d) = s.parse::<Date>() {
                return Some(d);
            }
            // RFC 3339 com offset/Z: interpreta como instante e reduz ao calendário UTC.
            if let Ok(ts) = s.parse::<Timestamp>() {
                return Some(ts.to_zoned_datetime(jiff::tz::TimeZone::UTC).date());
            }
            None
        }
        serde_json::Value::Number(n) => n
            .as_i64()
            .and_then(|ms| Timestamp::from_millisecond(ms).ok())
            .map(|ts| ts.to_zoned_datetime(jiff::tz::TimeZone::UTC).date()),
        _ => None,
    }
}

impl<'de> Deserialize<'de> for Periodo {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum PeriodoRaw {
            /// Últimos N dias até hoje (atalho usado nos bindings).
            ///
            /// A resolução usa o relógio do sistema no momento da
            /// desserialização. Para saída determinística, envie `dataInicial`
            /// e `dataFinal` explicitamente ou use `ultimos_dias_em`.
            Dias { dias: i64 },
            /// Intervalo explícito — datas em `dd-MM-yyyy`, ISO 8601
            /// (`new Date()` no JS) ou timestamp em ms.
            Intervalo {
                #[serde(rename = "dataInicial")]
                data_inicial: serde_json::Value,
                #[serde(rename = "dataFinal")]
                data_final: serde_json::Value,
            },
        }
        match PeriodoRaw::deserialize(d)? {
            PeriodoRaw::Dias { dias } => Ok(Self::ultimos_dias(dias)),
            PeriodoRaw::Intervalo {
                data_inicial,
                data_final,
            } => {
                let inicio = DataMtr(data_flexivel(&data_inicial).ok_or_else(|| {
                    serde::de::Error::custom(format!(
                        "dataInicial inválida: {data_inicial} (use dd-MM-yyyy, ISO 8601 ou timestamp)"
                    ))
                })?);
                let fim = DataMtr(data_flexivel(&data_final).ok_or_else(|| {
                    serde::de::Error::custom(format!(
                        "dataFinal inválida: {data_final} (use dd-MM-yyyy, ISO 8601 ou timestamp)"
                    ))
                })?);
                Self::new(inicio, fim).map_err(serde::de::Error::custom)
            }
        }
    }
}
