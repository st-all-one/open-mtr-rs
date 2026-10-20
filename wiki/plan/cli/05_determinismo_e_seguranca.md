# 05 — Determinismo, saída de máquina e segurança

> **Status:** etapa 5 (contrato transversal). **Depende de:** superfície e dump/batch.
> **Alimenta:** testes, distribuição e operação.
>
> Determinismo aqui significa: **mesma config + mesmos parâmetros + mesmo clock fixo ⇒ mesmos
> bytes de dados**. Segurança significa: **nenhum segredo em stdout/stderr/artefato**.

## 1. Dados vs logs

| Canal | Conteúdo | Regra |
|---|---|---|
| stdout | dados | JSON/JSONL/CSV/tabela/relatório; nunca log |
| stderr | logs/progresso/erros humanos | pode variar, mas nunca contém segredo |
| arquivos | dump/PDF/relatório | determinísticos quando aplicável |

- Nunca `println!`/`eprintln!` no código de produção; usar `output::emit_stdout`/`emit_stderr`.
- EPIPE em stdout → exit 0; outras falhas de I/O → exit 5.
- `--quiet` desliga logs em stderr; **não** silencia erros fatais (mensagem mínima em stderr).
- `--json` não pode ser misturado com logs no stdout; `tracing` sempre em stderr.

## 2. Fontes de variabilidade e tratamento

| Fonte | Hoje no core | Política CLI |
|---|---|---|
| `Resposta.id` UUID v7 | `MtrCtx::new_id` | injetar `SequentialIdGenerator` quando `--fixed-time` ou `--deterministic` |
| `OpenMTRError.instance` UUID v7 | `with_kind` | `OpenMtrClient` já reatribui com `ctx.new_id`; CLI injeta gerador determinístico |
| `Periodo::ultimos_dias` / `{"dias":N}` | relógio global | CLI sempre envia datas explícitas; `--days` usa `--fixed-time` quando definido |
| `gerar_cdfs`/data de emissão | default do adapter se ausente | CLI exige `data_emissao` explícita |
| `generated_at` de relatório | `Utc::now` | omitir ou fixar em modo determinístico |
| ordem de listas da API | ordem do JSON | ordenar por `codigo`/`codigo_ibama` antes de emitir |
| ordem de contas | ordem do TOML | ordenar por `id` na resolução de escopo |
| ordem de dump | conclusão concorrente | merge final ordenado por `(account, system, numero)` |
| `HashMap` | onde usado | não usar para ordem de saída; `BTreeMap`/`Vec` ordenado |
| duração/latência | medida real | só em stderr/relatório fora do modo determinístico |

## 3. Modo determinístico

Flag global:

```bash
openmtr --fixed-time 2026-06-01T00:00:00Z ... 
```

Comportamento:

1. Cria `FixedClock` e `SequentialIdGenerator` (IDs 1,2,3...).
2. Injeta em todos os `OpenMtrClientBuilder` de todas as contas.
3. `--days`/`--to` default (se existirem) usam a data fixa.
4. `relatorio.json`:
   - `generated_at` = valor fixo ou campo omitido;
   - `duration_ms` omitido;
   - `run_id` derivado de `period + accounts` (hash estável), não de timestamp.
5. Nomes de arquivos de backup/estado não usam timestamp no modo determinístico.
6. Erros de negócio mantêm `instance` sequencial.

Sem `--fixed-time`, a saída ainda deve ser estável **exceto** campos explicitamente voláteis
(`generated_at`, `duration_ms`, UUIDs de envelope). Documentar no help.

## 4. Contrato de envelope

`--json` (comandos não-dump):

```json
{
  "success": true,
  "command": "mtr.consultar",
  "data": {
    "account": "sinir-prod",
    "system": "sinir",
    "resposta": { "id": "...", "ok": true, "status": 200, "dados": [], "erros": [], "total": 0 }
  },
  "warnings": []
}
```

- Chaves em `snake_case`; ordem de serialização estável (serde struct, não `HashMap`).
- `data` pode ser `null` em comandos sem retorno (ex.: `config init` retorna caminho e contas).
- `error.code` é um conjunto fechado (`invalid_input`, `not_found`, `conflict`, `io`, `timeout`,
  `config`, `schema`, `unsafe`, `auth`, `api`, `internal`).
- `retryable` é derivado do `ErrorKind` (timeout/network/5xx ⇒ `true`; validação/auth ⇒ `false`).
- `warnings` só aparece se não vazio; `strict` promove warning de config/segurança a erro.
- O envelope nunca contém token, senha, CPF/CNPJ completo? **Decisão:** CPF/CNPJ podem aparecer em
  dados de domínio (MTR), mas **nunca** credenciais. Logs de erro devem redigir CPF/senha quando
  vierem de `--params`/config.

## 5. JSONL de dump/batch

