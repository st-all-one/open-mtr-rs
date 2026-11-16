#!/usr/bin/env bash
# openmtr installer — estilo `curl | bash` (release) ou `./install.sh --from-source`.
#
# Modo release: baixa o binário pré-compilado do GitHub Release (latest por padrão,
# ou OPENMTR_VERSION=vX.Y.Z), verifica o SHA-256 contra `sha256sums.txt` e instala
# em ~/.local/bin. Modo source: compila `open-mtr-cli` com Cargo e instala o binário
# otimizado. O binário anterior é movido para um lixo recuperável no cache.
#
# Uso:
#   curl --proto '=https' --tlsv1.2 -fsSL \
#     https://raw.githubusercontent.com/st-all-one/open-mtr-rs/main/install.sh | bash
#   curl ... | OPENMTR_VERSION=v0.2.0 bash
#   curl ... | OPENMTR_INSTALL_DIR="$HOME/bin" bash
#   ./install.sh --from-source
#   ./install.sh --uninstall
#
# Variáveis:
#   OPENMTR_VERSION      tag a instalar (default: latest)
#   OPENMTR_INSTALL_DIR  diretório de destino (default: ~/.local/bin)
#   OPENMTR_TARGET       sobrescreve o target triple detectado
#   OPENMTR_REPO         owner/repo do GitHub
#   OPENMTR_BASE_URL     base URL (default: https://github.com; OPENMTR_RELEASES_URL também)
#   OPENMTR_CACHE_DIR    raiz de cache/lixo (default: ${XDG_CACHE_HOME:-~/.cache}/openmtr)
#   OPENMTR_NO_PATH=1    não edita rc files/PATH
set -euo pipefail

if [ -z "${BASH_VERSION:-}" ]; then
    printf '%s\n' "Este instalador usa recursos do bash. Rode: curl ... | bash" >&2
    exit 1
fi

REPO="${OPENMTR_REPO:-st-all-one/open-mtr-rs}"
BASE_URL_ROOT="${OPENMTR_BASE_URL:-${OPENMTR_RELEASES_URL:-https://github.com}}"
CACHE_ROOT="${OPENMTR_CACHE_DIR:-${XDG_CACHE_HOME:-${HOME}/.cache}/openmtr}"
INSTALL_DIR="${OPENMTR_INSTALL_DIR:-${INSTALL_DIR:-${HOME}/.local/bin}}"
VERSION="${OPENMTR_VERSION:-${VERSION:-latest}}"
FROM_SOURCE=0
DO_UNINSTALL=0
FORCE=0

# HTTPS obrigatório por padrão (evita downgrade de TLS no `curl | bash`). Uma
# base `http://` explícita (servidor de teste local) libera.
CURL_PROTO=""
case "$BASE_URL_ROOT" in
    https://*) CURL_PROTO="--proto =https --proto-redir =https --tlsv1.2" ;;
esac

info() { printf '\033[34m==>\033[0m %s\n' "$*"; }
ok()   { printf '\033[32m  ✓\033[0m %s\n' "$*"; }
warn() { printf '\033[33m  !\033[0m %s\n' "$*" >&2; }
err()  { printf '\033[31m  ✗\033[0m %s\n' "$*" >&2; exit 1; }

require() {
    command -v "$1" >/dev/null 2>&1 || err "Ferramenta necessária não encontrada: $1"
}

tilde() { printf '%s' "${1/#$HOME/\~}"; }

trash() {
    local src="$1"
    [ -e "$src" ] || [ -L "$src" ] || return 0
    local dest="${CACHE_ROOT}/trash/$(basename "$src").$(date +%Y%m%d%H%M%S).$$"
    mkdir -p "${CACHE_ROOT}/trash"
    mv "$src" "$dest"
}

