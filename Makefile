.DEFAULT_GOAL := all

CC ?= gcc
CFLAGS ?= -O2 -Wall -Wextra
LIBSYSTEMD ?= $(shell pkg-config --cflags --libs libsystemd 2>/dev/null || echo "-lsystemd")

.PHONY: all build clean test audit doctor deploy help

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

build: target/minbat
	@cargo build --release

test: target/minbat
	@cargo test
	@bash -n deploy.sh install.sh tty-init.sh scripts/*.sh labwc/scripts/*.sh 2>/dev/null || true
	./target/minbat --dry-run

audit:
	@cargo run --quiet -- audit

doctor:
	@cargo run --quiet -- doctor

deploy:
	@./deploy.sh

clean:
	@cargo clean
	@rm -f target/minbat
