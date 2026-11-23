//! Conversões de JSON das listas de domínio do MTR-PE.

use serde_json::Value;

use crate::openmtr::domain::{
    Acondicionamento, ClasseResiduo, EstadoFisico, MtrCriado, ResiduoIbama, TratamentoResiduo,
    UnidadeMedida,
};

/// Converte uma classe de resíduo do JSON.
pub(crate) fn json_para_classe(v: &Value) -> ClasseResiduo {
    ClasseResiduo {
        codigo: v.get("tpclaCodigo").and_then(Value::as_i64).unwrap_or(0) as u32,
        descricao: v
            .get("tpclaDescricao")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
        resolucao: None,
    }
}

/// Converte uma unidade de medida do JSON.
pub(crate) fn json_para_unidade(v: &Value) -> UnidadeMedida {
    UnidadeMedida {
        codigo: v.get("tpuniCodigo").and_then(Value::as_i64).unwrap_or(0) as u32,
        descricao: v
            .get("tpuniDescricao")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
        sigla: v
            .get("tpuniSigla")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
    }
}

/// Converte um tratamento de resíduo do JSON.
pub(crate) fn json_para_tratamento(v: &Value) -> TratamentoResiduo {
    TratamentoResiduo {
        codigo: v.get("tipoCodigo").and_then(Value::as_i64).unwrap_or(0) as u32,
        descricao: v
            .get("tipoDescricao")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
    }
}

/// Converte um estado físico do JSON.
pub(crate) fn json_para_estado_fisico(v: &Value) -> EstadoFisico {
    EstadoFisico {
        codigo: v.get("tpestCodigo").and_then(Value::as_i64).unwrap_or(0) as u32,
        descricao: v
            .get("tpestDescricao")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
    }
}

/// Converte um resíduo IBAMA do JSON.
pub(crate) fn json_para_residuo_ibama(v: &Value) -> ResiduoIbama {
    ResiduoIbama {
        codigo_ibama: v
            .get("tpre3Numero")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
        descricao: v
            .get("tpre3Descricao")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
    }
}

/// Converte um acondicionamento do JSON.
pub(crate) fn json_para_acondicionamento(v: &Value) -> Acondicionamento {
    Acondicionamento {
        codigo: v.get("tipoCodigo").and_then(Value::as_i64).unwrap_or(0) as u32,
        descricao: v
            .get("tipoDescricao")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
    }
}

/// Converte a confirmação de criação de MTR do JSON.
pub(crate) fn json_para_mtr_criado(v: &Value) -> MtrCriado {
    let retorno_codigo = v
        .get("retornoCodigo")
        .and_then(Value::as_i64)
        .unwrap_or(999);
    let codigo = v.get("manifestoCodigo").and_then(Value::as_i64);
    let mensagem = v
        .get("retorno")
        .and_then(|s| s.as_str())
        .unwrap_or_default()
        .to_string();
    MtrCriado {
        numero: codigo.map(|c| c.to_string()),
        codigo_gerado: codigo,
        sucesso: retorno_codigo == 0,
        mensagem,
    }
}
