use std::sync::Arc;

use async_trait::async_trait;

use crate::openmtr::client::http::{MtrHttpClient, RawResponse};
use crate::openmtr::config::Config;
use crate::openmtr::domain::{
    AceitarAlteracao, Acondicionamento, AlteracaoAceita, CancelarMtr, Cdf, CdfEmitido,
    ClasseResiduo, CriarMtr, CriarMtrComplementar, DownloadCdf, DownloadItem, EstadoFisico,
    FiltroCdf, FiltroMtr, GerarCdf, ModificarMtr, Mtr, MtrCancelado, MtrCriado, MtrModificado,
    MtrRecebido, ReceberMtr, ResiduoIbama, TratamentoResiduo, UnidadeMedida,
};
use crate::openmtr::error::OpenMTRError;
use crate::openmtr::open_mtr_auth::AuthParams;
use crate::openmtr::system::System;
use crate::openmtr::time_util::{Clock, IdGenerator, SystemClock, UuidV7Generator};
use serde_json::Value;

#[derive(Debug, Clone)]
#[allow(
    clippy::large_enum_variant,
    reason = "Ok/Erro são o contrato de resultado em lote; boxing esconderia o domínio"
)]
pub enum ItemResult<T> {
    Ok(T),
    Erro {
        indice: usize,
        problema: OpenMTRError,
    },
}

#[derive(Debug, Clone)]
pub struct LoteResult<T> {
    pub itens: Vec<ItemResult<T>>,
    /// Total de registros disponíveis (paginação)
    pub total: Option<i64>,
}

impl<T> LoteResult<T> {
    #[must_use]
    pub fn new(itens: Vec<ItemResult<T>>) -> Self {
        Self { itens, total: None }
    }

    #[must_use]
    pub fn com_total(itens: Vec<ItemResult<T>>, total: i64) -> Self {
        Self {
            itens,
            total: Some(total),
        }
    }

    #[must_use]
    pub fn sucessos(&self) -> Vec<&T> {
        self.itens
            .iter()
            .filter_map(|i| match i {
                ItemResult::Ok(val) => Some(val),
                ItemResult::Erro { .. } => None,
            })
            .collect()
    }

    #[must_use]
    pub fn erros(&self) -> Vec<&OpenMTRError> {
        self.itens
            .iter()
            .filter_map(|i| match i {
                ItemResult::Ok(_) => None,
                ItemResult::Erro { problema, .. } => Some(problema),
            })
            .collect()
    }

    #[must_use]
    pub fn total_ok(&self) -> usize {
        self.itens
            .iter()
            .filter(|item| matches!(item, ItemResult::Ok(_)))
            .count()
    }

    #[must_use]
    pub fn total_erros(&self) -> usize {
        self.itens
            .iter()
            .filter(|item| matches!(item, ItemResult::Erro { .. }))
            .count()
    }

    pub fn from_vec(dados: Vec<T>) -> Self {
        let itens = dados.into_iter().map(ItemResult::Ok).collect();
        Self { itens, total: None }
    }

    pub fn from_vec_com_total(dados: Vec<T>, total: i64) -> Self {
        let itens = dados.into_iter().map(ItemResult::Ok).collect();
        Self {
            itens,
            total: Some(total),
        }
    }
}

pub type HttpClient = MtrHttpClient;

pub struct MtrCtx {
    pub http: HttpClient,
    pub config: Config,
    pub clock: Arc<dyn Clock>,
    pub id_generator: Arc<dyn IdGenerator>,
}

impl MtrCtx {
    /// Cria um contexto com relógio e gerador de IDs padrão.
    #[must_use]
    pub fn new(http: HttpClient, config: Config) -> Self {
        Self {
            http,
            config,
            clock: Arc::new(SystemClock),
            id_generator: Arc::new(UuidV7Generator),
        }
    }

    /// Cria um contexto com relógio e gerador de IDs injetados.
    #[must_use]
    pub fn with_determinism(
        http: HttpClient,
        config: Config,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            http,
            config,
            clock,
            id_generator,
        }
    }

    #[must_use]
    pub fn now(&self) -> chrono::DateTime<chrono::Utc> {
        self.clock.now()
    }

    #[must_use]
    pub fn today(&self) -> chrono::NaiveDate {
        self.clock.today()
    }

    #[must_use]
    pub fn new_id(&self) -> uuid::Uuid {
        self.id_generator.new_id()
    }
}

