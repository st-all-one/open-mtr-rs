use super::*;

#[test]
fn windows_respect_bounds_and_step() {
    let from = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let to = NaiveDate::from_ymd_opt(2026, 5, 31).unwrap();
    let w = windows(30, from, to);
    assert_eq!(w.len(), 2);
    assert_eq!(w[0], (from, NaiveDate::from_ymd_opt(2026, 5, 30).unwrap()));
    assert_eq!(w[1], (NaiveDate::from_ymd_opt(2026, 5, 31).unwrap(), to));
}

#[test]
fn parse_date_accepts_iso_and_brazilian() {
    let expected = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    assert_eq!(parse_date("2026-05-01").unwrap(), expected);
    assert_eq!(parse_date("01-05-2026").unwrap(), expected);
}
