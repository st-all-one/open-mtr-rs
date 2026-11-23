//! Bindings UniFFI da biblioteca Open MTR para Swift, Kotlin e Python.
//!
//! Expõe uma fachada síncra sobre o cliente assíncrono do `open-mtr-core`:
//! cada método bloqueia num runtime Tokio global e devolve o resultado como
//! JSON (texto), aproveitando a serialização já usada pelos adapters dos
//! sistemas SINIR, MTR-PE, SIGOR e estaduais. As anotações desta crate são
//! copiadas pelo UniFFI para as APIs geradas nas linguagens alvo.

#![allow(
    clippy::expect_used,
    clippy::wildcard_imports,
    reason = "binding UniFFI: runtime global é inicializado uma vez e o prelude é reexportado"
)]

uniffi::setup_scaffolding!();

use open_mtr_core::*;
use std::sync::OnceLock;
use tokio::runtime::Runtime;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

fn runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| Runtime::new().expect("Failed to create tokio runtime"))
}

fn map_error(e: OpenMTRError) -> OpenMtrError {
    let url = e.r#type.as_str();
    if url.contains("network") {
        OpenMtrError::Network
    } else if url.contains("timeout") {
        OpenMtrError::Timeout
    } else if url.contains("authentication") || url.contains("auth") {
        OpenMtrError::Auth
    } else if url.contains("configuration") || url.contains("config") {
        OpenMtrError::Config
    } else if url.contains("validation") {
        OpenMtrError::Validation
    } else {
        OpenMtrError::Api
    }
}

fn parse_json<T: serde::de::DeserializeOwned>(json: &str) -> Result<T, OpenMtrError> {
    serde_json::from_str(json).map_err(|_| OpenMtrError::Validation)
}

fn to_json<T: serde::Serialize>(val: &T) -> Result<String, OpenMtrError> {
    serde_json::to_string(val).map_err(|_| OpenMtrError::Api)
}

/// Erro tipado devolvido pelas chamadas da fachada UniFFI.
///
/// As variantes representam a classe do erro e são copiadas pelo UniFFI para
/// as exceções/erros gerados nas linguagens alvo; os detalhes originais de
/// `OpenMTRError` (título, status, `instance`) ficam no JSON de resposta dos
/// métodos que o retornam.
#[derive(Debug, uniffi::Error)]
pub enum OpenMtrError {
    /// Falha genérica da API de origem: resposta inesperada, erro HTTP não
    /// classificado ou falha ao serializar a resposta em JSON.
    Api,
    /// Falha de transporte/redes (conexão recusada, DNS, TLS).
    Network,
    /// Tempo limite excedido ao aguardar a API de origem.
    Timeout,
    /// Credenciais ausentes, inválidas ou expiradas (token/CPF e senha).
    Auth,
    /// Configuração inválida, como sistema desconhecido ou URL base ausente.
    Config,
    /// Dados de entrada inválidos (JSON desserializado em tipo de domínio).
    Validation,
}

impl std::fmt::Display for OpenMtrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api => write!(f, "Erro de API"),
            Self::Network => write!(f, "Erro de rede"),
            Self::Timeout => write!(f, "Tempo limite excedido"),
            Self::Auth => write!(f, "Erro de autenticação"),
            Self::Config => write!(f, "Erro de configuração"),
            Self::Validation => write!(f, "Erro de validação"),
        }
    }
}

impl std::error::Error for OpenMtrError {}

/// Obtém o token de autenticação clássica (CPF + senha) para sistemas que o
/// exigem (ex.: MTR-PE, SIGOR). Não depende de um cliente instanciado.
///
/// `params_json`: `{"tipo":"cpf_senha_unidade","cpf":"...","senha":"...","unidade":123,"cnpj":"..."}`
///
/// Retorna o token JWT diretamente.
pub fn obter_token(system: String, params_json: String) -> Result<String, OpenMtrError> {
    let sys: System = system.parse().map_err(|_: String| OpenMtrError::Config)?;
    let params: AuthParams = parse_json(&params_json)?;
    let token = runtime()
        .block_on(OpenMtrClassicAuth::new(sys).params(params).build())
        .map_err(map_error)?;
    Ok(token)
}

