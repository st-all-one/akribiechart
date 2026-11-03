# ADR-0004 — `.excalidraw` é derivado, nos dois sentidos

- **Status:** Accepted
- **Data:** 2026-11-03
- **Decisores:** mantenedores do akribie
- **Relaciona:** ADR-0002 (fonte canônica), ADR-0001 (determinismo)

## Contexto

Excalidraw é a referência visual do projeto: o traço à mão, a fonte Excalifont e o estilo de preenchimento
hachurado vêm de lá. Usuários vão querer abrir o diagrama gerado em `excalidraw.com` e editar; e vão
querer trazer um `.excalidraw` existente para o formato textual.

Três cenários precisam de decisão separada:

1. **Exportar** para `.excalidraw` — precisa produzir arquivo que o Excalidraw aceite e valide.
2. **Importar** de `.excalidraw` — precisa produzir `.akrc` que `akr check` aceite.
3. **Round-trip** — precisa de uma promessa honesta sobre o que se perde.

O formato do Excalidraw carrega invariantes não óbvias que, se violadas, corrompem o arquivo de forma
silenciosa: `index` precisa ser ordenável lexicograficamente e não pode terminar em `0`
(`validateOrderKey` rejeita); `fixedPoint` com valor exatamente `0.5` é rejeitado pela validação de
binding; `versionNonce` e `updated` mudam a cada edição no app e não têm significado semântico.

## Decisão

**Exportação é derivada e determinística.** `akr render --format excalidraw` produz um JSON cujo
conteúdo é função pura do `Chart` e do tema. Nada de timestamp: `updated: 0`, `created: 0`,
`versionNonce: 0`, `version: 1`, `seed = seed_for(id) & 0x7fff_ffff`. Duas execuções com o mesmo
`.akrc` produzem o mesmo arquivo byte a byte.

**Ordem e `index` são construídos, não herdados.** Elementos saem na ordem `group`, `edge`, `node`,
`note`; cada `node` é seguido imediatamente pelo seu elemento `text` (o container precisa vir antes do
texto que o referencia). O `index` é `format!("a{:0>4}V", base62(i))`: alfabeto `0-9A-Za-z`, largura
fixa (logo, ordem lexicográfica = ordem numérica) e sufixo `V` para nunca terminar em `0`.

**Binding é explícito e nunca exatamente `0.5`.** `startBinding`/`endBinding` apontam para o id do
alvo com `fixedPoint` = razão do ponto de ancoragem no bbox do alvo; quando a razão calculada é
`0.5`, grava-se `0.5001`. O arrow aponta para elementos que existem no mesmo arquivo, com
`elementId` igual ao `id` do elemento (`akr-<id>`).

**Importação é best-effort, com downgrade anunciado.** `akr import` mapeia:

| origem | destino |
|---|---|
| `rectangle` | `node` com `shape=rounded` |
| `ellipse` | `node` com `shape=circle` |
| `diamond` | `node` com `shape=diamond` |
| `text` com `containerId` | rótulo do container |
| `text` solto | `note` |
| `arrow` com ambos os bindings | `edge` |
| `arrow` sem bindings, demais tipos | descartado com `W006` |
| `backgroundColor` sem token equivalente | `fill=none` + `W005` |

O resultado é `chart ... layout=free`, título = nome do arquivo, com `at`/`size` de cada elemento.
**A rota da aresta é recalculada**, nunca importada: a geometria do Excalidraw não é contrato nosso e
carregá-la violaria o ADR-0003.

**Round-trip é lossy por desenho, e o loss é declarado.** Exportar e reimportar preserva topologia,
rótulos, cores mapeáveis, estilos de traço e posições; perde o que não tem representação no `.akrc`
(imagens, elementos agrupados do Excalidraw, `frame`s, texto com formatação rica) e o que não tem
token de tema equivalente (`W005`). `W006` lista o que foi descartado, em vez de sumir em silêncio.

## Consequências

**Positivas**

- O export pode ser golden testado (`jq -e '.type=="excalidraw"'` e ordenação de `index`).
- O import devolve trabalho útil: um `.excalidraw` existente vira fonte versionável em um comando.
- O arquivo abre em `excalidraw.com` porque as invariantes (`index`, `fixedPoint`, ordem de container
  e texto) são construídas por construção, não por sorte.

**Negativas / custos aceitos**

- Não há edição de `.excalidraw` no akribie; o fluxo é sempre `.excalidraw` → `.akrc` → artefatos.
- Campos cosméticos do Excalidraw (roughness fino, `roundness`, `elbowed`) não são controláveis pela
  v1; o export fixa `roughness: 1` e `elbowed: false`.
- A importação descarta elementos sem correspondência; o usuário precisa ler `W005`/`W006` para saber
  o que perdeu.

## Alternativas rejeitadas

- **`.excalidraw` como fonte primária:** diff ilegível (ADR-0002) e campos derivados mudando sem
  mudança semântica.
- **Import preservando rotas de aresta:** violaria o contrato de determinismo e a fonte única de
  geometria; uma rota importada não sobrevive a mover um nó.
- **Export com timestamps reais:** impossível sem violar ADR-0001 §2 e inútil — `updated` não é lido
  pelo usuário nem pelo diff.
