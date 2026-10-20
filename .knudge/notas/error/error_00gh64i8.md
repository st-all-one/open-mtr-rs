---
id: error_00gh64i8
type: error
statement: Propagar erro de serialização no retry 401 do cliente HTTP
created_at: 2026-10-20T02:08:04.449Z
body_hash: 0024a932
schema_version: 2
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-20T02:25:11.528Z
    notes: Body serializado uma unica vez e reutilizado no retry 401; erro de serializacao propagado
anchors: [UPDATE.md, core/src/openmtr/client/http.rs]
status: closed
scope: task
---
<!-- knudge:parent epic_013avsgb -->