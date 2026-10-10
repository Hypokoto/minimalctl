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

if command -v nasm >/dev/null 2>&1; then
	echo "Building pure Assembly minbat..."
	nasm -f elf64 "$DOTFILES_DIR/src/minbat/minbat.asm" -o "$DOTFILES_DIR/src/minbat/minbat.o"
	ld "$DOTFILES_DIR/src/minbat/minbat.o" -o "$DOTFILES_DIR/src/minbat/minbat"
	install -m 755 "$DOTFILES_DIR/src/minbat/minbat" "$HOME/.local/bin/minbat"
	echo "Installed minbat to ~/.local/bin/minbat"
else
	echo "ERROR: nasm is required to build minbat. Please install nasm."
	exit 1
fi
