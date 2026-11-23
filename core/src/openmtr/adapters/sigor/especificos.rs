//! Endpoint específico do SIGOR: manifesto complementar.

use serde_json::{Value, json};
use tracing::instrument;

use super::SigorAdapter;
use super::tipos::extrair_objeto;
use crate::openmtr::adapters::MtrCtx;
use crate::openmtr::domain::{CriarMtrComplementar, MtrCriado};
use crate::openmtr::error::OpenMTRError;

impl SigorAdapter {
    /// Cria um manifesto complementar via `/salvarManifestoComplementar`.
    #[instrument(name = "sigor.criar_manifesto_complementar", skip_all)]
    pub(super) async fn criar_manifesto_complementar_impl(
        &self,
        ctx: &MtrCtx,
        req: CriarMtrComplementar,
    ) -> Result<MtrCriado, OpenMTRError> {
        let body = json!({
            "nomeMotorista": req.motorista,
            "placaVeiculo": req.placa_veiculo,
            "observacoes": req.observacoes,
            "transportador": {
                "unidade": req.transportador.unidade.unwrap_or(0),
                "cpfCnpj": req.transportador.documento.as_str(),
            },
            "listaManifesto": req.manifestos.iter().map(|m| json!({"manNumero": m})).collect::<Vec<_>>(),
        });
        let response: Value = ctx
            .http
            .post("/salvarManifestoComplementar", &body, "criar_complementar")
            .await?;
        match extrair_objeto(&response) {
            Some(obj) => {
                let valido = obj
                    .get("restResponseValido")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                Ok(MtrCriado {
                    numero: obj
                        .get("manifestoComplementar")
                        .and_then(Value::as_i64)
                        .map(|n| n.to_string()),
                    codigo_gerado: obj.get("manifestoComplementar").and_then(Value::as_i64),
                    sucesso: valido,
                    mensagem: obj
                        .get("restResponseMensagem")
                        .and_then(|s| s.as_str())
                        .unwrap_or_default()
                        .to_string(),
                })
            }
            None => Err(OpenMTRError::validation(
                "Resposta vazia do manifesto complementar",
            )),
        }
    }
}
