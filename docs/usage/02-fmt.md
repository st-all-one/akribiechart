# `akr fmt`

Normaliza a fonte para a **forma canônica**. Não muda semântica: só ordem, espaçamento e números.

```sh
akr fmt chart.akrc                 # forma canônica em stdout
akr fmt chart.akrc --write         # reescreve no lugar
akr fmt chart.akrc --check         # exit 1 quando não está canônico
akr fmt - < chart.akrc
akr fmt chart.akrc --params '{"write":true}'
```

| flag | efeito |
|---|---|
| `[FILE]` | arquivo `.akrc`; `-` (ou ausência) lê `stdin` |
| `--params <JSON>` | parâmetros em JSON; substitui posicionais e flags do verbo |
| `--write` | grava o resultado no próprio arquivo |
| `--check` | sai 1 quando o arquivo não estiver na forma canônica |

`--write` e `--check` são mutuamente exclusivos (exit 2).

## O que a forma canônica garante

- `chart` primeiro; depois **grupos, nós, arestas, notas**, nesta ordem, estáveis entre execuções.
- Uma afirmação por linha, um espaço entre tokens, sem linha em branco entre statements do mesmo
  grupo; `chart` separado do corpo por uma linha em branco.
- Opções em ordem alfabética de chave (`at` antes de `fill` antes de `shape`…), sem repetição.
- Números com até duas casas decimais, sem `-0` e sem zeros à direita (`20` e não `20.00`).
- String com aspas duplas; `\` e `"` escapados; ids e palavras nunca entre aspas.
- Comentários (`#`) ficam colados ao statement **seguinte** e sobrevivem à formatação.
- Um `\n` final, nada depois.

`fmt` é idempotente: `akr fmt` sobre a saída de `akr fmt` não muda um byte. Arquivo com erro de
schema não é formatado: os diagnósticos vão para `stdout` e o exit é 8.

## No CI

```sh
akr fmt chart.akrc --check || echo "rode: akr fmt chart.akrc --write"
```
