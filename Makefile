.DEFAULT_GOAL := all

CC ?= gcc
CFLAGS ?= -O2 -Wall -Wextra
LIBSYSTEMD ?= $(shell pkg-config --cflags --libs libsystemd 2>/dev/null || echo "-lsystemd")
WAYLAND_FLAGS ?= $(shell pkg-config --cflags --libs wayland-client 2>/dev/null || echo "-lwayland-client")
PIXMAN_FLAGS ?= $(shell pkg-config --cflags --libs pixman-1 2>/dev/null || echo "-lpixman-1")

DAEMONS := target/minbat target/minosd target/minclip

.PHONY: all build clean test audit doctor deploy help $(DAEMONS)

all: build

help:
	@echo "Minimal Desktop Management"
	@echo "  make build   - Compile native control plane (minimalctl) and C daemons"
	@echo "  make test    - Run test suite, shell syntax checks, and daemon dry-runs"
	@echo "  make audit   - Run security, syntax, and permission audits via minimalctl"
	@echo "  make doctor  - Run operational diagnostics suite"
	@echo "  make deploy  - Deploy configuration symlinks via deploy.sh"
	@echo "  make clean   - Clean build targets"

target/minbat: src/minbat/main.c
	@mkdir -p target
	$(CC) $(CFLAGS) $< $(LIBSYSTEMD) -o $@

target/minosd: src/minosd/main.c src/minosd/wlr-layer-shell-unstable-v1-protocol.c src/minosd/xdg-shell-protocol.c
	@mkdir -p target
	$(CC) $(CFLAGS) -Isrc/minosd $< src/minosd/wlr-layer-shell-unstable-v1-protocol.c src/minosd/xdg-shell-protocol.c $(WAYLAND_FLAGS) $(PIXMAN_FLAGS) -lm -o $@

target/minclip: src/minclip/main.c
	@mkdir -p target
	$(CC) $(CFLAGS) $< $(WAYLAND_FLAGS) -o $@

build: $(DAEMONS)
	@cargo build --release

test: $(DAEMONS)
	@cargo test
	@bash -n deploy.sh install.sh tty-init.sh scripts/*.sh labwc/scripts/*.sh 2>/dev/null || true
	./target/minbat --dry-run
	./target/minosd --dry-run
	./target/minclip --dry-run

audit:
	@cargo run --quiet -- audit

doctor:
	@cargo run --quiet -- doctor

deploy:
	@./deploy.sh

clean:
	@cargo clean
	@rm -f $(DAEMONS)
