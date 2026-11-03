# Fase M8 — `init`, `prompt`, MCP e documentação final

**Status:** concluída · **Depende de:** M0–M7

## Entrega

| peça | responsabilidade |
|---|---|
| `commands/init.rs` (+ `init/tests.rs`) | `akribie.toml`, bloco em `AGENTS.md`, skill em `.agents/skills/akr/`, idempotente |
| `commands/prompt.rs` | `PROMPT_COMPACT` (constante) e `--long` (BNF + tabelas de `option_specs` + temas + códigos) |
| `akribie-mcp` (`main`, `protocol`, `tools`, `fs`, `base64`) | JSON-RPC 2.0 por stdio com 3 ferramentas |
| `docs/usage/00…07` | um guia por verbo, com flags reais |
| `docs/specs/{GRAMMAR,ARCHITECTURE,A11Y,DETERMINISM}.md` | contratos normativos |
| `crates/*/MODULE.md` | mapa de módulos por crate |
| `tests/golden/prompt-*.txt` | prompt byte-idêntico por versão |

## Decisões de execução

- **Idempotência é o critério de aceite do `init`.** `akribie.toml` existente não é reescrito sem
  `--force`; o bloco vive entre marcadores de linha inteira e é substituído no lugar; a skill só é
  tocada quando carrega `<!-- akribie:skill:version: 1 -->` — um arquivo de terceiro no mesmo
  caminho é **preservado** e reportado como tal.
- **Configuração gerada usa valores do léxico**, não strings TOML: `layout = flow`,
  `fontsize = 20`, e apenas `lang = "en"` entre aspas (texto). O arquivo é um subconjunto do TOML
  validado pelo **mesmo** léxico do `.akrc`; a primeira versão escrevia `layout = "flow"` e o
  próprio `check` recusava — corrigido e coberto por teste.
- **Saída do `init` é relatório, não silêncio.** Uma linha por arquivo (`novo`, `atualizado`,
  `inalterado`, `preservado`) e `próximos:` com o primeiro comando; sem isso o usuário não distingue
  "não fiz nada" de "não havia o que fazer".
- **Prompt compacto é constante; o `--long` é derivado.** A parte estável vira golden; as tabelas
  saem de `syntax::option_specs`, `theme` e `diag` — uma chave nova no núcleo aparece no prompt sem
  edição. Faixas abertas (`size`) aparecem como `range=1..`.
- **MCP trata falha como dado.** Fonte inválida vira `isError: true` com a **primeira linha** do
  diagnóstico em `content[0].text`; o servidor continua atendendo. Só nome de ferramenta
  desconhecido é erro de protocolo (`-32602`); JSON malformado, `-32700`; método desconhecido,
  `-32601`. EOF/EPIPE encerram com exit 0.
- **`chart_render` raster devolve bloco `image`** em base64 (mime correto), não um caminho: MCP é
  transporte de conteúdo, e o cliente não compartilha o filesystem do servidor.
- **Governança de dependências apertada.** `rustybuzz`/`ttf-parser` estão sem mantenedor; a medição
  migrou para `harfrust` (sucessor HarfBuzz) com **bytes idênticos** de SVG/PNG nos testes, e as
  duas advisories saíram do `deny.toml`. O `xtask` saiu do workspace (lock próprio), tirando do
  lockfile publicado o `bytes 1.9.0` vulnerável que o `woff2` exige. Resta um desvio documentado:
  `paste` (proc-macro do `rav1e`, encoder AVIF), sem código no binário.
- **Documentação viva por teste.** `docs/usage/*.md` e a ajuda de cada verbo são comparados em
  `akribie-cli/tests/docs.rs`: flag na ajuda e ausente no guia (ou vice-versa) quebra o CI.

## Verificação

| checagem | resultado |
|---|---|
| `akr init` duas vezes | todas as linhas `inalterado`; bloco não duplica; skill não muda |
| skill de terceiro no caminho | `preservado`, conteúdo intacto |
| `akr prompt` duas vezes | bytes idênticos; golden confere |
| `akr prompt --long` | compacto + BNF + tabelas + temas + códigos |
| `tools/list` | 3 ferramentas |
| `chart_check` inválido | `isError: true`, `E003` na primeira linha, servidor responde à próxima |
| `chart_render` PNG | bloco `image/png`, base64 com assinatura `\x89PNG` |
| `--json`/pipe/EPIPE | inalterados (T1–T3 da matriz) |
| `make ci` | `fmt`, clippy `-D warnings`, testes, tamanho, deny, typos — tudo verde |

| teste | garante |
|---|---|
| `akribie-cli/src/commands/init/tests.rs` (5 testes) | criação, idempotência, bloco preservado, skill de terceiro, `--force` |
| `akribie-cli/tests/init.rs` (2 testes) | processo real, arquivos, config lido pelo `check` |
| `akribie-cli/tests/prompt.rs` (2 testes) | golden compacto e `--long` |
| `akribie-cli/tests/docs.rs` | paridade ajuda × `docs/usage` |
| `akribie-mcp/tests/mcp.rs` (5 testes) | `tools/list`, `isError`, JSON malformado, `image`, `diff` |
| `akribie-mcp/src/base64.rs::tests` | vetores RFC 4648 |
| `scripts/check_file_length.sh` | ≤ 300 linhas por arquivo de produção |
