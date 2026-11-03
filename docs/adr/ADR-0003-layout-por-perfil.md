# ADR-0003 — Layout por perfil, com o roteamento de aresta sob nosso controle

- **Status:** Accepted
- **Data:** 2026-11-03
- **Decisores:** mantenedores do akribie
- **Relaciona:** ADR-0001 (determinismo), ADR-0002 (fonte canônica)

## Contexto

Layout é o único estágio do compilador que não é local: a posição de um nó depende da posição dos
outros. É também o estágio em que uma dependência externa cobra mais caro — motores de layout reais
(dagre, Graphviz) trazem heurísticas grandes, APIs instáveis e ordem de iteração dependente de hash
em vários pontos.

O requisito do produto é estreito: diagramas de 3 a 30 elementos, seis topologias reconhecíveis
(fluxograma, árvore, mapa mental, grade, pilha, livre), com resultado determinístico e legível. Não
precisamos de layout genérico de grafo com milhares de nós.

O que o formato promete ao usuário é ainda mais estreito: `flow` com `a -> b -> c` em `direction=lr`
produz `a.x < b.x < c.x` com `a.y == c.y`. Essa promessa precisa de um motor que respeite `rankdir`
e camadas — daí o uso de um port de dagre para a fase de ranqueamento.

## Decisão

**Seis perfis explícitos, um dispatch, nenhuma heurística escondida.** `layout` é um enum fechado:

| perfil | algoritmo | ordem de irmãos |
|---|---|---|
| `flow` | dagre (via `dugong`) para ranquear; arestas roteadas por nós | ordem de declaração |
| `tree` | Reingold–Tilford; raízes = nós sem aresta de entrada | ordem de declaração |
| `mindmap` | radial; raiz única, ângulo proporcional às folhas da subárvore | ordem de declaração |
| `grid` | `columns = ceil(sqrt(n))`, célula = maior nó | ordem de declaração |
| `stack` | uma coluna, `GAP_STACK = 28.0` | ordem de declaração |
| `free` | posição de `at=`, `bbox` normalizado | irrelevante |

**O motor de layout entrega posições de nó, não rotas.** `layout/flow.rs` usa `dugong` apenas para
`NodeLabel { x, y }` (centro) e, portanto, para `Rect`. O roteamento de aresta é nosso
(`geometry::route_line`, `route_arc`, `route_elbow`), porque a rota precisa respeitar a forma do nó
(`outline_hit` para `rect`/`circle`/`diamond`), o estilo declarado e a regra de self-edge. Deixar a
rota com o motor significaria aceitar uma segunda fonte de geometria e um segundo contrato de bytes.

**`dugong = "=0.9.3"` é a única dependência estrutural, e o risco é confinado.** A adaptação vive em
um arquivo (`layout/flow.rs`) atrás de `pub fn run(chart: &Chart, font: &Font) -> Vec<Rect>`. Se a
API divergir ou o crate deixar de compilar, a substituição por um Sugiyama próprio é local — nenhum
outro módulo muda.

**Todo perfil recebe `&Chart` + `&Font` e devolve geometria pura.** Nenhum perfil conhece tema, cor,
SVG ou I/O. Medição de texto entra só para dimensionar o nó (`text::wrap` + `advance_px`) e é
compartilhada por todos os perfis, o que garante que um nó do mesmo rótulo tenha o mesmo tamanho em
qualquer layout.

**`at=` é pino local, não modo global.** Em `layout=free`, todo `node`/`note` exige `at` (`E006`). Nos
demais perfis, `at` é permitido e fixa aquele elemento; os outros seguem o layout. Isso permite
corrigir um único nó sem abandonar o perfil.

**Constantes de espaçamento são fixas e nomeadas** (`nodesep = 48.0`, `ranksep = 64.0`,
`GAP_GRID = 40.0`, `GAP_STACK = 28.0`, `V_GAP = 80.0`, `H_GAP = 40.0`, `RADIUS_STEP = 180.0`).
Não há parâmetro de layout numérico vindo do `.akrc` na v1: uma constante a mais é uma combinação a
mais para testar e documentar.

## Consequências

**Positivas**

- Cada perfil é testável por asserção direta sobre coordenadas (`a.x < b.x < c.x`), sem golden de PNG.
- Trocar o motor de `flow` é uma mudança de um arquivo, com o resto do pipeline intacto.
- O roteamento de aresta é único para todos os perfis: uma correção de rota vale para os seis.
- Determinismo verificável: entrada ordenada + ordem de declaração ⇒ mesmos `Rect`.

**Negativas / custos aceitos**

- Perfis são mais simples que um motor genérico: `flow` não faz otimização de cruzamento de arestas
  além do que o dagre faz, e `grid` não tenta minimizar área.
- O roteamento próprio não é tão bom quanto o de um motor dedicado em grafos densos; aceitável para
  o tamanho-alvo (3–30 elementos).
- `dugong` pina uma versão exata; atualizar é decisão consciente, com reexecução dos goldens.

## Alternativas rejeitadas

- **Graphviz via subprocesso:** viola ADR-0001 (binário único, sem dependência externa de runtime) e
  a saída depende de versão do binário do sistema.
- **Layout forçado a `free`:** devolveria ao autor exatamente o problema que o produto resolve —
  coordenar geometria à mão.
- **Um perfil único configurável por pesos:** menos previsível; o usuário não consegue antecipar o
  resultado, e o teste vira "parece razoável" em vez de asserção.
