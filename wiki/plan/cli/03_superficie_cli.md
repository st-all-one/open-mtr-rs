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
│   ├── consultar [NUMERO]
│   ├── criar
│   ├── receber
│   ├── cancelar
│   ├── modificar
│   ├── aceitar
│   ├── complementar
│   └── download [NUMERO]
├── cdf
│   ├── consultar [CODIGO]
│   ├── gerar
│   └── download
├── ref
│   ├── listar <TABELA>
│   ├── por-residuo <CODIGO>
│   └── por-estado-fisico <CODIGO>
├── health
├── dump
├── batch --file <JSONL|->
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
| `--login <JSON|@ARQUIVO|->` | string | sessão única a partir de um objeto JSON; dispensa `open-mtr-config.toml` |
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
- `--login` × `--account` × `--system` × `--tag` × `--all-accounts` (a sessão já é única).
- `--login` × comandos `config`, `account` e `self` (esses comandos operam sobre a configuração
  local ou o binário, não sobre uma sessão).
- `--login -` × outro consumidor de stdin (`--params -` ou `--file -`): só um leitor de stdin por
  invocação.
- `--params` × `--batch`.
- `--params` × flags de payload planejadas do mesmo comando (`--numero`, `--from`, `--to`, `--dias`,
  etc.) — quando implementadas.
- `--json` × `--format csv|table` é erro (o envelope é JSON).
- `--dry-run` com comando de leitura é permitido (mostra plano, sem rede).

### 3.1 Objeto de login (`--login`)

`--login` dispensa o `open-mtr-config.toml`: cria uma **sessão única** para a invocação, com uma
conta efêmera de id `login`. O mesmo objeto vale para todos os comandos da linha de comando —
inclusive `health`, `dump` e `batch` — e não é persistido.

O valor é um objeto JSON **plano**: o campo `system` escolhe o esquema; os demais campos ficam no
mesmo nível (sem aninhamento por sistema).

| `system` | Campos obrigatórios | Autenticação |
|---|---|---|
| `sinir` | `token` ou `token_env` | refresh token → access token via `POST /token` |
| `mtr_pe` | `cnpj`, `cpf`, `unidade`, `senha` ou `senha_env` | credenciais clássicas |
| `sigor` | `token` ou `token_env`; **ou** `cpf`, `unidade`, `senha`/`senha_env` (`cnpj` opcional) | token ou credenciais clássicas |

Campos opcionais comuns aos três sistemas:

| Campo | Tipo | Efeito |
|---|---|---|
| `base_url` | string | sobrescreve a URL base do sistema |
| `timeout_secs` | u64 (1..=3600) | timeout de rede da sessão |
| `max_rps` | u32 (1..=32) | limite de requisições por segundo |

Formas de passagem:

- `--login '<JSON>'` — objeto inline;
- `--login @caminho/arquivo.json` — lê o objeto de um arquivo;
- `--login -` — lê o objeto inteiro do stdin (heredoc/pipe).

`--login -` não pode coexistir com outro consumidor de stdin (`--params -` ou `--file -`).
Segredos (`token`, `senha`) nunca aparecem em log, erro ou mensagem de validação.

Exemplos:

```bash
openmtr --login '{"system":"sinir","token":"..."}' mtr consultar 2114014590
openmtr --login '{"system":"mtr_pe","cnpj":"...","cpf":"...","unidade":3615,"senha":"..."}' \
  mtr consultar --params '{"periodo":{"dataInicial":"01-05-2026","dataFinal":"31-05-2026"}}'
cat <<'JSON' | openmtr --login '{"system":"sinir","token":"..."}' mtr criar --params -
[{ "gerador": {}, "transportador": {}, "destinador": {}, "residuos": [] }]
JSON
openmtr --login '{"system":"sinir","token":"..."}' ref listar estado-fisico
openmtr --login '{"system":"sinir","token":"..."}' batch --file ops.jsonl
```

### 3.1.1 Sessão e cache de access token

Externamente a autenticação continua explícita: `--login` recebe o token do sistema (SINIR) ou
as credenciais clássicas (MTR-PE/SIGOR). Internamente o CLI troca isso por um **access token
intermediário** e o reutiliza por até 5 minutos:

- SINIR: o `token` informado é um **refresh token**; o CLI faz `POST /token`, guarda o access
  token (`~8 h`) e não repete o refresh a cada operação do batch.
- MTR-PE/SIGOR clássicos: o CLI faz o login (`/gettoken`) uma vez e guarda o JWT.
- MTR-PE/SIGOR com `token`/`token_env` direto: o token já é o access token; não há cache.

O cache é em memória e em arquivo (`0600`, diretório `0700`) sob
`$XDG_RUNTIME_DIR/openmtr/session`, com fallback `$XDG_CACHE_HOME/openmtr/session` ou
`~/.cache/openmtr/session`. O arquivo guarda apenas o access token e `expires_at`; a chave é um
SHA-256 de sistema, conta, URL e credenciais, então trocar a senha/token invalida a entrada.
O TTL efetivo é `min(5 min, exp do JWT - 30 s)`; tokens sem `exp` usam 5 min.
`OPENMTR_NO_SESSION_CACHE=1` desliga a persistência em disco (o cache em memória do processo
continua); `OPENMTR_SESSION_DIR` sobrescreve o diretório.

### 3.2 Posicionais

Cada comando tem no máximo **um** posicional: o **ALVO primário** da operação. Flags continuam
existindo para filtros, paginação e payload; quando o posicional identifica o mesmo alvo de
`--params`, os dois são mutuamente exclusivos.

