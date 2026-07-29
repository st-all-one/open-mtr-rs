use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ResiduoIbama {
    #[serde(rename = "resCodigoIbama")]
    pub codigo_ibama: String,
    #[serde(rename = "resDescricao")]
    pub descricao: String,
}

impl ResiduoIbama {
    #[must_use]
    pub fn is_perigoso(&self) -> bool {
        self.codigo_ibama.contains('*')
    }
}
