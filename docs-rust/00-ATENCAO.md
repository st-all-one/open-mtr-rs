# docs-rust/ — AVISO

Esta documentação é **histórica/arquitetural** e reflete um estágio anterior do
projeto. Pode conter APIs, structs e fluxos que **não correspondem** ao código
atual.

## Principais diferenças

| Tópico | docs-rust/ (desatualizado) | Código real |
|--------|---------------------------|-------------|
| `ObjetoRespostaRest` | `objeto: Option<T>` | `objeto_resposta: Option<T>`, `mensagem: Option<String>`, `total_records: Option<i64>` |
| `ManifestoResumo` | 4 campos | ~40 campos completos |
| Cnpj | Função `validar_cnpj()` | `Cnpj` newtype |
| Cpf | Não documentado | `Cpf`, `CpfCnpj` newtypes |
| `ApiwsListaManifestoDTO` | 3 campos | 5 campos (+ pagina, tamanho_pagina) |
| `MtrError` | Simples, 5 variantes | RFC 7807 completo (ErrorCategory, ErrorContext, UUID v7) |
| Services | Assinaturas antigas | Usam builder pattern, retornam tuplas |
| Testes | Unitários em `tests/` | Movidos inline `#[cfg(test)]` nos schemas |
| Tabelas | Só `TipoAcondicionamento` | + UnidadeMedida, EstadoFisico, ClasseResiduo, TratamentoResiduo |
| `trait Endpoint` | Não documentado | `client::endpoint` + `PaginatedIter` |
| Builders | Não documentados | `ManifestoListBuilder`, `SalvarManifestoLoteBuilder` (typestate) |

## Documentação atualizada

Consulte `docs/wiki/` para documentação precisa do código atual.
