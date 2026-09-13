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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantidade_recebida: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub densidade: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numero_onu: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classe_risco: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nome_embarque: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grupo_embalagem: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codigo_interno: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descricao_interna: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub densidade: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numero_onu: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classe_risco: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nome_embarque: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grupo_embalagem: Option<u32>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codigo_ibama_novo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tratamento_novo: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classe_nova: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub densidade: Option<f64>,
    /// Quantidade original declarada no manifesto (`marQuantidade`).
    /// Quando ausente, usa-se `quantidade_recebida` como fallback.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantidade: Option<f64>,
    /// Justificativa da divergência de quantidade ou da substituição de
    /// resíduo (`marJustificativa`). Obrigatória nesses casos no SINIR.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub justificativa: Option<String>,
    /// Nº ONU — apenas resíduo perigoso (`marNumeroONU`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numero_onu: Option<String>,
    /// Classe de risco — apenas resíduo perigoso (`marClasseRisco`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classe_risco: Option<String>,
    /// Nome de embarque — apenas resíduo perigoso (`marNomeEmbarque`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nome_embarque: Option<String>,
    /// Grupo de embalagem — apenas resíduo perigoso (`greCodigo`, inteiro).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grupo_embalagem: Option<u32>,
}

impl ResiduoRecebido {
    /// Quantidade original declarada no manifesto (fallback: a recebida).
    #[must_use]
    pub fn quantidade_original(&self) -> f64 {
        self.quantidade.unwrap_or(self.quantidade_recebida)
    }

    /// `true` quando a quantidade recebida difere da original.
    #[must_use]
    pub fn quantidade_divergente(&self) -> bool {
        (self.quantidade_original() - self.quantidade_recebida).abs() > f64::EPSILON
    }

    /// Valida as regras mínimas do recebimento SINIR para este resíduo.
    ///
    /// # Errors
    /// Retorna mensagem quando falta estado físico, classe, acondicionamento
    /// ou tratamento, ou quando há divergência de quantidade/substituição sem
    /// `justificativa`.
    pub fn validar(&self) -> Result<(), String> {
        if self.estado_fisico.is_none() {
            return Err("tieCodigo (estado físico) é obrigatório no recebimento SINIR".into());
        }
        if self.classe.is_none() {
            return Err("claCodigo (classe) é obrigatório no recebimento SINIR".into());
        }
        if self.acondicionamento.is_none() {
            return Err("tiaCodigo (acondicionamento) é obrigatório no recebimento SINIR".into());
        }
        if self.tratamento.is_none() {
            return Err("traCodigo (tratamento) é obrigatório no recebimento SINIR".into());
        }
        let substitui = self.codigo_ibama_novo.as_deref().is_some_and(|n| !n.trim().is_empty());
        let tem_justificativa =
            self.justificativa.as_deref().is_some_and(|j| !j.trim().is_empty());
        if (self.quantidade_divergente() || substitui) && !tem_justificativa {
            return Err(
                "marJustificativa é obrigatória quando há divergência de quantidade ou substituição de resíduo"
                    .into(),
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResiduoIbama {
    pub codigo_ibama: String,
    pub descricao: String,
}

impl ResiduoIbama {
    #[must_use]
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
            densidade: None,
            numero_onu: None,
            classe_risco: None,
            nome_embarque: None,
            grupo_embalagem: None,
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("\"codigo_ibama\""));
        assert!(json.contains("\"quantidade\""));
    }

    fn residuo_recebido_exemplo() -> ResiduoRecebido {
        ResiduoRecebido {
            codigo_ibama: "Classe A".to_string(),
            quantidade_recebida: 5.0,
            unidade: 3,
            estado_fisico: Some(4),
            classe: Some(11),
            acondicionamento: Some(4),
            tratamento: Some(43),
            codigo_ibama_novo: None,
            tratamento_novo: None,
            classe_nova: None,
            densidade: None,
            quantidade: Some(5.0),
            justificativa: None,
            numero_onu: None,
            classe_risco: None,
            nome_embarque: None,
            grupo_embalagem: None,
        }
    }

    #[test]
    fn test_residuo_recebido_serializa_substituicao() {
        let mut r = residuo_recebido_exemplo();
        r.codigo_ibama_novo = Some("170107".to_string());
        r.justificativa = Some("RCC".to_string());
        let json = serde_json::to_value(&r).unwrap();
        assert_eq!(json["codigo_ibama_novo"], "170107");
        assert_eq!(json["justificativa"], "RCC");
        assert_eq!(json["quantidade"], 5.0);
        assert!(json.get("numero_onu").is_none(), "perigoso não deve serializar vazio");
    }

    #[test]
    fn test_residuo_recebido_desserializa_json_antigo() {
        let json = r#"{"codigo_ibama":"170107","quantidade_recebida":5.0,"unidade":3}"#;
        let r: ResiduoRecebido = serde_json::from_str(json).unwrap();
        assert_eq!(r.quantidade, None);
        assert_eq!(r.justificativa, None);
        assert_eq!(r.estado_fisico, None);
    }

    #[test]
    fn test_validar_exige_justificativa_em_substituicao() {
        let mut r = residuo_recebido_exemplo();
        r.codigo_ibama_novo = Some("170107".to_string());
        assert!(r.validar().is_err(), "substituição sem justificativa deve falhar");
        r.justificativa = Some("RCC".to_string());
        assert!(r.validar().is_ok());
    }

    #[test]
    fn test_validar_exige_campos_obrigatorios() {
        let mut r = residuo_recebido_exemplo();
        r.estado_fisico = None;
        assert!(r.validar().is_err());
    }

    #[test]
    fn test_quantidade_original_fallback() {
        let mut r = residuo_recebido_exemplo();
        r.quantidade = None;
        assert_eq!(r.quantidade_original(), 5.0);
        assert!(!r.quantidade_divergente());

        let mut d = residuo_recebido_exemplo();
        d.quantidade = Some(5.0);
        d.quantidade_recebida = 4.0;
        assert!(d.quantidade_divergente());
    }
}
