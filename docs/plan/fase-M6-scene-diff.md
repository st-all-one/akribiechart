# Fase M6 — `scene` JSON e diff semântico

**Status:** concluída · **Depende de:** M4–M5 · **Habilita:** M7

## Entrega

| peça | responsabilidade |
|---|---|
| `emit/scene_json.rs` + `emit/scene_json/tests.rs` | cena posicionada em JSON: `canvas`, `nodes`, `edges`, `groups`, `notes`, cores resolvidas e rotas em pontos |
| `diff/mod.rs` | `Change`/`Kind`, `diff(old, new)` ordenado, comparação por identidade |
| `diff/describe.rs` | texto de cada elemento (`node api "API"`, `edge a->b#0`, `note near=api`) e os campos comparados |
| `commands/diff.rs` | `akr diff OLD NEW [--params] [--json]`, exit 1 quando há mudança |
| `paint`/`layout` | `PlacedEdge.ordinal` e `data-id="from->to#ordinal"` no SVG (identidade estável, ADR-0002) |

## Decisões de execução

- **`scene` não carrega `d`.** Caminho é derivado e ruidoso; o JSON expõe caixa (`rect`), rota em
  pontos, cor resolvida, estilo e rótulo. Os números passam por `num::format` e entram no JSON como
  **número** (não string): `214`, não `214.0` — `serde_json::Number` é construído a partir do texto
  canônico, o que evita o ruído de `f32 → f64`.
- **Chaves em ordem alfabética.** `serde_json::Map` é `BTreeMap` sem `preserve_order`, então a
  ordem é do tipo, não da inserção — determinismo por construção.
- **Um número de formatação só.** `sketch::num` era duplicata de `num::format` (criada em M5);
  foi removida e todos os emissores usam `crate::num::format`.
- **O diff compara o modelo resolvido**, nunca geometria: mover um nó não é mudança semântica.
  Identidade: `node`/`group` por `id`, `edge` por `(from, to, ordinal)`, `note` por
  `near` + ordem entre as notas da mesma âncora (a gramática não dá id a nota — a consequência 4 do
  ADR-0002 é sobre `node`/`group`/`edge`).
- **Linhas de adição mostram o rótulo; linhas de alteração mostram só a identidade.** A primeira
  versão repetia o rótulo novo em `~ node api "API Gateway" label: …`; o rótulo na identidade
  esconde o campo que mudou. `+` usa `node api "API"`, `~` usa `node api`.
- **Um campo por linha.** Três campos alterados são três linhas; o aceite conta linhas, e um
  consumidor de texto consegue ver cada campo sem parsing.
- **Ordem fixa:** `added` → `removed` → `changed`, cada grupo por id (ADR-0002).
- **Exit 1 quando há diferença** (como `git diff --exit-code` e `fmt --check`): CI distingue
  "comparou e difere" de "falhou ao comparar" (8).
- **Fonte inválida bloqueia o diff.** Sem modelo resolvido dos dois lados não há diff honesto:
  os diagnósticos do arquivo inválido vão para stdout e o exit é 8.
- **Avisos dos dois lados levam o rótulo do próprio arquivo** (`(file, Diagnostic)`), senão o
  warning do antigo apareceria como se fosse do novo.

## Verificação

| checagem | resultado |
|---|---|
| `diff` de três mudanças | exatamente 3 linhas: `+ edge b->c`, `+ node c "C"`, `~ node a label: A -> A2` |
| `diff` de arquivos iguais | stdout vazio, exit 0 |
| `diff --json` | `added` 2, `removed` 0, `changed` 1 |
| arquivo inválido | exit 8, `E003` com linha/coluna |
| `diff` sem posicionais | exit 2 |
| `scene` duas vezes | bytes idênticos; `canvas 558.52 × 115`, números inteiros (`h: 51`) |
| ids do SVG com aresta repetida | `a->b#0`, `a->b#1` |

| teste | garante |
|---|---|
| `diff/tests.rs::three_changes_are_exactly_three_lines_in_order` | aceite de contagem/ordem |
| `diff/tests.rs::edges_match_by_from_to_and_ordinal` | identidade de aresta |
| `diff/tests.rs::chart_level_fields_are_reported` | campos do `chart` |
| `diff/tests.rs::notes_are_matched_by_anchor_and_position` | identidade de nota |
| `emit/scene_json/tests.rs::scene_json_is_deterministic` | bytes estáveis |
| `emit/scene_json/tests.rs::numbers_are_rounded_to_two_places` | números canônicos |
| `akribie-cli/tests/diff.rs` (6 testes) | contrato de processo, JSON, erros |
