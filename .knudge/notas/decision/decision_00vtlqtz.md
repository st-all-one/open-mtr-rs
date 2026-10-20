---
id: decision_00vtlqtz
type: decision
statement: "Revisão de determinismo, simplicidade e otimização do openMTR está em UPDATE.md e no épico epic_013avsgb"
created_at: 2026-10-20T02:08:15.393Z
body_hash: 4bdfa674
schema_version: 2
classification: foundational
anchors: [UPDATE.md, core/src/openmtr/client/http.rs, core/src/openmtr/adapters/mtr_pe.rs]
---
Why: a revisão identificou riscos de build, ordem não determinística, credencial usada como CNPJ e duplicação de retry/token.
Evidence: UPDATE.md lista os achados; o épico epic_013avsgb agrupa 13 tarefas priorizadas.
Consequence: executar as tarefas na ordem do UPDATE.md e fechar cada uma com --outcome.
