#!/usr/bin/env bash
set -euo pipefail

DOTFILES_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mkdir -p "$HOME/.local/bin"

if command -v cargo >/dev/null 2>&1; then
	echo "Building minimalctl..."
	(cd "$DOTFILES_DIR" && cargo build --release)
	if [[ -f "$DOTFILES_DIR/target/release/minimalctl" ]]; then
		install -m 755 "$DOTFILES_DIR/target/release/minimalctl" "$HOME/.local/bin/minimalctl"
		echo "Installed minimalctl to ~/.local/bin/minimalctl"
	fi
fi

if make -C "$DOTFILES_DIR" "target/mincore" >/dev/null 2>&1; then
	install -m 755 "$DOTFILES_DIR/target/mincore" "$HOME/.local/bin/mincore"
	echo "Installed unified multi-call daemon to ~/.local/bin/mincore"
	for daemon in minbat minosd minclip; do
		ln -sfn "$HOME/.local/bin/mincore" "$HOME/.local/bin/$daemon"
		echo "Symlinked $daemon -> mincore"
	done
fi
