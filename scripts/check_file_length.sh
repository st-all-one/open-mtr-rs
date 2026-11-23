#!/usr/bin/env bash
# Gate de tamanho de arquivo — orçamento ativo da revitalização.
#
# Alvo (padrão do knudge): 300 linhas por arquivo de produção. Os adapters,
# o cliente HTTP, o domínio, o erro e a CLI já foram divididos; o teto agora é
# o próprio alvo, para impedir regressão. `cli/` entra no gate como os demais.
set -euo pipefail

limit=300
status=0

report() {
    local file="$1"
    local lines
    lines=$(wc -l <"$file")
    if ((lines > limit)); then
        printf 'ERRO: %s tem %d linhas (limite %d)\n' "$file" "$lines" "$limit" >&2
        status=1
    fi
}

while IFS= read -r -d '' file; do
    report "$file"
done < <(find core cli napi uniffi wasm -type f -name '*.rs' -path '*/src/*' -print0)

if ((status == 0)); then
    printf 'file-length: ok (limite %d por arquivo de produção)\n' "$limit"
fi

exit "$status"
