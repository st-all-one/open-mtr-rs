# 14 — CLI `openmtr`

> Guia operacional do binário `openmtr`: instalação, autenticação, entrada,
> comandos, `batch`, `dump`, saída e distribuição. O contrato detalhado do dump
> continua em [`13-cli-dump.md`](13-cli-dump.md); a referência de flags gerada a
> partir do binário está em [`14-cli-reference.md`](14-cli-reference.md).

O `openmtr` é o adaptador de linha de comando do `open-mtr-core`. Ele cobre as
operações universais de MTR/CDF, as tabelas de referência, o `dump` multi-conta
e o `batch` JSONL. Não há regra de domínio no CLI: autenticação, retry e
normalização ficam no core.

---

## 1. Instalação e atualização

### 1.1 Release (recomendado)

O instalador detecta SO/arquitetura, baixa o asset da release, confere o
`sha256sums.txt` e instala de forma atômica em `~/.local/bin` (ou no diretório
escolhido):

```bash
curl --proto '=https' --tlsv1.2 -fsSL \
  https://raw.githubusercontent.com/st-all-one/open-mtr-rs/main/install.sh | bash
openmtr --version
```

Opções e variáveis úteis:

```bash
# versão específica
curl ... | OPENMTR_VERSION=v0.2.0 bash

# destino, prefixo e target explícito
./install.sh --install-dir "$HOME/bin"
./install.sh --prefix "$HOME/.local"
./install.sh --version v0.2.0 --target x86_64-unknown-linux-musl

# não alterar rc files/PATH
./install.sh --no-path

# remover o binário (vai para o lixo recuperável do cache)
./install.sh --uninstall
```

Variáveis aceitas: `OPENMTR_VERSION`, `OPENMTR_INSTALL_DIR`,
`OPENMTR_TARGET`, `OPENMTR_REPO`, `OPENMTR_BASE_URL`, `OPENMTR_CACHE_DIR` e
`OPENMTR_NO_PATH=1`. O binário anterior é movido para
`${XDG_CACHE_HOME:-$HOME/.cache}/openmtr/trash`; nada é apagado de forma
irreversível.

### 1.2 A partir do source

Dentro do repositório:

```bash
./install.sh --from-source
# equivale a:
cargo build --release --locked -p open-mtr-cli
```

O modo source instala `target/release/openmtr` (ou `openmtr.exe`) no mesmo
destino do modo release. Requer Rust 1.100+ e o repositório completo.

### 1.3 Atualização pelo próprio binário

```bash
openmtr self version
openmtr self update --check       # compara com a última release
openmtr self update --dry-run     # mostra asset/URL/checksum
openmtr self update               # baixa, valida SHA-256 e substitui
openmtr self update --version v0.2.0   # aceita também 0.2.0
```

`OPENMTR_VERSION` força a tag; `OPENMTR_API_URL`/`OPENMTR_RELEASES_URL` mudam
os endpoints (útil para testes). A extração `.zip` no Windows ainda não é
suportada por `self update`; use o `install.sh` em Git Bash/WSL ou instale
manualmente.

### 1.4 Assets publicados

| Plataforma | Asset |
|---|---|
| Linux x86_64 | `openmtr-vX.Y.Z-x86_64-unknown-linux-musl.tar.gz` |
| Linux ARM64 | `openmtr-vX.Y.Z-aarch64-unknown-linux-musl.tar.gz` |
| macOS Apple Silicon | `openmtr-vX.Y.Z-aarch64-apple-darwin.tar.gz` |
| macOS Intel | `openmtr-vX.Y.Z-x86_64-apple-darwin.tar.gz` |
| Windows x86_64 | `openmtr-vX.Y.Z-x86_64-pc-windows-msvc.zip` |
| Windows ARM64 | `openmtr-vX.Y.Z-aarch64-pc-windows-msvc.zip` |

Cada release também publica `sha256sums.txt` e a referência gerada
`openmtr-cli-vX.Y.Z.md`.

---

