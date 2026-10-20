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

    /// Atalho de conveniência relativo ao relógio do sistema.
    ///
    /// Para determinismo (golden tests, reprocessamento), prefira
    /// [`Self::ultimos_dias_em`] com uma data injetada ou um intervalo
    /// explícito via [`Self::new`].
    #[must_use]
    pub fn ultimos_dias(dias: i64) -> Self {
        Self::ultimos_dias_em(crate::openmtr::time_util::utc_now().date_naive(), dias)
    }

    /// Últimos `dias` até `hoje`, sem consultar o relógio global.
    #[must_use]
    pub fn ultimos_dias_em(hoje: NaiveDate, dias: i64) -> Self {
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

/// Converte um valor de data recebido nos bindings em `NaiveDate`.
///
/// Aceita (como o JS costuma enviar):
/// - String `dd-MM-yyyy` (formato canônico do wrapper);
/// - String ISO 8601 — ex.: o que `JSON.stringify(new Date())` produz,
///   `"2026-06-01T00:00:00.000Z"`, ou `"2026-06-01"`;
/// - Número (timestamp em milissegundos desde Unix epoch).
fn data_flexivel(v: &serde_json::Value) -> Option<NaiveDate> {
    match v {
        serde_json::Value::String(s) => {
            if let Ok(d) = NaiveDate::parse_from_str(s, "%d-%m-%Y") {
                return Some(d);
            }
            if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                return Some(d);
            }
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                return Some(dt.date_naive());
            }
            None
        }
        serde_json::Value::Number(n) => n
            .as_i64()
            .and_then(chrono::DateTime::from_timestamp_millis)
            .map(|dt| dt.date_naive()),
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Mtr {
    pub numero: String,
    /// Código de barras completo do manifesto, quando o sistema o fornece
    /// (ex.: MTR-PE). Fica vazio em sistemas que não expõem esse código
    /// (ex.: SINIR, SIGOR).
    #[serde(default)]
    pub barcode: String,
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

#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
#[derive(Default)]
pub enum StatusMtr {
    #[default]
    Ativo,
    EmTransporte,
    Recebido,
    Cancelado,
    Expirado,
    Outro(String),
}

impl Serialize for StatusMtr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_periodo_deserializa_intervalo() {
        let p: Periodo =
            serde_json::from_str(r#"{"dataInicial":"01-06-2026","dataFinal":"10-08-2026"}"#)
                .expect("intervalo válido");
        assert_eq!(
            p.inicio.0,
            NaiveDate::from_ymd_opt(2026, 6, 1).expect("data válida")
        );
        assert_eq!(
            p.fim.0,
            NaiveDate::from_ymd_opt(2026, 8, 10).expect("data válida")
        );
    }

    #[test]
    fn test_periodo_deserializa_iso8601_new_date() {
        // Equivalente ao que JSON.stringify(new Date()) produz no JS
        let p: Periodo = serde_json::from_str(
            r#"{"dataInicial":"2026-06-01T00:00:00.000Z","dataFinal":"2026-08-10T12:30:00.000Z"}"#,
        )
        .expect("ISO 8601 válido");
        assert_eq!(
            p.inicio.0,
            NaiveDate::from_ymd_opt(2026, 6, 1).expect("data válida")
        );
        assert_eq!(
            p.fim.0,
            NaiveDate::from_ymd_opt(2026, 8, 10).expect("data válida")
        );
    }

    #[test]
    fn test_periodo_deserializa_iso8601_data_simples() {
        let p: Periodo =
            serde_json::from_str(r#"{"dataInicial":"2026-06-01","dataFinal":"2026-08-10"}"#)
                .expect("ISO data simples");
        assert_eq!(
            p.inicio.0,
            NaiveDate::from_ymd_opt(2026, 6, 1).expect("data válida")
        );
        assert_eq!(
            p.fim.0,
            NaiveDate::from_ymd_opt(2026, 8, 10).expect("data válida")
        );
    }

    #[test]
    fn test_periodo_deserializa_timestamp_ms() {
        let inicio = NaiveDate::from_ymd_opt(2026, 6, 1).expect("data válida");
        let fim = NaiveDate::from_ymd_opt(2026, 8, 10).expect("data válida");
        let ts_inicio = inicio
            .and_hms_opt(0, 0, 0)
            .expect("hora válida")
            .and_utc()
            .timestamp_millis();
        let ts_fim = fim
            .and_hms_opt(0, 0, 0)
            .expect("hora válida")
            .and_utc()
            .timestamp_millis();
        let p: Periodo = serde_json::from_str(&format!(
            r#"{{"dataInicial":{ts_inicio},"dataFinal":{ts_fim}}}"#
        ))
        .expect("timestamp válido");
        assert_eq!(p.inicio.0, inicio);
        assert_eq!(p.fim.0, fim);
    }

    #[test]
    fn test_periodo_deserializa_data_invalida() {
        let result: Result<Periodo, _> =
            serde_json::from_str(r#"{"dataInicial":"31-13-2026","dataFinal":"10-08-2026"}"#);
        assert!(result.is_err(), "data inválida deve falhar");
    }

    #[test]
    fn test_periodo_deserializa_dias() {
        let hoje = crate::openmtr::time_util::utc_now().date_naive();
        let p: Periodo = serde_json::from_str(r#"{"dias":7}"#).expect("atalho dias");
        assert_eq!(p.fim.0, hoje);
        assert_eq!(p.inicio.0, hoje - chrono::Duration::days(7));
    }

    #[test]
    fn test_ultimos_dias_em_e_deterministico() {
        let hoje = NaiveDate::from_ymd_opt(2026, 6, 1).expect("data válida");
        let p = Periodo::ultimos_dias_em(hoje, 7);
        assert_eq!(p.fim.0, hoje);
        assert_eq!(
            p.inicio.0,
            NaiveDate::from_ymd_opt(2026, 5, 25).expect("data válida")
        );
    }

    #[test]
    fn test_periodo_deserializa_intervalo_invertido() {
        let result: Result<Periodo, _> =
            serde_json::from_str(r#"{"dataInicial":"10-08-2026","dataFinal":"01-06-2026"}"#);
        assert!(result.is_err(), "intervalo invertido deve falhar");
    }

    #[test]
    fn test_periodo_serializa_intervalo() {
        let p = Periodo::new(
            DataMtr(NaiveDate::from_ymd_opt(2026, 6, 1).expect("data válida")),
            DataMtr(NaiveDate::from_ymd_opt(2026, 6, 30).expect("data válida")),
        )
        .expect("período válido");
        let json = serde_json::to_string(&p).expect("serialize");
        assert!(json.contains(r#""dataInicial":"01-06-2026""#));
        assert!(json.contains(r#""dataFinal":"30-06-2026""#));
    }

    #[test]
    fn test_filtro_mtr_aceita_dias() {
        let f: FiltroMtr =
            serde_json::from_str(r#"{"periodo":{"dias":7},"tamanho":5}"#).expect("filtro com dias");
        assert!(f.periodo.is_some());
        assert_eq!(f.tamanho, Some(5));
    }

    #[test]
    fn test_status_mtr_serializa_como_string() {
        let status = StatusMtr::Outro("Cancelado Automático por prazo".into());
        let json = serde_json::to_string(&status).expect("serialize");
        assert_eq!(json, r#""Cancelado Automático por prazo""#);
        assert!(!json.contains("Outro"), "não pode serializar como objeto");
    }

    #[test]
    fn test_status_mtr_variantes_serializam_string() {
        for (status, esperado) in [
            (StatusMtr::Ativo, "Ativo"),
            (StatusMtr::EmTransporte, "Em Transporte"),
            (StatusMtr::Recebido, "Recebido"),
            (StatusMtr::Cancelado, "Cancelado"),
        ] {
            assert_eq!(
                serde_json::to_string(&status).expect("serialize"),
                format!(r#""{esperado}""#)
            );
        }
    }
}
