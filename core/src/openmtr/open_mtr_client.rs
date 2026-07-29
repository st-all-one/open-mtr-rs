use std::sync::Arc;
use std::time::Duration;
use tracing::{info, error, info_span};
use uuid::Uuid;

use crate::openmtr::domain::{
    Acondicionamento, CancelarMtr, Cdf, CdfEmitido, ClasseResiduo, CriarMtr, DownloadCdf,
    DownloadItem, EstadoFisico, FiltroCdf, FiltroMtr, GerarCdf,
    ModificarMtr, Mtr, MtrCancelado, MtrCriado, MtrModificado, MtrRecebido, ReceberMtr,
    ResiduoIbama, TratamentoResiduo, UnidadeMedida,
};
use crate::openmtr::error::OpenMTRError;
use crate::openmtr::resposta::{Problema, Resposta};
use crate::openmtr::system::System;
use crate::openmtr::config::Config;
use crate::openmtr::client::http::MtrHttpClient;
use crate::openmtr::adapters::{ItemResult, LoteResult, MtrAdapter, MtrCtx};

pub type MtrResult<T> = Result<Resposta<T>, OpenMTRError>;

pub struct OpenMtrClient {
    ctx: Arc<MtrCtx>,
    adapter: Box<dyn MtrAdapter>,
}

impl OpenMtrClient {
    pub fn builder() -> OpenMtrClientBuilder {
        OpenMtrClientBuilder::default()
    }
}

impl OpenMtrClient {
    async fn exec_lote<T>(
        &self,
        op: &str,
        future: impl std::future::Future<Output = Result<LoteResult<T>, OpenMTRError>>,
    ) -> MtrResult<T> {
        let id = Uuid::now_v7();
        let _span = info_span!("openmtr", operation = %op, instance = %id, system = %self.ctx.config.system.nome());

        match future.await {
            Ok(lote) => {
                let erros: Vec<Problema> = lote.erros().into_iter().cloned().collect();
                let dados: Vec<T> = lote.itens.into_iter().filter_map(|item| match item {
                    ItemResult::Ok(val) => Some(val),
                    ItemResult::Erro { .. } => None,
                }).collect();
                let len = dados.len();
                let total_reg = lote.total;
                info!(ok = true, total = len, erros = erros.len(), total_reg, "Operação concluída");
                let mut resp = if erros.is_empty() {
                    Resposta::com_mensagem(id, dados, "OK")
                } else {
                    Resposta { id, ok: true, status: 200, mensagem: "OK".into(), dados, erros, total: None }
                };
                resp.total = total_reg;
                Ok(resp)
            }
            Err(e) => {
                error!(ok = false, error = %e.detail, "Operação falhou");
                Err(e)
            }
        }
    }

    pub async fn consultar_mtrs(&self, filtro: &FiltroMtr) -> MtrResult<Mtr> {
        self.exec_lote("consultar_mtrs", self.adapter.consultar_mtrs(&self.ctx, filtro)).await
    }

    pub async fn criar_mtrs(&self, mtrs: Vec<CriarMtr>) -> MtrResult<MtrCriado> {
        self.exec_lote("criar_mtrs", self.adapter.criar_mtrs(&self.ctx, mtrs)).await
    }

    pub async fn receber_mtrs(&self, lotes: Vec<ReceberMtr>) -> MtrResult<MtrRecebido> {
        self.exec_lote("receber_mtrs", self.adapter.receber_mtrs(&self.ctx, lotes)).await
    }

    pub async fn cancelar_mtrs(&self, reqs: Vec<CancelarMtr>) -> MtrResult<MtrCancelado> {
        self.exec_lote("cancelar_mtrs", self.adapter.cancelar_mtrs(&self.ctx, reqs)).await
    }

    pub async fn modificar_mtrs(&self, reqs: Vec<ModificarMtr>) -> MtrResult<MtrModificado> {
        self.exec_lote("modificar_mtrs", self.adapter.modificar_mtrs(&self.ctx, reqs)).await
    }

