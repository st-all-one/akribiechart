# Changelog

Todas as mudanças notáveis deste projeto são documentadas aqui.

O formato segue [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/) e o projeto adere ao
[Versionamento Semântico](https://semver.org/lang/pt-BR/).

## [0.1.2] - 2026-11-03

### Alterado

- `akr prompt` passa a se chamar `akr prime`; o texto emitido é o mesmo, byte a byte, e o golden
  acompanha o novo nome.

### Adicionado

- `chart.sketch` (`on`/`off`): `off` é o **traço técnico** — geometria exata, arcos reais e uma
  passada, sem perturbação; `on` continua o traço à mão determinístico.
- `chart.font` (`hand`/`sans`): família tipográfica do documento; `sans` usa a Liberation Sans
  (OFL-1.1, `assets/LiberationSans-Regular.ttf` + `assets/LIBERATION-OFL.txt`) com métricas próprias
  em layout, SVG e raster.
- Caixa segura de texto por forma (`layout/inset`): o nó é dimensionado pelo retângulo inscrito menos
  o respiro, então losango, círculo, hexágono, nuvem e cilindro crescem até o rótulo caber sem
  encostar no contorno.
- Faixa de título reservada na caixa do grupo, acima do `pad`.
- Rótulo de aresta com fundo próprio desenhado **sobre** o traço e afastamento iterativo que garante
  `LABEL_PAD` de folga também nos cantos de rotas curvas.

- Esqueleto do workspace Rust (`akribie-core`, `akribie-cli`, `akribie-mcp`, `xtask`), edition 2024,
  com `[workspace.lints]` e `clippy.toml` derivados do ADR-0001.
- ADR-0001 (simplicidade, determinismo e injeção de dependência), ADR-0002 (fonte texto canônica),
  ADR-0003 (layout por perfil), ADR-0004 (Excalidraw derivado) e ADR-0005 (acessibilidade do SVG).
- `docs/plan/` com panorama, fases M0–M8 e matriz de aceitação; `docs/review-protocol.md`.
- `xtask font build`: baixa o `Excalifont-Regular.woff2` oficial, verifica o SHA-256 de
  `assets/FONT.lock` e materializa `assets/Excalifont-Regular.ttf` de forma idempotente.
- `scripts/check_file_length.sh` (limite de 300 linhas por arquivo de produção) e `Makefile` com
  `check`, `ci`, `build`, `release`, `install` e `font`.
- Léxico e parser do `.akrc` com recuperação por linha: `chart`/`group`/`node`/`edge`/`note`,
  opções `chave=valor` tipadas, strings com escapes, `->`, pares `x,y` e listas de ids.
- Tabela única de opções (`syntax::option_specs`) que alimenta parser, `akr schema` e, adiante,
  `akr prime`.
- `akr check [FILE|-] [--json] [--strict] [--params JSON|-]`: diagnósticos
  `arquivo:linha:col: CODE nome: mensagem` com `hint`, exit 8 para erro, 2 para uso inválido,
  3 para arquivo ausente.
- `akr schema`: statements, chaves, tipos, defaults, valores fechados, 16 códigos de diagnóstico e
  a tabela de exit codes, todos derivados do código.
- Portas `Fs`, `Git`, `Env`, `Logger` com fakes determinísticos (`MemFs`, `FakeGit`, `FakeEnv`,
  `RecordingLogger`).
- `akr fmt`: forma canônica do `.akrc` (classes ordenadas, opções alfabéticas, um espaço, números com
  duas casas), com `--write` (no lugar), `--check` (exit 1 quando difere) e `--params`.
- Tema e contraste: `Token`/`Theme` com paletas fechadas `paper` e `slate`, `contrast_ratio` WCAG 2.2
  e `W007` para contraste abaixo do mínimo (texto × `fill` ≥ 4.5:1; contorno × superfície ≥ 3:1).
- Modelo resolvido (`Chart`, `Group`, `Node`, `Edge`, `Note`) e `resolve` com `E004`/`E005`/`E006`
  e nós implícitos `W001`, preservando `ordinal` de arestas repetidas.
- `akribie.toml`: `layout`, `direction`, `theme`, `fontsize`, `pad` e `lang`, com descoberta subindo
  diretórios e validação pelo mesmo léxico do `.akrc`.
- Medição de texto com a Excalifont embutida (shaping via `harfrust`), `wrap`, `label_size` e
  `node_size`, com `W003` para codepoint sem glifo.
- `pipeline::check` unifica parse + resolve + `W002` (nó isolado em layout de grafo) + `W003` +
  `W007`; `akr check` passa a usar a configuração do projeto.
- Descrição acessível (`a11y::describe`): contagem, nós e arestas em texto determinístico.
- `akr schema` publica `themes` (canvas, ink, paleta) e os mínimos de contraste.
- Layout por perfil: `flow` (dagre/`dugong`), `tree` (Reingold–Tilford), `mindmap` (radial), `grid`,
  `stack` e `free`, com `at` como pino em qualquer perfil e canvas normalizado.
- Geometria própria: `outline_hit` por forma, rotas `line`/`arc`/`elbow`, laço de auto-aresta e
  caixas de grupo recursivas (membros + `pad`).
- Notas ancoradas em `near=` na diagonal inferior direita do nó, sem colidir com o rank vizinho.
- Pintura (`paint`): `Scene` com formas em `d`, texto quebrado e cores já resolvidas; nós com
  `fillstyle` sólido/hachura/cross/zigzag recortados por `clipPath`, setas respeitando `head`/`tail`
  e rótulos de aresta deslocados para fora da linha.
- Traço à mão determinístico (`sketch`): mulberry32 semeado por FNV-1a do id, duas passagens por
  traço, `num` com duas casas e sem zeros.
- `akr render`: SVG acessível em stdout (`role="img"`, `<title>`, `<desc>`, `lang`) ou arquivo com
  `--out`; `--format svg|png|jpg|avif`, `--scale` (padrão 2), `--params` e `--json` com `bytes`;
  raster exige `--out` (exit 2 sem ele) e usa a fonte embutida injetada no `usvg`.
- `ranksep` do `flow` sobe para 96 px para caber o rótulo de aresta de 20 px.
- `akr render --format scene`: JSON da geometria posicionada (caixas, rotas em pontos, cores
  resolvidas), determinístico, sem `path`.
- `akr diff OLD NEW [--json]`: diff semântico do modelo resolvido, por `id` (e por
  `(from, to, ordinal)` em arestas), com linhas `+`/`-`/`~` na ordem added→removed→changed e exit 1
  quando há diferença.
- `PlacedEdge.ordinal` e `data-id="from->to#ordinal"` no SVG: identidade de aresta estável.
- `num::format` é a única formatação numérica (`sketch::num` duplicado removido).
- `akr render --format excalidraw`: export derivado e determinístico (`index` base62 de largura fixa,
  `seed` do id, `fixedPoint` fora dos extremos), com `customData.akr` para ida-e-volta fiel.
- `akr render --dry-run`: calcula bytes e dimensões sem gravar; o envelope diz `path` (alvo),
  `width`/`height` (pixels no raster) e `dry_run`.
- `akr import --params`: `path`/`source` e `name` (título do `chart` gerado).
- `akr import`: `.excalidraw` → `.akrc` canônico (`layout=free`), com mapeamento de formas, rótulos,
  setas e cores, perdas em `W005`/`W006`, e restauração de grupo/nota pelo `customData`.
- Lote JSONL (`akr check --batch`, `akr render --batch --out-dir`): cada linha é o `--params` do verbo
  mais `name`; best-effort, ordem do arquivo, exit 0 quando ao menos um item passa e 8 quando todos
  falham. Texto responde `<alvo>  ok  <bytes> B` / `<alvo>  erro  <CODE> <msg>`; `--json` traz
  `{ok,failed,items[]}`.
- `akr check --docs` e `doc="path#anchor"`: linhagem resolvida só sob demanda, âncora aceita como
  texto (sem caixa) ou slug de título, `W004` quando ausente.
- `--at REV` em `render` e `diff`: lê a fonte pelo `git show` pela porta `Git`, sem tocar a árvore.
- `akr init`: `akribie.toml`, bloco marcado em `AGENTS.md` e skill com marcador de versão —
  idempotente, com `--force` para a configuração e preservação de arquivo de terceiro.
- `akr prime [--long]`: gramática compacta (golden por versão) e, em `--long`, BNF, tabelas de
  chaves derivadas de `syntax::option_specs`, tokens de tema e códigos de diagnóstico.
- `akribie-mcp`: servidor MCP JSON-RPC por stdio com `chart_check`, `chart_render` (texto ou imagem
  base64) e `chart_diff`; falha de execução vira `isError`, o servidor nunca cai por fonte ruim.
- `docs/usage/00…07`, `docs/specs/{GRAMMAR,ARCHITECTURE,A11Y,DETERMINISM}.md` e `crates/*/MODULE.md`;
  teste de paridade entre `--help` e `docs/usage` (`tests/docs.rs`).
- Medição migrada de `rustybuzz`/`ttf-parser` (sem mantenedor) para `harfrust`, com bytes de SVG e
  PNG idênticos; `xtask` saiu do workspace (lock próprio) e o `bytes 1.9.0` vulnerável saiu do
  lockfile publicado. `cargo deny` verde com um desvio documentado (`paste`, via `rav1e`).
- `install.sh` (estilo `curl | bash`) para os mesmos seis alvos do release: baixa o pacote do
  GitHub Release, verifica o SHA-256 de `sha256sums.txt`, preserva o binário anterior em lixo
  recuperável e edita o PATH; `--from-source` compila os dois binários e `--uninstall` só move.
- `.github/workflows/release.yml`: tag `v*` → portão `make check` e tag == versão → matriz
  `x86_64/aarch64-unknown-linux-musl`, `x86_64/aarch64-apple-darwin`,
  `x86_64/aarch64-pc-windows-msvc` → `akribie-<versão>-<target>.{tar.gz,zip}` +
  `sha256sums.txt` no Release.
- `make release` e `make install` passam a construir e instalar `akr` **e** `akribie-mcp`.

### Corrigido

- A medição de texto não chamava `Buffer::guess_segment_properties`; o shaping devolvia
  `DirectionUnset` e toda largura caía na estimativa `caracteres × 0,5 × corpo`. Agora os avanços vêm
  da face embutida (e `font=sans` mede com a Liberation Sans).
- O fundo do rótulo de aresta era pintado **antes** do traço, que atravessava o texto; agora o fundo
  interrompe a linha, com 6 px de folga — e afastamento crescente em rotas curvas.
- Rótulos de aresta reta eram ancorados no nó de destino (índice central de uma rota de dois pontos);
  passam a usar o meio por comprimento de arco.
- O cilindro desenhava a tampa duas vezes (aba no topo) e a nuvem ultrapassava o retângulo do nó.
- A elipse à mão usava os controles cúbicos uma casa à frente e desenhava um asterisco no lugar do
  círculo; os controles agora seguem a ordem `direita → baixo → esquerda → topo`.
- Rótulo que cabia exatamente na caixa segura era quebrado em duas linhas pelo arredondamento do
  `f32` entre `node_box` e `text_box` (ex.: `Auth / OIDC` em `font=sans`); a quebra ganhou uma
  tolerância de 0,05 px.
