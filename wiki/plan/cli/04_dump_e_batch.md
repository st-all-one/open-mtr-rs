# 04 — Pipeline de dump e protocolo de batch

> **Status:** etapa 4 (orquestração de volume). **Depende de:** superfície e config.
> **Alimenta:** determinismo, testes, distribuição e operação.
>
> Este documento traduz o `_OPEN_MTR_API/scripts/dump.ts` para um pipeline nativo, multi-conta e
> reproduzível.

## 1. Escopo do `dump` na v1

Sistemas suportados:

| Sistema | Listagem por período | Estratégia |
|---|---|---|
| SINIR | sim (`/listaManifesto` paginado) | listar páginas → coletar números → detalhar em lotes concorrentes |
| MTR-PE | sim (barcodes em janelas de até 30 dias) | core já lista barcodes e detalha com `buffered` |
| SIGOR | **não** (adapter exige números) | `dump` retorna `not_supported` claro; usar `mtr consultar --numero` ou batch |

Contas desabilitadas nunca participam. `dump` sem seletor de conta é erro. Contas do mesmo sistema
são processadas separadamente e identificadas por `account` na saída.

## 2. Pipeline de alto nível

```text
1. carregar config + validar
2. resolver escopo -> Vec<AccountRuntime> ordenado por id
3. validar --from/--to/janelas
4. para cada conta (concorrência limitada):
     a. autenticar/criar cliente
     b. quebrar período em janelas (default 30d)
     c. listar (sem detalhe) e coletar chaves
     d. detalhar em lotes concorrentes quando necessário
     e. normalizar para MtrNormalizada
     f. persistir progresso no manifesto de retomada
5. merge de todas as contas
6. ordenar por (account, system, numero)
7. escrever jsonl/csv/relatorio
8. opcional: baixar PDFs com retomada
9. emitir resumo em stderr; stdout vazio (default)
```

Nada de `join_all` irrestrito: todo fan-out usa `buffer_unordered`/`buffered` com
`max_concurrency` da conta ou `--concurrency`.

## 3. Janelas e período

- `--from` e `--to` em `YYYY-MM-DD` (também aceitar `dd-MM-yyyy` por compatibilidade com a API,
  documentado no help).
- `--to` default: hoje **somente** se `--fixed-time` estiver definido? **Decisão:** sem default
  temporal oculto; exigir `--from` e `--to`. `--days N` é conveniência explícita e usa
  `--fixed-time`/hoje.
- `--window-days` default `[defaults].dump_window_days` (30).
- MTR-PE: janela efetiva é `min(window_days, 30)`; o core já limita, o CLI deve avisar se o usuário
  pedir mais.
- Validar `from <= to`; janela > 3660 dias é erro (proteção contra dump acidental gigante).
- Cada janela tem uma chave estável `YYYY-MM-DD..YYYY-MM-DD` no manifesto de retomada.

## 4. Listagem e detalhe por sistema

### 4.1 SINIR

1. Para cada janela, paginar com `pagina`/`tamanho` (default `50`, máximo `500`).
2. Chamar `consultar_mtrs` sem `detalhado` para obter `numero`, `status`, datas e partes básicas.
3. Parar quando:
   - `total` conhecido for atingido;
   - `dados.len() < tamanho`;
   - página vazia.
4. Coletar números únicos preservando a primeira ocorrência.
5. Detalhar em lotes de `detail_batch_size` (default 25) chamando
   `consultar_mtrs({ numeros: lote, detalhado: true })` em paralelo limitado.
6. Merge dos detalhes: substituir `residuos`, `data_recebimento`, `gerador/transportador/destinador`
   quando o detalhe tiver documento/razão social útil; não apagar dados já preenchidos.
7. Se um lote falhar, registrar erro por lote e continuar; o relatório mostra completude.

> Nota de core: `consultar_mtrs_por_numeros` do SINIR é sequencial internamente. O CLI deve
> paralelizar **lotes**: várias chamadas concorrentes, cada uma com N números. Não chamar um
> número por vez.

