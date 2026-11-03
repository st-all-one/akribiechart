# Protocolo de revisão

Norma de processo do akribie. Deriva do ADR-0001 §4 e §5. Vale para toda mudança, sem exceção por
tamanho: um ajuste de mensagem de erro é mudança de superfície.

## 1. O que conta como mudança de superfície

Uma mudança é de superfície quando altera qualquer um destes contratos observáveis:

| contrato | exemplos |
|---|---|
| verbo / flag / argumento | flag nova, default alterado, exit code alterado |
| gramática do `.akrc` | chave nova, valor novo em conjunto fechado, regra de resolução |
| forma canônica | ordem de classes, separador, formatação de número |
| bytes de artefato | SVG, `scene` JSON, `.excalidraw` |
| texto de diagnóstico | código, mensagem, `hint` |
| arquivo escrito | `akribie.toml`, bloco em `AGENTS.md`, `SKILL.md` instalada |

Mudança interna (refatorar `paint.rs`, otimizar `sketch.rs`) **não** é de superfície, desde que os
bytes emitidos permaneçam idênticos — e é exatamente isso que os goldens verificam.

## 2. Ritual obrigatório

Toda mudança de superfície atualiza, **no mesmo commit**:

1. `docs/usage/<verbo>.md` — o guia do verbo afetado (com exemplo executável).
2. `docs/plan/02-matriz-aceitacao.md` — a linha correspondente, com o teste que a cobre.
3. O golden correspondente (`crates/akribie-cli/tests/golden/`) ou o teste de contrato.
4. `CHANGELOG.md`, na seção `Não publicado`, sob `Adicionado`/`Alterado`/`Removido`/`Corrigido`.
5. `AGENTS.md`, `llms.txt` e `SKILL.md` quando a mudança afeta como um agente usa o binário
   (verbo novo, flag nova, regra de correção de erro, exit code novo).
6. As specs normativas afetadas: `docs/specs/GRAMMAR.md` (gramática), `A11Y.md` (SVG/contraste),
   `DETERMINISM.md` (contrato de bytes), `ARCHITECTURE.md` (módulos).
7. O ADR, quando a mudança contraria ou substitui uma decisão existente — ADR novo com `Substitui:`,
   nunca edição silenciosa do texto aceito.

Commit que muda superfície sem os itens acima é rejeitado na revisão, independentemente de o código
estar correto.

## 3. Portão mecânico

```sh
make check     # fmt + clippy -D warnings + test + file-length
make ci        # check + deny + typos   (quando deny/typos instalados)
```

Nenhum `#[allow]` sem `reason = "..."`. Nenhum arquivo de produção acima de 300 linhas. Nenhuma
dependência fora da tabela do ADR-0001 §5.1.

## 4. Paridade entre o prompt vivo e a documentação estática

`akr prompt` é a superfície que um agente lê em tempo de execução; `docs/` é a que um humano lê. As
duas descrevem o mesmo contrato e precisam concordar. A revisão verifica, mecanicamente:

```sh
akr prompt > /tmp/prompt.txt
grep -o -- '--[a-z-]*' /tmp/prompt.txt | sort -u          # flags citadas no prompt
grep -o -- '--[a-z-]*' docs/usage/*.md | sort -u          # flags citadas nos guias
```

Toda flag citada em um lado precisa existir do outro. O mesmo vale para códigos de diagnóstico
(`E00x`/`W00x`) e exit codes. `akr prompt` é byte-idêntico por versão do binário (golden em
`crates/akribie-cli/tests/golden/prompt*.txt`).

## 5. Determinismo na revisão

Antes de aprovar mudança que toca `layout/`, `paint/`, `emit/` ou `sketch/`:

```sh
akr render fixtures/retry.akrc > /tmp/a.svg
akr render fixtures/retry.akrc > /tmp/b.svg
cmp /tmp/a.svg /tmp/b.svg          # precisa ser idêntico
git diff --stat -- '*.svg'        # artefato não é commitado; nada deve aparecer
```

Se os bytes mudaram, o golden é atualizado **e** o `CHANGELOG.md` diz que o SVG mudou. Mudança de
bytes sem entrada no changelog é regressão de contrato.

## 6. Revisão de código: perguntas obrigatórias

1. O domínio importa `std::fs`, `std::env` ou `std::process::Command`? (proibido — ADR-0001 §3)
2. Há iteração sobre `HashMap`/`HashSet` no caminho de emissão? (proibido — ADR-0001 §2)
3. A saída é função pura da entrada, ou depende de relógio, ambiente ou ordem de alocação?
4. O erro novo tem código, posição, o que se esperava e a correção? Um LLM consegue consertar a
   linha sem adivinhar?
5. A mensagem cabe em pipe (`stdout` = dados, `stderr` = logs) e não depende de cor?
6. O teste cobre o comportamento ou apenas a fiação? Se ele passaria com a implementação trocada por
   um stub, não serve.
7. O arquivo passou de 300 linhas? Se sim, dividir em vez de pedir exceção.

## 7. Ordem de merge

`docs` + `código` + `teste` no mesmo commit temático. Se a mudança for grande (fase M), o commit
pode ser dividido por módulo, mas a última unidade do conjunto precisa deixar `make check` verde e a
matriz consistente. **Nunca** deixar documentação "para o commit seguinte".
