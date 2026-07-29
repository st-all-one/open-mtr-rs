use crate::client::http::MtrClient;
use crate::core::errors::{ErrorCategory, ErrorContext, MtrError};
use crate::schemas::common::ObjetoRespostaRest;
use crate::schemas::tabelas::{
    ClasseResiduo, EstadoFisico, TipoAcondicionamento, TratamentoResiduo, UnidadeMedida,
    COMPATIBILIDADE_ACOND, ESTADOS_FISICOS,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

struct CacheEntry<T> {
    data: T,
    timestamp: Instant,
}

impl<T> CacheEntry<T> {
    fn is_fresh(&self, ttl: std::time::Duration) -> bool {
        self.timestamp.elapsed() < ttl
    }
}

pub struct TabelasService {
    client: MtrClient,
    cache_all: Arc<RwLock<Option<CacheEntry<Vec<TipoAcondicionamento>>>>>,
    cache_estado: Arc<RwLock<HashMap<i32, CacheEntry<Vec<TipoAcondicionamento>>>>>,
}

impl TabelasService {
    const TTL: std::time::Duration = std::time::Duration::from_secs(86_400);

    #[must_use]
    pub fn new(client: MtrClient) -> Self {
        Self {
            client,
            cache_all: Arc::new(RwLock::new(None)),
            cache_estado: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn listar_todos(&self) -> Result<Vec<TipoAcondicionamento>, MtrError> {
        {
            let cache = self.cache_all.read().await;
            if let Some(entry) = cache.as_ref() {
                if entry.is_fresh(Self::TTL) {
                    return Ok(entry.data.clone());
                }
            }
        }

        let raw: ObjetoRespostaRest<Vec<TipoAcondicionamento>> = self
            .client
            .request(
                reqwest::Method::GET,
                "/retornaListaAcondicionamento",
                "tabelas.listar_todos",
            )
            .await?;

        let data = raw.objeto_resposta.ok_or_else(|| {
            MtrError::api(
                ErrorCategory::InternalError,
                500,
                "Lista de acondicionamentos vazia",
                ErrorContext::new("tabelas.listar_todos"),
            )
        })?;

        let mut cache = self.cache_all.write().await;
        *cache = Some(CacheEntry {
            data: data.clone(),
            timestamp: Instant::now(),
        });

        Ok(data)
    }

    pub async fn listar_por_estado_fisico(
        &self,
        tie_codigo: i32,
    ) -> Result<Vec<TipoAcondicionamento>, MtrError> {
        {
            let cache = self.cache_estado.read().await;
            if let Some(entry) = cache.get(&tie_codigo) {
                if entry.is_fresh(Self::TTL) {
                    return Ok(entry.data.clone());
                }
            }
        }

        let path = format!("/retornaListaAcondicionamentoPorEstadoFisico/{tie_codigo}");

        let raw: ObjetoRespostaRest<Vec<TipoAcondicionamento>> = self
            .client
            .request(reqwest::Method::GET, &path, "tabelas.listar_por_estado")
            .await?;

        let data = raw.objeto_resposta.ok_or_else(|| {
            MtrError::api(
                ErrorCategory::InternalError,
                500,
                "Lista por estado físico vazia",
                ErrorContext::new("tabelas.listar_por_estado"),
            )
        })?;

        let mut cache = self.cache_estado.write().await;
        cache.insert(
            tie_codigo,
            CacheEntry {
                data: data.clone(),
                timestamp: Instant::now(),
            },
        );

        Ok(data)
    }

    #[must_use]
    pub fn get_compativeis(tie_codigo: i32) -> &'static [i32] {
        COMPATIBILIDADE_ACOND
            .get(&tie_codigo)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    #[must_use]
    pub fn get_nome_estado_fisico(tie_codigo: i32) -> &'static str {
        ESTADOS_FISICOS
            .get(&tie_codigo)
            .copied()
            .unwrap_or("Desconhecido")
    }

    pub async fn listar_unidades(&self) -> Result<Vec<UnidadeMedida>, MtrError> {
        let raw: ObjetoRespostaRest<Vec<UnidadeMedida>> = self
            .client
            .request(reqwest::Method::GET, "/retornaListaUnidade", "tabelas.unidades")
            .await?;
        raw.objeto_resposta.ok_or_else(|| {
            MtrError::api(ErrorCategory::InternalError, 500, "Lista de unidades vazia", ErrorContext::new("tabelas.unidades"))
        })
    }

    pub async fn listar_estados_fisicos(&self) -> Result<Vec<EstadoFisico>, MtrError> {
        let raw: ObjetoRespostaRest<Vec<EstadoFisico>> = self
            .client
            .request(reqwest::Method::GET, "/retornaListaEstadoFisico", "tabelas.estados_fisicos")
            .await?;
        raw.objeto_resposta.ok_or_else(|| {
            MtrError::api(ErrorCategory::InternalError, 500, "Lista de estados físicos vazia", ErrorContext::new("tabelas.estados_fisicos"))
        })
    }

    pub async fn listar_classes(&self) -> Result<Vec<ClasseResiduo>, MtrError> {
        let raw: ObjetoRespostaRest<Vec<ClasseResiduo>> = self
            .client
            .request(reqwest::Method::GET, "/retornaListaClasse", "tabelas.classes")
            .await?;
        raw.objeto_resposta.ok_or_else(|| {
            MtrError::api(ErrorCategory::InternalError, 500, "Lista de classes vazia", ErrorContext::new("tabelas.classes"))
        })
    }

    pub async fn listar_tratamentos(&self) -> Result<Vec<TratamentoResiduo>, MtrError> {
        let raw: ObjetoRespostaRest<Vec<TratamentoResiduo>> = self
            .client
            .request(reqwest::Method::GET, "/retornaListaTratamento", "tabelas.tratamentos")
            .await?;
        raw.objeto_resposta.ok_or_else(|| {
            MtrError::api(ErrorCategory::InternalError, 500, "Lista de tratamentos vazia", ErrorContext::new("tabelas.tratamentos"))
        })
    }

    pub async fn invalidate_cache(&self) {
        let mut all = self.cache_all.write().await;
        *all = None;
        let mut estado = self.cache_estado.write().await;
        estado.clear();
    }
}

pub async fn validar_acondicionamento(
    service: &TabelasService,
    tie_codigo: i32,
    acond_codigo: i32,
) -> bool {
    match service.listar_por_estado_fisico(tie_codigo).await {
        Ok(permitidos) => permitidos.iter().any(|a| a.codigo == acond_codigo),
        Err(_) => TabelasService::get_compativeis(tie_codigo).contains(&acond_codigo),
    }
}