### 4.2 MTR-PE

1. Chamar `consultar_mtrs` com `periodo` da janela.
2. O adapter coleta barcodes e detalha cada um com `buffered(max_concurrency)`.
3. Usar `barcode` como chave estável quando `numero` ausente.
4. Respeitar limite de 30 dias por chamada; o CLI pode fatiar antes para maior previsibilidade.
5. Não usar o modo “direto via `fetch`” do dump.ts: o core nativo já implementa o adapter.

## 5. Modelo normalizado de saída

`MtrNormalizada` (campos inspirados no `dump.ts`, com `account`):

```json
{
  "account": "mtrpe-empresa",
  "system": "mtr_pe",
  "numero": "2114014590",
  "barcode": "341234...",
  "status": "Ativo",
  "tipo_manifesto": null,
  "estado_uf": null,
  "data_criacao": "2026-05-01",
  "data_expedicao": "2026-05-01",
  "data_recebimento": null,
  "armazenamento_temporario": null,
  "tem_alteracao_pendente": null,
  "hash": null,
  "observacao": null,
  "motorista": null,
  "placa": null,
  "numero_estadual": null,
  "codigo_estadual": null,
  "gerador":      { "documento": "...", "razao_social": "...", "unidade": 1 },
  "transportador":{ "documento": "...", "razao_social": "...", "unidade": 2 },
  "destinador":   { "documento": "...", "razao_social": "...", "unidade": 3 },
  "residuos": [
    {
      "codigo_ibama": "...", "descricao": null, "quantidade": 1.5,
      "quantidade_recebida": null, "unidade": 1, "estado_fisico": 1,
      "classe": 2, "acondicionamento": 3, "tratamento": 4,
      "densidade": null, "numero_onu": null, "classe_risco": null,
      "nome_embarque": null, "grupo_embalagem": null,
      "codigo_interno": null, "descricao_interna": null
    }
  ],
  "qtd_residuo_esperada": 1.5,
  "qtd_residuo_recebida": null,
  "classe_residuo_nome": "CLASSE II A"
}
```

Regras:

- `account` sempre presente; `system` é o enum estável (`sinir`, `mtr_pe`, `sigor`).
- `numero` é a chave de merge; MTR-PE pode ter `barcode` como fallback.
- Agregados:
  - `qtd_residuo_esperada = soma(quantidade)` arredondada a 4 casas;
  - `qtd_residuo_recebida = soma(quantidade_recebida)` se ao menos um valor existir, senão `null`;
  - `classe_residuo_nome = nomes únicos ordenados e unidos por "; "`.
- `classe_residuo_nome` usa a tabela de classes **da mesma conta/sistema** (não global).
- Nada de UUID/timestamp gerado no item; o envelope de dump não tem `id`.
- Campos ausentes viram `null` (não omitir) para CSV estável.

## 6. Artefatos de saída

Layout:

```text
dump/
├── mtrs.jsonl          # um MtrNormalizada por linha; UTF-8; \n
├── mtrs.csv            # cabeçalho fixo + uma linha por MTR
├── residuos.csv        # cabeçalho fixo + uma linha por resíduo
├── relatorio.json      # contas, janelas, completude, erros, arquivos
├── .openmtr-dump-state.json
└── pdfs/
    ├── sinir_123456.pdf
    └── mtr_pe_2114014590.pdf
```

- `mtrs.jsonl` e CSVs são escritos por streaming; não montar o arquivo inteiro em memória.
- Ordenação global antes de escrever: `(account, system, numero)` com comparação estável de strings.
- CSV:
  - sempre `quoted` quando necessário, `\n` como quebra de linha;
  - datas em ISO `YYYY-MM-DD`;
  - `residuos_json` compacto no CSV de MTRs (compatível com o dump.ts);
  - `residuos.csv` inclui `account`, `system`, `numero`, `idx`.
