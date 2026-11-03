---
id: fact_01gx4ywo
type: fact
statement: docs/usage flags must match each verb's --help; tests/docs.rs enforces both directions.
created_at: 2026-11-03T15:31:06.157Z
body_hash: d56bc35e
schema_version: 3
tags: [docs]
anchors: [crates/akribie-cli/tests/docs.rs]
---
Every flag in --help must appear in the guide, and every --flag in the guide must exist in some verb's help. Adding a flag without docs breaks CI.