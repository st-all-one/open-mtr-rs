# openMTR — Índice da Documentação

## Links Rápidos

| Recurso | Local |
|---------|-------|
| README principal | [README.md](../../README.md) |
| Guia do agente | [AGENTS.md](../AGENTS.md) |
| Código fonte | `core/src/openmtr/` |
| Testes | `core/tests/` |

---

## Documentação Atual (Arquitetura openMTR 0.2)

| Arquivo | Tópico |
|---------|--------|
| [09-http-transport.md](09-http-transport.md) | Trait `HttpTransport`, `MtrHttpClient`, `TokenManager`, feature flags, polyfills WASM |
| [10-bindings.md](10-bindings.md) | Targets de binding: napi-rs (Node.js), UniFFI (Swift/Kotlin/Python), WASM (Deno/Web) |
| [11-wasm-deno.md](11-wasm-deno.md) | Integração WASM/Deno: `JsFetchTransport`, build pipeline, API exportada, exemplo funcional |
| [12-architecture-refactored.md](12-architecture-refactored.md) | Mapa completo da arquitetura: camadas, fluxo de requisição, workspace, decisões, métricas |

---

## Wiki (Arquivo — Arquitetura Antiga `mtr-sinir`)

Os documentos abaixo descrevem a arquitetura anterior à refatoração
para openMTR. Mantidos para referência histórica.

| Arquivo | Tópico |
|---------|--------|
| [01-config-client.md](01-config-client.md) | MtrConfig + MtrClient (antigo) |
| [02-schemas.md](02-schemas.md) | Schemas SINIR (antigos) |
| [03-services.md](03-services.md) | Service layer (antigo) |
| [04-endpoint.md](04-endpoint.md) | trait Endpoint (antigo) |
| [05-errors.md](05-errors.md) | MtrError (antigo) |
| [06-newtypes.md](06-newtypes.md) | Cnpj, Cpf, DataMtr (movido para domain/) |
| [07-cache.md](07-cache.md) | Cache (removido) |
| [08-testing.md](08-testing.md) | Testes antigos |