usage() {
    cat <<'EOF'
openmtr installer

Uso: install.sh [--from-source] [--install-dir DIR] [--version TAG] [--uninstall]

  --from-source, -s   compila com cargo (requer o repositório)
  --install-dir DIR   destino (default: ~/.local/bin)
  --prefix DIR        atalho para --install-dir DIR/bin
  --version TAG       tag do release (default: latest; aceita vX.Y.Z ou X.Y.Z)
  --target TRIPLE     sobrescreve o target detectado
  --no-path           não edita os rc files
  --uninstall         move o binário instalado para o lixo recuperável
  --force             reinstala mesmo se a versão já estiver instalada
  -h, --help          mostra esta ajuda

Variáveis: OPENMTR_VERSION, OPENMTR_INSTALL_DIR, OPENMTR_TARGET, OPENMTR_REPO,
           OPENMTR_BASE_URL, OPENMTR_RELEASES_URL, OPENMTR_CACHE_DIR, OPENMTR_NO_PATH.
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --from-source|-s) FROM_SOURCE=1 ;;
        --uninstall)      DO_UNINSTALL=1 ;;
        --install-dir)    [ $# -ge 2 ] || err "--install-dir exige um valor"; INSTALL_DIR="$2"; shift ;;
        --install-dir=*)  INSTALL_DIR="${1#*=}" ;;
        --prefix)         [ $# -ge 2 ] || err "--prefix exige um valor"; INSTALL_DIR="$2/bin"; shift ;;
        --prefix=*)       INSTALL_DIR="${1#*=}/bin" ;;
        --version)        [ $# -ge 2 ] || err "--version exige um valor"; VERSION="$2"; shift ;;
        --version=*)      VERSION="${1#*=}" ;;
        --target)         [ $# -ge 2 ] || err "--target exige um valor"; OPENMTR_TARGET="$2"; shift ;;
        --target=*)       OPENMTR_TARGET="${1#*=}" ;;
        --no-path)        OPENMTR_NO_PATH=1 ;;
        --force)          FORCE=1 ;;
        -h|--help)        usage; exit 0 ;;
        *) err "Argumento desconhecido: $1 (use --help)" ;;
    esac
    shift
done

# Normaliza a tag para o padrão dos assets (`vX.Y.Z`).
if [ "$VERSION" != "latest" ]; then
    VERSION="v${VERSION#v}"
fi

SCRIPT_DIR="$(pwd)"
if [ -n "${BASH_SOURCE[0]:-}" ] && [ -f "${BASH_SOURCE[0]}" ]; then
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    if [ "$FROM_SOURCE" -eq 0 ] && [ "$DO_UNINSTALL" -eq 0 ] &&
        [ -f "${SCRIPT_DIR}/Cargo.toml" ] && [ -d "${SCRIPT_DIR}/cli" ]; then
        FROM_SOURCE=1
        info "Repositório detectado; instalando a partir do source."
    fi
fi

detect_target() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux) os="linux" ;;
        Darwin) os="darwin" ;;
        MINGW*|MSYS*|CYGWIN*) os="windows" ;;
        *) err "Sistema operacional não suportado: $os" ;;
    esac

    case "$arch" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *) err "Arquitetura não suportada: $arch" ;;
    esac

    case "$os-$arch" in
        linux-x86_64) echo "x86_64-unknown-linux-musl" ;;
        linux-aarch64) echo "aarch64-unknown-linux-musl" ;;
        darwin-aarch64) echo "aarch64-apple-darwin" ;;
        darwin-x86_64) echo "x86_64-apple-darwin" ;;
        windows-x86_64) echo "x86_64-pc-windows-msvc" ;;
        windows-aarch64) echo "aarch64-pc-windows-msvc" ;;
        *) err "Sem binário publicado para $os/$arch" ;;
    esac
}

resolve_version() {
    if [ "$VERSION" != "latest" ]; then
        echo "$VERSION"
        return
    fi
    local effective tag
    # `$CURL_PROTO` é intencionalmente não-entre-aspas (lista de flags).
    # shellcheck disable=SC2086
    effective="$(curl -fsSL $CURL_PROTO -I -o /dev/null -w '%{url_effective}' \
        "${BASE_URL_ROOT}/${REPO}/releases/latest")" ||
        err "Falha ao resolver a versão latest em ${BASE_URL_ROOT}/${REPO}"
    tag="${effective##*/tag/}"
    case "$tag" in
        v[0-9]*|[0-9]*) echo "v${tag#v}" ;;
        *) err "Não foi possível resolver a versão latest (release não encontrado em ${BASE_URL_ROOT}/${REPO})" ;;
    esac
}

current_version() {
    local bin="${INSTALL_DIR}/openmtr"
    case "$(uname -s)" in
        MINGW*|MSYS*|CYGWIN*) bin="${bin}.exe" ;;
    esac
    [ -x "$bin" ] || return 0
    "$bin" --version 2>/dev/null | awk '{print $NF}'
}

