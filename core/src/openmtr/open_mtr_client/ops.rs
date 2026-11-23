use crate::openmtr::domain::{
    AceitarAlteracao, Acondicionamento, AlteracaoAceita, CancelarMtr, Cdf, CdfEmitido,
    ClasseResiduo, CriarMtr, CriarMtrComplementar, DownloadCdf, DownloadItem, EstadoFisico,
    FiltroCdf, FiltroMtr, GerarCdf, ModificarMtr, Mtr, MtrCancelado, MtrCriado, MtrModificado,
    MtrRecebido, ReceberMtr, ResiduoIbama, TratamentoResiduo, UnidadeMedida,
};
use crate::openmtr::error::OpenMTRError;
use crate::openmtr::resposta::Resposta;

use super::{MtrResult, OpenMtrClient};

impl OpenMtrClient {
    /// Consulta manifestos segundo um filtro (período, situação, partes).
    pub async fn consultar_mtrs(&self, filtro: &FiltroMtr) -> MtrResult<Mtr> {
        self.exec_lote(
            "consultar_mtrs",
            self.adapter.consultar_mtrs(&self.ctx, filtro),
        )
        .await
    }

    /// Cria vários manifestos em lote; devolve os registros criados.
    pub async fn criar_mtrs(&self, mtrs: Vec<CriarMtr>) -> MtrResult<MtrCriado> {
        self.exec_lote("criar_mtrs", self.adapter.criar_mtrs(&self.ctx, mtrs))
            .await
    }

    /// Registra o recebimento de manifestos por unidade.
    pub async fn receber_mtrs(&self, lotes: Vec<ReceberMtr>) -> MtrResult<MtrRecebido> {
        self.exec_lote("receber_mtrs", self.adapter.receber_mtrs(&self.ctx, lotes))
            .await
    }

    /// Cancela manifestos existentes.
    pub async fn cancelar_mtrs(&self, reqs: Vec<CancelarMtr>) -> MtrResult<MtrCancelado> {
        self.exec_lote("cancelar_mtrs", self.adapter.cancelar_mtrs(&self.ctx, reqs))
            .await
    }

    /// Modifica manifestos existentes (por exemplo, dados de resíduo ou transporte).
    pub async fn modificar_mtrs(&self, reqs: Vec<ModificarMtr>) -> MtrResult<MtrModificado> {
        self.exec_lote(
            "modificar_mtrs",
            self.adapter.modificar_mtrs(&self.ctx, reqs),
        )
        .await
    }

    /// Aceita alterações propostas em manifestos.
    pub async fn aceitar_alteracoes(
        &self,
        reqs: Vec<AceitarAlteracao>,
    ) -> MtrResult<AlteracaoAceita> {
        self.exec_lote(
            "aceitar_alteracoes",
            self.adapter.aceitar_alteracoes(&self.ctx, reqs),
        )
        .await
    }

    /// Baixa os arquivos (CDF/XML) dos manifestos pelos números informados.
    pub async fn download_mtrs(&self, nums: Vec<String>) -> MtrResult<DownloadItem> {
        self.exec_lote("download_mtrs", self.adapter.download_mtrs(&self.ctx, nums))
            .await
    }

    /// Emite CDFs (Comprovantes de Destinação Final) para os manifestos informados.
    pub async fn gerar_cdfs(&self, reqs: Vec<GerarCdf>) -> MtrResult<CdfEmitido> {
        self.exec_lote("gerar_cdfs", self.adapter.gerar_cdfs(&self.ctx, reqs))
            .await
    }

    /// Consulta CDFs segundo um filtro.
    pub async fn consultar_cdfs(&self, filtro: &FiltroCdf) -> MtrResult<Cdf> {
        self.exec_lote(
            "consultar_cdfs",
            self.adapter.consultar_cdfs(&self.ctx, filtro),
        )
        .await
    }

    /// Baixa os arquivos dos CDFs informados.
    pub async fn download_cdfs(&self, reqs: Vec<DownloadCdf>) -> MtrResult<DownloadItem> {
        self.exec_lote("download_cdfs", self.adapter.download_cdfs(&self.ctx, reqs))
            .await
    }

