---
id: risk_01xbs3yk
type: risk
statement: Validar e normalizar base_url e limitar o TTL de tokens
created_at: 2026-10-20T02:08:05.148Z
body_hash: 825062a7
schema_version: 2
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-20T02:25:11.824Z
    notes: base_url validada/normalizada em Config::endpoint; TTL de token limitado a 7 dias com checked_add
anchors: [UPDATE.md, core/src/openmtr/config.rs]
status: closed
scope: task
---
<!-- knudge:parent epic_013avsgb -->