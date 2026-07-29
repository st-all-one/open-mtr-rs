use std::borrow::Cow;

use reqwest::Method;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

use super::http::MtrClient;
use crate::core::errors::MtrError;
use crate::schemas::common::ObjetoRespostaRest;
use crate::schemas::manifesto::{ApiwsListaManifestoDTO, ManifestoResumo};

pub trait Endpoint {
    type Response: DeserializeOwned;
    fn method(&self) -> Method;
    fn path(&self) -> Cow<'static, str>;
    fn query_params(&self) -> Vec<(String, String)> {
        vec![]
    }
    fn body(&self) -> Option<Value> {
        None
    }
}

pub struct ListarManifestos {
    pub dto: ApiwsListaManifestoDTO,
}

impl Endpoint for ListarManifestos {
    type Response = ObjetoRespostaRest<Vec<ManifestoResumo>>;
    fn method(&self) -> Method {
        Method::POST
    }
    fn path(&self) -> Cow<'static, str> {
        "/listaManifesto".into()
    }
    fn body(&self) -> Option<Value> {
        Some(serde_json::to_value(&self.dto).unwrap())
    }
}

pub struct DownloadPdf {
    pub man_numero: String,
}

impl Endpoint for DownloadPdf {
    type Response = Vec<u8>;
    fn method(&self) -> Method {
        Method::POST
    }
    fn path(&self) -> Cow<'static, str> {
        format!("/downloadManifesto/{}", self.man_numero).into()
    }
}

pub struct CancelarManifesto {
    pub dto: crate::schemas::manifesto::CancelaManifestoDTO,
}

impl Endpoint for CancelarManifesto {
    type Response = ObjetoRespostaRest<crate::schemas::manifesto::CancelamentoResponse>;
    fn method(&self) -> Method {
        Method::POST
    }
    fn path(&self) -> Cow<'static, str> {
        "/cancelarManifesto".into()
    }
    fn body(&self) -> Option<Value> {
        Some(serde_json::to_value(&self.dto).unwrap())
    }
}

impl MtrClient {
    pub async fn call<E: Endpoint>(&self, ep: E) -> Result<E::Response, MtrError> {
        match ep.body() {
            Some(body) => {
                self.request_with_body::<E::Response, Value>(
                    ep.method(),
                    &ep.path(),
                    body,
                    &ep.path(),
                )
                .await
            }
            None => {
                let query = ep.query_params();
                if query.is_empty() {
                    self.request::<E::Response>(ep.method(), &ep.path(), &ep.path())
                        .await
                } else {
                    let params: Vec<(&str, String)> =
                        query.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
                    self.request_with_params::<E::Response>(
                        ep.method(),
                        &ep.path(),
                        params,
                        &ep.path(),
                    )
                    .await
                }
            }
        }
    }

    pub async fn call_download<E: Endpoint<Response = Vec<u8>>>(
        &self,
        ep: E,
    ) -> Result<Vec<u8>, MtrError> {
        self.download_raw(ep.method(), &ep.path()).await
    }
}

pub struct PaginatedIter {
    client: MtrClient,
    filtro_base: ApiwsListaManifestoDTO,
    page: i32,
    page_size: i32,
    total_elements: i64,
    buffer: Vec<ManifestoResumo>,
    idx: usize,
    done: bool,
}

impl PaginatedIter {
    pub fn new(client: MtrClient, filtro_base: ApiwsListaManifestoDTO, page_size: i32) -> Self {
        Self {
            client,
            filtro_base,
            page: 0,
            page_size,
            total_elements: 0,
            buffer: Vec::new(),
            idx: 0,
            done: false,
        }
    }

    pub async fn next(&mut self) -> Result<Option<ManifestoResumo>, MtrError> {
        if self.done {
            return Ok(None);
        }

        if self.idx < self.buffer.len() {
            let item = self.buffer[self.idx].clone();
            self.idx += 1;
            return Ok(Some(item));
        }

        self.buffer.clear();
        self.idx = 0;

        let mut dto = self.filtro_base.clone();
        dto.pagina = Some(self.page);
        dto.tamanho_pagina = Some(self.page_size);

        let ep = ListarManifestos { dto };
        let raw: ObjetoRespostaRest<Vec<ManifestoResumo>> = self.client.call(ep).await?;
        let items = raw.objeto_resposta.unwrap_or_default();

        self.total_elements = raw.total_records.unwrap_or(0);
        self.buffer = items;
        self.page += 1;

        if self.buffer.is_empty()
            || (self.page as i64 * self.page_size as i64) >= self.total_elements
        {
            self.done = true;
        }

        if self.buffer.is_empty() {
            Ok(None)
        } else {
            let item = self.buffer[0].clone();
            self.idx = 1;
            Ok(Some(item))
        }
    }

    pub async fn collect_all(&mut self) -> Result<Vec<ManifestoResumo>, MtrError> {
        let mut todos = Vec::new();
        while let Some(item) = self.next().await? {
            todos.push(item);
        }
        Ok(todos)
    }
}
