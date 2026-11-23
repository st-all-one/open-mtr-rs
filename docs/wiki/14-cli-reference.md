# Referência da CLI `openmtr`

> Gerado automaticamente por `scripts/gen-cli-docs.sh` a partir de `openmtr 0.2.0`.
> Não edite à mão: rode `make cli-docs` e versione o resultado.

## `openmtr --help`

```text
CLI utilitária multi-conta do openMTR

Usage: openmtr [OPTIONS] <COMMAND>

Commands:
  account  Gerencia as contas configuradas
  config   Valida e migra o arquivo de configuração
  mtr      Consulta e altera MTRs
  cdf      Consulta, gera e baixa CDFs
  ref      Consulta tabelas de referência do sistema
  health   Verifica o binário sem exigir configuração nem chamar a API
  dump     Exporta MTRs de várias contas para arquivos locais
  batch    Executa um lote de operações lidas de um JSONL
  self     Mostra a versão ou atualiza o próprio binário
  help     Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>          Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`) [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída
      --fixed-time <FIXED_TIME>  Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`
      --quiet                    Suprime a inicialização dos logs (a saída de dados permanece)
      --log-level <LOG_LEVEL>    Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)
  -h, --help                     Print help
  -V, --version                  Print version

ENTRADA
  --login <JSON|@ARQUIVO|->  credenciais explícitas por sistema (dispensa --config)
  --params <JSON|->          payload do comando
  --file <JSONL|->           lote do `batch`

POSICIONAIS
  Quando existir, o posicional é o ALVO primário do comando (NUMERO, CODIGO ou
  TABELA) e é mutuamente exclusivo com --params. Nenhum comando lê stdin
  implicitamente: use `-` explícito.

SAÍDA
  stdout = dados; stderr = logs. `--json` emite JSON de máquina.
```

## `openmtr account`

```text
Gerencia as contas configuradas

Usage: openmtr account [OPTIONS] <COMMAND>

Commands:
  list  Lista as contas habilitadas (id, sistema, acesso e tags)
  test  Testa uma conta configurada pelo ID
  help  Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>          Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`) [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída
      --fixed-time <FIXED_TIME>  Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`
      --quiet                    Suprime a inicialização dos logs (a saída de dados permanece)
      --log-level <LOG_LEVEL>    Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)
  -h, --help                     Print help
```

## `openmtr account list`

```text
Lista as contas habilitadas (id, sistema, acesso e tags)

Usage: openmtr account list [OPTIONS]

Options:
      --config <CONFIG>          Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`) [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída
      --fixed-time <FIXED_TIME>  Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`
      --quiet                    Suprime a inicialização dos logs (a saída de dados permanece)
      --log-level <LOG_LEVEL>    Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)
  -h, --help                     Print help
```

## `openmtr account test`

```text
Testa uma conta configurada pelo ID

Usage: openmtr account test [OPTIONS] <ID>

Arguments:
  <ID>  ID da conta a testar

Options:
      --config <CONFIG>          Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`) [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída
      --fixed-time <FIXED_TIME>  Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`
      --quiet                    Suprime a inicialização dos logs (a saída de dados permanece)
      --log-level <LOG_LEVEL>    Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)
  -h, --help                     Print help
```

## `openmtr config`

```text
Valida e migra o arquivo de configuração

Usage: openmtr config [OPTIONS] <COMMAND>

Commands:
  validate  Valida o arquivo de configuração carregado
  migrate   Migra o arquivo de configuração para a versão atual
  help      Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>          Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`) [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída
      --fixed-time <FIXED_TIME>  Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`
      --quiet                    Suprime a inicialização dos logs (a saída de dados permanece)
      --log-level <LOG_LEVEL>    Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)
  -h, --help                     Print help
```

## `openmtr config validate`

```text
Valida o arquivo de configuração carregado

Usage: openmtr config validate [OPTIONS]

Options:
      --config <CONFIG>          Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`) [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída
      --fixed-time <FIXED_TIME>  Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`
      --quiet                    Suprime a inicialização dos logs (a saída de dados permanece)
      --log-level <LOG_LEVEL>    Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)
  -h, --help                     Print help
```

## `openmtr config migrate`

```text
Migra o arquivo de configuração para a versão atual

Usage: openmtr config migrate [OPTIONS]

Options:
      --config <CONFIG>          Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`) [default: open-mtr-config.toml]
      --dry-run                  Mostra a migração sem gravar alterações no arquivo
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída
      --fixed-time <FIXED_TIME>  Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`
      --quiet                    Suprime a inicialização dos logs (a saída de dados permanece)
      --log-level <LOG_LEVEL>    Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)
  -h, --help                     Print help
```

