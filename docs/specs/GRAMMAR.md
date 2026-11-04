# GRAMMAR — contrato do `.akrc`

Normativo. A fonte é UTF-8 sem BOM, baseada em linhas: **uma afirmação por linha**. O parser lê a
linha inteira e recupera erro por linha (uma linha inválida produz diagnóstico e não invalida as
demais). A forma canônica é produzida por `akr fmt` e verificada por `akr fmt --check`.

## Documento

```bnf
document   = chart { statement } [ trailing_comments ] ;
chart      = "chart" "v1" quoted { option } ;
statement  = group | node | edge | note ;
```

`chart` é obrigatório e é a primeira afirmação não vazia (`E008`). A versão aceita é `v1` (`E009`).

## Statements

```bnf
group = "group" ident [ quoted ] { option } ;
node  = "node"  ident [ quoted ] { option } ;
edge  = "edge"  ident "->" ident [ quoted ] { option } ;
note  = "note"  quoted { option } ;
option = key "=" value ;
value  = quoted | number | ident | pair | list | "none" ;
pair   = number "," number ;
list   = ident { "," ident } ;
```

- `ident` — letras, dígitos, `_` e `-`, começando por letra ou `_`. Nunca entre aspas.
- `quoted` — texto entre `"`, com `\"` e `\\`; não pode conter quebra de linha. Um token que **abre**
  com aspas é uma string inteira: `=` dentro dela é conteúdo (`node a "user=admin"`, `edge a -> b
  "x=1"`), nunca separador de `chave=valor`. Fora das aspas, o primeiro `=` é sempre o separador.
- `number` — inteiro ou decimal com ponto; `-` permitido; não-finitos são rejeitados.
- Chaves são **fechadas**: chave desconhecida é `E002`, valor fora do conjunto é `E003` (com a lista
  do válido no `hint`). A fonte única da tabela é `syntax::option_specs`; `akr schema` e
  `akr prime --long` derivam dela.

## Chaves

| statement | chaves |
|---|---|
| `chart` | `layout` `direction` `theme` `fontsize` `pad` `lang` `sketch` `font` `doc` |
| `group` | `fill` `stroke` `style` `width` `pad` `in` |
| `node` | `shape` `fill` `fillstyle` `stroke` `style` `width` `in` `at` `size` |
| `edge` | `curve` `head` `tail` `style` `stroke` `width` |
| `note` | `fill` `at` `size` `near` |

Conjuntos fechados: `layout` ∈ {flow, tree, mindmap, grid, stack, free}; `direction` ∈ {lr, tb, bt,
rl}; `theme` ∈ {paper, slate}; `shape` ∈ {rect, rounded, circle, diamond, cloud, cylinder, hexagon,
note, text}; `fillstyle` ∈ {hachure, solid, cross-hatch, zigzag}; `style` ∈ {solid, dashed, dotted};
`curve` ∈ {line, arc, elbow}; `head`/`tail` ∈ {arrow, none}; `sketch` ∈ {on, off} (padrão `off`:
traço técnico; `on`: à mão); `font` ∈ {hand, sans} (padrão `sans`: Liberation Sans; `hand`:
Excalifont); tokens de cor ∈ {none, paper, ink, muted, blue, green, amber, red, violet}.

Faixas: `fontsize` ∈ [14, 96] (20); `pad` ∈ [0, 256] (32); `width` ∈ [2, 6] (2); `size` ≥ 1.

## Forma canônica

1. Classes na ordem `chart`, `group`, `node`, `edge`, `note`; a ordem de declaração é preservada
   dentro de cada classe.
2. Exatamente uma linha em branco entre classes não vazias; nenhuma dentro de uma classe.
3. Comentários (`#`) ficam colados ao statement **seguinte**; sem statement seguinte, no fim.
4. Um espaço como separador, sem alinhamento de colunas.
5. Strings sempre entre aspas; ids nunca.
6. Opções em ordem alfabética de chave, sem repetição.
7. Números com até duas casas, sem zeros à direita (`num::format`).
8. Arquivo termina com exatamente um `\n` (documento vazio ⇒ saída vazia).

`fmt` é idempotente e nunca muda a semântica; arquivo com erro de schema não é formatado.

## Diagnósticos da gramática

| código | quando |
|---|---|
| `E001` | keyword desconhecida |
| `E002` | chave desconhecida no statement |
| `E003` | valor inválido (conjunto fechado, tipo ou faixa) |
| `E004` | id duplicado |
| `E005` | referência a id inexistente (`in=`, `edge`, `near=`) |
| `E006` | `at=` ausente em `layout=free`, ou `at` em layout calculado |
| `E007` | linha malformada (token inválido, aspas não fechadas) |
| `E008` | primeira afirmação não é `chart` |
| `E009` | versão diferente de `v1` |
| `W001` | nó implícito criado por `edge` |
| `W002` | nó isolado em `layout=flow` |
| `W003` | codepoint sem glifo na fonte embutida |
| `W004` | linhagem `doc=` ausente (só com `--docs`) |
| `W005` | cor fora da paleta (import) |
| `W006` | elemento Excalidraw sem equivalente (import) |
| `W007` | contraste abaixo do mínimo |

`E00x` ⇒ exit 8; `W00x` ⇒ exit 0 (8 sob `--strict`). `akr schema` lista os códigos com nome e
gravidade; `akr prime --long` os lista em texto.
