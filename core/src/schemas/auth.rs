use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ApiwsLoginDTO {
    pub usuario: String,
    pub senha: String,
}

#[derive(Debug, Deserialize)]
pub struct TokenObjeto {
    pub token: String,
    #[serde(rename = "tipo")]
    pub _tipo: String,
    #[serde(rename = "expiraEm")]
    pub expira_em: u64,
}

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub erro: bool,
    pub objeto: Option<TokenObjeto>,
    pub mensagem: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_response_deserialize() {
        let json = r#"{
            "erro": false,
            "objeto": {
                "token": "eyJhbGciOiJIUzI1NiIs...",
                "tipo": "Bearer",
                "expiraEm": 28800
            },
            "mensagem": "Autenticado com sucesso"
        }"#;

        let parsed: TokenResponse = serde_json::from_str(json).unwrap();
        assert!(!parsed.erro);
        assert!(parsed.objeto.is_some());
        assert_eq!(parsed.objeto.unwrap().expira_em, 28800);
    }

    #[test]
    fn test_token_response_error() {
        let json = r#"{
            "erro": true,
            "objeto": null,
            "mensagem": "Token inválido"
        }"#;

        let parsed: TokenResponse = serde_json::from_str(json).unwrap();
        assert!(parsed.erro);
        assert!(parsed.objeto.is_none());
    }
}
