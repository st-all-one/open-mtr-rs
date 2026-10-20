---
id: risk_00smv7un
type: risk
statement: Tornar determinística a ordem dos resultados do MTR-PE
created_at: 2026-10-20T02:07:59.274Z
body_hash: 54dd1831
schema_version: 2
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-20T02:25:11.424Z
    notes: "MTR-PE por periodo usa stream::buffered, preservando a ordem dos barcodes"
anchors: [UPDATE.md, core/src/openmtr/adapters/mtr_pe.rs]
status: closed
scope: task
---
<!-- knudge:parent epic_013avsgb -->