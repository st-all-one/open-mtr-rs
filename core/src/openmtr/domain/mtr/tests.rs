use super::*;
use crate::openmtr::domain::newtypes::DataMtr;

#[test]
fn test_periodo_deserializa_intervalo() {
    let p: Periodo =
        serde_json::from_str(r#"{"dataInicial":"01-06-2026","dataFinal":"10-08-2026"}"#)
            .expect("intervalo válido");
    assert_eq!(
        p.inicio.0,
        NaiveDate::from_ymd_opt(2026, 6, 1).expect("data válida")
    );
    assert_eq!(
        p.fim.0,
        NaiveDate::from_ymd_opt(2026, 8, 10).expect("data válida")
    );
}

#[test]
fn test_periodo_deserializa_iso8601_new_date() {
    // Equivalente ao que JSON.stringify(new Date()) produz no JS
    let p: Periodo = serde_json::from_str(
        r#"{"dataInicial":"2026-06-01T00:00:00.000Z","dataFinal":"2026-08-10T12:30:00.000Z"}"#,
    )
    .expect("ISO 8601 válido");
    assert_eq!(
        p.inicio.0,
        NaiveDate::from_ymd_opt(2026, 6, 1).expect("data válida")
    );
    assert_eq!(
        p.fim.0,
        NaiveDate::from_ymd_opt(2026, 8, 10).expect("data válida")
    );
}

#[test]
fn test_periodo_deserializa_iso8601_data_simples() {
    let p: Periodo =
        serde_json::from_str(r#"{"dataInicial":"2026-06-01","dataFinal":"2026-08-10"}"#)
            .expect("ISO data simples");
    assert_eq!(
        p.inicio.0,
        NaiveDate::from_ymd_opt(2026, 6, 1).expect("data válida")
    );
    assert_eq!(
        p.fim.0,
        NaiveDate::from_ymd_opt(2026, 8, 10).expect("data válida")
    );
}

#[test]
fn test_periodo_deserializa_timestamp_ms() {
    let inicio = NaiveDate::from_ymd_opt(2026, 6, 1).expect("data válida");
    let fim = NaiveDate::from_ymd_opt(2026, 8, 10).expect("data válida");
    let ts_inicio = inicio
        .and_hms_opt(0, 0, 0)
        .expect("hora válida")
        .and_utc()
        .timestamp_millis();
    let ts_fim = fim
        .and_hms_opt(0, 0, 0)
        .expect("hora válida")
        .and_utc()
        .timestamp_millis();
    let p: Periodo = serde_json::from_str(&format!(
        r#"{{"dataInicial":{ts_inicio},"dataFinal":{ts_fim}}}"#
    ))
    .expect("timestamp válido");
    assert_eq!(p.inicio.0, inicio);
    assert_eq!(p.fim.0, fim);
}

#[test]
fn test_periodo_deserializa_data_invalida() {
    let result: Result<Periodo, _> =
        serde_json::from_str(r#"{"dataInicial":"31-13-2026","dataFinal":"10-08-2026"}"#);
    assert!(result.is_err(), "data inválida deve falhar");
}

#[test]
fn test_periodo_deserializa_dias() {
    let hoje = crate::openmtr::time_util::utc_now().date_naive();
    let p: Periodo = serde_json::from_str(r#"{"dias":7}"#).expect("atalho dias");
    assert_eq!(p.fim.0, hoje);
    assert_eq!(p.inicio.0, hoje - chrono::Duration::days(7));
}

#[test]
fn test_ultimos_dias_em_e_deterministico() {
    let hoje = NaiveDate::from_ymd_opt(2026, 6, 1).expect("data válida");
    let p = Periodo::ultimos_dias_em(hoje, 7);
    assert_eq!(p.fim.0, hoje);
    assert_eq!(
        p.inicio.0,
        NaiveDate::from_ymd_opt(2026, 5, 25).expect("data válida")
    );
}

#[test]
fn test_periodo_deserializa_intervalo_invertido() {
    let result: Result<Periodo, _> =
        serde_json::from_str(r#"{"dataInicial":"10-08-2026","dataFinal":"01-06-2026"}"#);
    assert!(result.is_err(), "intervalo invertido deve falhar");
}

#[test]
fn test_periodo_serializa_intervalo() {
    let p = Periodo::new(
        DataMtr(NaiveDate::from_ymd_opt(2026, 6, 1).expect("data válida")),
        DataMtr(NaiveDate::from_ymd_opt(2026, 6, 30).expect("data válida")),
    )
    .expect("período válido");
    let json = serde_json::to_string(&p).expect("serialize");
    assert!(json.contains(r#""dataInicial":"01-06-2026""#));
    assert!(json.contains(r#""dataFinal":"30-06-2026""#));
}

#[test]
fn test_filtro_mtr_aceita_dias() {
    let f: FiltroMtr =
        serde_json::from_str(r#"{"periodo":{"dias":7},"tamanho":5}"#).expect("filtro com dias");
    assert!(f.periodo.is_some());
    assert_eq!(f.tamanho, Some(5));
}

#[test]
fn test_status_mtr_serializa_como_string() {
    let status = StatusMtr::Outro("Cancelado Automático por prazo".into());
    let json = serde_json::to_string(&status).expect("serialize");
    assert_eq!(json, r#""Cancelado Automático por prazo""#);
    assert!(!json.contains("Outro"), "não pode serializar como objeto");
}

#[test]
fn test_status_mtr_variantes_serializam_string() {
    for (status, esperado) in [
        (StatusMtr::Ativo, "Ativo"),
        (StatusMtr::EmTransporte, "Em Transporte"),
        (StatusMtr::Recebido, "Recebido"),
        (StatusMtr::Cancelado, "Cancelado"),
    ] {
        assert_eq!(
            serde_json::to_string(&status).expect("serialize"),
            format!(r#""{esperado}""#)
        );
    }
}
