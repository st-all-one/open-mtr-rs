use std::marker::PhantomData;

use crate::client::http::MtrClient;
use crate::client::retry::with_retry;
use crate::core::errors::{ErrorCategory, ErrorContext, MtrError};
use crate::schemas::common::{DataMtr, ObjetoRespostaRest, Pagina, TamanhoPagina};
use crate::schemas::manifesto::{
    ApiwsListaManifestoDTO, CancelaManifestoDTO, CancelamentoResponse, ManifestoResumo,
    TipoParceiro,
};

pub struct StFaltando;
pub struct StPronto;

#[must_use]
pub struct ManifestoListBuilder<State = StFaltando> {
    data_inicial: Option<DataMtr>,
    data_final: Option<DataMtr>,
    tipo_parceiro: Option<TipoParceiro>,
    pagina: Option<Pagina>,
    tamanho_pagina: Option<TamanhoPagina>,
    _state: PhantomData<State>,
}

impl ManifestoListBuilder<StFaltando> {
    pub fn new() -> Self {
        Self {
            data_inicial: None,
            data_final: None,
            tipo_parceiro: None,
            pagina: None,
            tamanho_pagina: None,
            _state: PhantomData,
        }
    }

    pub fn periodo(
        self,
        inicio: DataMtr,
        fim: DataMtr,
    ) -> Result<ManifestoListBuilder<StPronto>, MtrError> {
        if fim.0 < inicio.0 {
            return Err(MtrError::validation(
                "data final deve ser maior ou igual à data inicial",
            ));
        }
        Ok(ManifestoListBuilder {
            data_inicial: Some(inicio),
            data_final: Some(fim),
            tipo_parceiro: self.tipo_parceiro,
            pagina: self.pagina,
            tamanho_pagina: self.tamanho_pagina,
            _state: PhantomData,
        })
    }
}

impl<State> ManifestoListBuilder<State> {
    pub fn tipo_parceiro(mut self, t: TipoParceiro) -> Self {
        self.tipo_parceiro = Some(t);
        self
    }

    pub fn pagina(mut self, p: Pagina) -> Self {
        self.pagina = Some(p);
        self
    }

    pub fn tamanho_pagina(mut self, s: TamanhoPagina) -> Self {
        self.tamanho_pagina = Some(s);
        self
    }

    pub fn build(self) -> Result<ApiwsListaManifestoDTO, MtrError> {
        let data_inicial = self
            .data_inicial
            .ok_or_else(|| MtrError::validation("data_inicial é obrigatória"))?;
        let data_final = self
            .data_final
            .ok_or_else(|| MtrError::validation("data_final é obrigatória"))?;

        Ok(ApiwsListaManifestoDTO {
            data_inicial,
            data_final,
            tipo_parceiro: self.tipo_parceiro.unwrap_or(TipoParceiro::Transportador),
            pagina: self.pagina.map(|p| p.get()),
            tamanho_pagina: self.tamanho_pagina.map(|s| s.get()),
        })
    }
}

#[derive(Clone)]
pub struct ManifestoService {
    client: MtrClient,
}

impl ManifestoService {
    #[must_use]
    pub fn new(client: MtrClient) -> Self {
        Self { client }
    }

    pub async fn listar(
        &self,
        data_inicial: DataMtr,
        data_final: DataMtr,
        tipo_parceiro: TipoParceiro,
        page: i32,
        size: i32,
    ) -> Result<(Vec<ManifestoResumo>, Option<i64>), MtrError> {
        let filtro = ApiwsListaManifestoDTO {
            data_inicial,
            data_final,
            tipo_parceiro,
            pagina: Some(page),
            tamanho_pagina: Some(size),
        };

        let raw: ObjetoRespostaRest<Vec<ManifestoResumo>> = with_retry(|| async {
            self.client
                .request_with_body(
                    reqwest::Method::POST,
                    "/listaManifesto",
                    &filtro,
                    "manifesto.listar",
                )
                .await
        })
        .await?;

        let total = raw.total_records;
        let dados = raw.objeto_resposta.ok_or_else(|| {
            MtrError::api(
                ErrorCategory::InternalError,
                500,
                "Lista vazia",
                ErrorContext::new("manifesto.listar"),
            )
        })?;
        Ok((dados, total))
    }

    pub async fn listar_todos(
        &self,
        data_inicial: DataMtr,
        data_final: DataMtr,
        tipo_parceiro: TipoParceiro,
    ) -> Result<Vec<ManifestoResumo>, MtrError> {
        let mut page: i32 = 0;
        let size: i32 = 100;
        let mut todos = Vec::new();

        loop {
            let filtro = ApiwsListaManifestoDTO {
                data_inicial: data_inicial.clone(),
                data_final: data_final.clone(),
                tipo_parceiro,
                pagina: Some(page),
                tamanho_pagina: Some(size),
            };

            let raw: ObjetoRespostaRest<Vec<ManifestoResumo>> = self
                .client
                .request_with_body(
                    reqwest::Method::POST,
                    "/listaManifesto",
                    &filtro,
                    "manifesto.listar_todos",
                )
                .await?;

            let batch = raw.objeto_resposta.unwrap_or_default();
            let total = raw.total_records.unwrap_or(0);
            todos.extend(batch);

            if todos.len() as i64 >= total {
                break;
            }
            page += 1;
        }

        Ok(todos)
    }

    pub async fn cancelar(
        &self,
        man_numero: impl Into<String>,
        justificativa: impl Into<String>,
    ) -> Result<CancelamentoResponse, MtrError> {
        let payload = CancelaManifestoDTO::new(man_numero, justificativa);
        payload.validate()?;

        let raw: ObjetoRespostaRest<CancelamentoResponse> = self
            .client
            .request_with_body(
                reqwest::Method::POST,
                "/cancelarManifesto",
                &payload,
                "manifesto.cancelar",
            )
            .await?;

        raw.objeto_resposta.ok_or_else(|| {
            MtrError::api(
                ErrorCategory::InternalError,
                500,
                "Cancelamento falhou",
                ErrorContext::new("manifesto.cancelar"),
            )
        })
    }

    pub async fn download_pdf(&self, man_numero: &str) -> Result<Vec<u8>, MtrError> {
        let path = format!("/downloadManifesto/{man_numero}");
        self.client.download_raw(reqwest::Method::POST, &path).await
    }

    pub async fn salvar_pdf(
        &self,
        man_numero: &str,
        caminho: impl AsRef<std::path::Path>,
    ) -> Result<(), MtrError> {
        let bytes = self.download_pdf(man_numero).await?;
        tokio::fs::write(&caminho, &bytes)
            .await
            .map_err(|e| MtrError::config(format!("Falha ao salvar PDF: {e}")))
    }
}
