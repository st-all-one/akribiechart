# akribie — fonte compilada de diagramas para LLMs (v1)

## Contexto

Criar `akr`, um binário Rust único, 100% local e offline, que compila um arquivo-texto (`.akrc`) em
diagramas estilo Excalidraw — SVG por padrão, com PNG/JPG/AVIF sob demanda e `.excalidraw` derivado.
O usuário é um LLM ou um agente: ele escreve **topologia e rótulos**, nunca coordenadas; o layout é
estágio separado, determinístico e pinado no arquivo. O `.akrc` é a verdade versionada; todo o resto
é artefato de build. O diagrama é um **elemento vivo de uma decisão**: diff semântico no Git,
linhagem explícita (`doc=`) para o ADR que ele ilustra e saída acessível por construção.

Estado atual do repositório: `README.md` de uma linha, `LICENSE`, `_REF/` (referências, não editar).
Nenhum código. Greenfield.

---

## 1. Constituição

`docs/adr/ADR-0001-simplicidade-determinismo-di.md` é o primeiro artefato do projeto e vale como
norma para todo o resto. Conteúdo obrigatório, em pt-BR, com estas cinco decisões numeradas:

1. **Simplicidade primeiro.** Toda infraestrutura precisa pagar por si no caso local. Proibido:
   servidor, daemon, banco, `tokio`, `async` em qualquer caminho, threads no domínio.
2. **Determinismo é contrato.** Mesma entrada + mesma versão do binário ⇒ mesmos bytes de SVG e de
   scene JSON. Proibido no domínio: `SystemTime::now`, `env::var`, `rand`, iteração sobre
   `HashMap`/`HashSet`. Sementes sempre derivadas do id (`sketch::seed_for`).
3. **Injeção de dependência por portas.** O domínio expõe funções puras; toda interação com o mundo
   (`Fs`, `Git`, `Env`, `Logger`) atravessa traits em `ports`, com fakes em `ports::fakes`. Os
   comandos recebem `&dyn Fs` / `&dyn Git`; a CLI é a única que monta `StdFs`/`StdGit`.
4. **Rigor de processo.** `make check` (fmt + clippy `-D warnings` + test + limite de linhas) é o
   portão único; nenhuma exceção por arquivo. `clippy.toml` + `[workspace.lints]` proíbem as
   construções listadas em §6.3. Arquivos de produção ≤ 300 linhas.
5. **Documentação é entregável.** `docs/` com ADRs, plano em fases separadas, protocolo de revisão,
   specs e guias de uso; `AGENTS.md`, `llms.txt`, `SKILL.md` e `CHANGELOG.md` na raiz. Superfície,
   docs e testes mudam no mesmo passo.

Referências de execução dos cinco pontos: `_REF/knudge/AGENTS.md` §0 e §3,
`_REF/knudge/crates/knudge-core/MODULE.md`, `_REF/knudge/Makefile`.

---

## 2. Superfície

Binário `akr`. Oito verbos, `stdout` = dados, `stderr` = logs. Sem TUI, sem daemon, sem rede.

```
akr init   [--force]
akr prompt [--long]
akr check  [FILE|-] [--batch FILE|-] [--params JSON|-] [--json] [--docs] [--strict]
akr fmt    [FILE|-] [--params JSON|-] [--write | --check]
akr render [FILE|-] [--batch FILE|-] [--params JSON|-]
                    [--format svg|png|jpg|avif|scene|excalidraw]
                    [--out PATH] [--out-dir DIR] [--scale N] [--at REV] [--json] [--dry-run]
akr import [FILE|-] [--params JSON|-] [--json]
akr diff   [OLD] [NEW] [--params JSON|-] [--json] [--at REV]
akr schema
```

### 2.1 Entrada

- `FILE` ausente **e** stdin não-TTY → lê de stdin. `-` explícito → stdin. `FILE` ausente com TTY →
  uso em `stderr`, exit 2.
- `render --format svg|scene|excalidraw` escreve em stdout por padrão; `--out` sobrescreve.
- `render --format png|jpg|avif` **exige** `--out` (exit 2 sem ele): raster é binário.
- `render --at REV` lê via `git show REV:<path>`; sem `git` no PATH → exit 5.

### 2.2 `--params`

Aceito em `check`, `fmt`, `render`, `import`, `diff`. É **um objeto JSON** com os parâmetros daquele
verbo; `--params -` lê o objeto do stdin. Substitui integralmente posicionais e flags daquele verbo;
combinar `--params` com posicional ou flag do verbo → exit 2 (`--json` é a exceção permitida).

Chaves válidas por verbo (chave fora da lista → exit 2 com a lista):

| verbo | chaves |
|---|---|
| `check` | `path`, `source`, `docs`, `strict` |
| `fmt` | `path`, `source`, `write`, `check` |
| `render` | `path`, `source`, `format`, `out`, `scale`, `at`, `dry_run` |
| `import` | `path`, `source`, `name` |
| `diff` | `old`, `new`, `at` |

`path` e `source` são mutuamente exclusivos (ambos ou nenhum → exit 2). Com `--params`, o stdin só é
lido quando o valor do próprio flag é `-`; nunca como fonte do chart.

### 2.3 `--batch`

Aceito em `render` (criação de vários charts) e em `check` (validação em lote). `--batch` e
`--params` são mutuamente exclusivos (exit 2). Entrada = JSONL, uma linha por item; linhas vazias são
ignoradas. Cada linha é um objeto com as mesmas chaves de `--params` daquele verbo, mais `name`.
Processamento **sequencial na ordem do arquivo**; falha de um item não interrompe o lote.

```json
{"name":"retry","source":"chart v1 \"Retry\"\nnode a \"A\"\n","format":"svg"}
{"name":"arch","path":"charts/arch.akrc","format":"png","scale":2}
```

Regras do item:

- `name` obrigatório, `[A-Za-z0-9_-]{1,64}`; violação → item falha com `invalid_input`.
- `source` XOR `path` (ambos/nenhum → item falha).
- `format` default `svg` em `render`.
- `out` ausente em `render` → `<out-dir>/<name>.<ext>`, com `ext` = `svg`, `png`, `jpg`, `avif`,
  `json` (para `scene`) ou `excalidraw`. `--out-dir` default `.` e **precisa existir** (ausente →
  exit 5 antes de processar).
- `out` explícito sobrescreve a derivação.
- `--dry-run` valida e reporta sem escrever nada.

Saída de texto, uma linha por item, na ordem de entrada. Em `render`:
`<alvo>  ok  <bytes> B` ou `<alvo>  erro  <CODE> <message>`. Em `check`:
`<name>  ok  <n> aviso(s)` ou `<name>  erro  <CODE> <message>`, e o item não grava arquivo algum.

Exit code do lote: `0` se ao menos um item teve sucesso; `8` se todos falharam. Arquivo vazio →
exit 2 (`invalid_input`). `--json`:

```json
{"success":true,"command":"render",
 "data":{"ok":1,"failed":1,"items":[
   {"index":0,"name":"retry","ok":true,"out":"retry.svg","bytes":4210},
   {"index":1,"name":"arch","ok":false,"error":{"code":"schema","message":"..."}}]},
 "warnings":[]}
```

### 2.4 Formatos

| `--format` | extensão | destino | observação |
|---|---|---|---|
| `svg` (default) | `.svg` | stdout ou `--out` | único formato com bytes estáveis garantidos |
| `png` | `.png` | `--out` / `out` | `--scale` aplica (default `2.0`) |
| `jpg` | `.jpg` | `--out` / `out` | qualidade fixa `90`; fundo do tema é opaco |
| `avif` | `.avif` | `--out` / `out` | qualidade `80`; mais lento (rav1e) |
| `scene` | `.json` | stdout ou `--out` | JSON interno de depuração |
| `excalidraw` | `.excalidraw` | stdout ou `--out` | derivado determinístico |

Raster sai do mesmo SVG: `usvg` → `resvg` → `tiny_skia::Pixmap`. SVG e `scene` são contratos de
bytes entre versões. Raster é determinístico dentro de uma versão (threads de codificação pinadas em
1), mas não é contrato de bytes entre versões.

