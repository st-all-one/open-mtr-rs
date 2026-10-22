# CLI utilitária do openMTR — cadeia de planos

> **Status:** planejamento aprovado para execução incremental.
> **Binário alvo:** `openmtr`.
> **Arquivo de configuração:** `open-mtr-config.toml`.
> **Inspiração funcional:** `_OPEN_MTR_API/scripts/dump.ts` + contrato de borda do `_KNUDGE`.
>
> Esta pasta é a **fonte da verdade do produto CLI**. Cada arquivo é uma etapa da cadeia; a ordem
> dos arquivos é a ordem de dependência.

## 1. Objetivo

Criar uma CLI utilitária, scriptável e determinística para operar os sistemas MTR suportados pelo
`open-mtr-core`, com:

- uma configuração cumulativa em `open-mtr-config.toml`, com **várias credenciais** e **mais de uma
  conta do mesmo sistema**;
- seleção explícita de escopo (`--account`, `--system`, `--tag`, `--all-accounts`);
- comandos operacionais equivalentes ao core (`mtr`, `cdf`, `ref`, `health`);
- um comando `dump` inspirado no `deno task dump` do `_OPEN_MTR_API`, mas genérico para N contas;
- modo `batch` com JSONL, `--params`, stdin, heredoc e stdout limpo;
- saída de máquina estável, logs em stderr e redação de segredos;
- processo de instalação/atualização simples, com binários pré-compilados, checksums e CI/CD.

O produto final deve permitir tanto uso humano quanto uso por LLM/automação:

```bash
openmtr --config ./open-mtr-config.toml mtr consultar --account mtrpe-empresa \
  --params '{"periodo":{"dataInicial":"01-05-2026","dataFinal":"31-05-2026"}}'

cat <<'JSON' | openmtr mtr consultar --account sinir-prod --params -
{"periodo":{"dataInicial":"01-05-2026","dataFinal":"31-05-2026"},"detalhado":true}
JSON

openmtr dump --all-accounts --from 2026-05-01 --to 2026-05-31 --out dump/
openmtr batch --file operacoes.jsonl --continue-on-error > resultados.jsonl
```

## 2. O que foi investigado

| Fonte | O que foi extraído |
|---|---|
| `_OPEN_MTR_API/scripts/dump.ts` | fluxo real de dump multi-sistema, retries, janelas de 30 dias, normalização, CSV/JSONL, PDF, relatório, resumo de completude, caso MTR-PE transportadora como segunda conta do mesmo sistema |
| `_OPEN_MTR_API/lib/*.ts` | autenticação por env, cache de JWT, clientes WASM por sistema, envelope de resposta |
| `_OPEN_MTR_API/routes/v1/*.ts` | catálogo de operações MTR/CDF/tabelas/health e parâmetros usados |
| `_KNUDGE/wiki/specs/cli.md` | contrato stdout=dados/stderr=logs, `--json`, exit codes, EPIPE, `--params`, stdin/heredoc |
| `_KNUDGE/plan/implementation/16_cli_surface.md` | desenho de superfície com poucos verbos, subcomandos e convenções de argumentos |
| `_KNUDGE/plan/implementation/22_comandos_scriptados.md` | wrapper fino para atualização, scripts versionados, checksum |
| `_KNUDGE/.github/workflows/{ci,release}.yml` | matriz de binários, gate de versão, checksums, release |
| `_KNUDGE/install.sh` | instalador `curl | bash`, verificação SHA-256, upgrade idempotente |
| `open-mtr-rs/core` | fachada `OpenMtrClient`, `OpenMtrClassicAuth`, `Config`, `System`, tipos de domínio |

## 3. Decisões estruturantes

| Id | Decisão |
|---|---|
| **CLI-D01** | A CLI é um **adaptador fino** sobre `open-mtr-core`; nenhuma regra de domínio nova vive no CLI. |
| **CLI-D02** | O arquivo `open-mtr-config.toml` é único e versionado pelo usuário; contém `[[accounts]]` cumulativas e declara explicitamente o sistema de cada conta. |
| **CLI-D03** | Contas são identificadas por `id` estável. **Nunca** usar `system` como chave única: várias contas do mesmo sistema são cidadãs de primeira classe. |
| **CLI-D04** | Leitura exige escopo inequívoco: uma conta, várias contas explícitas, um sistema inteiro ou `--all-accounts`. Escrita exige **exatamente uma conta**. |
| **CLI-D05** | `stdout` é dados; `stderr` é log/progresso. `--json` emite envelope de máquina. `dump` pode emitir JSONL/CSV diretamente no stdout. |
| **CLI-D06** | `--params <JSON>` é universal; `--params -` lê de stdin; `--batch <JSONL|->` executa lotes. Conteúdo posicional e stdin seguem a convenção do `kd`. |
| **CLI-D07** | Toda saída de dados é determinística: ordenação estável, sem UUID/timestamp volátil fora de campos explicitamente voláteis; relógio e IDs injetáveis para golden tests. |
| **CLI-D08** | Segredos nunca vão para stdout/stderr. Config aceita segredo inline ou via `*_env`; inline gera aviso e checagem de permissão do arquivo. |
| **CLI-D09** | `dump` é um comando de primeira classe, derivado do `dump.ts`, mas dirigido por contas do config em vez de env vars fixas. |
| **CLI-D10** | Atualização é `openmtr self update` → script fino → `install.sh` com checksum SHA-256; releases são gerados por tag e matriz de CI. |
| **CLI-D11** | Nenhum comando de escrita roda implicitamente em todas as contas. `--all-accounts` é inválido para mutações. |
| **CLI-D13** | Dump multi-conta deduplica MTRs por chave (`numero`/`barcode`) com prioridade por tipo de acesso da conta: transportadora (0) > destinadora/receptora (1) > geradora (2); `--no-dedup` preserva duplicatas. |
| **CLI-D14** | `self update` baixa o artefato da release e verifica SHA-256 diretamente em Rust (staging + rename atômico), em vez de executar `install.sh` remotamente; o `install.sh` continua sendo o caminho de bootstrap. |

