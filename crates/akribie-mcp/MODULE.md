# MODULE — `akribie-mcp`

Servidor MCP (JSON-RPC 2.0) sobre `stdin`/`stdout`, uma mensagem por linha. Reusa `akribie-core`;
não duplica regra de diagrama.

| módulo | responsabilidade |
|---|---|
| `main.rs` | laço de leitura/escrita por linha; EOF e EPIPE encerram com exit 0 |
| `protocol.rs` | parse do JSON-RPC, despacho (`initialize`, `ping`, `tools/*`), erros `-32700`/`-32601`/`-32602` |
| `tools.rs` | `tools/list` e `tools/call`; falha de execução vira `isError: true` |
| `fs.rs` | `Fs` real para `path` (mesma semântica da CLI) |
| `base64.rs` | base64 com padding para `content: image` |

Ferramentas: `chart_check {source|path, strict?}`, `chart_render {source|path, format?, scale?}`,
`chart_diff {old_source, new_source}`. O servidor nunca cai por causa de uma fonte ruim: o erro é
conteúdo (`content[0].text` = primeira linha do diagnóstico) e a próxima linha é atendida.