### 2.5 Saída e erros

Envelope `--json` (uma linha, `\n` final):

```json
{"success":true,"command":"render","data":{...},"warnings":[]}
{"success":false,"command":"check","error":{"code":"schema","message":"..."}}
```

`--json` em `render` devolve `data.path`, `data.format`, `data.bytes`, `data.width`, `data.height`.
`warnings` é sempre `[]` ou lista de strings `"<code>: <message>"`.

Exit codes (fonte única em `akribie-core::error::ErrorKind`):

| código | `ErrorKind` | uso |
|---|---|---|
| 0 | — | sucesso (inclui EPIPE em stdout) |
| 1 | — | `fmt --check` detectou diferença |
| 2 | `InvalidInput` | uso inválido: flag, formato, stdin ausente, lote vazio |
| 3 | `NotFound` | arquivo, revisão git ou `--out-dir` inexistente |
| 5 | `Io` | falha de leitura/escrita |
| 8 | `Schema` | `.akrc` inválido (≥1 diagnóstico `error`) ou lote 100% falho |
| 70 | `Internal` | invariante interna quebrada |

Configuração de projeto: `akribie.toml` é procurado subindo do diretório do arquivo `.akrc` até a
raiz; o primeiro encontrado vence; ausente → defaults embutidos. Chaves: `theme`, `layout`,
`direction`, `fontsize`, `pad`, `lang` (mesmos tipos/defaults da tabela `chart`). Opções do próprio
`chart` sobrepõem o arquivo.

---

## 3. Formato `.akrc`

UTF-8 sem BOM. **Uma afirmação por linha.** Indentação não tem significado. `\r` é removido na
leitura; `fmt` sempre emite LF. Comentário: `#` como primeiro caractere não-branco da linha.

### 3.1 Gramática

```
document   = { line } ;
line       = blank | comment | statement ;
blank      = { " " } newline ;
comment    = { " " } "#" { char } newline ;
statement  = stmt_chart | stmt_group | stmt_node | stmt_edge | stmt_note ;
stmt_chart = "chart" SP VERSION [ SP STRING ] { SP option } newline ;
stmt_group = "group" SP IDENT [ SP STRING ] { SP option } newline ;
stmt_node  = "node"  SP IDENT [ SP STRING ] { SP option } newline ;
stmt_edge  = "edge"  SP IDENT SP "->" SP IDENT [ SP STRING ] { SP option } newline ;
stmt_note  = "note"  SP STRING { SP option } newline ;
option     = KEY "=" value ;
value      = STRING | BAREWORD | PAIR ;
PAIR       = NUMBER "," NUMBER ;
```

Léxico:

- `IDENT`: `[A-Za-z][A-Za-z0-9_-]{0,63}`.
- `VERSION`: `v[0-9]+`, **não entre aspas**; o único valor aceito na v1 é `v1`.
- `KEY`: `[a-z][a-z0-9_]{0,31}`.
- `BAREWORD`: `[A-Za-z0-9_.-]+`.
- `NUMBER`: `-?[0-9]+(\.[0-9]+)?`.
- `STRING`: `"` … `"` com escapes `\"`, `\\`, `\n`, `\t`; nenhum outro escape é aceito.
- Separador entre tokens: um ou mais espaços (`0x20`). **Tab (`\t`) é erro `E007`**.

A primeira linha não-comentário e não-vazia **deve** ser `chart`; caso contrário `E008`. O token
seguinte deve ser `v1`; outro `VERSION` → `E009`. O título é opcional (STRING logo após a versão);
ausente → título vazio e o SVG omite `<title>`/`aria-label`.

### 3.2 Chaves por statement (conjunto fechado)

`chart`:

| chave | tipo | default | valores |
|---|---|---|---|
| `layout` | enum | `flow` | `flow tree mindmap grid stack free` |
| `direction` | enum | `lr` | `lr tb bt rl` |
| `theme` | enum | `paper` | `paper slate` |
| `fontsize` | number | `20` | 14–96 |
| `pad` | number | `32` | 0–256 |
| `lang` | bareword | `en` | BCP-47 simples (`en`, `pt-BR`, `es`) |
| `doc` | string | — | `path[#anchor]` |

`node`:

| chave | tipo | default | valores |
|---|---|---|---|
| `shape` | enum | `rounded` | `rect rounded circle diamond cloud cylinder hexagon note text` |
| `fill` | token | `none` | `none paper blue green amber red violet` |
| `fillstyle` | enum | `hachure` | `hachure solid cross-hatch zigzag` |
| `stroke` | token | `ink` | `none paper ink muted blue green amber red violet` |
| `style` | enum | `solid` | `solid dashed dotted` |
| `width` | number | `2` | 2–6 |
| `in` | idlist | — | ids de `group` declarados |
| `at` | pair | — | obrigatório quando `layout=free` |
| `size` | pair | — | ambos ≥ 1 |

`edge`: `curve` (`line|arc|elbow`, default `line`), `head` (`arrow|none`, default `arrow`),
`tail` (`arrow|none`, default `none`), `style`, `stroke`, `width` — mesmos tipos/defaults de `node`.

`group`: `fill` (default `none`), `stroke` (default `ink`), `style`, `width`, `pad` (default `24`,
0–256), `in` (idlist).

`note`: `fill` (default `amber`), `at`, `size`, `near` (IDENT).

`idlist` = `IDENT` ou `IDENT,IDENT,...` (vírgula sem espaços).

`chart.pad` é a margem externa do canvas (somada ao `bbox` do conteúdo para compor o `viewBox`).
`chart.fontsize` vale para rótulos de nó, rótulos de aresta e notas. `chart.lang` é emitido como
atributo `lang` no `<svg>`.

`fill` exclui `ink` e `muted` por decisão de contraste (ver §5.2); `stroke` aceita os oito tokens.

Chave desconhecida → `E002` com lista das chaves válidas daquele statement. Valor fora do conjunto
fechado → `E003` listando os valores válidos.

### 3.3 Semântica de resolução

1. `chart` define `layout`, `direction`, `theme`, `fontsize`, `pad`, `lang` e `doc`.
2. `group`, `node`, `note` têm id próprio; **ids são únicos** entre os três (`E004`).
3. `node` sem rótulo usa o próprio id como rótulo. `group` idem.
4. `edge FROM -> TO` referencia ids. Se o id não foi declarado, cria-se um nó implícito `node <id>`
   com aviso `W001`; os implícitos entram ao final da lista de nós, na ordem em que aparecem no
   arquivo. Se o id declarado for `group` ou `note`, → `E005` (aresta só liga nós).
5. `in=` referencia apenas `group`; id ausente → `E005`.
6. Múltiplas arestas com o mesmo par `(from,to)` são permitidas; a chave de identidade para `diff`
   é `(from,to,ordinal)`, com `ordinal` = ordem de declaração entre pares iguais (0-based).
7. `layout=free` exige `at` em todo `node` e `note` (`E006`). `at` em layout não-`free` é permitido e
   **fixa** aquele elemento (pinning parcial) — os demais seguem o layout.
8. `size` sobrepõe o tamanho calculado pela medição de texto.
9. `note near=ID` posiciona a nota à direita do nó, deslocada `24.0`; `near` exige id de `node`
   (`E005`).
10. `doc=` é metadado puro: não afeta layout nem SVG. Só `check --docs` o resolve.

### 3.4 Forma canônica (`fmt`)

- Linhas na ordem: `chart`, depois **todos os `group`**, **todos os `node`**, **todas as `edge`**,
  **todos os `note`**; dentro de cada classe, **ordem de declaração preservada** (sort estável).
- Exatamente uma linha em branco entre classes não-vazias; nenhuma em branco dentro de uma classe.
- Comentários viajam colados ao statement seguinte (sort estável carrega o par
  `[comentários…, statement]`); comentários sem statement seguinte ficam no fim do arquivo.
