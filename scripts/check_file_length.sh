#!/usr/bin/env bash
# Gate de tamanho de arquivo (ADR-0001 §4): produção em crates/*/src e xtask/src ≤ 300 linhas.
# Módulos de teste embutidos (#[cfg(test)]) contam; o limite é aplicado ao arquivo.
set -euo pipefail

limit=300
status=0

while IFS= read -r -d '' file; do
    # `awk` conta a última linha mesmo sem `\n` final; `wc -l` conta quebras.
    lines=$(awk 'END { print NR }' "$file")
    if ((lines > limit)); then
        printf 'ERRO: %s tem %d linhas (limite %d)\n' "$file" "$lines" "$limit" >&2
        status=1
    fi
done < <(find crates xtask -type f -name '*.rs' -path '*/src/*' -print0)

if ((status == 0)); then
    printf 'file-length: ok (limite %d)\n' "$limit"
fi

exit "$status"
