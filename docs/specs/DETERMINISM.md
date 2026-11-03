# DETERMINISM — o que é contrato de bytes

ADR-0001 §2: mesma entrada + mesma versão do binário ⇒ **mesmos bytes**. Determinismo aqui é
propriedade verificada por teste, não intenção.

## Contratos

| artefato | contrato |
|---|---|
| SVG (`--format svg`) | **sim** — byte a byte |
| `scene` JSON | **sim** — byte a byte |
| `.excalidraw` | **sim** — byte a byte (chaves de ordem fixas) |
| PNG / JPG / AVIF | determinístico **na versão** (`--scale` fixo, uma thread de codificação); não é contrato entre versões |
| `akr prompt` | **sim**, por versão (golden) |
| forma canônica (`fmt`) | **sim** — idempotente |

## Como

- **Sem relógio.** Nada lê `SystemTime`/`Instant`; `.excalidraw` grava `updated`, `created` e
  `versionNonce` como `0`.
- **Sem RNG do sistema.** O traço à mão usa RNG próprio semeado por FNV-1a 32 bits do **id** do
  elemento: o mesmo id desenha o mesmo traço em qualquer máquina e a qualquer momento.
- **Sem ordem de iteração implícita.** Nada itera `HashMap`; ids preservam a ordem de declaração
  (`IndexMap`) e as classes saem sempre em `chart`, `group`, `node`, `edge`, `note`.
- **Números com formatação própria.** `num::format` fixa até duas casas, sem zeros à direita e sem
  `-0`. `f32::to_string` é proibido na saída: muda entre versões da std.
- **Fonte embutida.** As duas faces (Excalifont em `font=hand`, Liberation Sans em `font=sans`) vão
  dentro do binário e medem layout, pintura e raster; nada depende de fontes do sistema nem de cache
  de fontes. `sketch=off` é geometria exata, sem RNG: o mesmo arquivo gera os mesmos bytes.
- **Uma thread na rasterização.** O encoder é chamado com `num_threads(1)` para que a ordem dos
  blocos não dependa do escalonador.
- **Sem ambiente no domínio.** O núcleo não lê variável de ambiente; a CLI usa a porta `Env`
  apenas para configurar caminhos.
- **Sem relógio no lote.** Itens são processados na ordem do arquivo; `--out-dir` não afeta o
  conteúdo dos arquivos.

## Testes

- `akribie-core/src/emit/svg/tests.rs::svg_is_byte_identical_for_the_same_scene`
- `akribie-cli/tests/render.rs::two_renders_are_byte_identical`
- goldens de `scene`, `fmt` e `prompt` em `crates/akribie-cli/tests/golden/`
- `akribie-core/src/emit/excalidraw/tests.rs::export_is_deterministic`

## O que muda entre versões

- Geometria de layout e escolha de rota podem melhorar; o SVG muda **com a versão**, nunca com o
  ambiente.
- Codecs de raster e compressão podem mudar bytes de PNG/JPG/AVIF entre versões; por isso o raster
  não é contrato de bytes.
- A forma canônica e a gramática (`v1`) são estáveis: mudança exige nova versão de `chart` (`E009`).
