use serde::{Deserialize, Serialize};

use crate::core::errors::MtrError;
use crate::schemas::cnpj::Cnpj;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParteMTR {
    pub cnpj: Cnpj,
    #[serde(rename = "razaoSocial")]
    pub razao_social: String,
    pub endereco: Option<String>,
    pub municipio: Option<String>,
    pub uf: Option<String>,
}

impl ParteMTR {
    pub fn validar(&self) -> Result<(), MtrError> {
        Cnpj::new(self.cnpj.as_str()).map(|_| ())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResiduoIntegracao {
    #[serde(rename = "resCodigoIbama")]
    pub res_codigo_ibama: String,
    pub quantidade: f64,
    #[serde(rename = "uniCodigo")]
    pub uni_codigo: i32,
    pub acondicionamento: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiwsManifestoDTO {
    #[serde(rename = "manNumero")]
    pub man_numero: String,
    #[serde(rename = "manDataEmissao")]
    pub man_data_emissao: i64,
    pub gerador: ParteMTR,
    pub transportador: ParteMTR,
    pub destinador: ParteMTR,
    #[serde(rename = "listaResiduos")]
    pub lista_residuos: Vec<ResiduoIntegracao>,
    pub observacoes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ErroIntegracao {
    #[serde(rename = "manNumero")]
    pub man_numero: String,
    pub mensagem: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RespostaIntegracaoObjeto {
    #[serde(rename = "manifestosProcessados")]
    pub manifestos_processados: i32,
    #[serde(rename = "manifestosComErro")]
    pub manifestos_com_erro: i32,
    pub erros: Vec<ErroIntegracao>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::cnpj::Cnpj;
    use crate::schemas::cpf::{Cpf, CpfCnpj};

    #[test]
    fn test_manifesto_estadual_serialize() {
        let m = ApiwsManifestoDTO {
            man_numero: "MTR-SP-001".into(),
            man_data_emissao: 1_704_067_200_000,
            gerador: ParteMTR {
                cnpj: Cnpj::new("11222333000181").unwrap(),
                razao_social: "Gerador Ltda".into(),
                endereco: None,
                municipio: None,
                uf: None,
            },
            transportador: ParteMTR {
                cnpj: Cnpj::new("55666777000181").unwrap(),
                razao_social: "Transportadora Ltda".into(),
                endereco: None,
                municipio: None,
                uf: None,
            },
            destinador: ParteMTR {
                cnpj: Cnpj::new("99888777000100").unwrap(),
                razao_social: "Destinadora Ltda".into(),
                endereco: None,
                municipio: None,
                uf: None,
            },
            lista_residuos: vec![ResiduoIntegracao {
                res_codigo_ibama: "A001".into(),
                quantidade: 1500.5,
                uni_codigo: 1,
                acondicionamento: Some(5),
            }],
            observacoes: None,
        };

        let json = serde_json::to_value(&m).unwrap();
        assert_eq!(json["manNumero"], "MTR-SP-001");
        assert_eq!(json["listaResiduos"][0]["quantidade"], 1500.5);
        assert_eq!(json["gerador"]["cnpj"], "11222333000181");
        assert_eq!(json["transportador"]["cnpj"], "55666777000181");
        assert_eq!(json["destinador"]["cnpj"], "99888777000100");
    }

    #[test]
    fn test_resposta_integracao_deserialize() {
        let json = r#"{
            "manifestosProcessados": 10,
            "manifestosComErro": 2,
            "erros": [
                { "manNumero": "MTR-RJ-001", "mensagem": "Manifesto já integrado" }
            ]
        }"#;

        let parsed: RespostaIntegracaoObjeto = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.manifestos_processados, 10);
        assert_eq!(parsed.manifestos_com_erro, 2);
        assert_eq!(parsed.erros.len(), 1);
    }

    #[test]
    fn test_erro_integracao_deserialize() {
        let json = r#"{"manNumero": "MTR-001", "mensagem": "CNPJ inválido"}"#;
        let erro: ErroIntegracao = serde_json::from_str(json).unwrap();
        assert_eq!(erro.man_numero, "MTR-001");
    }
}
