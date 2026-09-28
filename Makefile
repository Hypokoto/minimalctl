.DEFAULT_GOAL := help

.PHONY: all build clean test audit doctor deploy help

all: build

help:
	@echo "Minimal Desktop Management"
	@echo "  make build   - Compile native control plane (minimalctl) and C daemons (minbat)"
	@echo "  make test    - Run test suite and shell syntax checks"
	@echo "  make audit   - Run security, syntax, and permission audits via minimalctl"
	@echo "  make doctor  - Run operational diagnostics suite"
	@echo "  make deploy  - Deploy configuration symlinks via deploy.sh"
	@echo "  make clean   - Clean build targets"

build:
	@cargo build --release
	@if [ -d tools/minbat ]; then $(MAKE) -C tools/minbat; fi

test:
	@cargo test
	@bash -n deploy.sh install.sh tty-init.sh scripts/*.sh labwc/scripts/*.sh

audit:
	@cargo run --quiet -- audit

doctor:
	@cargo run --quiet -- doctor

deploy:
	@./deploy.sh

clean:
	@cargo clean
	@if [ -d tools/minbat ]; then $(MAKE) -C tools/minbat clean; fi
