#![allow(clippy::missing_errors_doc, clippy::must_use_candidate, clippy::return_self_not_must_use, clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use async_trait::async_trait;
use futures::future::join_all;
use futures::stream::{self, StreamExt};
use serde_json::{json, Value};
use tracing::{info, info_span, warn};

use crate::openmtr::adapters::{ItemResult, LoteResult, MtrAdapter, MtrCtx, validate_download};
use crate::openmtr::domain::{
    AceitarAlteracao, Acondicionamento, AlteracaoAceita, CancelarMtr, Cdf, CdfEmitido,
    ClasseResiduo, Cnpj, CpfCnpj, CriarMtr, DownloadCdf, DownloadItem, EstadoFisico, FiltroCdf,
    FiltroMtr, GerarCdf, ModificacaoTipo, ModificarMtr, Mtr, MtrCancelado, MtrCriado,
    MtrModificado, MtrRecebido, Parte, Periodo, ReceberMtr, Residuo, ResiduoIbama, StatusMtr,
    TratamentoResiduo, UnidadeMedida,
};
use crate::openmtr::error::OpenMTRError;
use crate::openmtr::system::System;

fn map_status_pe(codigo: Option<i64>) -> StatusMtr {
    match codigo {
        Some(1) => StatusMtr::Ativo,
        Some(3) => StatusMtr::Recebido,
        Some(4) => StatusMtr::Cancelado,
        Some(9) => StatusMtr::Outro("Em Armazenamento Temporário".into()),
        Some(c) => StatusMtr::Outro(format!("Código {c}")),
        None => StatusMtr::Outro("Desconhecido".into()),
    }
}

fn json_para_classe(v: &Value) -> ClasseResiduo {
    ClasseResiduo {
        codigo: v.get("tpclaCodigo").and_then(serde_json::Value::as_i64).unwrap_or(0) as u32,
        descricao: v.get("tpclaDescricao").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
        resolucao: None,
    }
}

fn json_para_unidade(v: &Value) -> UnidadeMedida {
    UnidadeMedida {
        codigo: v.get("tpuniCodigo").and_then(serde_json::Value::as_i64).unwrap_or(0) as u32,
        descricao: v.get("tpuniDescricao").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
        sigla: v.get("tpuniSigla").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
    }
}

fn json_para_tratamento(v: &Value) -> TratamentoResiduo {
    TratamentoResiduo {
        codigo: v.get("tipoCodigo").and_then(serde_json::Value::as_i64).unwrap_or(0) as u32,
        descricao: v.get("tipoDescricao").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
    }
}

fn json_para_estado_fisico(v: &Value) -> EstadoFisico {
    EstadoFisico {
        codigo: v.get("tpestCodigo").and_then(serde_json::Value::as_i64).unwrap_or(0) as u32,
        descricao: v.get("tpestDescricao").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
    }
}

fn json_para_residuo_ibama(v: &Value) -> ResiduoIbama {
    ResiduoIbama {
        codigo_ibama: v.get("tpre3Numero").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
        descricao: v.get("tpre3Descricao").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
    }
}

fn json_para_acondicionamento(v: &Value) -> Acondicionamento {
    Acondicionamento {
        codigo: v.get("tipoCodigo").and_then(serde_json::Value::as_i64).unwrap_or(0) as u32,
        descricao: v.get("tipoDescricao").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
    }
}

fn json_para_mtr_criado(v: &Value) -> MtrCriado {
    let retorno_codigo = v.get("retornoCodigo").and_then(serde_json::Value::as_i64).unwrap_or(999);
    let codigo = v.get("manifestoCodigo").and_then(serde_json::Value::as_i64);
    let mensagem = v.get("retorno").and_then(|s| s.as_str()).unwrap_or_default().to_string();
    MtrCriado {
        numero: codigo.map(|c| c.to_string()),
        codigo_gerado: codigo,
        sucesso: retorno_codigo == 0,
        mensagem,
    }
}

fn cnpj_para_parte(cnpj: &Value, unidade: Option<u32>) -> Parte {
    let cnpj_str = cnpj.as_str().unwrap_or_default();
    let documento = if cnpj_str.is_empty() || cnpj_str.len() != 14 {
        CpfCnpj::Cnpj(Cnpj::parse_unchecked("00000000000000"))
    } else {
        CpfCnpj::new(cnpj_str)
            .unwrap_or_else(|_| CpfCnpj::Cnpj(Cnpj::parse_unchecked(cnpj_str)))
    };
    Parte { documento, razao_social: String::new(), endereco: None, municipio: None, uf: None, unidade }
}

