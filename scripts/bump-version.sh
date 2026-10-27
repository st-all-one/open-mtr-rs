#!/usr/bin/env bash
# bump-version — atualiza a versão do openMTR/CLI em todos os pontos do repositório.
#
# Uso:
#   ./scripts/bump-version.sh v0.2.1        # aceita vX.Y.Z ou X.Y.Z
#   make update-version VERSION=v0.2.1
#
# Pontos atualizados (o portão da release exige tag == Cargo.toml):
#   - Cargo.toml  ([workspace.package] version)
#   - Cargo.lock  (crates open-mtr-*)
#   - install.sh  (exemplo `OPENMTR_VERSION=vX.Y.Z`)
#
# Nunca usa `rm` (reusa com `mv`); recusa versão igual ou inválida.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

raw="${1:-}"
if [ -z "$raw" ]; then
    echo "uso: $0 vX.Y.Z" >&2
    exit 2
fi
new="${raw#v}"
if ! printf '%s' "$new" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo "versão inválida: $raw (use X.Y.Z ou vX.Y.Z)" >&2
    exit 2
fi

old="$(grep -m1 -E '^version = "[0-9]' Cargo.toml | cut -d'"' -f2)"
if [ "$old" = "$new" ]; then
    echo "já está na versão $new; nada a fazer"
    exit 0
fi
echo "atualizando versão: $old -> $new"

# 1. Cargo.toml — só a linha de versão do workspace.
sed -E "s|^version = \"$old\"$|version = \"$new\"|" Cargo.toml >Cargo.toml.bump.tmp
mv Cargo.toml.bump.tmp Cargo.toml

# 2. Cargo.lock — todos os crates do workspace open-mtr-*.
awk -v old="$old" -v new="$new" '
    /^name = "open-mtr-/ { name=$0; print; in_ws=1; next }
    in_ws && /^version = / {
        if ($0 == "version = \"" old "\"") print "version = \"" new "\""
        else print
        in_ws=0
        next
    }
    { print }
' Cargo.lock >Cargo.lock.bump.tmp
mv Cargo.lock.bump.tmp Cargo.lock

# 3. Exemplo de instalação.
if [ -f install.sh ]; then
    sed -e "s|OPENMTR_VERSION=v$old|OPENMTR_VERSION=v$new|g" \
        -e "s|VERSION=v$old|VERSION=v$new|g" install.sh >install.sh.bump.tmp
    mv install.sh.bump.tmp install.sh
    chmod +x install.sh
fi

# 4. Metadados do binding Node (package.json + lock).
if [ -f napi/package.json ]; then
    sed -E "s/\"version\": \"$old\"/\"version\": \"$new\"/" napi/package.json >napi/package.json.bump.tmp
    mv napi/package.json.bump.tmp napi/package.json
fi
if [ -f napi/package-lock.json ]; then
    awk -v old="$old" -v new="$new" '
        NR <= 12 && $0 ~ "\"version\": \"" old "\"" {
            sub("\"version\": \"" old "\"", "\"version\": \"" new "\"")
        }
        { print }
    ' napi/package-lock.json >napi/package-lock.json.bump.tmp
    mv napi/package-lock.json.bump.tmp napi/package-lock.json
fi

echo "pronto. Próximos passos:"
echo "  git add -A && git commit -S -m \"chore(release): $new\""
echo "  git tag -a v$new -m \"openmtr v$new\""
echo "  git push origin main && git push origin v$new"