    /// Lista os resíduos cadastrados com código IBAMA.
    pub async fn listar_residuos(&self) -> MtrResult<ResiduoIbama> {
        self.exec_lote("listar_residuos", self.adapter.listar_residuos(&self.ctx))
            .await
    }

    /// Lista as formas de acondicionamento válidas.
    pub async fn listar_acondicionamentos(&self) -> MtrResult<Acondicionamento> {
        self.exec_lote(
            "listar_acondicionamentos",
            self.adapter.listar_acondicionamentos(&self.ctx),
        )
        .await
    }

    /// Lista as classes de resíduo válidas.
    pub async fn listar_classes(&self) -> MtrResult<ClasseResiduo> {
        self.exec_lote("listar_classes", self.adapter.listar_classes(&self.ctx))
            .await
    }

    /// Lista os estados físicos dos resíduos (sólido, líquido etc.).
    pub async fn listar_estados_fisicos(&self) -> MtrResult<EstadoFisico> {
        self.exec_lote(
            "listar_estados_fisicos",
            self.adapter.listar_estados_fisicos(&self.ctx),
        )
        .await
    }

    /// Lista as formas de tratamento admitidas.
    pub async fn listar_tratamentos(&self) -> MtrResult<TratamentoResiduo> {
        self.exec_lote(
            "listar_tratamentos",
            self.adapter.listar_tratamentos(&self.ctx),
        )
        .await
    }

    /// Lista as unidades de medida válidas.
    pub async fn listar_unidades(&self) -> MtrResult<UnidadeMedida> {
        self.exec_lote("listar_unidades", self.adapter.listar_unidades(&self.ctx))
            .await
    }

    // ─── SIGOR-specific ───

    /// Lista as classes compatíveis com um resíduo, pelo código IBAMA (SIGOR).
    pub async fn listar_classes_por_residuo(&self, codigo_ibama: &str) -> MtrResult<ClasseResiduo> {
        self.exec_lote(
            "listar_classes_por_residuo",
            self.adapter
                .listar_classes_por_residuo(&self.ctx, codigo_ibama),
        )
        .await
    }

    /// Lista acondicionamentos compatíveis com um estado físico, pelo código (SIGOR).
    pub async fn listar_acondicionamentos_por_estado_fisico(
        &self,
        codigo_estado: u32,
    ) -> MtrResult<Acondicionamento> {
        self.exec_lote(
            "listar_acondicionamentos_por_estado_fisico",
            self.adapter
                .listar_acondicionamentos_por_estado_fisico(&self.ctx, codigo_estado),
        )
        .await
    }

    /// Busca um manifesto pelo código de referência e unidade (SIGOR).
    ///
    /// Retorna `Ok(None)` quando não há manifesto com esse código.
    pub async fn consultar_mtr_por_codigo_referencia(
        &self,
        unidade: u32,
        codigo: &str,
    ) -> Result<Option<Mtr>, OpenMTRError> {
        self.adapter
            .consultar_mtr_por_codigo_referencia(&self.ctx, unidade, codigo)
            .await
            .map_err(|error| self.normalize_error(error))
    }

    /// Cria um manifesto complementar; falha se o sistema responder `sucesso = false`.
    pub async fn criar_manifesto_complementar(
        &self,
        req: CriarMtrComplementar,
    ) -> MtrResult<MtrCriado> {
        let criado = self
            .adapter
            .criar_manifesto_complementar(&self.ctx, req)
            .await
            .map_err(|error| self.normalize_error(error))?;

        if !criado.sucesso {
            return Err(self.normalize_error(OpenMTRError::validation(criado.mensagem)));
        }

        let id = self.ctx.new_id();
        Ok(Resposta {
            id,
            ok: true,
            status: 200,
            mensagem: "OK".into(),
            dados: vec![criado],
            erros: Vec::new(),
            total: None,
        })
    }
}