## 2. Primeira execução

Há dois modos de credencial:

1. **Sessão efêmera** com `--login`: um único JSON, sem config.
2. **Config multi-conta** em `open-mtr-config.toml` (default), com `--account`,
   `--system`, `--tag` ou `--all-accounts`.

### 2.1 Sessão efêmera (`--login`)

O objeto é validado antes de qualquer chamada; segredos são redigidos em erros
e logs.

```bash
# SINIR: refresh token
openmtr --login '{"system":"sinir","token":"..."}' \
  mtr consultar 2114014590

# SINIR lendo o token de uma variável de ambiente
openmtr --login '{"system":"sinir","token_env":"SINIR_TOKEN"}' \
  ref listar residuo

# MTR-PE: credenciais clássicas (cnpj/cpf/unidade/senha)
openmtr --login '{"system":"mtr_pe","cnpj":"...","cpf":"...","unidade":3615,"senha_env":"MTRPE_SENHA"}' \
  mtr consultar --params '{"periodo":{"dataInicial":"01-05-2026","dataFinal":"31-05-2026"}}'

# SIGOR: token direto ou credenciais clássicas
openmtr --login '{"system":"sigor","token":"..."}' ref listar classe
```

Campos aceitos: `system`, `token`, `token_env`, `cpf`, `cnpj`, `unidade`,
`senha`, `senha_env`, `base_url`, `timeout_secs` (1..=3600) e `max_rps`
(1..=32). Use `token` **ou** `token_env`, `senha` **ou** `senha_env`; não ambos.
`--login @arquivo.json` lê de arquivo e `--login -` lê do stdin.

`--login` é exclusivo com `--account`, `--system`, `--tag`,
`--all-accounts` e com os comandos `config`, `account` e `self`. Também não
combina com `--params -`/`--file -`, pois só um consumidor pode ler o stdin.

### 2.2 Config multi-conta

```bash
cp open-mtr-config.example.toml open-mtr-config.toml
chmod 600 open-mtr-config.toml
openmtr config validate
openmtr account list
```

Estrutura resumida:

```toml
config_version = 1

[defaults]
dump_window_days = 30
max_concurrency = 10
timeout_secs = 30

[[accounts]]
id = "sinir-prod"
system = "sinir"
access_type = "transportadora"     # prioridade 0 no dump
token_env = "SINIR_TOKEN"
tags = ["prod"]

[[accounts]]
id = "mtrpe-empresa"
system = "mtr_pe"
access_type = "geradora"           # prioridade 2 no dump
cpf = "00000000000"
senha_env = "MTRPE_SENHA"
unidade = 1
cnpj = "00000000000000"
```

`access_type` aceita `transportadora` (prioridade 0), `destinadora`/
`receptora` (1) e `geradora` (2). A prioridade decide quem vence a deduplicação
do dump. Prefira `token_env`/`senha_env`; segredo inline gera aviso e o arquivo
deve ser `0600`.

### 2.3 Cache de sessão

O login é trocado por um access token curto e reaproveitado entre operações,
inclusive dentro de um `batch`:

- SINIR: refresh token → access token (`POST /token`);
- MTR-PE/SIGOR clássicos: credenciais → JWT (`/gettoken`);
- token direto em MTR-PE/SIGOR: usado como está, sem cache de login.

O cache dura 5 minutos (ou menos, se o JWT vencer antes) e fica em memória +
arquivo `0600` sob `$XDG_RUNTIME_DIR/openmtr/session` (fallback
`$XDG_CACHE_HOME/openmtr/session` ou `~/.cache/openmtr/session`).
`OPENMTR_NO_SESSION_CACHE=1` desliga somente o arquivo; o cache em memória do
processo continua. `OPENMTR_SESSION_DIR` sobrescreve o diretório.

---

## 3. Flags globais

