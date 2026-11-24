use super::*;

#[test]
fn windows_respect_bounds_and_step() {
    let from = jiff::civil::date(2026, 5, 1);
    let to = jiff::civil::date(2026, 5, 31);
    let w = windows(30, from, to);
    assert_eq!(w.len(), 2);
    assert_eq!(w[0], (from, jiff::civil::date(2026, 5, 30)));
    assert_eq!(w[1], (jiff::civil::date(2026, 5, 31), to));
}

#[test]
fn parse_date_accepts_iso_and_brazilian() {
    let expected = jiff::civil::date(2026, 5, 1);
    assert_eq!(parse_date("2026-05-01").unwrap(), expected);
    assert_eq!(parse_date("01-05-2026").unwrap(), expected);
}
