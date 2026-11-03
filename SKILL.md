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

# akr — diagramas a partir de texto

Use quando o pedido envolver desenhar/atualizar um diagrama de nós e setas, converter um
`.excalidraw` para texto, ou comparar duas versões de um diagrama.

## Regra de ouro

**Escreva topologia, não coordenadas.** O layout é calculado. Se você se pegar calculando `x`/`y`,
pare: declare `node`/`edge` e deixe o `akr` posicionar. Use `at=x,y` só quando o pedido exigir
posição fixa (e então `layout=free`).

## Ciclo

```sh
akr prompt                 # gramática compacta (o mesmo que você precisa saber)
akr check chart.akrc        # valide ANTES de renderizar
akr render chart.akrc > chart.svg
```

`akr check` sai 8 quando há erro e imprime `arquivo:linha:col: E0xx nome: mensagem` com `hint:`.
**Corrija exatamente a linha apontada e reenvie.** Não reescreva o documento inteiro nem invente
chaves: o conjunto é fechado (`E002`/`E003` listam o válido).

Em um projeto novo, `akr init` instala `akribie.toml`, o bloco de instruções em `AGENTS.md` e esta
skill em `.agents/skills/akr/SKILL.md` (idempotente). Se o ambiente falar MCP, o servidor
`akribie-mcp` expõe `chart_check`, `chart_render` e `chart_diff` sem sair do processo.

## Escolha do layout

| pedido | layout | observação |
|---|---|---|
| fluxo, processo, pipeline | `flow` | `direction=lr` ou `tb` |
| hierarquia, organograma | `tree` | raízes = nós sem entrada |
| mapa mental, tópicos | `mindmap` | ignora `direction` |
| catálogo, comparação | `grid` | 2×2 para 4 nós |
| lista, passos | `stack` | uma coluna |
| posição declarada | `free` | exige `at` em todo nó/nota |

## Escolha visual

- `fill` para dar peso: `blue` (primário), `green` (sucesso), `amber` (atenção), `red` (erro),
  `violet` (destaque), `paper` (neutro), `none`.
- `shape`: `rounded` (default), `rect`, `circle`, `diamond` (decisão), `cloud` (externo/lento),
  `cylinder` (armazenamento), `hexagon`, `note`, `text`.
- `style=dashed` para caminho alternativo/assíncrono; `curve=arc` para retorno (backoff, retry).
- **Sempre** declare o título do `chart`: ele vira `<title>` no SVG (acessibilidade).

## Acessibilidade (não opcional)

O SVG já sai com `role="img"`, `<title>`, `<desc>` com toda a topologia e `aria-label` por elemento.
Rótulos de aresta são texto, não decoração de cor. Não descreva o diagrama só por cor no texto que
acompanha o arquivo.

## Acabamento e fonte

`sketch=on` (padrão) desenha à mão; `sketch=off` desenha linhas exatas, arcos reais e uma única
passada (modo técnico). `font=hand` usa a Excalifont; `font=sans` usa a Liberation Sans embutida.
Ambas são chaves de `chart` (ou do `akribie.toml`):

```akrc
chart v1 "Arquitetura" sketch=off font=sans
```

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
