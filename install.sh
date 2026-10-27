#!/bin/sh
# Instala/atualiza o binário openmtr a partir das releases do GitHub.
#
# Uso:
#   curl -fsSL https://raw.githubusercontent.com/st-all-one/open-mtr-rs/main/install.sh | sh
#   OPENMTR_VERSION=v0.1.3 sh install.sh
#   VERSION=v0.1.3 sh install.sh       # alias compatível
#   OPENMTR_INSTALL_DIR="$HOME/bin" sh install.sh
#
# Verifica SHA-256 contra sha256sums.txt da release e instala atomicamente.
# HTTPS é obrigatório no caminho default; URLs `http://` explícitas (teste local) liberam.

set -eu

REPO="${OPENMTR_REPO:-st-all-one/open-mtr-rs}"
BIN="openmtr"
CACHE="${XDG_CACHE_HOME:-$HOME/.cache}/openmtr"
DOWNLOAD_BASE="${OPENMTR_RELEASES_URL:-https://github.com}"
API_BASE="${OPENMTR_API_URL:-https://api.github.com}"
VERSION="${OPENMTR_VERSION:-${VERSION:-}}"

# HTTPS obrigatório por padrão (evita downgrade de TLS no `curl | sh`).
CURL_PROTO=""
case "$DOWNLOAD_BASE" in
    https://*) CURL_PROTO="--proto =https --proto-redir =https --tlsv1.2" ;;
esac

need() {
    command -v "$1" >/dev/null 2>&1 || {
        echo "erro: $1 é obrigatório" >&2
        exit 1
    }
}

need uname
need mkdir
need mv
need rm
need tar
if command -v curl >/dev/null 2>&1; then
    # shellcheck disable=SC2086 # CURL_PROTO é uma lista de flags intencional
    fetch() { curl $CURL_PROTO -fsSL "$1"; }
    # shellcheck disable=SC2086
    download() { curl $CURL_PROTO -fsSL -o "$2" "$1"; }
elif command -v wget >/dev/null 2>&1; then
    fetch() { wget -qO- "$1"; }
    download() { wget -qO "$2" "$1"; }
else
    echo "erro: curl ou wget é obrigatório" >&2
    exit 1
fi

if command -v sha256sum >/dev/null 2>&1; then
    sha256() { sha256sum "$1" | awk '{print $1}'; }
elif command -v shasum >/dev/null 2>&1; then
    sha256() { shasum -a 256 "$1" | awk '{print $1}'; }
else
    echo "erro: sha256sum ou shasum é obrigatório" >&2
    exit 1
fi

os="$(uname -s)"
arch="$(uname -m)"
case "$os" in
    Linux) os_part="unknown-linux-musl" ;;
    Darwin) os_part="apple-darwin" ;;
    *) echo "erro: sistema não suportado: $os" >&2; exit 1 ;;
esac
case "$arch" in
    x86_64|amd64) arch_part="x86_64" ;;
    arm64|aarch64) arch_part="aarch64" ;;
    *) echo "erro: arquitetura não suportada: $arch" >&2; exit 1 ;;
esac
target="$arch_part-$os_part"

tag="$VERSION"
if [ -z "$tag" ]; then
    tag="$(fetch "$API_BASE/repos/$REPO/releases/latest" |
        sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n1)"
fi
[ -n "$tag" ] || { echo "erro: não foi possível resolver a versão" >&2; exit 1; }

asset="$BIN-$tag-$target.tar.gz"
url="$DOWNLOAD_BASE/$REPO/releases/download/$tag/$asset"
sums_url="$DOWNLOAD_BASE/$REPO/releases/download/$tag/sha256sums.txt"

mkdir -p "$CACHE"
tmp="$CACHE/.$asset.$$"
trap 'rm -f "$tmp" "$CACHE/sha256sums.txt"' EXIT INT TERM

echo "openmtr: versão $tag ($target)" >&2
download "$url" "$tmp"
fetch "$sums_url" > "$CACHE/sha256sums.txt"

expected="$(awk -v a="$asset" '$2 == a || $2 == "*"a {print $1}' "$CACHE/sha256sums.txt" | head -n1)"
[ -n "$expected" ] || { echo "erro: checksum ausente para $asset" >&2; exit 1; }
actual="$(sha256 "$tmp")"
if [ "$expected" != "$actual" ]; then
    echo "erro: checksum divergente" >&2
    echo "  esperado: $expected" >&2
    echo "  obtido:   $actual" >&2
    exit 1
fi
echo "openmtr: checksum ok" >&2

extract="$CACHE/extract.$$"
mkdir -p "$extract"
trap 'rm -rf "$tmp" "$extract" "$CACHE/sha256sums.txt"' EXIT INT TERM
tar -xzf "$tmp" -C "$extract"
[ -f "$extract/$BIN" ] || { echo "erro: binário ausente no pacote" >&2; exit 1; }

install_dir="${OPENMTR_INSTALL_DIR:-}"
if [ -z "$install_dir" ]; then
    if [ -w /usr/local/bin ]; then install_dir="/usr/local/bin"; else install_dir="$HOME/.local/bin"; fi
fi
mkdir -p "$install_dir"

dest="$install_dir/$BIN"
if [ -f "$dest" ]; then
    mkdir -p "$CACHE/trash"
    mv "$dest" "$CACHE/trash/$BIN.$(date +%s)" 2>/dev/null || true
fi
staging="$install_dir/.$BIN.new.$$"
cp "$extract/$BIN" "$staging"
chmod 755 "$staging"
mv "$staging" "$dest"

echo "openmtr: instalado em $dest" >&2
"$dest" --version >&2 || { echo "erro: binário não executou" >&2; exit 1; }
echo "openmtr: pronto" >&2