## `openmtr mtr`

```text
Consulta e altera MTRs

Usage: openmtr mtr [OPTIONS] <COMMAND>

Commands:
  consultar  Consulta MTRs por número (posicional) ou por filtro em `--params`
  criar      Cria MTRs a partir de um payload em JSON
  receber    Registra o recebimento de MTRs
  cancelar   Cancela MTRs existentes
  modificar  Modifica MTRs existentes
  aceitar    Aceita MTRs recebidos
  download   Baixa PDFs de um ou mais números
  help       Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>          Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`) [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída
      --fixed-time <FIXED_TIME>  Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`
      --quiet                    Suprime a inicialização dos logs (a saída de dados permanece)
      --log-level <LOG_LEVEL>    Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)
  -h, --help                     Print help
```

## `openmtr mtr consultar`

```text
Consulta MTRs por número (posicional) ou por filtro em `--params`

Usage: openmtr mtr consultar [OPTIONS] [NUMERO]

Arguments:
  [NUMERO]  Número do MTR; exclusivo com `--params`

Options:
      --account <ACCOUNT>        ID da conta a consultar; padrão quando há uma única conta
      --config <CONFIG>          Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`) [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --system <SYSTEM>          Filtra contas pelo slug do sistema (`sinir`, `mtr_pe`, `sigor`)
      --json                     Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída
      --tag <TAG>                Filtra contas por tag; pode ser repetido
      --all-accounts             Consulta em todas as contas habilitadas
      --fixed-time <FIXED_TIME>  Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`
      --params <JSON|->          Filtro de consulta em JSON, ou `-` para lê-lo de stdin
      --quiet                    Suprime a inicialização dos logs (a saída de dados permanece)
      --log-level <LOG_LEVEL>    Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)
  -h, --help                     Print help
```

## `openmtr mtr criar`

```text
Cria MTRs a partir de um payload em JSON

Usage: openmtr mtr criar [OPTIONS] --params <JSON|->

Options:
      --account <ACCOUNT>        ID da conta; padrão quando há uma única conta
      --config <CONFIG>          Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`) [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --params <JSON|->          Payload de criação em JSON, ou `-` para stdin (obrigatório)
      --json                     Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída
      --fixed-time <FIXED_TIME>  Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`
      --quiet                    Suprime a inicialização dos logs (a saída de dados permanece)
      --log-level <LOG_LEVEL>    Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)
  -h, --help                     Print help
```

## `openmtr mtr receber`

```text
Registra o recebimento de MTRs

Usage: openmtr mtr receber [OPTIONS] --params <JSON|->

Options:
      --account <ACCOUNT>        ID da conta; padrão quando há uma única conta
      --config <CONFIG>          Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`) [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --params <JSON|->          Payload de recebimento em JSON, ou `-` para stdin (obrigatório)
      --json                     Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída
      --fixed-time <FIXED_TIME>  Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`
      --quiet                    Suprime a inicialização dos logs (a saída de dados permanece)
      --log-level <LOG_LEVEL>    Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)
  -h, --help                     Print help
```

## `openmtr mtr cancelar`

```text
Cancela MTRs existentes

Usage: openmtr mtr cancelar [OPTIONS] --params <JSON|->

Options:
      --account <ACCOUNT>        ID da conta; padrão quando há uma única conta
      --config <CONFIG>          Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`) [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --params <JSON|->          Payload de cancelamento em JSON, ou `-` para stdin (obrigatório)
      --json                     Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída
      --fixed-time <FIXED_TIME>  Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`
      --quiet                    Suprime a inicialização dos logs (a saída de dados permanece)
      --log-level <LOG_LEVEL>    Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)
  -h, --help                     Print help
```

## `openmtr mtr modificar`

```text
Modifica MTRs existentes

Usage: openmtr mtr modificar [OPTIONS] --params <JSON|->

Options:
      --account <ACCOUNT>        ID da conta; padrão quando há uma única conta
      --config <CONFIG>          Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`) [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --params <JSON|->          Payload de modificação em JSON, ou `-` para stdin (obrigatório)
      --json                     Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída
      --fixed-time <FIXED_TIME>  Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`
      --quiet                    Suprime a inicialização dos logs (a saída de dados permanece)
      --log-level <LOG_LEVEL>    Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)
  -h, --help                     Print help
```

## `openmtr mtr aceitar`

```text
Aceita MTRs recebidos

Usage: openmtr mtr aceitar [OPTIONS] --params <JSON|->

Options:
      --account <ACCOUNT>        ID da conta; padrão quando há uma única conta
      --config <CONFIG>          Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`) [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --params <JSON|->          Payload de aceite em JSON, ou `-` para stdin (obrigatório)
      --json                     Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída
      --fixed-time <FIXED_TIME>  Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`
      --quiet                    Suprime a inicialização dos logs (a saída de dados permanece)
      --log-level <LOG_LEVEL>    Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)
  -h, --help                     Print help
```

## `openmtr mtr download`

```text
Baixa PDFs de um ou mais números

Usage: openmtr mtr download [OPTIONS] --out <OUT> [NUMERO]

Arguments:
  [NUMERO]  Número do MTR; exclusivo com `--params`

Options:
      --account <ACCOUNT>        ID da conta; padrão quando há uma única conta
      --config <CONFIG>          Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`) [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --params <JSON|->          Números ou filtro em JSON, ou `-` para stdin
      --json                     Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída
      --out <OUT>                Diretório onde salvar os PDFs baixados
      --fixed-time <FIXED_TIME>  Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`
      --quiet                    Suprime a inicialização dos logs (a saída de dados permanece)
      --log-level <LOG_LEVEL>    Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)
  -h, --help                     Print help
```

## `openmtr cdf`

```text
Consulta, gera e baixa CDFs

Usage: openmtr cdf [OPTIONS] <COMMAND>

Commands:
  consultar  Consulta CDFs por código (posicional) ou por filtro em `--params`
  gerar      Gera CDFs a partir de um payload em JSON
  download   Baixa os PDFs dos CDFs pedidos no payload
  help       Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>          Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`) [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída
      --fixed-time <FIXED_TIME>  Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`
      --quiet                    Suprime a inicialização dos logs (a saída de dados permanece)
      --log-level <LOG_LEVEL>    Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)
  -h, --help                     Print help
```

## `openmtr cdf consultar`

```text
Consulta CDFs por código (posicional) ou por filtro em `--params`

Usage: openmtr cdf consultar [OPTIONS] [CODIGO]

Arguments:
  [CODIGO]  Código do CDF; exclusivo com `--params`

Options:
      --account <ACCOUNT>        ID da conta; padrão quando há uma única conta
      --config <CONFIG>          Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`) [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --params <JSON|->          Filtro de consulta em JSON, ou `-` para lê-lo de stdin
      --json                     Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída
      --fixed-time <FIXED_TIME>  Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`
      --quiet                    Suprime a inicialização dos logs (a saída de dados permanece)
      --log-level <LOG_LEVEL>    Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)
  -h, --help                     Print help
```

## `openmtr cdf gerar`

```text
Gera CDFs a partir de um payload em JSON

Usage: openmtr cdf gerar [OPTIONS] --params <JSON|->

Options:
      --account <ACCOUNT>        ID da conta; padrão quando há uma única conta
      --config <CONFIG>          Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`) [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --params <JSON|->          Payload de geração em JSON, ou `-` para stdin (obrigatório)
      --json                     Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída
      --fixed-time <FIXED_TIME>  Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`
      --quiet                    Suprime a inicialização dos logs (a saída de dados permanece)
      --log-level <LOG_LEVEL>    Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)
  -h, --help                     Print help
```

## `openmtr cdf download`

```text
Baixa os PDFs dos CDFs pedidos no payload

Usage: openmtr cdf download [OPTIONS] --params <JSON|-> --out <OUT>

Options:
      --account <ACCOUNT>        ID da conta; padrão quando há uma única conta
      --config <CONFIG>          Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`) [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --params <JSON|->          Payload com os CDFs em JSON, ou `-` para stdin (obrigatório)
      --json                     Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída
      --out <OUT>                Diretório onde salvar os PDFs baixados
      --fixed-time <FIXED_TIME>  Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`
      --quiet                    Suprime a inicialização dos logs (a saída de dados permanece)
      --log-level <LOG_LEVEL>    Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)
  -h, --help                     Print help
```

## `openmtr ref`

```text
Consulta tabelas de referência do sistema

Usage: openmtr ref [OPTIONS] <COMMAND>

Commands:
  listar             Lista uma tabela de referência
  por-residuo        Classes compatíveis com um código IBAMA
  por-estado-fisico  Acondicionamentos de um estado físico
  help               Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>          Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`) [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída
      --fixed-time <FIXED_TIME>  Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`
      --quiet                    Suprime a inicialização dos logs (a saída de dados permanece)
      --log-level <LOG_LEVEL>    Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)
  -h, --help                     Print help
```

## `openmtr ref listar`

```text
Lista uma tabela de referência

Usage: openmtr ref listar [OPTIONS] <TABELA>

Arguments:
  <TABELA>
          Tabela a listar: residuo|classe|unidade|acondicionamento|estado-fisico|tratamento

          Possible values:
          - residuo:          Resíduos (`residuo`, alias: `residuos`)
          - classe:           Classes de resíduos (`classe`, alias: `classes`)
          - unidade:          Unidades de medida (`unidade`, alias: `unidades`)
          - acondicionamento: Acondicionamentos (`acondicionamento`, alias: `acondicionamentos`)
          - estado-fisico:    Estados físicos (`estado-fisico`, aliases: `estado_fisico`, `estados_fisicos`)
          - tratamento:       Tratamentos (`tratamento`, alias: `tratamentos`)

Options:
      --account <ACCOUNT>
          ID da conta; padrão quando há uma única conta

      --config <CONFIG>
          Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`)
          
          [default: open-mtr-config.toml]

      --login <JSON|@ARQUIVO|->
          Credenciais explícitas por sistema (dispensa `--config`)

      --json
          Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída

      --fixed-time <FIXED_TIME>
          Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`

      --quiet
          Suprime a inicialização dos logs (a saída de dados permanece)

      --log-level <LOG_LEVEL>
          Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)

  -h, --help
          Print help (see a summary with '-h')
```

