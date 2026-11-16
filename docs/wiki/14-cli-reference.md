# Referência da CLI `openmtr`

> Gerado automaticamente por `scripts/gen-cli-docs.sh` a partir de `openmtr 0.2.0`.
> Não edite à mão: rode `make cli-docs` e versione o resultado.

## `openmtr --help`

```text
CLI utilitária multi-conta do openMTR

Usage: openmtr [OPTIONS] <COMMAND>

Commands:
  account  
  config   
  mtr      
  cdf      
  ref      
  health   
  dump     
  batch    
  self     
  help     Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>          [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     
      --fixed-time <FIXED_TIME>  
      --quiet                    
      --log-level <LOG_LEVEL>    
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
Usage: openmtr account [OPTIONS] <COMMAND>

Commands:
  list  
  test  
  help  Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>          [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     
      --fixed-time <FIXED_TIME>  
      --quiet                    
      --log-level <LOG_LEVEL>    
  -h, --help                     Print help
```

## `openmtr account list`

```text
Usage: openmtr account list [OPTIONS]

Options:
      --config <CONFIG>          [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     
      --fixed-time <FIXED_TIME>  
      --quiet                    
      --log-level <LOG_LEVEL>    
  -h, --help                     Print help
```

## `openmtr account test`

```text
Usage: openmtr account test [OPTIONS] <ID>

Arguments:
  <ID>  

Options:
      --config <CONFIG>          [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     
      --fixed-time <FIXED_TIME>  
      --quiet                    
      --log-level <LOG_LEVEL>    
  -h, --help                     Print help
```

## `openmtr config`

```text
Usage: openmtr config [OPTIONS] <COMMAND>

Commands:
  validate  
  migrate   
  help      Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>          [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     
      --fixed-time <FIXED_TIME>  
      --quiet                    
      --log-level <LOG_LEVEL>    
  -h, --help                     Print help
```

## `openmtr config validate`

```text
Usage: openmtr config validate [OPTIONS]

Options:
      --config <CONFIG>          [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     
      --fixed-time <FIXED_TIME>  
      --quiet                    
      --log-level <LOG_LEVEL>    
  -h, --help                     Print help
```

## `openmtr config migrate`

```text
Usage: openmtr config migrate [OPTIONS]

Options:
      --config <CONFIG>          [default: open-mtr-config.toml]
      --dry-run                  
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     
      --fixed-time <FIXED_TIME>  
      --quiet                    
      --log-level <LOG_LEVEL>    
  -h, --help                     Print help
```

## `openmtr mtr`

```text
Usage: openmtr mtr [OPTIONS] <COMMAND>

Commands:
  consultar  Consulta MTRs por número (posicional) ou por filtro em `--params`
  criar      
  receber    
  cancelar   
  modificar  
  aceitar    
  download   Baixa PDFs de um ou mais números
  help       Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>          [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     
      --fixed-time <FIXED_TIME>  
      --quiet                    
      --log-level <LOG_LEVEL>    
  -h, --help                     Print help
```

## `openmtr mtr consultar`

```text
Consulta MTRs por número (posicional) ou por filtro em `--params`

Usage: openmtr mtr consultar [OPTIONS] [NUMERO]

Arguments:
  [NUMERO]  Número do MTR; exclusivo com `--params`

Options:
      --account <ACCOUNT>        
      --config <CONFIG>          [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --system <SYSTEM>          
      --json                     
      --tag <TAG>                
      --all-accounts             
      --fixed-time <FIXED_TIME>  
      --params <JSON|->          
      --quiet                    
      --log-level <LOG_LEVEL>    
  -h, --help                     Print help
```

## `openmtr mtr criar`

```text
Usage: openmtr mtr criar [OPTIONS] --params <JSON|->

Options:
      --account <ACCOUNT>        
      --config <CONFIG>          [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --params <JSON|->          
      --json                     
      --fixed-time <FIXED_TIME>  
      --quiet                    
      --log-level <LOG_LEVEL>    
  -h, --help                     Print help
```

## `openmtr mtr receber`

