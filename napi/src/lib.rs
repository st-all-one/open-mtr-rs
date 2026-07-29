use mtr_sinir_core::prelude::*;
use napi::bindgen_prelude::*;
use napi_derive::napi;

fn parse_data(s: String) -> Result<DataMtr> {
    DataMtr::parse(&s).map_err(|e| napi::Error::from_reason(e))
}

fn parse_tipo(v: i32) -> Result<TipoParceiro> {
    TipoParceiro::try_from(v).map_err(|e| napi::Error::from_reason(e.to_string()))
}

fn map_err(e: MtrError) -> napi::Error {
    napi::Error::from_reason(e.to_string())
}

#[napi(object)]
pub struct ListaManifestoResponse {
    pub content: Vec<ManifestoResumoNapi>,
    pub total_pages: i32,
    pub total_elements: i64,
    pub page: i32,
    pub page_size: i32,
}

#[napi(object)]
pub struct ManifestoResumoNapi {
    pub man_numero: String,
    pub man_status: String,
    pub gerador: String,
}

#[napi(object)]
pub struct CancelamentoResponseNapi {
    pub man_numero: String,
    pub status: String,
    pub data_cancelamento: String,
}

#[napi(object)]
pub struct HealthResponseNapi {
    pub status: String,
    pub token_valido: bool,
}

#[napi]
pub struct MtrSinirClient {
    manifesto: ManifestoService,
    recebimento: RecebimentoService,
    cdf: CdfService,
    lr: LogisticaReversaService,
    integracao: IntegracaoEstadualService,
    tabelas: TabelasService,
    core: MtrClient,
}

#[napi]
impl MtrSinirClient {
    #[napi(constructor)]
    pub fn new(integration_token: String, base_url: String) -> Self {
        let config = MtrConfig::builder()
            .base_url(base_url)
            .integration_token(integration_token)
            .build()
            .expect("Config válida");
        let client = MtrClient::new(config).expect("Cliente válido");
        Self {
            manifesto: ManifestoService::new(client.clone()),
            recebimento: RecebimentoService::new(client.clone()),
            cdf: CdfService::new(client.clone()),
            lr: LogisticaReversaService::new(client.clone()),
            integracao: IntegracaoEstadualService::new(client.clone()),
            tabelas: TabelasService::new(client.clone()),
            core: client,
        }
    }

    #[napi]
    pub async fn listar_manifestos(
        &self,
        data_inicial: String,
        data_final: String,
        tipo_parceiro: i32,
        page: i32,
        size: i32,
    ) -> Result<Vec<ManifestoResumoNapi>> {
        let r = self
            .manifesto
            .listar(
                parse_data(data_inicial)?,
                parse_data(data_final)?,
                parse_tipo(tipo_parceiro)?,
                page,
                size,
            )
            .await
            .map_err(map_err)?;
        Ok(r.0.into_iter()
            .map(|m| ManifestoResumoNapi {
                man_numero: m.man_numero,
                man_status: m.man_status.unwrap_or_default(),
                gerador: m.gerador.unwrap_or_default(),
            })
            .collect())
    }

    #[napi]
    pub async fn cancelar_manifesto(
        &self,
        man_numero: String,
        justificativa: String,
    ) -> Result<CancelamentoResponseNapi> {
        let r = self
            .manifesto
            .cancelar(man_numero, justificativa)
            .await
            .map_err(map_err)?;
        Ok(CancelamentoResponseNapi {
            man_numero: r.man_numero,
            status: r.status,
            data_cancelamento: r.data_cancelamento,
        })
    }

    #[napi]
    pub async fn download_pdf(&self, man_numero: String) -> Result<Buffer> {
        let bytes = self
            .manifesto
            .download_pdf(&man_numero)
            .await
            .map_err(map_err)?;
        Ok(Buffer::from(bytes))
    }

    #[napi]
    pub async fn health(&self) -> Result<HealthResponseNapi> {
        let valido = self.core.token_valido().await;
        Ok(HealthResponseNapi {
            status: "ok".into(),
            token_valido: valido,
        })
    }
}
