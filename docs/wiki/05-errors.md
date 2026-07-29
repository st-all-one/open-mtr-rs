# Sistema de Erros

Dois tipos de erro seguindo RFC 9457 (Problem Details).

## OpenMTRError — Erro Catastrófico

A requisição não completou. Usado no `Err()` do `Result`.

```rust
pub struct OpenMTRError {
    pub r#type: Url,        // urn:openmtr:error:{tipo}
    pub title: String,       // "Erro de rede"
    pub status: u16,         // 503
    pub detail: String,      // "connection refused"
    pub instance: Uuid,      // 019fae0a-...
}
```

### Construtores

| Método | status | Uso |
|--------|--------|-----|
| `OpenMTRError::network("...")` | 503 | Falha de rede |
| `OpenMTRError::timeout("...")` | 504 | Timeout |
| `OpenMTRError::auth("...")` | 401 | Falha de autenticação |
| `OpenMTRError::validation("...")` | 422 | Erro de validação |
| `OpenMTRError::config("...")` | 500 | Erro de configuração |
| `OpenMTRError::not_supported("...")` | 501 | Operação não suportada |
| `OpenMTRError::new(status, title, detail)` | — | Genérico |

### Serialização

```json
{
  "type": "urn:openmtr:error:network",
  "title": "Erro de rede",
  "status": 503,
  "detail": "connection refused",
  "instance": "019fae0a-6c82-7840-a7d2-282ae8c71630"
}
```

## Problema — Erro de Negócio

A API respondeu mas recusou ou apontou inconsistência. Usado dentro de `Resposta.erros`.

```rust
pub struct Problema {
    pub r#type: Url,
    pub title: String,
    pub status: u16,
    pub detail: String,
    pub instance: Uuid,
}
```

### Construtores

| Método | status | Uso |
|--------|--------|-----|
| `Problema::validation("...")` | 422 | Validação |
| `Problema::not_found("...")` | 404 | Não encontrado |
| `Problema::api(status, "..." )` | — | Erro de API genérico |
| `Problema::new(status, title, detail)` | — | Genérico |

## Fluxo

```
Requisição
  ├── Rede caiu / timeout / 401 sem refresh
  │     └── Err(OpenMTRError)
  │
  ├── HTTP 200, API retornou erro(s)
  │     └── Ok(Resposta { ok: true, erros: [Problema, ...] })
  │
  └── HTTP 200, sucesso
        └── Ok(Resposta { ok: true, dados: [...], erros: [] })
```

## Exemplo

```rust
match mtr.consultar_mtrs(&filtro).await {
    Ok(resp) => {
        for m in &resp.dados { println!("OK: {}", m.numero); }
        for e in &resp.erros { eprintln!("Erro: {}", e.detail); }
    }
    Err(cat) => {
        eprintln!("Catastrófico: {} — {}", cat.title, cat.detail);
    }
}
```