| Flag | Default | Descrição |
|---|---|---|
| `--config <ARQ>` | `open-mtr-config.toml` | Config multi-conta |
| `--login <JSON\|@ARQ\|->` | — | Sessão efêmera; dispensa `--config` |
| `--json` | desligado | JSON de máquina (pretty) em vez do JSON compacto |
| `--fixed-time <DATA>` | relógio real | Congela relógio/IDs para saída determinística |
| `--quiet` | desligado | Não inicializa logs |
| `--log-level <NÍVEL>` | `warn`/`RUST_LOG` | Filtro de `tracing` |
| `-h`, `--help` | — | Ajuda |
| `-V`, `--version` | — | Versão do binário |

`--fixed-time` aceita RFC 3339 ou `YYYY-MM-DD`. É útil para golden tests e para
reproduzir o dump byte a byte.

---

## 4. Contrato de entrada

### 4.1 Posicionais

Quando o comando tem alvo primário, ele é **posicional** e exclusivo com
`--params`:

```bash
openmtr mtr consultar 2114014590
openmtr mtr download 2114014590 --out pdfs/
openmtr cdf consultar 123456
openmtr ref listar residuo
openmtr ref por-residuo F001
openmtr ref por-estado-fisico 3
```

Nenhum comando lê stdin implicitamente. Para ler de stdin, use `-` explícito.

### 4.2 `--params`

`--params '<JSON>'` é o payload do comando:

- consultas: filtros (`numeros`, `periodo`, `parte`, `detalhado`, ...);
- escritas: objeto ou array aceito pelo método correspondente do core;
- `--params -`: lê o JSON do stdin (heredoc/pipe).

```bash
# consulta por período
openmtr --login '{"system":"sinir","token":"..."}' mtr consultar \
  --params '{"periodo":{"dataInicial":"01-09-2026","dataFinal":"30-09-2026"},"detalhado":true}'

# criação via heredoc
cat <<'JSON' | openmtr --login '{"system":"sinir","token":"..."}' mtr criar --params -
[{ "gerador": {}, "transportador": {}, "destinador": {}, "residuos": [] }]
JSON

# download em lote
openmtr --login '{"system":"sinir","token":"..."}' mtr download \
  --params '{"numeros":["2114014590","2114014591"]}' --out pdfs/
```

Leituras de `numeros`/`codigos` são estritas: item não-string é erro, não é
descartado silenciosamente.

### 4.3 Escopo de conta

Leituras aceitam `--account` (repetível), `--system`, `--tag` e
`--all-accounts`. Sem escopo, uma única conta habilitada é usada; com várias, o
comando falha com código 2. Escritas exigem exatamente uma conta (`--account`
ou `--login`); `--login` + `--account` é erro.

```bash
openmtr mtr consultar 2114014590 --account sinir-prod
openmtr dump --all-accounts --from 2026-09-01 --to 2026-09-30
openmtr dump --system mtr_pe --tag prod --from 2026-09-01 --to 2026-09-30
```

---

## 5. Comandos

### 5.1 Conta e configuração

```bash
openmtr account list                 # id, sistema, access_type, tags
openmtr account test sinir-prod      # valida a conta
openmtr config validate              # valida schema/credenciais/escopos
openmtr config migrate --dry-run     # simula migração de config_version
openmtr config migrate               # migra e cria backup datado
```

`account` e `config` não aceitam `--login`; use `--config`.

### 5.2 MTR

| Comando | Entrada | Saída |
|---|---|---|
| `mtr consultar [NUMERO]` | posicional ou `--params` | envelope `Resposta<Mtr>` por conta |
| `mtr criar --params` | um objeto ou array | `Resposta<MtrCriado>` |
| `mtr receber --params` | um objeto ou array | `Resposta<MtrRecebido>` |
| `mtr cancelar --params` | um objeto ou array | `Resposta<MtrCancelado>` |
| `mtr modificar --params` | um objeto ou array | `Resposta<MtrModificado>` |
| `mtr aceitar --params` | um objeto ou array | `Resposta<AlteracaoAceita>` |
| `mtr download [NUMERO]` | posicional ou `--params {numeros}` + `--out` | lista de arquivos baixados |

