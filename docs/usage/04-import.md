# `akr import`

Converte um `.excalidraw` em `.akrc` canônico. A rota não é importada: o layout é recalculado.

```sh
akr import desenho.excalidraw > desenho.akrc
akr import desenho.excalidraw --json
akr import - < desenho.excalidraw
akr import --params '{"path":"desenho.excalidraw","name":"Retry"}'   # título do chart
```

| flag | efeito |
|---|---|
| `[FILE]` | arquivo `.excalidraw`; `-` (ou ausência) lê `stdin` |
| `--params <JSON>` | `path`, `source` (JSON literal) e `name`; substitui o posicional |
| `--json` | envelope JSON em vez da fonte |

`--params` traz o desenho (`path` XOR `source`) e, opcionalmente, `name` — o título do `chart`
gerado (padrão: radical do arquivo, ou `diagrama` no `stdin`). Chave fora de `path`, `source`,
`name` ⇒ exit 2.

## Mapeamento

| elemento Excalidraw | vira |
|---|---|
| `rectangle` | `node … shape=rounded` |
| `ellipse` | `node … shape=circle` |
| `diamond` | `node … shape=diamond` |
| `text` com `containerId` | rótulo do container |
| `text` solto | `note "…"` |
| `arrow` com os dois bindings | `edge FROM -> TO ["rótulo"]` |
| `linear` com `strokeStyle` | `style=solid\|dashed\|dotted` |
| `backgroundColor`/`strokeColor` da paleta | token (`blue`, `amber`, …) |

O resultado sai com `layout=free` e `at=`/`size=` explícitos, porque a posição do desenho é a
informação útil da importação.

## Perdas anunciadas

- `W005 color-unmapped` — cor fora da paleta do tema; o token cai no mais próximo declarado.
- `W006 element-dropped` — tipo sem equivalente (`frame`, `image`, `freedraw`…).

Em texto, as perdas vão para `stderr`; em `--json` vão em `data.diagnostics` junto de `data.source`.

## Ida e volta

O export `.excalidraw` marca cada elemento com `customData.akr` (`group`, `node`, `note`, `edge`).
No import, esses marcadores restauram **grupo**, **nota** e a **pertinência** (`in=`) que a geometria
sozinha não distinguiria; sem eles, a classificação é pelo tipo do elemento.

```sh
akr render chart.akrc --format excalidraw > chart.excalidraw
akr import chart.excalidraw | akr check -
```
