# `akr check`

Valida a fonte e reporta diagnósticos posicionados. Não escreve arquivo nenhum.

```sh
akr check chart.akrc
akr check chart.akrc --strict
akr check chart.akrc --docs
akr check - < chart.akrc
akr check --batch lote.jsonl
akr check chart.akrc --json
```

| flag | efeito |
|---|---|
| `[FILE]` | arquivo `.akrc`; `-` (ou ausência) lê `stdin` |
| `--params <JSON>` | parâmetros em JSON; substitui posicionais e flags do verbo |
| `--json` | envelope JSON de uma linha em vez de texto |
| `--strict` | promove aviso a erro (exit 8) |
| `--docs` | resolve a linhagem `doc=` no disco (`W004`) |
| `--batch <FILE>` | lote JSONL: um objeto por linha (chaves de `--params` + `name`) |

## Formato do diagnóstico

```text
chart.akrc:2:14: E003 invalid-value: `shape=nope` não é um valor válido
  hint: valores de `shape`: rect, rounded, circle, diamond, cloud, cylinder, hexagon, note, text
```

Uma linha por diagnóstico em `stdout`, ordenada por posição; avisos aparecem junto. Exit 8 quando há
erro (ou aviso sob `--strict`).

## Envelope `--json`

```json
{"success":true,"command":"check","data":{"diagnostics":[]},"warnings":[]}
```

Com erro: `success:false`, `error:{code:"schema",message:"N erro(s), M aviso(s) (ver diagnósticos)"}`
e os diagnósticos em `data.diagnostics`.

## Lote (`--batch`)

Cada linha é um objeto com as chaves de `--params` (`path`, `source`, `docs`, `strict`) **mais
`name`**:

```json
{"name":"api","source":"chart v1 \"API\"\nnode a \"A\"\nnode b \"B\"\nedge a -> b\n"}
{"name":"arq","path":"docs/diagramas/fluxo.akrc","docs":true}
```

`name` é obrigatório e casa `[A-Za-z0-9_-]{1,64}`; `source` XOR `path`. Linhas vazias são ignoradas;
lote vazio ⇒ exit 2. `--batch` com `--params` ou `FILE` ⇒ exit 2. Nenhum arquivo é escrito.

Texto, uma linha por item:

```text
api  ok  0 aviso(s)
arq  erro  E003 `shape=nope` não é um valor válido
```

`--json` devolve o envelope com a contagem e os itens:

```json
{"success":true,"command":"check","data":{"failed":1,"items":[
  {"diagnostics":[],"index":0,"name":"api","ok":true},
  {"diagnostics":[{"code":"E003"}],"error":{"code":"E003","message":"…"},"index":1,"name":"arq","ok":false}],"ok":1},"warnings":[]}
```

Exit `0` quando ao menos um item passa; `8` quando todos falham (`success:false`,
`error.code:"schema"`). `strict` por item promove aviso a erro só naquele item.

## Linhagem `doc=` (`--docs`)

```akrc
chart v1 "Retry" doc="docs/adr/ADR-0003-layout-por-perfil.md#layout"
```

O caminho é relativo ao diretório do `.akrc`; a âncora é aceita como texto (sem caixa) ou como slug
do título (`# Layout` → `layout`). Arquivo ou âncora ausente ⇒ `W004` (warning, não bloqueia).
Sem `--docs`, `doc=` é apenas metadado: nada é lido do disco.

## Configuração do projeto

`akribie.toml` na raiz (criado por `akr init`) define `layout`, `direction`, `theme`, `fontsize`,
`pad`, `lang`, `sketch` e `font`. A precedência é: opção no `.akrc` > `akribie.toml` > default do
binário.
