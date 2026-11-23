use super::Login;
use std::fmt;

impl fmt::Debug for Login {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Login")
            .field("system", &self.system)
            .field("token", &self.token.as_ref().map(|_| "<redacted>"))
            .field("token_env", &self.token_env)
            .field("cpf", &self.cpf)
            .field("cnpj", &self.cnpj)
            .field("unidade", &self.unidade)
            .field("senha", &self.senha.as_ref().map(|_| "<redacted>"))
            .field("senha_env", &self.senha_env)
            .field("base_url", &self.base_url)
            .field("timeout_secs", &self.timeout_secs)
            .field("max_rps", &self.max_rps)
            .finish()
    }
}
