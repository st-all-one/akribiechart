---
id: decision_01qfnznr
type: decision
statement: "Batch JSONL is best-effort: exit 0 when at least one item passes, exit 8 when every item fails."
created_at: 2026-11-03T15:31:06.031Z
body_hash: 9982b01c
schema_version: 3
tags: [batch]
revision: 2
anchors: [crates/akribie-core/src/pipeline.rs]
---
## Por quê
Um lote de agente costuma ter ruído; parar no primeiro item ruim desperdiça o resto.
## Alternativas
Abortar no primeiro erro; exit 1 por item falho.
## Consequência
Uma linha JSONL por item, na ordem do arquivo; exit 0 se ao menos um passa, 8 se todos falham.
