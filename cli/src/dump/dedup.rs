use super::DedupRecord;
use crate::normalize::DumpRow;
use std::collections::BTreeMap;

pub(super) fn deduplicate(rows: Vec<DumpRow>) -> (Vec<DumpRow>, Vec<DedupRecord>) {
    let mut groups: BTreeMap<String, Vec<DumpRow>> = BTreeMap::new();
    for row in rows {
        groups.entry(row.dedup_key()).or_default().push(row);
    }
    let mut kept = Vec::with_capacity(groups.len());
    let mut records = Vec::new();
    for (key, mut group) in groups {
        group.sort_by(|a, b| {
            (a.access_priority, &a.account, &a.numero, &a.barcode).cmp(&(
                b.access_priority,
                &b.account,
                &b.numero,
                &b.barcode,
            ))
        });
        let winner = group.remove(0);
        let dropped: std::collections::BTreeSet<String> = group
            .iter()
            .map(|r| r.account.clone())
            .filter(|a| *a != winner.account)
            .collect();
        if !dropped.is_empty() {
            records.push(DedupRecord {
                key,
                kept_account: winner.account.clone(),
                dropped_accounts: dropped.into_iter().collect(),
            });
        }
        kept.push(winner);
    }
    (kept, records)
}

#[cfg(test)]
mod tests {
    use super::*;
    use open_mtr_core::openmtr::domain::Mtr;

    fn row(account: &str, numero: &str, priority: u8) -> DumpRow {
        let mtr = Mtr {
            numero: numero.to_string(),
            ..Mtr::default()
        };
        DumpRow::from_mtr(account, "sinir", priority, mtr)
    }

    #[test]
    fn keeps_transportadora_over_geradora() {
        let rows = vec![
            row("geradora", "1", 2),
            row("transportadora", "1", 0),
            row("destinadora", "1", 1),
            row("geradora", "2", 2),
        ];
        let (kept, dedup) = deduplicate(rows);
        assert_eq!(kept.len(), 2);
        let one = kept.iter().find(|r| r.numero == "1").unwrap();
        assert_eq!(one.account, "transportadora");
        assert_eq!(dedup.len(), 1);
        assert_eq!(dedup[0].kept_account, "transportadora");
        let mut dropped = dedup[0].dropped_accounts.clone();
        dropped.sort();
        assert_eq!(dropped, vec!["destinadora", "geradora"]);
    }

    #[test]
    fn tie_breaks_by_account_id() {
        let rows = vec![row("b-geradora", "9", 2), row("a-geradora", "9", 2)];
        let (kept, _) = deduplicate(rows);
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].account, "a-geradora");
    }

    #[test]
    fn same_account_duplicates_do_not_create_record() {
        let rows = vec![row("a", "1", 0), row("a", "1", 0)];
        let (kept, dedup) = deduplicate(rows);
        assert_eq!(kept.len(), 1);
        assert!(dedup.is_empty());
    }
}
