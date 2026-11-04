# AGENTS.md — akribie

Guia de contribuição para agentes e humanos. As regras abaixo são normativas e derivam dos ADRs.

## O que é este repositório

`akr` compila um arquivo-texto de topologia (`.akrc`) em diagramas determinísticos e acessíveis:
SVG por padrão; PNG/JPG/AVIF sob demanda; `scene` JSON para depuração; `.excalidraw` para interop.
Binário único, local, offline. O usuário é um LLM: ele escreve **topologia e rótulos**, nunca
coordenadas.

## Regras de ouro

1. **Simplicidade primeiro.** Nada de servidor, daemon, banco, `tokio`, `async` ou threads no
   domínio. Infraestrutura precisa pagar por si no caso local.
2. **Determinismo é contrato.** Mesma entrada + mesma versão ⇒ mesmos bytes de SVG, `scene` e
   `.excalidraw`.
   Proibido no domínio: `SystemTime::now`, `env::var`, `rand`, iteração sobre `HashMap`/`HashSet`.
   Sementes sempre derivadas do id (`sketch::seed_for`).
3. **Portas, não I/O direto.** O domínio só conhece `ports::{Fs, Git}`; fakes em `ports::fakes`.
   Os adaptadores reais (`StdFs`, `StdGit`) vivem em `crates/akribie-adapters`; CLI e MCP são as
   únicas camadas que os montam (ADR-0006 §3).
4. **`make check` é o portão único.** `fmt` + `clippy -D warnings` + `test` + limite de linhas +
   `cargo tree` sem runtime assíncrono. Nenhum `#[allow]` sem `reason = "..."`. Nenhum arquivo de
   produção acima de 300 linhas (`crates/**/src` e `xtask/src`).
5. **Documentação é entregável.** Superfície, docs e testes mudam no mesmo passo. Um verbo novo sem
   `docs/usage/`, sem linha na matriz e sem golden está incompleto.

## Fluxo de trabalho

```sh
make check                 # antes de qualquer entrega
make font                  # regenera assets/Excalifont-Regular.ttf (idempotente)
cargo test -p akribie-core # testes de domínio (rápidos, sem I/O real)
```

Ordem de leitura para entender o projeto: `docs/plan/00-panorama.md` →
`docs/adr/ADR-0001-simplicidade-determinismo-di.md` → `docs/specs/ARCHITECTURE.md` →
`docs/review-protocol.md`.

## Mapa do repositório

| caminho | conteúdo |
|---|---|
| `crates/akribie-core/src/` | domínio puro (lex, parse, resolve, layout, paint, emit, diff) |
| `crates/akribie-adapters/src/` | adaptadores reais das portas (`StdFs`, `StdGit`) — ADR-0006 §3 |
| `crates/akribie-cli/src/` | bin `akr`: envelope, comandos, apresentação |
| `crates/akribie-mcp/src/` | servidor MCP (JSON-RPC sobre stdio) |
| `xtask/` | utilitário de build (fonte woff2 → ttf) |
| `assets/` | `Excalifont-Regular.ttf` commitado, OFL e `FONT.lock` |
| `docs/adr/` | decisões normativas (não editar ADR aceito; criar substituto) |
| `docs/plan/` | panorama, fases e matriz de aceitação |
| `docs/specs/` | gramática, arquitetura, acessibilidade, determinismo |
| `docs/usage/` | um guia por verbo, com exemplo executável |
| `docs/exemplos/` | diagramas reais renderizados (PNG) com o `.akrc` de origem |
| `fixtures/reais/` | os quatro `.akrc` de `docs/exemplos/` — regressão de renderização |
| `_REF/` | referências externas — **nunca editar** |

## Coisas que quebram em silêncio (leia antes de editar)

- **Ordem de emissão.** SVG e `scene` são contratos de bytes. Trocar a ordem de um `Vec` de `IndexMap`
  por `HashMap` muda a saída sem mudar o desenho.
- **Números.** Todo número no SVG/scene passa por `num::format` (2 casas, zeros removidos). `f32::to_string`
  direto produz `0.30000001192092896` e quebra o golden.
- **Fonte.** Medição, SVG e raster usam a **mesma face embutida** (`assets/Excalifont-Regular.ttf`).
  Nunca `fontdb` do sistema: o PNG deixaria de corresponder ao SVG.
- **Cyclomaticidade.** Recursão em árvore/mapa mental pode estourar em grafo com ciclo; o layout
  precisa de conjunto de visitados.
- **`--params`.** Substitui as flags do verbo; combinar com posicional é exit 2.

## Diagnósticos

Todo erro tem código (`E00x`/`W00x`), posição `arquivo:linha:col`, o que se esperava e a correção em
`hint:`. "syntax error" não é mensagem aceitável: um agente precisa corrigir a linha sem adivinhar.
`check` sai 0 com só warnings; `--strict` promove warning a erro (exit 8).

## Commits

Um commit temático = código + teste + docs. `CHANGELOG.md` na seção `Não publicado` sempre que os
bytes emitidos ou a superfície mudarem. Ver `docs/review-protocol.md` §2.

<!-- knudge:start -->
<!-- knudge:version: 4 -->
# knudge — project memory

`notas/` is the source of truth; index/graph/embeddings are **derived** — never edit `notas/` by hand.

Loop: `kd prime` (protocol; `--long` = full) → `kd ask "<q>"` before acting → `kd write` each
learning → `kd rewind` for handoff (`--with-activity` adds recent agent activity).
- `kd ask` dedups: score <0.75 create · 0.75–0.92 merge · ≥0.92 reject.
- One assertion per note; anchor code with `--anchor PATH` (glob `src/**`).
- Work lives in `kd task`: `epic ⊃ {issue ⊃ task | task}`; `--scope epic|issue|task`,
`--kind task|error|question|risk|decision`; `kd task list --ready`; close with `--outcome`.
- `kd write --type task` is rejected; `ask` excludes work unless `--with-task`.
- Health: `kd doctor` (`--fix`); `kd maintenance learn|compact|prune` propose only.
- Embeddings are derived: `kd drain --status` / `--digest`.
- Agent memory shards: `kd ai --memory|--session "<q>"` (read-only NDJSON,
`.knudge/ai/<area>/<agent_id>.jsonl`); `--sort time`, `--agent-id/--agent-role`,
`--recap` (compact recap); filter `ask` with `--agent/--activity/--agent-id/--agent-role`.
- stdout = data, stderr = logs; `--json` = {success, command, data?, error?, warnings?}.
- If `kd` is missing on the host: `curl -fsSL https://raw.githubusercontent.com/st-all-one/knudge/main/install.sh | bash` (installs `~/.local/bin`), then `kd prime`.
Config: `.knudge/config.toml`.
<!-- knudge:end -->
