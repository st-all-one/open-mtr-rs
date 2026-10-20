---
id: risk_004booii
type: risk
statement: Entrar os spans de tracing nos adapters e no exec_lote
created_at: 2026-10-20T02:08:04.977Z
body_hash: f5bc8767
schema_version: 2
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-20T02:25:11.761Z
    notes: Spans dos adapters entram via guard e exec_lote abre o span da operacao
anchors: [UPDATE.md, core/src/openmtr/adapters/sinir.rs]
status: closed
scope: task
---
<!-- knudge:parent epic_013avsgb -->