- Separador único: um espaço. Sem alinhamento de colunas (alinhamento reescreveria linhas inteiras
  a cada id novo).
- Rótulos e strings sempre entre aspas; ids nunca.
- Opções em ordem alfabética de chave.
- Números: até 2 casas decimais, zeros à direita removidos (`120`, `120.5`, `0.25`).
- Arquivo termina com exatamente um `\n`.

`fmt --check` sai 1 quando o arquivo difere; `fmt --write` grava; sem flags, imprime em stdout.

### 3.5 Diagnósticos

Texto: `<file>:<line>:<col>: <CODE> <name>: <message>` e, quando houver, uma segunda linha
`  hint: <hint>`. `--json` devolve `data.diagnostics[]` com
`{code, severity, line, col, message, hint?}`.

| código | severidade | condição |
|---|---|---|
| `E001` | error | keyword desconhecida |
| `E002` | error | chave desconhecida no statement |
| `E003` | error | valor fora do conjunto fechado / tipo inválido |
| `E004` | error | id duplicado |
| `E005` | error | referência ausente (`in=`, `near=`) |
| `E006` | error | `layout=free` sem `at` |
| `E007` | error | linha malformada (tab, aspas não fechadas, `->` ausente) |
| `E008` | error | primeira afirmação não é `chart` |
| `E009` | error | versão diferente de `v1` |
| `W001` | warn | nó implícito criado por `edge` |
| `W002` | warn | nó sem arestas em `layout=flow|tree|mindmap` |
| `W003` | warn | codepoint sem glifo na fonte embutida |
| `W004` | warn | `doc=` ausente (arquivo ou âncora) — só com `--docs` |
| `W005` | warn | cor do `.excalidraw` sem token equivalente (import) |
| `W006` | warn | elemento não representável foi descartado (import) |
| `W007` | warn | contraste abaixo do mínimo em §5.2 (stroke×fundo, stroke×fill ou texto×fill) |

`check` sem `--strict` sai 0 quando só há warnings. `--strict` promove warning a erro (exit 8).

---

## 4. Acessibilidade

Vale para a saída SVG e para o uso do binário. As regras abaixo são normativas e testadas.

### 4.1 Contrato do SVG

```svg
<svg xmlns="http://www.w3.org/2000/svg" width="800" height="480" viewBox="0 0 800 480"
     role="img" aria-labelledby="akr-t akr-d" lang="pt-BR" focusable="false">
  <title id="akr-t">Política de retry</title>
  <desc id="akr-d">3 nós, 3 arestas. Nós: API Gateway; Retry ×3; DLQ.
Arestas: API Gateway → Retry ×3 (5xx); Retry ×3 → API Gateway (backoff);
Retry ×3 → DLQ (esgotado).</desc>
  <rect .../>                       <!-- fundo do tema -->
  <g id="groups">…</g>
  <g id="edges">…</g>
  <g id="nodes" role="list">
    <g data-id="api" data-kind="node" role="listitem" aria-label="API Gateway">…</g>
  </g>
  <g id="notes">…</g>
</svg>
```

Regras:

1. `role="img"` + `aria-labelledby="akr-t akr-d"` na raiz; `<title id="akr-t">` e
   `<desc id="akr-d">` são os **primeiros** filhos do `<svg>`.
2. `<desc>` é a alternativa textual completa: contagem de nós/arestas, lista de rótulos de nó e
   lista de arestas no formato `<from> → <to> (<label>)`, separadas por `; `. Sem rótulo de aresta,
   omite o parêntese. Texto derivado do modelo, não do desenho.
3. `lang` na raiz, vindo de `chart.lang`.
4. Cada elemento desenhado é um `<g>` com `data-id`, `data-kind` (`group|edge|node|note`) e
   `aria-label` igual ao rótulo (vazio quando não há rótulo — nesse caso `aria-label` é omitido).
5. `id` do SVG usam o prefixo `akr-` para não colidir quando o arquivo é embutido em HTML.
6. `focusable="false"` (evita parada de tab no IE/Edge legado; inofensivo nos demais).
7. Nenhuma informação é transmitida só por cor: arestas têm rótulo textual e o `<desc>` lista as
   relações.

### 4.2 Contraste e legibilidade

- Texto sempre em `theme.ink` sobre o `fill` do elemento. Mínimo **4.5:1** (WCAG 2.2 AA, texto
  normal).
- Contorno (`stroke`) contra o fundo do canvas e contra o `fill` do próprio elemento: mínimo
  **3:1** (WCAG 2.2 SC 1.4.11, conteúdo não textual).
- `width` de contorno: 2–6, default 2 — nunca abaixo de 2 px.
- `fontsize`: 14–96, default 20.
- Dimensões mínimas: `MIN_NODE_W = 96.0`, `MIN_NODE_H = 44.0`, `NODE_PADDING = 12.0`,
  `LINE_HEIGHT = 1.35`, `MAX_LABEL_W = 320.0`.
- Espaçamento: `nodesep = 48.0`, `ranksep = 64.0`, `GAP_GRID = 40.0`, `GAP_STACK = 28.0`,
  `GROUP_PAD = 24.0`, `CANVAS_PAD = 32.0`.
- As paletas de §6.4 são validadas por teste: para toda combinação `(fill, ink)` do tema, o
  contraste é ≥ 4.5:1; para todo `(stroke, canvas)` e `(stroke, fill)`, ≥ 3:1. `check` emite
  `W007` quando uma combinação explícita do usuário viola o mínimo.

### 4.3 Uso acessível pelo agente

- Toda mensagem de diagnóstico é auto-suficiente: código, posição, o que se esperava e a correção.
  Nunca "syntax error".
- Nenhum comando depende de cor no terminal; nenhum depende de largura de terminal.
- `akr prompt` documenta a regra de correção: em caso de `E`, reescrever a linha indicada e reenviar.

---

## 5. Arquitetura

Workspace Rust, edition 2024, `rust-version = "1.99"`, `resolver = "3"`.

```
crates/
  akribie-core/    # lib pura: parse, resolve, layout, paint, emit, diff, ports  (publish = false)
  akribie-cli/     # bin `akr` (sem [lib])
  akribie-mcp/     # bin `akribie-mcp` (JSON-RPC 2.0 sobre stdio)
xtask/             # utilitário de build: converte a fonte woff2 → ttf
assets/            # Excalifont-Regular.ttf + OFL.txt + FONT.lock
docs/              # ADRs, plano em fases, protocolo de revisão, specs, guias
```

### 5.1 Dependências (versões exatas; nada além disto)

| crate | versão | onde | papel |
|---|---|---|---|
| `clap` | `4.6`, feature `derive` | cli | superfície |
| `serde` | `1`, `derive` | core, cli, mcp | scene/excalidraw/envelope |
| `serde_json` | `1` | core, cli, mcp | idem |
| `indexmap` | `2` | core | ordem determinística |
| `unicode-segmentation` | `1.13` | core | quebra de grafema |
| `harfrust` | `0.15` | core | shaping para medição |
| `dugong` | `=0.9.3` | core | layout em camadas (dagre) |
| `resvg` | `0.49` | core (feature `raster`) | SVG→bitmap |
| `usvg` | `0.49` | core (feature `raster`) | parse do SVG + fontdb |
| `tiny-skia` | `0.12`, feature `png-format` | core (feature `raster`) | bitmap + PNG |
| `image` | `0.25`, `default-features = false`, features `jpeg`, `avif` | core (feature `raster`) | JPG/AVIF |
| `woff2` | `0.3` | xtask | woff2→ttf |

Feature `raster` é default-on em `akribie-core` e `akribie-cli`; desligá-la deixa só SVG/scene/
excalidraw e remove a cadeia resvg/rav1e.

### 5.2 Módulos de `akribie-core/src`

