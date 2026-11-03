# Fases

Cada fase deixa a árvore compilando, `make check` verde e a documentação correspondente escrita. A
ordem é obrigatória: M5 consome a geometria de M4, e M7 consome o `Scene` de M5.

| fase | entrega | aceite resumido |
|---|---|---|
| **M0** | esqueleto do workspace, ADRs, plano, protocolo de revisão, fonte embutida, `make check` | workspace vazio verde; `xtask font build` idempotente; `.ttf` presente |
| **M1** | `span`, `error`, `syntax::option_specs`, `lex`, `parse`, portas + fakes, CLI (`check`, `schema`), `--params` | `E008` na linha certa; `schema` lista os 6 layouts; `--params` aceita e rejeita conforme §2.2 |
| **M2** | `fmt` canônico com transporte de comentários | `fmt` idempotente byte a byte; `--check` sai 1 em arquivo não canônico |
| **M3** | `theme`, contraste WCAG, `resolve`, `config`, `a11y::describe`, `text::Font`, `check` com `W002`/`W003`/`W007` | teste de contraste verde; `W007` em `fill=blue stroke=blue`; `--strict` sai 8 |
| **M4** | `scene` (Placed/Scene), perfis `flow`/`tree`/`mindmap`/`grid`/`stack`/`free`, `geometry` | `flow lr` com `a→b→c` dá `a.x < b.x < c.x`, `a.y == c.y`; `free` sem `at` dá `E006`; `grid` 4 nós = 2×2 |
| **M5** | `sketch`, `paint`, `emit/svg` acessível, `emit/raster`, comando `render` | dois renders = SVG idêntico; `role`/`title`/`desc` presentes e válidos em `xmllint`; PNG/JPG/AVIF reconhecidos |
| **M6** | `emit/scene_json`, `diff`, comando `diff` | três mudanças produzem exatamente três linhas, na ordem `added`/`removed`/`changed` |
| **M7** | `pipeline::batch_lines` + driver de lote na CLI, `--batch` em `render`/`check`, export/import Excalidraw, `doc.rs`, `--at REV` | lote de 3 (2 válidos, 1 inválido) sai 0 e escreve 2 arquivos; export valida em `jq`; import gera `.akrc` aceito |
| **M8** | `init`, `prime`, `akribie-mcp`, docs de uso/specs/MODULE, matriz preenchida | `init` idempotente; `prime` byte-idêntico; MCP lista 3 tools e não cai em erro |

## M0 — esqueleto, governança e fonte

Workspace com `akribie-core` (lib), `akribie-cli` (bin `akr`), `akribie-mcp` (bin), `xtask` excluído
do workspace. Lints de workspace, `clippy.toml`, `rustfmt.toml`, `rust-toolchain.toml`, `deny.toml`,
`typos.toml`. ADRs 0001–0005, `review-protocol.md`, `00-panorama.md`, `01-fases.md` e a matriz vazia.
`xtask font build` baixa o woff2 oficial, verifica o SHA-256 de `assets/FONT.lock` e converte para
`assets/Excalifont-Regular.ttf` (commitado). `scripts/check_file_length.sh`, `Makefile`, CI.

## M1 — léxico, AST, parser e `check`

Uma tabela de opções (`option_specs`) é a fonte única de chaves, tipos, defaults e valores fechados;
parser, `akr schema` e `akr prime` leem dela. Recuperação **por linha**: uma linha inválida produz
diagnóstico e não invalida as demais. Portas com fakes. `--params` deserializado por verbo, com
conflito posicional ⇒ exit 2.

## M2 — `fmt` canônico

Ordem de classes, comentários colados ao statement seguinte, um espaço, opções alfabéticas, números
com duas casas, um `\n` final. `--check` é o que impede degradação.

## M3 — modelo, temas, contraste e texto

`Chart`/`Node`/`Edge`/`Group`/`Note`; resolução com `E004`/`E005`/`E006`/`W001`; temas fechados e
teste exaustivo de contraste; `a11y::describe` derivado do modelo; medição com a face embutida
(`harfrust`), `W003` para codepoint sem glifo.

## M4 — layout

`Placed` é geometria pura; `Scene` é pronto para emitir. Cada perfil em um arquivo, atrás de
`layout(&Chart, &Font) -> Placed`. Roteamento de aresta é nosso (`route_line`/`route_arc`/
`route_elbow`, `outline_hit`, self-edge).

## M5 — pintura, traço e emissão

`sketch` com RNG próprio semeado pelo id (FNV-1a 32), duas passagens por traço. `paint` resolve a
matriz forma × fillstyle (hachure, cross-hatch, zigzag, solid) com `clipPath` no próprio `d`.
`emit/svg` produz o contrato acessível; `emit/raster` usa a mesma face embutida e pina uma thread de
codificação.

## M6 — `scene` JSON e diff

`scene` expõe geometria estável para depuração. `diff` casa por id (e por `(from,to,ordinal)` para
arestas) e reporta `added`/`removed`/`changed` ordenados por id.

## M7 — lote, Excalidraw e linhagem

Lote = JSONL, sequencial, best-effort: uma linha por item (`name`, `success`, `diagnostics`, e
`bytes`/`out` no `render`); o processo sai 0 quando ao menos um item passa e 8 quando todos falham
(`ErrorKind::Schema`, "lote 100% falho"). `out` é derivado de `name`+extensão. Export Excalidraw
constrói `index` e `fixedPoint` válidos; import recalcula rota e anuncia perdas (`W005`/`W006`).
`doc=` é resolvido só com `--docs` (`W004`), com âncora aceita como texto sem caixa ou como slug do
título. `--at REV` lê via `git show` pela porta `Git`.

## M8 — `init`, `prime`, MCP e documentação final

`init` instala `akribie.toml`, bloco em `AGENTS.md` e skill em `.agents/skills/akr/SKILL.md`, com
marcadores e idempotência. `prime` é constante em inglês, byte-idêntica por versão. MCP expõe
`chart_check`, `chart_render`, `chart_diff` sobre stdio, com erro como `isError` e servidor vivo.
Documentação de uso por verbo, specs normativas, `MODULE.md` por crate e matriz preenchida.