- Uma linha = um objeto compacto, `\n` no final.
- Sem espaços/pretty no modo streaming.
- Campos ausentes em `MtrNormalizada` viram `null`; não omitir.
- `batch` emite flush por linha; o consumidor pode processar streaming.
- Ordenação de batch = ordem de entrada (`index`); dump = ordenação global explícita.

## 6. CSV

- Cabeçalho fixo e versionado; mudanças exigem `schema_version` no relatório.
- Sempre aspas duplas; aspas internas duplicadas; `\n` interno substituído por espaço (como dump.ts).
- Datas `YYYY-MM-DD`; null = campo vazio.
- Números com ponto decimal; arredondamento a 4 casas para agregados.
- Codificação UTF-8 sem BOM.

## 7. Redaction e segredos

### 7.1 O que nunca pode sair

- Token de integração, JWT, senha, `token_env`/`senha_env` valores.
- Conteúdo bruto de `Authorization`.
- CPF/senha de `AuthParams` em log de erro.
- Caminho de config com segredo em mensagem de erro? O caminho pode sair; o conteúdo não.

### 7.2 Implementação

- `Redactor` no CLI: lista de padrões/segredos conhecidos + regex para `Bearer ...`, `token=...`,
  `senha=...`, `password=...`.
- `tracing` formatter customizado/`RedactingWriter` em stderr; toda mensagem passa por redaction.
- `CliError` guarda apenas mensagem já redigida; nunca carrega payload bruto.
- `config show` sempre redige `token`/`senha`; não existe flag `--show-secrets`.
- `--params` inválido: mensagem cita caminho do campo e linha/coluna, não o valor.

### 7.3 Permissões

- `config init` cria `0600` no Unix.
- `config validate` avisa se `mode & 0o077 != 0` com segredo inline.
- `config fix-permissions` aplica `0600` explicitamente.
- Windows: sem checagem de ACL na v1; documentar.

## 8. Segurança de arquivos e caminhos

- `--out` é criado com `create_dir_all`; nunca seguir symlink para sobrescrever fora do diretório?
  **Regra:** abrir arquivos com `OpenOptions` sem `append`; se existir e `--force` ausente,
  `Conflict` (4).
- Nome de arquivo de PDF/dump: sanitizar `[A-Za-z0-9._-]`, remover `/`, `\`, `..`, NUL.
- Backups de config: `open-mtr-config.toml.bak.<timestamp>`; no modo determinístico, sufixo estável
  `.bak`.
- Nunca escrever segredo em `relatorio.json`, `manifest.json`, logs de retomada.
- `--params` com segredo? Não deveria haver segredo em payload MTR; se houver, redigir em erro/log.

## 9. Concorrência determinística

- `buffer_unordered` pode concluir fora de ordem; a saída final é sempre reordenada por chave
  estável antes de escrever.
- Para batch de leitura concorrente, a saída deve ser reordenada por `index` (ou emitida em ordem
  de entrada com buffer de conclusão). **Decisão:** batch concurrent mantém ordem de entrada.
- Para dump, cada conta/janela grava em estrutura própria; merge final ordena.
- Nunca usar `HashMap` para agregar saída final.

## 10. Testes de determinismo e segurança

| Teste | Critério |
|---|---|
| `deterministic_envelope` | duas execuções com `--fixed-time` geram bytes idênticos |
| `deterministic_dump` | fixture SINIR + 2 MTR-PE gera JSONL/CSV idênticos (exceto paths) |
| `stable_ref_order` | resposta mockada fora de ordem gera saída ordenada por código |
| `account_order` | contas no TOML fora de ordem são resolvidas/emitidas por `id` |
| `no_secret_in_stdout_stderr` | token/senha fake nunca aparecem em capturas |
| `redact_error_payload` | erro de JSON/HTTP não ecoa Authorization nem senha |
| `epipe_exit_zero` | `openmtr ... | head -1` → exit 0 |
| `file_permissions` | `config init` cria 0600 no Unix |
| `resume_idempotent` | retomar dump não duplica linhas |

## 11. Riscos e mitigação

| Risco | Mitigação |
|---|---|
| `f64` de quantidade gerar diferença de serialização | arredondar agregados a 4 casas; manter valores originais nos resíduos; documentar |
| `generated_at` quebrar golden | omitir/fixar em modo determinístico |
| `HashMap` reintroduzir ordem instável | lint/revisão + testes de ordenação |
| segredo vazar em URL de erro | redactor + teste com token fake |
| symlink em `--out` | `OpenOptions` sem seguir? validar `canonicalize` do diretório; nunca sobrescrever fora |
| logs de auth incluírem payload | `#[instrument(skip_all)]` e redaction no writer |
| mode determinístico não cobrir `dias` | help/validador: `--days` exige `--fixed-time` para golden; preferir `--from/--to` |
