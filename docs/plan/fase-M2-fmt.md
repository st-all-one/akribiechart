# Fase M2 — forma canônica e `akr fmt`

**Status:** concluída · **Depende de:** M1 · **Habilita:** M3 (e todo golden de render)

## Entrega

| peça | responsabilidade |
|---|---|
| `core/src/fmt.rs` | `format(&Document) -> String`: a **única** escrita para cada documento |
| `cli/src/commands/fmt.rs` | `akr fmt [FILE|-] [--write] [--check] [--params]` |
| `tests/golden/fmt_canonical.akrc` | entrada fora de ordem, comentários, opções desalinhadas |
| `tests/golden/fmt_expected.akrc` | saída canônica esperada, byte a byte |
| `cli/tests/fmt.rs` | 11 testes de contrato de borda |

## A forma canônica

1. Classes na ordem `chart`, `group`, `node`, `edge`, `note`; dentro da classe, ordem de declaração.
2. Exatamente uma linha em branco entre classes não vazias; nenhuma dentro de uma classe.
3. Comentários colados ao statement seguinte; sem statement seguinte, no fim do arquivo.
4. Um espaço como separador — **sem alinhamento de colunas** (alinhar produz diff ruidoso em `git`).
5. Strings sempre entre aspas, ids nunca.
6. Opções em ordem **alfabética** de chave.
7. Números com até duas casas, sem zeros à direita (`num::format`).
8. Arquivo terminado por exatamente um `\n`; documento vazio ⇒ saída vazia.

## Decisões de execução

- **Itens 1 e 6 são do ADR-0002, não inventados aqui**: a ordem alfabética de chaves e a ordem de
  classes tornam o diff de duas versões do mesmo diagrama o menor possível, que é o critério de
  aceitação de `akr diff` (M6).
- **`fmt` ignora semântica.** Ele chama só `parse`; `E004`/`E005`/`E006` não impedem formatar. Um
  arquivo com id duplicado ainda tem forma canônica, e reescrevê-lo não deve ser bloqueado por um
  problema que `check` já reporta com posição. Regra: `fmt` recusa apenas o que **não é parseável**
  (`E001`–`E003`, `E007`–`E009`).
- **`--check` sai 1, não 8.** Divergência de forma não é schema inválido: 1 é o código de
  `exit_fmt_check` e permite usar `akr fmt --check` em pre-commit sem máscara de "erro de conteúdo".
- **`--write` exige arquivo.** Escrever a saída em `stdout` já é o comportamento padrão; `--write`
  em `stdin` não teria destino e é uso inválido (exit 2).
- **`--write` e `--check` são exclusivos**, inclusive quando vêm por `--params`.
- **Comentário órfão não é descartado.** `Document::trailing_comments` guarda o que vem depois do
  último statement; sem isso `fmt` apagaria comentários — perda de dado, não formatação.

## Verificação

```sh
make check                                    # verde
cargo test -p akribie-cli --test fmt          # 11 testes
target/debug/akr fmt crates/akribie-cli/tests/golden/fmt_canonical.akrc | diff - crates/akribie-cli/tests/golden/fmt_expected.akrc
target/debug/akr fmt --check crates/akribie-cli/tests/golden/fmt_expected.akrc   # exit 0
target/debug/akr fmt --write /tmp/fora.akrc   # reescreve no lugar
```

Resultados observados:

| checagem | resultado |
|---|---|
| golden fora de ordem | idêntico a `fmt_expected.akrc` |
| `fmt` sobre a própria saída | idêntico (idempotente) |
| `--check` em forma canônica | exit 0, stdout vazio |
| `--check` em forma divergente | exit 1 |
| `--write` | arquivo alterado no lugar; stdout vazio |
| `--write` sem arquivo | exit 2 |
| `fmt` de fonte inválida | diagnósticos em `stdout`, exit 8 |
