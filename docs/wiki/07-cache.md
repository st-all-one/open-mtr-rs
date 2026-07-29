# Cache

O cache **não é responsabilidade do openMTR Core**.

O openMTR é um shell unificado que apenas roteia chamadas para o adapter correto. Quem cuida do cache é a **implementação final** (ex: uma aplicação que usa a biblioteca).

## Recomendações

Cada aplicação pode implementar cache de acordo com sua necessidade:

```rust
// Exemplo: cache simples com HashMap + TTL na aplicação
struct MeuApp {
    mtr: OpenMtrClient,
    cache: Arc<RwLock<HashMap<String, (Instant, Vec<ResiduoIbama>)>>>,
}

impl MeuApp {
    async fn listar_residuos(&self) -> Result<Vec<ResiduoIbama>, OpenMTRError> {
        // Verificar cache antes de chamar a API
        // ...
    }
}
```

## Tabelas de Apoio (Offline)

O módulo `domain::tabela` contém mapas estáticos `LazyLock` para compatibilidade entre acondicionamentos e estados físicos:

```rust
COMPATIBILIDADE_ACOND: HashMap<u32, Vec<u32>>
ESTADOS_FISICOS: HashMap<u32, &'static str>
```

Esses mapas são embutidos no código e servem como fallback offline quando a API não está disponível.
