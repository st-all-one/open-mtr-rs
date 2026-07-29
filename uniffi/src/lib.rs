uniffi::setup_scaffolding!();

use mtr_sinir_core::prelude::*;
use tokio::runtime::Runtime;

static RT: std::sync::OnceLock<Runtime> = std::sync::OnceLock::new();

fn rt() -> &'static Runtime {
    RT.get_or_init(|| Runtime::new().expect("tokio"))
}

pub struct MtrSinirClient {
    manifesto: ManifestoService,
}

impl MtrSinirClient {
    pub fn init(integration_token: String, base_url: String) -> Self {
        let config = MtrConfig::builder()
            .base_url(base_url)
            .integration_token(integration_token)
            .build()
            .expect("Config válida");
        let client = MtrClient::new(config).expect("Cliente válido");
        Self {
            manifesto: ManifestoService::new(client),
        }
    }

    pub fn listar_manifestos(
        &self,
        data_inicial: String,
        data_final: String,
        tipo_parceiro: i32,
        page: i32,
        size: i32,
    ) -> Result<Vec<ManifestoResumo>, MtrError> {
        rt().block_on(async {
            let r = self.manifesto.listar(
                DataMtr::parse(&data_inicial)
                    .map_err(|e| MtrError::validation(e))?,
                DataMtr::parse(&data_final)
                    .map_err(|e| MtrError::validation(e))?,
                TipoParceiro::try_from(tipo_parceiro)
                    .map_err(|e| MtrError::validation(e.to_string()))?,
                page,
                size,
            ).await?;
            Ok(r.0.into_iter().map(|m| ManifestoResumo {
                man_numero: m.man_numero,
                man_status: m.man_status.unwrap_or_default(),
                gerador: m.gerador.unwrap_or_default(),
            }).collect())
        })
    }

    pub fn cancelar_manifesto(
        &self,
        man_numero: String,
        justificativa: String,
    ) -> Result<CancelamentoResponse, MtrError> {
        rt().block_on(async {
            let r = self.manifesto.cancelar(man_numero, justificativa).await?;
            Ok(CancelamentoResponse {
                man_numero: r.man_numero,
                status: r.status,
                data_cancelamento: r.data_cancelamento,
            })
        })
    }

    pub fn download_pdf(&self, man_numero: String) -> Result<Vec<u8>, MtrError> {
        rt().block_on(self.manifesto.download_pdf(&man_numero))
    }
}

pub struct ListaManifestoResponse {
    pub content: Vec<ManifestoResumo>,
    pub total_pages: i32,
    pub total_elements: i64,
    pub page: i32,
    pub page_size: i32,
}

pub struct ManifestoResumo {
    pub man_numero: String,
    pub man_status: String,
    pub gerador: String,
}

pub struct CancelamentoResponse {
    pub man_numero: String,
    pub status: String,
    pub data_cancelamento: String,
}