# Comparação semver simples (X.Y.Z); retorna 0 quando $1 > $2.
version_gt() {
    case "$1" in
        [0-9]*.[0-9]*.[0-9]*) : ;;
        *) return 1 ;;
    esac
    case "$2" in
        [0-9]*.[0-9]*.[0-9]*) : ;;
        *) return 1 ;;
    esac
    [ "$1" = "$2" ] && return 1
    local IFS=. i
    # shellcheck disable=SC2206
    local a=($1) b=($2)
    for i in 0 1 2; do
        [ "${a[$i]:-0}" -gt "${b[$i]:-0}" ] && return 0
        [ "${a[$i]:-0}" -lt "${b[$i]:-0}" ] && return 1
    done
    return 1
}

check_upgrade() {
    local installed
    installed="$(current_version)"
    [ -n "$installed" ] || return 0

    if [ "$installed" = "${VERSION#v}" ] && [ "$FORCE" -eq 0 ]; then
        ok "openmtr ${VERSION} já instalado em $(tilde "$INSTALL_DIR") — nada a fazer."
        exit 0
    fi
    if version_gt "${VERSION#v}" "$installed"; then
        info "Atualizando openmtr v${installed} → ${VERSION}"
    else
        warn "Versão mais nova instalada (v${installed}); instalando ${VERSION} (downgrade)."
    fi
}

build_from_source() {
    require cargo
    [ -f "${SCRIPT_DIR}/Cargo.toml" ] ||
        err "--from-source exige rodar de dentro do repositório (Cargo.toml não encontrado em ${SCRIPT_DIR})"
    [ -d "${SCRIPT_DIR}/cli" ] || err "Crate cli não encontrado em ${SCRIPT_DIR}"
    info "Compilando release a partir do source..."
    ( cd "$SCRIPT_DIR" && cargo build --release --locked -p open-mtr-cli )
    SRC_DIR="${SCRIPT_DIR}/target/release"
}

download_release() {
    local target="$1"
    local ext="tar.gz"
    local asset base_url staging
    case "$target" in
        *-pc-windows-*) ext="zip" ;;
    esac
    require curl
    if [ "$ext" = "tar.gz" ]; then
        require tar
    else
        require unzip
    fi

    asset="openmtr-${VERSION}-${target}.${ext}"
    base_url="${BASE_URL_ROOT}/${REPO}/releases/download/${VERSION}"

    staging="${CACHE_ROOT}/staging"
    mkdir -p "$staging"
    trash "${staging}/out"
    mkdir -p "${staging}/out"

    info "Baixando ${asset}..."
    # shellcheck disable=SC2086
    curl -fsSL $CURL_PROTO -o "${staging}/${asset}" "${base_url}/${asset}" ||
        err "Falha no download: ${base_url}/${asset}"

    info "Verificando checksum SHA-256..."
    if ! command -v sha256sum >/dev/null 2>&1 && ! command -v shasum >/dev/null 2>&1; then
        err "Nenhum verificador SHA-256 encontrado (instale coreutils ou perl-Digest-SHA)"
    fi
    # shellcheck disable=SC2086
    curl -fsSL $CURL_PROTO -o "${staging}/sha256sums.txt" "${base_url}/sha256sums.txt" ||
        err "Falha no download do checksum: ${base_url}/sha256sums.txt"
    (
        cd "$staging"
        grep -F "  ${asset}" sha256sums.txt > "checksum.one" ||
            err "Checksum para ${asset} não encontrado em sha256sums.txt"
        sha256sum -c "checksum.one" >/dev/null 2>&1 ||
            shasum -a 256 -c "checksum.one" >/dev/null 2>&1 ||
            err "Verificação de checksum falhou. Abortando por segurança."
    )
    ok "Checksum OK"

    info "Extraindo binário..."
    if [ "$ext" = "zip" ]; then
        unzip -o -q "${staging}/${asset}" -d "${staging}/out"
    else
        tar -xzf "${staging}/${asset}" -C "${staging}/out"
    fi
    SRC_DIR="${staging}/out"
}

