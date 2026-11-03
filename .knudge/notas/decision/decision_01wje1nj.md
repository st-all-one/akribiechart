---
id: decision_01wje1nj
type: decision
statement: xtask is excluded from the workspace so the published lockfile never contains bytes 1.9.0 from woff2.
created_at: 2026-11-03T15:31:06.434Z
body_hash: 84d616b0
schema_version: 3
tags: [deps]
revision: 2
anchors: [Cargo.toml]
---
## Por quê
`woff2` (só usado no `xtask`) puxa `bytes 1.9.0` com advisory; o binário publicado nunca o usa.
## Alternativas
Aceitar o advisory no binário; trocar o decodificador woff2.
## Consequência
`xtask` tem `Cargo.lock` próprio; sobra um desvio documentado, `paste` (proc-macro do `rav1e`).
