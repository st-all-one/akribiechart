---
id: decision_01dwvhi5
type: decision
statement: MCP tool failures return a result with isError:true; only an unknown tool name is a protocol error.
created_at: 2026-11-03T15:31:06.308Z
body_hash: bbee5396
schema_version: 3
tags: [mcp]
revision: 2
anchors: [crates/akribie-mcp/src/tools.rs]
---
## Por quê
O agente itera sobre fonte ruim; derrubar o servidor encerra a sessão inteira.
## Alternativas
Erro JSON-RPC `-32602` para fonte inválida; encerrar o processo.
## Consequência
`content[0].text` traz a primeira linha do diagnóstico e a próxima requisição é atendida.
