#!/usr/bin/env bash
# Gera a referência da CLI a partir do `--help` real do binário.
#
# Uso:
#   ./scripts/gen-cli-docs.sh                  # atualiza docs/wiki/14-cli-reference.md
#   ./scripts/gen-cli-docs.sh --check          # falha se a referência estiver desatualizada
#   ./scripts/gen-cli-docs.sh --out dist/cli.md
#   OPENMTR_BIN=target/release/openmtr ./scripts/gen-cli-docs.sh
#
# `make cli-docs` atualiza; `make check` usa `--check`. A saída é determinística
# (sem data, host ou caminho absoluto) para caber em revisão e gate de CI.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="$root/docs/wiki/14-cli-reference.md"
check=0

usage() {
    cat <<'EOF'
Uso: gen-cli-docs.sh [--check] [--out ARQUIVO]

  --check       compara a referência gerada com o arquivo e falha se diferir
  --out ARQUIVO grava a referência no caminho indicado (default: docs/wiki/14-cli-reference.md)
  -h, --help    mostra esta ajuda

Variável: OPENMTR_BIN (binário a inspecionar; default: target/debug/openmtr)
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --check) check=1 ;;
        --out)
            [ $# -ge 2 ] || { echo "erro: --out exige um valor" >&2; exit 2; }
            out="$2"
            shift
            ;;
        --out=*) out="${1#*=}" ;;
        -h|--help) usage; exit 0 ;;
        *) echo "erro: argumento desconhecido: $1 (use --help)" >&2; exit 2 ;;
    esac
    shift
done

bin="${OPENMTR_BIN:-}"
if [ -z "$bin" ]; then
    if [ -x "$root/target/debug/openmtr" ]; then
        bin="$root/target/debug/openmtr"
    elif [ -x "$root/target/debug/openmtr.exe" ]; then
        bin="$root/target/debug/openmtr.exe"
    else
        (cd "$root" && cargo build --locked -p open-mtr-cli)
        if [ -x "$root/target/debug/openmtr" ]; then
            bin="$root/target/debug/openmtr"
        else
            bin="$root/target/debug/openmtr.exe"
        fi
    fi
fi
[ -x "$bin" ] || { echo "erro: binário não encontrado/executável: $bin" >&2; exit 1; }

tmp="$(mktemp "${TMPDIR:-/tmp}/openmtr-cli-docs.XXXXXX")"
trap 'rm -f "$tmp"' EXIT

# A lista é explícita para manter a ordem do guia e evitar depender de parsing
# frágil do texto do clap. Ao adicionar um comando, acrescente-o aqui.
commands=(
    "account"
    "account list"
    "account test"
    "config"
    "config validate"
    "config migrate"
    "mtr"
    "mtr consultar"
    "mtr criar"
    "mtr receber"
    "mtr cancelar"
    "mtr modificar"
    "mtr aceitar"
    "mtr download"
    "cdf"
    "cdf consultar"
    "cdf gerar"
    "cdf download"
    "ref"
    "ref listar"
    "ref por-residuo"
    "ref por-estado-fisico"
    "health"
    "dump"
    "batch"
    "self"
    "self version"
    "self update"
)

version="$(COLUMNS=100 NO_COLOR=1 "$bin" --version)"
{
    printf '# Referência da CLI `openmtr`\n\n'
    printf '> Gerado automaticamente por `scripts/gen-cli-docs.sh` a partir de `%s`.\n' "$version"
    printf '> Não edite à mão: rode `make cli-docs` e versione o resultado.\n\n'
    printf '## `openmtr --help`\n\n```text\n'
    COLUMNS=100 NO_COLOR=1 "$bin" --help
    printf '```\n'
    for cmd in "${commands[@]}"; do
        read -r -a argv <<<"$cmd"
        printf '\n## `openmtr %s`\n\n```text\n' "$cmd"
        COLUMNS=100 NO_COLOR=1 "$bin" "${argv[@]}" --help
        printf '```\n'
    done
} >"$tmp"

if [ "$check" -eq 1 ]; then
    if diff -u "$out" "$tmp"; then
        echo "cli-docs: ok ($out)"
    else
        echo "cli-docs: desatualizado; rode 'make cli-docs'" >&2
        exit 1
    fi
else
    mkdir -p "$(dirname "$out")"
    mv "$tmp" "$out"
    trap - EXIT
    echo "cli-docs: atualizado ($out)"
fi