fn parse_data_yyyymmdd(v: &Value) -> Option<chrono::NaiveDate> {
    let s = v.as_str().unwrap_or("");
    if s.is_empty() { return None; }
    chrono::NaiveDate::parse_from_str(s, "%Y%m%d").ok()
}

fn json_para_residuo_item(v: &Value) -> Residuo {
    Residuo {
        codigo_ibama: v.get("residuo").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
        descricao: None,
        quantidade: v.get("quantidade").and_then(serde_json::Value::as_f64).unwrap_or(0.0),
        unidade: v.get("codigoUnidade").and_then(serde_json::Value::as_i64).map(|n| n as u32),
        estado_fisico: v.get("codigoTipoEstado").and_then(serde_json::Value::as_i64).map(|n| n as u32),
        classe: v.get("codigoClasse").and_then(serde_json::Value::as_i64).map(|n| n as u32),
        acondicionamento: v.get("codigoAcondicionamento").and_then(serde_json::Value::as_i64).map(|n| n as u32),
        tratamento: v.get("codigoTecnologia").and_then(serde_json::Value::as_i64).map(|n| n as u32),
        quantidade_recebida: None,
        densidade: None,
        numero_onu: None,
        classe_risco: None,
        nome_embarque: None,
        grupo_embalagem: None,
        codigo_interno: None,
        descricao_interna: None,
    }
}

fn json_para_mtr(v: &Value, barcode: &str) -> Mtr {
    let situacao = v.get("situacaoManifestoCodigo").and_then(serde_json::Value::as_i64);

    let residuos: Vec<Residuo> = v
        .get("itemManifestoJSONs")
        .and_then(|a| a.as_array())
        .map(|arr| arr.iter().map(json_para_residuo_item).collect())
        .unwrap_or_default();

    Mtr {
        numero: v.get("manifestoCodigo").and_then(serde_json::Value::as_i64).map(|n| n.to_string()).unwrap_or_default(),
        barcode: barcode.to_string(),
        status: map_status_pe(situacao),
        gerador: Some(cnpj_para_parte(
            &v["cnpGerador"],
            v.get("codUnidadeGerador").and_then(serde_json::Value::as_i64).map(|n| n as u32),
        )),
        transportador: Some(cnpj_para_parte(
            &v["cnpTransportador"],
            v.get("codUnidadeTransportador").and_then(serde_json::Value::as_i64).map(|n| n as u32),
        )),
        destinador: Some(cnpj_para_parte(
            &v["cnpDestinador"],
            v.get("codUnidadeDestinador").and_then(serde_json::Value::as_i64).map(|n| n as u32),
        )),
        residuos,
        data_criacao: parse_data_yyyymmdd(&v["manifData"]),
        data_recebimento: parse_data_yyyymmdd(&v["manifDataRecebimento"]),
        observacao: v.get("manifObservacao").and_then(|s| s.as_str()).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        motorista: v.get("manifTransportadorNomeMotorista").and_then(|s| s.as_str()).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        placa: v.get("manifTransportadorPlacaVeiculo").and_then(|s| s.as_str()).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        tipo_manifesto: None,
        estado_uf: None,
        data_expedicao: parse_data_yyyymmdd(&v["manifDataExpedicao"]),
        armazenamento_temporario: v.get("cnpArmazenador").and_then(|c| c.as_str()).map(|s| !s.is_empty()),
        hash: None,
        tem_alteracao_pendente: None,
        numero_estadual: None,
        codigo_estadual: None,
    }
}

fn residuo_criacao_para_item(r: &crate::openmtr::domain::ResiduoCriacao) -> Value {
    json!({
        "quantidade": r.quantidade,
        "residuo": r.codigo_ibama,
        "codigoAcondicionamento": r.acondicionamento,
        "codigoClasse": r.classe,
        "codigoTecnologia": r.tratamento,
        "codigoTipoEstado": r.estado_fisico,
        "codigoUnidade": r.unidade,
        "manifestoItemObservacao": r.descricao_interna,
        "manifestoItemCodInterno": r.codigo_interno,
        "manifestoItemCodInternoDestinador": Option::<String>::None,
        "tipoDensidadeValor": 1,
        "tipoDensidadeUnidade": 1,
    })
}

