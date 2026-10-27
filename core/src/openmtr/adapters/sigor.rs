#![allow(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::indexing_slicing,
    reason = "adapters legados normalizam números/índices de JSON; a extração tipada entra na revitalização"
)]

use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use serde_json::{Value, json};
use tracing::{info, instrument, warn};

use crate::openmtr::adapters::{ItemResult, LoteResult, MtrAdapter, MtrCtx, validate_download};
use crate::openmtr::domain::{
    AceitarAlteracao, Acondicionamento, AlteracaoAceita, CancelarMtr, Cdf, CdfEmitido,
    ClasseResiduo, Cnpj, CpfCnpj, CriarMtr, CriarMtrComplementar, DownloadCdf, DownloadItem,
    EstadoFisico, FiltroCdf, FiltroMtr, GerarCdf, ModificacaoTipo, ModificarMtr, Mtr, MtrCancelado,
    MtrCriado, MtrModificado, MtrRecebido, Parte, ReceberMtr, Residuo, ResiduoCriacao,
    ResiduoIbama, StatusMtr, TratamentoResiduo, UnidadeMedida,
};
use crate::openmtr::error::OpenMTRError;
use crate::openmtr::system::System;

fn map_status_sigor(descricao: &str) -> StatusMtr {
    match descricao {
        "Ativo" | "Salvo" | "Cadastrado" => StatusMtr::Ativo,
        "Em Transporte" | "EM_TRANSPORTE" => StatusMtr::EmTransporte,
        "Recebido" => StatusMtr::Recebido,
        "Cancelado" => StatusMtr::Cancelado,
        "Expirado" => StatusMtr::Expirado,
        other => StatusMtr::Outro(other.to_string()),
    }
}

fn extrair_objeto(v: &Value) -> Option<&Value> {
    v.get("objetoResposta")
}

fn tem_erro(v: &Value) -> bool {
    v.get("erro").and_then(Value::as_bool).unwrap_or(false)
}

fn json_para_parte_sigor(v: &Value) -> Parte {
    let cnpj_str = v
        .get("parCnpj")
        .and_then(|c| c.as_str())
        .unwrap_or_default();
    let documento = if cnpj_str.is_empty() || cnpj_str == "00000000000000" {
        CpfCnpj::Cnpj(Cnpj::parse_unchecked("00000000000000"))
    } else {
        CpfCnpj::new(cnpj_str)
            .unwrap_or_else(|_| CpfCnpj::Cnpj(Cnpj::parse_unchecked("00000000000000")))
    };
    Parte {
        documento,
        razao_social: v
            .get("parDescricao")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
        endereco: v
            .get("parEndereco")
            .and_then(|s| s.as_str())
            .map(String::from),
        municipio: v
            .get("parCidade")
            .and_then(|s| s.as_str())
            .map(String::from),
        uf: v.get("parUf").and_then(|s| s.as_str()).map(String::from),
        unidade: v.get("parCodigo").and_then(Value::as_i64).map(|n| n as u32),
    }
}

fn parse_timestamp_ms(v: &Value) -> Option<chrono::NaiveDate> {
    v.as_i64()
        .and_then(chrono::DateTime::from_timestamp_millis)
        .map(|dt| dt.date_naive())
}

fn parse_data_br(v: &Value) -> Option<chrono::NaiveDate> {
    let s = v.as_str().unwrap_or("");
    if s.is_empty() {
        return None;
    }
    chrono::NaiveDate::parse_from_str(s, "%d/%m/%Y").ok()
}

// ─── Parse helpers for list tables ───

fn json_para_classe(v: &Value) -> ClasseResiduo {
    ClasseResiduo {
        codigo: v.get("claCodigo").and_then(Value::as_i64).unwrap_or(0) as u32,
        descricao: v
            .get("claDescricao")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
        resolucao: None,
    }
}

fn json_para_unidade(v: &Value) -> UnidadeMedida {
    UnidadeMedida {
        codigo: v.get("uniCodigo").and_then(Value::as_i64).unwrap_or(0) as u32,
        descricao: v
            .get("uniDescricao")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
        sigla: v
            .get("uniSigla")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
    }
}

fn json_para_tratamento(v: &Value) -> TratamentoResiduo {
    TratamentoResiduo {
        codigo: v.get("traCodigo").and_then(Value::as_i64).unwrap_or(0) as u32,
        descricao: v
            .get("traDescricao")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
    }
}

fn json_para_estado_fisico(v: &Value) -> EstadoFisico {
    EstadoFisico {
        codigo: v.get("tieCodigo").and_then(Value::as_i64).unwrap_or(0) as u32,
        descricao: v
            .get("tieDescricao")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
    }
}

