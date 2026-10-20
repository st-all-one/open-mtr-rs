---
id: decision_01i3h5xu
type: decision
statement: "CLI openmtr tem cadeia de planos em wiki/plan/cli com config multi-conta, dump e distribuição"
created_at: 2026-10-20T02:54:27.814Z
body_hash: aac06f2d
schema_version: 2
anchors: [wiki/plan/cli/README.md, wiki/plan/cli/01_contrato_config.md, wiki/plan/cli/04_dump_e_batch.md, wiki/plan/cli/09_cadeia_implementacao.md]
---
Why: consolidar o comportamento do deno task dump e o contrato de borda do kd em uma CLI Rust nativa, scriptável e determinística.
Evidence: _OPEN_MTR_API/scripts/dump.ts (multi-sistema, retries, janelas, CSV/JSONL, PDFs), _KNUDGE/wiki/specs/cli.md e plan/implementation/22_comandos_scriptados.md.
Consequence: execução começa por CLI-00..03 (M0), com config open-mtr-config.toml contendo [[accounts]] e N contas por sistema; épico kd epic_01k7qz1b.