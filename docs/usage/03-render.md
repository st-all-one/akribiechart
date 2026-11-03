# `akr render`

Compila o `.akrc` em um artefato. Formatos de texto vão para `stdout`; binários exigem `--out`.

```sh
akr render chart.akrc                                   # SVG em stdout
akr render chart.akrc --out chart.svg                   # SVG em arquivo (stdout silencioso)
akr render chart.akrc --format png --out chart.png      # raster escala 2
akr render chart.akrc --format png --scale 1 --out chart.png
akr render chart.akrc --format scene                    # JSON de depuração
akr render chart.akrc --format excalidraw > chart.excalidraw
akr render chart.akrc --at HEAD                         # fonte do último commit
akr render chart.akrc --format png --out chart.png --dry-run   # só mede e reporta
akr render --batch lote.jsonl --out-dir saida/          # lote
akr render chart.akrc --json                            # envelope com bytes/dimensões
```

| flag | efeito |
|---|---|
| `[FILE]` | arquivo `.akrc`; `-` (ou ausência) lê `stdin` |
| `--params <JSON>` | parâmetros em JSON; substitui posicionais e flags do verbo |
| `--format <FORMAT>` | `svg` (padrão), `scene`, `excalidraw`, `png`, `jpg` ou `avif` |
| `--out <PATH>` | arquivo de saída (obrigatório para raster) |
| `--scale <N>` | fator de escala da rasterização (padrão 2) |
| `--json` | envelope JSON em vez de dados |
| `--out-dir <DIR>` | diretório de saída do lote (`--batch`); precisa existir |
| `--batch <FILE>` | lote JSONL: um objeto por linha (chaves de `--params` + `name`) |
| `--at <REV>` | lê o fonte na revisão `REV` do Git (`git show REV:FILE`) |
| `--dry-run` | calcula bytes e dimensões sem gravar arquivo |

## Formatos

| formato | saída | destinos |
|---|---|---|
| `svg` | XML acessível (`role="img"`, `<title>`, `<desc>`) | `stdout` ou `--out` |
| `scene` | JSON determinístico da geometria | `stdout` ou `--out` |
| `excalidraw` | `.excalidraw` (JSON Excalidraw v2) | `stdout` ou `--out` |
| `png` / `jpg` / `avif` | bitmap | **só** `--out` (exit 2 sem ele) |

Raster sem `--out` é uso inválido: binário nunca vai para o terminal. `--scale 1` acompanha o
tamanho em pontos; `--scale 2` dobra pixels nas duas dimensões.

## Acabamento e fonte

Duas chaves de `chart` (ou do `akribie.toml`) mudam o traço e a tipografia sem tocar na topologia:

| chave | valores | padrão | efeito |
|---|---|---|---|
| `sketch` | `on`, `off` | `on` | `on` = traço à mão (duas passadas, semente do id); `off` = traço técnico (linhas exatas, arcos reais, uma passada) |
| `font` | `hand`, `sans` | `hand` | `hand` = Excalifont; `sans` = Liberation Sans (embutida, com métricas próprias) |

```akrc
chart v1 "Arquitetura" sketch=off font=sans
```

O modo técnico é o mesmo modelo e o mesmo layout: só a pintura muda. Os bytes do SVG também são
determinísticos nos dois modos.

## `--params`

Um objeto JSON com as mesmas chaves do lote: `path`, `source`, `format`, `out`, `scale`, `at`,
`dry_run`. `path` e `source` são exclusivos; com `--params` o stdin só é lido quando o próprio valor
é `-`. Chave fora da lista ⇒ exit 2 listando as válidas.

```sh
akr render --params '{"path":"chart.akrc","format":"png","out":"chart.png","scale":2}'
```

## Envelope `--json`

```json
{"success":true,"command":"render","data":{"bytes":2157,"dry_run":false,"format":"svg","height":115.0,"path":null,"width":590.0},"warnings":[]}
```

Em `--json` o payload não é impresso. `data` traz `path` (arquivo gravado, ou o alvo de
`--dry-run`), `format`, `bytes`, `width`/`height` (pontos nos vetoriais, pixels no raster já
escalado) e `dry_run`. Fonte inválida ⇒ `success:false`, `error.code:"schema"` e
`data.diagnostics`; nada é escrito.

## Lote

Cada linha é um objeto com as chaves de `--params` **mais `name`**:

```json
{"name":"um","source":"chart v1 \"x\"\nnode a \"A\"\nnode b \"B\"\nedge a -> b\n"}
{"name":"arq","path":"fixtures/retry.akrc","format":"png","scale":1}
{"name":"ruim","source":"chart v1 \"x\"\nnode a shape=nope\n"}
```

- `name` é obrigatório e casa `[A-Za-z0-9_-]{1,64}`; violação falha só aquele item.
- `source` XOR `path` (ambos/nenhum falham o item).
- `format` do item cai para `--format` e depois para `svg`; `scale` segue a mesma ordem (padrão 2).
- `out` ausente ⇒ `<out-dir>/<name>.<ext>`; `--out-dir` (padrão `.`) **precisa existir** (exit 3
  antes de processar). `out` explícito vence a derivação.
- `--dry-run` valida e reporta sem escrever nenhum arquivo.
- Linhas vazias são ignoradas; lote vazio ⇒ exit 2. `--batch` com `--params` ou `FILE` ⇒ exit 2.
- Processamento na ordem do arquivo; um item ruim não interrompe os demais.

Texto, uma linha por item:

```text
saida/um.svg  ok  4210 B
saida/arq.png  ok  19119 B
c  erro  invalid_input informe `path` ou `source`, não ambos
```

`--json` devolve um envelope com a contagem e os itens:

```json
{"success":true,"command":"render","data":{"ok":2,"failed":1,"items":[
  {"bytes":4210,"index":0,"name":"um","ok":true,"out":"saida/um.svg"},
  {"bytes":19119,"index":1,"name":"arq","ok":true,"out":"saida/arq.png"},
  {"error":{"code":"invalid_input","message":"informe `path` ou `source`, não ambos"},"index":2,"name":"ruim","ok":false}]},"warnings":[]}
```

Exit `0` quando ao menos um item renderiza; `8` quando todos falham (o envelope fica
`success:false` com `error.code:"schema"`).

## `--at REV`

`--at` lê o arquivo pela porta `Git` (`git show REV:FILE`) e renderiza essa versão, sem tocar o
diretório de trabalho. Revisão inexistente ⇒ exit 3, nada escrito.
