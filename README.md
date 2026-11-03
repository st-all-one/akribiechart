# akribie

Diagramas determinísticos e acessíveis a partir de texto, para LLMs e agentes.

Você escreve topologia e rótulos; `akr` calcula geometria, cor, texto alternativo e bytes. Sem
servidor, sem banco, sem rede: um binário Rust.

```akrc
chart v1 "Política de retry" direction=lr layout=flow lang=pt-BR

node api "API Gateway" fill=blue
node retry "Retry ×3" fill=amber shape=cloud
edge api -> retry "5xx"
edge retry -> api "backoff" curve=arc style=dashed
```

```sh
akr check chart.akrc            # valida; exit 8 se houver erro
akr render chart.akrc > a.svg   # SVG acessível (role/title/desc), bytes estáveis
akr render chart.akrc --format png --out a.png --scale 2
akr fmt chart.akrc --write      # forma canônica (diff semântico no Git)
```

## Para agentes

```sh
akr init                 # akribie.toml + bloco em AGENTS.md + skill em .agents/skills/akr/
akr prompt               # gramática e regras em texto (--long: BNF + tabelas)
akr render chart.akrc --format scene | jq .   # geometria para depuração
```

`akribie-mcp` fala JSON-RPC por stdio e expõe `chart_check`, `chart_render` e `chart_diff`. Para
lote, `akr check --batch lote.jsonl` e `akr render --batch lote.jsonl --out-dir saida/` (uma linha
JSONL por item, best-effort).

## Instalação

```sh
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/st-all-one/akribie/main/install.sh | bash
```

O instalador baixa o pacote do GitHub Release do alvo detectado, confere o SHA-256 e instala `akr`
e `akribie-mcp` em `~/.local/bin`. Variáveis: `VERSION=v0.1.0`, `INSTALL_DIR=...`,
`AKR_TARGET=...`, `AKR_NO_PATH=1`. `--from-source` compila do repositório e `--uninstall` move os
binários para o lixo recuperável.

Do source:

```sh
make release            # target/release/{akr,akribie-mcp}
make install            # ~/.local/bin/{akr,akribie-mcp}  (PREFIX/BINDIR mudam o destino)
```

O Release publica seis alvos — `x86_64`/`aarch64` em Linux musl (estático), macOS e Windows — com
`akribie-<versão>-<target>.{tar.gz,zip}` e `sha256sums.txt`.

`assets/Excalifont-Regular.ttf` é commitado; para regenerá-lo, `make font` (usa `curl` uma vez e
verifica o SHA-256 de `assets/FONT.lock`).

## O que o formato garante

- **Determinismo.** Mesmo `.akrc` + mesma versão do binário ⇒ mesmos bytes de SVG e de `scene` JSON.
- **Acessibilidade.** `role="img"`, `<title>`, `<desc>` com a topologia em texto, `aria-label` por
  elemento, contraste mínimo 4.5:1 (texto) e 3:1 (contorno) verificado por teste.
- **Diff semântico.** `akr diff OLD NEW` reporta `+ node dlq "DLQ"`, `- node legacy`,
  `~ edge api->retry style: solid -> dashed` — não coordenadas.
- **Um formato por vez.** SVG, PNG, JPG, AVIF, `scene` JSON, `.excalidraw`, todos derivados da mesma
  fonte.

## Documentação

| documento | conteúdo |
|---|---|
| `docs/plan/00-panorama.md` | visão geral, pipeline e escopo |
| `docs/adr/` | cinco decisões normativas |
| `docs/specs/GRAMMAR.md` | gramática do `.akrc` e diagnóstico |
| `docs/specs/A11Y.md` | contrato de acessibilidade do SVG |
| `docs/specs/DETERMINISM.md` | o que é contrato de bytes |
| `docs/usage/` | um guia por verbo (00-quickstart … 07-schema) |
| `docs/specs/ARCHITECTURE.md` | módulos, portas e fluxo |
| `llms.txt` / `SKILL.md` | fatos e guia acionável para agentes |
| `AGENTS.md` | regras de contribuição |
| `SKILL.md` | guia acionável para agentes |

## Licença

MIT OR Apache-2.0. A fonte Excalifont é OFL-1.1 (`assets/EXCALIFONT-OFL.txt`).
