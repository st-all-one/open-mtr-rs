# 08 — Atualização, migração e operação

> **Status:** etapa 8 (operação). **Depende de:** distribuição.
> **Alimenta:** cadeia de implementação e documentação final.
>
> Objetivo: atualizar binário/config com um comando, sem perder credenciais/artefatos e com
> rollback possível.

## 1. Superfície de atualização

```bash
openmtr self version
openmtr self update --check
openmtr self update --dry-run
openmtr self update
openmtr self update --version v0.2.0
openmtr config validate
openmtr config migrate --dry-run
```

`self update` é um **wrapper fino**:

1. resolve tag/versão (default `latest`);
2. localiza o script `scripts/openmtr-upgrade.sh` embutido ou no repo;
3. executa o script com `VERSION=<tag>`;
4. o script chama o `install.sh` oficial com checksum;
5. o binário novo é verificado por `self version`;
6. em caso de falha, o binário antigo permanece (instalação atômica via staging + rename).

Não reimplementar download/descompactação/checksum em Rust: a fonte da verdade é o `install.sh`, como
no `_KNUDGE`.

## 2. Fluxo de release

```text
1. make check / make ci / make doc verde
2. editar versão (make update-version VERSION=vX.Y.Z)
3. commit + tag vX.Y.Z + push
4. GitHub Actions release:
   verify (tag == Cargo.toml, check, doc, supply-chain)
   build (6 targets)
   package (tar.gz/zip + sha256sums.txt + config example)
   release (GitHub Release)
5. usuário: openmtr self update
```

## 3. `self update` — contrato

| Modo | Comportamento |
|---|---|
| `--check` | consulta a última versão e imprime `current`, `latest`, `up_to_date`; não instala |
| `--dry-run` | imprime target, URL, versão, diretório e checksum esperado; não baixa |
| default | baixa, verifica checksum, instala, verifica execução |
| `--version <TAG>` | instala/atualiza para a tag específica; permite downgrade explícito |
| `--force` | reinstala a mesma versão (reparo) |
| `--from-source` | build local via cargo (dev) |

Saída:

- humano: logs em stderr (`current`, `latest`, `downloading`, `checksum ok`, `installed`);
- `--json`: envelope `{success, command:"self.update", data:{current,latest,from,to,target,changed}}`;
- `--check` não modifica arquivos; exit 0 mesmo se desatualizado (informação), `--strict` pode
  retornar 1 se houver atualização? **Decisão:** v1 exit 0; scripts usam `jq` no envelope.

## 4. Migração de config

### 4.1 Quando roda

- `self update` **não** altera config automaticamente.
- Após atualização, o usuário roda:

```bash
openmtr config validate
openmtr config migrate --dry-run
openmtr config migrate
```

- O binário pode detectar `config_version` antiga no primeiro comando e falhar com instrução,
  sem migrar silenciosamente.

### 4.2 Garantias

- `migrate --dry-run` imprime diff canônico (stdout), não grava.
- `migrate` grava backup `open-mtr-config.toml.bak.<timestamp>` antes de escrever.
- Migração é determinística: mesma entrada ⇒ mesmo TOML canônico.
- Campos desconhecidos são preservados com warning (forward compatibility) ou erro em `strict`.
- Segredos inline são preservados, mas redigidos no diff.
- `config_version` mais nova que o binário ⇒ erro `schema` com instrução de atualizar.

### 4.3 Compatibilidade

| Versão do binário | `config_version` suportada |
|---|---|
| v0.2.x | 1 |
| v0.3.x | 1, 2 (leitura + migração) |
| futuro | N-1 por pelo menos uma minor |

## 5. Rollback

### 5.1 Binário

`install.sh` move o binário anterior para
`${XDG_CACHE_HOME:-~/.cache}/openmtr/trash/openmtr.<timestamp>` antes de instalar.

Rollback manual:

```bash
OPENMTR_VERSION=vX.Y.Z openmtr self update
```

ou restaurar do trash com instrução impressa pelo installer.

### 5.2 Config

- Migração cria backup datado.
- `openmtr config migrate --rollback` (futuro) restaura o backup mais recente; na v1, restaurar
  manualmente o `.bak` é documentado.
- Nunca apagar config em update/uninstall.

## 6. Operação e troubleshooting

| Sintoma | Diagnóstico | Ação |
|---|---|---|
| `config missing` | `openmtr config show --json` | `config init`; conferir `--config`/`OPENMTR_CONFIG` |
| `secret env missing` | `config validate` | exportar env ou usar inline local |
| `auth` (10) | `account test --id` | conferir CPF/CNPJ/unidade/senha; MTR-PE exige CNPJ |
| `timeout` (6) | `--timeout`, `--concurrency` | reduzir concorrência, testar `health` |
| `rate limited` | relatório de dump | reduzir `max_concurrency`, usar `--resume` |
| dump lento | stderr/progresso | ajustar `--window-days`, `--concurrency`, `--detail-batch-size` |
| PDFs inválidos | `pdfs/manifest.jsonl` | refazer; verificar magic bytes/mime |
| `schema` (8) | `--params`/TOML | validar JSON/TOML; ver linha/coluna |
| `unsafe` (9) | permissões do config | `config fix-permissions` |
| atualização falhou | `self update --dry-run` | verificar target/URL/checksum; instalar manualmente |
| binário não executa | `--version`/`self version` | verificar arquitetura/linker; reinstalar target correto |

## 7. Observabilidade

- Logs em stderr com campos:
  - `command`, `account`, `system`, `operation`;
  - `window`, `page`, `batch_index`, `attempt` no dump;
  - `duration_ms` apenas em logs (não em dados determinísticos).
- Nunca logar payloads completos; no máximo tamanho/contagens.
- `--log-level debug` mostra resolução de config/conta, sem segredos.
- `RUST_LOG` compatível com `tracing_subscriber::EnvFilter`; `--log-level` tem precedência.
- `--quiet` = logs off, mas erros fatais ainda em stderr.

## 8. Atualização da documentação

Cada release deve atualizar:

- `README.md` (instalação/quickstart do CLI);
- `wiki/plan/cli/README.md` (status/marco);
- `wiki/usage/cli/*` (novo conjunto de docs de uso, se criado);
- `CHANGELOG.md` (breaking changes de config/comando);
- `open-mtr-config.example.toml` no pacote;
- `install.sh` (targets/versão, se necessário).

## 9. Política de quebra

- **Config:** mudança incompatível exige `config_version` novo + migração.
- **CLI:** remover/renomear verbo/flag exige major; `--help`/CHANGELOG apontam substituto.
- **Saída:** campos de JSON/CSV são aditivos; remover campo exige major + `schema_version` no
  relatório de dump.
- **Exit codes:** nunca renumerar códigos existentes; novos códigos apenas em faixa livre.

## 10. Critérios de aceite da operação

- [ ] `self update --check` funciona sem rede? **Não**: precisa consultar release; erro de rede é
  6 com mensagem clara.
- [ ] `self update --dry-run` imprime plano sem baixar.
- [ ] `self update` instala via install.sh com checksum e valida execução.
- [ ] Config existente é preservada em install/update/uninstall.
- [ ] `config migrate --dry-run` não escreve.
- [ ] Backup criado antes de migração.
- [ ] Rollback documentado e testado com fixture de versão anterior.
- [ ] Nenhum segredo em log de update/migração.
- [ ] Docs de troubleshooting cobrem os códigos de erro.
