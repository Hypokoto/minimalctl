#!/usr/bin/env bash
set -euo pipefail

DOTFILES_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mkdir -p "$HOME/.local/bin"

if command -v cargo >/dev/null 2>&1; then
	echo "Building minimalctl..."
	(cd "$DOTFILES_DIR" && cargo build --release)
	for daemon in minimalctl minosd min-auth; do
		if [[ -f "$DOTFILES_DIR/target/release/$daemon" ]]; then
			install -m 755 "$DOTFILES_DIR/target/release/$daemon" "$HOME/.local/bin/$daemon"
			echo "Installed $daemon to ~/.local/bin/$daemon"
		fi
	done
fi
