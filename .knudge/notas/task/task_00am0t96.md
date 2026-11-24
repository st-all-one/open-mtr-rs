---
id: task_00am0t96
type: task
statement: Dividir adapters sinir/sigor/mtr_pe em modulos de ate 300 linhas
created_at: 2026-11-23T22:23:10.641Z
body_hash: b5272a24
schema_version: 3
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-11-23T23:38:32.575Z
    notes: "adapters/*, client/http, domain/{newtypes,mtr}, error, open_mtr_client e cli/* divididos; gate de 300 linhas no make check"
anchors: [core/src/openmtr/adapters/mod.rs]
status: closed
scope: task
---
<!-- knudge:parent epic_013avsgb -->