    pub async fn download_mtrs(&self, nums: Vec<String>) -> MtrResult<DownloadItem> {
        self.exec_lote("download_mtrs", self.adapter.download_mtrs(&self.ctx, nums)).await
    }

    pub async fn gerar_cdfs(&self, reqs: Vec<GerarCdf>) -> MtrResult<CdfEmitido> {
        self.exec_lote("gerar_cdfs", self.adapter.gerar_cdfs(&self.ctx, reqs)).await
    }

    pub async fn consultar_cdfs(&self, filtro: &FiltroCdf) -> MtrResult<Cdf> {
        self.exec_lote("consultar_cdfs", self.adapter.consultar_cdfs(&self.ctx, filtro)).await
    }

    pub async fn download_cdfs(&self, reqs: Vec<DownloadCdf>) -> MtrResult<DownloadItem> {
        self.exec_lote("download_cdfs", self.adapter.download_cdfs(&self.ctx, reqs)).await
    }

    pub async fn listar_residuos(&self) -> MtrResult<ResiduoIbama> {
        self.exec_lote("listar_residuos", self.adapter.listar_residuos(&self.ctx)).await
    }

    pub async fn listar_acondicionamentos(&self) -> MtrResult<Acondicionamento> {
        self.exec_lote("listar_acondicionamentos", self.adapter.listar_acondicionamentos(&self.ctx)).await
    }

    pub async fn listar_classes(&self) -> MtrResult<ClasseResiduo> {
        self.exec_lote("listar_classes", self.adapter.listar_classes(&self.ctx)).await
    }

    pub async fn listar_estados_fisicos(&self) -> MtrResult<EstadoFisico> {
        self.exec_lote("listar_estados_fisicos", self.adapter.listar_estados_fisicos(&self.ctx)).await
    }

    pub async fn listar_tratamentos(&self) -> MtrResult<TratamentoResiduo> {
        self.exec_lote("listar_tratamentos", self.adapter.listar_tratamentos(&self.ctx)).await
    }

    pub async fn listar_unidades(&self) -> MtrResult<UnidadeMedida> {
        self.exec_lote("listar_unidades", self.adapter.listar_unidades(&self.ctx)).await
    }
}

#[derive(Default)]
pub struct OpenMtrClientBuilder {
    system: Option<System>,
    base_url: Option<String>,
    integration_token: Option<String>,
    timeout: Option<Duration>,
    max_rps: Option<u32>,
}

impl OpenMtrClientBuilder {
    pub fn base(mut self, system: System) -> Self {
        self.system = Some(system);
        self
    }

    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.integration_token = Some(token.into());
        self
    }

    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout = Some(Duration::from_secs(secs));
        self
    }

    pub fn max_rps(mut self, rps: u32) -> Self {
        self.max_rps = Some(rps);
        self
    }

    pub async fn build(self) -> Result<OpenMtrClient, OpenMTRError> {
        let system = self.system.ok_or_else(|| {
            OpenMTRError::config("system é obrigatório — use .base(System::...)")
        })?;

        let integration_token = self.integration_token.ok_or_else(|| {
            OpenMTRError::config("integration_token é obrigatório — use .token(...)")
        })?;

        let base_url = self
            .base_url
            .unwrap_or_else(|| system.default_base_url().to_string());

        let config = Config::builder()
            .system(system)
            .base_url(base_url)
            .token(integration_token.clone())
            .timeout_secs(self.timeout.map(|d| d.as_secs()).unwrap_or(60))
            .max_rps(self.max_rps.unwrap_or(10))
            .build()?;

        let http = MtrHttpClient::new(config.clone())?;

        let adapter = system.adapter()?;

        let ctx = MtrCtx { http, config };

        adapter.autenticar_token(&ctx).await?;

        info!(
            system = %adapter.nome(),
            "openMTR client initialized"
        );

        Ok(OpenMtrClient {
            ctx: Arc::new(ctx),
            adapter,
        })
    }
}
