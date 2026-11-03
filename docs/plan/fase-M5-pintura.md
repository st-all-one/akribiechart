# Fase M5 — pintura, SVG acessível e raster

**Status:** concluída · **Depende de:** M4 · **Habilita:** M6–M7

## Entrega

| peça | responsabilidade |
|---|---|
| `sketch/mod.rs` | reexporta `num`, `Rng`, `seed_for`, formas e hachura; `MAX_OFFSET`; `arrow_head` |
| `sketch/rng.rs` | mulberry32 (`next_u32`, `signed`), `seed_for` (FNV-1a), `two_passes`, `jitter` |
| `sketch/helpers.rs` | `moveto`/`lineto`/`xy`, `num` (2 casas, sem zeros) e `clamp_to_rect` |
| `sketch/shapes.rs` | `line`, `polyline` (`Trace::Open/Closed`), `rect`, `ellipse`, `cloud`, `note_shape`, `cylinder`, `diamond`/`hexagon` por pontos |
| `sketch/hatch.rs` | `hatch` (`Tilt::Forward/Mirrored`) e `zigzag`, recortados por `clipPath` |
| `paint/mod.rs` | `Scene` e `*Draw`, `paint(chart, placed, font)`, `text_runs`, `dash_of`, `aria_of` |
| `paint/node.rs` | forma × `fillstyle` (sólido, hachura, cross, zigzag) e rótulo com quebra |
| `paint/edge.rs` | rota, setas por lado (`head`/`tail`), rótulo com fundo e deslocamento perpendicular |
| `paint/group.rs` | caixa tracejada com rótulo e nota com canto dobrado |
| `emit/svg.rs` | documento acessível (`role`, `title`, `desc`, `lang`), grupos `groups`/`edges`/`nodes`/`notes`, `clipPath` por nó |
| `emit/raster.rs` | SVG → PNG/JPEG/AVIF com a fonte embutida injetada no `usvg` |
| `commands/render.rs` | `akr render` (`--format`, `--out`, `--scale`, `--params`, `--json`) |

## Decisões de execução

- **A `Scene` é pura.** `paint` resolve tema, quebra texto e gera `d`; os emissores só concatenam
  strings. Assim SVG e raster consomem a mesma cena e não divergem na fonte.
- **`clipPath` por nó** (`c-<id>`) recorta hachura/zigzag na forma; sem recorte, `diamond` e `cloud`
  vazariam o preenchimento.
- **`num` é a única formatação de número** em `d` e atributos: duas casas, sem zeros à direita, sem
  notação científica. É o que garante bytes estáveis.
- **Rótulo de aresta fica no meio da rota, deslocado para o lado "de cima".** A primeira versão
  desenhava o rótulo sobre a linha (cobria a seta) e usava `route[len/2]`, que numa rota de dois
  pontos é o **ponto final** — rótulos colavam no nó de destino. Corrigido para a média dos dois
  pontos centrais; `ranksep` do `flow` subiu de 64 para 96 px para caber texto de 20 px. Diagramas
  muito densos ainda podem sobrepor — regra determinística, não solver de colisão.
- **Setas respeitam `head`/`tail`.** `arrow_at_end`/`arrow_at_start` (sem `bool` de parâmetro, pelo
  lint `fn_params_excessive_bools`).
- **A fonte é injetada no raster.** `usvg::Options.fontdb` recebe os mesmos bytes embutidos que
  medem o layout; sem isso o PNG sairia com métricas do sistema. AVIF é codificado com uma thread
  fixa (`with_num_threads(Some(1))`) — paralelismo variável muda os bytes.
- **Raster nunca vai para `stdout`** (exit 2 sem `--out`): terminal não é destino de binário.
  `--json` devolve `bytes` do que foi gravado, o que permite conferir tamanho sem `stat`.
- **`--scale` padrão 2** e canvas em unidades do documento: 579.12 × 238 → 1159 × 476 px.
- Os três arquivos passavam de 300 linhas; `sketch` e `paint` viraram diretórios (`rng`, `helpers`,
  `shapes`, `hatch`; `mod`, `node`, `edge`, `group`) e os testes do SVG foram para
  `emit/svg/tests.rs`.

## Verificação

Caso de aceitação (`probe.akrc`, 4 nós, 3 arestas, 1 grupo, 1 nota, `flow`+`lr`):

| checagem | resultado |
|---|---|
| dois renders ⇒ bytes idênticos | `cmp` ok |
| SVG válido | `xmllint --noout` sem erro |
| `role="img"`, `<title>`, `<desc>`, `aria-labelledby`, `lang="pt-BR"` | presentes |
| PNG `--scale 2` | 1159 × 476 px |
| JPG / AVIF | 104 115 / 28 402 bytes |
| `--json` `bytes` × `stat` | 63 979 = 63 979 |
| raster sem `--out` | exit 2 |
| fonte inválida | exit 8, sem arquivo, diagnósticos em stdout |
| inspeção visual do PNG | setas no lado `to`, rótulos fora dos nós, texto legível |

| teste | garante |
|---|---|
| `sketch::tests::seed_is_stable_and_well_distributed` | semente por id |
| `sketch::tests::generators_are_identical_for_the_same_seed` | determinismo dos `d` |
| `sketch::tests::paths_are_closed_and_use_no_scientific_notation` | `num` canônico |
| `emit/svg/tests.rs::svg_has_the_accessible_structure` | estrutura A11y |
| `emit/svg/tests.rs::arrows_follow_the_declared_sides` | `head`/`tail` |
| `emit/svg/tests.rs::every_shape_produces_stroke_and_fill` | 8 formas com traço, fill e clip |
| `emit/svg/tests.rs::markup_is_escaped` | escaping de texto |
| `akribie-cli/tests/render.rs` (7 testes) | contrato de processo: stdout, determinismo, raster, erros |
