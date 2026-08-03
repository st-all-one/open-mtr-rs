#![allow(clippy::missing_errors_doc, clippy::must_use_candidate, clippy::return_self_not_must_use, clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use async_trait::async_trait;
use futures::future::join_all;
use serde_json::{json, Value};
use tracing::{info, info_span, warn};

use crate::openmtr::adapters::{ItemResult, LoteResult, MtrAdapter, MtrCtx, validate_download};
use crate::openmtr::domain::{
    AceitarAlteracao, Acondicionamento, AlteracaoAceita, CancelarMtr, Cdf, CdfEmitido,
    ClasseResiduo, Cnpj, CpfCnpj, CriarMtr, DownloadCdf, DownloadItem, EstadoFisico, FiltroCdf,
    FiltroMtr, FormatoDownload, GerarCdf, ModificacaoTipo, ModificarMtr, Mtr, MtrCancelado,
    MtrCriado, MtrModificado, MtrRecebido, Parte, ReceberMtr, Residuo, ResiduoCriacao,
    ResiduoIbama, ResiduoRecebido, StatusMtr, TipoParte, TratamentoResiduo, UnidadeMedida,
};
use crate::openmtr::error::OpenMTRError;
use crate::openmtr::system::System;

fn map_status_sinir(status: &str) -> StatusMtr {
    match status {
        "ATIVO" | "Ativo" | "Salvo" => StatusMtr::Ativo,
        "EM_TRANSPORTE" | "Em Transporte" => StatusMtr::EmTransporte,
        "RECEBIDO" | "Recebido" => StatusMtr::Recebido,
        "CANCELADO" | "Cancelado" => StatusMtr::Cancelado,
        "EXPIRADO" | "Expirado" => StatusMtr::Expirado,
        other => StatusMtr::Outro(other.to_string()),
    }
}

fn tipo_parte_para_i32(tipo: &TipoParte) -> i32 {
    match tipo {
        TipoParte::Transportador => 5,
        TipoParte::Destinador => 9,
        TipoParte::Armazenador => 10,
        TipoParte::Gerador | TipoParte::Outro(_) => 8,
    }
}

fn filtro_para_json(filtro: &FiltroMtr) -> Value {
    let (data_inicial, data_final) = if let Some(periodo) = &filtro.periodo { (
        periodo.inicio.to_string(),
        periodo.fim.to_string(),
    ) } else {
        let hoje = crate::openmtr::time_util::utc_now().date_naive();
        let mes_atras = hoje - chrono::Duration::days(30);
        (mes_atras.format("%d-%m-%Y").to_string(), hoje.format("%d-%m-%Y").to_string())
    };

    let tipo_parceiro = filtro
        .parte
        .as_ref()
        .map_or(8, |p| tipo_parte_para_i32(&p.tipo));

    json!({
        "dataInicial": data_inicial,
        "dataFinal": data_final,
        "tipoParceiro": tipo_parceiro,
        "pagina": filtro.pagina,
        "tamanhoPagina": filtro.tamanho,
    })
}

fn json_para_parte(v: &Value) -> Parte {
    let cnpj_str = v.get("parCnpj").and_then(|c| c.as_str()).unwrap_or_default();

    // Tenta parse do CNPJ; se vier null (comum na listagem), usa fallback
    let documento = if cnpj_str.is_empty() || cnpj_str == "00000000000000" {
        CpfCnpj::Cnpj(Cnpj::parse_unchecked("00000000000000"))
    } else {
        CpfCnpj::new(cnpj_str)
            .unwrap_or_else(|_| CpfCnpj::Cnpj(Cnpj::parse_unchecked("00000000000000")))
    };

    Parte {
        documento,
        razao_social: v.get("parDescricao").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
        endereco: v.get("parEndereco").and_then(|s| s.as_str()).map(String::from),
        municipio: v.get("parCidade").and_then(|s| s.as_str()).map(String::from),
        uf: v.get("parUf").and_then(|s| s.as_str()).map(String::from),
        unidade: v.get("parCodigo").and_then(serde_json::Value::as_i64).map(|n| n as u32),
    }
}

