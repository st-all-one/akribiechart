# MODULE — `akribie-core`

Domínio puro. Sem `std::fs`, relógio, RNG global ou ambiente: tudo que é externo atravessa
`ports`. Cada módulo tem `tests.rs` próprio; a API pública é o que a CLI e o MCP consomem.

| módulo | responsabilidade |
|---|---|
| `lex` | bytes → tokens com `span` (linha, coluna) |
| `parse` | tokens → `Document`; recuperação por linha |
| `syntax` | tabela única de chaves/valores (`option_specs`), validação, tipos de statement |
| `resolve` | `Document` + `Config` → `Chart`; `E004`/`E005`/`E006`/`W001`/`W002` |
| `model` | `Chart`, `Node`, `Edge`, `Group`, `Note`, enums fechados |
| `theme` | paletas `paper`/`slate`, tokens, contraste WCAG |
| `config` | `akribie.toml` (subconjunto determinístico de TOML) |
| `text` | duas faces embutidas (`Gauge` por família), shaping via `harfrust`, caixa segura, quebra de linha |
| `a11y` | `<desc>` derivado do modelo, checagem de contraste (`W007`) |
| `layout` | `Placed` por perfil: `flow`, `tree`, `mindmap`, `grid`, `stack`, `free` |
| `geometry` | pontos, retângulos, `outline_hit`, rotas (`line`/`arc`/`elbow`), self-edge |
| `sketch` | RNG FNV-1a por id, `Ink::Rough`/`Exact`, hachura/zigzag, formas e nuvem |
| `paint` | matriz forma × fillstyle → `Scene` |
| `emit` | bytes: `svg` (acessível), `scene_json`, `excalidraw`, `raster`, `import` |
| `import` | `.excalidraw` → `.akrc` (perdas em `W005`/`W006`) |
| `doc` | linhagem `doc=path#anchor` (`W004`, só com `--docs`) |
| `diff` | mudanças semânticas por id |
| `fmt` | forma canônica do `.akrc` |
| `pipeline` | `check` (estágios) e `batch_lines` (mecânica do JSONL; o verbo valida as chaves) |
| `num` | formatação canônica de número (contrato de bytes) |
| `error`, `span`, `diag` | tipo de erro + exit code, posição, códigos estáveis |
| `ports` | traits `Fs`, `Git`, `Env`, `Logger` e fakes |

Regra de leitura: `lib.rs` documenta os estágios; cada módulo documenta o contrato no topo do
arquivo. Nada aqui pode crescer além de 300 linhas por arquivo.
