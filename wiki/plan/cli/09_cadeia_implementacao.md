# 09 — Cadeia de implementação, marcos e aceite

> **Status:** etapa 9 (execução). **Depende de:** todas as etapas anteriores.
> **Objetivo:** transformar os planos em tarefas executáveis, com ordem, dependências, entregáveis
> e aceite verificável.

## 1. Visão geral

```text
CLI-00 Scaffold/workspace
   │
   ├── CLI-01 Config
   │      └── CLI-02 Accounts/auth
   │             └── CLI-03 Output/erros/logging
   │                    ├── CLI-04 Comandos
   │                    │     ├── CLI-05 Batch
   │                    │     └── CLI-06 Dump
   │                    └── CLI-07 Determinismo/segurança
   │                           └── CLI-08 Qualidade
   │                                  ├── CLI-09 Distribuição/release
   │                                  └── CLI-10 Self-update/migração
   │                                         └── CLI-11 Docs/aceite final
   └── (CLI-09 pode começar após CLI-03, em paralelo com CLI-04..06)
```

Cada tarefa só fecha com `make check` verde e teste/golden correspondente.

## 2. Tarefas

### CLI-00 — Scaffold do crate, workspace e gates

- **Escopo:** criar `cli/` (`open-mtr-cli` + bin `openmtr`), adicionar ao workspace,
  `default-members`, dependências iniciais, `main.rs` com `help`, `self version`, logging mínimo e
  `output` EPIPE-safe. Ajustar `scripts/check_file_length.sh` (300 para `cli`) e Makefile.
- **Entregáveis:** `cli/Cargo.toml`, `cli/src/main.rs`, `output`, `error`, `logging`;
  `Makefile` targets `cli`, `dist`; workspace `Cargo.toml`; CI passa a compilar/testar o crate.
- **Aceite:** `cargo run -p open-mtr-cli -- --version`; `openmtr` sem args = help exit 0;
  `make check` verde; nenhum arquivo >300 linhas; `cargo deny/machete` verdes.
- **Depende de:** nenhum (pode começar após aprovação deste plano).
- **Risco:** dependências novas; mitigar com justificativa e `deny`.

### CLI-01 — Config TOML, schema, segredos e validação

- **Escopo:** `ConfigFile`, `AccountConfig`, `Defaults`, `AuthConfig`; resolução de path; parse
  `toml`; validação estrutural/negócio; `token_env`/`senha_env`; redaction; permissões; `config
  init/validate/show/accounts/edit/migrate/fix-permissions`.
- **Entregáveis:** módulos `config/*`; fixtures de config; golden de `config validate/show`;
  `config init --from-env` (nomes do `_OPEN_MTR_API`).
- **Aceite:** fixtures válidas/inválidas da etapa 06 passam; duas contas `mtr_pe` convivem;
  `config show` nunca exibe segredo; `config init` cria `0600` no Unix.
- **Depende de:** CLI-00.
- **Risco:** schema virar complexo demais; manter v1 mínima e documentar questões em aberto.

### CLI-02 — Resolução de contas, autenticação e runtime de cliente

- **Escopo:** `Scope` (`--account/--system/--tag/--all-accounts`), resolução determinística,
  conflitos, `AccountRuntime` com `OnceCell<Arc<OpenMtrClient>>`, auth token vs
  `OpenMtrClassicAuth`, injeção de `Clock`/`IdGenerator` quando determinístico, `account list/test`.
- **Entregáveis:** `accounts/*`; testes de escopo; `account test` com wiremock; sem segredo em log.
- **Aceite:** escrita multi-conta é rejeitada; leitura ambígua é rejeitada; duas contas MTR-PE
  autenticam independentemente; `account test` retorna 0/10.
- **Depende de:** CLI-01.
- **Risco:** MTR-PE com CNPJ ausente; validação na config evita chamada.

### CLI-03 — Output, envelope, erros e logging

- **Escopo:** `CliError`/exit codes; `output::emit_stdout/emit_stderr` com EPIPE; envelope
  `{success, command, data, error, warnings}`; formatos `text|json|jsonl|csv`; `--quiet`,
  `--log-level`, `--no-color`; redaction no writer.
