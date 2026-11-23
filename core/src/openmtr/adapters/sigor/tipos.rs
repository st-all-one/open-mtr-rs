//! Conversão de respostas JSON do SIGOR para os tipos de domínio.
//!
//! As funções de conversão de MTRs e de serialização para requisições
//! encontram-se em submodules deste módulo.

pub(crate) mod mtr;
pub(crate) mod serializacao;

pub(super) use mtr::{json_para_mtr, json_para_mtr_criado, json_para_mtr_recebido};
pub(super) use serializacao::criar_mtr_para_json;

use serde_json::Value;

use crate::openmtr::domain::{
    Acondicionamento, ClasseResiduo, EstadoFisico, Parte, ResiduoIbama, StatusMtr,
    TratamentoResiduo, UnidadeMedida,
};
use crate::openmtr::domain::{Cnpj, CpfCnpj};

/// Mapeia a descrição de situação do SIGOR para o status canônico.
pub(crate) fn map_status_sigor(descricao: &str) -> StatusMtr {
    match descricao {
        "Ativo" | "Salvo" | "Cadastrado" => StatusMtr::Ativo,
        "Em Transporte" | "EM_TRANSPORTE" => StatusMtr::EmTransporte,
        "Recebido" => StatusMtr::Recebido,
        "Cancelado" => StatusMtr::Cancelado,
        "Expirado" => StatusMtr::Expirado,
        other => StatusMtr::Outro(other.to_string()),
    }
}

/// Extrai o objeto de resposta de um envelope do SIGOR.
pub(crate) fn extrair_objeto(v: &Value) -> Option<&Value> {
    v.get("objetoResposta")
}

/// Indica se o envelope de resposta sinaliza erro.
pub(crate) fn tem_erro(v: &Value) -> bool {
    v.get("erro").and_then(Value::as_bool).unwrap_or(false)
}

/// Converte um parceiro do JSON para [`Parte`].
pub(crate) fn json_para_parte_sigor(v: &Value) -> Parte {
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

/// Interpreta um timestamp em milissegundos como data.
pub(crate) fn parse_timestamp_ms(v: &Value) -> Option<chrono::NaiveDate> {
    v.as_i64()
        .and_then(chrono::DateTime::from_timestamp_millis)
        .map(|dt| dt.date_naive())
}

/// Interpreta uma data no formato brasileiro `dd/mm/aaaa`.
pub(crate) fn parse_data_br(v: &Value) -> Option<chrono::NaiveDate> {
    let s = v.as_str().unwrap_or("");
    if s.is_empty() {
        return None;
    }
    chrono::NaiveDate::parse_from_str(s, "%d/%m/%Y").ok()
}

// ─── Parse helpers for list tables ───

/// Converte uma classe de resíduo da listagem.
pub(crate) fn json_para_classe(v: &Value) -> ClasseResiduo {
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

/// Converte uma unidade de medida da listagem.
pub(crate) fn json_para_unidade(v: &Value) -> UnidadeMedida {
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

/// Converte um tratamento de resíduo da listagem.
pub(crate) fn json_para_tratamento(v: &Value) -> TratamentoResiduo {
    TratamentoResiduo {
        codigo: v.get("traCodigo").and_then(Value::as_i64).unwrap_or(0) as u32,
        descricao: v
            .get("traDescricao")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
    }
}

/// Converte um estado físico da listagem.
pub(crate) fn json_para_estado_fisico(v: &Value) -> EstadoFisico {
    EstadoFisico {
        codigo: v.get("tieCodigo").and_then(Value::as_i64).unwrap_or(0) as u32,
        descricao: v
            .get("tieDescricao")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
    }
}

/// Converte um acondicionamento da listagem.
pub(crate) fn json_para_acondicionamento(v: &Value) -> Acondicionamento {
    Acondicionamento {
        codigo: v.get("tiaCodigo").and_then(Value::as_i64).unwrap_or(0) as u32,
        descricao: v
            .get("tiaDescricao")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
    }
}

/// Converte um resíduo IBAMA da listagem.
pub(crate) fn json_para_residuo_ibama(v: &Value) -> ResiduoIbama {
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
