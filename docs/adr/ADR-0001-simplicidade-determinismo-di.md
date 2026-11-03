# ADR-0001 — Simplicidade, determinismo e injeção de dependência

- **Status:** Accepted
- **Data:** 2026-11-03
- **Decisores:** mantenedores do akribie
- **Substitui:** —

## Contexto

`akr` é um compilador de diagramas usado sobretudo por LLMs e agentes. O usuário escreve topologia
e rótulos; o binário produz SVG, PNG/JPG/AVIF, `scene` JSON e `.excalidraw`. Não há colaboração em
tempo real, não há serviço hospedado, não há estado global.

Esse perfil de uso impõe três exigências que competem entre si:

1. **Custo de operação zero.** O binário precisa rodar em CI, em contêiner descartável e na máquina
   do agente, sem instalação de serviço, sem migração de banco, sem porta aberta.
2. **Reprodutibilidade.** O mesmo `.akrc` com o mesmo binário precisa produzir os mesmos bytes. Sem
   isso, o Git mostra ruído a cada build e o cache de artefatos é inútil.
3. **Testabilidade.** O domínio tem seis estágios (lex, parse, resolve, layout, paint, emit) e
   precisa ser testável sem tocar o sistema de arquivos, sem rede e sem relógio.

As decisões abaixo são normativas: valem para todo código presente e futuro do workspace. Cada uma
tem guarda mecânica — lint, script ou teste — porque regra sem guarda é sugestão.

## Decisão

### 1. Simplicidade primeiro

Toda infraestrutura precisa pagar por si **no caso local**. É proibido:

- servidor HTTP, daemon, socket, IPC, fila;
- banco de dados, ORM, migração, cache externo;
- `tokio`, `async`/`await` e runtime assíncrono em qualquer caminho de execução;
- threads, pools e execução paralela no domínio (o raster pina a codificação em uma thread
  justamente para manter o determinismo do §2).

O binário lê bytes, computa, escreve bytes e termina. O único I/O externo é `std::fs`, `git show`
e a escrita em `stdout`. Quando uma funcionalidade exigir infraestrutura, ela não entra na v1.

**Guarda:** `[workspace.lints]` proíbe as crates relevantes por ausência na tabela de dependências;
`cargo tree` em CI falha se aparecer `tokio`, `async-std` ou `rayon`.

### 2. Determinismo é contrato

Mesma entrada + mesma versão do binário ⇒ **mesmos bytes** de SVG e de `scene` JSON. O raster é
determinístico dentro de uma versão (`--scale` fixo, uma thread de codificação), mas não é contrato
de bytes entre versões — os codecs mudam.

É proibido no domínio:

| construto proibido | motivo | substituto |
|---|---|---|
| `SystemTime::now` / `Instant::now` | muda a saída entre execuções | nada (o scene não carrega tempo) |
| `std::env::var` / `vars` | muda com o ambiente | porta `Env`, usada só pela CLI |
| `rand` / `thread_rng` | semente não reproduzível | `sketch::seed_for(id)` (FNV-1a 32 do id) |
| iteração sobre `HashMap` / `HashSet` | ordem de iteração não especificada | `IndexMap`, `Vec`, `BTreeMap` |
| `f32`/`f64` com `to_string` | formatação depende da versão da std | `fn num(v: f32) -> String` (2 casas, zeros removidos) |

A ordem de tudo que é emitido vem do arquivo (ordem de declaração) ou do id, nunca do endereçamento
de memória. A única aleatoriedade do sistema é o traço à mão do `sketch`, semeado pelo id do
elemento — logo, estável entre execuções e entre máquinas.

**Guarda:** `clippy.toml` fixa `disallowed-methods` para `SystemTime::now`, `Instant::now`,
`env::var`, `env::vars`; `disallowed-types` para `HashMap` e `HashSet`; teste de determinismo
renderiza o mesmo fixture duas vezes e compara bytes.

### 3. Injeção de dependência por portas

O domínio expõe funções puras. Toda interação com o mundo atravessa uma trait em `ports`:

