//! Conversões de JSON do SINIR para resíduos, parceiros e status.

use serde_json::Value;

use crate::openmtr::domain::{Cnpj, CpfCnpj, Parte, Residuo, StatusMtr};

/// Mapeia a descrição de status do SINIR para `StatusMtr`.
pub(super) fn sinir_status(status: &str) -> StatusMtr {
    match status {
        "ATIVO" | "Ativo" | "Salvo" => StatusMtr::Ativo,
        "EM_TRANSPORTE" | "Em Transporte" => StatusMtr::EmTransporte,
        "RECEBIDO" | "Recebido" => StatusMtr::Recebido,
        "CANCELADO" | "Cancelado" => StatusMtr::Cancelado,
        "EXPIRADO" | "Expirado" => StatusMtr::Expirado,
        other => StatusMtr::Outro(other.to_string()),
    }
}
/// Converte o JSON de um parceiro em `Parte`.
pub(super) fn json_parte(v: &Value) -> Parte {
    let cnpj_str = v
        .get("parCnpj")
        .and_then(|c| c.as_str())
        .unwrap_or_default();

    // Tenta parse do CNPJ; se vier null (comum na listagem), usa fallback
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

/// Converte o JSON resumido de resíduo em `Residuo`.
pub(super) fn json_residuo(v: &Value) -> Residuo {
    Residuo {
        codigo_ibama: v
            .get("resCodigoIbama")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
        descricao: v
            .get("resDescricao")
            .and_then(|s| s.as_str())
            .map(String::from),
        quantidade: v
            .get("marQuantidade")
            .and_then(Value::as_f64)
            .unwrap_or(0.0),
        unidade: v.get("uniCodigo").and_then(Value::as_i64).map(|n| n as u32),
        estado_fisico: v.get("tieCodigo").and_then(Value::as_i64).map(|n| n as u32),
        classe: v.get("claCodigo").and_then(Value::as_i64).map(|n| n as u32),
        acondicionamento: v.get("tiaCodigo").and_then(Value::as_i64).map(|n| n as u32),
        tratamento: v.get("traCodigo").and_then(Value::as_i64).map(|n| n as u32),
        quantidade_recebida: None,
        densidade: None,
        numero_onu: None,
        classe_risco: None,
        nome_embarque: None,
        grupo_embalagem: None,
        codigo_interno: None,
        descricao_interna: None,
        observacao: v
            .get("marObservacao")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from),
    }
}

/// Parseia resíduo do formato detalhado (GET /retornaManifesto/{num}).
/// Os campos vêm aninhados em objetos: { "residuo": {...}, "unidade": {...}, ... }
/// Converte o JSON detalhado de resíduo em `Residuo`.
pub(super) fn json_residuo_detalhe(v: &Value) -> Residuo {
    let res = v.get("residuo");
    let unidade = v.get("unidade");
    let tratamento = v.get("tratamento");
    let tipo_estado = v.get("tipoEstado");
    let acond = v.get("tipoAcondicionamento");
    let classe = v.get("classe");

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
        grupo_embalagem: v.get("greCodigo").and_then(Value::as_i64).map(|n| n as u32),
        codigo_interno: v
            .get("codigolinterno")
            .and_then(|s| s.as_str())
            .map(String::from),
        descricao_interna: v
            .get("descricaolnterna")
            .and_then(|s| s.as_str())
            .map(String::from),
        observacao: v
            .get("marObservacao")
            .and_then(|s| s.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from),
    }
}