fn criar_mtr_para_json(criar: &CriarMtr) -> Value {
    let mut m = json!({
        "cnpGerador": criar.gerador.documento.as_str(),
        "cnpTransportador": criar.transportador.documento.as_str(),
        "cnpDestinador": criar.destinador.documento.as_str(),
        "manifGeradorNomeResponsavel": criar.responsavel,
        "manifGeradorCargoResponsavel": criar.cargo.as_deref().unwrap_or("Responsavel"),
        "itemManifestoJSONs": criar.residuos.iter().map(residuo_criacao_para_item).collect::<Vec<_>>(),
    });
    if let Some(u) = criar.gerador.unidade { m["codUnidadeGerador"] = json!(u); }
    if let Some(u) = criar.transportador.unidade { m["codUnidadeTransportador"] = json!(u); }
    if let Some(u) = criar.destinador.unidade { m["codUnidadeDestinador"] = json!(u); }
    if let Some(ref obs) = criar.observacoes { m["manifObservacao"] = json!(obs); }
    if let Some(ref mot) = criar.motorista { m["manifTransportadorNomeMotorista"] = json!(mot); }
    if let Some(ref placa) = criar.placa_veiculo { m["manifTransportadorPlacaVeiculo"] = json!(placa); }
    if let Some(dt) = criar.data_expedicao { m["manifTransportadorDataExpedicao"] = json!(dt.format("%Y%m%d").to_string()); }
    if let Some(ref refcode) = criar.codigo_referencia
        && let Ok(num) = refcode.parse::<i64>() {
            m["seuCodigoReferencia"] = json!(num);
        }
    if let Some(ref arm) = criar.armazenador_temporario {
        m["cnpArmazenador"] = json!(arm.documento.as_str());
        if let Some(u) = arm.unidade { m["codUnidadeArmazenador"] = json!(u); }
    }
    m
}

fn retorno_eh_sucesso(v: &Value) -> bool {
    v.get("retornoCodigo").and_then(serde_json::Value::as_i64).unwrap_or(999) == 0
}

pub(crate) struct MtrPeAdapter;

#[async_trait]
impl MtrAdapter for MtrPeAdapter {
    fn system(&self) -> System {
        System::MtrPe
    }

