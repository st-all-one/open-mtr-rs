---
id: decision_01we5dnn
type: decision
statement: Injetar Clock e IdGenerator para determinismo de UUID e períodos default
created_at: 2026-10-20T02:08:04.718Z
body_hash: 0d83ec35
schema_version: 2
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-20T02:25:11.652Z
    notes: MtrCtx aceita Clock e IdGenerator injetaveis; UUIDs e datas default usam o contexto
anchors: [UPDATE.md, core/src/openmtr/time_util.rs]
status: closed
scope: task
---
<!-- knudge:parent epic_013avsgb -->