```text
Usage: openmtr mtr receber [OPTIONS] --params <JSON|->

Options:
      --account <ACCOUNT>        
      --config <CONFIG>          [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --params <JSON|->          
      --json                     
      --fixed-time <FIXED_TIME>  
      --quiet                    
      --log-level <LOG_LEVEL>    
  -h, --help                     Print help
```

## `openmtr mtr cancelar`

```text
Usage: openmtr mtr cancelar [OPTIONS] --params <JSON|->

Options:
      --account <ACCOUNT>        
      --config <CONFIG>          [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --params <JSON|->          
      --json                     
      --fixed-time <FIXED_TIME>  
      --quiet                    
      --log-level <LOG_LEVEL>    
  -h, --help                     Print help
```

## `openmtr mtr modificar`

```text
Usage: openmtr mtr modificar [OPTIONS] --params <JSON|->

Options:
      --account <ACCOUNT>        
      --config <CONFIG>          [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --params <JSON|->          
      --json                     
      --fixed-time <FIXED_TIME>  
      --quiet                    
      --log-level <LOG_LEVEL>    
  -h, --help                     Print help
```

## `openmtr mtr aceitar`

```text
Usage: openmtr mtr aceitar [OPTIONS] --params <JSON|->

Options:
      --account <ACCOUNT>        
      --config <CONFIG>          [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --params <JSON|->          
      --json                     
      --fixed-time <FIXED_TIME>  
      --quiet                    
      --log-level <LOG_LEVEL>    
  -h, --help                     Print help
```

## `openmtr mtr download`

```text
Baixa PDFs de um ou mais números

Usage: openmtr mtr download [OPTIONS] --out <OUT> [NUMERO]

Arguments:
  [NUMERO]  Número do MTR; exclusivo com `--params`

Options:
      --account <ACCOUNT>        
      --config <CONFIG>          [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --params <JSON|->          
      --json                     
      --out <OUT>                
      --fixed-time <FIXED_TIME>  
      --quiet                    
      --log-level <LOG_LEVEL>    
  -h, --help                     Print help
```

## `openmtr cdf`

```text
Usage: openmtr cdf [OPTIONS] <COMMAND>

Commands:
  consultar  Consulta CDFs por código (posicional) ou por filtro em `--params`
  gerar      
  download   
  help       Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>          [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     
      --fixed-time <FIXED_TIME>  
      --quiet                    
      --log-level <LOG_LEVEL>    
  -h, --help                     Print help
```

## `openmtr cdf consultar`

```text
Consulta CDFs por código (posicional) ou por filtro em `--params`

Usage: openmtr cdf consultar [OPTIONS] [CODIGO]

Arguments:
  [CODIGO]  Código do CDF; exclusivo com `--params`

Options:
      --account <ACCOUNT>        
      --config <CONFIG>          [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --params <JSON|->          
      --json                     
      --fixed-time <FIXED_TIME>  
      --quiet                    
      --log-level <LOG_LEVEL>    
  -h, --help                     Print help
```

## `openmtr cdf gerar`

```text
Usage: openmtr cdf gerar [OPTIONS] --params <JSON|->

Options:
      --account <ACCOUNT>        
      --config <CONFIG>          [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --params <JSON|->          
      --json                     
      --fixed-time <FIXED_TIME>  
      --quiet                    
      --log-level <LOG_LEVEL>    
  -h, --help                     Print help
```

## `openmtr cdf download`

```text
Usage: openmtr cdf download [OPTIONS] --params <JSON|-> --out <OUT>

Options:
      --account <ACCOUNT>        
      --config <CONFIG>          [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --params <JSON|->          
      --json                     
      --out <OUT>                
      --fixed-time <FIXED_TIME>  
      --quiet                    
      --log-level <LOG_LEVEL>    
  -h, --help                     Print help
```

## `openmtr ref`

```text
Usage: openmtr ref [OPTIONS] <COMMAND>

Commands:
  listar             Lista uma tabela de referência
  por-residuo        Classes compatíveis com um código IBAMA
  por-estado-fisico  Acondicionamentos de um estado físico
  help               Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>          [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     
      --fixed-time <FIXED_TIME>  
      --quiet                    
      --log-level <LOG_LEVEL>    
  -h, --help                     Print help
```

## `openmtr ref listar`

