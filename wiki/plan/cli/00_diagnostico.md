# 00 — Diagnóstico e levantamento

> **Status:** etapa 0 (diagnóstico). **Depende de:** nada. **Alimenta:** todas as etapas.
>
> Objetivo: transformar a inspiração do `deno task dump` em requisitos verificáveis para uma CLI
> nativa, multi-conta e determinística.

## 1. Evidência principal: `deno task dump`

O comando mais próximo do produto desejado está em:

- `_OPEN_MTR_API/README.md` §“Dump de MTRs (massive time dump)”;
- `_OPEN_MTR_API/scripts/dump.ts` (~1.100 linhas);
- `_OPEN_MTR_API/lib/{config,wasm,mtr_pe_auth,cache}.ts`;
- `_OPEN_MTR_API/routes/v1/*.ts`;
- `core/src/openmtr/open_mtr_client.rs` (fachada real consumida pelo WASM).

### 1.1 O que o `dump.ts` faz hoje

| Bloco | Comportamento observado | Consequência para o CLI |
|---|---|---|
| Argumentos | `--inicio`, `--fim`, `--base`, `--base-sinir`, `--base-mtr-pe`, `--tamanho`, `--conc`, `--janela`, `--out`, `--pdf`, `--mtr-pe-direct`, `--no-mtr-pe-transportadora` | Precisa de parser tipado (`clap`) + config; flags equivalentes e nomes consistentes |
| Credenciais | env vars fixas: `MTR_PE_*`, `MTR_PE_TRANSPORTADORA_*`, `OPENMTR_SINIR_TOKEN`, `OPENMTR_API_TOKEN` | Precisa de contas nomeadas no TOML, com N contas por sistema |
| Multi-conta | duas contas MTR-PE hardcoded (empresa + transportadora) | Generalizar para qualquer quantidade e qualquer sistema |
| SINIR | consulta paginada em janelas; depois detalha por números em lotes de 25 | Reaproveitar `consultar_mtrs` do core com `detalhado`/paginação quando possível |
| MTR-PE | modo via API publicada ou modo direto (`gettoken` + barcodes + detalhe) | O CLI deve usar o core nativo; modo “direto” só se necessário para diagnóstico |
| Janelas | quebra intervalo em janelas de N dias; MTR-PE usa janela máxima de 30 dias no adapter | Generalizar janela no CLI e respeitar limites por sistema |
| Retry | backoff exponencial e timeout em `api()` e `rawMtrPe()` | Já existe retry no core; CLI deve controlar timeout/concorrência e reportar tentativas |
| Concorrência | workers manuais com `Promise.all`, até 8 no detalhe e 6 no PDF | Limitar via `max_concurrency` do core/config; evitar explosão |
| Normalização | `MtrNormalizada` com partes, resíduos, quantidades agregadas, nome de classe | Mover para módulo de dump do CLI; não alterar domínio sem necessidade |
| CSV/JSONL | `mtrs.jsonl`, `mtrs.csv`, `residuos.csv`, `relatorio.json` | Contrato de arquivos precisa ser estável e golden-tested |
| PDF | download por número (SINIR) ou barcode (MTR-PE), resumível por existência do arquivo | Implementar retomada explícita e nomes determinísticos |
| Relatório | completude por sistema, erros por conta, duração, totais | Estender com `account`/`system`; separar duração/timestamp voláteis |
| Ordenação | `sort` por sistema e número ao final | Replicar e ampliar para `account,system,numero` |
| Config de API | `.env` e API token separado, sem arquivo cumulativo | Substituir por `open-mtr-config.toml` local ao CLI |

### 1.2 Limitações do `dump.ts` que o CLI deve resolver

1. Credenciais em env são globais e não permitem duas contas arbitrárias do mesmo sistema sem
   variáveis inventadas.
2. O script duplica regras que já existem no core (retry, janela MTR-PE, normalização MTR-PE).
3. Não há contrato de `--params`, `--batch`, stdin/heredoc ou envelope JSON de máquina.
4. Não há separação formal de escopos: qualquer execução pode varrer tudo por default.
5. Saídas misturam progresso humano e dados; o contrato stdout/stderr não é explícito.
6. Timestamps e IDs voláteis aparecem no relatório; não há modo determinístico/golden.
7. Atualização/instalação não existe: depende de `deno`, checkout do repositório e `.env`.
8. Não há binário pré-compilado, checksum, release ou rollback.