/// Cliente síncrono da fachada UniFFI para os sistemas de manifestos de
/// resíduos (SINIR, MTR-PE, SIGOR e estaduais).
///
/// Cada instância encapsula um `open-mtr-core::OpenMtrClient` autenticado;
/// os métodos bloqueiam num runtime Tokio global e devolvem o resultado como
/// JSON. Construa com [`OpenMtrClient::init`].
pub struct OpenMtrClient {
    inner: open_mtr_core::OpenMtrClient,
}

macro_rules! blk {
    ($e:expr) => {
        runtime().block_on($e).map_err(map_error)?
    };
}

impl OpenMtrClient {
    /// Cria um cliente autenticado para um sistema de manifestos.
    ///
    /// `system`: identificador do sistema (`sinir`, `sigor`, `mtrpe`, `mtrce`,
    /// `mtrsp`, `mtrrj`, `mtrmg` ou `mtrba`); `token`: access token JWT já
    /// emitido (sem o prefixo `Bearer`); `base_url`: sobrescreve a URL padrão
    /// do ambiente, ou `null` para usar o padrão do sistema.
    pub fn init(
        system: String,
        token: String,
        base_url: Option<String>,
    ) -> Result<Self, OpenMtrError> {
        let sys: System = system.parse().map_err(|_: String| OpenMtrError::Config)?;
        let mut builder = open_mtr_core::OpenMtrClient::builder()
            .base(sys)
            .token(token);
        if let Some(url) = base_url {
            builder = builder.base_url(url);
        }
        let client = runtime().block_on(builder.build()).map_err(map_error)?;
        Ok(Self { inner: client })
    }

    // ─── MTR ───────────────────────────────────────────────────────────

    /// Consulta manifestos de transporte de resíduos (MTRs) com filtros.
    ///
    /// `filtro_json`: objeto `FiltroMtr` em JSON (período, situação, etc.).
    /// Retorna o envelope `Resposta<Mtr>` em JSON, com os manifestos em
    /// `dados`, os erros por registro em `erros` e o total em `total`.
    pub fn consultar_mtrs(&self, filtro_json: String) -> Result<String, OpenMtrError> {
        let filtro: FiltroMtr = parse_json(&filtro_json)?;
        let resposta = blk!(self.inner.consultar_mtrs(&filtro));
        to_json(&resposta)
    }

    /// Cria um ou mais MTRs no sistema de origem.
    ///
    /// `mtrs_json`: array de objetos `CriarMtr` em JSON. Retorna o envelope
    /// `Resposta<MtrCriado>` em JSON, com número, código de barras e eventuais
    /// erros por manifesto.
    pub fn criar_mtrs(&self, mtrs_json: String) -> Result<String, OpenMtrError> {
        let mtrs: Vec<CriarMtr> = parse_json(&mtrs_json)?;
        let resposta = blk!(self.inner.criar_mtrs(mtrs));
        to_json(&resposta)
    }

    /// Registra o recebimento de lotes de MTRs.
    ///
    /// `lotes_json`: array de objetos `ReceberMtr` em JSON. Retorna o envelope
    /// `Resposta<MtrRecebido>` em JSON com a confirmação por manifesto.
    pub fn receber_mtrs(&self, lotes_json: String) -> Result<String, OpenMtrError> {
        let lotes: Vec<ReceberMtr> = parse_json(&lotes_json)?;
        let resposta = blk!(self.inner.receber_mtrs(lotes));
        to_json(&resposta)
    }

    /// Cancela um manifesto pelo número, com justificativa.
    ///
    /// Retorna o envelope `Resposta<MtrCancelado>` em JSON com o resultado do
    /// cancelamento e eventuais erros.
    pub fn cancelar_manifesto(
        &self,
        numero: String,
        justificativa: String,
    ) -> Result<String, OpenMtrError> {
        let reqs = vec![CancelarMtr {
            numero,
            justificativa,
        }];
        let resposta = blk!(self.inner.cancelar_mtrs(reqs));
        to_json(&resposta)
    }

    /// Modifica um ou mais MTRs existentes.
    ///
    /// `mods_json`: array de objetos `ModificarMtr` em JSON. Retorna o envelope
    /// `Resposta<MtrModificado>` em JSON com o resultado por manifesto.
    pub fn modificar_mtrs(&self, mods_json: String) -> Result<String, OpenMtrError> {
        let mods: Vec<ModificarMtr> = parse_json(&mods_json)?;
        let resposta = blk!(self.inner.modificar_mtrs(mods));
        to_json(&resposta)
    }

