use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResiduoRecebido {
    #[serde(rename = "resCodigoIbama")]
    pub res_codigo_ibama: String,
    #[serde(rename = "quantidadeRecebida")]
    pub quantidade_recebida: f64,
    #[serde(rename = "uniCodigo")]
    pub uni_codigo: i32,
    #[serde(rename = "tieCodigo", skip_serializing_if = "Option::is_none")]
    pub tie_codigo: Option<i32>,
    #[serde(rename = "claCodigo", skip_serializing_if = "Option::is_none")]
    pub cla_codigo: Option<i32>,
    #[serde(rename = "tiaCodigo", skip_serializing_if = "Option::is_none")]
    pub tia_codigo: Option<i32>,
    #[serde(rename = "traCodigo", skip_serializing_if = "Option::is_none")]
    pub tra_codigo: Option<i32>,
    #[serde(rename = "marSequencial", skip_serializing_if = "Option::is_none")]
    pub sequencial: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct ApiwsManifestoDeRecebimentoDTO {
    #[serde(rename = "manNumero")]
    pub man_numero: String,
    #[serde(rename = "dataRecebimento")]
    pub data_recebimento: TimestampMs,
    #[serde(rename = "nomeResponsavel")]
    pub nome_responsavel: String,
    pub observacoes: Option<String>,
    #[serde(rename = "listaResiduosRecebidos")]
    pub lista_residuos_recebidos: Option<Vec<ResiduoRecebido>>,
}

#[derive(Debug, Clone, Copy)]
pub struct TimestampMs(pub i64);

impl TimestampMs {
    #[must_use]
    pub fn from_datetime(dt: &DateTime<Utc>) -> Self {
        Self(dt.timestamp_millis())
    }

    #[must_use]
    pub fn now() -> Self {
        Self(Utc::now().timestamp_millis())
    }
}

impl Default for TimestampMs {
    fn default() -> Self {
        Self::now()
    }
}

impl Serialize for TimestampMs {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_i64(self.0)
    }
}

impl<'de> Deserialize<'de> for TimestampMs {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        i64::deserialize(d).map(Self)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecebimentoLoteItem {
    #[serde(rename = "manNumero")]
    pub man_numero: String,
    pub recebido: bool,
    pub mensagem: String,
    #[serde(rename = "codigoGerado", default)]
    pub codigo_gerado: Option<i64>,
    #[serde(rename = "dataRecebimento", default)]
    pub data_recebimento: Option<TimestampMs>,
    #[serde(rename = "listaResiduosRecebidos", default)]
    pub lista_residuos_recebidos: Option<Vec<ResiduoRecebido>>,
}

#[derive(Debug, Deserialize)]
pub struct RecebimentoLoteResponse {
    pub objeto: Option<Vec<RecebimentoLoteItem>>,
}

#[derive(Debug, Serialize)]
pub struct ResiduoAlterado {
    #[serde(rename = "resCodigoIbama")]
    pub res_codigo_ibama: String,
    #[serde(rename = "novaQuantidade")]
    pub nova_quantidade: f64,
    #[serde(rename = "uniCodigo")]
    pub uni_codigo: i32,
    pub observacoes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiwsAlteraRecebimentoDTO {
    #[serde(rename = "manNumero")]
    pub man_numero: String,
    pub justificativa: String,
    #[serde(rename = "listaResiduosAlterados")]
    pub lista_residuos_alterados: Vec<ResiduoAlterado>,
}

#[derive(Debug, Serialize)]
pub struct AceiteAlteracaoDTO {
    #[serde(rename = "manNumero")]
    pub man_numero: String,
    pub aceite: bool,
    pub observacoes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AceiteAlteracaoResponse {
    #[serde(rename = "manNumero")]
    pub man_numero: String,
    #[serde(rename = "alteracaoAceita")]
    pub alteracao_aceita: bool,
    #[serde(rename = "dataProcessamento")]
    pub data_processamento: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_ms_now() {
        let ts = TimestampMs::now();
        assert!(ts.0 > 1_700_000_000_000);
    }

    #[test]
    fn test_serialize_timestamp_ms() {
        let ts = TimestampMs(1_704_067_200_000);
        let json = serde_json::to_value(&ts).unwrap();
        assert_eq!(json, serde_json::json!(1704067200000i64));
    }

    #[test]
    fn test_recebimento_dto_com_residuos() {
        let dto = ApiwsManifestoDeRecebimentoDTO {
            man_numero: "00001".into(),
            data_recebimento: TimestampMs(1_704_067_200_000),
            nome_responsavel: "Maria".into(),
            observacoes: None,
            lista_residuos_recebidos: Some(vec![ResiduoRecebido {
                res_codigo_ibama: "A001".into(),
                quantidade_recebida: 1500.5,
                uni_codigo: 1,
                tie_codigo: None,
                cla_codigo: None,
                tia_codigo: None,
                tra_codigo: None,
                sequencial: None,
            }]),
        };

        let json = serde_json::to_value(&dto).unwrap();
        assert_eq!(json["manNumero"], "00001");
        assert_eq!(json["listaResiduosRecebidos"][0]["quantidadeRecebida"], 1500.5);
    }

    #[test]
    fn test_aceite_alteracao_serialize() {
        let aceite = AceiteAlteracaoDTO {
            man_numero: "00001".into(),
            aceite: false,
            observacoes: Some("Divergência não justificada".into()),
        };

        let json = serde_json::to_value(&aceite).unwrap();
        assert_eq!(json["aceite"], false);
        assert_eq!(json["observacoes"], "Divergência não justificada");
    }
}