| porta | métodos | implementação real | fake |
|---|---|---|---|
| `Fs` | `read`, `read_bytes`, `write`, `exists` | `StdFs` (CLI) | `MemFs` |
| `Git` | `show(rev, path)` | `StdGit` (CLI) | `FakeGit` |
| `Env` | `var(key)` | `StdEnv` (CLI) | `FakeEnv` |
| `Logger` | `warn`, `info` | `TracingLogger` (CLI) | `RecordingLogger` |

Os comandos recebem `&dyn Fs` / `&dyn Git`; **a CLI é a única que monta `StdFs`/`StdGit`/`StdEnv`/
`TracingLogger`**. Nenhum módulo de `akribie-core` importa `std::fs`, `std::env` ou
`std::process::Command`.

**Guarda:** `disallowed-types`/`disallowed-methods` para as APIs cruas; `ports::fakes` é a única
forma de testar I/O; a compilação de `akribie-core` sem a CLI é verificada por `cargo check -p
akribie-core`.

### 4. Rigor de processo

`make check` é o **portão único**: `fmt` + `clippy -D warnings` + `test` + limite de linhas. Não há
exceção por arquivo, não há "isso é só um teste", não há `#[allow]` sem `reason = "..."`.

- `#![forbid(unsafe_code)]`; `#![warn(missing_docs)]`.
- Proibidos no domínio: `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `dbg!`,
  `print_stdout`, `print_stderr`, `process::exit`, `indexing_slicing`, `as_conversions`,
  `wildcard_imports`, `HashMap`/`HashSet`/`Rc`/`RefCell`, `LinkedList`.
- Aritmética com `checked_*` / `saturating_*`; `overflow-checks = true` em `dev` e `release`.
- Arquivos de produção ≤ **300 linhas**; acima disso, o módulo é dividido, não autorizado.

**Guarda:** `clippy.toml` + `[workspace.lints]` (mesmo formato de `_REF/knudge/Cargo.toml`) e
`scripts/check_file_length.sh` no `Makefile`.

### 5. Documentação é entregável

A documentação não é subproduto: é artefato versionado com critério de pronto. `docs/` contém ADRs,
o plano em fases separadas, o protocolo de revisão, as specs normativas e um guia por verbo.
`AGENTS.md`, `llms.txt`, `SKILL.md` e `CHANGELOG.md` ficam na raiz.

Superfície, documentação e testes mudam **no mesmo passo**. Um verbo novo sem `docs/usage/`, sem
linha na matriz de aceitação e sem golden correspondente está incompleto, mesmo que compile.

**Guarda:** `docs/plan/02-matriz-aceitacao.md` amarra cada linha a um teste existente;
`docs/review-protocol.md` define o ritual e é citado em `AGENTS.md`.

## Consequências

**Positivas**

- Binário único, sem dependência de runtime: instalação por cópia, uso em CI sem serviço.
- SVG e `scene` JSON viram contrato de bytes; diff de artefato é ruído zero.
- Todo o domínio é testável com `MemFs`/`FakeGit`, sem mock de framework e sem rede.
- O custo de trocar uma dependência (por exemplo o motor de layout) fica confinado a um arquivo.

**Negativas / custos aceitos**

- Sem paralelismo: AVIF é lento e assim permanece na v1 (medir antes de otimizar).
- Sem cache entre execuções: cada render recalcula layout e raster.
- O `sketch` (traço à mão) é implementação própria; não usamos `roughr`, cuja API é instável e cujo
  gerador não é garantidamente determinístico entre versões.
- A regra de 300 linhas força divisão de módulos que, de outro modo, ficariam coesos em um arquivo.

## Referências de execução

- `_REF/knudge/AGENTS.md` §0 (regras de ouro) e §3 (portas e determinismo).
- `_REF/knudge/crates/knudge-core/MODULE.md` (contrato de módulo do core).
- `_REF/knudge/Makefile` (alvos `check`/`ci`).
- `_REF/knudge/Cargo.toml` §`[workspace.lints]` e `_REF/knudge/clippy.toml` (formato das guardas).
