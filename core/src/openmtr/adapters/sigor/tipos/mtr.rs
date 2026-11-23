//! Conversão de MTRs e de confirmações de recebimento do JSON do SIGOR.

use serde_json::Value;

use super::{json_para_parte_sigor, map_status_sigor, parse_data_br, parse_timestamp_ms};
use crate::openmtr::domain::{Mtr, MtrCriado, MtrRecebido, Residuo};

/// Converte o detalhe de um resíduo de um manifesto.
pub(crate) fn json_para_residuo_detalhe(v: &Value) -> Residuo {
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

/// Converte um manifesto completo do JSON do SIGOR.
pub(crate) fn json_para_mtr(v: &Value) -> Mtr {
    let status_str = v
        .get("situacaoManifesto")
        .and_then(|s| s.get("simDescricao"))
        .and_then(|s| s.as_str())
        .unwrap_or_default();

    let residuos: Vec<Residuo> = v
        .get("listaManifestoResiduo")
        .and_then(|a| a.as_array())
        .map_or_default(|arr| arr.iter().map(json_para_residuo_detalhe).collect());

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

/// Converte a resposta de criação de manifesto.
pub(crate) fn json_para_mtr_criado(v: &Value) -> MtrCriado {
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

/// Converte a resposta de recebimento de manifesto.
pub(crate) fn json_para_mtr_recebido(v: &Value) -> MtrRecebido {
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
