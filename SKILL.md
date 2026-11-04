---
name: akr
description: Compilar, validar e comparar diagramas de topologia `.akrc` com o binário `akr` (SVG acessível, PNG/JPG/AVIF, scene JSON, Excalidraw).
triggers:
  - diagrama
  - flowchart
  - topologia
  - excalidraw
  - akrc
  - akr
---

<!-- akribie:skill:version: 1 -->

# akr — diagramas a partir de texto

Use quando o pedido envolver desenhar/atualizar um diagrama de nós e setas, converter um
`.excalidraw` para texto, ou comparar duas versões de um diagrama. O binário é offline: nada aqui
depende de rede em tempo de execução, e SVG/PNG são **derivados** do `.akrc` — nunca os edite.

## Regra de ouro

**Escreva topologia, não coordenadas.** O layout é calculado. Se você se pegar calculando `x`/`y`,
pare: declare `node`/`edge` e deixe o `akr` posicionar. `at=x,y` fixa um elemento (pinning parcial,
permitido em qualquer layout); `layout=free` exige `at` em todo nó e nota.

## Superfície

| verbo | para quê |
|---|---|
| `akr init [--force]` | cria `akribie.toml`, o bloco em `AGENTS.md` e esta skill (idempotente) |
| `akr prime [--long]` | gramática compacta; `--long` acrescenta BNF, tabelas de chaves e temas |
| `akr schema` | JSON com chaves, tipos, defaults, valores fechados, tokens e códigos |
| `akr check [FILE\|-]` | valida (resolve, layout, contraste); `--strict` promove aviso a erro; `--docs` resolve `doc=` |
| `akr fmt [FILE\|-]` | forma canônica; `--check` sai 1 quando difere, `--write` grava |
| `akr render [FILE\|-]` | SVG em stdout, raster com `--out`, ou `--format scene\|excalidraw` |
| `akr import [FILE\|-]` | `.excalidraw` → `.akrc` canônico (`layout=free`) |
| `akr diff OLD NEW` | diff semântico por id; exit 1 quando há diferença |

`check`, `fmt`, `render`, `import` e `diff` aceitam `--params '<json>'` (substitui integralmente
posicionais e flags do verbo); `check`, `render`, `import` e `diff` aceitam também `--json`
(envelope `{success,command,data,warnings}`) e, em `check`/`render`, `--batch arquivo.jsonl` (uma
linha por chart, best-effort, na ordem do arquivo).
`render` e `diff` aceitam `--at REV` (fonte lida com `git show`, sem tocar a árvore).

## Ciclo

```sh
akr check chart.akrc        # valide ANTES de renderizar; exit 0 quando só há avisos
akr fmt chart.akrc --write  # forma canônica (o diff do Git fica semântico)
akr render chart.akrc > chart.svg
akr render chart.akrc --format png --out chart.png --scale 2
```

`akr check` sai 8 quando há erro e imprime `arquivo:linha:col: E0xx nome: mensagem` com `hint:`.
**Corrija exatamente a linha apontada e reenvie.** Não reescreva o documento inteiro nem invente
chaves: o conjunto é fechado (`E002`/`E003` listam o válido).

Em um projeto novo, `akr init` instala `akribie.toml`, o bloco de instruções em `AGENTS.md` e esta
skill em `.agents/skills/akr/SKILL.md`. Se o ambiente falar MCP, o servidor `akribie-mcp` expõe
`chart_check`, `chart_render` e `chart_diff` sem sair do processo.

## Configuração

`akribie.toml` (procurado do diretório do `.akrc` para cima; o primeiro vence) define `layout`,
`direction`, `theme`, `fontsize`, `pad`, `lang`, `sketch` e `font`. Precedência: opção no `.akrc` >
arquivo > default do binário. Chaves, valores fechados e defaults: `akr schema`.

## Saída e erros

| exit | significado |
|---|---|
| 0 | sucesso (inclui EPIPE em stdout) |
| 1 | `fmt --check` difere, ou `diff` encontrou mudanças |
| 2 | uso inválido (flag, formato, stdin ausente, lote vazio) |
| 3 | arquivo, revisão git ou `--out-dir` inexistente |
| 5 | falha de leitura/escrita |
| 8 | `.akrc` inválido (≥1 erro) ou lote 100% falho |
| 70 | invariante interna quebrada |

`render --format png|jpg|avif` exige `--out` (raster é binário); `--scale` (default 2) multiplica os
pixels do raster — `--scale 2` dobra cada eixo — e o `viewBox` do SVG não muda. `--dry-run` mede e
reporta sem gravar.

## Escolha do layout

| pedido | layout | observação |
|---|---|---|
| fluxo, processo, pipeline | `flow` | `direction=lr` ou `tb` |
| hierarquia, organograma | `tree` | raízes = nós sem entrada |
| mapa mental, tópicos | `mindmap` | ignora `direction` |
| catálogo, comparação | `grid` | 2×2 para 4 nós |
| lista, passos | `stack` | uma coluna |
| posição declarada | `free` | exige `at` em todo nó/nota |

Grupos são caixas em volta de membros: o nó declara `in=grupo` (um grupo pode ter `in=pai`).

## Escolha visual

- `fill` para dar peso: `blue` (primário), `green` (sucesso), `amber` (atenção), `red` (erro),
  `violet` (destaque), `paper` (neutro), `none`.
- `shape`: `rounded` (default), `rect`, `circle`, `diamond` (decisão), `cloud` (externo/lento),
  `cylinder` (armazenamento), `hexagon`, `note`, `text`.
- `fillstyle`: `hachure` (default), `solid`, `cross-hatch`, `zigzag`.
- `style=dashed` para caminho alternativo/assíncrono; `curve=arc` para retorno (backoff, retry).
- **Sempre** declare o título do `chart`: ele vira `<title>` no SVG (acessibilidade).

## Acabamento e fonte

`sketch=off` (padrão) desenha linhas exatas, arcos reais e uma única passada (modo técnico);
`sketch=on` desenha à mão. `font=sans` (padrão) usa a Liberation Sans embutida; `font=hand` usa a
Excalifont. Ambas são chaves de `chart` (ou do `akribie.toml`):

```akrc
chart v1 "Arquitetura" sketch=on font=hand
```

## Acessibilidade (não opcional)

O SVG já sai com `role="img"`, `<title>`, `<desc>` com toda a topologia e `aria-label` por elemento.
Rótulos de aresta são texto, não decoração de cor. Não descreva o diagrama só por cor no texto que
acompanha o arquivo.

## Exemplos

Fluxograma com retorno:

```akrc
chart v1 "Retry" direction=lr layout=flow theme=paper lang=pt-BR

node api "API" fill=blue
node retry "Retry ×3" fill=amber shape=cloud
node dlq "DLQ" fill=red shape=cylinder

edge api -> retry "5xx"
edge retry -> api "backoff" curve=arc style=dashed
edge retry -> dlq "esgotado" style=dashed
```

Comparar duas versões:

```sh
akr diff old.akrc new.akrc          # + node dlq "DLQ" · - node legacy · ~ edge api->retry style: …
akr diff old.akrc new.akrc --json   # {"added":[…],"removed":[…],"changed":[…]}
```

Converter um Excalidraw:

```sh
akr import desenho.excalidraw > desenho.akrc   # layout=free; W005/W006 listam perdas
akr fmt desenho.akrc --write
```

## Não faça

- Não edite o SVG/PNG à mão: regenere a partir do `.akrc`.
- Não use `fill=ink` nem `fill=muted`: `fill` aceita `none paper blue green amber red violet`.
- Não conte com rede: o binário é offline.
