# ADR-0005 — Acessibilidade é contrato do SVG

- **Status:** Accepted
- **Data:** 2026-11-03
- **Decisores:** mantenedores do akribie
- **Relaciona:** ADR-0002 (fonte canônica), ADR-0001 (determinismo)

## Contexto

O consumidor do diagrama não é só humano. Dois casos concretos:

1. **Leitor de tela.** Um SVG sem `<title>`/`<desc>` é anunciado como "gráfico" e nada mais; o
   usuário não tem acesso à informação. Isso não é detalhe estético: é o diagrama sendo inacessível.
2. **Agente que lê o artefato.** Um LLM recebendo o SVG para entender a arquitetura precisa de texto,
   não de `path d="M12 0.5..."`. Sem alternativa textual derivada do modelo, o modelo teria de
   reconstruir a topologia a partir de coordenadas — exatamente o que o formato evita.

Também há o caso de contraste: uma paleta "bonita" escolhida por gosto produz diagramas ilegíveis para
baixa visão e para impressão. Cor não pode ser o único canal de informação (WCAG 2.2 SC 1.4.1).

## Decisão

**O SVG carrega a informação duas vezes: no desenho e no texto.** Estrutura normativa:

- Raiz com `role="img"`, `aria-labelledby="akr-t akr-d"`, `lang` (de `chart.lang`) e
  `focusable="false"`.
- `<title id="akr-t">` e `<desc id="akr-d">` são os **primeiros** filhos do `<svg>`.
- `<desc>` é derivado do **modelo**, não da geometria: contagem de nós e arestas, lista de rótulos e
  lista de relações no formato `from → to (label)`, separadas por `; `. Sem rótulo, omite o
  parêntese.
- Cada elemento desenhado é um `<g data-id="…" data-kind="group|edge|node|note">` com
  `aria-label` igual ao rótulo — omitido quando não há rótulo.
- Grupos de conteúdo na ordem `#groups`, `#edges`, `#nodes`, `#notes`; `#nodes` com `role="list"` e
  cada nó `role="listitem"`.
- Ids internos prefixados `akr-` para não colidir quando o SVG é embutido em HTML.

**Contraste é mínimo verificável, não preferência.**

| par | mínimo | origem |
|---|---|---|
| texto × `fill` do elemento | 4.5:1 | WCAG 2.2 AA (texto normal) |
| `stroke` × canvas | 3:1 | WCAG 2.2 SC 1.4.11 |
| `stroke` × `fill` do elemento | 3:1 | WCAG 2.2 SC 1.4.11 |

As paletas `paper` e `slate` são fechadas e o teste `theme::tests::all_combinations_meet_wcag` roda
sobre **todas** as combinações. Uma combinação explícita do usuário que viole o mínimo produz `W007`
em `akr check` — o valor é aceito (o autor pode ter razão), mas o custo é anunciado.

**Cor nunca é o único canal.** Arestas têm rótulo textual quando declarado, e o `<desc>` lista todas
as relações. Um diagrama em preto e branco continua transmitindo a topologia completa.

**Legibilidade tem piso numérico.** `fontsize` ∈ [14, 96] (default 20), `width` ∈ [2, 6] (default 2),
nós com mínimo de `96 × 44`, padding interno `12`, altura de linha `1.35`, largura máxima de rótulo
`320`. Nenhum caminho de layout pode produzir elemento abaixo do piso; o `size=` explícito do autor é
validado com `≥ 1` e o mínimo de nó é aplicado na medição.

**Diagnóstico é auto-suficiente.** Toda mensagem traz código, posição, o que se esperava e a
correção; `hint:` é parte da mensagem, não decoração. Nenhum comando depende de cor de terminal ou de
largura de terminal, e nada imprime tabela alinhada que quebre em pipe.

## Consequências

**Positivas**

- O SVG passa em `xmllint` e é utilizável por leitor de tela sem pós-processamento.
- Um agente consegue responder "quais são as arestas?" lendo o `<desc>`, sem parsear geometria.
- O teste de contraste impede regressão de paleta: mudar um hex quebra o teste antes de quebrar o
  usuário.
- `akr check --strict` transforma `W007` em falha, o que permite usar o binário em CI de acessibilidade.

**Negativas / custos aceitos**

- O SVG fica maior (title, desc, aria-label, ids) e o `<desc>` precisa ser mantido em sincronia com o
  modelo — é gerado, então a sincronia é estrutural.
- Paleta fechada limita a expressão visual; adicionar tema novo exige validar 4.5:1/3:1 para todas as
  combinações.
- O `lang` é declarado e não traduzido: o akribie emite o texto que o autor escreveu, no idioma que o
  autor declarou.

## Alternativas rejeitadas

- **`<desc>` gerado da geometria:** descreveria posições em vez de relações — ruído para o leitor de
  tela e para o agente.
- **Contraste apenas documentado:** sem teste, a próxima mudança de paleta regride silenciosamente.
- **`aria-hidden` com texto alternativo externo:** transfere ao hospedeiro uma responsabilidade que o
  artefato pode cumprir sozinho.