Exemplos:

```bash
openmtr mtr consultar 2114014590 --json
openmtr mtr consultar --params '{"numeros":["2114014590"]}' --account sinir-prod
openmtr mtr download 2114014590 --out pdfs/
```

### 5.3 CDF

| Comando | Entrada | Saída |
|---|---|---|
| `cdf consultar [CODIGO]` | posicional ou `--params` | envelope `Resposta<Cdf>` |
| `cdf gerar --params` | objeto ou array | `Resposta<CdfEmitido>` |
| `cdf download --params --out` | objeto/array de downloads | arquivos gravados |

### 5.4 Tabelas de referência

```bash
openmtr ref listar residuo
openmtr ref listar classe
openmtr ref listar unidade
openmtr ref listar acondicionamento
openmtr ref listar estado-fisico
openmtr ref listar tratamento

openmtr ref por-residuo F001
openmtr ref por-estado-fisico 3
```

`ref` exige exatamente uma conta. Os aliases no plural
(`residuos`, `classes`, `estados_fisicos`, ...) também são aceitos.

### 5.5 Health

```bash
openmtr health          # imprime "ok" (não chama a API)
```

---

## 6. `batch` JSONL

`batch` executa uma operação por linha. O arquivo é obrigatório:
`--file operacoes.jsonl` ou `--file -` para stdin.

```bash
openmtr --login '{"system":"sinir","token":"..."}' batch --file ops.jsonl
cat ops.jsonl | openmtr --login '{"system":"sinir","token":"..."}' batch --file -
```

Linha:

```json
{"op":"mtr.consultar","id":"1","account":"sinir-prod","params":{"numeros":["2114014590"]},"out":null,"dry_run":false}
```

Campos:

| Campo | Tipo | Uso |
|---|---|---|
| `op` | string | operação (obrigatória) |
| `id` | string | identificador livre, ecoado na saída |
| `account` | string | conta; omitível se houver uma única conta habilitada |
| `params` | objeto | filtros/corpo da operação |
| `out` | string | diretório de saída para `mtr.download`/`cdf.download` |
| `dry_run` | bool | não chama a API; devolve o plano |

Operações: `health`, `account.test`, `mtr.consultar`, `mtr.criar`,
`mtr.receber`, `mtr.cancelar`, `mtr.modificar`, `mtr.aceitar`, `mtr.download`,
`cdf.consultar`, `cdf.gerar`, `cdf.download`, `ref.listar`,
`ref.por-residuo`, `ref.por-estado-fisico`.

Cada linha produz um NDJSON:

```json
{"index":1,"id":"1","op":"mtr.consultar","account":"sinir-prod","ok":true,"status":200,"data":{...},"warnings":[]}
```

Em erro:

```json
{"index":2,"id":"2","op":"mtr.consultar","account":"sinir-prod","ok":false,"status":7,"error":{"code":"network","message":"...","retryable":true}}
```

Com `--continue-on-error`, todas as linhas são tentadas; sem a flag, o lote
para na primeira falha. O processo termina com código 1 se alguma linha falhou.
`--dry-run` no comando vale para todas as linhas. Linhas vazias e iniciadas por
`#` são ignoradas.

---

## 7. `dump`

O dump consulta um período, normaliza os registros, deduplica por
`system + numero` (prioridade de `access_type`) e grava `mtrs.csv`,
`mtrs.jsonl`, `residuos.csv` e `relatorio.json`.

```bash
openmtr dump --all-accounts \
  --from 2026-09-01 --to 2026-09-30 \
  --window-days 30 --out dump/2026-09 \
  --format both --concurrency 10
```

Flags: `--all-accounts`, `--account`, `--system`, `--tag`, `--from`, `--to`,
`--window-days` (default 30), `--out` (default `dump`), `--format`
(`both|csv|jsonl|none`), `--concurrency` (default 10), `--pdf`, `--no-dedup`,
`--resume`, `--force`.

