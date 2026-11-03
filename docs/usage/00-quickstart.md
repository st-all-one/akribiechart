# Quickstart

`akr` compila uma fonte textual `.akrc` em diagramas. Você declara **topologia e rótulos**;
geometria, cor, texto alternativo e bytes são derivados. Nada de rede em tempo de execução.

## Instalar

```sh
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/st-all-one/akribie/main/install.sh | bash
```

Do source: `make release && make install` instala `akr` e `akribie-mcp` em `~/.local/bin`. O
instalador confere o SHA-256 do pacote do Release e aceita `VERSION`, `INSTALL_DIR` e
`AKR_TARGET`.

A fonte tipográfica (Excalifont, OFL) é embutida no binário; não há dependência de fontes do sistema.

## Primeiro diagrama

`retry.akrc`:

```akrc
chart v1 "Retry" layout=flow direction=lr theme=paper lang=pt-BR

node api "API" fill=blue
node retry "Retry ×3" fill=amber shape=cloud
node dlq "DLQ" fill=red shape=cylinder

edge api -> retry "5xx"
edge retry -> api "backoff" curve=arc style=dashed
edge retry -> dlq "esgotado" style=dashed
```

```sh
akr check retry.akrc            # valida (silencioso quando está tudo certo)
akr render retry.akrc > retry.svg
akr render retry.akrc --format png --out retry.png
```

## Ciclo recomendado

1. `akr check chart.akrc` — corrija **só a linha apontada**.
2. `akr render chart.akrc > chart.svg` — o SVG é o artefato versionado.
3. `akr fmt chart.akrc --check` — garante a forma canônica no CI.

## Saída e exit codes

`stdout` é dado; `stderr` é log. `--json` troca a saída por um envelope de uma linha:

```json
{"success":true,"command":"check","data":{"diagnostics":[]},"warnings":[]}
```

| exit | significado |
|---|---|
| 0 | sucesso |
| 1 | `fmt --check` diferiu ou `diff` encontrou mudanças |
| 2 | uso inválido (flag, formato, combinação) |
| 3 | arquivo, revisão git ou diretório não encontrado |
| 5 | falha de leitura/escrita |
| 8 | fonte inválida (≥1 erro) ou lote 100 % falho |
| 70 | invariante interna quebrada |

## Guias por verbo

- [01-check](01-check.md) · [02-fmt](02-fmt.md) · [03-render](03-render.md)
- [04-import](04-import.md) · [05-diff](05-diff.md) · [06-init-prompt](06-init-prompt.md)
- [07-schema](07-schema.md)

Contratos normativos: [GRAMMAR](../specs/GRAMMAR.md), [A11Y](../specs/A11Y.md),
[DETERMINISM](../specs/DETERMINISM.md), [ARCHITECTURE](../specs/ARCHITECTURE.md).
