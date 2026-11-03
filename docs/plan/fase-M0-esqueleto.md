# Fase M0 — esqueleto, governança e fonte

**Status:** concluída · **Depende de:** — · **Habilita:** M1

## Entrega

1. Workspace Rust com `akribie-core` (lib pura), `akribie-cli` (bin `akr`), `akribie-mcp` (bin) e
   `xtask` (utilitário de build). `default-members = ["crates/akribie-cli"]`.
2. `Cargo.toml` com edition 2024, `resolver = "3"`, `overflow-checks = true` em `dev` e `release`,
   `[workspace.lints]` e dependências com versões fixadas (ADR-0001 §5.1).
3. `rust-toolchain.toml` (1.99), `rustfmt.toml`, `clippy.toml`, `deny.toml`, `typos.toml`,
   `.gitignore`.
4. ADRs 0001–0005, `docs/review-protocol.md`, `docs/plan/00-panorama.md`, `01-fases.md`,
   `02-matriz-aceitacao.md`.
5. `xtask font build` + `assets/Excalifont-Regular.ttf` commitado, `assets/FONT.lock`,
   `assets/EXCALIFONT-OFL.txt`.
6. `scripts/check_file_length.sh`, `Makefile`, `.github/workflows/ci.yml`.
7. `README.md`, `AGENTS.md`, `llms.txt`, `SKILL.md`, `CHANGELOG.md`.

## Decisões de execução

- **`xtask` é membro do workspace** (não `exclude`). O critério de aceite desta fase é
  `cargo run -p xtask -- font build`, que não funciona para um pacote excluído do workspace. A
  intenção do `exclude` — não construir `xtask` por padrão — é preservada por `default-members`, que
  contém só a CLI. `xtask` não declara `[lints] workspace = true`: ele é ferramenta de build, não
  domínio.
- **SHA-256 implementado em `xtask`** (FIPS 180-4, ~60 linhas) em vez de acrescentar `sha2` à tabela
  de dependências: a tabela do ADR-0001 §5.1 é fechada e o cálculo é usado uma vez por build de fonte.
- **Pin transitivo de `bytes = "=1.9.0"` em `xtask`.** `woff2 0.3.0` resolve
  `SafeBuf::try_get_u8` de forma ambígua porque `bytes >= 1.10` acrescentou `Buf::try_get_u8`; o
  crate não compila em nenhuma resolução mais nova. O pin é do resolver, não uso direto, e está
  comentado no manifesto.
- **Download por `curl`.** `xtask` não tem cliente HTTP e não deve ter: a fonte é baixada uma vez e o
  `.ttf` resultante é commitado. `curl` é pré-requisito de build documentado no `README`.
- **Fonte**: `https://excalidraw.nyc3.cdn.digitaloceanspaces.com/fonts/Excalifont-Regular.woff2`,
  SHA-256 `ee41ec4c06bfa0728665499de6f4b4019e7953119ab20b5aeb5917f1609c3b2a`, OFL-1.1.

## Verificação

```sh
make check                                     # fmt + clippy -D warnings + test + file-length
cargo run -p xtask -- font build               # idempotente: "inalterado" na segunda execução
ls assets/Excalifont-Regular.ttf               # existe, 196432 bytes
grep -c '^[0-9]\+\.' docs/adr/ADR-0001-*.md    # cinco decisões numeradas
```

Resultados observados nesta fase:

| checagem | resultado |
|---|---|
| `make check` | verde |
| `xtask font build` (2ª execução) | `assets/Excalifont-Regular.ttf inalterado (196432 bytes)` |
| SHA-256 do woff2 | `ee41ec4c…3b2a`, igual a `assets/FONT.lock` |
| `assets/FONT.lock` | `<hex>  Excalifont-Regular.woff2` |
