# 01 — Contrato do `open-mtr-config.toml`

> **Status:** etapa 1 (contrato de configuração). **Depende de:** `00_diagnostico.md`.
> **Alimenta:** arquitetura, superfície, dump, testes e distribuição.
>
> Este arquivo é o **contrato congelado** do arquivo de configuração. Mudanças incompatíveis exigem
> incremento de `config_version` e migração.

## 1. Objetivos do arquivo

1. Representar **N credenciais** de forma cumulativa.
2. Permitir **mais de uma conta do mesmo sistema** sem variáveis de ambiente inventadas.
3. Declarar escopo de operação por conta (`id`, `system`, `tags`), com seleção explícita no CLI.
4. Separar defaults globais de overrides por conta.
5. Permitir segredos inline **ou** por variável de ambiente, com preferência e aviso claros.
6. Ser legível e editável à mão, com validação/migração simples.

## 2. Localização e precedência

Ordem de resolução do arquivo:

1. `--config <PATH>`;
2. `OPENMTR_CONFIG` (variável de ambiente);
3. `./open-mtr-config.toml` (diretório atual);
4. `$XDG_CONFIG_HOME/open-mtr/open-mtr-config.toml` (Linux/macOS);
5. `%APPDATA%\open-mtr\open-mtr-config.toml` (Windows);
6. `~/.config/open-mtr/open-mtr-config.toml` (fallback Unix).

O caminho efetivamente usado deve aparecer em:
- `openmtr config show --json` no campo `config_path`;
- logs de debug em stderr (nunca no stdout de dados).

**Nunca** procurar em `.env` automaticamente em runtime. `config init --from-env` é uma migração
explícita; depois disso o TOML é a única fonte da verdade.

## 3. Esqueleto

```toml
config_version = 1
default_account = "sinir-prod"

[defaults]
timeout_secs = 60
max_concurrency = 8
dump_window_days = 30
output_dir = "dump"
log_level = "warn"
strict = false

[[accounts]]
id = "sinir-prod"
system = "sinir"
label = "SINIR produção"
tags = ["prod", "nacional"]
enabled = true

[accounts.auth]
type = "token"
token_env = "OPENMTR_SINIR_TOKEN"

[[accounts]]
id = "mtrpe-empresa"
system = "mtr_pe"
label = "MTR-PE empresa"
tags = ["prod", "pe", "gerador", "transportador"]
base_url = "https://mtr.cprh.pe.gov.br/api"

[accounts.auth]
type = "cpf_senha_unidade"
cpf = "00000000000"
cnpj = "00000000000000"
unidade = 3623
password_env = "MTR_PE_PASSWORD"

[[accounts]]
id = "mtrpe-transportadora"
system = "mtr_pe"
label = "MTR-PE transportadora"
tags = ["prod", "pe", "transportadora"]
base_url = "https://mtr.cprh.pe.gov.br/api"

[accounts.auth]
type = "cpf_senha_unidade"
cpf = "00000000000"
cnpj = "00000000000000"
unidade = 9999
password_env = "MTR_PE_TRANSPORTADORA_PASSWORD"

[[accounts]]
id = "sigor-sp"
system = "sigor"
label = "SIGOR CETESB-SP"
tags = ["prod", "sp"]
enabled = false

[accounts.auth]
type = "token"
token_env = "OPENMTR_SIGOR_TOKEN"
```

## 4. Campos de topo

| Campo | Tipo | Obrigatório | Regra |
|---|---|---:|---|
| `config_version` | inteiro | sim | deve ser `1`; ausência ⇒ erro de migração |
| `default_account` | string | não | id de conta habilitada; usado apenas como sugestão/erro, **nunca** como fallback silencioso de escrita |
| `defaults` | tabela | não | defaults do CLI; overrides por conta vencem |
| `accounts` | array de tabelas | sim | pelo menos uma conta habilitada |

### 4.1 `[defaults]`

| Campo | Tipo | Default | Aplicação |
|---|---|---:|---|
| `timeout_secs` | inteiro | `60` | timeout HTTP por requisição (1..3600) |
| `max_concurrency` | inteiro | `8` | concorrência máxima de operações/dump (1..32) |
| `dump_window_days` | inteiro | `30` | janela padrão do dump; MTR-PE nunca passa de 30 |
| `output_dir` | string | `dump` | diretório padrão de saída de dump/PDF |
| `log_level` | string | `warn` | `error|warn|info|debug|trace|off` |
| `strict` | bool | `false` | promove `warnings[]` a erro de saída (exit 2) |

## 5. Conta (`[[accounts]]`)

