# akribie — alvos de qualidade e build (ADR-0001 §4, ADR-0006)

CARGO ?= cargo
PREFIX ?= $(HOME)/.local
BINDIR ?= $(PREFIX)/bin

.PHONY: check fmt clippy test file-length tree xtask build release install ci deny typos clean font

## Portão único: formatação, lints, testes, limite de 300 linhas por arquivo e árvore de deps.
check: fmt clippy test file-length tree

## Verifica formatação sem alterar (workspace e xtask, que tem lock próprio).
fmt:
	$(CARGO) fmt --all -- --check
	$(CARGO) fmt --manifest-path xtask/Cargo.toml -- --check

## Lints de todos os alvos, warnings como erro.
clippy:
	$(CARGO) clippy --workspace --all-targets -- -D warnings

## Testes do workspace (inclui doc-tests).
test:
	$(CARGO) test --workspace

## Nenhum arquivo de produção passa de 300 linhas.
file-length:
	./scripts/check_file_length.sh

## Nenhum runtime assíncrono na árvore; nenhuma thread na árvore do domínio (ADR-0001 §1).
## `rayon` aparece só via `ravif`→`rav1e`, o encoder AVIF da feature `raster`; o domínio compilado
## sem `raster` não o puxa.
tree:
	@! $(CARGO) tree --workspace -e normal --prefix none | grep -Eq '^(tokio|async-std) ' \
		|| { echo "ERRO: runtime assíncrono na árvore de dependências" >&2; exit 1; }
	@! $(CARGO) tree -p akribie-core --no-default-features -e normal --prefix none \
		| grep -Eq '^(tokio|async-std|rayon) ' \
		|| { echo "ERRO: thread ou runtime assíncrono no domínio" >&2; exit 1; }

## Build de debug do workspace.
build:
	$(CARGO) build --workspace

## Build de release dos dois binários (`akr` e `akribie-mcp`).
release:
	$(CARGO) build --release -p akribie-cli -p akribie-mcp

## Instala `akr` e `akribie-mcp` em $(BINDIR) (default: ~/.local/bin).
install: release
	install -Dm755 target/release/akr "$(DESTDIR)$(BINDIR)/akr"
	install -Dm755 target/release/akribie-mcp "$(DESTDIR)$(BINDIR)/akribie-mcp"

## Lints do `xtask` (ferramenta de build com lock próprio, fora do workspace). Fica fora de
## `check` de propósito: resolver as dependências do `xtask` exige rede, e o portão local precisa
## rodar offline. Roda no `ci`, onde o toolchain tem rede.
xtask:
	$(CARGO) clippy --manifest-path xtask/Cargo.toml --all-targets -- -D warnings

## Portão completo de CI (deny/typos precisam estar instalados).
ci: check xtask deny typos

deny:
	cargo deny check

typos:
	typos

## Regenera `assets/Excalifont-Regular.ttf` a partir do woff2 pinado.
font:
	$(CARGO) run -p xtask -- font build

clean:
	$(CARGO) clean
