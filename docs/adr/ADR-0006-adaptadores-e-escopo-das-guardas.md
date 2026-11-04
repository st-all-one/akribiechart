# ADR-0006 — Adaptadores compartilhados e escopo real das guardas

- **Status:** Accepted
- **Data:** 2026-11-03
- **Decisores:** mantenedores do akribie
- **Substitui:** ADR-0001 §3 (injeção de dependência por portas) e §4 (rigor de processo), na
  parte que eles descrevem errado
- **Relaciona:** ADR-0001 (simplicidade, determinismo e DI)

## Contexto

A revisão da 0.1.2 encontrou duas divergências entre o ADR-0001 e o que o repositório faz. Nenhuma
das duas é regressão de código: são decisões que mudaram na prática e ficaram documentadas no texto
antigo.

**Portas e adaptadores.** O ADR-0001 §3 declara quatro portas (`Fs`, `Git`, `Env`, `Logger`) e diz
que "a CLI é a única que monta `StdFs`/`StdGit`/`StdEnv`/`TracingLogger`". Na prática:

1. `Env` e `Logger` nunca ganharam consumidor no domínio. Os avisos do `akr` são diagnóstico puro
   (`Diagnostic` com código, posição e `hint`) escrito pela CLI em `stderr` com tratamento de
   `EPIPE`; nada no modelo lê variável de ambiente — a CLI lê `std::env::args_os` e só.
2. O servidor MCP também precisa de `Fs`, e não podia montar o da CLI: `crates/akribie-mcp/src/fs.rs`
   era uma cópia do adaptador da CLI, com a mesma semântica e uma mensagem de erro diferente. Duas
   cópias da mesma fronteira divergem na primeira correção.

**Guardas em teste.** O ADR-0001 §4 diz "não há exceção por arquivo, não há 'isso é só um teste'",
listando `unwrap`/`expect`/`panic!`/indexação como proibidos. O `clippy.toml` já faz o contrário, e
com razão: `allow-unwrap-in-tests`, `allow-expect-in-tests` e `allow-indexing-slicing-in-tests` estão
ligados. Uma asserção de teste precisa falhar na linha exata com a mensagem do valor observado; um
`Result` propagado com `?` transforma a falha em "test failed" sem dizer o que quebrou. O texto do ADR
descrevia uma disciplina que o repositório decidiu não seguir.

## Decisão

### §3 — Adaptadores compartilhados, portas mínimas

Os adaptadores reais vivem em **`crates/akribie-adapters`** (`StdFs`, `StdGit`), crate sem estado e
sem dependência além de `akribie-core`. `akribie-cli` e `akribie-mcp` são as duas camadas que os
montam; nenhum outro ponto do workspace fala com o sistema operacional.

As portas do domínio são exatamente as que têm consumidor:

| porta | métodos | implementação real | fake |
|---|---|---|---|
| `Fs` | `read`, `read_bytes`, `write`, `exists` | `StdFs` (`akribie-adapters`) | `MemFs` |
| `Git` | `show(rev, path)` | `StdGit` (`akribie-adapters`) | `FakeGit` |

`akribie-core` continua sem `std::fs`, `std::env` ou `std::process::Command`; a fronteira apenas
deixou de ser um arquivo dentro da CLI e virou uma crate. Porta sem consumidor no domínio não é
porta: `Env` e `Logger` foram removidas em vez de mantidas como superfície morta (ADR-0001 §1,
"simplicidade primeiro").

### §4 — O que as guardas cobrem

Em **produção** (`cfg(not(test))`), seguem proibidos `unwrap`, `expect`, `panic!`, `todo!`,
`unimplemented!`, `dbg!`, indexação/fatiamento e `as`; `arithmetic_side_effects` e `string_slice`
passam a valer também, o que obriga `saturating_*`/`checked_*` em contador, dimensão e índice. Onde o
limite é estrutural (índice de laço sobre `0..len`), a exceção é um `#[expect(..., reason = "…")]`
pontual, nunca um `#[allow]` sem motivo.

Em **teste**, `unwrap`, `expect` e indexação de fatia literal são permitidos: a asserção precisa
falhar alto e apontar a linha. `panic!` continua proibido em teste — `assert!`/`assert_eq!` dizem o
mesmo com o valor esperado e o obtido.

As guardas são impostas por `clippy.toml` (`allow-*-in-tests`), por `[workspace.lints]` e pelo
`#![cfg_attr(not(test), deny(clippy::arithmetic_side_effects, clippy::string_slice))]` no topo de
`akribie-core`, `akribie-cli`, `akribie-mcp` e `akribie-adapters` — o `cargo clippy --all-targets`
do `make check` cobre os dois mundos.

## Consequências

**Positivas**

- Uma cópia de cada adaptador. Corrigir uma mensagem de I/O corrige os dois binários.
- A fronteira com o sistema operacional tem nome e lugar: `grep -rn "std::fs" crates/` só encontra
  `crates/akribie-adapters/src/`.
- `arithmetic_side_effects` em produção encontra estouro de contador e de índice que a revisão
  manual não encontra.

**Negativas**

- Uma crate a mais no workspace; `cargo build -p akribie-cli` passa a compilar `akribie-adapters`
  (custo desprezível: sem dependências próprias).
- Remover `Env`/`Logger` quebra quem implementava essas traits — não há esse consumidor no
  workspace nem na API pública publicada (`publish = false`).

**Neutras**

- O texto do ADR-0001 §3/§4 permanece no repositório como registro histórico; este ADR é a leitura
  vigente. `_REF/` e ADRs aceitos não são editados.
