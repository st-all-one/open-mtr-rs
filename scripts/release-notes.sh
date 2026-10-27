#!/usr/bin/env bash
# Extrai a seção de uma versão do CHANGELOG para as notas do GitHub Release.
#
# Uso:
#   ./scripts/release-notes.sh 0.1.2 > release-notes.md
#   ./scripts/release-notes.sh v0.1.2
#
# A seção começa em "## [X.Y.Z]" (ou "## X.Y.Z") e termina no próximo "## ".
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
version="${1:-}"
if [ -z "$version" ]; then
    echo "uso: $0 X.Y.Z" >&2
    exit 2
fi
version="${version#v}"

if ! printf '%s' "$version" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo "versão inválida: $version (use X.Y.Z ou vX.Y.Z)" >&2
    exit 2
fi

notes="$(
    awk -v version="$version" '
        $0 ~ "^## \\[" version "\\]" { found = 1; next }
        $0 ~ "^## " version "$" { found = 1; next }
        found && /^## / { exit }
        found { print }
    ' "$root/CHANGELOG.md"
)"

if ! printf '%s' "$notes" | grep -q '[^[:space:]]'; then
    echo "seção $version ausente em $root/CHANGELOG.md" >&2
    exit 1
fi
printf '%s\n' "$notes"
