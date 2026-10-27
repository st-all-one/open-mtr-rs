# 13 — CLI `openmtr` e pipeline de dump

> Contrato operacional do binário `openmtr`: configuração multi-conta, dump
> multi-sistema, arquivos gerados e release via pipeline. O dump é a parte do
> CLI com maior superfície de dados; o restante da superfície está descrito em
> [`../plan/cli/03_superficie_cli.md`](../../wiki/plan/cli/03_superficie_cli.md).

## 1. Instalação e atualização

A pipeline de release publica binários por plataforma em
`github.com/st-all-one/open-mtr-rs/releases`:

| Plataforma | Asset |
|---|---|
| Linux x86_64 | `openmtr-vX.Y.Z-x86_64-unknown-linux-musl.tar.gz` |
| Linux ARM64 | `openmtr-vX.Y.Z-aarch64-unknown-linux-musl.tar.gz` |
| macOS Apple Silicon | `openmtr-vX.Y.Z-aarch64-apple-darwin.tar.gz` |
| macOS Intel | `openmtr-vX.Y.Z-x86_64-apple-darwin.tar.gz` |
| Windows x86_64 | `openmtr-vX.Y.Z-x86_64-pc-windows-msvc.zip` |
| Windows ARM64 | `openmtr-vX.Y.Z-aarch64-pc-windows-msvc.zip` |

Instalação via script (verifica SHA-256 e instala atomicamente):

```sh
curl -fsSL https://raw.githubusercontent.com/st-all-one/open-mtr-rs/main/install.sh | sh
openmtr --version
```

Atualização pelo próprio binário:

```sh
openmtr self update --check
openmtr self update
```

O `self update` consulta `/releases/latest`, baixa `sha256sums.txt`, valida o
hash do asset e substitui o executável. `OPENMTR_VERSION` força uma tag.

## 2. Configuração multi-conta

O CLI lê `open-mtr-config.toml` por padrão (`--config` troca o caminho). Comece
pelo exemplo:

```sh
cp open-mtr-config.example.toml open-mtr-config.toml
chmod 600 open-mtr-config.toml
openmtr --config open-mtr-config.toml config validate
openmtr --config open-mtr-config.toml account list
```

Cada `[[accounts]]` tem `id`, `system`, `access_type`, `tags`, credenciais
(`token_env`/`senha_env` são preferíveis a valores inline) e
`max_concurrency`. `access_priority` define quem vence a deduplicação do dump:
menor valor vence; a transportadora é a fonte preferida.

## 3. Superfície do CLI

```text
openmtr [--config PATH] [--json] [--fixed-time DATA] [--quiet] [--log-level NÍVEL] <comando>

account list | account test <id>
config validate | config migrate [--dry-run]
mtr consultar|criar|receber|cancelar|modificar|aceitar|download ...
cdf consultar|gerar|download ...
ref listar --table <tabela> [--account <id>]
health
dump [--all-accounts|--account ID|--system SISTEMA|--tag TAG]...
     --from YYYY-MM-DD --to YYYY-MM-DD [--window-days N] [--out DIR]
     [--format both|csv|jsonl|none] [--concurrency N]
     [--pdf] [--no-dedup] [--resume] [--force]
batch [--file ARQUIVO] [--continue-on-error] [--dry-run]
self version | self update [--check|--dry-run|--version TAG|--force]
```

O `dump` exige um escopo de conta: `--all-accounts`, um ou mais `--account`,
`--system` ou `--tag`. Sem escopo o comando falha com código 2.

### 3.1 Exemplo de dump

```sh
openmtr --config open-mtr-config.toml dump \
  --all-accounts \
  --from 2026-09-01 \
  --to 2026-09-30 \
  --window-days 30 \
  --out dump/2026-09 \
  --format both \
  --concurrency 10
```

O período é dividido em janelas de `--window-days` (máximo prático: 30 dias no
MTR-PE). Para retomar após falha, repita com `--resume`; para descartar o
estado anterior, use `--force`. `--pdf` baixa os PDFs dos MTRs; `--no-dedup`
mantém duplicatas entre contas.

## 4. Fluxo do dump

```text
contas resolvidas
  └─ por conta
      └─ por janela
          └─ SINIR: 4 papéis (gerador, transportador, destinador, armazenador)
              └─ páginas de 500 manifestos, detalhado = true
                  └─ DumpRow
  └─ deduplicação (system + numero/barcode; access_priority)
  └─ ordenação estável (account, system, numero, barcode)
  └─ mtrs.jsonl / mtrs.csv / residuos.csv / relatorio.json
```

- **SINIR:** consulta os quatro `tipoParceiro`; sem isso o dump só enxerga os
  manifestos em que a conta é geradora.
- **MTR-PE / SIGOR:** sem filtro de papel; a listagem já é limitada pela conta.
- **SIGOR:** não suporta listagem por período. Em dump multi-conta a conta é
  ignorada e registrada em `skipped`; com uma única conta SIGOR o comando falha
  e sugere `mtr consultar --numero` ou `batch`.
- **Classes de resíduo:** a tabela `listar_classes` da conta é consultada uma
  vez por conta e usada para traduzir `codigo → descrição` em maiúsculas; se a
  tabela falhar, o CSV mantém `CLASSE <código>`.
