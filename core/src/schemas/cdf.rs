use crate::core::errors::MtrError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ApiwsCertificadoDestinacaoDTO {
    #[serde(rename = "listaManifestos")]
    pub lista_manifestos: Vec<String>,
    #[serde(rename = "dataEmissao")]
    pub data_emissao: i64,
    #[serde(rename = "nomeResponsavel")]
    pub nome_responsavel: String,
    pub observacoes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CDFEmissaoObjeto {
    #[serde(rename = "cdfCodigo")]
    pub cdf_codigo: i32,
    #[serde(rename = "cdfNumero")]
    pub cdf_numero: String,
    #[serde(rename = "dataEmissao")]
    pub data_emissao: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct CDFEmissaoResponse {
    pub objeto: Option<CDFEmissaoObjeto>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormatoDownload {
    Pdf,
    Xml,
}

impl FormatoDownload {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Xml => "xml",
        }
    }
}

impl std::fmt::Display for FormatoDownload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub fn validar_emissao(payload: &ApiwsCertificadoDestinacaoDTO) -> Result<(), MtrError> {
    if payload.lista_manifestos.is_empty() {
        return Err(MtrError::validation(
            "Lista de manifestos não pode estar vazia para emissão do CDF",
        ));
    }

    let now_ms = chrono::Utc::now().timestamp_millis();
    if payload.data_emissao > now_ms {
        return Err(MtrError::validation(
            "Data de emissão não pode ser futura",
        ));
    }

    let noventa_dias_ms = 90i64 * 24 * 60 * 60 * 1000;
    if now_ms - payload.data_emissao > noventa_dias_ms {
        return Err(MtrError::validation(
            "Data de emissão não pode ser superior a 90 dias no passado",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formato_download_display() {
        assert_eq!(FormatoDownload::Pdf.to_string(), "pdf");
        assert_eq!(FormatoDownload::Xml.to_string(), "xml");
    }

    #[test]
    fn test_cdf_emissao_deserialize() {
        let json = r#"{
            "cdfCodigo": 12345,
            "cdfNumero": "CDF-2024-00001",
            "dataEmissao": "31/12/2024",
            "status": "EMITIDO"
        }"#;

        let parsed: CDFEmissaoObjeto = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.cdf_codigo, 12345);
        assert_eq!(parsed.status, "EMITIDO");
    }

    #[test]
    fn test_certificado_destinacao_serialize() {
        let dto = ApiwsCertificadoDestinacaoDTO {
            lista_manifestos: vec!["00001".into(), "00002".into()],
            data_emissao: 1_704_067_200_000,
            nome_responsavel: "Carlos".into(),
            observacoes: Some("Tratamento concluído".into()),
        };

        let json = serde_json::to_value(&dto).unwrap();
        assert_eq!(json["listaManifestos"].as_array().unwrap().len(), 2);
        assert_eq!(json["nomeResponsavel"], "Carlos");
    }

    #[test]
    fn test_validacao_lista_vazia() {
        let dto = ApiwsCertificadoDestinacaoDTO {
            lista_manifestos: vec![],
            data_emissao: 1_704_067_200_000,
            nome_responsavel: "Carlos".into(),
            observacoes: None,
        };

        let result = validar_emissao(&dto);
        assert!(result.is_err());
    }

    #[test]
    fn test_validacao_data_futura() -> Result<(), Box<dyn std::error::Error>> {
        let futuro = chrono::Utc::now().timestamp_millis() + 86_400_000;
        let dto = ApiwsCertificadoDestinacaoDTO {
            lista_manifestos: vec!["00001".into()],
            data_emissao: futuro,
            nome_responsavel: "Carlos".into(),
            observacoes: None,
        };

        let result = validar_emissao(&dto);
        assert!(result.is_err());
        Ok(())
    }
}
