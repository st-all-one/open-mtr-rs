//! Contrato de bytes das datas: serialização, `Display` e parsers das APIs externas.
//!
//! Este teste fixa o formato exato consumido/emitido pelos sistemas SINIR,
//! MTR-PE e SIGOR. É a rede de segurança da migração `chrono` → `jiff`: qualquer
//! mudança de byte no JSON, no `Display` ou nos parsers quebra aqui.

use jiff::civil::{Date, DateTime, date};
use open_mtr_core::openmtr::domain::{DataMtr, FiltroMtr, Periodo, TimestampMs};

fn d(y: i16, m: i8, day: i8) -> Date {
    date(y, m, day)
}

#[test]
fn data_mtr_serializes_dd_mm_yyyy() {
    let dm = DataMtr(d(2026, 3, 7));
    assert_eq!(serde_json::to_string(&dm).unwrap(), "\"07-03-2026\"");
    assert_eq!(dm.to_string(), "07-03-2026");
    // Round-trip e rejeição de formatos fora do contrato canônico.
    assert_eq!(
        serde_json::from_str::<DataMtr>("\"07-03-2026\"").unwrap(),
        dm
    );
    assert!(serde_json::from_str::<DataMtr>("\"2026-03-07\"").is_err());
}

#[test]
fn timestamp_ms_and_periodo_serialize() {
    let ts = TimestampMs(1_780_000_000_000);
    assert_eq!(serde_json::to_string(&ts).unwrap(), "1780000000000");
    let p = Periodo {
        inicio: DataMtr(d(2026, 6, 1)),
        fim: DataMtr(d(2026, 8, 10)),
    };
    assert_eq!(
        serde_json::to_string(&p).unwrap(),
        r#"{"dataInicial":"01-06-2026","dataFinal":"10-08-2026"}"#
    );
    // A leitura também aceita ISO 8601.
    let iso: Periodo =
        serde_json::from_str(r#"{"dataInicial":"2026-06-01","dataFinal":"2026-08-10"}"#).unwrap();
    assert_eq!(iso, p);
}

#[test]
fn periodo_accepts_rfc3339_and_epoch_ms() {
    // `JSON.stringify(new Date())` e epoch ms, reduzidos ao calendário UTC.
    // 1_780_000_000_000 ms = 2026-05-28T20:26:40Z; o fim é depois do início.
    let p: Periodo = serde_json::from_str(
        r#"{"dataInicial":"2026-05-01T00:00:00.000Z","dataFinal":1780000000000}"#,
    )
    .unwrap();
    assert_eq!(p.inicio, DataMtr(d(2026, 5, 1)));
    assert_eq!(p.fim, DataMtr(d(2026, 5, 28)));
}

#[test]
fn filtro_com_dias_usa_o_relogio() {
    let f: FiltroMtr = serde_json::from_str(r#"{"periodo":{"dias":7},"tamanho":5}"#).unwrap();
    let p = f.periodo.expect("período");
    let hoje = jiff::Timestamp::now()
        .to_zoned_datetime(jiff::tz::TimeZone::UTC)
        .date();
    assert_eq!(p.fim, DataMtr(hoje));
    assert_eq!(
        p.inicio,
        DataMtr(hoje.checked_sub(jiff::ToSpan::days(7)).expect("data"))
    );
}

#[test]
fn civil_datetimes_keep_iso_form() {
    let dt: DateTime = d(2026, 6, 1).at(13, 45, 7, 0);
    assert_eq!(
        serde_json::to_string(&dt).unwrap(),
        "\"2026-06-01T13:45:07\""
    );
    let ts = jiff::Timestamp::from_second(1_780_000_000).unwrap();
    assert_eq!(
        serde_json::to_string(&ts).unwrap(),
        "\"2026-05-28T20:26:40Z\""
    );
}
