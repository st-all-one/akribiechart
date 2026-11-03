# ARCHITECTURE — módulos, portas e fluxo

## Workspace

| crate | papel | I/O |
|---|---|---|
| `akribie-core` | domínio puro: lex → parse → resolve → layout → paint → emit | nenhum (só portas) |
| `akribie-cli` | binário `akr`: adaptadores reais, envelope, exit codes | arquivo, `git`, `stdout` |
| `akribie-mcp` | servidor MCP (JSON-RPC por stdio) sobre o mesmo núcleo | `stdin`/`stdout` |
| `xtask` | ferramenta de build (woff2 → ttf), lock próprio, fora do lockfile | rede (uma vez) |

`akribie-core` não conhece `std::fs`, relógio, RNG global nem variável de ambiente. Tudo que é
externo atravessa as portas de `ports`:

| porta | uso |
|---|---|
| `Fs` | ler/gravar arquivos, testar existência (check `--docs`, batch, `--out`) |
| `Git` | `git show REV:FILE` para `--at` |
| `Env` | variáveis de ambiente, confinadas à CLI |
| `Logger` | avisos e log em `stderr` |

`ports::fakes` oferece `MemFs`, `FakeGit`, `FakeEnv` e `RecordingLogger`; todo teste de domínio usa
fakes e nenhum toca o disco.

## Fluxo

```text
bytes ──lex──▶ tokens ──parse──▶ Document ──resolve──▶ Chart ──layout──▶ Placed ──paint──▶ Scene
                                                                                    │
                                        emit/svg ◀──────────────────────────────────┤
                                        emit/scene_json ◀───────────────────────────┤
                                        emit/excalidraw ◀───────────────────────────┤
                                        emit/raster (SVG → usvg → tiny-skia → codec) ┘
```

- `lex`/`parse`: erro por linha, com `span` (linha e coluna) em cada diagnóstico.
- `resolve`: id → objeto; aplica `akribie.toml` onde o `.akrc` não declara; emite `E004`/`E005`/
  `E006`/`W001`/`W002`.
- `layout`: geometria pura por perfil (`flow`, `tree`, `mindmap`, `grid`, `stack`, `free`), rotas de
  aresta próprias (`route_line`/`route_arc`/`route_elbow`), `outline_hit` e self-edge. `layout/inset`
  define a caixa segura do texto por forma e é a mesma função que dimensiona o nó.
- `text`: duas faces embutidas (`Gauge` por família) — Excalifont (`font=hand`) e Liberation Sans
  (`font=sans`); medição, SVG e raster usam os mesmos bytes.
- `paint`: resolve a matriz forma × fillstyle (hachure, cross-hatch, zigzag, solid) com `clipPath` e
  escolhe o acabamento (`sketch::Ink::Rough` ou `::Exact`); o rótulo de aresta ganha fundo próprio
  e folga mínima de 6 px.
- `emit`: bytes. `svg` é contrato de acessibilidade; `scene_json` é depuração estável; `excalidraw`
  é interop; `raster` registra as duas faces que mediram o layout.

`pipeline::check` amarra os estágios sem I/O; `pipeline::batch_lines` só separa o JSONL — as chaves de
cada verbo são validadas na CLI, pela mesma tabela de `--params`.
`akr check`, `akr render` e o MCP chamam o mesmo `pipeline`.

## Invariantes

1. **Determinismo** — sem relógio, sem `HashMap` iterado, sem `f32` impresso pela std (use `num`).
   Ver [DETERMINISM](DETERMINISM.md).
2. **Um caminho de medição** — layout, pintura e SVG usam `text::Font` (mesma face, mesmo shaping).
3. **Acessibilidade derivada do modelo**, não da geometria ([A11Y](A11Y.md)).
4. **Portas para tudo que é externo**; o núcleo é testável sem ambiente.
5. **Arquivos de produção ≤ 300 linhas** (`scripts/check_file_length.sh`), um conceito por módulo.

## Qualidade

`make check` = `fmt` + `clippy -D warnings` (pedantic + nursery, `unsafe` negado, sem
`unwrap`/`expect`/`panic`) + testes + tamanho de arquivo. `make ci` acrescenta `cargo deny` e
`typos`.

| camada | o que cobre |
|---|---|
| unidade em `src/*/tests.rs` | estágios isolados com dados puros |
| integração em `crates/*/tests/` | o binário real, por pipe, com fixtures temporárias |
| goldens | forma canônica, SVG, `scene`, `prompt` |

Desvios de `cargo deny` são explícitos em `deny.toml` com motivo e rota de saída; o único hoje é
`paste` (proc-macro do `rav1e`, encoder AVIF), que não entra no binário.
