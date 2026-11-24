---
id: risk_00g2u3uv
type: risk
statement: Migrar chrono para time/jiff e remover ignore RUSTSEC-2026-0543
created_at: 2026-11-23T22:23:10.305Z
body_hash: e269ec69
schema_version: 3
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-11-24T00:09:50.642Z
    notes: chrono->jiff 0.3.0; contrato de bytes travado por date_bytes.rs e mtr_pe_request_bytes.rs; ignore RUSTSEC-2026-0543 removido
anchors: [Cargo.toml]
status: closed
scope: task
---
<!-- knudge:parent epic_013avsgb -->