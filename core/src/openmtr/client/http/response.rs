/// Corpo de uma resposta mantido em bytes, sem desserialização, para downloads.
///
/// Devolvido por [`MtrHttpClient::post_bytes`](super::MtrHttpClient::post_bytes)
/// e [`MtrHttpClient::get_bytes`](super::MtrHttpClient::get_bytes).
/// Use [`Self::validate`] para conferir o formato antes de persistir o arquivo.
#[derive(Debug, Clone)]
pub struct RawResponse {
    /// Corpo da resposta exatamente como veio do transporte.
    pub bytes: Vec<u8>,
    /// Valor do cabeçalho `Content-Type`, ou `None` se ausente/ilegível.
    pub content_type: Option<String>,
    /// Valor do cabeçalho `Content-Length` em bytes, quando informado.
    pub content_length: Option<u64>,
}

impl RawResponse {
    /// Valida a resposta contra o formato esperado antes de persisti-la em arquivo.
    ///
    /// `expected` é `"pdf"` ou `"xml"`; qualquer outro valor desativa a checagem
    /// de magic bytes. Um `Content-Type` `text/html` ou `text/plain` é sempre
    /// rejeitado — indica página de erro em vez do arquivo. Para `"xml"`, o
    /// `Content-Type` (quando presente) deve conter `xml` ou `octet-stream`.
    /// Os magic bytes exigem `%PDF` no início de um PDF e `<?xml` ou `<` no
    /// início de um XML. Se `content_length` foi informado, deve ser igual a
    /// [`Self::bytes`]. Em caso de divergência, devolve a mensagem do problema.
    pub fn validate(&self, expected: &str) -> Result<(), String> {
        if let Some(ct) = &self.content_type {
            let ct_lower = ct.to_lowercase();
            if ct_lower.contains("text/html") || ct_lower.contains("text/plain") {
                return Err(format!(
                    "Content-Type inesperado: {ct} (esperado {expected})"
                ));
            }
            if expected == "xml" && !ct_lower.contains("xml") && !ct_lower.contains("octet-stream")
            {
                return Err(format!(
                    "Content-Type inesperado: {ct} (esperado application/xml)"
                ));
            }
        }

        let magic_ok = match expected {
            "pdf" => self.bytes.starts_with(b"%PDF"),
            "xml" => self.bytes.starts_with(b"<?xml") || self.bytes.starts_with(b"<"),
            _ => true,
        };
        if !magic_ok {
            return Err(format!(
                "Arquivo inválido: magic bytes não correspondem ao formato {expected}"
            ));
        }

        if let Some(cl) = self.content_length
            && cl != u64::try_from(self.bytes.len()).unwrap_or(u64::MAX)
        {
            return Err(format!(
                "Content-Length ({cl}) difere dos bytes recebidos ({})",
                self.bytes.len()
            ));
        }

        Ok(())
    }
}
