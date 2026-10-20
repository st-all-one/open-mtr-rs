# UPDATE — Revisão de qualidade do openMTR

> Revisão focada em **determinismo, simplicidade e otimização**, feita após a migração do
> projeto para o padrão de qualidade do knudge (Rust 1.99, clippy, deny, rustfmt, lints de
> workspace).
>
> Memória knudge:
> - épico: `epic_013avsgb`
> - decisão/registro da revisão: `decision_00vtlqtz`
> - tarefas: `kd task list --scope epic_013avsgb`

## Resumo executivo

O projeto está com o portão de qualidade verde (`make check`), mas a revisão encontrou pontos
que afetam **reprodutibilidade**, **segurança de credenciais**, **consumo desnecessário de
recursos** e **manutenção**. Os itens estão priorizados abaixo e foram registrados como tarefas
no `kd`.

Prioridade imediata:

1. corrigir a compilação sem `native-http` (duas definições de `sleep`);
2. tornar determinística a ordem dos resultados do MTR-PE;
3. remover o fallback de CNPJ para `integration_token`;
4. separar refresh token de access token no `TokenManager`;
5. propagar erro de serialização no retry 401.

---

## 1. Bloqueadores / correções imediatas

### 1.1 `--no-default-features` não compila

Há duas definições de `sleep` para o mesmo `cfg` em
`core/src/openmtr/client/http.rs`:

```rust
#[cfg(all(not(feature = "native-http"), not(target_arch = "wasm32")))]
async fn sleep(d: Duration) { std::thread::sleep(d); }

#[cfg(all(not(feature = "native-http"), not(target_arch = "wasm32")))]
async fn sleep(d: Duration) { std::thread::sleep(d); }
```

Verificação:

```bash
cargo check -p open-mtr-core --no-default-features
# error[E0428]: the name `sleep` is defined multiple times
```

Além da duplicação, `std::thread::sleep` dentro de `async fn` bloqueia o executor. A correção
deve manter uma única definição e usar um timer compatível com o runtime, ou exigir
`native-http` no host.

**Tarefa:** `error_008ynryk` — Corrigir compilação sem native-http e unificar a função sleep.

### 1.2 Ordem do MTR-PE não é determinística

Em `core/src/openmtr/adapters/mtr_pe.rs`, a consulta por período usa
`buffer_unordered(CONCORRENCIA)`. Os itens chegam na ordem de conclusão, e o `indice` é
descartado por `exec_lote`.

Resultado: duas execuções equivalentes podem retornar `dados` em ordens diferentes.

Correções possíveis:

- usar `buffered(n)`, que preserva a ordem de entrada; ou
- ordenar por `indice` antes de montar o `LoteResult`; para múltiplos subitens por requisição,
  usar uma chave estável `(indice, subindice)`.

**Tarefa:** `risk_00smv7un` — Tornar determinística a ordem dos resultados do MTR-PE.

### 1.3 Fallback de CNPJ para token no MTR-PE

Em `core/src/openmtr/adapters/mtr_pe.rs`:

```rust
let cnpj = cnpj
    .or_else(|| ctx.config.cnpj.clone())
    .or_else(|| ctx.config.integration_token.clone())
    .unwrap_or_default();
```

Se o CNPJ não for informado, o token de integração/JWT pode ser enviado como `pessoaCnpj`.
Isso parece bug e potencial vazamento de credencial. Se o CNPJ é obrigatório, retornar erro de
validação; o token nunca deve ocupar esse campo.

**Tarefa:** `error_002wo0ky` — Remover o fallback de CNPJ para integration_token no MTR-PE.

### 1.4 Cache de token confunde refresh token e access token

`MtrHttpClient::new` chama `TokenManager::from_cached(integration_token, expires)`, e
`from_cached` grava o mesmo valor em `integration_token` e em `cached.access_token`.

Para SINIR isso funciona por acidente: o token do `.env` é um refresh token usado inicialmente
como access token até `autenticar_token()` chamar `refresh_token()`.

Riscos:

- `get_token()` pode devolver refresh token como se fosse access token;
- `force_refresh_data()` devolve o token cacheado como se fosse o token de integração;
- adapters que não autenticam antes de usar o cliente recebem token errado.

O modelo correto separa:

- `integration_token` / `refresh_token`;
- `cached_access_token`;
- `refresh_url`;
- `expires_at`.

