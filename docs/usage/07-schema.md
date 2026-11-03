# `akr schema`

Publica a gramática, os tokens de tema e os códigos de diagnóstico em JSON — a versão legível por
máquina de `GRAMMAR.md`.

```sh
akr schema
akr schema | jq '.statements.node.options[] | select(.key=="shape")'
akr schema | jq -r '.keywords[]'
```

| flag | efeito |
|---|---|
| — | não recebe argumentos |

A saída é JSON *pretty* com `\n` final e **não** é um envelope `--json`: o schema é o dado.

## Forma

```json
{
  "schema_version": 1,
  "akribie_version": "0.1.0",
  "keywords": ["chart", "group", "node", "edge", "note"],
  "statements": {
    "chart": { "options": [ { "key": "layout", "type": "enum", "default": "flow",
                              "values": ["flow", "tree", "mindmap", "grid", "stack", "free"],
                              "min": null, "max": null } ] }
  },
  "themes": [ { "name": "paper", "canvas": "#ffffff", "ink": "#1e1e1e",
                "palette": { "blue": "#a5d8ff" } } ],
  "contrast": { "min_text": 4.5, "min_stroke": 3.0 },
  "diagnostics": [ { "code": "E003", "name": "invalid-value", "severity": "error" } ],
  "exit_codes": [ { "exit": 8, "code": "schema", "description": "…" } ],
  "exit_success": 0,
  "exit_fmt_check": 1,
  "exit_diff_changes": 1
}
```

Nada aqui é escrito à mão: `statements` vem de `syntax::option_specs` (a mesma tabela do parser),
`themes` de `theme`, `diagnostics` de `diag` e `exit_codes` de `error`. Uma chave nova no núcleo
aparece no schema sem edição adicional — e `akr prompt --long` mostra as mesmas tabelas em texto.

## `--params` por verbo

Cada verbo aceita um JSON equivalente às flags (útil para agentes que montam a chamada como dado):

| verbo | chaves |
|---|---|
| `check` | `path`, `source`, `strict`, `docs`, `batch` |
| `fmt` | `path`, `source`, `write`, `check` |
| `render` | `path`, `source`, `format`, `out`, `scale`, `at`, `batch`, `out_dir` |
| `diff` | `old`, `new`, `at`, `old_source`, `new_source` |

`--params` substitui posicionais e flags do verbo: usar os dois juntos é exit 2.
