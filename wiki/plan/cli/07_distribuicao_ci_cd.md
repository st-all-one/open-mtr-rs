# 07 — Workspace, empacotamento, release e CI/CD

> **Status:** etapa 7 (distribuição). **Depende de:** arquitetura e qualidade.
> **Alimenta:** atualização/operação.
>
> Objetivo: qualquer usuário instala o `openmtr` pré-compilado, sem Rust/Deno, com checksum,
> e atualiza depois com `openmtr self update`.

## 1. Alterações no workspace

### 1.1 `Cargo.toml`

```toml
[workspace]
members = ["core", "cli", "napi", "uniffi", "wasm"]
default-members = ["core", "cli"]
```

Adicionar ao `[workspace.dependencies]` (somente o necessário):

```toml
clap = { version = "4", features = ["derive"] }
clap_complete = "4"
toml = "0.9"
toml_edit = "0.23"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
csv = "1"
sha2 = "0.10"        # manifesto de PDFs e/ou checksum local
# dev
assert_cmd = "2"
predicates = "3"
tempfile = "3"
```

> Versões exatas são resolvidas no momento da implementação e travadas no `Cargo.lock`.
> `sha2` já é usado no `_KNUDGE`; no openMTR só entra se o manifesto de PDFs exigir hash.

### 1.2 `cli/Cargo.toml`

- `[[bin]] name = "openmtr"`.
- `open-mtr-core` com features default (`native-http`).
- `[lints] workspace = true`.
- Sem `reqwest` direto, sem `wasm-bindgen`, sem `napi`, sem `uniffi`.
- `publish = false` herdado do workspace.

### 1.3 Ajustes de scripts

`scripts/check_file_length.sh`:

- incluir `cli` no `find`;
- manter o limite transitório 1500 para legado e aplicar 300 ao diretório `cli` (regra nova):
  - opção A: `find core napi uniffi wasm` com 1500 + `find cli` com 300;
  - opção B: reduzir o limite global apenas quando core for dividido (não neste épico).
- **Decisão:** A. O CLI nasce com 300; o legado não regride.

`Makefile`:

```make
.PHONY: cli dist release-check

## Build do CLI (debug).
cli:
	$(CARGO) build -p open-mtr-cli

## Empacota a plataforma atual no padrão do release.
dist:
	./scripts/package.sh

## Valida versão/tag e roda o portão antes de publicar.
release-check:
	./scripts/check-release-version.sh
	$(MAKE) check
```

## 2. Empacotamento

Novo `scripts/package.sh` (espelha o do knudge, adaptado):

- compila `open-mtr-cli` em release (`--locked`);
- nome: `openmtr-<versão>-<target>.tar.gz` (Unix) / `.zip` (Windows);
- conteúdo:
  - `openmtr` (ou `openmtr.exe`);
  - `README-CLI.md` curto;
  - `open-mtr-config.example.toml`;
  - `LICENSE-MIT`, `LICENSE-APACHE`;
- gera `sha256sums.txt` no diretório `dist/`;
- falha se o binário não executar `openmtr self version`.

Não empacotar o core como biblioteca; o binário é estático (Linux musl) ou dinâmico de sistema
(macOS/Windows) como no knudge.

## 3. Release workflow

Novo `.github/workflows/release.yml`:

```yaml
on:
  push:
    tags: ["v*"]
  workflow_dispatch:
```

### 3.1 Job `verify`

1. checkout;
2. toolchain 1.99 + rustfmt/clippy;
3. cache;
4. `make check` (fmt, clippy, test, file-length);
5. verificar `tag == versão` do `Cargo.toml` (mesma lógica do knudge);
6. `make doc`;
7. `cargo deny check`/`cargo machete`/`typos`.

### 3.2 Job `build` — matriz

| OS | Target | Archive |
|---|---|---|
| ubuntu-latest | `x86_64-unknown-linux-musl` | tar.gz |
| ubuntu-24.04-arm | `aarch64-unknown-linux-musl` | tar.gz |
| macos-latest | `aarch64-apple-darwin` | tar.gz |
| macos-latest | `x86_64-apple-darwin` | tar.gz |
| windows-latest | `x86_64-pc-windows-msvc` | zip |
| windows-latest | `aarch64-pc-windows-msvc` | zip |

Passos por target:

1. instalar target Rust;
2. dependências musl (Linux) / toolchain MSVC ARM64 (Windows);
3. `cargo build --release --locked -p open-mtr-cli --target <target>`;
4. `scripts/package.sh --target <target> --version <versão>` (ou staging equivalente);
5. upload de artefato.

### 3.3 Job `release`

1. baixar artefatos;
2. gerar `sha256sums.txt` com todos os arquivos;
3. publicar GitHub Release com tar.gz/zip + checksums + `open-mtr-config.example.toml`;
4. release notes geradas;
5. só publica em tag; `workflow_dispatch` em branch apenas compila.

## 4. CI contínuo (`.github/workflows/build.yml`)

Adicionar/ajustar jobs:

