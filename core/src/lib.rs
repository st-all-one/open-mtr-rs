pub mod client;
pub mod core;
pub mod schemas;
pub mod services;
pub mod cache;

pub mod prelude {
    pub use crate::client::endpoint::{
        CancelarManifesto, DownloadPdf, Endpoint, ListarManifestos, PaginatedIter,
    };
    pub use crate::client::http::MtrClient;
    pub use crate::client::token::TokenManager;
    pub use crate::core::config::{AuthMethod, MtrConfig, MtrConfigBuilder};
    pub use crate::core::errors::{ErrorCategory, ErrorContext, MtrError};
    pub use crate::schemas::common::{DataMtr, ObjetoRespostaRest, Pagina, Paginacao, TamanhoPagina};
    pub use crate::schemas::manifesto::{
        ApiwsListaManifestoDTO, CancelaManifestoDTO, CancelamentoResponse, Estado,
        ManifestoResiduo, ManifestoResiduoVariacao, ManifestoResumo, Pais, ParceiroAcesso,
        ParceiroMTR, SituacaoManifesto, TipoParceiro,
    };
    pub use crate::schemas::recebimento::{
        AceiteAlteracaoDTO, AceiteAlteracaoResponse, ApiwsAlteraRecebimentoDTO,
        ApiwsManifestoDeRecebimentoDTO, RecebimentoLoteItem, ResiduoAlterado,
        ResiduoRecebido, TimestampMs,
    };
    pub use crate::schemas::cdf::{
        ApiwsCertificadoDestinacaoDTO, CDFEmissaoObjeto, CDFEmissaoResponse, FormatoDownload,
    };
    pub use crate::schemas::logistica_reversa::{
        ApiwsAlteraManifestoOrigemDTO, MtrLrDataDTO, MtrOrigemCompletoDTO, MtrOrigemCompletoObjeto,
        MtrProcessoCompletoDTO, RastreioEvento,
    };
    pub use crate::schemas::integracao_estadual::{
        ApiwsManifestoDTO, ErroIntegracao, ParteMTR, ResiduoIntegracao, RespostaIntegracaoObjeto,
    };
    pub use crate::schemas::tabelas::{
        ClasseResiduo, EstadoFisico, TipoAcondicionamento, TratamentoResiduo, UnidadeMedida,
        COMPATIBILIDADE_ACOND, ESTADOS_FISICOS,
    };
    pub use crate::schemas::cnpj::Cnpj;
    pub use crate::schemas::cpf::{Cpf, CpfCnpj};
    pub use crate::services::manifesto::{ManifestoListBuilder, ManifestoService};
    pub use crate::services::recebimento::RecebimentoBuilder;
    pub use crate::services::recebimento::RecebimentoService;
    pub use crate::services::cdf::CdfService;
    pub use crate::services::logistica_reversa::LogisticaReversaService;
    pub use crate::services::integracao_estadual::IntegracaoEstadualService;
    pub use crate::schemas::residuo::ResiduoIbama;
    pub use crate::schemas::salvar_manifesto::{
        ParteManifesto, ResiduoManifestoRequest, RespostaManifestoLote, RespostaResiduoLote,
        SalvarManifestoLoteRequest, SalvarManifestoResponse,
    };
    pub use crate::services::salvar_manifesto::{
        ManifestoLoteService, SalvarManifestoLoteBuilder,
    };
    pub use crate::services::residuo::ResiduoService;
    pub use crate::services::tabelas::TabelasService;
}