- `relatorio.json`:
  - `config_version`, `accounts[]`, `periodo`, `files`, `totals`, `completeness`, `errors`;
  - `generated_at` só fora do modo determinístico;
  - erros agrupados por `account`, janela/lote, com código estável e mensagem curta;
  - nenhum segredo.
- `--format both` (default) escreve JSONL+CSV+relatório; `jsonl`/`csv`/`none` reduzem artefatos.
- `--stdout-format jsonl` (opcional) ecoa os itens no stdout **em vez de** gravar `mtrs.jsonl`
  (não ambos), para pipes.

## 7. Retomada (`--resume`)

Estado em `.openmtr-dump-state.json`:

```json
{
  "version": 1,
  "run_id": "dump-20260501-20260531",
  "period": { "from": "2026-05-01", "to": "2026-05-31" },
  "accounts": {
    "sinir-prod": {
      "windows": {
        "2026-05-01..2026-05-30": { "listed": true, "finished": true, "pages": 12 }
      },
      "numbers": ["123", "456"]
    }
  }
}
```

- `--resume` lê o estado existente apenas se `period` e conjunto de contas coincidirem; caso
  contrário, erro de conflito (4) com instrução para `--force`/novo `--out`.
- Antes de cada janela/lote, atualizar estado de forma atômica (tmp + rename).
- `--force` ignora estado e recomeça; arquivos anteriores são sobrescritos.
- PDFs usam retomada por existência + manifesto de hash; arquivo truncado é refeito.
- Um `Ctrl-C` deve deixar o estado coerente (última transação concluída).

## 8. PDFs

- `--pdf` baixa PDFs de MTRs suportados.
- Identificador por sistema:
  - SINIR: `numero`;
  - MTR-PE: `barcode` (34 dígitos), com fallback para `numero` só quando barcode ausente;
  - SIGOR: `not_supported` no dump v1.
- Nome do arquivo: `<account>_<numero>.pdf`; sanitizar caracteres inválidos.
- Concorrência `min(max_concurrency, 6)` por conta.
- Verificar magic bytes `%PDF` e tamanho mínimo; se inválido, registrar erro e não criar arquivo.
- Manifesto `pdfs/manifest.jsonl`: `account, system, numero, file, sha256, size, status`.
- Hash SHA-256 é do arquivo; usar `sha2`? **Proposta:** o CLI já terá `sha2` apenas se necessário;
  caso contrário, implementar hash no script de release? Para PDFs, `sha2` é dependência pequena e
  útil; se evitar, manifesto registra só tamanho. **Decisão:** v1 registra `size` e `sha256` com a
  crate `sha2` (MIT/Apache), justificada no `cargo deny`.

## 9. Concorrência, timeouts e falhas

- Timeout por requisição vem do core (`Config::timeout`); CLI override `--timeout`.
- Concorrência:
  - global: `--concurrency`;
  - por conta: `min(global, account.max_concurrency, defaults.max_concurrency)`;
  - MTR-PE: core já aplica `max_concurrency`; o CLI não abre outro fan-out por cima.
- Falha de uma janela/lote:
  - registrar erro no relatório;
  - continuar com outras janelas, salvo `--fail-fast`? **Decisão:** default continua e retorna
    exit 1 se houver qualquer erro; `--fail-fast` aborta na primeira falha catastrófica.
- 429/5xx já têm retry no core; o CLI registra `warnings`/`errors` de lote e não reimplementa
  backoff.
- Limite de memória: nunca manter todos os MTRs de todas as contas em memória se o volume for
  grande? Para merges determinísticos é mais simples manter por conta; o plano aceita memória
  proporcional ao número de MTRs do período (como o dump.ts), mas exige streaming na escrita e
  limite de contas simultâneas.

## 10. Protocolo de batch JSONL

### 10.1 Entrada

Uma linha = uma operação JSON:

