---
id: decision_01ol0asr
type: decision
statement: "Revitalização usa Instrument em spans async, single-flight no get_token e IdGenerator para instance de erro"
created_at: 2026-10-20T02:31:52.591Z
body_hash: de2c761f
schema_version: 2
anchors: [core/src/openmtr/client/http.rs, core/src/openmtr/open_mtr_client.rs, core/src/openmtr/adapters/sinir.rs, core/src/openmtr/adapters/mtr_pe.rs, core/src/openmtr/adapters/sigor.rs]
---
Why: guards de span e refresh concorrente não podem vazar contexto entre tarefas; erros também precisam ser determinísticos quando há Clock/IdGenerator injetados.
Evidence: make check, make ci, make doc, --no-default-features e wasm32 verdes; teste concurrent_get_token_refreshes_once passa.
Consequence: adapters usam #[instrument], get_token renova uma única vez em concorrência e OpenMtrClient reatribui instance dos erros com ctx.new_id().