| Campo | Tipo | Obrigatório | Regra |
|---|---|---:|---|
| `id` | string | sim | único, `^[a-z0-9][a-z0-9._-]{0,63}$`; não pode repetir |
| `system` | string | sim | `sinir`, `mtr_pe`, `sigor`; `config validate` rejeita sistemas sem adapter |
| `label` | string | não | nome humano; default = `id` |
| `tags` | lista de strings | não | minúsculas, sem espaços; usadas para seleção por `--tag` |
| `enabled` | bool | não | default `true`; contas desabilitadas não entram em `--all-accounts` |
| `base_url` | string | não | override do `default_base_url`; validado com `Url::parse` (`http`/`https`) |
| `timeout_secs` | inteiro | não | override de `[defaults]` |
| `max_concurrency` | inteiro | não | override de `[defaults]` |
| `metadata` | tabela string→string | não | livre para operação (ex.: grupo, contrato); nunca segredo |

### 5.1 Regras de unicidade

- `id` é a chave estável; **nunca** usar `system` como identidade.
- `id` duplicado ⇒ erro `config`.
- `(system, base_url, auth)` não precisa ser único: duas contas MTR-PE com CNPJs diferentes são
  válidas e esperadas.
- `label` duplicado é permitido, mas gera warning.

## 6. Autenticação

O bloco `[accounts.auth]` é uma **tagged union** discriminada por `type`.

### 6.1 `type = "token"`

Para SINIR e SIGOR (e qualquer sistema que aceite token de integração/JWT direto).

| Campo | Tipo | Obrigatório | Regra |
|---|---|---:|---|
| `token` | string | um dos dois | segredo inline; gera warning e checagem de permissão |
| `token_env` | string | um dos dois | nome de variável de ambiente; tem precedência se ambos ausentes? ver §6.4 |
| `refresh_url` | string | não | override opcional; o core normalmente deriva de `base_url` |

### 6.2 `type = "cpf_senha_unidade"`

Para MTR-PE e SIGOR com autenticação clássica.

| Campo | Tipo | Obrigatório | Regra |
|---|---|---:|---|
| `cpf` | string | sim | 11 dígitos; validado pelo newtype do core |
| `cnpj` | string | MTR-PE sim; SIGOR não | 14 dígitos; MTR-PE exige |
| `unidade` | inteiro | sim | código da unidade/pessoa |
| `senha` | string | um dos dois | segredo inline; warning + permissão |
| `senha_env` | string | um dos dois | variável de ambiente preferencial |
| `token_env` | string | não | se o usuário já tem JWT pronto, permite pular auth clássica |

### 6.3 `type = "none"`

Reservado para comandos que não autenticam (ex.: `config`, `health --offline`). Contas de operação
não podem usar `none`.

### 6.4 Resolução de segredos

Para cada segredo (`token`, `senha`):

1. Se `*_env` estiver definido, ler a variável de ambiente.
2. Caso contrário, usar o valor inline.
3. Definir **os dois** (`token` e `token_env`, `senha` e `senha_env`) é erro de configuração.
4. `*_env` definido com variável ausente é erro em qualquer comando que use a conta; `config
   validate --offline` pode rebaixar para warning quando `--no-env` for usado.
5. Valores nunca são serializados em `config show`, logs, envelope, CSV, JSONL ou relatório.

### 6.5 Política de segredo inline

- Permitido para uso local, mas gera `warnings[]`: “segredo inline em open-mtr-config.toml; prefira
  `*_env`”.
- No Unix, se o arquivo tiver bits de leitura/escrita para grupo/outros (`mode & 0o077 != 0`), gerar
  warning forte; se `strict`, vira erro.
- Nunca alterar permissões automaticamente sem `config fix-permissions`.
- `config init` cria o arquivo com `0600` no Unix.

## 7. Seleção de escopo

Flags globais de seleção:

| Flag | Semântica |
|---|---|
| `--account <ID>` | id exato; repetível e/ou lista com vírgula |
| `--system <S>` | todos os enabled do sistema; `sinir|mtr_pe|sigor` |
| `--tag <T>` | filtra contas por tag; repetível/com vírgula; interseção |
| `--all-accounts` | todos os enabled; explícito |

Regras:

1. `--account` conflita com `--system`, `--tag` e `--all-accounts`.
2. `--system` pode combinar com `--tag` (interseção).
3. `--all-accounts` conflita com qualquer outro seletor.
4. **Leitura** sem seletor:
   - se houver exatamente uma conta enabled, ela é implícita;
   - caso contrário, erro `invalid_input` com a lista de contas.
5. **Escrita** (`criar`, `receber`, `cancelar`, `modificar`, `aceitar`, `complementar`, `gerar`,
   `download` quando materializa arquivo) exige **exatamente uma conta**. `--all-accounts`,
   `--system` com >1 resultado ou `--tag` com >1 resultado são erro.
6. `dump` sem seletor é erro: o usuário deve declarar `--all-accounts`, `--system`, `--tag` ou
   `--account`.
7. `config`, `self` e `version` não exigem seletor.

A resolução final é sempre uma lista ordenada por `id` antes de executar. O CLI registra em stderr
`scope: account=<id> system=<s>` por operação; no `--json`, o campo `accounts` é explícito.

## 8. Validação (`openmtr config validate`)

Checagens obrigatórias:

