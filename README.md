# Minimal Labwc Dotfiles

Minimal is an Arch Linux dotfiles repository for a **Labwc-based Wayland desktop**, engineered for extreme speed (native C/Rust control plane) and zero configuration drift.

## Setup & Installation

This repository is designed to be deployed cleanly onto an Arch Linux system.

### 1. First-time system setup (Packages + Deploy):
```bash
# This installs all necessary Arch packages via pacman and yay,
# builds the native C/Rust daemons, and deploys the symlinks.
./install.sh
```

### 2. Redeploying configs (After editing):
```bash
# This safely re-links all dotfiles into ~/.config/ without reinstalling packages.
# It is completely idempotent and safe to run multiple times.
./deploy.sh
```

*(Note: Both scripts log output to `~/.local/state/minimal-deploy.log` and `~/.local/state/minimal-install.log`)*

## Architecture

See [docs/CODEMAPS/INDEX.md](docs/CODEMAPS/INDEX.md) for detailed architecture maps.

### Core Components
- **Labwc**: The core Wayland compositor (replacing Hyprland/Sway).
- **Foot**: The native, lightweight Wayland terminal emulator.
- **Native C Daemons (`src/`)**:
  - `minbat` (Battery ACPI monitor)
  - `minclip` (Wayland clipboard ring)
  - `minosd` (Wayland layer-shell OSD overlay)
- **Rust Control Plane (`minimalctl`)**: Orchestration CLI for diagnostics, security audits, and themes.

## Features
- **Zero-Wakeup Daemons**: Uses `PF_NETLINK` and direct `wayland-client` protocols instead of shell loops.
- **Strict Theming**: A single source of truth for colors via `themes/*.toml`.
- **Command Firewall**: Real-time auditing of commands to prevent malicious actions (`zsh/sec.zsh`).
- **AI Sandbox**: A strict Bubblewrap (`bwrap`) container for AI CLIs to prevent unwanted system modifications (`scripts/ai`).

## Documentation

- [User Guide & Shortcuts](docs/user-guide.md)
- [Architecture Codemaps](docs/CODEMAPS/INDEX.md)

## Contributing

Make sure to run `./deploy.sh` after editing any configuration files to apply them to your system.
