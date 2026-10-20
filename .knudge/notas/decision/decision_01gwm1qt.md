---
id: decision_01gwm1qt
type: decision
statement: Tipar OpenMTRError com ErrorKind e eliminar is_timeout por substring
created_at: 2026-10-20T02:08:04.545Z
body_hash: 0831df6b
schema_version: 2
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-20T02:25:11.562Z
    notes: OpenMTRError ganhou ErrorKind tipado; is_timeout nao usa mais substring
anchors: [UPDATE.md, core/src/openmtr/error.rs]
status: closed
scope: task
---
<!-- knudge:parent epic_013avsgb -->