| arquivo | responsabilidade |
|---|---|
| `lib.rs` | fachada: `parse`, `resolve`, `layout`, `paint`, `svg`, `raster`, `scene_json`, `excalidraw`, `from_excalidraw`, `diff`, `run_render` |
| `error.rs` | `Error`, `ErrorKind` (exit codes de §2.5) |
| `span.rs` | `Span { line: u32, col: u32 }` (1-based, `col` em caracteres) |
| `lex.rs` | tokeniza uma linha em `Token { kind, span }` |
| `syntax.rs` | `Statement`, `OptKey`, `OptVal`, `StmtKind` e as **tabelas de opções** (`option_specs`) |
| `parse.rs` | linhas → `Document` + `Vec<Diagnostic>` |
| `fmt.rs` | `Document` → forma canônica |
| `model.rs` | `Chart`, `Node`, `Edge`, `Group`, `Note`, `IdList` |
| `resolve.rs` | `Document` + `Config` → `Chart` |
| `config.rs` | `Config` (defaults + `akribie.toml`) |
| `theme.rs` | `Theme`, `Token`, paletas `paper`/`slate`, `token_of_hex`, `contrast_ratio` |
| `a11y.rs` | geração do `<desc>` textual, `check_contrast(chart, theme) -> Vec<Diagnostic>` |
| `text.rs` | `Font::load`, `advance_px`, `wrap` |
| `layout/mod.rs` | `layout(&Chart, &Font) -> Placed` + dispatch por `layout` |
| `layout/flow.rs`, `tree.rs`, `radial.rs`, `grid.rs`, `free.rs` | perfis |
| `geometry.rs` | `Rect`, `Point`, `outline_hit`, `route_line`, `route_arc`, `route_elbow` |
| `paint.rs` | `paint(&Placed, &Theme, &Font) -> Scene` |
| `sketch.rs` | traço à mão determinístico (RNG próprio, seed do id) |
| `scene.rs` | `Placed` (geometria) e `Scene` (pronto para emitir) |
| `emit/svg.rs` | `Scene` → SVG acessível |
| `emit/raster.rs` | SVG → PNG/JPG/AVIF |
| `emit/scene_json.rs` | `Scene` → JSON |
| `emit/excalidraw.rs` | `Scene` → `.excalidraw` |
| `import/excalidraw.rs` | `.excalidraw` → `Document` |
| `diff.rs` | dois `Chart` → lista de mudanças |
| `doc.rs` | resolução de `doc=` |
| `ports.rs` | traits `Fs`, `Git`, `Env`, `Logger` + `ports::fakes` |
| `pipeline.rs` | `run_render`, `run_check`, `run_batch` recebendo `&dyn Fs`/`&dyn Git` |

`ports` (mesmo contrato de `_REF/knudge/crates/knudge-core/src/ports/mod.rs`):

| porta | métodos mínimos | fake |
|---|---|---|
| `Fs` | `read(path) -> Result<String>`, `read_bytes`, `write(path, &[u8])`, `exists(path)` | `MemFs` |
| `Git` | `show(rev, path) -> Result<String>` | `FakeGit` |
| `Env` | `var(key) -> Option<String>` | `FakeEnv` |
| `Logger` | `warn(&str)`, `info(&str)` | `RecordingLogger` |

A CLI é a única que monta `StdFs`/`StdGit`/`StdEnv`/`TracingLogger`. Nenhum módulo de domínio
importa `std::fs`, `std::env` ou `std::process::Command`.

### 5.3 Regras de rigor

- `#![forbid(unsafe_code)]`; `#![warn(missing_docs)]`.
- Proibidos no domínio: `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `dbg!`,
  `print_stdout`, `print_stderr`, `process::exit`, `indexing_slicing`, `as_conversions`,
  `wildcard_imports`, `HashMap`/`HashSet`/`Rc`/`RefCell`, `SystemTime::now`, `env::var`,
  `LinkedList`. Garantido por `[workspace.lints]` + `clippy.toml` (mesmo formato de
  `_REF/knudge/Cargo.toml` §`[workspace.lints]` e `_REF/knudge/clippy.toml`).
- Todo `#[allow]` exige `reason = "..."`.
- Arquivos de produção ≤ 300 linhas (`scripts/check_file_length.sh`).
- Aritmética com `checked_*`/`saturating_*`; `overflow-checks = true` em dev e release.

### 5.4 Árvore documental

```
README.md                     # o que é, instalação, exemplo de 5 linhas
AGENTS.md                     # guia de contribuição + regras de ouro
llms.txt                      # índice denso para LLM
SKILL.md                      # guia acionável (frontmatter name/description/triggers)
CHANGELOG.md                  # Keep a Changelog, pt-BR
docs/
  adr/ADR-0001-simplicidade-determinismo-di.md
  adr/ADR-0002-fonte-texto-canonica.md
  adr/ADR-0003-layout-por-perfil.md
  adr/ADR-0004-excalidraw-derivado.md
  adr/ADR-0005-acessibilidade-svg.md
  plan/00-panorama.md
  plan/01-fases.md
  plan/fase-M0-esqueleto.md … fase-M8-integracao.md   # um arquivo por fase
  plan/02-matriz-aceitacao.md
  review-protocol.md
  specs/GRAMMAR.md
  specs/ARCHITECTURE.md
  specs/A11Y.md
  specs/DETERMINISM.md
  usage/00-quickstart.md … usage/07-schema.md          # um arquivo por verbo
crates/*/MODULE.md
```

`docs/plan/02-matriz-aceitacao.md` é a definição de pronto, linha a linha:
`verbo | pipe | --json | erro+exit | estado/arquivo escrito | teste`. `docs/review-protocol.md`
define o ritual de revisão: toda mudança de superfície atualiza, no mesmo commit, `usage/`, a
matriz, `AGENTS.md`/`SKILL.md`/`llms.txt` quando aplicável, e o golden correspondente; revisão
verifica paridade entre `akr prompt` (vivo) e `docs/` (estático).

### 5.5 Fonte embutida

`assets/Excalifont-Regular.ttf` é **commitado**. É produzido uma vez por
`cargo run -p xtask -- font build`, que baixa
`https://excalidraw.nyc3.cdn.digitaloceanspaces.com/fonts/Excalifont-Regular.woff2`
(OFL-1.1, confirmado em `_REF/excalidraw/packages/excalidraw/fonts/Excalifont/index.ts:26`),
verifica o SHA-256 de `assets/FONT.lock` e chama
`woff2::decode::convert_woff2_to_ttf(&mut Cursor::new(bytes))`. O `.ttf` e
`assets/EXCALIFONT-OFL.txt` são gravados em `assets/`.

`akribie-core` embute via `include_bytes!("../../assets/Excalifont-Regular.ttf")`. Medição, SVG e
raster usam **a mesma face** — nunca fonte do sistema.

### 5.6 Paletas (constantes de `theme.rs`)

`paper` — canvas `#ffffff`, ink `#1e1e1e`, muted `#5c5c5c`, blue `#a5d8ff`, green `#b2f2bb`,
amber `#ffec99`, red `#ffc9c9`, violet `#d0bfff`.

`slate` — canvas `#121212`, ink `#e8e8e8`, muted `#b4b4b4`, blue `#1e3a5f`, green `#1e4620`,
amber `#5c4813`, red `#5f1e1e`, violet `#3c2f6b`.

Os valores são fechados: o teste de contraste de §4.2 roda sobre esta tabela.

---

## 6. Etapas

Ordem de construção; cada etapa deixa a árvore compilando e `make check` verde. As fases também são
documentadas em `docs/plan/fase-M*-*.md`, escritas junto com o código da fase.

### M0 — Esqueleto, governança e asset de fonte

1. `Cargo.toml` de workspace: `members = ["crates/akribie-core","crates/akribie-cli","crates/akribie-mcp"]`,
   `default-members = ["crates/akribie-cli"]`, `exclude = ["xtask"]`; `[workspace.package]`
   `version = "0.1.0"`, `edition = "2024"`, `rust-version = "1.99"`,
   `license = "MIT OR Apache-2.0"`, `publish = false`; `[profile.release] lto = "fat"`,
   `codegen-units = 1`, `strip = "symbols"`, `panic = "abort"`, `overflow-checks = true`;
   `[profile.dev] overflow-checks = true`.