| Job | Conteúdo |
|---|---|
| `quality` | `make check` + `make doc` (já existe; passa a incluir CLI) |
| `cli` | `cargo test -p open-mtr-cli` + golden/security |
| `cross-platform` | core + CLI em Linux/macOS/Windows |
| `supply-chain` | deny/machete/typos (+ audit quando instalado) |
| `windows-smoke` | `openmtr self version`, paths, `self completions powershell` |
| `release-check` | em PRs que tocam `Cargo.toml`: validar versão/tag e packaging dry-run |

O CI não publica release; isso fica no workflow de tag.

## 5. `install.sh`

Novo `install.sh` na raiz (alvo `curl | bash`), inspirado no knudge:

### 5.1 Modos

- `./install.sh` (detecta repo e instala do source) ou
- `curl ... | bash` (release) ou
- `./install.sh --from-source`.

### 5.2 Variáveis

| Variável | Default | Uso |
|---|---|---|
| `VERSION` | `latest` | tag a instalar |
| `INSTALL_DIR` | `~/.local/bin` | destino do binário |
| `OPENMTR_TARGET` | detectado | override do target triple |
| `OPENMTR_REPO` | `st-all-one/open-mtr-rs` | repo GitHub |
| `OPENMTR_BASE_URL` | `https://github.com` | base de download (teste local) |
| `OPENMTR_CACHE_DIR` | `${XDG_CACHE_HOME:-~/.cache}/openmtr` | staging/trash |
| `OPENMTR_NO_PATH` | `0` | não editar rc files |
| `OPENMTR_NO_COMPLETIONS` | `0` | não instalar completions |

### 5.3 Fluxo

1. detectar OS/arch → target canônico;
2. resolver `latest` via redirect de releases;
3. baixar `openmtr-<version>-<target>.{tar.gz,zip}` via HTTPS;
4. baixar `sha256sums.txt`;
5. verificar SHA-256 antes de extrair;
6. instalar em `INSTALL_DIR` (`install -m 0755`);
7. mover binário anterior para lixeira recuperável em cache;
8. verificar `openmtr self version` executando;
9. instalar completions (`openmtr self completions`) se disponível;
10. criar `open-mtr-config.toml` inicial **apenas se não existir** (`0600`), a partir do exemplo;
11. imprimir próximos passos.

### 5.4 Segurança

- HTTPS obrigatório por default (`--proto =https`).
- checksum obrigatório; aborta se ausente ou inválido.
- nunca `curl | sh` de script remoto sem checksum.
- nunca sobrescrever config existente.
- `--uninstall` remove binário/completions e preserva config/config de usuário.
- `--from-source` usa `cargo build --release --locked`.

### 5.5 Config inicial

`install.sh` **não** inventa credenciais. Cria template com contas comentadas e instruções:

```toml
config_version = 1

# Preencha ao menos uma conta e rode:
#   openmtr config validate
[[accounts]]
id = "sinir-prod"
system = "sinir"
[accounts.auth]
type = "token"
token_env = "OPENMTR_SINIR_TOKEN"
```

Alternativa: `install.sh` só copia `open-mtr-config.example.toml`; o usuário roda `config init`.

## 6. Versionamento

- Versão única do workspace (`Cargo.toml`), tag `vX.Y.Z`.
- Gate `tag == Cargo.toml` no release.
- `make update-version VERSION=vX.Y.Z` (novo script `scripts/bump-version.sh`) atualiza:
  - `Cargo.toml` (workspace);
  - `Cargo.lock`;
  - `wiki/CHANGELOG.md`/`CHANGELOG`;
  - `install.sh` (se tiver versão default embutida);
  - `README.md`/docs de versão.
- `--version` e `self version` leem `env!("CARGO_PKG_VERSION")` e, se disponível, target/commit
  injetados no build.

## 7. Documentação de release

No PR de release:

- atualizar [`08_atualizacao_operacao.md`](08_atualizacao_operacao.md) se o processo mudar;
- atualizar `README.md` (instalação, exemplos de dump);
- atualizar `wiki/plan/cli/` com status/marco;
- `CHANGELOG` com breaking changes de config/comando;
- publicar exemplos de config sem segredo.

## 8. Critérios de aceite da distribuição

- [ ] `make check` e `make ci` verdes com o novo crate.
- [ ] `make doc` verde.
- [ ] `cargo build --release -p open-mtr-cli --target <target>` verde nos 6 targets.
- [ ] `scripts/package.sh` gera pacote + checksum local.
- [ ] `install.sh` instala a partir de fixture local e valida checksum.
- [ ] `openmtr self version` funciona no binário instalado em cada SO.
- [ ] `openmtr self update --check`/`update` delegam ao script.
- [ ] Release publica `sha256sums.txt` e exemplo de config.
- [ ] Nenhum segredo ou `.env` no pacote.
- [ ] `install.sh` preserva config existente.

## 9. Riscos

| Risco | Mitigação |
|---|---|
| matriz de release quebrar em musl/ARM | runner nativo + cache + smoke `self version` |
| binário não executar por linker | `verify_binaries` no install e no package |
| checksum ausente/inválido | abortar; teste de fixture com checksum errado |
| `install.sh` sobrescrever config | só cria se ausente; teste dedicado |
| dependências novas aumentarem supply-chain | `cargo deny`/`machete`; licenças MIT/Apache |
| release sem gate de qualidade | `verify` roda `make check` antes do build |
