---
id: error_002wo0ky
type: error
statement: Remover o fallback de CNPJ para integration_token no MTR-PE
created_at: 2026-10-20T02:08:04.298Z
body_hash: 3dc50fca
schema_version: 2
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-20T02:25:11.457Z
    notes: Fallback CNPJ->integration_token removido; CNPJ vazio agora retorna erro de validacao
anchors: [UPDATE.md, core/src/openmtr/adapters/mtr_pe.rs]
status: closed
scope: task
---
<!-- knudge:parent epic_013avsgb -->