2. `rust-toolchain.toml` (`channel = "1.99"`), `rustfmt.toml` (`max_width = 100`, `tab_spaces = 4`,
   `newline_style = "Unix"`, `hex_literal_case = "Lower"`), `clippy.toml`, `deny.toml`,
   `typos.toml`, `.gitignore` (`/target`, `/fixtures/out`).
3. `[workspace.lints]` conforme §5.3.
4. `docs/adr/ADR-0001-simplicidade-determinismo-di.md` com as cinco decisões de §1 e as referências
   cruzadas ao knudge.
5. `docs/adr/ADR-0002` a `ADR-0005` com status `Accepted`, uma página cada: fonte texto canônica,
   layout por perfil, Excalidraw derivado, acessibilidade SVG.
6. `docs/review-protocol.md` (ritual de §5.4) e `docs/plan/00-panorama.md`,
   `docs/plan/01-fases.md`, `docs/plan/02-matriz-aceitacao.md` (tabela vazia por verbo).
7. `xtask/src/main.rs` com `font build`: se `assets/Excalifont-Regular.woff2` não existe, baixa a
   URL oficial; calcula SHA-256 e compara com `assets/FONT.lock` (`<hex>  Excalifont-Regular.woff2`).
   Na primeira execução, sem hash registrado, grava o valor obtido e passa a ser o pino. Converte e
   grava `assets/Excalifont-Regular.ttf`; não regrava quando o `.ttf` já corresponde ao pino.
8. `scripts/check_file_length.sh` (falha acima de 300 linhas em `crates/*/src/**/*.rs`).
9. `Makefile` com `check: fmt clippy test file-length`, `ci: check deny typos`, `build`, `install`.
10. `.github/workflows/ci.yml` rodando `make check` com `RUSTFLAGS=-D warnings`.
11. `README.md`, `AGENTS.md`, `llms.txt`, `SKILL.md`, `CHANGELOG.md` iniciais (v0.1.0, seção
    `Não publicado`).

Aceite: `make check` verde no workspace vazio; `cargo run -p xtask -- font build` idempotente;
`ls assets/Excalifont-Regular.ttf` existe e `docs/adr/ADR-0001-*.md` contém as cinco decisões.

### M1 — Léxico, AST, parser e `check`

1. `span.rs`: `pub struct Span { pub line: u32, pub col: u32 }` (1-based).
2. `error.rs`: `#[non_exhaustive] pub enum ErrorKind { InvalidInput, NotFound, Io, Schema, Internal }`
   com `pub const fn exit_code(self) -> u8` e `pub const fn code(self) -> &'static str`
   (`"invalid_input"`, `"not_found"`, `"io"`, `"schema"`, `"internal"`).
3. `syntax.rs`: `pub const fn option_specs(kind: StmtKind) -> &'static [OptSpec]` com
   `OptSpec { key: &'static str, kind: ValKind, default: &'static str, allowed: &'static [&'static str] }`
   — as tabelas de §3.2 são a **única** fonte; `schema`, `prompt` e o parser leem daqui.
4. `lex.rs`: `pub fn lex(line: &str, span: Span) -> Result<Vec<Token>, Diagnostic>` com
   `TokKind::{Ident, Version, Str, Key, Bare, Arrow, Pair}`; tab → `E007`.
5. `parse.rs`: `pub fn parse(src: &str) -> (Document, Vec<Diagnostic>)` com
   `Document { chart: Option<ChartStmt>, groups, nodes, edges, notes, trivia }`; `Trivia` guarda
   comentários/linhas em branco com o índice do statement seguinte. Recuperação **por linha**:
   uma linha inválida produz um diagnóstico e não invalida as demais.
6. `ports.rs`: traits de §5.2 + `MemFs`, `FakeGit`, `FakeEnv`, `RecordingLogger`.
7. `akribie-cli`: `main.rs` (clap), `envelope.rs` (`{success, command, data?, error?, warnings?}`
   com `skip_serializing_if`), `output.rs` (`BrokenPipe` → exit 0), `commands/check.rs`,
   `commands/schema.rs`.
8. `akr schema`: JSON gerado de `syntax.rs`, `theme.rs` e da tabela de diagnósticos — statements,
   chaves, tipos, defaults, valores fechados, tokens→hex, códigos, exit codes.
9. Implementar `--params` genérico no `main.rs`: um `Params` deserializado por verbo, com a lista de
   chaves de §2.2; conflito com posicional → exit 2.
10. `docs/plan/fase-M1-*.md` e linhas correspondentes da matriz.

Aceite: `akr check` num `.akrc` com duas linhas trocadas reporta `E008` na linha 1; `akr schema`
lista `layout` com os 6 valores; `printf 'chart v1 "x"\nnode a\n' | akr check -` sai 0;
`akr render --params '{"source":"chart v1 \"x\"\n"}' --format svg` funciona e
`akr render --params '{"format":"svg"}' x.akrc` sai 2.

### M2 — `fmt` canônico

1. `fmt.rs`: `pub fn format(doc: &Document) -> String` conforme §3.4, com transporte de comentários.
2. `commands/fmt.rs`: sem flags → stdout; `--write`; `--check` (exit 1 se diferir).
3. Golden `crates/akribie-cli/tests/golden/fmt_canonical.akrc` (entrada desordenada) e
   `fmt_expected.akrc`.

Aceite: `akr fmt x.akrc | akr fmt -` idempotente byte-a-byte; `--check` no golden canônico sai 0.

### M3 — Modelo, temas, contraste e medição de texto

1. `theme.rs`: `pub enum Token { None, Paper, Ink, Muted, Blue, Green, Amber, Red, Violet }` com
   `pub const fn token(s: &str) -> Option<Token>`, `pub const fn token_of_hex(hex: &str) -> Option<Token>`
   e `pub fn contrast_ratio(a: &str, b: &str) -> f32` (WCAG 2.2, luminância relativa).
   `pub struct Theme { pub canvas: &'static str, pub ink: &'static str, pub palette: [(&'static str, &'static str); 8] }`
   com os valores de §5.6.
2. Teste `theme::tests::all_combinations_meet_wcag`: para cada tema, `contrast(ink, fill) ≥ 4.5`
   para `fill` em `{paper, blue, green, amber, red, violet}`; `contrast(stroke, canvas) ≥ 3.0` e
   `contrast(stroke, fill) ≥ 3.0` para os oito tokens de stroke.
3. `resolve.rs`: `pub fn resolve(doc: &Document, cfg: &Config) -> (Chart, Vec<Diagnostic>)` com as
   regras de §3.3; emite `E004`, `E005`, `E006`, `W001`.
4. `config.rs`: `Config { layout, direction, theme, fontsize, pad, lang }` com defaults
   `flow/lr/paper/20/32/en`.
5. `a11y.rs`: `pub fn describe(chart: &Chart) -> String` (o texto do `<desc>`), e
   `pub fn check_contrast(chart: &Chart, theme: &Theme) -> Vec<Diagnostic>` (`W007`).
6. `text.rs`: `Font::load()` sobre `include_bytes!`; `advance_px(&self, text, size) -> f32` via
   `harfrust::shape`; `wrap(&self, text, size, max_px) -> Vec<String>` (split `\n`, quebra gulosa,
   grafemas para palavras longas). Constantes de §4.2; `W003` para `glyph_id == 0`.
7. `commands/check.rs` passa a rodar resolve + medição + `check_contrast` e a emitir `W002`, `W003`,
   `W007`.

Aceite: `node a "API Gateway"` mede `w = advance("API Gateway") + 24` e `h = 27 + 24`;
`printf 'chart v1 "x"\nnode a "A" fill=blue stroke=blue\n' | akr check -` emite `W007`;
`akr check --strict` do mesmo arquivo sai 8.

### M4 — Layout

