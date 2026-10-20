# 03 — Superfície da CLI, flags, entrada e saída

> **Status:** etapa 3 (contrato de interface). **Depende de:** `01_contrato_config.md`,
> `02_arquitetura.md`. **Alimenta:** dump/batch, determinismo, testes e distribuição.
>
> Este arquivo congela os **nomes e convenções** da v1. Aliases legados e superfície paralela não
> existem.

## 1. Princípios

1. **Poucos verbos, muitos subcomandos.**
2. **Escopo explícito** para toda operação de rede.
3. **`--params` é universal** para payload JSON; flags de conveniência não competem com ele.
4. **stdout = dados; stderr = logs/progresso.**
5. **`--json` é o contrato de máquina**; `--format` controla dados tabulares/linha.
6. **Sem default temporal oculto:** `consultar` sem período e sem números é erro de entrada.
7. **EPIPE → exit 0**; logs nunca quebram pipe.
8. **Segredos nunca aparecem** em nenhuma saída.

## 2. Árvore de comandos

```text
openmtr
├── config
│   ├── init [--force] [--from-env] [--template basic|multi-account]
│   ├── validate [--offline] [--json]
│   ├── show [--redacted] [--json]
│   ├── accounts [--json]
│   ├── edit [--editor <CMD>]
│   ├── migrate [--dry-run] [--from <PATH>]
│   └── fix-permissions
├── account
│   ├── list [--json]
│   ├── show --id <ID>
│   └── test --id <ID>
├── mtr
│   ├── consultar
│   ├── criar
│   ├── receber
│   ├── cancelar
│   ├── modificar
│   ├── aceitar
│   ├── complementar
│   └── download
├── cdf
│   ├── consultar
│   ├── gerar
│   └── download
├── ref
│   ├── listar <TABELA>
│   ├── por-residuo <CODIGO>
│   └── por-estado-fisico <CODIGO>
├── health
├── dump
├── batch
├── self
│   ├── version
│   ├── completions <SHELL>
│   └── update [--check] [--dry-run] [--version <TAG>]
└── help [COMANDO]
```

`openmtr` sem argumentos = `openmtr help` (exit 0). `openmtr --json` sem verbo = erro de entrada (2).

## 3. Flags globais

| Flag | Tipo | Aplicação |
|---|---|---|
| `--config <PATH>` | path | arquivo TOML; precedência sobre env |
| `--account <ID>` | repetível/vírgula | escopo exato |
| `--system <S>` | `sinir|mtr_pe|sigor` | escopo por sistema |
| `--tag <T>` | repetível/vírgula | filtro de contas por tag |
| `--all-accounts` | bool | todas as contas enabled |
| `--json` | bool | envelope JSON de máquina |
| `--format <F>` | `text|json|jsonl|table|csv` | formato de dados; default `text` |
| `--params <JSON|->` | string | payload/filtro JSON |
| `--batch <PATH|->` | path | lote JSONL |
| `--dry-run` | bool | valida e mostra plano, sem rede/mutação |
| `--out <PATH>` | path | diretório/arquivo de saída materializada |
| `--force` | bool | sobrescreve saída existente (nunca config sem backup) |
| `--continue-on-error` | bool | batch continua após falha de item |
| `--fail-fast` | bool | dump aborta na primeira falha catastrófica (default: continuar) |
| `--concurrency <N>` | u32 | override de `max_concurrency` (1..32) |
| `--timeout <SECS>` | u64 | override de `timeout_secs` |
| `--fixed-time <RFC3339>` | string | relógio determinístico para golden |
| `--quiet` | bool | silencia stderr |
| `--log-level <N>` | `error|warn|info|debug|trace|off` | nível de stderr |
| `--no-color` | bool | sem ANSI (também `NO_COLOR`) |
| `--version` | flag | versão do binário |

Conflitos:

- `--account` × `--system` × `--tag` × `--all-accounts` (ver arquivo 01 §7).
- `--params` × `--batch`.
- `--params` × flags de payload do mesmo comando (`--numero`, `--from`, `--to`, `--dias`, etc.).
- `--json` × `--format csv|table` é erro (o envelope é JSON).
- `--dry-run` com comando de leitura é permitido (mostra plano, sem rede).

## 4. Comandos MTR

### 4.1 `mtr consultar`

```bash
openmtr mtr consultar --account sinir-prod --params '{"periodo":{...},"detalhado":true}'
openmtr mtr consultar --account mtrpe-empresa --numero 2114014590
openmtr mtr consultar --account sinir-prod --from 2026-05-01 --to 2026-05-31 --page 1 --size 50
```

Flags de conveniência (mutuamente exclusivas com `--params`):