- **Observação do resíduo:** o campo `marObservacao` (SINIR/SIGOR) e
  `manifestoItemObservacao`/`observacao` (MTR-PE) é preservado em
  `OBSERVACOES_RESIDUOS`, no JSON detalhado e no `residuos.csv`.
- **Paginação:** 500 por página, até 1000 páginas por papel.
- **Deduplicação:** a chave é `system|numero` (ou `barcode` quando o número é
  vazio). Menor `access_priority` vence; empate resolve por `account`.
- **Determinismo:** com `--fixed-time` e a mesma entrada, os arquivos são byte a
  byte iguais; a suíte `dump_is_byte_deterministic_with_fixed_time` cobre isso.

## 5. Arquivos gerados

### 5.1 `mtrs.csv`

CSV com quebra de linha LF e cabeçalho fixo:

```text
OTR,MTR_NUMERO,DATA_EMISSAO,DATA_RECEBIMENTO,QNT_RECEBIDA,QNT_ESPERADA,STATUS,
OBSERVACOES,OBSERVACOES_RESIDUOS,GERADOR_CNPJ,GERADOR_NOME,DESTINADOR_CNPJ,
DESTINADOR_NOME,MOTORISTA,PLACA,CLASSE_RESIDUO,RESIDUO_JSON_DETALHADO,SISTEMA
```

| Coluna | Semântica |
|---|---|
| `OTR` | `-` no dump bruto; o vínculo MTR↔OTR é preenchido a jusante |
| `MTR_NUMERO` | número do manifesto |
| `DATA_EMISSAO` | `data_criacao`, com fallback para `data_expedicao` |
| `DATA_RECEBIMENTO` | data de recebimento, quando houver |
| `QNT_RECEBIDA` | soma de `quantidade_recebida`; vazio se nenhum recebimento |
| `QNT_ESPERADA` | soma de `quantidade`; `0` quando não há resíduos |
| `STATUS` | status normalizado do domínio |
| `OBSERVACOES` | observação do MTR |
| `OBSERVACOES_RESIDUOS` | descrições e observações únicas dos resíduos, separadas por barra vertical |
| `GERADOR_CNPJ` / `GERADOR_NOME` | documento e razão social do gerador |
| `DESTINADOR_CNPJ` / `DESTINADOR_NOME` | documento e razão social do destinador |
| `MOTORISTA` / `PLACA` | dados de transporte, quando expostos pela API |
| `CLASSE_RESIDUO` | descrição da classe na tabela da conta (maiúsculas), ex.: `CLASSE I`, `CLASSE II A`, `OUTROS`; fallback `CLASSE <código>`; classes únicas separadas por barra vertical |
| `RESIDUO_JSON_DETALHADO` | array JSON com todos os campos do resíduo, incluindo `null` |
| `SISTEMA` | slug do sistema (`sinir`, `mtr_pe`, `sigor`) |

O JSON detalhado contém: `codigo_ibama`, `descricao`, `quantidade`,
`quantidade_recebida`, `unidade`, `estado_fisico`, `classe`, `acondicionamento`,
`tratamento`, `densidade`, `numero_onu`, `classe_risco`, `nome_embarque`,
`grupo_embalagem`, `codigo_interno`, `descricao_interna` e `observacao`.

### 5.2 `mtrs.jsonl`

Uma linha por MTR, com o registro completo de `DumpRow`: `account`, `system`,
`numero`, `barcode`, status, datas, partes (documento/razão social/unidade),
`residuos_json`, `classe_residuo`, `observacoes_residuos`, quantidades e
`access_priority`. É a fonte para retomada (`--resume`) e para integrações.

### 5.3 `residuos.csv`

Uma linha por resíduo (não por MTR):

```text
account,system,numero,idx,codigo_ibama,descricao,quantidade,quantidade_recebida,
unidade,estado_fisico,classe,acondicionamento,tratamento,observacao
```

### 5.4 `relatorio.json`

Resumo da execução: `config_version`, período, contas, arquivos, `totals`
(`fetched`, `emitted`, `deduplicated`, `errors`), `completeness`, `deduplicated`
detalhado, `skipped` e `errors` por conta/janela. Erros parciais geram
`CliError::Partial` (código 1), mas os arquivos já produzidos são preservados.

## 6. Códigos de saída

| Código | Classe |
|---|---|
| 0 | sucesso |
| 1 | parcial ou erro interno |
| 2 | uso inválido |
| 3 | configuração |
| 4 | autenticação |
| 5 | não encontrado |
| 6 | conflito |
| 7 | rede |

## 7. Release 0.1.3 pela pipeline

A release é disparada por tag `vX.Y.Z` (`.github/workflows/release.yml`):

1. `verify` roda `make check` e exige `tag == [workspace.package].version`.
2. `build` compila a matriz otimizada (`--release --locked`) por runner nativo.
3. `release` reúne os artefatos, gera `sha256sums.txt`, extrai a seção da versão
   do `CHANGELOG.md` com `scripts/release-notes.sh` e publica o GitHub Release.

Fluxo local:

```sh
make check
./scripts/bump-version.sh v0.1.3
git add -A && git commit -S -m "chore(release): 0.1.3"
git tag -a v0.1.3 -m "openmtr v0.1.3"
git push origin main && git push origin v0.1.3
```

O binário instalado reporta a versão do workspace:

```sh
openmtr --version        # openmtr 0.1.3
openmtr self version     # openmtr 0.1.3
```
