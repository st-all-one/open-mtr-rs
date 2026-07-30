use async_trait::async_trait;

use crate::openmtr::domain::{
    Acondicionamento, CancelarMtr, Cdf, CdfEmitido, ClasseResiduo, CriarMtr, CriarMtrComplementar,
    DownloadCdf, DownloadItem, EstadoFisico, FiltroCdf, FiltroMtr, GerarCdf,
    ModificarMtr, Mtr, MtrCancelado, MtrCriado, MtrModificado, MtrRecebido, ReceberMtr,
    ResiduoIbama, TratamentoResiduo, UnidadeMedida,
};
use crate::openmtr::error::OpenMTRError;
use crate::openmtr::config::Config;
use crate::openmtr::client::http::{MtrHttpClient, RawResponse};
use crate::openmtr::resposta::Problema;
use crate::openmtr::system::System;

#[derive(Debug, Clone)]
pub enum ItemResult<T> {
    Ok(T),
    Erro { indice: usize, problema: Problema },
}

#[derive(Debug, Clone)]
pub struct LoteResult<T> {
    pub itens: Vec<ItemResult<T>>,
    /// Total de registros disponíveis (paginação)
    pub total: Option<i64>,
}

impl<T> LoteResult<T> {
    pub fn new(itens: Vec<ItemResult<T>>) -> Self {
        Self { itens, total: None }
    }

    pub fn com_total(itens: Vec<ItemResult<T>>, total: i64) -> Self {
        Self { itens, total: Some(total) }
    }

    pub fn sucessos(&self) -> Vec<&T> {
        self.itens.iter().filter_map(|i| match i {
            ItemResult::Ok(val) => Some(val),
            ItemResult::Erro { .. } => None,
        }).collect()
    }

    pub fn erros(&self) -> Vec<&Problema> {
        self.itens.iter().filter_map(|i| match i {
            ItemResult::Ok(_) => None,
            ItemResult::Erro { problema, .. } => Some(problema),
        }).collect()
    }

    pub fn total_ok(&self) -> usize {
        self.sucessos().len()
    }

    pub fn total_erros(&self) -> usize {
        self.erros().len()
    }

    pub fn from_vec(dados: Vec<T>) -> Self {
        let itens = dados.into_iter().map(ItemResult::Ok).collect();
        Self { itens, total: None }
    }

    pub fn from_vec_com_total(dados: Vec<T>, total: i64) -> Self {
        let itens = dados.into_iter().map(ItemResult::Ok).collect();
        Self { itens, total: Some(total) }
    }
}

pub(crate) struct MtrCtx {
    pub http: MtrHttpClient,
    pub config: Config,
}

#[async_trait]
pub(crate) trait MtrAdapter: Send + Sync {
    #[allow(dead_code)]
    fn system(&self) -> System;
    fn nome(&self) -> &'static str;

    async fn autenticar_token(&self, ctx: &MtrCtx) -> Result<String, OpenMTRError>;

    async fn autenticar_credenciais(
        &self,
        ctx: &MtrCtx,
        cpf: &str,
        senha: &str,
        unidade: u32,
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

    async fn listar_residuos(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<ResiduoIbama>, OpenMTRError>;

    async fn listar_acondicionamentos(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<Acondicionamento>, OpenMTRError>;

    async fn listar_classes(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<ClasseResiduo>, OpenMTRError>;

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
        Err(OpenMTRError::not_supported("listar_acondicionamentos_por_estado_fisico"))
    }

    async fn consultar_mtr_por_codigo_referencia(
        &self,
        _ctx: &MtrCtx,
        _unidade: u32,
        _codigo: &str,
    ) -> Result<Option<Mtr>, OpenMTRError> {
        Err(OpenMTRError::not_supported("consultar_mtr_por_codigo_referencia"))
    }

    async fn criar_manifesto_complementar(
        &self,
        _ctx: &MtrCtx,
        _req: CriarMtrComplementar,
    ) -> Result<MtrCriado, OpenMTRError> {
        Err(OpenMTRError::not_supported("criar_manifesto_complementar"))
    }
}

pub(crate) fn validate_download(response: &RawResponse, expected: &str) -> Option<Problema> {
    if response.bytes.is_empty() {
        return Some(Problema::validation("Download retornou arquivo vazio"));
    }

    if response.bytes.len() < 100 {
        return Some(Problema::validation(format!(
            "Arquivo muito pequeno ({} bytes) para ser um {expected} válido",
            response.bytes.len()
        )));
    }

    if let Err(msg) = response.validate(expected) {
        return Some(Problema::validation(msg));
    }

    None
}

pub(crate) mod sinir;
pub(crate) mod mtr_pe;
pub(crate) mod sigor;
