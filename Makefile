# akribie — alvos de qualidade e build (ADR-0001 §4)

CARGO ?= cargo
PREFIX ?= $(HOME)/.local
BINDIR ?= $(PREFIX)/bin

.PHONY: check fmt clippy test file-length build release install ci deny typos clean

## Portão único: formatação, lints, testes e limite de 300 linhas por arquivo.
check: fmt clippy test file-length

## Verifica formatação sem alterar.
fmt:
	$(CARGO) fmt --all -- --check

## Lints de todos os alvos, warnings como erro.
clippy:
	$(CARGO) clippy --workspace --all-targets -- -D warnings

## Testes do workspace (inclui doc-tests).
test:
	$(CARGO) test --workspace

## Nenhum arquivo de produção passa de 300 linhas.
file-length:
	./scripts/check_file_length.sh

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

## Portão completo de CI (deny/typos precisam estar instalados).
ci: check deny typos

deny:
	cargo deny check

typos:
	typos

## Regenera `assets/Excalifont-Regular.ttf` a partir do woff2 pinado.
font:
	$(CARGO) run -p xtask -- font build

clean:
	$(CARGO) clean