- `config_version` presente e suportado.
- `accounts` não vazio após filtro de enabled.
- `id` único e no formato.
- `system` existe no enum `System` e tem adapter implementado.
- `base_url` parseável, esquema `http`/`https`, sem query/fragmento.
- `auth.type` conhecido e campos obrigatórios completos.
- MTR-PE com `cpf_senha_unidade` exige `cnpj`.
- `unidade > 0`.
- `timeout_secs` e `max_concurrency` dentro do range.
- `token`/`token_env` e `senha`/`senha_env` mutuamente exclusivos.
- `*_env` com nome válido (`^[A-Z_][A-Z0-9_]*$`) e existência verificada no modo online.
- `default_account`, se definido, aponta para conta existente e habilitada.
- tags normalizadas (`trim`, lowercase) e sem duplicatas.
- permissões do arquivo (Unix) verificadas para segredo inline.

Saída:

```json
{
  "success": true,
  "command": "config.validate",
  "data": {
    "config_path": "/home/user/open-mtr-config.toml",
    "config_version": 1,
    "accounts": 4,
    "enabled": 3,
    "warnings": ["segredo inline na conta mtrpe-empresa"]
  }
}
```

## 9. Comandos de configuração

```bash
openmtr config init [--force] [--from-env] [--template basic|multi-account]
openmtr config validate [--offline] [--json]
openmtr config show [--redacted] [--json]
openmtr config accounts [--json]
openmtr config edit [--editor <CMD>]
openmtr config migrate [--dry-run] [--from <PATH>]
openmtr config fix-permissions
```

- `init` nunca sobrescreve sem `--force`; com `--force`, move o arquivo anterior para
  `open-mtr-config.toml.bak.<timestamp>` (timestamp só no nome do backup, não na saída de dados).
- `init --from-env` importa explicitamente os nomes conhecidos do `_OPEN_MTR_API`:
  `OPENMTR_SINIR_TOKEN`, `MTR_PE_{CNPJ,CPF,PASSWORD,UNITY}` e
  `MTR_PE_TRANSPORTADORA_{CNPJ,CPF,PASSWORD,UNITY}`; cria contas `sinir-default`,
  `mtrpe-default` e `mtrpe-transportadora`.
- `edit` abre o `$EDITOR` (ou `--editor`) e roda `validate` ao fechar. Não é interativo em TUI.
- `migrate` lê `config_version` anterior, aplica transformações puras e escreve canonicamente;
  `--dry-run` imprime o diff (stdout de dados) sem gravar.
- `fix-permissions` aplica `0600` no Unix quando o usuário pedir explicitamente.

## 10. Migração e compatibilidade

- `config_version = 1` é a primeira versão pública.
- Versões futuras devem:
  1. declarar `config_version` novo;
  2. manter leitura da versão anterior por pelo menos uma minor;
  3. ter `config migrate` com teste de fixture antigo → novo;
  4. falhar com mensagem clara se a versão for mais nova que o binário.
- Nunca remover campo sem antes: warning de depreciação → migração → remoção em major.
- Contas desabilitadas continuam no arquivo; não são apagadas por migração.

## 11. Exemplo de config multi-conta do mesmo sistema

Caso real extraído do `dump.ts`: empresa + transportadora, ambas MTR-PE.

```toml
config_version = 1
default_account = "mtrpe-empresa"

[defaults]
max_concurrency = 6
dump_window_days = 30
output_dir = "dump"

[[accounts]]
id = "mtrpe-empresa"
system = "mtr_pe"
label = "MTR-PE empresa"
tags = ["prod", "pe"]

[accounts.auth]
type = "cpf_senha_unidade"
cpf = "00000000000"
cnpj = "00000000000000"
unidade = 3623
senha_env = "MTR_PE_PASSWORD"

[[accounts]]
id = "mtrpe-transportadora"
system = "mtr_pe"
label = "MTR-PE transportadora"
tags = ["prod", "pe", "transportadora"]

[accounts.auth]
type = "cpf_senha_unidade"
cpf = "00000000000"
cnpj = "00000000000000"
unidade = 9999
senha_env = "MTR_PE_TRANSPORTADORA_PASSWORD"
```

`openmtr dump --system mtr_pe --all-accounts` não é permitido; use `--system mtr_pe` (com as duas
contas selecionadas) ou `--account mtrpe-empresa --account mtrpe-transportadora`. A saída distingue
as duas pelo campo `account`, nunca por um `sistema` sintético.

## 12. Questões em aberto

- `config set`/`unset` por chave entra na v1 ou fica para depois? **Proposta:** não na v1; edição é
  manual/`config edit`, com validação. Reduz superfície e evita reescrever TOML.
- Suporte a `keyring`/secret manager: **fora de escopo da v1**; `*_env` permite integração externa.
- Múltiplos arquivos de config (include): **fora de escopo**; um arquivo cumulativo basta.
- `default_account`: **não** usar como fallback automático de escrita; apenas exibir sugestão e
  permitir `--account @default` como alias explícito? **Proposta:** v1 sem alias; usuário escreve o
  id.
