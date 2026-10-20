---
id: risk_00rmesux
type: risk
statement: Separar refresh token e access token no TokenManager
created_at: 2026-10-20T02:08:04.384Z
body_hash: 02b84968
schema_version: 2
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-20T02:25:11.494Z
    notes: TokenManager separa refresh token de access token; set_token_direct preserva o refresh token
anchors: [UPDATE.md, core/src/openmtr/client/token.rs]
status: closed
scope: task
---
<!-- knudge:parent epic_013avsgb -->