## 4. Cadeia de planos

| Ordem | Arquivo | Entrega | Depende de |
|---:|---|---|---|
| 0 | [`00_diagnostico.md`](00_diagnostico.md) | inventário, gaps, requisitos e riscos | — |
| 1 | [`01_contrato_config.md`](01_contrato_config.md) | contrato de `open-mtr-config.toml`, contas, segredos e escopos | 00 |
| 2 | [`02_arquitetura.md`](02_arquitetura.md) | crate, módulos, portas, erros, runtime e dependências | 01 |
| 3 | [`03_superficie_cli.md`](03_superficie_cli.md) | verbo/subcomandos, flags globais, matriz comando→core | 01, 02 |
| 4 | [`04_dump_e_batch.md`](04_dump_e_batch.md) | pipeline de dump multi-conta e protocolo de batch | 03 |
| 5 | [`05_determinismo_e_seguranca.md`](05_determinismo_e_seguranca.md) | contrato de dados, redação, ordenação e golden | 03, 04 |
| 6 | [`06_qualidade_testes.md`](06_qualidade_testes.md) | matriz de testes, fixtures, gates e critérios de aceite | 02–05 |
| 7 | [`07_distribuicao_ci_cd.md`](07_distribuicao_ci_cd.md) | workspace, release matrix, empacotamento e install | 06 |
| 8 | [`08_atualizacao_operacao.md`](08_atualizacao_operacao.md) | `self update`, migração de config, rollback e operação | 07 |
| 9 | [`09_cadeia_implementacao.md`](09_cadeia_implementacao.md) | tarefas executáveis, marcos, aceite final e riscos | todos |

## 5. Rastreabilidade no `kd`

Épico raiz: `epic_01k7qz1b` — “Implementar CLI utilitária multi-conta do openMTR (openmtr)”.

| Issue | Marco | Âncora principal |
|---|---|---|
| `task_008lrh5v` | M0 — Fundação (CLI-00..03) | [`01_contrato_config.md`](01_contrato_config.md) |
| `task_002bl333` | M1 — Operações (CLI-04) | [`03_superficie_cli.md`](03_superficie_cli.md) |
| `task_018zlnjv` | M2 — Dump/batch (CLI-05/06) | [`04_dump_e_batch.md`](04_dump_e_batch.md) |
| `task_008e8tq8` | M3 — Determinismo/qualidade (CLI-07/08) | [`05_determinismo_e_seguranca.md`](05_determinismo_e_seguranca.md) |
| `task_00yip7w4` | M4 — Distribuição/update (CLI-09..11) | [`07_distribuicao_ci_cd.md`](07_distribuicao_ci_cd.md) |

Consulta: `kd task list --scope epic_01k7qz1b`.

## 6. Marcos

| Marco | Escopo | Critério de saída |
|---|---|---|
| **M0 — Fundação** | crate, config, auth, resolução de contas, erros, output | `openmtr account list` e `openmtr config validate` funcionam com fixture local |
| **M1 — Operações** | `mtr`, `cdf`, `ref`, `health`, `--params`, stdin | wiremock cobre os comandos e exit codes |
| **M2 — Volume** | dump multi-conta + batch JSONL, JSONL/CSV/report, PDFs, retomada | dump de duas contas do mesmo sistema é reproduzível; batch misto com resultado JSONL |
| **M3 — Robustez** | determinismo, redaction, ordenação, suíte/golden, CI cross-platform | bytes estáveis com clock fixo; nenhum segredo em captura; suíte verde |
| **M4 — Distribuição** | release matrix, install.sh, `self update`, migração, docs | instalação em Linux/macOS/Windows a partir de release assinada por checksum |

## 7. Regras de ouro do projeto CLI