`--resume` retoma janelas concluídas; `--force` reinicia; `--pdf` baixa os
PDFs; `--no-dedup` mantém duplicatas. O contrato completo de colunas,
determinismo e limitações (incluindo SIGOR) está em
[`13-cli-dump.md`](13-cli-dump.md).

---

## 8. Saída e códigos de erro

- stdout = dados; stderr = logs/avisos;
- sem `--json`, os dados são JSON compacto;
- com `--json`, JSON pretty (e `--json` também afeta `config migrate` e
  `self update`);
- `batch` sempre em NDJSON;
- logs nunca vão para stdout.

| Código | Classe |
|---:|---|
| 0 | sucesso |
| 1 | parcial (lote/dump com erro) ou erro interno |
| 2 | uso inválido, JSON inválido ou conflito de flags |
| 3 | configuração |
| 4 | autenticação |
| 5 | não encontrado |
| 6 | conflito |
| 7 | rede, timeout ou rate limit |

---

## 9. Variáveis de ambiente

| Variável | Escopo | Efeito |
|---|---|---|
| `OPENMTR_VERSION` | `self update`/instalador | Tag forçada |
| `OPENMTR_INSTALL_PATH` | `self update` | Caminho do binário a substituir |
| `OPENMTR_API_URL` | `self update` | Base da API GitHub |
| `OPENMTR_RELEASES_URL` | `self update`/instalador | Base de download |
| `OPENMTR_NO_SESSION_CACHE` | CLI | `1/true` desliga cache em arquivo |
| `OPENMTR_SESSION_DIR` | CLI | Diretório do cache de sessão |
| `RUST_LOG` | CLI | Filtro de log quando `--log-level` não é passado |
| `SINIR_TOKEN`, `MTRPE_SENHA`, ... | config/login | Nomes livres referenciados por `token_env`/`senha_env` |

O instalador também lê `OPENMTR_INSTALL_DIR`, `OPENMTR_TARGET`,
`OPENMTR_REPO`, `OPENMTR_CACHE_DIR` e `OPENMTR_NO_PATH`.

---

## 10. Segurança e limites

- Prefira `token_env`/`senha_env`; segredo inline no TOML gera aviso.
- Mantenha config e cache com `0600`; o diretório de sessão é `0700`.
- Segredos são redigidos em `Debug` e mensagens de erro.
- Downloads usam apenas o basename retornado pelo core; `../` e caminho
  absoluto não escapam de `--out`.
- `--fixed-time` + mesmos dados produz dump byte a byte determinístico.
- SIGOR não participa de `dump` por período: em lote multi-conta ele é
  registrado em `skipped`; sozinho, o comando falha e sugere `mtr consultar`
  por número ou `batch`.
- `self update` no Windows não extrai `.zip` (limitação atual).

---

## 11. Documentação e release

A referência de flags é gerada do binário real:

```bash
make cli-docs       # atualiza docs/wiki/14-cli-reference.md
make check          # valida também se a referência está atualizada
```

O arquivo gerado não deve ser editado à mão. Ao adicionar/remover comando ou
flag, rode `make cli-docs` e versione o resultado.

A release é disparada por tag `vX.Y.Z` (ou manualmente com `workflow_dispatch`
informando a tag). O workflow:

1. exige `tag == [workspace.package].version` e roda `make check` (inclui a
   referência da CLI);
2. compila os seis targets com `--release --locked`;
3. gera `openmtr-cli-vX.Y.Z.md` a partir do binário Linux da release;
4. publica `.tar.gz`/`.zip`, `sha256sums.txt`, a referência da CLI e as notas
   extraídas do `CHANGELOG.md`.

Fluxo local:

```bash
make cli-docs
make check
./scripts/bump-version.sh v0.2.0
git add -A && git commit -S -m "chore(release): 0.2.0"
git tag -a v0.2.0 -m "openmtr v0.2.0"
git push origin main && git push origin v0.2.0
```

O commit da tag precisa já conter a referência atualizada; o portão de release
falha se ela estiver desatualizada.
