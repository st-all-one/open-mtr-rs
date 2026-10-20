use std::sync::Arc;
use std::time::Duration;
use tracing::{Instrument, error, info, info_span};

use crate::openmtr::adapters::{ItemResult, LoteResult, MtrAdapter, MtrCtx};
use crate::openmtr::client::http::MtrHttpClient;
use crate::openmtr::config::Config;
use crate::openmtr::domain::{
    AceitarAlteracao, Acondicionamento, AlteracaoAceita, CancelarMtr, Cdf, CdfEmitido,
    ClasseResiduo, CriarMtr, CriarMtrComplementar, DownloadCdf, DownloadItem, EstadoFisico,
    FiltroCdf, FiltroMtr, GerarCdf, ModificarMtr, Mtr, MtrCancelado, MtrCriado, MtrModificado,
    MtrRecebido, ReceberMtr, ResiduoIbama, TratamentoResiduo, UnidadeMedida,
};
use crate::openmtr::error::OpenMTRError;
use crate::openmtr::resposta::Resposta;
use crate::openmtr::system::System;
use crate::openmtr::time_util::{Clock, IdGenerator};
use crate::openmtr::transport::HttpTransport;

pub type MtrResult<T> = Result<Resposta<T>, OpenMTRError>;

pub struct OpenMtrClient {
    ctx: Arc<MtrCtx>,
    adapter: Box<dyn MtrAdapter>,
}

impl OpenMtrClient {
    #[must_use]
    pub fn from_parts(ctx: Arc<MtrCtx>, adapter: Box<dyn MtrAdapter>) -> Self {
        Self { ctx, adapter }
    }

    #[must_use]
    pub fn builder() -> OpenMtrClientBuilder {
        OpenMtrClientBuilder::default()
    }

    #[must_use]
    pub fn system_name(&self) -> &'static str {
        self.ctx.config.system.nome()
    }
}