install_binaries() {
    mkdir -p "$INSTALL_DIR" 2>/dev/null ||
        err "Não foi possível criar $(tilde "$INSTALL_DIR") (use sudo ou outro --install-dir)"
    [ -w "$INSTALL_DIR" ] ||
        err "$(tilde "$INSTALL_DIR") não é gravável (use sudo ou outro --install-dir)"

    local src="${SRC_DIR}/openmtr" dest="${INSTALL_DIR}/openmtr"
    case "$(uname -s)" in
        MINGW*|MSYS*|CYGWIN*) src="${src}.exe"; dest="${dest}.exe" ;;
    esac
    [ -f "$src" ] || err "Binário não encontrado no pacote: ${src}"

    trash "$dest"
    if command -v install >/dev/null 2>&1; then
        install -m 0755 "$src" "$dest"
    else
        cp "$src" "$dest"
        chmod 0755 "$dest"
    fi
    ok "Instalado: $(tilde "$dest")"
}

verify_binaries() {
    local bin="${INSTALL_DIR}/openmtr" version
    case "$(uname -s)" in
        MINGW*|MSYS*|CYGWIN*) bin="${bin}.exe" ;;
    esac
    [ -x "$bin" ] || err "Binário não instalado: $(tilde "$bin")"
    version="$("$bin" --version 2>/dev/null | head -n1)" ||
        err "O binário instalado não executa ($(tilde "$bin"))"
    ok "Binário responde: ${version}"
}

add_path_line() {
    local file="$1"
    local line="export PATH=\"${INSTALL_DIR}:\${PATH}\""
    local marker="# --- openmtr path ---"

    grep -qxF "$line" "$file" 2>/dev/null && return 0
    grep -qxF "$marker" "$file" 2>/dev/null && return 0

    if [ -s "$file" ] && [ "$(tail -c1 "$file" | wc -l)" -eq 0 ]; then
        echo "" >> "$file"
    fi
    {
        echo "$marker"
        echo "$line"
    } >> "$file"
    ok "PATH adicionado a $(tilde "$file")"
}

setup_path() {
    [ "${OPENMTR_NO_PATH:-0}" = "1" ] && return 0
    info "Verificando PATH..."
    if printf '%s' "$PATH" | tr ':' '\n' | grep -qxF "$INSTALL_DIR"; then
        ok "$(tilde "$INSTALL_DIR") já está no PATH"
        return 0
    fi

    local touched=0 file
    for file in \
        "${HOME}/.profile" \
        "${HOME}/.bashrc" \
        "${HOME}/.bash_profile" \
        "${HOME}/.zprofile" \
        "${HOME}/.zshrc" \
        "${ZDOTDIR:-${HOME}}/.zshrc"; do
        [ -f "$file" ] || continue
        touched=1
        add_path_line "$file"
    done

    if [ "$touched" -eq 0 ]; then
        : > "${HOME}/.profile"
        add_path_line "${HOME}/.profile"
    fi

    warn "Reinicie o shell ou rode: export PATH=\"${INSTALL_DIR}:\$PATH\""
}

uninstall() {
    local found=0 bin="${INSTALL_DIR}/openmtr"
    for bin in "${INSTALL_DIR}/openmtr" "${INSTALL_DIR}/openmtr.exe"; do
        if [ -e "$bin" ] || [ -L "$bin" ]; then
            found=1
            info "Invalidando $(tilde "$bin") (movido para o lixo)..."
            trash "$bin"
        fi
    done
    [ "$found" -eq 1 ] || warn "Nenhum openmtr encontrado em $(tilde "$INSTALL_DIR")"
    ok "Lixo recuperável em $(tilde "${CACHE_ROOT}/trash")"
}

if [ "$DO_UNINSTALL" -eq 1 ]; then
    uninstall
    exit 0
fi

TARGET="${OPENMTR_TARGET:-$(detect_target)}"
if [ "$FROM_SOURCE" -eq 0 ]; then
    require curl
    info "Detectado: ${TARGET}"
    VERSION="$(resolve_version)"
    info "Instalando openmtr ${VERSION} (${TARGET}) em $(tilde "$INSTALL_DIR")..."
    check_upgrade
    download_release "$TARGET"
else
    info "Instalando openmtr a partir do source em $(tilde "$INSTALL_DIR")..."
    build_from_source
fi

install_binaries
verify_binaries
setup_path

echo ""
info "Pronto! openmtr instalado em $(tilde "$INSTALL_DIR")."
echo "  openmtr --version"
echo "  export SINIR_TOKEN=seu-refresh-token"
echo "  openmtr --login '{\"system\":\"sinir\",\"token_env\":\"SINIR_TOKEN\"}' mtr consultar 2114014590"
echo ""
info "Docs: https://github.com/${REPO}/blob/main/docs/wiki/14-cli.md"
