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

#[derive(Debug, uniffi::Error)]
pub enum OpenMtrError {
    Api,
    Network,
    Timeout,
    Auth,
    Config,
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

pub struct OpenMtrClient {
    inner: open_mtr_core::OpenMtrClient,
}

macro_rules! blk {
    ($e:expr) => {
        runtime().block_on($e).map_err(map_error)?
    };
}

impl OpenMtrClient {
    pub fn init(
        system: String,
        token: String,
        base_url: Option<String>,
    ) -> Result<Self, OpenMtrError> {
        let sys: open_mtr_core::System =
            system.parse().map_err(|_: String| OpenMtrError::Config)?;
        let mut builder = open_mtr_core::OpenMtrClient::builder().base(sys).token(token);
        if let Some(url) = base_url {
            builder = builder.base_url(url);
        }
        let client = runtime()
            .block_on(builder.build())
            .map_err(map_error)?;
        Ok(Self { inner: client })
    }

    // ─── MTR ───────────────────────────────────────────────────────────

    pub fn consultar_mtrs(&self, filtro_json: String) -> Result<String, OpenMtrError> {
        let filtro: FiltroMtr = parse_json(&filtro_json)?;
        let resposta = blk!(self.inner.consultar_mtrs(&filtro));
        to_json(&resposta)
    }

    pub fn criar_mtrs(&self, mtrs_json: String) -> Result<String, OpenMtrError> {
        let mtrs: Vec<CriarMtr> = parse_json(&mtrs_json)?;
        let resposta = blk!(self.inner.criar_mtrs(mtrs));
        to_json(&resposta)
    }

    pub fn receber_mtrs(&self, lotes_json: String) -> Result<String, OpenMtrError> {
        let lotes: Vec<ReceberMtr> = parse_json(&lotes_json)?;
        let resposta = blk!(self.inner.receber_mtrs(lotes));
        to_json(&resposta)
    }

    pub fn cancelar_manifesto(
        &self,
        numero: String,
        justificativa: String,
    ) -> Result<String, OpenMtrError> {
        let reqs = vec![CancelarMtr { numero, justificativa }];
        let resposta = blk!(self.inner.cancelar_mtrs(reqs));
        to_json(&resposta)
    }

    pub fn modificar_mtrs(&self, mods_json: String) -> Result<String, OpenMtrError> {
        let mods: Vec<ModificarMtr> = parse_json(&mods_json)?;
        let resposta = blk!(self.inner.modificar_mtrs(mods));
        to_json(&resposta)
    }

    pub fn download_mtrs(&self, nums: Vec<String>) -> Result<String, OpenMtrError> {
        let resposta = blk!(self.inner.download_mtrs(nums));
        to_json(&resposta)
    }

    // ─── CDF ───────────────────────────────────────────────────────────

    pub fn gerar_cdfs(&self, reqs_json: String) -> Result<String, OpenMtrError> {
        let reqs: Vec<GerarCdf> = parse_json(&reqs_json)?;
        let resposta = blk!(self.inner.gerar_cdfs(reqs));
        to_json(&resposta)
    }

    pub fn consultar_cdfs(&self, filtro_json: String) -> Result<String, OpenMtrError> {
        let filtro: FiltroCdf = parse_json(&filtro_json)?;
        let resposta = blk!(self.inner.consultar_cdfs(&filtro));
        to_json(&resposta)
    }

    pub fn download_cdfs(&self, reqs_json: String) -> Result<String, OpenMtrError> {
        let reqs: Vec<DownloadCdf> = parse_json(&reqs_json)?;
        let resposta = blk!(self.inner.download_cdfs(reqs));
        to_json(&resposta)
    }

    // ─── Tabelas ───────────────────────────────────────────────────────

    pub fn listar_residuos(&self) -> Result<String, OpenMtrError> {
        let resposta = blk!(self.inner.listar_residuos());
        to_json(&resposta)
    }

    pub fn listar_acondicionamentos(&self) -> Result<String, OpenMtrError> {
        let resposta = blk!(self.inner.listar_acondicionamentos());
        to_json(&resposta)
    }

    pub fn listar_classes(&self) -> Result<String, OpenMtrError> {
        let resposta = blk!(self.inner.listar_classes());
        to_json(&resposta)
    }

    pub fn listar_estados_fisicos(&self) -> Result<String, OpenMtrError> {
        let resposta = blk!(self.inner.listar_estados_fisicos());
        to_json(&resposta)
    }

    pub fn listar_tratamentos(&self) -> Result<String, OpenMtrError> {
        let resposta = blk!(self.inner.listar_tratamentos());
        to_json(&resposta)
    }

    pub fn listar_unidades(&self) -> Result<String, OpenMtrError> {
        let resposta = blk!(self.inner.listar_unidades());
        to_json(&resposta)
    }

    // ─── Conveniência ──────────────────────────────────────────────────

    pub fn download_pdf(&self, numero: String) -> Result<Vec<u8>, OpenMtrError> {
        let mut resposta = blk!(self.inner.download_mtrs(vec![numero]));
        resposta.dados.pop().map(|item| item.buffer).ok_or(OpenMtrError::Api)
    }
}