## 2. Evidência de borda: `_KNUDGE`

O `kd` já resolveu o problema de **CLI para LLM/automação**. O CLI do openMTR deve copiar o
contrato, não a implementação.

| Prática do `kd` | Aplicação no `openmtr` |
|---|---|
| `stdout = dados`, `stderr = logs` | obrigatório em todos os comandos |
| `--json` com `{success, command, data, error, warnings}` | envelope de máquina para comandos não-dump; dump usa JSONL/CSV |
| exit codes estáveis (`2,3,4,5,6,7,...`) | mapear `OpenMTRError::kind` e falhas de config/IO |
| EPIPE → exit 0 | essencial para `openmtr ... | head` |
| `--params '<JSON>'` e `--params -` | padrão para filtros/payloads de domínio |
| conteúdo por stdin/heredoc quando não-TTY | `cat <<JSON | openmtr ... --params -` |
| posicional = conteúdo, id = flag | aqui: payload complexo sempre por `--params`/`--batch`; ids como flags |
| listas repetíveis/com vírgula | `--account a --account b`, `--tag x,y` |
| escopo obrigatório para varreduras | `--account`, `--system` ou `--all-accounts` explícito |
| `--dry-run` em lote | `openmtr batch --dry-run` |
| `self version`, `self completions`, `self update` | mesmo trio no CLI |
| release por tag + matriz nativa + `sha256sums.txt` | reutilizar o desenho no CI do openMTR |
| install.sh com checksum e upgrade idempotente | `install.sh` na raiz do openMTR |

### 2.1 O que **não** copiar

- O codec TOML próprio do `kd`: o CLI do openMTR pode usar uma crate de TOML madura; simplicidade
  aqui é reduzir código de parser, não escrever outro.
- O envelope de conhecimento/notas: o openMTR tem seu próprio envelope `Resposta<T>`.
- O `prime`/`rewind`: não há memória de projeto neste produto.
- `surfaces` gigantes: manter a superfície pequena e orientada a operações MTR.

## 3. Estado atual do `open-mtr-rs`

### 3.1 O que já existe e deve ser reaproveitado

| Capacidade | Onde | Uso pelo CLI |
|---|---|---|
| Fachada de operações | `OpenMtrClient` | todos os comandos `mtr`, `cdf`, `ref`, `health` |
| Autenticação clássica | `OpenMtrClassicAuth` + `AuthParams` | contas MTR-PE/SIGOR por CPF/senha/unidade |
| Token de integração | `Config::integration_token` + `OpenMtrClientBuilder` | contas SINIR/SIGOR com token |
| Retry, refresh, rate limit | `MtrHttpClient` | não reimplementar no CLI |
| Clock/IdGenerator | `MtrCtx`, `time_util` | saída determinística e testes |
| Erros tipados | `OpenMTRError::kind` | exit codes e envelope |
| Domínio completo | `Mtr`, `Residuo`, `FiltroMtr`, `GerarCdf`, `DownloadItem` etc. | serialização de `--params` e dump |
| Transporte nativo | `native-http` | CLI sempre compila com essa feature |

### 3.2 Gaps atuais relevantes

| Gap | Impacto | Etapa que resolve |
|---|---|---|
| Nenhum crate/binário CLI | não há ponto de entrada | 02 |
| `Config` suporta uma credencial por cliente | não representa N contas no mesmo processo | 01, 02 |
| `Config` não tem noção de `account id`/label | saída e logs não distinguem contas | 01 |
| Autenticação clássica é separada da criação do cliente | CLI precisa orquestrar `obter_token` → `builder().token()` | 02 |
| Não há parser/validador de TOML no workspace | config precisa de crate e schema | 01, 02 |
| Dump só existe como script Deno | não há binário nativo | 04 |
| `Periodo` com `{"dias":N}` usa relógio global | golden test exige data explícita ou clock fixo | 05 |
| `OpenMTRError.instance` é UUID v7 ao criar | envelopes podem variar se não forem normalizados | 05 |
| Ordem de listas vem da API | dump/envelope pode variar entre execuções | 05 |
| Sem release/CI de binário | instalação exige cargo/Deno | 07 |
| Sem `self update`/install.sh | atualização manual | 08 |
| Orçamento de arquivo 1500 linhas é transitório | CLI deve nascer ≤300 linhas/arquivo | 02, 06 |