- **Entregáveis:** `output/*`, `error.rs`, `logging.rs`; testes de EPIPE, envelope golden,
  no-secret-in-output.
- **Aceite:** stdout limpo em todos os comandos; `--json` válido; EPIPE exit 0; erros mapeados.
- **Depende de:** CLI-02.
- **Risco:** logs vazando no stdout; teste de captura obrigatório.

### CLI-04 — Comandos MTR/CDF/REF/health

- **Escopo:** `mtr consultar/criar/receber/cancelar/modificar/aceitar/complementar/download`;
  `cdf consultar/gerar/download`; `ref listar/por-residuo/por-estado-fisico`; `health`;
  `--params`/`--params -`/conveniências; `--dry-run`.
- **Entregáveis:** `commands/*`; mapeamento para o core; testes wiremock por comando; golden de
  text/json/jsonl.
- **Aceite:** cada comando tem sucesso + erro + exit code; payload enviado confere; `ref` ordena.
- **Depende de:** CLI-03.
- **Risco:** duplicar normalização; usar tipos do core e só mapear flags.

### CLI-05 — Batch JSONL

- **Escopo:** parser JSONL, `op` fechado, conta por linha/global, `--dry-run`,
  `--continue-on-error`, saída JSONL por linha, ordem de entrada, concorrência apenas leitura.
- **Entregáveis:** `batch/*`; golden de batch; testes de linha inválida/op desconhecido.
- **Aceite:** lote misto lê/escrita com saída estável; `--dry-run` sem rede; exit 1 em falha
  parcial.
- **Depende de:** CLI-04.
- **Risco:** escrita concorrente; manter escrita sequencial na v1.

### CLI-06 — Dump multi-conta

- **Escopo:** pipeline da etapa 04; janelas, paginação SINIR, detalhe em lotes, MTR-PE por período,
  normalização `MtrNormalizada`, CSV/JSONL/relatório, resume, PDFs/manifest.
- **Entregáveis:** `dump/*`; fixtures SINIR + 2 MTR-PE; golden de `mtrs.jsonl`/CSV/relatório;
  testes de retomada e sanitização.
- **Aceite:** duas contas do mesmo sistema não se sobrescrevem; bytes estáveis com `--fixed-time`;
  `--resume` não duplica; PDF inválido não é gravado.
- **Depende de:** CLI-04 (e CLI-03 para output).
- **Risco:** volume/memória; streaming na escrita; concorrência limitada.

### CLI-07 — Determinismo, segurança e redaction hardening

- **Escopo:** `--fixed-time`/`SequentialIdGenerator` em todas as contas; normalização de
  `Resposta.id`; redução de campos voláteis; ordenação de listas/contas; redactor global;
  permissões; paths seguros; golden com clock fixo.
- **Entregáveis:** ajustes em `accounts/runtime`, `output`, `dump`, `batch`; testes
  `deterministic_*`, `no_secret_*`, `stable_order_*`.
- **Aceite:** duas execuções iguais geram bytes iguais; nenhum segredo em captura; listas
  ordenadas.
- **Depende de:** CLI-04, CLI-05, CLI-06.
- **Risco:** campos voláteis remanescentes; checklist no arquivo 05.

### CLI-08 — Qualidade, CI e matriz de testes

- **Escopo:** completar suíte da etapa 06; `make check`; CI `cli`/`cross-platform`/`supply-chain`;
  docs de teste; orçamento de linhas.
- **Entregáveis:** testes unitários/integração/golden/security; workflows atualizados; cobertura
  mínima dos comandos.
- **Aceite:** todos os itens da Definition of Done da etapa 06; matriz cross-platform verde.
- **Depende de:** CLI-07.
- **Risco:** CI caro; cache e jobs paralelos.

### CLI-09 — Empacotamento, install.sh e release CI/CD

- **Escopo:** `scripts/package.sh`, `install.sh`, `release.yml`, `build.yml`, `scripts/bump-version.sh`,
  `open-mtr-config.example.toml`, pacote com README/ licenses.
- **Entregáveis:** release com 6 targets + `sha256sums.txt`; instalação local testada; docs de
  distribuição.
- **Aceite:** `make dist` local; `install.sh` verifica checksum e executa `openmtr self version`;
  release workflow publica artefatos.