impl OpenMtrClient {
    async fn exec_lote<T>(
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

    fn normalize_error(&self, mut error: OpenMTRError) -> OpenMTRError {
        error.instance = self.ctx.new_id();
        error
    }

    pub async fn consultar_mtrs(&self, filtro: &FiltroMtr) -> MtrResult<Mtr> {
        self.exec_lote(
            "consultar_mtrs",
            self.adapter.consultar_mtrs(&self.ctx, filtro),
        )
        .await
    }

    pub async fn criar_mtrs(&self, mtrs: Vec<CriarMtr>) -> MtrResult<MtrCriado> {
        self.exec_lote("criar_mtrs", self.adapter.criar_mtrs(&self.ctx, mtrs))
            .await
    }

    pub async fn receber_mtrs(&self, lotes: Vec<ReceberMtr>) -> MtrResult<MtrRecebido> {
        self.exec_lote("receber_mtrs", self.adapter.receber_mtrs(&self.ctx, lotes))
            .await
    }

    pub async fn cancelar_mtrs(&self, reqs: Vec<CancelarMtr>) -> MtrResult<MtrCancelado> {
        self.exec_lote("cancelar_mtrs", self.adapter.cancelar_mtrs(&self.ctx, reqs))
            .await
    }

    pub async fn modificar_mtrs(&self, reqs: Vec<ModificarMtr>) -> MtrResult<MtrModificado> {
        self.exec_lote(
            "modificar_mtrs",
            self.adapter.modificar_mtrs(&self.ctx, reqs),
        )
        .await
    }

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

    pub async fn download_mtrs(&self, nums: Vec<String>) -> MtrResult<DownloadItem> {
        self.exec_lote("download_mtrs", self.adapter.download_mtrs(&self.ctx, nums))
            .await
    }

    pub async fn gerar_cdfs(&self, reqs: Vec<GerarCdf>) -> MtrResult<CdfEmitido> {
        self.exec_lote("gerar_cdfs", self.adapter.gerar_cdfs(&self.ctx, reqs))
            .await
    }

    pub async fn consultar_cdfs(&self, filtro: &FiltroCdf) -> MtrResult<Cdf> {
        self.exec_lote(
            "consultar_cdfs",
            self.adapter.consultar_cdfs(&self.ctx, filtro),
        )
        .await
    }

    pub async fn download_cdfs(&self, reqs: Vec<DownloadCdf>) -> MtrResult<DownloadItem> {
        self.exec_lote("download_cdfs", self.adapter.download_cdfs(&self.ctx, reqs))
            .await
    }

    pub async fn listar_residuos(&self) -> MtrResult<ResiduoIbama> {
        self.exec_lote("listar_residuos", self.adapter.listar_residuos(&self.ctx))
            .await
    }

    pub async fn listar_acondicionamentos(&self) -> MtrResult<Acondicionamento> {
        self.exec_lote(
            "listar_acondicionamentos",
            self.adapter.listar_acondicionamentos(&self.ctx),
        )
        .await
    }

    pub async fn listar_classes(&self) -> MtrResult<ClasseResiduo> {
        self.exec_lote("listar_classes", self.adapter.listar_classes(&self.ctx))
            .await
    }

    pub async fn listar_estados_fisicos(&self) -> MtrResult<EstadoFisico> {
        self.exec_lote(
            "listar_estados_fisicos",
            self.adapter.listar_estados_fisicos(&self.ctx),
        )
        .await
    }

    pub async fn listar_tratamentos(&self) -> MtrResult<TratamentoResiduo> {
        self.exec_lote(
            "listar_tratamentos",
            self.adapter.listar_tratamentos(&self.ctx),
        )
        .await
    }

    pub async fn listar_unidades(&self) -> MtrResult<UnidadeMedida> {
        self.exec_lote("listar_unidades", self.adapter.listar_unidades(&self.ctx))
            .await
    }

    // ─── SIGOR-specific ───

    pub async fn listar_classes_por_residuo(&self, codigo_ibama: &str) -> MtrResult<ClasseResiduo> {
        self.exec_lote(
            "listar_classes_por_residuo",
            self.adapter
                .listar_classes_por_residuo(&self.ctx, codigo_ibama),
        )
        .await
    }

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

#[derive(Default)]
pub struct OpenMtrClientBuilder {
    system: Option<System>,
    base_url: Option<String>,
    integration_token: Option<String>,
    timeout: Option<Duration>,
    max_rps: Option<u32>,
    clock: Option<Arc<dyn Clock>>,
    id_generator: Option<Arc<dyn IdGenerator>>,
}

impl OpenMtrClientBuilder {
    #[must_use]
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

    #[must_use]
    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout = Some(Duration::from_secs(secs));
        self
    }

    #[must_use]
    pub fn max_rps(mut self, rps: u32) -> Self {
        self.max_rps = Some(rps);
        self
    }

    /// Injeta um relógio determinístico (testes/golden snapshots).
    #[must_use]
    pub fn clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = Some(clock);
        self
    }

    /// Injeta um gerador determinístico de IDs (testes/golden snapshots).
    #[must_use]
    pub fn id_generator(mut self, id_generator: Arc<dyn IdGenerator>) -> Self {
        self.id_generator = Some(id_generator);
        self
    }

    #[cfg(feature = "native-http")]
    pub async fn build(self) -> Result<OpenMtrClient, OpenMTRError> {
        let transport = crate::openmtr::transport::reqwest::ReqwestTransport::new(
            self.timeout.unwrap_or(Duration::from_mins(1)),
        )?;
        self.build_with_transport(transport).await
    }

    pub async fn build_with_transport(
        self,
        transport: impl HttpTransport + 'static,
    ) -> Result<OpenMtrClient, OpenMTRError> {
        let system = self
            .system
            .ok_or_else(|| OpenMTRError::config("system é obrigatório — use .base(System::...)"))?;

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
            .timeout_secs(self.timeout.map_or(60, |d| d.as_secs()))
            .max_rps(self.max_rps.unwrap_or(10))
            .build()?;

        let http = MtrHttpClient::new(config.clone(), transport);

        let adapter = system.adapter()?;

        let clock = self
            .clock
            .unwrap_or_else(|| Arc::new(crate::openmtr::time_util::SystemClock));
        let id_generator = self
            .id_generator
            .unwrap_or_else(|| Arc::new(crate::openmtr::time_util::UuidV7Generator));
        let ctx = MtrCtx::with_determinism(http, config, clock, id_generator);

        adapter.autenticar_token(&ctx).await.map_err(|mut error| {
            error.instance = ctx.new_id();
            error
        })?;

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