**Tarefa:** `risk_00rmesux` — Separar refresh token e access token no TokenManager.

### 1.5 Retry 401 engole erro de serialização

Em `core/src/openmtr/client/http.rs`:

```rust
body: body.map(|b| serde_json::to_vec(b).unwrap_or_default()),
```

Se a serialização falhar, o retry envia corpo vazio em vez de propagar o erro. O corpo já foi
serializado antes; reutilizar o `Vec<u8>` ou usar `?`.

**Tarefa:** `error_00gh64i8` — Propagar erro de serialização no retry 401 do cliente HTTP.

---

## 2. Determinismo

### 2.1 UUID v7 e relógio em toda resposta/erro

`Resposta.id` e `OpenMTRError.instance` usam `Uuid::now_v7` / `Uuid::new_v7`. A mesma operação
produz JSON diferente a cada execução.

Isso é aceitável como correlation id, mas impede:

- snapshots/golden tests;
- comparação byte a byte de respostas;
- idempotência baseada no `id`;
- reprodutibilidade de logs.

Sugestão: portas para `Clock` e `IdGenerator`, com implementações determinísticas nos testes. No
mínimo, permitir injetar `instance`/`id` no `MtrCtx`.

**Tarefa:** `decision_01we5dnn` — Injetar Clock e IdGenerator para determinismo de UUID e
períodos default.

### 2.2 Datas default dependem do dia atual

Pontos em que a ausência de período vira “últimos 30 dias” ou “hoje”:

- `Periodo::ultimos_dias` (`core/src/openmtr/domain/mtr.rs`);
- deserialização `{"dias": N}`;
- `filtro_para_json` (`core/src/openmtr/adapters/sinir.rs`);
- `json_para_cdf_emitido` (`core/src/openmtr/adapters/sinir.rs`);
- `consultar_cdfs` do MTR-PE;
- `GerarCdf`/`data_emissao` em MTR-PE e SIGOR.

O mesmo JSON de entrada pode gerar requisições/saídas diferentes conforme o dia. Para o core
puro, o ideal é exigir período explícito e deixar atalhos como `dias` apenas nos bindings ou em
uma camada de conveniência.

### 2.3 `HashMap` em tabelas públicas

`COMPATIBILIDADE_ACOND` e `ESTADOS_FISICOS` (`core/src/openmtr/domain/tabela.rs`) são `HashMap`
públicos. Hoje o acesso é por chave, mas qualquer iteração/serialização futura terá ordem não
determinística.

Como são apenas 5 entradas, a solução mais simples é `BTreeMap` ou arrays/match fixos, que também
eliminam alocação e lookup.

### 2.4 Ordem de listas vem da API

Os métodos `listar_*` preservam a ordem do JSON retornado. Se a API não garante ordem estável, a
saída também não é estável. Definir contrato:

- ordenar por `codigo`, `numero` ou chave equivalente; ou
- documentar explicitamente que a ordem é a da API.

### 2.5 `is_timeout` por substring

`OpenMTRError::is_timeout` faz `detail.to_lowercase().contains("timeout")`. Isso:

- aloca uma `String` a cada chamada;
- pode classificar erro errado se a mensagem contiver “timeout” por acaso;
- não distingue timeout de rede, timeout de servidor etc.

Melhor: um `ErrorKind` tipado (`Network`, `Timeout`, `Auth`, `Validation`, `Config`, `Api`,
`NotFound`, `NotSupported`) e `is_timeout()` baseado no enum.

**Tarefa:** `decision_01gwm1qt` — Tipar OpenMTRError com ErrorKind e eliminar is_timeout por
substring.

---

## 3. Simplicidade / arquitetura

### 3.1 `request` e `request_raw` são quase duplicados

`core/src/openmtr/client/http.rs` repete praticamente toda a lógica em `request` e
`request_raw`:

- montagem de URL;
- injeção de Authorization;
- retry 429/5xx;
- refresh em 401;
- logging de duração;
- tratamento de timeout.

Extrair um núcleo comum, por exemplo:

```rust
async fn execute_with_retry<T>(
    &self,
    method: &str,
    path: &str,
    body: Option<Vec<u8>>,
    op: &str,
    decode: impl FnOnce(HttpResponse) -> Result<T, OpenMTRError>,
) -> Result<T, OpenMTRError>
```