- **Depende de:** CLI-08 (pode começar após CLI-03 para scaffolding).
- **Risco:** musl/ARM/Windows; smoke por target.

### CLI-10 — `self update`, migração de config e operação

- **Escopo:** `self version/completions/update`; script `openmtr-upgrade.sh`; `config migrate`;
  backups/rollback; docs de troubleshooting/observabilidade.
- **Entregáveis:** wrapper fino, script embutido, testes com fixture de release local, docs.
- **Aceite:** `self update --dry-run` mostra plano; `self update` instala com checksum; config
  preservada; migração dry-run não escreve; backup criado.
- **Depende de:** CLI-09 e CLI-01.
- **Risco:** atualização parcial; staging + rename + verificação de execução.

### CLI-11 — Fecho: docs, exemplos, matriz e CHANGELOG

- **Escopo:** atualizar `README.md`, `wiki/plan/cli/*` (status), `wiki/usage`, `CHANGELOG`,
  `open-mtr-config.example.toml`; revisar comandos/help; grep por nomes obsoletos; atualizar
  `AGENTS.md` se necessário.
- **Entregáveis:** documentação final, exemplos de dump/batch, política de versão, matriz de aceite
  preenchida.
- **Aceite:** `make check`, `make ci`, `make doc` verdes; todos os marcos M0–M4 marcados; nenhum
  `TODO`/stub na superfície.
- **Depende de:** CLI-10.
- **Risco:** docs divergirem da superfície; revisão com `rg` e golden.

## 3. Dependências resumidas

| Tarefa | Depende de | Pode paralelizar com |
|---|---|---|
| CLI-00 | — | — |
| CLI-01 | CLI-00 | CLI-09 (scaffold) |
| CLI-02 | CLI-01 | — |
| CLI-03 | CLI-02 | — |
| CLI-04 | CLI-03 | CLI-09 |
| CLI-05 | CLI-04 | CLI-06 |
| CLI-06 | CLI-04 | CLI-05 |
| CLI-07 | CLI-04/05/06 | CLI-08 preparação |
| CLI-08 | CLI-07 | CLI-09 docs |
| CLI-09 | CLI-08 (ou CLI-03 para scaffold) | CLI-04..06 |
| CLI-10 | CLI-09, CLI-01 | CLI-11 docs |
| CLI-11 | tudo | — |

## 4. Marcos

| Marco | Critério de saída | Tarefas |
|---|---|---|
| **M0 — Fundação** | `config validate`, `account list`, envelope/exit básicos com fixture | CLI-00..03 |
| **M1 — Operações** | todos os comandos `mtr/cdf/ref/health` testados com wiremock | CLI-04 |
| **M2 — Volume** | dump multi-conta reproduzível + batch | CLI-05, CLI-06 |
| **M3 — Robustez** | determinismo/segurança + suíte completa + CI cross-platform | CLI-07, CLI-08 |
| **M4 — Distribuição** | install/release/self update/migração + docs | CLI-09..11 |

## 5. Definition of Done global

- [ ] `make check` verde (fmt, clippy, test, file-length).
- [ ] `make ci` verde (deny, machete, typos; audit quando disponível).
- [ ] `make doc` verde.
- [ ] `cargo test -p open-mtr-cli` verde em Linux/macOS/Windows.
- [ ] `cargo build --release -p open-mtr-cli` verde nos 6 targets da release.
- [ ] `cli/` ≤300 linhas por arquivo; nenhum `unwrap/expect/panic/unsafe`.
- [ ] Config multi-conta validada, incluindo 2 contas do mesmo sistema.
- [ ] Saída determinística com `--fixed-time` e ordenação estável.
- [ ] Nenhum segredo em stdout/stderr/relatório/CSV/JSONL.
- [ ] Comandos documentados; help embutido; exemplos de `--params`/stdin/heredoc.
- [ ] Dump multi-conta com JSONL/CSV/relatório/resume/PDF.
- [ ] Batch JSONL com dry-run/continue-on-error/ordem.
- [ ] `install.sh` com checksum; `self update` funcional; config preservada.
- [ ] Release CI publica binários + `sha256sums.txt` + config example.
- [ ] CHANGELOG, README e wiki/plan/cli sincronizados.

## 6. Matriz de aceite por comando