| Flag | Mapeia para |
|---|---|
| `--numero <N>` | `FiltroMtr.numeros = [N]`; repetível/vírgula |
| `--from <YYYY-MM-DD>` | `periodo.dataInicial` |
| `--to <YYYY-MM-DD>` | `periodo.dataFinal` |
| `--days <N>` | período explícito calculado a partir de `--fixed-time` ou hoje |
| `--page <N>` | `FiltroMtr.pagina` |
| `--size <N>` | `FiltroMtr.tamanho` |
| `--detailed` | `FiltroMtr.detalhado = true` |
| `--part <TIPO>` | `FiltroMtr.parte.tipo` |
| `--cnpj <CNPJ>` | `FiltroMtr.parte.documento` |

Sem `--numero` e sem período ⇒ `InvalidInput` (2). Não existe default de 7/30 dias no CLI.

Saída padrão (`--format text`): uma linha por MTR (`numero<TAB>status<TAB>data_criacao`). `--json`
devolve o envelope com `dados` completos e `total`.

### 4.2 `mtr criar|receber|cancelar|modificar|aceitar|complementar`

Payload obrigatório por `--params` (objeto/array) ou `--batch`. Exemplo:

```bash
openmtr mtr criar --account mtrpe-empresa --params - <<'JSON'
[{ "gerador": {...}, "transportador": {...}, "destinador": {...}, "residuos": [...] }]
JSON
```

`--dry-run` valida o JSON contra os tipos do core sem chamar a rede.

### 4.3 `mtr download`

```bash
openmtr mtr download --account sinir-prod --numeros 123,456 --out pdfs/
openmtr mtr download --account mtrpe-empresa --params '{"numeros":["barcode..."]}' --out pdfs/
```

- `--numeros` é conveniência; `--params` aceita `{"numeros":[...]}`.
- Saída: arquivos no `--out` com nome determinístico `<account>_<numero>.<ext>` e manifesto
  `manifest.json` (ou JSONL) com hashes.
- MTR-PE usa `barcode` quando disponível; o core já resolve no adapter.

## 5. Comandos CDF

| Comando | Payload | Core |
|---|---|---|
| `cdf consultar` | `FiltroCdf` via `--params` ou `--from/--to` | `consultar_cdfs` |
| `cdf gerar` | `Vec<GerarCdf>` | `gerar_cdfs` |
| `cdf download` | `Vec<DownloadCdf>` | `download_cdfs` |

`cdf gerar` exige `data_emissao` explícita no payload; não há default temporal.

## 6. Comandos de referência

```bash
openmtr ref listar residuo --account sinir-prod
openmtr ref listar classe --account mtrpe-empresa --format jsonl
openmtr ref por-residuo 123456 --account sigor-sp
openmtr ref por-estado-fisico 1 --account sigor-sp
```

Tabelas válidas (conjunto fechado):

`residuo`, `classe`, `unidade`, `acondicionamento`, `estado-fisico`, `tratamento`.

Mapeamento:

| Tabela | Método do core |
|---|---|
| `residuo` | `listar_residuos` |
| `classe` | `listar_classes` |
| `unidade` | `listar_unidades` |
| `acondicionamento` | `listar_acondicionamentos` |
| `estado-fisico` | `listar_estados_fisicos` |
| `tratamento` | `listar_tratamentos` |
| `por-residuo` | `listar_classes_por_residuo` |
| `por-estado-fisico` | `listar_acondicionamentos_por_estado_fisico` |

Saída determinística: ordenar por `codigo` (ou `codigo_ibama`) antes de emitir.

## 7. `health`

O core não tem um endpoint `health` nativo; o WASM simula com `consultar_mtrs` vazio.

Contrato do CLI v1:

1. resolve a conta;
2. autentica/cria o cliente;
3. executa uma leitura barata e universal do sistema (`listar_unidades` por padrão);
4. retorna `{ok, account, system, latency_ms}`.

`--json` inclui `account` e `system`; `--dry-run` só mostra o plano. Se um sistema não suportar
`listar_unidades` no futuro, o contrato permite trocar por outra leitura barata documentada.

## 8. `dump` e `batch`

Contratos detalhados nos arquivos [`04_dump_e_batch.md`](04_dump_e_batch.md). Resumo da superfície:

```bash
openmtr dump [--from <YYYY-MM-DD>] [--to <YYYY-MM-DD>] [--window-days <N>]
             [--out <DIR>] [--format jsonl|csv|both|none] [--pdf]
             [--resume] [--force] [--concurrency <N>]
             [--detail-batch-size <N>] [--stdout-format jsonl] [--fail-fast]
             [--all-accounts|--account ...|--system ...|--tag ...]

openmtr batch --file <JSONL|-> [--dry-run] [--continue-on-error]
              [--out <JSONL>] [--concurrency <N>]
```

`dump` sem seletor de conta é erro (2). `dump --dry-run` mostra contas, janelas e arquivos que
seriam gerados, sem rede.

## 9. `config` e `account`

`config` é puramente local; não exige seletor de conta. `account test` autentica uma conta e faz
uma leitura barata, reportando `ok/latency/erro` sem expor token.

```bash
openmtr account list
openmtr account test --id mtrpe-empresa
```

## 10. `self`