**Tarefa:** `decision_00onpkew` — Reduzir a duplicação entre request, request_raw e post_public.

### 3.2 `max_rps` é configuração morta

`Config.max_rps` e `OpenMtrClientBuilder::max_rps` existem, mas não há rate limiter usando esse
valor. Em `open_mtr_auth.rs` ele é fixado em `10` sem efeito.

Ou implementar um token bucket/limiter compartilhado no `MtrHttpClient`, ou remover o campo para
não prometer algo que não acontece.

**Tarefa:** `decision_0060pago` — Implementar ou remover max_rps e limitar a concorrência dos
join_all.

### 3.3 `join_all` sem limite de concorrência

`join_all` é usado em SINIR, MTR-PE e SIGOR para downloads e consultas:

- `adapters/sinir.rs`;
- `adapters/mtr_pe.rs`;
- `adapters/sigor.rs`.

Para lotes grandes isso:

- estoura conexões/rate limit da API;
- aumenta memória;
- piora o tempo de resposta por contenção;
- pode gerar muitos 429/5xx.

Usar `stream::iter(tasks).buffered(n)` ou `buffer_unordered(n)` com coleta ordenada, conectando
`n` a uma política de concorrência/rate limit.

### 3.4 Duplicação do transporte WASM

Existem dois transportes de fetch:

- `core/src/openmtr/transport/fetch.rs`;
- `wasm/src/transport.rs`.

O da `wasm` reimplementa o do core com `spawn_local` + oneshot. Além disso, `HttpTransport` exige
`Send + Sync`, mas `fetch.rs` usa `#[async_trait(?Send)]`; isso é incompatível com o trait atual e
provavelmente deixa a feature `wasm-http` quebrada.

Opções:

- separar `HttpTransport` nativo e `LocalHttpTransport` WASM; ou
- remover a feature `wasm-http` do core e manter só o transporte do crate `wasm`.

**Tarefa:** `decision_01eznwt4` — Consolidar o transporte WASM e revisar a feature wasm-http.

### 3.5 Spans criados mas não entrados

Em quase todos os adapters há:

```rust
let _span = info_span!("sinir.consultar_mtrs");
```

Mas o span não é entered. Ele é criado e descartado, então não há contexto nos logs. Usar:

```rust
let _span = info_span!("sinir.consultar_mtrs").entered();
```

ou preferir `#[instrument]`.

**Tarefa:** `risk_004booii` — Entrar os spans de tracing nos adapters e no exec_lote.

### 3.6 `criar_manifesto_complementar` quebra o contrato do envelope

Em `open_mtr_client.rs`, erros do adapter são convertidos em `Ok(Resposta { ok: false, ... })`,
enquanto todos os outros métodos retornam `Err(OpenMTRError)` para erro catastrófico. Isso torna
o comportamento inconsistente e perde o `r#type` original do erro.

### 3.7 Duplicação de `Config`

`MtrCtx` tem `config: Config`, e `MtrHttpClient` também tem `config: Config`. O builder faz
`config.clone()`, duplicando `String`s de `base_url`, token etc.

Simplificação: `MtrCtx { http }` com `http.config()` ou `Arc<Config>` compartilhado.

### 3.8 Parsing de listas duplicado nos adapters

Os métodos `listar_residuos`, `listar_acondicionamentos`, `listar_classes` etc. repetem o mesmo
padrão:

- pegar `objetoResposta` ou array raiz;
- clonar o array;
- mapear com `json_para_*`.

Um helper genérico reduziria bastante o código e os pontos de divergência.

### 3.9 `base_url` sem validação/normalização

`format!("{}{path}", self.config.base_url)` pode gerar `//` se `base_url` terminar com `/`, ou URL
inválida se o usuário passar qualquer string. `Config::build` só verifica se é vazia.

Validar com `Url::parse` e normalizar barras, ou usar `Url::join`.

**Tarefa:** `risk_01xbs3yk` — Validar e normalizar base_url e limitar o TTL de tokens.

---

## 4. Otimizações concretas

### 4.1 `.cloned()` desnecessário em arrays JSON

Há muitos casos como:

```rust
let items = response
    .get("objetoResposta")
    .and_then(|a| a.as_array())
    .cloned()
    .unwrap_or_default();
```

Isso clona todo o `Vec<Value>`, incluindo strings e objetos, só para iterar por referência. Usar o
slice emprestado:

```rust
let items = response
    .get("objetoResposta")
    .and_then(Value::as_array)
    .map(Vec::as_slice)
    .unwrap_or_default();
```

**Tarefa:** `risk_00lytxhp` — Eliminar clones desnecessários em exec_lote, listas JSON e total_ok.

### 4.2 `exec_lote` clona erros desnecessariamente

```rust
let erros: Vec<OpenMTRError> = lote.erros().into_iter().cloned().collect();
```

Depois o código consome `lote.itens`. Melhor desestruturar uma única vez:

```rust
let LoteResult { itens, total } = lote;
let mut dados = Vec::with_capacity(itens.len());
let mut erros = Vec::new();

for item in itens {
    match item {
        ItemResult::Ok(v) => dados.push(v),
        ItemResult::Erro { problema, .. } => erros.push(problema),
    }
}
```

### 4.3 `total_ok`/`total_erros` alocam vetores

`total_ok` chama `self.sucessos().len()`, e `sucessos()` aloca um `Vec<&T>`. O mesmo para
`total_erros`.

Usar:

```rust
self.itens.iter().filter(|i| matches!(i, ItemResult::Ok(_))).count()
```

### 4.4 `TokenManager::extract_expiry` aloca `Vec`

```rust
let parts: Vec<&str> = jwt.split('.').collect();
```

Pode ser:

```rust
let (_, rest) = jwt.split_once('.')?;
let payload = rest.split('.').next()?;
```

### 4.5 TTL de token sem clamp/checked_add

Em `MtrHttpClient::new`, `set_token_direct` e `TokenManager::set_cached_token`, o TTL vem de
`exp - now` e é somado a `Instant::now()`. Um JWT com `exp` absurdo pode gerar `Duration` enorme
e potencial overflow/panic no `Instant + Duration`.

Usar teto (ex.: 24h ou 7 dias) e `checked_add`:

```rust
let expires = Instant::now()
    .checked_add(Duration::from_secs(ttl.min(MAX_TTL_SECS)))
    .unwrap_or_else(Instant::now);
```

### 4.6 `error_types::make_urn` parseia URL a cada erro

Cada erro chama `Url::parse(&format!(...))`. Como são poucos tipos fixos, usar `LazyLock<Url>` ou
trocar `Url` por um `ErrorKind` e serializar a URI sob demanda.

### 4.7 Serialização de body repetida

No `request`, o body é serializado antes do envio e novamente no retry 401. Serializar uma vez em
`Option<Vec<u8>>` e reutilizar.

### 4.8 Refresh de token sem single-flight

Várias requisições concorrentes podem chamar `refresh_token()` ao mesmo tempo, gerando múltiplos
POSTs de refresh. Um `tokio::sync::Mutex`/`OnceCell` por cliente resolve isso.

---

## 5. Outros pontos de contrato e robustez

### 5.1 `Quantidade(f64)` e igualdade exata

`Quantidade` usa `f64` com `PartialEq`. Para quantidades de resíduo, igualdade exata e
serialização decimal podem gerar diferenças de precisão. Se quantidades forem contrato de
negócio, considerar inteiros escalados (ex.: miligramas) ou um tipo decimal.

### 5.2 Normalização de status por string

`map_status_sinir`, `map_status_pe` e `map_status_sigor` fazem match em variações manuais
(`"ATIVO" | "Ativo"`). Isso é frágil. Normalizar uma vez (`trim`, `to_ascii_uppercase`) ou usar
`eq_ignore_ascii_case`.

### 5.3 `StatusMtr::Outro(String)` e serialização

`StatusMtr` tem `Serialize` manual via `Display`, mas `Deserialize` derivado. Garantir teste de
round-trip para todas as variantes, inclusive `Outro`.

---

## 6. Testes recomendados

1. **Build sem features**
   ```bash
   cargo check -p open-mtr-core --no-default-features
   ```
2. **Build WASM**
   ```bash
   cargo check -p open-mtr-wasm --target wasm32-unknown-unknown
   ```
3. **Ordem determinística**
   Teste com `MockTransport` atrasando respostas fora de ordem e verificar saída ordenada.
4. **Golden tests dos adapters**
   Fixtures JSON reais de SINIR/MTR-PE/SIGOR → saída serializada esperada, sem UUID/timestamp
   voláteis. Para isso, injetar `id`/`clock`.
