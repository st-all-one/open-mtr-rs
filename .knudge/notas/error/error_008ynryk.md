---
id: error_008ynryk
type: error
statement: Corrigir compilação sem native-http e unificar a função sleep
created_at: 2026-10-20T02:07:54.042Z
body_hash: 31cd7915
schema_version: 2
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-20T02:25:11.387Z
    notes: make check verde; --no-default-features compila com uma unica sleep (sem bloquear o executor)
anchors: [UPDATE.md, core/src/openmtr/client/http.rs]
status: closed
scope: task
---
Tornar determinística a ordem dos resultados do MTR-PE	core/src/openmtr/adapters/mtr_pe.rs	risk
Remover o fallback de CNPJ para integration_token no MTR-PE	core/src/openmtr/adapters/mtr_pe.rs	error
Separar refresh token e access token no TokenManager	core/src/openmtr/client/token.rs	risk
Propagar erro de serialização no retry 401 do cliente HTTP	core/src/openmtr/client/http.rs	error
Tipar OpenMTRError com ErrorKind e eliminar is_timeout por substring	core/src/openmtr/error.rs	decision
Implementar ou remover max_rps e limitar a concorrência dos join_all	core/src/openmtr/config.rs	decision
Injetar Clock e IdGenerator para determinismo de UUID e períodos default	core/src/openmtr/time_util.rs	decision
Reduzir a duplicação entre request, request_raw e post_public	core/src/openmtr/client/http.rs	decision
Eliminar clones desnecessários em exec_lote, listas JSON e total_ok	core/src/openmtr/open_mtr_client.rs	risk
Entrar os spans de tracing nos adapters e no exec_lote	core/src/openmtr/adapters/sinir.rs	risk
Consolidar o transporte WASM e revisar a feature wasm-http	wasm/src/transport.rs	decision
Validar e normalizar base_url e limitar o TTL de tokens	core/src/openmtr/config.rs	risk
<!-- knudge:parent epic_013avsgb -->