1. `scene.rs`: `pub struct Placed { width, height, groups: Vec<PlacedGroup>, edges: Vec<PlacedEdge>, nodes: Vec<PlacedNode>, notes: Vec<PlacedNote> }`
   — geometria apenas (`Rect`, `Vec<Point>` da rota, `shape`, `label: Vec<String>`, `style`).
   `pub struct Scene { width, height, groups: Vec<GroupDraw>, edges: Vec<EdgeDraw>, nodes: Vec<NodeDraw>, notes: Vec<NoteDraw> }`
   — cada `*Draw` carrega `id`, `kind`, `aria_label: Option<String>`, `d: String`,
   `fill_d: Option<String>`, `clip: Option<String>`, `text: Vec<TextRun>`, `head: Option<String>`.
2. `layout/mod.rs`: `pub fn layout(chart: &Chart, font: &Font) -> Placed` — dimensiona cada nó
   (texto ou `size`), delega ao perfil, calcula grupos (`bbox` + `pad`) e notas (`near`/`at`).
3. `layout/flow.rs`: monta `dugong::graphlib::Graph` com `NodeLabel { width, height, .. }` e
   `GraphLabel { rankdir: RankDir::{LR,TB,BT,RL}, nodesep: 48.0, ranksep: 64.0, marginx: 40.0, marginy: 40.0, .. }`,
   chama `dugong::layout_controlled(&mut g, &mut dugong::NoopWorkControl)` e lê `NodeLabel { x, y }`
   (centro) → `Rect`. **Roteamento de aresta é nosso**, não do dagre. `W002` para nó isolado.
4. `layout/tree.rs`: Reingold–Tilford; raízes = nós sem aresta de entrada (ordem de declaração),
   filhos na ordem de declaração; `H_GAP = 40.0`, `V_GAP = 80.0`.
5. `layout/radial.rs`: raiz = primeiro nó sem aresta de entrada; anéis com `RADIUS_STEP = 180.0`;
   ângulo do filho proporcional às folhas da subárvore. `layout=mindmap` ignora `direction`.
6. `layout/grid.rs`: `columns = ceil(sqrt(n))`, célula = maior nó, `GAP_GRID = 40.0`.
   `run_stack` no mesmo arquivo: uma coluna, `GAP_STACK = 28.0`.
7. `layout/free.rs`: usa `at`/`size`; normaliza o `bbox` global.
8. `geometry.rs`: `Rect { x, y, w, h }` com `center`/`union`; `outline_hit(rect, shape, from, to)`
   para rect/circle/diamond (demais usam o retângulo); `route_line`, `route_arc` (quadrática com
   controle a 15% da distância na perpendicular), `route_elbow` (H-V-H quando `|dx| ≥ |dy|`, senão
   V-H-V, metade do caminho no segmento central); self-edge = arco de raio 32 no lado direito.

Aceite: `flow` com `a->b->c` em `direction=lr` dá `a.x < b.x < c.x` e `a.y == c.y`; `free` sem `at`
→ `E006`; `grid` com 4 nós dá 2×2.

### M5 — Pintura, traço à mão, SVG acessível e raster

1. `sketch.rs`: RNG `mulberry32(seed: u32)`; `pub fn seed_for(id: &str) -> u32` = FNV-1a 32 do id.
   Geradores `line`, `rect`, `circle` (4 cúbicas com controles perturbados), `polygon`,
   `path_from_points`; cada um devolve `d` pronto. `MAX_OFFSET = 2.0`, duas passagens por traço.
2. `paint.rs`: `pub fn paint(placed: &Placed, theme: &Theme, font: &Font) -> Scene`. Regras:
   - `fill != none`: hachure (45°, passo `8.0 + width * 2.0`), `cross-hatch` (±45°), `zigzag`
     (amplitude `4.0`), `solid` (`fill` direto). Recorte por `<clipPath>` com o próprio `d` do
     shape; `*Draw.clip = Some(d)` e `emit/svg.rs` emite
     `<defs><clipPath id="c-<id>"><path d="…"/></clipPath></defs>` antes do corpo.
   - contorno: `stroke`, `stroke-width`, `stroke-dasharray` = `8 8` (dashed) ou `1.5 6` (dotted).
   - formas: `rect`, `rounded` (raio `12.0`), `circle`, `diamond`, `hexagon`, `cylinder`
     (elipse + corpo + elipse inferior), `cloud` (8 arcos fixos escalados ao `Rect`), `note`
     (rect com canto dobrado de `16.0`), `text` (sem contorno).
   - texto: uma `<text>` por linha, `text-anchor="middle"`, `dominant-baseline="middle"`,
     `font-size` do chart, `font-family="Excalifont, Virgil, Comic Shanns, cursive"`.
   - setas: `d` da rota + cabeça como `<path>` preenchido (`HEAD_LEN = 12.0`, `HEAD_W = 8.0`);
     rótulo de aresta no ponto médio, com fundo `canvas` e padding `4.0`.
   - `aria_label`: rótulo do elemento, `None` quando vazio.
3. `emit/svg.rs`: `pub fn svg(scene: &Scene, theme: &Theme, title: &str, lang: &str, desc: &str) -> String`
   conforme §4.1: `<title id="akr-t">`, `<desc id="akr-d">`, `role="img"`,
   `aria-labelledby="akr-t akr-d"`, `lang`, `focusable="false"`; grupos `id="groups"`, `id="edges"`,
   `id="nodes"`, `id="notes"`, nesta ordem; cada elemento `<g data-id data-kind aria-label?>`.
   `fn num(v: f32) -> String` arredonda a 2 casas e remove zeros — usado em **todo** número.
   `escape(text)` para `& < > " '`.
4. `emit/raster.rs`: `pub fn encode(svg: &str, format: RasterFormat, scale: f32) -> Result<Vec<u8>, Error>`
   com `usvg::Options::default()`, `opts.fontdb_mut().load_font_data(FONT_BYTES.to_vec())`,
   `opts.font_family = "Excalifont".into()`, `usvg::Tree::from_str`, `resvg::render(&tree,
   tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut())`, depois:
   - PNG: `pixmap.encode_png()`.
   - JPG: `image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 90)` +
     `write_image(rgb, w, h, ExtendedColorType::Rgb8)`.
   - AVIF: `image::codecs::avif::AvifEncoder::new_with_speed_quality(&mut out, 6, 80)`
     `.with_num_threads(Some(1))` + `write_image(...)`.
   `rgb` = `Pixmap::data()` com o alfa descartado (o canvas é sempre opaco). Pinar threads em 1 é o
   que mantém o AVIF byte-estável.
5. `commands/render.rs`: `--format`, `--out`, `--scale` (default `2.0`), `--json`.
6. Teste de determinismo: `svg()` chamado duas vezes sobre o mesmo `Scene` é byte-idêntico.

Aceite: dois renders do mesmo `.akrc` produzem SVG byte-idêntico; o SVG contém
`role="img"` + `<title id="akr-t">` + `<desc id="akr-d">` e passou por `xmllint --noout`;
`--format png --out /tmp/a.png --scale 2` gera PNG com o dobro da largura do `viewBox`;
`--format jpg` e `--format avif` produzem arquivos que `file` identifica como JPEG e AVIF.

### M6 — Scene JSON e `diff`

1. `emit/scene_json.rs`: `{"schema_version":1,"width":..,"height":..,"groups":[],"edges":[],"nodes":[],"notes":[]}`
   com `rect: [x,y,w,h]`, `path` em `d` do SVG, `shape`, `fill`, `stroke`, `lines`, `aria_label`.
2. `diff.rs`: `pub fn diff(old: &Chart, new: &Chart) -> Vec<Change>` com
   `Change { kind, id, field, from, to }`; casamento por id para `group`/`node`/`note` e por
   `(from,to,ordinal)` para `edge`.
3. `commands/diff.rs`: `+ node dlq "DLQ"` / `- node legacy` /
   `~ node retry label: "Retry x3" -> "Retry x5"` / `~ edge api->retry style: solid -> dashed`;
   `--json` com `{added, removed, changed}`; `--at REV` via porta `Git`.

Aceite: adicionar um nó, remover outro e mudar `style` de uma aresta produz exatamente três linhas,
na ordem `added`, `removed`, `changed`, ordenadas por id.

