# `akr diff`

Compara dois `.akrc` e reporta mudanças **semânticas**, casadas por id (nós) e por
`(FROM, TO, ordinal)` (arestas).

```sh
akr diff antigo.akrc novo.akrc
akr diff antigo.akrc novo.akrc --json
akr diff chart.akrc --at HEAD          # árvore de trabalho × último commit
akr diff --params '{"old":"chart.akrc","at":"HEAD"}'   # o mesmo, em JSON
akr diff - novo.akrc < antigo.akrc
```

| flag | efeito |
|---|---|
| `[OLD] [NEW]` | arquivos `.akrc` (ou `-`); só um lado pode vir de `stdin` |
| `--params <JSON>` | parâmetros em JSON; substitui posicionais e flags do verbo |
| `--json` | envelope JSON em vez de linhas |
| `--at <REV>` | compara o arquivo atual com a revisão `REV` do Git (só um `FILE`) |

`--params` aceita `old`, `new` e `at`; com `at`, informe só um dos dois caminhos (o outro lado é a
revisão).

## Linhas de mudança

```text
+ node dlq "DLQ"              # added
- node legacy                 # removed
~ edge api->retry style: solid -> dashed
```

A ordem é `added`, `removed`, `changed`; dentro de cada classe, por id.

## Códigos de saída

| situação | exit |
|---|---|
| fontes iguais | 0 |
| ao menos uma mudança | 1 |
| fonte inválida de um lado | 8 (diagnósticos daquele arquivo, nada comparado) |
| posicionais ausentes / os dois de `stdin` | 2 |
| `--at` com revisão inexistente | 3 |

## Envelope `--json`

```json
{"success":true,"command":"diff","data":{"added":[],"changed":["~ node a label: A -> A2"],"new":"chart.akrc","old":"chart.akrc@HEAD","removed":[]},"warnings":[]}
```
