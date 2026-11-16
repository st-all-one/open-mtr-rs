# Changelog

Todas as mudanças relevantes do openMTR/CLI. O formato segue
[Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/) e o projeto usa
versionamento semântico.

## [0.2.0] - 2026-11-15

### Adicionado

- CLI: login de sessão única com `--login <JSON|@ARQUIVO|->` para SINIR
  (refresh token), MTR-PE (`cnpj`, `cpf`, `unidade`, `senha`) e SIGOR (token ou
  credenciais clássicas), com `token_env`/`senha_env`, `base_url`,
  `timeout_secs` e `max_rps`; segredos redigidos.
- CLI: posicionais ALVO (`mtr consultar [NUMERO]`, `mtr download [NUMERO]`,
  `cdf consultar [CODIGO]`, `ref listar <TABELA>`, `ref por-residuo <CODIGO>`,
  `ref por-estado-fisico <CODIGO>`), exclusivos com `--params`.
- CLI: stdin explícito (`--params -`, `--file -`, `--login -`), heredoc e um
  único consumidor por invocação.
- CLI: `batch --file` obrigatório, operações `health`, `account.test`,
  `mtr.*`, `cdf.*` e `ref.*`, `--dry-run`, `--continue-on-error` e `account`
  omitível sob `--login`.
- CLI: cache de access token em memória + arquivo (TTL 5 min ou `exp` do JWT
  menos 30 s), chaveado por sistema/conta/URL/credenciais; arquivo `0600` em
  diretório `0700`.
- Core: `OpenMtrClientBuilder::access_token` e `OpenMtrClient::access_token`
  para injetar/ler access token sem repetir autenticação.
- Docs: README e plano da superfície da CLI com login, posicionais, stdin,
  batch, cache e exemplos.
- Testes de unidade/integração para validação de login, conflitos de stdin,
  persistência do cache e batch.

### Alterado

- Rust 1.100, Edition 2024 e MSRV 1.100 (toolchain, `rust-version` e
  `clippy.toml` alinhados).
- Escritas (`mtr criar/receber/cancelar/modificar/aceitar`, `cdf
  gerar/download`) exigem `--params` e uma conta (`--account` ou `--login`);
  `--login` + `--account` é erro também nas escritas.
- `ref listar` usa o mesmo conjunto canônico da CLI no batch, incluindo
  `estado-fisico`.
- Downloads usam apenas o basename do arquivo retornado pelo core; nomes com
  `../` ou caminho absoluto não escapam de `--out`.
- Leitura de `numeros`/`codigos` é estrita: item não-string não é descartado
  silenciosamente.
- `batch` passa a exigir `--file` explícito; `-` continua lendo stdin.

### Corrigido

- Autenticação repetida a cada operação de batch (SINIR fazia `POST /token` e
  MTR-PE/SIGOR clássicos faziam `/gettoken` por cliente).
- Escrita potencial fora de `--out` em `mtr.download`/`cdf.download` do batch.
- Descarte silencioso de itens inválidos em `params.numeros`/`params.codigos`.
- Token/senha vazios passavam na validação de login e falhavam tarde na API.

### Segurança

- Credenciais e refresh tokens não são gravados no cache; somente access token
  e expiração, em arquivo `0600` (diretório `0700`).
- `OPENMTR_NO_SESSION_CACHE=1` desliga a persistência em disco;
  `OPENMTR_SESSION_DIR` sobrescreve o diretório.
- Segredos são redigidos em `Debug`/mensagens de erro.

## [0.1.3] - 2026-10-26

### Corrigido

- Observação por item de resíduo não era capturada. O domínio ganhou
  `Residuo.observacao`; o SINIR/SIGOR leem `marObservacao` e o MTR-PE lê
  `manifestoItemObservacao`/`observacao`.
- `OBSERVACOES_RESIDUOS` do `mtrs.csv` agora inclui essas observações, além das
  descrições. Exemplo real (`MTR 251033145388`): `CX 387` e `CX 574`.
- `residuos.csv` ganhou a coluna `observacao`; o JSON detalhado passou a expor
  `observacao` em cada resíduo.

### Compatibilidade

- O campo é aditivo no domínio/JSONL; consumidores antigos continuam lendo os
  campos existentes.
- O cabeçalho do `residuos.csv` mudou para incluir `observacao`.

## [0.1.2] - 2026-10-26

### Adicionado

- `openmtr dump` consulta os quatro papéis do SINIR (`gerador`,
  `transportador`, `destinador`, `armazenador`); antes o dump trazia apenas os
  manifestos em que a conta era geradora.
- Schema rico e estável para `mtrs.csv`: `OTR`, emissão/recebimento,
  quantidades recebida/esperada, status, observações, CNPJ/nomes de
  gerador/destinador, motorista, placa, classe de resíduo, JSON detalhado dos
  resíduos e sistema de origem.
- Colunas `classe_residuo` e `observacoes_residuos` no `mtrs.jsonl`.
- Tradução de `classe_residuo` pela tabela `listar_classes` da própria conta
  (SINIR e MTR-PE têm códigos diferentes; ex.: SINIR `43` = `CLASSE II A`).
- Testes de contrato para o schema do CSV, normalização de classes/observações
  e JSON detalhado dos resíduos.
- Wiki `docs/wiki/13-cli-dump.md` com o contrato completo do dump.
- Script `scripts/release-notes.sh`: extrai a seção da versão do CHANGELOG para
  as notas do GitHub Release.
- Release 0.1.2 publicada pela pipeline a partir da tag `v0.1.2`.

### Alterado

- `residuos_json` do JSONL agora carrega todos os campos do resíduo (inclusive
  `null`), em vez de apenas o objeto serializado do domínio.
- `mtrs.csv` passou a ser o contrato de trabalho manual do dump; a coluna
  `OTR` é `-` no dump bruto porque o vínculo MTR↔OTR é feito a jusante.
- `openmtr dump` pagina o SINIR em blocos de 500 manifestos por papel.
- Dependências não usadas do crate `open-mtr-cli` removidas (`futures`,
  `thiserror`, `uuid`).
- Metadados do binding Node (`napi/package.json` e lock) e o status em
  `wiki/plan/cli/README.md` alinhados à versão 0.1.2.

### Corrigido

- Clippy `excessive_nesting` na normalização das observações de resíduos.
- Mapa de classes fixo (1/2/3) substituído pela tabela da conta; antes o SINIR
  classificava `2` como `CLASSE II A` quando o correto é `OUTROS`.
- Falso positivo do `typos` em texto de configuração em português.
- `cargo machete` limpo no workspace.

### Compatibilidade

- Consumidores do `mtrs.jsonl` continuam recebendo os campos antigos. Os novos
  campos são aditivos e `residuos_json` permanece uma string JSON.
- O cabeçalho do `mtrs.csv` mudou para o contrato documentado em
  `docs/wiki/13-cli-dump.md`; scripts que liam colunas por posição precisam ser
  ajustados.
- `openmtr --version` e `openmtr self version` reportam `0.1.2`.

## [0.1.1] - 2026-10-26

### Corrigido

- Dump do SINIR passou a consultar transportador, destinador e armazenador
  além do gerador.

## [0.1.0] - 2026-10-22

### Adicionado

- Primeira release pública do core, CLI, bindings e binários multiplataforma.