    fn nome(&self) -> &'static str {
        "MTR Pernambuco"
    }

    async fn autenticar_token(&self, ctx: &MtrCtx) -> Result<String, OpenMTRError> {
        let token = ctx
            .config
            .integration_token
            .as_deref()
            .ok_or_else(|| OpenMTRError::config("integration_token é o JWT do MTR-PE"))?;
        ctx.http.set_token_direct(token);
        info!("Token MTR-PE configurado diretamente (sem refresh)");
        Ok(token.to_string())
    }

    async fn autenticar_credenciais(
        &self,
        ctx: &MtrCtx,
        params: &crate::openmtr::open_mtr_auth::AuthParams,
    ) -> Result<String, OpenMTRError> {
        let _span = info_span!("mtr_pe.autenticar_credenciais");

        #[allow(unreachable_patterns)] // defensivo: enum #[non_exhaustive]
        let (cpf, senha, unidade, cnpj) = match params {
            crate::openmtr::open_mtr_auth::AuthParams::CpfSenhaUnidade { cpf, senha, unidade, cnpj } => {
                (cpf.clone(), senha.clone(), *unidade, cnpj.clone())
            }
            _ => {
                return Err(OpenMTRError::validation(
                    "MTR-PE exige AuthParams::CpfSenhaUnidade",
                ));
            }
        };

        let cnpj = cnpj
            .or_else(|| ctx.config.cnpj.clone())
            .or_else(|| ctx.config.integration_token.clone())
            .unwrap_or_default();

        let body = json!({
            "pessoaCodigo": unidade,
            "pessoaCnpj": cnpj,
            "usuarioCpf": cpf,
            "senha": senha,
        });

        let response: Value = ctx.http.post_public("/gettoken", &body, "autenticar").await?;

        let jwt = response
            .get("token")
            .and_then(|s| s.as_str())
            .ok_or_else(|| OpenMTRError::auth("Resposta do /gettoken não contém token"))?;

        info!("Token MTR-PE obtido com sucesso");
        Ok(jwt.to_string())
    }

    async fn consultar_mtrs(
        &self,
        ctx: &MtrCtx,
        filtro: &FiltroMtr,
    ) -> Result<LoteResult<Mtr>, OpenMTRError> {
        let _span = info_span!("mtr_pe.consultar_mtrs");

        if let Some(ref nums) = filtro.numeros
            && !nums.is_empty() {
                let mut itens = Vec::with_capacity(nums.len());
                for (indice, numero) in nums.iter().enumerate() {
                    let path = format!("/retornaManifesto/{numero}");
                    match ctx.http.post::<Value, _>(&path, &json!({}), "consultar_mtr").await {
                        Ok(resp) => {
                            if retorno_eh_sucesso(&resp) {
                                itens.push(ItemResult::Ok(json_para_mtr(&resp, numero)));
                            } else {
                                let msg = resp.get("retorno").and_then(|s| s.as_str()).unwrap_or("Erro");
                                itens.push(ItemResult::Erro {
                                    indice,
                                    problema: OpenMTRError::not_found(msg),
                                });
                            }
                        }
                        Err(e) => {
                            itens.push(ItemResult::Erro {
                                indice,
                                problema: OpenMTRError::api_error(502, e.to_string()),
                            });
                        }
                    }
                }
                return Ok(LoteResult::new(itens));
            }

        // Consulta por período: o MTR-PE não lista manifestos estruturados —
        // retornaListaCodigoBarasManifesto devolve apenas códigos de barras
        // (janela máxima de 30 dias). O helper coleta os barcodes e busca o
        // detalhe de cada um via /retornaManifesto/{barcode}.
        if let Some(ref periodo) = filtro.periodo {
            return self.consultar_mtrs_por_periodo(ctx, periodo).await;
        }

        Err(OpenMTRError::validation(
            "MTR-PE: informe 'numeros' ou 'periodo' no FiltroMtr",
        ))
    }

    async fn criar_mtrs(
        &self,
        ctx: &MtrCtx,
        mtrs: Vec<CriarMtr>,
    ) -> Result<LoteResult<MtrCriado>, OpenMTRError> {
        let _span = info_span!("mtr_pe.criar_mtrs");
        let mut itens = Vec::with_capacity(mtrs.len());

        for (indice, criar) in mtrs.into_iter().enumerate() {
            let body = json!({
                "manifestoJSONDtos": [criar_mtr_para_json(&criar)]
            });

            match ctx
                .http
                .post::<Value, _>("/salvarManifestoLote", &body, "criar_mtr")
                .await
            {
                Ok(resp) => {
                    let dtos = resp.get("manifestoJSONDtos")
                        .and_then(|a| a.as_array()).cloned().unwrap_or_default();
                    if dtos.is_empty() {
                        itens.push(ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::validation("Resposta vazia do MTR-PE"),
                        });
                        continue;
                    }
                    for dto in &dtos {
                        let criado = json_para_mtr_criado(dto);
                        if criado.sucesso {
                            itens.push(ItemResult::Ok(criado));
                        } else {
                            itens.push(ItemResult::Erro {
                                indice,
                                problema: OpenMTRError::validation(criado.mensagem),
                            });
                        }
                    }
                }
                Err(e) => {
                    warn!(erro = %e, "Falha ao criar MTR no MTR-PE");
                    itens.push(ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::api_error(502, e.to_string()),
                    });
                }
            }
        }

        Ok(LoteResult::new(itens))
    }

    async fn receber_mtrs(
        &self,
        ctx: &MtrCtx,
        lotes: Vec<ReceberMtr>,
    ) -> Result<LoteResult<MtrRecebido>, OpenMTRError> {
        let _span = info_span!("mtr_pe.receber_mtrs");
        let mut itens = Vec::with_capacity(lotes.len());

        for (indice, receber) in lotes.into_iter().enumerate() {
            let body: Vec<Value> = receber
                .residuos_recebidos
                .iter()
                .map(|r| {
                    json!({
                        "codigoSequencial": Option::<i32>::None,
                        "qtdRecebida": r.quantidade_recebida,
                        "residuo": r.codigo_ibama,
                        "codigoTecnologia": r.tratamento,
                    })
                })
                .collect();

            let body = json!({
                "manifestoRecebimentoJSONs": [{
                    "manifestoCodigo": receber.numero,
                    "recebimentoMtrResponsavel": receber.responsavel,
                    "recebimentoMtrCargo": "Responsavel",
                    "recebimentoMtrData": receber.data_recebimento.format("%Y%m%d").to_string(),
                    "transporteMtrData": receber.data_recebimento.format("%Y%m%d").to_string(),
                    "itemManifestoRecebimentoJSONs": body,
                }]
            });

            match ctx
                .http
                .post::<Value, _>("/receberManifestoLote", &body, "receber_mtr")
                .await
            {
                Ok(resp) => {
                    let dtos = resp.get("manifestoRecebimentoJSONs")
                        .and_then(|a| a.as_array()).cloned().unwrap_or_default();
                    for dto in &dtos {
                        if retorno_eh_sucesso(dto) {
                            itens.push(ItemResult::Ok(MtrRecebido {
                                numero: receber.numero.clone(),
                                recebido: true,
                                mensagem: dto.get("retorno").and_then(|s| s.as_str()).map(String::from),
                            }));
                        } else {
                            let msg = dto.get("retorno").and_then(|s| s.as_str()).unwrap_or("Erro");
                            itens.push(ItemResult::Erro {
                                indice,
                                problema: OpenMTRError::validation(msg),
                            });
                        }
                    }
                }
                Err(e) => {
                    itens.push(ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::api_error(502, e.to_string()),
                    });
                }
            }
        }

        Ok(LoteResult::new(itens))
    }

    async fn cancelar_mtrs(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<CancelarMtr>,
    ) -> Result<LoteResult<MtrCancelado>, OpenMTRError> {
        let _span = info_span!("mtr_pe.cancelar_mtrs");
        let mut itens = Vec::with_capacity(reqs.len());

        for (indice, req) in reqs.into_iter().enumerate() {
            let body = json!({
                "manifestoCodigo": req.numero,
                "justificativa": req.justificativa,
            });

            match ctx
                .http
                .post::<Value, _>("/cancelarManifesto", &body, "cancelar_mtr")
                .await
            {
                Ok(resp) => {
                    if retorno_eh_sucesso(&resp) {
                        itens.push(ItemResult::Ok(MtrCancelado {
                            numero: resp.get("manifestoCodigo").and_then(serde_json::Value::as_i64).map(|n| n.to_string()).unwrap_or_default(),
                            status: "Cancelado".into(),
                            data_cancelamento: None,
                        }));
                    } else {
                        let msg = resp.get("retorno").and_then(|s| s.as_str()).unwrap_or("Erro");
                        itens.push(ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::validation(msg),
                        });
                    }
                }
                Err(e) => {
                    itens.push(ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::api_error(502, e.to_string()),
                    });
                }
            }
        }

        Ok(LoteResult::new(itens))
    }

    async fn modificar_mtrs(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<ModificarMtr>,
    ) -> Result<LoteResult<MtrModificado>, OpenMTRError> {
        let _span = info_span!("mtr_pe.modificar_mtrs");
        let mut itens = Vec::with_capacity(reqs.len());

        for (indice, req) in reqs.into_iter().enumerate() {
            match req.tipo {
                ModificacaoTipo::Recebimento => {
                    let body = json!({
                        "manifestoAlterarJSONs": [{
                            "manifestoCodigo": req.numero.parse::<i64>().unwrap_or(0),
                            "residuo": "",
                            "qtdRecebidaNovo": req.novos_residuos.as_ref().and_then(|r| r.first().map(|x| x.quantidade_recebida)),
                            "codigoClasseNovo": req.novos_residuos.as_ref().and_then(|r| r.first().and_then(|x| x.classe)),
                            "codigoTecnologiaNovo": req.novos_residuos.as_ref().and_then(|r| r.first().and_then(|x| x.tratamento)),
                            "codigoUnidadeNovo": req.novos_residuos.as_ref().and_then(|r| r.first().map(|x| x.unidade)),
                        }]
                    });

                    match ctx
                        .http
                        .post::<Value, _>("/alteraRecebimento", &body, "modificar_mtr")
                        .await
                    {
                        Ok(resp) => {
                            let dtos = resp.get("manifestoAlterarJSONs")
                                .and_then(|a| a.as_array()).cloned().unwrap_or_default();
                            if let Some(dto) = dtos.first() {
                                if retorno_eh_sucesso(dto) {
                                    itens.push(ItemResult::Ok(MtrModificado {
                                        numero: req.numero,
                                        modificado: true,
                                        mensagem: dto.get("retorno").and_then(|s| s.as_str()).unwrap_or("OK").to_string(),
                                    }));
                                } else {
                                    let msg = dto.get("retorno").and_then(|s| s.as_str()).unwrap_or("Erro");
                                    itens.push(ItemResult::Erro {
                                        indice,
                                        problema: OpenMTRError::validation(msg),
                                    });
                                }
                            }
                        }
                        Err(e) => {
                            itens.push(ItemResult::Erro {
                                indice,
                                problema: OpenMTRError::api_error(502, e.to_string()),
                            });
                        }
                    }
                }
                _ => {
                    itens.push(ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::validation(format!(
                            "Tipo de modificação não suportado no MTR-PE: {:?}",
                            req.tipo
                        )),
                    });
                }
            }
        }

        Ok(LoteResult::new(itens))
    }

    async fn aceitar_alteracoes(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<AceitarAlteracao>,
    ) -> Result<LoteResult<AlteracaoAceita>, OpenMTRError> {
        let _span = info_span!("mtr_pe.aceitar_alteracoes");
        let mut itens = Vec::with_capacity(reqs.len());

        for (indice, req) in reqs.into_iter().enumerate() {
            let num = req.numero.parse::<i64>().unwrap_or(0);
            let body = json!({
                "codigoManifesto": num,
                "aceite": if req.aceitar { "S" } else { "N" },
            });

            match ctx.http.post::<Value, _>("/aceiteAlteracaoRecebimento", &body, "aceitar_alteracao").await {
                Ok(resp) => {
                    let retorno_codigo = resp.get("retornoCodigo").and_then(serde_json::Value::as_i64).unwrap_or(999);
                    let msg = resp.get("retorno").and_then(|s| s.as_str()).unwrap_or_default().to_string();
                    if retorno_codigo == 0 {
                        itens.push(ItemResult::Ok(AlteracaoAceita {
                            numero: req.numero,
                            aceita: req.aceitar,
                            mensagem: msg,
                        }));
                    } else {
                        itens.push(ItemResult::Erro { indice, problema: OpenMTRError::validation(msg) });
                    }
                }
                Err(e) => {
                    warn!(erro = %e, "Falha ao aceitar alteração no MTR-PE");
                    itens.push(ItemResult::Erro { indice, problema: OpenMTRError::api_error(502, e.to_string()) });
                }
            }
        }

        Ok(LoteResult::new(itens))
    }

    async fn download_mtrs(
        &self,
        ctx: &MtrCtx,
        nums: Vec<String>,
    ) -> Result<LoteResult<DownloadItem>, OpenMTRError> {
        let _span = info_span!("mtr_pe.download_mtrs");
        let mut items = Vec::with_capacity(nums.len());

        let tasks: Vec<_> = nums.into_iter().enumerate().map(|(indice, numero)| {
            async move {
                let path = format!("/buscaPdfManifestoPorCodigoBarras/{numero}");
                match ctx.http.post_bytes(&path, "download_mtr").await {
                    Ok(response) => {
                        if let Some(problema) = validate_download(&response, "pdf") {
                            ItemResult::Erro { indice, problema }
                        } else {
                            ItemResult::Ok(DownloadItem {
                                filename: format!("{numero}.pdf"),
                                buffer: response.bytes,
                                formato: "pdf".into(),
                            })
                        }
                    }
                    Err(e) => ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::api_error(502, e.to_string()),
                    },
                }
            }
        }).collect();

        items.extend(join_all(tasks).await);

        Ok(LoteResult::new(items))
    }

    async fn gerar_cdfs(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<GerarCdf>,
    ) -> Result<LoteResult<CdfEmitido>, OpenMTRError> {
        let _span = info_span!("mtr_pe.gerar_cdfs");
        let mut itens = Vec::with_capacity(reqs.len());

        for (indice, req) in reqs.into_iter().enumerate() {
            let body = json!({
                "dataInicial": req.data_emissao.format("%Y%m%d").to_string(),
                "dataFinal": req.data_emissao.format("%Y%m%d").to_string(),
                "responsavel": req.responsavel,
                "txtObservacoes": req.observacoes,
                "listaGerador": [],
                "listaMtr": req.manifestos,
            });

            match ctx
                .http
                .post::<Value, _>("/emiteCDF", &body, "gerar_cdf")
                .await
            {
                Ok(resp) => {
                    if retorno_eh_sucesso(&resp) {
                        let cdf_obj = resp.get("listaCertificadoJSON");
                        if let Some(cdf) = cdf_obj {
                            if cdf.is_object() {
                                itens.push(ItemResult::Ok(CdfEmitido {
                                    codigo: cdf.get("codigo").and_then(serde_json::Value::as_i64).map(|n| n.to_string()).unwrap_or_default(),
                                    numero: cdf.get("codigo").and_then(serde_json::Value::as_i64).map(|n| n.to_string()).unwrap_or_default(),
                                    data_emissao: crate::openmtr::time_util::utc_now().date_naive(),
                                    status: String::new(),
                                }));
                            } else {
                                itens.push(ItemResult::Erro {
                                    indice,
                                    problema: OpenMTRError::validation("Resposta de CDF vazia"),
                                });
                            }
                        }
                    } else {
                        let msg = resp.get("retorno").and_then(|s| s.as_str()).unwrap_or("Erro");
                        itens.push(ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::validation(msg),
                        });
                    }
                }
                Err(e) => {
                    itens.push(ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::api_error(502, e.to_string()),
                    });
                }
            }
        }

        Ok(LoteResult::new(itens))
    }

    async fn consultar_cdfs(
        &self,
        ctx: &MtrCtx,
        filtro: &FiltroCdf,
    ) -> Result<LoteResult<Cdf>, OpenMTRError> {
        let _span = info_span!("mtr_pe.consultar_cdfs");

        let (data_ini, data_fim) = if let Some(p) = &filtro.periodo { (format!("{}", p.inicio.0.format("%d/%m/%Y")), format!("{}", p.fim.0.format("%d/%m/%Y"))) } else {
            let hoje = crate::openmtr::time_util::utc_now().date_naive();
            let mes_atras = hoje - chrono::Duration::days(30);
            (mes_atras.format("%d/%m/%Y").to_string(), hoje.format("%d/%m/%Y").to_string())
        };

        let body = json!({
            "cdfDataInicial": data_ini,
            "cdfDataFinal": data_fim,
            "cdfGeradorCodigo": null,
            "cdfDestinadorCodigo": null,
        });

        match ctx.http.post::<Value, _>("/consultaListaCdf", &body, "consultar_cdfs").await {
            Ok(resp) => {
                let lista = resp.get("listaCdfDTO").and_then(|a| a.as_array()).cloned().unwrap_or_default();
                let cdfs: Vec<Cdf> = lista.iter().map(|v| Cdf {
                    codigo: v.get("cdfCodigo").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
                    numero: v.get("cdfCodigo").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
                    data_emissao: crate::openmtr::time_util::utc_now().date_naive(),
                    status: String::new(),
                }).collect();
                Ok(LoteResult::from_vec(cdfs))
            }
            Err(e) => Err(e),
        }
    }

    async fn download_cdfs(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<DownloadCdf>,
    ) -> Result<LoteResult<DownloadItem>, OpenMTRError> {
        let _span = info_span!("mtr_pe.download_cdfs");
        let mut items = Vec::with_capacity(reqs.len());

        let tasks: Vec<_> = reqs.into_iter().enumerate().map(|(indice, req)| {
            async move {
                let path = format!("/buscaPdfCdf/{}", req.codigo);
                match ctx.http.post_bytes(&path, "download_cdf").await {
                    Ok(response) => {
                        if let Some(problema) = validate_download(&response, "pdf") {
                            ItemResult::Erro { indice, problema }
                        } else {
                            ItemResult::Ok(DownloadItem {
                                filename: format!("{}.pdf", req.codigo),
                                buffer: response.bytes,
                                formato: "pdf".into(),
                            })
                        }
                    }
                    Err(e) => ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::api_error(502, e.to_string()),
                    },
                }
            }
        }).collect();

        items.extend(join_all(tasks).await);

        Ok(LoteResult::new(items))
    }

    async fn listar_residuos(&self, ctx: &MtrCtx) -> Result<LoteResult<ResiduoIbama>, OpenMTRError> {
        let _span = info_span!("mtr_pe.listar_residuos");
        let response: Value = ctx.http.post("/retornaListaResiduo", &json!({}), "listar_residuos").await?;
        let items = response.as_array().cloned().unwrap_or_default();
        Ok(LoteResult::from_vec(items.iter().map(json_para_residuo_ibama).collect()))
    }

    async fn listar_acondicionamentos(&self, ctx: &MtrCtx) -> Result<LoteResult<Acondicionamento>, OpenMTRError> {
        let _span = info_span!("mtr_pe.listar_acondicionamentos");
        let response: Value = ctx.http.post("/retornaListaAcondicionamento", &json!({}), "listar_acondicionamentos").await?;
        let items = response.as_array().cloned().unwrap_or_default();
        Ok(LoteResult::from_vec(items.iter().map(json_para_acondicionamento).collect()))
    }

    async fn listar_classes(&self, ctx: &MtrCtx) -> Result<LoteResult<ClasseResiduo>, OpenMTRError> {
        let _span = info_span!("mtr_pe.listar_classes");
        let response: Value = ctx.http.post("/retornaListaClasse", &json!({}), "listar_classes").await?;
        let items = response.as_array().cloned().unwrap_or_default();
        Ok(LoteResult::from_vec(items.iter().map(json_para_classe).collect()))
    }

    async fn listar_estados_fisicos(&self, ctx: &MtrCtx) -> Result<LoteResult<EstadoFisico>, OpenMTRError> {
        let _span = info_span!("mtr_pe.listar_estados_fisicos");
        let response: Value = ctx.http.post("/retornaListaEstadoFisico", &json!({}), "listar_estados_fisicos").await?;
        let items = response.as_array().cloned().unwrap_or_default();
        Ok(LoteResult::from_vec(items.iter().map(json_para_estado_fisico).collect()))
    }

    async fn listar_tratamentos(&self, ctx: &MtrCtx) -> Result<LoteResult<TratamentoResiduo>, OpenMTRError> {
        let _span = info_span!("mtr_pe.listar_tratamentos");
        let response: Value = ctx.http.post("/retornaListaTecnologia", &json!({}), "listar_tratamentos").await?;
        let items = response.as_array().cloned().unwrap_or_default();
        Ok(LoteResult::from_vec(items.iter().map(json_para_tratamento).collect()))
    }

    async fn listar_unidades(&self, ctx: &MtrCtx) -> Result<LoteResult<UnidadeMedida>, OpenMTRError> {
        let _span = info_span!("mtr_pe.listar_unidades");
        let response: Value = ctx.http.post("/retornaListaUnidade", &json!({}), "listar_unidades").await?;
        let items = response.as_array().cloned().unwrap_or_default();
        Ok(LoteResult::from_vec(items.iter().map(json_para_unidade).collect()))
    }
}

