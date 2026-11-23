//! Conversões de JSON do SINIR: listas de domínio, CDFs e respostas genéricas.

use serde_json::Value;

use crate::openmtr::domain::{
    Acondicionamento, CdfEmitido, ClasseResiduo, EstadoFisico, ResiduoIbama, TratamentoResiduo,
    UnidadeMedida,
};

/// Converte JSON de acondicionamento em tipo de domínio.
pub(super) fn lista_acondicionamento(v: &Value) -> Acondicionamento {
    Acondicionamento {
        codigo: v.get("tiaCodigo").and_then(Value::as_i64).unwrap_or(0) as u32,
        descricao: v
            .get("tiaDescricao")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
    }
}

/// Converte JSON de unidade de medida em tipo de domínio.
pub(super) fn lista_unidade(v: &Value) -> UnidadeMedida {
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

/// Converte JSON de estado físico em tipo de domínio.
pub(super) fn lista_estado_fisico(v: &Value) -> EstadoFisico {
    EstadoFisico {
        codigo: v.get("tieCodigo").and_then(Value::as_i64).unwrap_or(0) as u32,
        descricao: v
            .get("tieDescricao")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
    }
}

/// Converte JSON de classe de resíduo em tipo de domínio.
pub(super) fn lista_classe(v: &Value) -> ClasseResiduo {
    ClasseResiduo {
        codigo: v.get("claCodigo").and_then(Value::as_i64).unwrap_or(0) as u32,
        descricao: v
            .get("claDescricao")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
        resolucao: v
            .get("claResolucao")
            .and_then(|s| s.as_str())
            .map(String::from),
    }
}

/// Converte JSON de tratamento de resíduo em tipo de domínio.
pub(super) fn lista_tratamento(v: &Value) -> TratamentoResiduo {
    TratamentoResiduo {
        codigo: v.get("traCodigo").and_then(Value::as_i64).unwrap_or(0) as u32,
        descricao: v
            .get("traDescricao")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
    }
}

/// Converte JSON de resíduo do IBAMA em tipo de domínio.
pub(super) fn lista_residuo_ibama(v: &Value) -> ResiduoIbama {
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

/// Converte JSON de CDF emitido em tipo de domínio.
pub(super) fn lista_cdf_emitido(v: &Value, hoje: chrono::NaiveDate) -> CdfEmitido {
    let data_emissao = v
        .get("dataEmissao")
        .and_then(|s| s.as_str())
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%d/%m/%Y").ok())
        .unwrap_or(hoje);

    CdfEmitido {
        codigo: v
            .get("cdfCodigo")
            .and_then(Value::as_i64)
            .map_or_default(|n| n.to_string()),
        numero: v
            .get("cdfNumero")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
        data_emissao,
        status: v
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
    }
}

/// Extrai o campo `objetoResposta` de uma resposta SINIR.
pub(super) fn extrair_objeto_resposta(v: &Value) -> Option<&Value> {
    v.get("objetoResposta")
}

/// Indica se a resposta SINIR sinaliza erro.
pub(super) fn tem_erro(v: &Value) -> bool {
    v.get("erro").and_then(Value::as_bool).unwrap_or(false)
}