| Comando | Posicional | Regra |
|---|---|---|
| `mtr consultar [NUMERO]` | número do MTR | exclusivo com `--params` |
| `mtr download [NUMERO]` | número do MTR | exclusivo com `--params`; lote segue em `--params` |
| `cdf consultar [CODIGO]` | código do CDF | exclusivo com `--params` |
| `ref listar <TABELA>` | tabela de referência | conjunto fechado (§6) |
| `ref por-residuo <CODIGO>` | código do resíduo | — |
| `ref por-estado-fisico <CODIGO>` | código do estado físico | — |

Nunca há leitura implícita de stdin: `--params -`, `--file -` e `--login -` exigem o `-`
explícito. Um posicional vazio ou um segundo posicional é erro de entrada (2).

## 4. Comandos MTR

### 4.1 `mtr consultar [NUMERO]`

```bash
openmtr mtr consultar --account sinir-prod --params '{"periodo":{...},"detalhado":true}'
openmtr mtr consultar --account mtrpe-empresa 2114014590
openmtr mtr consultar --account sinir-prod --from 2026-05-01 --to 2026-05-31 --page 1 --size 50
```

O posicional `[NUMERO]` é o alvo primário e é exclusivo com `--params`. Vários números vão em
`--params` (`{"numeros":[...]}`). A tabela abaixo é o **alvo da v1 completa**; a implementação
atual expõe `--params` e o posicional.

Flags de conveniência planejadas (mutuamente exclusivas com `--params`):

| Flag | Mapeia para |
|---|---|
| `--numero <N>` | `FiltroMtr.numeros = [N]`; repetível/vírgula (lote) |
| `--from <YYYY-MM-DD>` | `periodo.dataInicial` |
| `--to <YYYY-MM-DD>` | `periodo.dataFinal` |
| `--days <N>` | período explícito calculado a partir de `--fixed-time` ou hoje |
| `--page <N>` | `FiltroMtr.pagina` |
| `--size <N>` | `FiltroMtr.tamanho` |
| `--detailed` | `FiltroMtr.detalhado = true` |
| `--part <TIPO>` | `FiltroMtr.parte.tipo` |
| `--cnpj <CNPJ>` | `FiltroMtr.parte.documento` |

Sem posicional e sem `--params` ⇒ `InvalidInput` (2). Não existe default de 7/30 dias
no CLI.

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

### 4.3 `mtr download [NUMERO]`

```bash
openmtr mtr download --account sinir-prod 123 --out pdfs/
openmtr mtr download --account sinir-prod --params '{"numeros":["123","456"]}' --out pdfs/
openmtr mtr download --account mtrpe-empresa --params '{"numeros":["barcode..."]}' --out pdfs/
```

- `[NUMERO]` é o alvo primário e é exclusivo com `--params`; lote usa `--params`
  (`{"numeros":[...]}`).
- Saída: arquivos no `--out` com nome determinístico `<account>_<numero>.<ext>` e manifesto
  `manifest.json` (ou JSONL) com hashes.
- MTR-PE usa `barcode` quando disponível; o core já resolve no adapter.

## 5. Comandos CDF

| Comando | Payload | Core |
|---|---|---|
| `cdf consultar [CODIGO]` | `FiltroCdf` via `--params` ou `--from/--to` | `consultar_cdfs` |
| `cdf gerar` | `Vec<GerarCdf>` | `gerar_cdfs` |
| `cdf download` | `Vec<DownloadCdf>` | `download_cdfs` |

O posicional `[CODIGO]` de `cdf consultar` é exclusivo com `--params`; filtros de período usam
`--from/--to`. `cdf gerar` exige `data_emissao` explícita no payload; não há default temporal.

## 6. Comandos de referência

```bash
openmtr ref listar residuo --account sinir-prod
openmtr ref listar classe --account mtrpe-empresa --format jsonl
openmtr ref por-residuo 123456 --account sigor-sp
openmtr ref por-estado-fisico 1 --account sigor-sp
```

Tabelas válidas (conjunto fechado):

`residuo`, `classe`, `unidade`, `acondicionamento`, `estado-fisico`, `tratamento`.

Cada comando aceita exatamente um posicional: a tabela em `ref listar` (obrigatória e restrita ao
conjunto acima) ou o código em `ref por-residuo`/`ref por-estado-fisico`. Posicional ausente ou
fora do conjunto é erro de entrada (2).

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

`batch` exige `--file` (obrigatório); `-` lê o JSONL do stdin. Operações aceitas:

`health`, `account.test`, `mtr.consultar|criar|receber|cancelar|modificar|aceitar|download`,
`cdf.consultar|gerar|download`, `ref.listar|por-residuo|por-estado-fisico`.

Com `--login`, `dump` e `batch` usam a sessão única; no batch, o campo `account` de cada linha pode
ser omitido.

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
3. Nunca há leitura implícita de stdin: comando que exige payload sem `--params` é erro, mesmo com
   stdin não-TTY. `--params -`, `--file -` e `--login -` exigem o `-` explícito.
4. `--file -` (batch) lê o JSONL do stdin.
5. `--login -` lê o objeto de login do stdin. Só um consumidor de stdin por invocação: `--login -`
   é exclusivo com `--params -` e `--file -`.
6. `--params` e `--batch` são mutuamente exclusivos.
7. Exemplos de heredoc:

```bash
openmtr mtr consultar --account sinir-prod --params - <<'JSON'
{"periodo":{"dataInicial":"01-05-2026","dataFinal":"31-05-2026"},"detalhado":true}
JSON

cat <<'JSON' | openmtr --login '{"system":"sinir","token":"..."}' mtr criar --params -
[{ "gerador": {}, "transportador": {}, "destinador": {}, "residuos": [] }]
JSON
```

8. Um JSON inválido é erro de schema (exit 8) com `linha:coluna` na mensagem; nunca ecoar o conteúdo
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
