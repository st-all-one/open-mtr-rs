use chrono::NaiveDate;
use open_mtr_cli::normalize::{CSV_HEADER, DumpRow, csv_line};
use open_mtr_core::openmtr::domain::{CpfCnpj, Mtr, Parte, Residuo, StatusMtr};
use std::collections::BTreeMap;

fn residuo(classe: u32, descricao: &str, quantidade: f64, recebida: f64) -> Residuo {
    Residuo {
        codigo_ibama: "F001".to_string(),
        descricao: Some(descricao.to_string()),
        quantidade,
        unidade: Some(4),
        estado_fisico: Some(1),
        classe: Some(classe),
        acondicionamento: Some(3),
        tratamento: Some(2),
        quantidade_recebida: Some(recebida),
        densidade: None,
        numero_onu: None,
        classe_risco: None,
        nome_embarque: None,
        grupo_embalagem: None,
        codigo_interno: None,
        descricao_interna: Some(descricao.to_string()),
    }
}

fn parte(razao: &str) -> Parte {
    Parte {
        documento: CpfCnpj::new("11222333000181").expect("cnpj"),
        razao_social: razao.to_string(),
        endereco: None,
        municipio: None,
        uf: None,
        unidade: Some(7),
    }
}

fn mtr() -> Mtr {
    Mtr {
        numero: "MTR-1".to_string(),
        barcode: "BAR-1".to_string(),
        status: StatusMtr::Recebido,
        gerador: Some(parte("Geradora")),
        transportador: None,
        destinador: Some(parte("Destinadora")),
        residuos: vec![
            residuo(1, "Desc A", 1.5, 1.25),
            residuo(43, "Desc B", 2.0, 0.75),
        ],
        data_criacao: NaiveDate::from_ymd_opt(2026, 10, 1),
        data_recebimento: NaiveDate::from_ymd_opt(2026, 10, 2),
        observacao: Some("obs do MTR".to_string()),
        motorista: Some("Fulano".to_string()),
        placa: Some("ABC1D23".to_string()),
        tipo_manifesto: None,
        estado_uf: Some("PE".to_string()),
        data_expedicao: NaiveDate::from_ymd_opt(2026, 9, 30),
        armazenamento_temporario: None,
        hash: None,
        tem_alteracao_pendente: None,
        numero_estadual: None,
        codigo_estadual: None,
    }
}

#[test]
fn dump_row_preserva_campos_do_schema_rico() {
    let classes = BTreeMap::from([(1, "CLASSE I".to_string()), (43, "CLASSE II A".to_string())]);
    let row = DumpRow::from_mtr_with_classes("sinir-transportadora", "sinir", 0, mtr(), &classes);

    assert_eq!(row.numero, "MTR-1");
    assert_eq!(row.data_criacao.as_deref(), Some("2026-10-01"));
    assert_eq!(row.data_recebimento.as_deref(), Some("2026-10-02"));
    assert_eq!(row.qtd_residuo_esperada, 3.5);
    assert_eq!(row.qtd_residuo_recebida, Some(2.0));
    assert_eq!(row.status, "Recebido");
    assert_eq!(row.observacao.as_deref(), Some("obs do MTR"));
    assert_eq!(row.gerador_documento.as_deref(), Some("11.222.333/0001-81"));
    assert_eq!(row.gerador_razao_social.as_deref(), Some("Geradora"));
    assert_eq!(row.destinador_razao_social.as_deref(), Some("Destinadora"));
    assert_eq!(row.motorista.as_deref(), Some("Fulano"));
    assert_eq!(row.placa.as_deref(), Some("ABC1D23"));
    assert_eq!(row.classe_residuo, "CLASSE I | CLASSE II A");
    assert_eq!(row.observacoes_residuos, "Desc A | Desc B");
}

#[test]
fn residuo_json_e_detalhado_e_estavel() {
    let classes = BTreeMap::from([(1, "CLASSE I".to_string()), (43, "CLASSE II A".to_string())]);
    let row = DumpRow::from_mtr_with_classes("sinir", "sinir", 0, mtr(), &classes);
    let detalhado: Vec<serde_json::Value> =
        serde_json::from_str(&row.residuos_json).expect("json detalhado");

    assert_eq!(detalhado.len(), 2);
    assert_eq!(detalhado[0]["codigo_ibama"], "F001");
    assert_eq!(detalhado[0]["descricao"], "Desc A");
    assert_eq!(detalhado[0]["quantidade"], 1.5);
    assert_eq!(detalhado[0]["quantidade_recebida"], 1.25);
    assert_eq!(detalhado[0]["classe"], 1);
    assert_eq!(detalhado[1]["classe"], 43);
    assert!(detalhado[0]["numero_onu"].is_null());
    assert!(detalhado[0]["densidade"].is_null());
}

#[test]
fn csv_tem_cabecalho_e_colunas_do_contrato() {
    let classes = BTreeMap::from([(1, "CLASSE I".to_string()), (43, "CLASSE II A".to_string())]);
    let row = DumpRow::from_mtr_with_classes("sinir", "sinir", 0, mtr(), &classes);
    let line = csv_line(&row);

    assert_eq!(CSV_HEADER.split(',').count(), 18);
    assert_eq!(
        CSV_HEADER,
        "OTR,MTR_NUMERO,DATA_EMISSAO,DATA_RECEBIMENTO,QNT_RECEBIDA,QNT_ESPERADA,STATUS,\
         OBSERVACOES,OBSERVACOES_RESIDUOS,GERADOR_CNPJ,GERADOR_NOME,DESTINADOR_CNPJ,\
         DESTINADOR_NOME,MOTORISTA,PLACA,CLASSE_RESIDUO,RESIDUO_JSON_DETALHADO,SISTEMA"
    );
    assert!(line.starts_with("-,MTR-1,2026-10-01,2026-10-02,2,"));
    assert!(line.contains("CLASSE I | CLASSE II A"));
    assert!(line.contains("F001"));
    assert!(line.ends_with(",sinir"));
}

#[test]
fn classe_desconhecida_usa_codigo() {
    let mut mtr = mtr();
    mtr.residuos[0].classe = Some(99);
    let row = DumpRow::from_mtr_with_classes(
        "sinir",
        "sinir",
        0,
        mtr,
        &BTreeMap::from([(1, "CLASSE I".to_string()), (43, "CLASSE II A".to_string())]),
    );
    assert_eq!(row.classe_residuo, "CLASSE 99 | CLASSE II A");
}
