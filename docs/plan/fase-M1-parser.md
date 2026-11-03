# Fase M1 — léxico, AST, parser e `check`

**Status:** concluída · **Depende de:** M0 · **Habilita:** M2, M3

## Entrega

| módulo | responsabilidade |
|---|---|
| `span.rs` | `Span { line, col }` 1-based, coluna em caracteres |
| `diag.rs` | `Code`, `Severity`, `Diagnostic`, `ALL[16]`, `render`, `to_json`, `sort` |
| `error.rs` | `Error`, `ErrorKind` (2/3/5/8/70), `Result<T, E = Error>` |
| `lex.rs` | tokenizador de linha com `TokKind`, strings escapadas, `Key`+valor, `->`, `Pair` |
| `syntax/mod.rs` | tipos de statement (`ChartStmt`…), `OptVal`, `ValueError`, `Options` |
| `syntax/specs.rs` | **as tabelas**: `option_specs(kind)`, `spec_for`, `valid_keys`, conjuntos fechados |
| `syntax/value.rs` | validação por spec (enum, number+faixa, token, string, bareword, idlist, pair, ident) |
| `parse/mod.rs` | `Document`, driver linha a linha, recuperação por linha |
| `parse/stmt.rs` | cabeça de cada statement (`chart`, `group`, `node`, `edge`, `note`) |
| `parse/opts.rs` | opções `chave=valor` |
| `ports/mod.rs` + `ports/fakes.rs` | `Fs`, `Git`, `Env`, `Logger` + `MemFs`, `FakeGit`, `FakeEnv`, `RecordingLogger` |

Na CLI: `cli.rs` (clap), `envelope.rs` (`{success, command, data?, error?, warnings}`),
`output.rs` (EPIPE ⇒ 0), `input.rs` (posicional/`-`/stdin/`--params`), `params.rs` (chave fora da
lista ⇒ exit 2 com a lista), `adapters.rs` (`StdFs`), `commands/check.rs`, `commands/schema.rs`,
`app.rs` (erro → exit code), `main.rs`.

## Decisões de execução

- **`diag.rs` como módulo próprio.** O plano lista `span`/`error`/`syntax`/`parse`; `Diagnostic` não
  cabia em nenhum sem criar dependência circular (`resolve` emite diagnósticos, `parse` também).
  `error.rs` fica só com erro de **processo** (exit code); `diag.rs`, com diagnóstico de **schema**.
- **`parse` é um diretório** (`mod`, `stmt`, `opts`): o arquivo único passava de 300 linhas.
- **`syntax` é um diretório** (`mod`, `specs`, `value`) pelo mesmo motivo: as tabelas de §3.2 somam
  mais de 120 linhas e não pertencem ao mesmo arquivo que a validação.
- **Recuperação na granularidade da linha.** `parse` devolve `(Document, Vec<Diagnostic>)` e nunca
  falha: um erro não esconde os seguintes, o que é o ponto do produto para um LLM.
- **A primeira afirmação é `chart`**; `E008` é emitido uma única vez (no primeiro statement fora de
  ordem) para não repetir a cada linha.
- **`--docs` e a chave `docs` de `--params` ficam para M7**, quando `doc.rs` existe. Aceitá-los agora
  seria aceitar uma flag sem efeito — pior que a ausência.
- **`--params` genérico**: `params::parse<T>` valida o conjunto de chaves **antes** de desserializar,
  para que o erro liste as válidas; `path` XOR `source`; posicional + `--params` ⇒ exit 2.
- **Testes de integração sem dependência extra**: `std::process::Command` sobre
  `env!("CARGO_BIN_EXE_akr")`, porque a tabela de dependências do ADR-0001 §5.1 é fechada.
- **`unwrap`/`expect` permitidos em testes** (`clippy.toml`): o ADR-0001 §4 proíbe no **domínio**;
  em teste, falhar com a linha apontada é o comportamento desejado.

## Verificação

```sh
make check                                     # verde (fmt, clippy -D warnings, test, 300 linhas)
cargo test -p akribie-core                     # 20 testes (lex 7, parse 13)
cargo test -p akribie-cli                      # 12 testes de borda de processo
printf 'node a\nchart v1 "x"\n' > /tmp/e.akrc
target/debug/akr check /tmp/e.akrc             # :1:1 E008 …  hint: …
printf 'chart v1 "x"\nnode a "A" shape=cilinder\n' | target/debug/akr check -
                                               # :2:18 E003 invalid-value  hint: … cylinder …
target/debug/akr schema | jq -r '.statements.chart.options[]|select(.key=="layout").values|join(",")'
                                               # flow,tree,mindmap,grid,stack,free
target/debug/akr check --params '{"source":"chart v1 \"x\"\n"}'      # exit 0
target/debug/akr check --params '{"source":"chart v1"}' x.akrc       # exit 2
```

Resultados observados:

| checagem | resultado |
|---|---|
| `make check` | verde |
| `check` de fonte válida por stdin | exit 0, stdout vazio |
| `check` com `shape=cilinder` | exit 8, `E003` + `hint` com `cylinder` |
| `check --json` inválido | `success:false`, `error.code:"schema"`, `data.diagnostics[1]` |
| `check --params` + posicional | exit 2 |
| `check --params '{"nope":1}'` | exit 2, lista `path, source, strict` |
| `schema` | 16 diagnósticos, `layout` com 6 valores |
| EPIPE em `schema` | exit 0 |
