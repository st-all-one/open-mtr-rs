---
id: decision_01eznwt4
type: decision
statement: Consolidar o transporte WASM e revisar a feature wasm-http
created_at: 2026-10-20T02:08:05.065Z
body_hash: e027b208
schema_version: 2
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-20T02:25:11.792Z
    notes: Feature wasm-http e transport/fetch.rs removidos do core; open-mtr-wasm compila no target wasm32
anchors: [UPDATE.md, wasm/src/transport.rs]
status: closed
scope: task
---
<!-- knudge:parent epic_013avsgb -->