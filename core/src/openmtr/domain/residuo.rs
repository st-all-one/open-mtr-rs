#![allow(clippy::module_name_repetitions)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Residuo {
    pub codigo_ibama: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descricao: Option<String>,
    pub quantidade: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unidade: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estado_fisico: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classe: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acondicionamento: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tratamento: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResiduoCriacao {
    pub codigo_ibama: String,
    pub quantidade: f64,
    pub unidade: u32,
    pub estado_fisico: u32,
    pub classe: u32,
    pub acondicionamento: u32,
    pub tratamento: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descricao_interna: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codigo_interno: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResiduoRecebido {
    pub codigo_ibama: String,
    pub quantidade_recebida: f64,
    pub unidade: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estado_fisico: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classe: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acondicionamento: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tratamento: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResiduoIbama {
    pub codigo_ibama: String,
    pub descricao: String,
}

impl ResiduoIbama {
    pub fn is_perigoso(&self) -> bool {
        let code = self.codigo_ibama.trim().to_uppercase();
        code.starts_with('F')
            || code.starts_with('K')
            || code.starts_with('P')
            || code.starts_with('U')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_residuo_perigoso() {
        let r = ResiduoIbama {
            codigo_ibama: "F001".to_string(),
            descricao: "Resíduo perigoso".to_string(),
        };
        assert!(r.is_perigoso());
    }

    #[test]
    fn test_residuo_nao_perigoso() {
        let r = ResiduoIbama {
            codigo_ibama: "A001".to_string(),
            descricao: "Resíduo não perigoso".to_string(),
        };
        assert!(!r.is_perigoso());
    }

    #[test]
    fn test_residuo_criacao_serialize() {
        let r = ResiduoCriacao {
            codigo_ibama: "F001".to_string(),
            quantidade: 100.0,
            unidade: 1,
            estado_fisico: 1,
            classe: 1,
            acondicionamento: 1,
            tratamento: 1,
            descricao_interna: None,
            codigo_interno: None,
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("\"codigo_ibama\""));
        assert!(json.contains("\"quantidade\""));
    }
}
