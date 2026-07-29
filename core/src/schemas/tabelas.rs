use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct TipoAcondicionamento {
    #[serde(rename = "tiaCodigo")]
    pub codigo: i32,
    #[serde(rename = "tiaDescricao")]
    pub descricao: String,
}

pub static COMPATIBILIDADE_ACOND: std::sync::LazyLock<HashMap<i32, Vec<i32>>> =
    std::sync::LazyLock::new(|| {
        let mut m = HashMap::new();
        m.insert(1, vec![1, 2, 3, 6, 8]);
        m.insert(2, vec![2, 5, 9]);
        m.insert(3, vec![2, 5, 6]);
        m.insert(4, vec![9]);
        m.insert(5, vec![1, 2, 5, 9]);
        m
    });

#[derive(Debug, Clone, Deserialize)]
pub struct UnidadeMedida {
    #[serde(rename = "uniCodigo")]
    pub codigo: i32,
    #[serde(rename = "uniDescricao")]
    pub descricao: String,
    #[serde(rename = "uniSigla")]
    pub sigla: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EstadoFisico {
    #[serde(rename = "tieCodigo")]
    pub codigo: i32,
    #[serde(rename = "tieDescricao")]
    pub descricao: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClasseResiduo {
    #[serde(rename = "claCodigo")]
    pub codigo: i32,
    #[serde(rename = "claDescricao")]
    pub descricao: String,
    #[serde(rename = "claResolucao")]
    pub resolucao: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TratamentoResiduo {
    #[serde(rename = "traCodigo")]
    pub codigo: i32,
    #[serde(rename = "traDescricao")]
    pub descricao: String,
}

pub static ESTADOS_FISICOS: std::sync::LazyLock<HashMap<i32, &str>> =
    std::sync::LazyLock::new(|| {
        let mut m = HashMap::new();
        m.insert(1, "Sólido");
        m.insert(2, "Líquido");
        m.insert(3, "Pastoso");
        m.insert(4, "Gasoso");
        m.insert(5, "Multifásico");
        m
    });

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tipo_acondicionamento_deserialize() {
        let json = r#"{"tiaCodigo": 1, "tiaDescricao": "Container"}"#;
        let parsed: TipoAcondicionamento = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.codigo, 1);
        assert_eq!(parsed.descricao, "Container");
    }

    #[test]
    fn test_estados_fisicos_completos() {
        assert_eq!(ESTADOS_FISICOS.len(), 5);
        assert_eq!(*ESTADOS_FISICOS.get(&1).unwrap(), "Sólido");
        assert_eq!(*ESTADOS_FISICOS.get(&5).unwrap(), "Multifásico");
    }

    #[test]
    fn test_compatibilidade_completa() {
        for cod in 1..=5 {
            let comp = COMPATIBILIDADE_ACOND.get(&cod);
            assert!(comp.is_some(), "Estado {cod} sem compatibilidade");
            assert!(!comp.unwrap().is_empty(), "Estado {cod} com lista vazia");
        }
    }

    #[test]
    fn test_gasoso_so_tanque() {
        let comp = COMPATIBILIDADE_ACOND.get(&4).unwrap();
        assert_eq!(comp.as_slice(), &[9]);
    }

    #[test]
    fn test_solido_tem_container() {
        let comp = COMPATIBILIDADE_ACOND.get(&1).unwrap();
        assert!(comp.contains(&1));
    }
}