```text
Lista uma tabela de referência

Usage: openmtr ref listar [OPTIONS] <TABELA>

Arguments:
  <TABELA>  Tabela de referência [possible values: residuo, classe, unidade, acondicionamento, estado-fisico, tratamento]

Options:
      --account <ACCOUNT>        
      --config <CONFIG>          [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     
      --fixed-time <FIXED_TIME>  
      --quiet                    
      --log-level <LOG_LEVEL>    
  -h, --help                     Print help
```

## `openmtr ref por-residuo`

```text
Classes compatíveis com um código IBAMA

Usage: openmtr ref por-residuo [OPTIONS] <CODIGO>

Arguments:
  <CODIGO>  Código IBAMA do resíduo

Options:
      --account <ACCOUNT>        
      --config <CONFIG>          [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     
      --fixed-time <FIXED_TIME>  
      --quiet                    
      --log-level <LOG_LEVEL>    
  -h, --help                     Print help
```

## `openmtr ref por-estado-fisico`

```text
Acondicionamentos de um estado físico

Usage: openmtr ref por-estado-fisico [OPTIONS] <CODIGO>

Arguments:
  <CODIGO>  Código do estado físico

Options:
      --account <ACCOUNT>        
      --config <CONFIG>          [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     
      --fixed-time <FIXED_TIME>  
      --quiet                    
      --log-level <LOG_LEVEL>    
  -h, --help                     Print help
```

## `openmtr health`

```text
Usage: openmtr health [OPTIONS]

Options:
      --config <CONFIG>          [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     
      --fixed-time <FIXED_TIME>  
      --quiet                    
      --log-level <LOG_LEVEL>    
  -h, --help                     Print help
```

## `openmtr dump`

```text
Usage: openmtr dump [OPTIONS] --from <FROM> --to <TO>

Options:
      --all-accounts               
      --config <CONFIG>            [default: open-mtr-config.toml]
      --account <ACCOUNT>          
      --login <JSON|@ARQUIVO|->    Credenciais explícitas por sistema (dispensa `--config`)
      --json                       
      --system <SYSTEM>            
      --fixed-time <FIXED_TIME>    
      --tag <TAG>                  
      --from <FROM>                
      --quiet                      
      --log-level <LOG_LEVEL>      
      --to <TO>                    
      --window-days <WINDOW_DAYS>  [default: 30]
      --out <OUT>                  [default: dump]
      --pdf                        
      --no-dedup                   
      --resume                     
      --force                      
      --format <FORMAT>            [default: both]
      --concurrency <CONCURRENCY>  [default: 10]
  -h, --help                       Print help
```

## `openmtr batch`

```text
Usage: openmtr batch [OPTIONS]

Options:
      --config <CONFIG>          [default: open-mtr-config.toml]
      --file <JSONL|->           JSONL de operações; use `-` para stdin
      --continue-on-error        
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --dry-run                  
      --json                     
      --fixed-time <FIXED_TIME>  
      --quiet                    
      --log-level <LOG_LEVEL>    
  -h, --help                     Print help
```

## `openmtr self`

```text
Usage: openmtr self [OPTIONS] <COMMAND>

Commands:
  version  
  update   
  help     Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>          [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     
      --fixed-time <FIXED_TIME>  
      --quiet                    
      --log-level <LOG_LEVEL>    
  -h, --help                     Print help
```

## `openmtr self version`

```text
Usage: openmtr self version [OPTIONS]

Options:
      --config <CONFIG>          [default: open-mtr-config.toml]
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     
      --fixed-time <FIXED_TIME>  
      --quiet                    
      --log-level <LOG_LEVEL>    
  -h, --help                     Print help
```

## `openmtr self update`

```text
Usage: openmtr self update [OPTIONS]

Options:
      --check                    
      --config <CONFIG>          [default: open-mtr-config.toml]
      --dry-run                  
      --login <JSON|@ARQUIVO|->  Credenciais explícitas por sistema (dispensa `--config`)
      --json                     
      --version <VERSION>        
      --fixed-time <FIXED_TIME>  
      --force                    
      --quiet                    
      --log-level <LOG_LEVEL>    
  -h, --help                     Print help
```
