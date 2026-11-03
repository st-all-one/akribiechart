<div align="center">

# akribie

Diagramas determinísticos e acessíveis a partir de texto — topologia é a fonte, bytes são o contrato.

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-2b3a42?style=for-the-badge)](./LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.99%2B-DEA584?style=for-the-badge&logo=rust&logoColor=000)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-JSON--RPC%202.0-6E56CF?style=for-the-badge)](docs/usage/06-init-prime.md)
[![Version](https://img.shields.io/badge/version-0.1.2-009739?style=for-the-badge)](CHANGELOG.md)

</div>

O akribie compila uma fonte textual `.akrc` em diagramas determinísticos e acessíveis: SVG por
padrão; PNG/JPG/AVIF, `scene` JSON e `.excalidraw` sob demanda. O binário **`akr`** é a CLI;
**`akribie-mcp`** expõe `chart_check`, `chart_render` e `chart_diff` via MCP (JSON-RPC 2.0 sobre
`stdio`). Sem servidor, banco, daemon ou rede: um binário Rust, local e offline.

Você escreve **topologia e rótulos**; geometria, cor, texto alternativo e bytes são derivados.

```akrc
chart v1 "Política de retry" direction=lr layout=flow lang=pt-BR

node api "API Gateway" fill=blue
node retry "Retry ×3" fill=amber shape=cloud
edge api -> retry "5xx"
edge retry -> api "backoff" curve=arc style=dashed
```

## Instalação

Release pré-compilado, verificado por SHA-256, que instala `akr` e `akribie-mcp` em `~/.local/bin`:

```bash
curl --proto '=https' \
  --tlsv1.2 \
  --show-error \
  --fail \
  https://raw.githubusercontent.com/st-all-one/akribiechart/main/install.sh \
  | bash
```

Versão fixa: `... | VERSION=v0.1.2 bash`. Outro destino: `... | INSTALL_DIR=/usr/local/bin bash`.
Sem editar os rc files: `... | AKR_NO_PATH=1 bash`. A partir do código-fonte:
`./install.sh --from-source` ou `make release && make install`.

O Release publica seis alvos — `x86_64`/`aarch64` em Linux musl (estático), macOS e Windows — como
`akribie-<versão>-<target>.{tar.gz,zip}` + `sha256sums.txt`. `--uninstall` move os binários para um
lixo recuperável em `~/.cache/akribie/trash`.

`assets/Excalifont-Regular.ttf` e `assets/LiberationSans-Regular.ttf` são commitados; `make font`
regenera a Excalifont a partir do woff2 pinado e confere o SHA-256 de `assets/FONT.lock`.

## Uso

```bash
# check — valida e aponta a linha; silencioso quando está tudo certo (exit 8 em erro)
akr check chart.akrc
akr check chart.akrc --strict          # warning vira erro
akr check --batch lote.jsonl           # lote JSONL, best-effort

# render — SVG em stdout; raster exige --out; bytes estáveis
akr render chart.akrc > chart.svg
akr render chart.akrc --format png --scale 2 --out chart.png
akr render chart.akrc --format scene | jq .            # geometria para depuração
akr render chart.akrc --format excalidraw > chart.excalidraw

# fmt — forma canônica, sem ruído de coordenadas no Git
akr fmt chart.akrc --write
akr fmt chart.akrc --check             # exit 1 quando diverge

# diff — mudanças semânticas
akr diff antigo.akrc novo.akrc         # + node dlq "DLQ" · ~ edge api->retry style …

# import — .excalidraw → .akrc
akr import chart.excalidraw > chart.akrc

# schema — statements, chaves, valores fechados, temas e códigos em JSON
akr schema
```

## Para agentes

```bash
akr init                 # akribie.toml + bloco em AGENTS.md + skill em .agents/skills/akr/
akr prime                # gramática e regras em texto (--long: BNF + tabelas)
```

`akribie-mcp` fala JSON-RPC 2.0 por linha em `stdin`/`stdout` e expõe `chart_check`, `chart_render`
e `chart_diff`. `stdout` é dado e `stderr` é log; `akr check --json` e `akr render --json` emitem o
envelope `{"success","command","data"?,"error"?,"warnings"}`. Para lote,
`akr check --batch lote.jsonl` e `akr render --batch lote.jsonl --out-dir saida/` — uma linha JSONL
por item, best-effort.

## O que o formato garante

- **Determinismo.** Mesmo `.akrc` + mesma versão do binário ⇒ mesmos bytes de SVG e de `scene` JSON.
- **Acessibilidade.** `role="img"`, `<title>`, `<desc>` com a topologia em texto, `aria-label` por
  elemento, contraste mínimo 4.5:1 (texto) e 3:1 (contorno) verificado por teste.
- **Diff semântico.** `akr diff OLD NEW` reporta `+ node dlq "DLQ"`, `- node legacy`,
  `~ edge api->retry style: solid -> dashed` — não coordenadas.
- **Um formato por vez.** SVG, PNG, JPG, AVIF, `scene` JSON e `.excalidraw`, todos derivados da mesma
  fonte.

## Documentação

| Tema | Referência |
|---|---|
| Panorama, pipeline e escopo | [`docs/plan/00-panorama.md`](docs/plan/00-panorama.md) |
| Guias de uso (um por verbo) | [`docs/usage/00-quickstart.md`](docs/usage/00-quickstart.md) |
| Gramática do `.akrc` e diagnóstico | [`docs/specs/GRAMMAR.md`](docs/specs/GRAMMAR.md) |
| Contrato de acessibilidade do SVG | [`docs/specs/A11Y.md`](docs/specs/A11Y.md) |
| Contrato de bytes e determinismo | [`docs/specs/DETERMINISM.md`](docs/specs/DETERMINISM.md) |
| Arquitetura (módulos, portas, fluxo) | [`docs/specs/ARCHITECTURE.md`](docs/specs/ARCHITECTURE.md) |
| Decisões normativas | [`docs/adr/`](docs/adr/) |
| Servidor MCP | [`docs/usage/06-init-prime.md`](docs/usage/06-init-prime.md) |
| Guia para agentes | [`SKILL.md`](./SKILL.md) |
| Índice para LLM | [`llms.txt`](./llms.txt) |
| Histórico de mudanças | [`CHANGELOG.md`](CHANGELOG.md) |
| Contribuição | [`AGENTS.md`](./AGENTS.md) |

## Licença

Licenciado sob **MIT OR Apache-2.0**. A fonte Excalifont é OFL-1.1
([`assets/EXCALIFONT-OFL.txt`](assets/EXCALIFONT-OFL.txt)) e a Liberation Sans é OFL-1.1
([`assets/LIBERATION-OFL.txt`](assets/LIBERATION-OFL.txt)).