## `openmtr ref por-residuo`

```text
Classes compatíveis com um código IBAMA

Usage: openmtr ref por-residuo [OPTIONS] <CODIGO>

Arguments:
  <CODIGO>  Código IBAMA do resíduo

Options:
      --account <ACCOUNT>        ID da conta; padrão quando há uma única conta
      --config <CONFIG>          Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`) [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída
      --fixed-time <FIXED_TIME>  Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`
      --quiet                    Suprime a inicialização dos logs (a saída de dados permanece)
      --log-level <LOG_LEVEL>    Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)
  -h, --help                     Print help
```

## `openmtr ref por-estado-fisico`

```text
Acondicionamentos de um estado físico

Usage: openmtr ref por-estado-fisico [OPTIONS] <CODIGO>

Arguments:
  <CODIGO>  Código numérico do estado físico

Options:
      --account <ACCOUNT>        ID da conta; padrão quando há uma única conta
      --config <CONFIG>          Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`) [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída
      --fixed-time <FIXED_TIME>  Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`
      --quiet                    Suprime a inicialização dos logs (a saída de dados permanece)
      --log-level <LOG_LEVEL>    Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)
  -h, --help                     Print help
```

## `openmtr health`

```text
Verifica o binário sem exigir configuração nem chamar a API

Usage: openmtr health [OPTIONS]

Options:
      --config <CONFIG>          Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`) [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída
      --fixed-time <FIXED_TIME>  Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`
      --quiet                    Suprime a inicialização dos logs (a saída de dados permanece)
      --log-level <LOG_LEVEL>    Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)
  -h, --help                     Print help
```

## `openmtr dump`

```text
Exporta MTRs de várias contas para arquivos locais

Usage: openmtr dump [OPTIONS] --from <FROM> --to <TO>

Options:
      --all-accounts               Inclui todas as contas habilitadas no dump
      --config <CONFIG>            Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`) [default: open-mtr-config.toml]
      --account <ACCOUNT>          ID da conta a incluir; pode ser repetido
      --login <JSON|@ARQUIVO|->    Credenciais explícitas por sistema (dispensa `--config`)
      --json                       Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída
      --system <SYSTEM>            Filtra contas pelo slug do sistema (`sinir`, `mtr_pe`, `sigor`)
      --fixed-time <FIXED_TIME>    Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`
      --tag <TAG>                  Filtra contas por tag; pode ser repetido
      --from <FROM>                Data inicial do período, no formato `YYYY-MM-DD`
      --quiet                      Suprime a inicialização dos logs (a saída de dados permanece)
      --log-level <LOG_LEVEL>      Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)
      --to <TO>                    Data final do período, no formato `YYYY-MM-DD`
      --window-days <WINDOW_DAYS>  Número de dias por janela de consulta (padrão: 30) [default: 30]
      --out <OUT>                  Diretório de saída dos arquivos gerados (padrão: `dump`) [default: dump]
      --pdf                        Baixa também os PDFs dos MTRs
      --no-dedup                   Mantém duplicatas entre contas em vez de removê-las
      --resume                     Retoma um dump interrompido a partir do progresso salvo
      --force                      Descarta o estado anterior e refaz o dump do zero
      --format <FORMAT>            Arquivos a gerar: `both`, `csv`, `jsonl` ou `none` (padrão: `both`) [default: both]
      --concurrency <CONCURRENCY>  Número máximo de requisições concorrentes (padrão: 10) [default: 10]
  -h, --help                       Print help
```

