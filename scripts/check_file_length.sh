#!/usr/bin/env bash
# Gate de tamanho de arquivo — orçamento transitório da revitalização.
#
# O alvo de longo prazo é 300 linhas por arquivo de produção (padrão do knudge).
# O código legado ainda tem adapters grandes; congelamos o teto atual (1500) para
# impedir regressão e reduzir o limite conforme os adapters forem extraídos.
set -euo pipefail

limit=1500
status=0

while IFS= read -r -d '' file; do
    lines=$(wc -l <"$file")
    if ((lines > limit)); then
        printf 'ERRO: %s tem %d linhas (limite %d)\n' "$file" "$lines" "$limit" >&2
        status=1
    fi
done < <(find core napi uniffi wasm -type f -name '*.rs' -path '*/src/*' -print0)

if ((status == 0)); then
    printf 'file-length: ok (limite transitório %d; alvo 300)\n' "$limit"
fi

exit "$status"