5. **Concorrência do refresh**
   Várias chamadas simultâneas com token expirado devem gerar apenas um refresh.
6. **TTL absurdo**
   JWT com `exp = u64::MAX` não deve causar panic.

---

## 7. Tarefas registradas no `kd`

Épico: `epic_013avsgb` — Revitalização Rust 1.99 do openMTR: determinismo, simplicidade e
otimização.

| Ordem | ID | Tarefa | Arquivo principal |
|---:|---|---|---|
| 1 | `error_008ynryk` | Corrigir compilação sem native-http e unificar a função sleep | `core/src/openmtr/client/http.rs` |
| 2 | `risk_00smv7un` | Tornar determinística a ordem dos resultados do MTR-PE | `core/src/openmtr/adapters/mtr_pe.rs` |
| 3 | `error_002wo0ky` | Remover o fallback de CNPJ para integration_token no MTR-PE | `core/src/openmtr/adapters/mtr_pe.rs` |
| 4 | `risk_00rmesux` | Separar refresh token e access token no TokenManager | `core/src/openmtr/client/token.rs` |
| 5 | `error_00gh64i8` | Propagar erro de serialização no retry 401 do cliente HTTP | `core/src/openmtr/client/http.rs` |
| 6 | `decision_01gwm1qt` | Tipar OpenMTRError com ErrorKind e eliminar is_timeout por substring | `core/src/openmtr/error.rs` |
| 7 | `decision_0060pago` | Implementar ou remover max_rps e limitar a concorrência dos join_all | `core/src/openmtr/config.rs` |
| 8 | `decision_01we5dnn` | Injetar Clock e IdGenerator para determinismo de UUID e períodos default | `core/src/openmtr/time_util.rs` |
| 9 | `decision_00onpkew` | Reduzir a duplicação entre request, request_raw e post_public | `core/src/openmtr/client/http.rs` |
| 10 | `risk_00lytxhp` | Eliminar clones desnecessários em exec_lote, listas JSON e total_ok | `core/src/openmtr/open_mtr_client.rs` |
| 11 | `risk_004booii` | Entrar os spans de tracing nos adapters e no exec_lote | `core/src/openmtr/adapters/sinir.rs` |
| 12 | `decision_01eznwt4` | Consolidar o transporte WASM e revisar a feature wasm-http | `wasm/src/transport.rs` |
| 13 | `risk_01xbs3yk` | Validar e normalizar base_url e limitar o TTL de tokens | `core/src/openmtr/config.rs` |

Registro da revisão: `decision_00vtlqtz` — Revisão de determinismo, simplicidade e otimização do
openMTR está em UPDATE.md e no épico epic_013avsgb.

### Como consultar e atualizar

```bash
kd task list --scope epic_013avsgb
kd task show --id epic_013avsgb --history
kd task show --id error_008ynryk

# Ao concluir uma tarefa:
kd task close --id error_008ynryk --outcome success --note "make check verde; --no-default-features compila"

# Ao registrar evidência/contexto adicional:
kd write --summary "<fato ou decisão>" --type fact --anchor <PATH>

# Ao final:
kd sync --message "docs: revisão de qualidade e tarefas de revitalização"
```

## 8. Ordem sugerida de execução

1. `error_008ynryk` — corrigir build sem features.
2. `risk_00smv7un` — ordem determinística no MTR-PE.
3. `error_002wo0ky` — remover fallback CNPJ → token.
4. `risk_00rmesux` — separar refresh/access token.
5. `error_00gh64i8` — não engolir erro de serialização.
6. `decision_01gwm1qt` — tipar erros.
7. `decision_0060pago` — rate limit / concorrência.
8. `decision_01we5dnn` — Clock/IdGenerator.
9. `decision_00onpkew` — unificar HTTP.
10. `risk_00lytxhp` — remover clones.
11. `risk_004booii` — spans.
12. `decision_01eznwt4` — transporte WASM.
13. `risk_01xbs3yk` — validar URL/TTL.

## 9. Verificações de referência

```bash
make check
make ci
cargo check -p open-mtr-core --no-default-features
cargo check -p open-mtr-wasm --target wasm32-unknown-unknown
cargo test --workspace
```

O estado atual de `make check` e `make ci` está verde; os itens acima são melhorias e correções
identificadas na revisão, não regressões introduzidas pela migração de qualidade.