```bash
openmtr self version          # versão, commit opcional, target
openmtr self completions bash
openmtr self update --check   # consulta última versão, sem instalar
openmtr self update           # wrapper para install.sh com checksum
openmtr self update --dry-run # imprime URL/target/ações
```

`self update` não reimplementa download/instalação em Rust; delega ao script versionado
(ver [`08_atualizacao_operacao.md`](08_atualizacao_operacao.md)).

## 11. `--params`, stdin e heredoc

Convenções (idênticas ao `kd`):

1. `--params '<JSON>'` → payload inline.
2. `--params -` → lê **todo** o stdin como JSON.
3. `--params` não definido + comando que exige payload + stdin não-TTY → lê stdin, mas só se
   `--params -` ou `--batch -` não tiver sido usado? **Decisão:** exigir `-` explícito em v1 para
   evitar ambiguidade; comandos de escrita sem `--params` são erro.
4. `--batch -` lê JSONL do stdin.
5. `--params` e `--batch` são mutuamente exclusivos.
6. Heredoc:

```bash
openmtr mtr consultar --account sinir-prod --params - <<'JSON'
{"periodo":{"dataInicial":"01-05-2026","dataFinal":"31-05-2026"},"detalhado":true}
JSON
```

7. Um JSON inválido é erro de schema (exit 8) com `linha:coluna` na mensagem; nunca ecoar o conteúdo
   completo quando contiver segredo.

## 12. Saída

### 12.1 Texto (default)

- `mtr consultar`: linhas `numero\tstatus\tdata_criacao`.
- `ref listar`: linhas `codigo\tdescricao[\tsigla]`.
- `config accounts`: linhas `id\tsystem\tenabled\tlabel`.
- `dump`: progresso em stderr; arquivos em `--out`; stdout fica vazio por default, salvo
  `--stdout-format`.

### 12.2 `--json`

Envelope único:

```json
{
  "success": true,
  "command": "mtr.consultar",
  "data": {
    "account": "sinir-prod",
    "system": "sinir",
    "resposta": { "id": "...", "ok": true, "status": 200, "dados": [] }
  },
  "warnings": []
}
```

Erro:

```json
{
  "success": false,
  "command": "mtr.consultar",
  "error": {
    "code": "auth",
    "message": "credenciais inválidas",
    "retryable": false
  }
}
```

Nunca emitir log em stdout. O envelope é serializado em uma linha (JSON compacto) para pipe.

### 12.3 `--format jsonl`/`csv`

- `mtr consultar`/`ref listar` emitem um objeto/linha por item (sem envelope).
- `dump` usa os formatos de arquivo definidos no arquivo 04.
- `--json` e `--format jsonl/csv` são mutuamente exclusivos.

> `dump` reutiliza `--format`, mas aceita valores adicionais `both|none` (arquivos de saída). Em
> `dump`, `--format` não controla o envelope; o relatório é sempre JSON e o stdout é vazio por
> default.

## 13. Exit codes e erros

Tabela canônica no arquivo 02 §5. Resumo:

| Código | Significado |
|---:|---|
| 0 | sucesso (inclusive dry-run e EPIPE) |
| 1 | sucesso parcial / `Resposta.ok == false` |
| 2 | entrada inválida, escopo ambíguo, flag conflitante |
| 3 | não encontrado |
| 4 | conflito (arquivo existente, conta duplicada) |
| 5 | I/O local |
| 6 | timeout |
| 7 | config/credencial ausente |
| 8 | schema/parse (TOML/JSON/JSONL) |
| 9 | unsafe (permissão de config com segredo em strict) |
| 10 | autenticação/autorização |
| 11 | API/upstream |
| 70 | interno |
| 101 | panic reservado |

`--continue-on-error` só altera o fluxo do batch; o exit final é 1 se houve falha parcial, 0 se
tudo passou.

## 14. Ajuda e descoberta

- `openmtr help` e `openmtr help <verbo>` são embutidos e ensinam escopo obrigatório.
- `--help` de cada subcomando inclui exemplos e conjuntos fechados (tabelas, sistemas, formatos).
- `openmtr --version` e `openmtr self version` são equivalentes para humanos; o envelope
  `--json` traz `version`, `target`, `commit` opcional.
- Erros de valor em conjunto fechado listam as opções e sugerem a mais próxima (Levenshtein simples,
  determinística), como no `kd`.

## 15. Questões em aberto

- `--days` no CLI: usar `--fixed-time` quando fornecido; caso contrário, hoje do sistema. Para
  golden, preferir `--from/--to`. Documentar no help.
- `--format table` na v1: table simples alinhada ou apenas text? **Proposta:** `text` e `table` são
  equivalentes na v1; evita dependência de terminal.
- `dump --stdout-format`: incluir na v1? **Proposta:** sim, `jsonl` apenas, para pipes; CSV
  permanece em arquivo.
- `--out -` para download: binário em stdout? **Proposta:** v1 só diretório; download em stdout
  fica para quando houver necessidade real.
