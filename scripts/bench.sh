#!/usr/bin/env bash
# Mede o custo por execução do binário em três modos, sobre as fixtures reais.
#
# Uso: scripts/bench.sh [--bin PATH] [--runs N]
#   --bin   binário a medir (padrão: target/release/akr); não compila por conta própria
#   --runs  execuções por combinação modo × fixture (padrão: 20)
#
# Saída: uma linha por combinação, `<modo> <fixture> <ms>`, em ordem fixa (LC_ALL=C sort).
set -euo pipefail

bin="target/release/akr"
runs=20

while [ "$#" -gt 0 ]; do
  case "$1" in
    --bin)
      [ "$#" -ge 2 ] || { echo "bench: --bin exige um caminho" >&2; exit 2; }
      bin="$2"
      shift 2
      ;;
    --bin=*)
      bin="${1#--bin=}"
      shift
      ;;
    --runs)
      [ "$#" -ge 2 ] || { echo "bench: --runs exige um número" >&2; exit 2; }
      runs="$2"
      shift 2
      ;;
    --runs=*)
      runs="${1#--runs=}"
      shift
      ;;
    -h|--help)
      sed -n '2,9p' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *)
      echo "bench: argumento desconhecido: $1" >&2
      exit 2
      ;;
  esac
done

case "$runs" in
  ''|*[!0-9]*) echo "bench: --runs precisa ser inteiro positivo, veio '$runs'" >&2; exit 2 ;;
esac
[ "$runs" -gt 0 ] || { echo "bench: --runs precisa ser > 0" >&2; exit 2; }

[ -x "$bin" ] || {
  echo "bench: binário não encontrado ou não executável: $bin" >&2
  echo "bench: rode 'cargo build --release -p akribie-cli' primeiro" >&2
  exit 1
}

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# Mede uma combinação: $1 = modo, $2 = fixture (caminho), imprime "<modo> <nome> <ms>".
measure() {
  local mode="$1" fixture="$2"
  local name
  name="$(basename "$fixture" .akrc)"
  local start end i
  start="$(date +%s%N)"
  for ((i = 0; i < runs; i++)); do
    case "$mode" in
      svg) "$bin" render "$fixture" >/dev/null ;;
      scene) "$bin" render "$fixture" --format scene >/dev/null ;;
      png) "$bin" render "$fixture" --format png --scale 2 --out "$tmp/$name.png" >/dev/null ;;
    esac
  done
  end="$(date +%s%N)"
  awk -v mode="$mode" -v name="$name" -v ns="$((end - start))" -v runs="$runs" \
    'BEGIN { printf "%s %s %.1f\n", mode, name, ns / runs / 1000000 }'
}

fixtures=()
while IFS= read -r f; do
  fixtures+=("$f")
done < <(find "$root/fixtures/reais" -name '*.akrc' | LC_ALL=C sort)

[ "${#fixtures[@]}" -gt 0 ] || {
  echo "bench: nenhuma fixture em fixtures/reais/*.akrc" >&2
  exit 1
}

for mode in svg scene png; do
  for fixture in "${fixtures[@]}"; do
    measure "$mode" "$fixture"
  done
done | LC_ALL=C sort