### M7 — Lote, `.excalidraw` e `doc=`

1. `pipeline.rs`: `pub fn run_batch(items: &[RenderItem], fs: &dyn Fs, git: &dyn Git) -> BatchOutcome`
   com as regras de §2.3: ordem do arquivo, best-effort, derivação de `out` por `name`+`ext`,
   `--dry-run`, contagem `ok`/`failed`.
2. `commands/render.rs`: `--batch`, `--out-dir`, `--dry-run`; saída de texto por item e `--json` de
   §2.3. `commands/check.rs`: `--batch` usando o mesmo driver com saída `diagnostics` por item.
3. `emit/excalidraw.rs`: `pub fn excalidraw(scene: &Scene, theme: &Theme) -> String`.
   - Topo: `{"type":"excalidraw","version":2,"source":"akribie/<CARGO_PKG_VERSION>","elements":[...],
     "appState":{"gridSize":20,"gridStep":5,"gridModeEnabled":false,
     "viewBackgroundColor":"<theme.canvas>","lockedMultiSelections":{}},"files":{}}`.
   - Ordem: `group`s (como `rectangle`), `edge`s, `node`s, `note`s; cada nó é seguido imediatamente
     pelo seu elemento `text`.
   - Campos comuns: `id` = `"akr-" + id`, `x/y/width/height`, `angle: 0`, `strokeColor`,
     `backgroundColor` (`"transparent"` se `none`), `fillStyle`, `strokeWidth`, `strokeStyle`,
     `roughness: 1`, `opacity: 100`, `seed` = `seed_for(id) & 0x7fff_ffff`, `version: 1`,
     `versionNonce: 0`, `isDeleted: false`, `groupIds: []`, `frameId: null`, `boundElements`,
     `updated: 0`, `created: 0`, `link: null`, `locked: false`, `index`.
   - `index`: `format!("a{:0>4}V", base62(i))` com alfabeto `0-9A-Za-z`; largura fixa ⇒ ordem
     lexicográfica = numérica; o sufixo `V` evita terminar em `0`, que `validateOrderKey` rejeita.
   - `text`: `fontSize` do chart, `fontFamily: 5` (Excalifont), `text` (linhas unidas por `\n`),
     `originalText` idêntico, `textAlign:"center"`, `verticalAlign:"middle"`,
     `containerId:"akr-<id>"`, `autoResize: true`, `lineHeight: 1.35`, `angle: 0`.
   - `arrow`: `points` relativos a `x,y` com o primeiro `[0,0]`; `startBinding`/`endBinding` =
     `{elementId, fixedPoint:[fx,fy], mode:"orbit"}` com `fx,fy` = razão do ponto de ancoragem no
     bbox do alvo, `0.5` substituído por `0.5001`; `startArrowhead: null`, `endArrowhead: "arrow"`,
     `elbowed: false`.
4. `import/excalidraw.rs`: `pub fn from_excalidraw(json: &str, name: &str) -> (Document, Vec<Diagnostic>)`.
   `rectangle`/`ellipse`/`diamond` → `node` com `at`/`size` (ellipse → `circle`); `text` com
   `containerId` vira o rótulo do container; `text` solto → `note`; `arrow` com ambos os bindings →
   `edge` (sem bindings → `W006`); demais tipos → `W006`. `backgroundColor` mapeado por
   `token_of_hex`; sem correspondência → `fill=none` + `W005`. Resultado: `chart ... layout=free`,
   título = `name`. **Roteamento de aresta é recalculado**, não importado.
5. `commands/import.rs`: lê `.excalidraw`, passa por `from_excalidraw` e `fmt::format`, escreve
   `.akrc` em stdout; `--json` devolve `{nodes,edges,notes,warnings}`; JSON inválido → `E007` + exit 8.
6. `doc.rs`: `pub fn check(chart: &Chart, base: &Path, fs: &dyn Fs) -> Vec<Diagnostic>` — resolve
   `doc=path#anchor` relativo ao arquivo; arquivo ou âncora ausente → `W004`.
7. `commands/render.rs` e `commands/diff.rs`: `--at REV` pela porta `Git`.

Aceite: `akr render --batch lote.jsonl` com 3 itens (2 válidos, 1 com `source` e `path` juntos) sai 0,
escreve 2 arquivos e reporta 1 erro; `jq -e '.type=="excalidraw"'` aceita o export e
`jq -r '[.elements[].index] | . == (sort)'` devolve `true`; `akr import` do mesmo arquivo gera
`.akrc` com `layout=free` que `akr check` aceita sem `E`.

### M8 — `init`, `prompt`, MCP e documentação final

1. `commands/init.rs`: cria `akribie.toml` com `theme = "paper"`, `layout = "flow"`, `fontsize = 20`,
   `pad = 32`, `lang = "en"` (não sobrescreve sem `--force`); insere bloco
   `<!-- akribie:start -->` / `<!-- akribie:end -->` em `AGENTS.md` (idempotente, marcador em linha
   inteira); instala `.agents/skills/akr/SKILL.md` com `<!-- akribie:skill:version: 1 -->`, nunca
   sobrescrevendo arquivo sem o marcador. Saída lista `novo`/`atualizado`/`inalterado` e termina em
   `próximos:`.
2. `commands/prompt.rs`: `PROMPT_COMPACT` e `PROMPT_FULL` como `const &str` em inglês, byte-idênticos
   por versão. Compacto: gramática em 8 linhas, conjuntos fechados, 2 exemplos, regra de correção de
   diagnóstico (reescrever a linha apontada), regra de acessibilidade (título sempre presente,
   contraste garantido pelos tokens), exit codes. `--long` anexa BNF completa + tabelas de chaves +
   tokens de tema.
3. `akribie-mcp`: JSON-RPC 2.0 sobre stdio, uma mensagem por linha, EOF/EPIPE → exit 0. Tools:
   `chart_check {source|path, strict?}`, `chart_render {source|path, format, scale?}`,
   `chart_diff {old_source, new_source}`. Erros como `isError: true` com `content[0].text` = primeira
   linha de diagnóstico; nunca derrubam o servidor.
4. `docs/usage/00-quickstart.md` … `07-schema.md` (um por verbo) com exemplos executáveis;
   `docs/specs/GRAMMAR.md` (contrato de bytes), `ARCHITECTURE.md`, `A11Y.md`, `DETERMINISM.md`;
   `crates/*/MODULE.md`; `docs/plan/fase-M*-*.md` completos; `docs/plan/02-matriz-aceitacao.md`
   preenchida; `SKILL.md`, `llms.txt`, `AGENTS.md`, `README.md`, `CHANGELOG.md` em sincronia.

Aceite: `akr init` duas vezes não duplica bloco nem reescreve a skill; `akr prompt` é byte-idêntico
entre execuções (golden); `tools/list` devolve 3 tools e `chart_check` de fonte inválida devolve
`isError` sem encerrar; toda linha da matriz aponta um teste existente.

---

## 7. Arquivos críticos

| arquivo | região | por quê |
|---|---|---|
| `crates/akribie-core/src/syntax.rs` | `option_specs` | fonte única das chaves/valores; parser, `schema` e `prompt` derivam daqui |
| `crates/akribie-core/src/parse.rs` | recuperação por linha | define a qualidade dos diagnósticos, metade do produto para um LLM |
| `crates/akribie-core/src/layout/flow.rs` | adaptação `dugong` | único ponto onde a API do dagre é tocada; isola o risco |
| `crates/akribie-core/src/paint.rs` | `Shape` × `fillstyle` | matriz formas × preenchimentos; onde a estética se decide |
| `crates/akribie-core/src/emit/excalidraw.rs` | `index`, `fixedPoint` | invariantes do Excalidraw; se quebradas, corrompem o arquivo exportado |

---

## 8. Verificação

Pré-requisitos: `cargo` 1.99 (presente), `assets/Excalifont-Regular.ttf` gerado por
`cargo run -p xtask -- font build`, `jq` e `xmllint` instalados, diretório de trabalho na raiz.