## `openmtr batch`

```text
Executa um lote de operações lidas de um JSONL

Usage: openmtr batch [OPTIONS]

Options:
      --config <CONFIG>          Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`) [default: open-mtr-config.toml]
      --file <JSONL|->           JSONL de operações; use `-` para stdin
      --continue-on-error        Continua o lote após uma operação falhar
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --dry-run                  Valida o lote sem executar as operações
      --json                     Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída
      --fixed-time <FIXED_TIME>  Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`
      --quiet                    Suprime a inicialização dos logs (a saída de dados permanece)
      --log-level <LOG_LEVEL>    Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)
  -h, --help                     Print help
```

## `openmtr self`

```text
Mostra a versão ou atualiza o próprio binário

Usage: openmtr self [OPTIONS] <COMMAND>

Commands:
  version  Exibe a versão instalada do `openmtr`
  update   Verifica ou aplica atualizações do próprio binário
  help     Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>          Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`) [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída
      --fixed-time <FIXED_TIME>  Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`
      --quiet                    Suprime a inicialização dos logs (a saída de dados permanece)
      --log-level <LOG_LEVEL>    Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)
  -h, --help                     Print help
```

## `openmtr self version`

```text
Exibe a versão instalada do `openmtr`

Usage: openmtr self version [OPTIONS]

Options:
      --config <CONFIG>          Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`) [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída
      --fixed-time <FIXED_TIME>  Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`
      --quiet                    Suprime a inicialização dos logs (a saída de dados permanece)
      --log-level <LOG_LEVEL>    Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)
  -h, --help                     Print help
```

## `openmtr self update`

```text
Verifica ou aplica atualizações do próprio binário

Usage: openmtr self update [OPTIONS]

Options:
      --check                    Apenas verifica se existe versão mais recente
      --config <CONFIG>          Arquivo de configuração multi-conta (padrão: `open-mtr-config.toml`) [default: open-mtr-config.toml]
      --dry-run                  Mostra o plano de atualização sem aplicá-lo
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     Emite JSON de máquina (pretty) em vez de JSON compacto/texto na saída
      --version <VERSION>        Versão específica a instalar (tag do release)
      --fixed-time <FIXED_TIME>  Congela o relógio para saída determinística: RFC 3339 ou `YYYY-MM-DD`
      --force                    Força a atualização mesmo sem versão mais recente
      --quiet                    Suprime a inicialização dos logs (a saída de dados permanece)
      --log-level <LOG_LEVEL>    Filtro de logs do `tracing` (padrão: `warn` ou `RUST_LOG`)
  -h, --help                     Print help
```
