//! Serialização de solicitações de criação de manifesto para o SIGOR.

use serde_json::{Value, json};

use crate::openmtr::domain::{CriarMtr, ResiduoCriacao};

/// Converte um resíduo de criação para o JSON esperado pelo SIGOR.
pub(crate) fn residuo_criacao_para_json(r: &ResiduoCriacao) -> Value {
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

/// Converte uma solicitação de criação de manifesto para o JSON do SIGOR.
pub(crate) fn criar_mtr_para_json(criar: &CriarMtr) -> Value {
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
        let ms = match dt.to_zoned_datetime(jiff::tz::TimeZone::UTC) {
            Ok(zdt) => zdt.timestamp().as_millisecond(),
            Err(_) => 0,
        };
        m["dataExpedicao"] = json!(ms);
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