impl MtrPeAdapter {
    /// Lista MTRs por período no MTR-PE.
    ///
    /// A API estadual não tem listagem estruturada — `retornaListaCodigoBarasManifesto`
    /// devolve apenas códigos de barras (janela máxima de 30 dias). Este método
    /// percorre o período em janelas de 30 dias, deduplica os barcodes e consulta
    /// o detalhe de cada manifesto em paralelo (concorrência limitada).
    async fn consultar_mtrs_por_periodo(
        &self,
        ctx: &MtrCtx,
        periodo: &Periodo,
    ) -> Result<LoteResult<Mtr>, OpenMTRError> {
        const JANELA_DIAS: i64 = 30;
        const CONCORRENCIA: usize = 10;

        let inicio = periodo.inicio.0;
        let fim = periodo.fim.0;

        // 1. Coleta barcodes em janelas de 30 dias
        let mut barcodes: Vec<String> = Vec::new();
        let mut atual = inicio;
        while atual <= fim {
            let janela_fim = (atual + chrono::Duration::days(JANELA_DIAS - 1)).min(fim);
            let path = format!(
                "/retornaListaCodigoBarasManifesto/{}/{}",
                atual.format("%Y%m%d"),
                janela_fim.format("%Y%m%d")
            );
            let resp: Value = ctx.http.post(&path, &json!({}), "listar_barcodes").await?;
            if let Some(codigos) = resp.get("codigos").and_then(Value::as_array) {
                barcodes.extend(codigos.iter().filter_map(|c| c.as_str().map(String::from)));
            }
            atual = janela_fim + chrono::Duration::days(1);
        }

        barcodes.sort();
        barcodes.dedup();

        if barcodes.is_empty() {
            info!(
                inicio = %inicio,
                fim = %fim,
                "Nenhum código de barras no período"
            );
            return Ok(LoteResult::new(Vec::new()));
        }

        let total = i64::try_from(barcodes.len()).unwrap_or(i64::MAX);

        // 2. Busca o detalhe de cada barcode (concorrência limitada)
        let tasks = barcodes.into_iter().enumerate().map(|(indice, barcode)| {
            async move {
                let path = format!("/retornaManifesto/{barcode}");
                match ctx.http.post::<Value, _>(&path, &json!({}), "consultar_mtr").await {
                    Ok(resp) => {
                        if retorno_eh_sucesso(&resp) {
                            ItemResult::Ok(json_para_mtr(&resp, &barcode))
                        } else {
                            let msg = resp.get("retorno").and_then(|s| s.as_str()).unwrap_or("Erro");
                            ItemResult::Erro {
                                indice,
                                problema: OpenMTRError::not_found(msg),
                            }
                        }
                    }
                    Err(e) => ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::api_error(502, e.to_string()),
                    },
                }
            }
        });

        let itens: Vec<_> = stream::iter(tasks)
            .buffer_unordered(CONCORRENCIA)
            .collect()
            .await;

        info!(
            total = %total,
            sucessos = %itens.iter().filter(|i| matches!(i, ItemResult::Ok(_))).count(),
            "MTRs consultados por período via códigos de barras"
        );

        let mut lote = LoteResult::new(itens);
        lote.total = Some(total);
        Ok(lote)
    }
}