| Comando | Sucesso | Erro | Escopo | JSON | Stdin | Determinístico |
|---|---|---|---|---|---|---|
| `config init/validate/show/accounts/edit/migrate` | ✓ | ✓ | n/a | ✓ | n/a | ✓ |
| `account list/show/test` | ✓ | ✓ | ✓ | ✓ | n/a | ✓ |
| `mtr consultar` | ✓ | ✓ | leitura | ✓ | ✓ | ✓ |
| `mtr criar/receber/cancelar/modificar/aceitar/complementar` | ✓ | ✓ | escrita única | ✓ | ✓ | ✓ |
| `mtr download` | ✓ | ✓ | escrita única | ✓ | ✓ | arquivo/hash |
| `cdf consultar/gerar/download` | ✓ | ✓ | leitura/escrita | ✓ | ✓ | ✓ |
| `ref listar/por-*` | ✓ | ✓ | leitura | ✓ | n/a | ✓ |
| `health` | ✓ | ✓ | leitura | ✓ | n/a | latência omitida |
| `dump` | ✓ | ✓ | leitura multi | arquivos | ✓ | ✓ |
| `batch` | ✓ | ✓ | por linha | JSONL | ✓ | ✓ |
| `self version/completions/update` | ✓ | ✓ | n/a | ✓ | n/a | ✓ |

## 7. Riscos e mitigação

| Risco | Impacto | Mitigação |
|---|---|---|
| Config mal desenhada virar dívida | alto | contrato congelado + migração versionada |
| Duplicação do dump.ts | alto | reutilizar core; módulo de dump só normaliza/orquestra |
| Multi-conta misturar clientes | alto | runtime por conta + testes de escopo |
| Saída instável | médio | fixed clock/id + golden + ordenação |
| Segredo em log/artefato | alto | redactor + testes de captura + permissões |
| Release quebrar em target | médio | matriz nativa + smoke `self version` |
| Superfície crescer | médio | v1 congelada; aliases proibidos |
| Testes caros/lentos | médio | wiremock + fixtures sintéticas + cache CI |
| Dependências novas | médio | `deny`/`machete` + justificativa por crate |

## 8. Rastreabilidade com `kd`

Ao iniciar a execução, criar no `kd`:

- épico raiz: “CLI utilitária multi-conta do openMTR”;
- uma issue por marco M0–M4;
- uma task por `CLI-xx` (ou agrupar CLI-00..03 em M0, CLI-04 em M1, CLI-05/06 em M2,
  CLI-07/08 em M3, CLI-09..11 em M4);
- âncoras: `wiki/plan/cli/01_contrato_config.md`, `.../03_superficie_cli.md`,
  `.../04_dump_e_batch.md`, `.../07_distribuicao_ci_cd.md`, `cli/src/**`.

Fechar task só com evidência: `make check`/`make ci` + teste/golden correspondente.

## 9. Ordem recomendada de PRs

1. **PR 1 — Fundação:** CLI-00 + CLI-01 (scaffold + config) + testes de config.
2. **PR 2 — Contas e borda:** CLI-02 + CLI-03 (auth/scope/output) + envelope golden.
3. **PR 3 — Operações:** CLI-04 (mtr/cdf/ref/health) + wiremock.
4. **PR 4 — Batch:** CLI-05 + JSONL golden.
5. **PR 5 — Dump:** CLI-06 + fixtures/golden de dump.
6. **PR 6 — Determinismo/segurança:** CLI-07 + testes de redaction/ordem.
7. **PR 7 — Qualidade/CI:** CLI-08.
8. **PR 8 — Distribuição:** CLI-09 + install.sh/release.
9. **PR 9 — Atualização/migração:** CLI-10.
10. **PR 10 — Fecho:** CLI-11 + docs/CHANGELOG/release notes.

## 10. Critérios de corte (MVP)

Se for necessário reduzir escopo:

- **MVP mínimo:** CLI-00..04 (config multi-conta, auth, operações, output determinístico).
- **MVP de volume:** + CLI-05/06 (batch + dump).
- **MVP de distribuição:** + CLI-09 (install/release); `self update`/migração pode ficar em
  v0.2.x se o install manual funcionar.
- **Não cortar:** escopo de escrita única, redaction, determinismo de dump.
