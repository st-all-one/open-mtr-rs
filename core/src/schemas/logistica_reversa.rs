use crate::schemas::common::DataMtr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct MtrLrDataDTO {
    #[serde(rename = "dataInicial")]
    pub data_inicial: DataMtr,
    #[serde(rename = "dataFinal")]
    pub data_final: DataMtr,
}

#[derive(Debug, Serialize)]
pub struct MtrOrigemCompletoDTO {
    #[serde(rename = "manNumero")]
    pub man_numero: String,
}

#[derive(Debug, Deserialize)]
pub struct RastreioEvento {
    pub evento: String,
    pub data: String,
    pub responsavel: Option<String>,
    pub transportador: Option<String>,
    pub destinador: Option<String>,
    pub quantidade: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct MtrInfo {
    #[serde(rename = "manNumero")]
    pub man_numero: String,
    pub gerador: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MtrOrigemCompletoObjeto {
    pub mtr: MtrInfo,
    pub rastreio: Vec<RastreioEvento>,
}

#[derive(Debug, Serialize)]
pub struct MtrProcessoCompletoDTO {
    #[serde(rename = "manNumero")]
    pub man_numero: String,
}

#[derive(Debug, Serialize)]
pub struct ApiwsAlteraManifestoOrigemDTO {
    #[serde(rename = "manNumero")]
    pub man_numero: String,
    #[serde(rename = "novoTitular")]
    pub novo_titular: String,
    pub justificativa: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::common::DataMtr;

    #[test]
    fn test_mtr_lr_data_serialize() {
        let dto = MtrLrDataDTO {
            data_inicial: DataMtr::new(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            data_final: DataMtr::new(chrono::NaiveDate::from_ymd_opt(2024, 1, 31).unwrap()),
        };

        let json = serde_json::to_value(&dto).unwrap();
        assert_eq!(json["dataInicial"], "01-01-2024");
        assert_eq!(json["dataFinal"], "31-01-2024");
    }

    #[test]
    fn test_rastreio_evento_deserialize() {
        let json = r#"{
            "evento": "RECEBIMENTO",
            "data": "10/01/2024",
            "destinador": "Destinadora Ltda",
            "quantidade": 980.0
        }"#;

        let evento: RastreioEvento = serde_json::from_str(json).unwrap();
        assert_eq!(evento.evento, "RECEBIMENTO");
        assert_eq!(evento.destinador.unwrap(), "Destinadora Ltda");
        assert_eq!(evento.quantidade.unwrap(), 980.0);
        assert!(evento.responsavel.is_none());
    }

    #[test]
    fn test_mtr_origem_completo_deserialize() {
        let json = r#"{
            "mtr": {
                "manNumero": "00001",
                "gerador": "Gerador ABC",
                "status": "FINALIZADO"
            },
            "rastreio": [
                { "evento": "EMISSAO", "data": "01/01/2024", "responsavel": "Gerador ABC" }
            ]
        }"#;

        let parsed: MtrOrigemCompletoObjeto = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.mtr.man_numero, "00001");
        assert_eq!(parsed.rastreio.len(), 1);
    }

    #[test]
    fn test_alterar_titularidade_serialize() {
        let dto = ApiwsAlteraManifestoOrigemDTO {
            man_numero: "00001".into(),
            novo_titular: "11.222.333/0001-44".into(),
            justificativa: "Transferência conforme acordo setorial vigente".into(),
        };

        let json = serde_json::to_value(&dto).unwrap();
        assert_eq!(json["manNumero"], "00001");
        assert_eq!(json["novoTitular"], "11.222.333/0001-44");
    }
}
