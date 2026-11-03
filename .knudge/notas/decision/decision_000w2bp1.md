---
id: decision_000w2bp1
type: decision
statement: akr init is idempotent via version markers and preserves third-party files instead of overwriting them.
created_at: 2026-11-03T15:31:06.376Z
body_hash: e465831a
schema_version: 3
tags: [init]
revision: 2
anchors: [crates/akribie-cli/src/commands/init.rs]
---
## Por quê
`init` roda em repositórios existentes; sobrescrever instruções do usuário é perda de dados.
## Alternativas
Sobrescrever sempre; recusar quando o arquivo não tem marcador.
## Consequência
`akribie.toml` só muda com `--force`; `AGENTS.md` recebe bloco delimitado; skill sem marcador é preservada.