#[async_trait]
pub trait MtrAdapter: Send + Sync {
    #[allow(
        dead_code,
        reason = "campo/helper mantido para compatibilidade da API e testes"
    )]
    fn system(&self) -> System;
    fn nome(&self) -> &'static str;

    async fn autenticar_token(&self, ctx: &MtrCtx) -> Result<String, OpenMTRError>;

    async fn autenticar_credenciais(
        &self,
        ctx: &MtrCtx,
        params: &AuthParams,
    ) -> Result<String, OpenMTRError>;

    async fn consultar_mtrs(
        &self,
        ctx: &MtrCtx,
        filtro: &FiltroMtr,
    ) -> Result<LoteResult<Mtr>, OpenMTRError>;

    async fn criar_mtrs(
        &self,
        ctx: &MtrCtx,
        mtrs: Vec<CriarMtr>,
    ) -> Result<LoteResult<MtrCriado>, OpenMTRError>;

    async fn receber_mtrs(
        &self,
        ctx: &MtrCtx,
        lotes: Vec<ReceberMtr>,
    ) -> Result<LoteResult<MtrRecebido>, OpenMTRError>;

    async fn cancelar_mtrs(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<CancelarMtr>,
    ) -> Result<LoteResult<MtrCancelado>, OpenMTRError>;

    async fn modificar_mtrs(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<ModificarMtr>,
    ) -> Result<LoteResult<MtrModificado>, OpenMTRError>;

    async fn aceitar_alteracoes(
        &self,
        ctx: &MtrCtx,
        _reqs: Vec<AceitarAlteracao>,
    ) -> Result<LoteResult<AlteracaoAceita>, OpenMTRError> {
        let _ = ctx;
        Err(OpenMTRError::not_supported("aceitar_alteracoes"))
    }

    async fn download_mtrs(
        &self,
        ctx: &MtrCtx,
        nums: Vec<String>,
    ) -> Result<LoteResult<DownloadItem>, OpenMTRError>;

    async fn gerar_cdfs(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<GerarCdf>,
    ) -> Result<LoteResult<CdfEmitido>, OpenMTRError>;

    async fn consultar_cdfs(
        &self,
        ctx: &MtrCtx,
        filtro: &FiltroCdf,
    ) -> Result<LoteResult<Cdf>, OpenMTRError>;

    async fn download_cdfs(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<DownloadCdf>,
    ) -> Result<LoteResult<DownloadItem>, OpenMTRError>;

    async fn listar_residuos(&self, ctx: &MtrCtx)
    -> Result<LoteResult<ResiduoIbama>, OpenMTRError>;

    async fn listar_acondicionamentos(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<Acondicionamento>, OpenMTRError>;

    async fn listar_classes(&self, ctx: &MtrCtx)
    -> Result<LoteResult<ClasseResiduo>, OpenMTRError>;

    async fn listar_estados_fisicos(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<EstadoFisico>, OpenMTRError>;

    async fn listar_tratamentos(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<TratamentoResiduo>, OpenMTRError>;

    async fn listar_unidades(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<UnidadeMedida>, OpenMTRError>;

    // ─── SIGOR-specific endpoints (with default fallbacks) ───

    async fn listar_classes_por_residuo(
        &self,
        _ctx: &MtrCtx,
        _codigo_ibama: &str,
    ) -> Result<LoteResult<ClasseResiduo>, OpenMTRError> {
        Err(OpenMTRError::not_supported("listar_classes_por_residuo"))
    }

    async fn listar_acondicionamentos_por_estado_fisico(
        &self,
        _ctx: &MtrCtx,
        _codigo_estado: u32,
    ) -> Result<LoteResult<Acondicionamento>, OpenMTRError> {
        Err(OpenMTRError::not_supported(
            "listar_acondicionamentos_por_estado_fisico",
        ))
    }

    async fn consultar_mtr_por_codigo_referencia(
        &self,
        _ctx: &MtrCtx,
        _unidade: u32,
        _codigo: &str,
    ) -> Result<Option<Mtr>, OpenMTRError> {
        Err(OpenMTRError::not_supported(
            "consultar_mtr_por_codigo_referencia",
        ))
    }

    async fn criar_manifesto_complementar(
        &self,
        _ctx: &MtrCtx,
        _req: CriarMtrComplementar,
    ) -> Result<MtrCriado, OpenMTRError> {
        Err(OpenMTRError::not_supported("criar_manifesto_complementar"))
    }
}

pub(crate) fn validate_download(response: &RawResponse, expected: &str) -> Option<OpenMTRError> {
    if response.bytes.is_empty() {
        return Some(OpenMTRError::validation("Download retornou arquivo vazio"));
    }

    if response.bytes.len() < 100 {
        return Some(OpenMTRError::validation(format!(
            "Arquivo muito pequeno ({} bytes) para ser um {expected} válido",
            response.bytes.len()
        )));
    }

    if let Err(msg) = response.validate(expected) {
        return Some(OpenMTRError::validation(msg));
    }

    None
}

#[allow(
    dead_code,
    reason = "campo/helper mantido para compatibilidade da API e testes"
)]
pub(crate) async fn get_value(
    ctx: &MtrCtx,
    path: &str,
    label: &str,
) -> Result<Value, OpenMTRError> {
    ctx.http.get::<Value>(path, label).await
}

#[allow(
    dead_code,
    reason = "campo/helper mantido para compatibilidade da API e testes"
)]
pub(crate) async fn post_value(
    ctx: &MtrCtx,
    path: &str,
    body: &(impl serde::Serialize + Sync),
    label: &str,
) -> Result<Value, OpenMTRError> {
    ctx.http.post::<Value, _>(path, body, label).await
}

pub(crate) mod mtr_pe;
pub(crate) mod sigor;
pub(crate) mod sinir;