fn json_para_residuo(v: &Value) -> Residuo {
    Residuo {
        codigo_ibama: v.get("resCodigoIbama").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
        descricao: v.get("resDescricao").and_then(|s| s.as_str()).map(String::from),
        quantidade: v.get("marQuantidade").and_then(serde_json::Value::as_f64).unwrap_or(0.0),
        unidade: v.get("uniCodigo").and_then(serde_json::Value::as_i64).map(|n| n as u32),
        estado_fisico: v.get("tieCodigo").and_then(serde_json::Value::as_i64).map(|n| n as u32),
        classe: v.get("claCodigo").and_then(serde_json::Value::as_i64).map(|n| n as u32),
        acondicionamento: v.get("tiaCodigo").and_then(serde_json::Value::as_i64).map(|n| n as u32),
        tratamento: v.get("traCodigo").and_then(serde_json::Value::as_i64).map(|n| n as u32),
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

/// Parseia resíduo do formato detalhado (GET /retornaManifesto/{num}).
/// Os campos vêm aninhados em objetos: { "residuo": {...}, "unidade": {...}, ... }
fn json_para_residuo_detalhe(v: &Value) -> Residuo {
    let res = v.get("residuo");
    let unidade = v.get("unidade");
    let tratamento = v.get("tratamento");
    let tipo_estado = v.get("tipoEstado");
    let acond = v.get("tipoAcondicionamento");
    let classe = v.get("classe");

    Residuo {
        codigo_ibama: res.and_then(|r| r.get("resCodigoIbama")).and_then(|s| s.as_str()).unwrap_or_default().to_string(),
        descricao: res.and_then(|r| r.get("resDescricao")).and_then(|s| s.as_str()).map(String::from),
        quantidade: v.get("marQuantidade").and_then(serde_json::Value::as_f64).unwrap_or(0.0),
        unidade: unidade.and_then(|u| u.get("uniCodigo")).and_then(serde_json::Value::as_i64).map(|n| n as u32),
        estado_fisico: tipo_estado.and_then(|e| e.get("tieCodigo")).and_then(serde_json::Value::as_i64).map(|n| n as u32),
        classe: classe.and_then(|c| c.get("claCodigo")).and_then(serde_json::Value::as_i64).map(|n| n as u32),
        acondicionamento: acond.and_then(|a| a.get("tiaCodigo")).and_then(serde_json::Value::as_i64).map(|n| n as u32),
        tratamento: tratamento.and_then(|t| t.get("traCodigo")).and_then(serde_json::Value::as_i64).map(|n| n as u32),
        quantidade_recebida: v.get("marQuantidadeRecebida").and_then(serde_json::Value::as_f64),
        densidade: v.get("marDensidade").and_then(serde_json::Value::as_f64),
        numero_onu: v.get("marNumeroONU").and_then(|s| s.as_str()).map(String::from),
        classe_risco: v.get("marClasseRisco").and_then(|s| s.as_str()).map(String::from),
        nome_embarque: v.get("marNomeEmbarque").and_then(|s| s.as_str()).map(String::from),
        grupo_embalagem: v.get("greCodigo").and_then(serde_json::Value::as_i64).map(|n| n as u32),
        codigo_interno: v.get("codigolinterno").and_then(|s| s.as_str()).map(String::from),
        descricao_interna: v.get("descricaolnterna").and_then(|s| s.as_str()).map(String::from),
    }
}

fn json_para_mtr_detalhe(v: &Value) -> Mtr {
    let data_criacao = v
        .get("manData")
        .and_then(serde_json::Value::as_i64)
        .and_then(chrono::DateTime::from_timestamp_millis)
        .map(|dt| dt.date_naive());

    let data_expedicao = v
        .get("manDataExpedicao")
        .and_then(serde_json::Value::as_i64)
        .and_then(chrono::DateTime::from_timestamp_millis)
        .map(|dt| dt.date_naive());

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
        numero: v.get("manNumero").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
        status: map_status_sinir(status_str),
        gerador: v.get("parceiroGerador").map(json_para_parte),
        transportador: v.get("parceiroTransportador").map(json_para_parte),
        destinador: v.get("parceiroDestinador").map(json_para_parte),
        residuos,
        data_criacao,
        data_recebimento: v.get("dataRecebimentoAT").and_then(|s| s.as_str()).and_then(|s| {
            if s.is_empty() { None } else { chrono::NaiveDate::parse_from_str(s, "%d/%m/%Y").ok() }
        }),
        observacao: v.get("manObservacao").and_then(|s| s.as_str()).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        motorista: v.get("manNomeMotorista").and_then(|s| s.as_str()).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        placa: v.get("manPlacaVeiculo").and_then(|s| s.as_str()).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        tipo_manifesto: v.get("tipoManifesto").and_then(serde_json::Value::as_i64).map(|n| n as i32),
        estado_uf: v.get("estado").and_then(|e| e.get("estAbreviacao")).and_then(|s| s.as_str()).map(String::from),
        data_expedicao,
        armazenamento_temporario: v.get("possuiArmazenamentoTemporario").and_then(serde_json::Value::as_bool),
        hash: v.get("manHashCode").and_then(|s| s.as_str()).map(String::from),
        tem_alteracao_pendente: v.get("temSolicitacaoAlteracaoRecebimento").and_then(serde_json::Value::as_bool),
        numero_estadual: v.get("manNumeroEstadual").and_then(|s| s.as_str()).map(String::from).filter(|s| !s.is_empty()),
        codigo_estadual: v.get("manCodigoEstadual").and_then(serde_json::Value::as_i64),
    }
}

fn json_para_mtr(v: &Value) -> Mtr {
    let data_criacao = v
        .get("manData")
        .and_then(serde_json::Value::as_i64)
        .and_then(chrono::DateTime::from_timestamp_millis)
        .map(|dt| dt.date_naive());

    let data_expedicao = v
        .get("manDataExpedicao")
        .and_then(serde_json::Value::as_i64)
        .and_then(chrono::DateTime::from_timestamp_millis)
        .map(|dt| dt.date_naive());

    let status_str = v
        .get("manStatus")
        .and_then(|s| s.as_str())
        .or_else(|| {
            v.get("situacaoManifesto")
                .and_then(|s| s.get("simDescricao"))
                .and_then(|s| s.as_str())
        })
        .unwrap_or_default();

    let residuos: Vec<Residuo> = v
        .get("listaManifestoResiduo")
        .and_then(|a| a.as_array())
        .map(|arr| arr.iter().map(json_para_residuo).collect())
        .unwrap_or_default();

    Mtr {
        numero: v.get("manNumero").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
        status: map_status_sinir(status_str),
        gerador: v.get("parceiroGerador").map(json_para_parte),
        transportador: v.get("parceiroTransportador").map(json_para_parte),
        destinador: v.get("parceiroDestinador").map(json_para_parte),
        residuos,
        data_criacao,
        data_recebimento: None,
        // --- GAP 3: campos adicionais ---
        observacao: v.get("manObservacao").and_then(|s| s.as_str()).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        motorista: v.get("manNomeMotorista").and_then(|s| s.as_str()).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        placa: v.get("manPlacaVeiculo").and_then(|s| s.as_str()).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        tipo_manifesto: v.get("tipoManifesto").and_then(serde_json::Value::as_i64).map(|n| n as i32),
        estado_uf: v.get("estado").and_then(|e| e.get("estAbreviacao")).and_then(|s| s.as_str()).map(String::from),
        data_expedicao,
        armazenamento_temporario: v.get("possuiArmazenamentoTemporario").and_then(serde_json::Value::as_bool),
        hash: v.get("manHashCode").and_then(|s| s.as_str()).map(String::from),
        tem_alteracao_pendente: v.get("temSolicitacaoAlteracaoRecebimento").and_then(serde_json::Value::as_bool),
        numero_estadual: v.get("manNumeroEstadual").and_then(|s| s.as_str()).map(String::from).filter(|s| !s.is_empty()),
        codigo_estadual: v.get("manCodigoEstadual").and_then(serde_json::Value::as_i64),
    }
}

fn residuo_criacao_para_json(r: &ResiduoCriacao) -> Value {
    json!({
        "resCodigoIbama": r.codigo_ibama,
        "marQuantidade": r.quantidade,
        "uniCodigo": r.unidade,
        "tieCodigo": r.estado_fisico,
        "claCodigo": r.classe,
        "tiaCodigo": r.acondicionamento,
        "traCodigo": r.tratamento,
        "marDescricaoInterna": r.descricao_interna,
        "marCodigoInterno": r.codigo_interno,
    })
}

fn criar_mtr_para_json(criar: &CriarMtr) -> Value {
    json!({
        "possuiArmazenamentoTemporario": criar.armazenador_temporario.is_some(),
        "nomeResponsavel": criar.responsavel,
        "gerador": {
            "cpfCnpj": criar.gerador.documento.as_str(),
            "unidade": criar.gerador.unidade.unwrap_or(0),
        },
        "transportador": {
            "cpfCnpj": criar.transportador.documento.as_str(),
            "unidade": criar.transportador.unidade.unwrap_or(0),
        },
        "destinador": {
            "cpfCnpj": criar.destinador.documento.as_str(),
            "unidade": criar.destinador.unidade.unwrap_or(0),
        },
        "armazenadorTemporario": criar.armazenador_temporario.as_ref().map(|p| json!({
            "cpfCnpj": p.documento.as_str(),
            "unidade": p.unidade.unwrap_or(0),
        })),
        "nomeMotorista": criar.motorista,
        "placaVeiculo": criar.placa_veiculo,
        "dataExpedicao": criar.data_expedicao.map(|dt| dt.and_utc().timestamp_millis()),
        "observacoes": criar.observacoes,
        "tipoManifesto": criar.tipo_manifesto.as_ref().map(super::super::domain::tipo::TipoManifesto::codigo),
        "simCodigo": Option::<i32>::None,
        "listaManifestoResiduos": criar.residuos.iter().map(residuo_criacao_para_json).collect::<Vec<_>>(),
    })
}

fn json_para_mtr_criado(v: &Value) -> MtrCriado {
    let valido = v.get("restResponseValido").and_then(serde_json::Value::as_bool).unwrap_or(false);
    MtrCriado {
        numero: v.get("manifestoNumeroNacional").and_then(|s| s.as_str()).map(String::from),
        codigo_gerado: v.get("codigoGerado").and_then(serde_json::Value::as_i64),
        sucesso: valido,
        mensagem: v.get("restResponseMensagem").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
    }
}

fn residuo_recebido_para_json(r: &ResiduoRecebido) -> Value {
    json!({
        "resCodigoIbama": r.codigo_ibama,
        "quantidadeRecebida": r.quantidade_recebida,
        "uniCodigo": r.unidade,
        "tieCodigo": r.estado_fisico,
        "claCodigo": r.classe,
        "tiaCodigo": r.acondicionamento,
        "traCodigo": r.tratamento,
        "marSequencial": Option::<i32>::None,
    })
}

fn receber_mtr_para_json(receber: &ReceberMtr) -> Value {
    let residuos = if receber.residuos_recebidos.is_empty() {
        None
    } else {
        Some(receber.residuos_recebidos.iter().map(residuo_recebido_para_json).collect::<Vec<_>>())
    };

    json!({
        "manNumero": receber.numero,
        "dataRecebimento": receber.data_recebimento.and_utc().timestamp_millis(),
        "nomeResponsavel": receber.responsavel,
        "observacoes": receber.observacoes,
        "listaResiduosRecebidos": residuos,
    })
}

fn json_para_mtr_recebido(v: &Value) -> MtrRecebido {
    MtrRecebido {
        numero: v.get("manNumero").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
        recebido: v.get("recebido").and_then(serde_json::Value::as_bool).unwrap_or(false),
        mensagem: v.get("mensagem").and_then(|s| s.as_str()).map(String::from),
    }
}

fn json_para_acondicionamento(v: &Value) -> Acondicionamento {
    Acondicionamento {
        codigo: v.get("tiaCodigo").and_then(serde_json::Value::as_i64).unwrap_or(0) as u32,
        descricao: v.get("tiaDescricao").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
    }
}

fn json_para_unidade(v: &Value) -> UnidadeMedida {
    UnidadeMedida {
        codigo: v.get("uniCodigo").and_then(serde_json::Value::as_i64).unwrap_or(0) as u32,
        descricao: v.get("uniDescricao").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
        sigla: v.get("uniSigla").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
    }
}

fn json_para_estado_fisico(v: &Value) -> EstadoFisico {
    EstadoFisico {
        codigo: v.get("tieCodigo").and_then(serde_json::Value::as_i64).unwrap_or(0) as u32,
        descricao: v.get("tieDescricao").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
    }
}

fn json_para_classe(v: &Value) -> ClasseResiduo {
    ClasseResiduo {
        codigo: v.get("claCodigo").and_then(serde_json::Value::as_i64).unwrap_or(0) as u32,
        descricao: v.get("claDescricao").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
        resolucao: v.get("claResolucao").and_then(|s| s.as_str()).map(String::from),
    }
}

fn json_para_tratamento(v: &Value) -> TratamentoResiduo {
    TratamentoResiduo {
        codigo: v.get("traCodigo").and_then(serde_json::Value::as_i64).unwrap_or(0) as u32,
        descricao: v.get("traDescricao").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
    }
}

fn json_para_residuo_ibama(v: &Value) -> ResiduoIbama {
    ResiduoIbama {
        codigo_ibama: v.get("resCodigoIbama").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
        descricao: v.get("resDescricao").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
    }
}

fn json_para_cdf_emitido(v: &Value) -> CdfEmitido {
    let data_emissao = v.get("dataEmissao")
        .and_then(|s| s.as_str())
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%d/%m/%Y").ok())
        .unwrap_or_else(|| crate::openmtr::time_util::utc_now().date_naive());

    CdfEmitido {
        codigo: v.get("cdfCodigo").and_then(serde_json::Value::as_i64).map(|n| n.to_string()).unwrap_or_default(),
        numero: v.get("cdfNumero").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
        data_emissao,
        status: v.get("status").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
    }
}

fn extrair_objeto_resposta(v: &Value) -> Option<&Value> {
    v.get("objetoResposta")
}

fn tem_erro(v: &Value) -> bool {
    v.get("erro").and_then(serde_json::Value::as_bool).unwrap_or(false)
}

pub(crate) struct SinirAdapter;

#[async_trait]
impl MtrAdapter for SinirAdapter {
    fn system(&self) -> System {
        System::Sinir
    }

    fn nome(&self) -> &'static str {
        "SINIR Nacional"
    }

    async fn autenticar_token(&self, ctx: &MtrCtx) -> Result<String, OpenMTRError> {
        let span = info_span!("sinir.autenticar_token");
        let _guard = span.enter();
        let refresh_url = format!("{}/token", ctx.config.base_url);
        drop(_guard);
        ctx.http.set_refresh_url(&refresh_url);
        let token = ctx.http.get_token().await?;
        info!("Token SINIR renovado com sucesso");
        Ok(token)
    }

    async fn autenticar_credenciais(
        &self,
        _ctx: &MtrCtx,
        _cpf: &str,
        _senha: &str,
        _unidade: u32,
    ) -> Result<String, OpenMTRError> {
        Err(OpenMTRError::not_supported(
            "SINIR usa integration_token, não CPF/senha",
        ))
    }

    async fn consultar_mtrs(
        &self,
        ctx: &MtrCtx,
        filtro: &FiltroMtr,
    ) -> Result<LoteResult<Mtr>, OpenMTRError> {
        let _span = info_span!("sinir.consultar_mtrs");

        // Se há números específicos, usa o endpoint de detalhe
        if let Some(ref nums) = filtro.numeros
            && !nums.is_empty() {
                return self.consultar_mtrs_por_numeros(ctx, nums).await;
            }

        // Caso contrário, busca por período + tipo de parceiro
        let body = filtro_para_json(filtro);
        let response: Value = ctx.http.post("/listaManifesto", &body, "consultar_mtrs").await?;

        if tem_erro(&response) {
            warn!(
                mensagem = ?response.get("mensagem"),
                "SINIR retornou erro em consultar_mtrs"
            );
            return Ok(LoteResult::from_vec(Vec::new()));
        }

        let total_reg = response.get("totalRecords").and_then(serde_json::Value::as_i64);

        let items = response.get("objetoResposta").and_then(|a| a.as_array()).cloned().unwrap_or_default();

        // Se detalhado, extrai só os números e busca tudo via retornaManifesto
        if filtro.detalhado && !items.is_empty() {
            let numeros: Vec<String> = items.iter()
                .filter_map(|v| v.get("manNumero").and_then(|s| s.as_str()))
                .map(String::from)
                .collect();
            let mut lote = self.consultar_mtrs_por_numeros(ctx, &numeros).await?;
            lote.total = total_reg;
            info!(total = %numeros.len(), detalhados = %lote.total_ok(), "MTRs consultados com detalhes");
            return Ok(lote);
        }

        let mtrs: Vec<Mtr> = items.iter().map(json_para_mtr).collect();
        info!(total = %mtrs.len(), "MTRs consultados");

        match total_reg {
            Some(t) => Ok(LoteResult::from_vec_com_total(mtrs, t)),
            None => Ok(LoteResult::from_vec(mtrs)),
        }
    }

    async fn criar_mtrs(
        &self,
        ctx: &MtrCtx,
        mtrs: Vec<CriarMtr>,
    ) -> Result<LoteResult<MtrCriado>, OpenMTRError> {
        let _span = info_span!("sinir.criar_mtrs");

        let mut itens = Vec::with_capacity(mtrs.len());

        for (indice, criar) in mtrs.into_iter().enumerate() {
            let body = criar_mtr_para_json(&criar);
            match ctx
                .http
                .post::<Value, _>("/salvarManifestoLote", &body, "criar_mtr")
                .await
            {
                Ok(resp) => {
                    let erro_nacional = resp.get("erroNacional").and_then(serde_json::Value::as_bool).unwrap_or(false);
                    if erro_nacional {
                        let msg = resp
                            .get("mensagemErroNacional")
                            .and_then(|s| s.as_str())
                            .unwrap_or("Erro nacional na criação");
                        itens.push(ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::validation(msg),
                        });
                        continue;
                    }

                    let some_respostas = resp.get("respostaApiwsManifestoDTO")
                        .and_then(|a| a.as_array())
                        .cloned()
                        .unwrap_or_default();

                    if some_respostas.is_empty() {
                        itens.push(ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::validation(
                                "Nenhuma resposta retornada pela API",
                            ),
                        });
                        continue;
                    }

                    for r in &some_respostas {
                        let criado = json_para_mtr_criado(r);
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
                    warn!(erro = %e, "Falha ao criar MTR");
                    itens.push(ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::api_error(502, e.to_string()),
                    });
                }
            }
        }

        info!(total = %itens.len(), "MTRs criados");
        Ok(LoteResult::new(itens))
    }

    async fn receber_mtrs(
        &self,
        ctx: &MtrCtx,
        lotes: Vec<ReceberMtr>,
    ) -> Result<LoteResult<MtrRecebido>, OpenMTRError> {
        let _span = info_span!("sinir.receber_mtrs");

        let mut itens = Vec::with_capacity(lotes.len());

        for (indice, receber) in lotes.into_iter().enumerate() {
            let body = receber_mtr_para_json(&receber);
            match ctx
                .http
                .post::<Value, _>("/receberManifestoLote", &body, "receber_mtr")
                .await
            {
                Ok(resp) => {
                    let some_items = resp.get("objeto")
                        .and_then(|a| a.as_array())
                        .cloned()
                        .unwrap_or_default();

                    if some_items.is_empty() {
                        itens.push(ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::validation("Nenhum retorno no recebimento"),
                        });
                        continue;
                    }

                    for item in &some_items {
                        let recebido = json_para_mtr_recebido(item);
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
                    warn!(erro = %e, "Falha ao receber MTR");
                    itens.push(ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::api_error(502, e.to_string()),
                    });
                }
            }
        }

        info!(total = %itens.len(), "MTRs recebidos");
        Ok(LoteResult::new(itens))
    }

    async fn cancelar_mtrs(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<CancelarMtr>,
    ) -> Result<LoteResult<MtrCancelado>, OpenMTRError> {
        let _span = info_span!("sinir.cancelar_mtrs");

        let mut itens = Vec::with_capacity(reqs.len());

        for (indice, req) in reqs.into_iter().enumerate() {
            let body = json!({ "manNumero": req.numero, "justificativa": req.justificativa });
            match ctx
                .http
                .post::<Value, _>("/cancelarManifesto", &body, "cancelar_mtr")
                .await
            {
                Ok(resp) => {
                    if tem_erro(&resp) {
                        let msg = resp.get("mensagem")
                            .and_then(|s| s.as_str())
                            .unwrap_or("Erro no cancelamento");
                        itens.push(ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::validation(msg),
                        });
                        continue;
                    }

                    match extrair_objeto_resposta(&resp) {
                        Some(cancel) => {
                            let status_str = cancel.get("status")
                                .and_then(|s| s.as_str())
                                .unwrap_or_default()
                                .to_lowercase();
                            if status_str == "cancelado" {
                                itens.push(ItemResult::Ok(MtrCancelado {
                                    numero: cancel.get("manNumero")
                                        .and_then(|s| s.as_str())
                                        .unwrap_or_default()
                                        .to_string(),
                                    status: cancel.get("status")
                                        .and_then(|s| s.as_str())
                                        .unwrap_or_default()
                                        .to_string(),
                                    data_cancelamento: cancel.get("dataCancelamento")
                                        .and_then(|s| s.as_str())
                                        .map(String::from),
                                }));
                            } else {
                                let stat = cancel.get("status")
                                    .and_then(|s| s.as_str())
                                    .unwrap_or("unknown");
                                itens.push(ItemResult::Erro {
                                    indice,
                                    problema: OpenMTRError::api_error(
                                        400,
                                        format!("Status inesperado: {stat}"),
                                    ),
                                });
                            }
                        }
                        None => {
                            itens.push(ItemResult::Erro {
                                indice,
                                problema: OpenMTRError::validation("Resposta vazia no cancelamento"),
                            });
                        }
                    }
                }
                Err(e) => {
                    warn!(erro = %e, "Falha ao cancelar MTR");
                    itens.push(ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::api_error(502, e.to_string()),
                    });
                }
            }
        }

        info!(total = %itens.len(), "MTRs cancelados");
        Ok(LoteResult::new(itens))
    }

    async fn modificar_mtrs(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<ModificarMtr>,
    ) -> Result<LoteResult<MtrModificado>, OpenMTRError> {
        let _span = info_span!("sinir.modificar_mtrs");

        let mut itens = Vec::with_capacity(reqs.len());

        for (indice, req) in reqs.into_iter().enumerate() {
            match req.tipo {
                ModificacaoTipo::Recebimento => {
                    let justificativa =
                        req.justificativa.unwrap_or_else(|| "Alteração de recebimento".into());
                    let novos = req.novos_residuos.unwrap_or_default();

                    let body = json!({
                        "manNumero": req.numero,
                        "justificativa": justificativa,
                        "listaResiduosAlterados": novos.iter().map(|r| json!({
                            "resCodigoIbama": r.codigo_ibama,
                            "novaQuantidade": r.quantidade_recebida,
                            "uniCodigo": r.unidade,
                            "observacoes": Option::<String>::None,
                        })).collect::<Vec<_>>(),
                    });

                    match ctx
                        .http
                        .post::<Value, _>("/alteraRecebimento", &body, "modificar_mtr")
                        .await
                    {
                        Ok(_json) => {
                            itens.push(ItemResult::Ok(MtrModificado {
                                numero: req.numero,
                                modificado: true,
                                mensagem: "Solicitação de alteração enviada".into(),
                            }));
                        }
                        Err(e) => {
                            warn!(erro = %e, "Falha ao modificar MTR");
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
                            "Tipo de modificação não suportado no SINIR: {:?}",
                            req.tipo
                        )),
                    });
                }
            }
        }

        info!(total = %itens.len(), "MTRs modificados");
        Ok(LoteResult::new(itens))
    }

    async fn aceitar_alteracoes(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<AceitarAlteracao>,
    ) -> Result<LoteResult<AlteracaoAceita>, OpenMTRError> {
        let _span = info_span!("sinir.aceitar_alteracoes");
        let mut itens = Vec::with_capacity(reqs.len());

        for (indice, req) in reqs.into_iter().enumerate() {
            let body = json!({
                "manNumero": req.numero,
                "aceite": req.aceitar,
            });

            match ctx.http.post::<Value, _>("/aceiteAlteracaoRecebimento", &body, "aceitar_alteracao").await {
                Ok(resp) => {
                    if tem_erro(&resp) {
                        let msg = resp.get("mensagem").and_then(|s| s.as_str()).unwrap_or("Erro no aceite");
                        itens.push(ItemResult::Erro { indice, problema: OpenMTRError::validation(msg) });
                        continue;
                    }
                    let aceita = resp.get("alteracaoAceita").and_then(serde_json::Value::as_bool).unwrap_or(false);
                    let msg = resp.get("mensagem").and_then(|s| s.as_str()).unwrap_or_default().to_string();
                    itens.push(ItemResult::Ok(AlteracaoAceita {
                        numero: req.numero,
                        aceita,
                        mensagem: msg,
                    }));
                }
                Err(e) => {
                    warn!(erro = %e, "Falha ao aceitar alteração no SINIR");
                    itens.push(ItemResult::Erro { indice, problema: OpenMTRError::api_error(502, e.to_string()) });
                }
            }
        }

        info!(total = %itens.len(), "Alterações aceitas/recusadas");
        Ok(LoteResult::new(itens))
    }

    async fn download_mtrs(
        &self,
        ctx: &MtrCtx,
        nums: Vec<String>,
    ) -> Result<LoteResult<DownloadItem>, OpenMTRError> {
        let _span = info_span!("sinir.download_mtrs");

        let mut items = Vec::with_capacity(nums.len());

        let tasks: Vec<_> = nums.into_iter().enumerate().map(|(indice, numero)| {
            async move {
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
                    Err(e) => {
                        warn!(numero = %numero, erro = %e, "Falha ao baixar MTR");
                        ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::api_error(502, e.to_string()),
                        }
                    }
                }
            }
        }).collect();

        items.extend(join_all(tasks).await);

        info!(total = %items.len(), "MTRs baixados");
        Ok(LoteResult::new(items))
    }

    async fn gerar_cdfs(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<GerarCdf>,
    ) -> Result<LoteResult<CdfEmitido>, OpenMTRError> {
        let _span = info_span!("sinir.gerar_cdfs");

        let mut itens = Vec::with_capacity(reqs.len());

        for (indice, req) in reqs.into_iter().enumerate() {
            let body = json!({
                "listaManifestos": req.manifestos,
                "dataEmissao": req.data_emissao.and_hms_opt(0, 0, 0)
                    .map_or(0, |dt| dt.and_utc().timestamp_millis()),
                "nomeResponsavel": req.responsavel,
                "observacoes": req.observacoes,
            });

            match ctx
                .http
                .post::<Value, _>("/emiteCDF", &body, "gerar_cdf")
                .await
            {
                Ok(resp) => match resp.get("objeto") {
                    Some(objeto) => {
                        itens.push(ItemResult::Ok(json_para_cdf_emitido(objeto)));
                    }
                    None => {
                        itens.push(ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::validation("Resposta vazia na emissão de CDF"),
                        });
                    }
                },
                Err(e) => {
                    warn!(erro = %e, "Falha ao emitir CDF");
                    itens.push(ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::api_error(502, e.to_string()),
                    });
                }
            }
        }

        info!(total = %itens.len(), "CDFs emitidos");
        Ok(LoteResult::new(itens))
    }

    async fn consultar_cdfs(
        &self,
        _ctx: &MtrCtx,
        _filtro: &FiltroCdf,
    ) -> Result<LoteResult<Cdf>, OpenMTRError> {
        Err(OpenMTRError::not_supported(
            "SinirAdapter::consultar_cdfs — sem endpoint dedicado no SINIR",
        ))
    }

    async fn download_cdfs(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<DownloadCdf>,
    ) -> Result<LoteResult<DownloadItem>, OpenMTRError> {
        let _span = info_span!("sinir.download_cdfs");

        let mut items = Vec::with_capacity(reqs.len());

        let tasks: Vec<_> = reqs.into_iter().enumerate().map(|(indice, req)| {
            async move {
                let formato_str = match req.formato {
                    FormatoDownload::Pdf => "pdf",
                    FormatoDownload::Xml => "xml",
                };
                let path = format!(
                    "/downloadCertificado/{}?formato={}",
                    req.codigo, formato_str
                );
                match ctx.http.post_bytes(&path, "download_cdf").await {
                    Ok(response) => {
                        if let Some(problema) = validate_download(&response, formato_str) {
                            ItemResult::Erro { indice, problema }
                        } else {
                            ItemResult::Ok(DownloadItem {
                                filename: format!("{}.{}", req.codigo, formato_str),
                                buffer: response.bytes,
                                formato: formato_str.into(),
                            })
                        }
                    }
                    Err(e) => {
                        warn!(codigo = %req.codigo, erro = %e, "Falha ao baixar CDF");
                        ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::api_error(502, e.to_string()),
                        }
                    }
                }
            }
        }).collect();

        items.extend(join_all(tasks).await);

        info!(total = %items.len(), "CDFs baixados");
        Ok(LoteResult::new(items))
    }

    async fn listar_residuos(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<ResiduoIbama>, OpenMTRError> {
        let _span = info_span!("sinir.listar_residuos");

        let response: Value = ctx.http.get("/retornaListaResiduo", "listar_residuos").await?;

        let items = response.get("objetoResposta")
            .and_then(|a| a.as_array())
            .cloned()
            .unwrap_or_default();
        let domain: Vec<ResiduoIbama> = items.iter().map(json_para_residuo_ibama).collect();

        info!(total = %domain.len(), "Resíduos listados");
        Ok(LoteResult::from_vec(domain))
    }

    async fn listar_acondicionamentos(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<Acondicionamento>, OpenMTRError> {
        let _span = info_span!("sinir.listar_acondicionamentos");

        let response: Value = ctx.http
            .get("/retornaListaAcondicionamento", "listar_acondicionamentos")
            .await?;

        let items = response.get("objetoResposta")
            .and_then(|a| a.as_array())
            .cloned()
            .unwrap_or_default();
        let domain: Vec<Acondicionamento> = items.iter().map(json_para_acondicionamento).collect();

        info!(total = %domain.len(), "Acondicionamentos listados");
        Ok(LoteResult::from_vec(domain))
    }

    async fn listar_classes(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<ClasseResiduo>, OpenMTRError> {
        let _span = info_span!("sinir.listar_classes");

        let response: Value = ctx.http.get("/retornaListaClasse", "listar_classes").await?;

        let items = response.get("objetoResposta")
            .and_then(|a| a.as_array())
            .cloned()
            .unwrap_or_default();
        let domain: Vec<ClasseResiduo> = items.iter().map(json_para_classe).collect();

        info!(total = %domain.len(), "Classes listadas");
        Ok(LoteResult::from_vec(domain))
    }

    async fn listar_estados_fisicos(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<EstadoFisico>, OpenMTRError> {
        let _span = info_span!("sinir.listar_estados_fisicos");

        let response: Value = ctx.http
            .get("/retornaListaEstadoFisico", "listar_estados_fisicos")
            .await?;

        let items = response.get("objetoResposta")
            .and_then(|a| a.as_array())
            .cloned()
            .unwrap_or_default();
        let domain: Vec<EstadoFisico> = items.iter().map(json_para_estado_fisico).collect();

        info!(total = %domain.len(), "Estados físicos listados");
        Ok(LoteResult::from_vec(domain))
    }

    async fn listar_tratamentos(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<TratamentoResiduo>, OpenMTRError> {
        let _span = info_span!("sinir.listar_tratamentos");

        let response: Value = ctx.http
            .get("/retornaListaTratamento", "listar_tratamentos")
            .await?;

        let items = response.get("objetoResposta")
            .and_then(|a| a.as_array())
            .cloned()
            .unwrap_or_default();
        let domain: Vec<TratamentoResiduo> = items.iter().map(json_para_tratamento).collect();

        info!(total = %domain.len(), "Tratamentos listados");
        Ok(LoteResult::from_vec(domain))
    }

    async fn listar_unidades(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<UnidadeMedida>, OpenMTRError> {
        let _span = info_span!("sinir.listar_unidades");

        let response: Value = ctx.http.get("/retornaListaUnidade", "listar_unidades").await?;

        let items = response.get("objetoResposta")
            .and_then(|a| a.as_array())
            .cloned()
            .unwrap_or_default();
        let domain: Vec<UnidadeMedida> = items.iter().map(json_para_unidade).collect();

        info!(total = %domain.len(), "Unidades listadas");
        Ok(LoteResult::from_vec(domain))
    }
}

impl SinirAdapter {
    /// Consulta MTRs específicos via GET /retornaManifesto/{numero}
    async fn consultar_mtrs_por_numeros(
        &self,
        ctx: &MtrCtx,
        numeros: &[String],
    ) -> Result<LoteResult<Mtr>, OpenMTRError> {
        let mut itens = Vec::with_capacity(numeros.len());

        for (indice, numero) in numeros.iter().enumerate() {
            let path = format!("/retornaManifesto/{numero}");
            match ctx.http.get::<Value>(&path, "consultar_mtr_detalhe").await {
                Ok(response) => {
                    if tem_erro(&response) {
                        let msg = response.get("mensagem")
                            .and_then(|s| s.as_str())
                            .unwrap_or("Erro ao consultar MTR");
                        itens.push(ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::not_found(msg),
                        });
                        continue;
                    }
                    match extrair_objeto_resposta(&response) {
                        Some(detail) => {
                            let mtr = json_para_mtr_detalhe(detail);
                            itens.push(ItemResult::Ok(mtr));
                        }
                        None => {
                            itens.push(ItemResult::Erro {
                                indice,
                                problema: OpenMTRError::not_found("Resposta vazia do SINIR"),
                            });
                        }
                    }
                }
                Err(e) => {
                    warn!(numero = %numero, erro = %e, "Falha ao consultar MTR detalhe");
                    itens.push(ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::api_error(502, e.to_string()),
                    });
                }
            }
        }

        info!(
            total = %itens.len(),
            sucessos = %itens.iter().filter(|i| matches!(i, ItemResult::Ok(_))).count(),
            "MTRs consultados por número"
        );
        Ok(LoteResult::new(itens))
    }
}
