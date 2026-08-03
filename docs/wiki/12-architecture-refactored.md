# Arquitetura — openMTR 0.2

Mapa completo da arquitetura após a refatoração `HttpTransport`
e integração WASM.

## Camadas

```
┌──────────────────────────────────────────────────────────────┐
│  Bindings                                                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────┐  │
│  │ napi-rs  │  │  uniffi  │  │   WASM   │  │  (futuros) │  │
│  │ Node.js  │  │ Swift/Kt │  │ Deno/Web │  │            │  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └─────┬──────┘  │
│       │              │             │               │         │
│       ▼              ▼             ▼               ▼         │
├──────────────────────────────────────────────────────────────┤
│  open-mtr-core (lib)                                         │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  OpenMtrClient ──► OpenMtrClientBuilder                 │ │
│  │       │                                                 │ │
│  │       ├──► MtrAdapter (trait)                           │ │
│  │       │    ├── SinirAdapter                             │ │
│  │       │    ├── MtrPeAdapter                             │ │
│  │       │    └── SigorAdapter                             │ │
│  │       │                                                 │ │
│  │       └──► MtrCtx                                       │ │
│  │            ├── Config                                   │ │
│  │            └── MtrHttpClient                            │ │
│  │                 ├── Arc<dyn HttpTransport>  ◄── trait   │ │
│  │                 └── TokenManager                        │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                               │
│  Transportes (implementam HttpTransport)                      │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │ ReqwestTransport │  │ FetchTransport  │  │ JsFetchTransp │ │
│  │ (native, default)│  │ (WASM browser)  │  │ (Deno/Bun)   │ │
│  │ reqwest + rustls │  │ gloo-net        │  │ globalThis   │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
├──────────────────────────────────────────────────────────────┤
│  Domain                                                       │
│  Mtr, Parte, Residuo, Cnpj, FiltroMtr, Periodo, Cdf, ...    │
│  Resposta<T>, Problema, OpenMTRError, System, Config         │
└──────────────────────────────────────────────────────────────┘
```

## Fluxo de uma Requisição

```
1. Usuário chama client.consultar_mtrs(&filtro)
          │
2. OpenMtrClient::exec_lote("consultar_mtrs", ...)
          │
3. adapter.consultar_mtrs(&ctx, filtro)
          │  constrói JSON do body específico do sistema
          │
4. ctx.http.post("/listaManifesto", &body, "consultar_mtrs")
          │
5. MtrHttpClient::request("POST", path, body, op)
          │
          ├──① get_token() → token JWT do cache ou refresh
          │
          ├──② HttpRequest { method, url, headers: [Auth], body }
          │     │
          │     └── transport.send(req).await ────► rede
          │
          ├──③ se 401 → refresh_token() → retry send()
          │
          ├──④ se 5xx/429/timeout → sleep(backoff) → retry send()
          │
          └──⑤ resp.json::<T>() → retorna ao adapter
          │
6. adapter mapeia resposta JSON → Vec<Mtr>
          │
7. exec_lote empacota em LoteResult<Mtr> → Resposta<Mtr>
```

## Workspace

```
open-mtr-rs/
├── Cargo.toml              # workspace root
├── core/
│   ├── Cargo.toml           # open-mtr-core (lib)
│   └── src/openmtr/
│       ├── domain/          # tipos universais
│       ├── adapters/        # sinir.rs, mtr_pe.rs, sigor.rs
│       ├── client/          # http.rs, token.rs
│       ├── transport/       # mod.rs (trait), reqwest.rs, fetch.rs
│       ├── config.rs        # Config, ConfigBuilder
│       ├── error.rs         # OpenMTRError (RFC 9457)
│       ├── resposta.rs      # Resposta<T>, Problema
│       ├── system.rs        # enum System
│       ├── time_util.rs     # utc_now() polyfill WASM
│       ├── open_mtr_client.rs  # OpenMtrClient + Builder
│       ├── open_mtr_auth.rs    # OpenMtrAuthBuilder (legado)
│       └── prelude.rs       # re-exports públicos
├── napi/
│   ├── Cargo.toml           # open-mtr-napi (cdylib)
│   └── src/lib.rs           # #[napi] OpenMtrClient
├── uniffi/
│   ├── Cargo.toml           # open-mtr-uniffi (cdylib)
│   └── src/lib.rs           # UniFFI exports
├── wasm/
│   ├── Cargo.toml           # open-mtr-wasm (cdylib, wasm32)
│   ├── src/
│   │   ├── lib.rs           # #[wasm_bindgen] OpenMtrWasm
│   │   ├── transport.rs     # JsFetchTransport
│   │   └── deno_shim.js     # Bridge JS (não utilizado na versão final)
│   ├── examples/main.ts     # Demo SINIR funcional
│   └── pkg/                 # Output do wasm-bindgen
└── docs/wiki/               # Documentação
```

## Feature Matrix

| Feature do core | Deps ativas | Target |
|-----------------|-------------|--------|
| `native-http` (default) | `reqwest`, `tokio` | Linux, macOS, Windows |
| `wasm-http` | `gloo-net`, `web-sys` | Browser WASM |
| _nenhum_ (no-default-features) | — | WASM com `JsFetchTransport` externo |

## Métricas

| Métrica | Valor |
|---------|-------|
| Testes unitários (native) | 79 |
| Testes live SINIR | 14 |
| Testes domain | 8 |
| Testes mock transport | 11 |
| **Total** | **101** |
| Tamanho WASM (release, LTO, stripped) | ~660 KB |
| Adapters implementados | 3 (SINIR, MTR-PE, SIGOR) |
| Targets de binding | 3 (napi, uniffi, wasm) |
| Linhas de código core | ~8,000 |

## Decisões Arquiteturais

| Decisão | Motivo |
|---------|--------|
| `Arc<dyn HttpTransport>` em vez de genérico `<T>` | Evita propagar parâmetro de tipo por toda a stack (OpenMtrClient, MtrCtx, MtrAdapter). Simplifica bindings. |
| `Send + Sync` no trait | Necessário para `Arc<dyn HttpTransport>` ser `Send + Sync`. WASM resolve com `spawn_local` bridge. |
| `TokenManager` sem lógica de HTTP | Permite que o refresh funcione com qualquer transporte. A lógica HTTP fica no `MtrHttpClient`. |
| `build_with_transport()` público | Qualquer crate externo pode injetar transporte customizado sem modificar o core. |
| Feature `native-http` default | Mantém compatibilidade — 99% dos usuários usam reqwest/tokio. WASM desativa explicitamente. |
| `std::sync::RwLock` em vez de `tokio::sync::RwLock` | Compatível com WASM (single-threaded). Leitura/escrita do token é rápida — não bloqueia. |
| JSON strings como formato de binding | Todos os bindings (napi, uniffi, wasm) serializam entrada/saída como JSON. Evita mapeamento complexo de tipos na FFI. |
