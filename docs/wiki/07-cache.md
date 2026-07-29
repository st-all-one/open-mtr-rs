# Cache

## TabelasService (cache manual)

Cache de 24h via `Arc<RwLock<Option<CacheEntry>>>`:

```rust
struct CacheEntry<T> {
    data: T,
    timestamp: Instant,
}
```

```rust
pub async fn listar_todos(&self) -> Result<Vec<TipoAcondicionamento>, MtrError>
pub async fn listar_por_estado_fisico(&self, tie_codigo) -> Result<Vec<TipoAcondicionamento>, MtrError>
pub async fn invalidate_cache(&self)
```

### Fallback Offline

```rust
TabelasService::get_compativeis(1)  // → &[1, 2, 3, 6, 8]
TabelasService::get_nome_estado_fisico(4)  // → "Gasoso"
```

Usa `LazyLock<HashMap<i32, Vec<i32>>>` — sem acesso à rede.

### moka (alternativa produção)

```rust
use moka::future::Cache;

pub struct TabelasServiceMoka {
    client: MtrClient,
    cache: Cache<i32, Vec<TipoAcondicionamento>>,
}
```

- TTL 24h
- Capacidade máxima 100 entries
- Thread-safe nativo (sharded)

## LazyLock Statics

```rust
// core/src/schemas/tabelas.rs
pub static COMPATIBILIDADE_ACOND: LazyLock<HashMap<i32, Vec<i32>>>
pub static ESTADOS_FISICOS: LazyLock<HashMap<i32, &str>>
```

Inicialização sob demanda, thread-safe, sem custo de sincronização após init.

Compatibilidade:

| Estado Físico | Códigos de Acondicionamento |
|---------------|-----------------------------|
| Sólido (1) | 1, 2, 3, 6, 8 |
| Líquido (2) | 2, 5, 9 |
| Pastoso (3) | 2, 5, 6 |
| Gasoso (4) | 9 |
| Multifásico (5) | 1, 2, 5, 9 |

## Pré-aquecimento

```rust
pub async fn preaquecer_cache(service: &TabelasService) -> Result<(), MtrError> {
    let sem = Semaphore::new(5); // max 5 simultâneos
    for estado in 1..=5 {
        let _permit = sem.acquire().await.unwrap();
        service.listar_por_estado_fisico(estado).await?;
    }
    Ok(())
}
```
