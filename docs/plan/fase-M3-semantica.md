# Fase M3 — tema, contraste, configuração, resolução e medição

**Status:** concluída · **Depende de:** M1 · **Habilita:** M4–M7

## Entrega

| peça | responsabilidade |
|---|---|
| `theme/mod.rs` | `Token` (9), `Theme`, paletas fechadas `PAPER`/`SLATE`, `theme()`, `hex`, `background`, `token_of_hex` |
| `theme/contrast.rs` | `contrast_ratio(a, b)` WCAG 2.2; hex inválido ⇒ 1.0 |
| `model/` | `Chart`, `Group`, `Node`, `Edge`, `Note` + enums `Layout`, `Direction`, `Shape`, `FillStyle`, `StrokeStyle`, `Curve`, `Head` |
| `config.rs` | `Config` + `akribie.toml` (subconjunto TOML) + `discover` subindo diretórios |
| `resolve/` | `Document` + `Config` → `Chart`; `E004`, `E005`, `E006`, `W001`; arestas com `ordinal` |
| `a11y.rs` | `describe(&Chart)` (texto para leitor de tela) e `check_contrast` (`W007`) |
| `text.rs` | `Font::load` (Excalifont embutida), `advance_px`, `wrap`, `label_size`, `node_size`, `missing_glyphs` |
| `pipeline.rs` | `check(source, config, font) -> CheckReport`: parse + resolve + `W002` + `W003` + `W007` |
| `akr schema` | publica `themes` (canvas, ink, paleta) e `contrast` (mínimos) |

## Contraste: a contradição do plano e como foi resolvida

O plano garantia duas coisas incompatíveis: **(a)** tokens de cor são acentos usados em `stroke`
(§2.2/§4.2), **(b)** todo contorno tem ≥ 3:1 contra a superfície (§4.2/§6.4). Medido:

| tema | par | razão |
|---|---|---|
| paper | `blue` sobre canvas | **1.51:1** |
| paper | `green` sobre canvas | 1.29:1 |
| paper | `amber` sobre canvas | 1.13:1 |
| slate | `blue` sobre canvas | 1.63:1 |
| paper | `ink` sobre `blue` (fundo) | **1.23:1** — texto ilegível se o contorno fosse regra de fundo |

Escolha registrada (ADR-0005, nota de escopo): a **regra executável** é

- texto `ink` sobre `fill` ≥ **4.5:1** → garantido pela paleta em todos os 6 `fill` × 2 temas
  (pior caso: `violet`, 9.99:1 no paper e 7.17:1 no slate);
- contorno ≥ **3:1 contra a superfície em que aparece** — sobre o `fill` quando existe, sobre o
  canvas quando não existe (arestas, formas vazadas, grupos). `ink` e `muted` cumprem sempre
  (`muted` × `violet`: 4.01:1 paper / 4.24:1 slate);
- tokens coloridos como contorno ficam **abaixo** do mínimo e o `check` emite `W007` **com a razão
  real** — não é ruído, é a informação que o autor precisa para decidir. O teste
  `colored_strokes_are_accents_below_minimum` fixa esse fato; se a paleta mudar, o teste avisa.

## Decisões de execução

- **`Token::Paper` é o canvas do tema.** Num tema escuro, "paper" é escuro (`#121212`); é o que
  permite `fill=paper` significar "mesma cor do fundo" nos dois temas. Por isso a paleta de cada tema
  tem oito entradas e `muted` não é `fill` válido (contraste de texto sobre `muted` seria 1.36:1).
- **`Theme` é `&'static`** no `Chart`: tema é dado de build, não configuração por documento; o modelo
  continua `PartialEq` e compara ponteiros/valores sem alocação.
- **Config não é segundo dialeto.** `akribie.toml` é `chave = valor` e os valores passam pelo
  **mesmo** `lex` + `syntax::validate` do `.akrc`. Chave desconhecida e valor fora do contrato viram
  erro com linha (`akribie.toml:2: …`). Bareword aceita aspas (`lang = "pt-BR"`) porque é o que se
  escreve num arquivo de configuração; no `.akrc`, não.
- **Descoberta sobe até a raiz, primeiro vence** — `akribie.toml` num subdiretório especializa o
  projeto inteiro. `stdin`/`--params` não têm projeto: defaults.
- **`resolve` declara ids em duas passadas** (grupos e nós, depois `in=`), para aceitar referência
  para frente — a propriedade que permite escrever `edge api -> retry` antes de `node api`.
- **Implicito só para aresta.** `edge a -> b` cria `a` e `b` com `W001` (uma vez por id) e eles vão
  ao **fim** da lista de nós, na ordem de aparição: a ordem declarada manda no layout.
- **Aresta para grupo é `E005`**, não implícito: criar um nó homônimo ao grupo esconderia o erro.
- **`ordinal`** conta pares `(from, to)` iguais; é o que permite desenhar três retries paralelos como
  três linhas, sem inventar ids. Contado em `resolve`, não em `layout`.
- **Medição com shaping** (`harfrust`), não com soma de avanços de glifo: ligaduras e kerning da
  Excalifont mudariam a largura real. A fonte é embutida (`include_bytes!`), então o número é o mesmo
  em qualquer máquina — requisito de determinismo do ADR-0001.
- **`W003` por codepoint distinto** em cada elemento (não por ocorrência): um emoji repetido cinco
  vezes é um problema, não cinco.
- **`W002` só em layout que espera grafo** (`flow`, `tree`, `mindmap`); em `grid`/`stack`/`free` nó
  sem aresta é legítimo.
- **`check` executa todos os estágios mesmo com erro de parse**, para devolver a lista completa de
  correções de uma vez — o objetivo explícito do produto.

## Verificação

```sh
make check                                   # verde (fmt, clippy -D warnings, 65+ testes, 300 linhas)
cargo test -p akribie-core                   # 45 lib (theme 8, resolve 13, a11y 4, text 7, config 6, …)
target/debug/akr schema | jq .contrast       # {"min_text":4.5,"min_stroke":3.0}
target/debug/akr schema | jq -r '.themes[].name'   # paper, slate
```

Caso de ponta a ponta (projeto com `akribie.toml` em `layout = grid`):

```akrc
chart v1 "Retry"
node api "API Gateway" fill=blue stroke=blue
group core "Núcleo"
node retry "Retry 🦀" stroke=blue
edge api -> retry "5xx"
edge retry -> dlq "esgotado"
note "Checar backoff" near=retry
```

| checagem | resultado observado |
|---|---|
| `akr check docs/a.akrc` | `1:18 E002`; `2:1 W007` (contorno `blue` sobre `blue`, 1.0:1); `4:1 W003` (`🦀`) e `W007` (1.5:1); `6:1 W001` (`dlq`) — exit 8 |
| `chart v1 "x"` + `node a "A"` (sem config) | `W002` em `a`, exit 0 |
| o mesmo com `akribie.toml` de `layout = grid` | stdout vazio, exit 0 (`::config_from_project_directory_changes_layout`) |
| `advance_px("API Gateway", 20) + 24` | igual à largura de `node_size` (`::node_width_is_advance_plus_padding_or_minimum`) |
| `--strict` sobre aviso `W002` | exit 8 (`::strict_promotes_warnings_to_failure`) |
