---
id: risk_00vgp6bc
type: risk
statement: Periodo com atalho dias ainda resolve pelo relógio global na desserialização
created_at: 2026-10-20T02:31:52.771Z
body_hash: f927eef8
schema_version: 2
anchors: [core/src/openmtr/domain/mtr.rs]
---
Why: serde não recebe o Clock do MtrCtx ao desserializar {'dias': N}; o atalho é conveniência dos bindings.
Evidence: parâmetros default dos adapters já usam ctx.today() e Periodo::ultimos_dias_em permite data injetada, mas PeriodoRaw::Dias chama utc_now().
Consequence: para golden tests/reprocessamento use dataInicial/dataFinal explícitas ou ultimos_dias_em; reserve dias para atalho de borda.