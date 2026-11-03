# ADR-0002 — A fonte é texto canônico; o resto é artefato

- **Status:** Accepted
- **Data:** 2026-11-03
- **Decisores:** mantenedores do akribie
- **Relaciona:** ADR-0001 (determinismo), ADR-0003 (layout), ADR-0004 (Excalidraw)

## Contexto

Um diagrama guardado como SVG, PNG ou `.excalidraw` é um arquivo binário ou quase-binário do ponto
de vista de revisão: o diff mostra coordenadas, estilo de serialização e ordem de atributos, não a
mudança conceitual. Pior para o usuário-alvo: um LLM que edita coordenadas erra em silêncio — o
resultado continua "válido" e fica visualmente quebrado.

A pergunta é onde colocar a autoridade. Três opções foram consideradas:

1. **SVG como fonte** — editável, mas o diff carrega geometria; toda mudança estrutural reescreve
   caminhos e ids.
2. **`.excalidraw` como fonte** — formato de interop, com campos derivados (`versionNonce`,
   `updated`, `boundElements`) que mudam sem mudança semântica.
3. **Formato textual próprio, de topologia e rótulos** — o autor declara *o que existe e como se
   liga*; geometria é calculada.

## Decisão

A fonte versionada é o `.akrc`: UTF-8, uma afirmação por linha, sem coordenadas obrigatórias.

**O `.akrc` é a verdade.** SVG, PNG, JPG, AVIF, `scene` JSON e `.excalidraw` são artefatos de build,
gerados e descartáveis. Nenhum deles é editado à mão e nenhum é commitado como fonte.

**A forma canônica é única e verificável.** `akr fmt` produz exatamente uma escrita para cada
documento: classes na ordem `chart`, `group`, `node`, `edge`, `note`; ordem de declaração preservada
dentro de cada classe; uma linha em branco entre classes não-vazias; um espaço como separador;
opções em ordem alfabética; strings sempre entre aspas; números com até duas casas, sem zeros à
direita; arquivo terminado por exatamente um `\n`. `akr fmt --check` sai 1 quando o arquivo difere e
é o que impede a forma canônica de degradar.

**O layout é um estágio separado e pinável.** A posição de um elemento só é declarada (`at=x,y`)
quando o autor quer fixá-la; nos demais casos ela é derivada de forma determinística (ADR-0003). O
pino é local: fixar um nó não congela os vizinhos, e layout por perfil sem `free` aceita `at` como
pinning parcial.

**A identidade para diff é estável.** `node`/`group`/`note` casam por `id`; `edge` casa por
`(from, to, ordinal)`, com `ordinal` contando duplicatas do mesmo par. Renomear um id é remoção +
adição — o diff diz isso em vez de esconder atrás de heurística.

**A gramática é fechada.** Conjunto de chaves por statement, conjunto de valores por chave e
mensagem de erro que lista o conjunto válido. Não há extensão por chave livre: `E002`/`E003` são a
resposta. A tabela de opções vive em um único lugar (`syntax::option_specs`) e alimenta parser,
`akr schema` e `akr prompt`.

## Consequências

**Positivas**

- Diff no Git mostra a mudança semântica (`+ node dlq "DLQ"`), não ruído de coordenada.
- Um LLM escreve topologia — tarefa em que é bom — e não geometria — tarefa em que erra em silêncio.
- O artefato pode ser regenerado em CI; revisar artefato binário deixa de ser necessário.
- `fmt` idempotente dá base para golden tests e para cache por hash do `.akrc`.

**Negativas / custos aceitos**

- O autor não consegue posicionar livremente sem aprender `at=` e `layout=free`.
- A forma canônica exige uma etapa de normalização antes de comparar; `fmt --check` entra no CI.
- Comentários viajam colados ao statement seguinte (sort estável do par), o que é uma escolha — um
  comentário não pode ficar "solto" no meio de uma classe sem virar âncora do próximo statement.

## Alternativas rejeitadas

- **YAML/JSON como fonte:** mais ruído estrutural (chaves, chaves de escape, vírgulas) por linha de
  conteúdo; a gramática de uma afirmação por linha é verificável por olho e por LLM.
- **Markdown com bloco embutido:** permite prosa e diagrama no mesmo arquivo, mas acopla o formato a
  um parser de Markdown e cria duas fontes de verdade para o mesmo conteúdo.