```json
{"op":"mtr.consultar","account":"sinir-prod","params":{"periodo":{"dataInicial":"01-05-2026","dataFinal":"31-05-2026"}}}
{"op":"ref.listar","account":"mtrpe-empresa","params":{"table":"classe"}}
{"op":"mtr.download","account":"sinir-prod","params":{"numeros":["123"]},"out":"pdfs"}
```

Campos:

| Campo | Obrigatório | Regra |
|---|---|---|
| `op` | sim | conjunto fechado (`mtr.*`, `cdf.*`, `ref.*`, `health`, `account.test`) |
| `account` | condicional | id; se ausente, usa escopo global da CLI |
| `params` | condicional | objeto JSON; mesmo schema de `--params` |
| `out` | não | diretório de arquivos materializados (download/dump? dump não no batch v1) |
| `dry_run` | não | bool por linha; combinado com `--dry-run` |
| `id` | não | id lógico do usuário; ecoado na saída para correlação |

Linhas vazias e comentários `#` são ignorados. Linha inválida ⇒ erro de schema (8), com número da
linha; `--continue-on-error` não transforma schema inválido em sucesso.

### 10.2 Saída

Uma linha JSON por operação executada:

```json
{"index":1,"id":null,"op":"mtr.consultar","account":"sinir-prod","ok":true,"status":200,"data":{...},"warnings":[]}
```

Erro:

```json
{"index":2,"id":"x","op":"mtr.consultar","account":"mtrpe-empresa","ok":false,"status":401,"error":{"code":"auth","message":"...","retryable":false}}
```

- `index` é 1-based e estável.
- `data` é o conteúdo do envelope `Resposta`/domínio, sem `id` UUID quando determinístico.
- `--json` no batch não é necessário: o batch **sempre** emite JSONL; `--json` é inválido.
- `--dry-run` emite linhas `{index, op, account, ok:true, dry_run:true, plan:{...}}` sem rede.
- `--continue-on-error`: default `false`; com `true`, continua e exit final 1 se houve falha.
- `--out` é obrigatório para operações que materializam arquivos; caminhos relativos ao diretório
  atual.

### 10.3 Execução

1. Ler JSONL linha a linha (streaming).
2. Validar `op` e schema antes de qualquer rede.
3. Resolver conta por linha; manter cache de `AccountRuntime` por id.
4. Executar com `buffer_unordered` limitado? **Decisão:** batch é **sequencial por padrão** para
   previsibilidade de escrita/ordem; `--concurrency` opcional para operações de leitura. Operações
   de escrita permanecem sequenciais mesmo com `--concurrency`.
5. Emitir saída imediatamente (flush por linha) para permitir pipe/LLM.
6. Em erro catastrófico com `--continue-on-error=false`, emitir o erro da linha e parar.

## 11. Exemplos

```bash
# dump global explícito (SINIR + duas contas MTR-PE)
openmtr dump --all-accounts --from 2026-05-01 --to 2026-05-31 --out dump/

# dump só de uma conta
openmtr dump --account mtrpe-empresa --from 2026-05-01 --to 2026-05-31 --out dump-empresa/

# retomar
openmtr dump --all-accounts --from 2026-05-01 --to 2026-05-31 --out dump/ --resume

# batch de consultas
cat <<'JSONL' | openmtr batch --file - --continue-on-error > resultados.jsonl
{"op":"mtr.consultar","account":"sinir-prod","params":{"periodo":{"dataInicial":"01-05-2026","dataFinal":"31-05-2026"}}}
{"op":"ref.listar","account":"mtrpe-empresa","params":{"table":"classe"}}
JSONL
```

## 12. Critérios de aceite do dump

- Duas contas do mesmo sistema aparecem com `account` distintos e não se sobrescrevem.
- Mesma entrada + `--fixed-time` ⇒ mesmos bytes em `mtrs.jsonl`/CSV (exceto nome do diretório/estado).
- `--resume` não repete janelas concluídas; estado inconsistente é erro claro.
- Nenhum PDF inválido é gravado.
- Relatório contém completude por conta e erros por janela/lote.
- Nenhum segredo em nenhum artefato (teste de redaction).