    /// Baixa os arquivos/DANFs de MTRs pelos seus números.
    ///
    /// `nums`: números dos manifestos. Retorna o envelope
    /// `Resposta<DownloadItem>` em JSON com os dados dos arquivos.
    pub fn download_mtrs(&self, nums: Vec<String>) -> Result<String, OpenMtrError> {
        let resposta = blk!(self.inner.download_mtrs(nums));
        to_json(&resposta)
    }

    // ─── CDF ───────────────────────────────────────────────────────────

    /// Emite CDFs (Certificados de Destinação Final de resíduos).
    ///
    /// `reqs_json`: array de objetos `GerarCdf` em JSON. Retorna o envelope
    /// `Resposta<CdfEmitido>` em JSON com os certificados gerados.
    pub fn gerar_cdfs(&self, reqs_json: String) -> Result<String, OpenMtrError> {
        let reqs: Vec<GerarCdf> = parse_json(&reqs_json)?;
        let resposta = blk!(self.inner.gerar_cdfs(reqs));
        to_json(&resposta)
    }

    /// Consulta CDFs com filtros.
    ///
    /// `filtro_json`: objeto `FiltroCdf` em JSON. Retorna o envelope
    /// `Resposta<Cdf>` em JSON, com os certificados em `dados` e o total para
    /// paginação em `total`.
    pub fn consultar_cdfs(&self, filtro_json: String) -> Result<String, OpenMtrError> {
        let filtro: FiltroCdf = parse_json(&filtro_json)?;
        let resposta = blk!(self.inner.consultar_cdfs(&filtro));
        to_json(&resposta)
    }

    /// Baixa os arquivos de CDFs pelas requisições informadas.
    ///
    /// `reqs_json`: array de objetos `DownloadCdf` em JSON. Retorna o envelope
    /// `Resposta<DownloadItem>` em JSON com os dados dos arquivos.
    pub fn download_cdfs(&self, reqs_json: String) -> Result<String, OpenMtrError> {
        let reqs: Vec<DownloadCdf> = parse_json(&reqs_json)?;
        let resposta = blk!(self.inner.download_cdfs(reqs));
        to_json(&resposta)
    }

    // ─── Tabelas ───────────────────────────────────────────────────────

    /// Lista a tabela de resíduos do IBAMA usada no preenchimento de MTRs.
    ///
    /// Retorna o envelope `Resposta<ResiduoIbama>` em JSON.
    pub fn listar_residuos(&self) -> Result<String, OpenMtrError> {
        let resposta = blk!(self.inner.listar_residuos());
        to_json(&resposta)
    }

    /// Lista a tabela de formas de acondicionamento de resíduos.
    ///
    /// Retorna o envelope `Resposta<Acondicionamento>` em JSON.
    pub fn listar_acondicionamentos(&self) -> Result<String, OpenMtrError> {
        let resposta = blk!(self.inner.listar_acondicionamentos());
        to_json(&resposta)
    }

    /// Lista a tabela de classes de resíduos.
    ///
    /// Retorna o envelope `Resposta<ClasseResiduo>` em JSON.
    pub fn listar_classes(&self) -> Result<String, OpenMtrError> {
        let resposta = blk!(self.inner.listar_classes());
        to_json(&resposta)
    }

    /// Lista a tabela de estados físicos dos resíduos (sólido, líquido, etc.).
    ///
    /// Retorna o envelope `Resposta<EstadoFisico>` em JSON.
    pub fn listar_estados_fisicos(&self) -> Result<String, OpenMtrError> {
        let resposta = blk!(self.inner.listar_estados_fisicos());
        to_json(&resposta)
    }

    /// Lista a tabela de tratamentos/destinação de resíduos.
    ///
    /// Retorna o envelope `Resposta<TratamentoResiduo>` em JSON.
    pub fn listar_tratamentos(&self) -> Result<String, OpenMtrError> {
        let resposta = blk!(self.inner.listar_tratamentos());
        to_json(&resposta)
    }

    /// Lista a tabela de unidades de medida de massa/volume.
    ///
    /// Retorna o envelope `Resposta<UnidadeMedida>` em JSON.
    pub fn listar_unidades(&self) -> Result<String, OpenMtrError> {
        let resposta = blk!(self.inner.listar_unidades());
        to_json(&resposta)
    }
}
