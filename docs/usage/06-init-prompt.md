# `akr init`, `akr prompt` e o servidor MCP

Três entradas para agentes: instalar o projeto, ler a gramática, falar por MCP.

## `akr init`

```sh
akr init
akr init --force      # reescreve akribie.toml mesmo existente
```

| flag | efeito |
|---|---|
| `--force` | reescreve `akribie.toml` mesmo quando já existe |

Instala três arquivos, todos idempotentes:

| arquivo | conteúdo | regra |
|---|---|---|
| `akribie.toml` | `layout = flow`, `direction = lr`, `theme = paper`, `fontsize = 20`, `pad = 32`, `lang = "en"`, `sketch = on`, `font = hand` | existente só muda com `--force` |
| `AGENTS.md` | bloco entre `<!-- akribie:start -->` e `<!-- akribie:end -->` | substituído no lugar; nunca duplica |
| `.agents/skills/akr/SKILL.md` | skill com `<!-- akribie:skill:version: 1 -->` | arquivo **sem** o marcador é preservado |

A saída diz o que aconteceu com cada arquivo — `novo`, `atualizado`, `inalterado`, `preservado` — e
termina em `próximos:` com o primeiro comando a rodar.

## `akr prompt`

```sh
akr prompt            # compacto: gramática, conjuntos, 2 exemplos, regras
akr prompt --long     # + BNF, tabelas de chaves por statement e tokens de tema
```

| flag | efeito |
|---|---|
| `--long` | anexa BNF completa, tabelas de chaves e tokens de tema |

O texto é **inglês** e constante por versão (golden em `crates/akribie-cli/tests/golden/`). Ele
documenta a regra de correção: `check` sai 8, aponte a linha, reescreva **só** a linha apontada.

## Servidor MCP

`akribie-mcp` fala JSON-RPC 2.0 por linha em `stdin`/`stdout`; EOF encerra com exit 0.

```sh
printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' |
  akribie-mcp
```

| ferramenta | argumentos | saída |
|---|---|---|
| `chart_check` | `source` \| `path`, `strict?` | texto: `ok (N erro(s), M aviso(s))` ou o primeiro diagnóstico |
| `chart_render` | `source` \| `path`, `format?`, `scale?` | texto (svg/scene/excalidraw) ou bloco `image` base64 (png/jpg/avif) |
| `chart_diff` | `old_source`, `new_source` | linhas de mudança, ou `sem mudanças (0)` |

Falha de execução volta como **resultado** com `isError: true` e a primeira linha do diagnóstico em
`content[0].text` — o servidor continua atendendo. `tools/call` com nome desconhecido é erro de
protocolo (`-32602`); JSON malformado, `-32700`; método desconhecido, `-32601`.
