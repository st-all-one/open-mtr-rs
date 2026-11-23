//! Conversões de JSON do SINIR: filtros de consulta e payloads de escrita.

use serde_json::{Value, json};

use crate::openmtr::domain::{
    CriarMtr, FiltroMtr, MtrCriado, MtrRecebido, ReceberMtr, ResiduoCriacao, ResiduoRecebido,
    TipoParte,
};

/// Converte o tipo de parte no código numérico esperado pelo SINIR.
pub(super) fn escrita_tipo_parte_i32(tipo: &TipoParte) -> i32 {
    match tipo {
        TipoParte::Transportador => 5,
        TipoParte::Destinador => 9,
        TipoParte::Armazenador => 10,
        TipoParte::Gerador | TipoParte::Outro(_) => 8,
    }
}

/// Constrói o corpo JSON da consulta por período e tipo de parceiro.
pub(super) fn escrita_filtro_json(filtro: &FiltroMtr, hoje: chrono::NaiveDate) -> Value {
    let (data_inicial, data_final) = if let Some(periodo) = &filtro.periodo {
        (periodo.inicio.to_string(), periodo.fim.to_string())
    } else {
        let mes_atras = hoje - chrono::Duration::days(30);
        (
            mes_atras.format("%d-%m-%Y").to_string(),
            hoje.format("%d-%m-%Y").to_string(),
        )
    };

    let tipo_parceiro = filtro
        .parte
        .as_ref()
        .map_or(8, |p| escrita_tipo_parte_i32(&p.tipo));

    json!({
        "dataInicial": data_inicial,
        "dataFinal": data_final,
        "tipoParceiro": tipo_parceiro,
    })
}

/// SINIR pagination is driven by query parameters, not body fields.
///
/// The `/listaManifesto` endpoint (Spring Data style) exposes `?page={offset}&size={n}`.
/// Empirically, `page` behaves as the **record offset** (not a 1-based page index):
/// `page=1&size=10` returns records 1..11, `page=50&size=50` returns 50..100, etc.
///
/// This helper maps the high-level `FiltroMtr` (`pagina` = 1-based page, `tamanho` = page size)
/// to query params: `page = (pagina - 1) * tamanho` e `size = tamanho`.
/// If no page size is informed, no query params are sent (SINIR defaults `page=0&size=10`).
/// Monta a URL de listagem com os parâmetros de paginação.
pub(super) fn escrita_lista_manifesto_path(filtro: &FiltroMtr) -> String {
    let Some(size) = filtro.tamanho else {
        return "/listaManifesto".to_string();
    };
    let page = filtro.pagina.unwrap_or(1).saturating_sub(1);
    let offset = page.saturating_mul(size);
    format!("/listaManifesto?page={offset}&size={size}")
}
/// Serializa um resíduo para criação de MTR.
pub(super) fn escrita_residuo_criacao_json(r: &ResiduoCriacao) -> Value {
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

/// Serializa o pedido de criação de MTR.
pub(super) fn escrita_criar_mtr_json(criar: &CriarMtr) -> Value {
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
        "tipoManifesto": criar.tipo_manifesto.as_ref().map(crate::openmtr::domain::tipo::TipoManifesto::codigo),
        "simCodigo": Option::<i32>::None,
        "listaManifestoResiduos": criar.residuos.iter().map(escrita_residuo_criacao_json).collect::<Vec<_>>(),
    })
}

/// Converte a resposta de criação em `MtrCriado`.
pub(super) fn escrita_json_mtr_criado(v: &Value) -> MtrCriado {
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

/// Serializa um resíduo recebido.
pub(super) fn escrita_residuo_recebido_json(r: &ResiduoRecebido) -> Value {
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

/// Serializa o pedido de recebimento de MTR.
pub(super) fn escrita_receber_mtr_json(receber: &ReceberMtr) -> Value {
    let residuos = if receber.residuos_recebidos.is_empty() {
        None
    } else {
        Some(
            receber
                .residuos_recebidos
                .iter()
                .map(escrita_residuo_recebido_json)
                .collect::<Vec<_>>(),
        )
    };

    json!({
        "manNumero": receber.numero,
        "dataRecebimento": receber.data_recebimento.and_utc().timestamp_millis(),
        "nomeResponsavel": receber.responsavel,
        "observacoes": receber.observacoes,
        "listaResiduosRecebidos": residuos,
    })
}

/// Converte a resposta de recebimento em `MtrRecebido`.
pub(super) fn escrita_json_mtr_recebido(v: &Value) -> MtrRecebido {
    MtrRecebido {
        numero: v
            .get("manNumero")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
        recebido: v.get("recebido").and_then(Value::as_bool).unwrap_or(false),
        mensagem: v.get("mensagem").and_then(|s| s.as_str()).map(String::from),
    }
}
