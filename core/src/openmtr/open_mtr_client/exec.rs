use tracing::{Instrument, error, info, info_span};

use crate::openmtr::adapters::{ItemResult, LoteResult};
use crate::openmtr::error::OpenMTRError;
use crate::openmtr::resposta::Resposta;

use super::{MtrResult, OpenMtrClient};

impl OpenMtrClient {
    pub(crate) async fn exec_lote<T>(
        &self,
        op: &str,
        future: impl Future<Output = Result<LoteResult<T>, OpenMTRError>>,
    ) -> MtrResult<T> {
        let id = self.ctx.new_id();
        let span = info_span!("openmtr", operation = %op, instance = %id, system = %self.ctx.config.system.nome());

        match future.instrument(span).await {
            Ok(lote) => {
                let LoteResult { itens, total } = lote;
                let mut dados = Vec::with_capacity(itens.len());
                let mut erros = Vec::new();
                for item in itens {
                    match item {
                        ItemResult::Ok(valor) => dados.push(valor),
                        ItemResult::Erro { mut problema, .. } => {
                            problema.instance = self.ctx.new_id();
                            erros.push(problema);
                        }
                    }
                }
                let len = dados.len();
                let total_reg = total;
                info!(
                    ok = true,
                    total = len,
                    erros = erros.len(),
                    total_reg,
                    "Operação concluída"
                );
                let mut resp = if erros.is_empty() {
                    Resposta::com_mensagem(id, dados, "OK")
                } else {
                    Resposta {
                        id,
                        ok: true,
                        status: 200,
                        mensagem: "OK".into(),
                        dados,
                        erros,
                        total: None,
                    }
                };
                resp.total = total_reg;
                Ok(resp)
            }
            Err(e) => {
                let e = self.normalize_error(e);
                error!(ok = false, error = %e.detail, "Operação falhou");
                Err(e)
            }
        }
    }

    pub(crate) fn normalize_error(&self, mut error: OpenMTRError) -> OpenMTRError {
        error.instance = self.ctx.new_id();
        error
    }
}
