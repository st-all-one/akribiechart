# Fase M7 — lote, Excalidraw e linhagem documental

**Status:** concluída · **Depende de:** M1–M6 · **Habilita:** M8

## Entrega

| peça | responsabilidade |
|---|---|
| `pipeline::batch_lines` | separa o JSONL em itens (`index`, linha física, objeto ou erro), **best-effort**, na ordem |
| `batch::items` (CLI) | valida `name`, tira o campo e entrega o resto ao `params::parse` do verbo |
| `commands/check.rs::execute_batch` | `akr check --batch`, uma linha JSONL por item |
| `commands/render.rs::execute_batch` | `akr render --batch --out-dir`, `<name>.<ext>` por item válido |
| `emit/excalidraw.rs` (+ `support`, `shapes`, `edges`) | export derivado: `index` `a0000V`+, `fixedPoint` fora de 0/1, `seed` do id |
| `import/excalidraw.rs` (+ `collect`, `render`) | `.excalidraw` → `.akrc` canônico, perdas em `W005`/`W006` |
| `doc.rs` | `doc="path#anchor"` resolvido só com `--docs` (`W004`) |
| `app.rs` | `--batch` e `--out-dir` na CLI; `--at REV` |

## Decisões de execução

- **Lote sai 0 quando ao menos um item passa.** `ErrorKind::Schema` diz "lote 100 % falho"; o
  contrato é: `0` se algum item passou, `8` se todos falharam. Uma linha ruim não interrompe as
  seguintes, e o `success` de cada linha conta a história. (A primeira versão saía 8 a qualquer
  falha e foi corrigida contra a matriz.)
- **Saída do lote é JSONL, não envelope.** Um envelope por item seria parseável, mas o consumidor
  natural de lote é `| jq -c .`; o envelope fica para os verbos de item único.
- **`out` deriva de `name` + extensão do formato.** Sem `name`, o item não é renderizável; sem
  `--out-dir`, `render --batch` é uso inválido.
- **Export marca a origem em `customData.akr`.** Excalidraw aceita `customData` em qualquer
  elemento; usar isso deixa a ida-e-volta fiel: grupo volta a ser `group` (com `in=` recalculado por
  contenção), nota volta a ser `note`, e o nosso import não confunde os dois com `node`. Sem o
  marcador (arquivo de terceiros) a classificação é pelo tipo, como manda o ADR-0004.
- **Import não inventa rota.** `layout=free` + `at`/`size` explícitos: a posição é a informação
  útil; a rota é recalculada.
- **`index` de largura fixa.** `a<base62(i):0>4V`, sufixo `V` para nunca terminar em `0`
  (`validateOrderKey` do Excalidraw rejeita). Ordem dos elementos: `group`, `edge`, `node` (+ texto
  do container), `note`.
- **`binding` nunca cai em 0.5 exato**: meio lado é caso degenerado do Excalidraw; o cálculo desloca
  o ponto. Coberto por teste.
- **Âncora de documentação aceita texto ou slug.** GitHub usa slug (`# Guia Rápida` → `guia-rapida`);
  aceitar também o texto literal (sem caixa) evita falso `W004` em documento escrito à mão.
- **Item do lote = chaves do verbo + `name`.** O núcleo só separa as linhas; a CLI valida o objeto com a
  mesma tabela de `--params` (o `name` sai antes). Assim `format`/`scale`/`out`/`dry_run` do item e
  `docs`/`strict` do `check` têm o mesmo contrato da flag, sem uma segunda lista de chaves.
  `--out-dir` precisa existir (exit 3) e `out` explícito vence a derivação `<name>.<ext>`.
- **`W004` é warning.** `--docs` lê disco; sem a flag, `doc=` é metadado puro e nada é tocado.
- **Arquivos divididos por conceito**, para caber em 300 linhas: `emit/excalidraw/{support,shapes,
  edges}.rs` e `import/excalidraw/{collect,render}.rs`.

## Verificação

| checagem | resultado |
|---|---|
| lote de 3 (2 válidos, 1 inválido) | exit 0, 2 arquivos escritos, 3 linhas JSONL |
| lote 100 % inválido | exit 8 |
| export `.excalidraw` | `jq -e '.type=="excalidraw"'` ok; 18 elementos; `index` ordenado |
| import do próprio export | `group`, `note` e `in=` restaurados; `akr check` sem erro |
| `doc=` inexistente com `--docs` | `W004`, exit 0 |
| `doc=` válido (texto ou slug) | nenhum diagnóstico |
| `--at HEAD` (render e diff) | usa a versão do commit; exit 1 no diff com mudança |

| teste | garante |
|---|---|
| `akribie-cli/tests/check.rs::batch_reports_each_item_and_exits_zero_when_partially_valid` | lote parcial |
| `akribie-cli/tests/check.rs::batch_fails_only_when_every_item_fails` | lote 100 % falho |
| `akribie-cli/tests/check.rs::docs_lineage_resolves_relative_to_the_source` | linhagem válida |
| `akribie-cli/tests/check.rs::docs_lineage_reports_missing_document_or_anchor` | `W004` |
| `akribie-cli/tests/render.rs::batch_writes_only_valid_items_and_stays_zero_when_some_succeed` | `--out-dir` |
| `akribie-cli/tests/render.rs::excalidraw_export_is_json_with_order_keys` | export |
| `akribie-cli/tests/render.rs::at_revision_renders_the_committed_source` | `--at` |
| `akribie-cli/tests/import.rs` (4 testes) | import + envelope + JSON inválido |
| `akribie-cli/tests/diff.rs::at_revision_compares_the_working_tree_with_the_commit` | `diff --at` |
| `akribie-core/src/emit/excalidraw/tests.rs` | ordem, `index`, bindings |
| `akribie-core/src/import/excalidraw/tests.rs` | mapeamento, perdas, texto solto, JSON estrangeiro |
