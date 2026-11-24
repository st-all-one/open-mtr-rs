//! Serialização de requisições e interpretação do campo de retorno do MTR-PE.

use serde_json::{Value, json};

use crate::openmtr::domain::CriarMtr;

/// Serializa um resíduo de criação para o formato do MTR-PE.
pub(crate) fn residuo_criacao_para_item(r: &crate::openmtr::domain::ResiduoCriacao) -> Value {
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

/// Serializa uma criação de MTR para o payload da API estadual.
pub(crate) fn criar_mtr_para_json(criar: &CriarMtr) -> Value {
    let mut m = json!({
        "cnpGerador": criar.gerador.documento.as_str(),
        "cnpTransportador": criar.transportador.documento.as_str(),
        "cnpDestinador": criar.destinador.documento.as_str(),
        "manifGeradorNomeResponsavel": criar.responsavel,
        "manifGeradorCargoResponsavel": criar.cargo.as_deref().unwrap_or("Responsavel"),
        "itemManifestoJSONs": criar.residuos.iter().map(residuo_criacao_para_item).collect::<Vec<_>>(),
    });
    if let Some(u) = criar.gerador.unidade {
        m["codUnidadeGerador"] = json!(u);
    }
    if let Some(u) = criar.transportador.unidade {
        m["codUnidadeTransportador"] = json!(u);
    }
    if let Some(u) = criar.destinador.unidade {
        m["codUnidadeDestinador"] = json!(u);
    }
    if let Some(ref obs) = criar.observacoes {
        m["manifObservacao"] = json!(obs);
    }
    if let Some(ref mot) = criar.motorista {
        m["manifTransportadorNomeMotorista"] = json!(mot);
    }
    if let Some(ref placa) = criar.placa_veiculo {
        m["manifTransportadorPlacaVeiculo"] = json!(placa);
    }
    if let Some(dt) = criar.data_expedicao {
        m["manifTransportadorDataExpedicao"] =
            json!(jiff::fmt::strtime::format("%Y%m%d", dt).unwrap_or_default());
    }
    if let Some(ref refcode) = criar.codigo_referencia
        && let Ok(num) = refcode.parse::<i64>()
    {
        m["seuCodigoReferencia"] = json!(num);
    }
    if let Some(ref arm) = criar.armazenador_temporario {
        m["cnpArmazenador"] = json!(arm.documento.as_str());
        if let Some(u) = arm.unidade {
            m["codUnidadeArmazenador"] = json!(u);
        }
    }
    m
}

/// Indica se o campo `retornoCodigo` sinaliza sucesso.
pub(crate) fn retorno_eh_sucesso(v: &Value) -> bool {
    v.get("retornoCodigo")
        .and_then(Value::as_i64)
        .unwrap_or(999)
        == 0
}

/// `true` quando o payload de `/retornaManifesto` traz um manifesto aproveitável
/// mesmo com `retornoCodigo != 0`.
///
/// O MTR-PE responde `903` ("Erro não identificado") com o cabeçalho completo e
/// `itemManifestoJSONs` vazio enquanto o destinador não emite o CDF. Descartar
/// esse payload remove o MTR do dump inteiro.
pub(crate) fn manifesto_recuperavel(v: &Value) -> bool {
    v.get("manifestoCodigo").and_then(Value::as_i64).is_some()
}
