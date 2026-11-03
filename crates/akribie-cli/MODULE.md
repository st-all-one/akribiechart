# MODULE — `akribie-cli`

Binário `akr`. Só borda: lê bytes, chama o núcleo, emite dados em `stdout` e log em `stderr`,
traduz erro em exit code. Nenhuma regra de diagrama mora aqui.

| módulo | responsabilidade |
|---|---|
| `main.rs` | ponto de entrada: `app::run(cli)` |
| `cli.rs` | `clap`: subcomandos e flags (help é a fonte das flags documentadas) |
| `app.rs` | despacho verbos → `commands`; lote (`--batch`) |
| `app/emit.rs` | `Mode` (`Json`/`Text`), envelopes, linhas de diagnóstico, `fail` |
| `adapters.rs` | implementações reais de `Fs` (cria diretórios), `Git` (`git show`) |
| `envelope.rs` | envelope `{success,command,data?,error?,warnings}` |
| `output.rs` | escrita em `stdout` com EPIPE = sucesso; `warn_line`/`error_line` |
| `input.rs` | origem do fonte (arquivo, `-`, ausente) e `--params` |
| `params.rs` | deserialização de `--params` por verbo, com chaves e conflitos |
| `commands/check.rs` | `check` e `check --batch` |
| `commands/fmt.rs` | `fmt` (`stdout`, `--write`, `--check`) |
| `commands/render.rs` | `render` (`svg`, `scene`, `excalidraw`, raster, `--at`, lote) |
| `commands/format.rs` | enum `Format` do `render` |
| `commands/import.rs` | `import` |
| `commands/diff.rs` | `diff` (par de arquivos e `--at`) |
| `commands/init.rs` | `init` (config, bloco em `AGENTS.md`, skill), idempotente |
| `commands/prompt.rs` | `prompt` compacto e `--long` (tabelas derivadas do núcleo) |
| `commands/schema.rs` | `schema` JSON derivado de `syntax`/`theme`/`diag`/`error` |

Exit codes: 0 ok · 1 `fmt --check`/`diff` · 2 uso · 3 ausente · 5 I/O · 8 schema · 70 interno.
`--json` emite **um** envelope de uma linha; lote emite JSONL por item.