- **Rust 1.99 / Edition 2024**, MSRV igual ao workspace.
- `make check` verde em cada fase: fmt, clippy `-D warnings`, testes, orçamento de linhas.
- Arquivos de produção ≤ 300 linhas; o gate transitório atual não deve ser aumentado.
- Sem `unwrap`/`expect`/`panic` em produção. Testes/fixtures podem usar atalhos explícitos.
- Sem `unsafe` no CLI.
- Dependência nova só com justificativa, licença compatível e passagem por `cargo deny`/`machete`.
- Nenhum teste de CI depende de credencial real; live tests continuam opt-in.
- Todo contrato de bytes (config, envelope, JSONL de dump/batch) tem golden test.
- Documentação, nomes de comandos e mensagens em português; identificadores em inglês quando natural.

## 8. Não-objetivos

- Substituir a API Deno (`_OPEN_MTR_API`) ou o crate WASM; o CLI usa o core Rust nativo.
- Implementar novos sistemas estaduais fora dos adapters existentes (`SINIR`, `MTR-PE`, `SIGOR`).
- Ser daemon/servidor, manter cache persistente global ou banco de dados.
- Reimplementar a lógica de `dump.ts` linha a linha: o CLI deve reaproveitar o core e isolar apenas a
  orquestração e a normalização de saída.
- Suportar edição interativa de config em TUI.

## 9. Como atualizar esta cadeia

1. Mudou o contrato de config? Atualize [`01_contrato_config.md`](01_contrato_config.md) primeiro.
2. Mudou comando/flag? Atualize [`03_superficie_cli.md`](03_superficie_cli.md) e a matriz de aceite
   em [`09_cadeia_implementacao.md`](09_cadeia_implementacao.md).
3. Mudou saída/segurança? Atualize [`05_determinismo_e_seguranca.md`](05_determinismo_e_seguranca.md).
4. Mudou release/instalação? Atualize [`07_distribuicao_ci_cd.md`](07_distribuicao_ci_cd.md) e
   [`08_atualizacao_operacao.md`](08_atualizacao_operacao.md).
5. Ao iniciar código, crie/feche tarefas no `kd` ancoradas nos arquivos desta pasta.

## 10. Status de implementação

Versão atual do CLI: **0.1.0** (`[workspace.package].version`, herdada por `open-mtr-cli`; `openmtr --version`).
Release publica assets `openmtr-v<versão>-<target>.{tar.gz,zip}` + `sha256sums.txt`, compilados em
runners nativos (sem Docker/cross), com portão de tag == Cargo.toml.

Crate `cli/` (`open-mtr-cli`, binário `openmtr`), verificado por `make check` e testes de integração.

| Área | Estado | Evidência |
|---|---|---|
| M0 — config, contas, `account list/test`, `config validate` | implementado | `cli/src/config.rs`, `cli/src/account.rs` |
| M1 — `mtr consultar/criar/receber/cancelar/modificar/aceitar/download`, `cdf consultar/gerar/download`, `ref listar`, `health` | implementado | `cli/src/cli/` |
| M1 — `--params`, `--params -` (stdin), `--json`, leitura multi-conta | implementado | `cli/src/params.rs` |
| M2 — dump multi-conta, janelas, JSONL/CSV/residuos.csv/relatório | implementado | `cli/src/dump/`, `cli/tests/dump_pipeline.rs` |
| M2 — dedup por `access_type` (transportadora 0 > destinadora 1 > geradora 2), `--no-dedup` | implementado | `cli/src/dump/dedup.rs` (testes unitários) |
| M2 — retomada `--resume`/`--force`, estado atômico | implementado | `cli/src/dump/` |
| M2 — PDFs com validação `%PDF`, manifesto SHA-256 | implementado | `cli/src/dump/` |
| M2 — batch JSONL com execução real, `--dry-run`, `--continue-on-error` | implementado | `cli/src/batch.rs`, `cli/tests/batch_test.rs` |
| M3 — `--fixed-time`, IDs sequenciais, ordenação estável, golden de bytes | implementado | `cli/tests/dump_pipeline.rs` (determinismo) |
| M3 — config estrito, aviso de segredo inline, checagem de permissão | implementado | `cli/src/config.rs` |
| M3 — `config migrate` com `--dry-run` e backup datado | implementado | `cli/src/migrate.rs` |
| M4 — `self version/update` com `--check`/`--dry-run`, checksum e rename atômico | implementado | `cli/src/selfupdate.rs`, `cli/tests/selfupdate_test.rs` |
| M4 — `install.sh`, workflow de release 6 targets, config de exemplo | implementado (não exercitado em CI local) | `install.sh`, `.github/workflows/release.yml`, `open-mtr-config.example.toml` |

Limitações conhecidas: SIGOR não participa de `dump` por período (conta única → erro `usage`;
dump misto → conta listada em `skipped`); `ref listar` exige uma conta; extração `.zip` do
`self update` no Windows ainda não suportada; workflow de release não exercitado localmente.

