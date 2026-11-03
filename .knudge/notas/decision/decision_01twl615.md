---
id: decision_01twl615
type: decision
statement: Text shaping uses harfrust; rustybuzz and ttf-parser were removed because they are unmaintained.
created_at: 2026-11-03T15:31:05.919Z
body_hash: f013b690
schema_version: 3
tags: [shaping]
revision: 3
anchors: [crates/akribie-core/src/text.rs]
---
## Por quê
rustybuzz e ttf-parser ficaram sem mantenedor e carregavam a versão vulnerável de `bytes`.
## Alternativas
Permanecer em rustybuzz e aceitar o advisory; usar fontdue (não faz shaping completo).
## Consequência
Bytes de SVG/PNG idênticos aos goldens; `skrifa` entra pela harfrust e o `deny` fica limpo.
