#!/usr/bin/env bash
set -euo pipefail

if command -v rustup >/dev/null 2>&1 && ! rustup show >/dev/null 2>&1; then
	rustup default stable
fi

bash "$(dirname "${BASH_SOURCE[0]}")/deploy.sh"
