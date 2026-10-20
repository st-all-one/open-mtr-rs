---
id: decision_00onpkew
type: decision
statement: "Reduzir a duplicação entre request, request_raw e post_public"
created_at: 2026-10-20T02:08:04.807Z
body_hash: 80eabcfc
schema_version: 2
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-20T02:25:11.692Z
    notes: "request/request_raw/post_public compartilham execute com retry, refresh, auth e limiter unicos"
anchors: [UPDATE.md, core/src/openmtr/client/http.rs]
status: closed
scope: task
---
<!-- knudge:parent epic_013avsgb -->