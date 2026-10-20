use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::openmtr::error::OpenMTRError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resposta<T> {
    pub id: Uuid,
    pub ok: bool,
    pub status: u16,
    pub mensagem: String,
    pub dados: Vec<T>,
    pub erros: Vec<OpenMTRError>,
    /// Total de registros disponíveis (para paginação).
    /// Preenchido quando a API retorna totalRecords.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
}

impl<T> Resposta<T> {
    #[must_use]
    pub fn sucesso(id: Uuid, dados: Vec<T>) -> Self {
        Self {
            id,
            ok: true,
            status: 200,
            mensagem: String::new(),
            dados,
            erros: Vec::new(),
            total: None,
        }
    }

    #[must_use]
    pub fn com_erros(id: Uuid, erros: Vec<OpenMTRError>) -> Self {
        Self {
            id,
            ok: false,
            status: 400,
            mensagem: String::new(),
            dados: Vec::new(),
            erros,
            total: None,
        }
    }

    pub fn com_mensagem(id: Uuid, dados: Vec<T>, mensagem: impl Into<String>) -> Self {
        Self {
            id,
            ok: true,
            status: 200,
            mensagem: mensagem.into(),
            dados,
            erros: Vec::new(),
            total: None,
        }
    }

    #[must_use]
    pub fn com_total(id: Uuid, dados: Vec<T>, total: i64) -> Self {
        Self {
            id,
            ok: true,
            status: 200,
            mensagem: String::new(),
            dados,
            erros: Vec::new(),
            total: Some(total),
        }
    }

    #[must_use]
    pub fn tem_erros(&self) -> bool {
        !self.erros.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resposta_sucesso() {
        let id = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));
        let r = Resposta::sucesso(id, vec![1, 2, 3]);
        assert!(r.ok);
        assert_eq!(r.status, 200);
        assert_eq!(r.dados, vec![1, 2, 3]);
        assert!(r.erros.is_empty());
    }

    #[test]
    fn test_resposta_com_erros() {
        let id = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));
        let e = OpenMTRError::validation("campo obrigatório");
        let r: Resposta<i32> = Resposta::com_erros(id, vec![e]);
        assert!(!r.ok);
        assert!(r.tem_erros());
        assert!(r.dados.is_empty());
    }

    #[test]
    fn test_resposta_com_mensagem() {
        let id = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));
        let r = Resposta::sucesso(id, vec!["a"]);
        assert_eq!(r.mensagem, "");
        let r = Resposta::com_mensagem(id, vec!["a"], "criado com sucesso");
        assert_eq!(r.mensagem, "criado com sucesso");
    }
}