### 3.3 Restrições do core que afetam o CLI

1. `OpenMtrClientBuilder::build` exige `system` e `token`; o CLI autentica antes e passa o token.
2. MTR-PE exige CNPJ explícito no fluxo clássico; a config deve validar isso por conta.
3. `System::adapter` só implementa `Sinir`, `MtrPe` e `Sigor`; contas de sistemas não implementados
   devem falhar em `config validate` ou em runtime com mensagem clara.
4. `Config::max_rps` é o teto de concorrência do cliente HTTP; o CLI deve usar isso como default e
   permitir override por flag.
5. O core não expõe uma sessão multi-conta; o CLI deve manter um mapa `account_id -> cliente` lazy.
6. O core já normaliza MTR-PE por barcode e SINIR detalhado; o dump deve evitar duplicar regra de
   adapter.

## 4. Requisitos funcionais derivados

| Id | Requisito | Verificação |
|---|---|---|
| RF-01 | Ler `open-mtr-config.toml` com N contas e N sistemas | teste de config + fixture |
| RF-02 | Permitir duas ou mais contas do mesmo sistema | fixture `mtr-pe-empresa` + `mtr-pe-transportadora` |
| RF-03 | Selecionar conta por id, sistema, tag ou todas | testes de escopo e conflitos |
| RF-04 | Executar operações MTR/CDF/tabelas/health | wiremock por comando |
| RF-05 | Executar dump multi-conta com janelas, PDFs e relatório | golden de dump com fixture |
| RF-06 | Executar lote JSONL com `--batch`/`--dry-run`/`--continue-on-error` | golden de batch |
| RF-07 | Aceitar `--params` inline, arquivo/stdin e heredoc | testes de stdin/TTY |
| RF-08 | Produzir stdout de dados e stderr de logs | testes de captura |
| RF-09 | Redigir segredos e validar permissões | teste de redaction + permissão |
| RF-10 | Emitir envelope de máquina com exit codes estáveis | golden `--json` |
| RF-11 | Ser determinístico com clock/id injetados | golden com `--fixed-time` |
| RF-12 | Instalar e atualizar via binário pré-compilado + checksum | smoke local do install + CI release |

## 5. Requisitos não-funcionais

- **Determinismo:** mesma entrada + mesma config + mesmo clock fixo ⇒ mesmos bytes de saída.
- **Escopo:** nenhuma mutação sem conta única explícita.
- **Segurança:** nenhum segredo em stdout, stderr, relatório ou CSV.
- **Portabilidade:** Linux x86_64/ARM64, macOS x86_64/ARM64, Windows x86_64/ARM64.
- **Desempenho:** dump deve ter paralelismo limitado e memória proporcional ao lote, não ao total.
- **Manutenção:** arquivos ≤300 linhas; módulos pequenos; nenhuma dependência pesada sem necessidade.
- **Observabilidade:** progresso em stderr com conta, janela, página, lote e tentativa.

## 6. Riscos iniciais

| Risco | Probabilidade | Impacto | Mitigação |
|---|---|---|---|
| Config virar “cofre de segredos” versionado | alta | alto | `*_env` preferencial, aviso para inline, checagem de permissão, redaction |
| Duplicar lógica do `dump.ts` no CLI | média | alto | reutilizar core; módulo de dump só normaliza/orquestra |
| Vazamento de conta entre operações | média | alto | resolver conta por operação, cliente lazy chaveado por `account.id`, sem default global |
| Saída variar por UUID/timestamp | alta | médio | `--fixed-time`, clock/id injetáveis, normalização de envelope |
| API instável/rate limit | alta | médio | retry do core, concorrência limitada, relatório de erros por lote |
| Release matrix quebrar por feature nativa | média | médio | CLI sempre `native-http`; testes de `cargo check` por target |
| Superfície crescer demais | média | médio | verbos amplos + subcomandos; sem aliases legados na v1 |

## 7. Saída desta etapa

- Lista de operações e limitações mapeada.
- Requisitos RF-01..RF-12 e RNFs definidos.
- Riscos iniciais registrados.
- Próxima etapa: fechar o contrato de `open-mtr-config.toml` em
  [`01_contrato_config.md`](01_contrato_config.md).
