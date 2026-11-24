//! Conversões de JSON do MTR-PE para tipos de domínio: manifestos e resíduos.

use jiff::civil::Date;
use serde_json::Value;

use crate::openmtr::domain::{Cnpj, CpfCnpj, Mtr, Parte, Residuo, StatusMtr};

/// Mapeia o código de situação do MTR-PE para o status canônico.
pub(crate) fn map_status_pe(codigo: Option<i64>) -> StatusMtr {
    match codigo {
        Some(1) => StatusMtr::Ativo,
        Some(3) => StatusMtr::Recebido,
        Some(4) => StatusMtr::Cancelado,
        Some(9) => StatusMtr::Outro("Em Armazenamento Temporário".into()),
        Some(c) => StatusMtr::Outro(format!("Código {c}")),
        None => StatusMtr::Outro("Desconhecido".into()),
    }
}

/// Converte o CNPJ de um parceiro do JSON para [`Parte`].
pub(crate) fn cnpj_para_parte(cnpj: &Value, unidade: Option<u32>) -> Parte {
    let cnpj_str = cnpj.as_str().unwrap_or_default();
    let documento = if cnpj_str.is_empty() || cnpj_str.len() != 14 {
        CpfCnpj::Cnpj(Cnpj::parse_unchecked("00000000000000"))
    } else {
        CpfCnpj::new(cnpj_str).unwrap_or_else(|_| CpfCnpj::Cnpj(Cnpj::parse_unchecked(cnpj_str)))
    };
    Parte {
        documento,
        razao_social: String::new(),
        endereco: None,
        municipio: None,
        uf: None,
        unidade,
    }
}

/// Interpreta uma data `YYYYMMDD` do MTR-PE.
pub(crate) fn parse_data_yyyymmdd(v: &Value) -> Option<Date> {
    let s = v.as_str().unwrap_or("");
    if s.is_empty() {
        return None;
    }
    jiff::fmt::strtime::parse("%Y%m%d", s).ok()?.to_date().ok()
}

/// Converte um item de manifesto do JSON para [`Residuo`].
pub(crate) fn json_para_residuo_item(v: &Value) -> Residuo {
    // A quantidade recebida pode vir direto no item (`quantidadeRecebida`) ou
    // aninhada em `itemManifestoRecebimentoJSONs` (primeiro registro, campo
    // `qtdRecebida`), como no fluxo de recebimento.
    let quantidade_recebida = v
        .get("quantidadeRecebida")
        .and_then(Value::as_f64)
        .or_else(|| {
            v.get("itemManifestoRecebimentoJSONs")
                .and_then(Value::as_array)
                .and_then(|a| a.first())
                .and_then(|r| r.get("qtdRecebida"))
                .and_then(Value::as_f64)
        });
    Residuo {
        codigo_ibama: v
            .get("residuo")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
        descricao: None,
        quantidade: v.get("quantidade").and_then(Value::as_f64).unwrap_or(0.0),
        unidade: v
            .get("codigoUnidade")
            .and_then(Value::as_i64)
            .map(|n| n as u32),
        estado_fisico: v
            .get("codigoTipoEstado")
            .and_then(Value::as_i64)
            .map(|n| n as u32),
        classe: v
            .get("codigoClasse")
            .and_then(Value::as_i64)
            .map(|n| n as u32),
        acondicionamento: v
            .get("codigoAcondicionamento")
            .and_then(Value::as_i64)
            .map(|n| n as u32),
        tratamento: v
            .get("codigoTecnologia")
            .and_then(Value::as_i64)
            .map(|n| n as u32),
        quantidade_recebida,
        densidade: None,
        numero_onu: None,
        classe_risco: None,
        nome_embarque: None,
        grupo_embalagem: None,
        codigo_interno: None,
        descricao_interna: None,
        observacao: v
            .get("manifestoItemObservacao")
            .or_else(|| v.get("observacao"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from),
    }
}

/// Converte o detalhe de um manifesto do JSON para [`Mtr`].
pub(crate) fn json_para_mtr(v: &Value, barcode: &str) -> Mtr {
    let situacao = v.get("situacaoManifestoCodigo").and_then(Value::as_i64);

    let residuos: Vec<Residuo> = v
        .get("itemManifestoJSONs")
        .and_then(|a| a.as_array())
        .map_or_default(|arr| arr.iter().map(json_para_residuo_item).collect());

    Mtr {
        numero: v
            .get("manifestoCodigo")
            .and_then(Value::as_i64)
            .map_or_default(|n| n.to_string()),
        barcode: barcode.to_string(),
        status: map_status_pe(situacao),
        gerador: Some(cnpj_para_parte(
            &v["cnpGerador"],
            v.get("codUnidadeGerador")
                .and_then(Value::as_i64)
                .map(|n| n as u32),
        )),
        transportador: Some(cnpj_para_parte(
            &v["cnpTransportador"],
            v.get("codUnidadeTransportador")
                .and_then(Value::as_i64)
                .map(|n| n as u32),
        )),
        destinador: Some(cnpj_para_parte(
            &v["cnpDestinador"],
            v.get("codUnidadeDestinador")
                .and_then(Value::as_i64)
                .map(|n| n as u32),
        )),
        residuos,
        data_criacao: parse_data_yyyymmdd(&v["manifData"]),
        data_recebimento: parse_data_yyyymmdd(&v["manifDataRecebimento"]),
        observacao: v
            .get("manifObservacao")
            .and_then(|s| s.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        motorista: v
            .get("manifTransportadorNomeMotorista")
            .and_then(|s| s.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        placa: v
            .get("manifTransportadorPlacaVeiculo")
            .and_then(|s| s.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        tipo_manifesto: None,
        estado_uf: None,
        data_expedicao: parse_data_yyyymmdd(&v["manifDataExpedicao"]),
        armazenamento_temporario: v
            .get("cnpArmazenador")
            .and_then(|c| c.as_str())
            .map(|s| !s.is_empty()),
        hash: None,
        tem_alteracao_pendente: None,
        numero_estadual: None,
        codigo_estadual: None,
    }
}
