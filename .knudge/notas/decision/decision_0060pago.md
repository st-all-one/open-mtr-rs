---
id: decision_0060pago
type: decision
statement: Implementar ou remover max_rps e limitar a concorrência dos join_all
created_at: 2026-10-20T02:08:04.623Z
body_hash: b5a51d52
schema_version: 2
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-20T02:25:11.603Z
    notes: max_rps agora limita concorrencia no cliente HTTP e os lotes usam buffered(max_concurrency)
anchors: [UPDATE.md, core/src/openmtr/config.rs]
status: closed
scope: task
---
<!-- knudge:parent epic_013avsgb -->