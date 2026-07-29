#![allow(clippy::module_name_repetitions)]

use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Problema {
    pub r#type: Url,
    pub title: String,
    pub status: u16,
    pub detail: String,
    pub instance: Uuid,
}

impl Problema {
    pub fn new(status: u16, title: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            r#type: Url::parse("urn:openmtr:error:api-error")
                .expect("static URN is always valid"),
            title: title.into(),
            status,
            detail: detail.into(),
            instance: Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)),
        }
    }

    pub fn validation(detail: impl Into<String>) -> Self {
        Self {
            r#type: Url::parse("urn:openmtr:error:validation")
                .expect("static URN is always valid"),
            title: "Erro de validação".into(),
            status: 422,
            detail: detail.into(),
            instance: Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)),
        }
    }

    pub fn not_found(detail: impl Into<String>) -> Self {
        Self {
            r#type: Url::parse("urn:openmtr:error:not-found")
                .expect("static URN is always valid"),
            title: "Não encontrado".into(),
            status: 404,
            detail: detail.into(),
            instance: Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)),
        }
    }

    pub fn api(status: u16, detail: impl Into<String>) -> Self {
        Self {
            r#type: Url::parse("urn:openmtr:error:api-error")
                .expect("static URN is always valid"),
            title: "Erro de API".into(),
            status,
            detail: detail.into(),
            instance: Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resposta<T> {
    pub id: Uuid,
    pub ok: bool,
    pub status: u16,
    pub mensagem: String,
    pub dados: Vec<T>,
    pub erros: Vec<Problema>,
    /// Total de registros disponíveis (para paginação).
    /// Preenchido quando a API retorna totalRecords.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
}

impl<T> Resposta<T> {
    pub fn sucesso(id: Uuid, dados: Vec<T>) -> Self {
        Self { id, ok: true, status: 200, mensagem: String::new(), dados, erros: Vec::new(), total: None }
    }

    pub fn com_erros(id: Uuid, problemas: Vec<Problema>) -> Self {
        Self { id, ok: false, status: 400, mensagem: String::new(), dados: Vec::new(), erros: problemas, total: None }
    }

    pub fn com_mensagem(id: Uuid, dados: Vec<T>, mensagem: impl Into<String>) -> Self {
        Self { id, ok: true, status: 200, mensagem: mensagem.into(), dados, erros: Vec::new(), total: None }
    }

    pub fn com_total(id: Uuid, dados: Vec<T>, total: i64) -> Self {
        Self { id, ok: true, status: 200, mensagem: String::new(), dados, erros: Vec::new(), total: Some(total) }
    }

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
        let p = Problema::validation("campo obrigatório");
        let r: Resposta<i32> = Resposta::com_erros(id, vec![p]);
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

    #[test]
    fn test_problema_new() {
        let p = Problema::new(409, "Conflito", "registro duplicado");
        assert_eq!(p.status, 409);
        assert_eq!(p.title, "Conflito");
    }

    #[test]
    fn test_problema_validation() {
        let p = Problema::validation("cpf inválido");
        assert_eq!(p.status, 422);
    }

    #[test]
    fn test_problema_not_found() {
        let p = Problema::not_found("mtr não encontrado");
        assert_eq!(p.status, 404);
    }

    #[test]
    fn test_problema_api() {
        let p = Problema::api(502, "upstream error");
        assert_eq!(p.status, 502);
    }
}
