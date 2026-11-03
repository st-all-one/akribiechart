# Fase M4 — geometria e perfis de layout

**Status:** concluída · **Depende de:** M3 · **Habilita:** M5–M7

## Entrega

| peça | responsabilidade |
|---|---|
| `geometry.rs` | `Rect` (`center`, `right`, `bottom`, `union`, `inflate`, `translate`, `contains`), `outline_hit`, `route_line`/`route_arc`/`route_elbow`, `self_edge` |
| `layout/mod.rs` | `Placed`, `PlacedNode`, `PlacedEdge`, `PlacedGroup`, `PlacedNote`, `layout(chart, font)` |
| `layout/flow.rs` | dagre via `dugong` (`rankdir` de `direction`, `nodesep 48`, `ranksep 64`) |
| `layout/tree.rs` | Reingold–Tilford simplificado (`H_GAP 40`, `V_GAP 80`), espelhado em `bt`/`rl` |
| `layout/radial.rs` | anéis de `RADIUS_STEP = 180`; ângulo proporcional às folhas; raiz = 1º nó sem entrada |
| `layout/grid.rs` | grade `ceil(√n)` colunas (`GAP_GRID 40`) e pilha (`GAP_STACK 28`) |
| `layout/free.rs` | `at`/`size` declarados |
| `layout/finish.rs` | grupos (união recursiva + `pad`), notas, rotas, normalização do canvas |

## Decisões de execução

- **`Scene`/`*Draw` ficam em M5.** O plano lista `Placed` e `Scene` em M4, mas `Scene` carrega `d`
  (caminho SVG), `text: Vec<TextRun>` e `head` — tudo produzido pela pintura. Criar as structs antes
  do produtor geraria código morto; `Placed` é a fronteira do layout e `Scene` nasce onde é
  consumida.
- **`at` é o canto superior esquerdo** e é **relativo**: `finish` normaliza o bbox global e aplica
  `pad`. Posições declaradas preservam deslocamentos exatos (`b.x - a.x == 300`), o que é o que o
  autor controla; a origem absoluta do arquivo não vira origem do canvas.
- **Pino vale em qualquer perfil.** `at` em `flow`/`grid` reposiciona o nó depois do perfil; é a
  escotilha para ajustes pontuais sem trocar de layout.
- **Nota ancorada fica na diagonal inferior direita** do nó de `near=` (não à direita nem embaixo).
  Medido no caso de aceitação: à direita colidia com o rank seguinte (`dlq`) e embaixo com a linha
  de baixo (`auth`, grupo); na diagonal o caso fica sem sobreposição **sem** solver de colisão.
  Diagramas densos podem ainda sobrepor — a regra é determinística e previsível, não ótima.
- **Grupo é `bbox` recursivo + `pad`.** Subgrupos entram na união; ciclo em `in=` (impossível pelo
  parser, mas defensável) é cortado por `visiting`, sem laço infinito. Grupo sem membros não é
  desenhado.
- **Rota é nossa, não do dagre.** O dagre posiciona nós; `geometry` decide onde a linha toca o
  contorno (`outline_hit` por forma) e o traçado (`line`/`arc`/`elbow`). Assim a curva é a mesma
  nos três perfis que a usam e não depende do formato interno do `dugong`.
- **`arc` é amostrado em 17 pontos** (quadrática com controle a 15% na perpendicular): o `Placed`
  guarda pontos, não comandos SVG — a conversão para `path` é da pintura.
- **Falha do dagre cai para `stack`.** `dugong` só falha por controle de trabalho (aqui `Noop`), mas
  layout não pode desaparecer por um `Option`: o fallback é determinístico e visível.
- **`dugong` expõe `f64`.** A conversão para `f32` fica isolada em uma função com `#[expect]`
  justificado — uma exceção local e auditável à proibição de `as`.
- **`direction` espelha a profundidade em `bt`/`rl`** (árvore e fluxo); `mindmap` ignora
  `direction`, como o plano manda.

## Verificação

```sh
make check                                    # verde
cargo test -p akribie-core layout             # 15 testes de perfil/geometria
```

Caso de aceitação (`/tmp/probe.akrc`, `direction=lr`, grupo, nota ancorada):

| nó | x | y | w × h |
|---|---|---|---|
| api | 32.0 | 81.5 | 154.7 × 51.0 |
| retry | 250.7 | 32.0 | 110.5 × 51.0 |
| dlq | 425.2 | 32.0 | 96.0 × 51.0 |
| auth (`in=core`) | 257.9 | 131.0 | 96.0 × 51.0 |
| grupo `core` | 233.9 | 107.0 | 144.0 × 99.0 |
| nota | 385.2 | 95.0 | 161.9 × 43.0 |

| checagem | resultado |
|---|---|
| `a.x < b.x < c.x` e `a.y == c.y` em `flow`+`lr` | `flow_lr_orders_the_chain_left_to_right_on_one_row` |
| 4 nós em `grid` ⇒ 2 × 2 | `grid_with_four_nodes_is_two_by_two` |
| `free` preserva deslocamentos e `size` | `free_preserves_declared_offsets_and_sizes` |
| pai centrado nos filhos (`tree`) | `tree_centers_the_parent_over_the_children` |
| filhos no anel de 180 (`mindmap`) | `mindmap_puts_children_on_a_ring` |
| dois layouts idênticos | `layout_is_deterministic` |
| sobreposições nós+notas no caso acima | 0 (medido) |
| canvas 579.1 × 238.0, margem 32 | observado |