fn json_para_acondicionamento(v: &Value) -> Acondicionamento {
    Acondicionamento {
        codigo: v.get("tiaCodigo").and_then(Value::as_i64).unwrap_or(0) as u32,
        descricao: v
            .get("tiaDescricao")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
    }
}

fn json_para_residuo_ibama(v: &Value) -> ResiduoIbama {
    ResiduoIbama {
        codigo_ibama: v
            .get("resCodigoIbama")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
        descricao: v
            .get("resDescricao")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
    }
}

// ─── Parse helpers for MTR detail ───

fn json_para_residuo_detalhe(v: &Value) -> Residuo {
    let res = v.get("residuo");
    let unidade = v.get("unidade");
    let tratamento = v.get("tratamento");
    let tipo_estado = v.get("tipoEstado");
    let acond = v.get("tipoAcondicionamento");
    let classe = v.get("classe");
    let gre = v.get("grupoEmbalagem");

    Residuo {
        codigo_ibama: res
            .and_then(|r| r.get("resCodigoIbama"))
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
        descricao: res
            .and_then(|r| r.get("resDescricao"))
            .and_then(|s| s.as_str())
            .map(String::from),
        quantidade: v
            .get("marQuantidade")
            .and_then(Value::as_f64)
            .unwrap_or(0.0),
        unidade: unidade
            .and_then(|u| u.get("uniCodigo"))
            .and_then(Value::as_i64)
            .map(|n| n as u32),
        estado_fisico: tipo_estado
            .and_then(|e| e.get("tieCodigo"))
            .and_then(Value::as_i64)
            .map(|n| n as u32),
        classe: classe
            .and_then(|c| c.get("claCodigo"))
            .and_then(Value::as_i64)
            .map(|n| n as u32),
        acondicionamento: acond
            .and_then(|a| a.get("tiaCodigo"))
            .and_then(Value::as_i64)
            .map(|n| n as u32),
        tratamento: tratamento
            .and_then(|t| t.get("traCodigo"))
            .and_then(Value::as_i64)
            .map(|n| n as u32),
        quantidade_recebida: v.get("marQuantidadeRecebida").and_then(Value::as_f64),
        densidade: v.get("marDensidade").and_then(Value::as_f64),
        numero_onu: v
            .get("marNumeroONU")
            .and_then(|s| s.as_str())
            .map(String::from),
        classe_risco: v
            .get("marClasseRisco")
            .and_then(|s| s.as_str())
            .map(String::from),
        nome_embarque: v
            .get("marNomeEmbarque")
            .and_then(|s| s.as_str())
            .map(String::from),
        grupo_embalagem: gre
            .and_then(|g| g.get("greCodigo"))
            .and_then(Value::as_i64)
            .map(|n| n as u32),
        codigo_interno: v
            .get("codigoInterno")
            .or_else(|| v.get("codigolinterno"))
            .and_then(|s| s.as_str())
            .map(String::from),
        descricao_interna: v
            .get("descricaoInterna")
            .or_else(|| v.get("descricaolnterna"))
            .and_then(|s| s.as_str())
            .map(String::from),
        observacao: v
            .get("marObservacao")
            .or_else(|| v.get("observacao"))
            .and_then(|s| s.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from),
    }
}