1. **Gate**: `make check` verde (fmt, clippy `-D warnings`, `cargo test --workspace`, limite de 300
   linhas). `make ci` verde quando `deny`/`typos` estiverem instalados.
2. **Fixture canônica** — criar `fixtures/retry.akrc` (forma canônica: um espaço, opções em ordem
   alfabética, classes separadas por uma linha em branco):
   ```akrc
   chart v1 "Política de retry" direction=lr layout=flow theme=paper

   node api "API Gateway" fill=blue shape=rounded
   node retry "Retry ×3" fill=amber shape=cloud
   node dlq "DLQ" fill=red shape=cylinder

   edge api -> retry "5xx"
   edge retry -> api "backoff" curve=arc style=dashed
   edge retry -> dlq "esgotado" style=dashed
   ```
   - `akr check fixtures/retry.akrc` → exit 0, nenhum diagnóstico.
   - `akr render fixtures/retry.akrc > /tmp/a.svg && akr render fixtures/retry.akrc > /tmp/b.svg &&
     cmp /tmp/a.svg /tmp/b.svg` → idênticos.
   - `akr render fixtures/retry.akrc --format png --out /tmp/a.png --scale 2` → `file /tmp/a.png`
     reporta PNG; largura = `2 × viewBox`.
   - `for f in png jpg avif; do akr render fixtures/retry.akrc --format $f --out /tmp/a.$f; done` →
     `file` reporta PNG, JPEG e AVIF respectivamente.
3. **Acessibilidade**: `xmllint --noout /tmp/a.svg` sem erro;
   `grep -c 'role="img"' /tmp/a.svg` = 1; `grep -c 'aria-labelledby="akr-t akr-d"' /tmp/a.svg` = 1;
   `grep -c '<desc id="akr-d">' /tmp/a.svg` = 1 e o conteúdo do `<desc>` contém `API Gateway` e
   `Retry ×3 → DLQ`. `grep -o 'lang="[^"]*"' /tmp/a.svg` = `lang="pt-BR"` quando o fixture declarar
   `lang=pt-BR`.
4. **Contraste**: `cargo test -p akribie-core theme::tests::all_combinations_meet_wcag` verde;
   `printf 'chart v1 "x"\nnode a "A" fill=blue stroke=blue\n' | akr check -` → `W007`; o mesmo com
   `--strict` → exit 8.
5. **Diagnóstico acionável**:
   `printf 'chart v1 "x"\nnode a "A" shape=cilinder\n' > /tmp/e.akrc; akr check /tmp/e.akrc` →
   exit 8, `/tmp/e.akrc:2:<col>: E003 invalid-value:` e linha `hint:` contendo `cylinder`.
6. **Lote**: criar `/tmp/lote.jsonl` com três linhas —
   `{"name":"a","source":"chart v1 \"A\"\nnode n \"N\"\n"}`,
   `{"name":"b","path":"fixtures/retry.akrc","format":"png","scale":1}`,
   `{"name":"c","source":"chart v1 \"C\"\n","path":"fixtures/retry.akrc"}`.
   `akr render --batch /tmp/lote.jsonl --out-dir /tmp/out` (com `/tmp/out` criado) → exit 0,
   `/tmp/out/a.svg` e `/tmp/out/b.png` existem, e a terceira linha é reportada como erro.
   `akr render --batch /tmp/lote.jsonl --out-dir /tmp/inexistente` → exit 3.
   `akr render --batch /dev/null` → exit 2.
7. **`--params`**: `akr render --params '{"source":"chart v1 \"P\"\nnode n \"N\"\n","format":"svg"}'`
   → SVG em stdout, exit 0; `akr render --params '{"format":"svg"}' fixtures/retry.akrc` → exit 2.
8. **Diff**: `cp fixtures/retry.akrc /tmp/old.akrc`; remover `node dlq`, acrescentar
   `node ok "Entregue" shape=circle`, trocar `style=dashed` da primeira aresta por `solid`;
   `akr diff /tmp/old.akrc fixtures/retry.akrc` → exatamente uma linha `+ node dlq`, uma
   `- node ok`, uma `~ edge api->retry style:`.
9. **Formato idempotente**: `akr fmt fixtures/retry.akrc | akr fmt - | cmp - fixtures/retry.akrc`
   sem diferença.
10. **Interop Excalidraw**: `akr render fixtures/retry.akrc --format excalidraw --out /tmp/x.excalidraw`;
    `jq -e '.type=="excalidraw"'` → `true`; `jq -r '[.elements[].index] | . == (sort)'` → `true`;
    `akr import /tmp/x.excalidraw > /tmp/back.akrc && akr check /tmp/back.akrc` → exit 0 com no
    máximo `W005`/`W006`. Verificação manual única (não é requisito de runtime): abrir
    `/tmp/x.excalidraw` em `https://excalidraw.com` e confirmar 3 caixas com rótulo dentro e setas
    ancoradas.
11. **Linhagem**: copiar a fixture para `/tmp/doc.akrc` e trocar a linha `chart` por
    `chart v1 "x" direction=lr doc=../decisoes.md#D42 lang=pt-BR layout=flow theme=paper`. Sem
    `/tmp/decisoes.md`, `akr check --docs /tmp/doc.akrc` → exit 0 com `W004`; depois
    `printf 'D42 — decisão\n' > /tmp/decisoes.md` e repetir → sem warning.
12. **MCP**: `printf '%s\n%s\n' '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' \
    '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"chart_check","arguments":{"source":"chart v1 \"x\""}}}' \
    | cargo run -q -p akribie-mcp` → duas linhas JSON; a segunda com `"result"` e sem `isError`.
13. **Documentação viva**: `akr prompt` e `docs/usage/` citam as mesmas flags (conferência manual das
    oito entradas da matriz); `grep -R "akribe\b" docs README.md AGENTS.md SKILL.md llms.txt` não
    retorna nada — o nome é sempre `akribie`.

---

## 9. Pressupostos e contingências

- **`dugong = "=0.9.3"`** (MIT OR Apache-2.0, 187k downloads em 90 dias, publicado 2026-10-30).
  Única dependência de layout. *Se* a API divergir (`layout_controlled` + `NodeLabel{x,y}`), use
  `dugong::layout` e leia a posição pelos mesmos campos; *se* não compilar na toolchain, substitua
  por um Sugiyama próprio em `layout/flow.rs` mantendo `pub fn run(chart: &Chart, font: &Font) -> Vec<Rect>`
  — nenhum outro módulo muda.
- **Fonte**: `woff2::decode::convert_woff2_to_ttf` não suporta fontes com transformação de `hmtx`.
  *Se* a conversão do Excalifont falhar, use `LiberationSans-Regular.ttf` (OFL-1.1, já presente em
  `_REF/excalidraw/scripts/woff2/assets/`) e troque apenas a constante `FONT_FAMILY` de
  `emit/svg.rs` para `"Liberation Sans, Arial, sans-serif"`; medição, layout e raster seguem
  corretos porque leem o mesmo `assets/*.ttf`.
- **`usvg` 0.49**: `Options::fontdb_mut()` e `Database::load_font_data(Vec<u8>)` verificados na API
  pública. *Se* `resvg::render` exigir outra forma de `Transform`, use
  `tiny_skia::Transform::from_scale(scale, scale)`.
- **AVIF**: `image` 0.25.10 expõe `avif` (ravif + rgb) e `jpeg`. *Se* `AvifEncoder` exigir
  `ExtendedColorType::Rgba8` nesta versão, converta o `Pixmap` para `RgbImage` descartando o canal
  alfa (o canvas é sempre opaco). AVIF é o formato mais lento; não otimize antes de medir.
- **Sem `roughr`**: o traço à mão é implementação própria (`sketch.rs`) para garantir determinismo e
  evitar instabilidade de API do port. Não introduza a dependência durante a execução.
- **Escopo**: data charts (barra/linha/pizza) **não** entram na v1 — o modelo é nó/aresta/grupo/nota.
  Um `.akrc` com `shape=bar` falha com `E003` listando as formas válidas.
