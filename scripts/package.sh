#!/usr/bin/env bash
# Empacota `openmtr` da plataforma atual em `dist/`, no mesmo padrão de nome
# do GitHub Release: `openmtr-<tag>-<target>.{tar.gz,zip}`.
#
# Uso:
#   ./scripts/package.sh              # compila release (otimizado) e empacota
#   ./scripts/package.sh --no-build   # empacota o que já existe em target/release
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

build=1
if [ "${1:-}" = "--no-build" ]; then
    build=0
fi

version="$(grep -m1 -E '^version = "[0-9]' Cargo.toml | cut -d'"' -f2)"
tag="v${version}"
host="$(rustc -vV | sed -n 's/^host: //p')"

if [ "$build" -eq 1 ]; then
    cargo build --release --locked -p open-mtr-cli
fi

name="openmtr-${tag}-${host}"
staging="dist/${name}"
mkdir -p "$staging"

case "$host" in
    *-pc-windows-*)
        cp target/release/openmtr.exe "$staging/openmtr.exe"
        if command -v 7z >/dev/null 2>&1; then
            (cd "$staging" && 7z a -tzip "../${name}.zip" ./* >/dev/null)
        elif command -v zip >/dev/null 2>&1; then
            (cd "$staging" && zip -q "../${name}.zip" ./*)
        else
            printf 'ERRO: preciso de 7z ou zip para empacotar no Windows\n' >&2
            exit 1
        fi
        printf 'OK: dist/%s.zip\n' "$name"
        ;;
    *)
        cp target/release/openmtr "$staging/openmtr"
        chmod +x "$staging/openmtr"
        (cd "$staging" && tar -czf "../${name}.tar.gz" openmtr)
        printf 'OK: dist/%s.tar.gz\n' "$name"
        ;;
esac