fn json_para_mtr(v: &Value) -> Mtr {
    let status_str = v
        .get("situacaoManifesto")
        .and_then(|s| s.get("simDescricao"))
        .and_then(|s| s.as_str())
        .unwrap_or_default();

    let residuos: Vec<Residuo> = v
        .get("listaManifestoResiduo")
        .and_then(|a| a.as_array())
        .map(|arr| arr.iter().map(json_para_residuo_detalhe).collect())
        .unwrap_or_default();

    Mtr {
        numero: v
            .get("manNumero")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
        barcode: String::new(),
        status: map_status_sigor(status_str),
        gerador: v.get("parceiroGerador").map(json_para_parte_sigor),
        transportador: v.get("parceiroTransportador").map(json_para_parte_sigor),
        destinador: v.get("parceiroDestinador").map(json_para_parte_sigor),
        residuos,
        data_criacao: parse_timestamp_ms(&v["manData"]),
        data_recebimento: parse_data_br(&v["dataRecebimentoAT"])
            .or_else(|| parse_timestamp_ms(&v["dataRecebimentoAT"])),
        observacao: v
            .get("manObservacao")
            .and_then(|s| s.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        motorista: v
            .get("manNomeMotorista")
            .and_then(|s| s.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        placa: v
            .get("manPlacaVeiculo")
            .and_then(|s| s.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        tipo_manifesto: v
            .get("tipoManifesto")
            .and_then(Value::as_i64)
            .map(|n| n as i32),
        estado_uf: v
            .get("estado")
            .and_then(|e| e.get("estAbreviacao"))
            .and_then(|s| s.as_str())
            .map(String::from),
        data_expedicao: parse_timestamp_ms(&v["manDataExpedicao"]),
        armazenamento_temporario: v
            .get("possuiArmazenamentoTemporario")
            .and_then(Value::as_bool),
        hash: v
            .get("manHashCode")
            .and_then(|s| s.as_str())
            .map(String::from),
        tem_alteracao_pendente: v
            .get("temSolicitacaoAlteracaoRecebimento")
            .and_then(Value::as_bool),
        numero_estadual: v
            .get("manNumeroEstadual")
            .and_then(|s| s.as_str())
            .map(String::from)
            .filter(|s| !s.is_empty()),
        codigo_estadual: v.get("manCodigoEstadual").and_then(Value::as_i64),
    }
}

fn json_para_mtr_criado(v: &Value) -> MtrCriado {
    let valido = v
        .get("restResponseValido")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    MtrCriado {
        numero: v
            .get("manifestoNumeroNacional")
            .and_then(|s| s.as_str())
            .map(String::from),
        codigo_gerado: v.get("codigoGerado").and_then(Value::as_i64),
        sucesso: valido,
        mensagem: v
            .get("restResponseMensagem")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
    }
}

// ─── Serialization helpers ───

fn residuo_criacao_para_json(r: &ResiduoCriacao) -> Value {
    let mut item = json!({
        "marQuantidade": r.quantidade,
        "resCodigoIbama": r.codigo_ibama,
        "uniCodigo": r.unidade,
        "traCodigo": r.tratamento,
        "tieCodigo": r.estado_fisico,
        "tiaCodigo": r.acondicionamento,
        "claCodigo": r.classe,
    });
    if let Some(d) = r.densidade {
        item["marDensidade"] = json!(d);
    }
    if let Some(ref v) = r.numero_onu {
        item["marNumeroONU"] = json!(v);
    }
    if let Some(ref v) = r.classe_risco {
        item["marClasseRisco"] = json!(v);
    }
    if let Some(ref v) = r.nome_embarque {
        item["marNomeEmbarque"] = json!(v);
    }
    if let Some(v) = r.grupo_embalagem {
        item["greCodigo"] = json!(v);
    }
    if let Some(ref v) = r.descricao_interna {
        item["marDescricaoInterna"] = json!(v);
    }
    if let Some(ref v) = r.codigo_interno {
        item["marCodigoInterno"] = json!(v);
    }
    item
}

fn criar_mtr_para_json(criar: &CriarMtr) -> Value {
    let mut m = json!({
        "nomeResponsavel": criar.responsavel,
        "transportador": {
            "unidade": criar.transportador.unidade.unwrap_or(0),
            "cpfCnpj": criar.transportador.documento.as_str(),
        },
        "destinador": {
            "unidade": criar.destinador.unidade.unwrap_or(0),
            "cpfCnpj": criar.destinador.documento.as_str(),
        },
        "listaManifestoResiduos": criar.residuos.iter().map(residuo_criacao_para_json).collect::<Vec<_>>(),
    });
    if let Some(ref c) = criar.codigo_referencia {
        m["seuCodigo"] = json!(c);
    }
    if let Some(dt) = criar.data_expedicao {
        m["dataExpedicao"] = json!(dt.and_utc().timestamp_millis());
    }
    if let Some(ref mot) = criar.motorista {
        m["nomeMotorista"] = json!(mot);
    }
    if let Some(ref p) = criar.placa_veiculo {
        m["placaVeiculo"] = json!(p);
    }
    if let Some(ref obs) = criar.observacoes {
        m["observacoes"] = json!(obs);
    }
    if let Some(ref arm) = criar.armazenador_temporario {
        m["armazenadorTemporario"] = json!({
            "unidade": arm.unidade.unwrap_or(0),
            "cpfCnpj": arm.documento.as_str(),
        });
    }
    m
}

fn json_para_mtr_recebido(v: &Value) -> MtrRecebido {
    MtrRecebido {
        numero: v
            .get("manNumero")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
        recebido: v
            .get("restResponseValido")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        mensagem: v
            .get("restResponseMensagem")
            .and_then(|s| s.as_str())
            .map(String::from),
    }
}

pub(crate) struct SigorAdapter;

#[async_trait]
impl MtrAdapter for SigorAdapter {
    fn system(&self) -> System {
        System::Sigor
    }

    fn nome(&self) -> &'static str {
        "SIGOR-SP (CETESB)"
    }

    async fn autenticar_token(&self, ctx: &MtrCtx) -> Result<String, OpenMTRError> {
        let token = ctx
            .config
            .integration_token
            .as_deref()
            .ok_or_else(|| OpenMTRError::config("integration_token é o JWT do SIGOR"))?;
        ctx.http.set_token_direct(token);
        info!("Token SIGOR configurado diretamente (sem refresh)");
        Ok(token.to_string())
    }

    #[instrument(name = "sigor.autenticar_credenciais", skip_all)]
    async fn autenticar_credenciais(
        &self,
        ctx: &MtrCtx,
        params: &crate::openmtr::open_mtr_auth::AuthParams,
    ) -> Result<String, OpenMTRError> {
        #[allow(unreachable_patterns, reason = "defensivo: enum #[non_exhaustive]")]
        let (cpf, senha, unidade) = match params {
            crate::openmtr::open_mtr_auth::AuthParams::CpfSenhaUnidade {
                cpf,
                senha,
                unidade,
                ..
            } => (cpf.clone(), senha.clone(), *unidade),
            _ => {
                return Err(OpenMTRError::validation(
                    "SIGOR exige AuthParams::CpfSenhaUnidade",
                ));
            }
        };

        let body = json!({
            "cpfCnpj": cpf,
            "senha": senha,
            "unidade": unidade.to_string(),
        });

        let response: Value = ctx
            .http
            .post_public("/gettoken", &body, "autenticar")
            .await?;

        let token_str = response
            .get("objetoResposta")
            .and_then(|s| s.as_str())
            .ok_or_else(|| OpenMTRError::auth("Resposta do /gettoken não contém objetoResposta"))?;

        let jwt = token_str.strip_prefix("Bearer ").unwrap_or(token_str);

        info!("Token SIGOR obtido com sucesso");
        Ok(jwt.to_string())
    }

    #[instrument(name = "sigor.consultar_mtrs", skip_all)]
    async fn consultar_mtrs(
        &self,
        ctx: &MtrCtx,
        filtro: &FiltroMtr,
    ) -> Result<LoteResult<Mtr>, OpenMTRError> {
        if let Some(ref nums) = filtro.numeros
            && !nums.is_empty()
        {
            let mut itens = Vec::with_capacity(nums.len());
            for (indice, numero) in nums.iter().enumerate() {
                let path = format!("/retornaManifesto/{numero}");
                match ctx.http.get::<Value>(&path, "consultar_mtr").await {
                    Ok(resp) => {
                        if tem_erro(&resp) {
                            let msg = resp
                                .get("mensagem")
                                .and_then(|s| s.as_str())
                                .unwrap_or("Erro");
                            itens.push(ItemResult::Erro {
                                indice,
                                problema: OpenMTRError::not_found(msg),
                            });
                        } else {
                            match extrair_objeto(&resp) {
                                Some(detail) => itens.push(ItemResult::Ok(json_para_mtr(detail))),
                                None => itens.push(ItemResult::Erro {
                                    indice,
                                    problema: OpenMTRError::not_found("Resposta vazia do SIGOR"),
                                }),
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
            return Ok(LoteResult::new(itens));
        }

        Err(OpenMTRError::not_supported(
            "SIGOR: consulta sem número via lista de período não implementada",
        ))
    }

    #[instrument(name = "sigor.criar_mtrs", skip_all)]
    async fn criar_mtrs(
        &self,
        ctx: &MtrCtx,
        mtrs: Vec<CriarMtr>,
    ) -> Result<LoteResult<MtrCriado>, OpenMTRError> {
        let mut itens = Vec::with_capacity(mtrs.len());

        for (indice, criar) in mtrs.into_iter().enumerate() {
            let body = criar_mtr_para_json(&criar);

            match ctx
                .http
                .post::<Value, _>("/salvarManifestoLote", &body, "criar_mtr")
                .await
            {
                Ok(resp) => {
                    if resp
                        .get("restResponseValido")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                    {
                        itens.push(ItemResult::Ok(json_para_mtr_criado(&resp)));
                    } else {
                        let msg = resp
                            .get("restResponseMensagem")
                            .and_then(|s| s.as_str())
                            .unwrap_or("Erro na criação");
                        itens.push(ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::validation(msg),
                        });
                    }
                }
                Err(e) => {
                    warn!(erro = %e, "Falha ao criar MTR no SIGOR");
                    itens.push(ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::api_error(502, e.to_string()),
                    });
                }
            }
        }

        Ok(LoteResult::new(itens))
    }

    #[instrument(name = "sigor.receber_mtrs", skip_all)]
    async fn receber_mtrs(
        &self,
        ctx: &MtrCtx,
        lotes: Vec<ReceberMtr>,
    ) -> Result<LoteResult<MtrRecebido>, OpenMTRError> {
        let mut itens = Vec::with_capacity(lotes.len());

        for (indice, receber) in lotes.into_iter().enumerate() {
            let res_items: Vec<Value> = receber
                .residuos_recebidos
                .iter()
                .map(|r| {
                    let mut item = json!({
                        "resCodigoIbama": r.codigo_ibama,
                        "marQuantidade": r.quantidade_recebida,
                        "marQuantidadeRecebida": r.quantidade_recebida,
                        "traCodigo": r.tratamento,
                        "uniCodigo": r.unidade,
                    });
                    if let Some(ref n) = r.codigo_ibama_novo {
                        item["resCodigoIbamaNovo"] = json!(n);
                    }
                    if let Some(v) = r.tratamento_novo {
                        item["traCodigoNovo"] = json!(v);
                    }
                    if let Some(v) = r.classe_nova {
                        item["claCodigoNovo"] = json!(v);
                    }
                    if let Some(v) = r.estado_fisico {
                        item["tieCodigo"] = json!(v);
                    }
                    if let Some(v) = r.acondicionamento {
                        item["tiaCodigo"] = json!(v);
                    }
                    if let Some(v) = r.classe {
                        item["claCodigo"] = json!(v);
                    }
                    item
                })
                .collect();

            let body = json!([{
                "manNumero": receber.numero,
                "dataRecebimento": receber.data_recebimento.and_utc().timestamp_millis(),
                "nomeMotorista": receber.motorista.as_deref().unwrap_or("Motorista"),
                "placaVeiculo": receber.placa_veiculo.as_deref().unwrap_or("ABC1234"),
                "nomeResponsavelRecebimento": receber.responsavel,
                "observacoes": receber.observacoes,
                "listaManifestoResiduo": res_items,
            }]);

            match ctx
                .http
                .post::<Value, _>("/receberManifestoLote", &body, "receber_mtr")
                .await
            {
                Ok(resp) => {
                    let dtos = resp
                        .get("objetoResposta")
                        .and_then(Value::as_array)
                        .map(Vec::as_slice)
                        .unwrap_or_default();
                    if dtos.is_empty() {
                        itens.push(ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::validation("Resposta vazia"),
                        });
                        continue;
                    }
                    for dto in dtos {
                        let recebido = json_para_mtr_recebido(dto);
                        if recebido.recebido {
                            itens.push(ItemResult::Ok(recebido));
                        } else {
                            itens.push(ItemResult::Erro {
                                indice,
                                problema: OpenMTRError::validation(
                                    recebido.mensagem.unwrap_or_default(),
                                ),
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

    #[instrument(name = "sigor.cancelar_mtrs", skip_all)]
    async fn cancelar_mtrs(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<CancelarMtr>,
    ) -> Result<LoteResult<MtrCancelado>, OpenMTRError> {
        let mut itens = Vec::with_capacity(reqs.len());

        if reqs.len() == 1 {
            // Single cancel via /cancelarManifesto
            let req = &reqs[0];
            let body = json!({ "manNumero": req.numero, "justificativa": req.justificativa });
            match ctx
                .http
                .post::<Value, _>("/cancelarManifesto", &body, "cancelar_mtr")
                .await
            {
                Ok(resp) => {
                    if tem_erro(&resp) {
                        let msg = resp
                            .get("mensagem")
                            .and_then(|s| s.as_str())
                            .unwrap_or("Erro");
                        itens.push(ItemResult::Erro {
                            indice: 0,
                            problema: OpenMTRError::validation(msg),
                        });
                    } else {
                        itens.push(ItemResult::Ok(MtrCancelado {
                            numero: req.numero.clone(),
                            status: "Cancelado".into(),
                            data_cancelamento: None,
                        }));
                    }
                }
                Err(e) => {
                    itens.push(ItemResult::Erro {
                        indice: 0,
                        problema: OpenMTRError::api_error(502, e.to_string()),
                    });
                }
            }
        } else {
            // Batch cancel via /cancelarManifestoLote
            let batch: Vec<Value> = reqs
                .iter()
                .map(|r| json!({ "manNumero": r.numero, "justificativa": r.justificativa }))
                .collect();
            let body = json!(batch);
            match ctx
                .http
                .post::<Value, _>("/cancelarManifestoLote", &body, "cancelar_mtrs_lote")
                .await
            {
                Ok(resp) => {
                    let results = resp
                        .get("objetoResposta")
                        .and_then(Value::as_array)
                        .map(Vec::as_slice)
                        .unwrap_or_default();
                    for (i, dto) in results.iter().enumerate() {
                        let erro = dto.get("erro").and_then(Value::as_bool).unwrap_or(true);
                        let man_num = dto
                            .get("manNumero")
                            .and_then(|s| s.as_str())
                            .unwrap_or_default();
                        if erro {
                            let msg = dto
                                .get("mensgaemRetorno")
                                .and_then(|s| s.as_str())
                                .unwrap_or("Erro");
                            itens.push(ItemResult::Erro {
                                indice: i,
                                problema: OpenMTRError::validation(msg),
                            });
                        } else {
                            itens.push(ItemResult::Ok(MtrCancelado {
                                numero: man_num.to_string(),
                                status: "Cancelado".into(),
                                data_cancelamento: None,
                            }));
                        }
                    }
                }
                Err(e) => {
                    let msg = e.to_string();
                    for i in 0..reqs.len() {
                        itens.push(ItemResult::Erro {
                            indice: i,
                            problema: OpenMTRError::api_error(502, msg.clone()),
                        });
                    }
                }
            }
        }

        Ok(LoteResult::new(itens))
    }

    #[instrument(name = "sigor.modificar_mtrs", skip_all)]
    async fn modificar_mtrs(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<ModificarMtr>,
    ) -> Result<LoteResult<MtrModificado>, OpenMTRError> {
        let mut itens = Vec::with_capacity(reqs.len());

        for (indice, req) in reqs.into_iter().enumerate() {
            match req.tipo {
                ModificacaoTipo::Recebimento => {
                    let novos = req.novos_residuos.unwrap_or_default();
                    let novos_itens: Vec<Value> = novos
                        .iter()
                        .map(|r| {
                            let mut item = json!({
                                "resCodigoIbama": r.codigo_ibama,
                                "marQuantidade": r.quantidade_recebida,
                                "marQuantidadeRecebida": r.quantidade_recebida,
                                "traCodigo": r.tratamento,
                                "claCodigo": r.classe,
                            });
                            if let Some(v) = r.classe_nova {
                                item["claCodigoNovo"] = json!(v);
                            }
                            if let Some(v) = r.tratamento_novo {
                                item["traCodigoNovo"] = json!(v);
                            }
                            item
                        })
                        .collect();

                    let body = json!({
                            "parCodigoDestinador": 0,
                            "remObservacao": req.justificativa,
                            "manifesto": {
                                "manNumero": req.numero,
                    "listaManifestoResiduo": novos_itens,
                            }
                        });

                    match ctx
                        .http
                        .post::<Value, _>("/alteraRecebimento", &body, "modificar_mtr")
                        .await
                    {
                        Ok(_) => {
                            itens.push(ItemResult::Ok(MtrModificado {
                                numero: req.numero,
                                modificado: true,
                                mensagem: "Solicitação de alteração enviada".into(),
                            }));
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
                            "Tipo de modificação não suportado no SIGOR: {:?}",
                            req.tipo
                        )),
                    });
                }
            }
        }

        Ok(LoteResult::new(itens))
    }

    #[instrument(name = "sigor.download_mtrs", skip_all)]
    async fn download_mtrs(
        &self,
        ctx: &MtrCtx,
        nums: Vec<String>,
    ) -> Result<LoteResult<DownloadItem>, OpenMTRError> {
        let tasks: Vec<_> = nums
            .into_iter()
            .enumerate()
            .map(|(indice, numero)| async move {
                let path = format!("/downloadManifesto/{numero}");
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
            })
            .collect();

        let items: Vec<_> = stream::iter(tasks)
            .buffered(ctx.config.max_concurrency())
            .collect()
            .await;
        Ok(LoteResult::new(items))
    }

    #[instrument(name = "sigor.gerar_cdfs", skip_all)]
    async fn gerar_cdfs(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<GerarCdf>,
    ) -> Result<LoteResult<CdfEmitido>, OpenMTRError> {
        let mut itens = Vec::with_capacity(reqs.len());

        for (indice, req) in reqs.into_iter().enumerate() {
            let body = json!({
                "cerDataInicial": req.data_emissao.format("%Y%m%d").to_string(),
                "cerDataFinal": req.data_emissao.format("%Y%m%d").to_string(),
                "parceiroDestinador": "",
                "cnpjDestinador": "",
                "tipoCertificadoDestinacao": 1,
                "listaManifesto": req.manifestos,
                "listaParceiroGerador": [],
                "nomeResponsavel": req.responsavel,
                "cerObservacao": req.observacoes,
            });

            match ctx
                .http
                .post::<Value, _>("/emiteCDF", &body, "gerar_cdf")
                .await
            {
                Ok(resp) => {
                    let valido = resp
                        .get("restResponseValido")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    if valido {
                        itens.push(ItemResult::Ok(CdfEmitido {
                            codigo: resp
                                .get("numeroCDF")
                                .and_then(Value::as_i64)
                                .map(|n| n.to_string())
                                .unwrap_or_default(),
                            numero: String::new(),
                            data_emissao: ctx.today(),
                            status: String::new(),
                        }));
                    } else {
                        let msg = resp
                            .get("restResponseMensagem")
                            .and_then(|s| s.as_str())
                            .unwrap_or("Erro");
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
        _ctx: &MtrCtx,
        _filtro: &FiltroCdf,
    ) -> Result<LoteResult<Cdf>, OpenMTRError> {
        Err(OpenMTRError::not_supported(
            "SIGOR: consultar_cdfs sem endpoint dedicado",
        ))
    }

    #[instrument(name = "sigor.download_cdfs", skip_all)]
    async fn download_cdfs(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<DownloadCdf>,
    ) -> Result<LoteResult<DownloadItem>, OpenMTRError> {
        let tasks: Vec<_> = reqs
            .into_iter()
            .enumerate()
            .map(|(indice, req)| async move {
                let path = format!("/downloadCertificado/{}", req.codigo);
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
            })
            .collect();

        let items: Vec<_> = stream::iter(tasks)
            .buffered(ctx.config.max_concurrency())
            .collect()
            .await;
        Ok(LoteResult::new(items))
    }

    // ─── Listagem via GET (SIGOR usa GET, não POST) ───

    #[instrument(name = "sigor.listar_residuos", skip_all)]
    async fn listar_residuos(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<ResiduoIbama>, OpenMTRError> {
        let response: Value = ctx
            .http
            .get("/retornaListaResiduo", "listar_residuos")
            .await?;
        let items = response
            .get("objetoResposta")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default();
        Ok(LoteResult::from_vec(
            items.iter().map(json_para_residuo_ibama).collect(),
        ))
    }

    #[instrument(name = "sigor.listar_acondicionamentos", skip_all)]
    async fn listar_acondicionamentos(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<Acondicionamento>, OpenMTRError> {
        let response: Value = ctx
            .http
            .get("/retornaListaAcondicionamento", "listar_acondicionamentos")
            .await?;
        let items = response
            .get("objetoResposta")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default();
        Ok(LoteResult::from_vec(
            items.iter().map(json_para_acondicionamento).collect(),
        ))
    }

    #[instrument(name = "sigor.listar_classes", skip_all)]
    async fn listar_classes(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<ClasseResiduo>, OpenMTRError> {
        let response: Value = ctx
            .http
            .get("/retornaListaClasse", "listar_classes")
            .await?;
        let items = response
            .get("objetoResposta")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default();
        Ok(LoteResult::from_vec(
            items.iter().map(json_para_classe).collect(),
        ))
    }

    #[instrument(name = "sigor.listar_estados_fisicos", skip_all)]
    async fn listar_estados_fisicos(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<EstadoFisico>, OpenMTRError> {
        let response: Value = ctx
            .http
            .get("/retornaListaEstadoFisico", "listar_estados_fisicos")
            .await?;
        let items = response
            .get("objetoResposta")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default();
        Ok(LoteResult::from_vec(
            items.iter().map(json_para_estado_fisico).collect(),
        ))
    }

    #[instrument(name = "sigor.listar_tratamentos", skip_all)]
    async fn listar_tratamentos(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<TratamentoResiduo>, OpenMTRError> {
        let response: Value = ctx
            .http
            .get("/retornaListaTratamento", "listar_tratamentos")
            .await?;
        let items = response
            .get("objetoResposta")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default();
        Ok(LoteResult::from_vec(
            items.iter().map(json_para_tratamento).collect(),
        ))
    }

    #[instrument(name = "sigor.listar_unidades", skip_all)]
    async fn listar_unidades(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<UnidadeMedida>, OpenMTRError> {
        let response: Value = ctx
            .http
            .get("/retornaListaUnidade", "listar_unidades")
            .await?;
        let items = response
            .get("objetoResposta")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default();
        Ok(LoteResult::from_vec(
            items.iter().map(json_para_unidade).collect(),
        ))
    }

    // ─── SIGOR-specific endpoints ───

    #[instrument(name = "sigor.listar_classes_por_residuo", skip_all)]
    async fn listar_classes_por_residuo(
        &self,
        ctx: &MtrCtx,
        codigo_ibama: &str,
    ) -> Result<LoteResult<ClasseResiduo>, OpenMTRError> {
        let path = format!("/retornaListaClassePorResiduo/{codigo_ibama}");
        let response: Value = ctx.http.get(&path, "listar_classes_por_residuo").await?;
        let items = response
            .get("objetoResposta")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default();
        Ok(LoteResult::from_vec(
            items.iter().map(json_para_classe).collect(),
        ))
    }

    #[instrument(name = "sigor.listar_acondicionamentos_por_estado_fisico", skip_all)]
    async fn listar_acondicionamentos_por_estado_fisico(
        &self,
        ctx: &MtrCtx,
        codigo_estado: u32,
    ) -> Result<LoteResult<Acondicionamento>, OpenMTRError> {
        let path = format!("/retornaListaAcondicionamentoPorEstadoFisico/{codigo_estado}");
        let response: Value = ctx
            .http
            .get(&path, "listar_acondicionamentos_por_estado")
            .await?;
        let items = response
            .get("objetoResposta")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default();
        Ok(LoteResult::from_vec(
            items.iter().map(json_para_acondicionamento).collect(),
        ))
    }

    #[instrument(name = "sigor.consultar_mtr_por_codigo_referencia", skip_all)]
    async fn consultar_mtr_por_codigo_referencia(
        &self,
        ctx: &MtrCtx,
        unidade: u32,
        codigo: &str,
    ) -> Result<Option<Mtr>, OpenMTRError> {
        let body = json!({ "suaUnidade": unidade, "seuCodigo": codigo });
        let response: Value = ctx
            .http
            .post("/retornaManifestoSeuCodigo", &body, "consultar_por_codigo")
            .await?;
        if tem_erro(&response) {
            return Ok(None);
        }
        match extrair_objeto(&response) {
            Some(detail) => Ok(Some(json_para_mtr(detail))),
            None => Ok(None),
        }
    }

    #[instrument(name = "sigor.criar_manifesto_complementar", skip_all)]
    async fn criar_manifesto_complementar(
        &self,
        ctx: &MtrCtx,
        req: CriarMtrComplementar,
    ) -> Result<MtrCriado, OpenMTRError> {
        let body = json!({
            "nomeMotorista": req.motorista,
            "placaVeiculo": req.placa_veiculo,
            "observacoes": req.observacoes,
            "transportador": {
                "unidade": req.transportador.unidade.unwrap_or(0),
                "cpfCnpj": req.transportador.documento.as_str(),
            },
            "listaManifesto": req.manifestos.iter().map(|m| json!({"manNumero": m})).collect::<Vec<_>>(),
        });
        let response: Value = ctx
            .http
            .post("/salvarManifestoComplementar", &body, "criar_complementar")
            .await?;
        match extrair_objeto(&response) {
            Some(obj) => {
                let valido = obj
                    .get("restResponseValido")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                Ok(MtrCriado {
                    numero: obj
                        .get("manifestoComplementar")
                        .and_then(Value::as_i64)
                        .map(|n| n.to_string()),
                    codigo_gerado: obj.get("manifestoComplementar").and_then(Value::as_i64),
                    sucesso: valido,
                    mensagem: obj
                        .get("restResponseMensagem")
                        .and_then(|s| s.as_str())
                        .unwrap_or_default()
                        .to_string(),
                })
            }
            None => Err(OpenMTRError::validation(
                "Resposta vazia do manifesto complementar",
            )),
        }
    }

    #[instrument(name = "sigor.aceitar_alteracoes", skip_all)]
    async fn aceitar_alteracoes(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<AceitarAlteracao>,
    ) -> Result<LoteResult<AlteracaoAceita>, OpenMTRError> {
        let mut itens = Vec::with_capacity(reqs.len());

        for (indice, req) in reqs.into_iter().enumerate() {
            let body = json!({
                "parCodigoGerador": req.codigo_gerador.unwrap_or(0),
                "manNumero": req.numero,
                "aceite": if req.aceitar { "a" } else { "r" },
            });

            match ctx
                .http
                .post::<Value, _>("/aceiteAlteracaoRecebimento", &body, "aceitar_alteracao")
                .await
            {
                Ok(resp) => {
                    let erro = tem_erro(&resp);
                    let msg = resp
                        .get("mensagem")
                        .and_then(|s| s.as_str())
                        .unwrap_or_default();
                    if erro {
                        itens.push(ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::validation(msg),
                        });
                    } else {
                        itens.push(ItemResult::Ok(AlteracaoAceita {
                            numero: req.numero,
                            aceita: req.aceitar,
                            mensagem: msg.to_string(),
                        }));
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
}
