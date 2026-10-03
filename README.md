# Minimal Dotfiles & Terminal-Native Neovim IDE

Minimal is an Arch Linux dotfiles repository for a Hyprland-based desktop, featuring a native C/Rust control plane and a tightly integrated **Terminal-Native Neovim IDE**.

## Terminal-Native Neovim IDE

The **Terminal-Native Neovim IDE** provides the modern, agentic development workflow of VS Code and Google Antigravity while retaining **Neovim** as the high-speed, keyboard-driven text editor.

### Architecture

Unlike plugins that attempt to run an AI agent inside Neovim or bind to specific vendor APIs, this system adheres to a **strict decoupling principle**:
- **External CLI Agents** (Codex, Claude Code, Gemini CLI, Aider, custom scripts) run as independent processes in standard PTY terminals.
- **The Workspace Filesystem & Git Repository** serve as the common ground of truth.
- **The Rust IDE Core Sidecar (`minimal-ide`)** runs asynchronously alongside Neovim, observing filesystem changes, maintaining snapshot baselines, calculating structured diffs, and attributing modifications between user and agent.
- **The Neovim Plugin (`ide.nvim`)** acts as the presentation layer, rendering animated diff transitions, floating change reviews, and atomic accept/reject operations directly within the editor.

### Installation

The system is deployed via the standard Minimal installer:

```bash
# Redeploy symlinks and install IDE binaries
./deploy.sh
```

This installs `minimal-ide` to `~/.local/bin/minimal-ide` and injects the Neovim plugin into `~/.config/nvim`.

### Configuration

Configuration is managed via TOML. Example configuration elements include:

```toml
[diff]
animation = true
animation_duration_ms = 180
```

### Supported Agents

Any CLI agent that modifies the filesystem will work out of the box. Pre-configured presets include:
- `codex`
- `claude`
- `gemini`
- `aider`

### Development

The IDE is built in Rust (the sidecar) and Lua (the Neovim plugin). See the `docs/` directory for detailed architecture, protocol, and configuration specifications.

### Limitations & Roadmap
- Currently supports atomic file modifications and diff visualization.
- Future roadmap includes parallel sessions, advanced conflict handling, and persistent workspace state.
- See `docs/roadmap.md` for details.

---

## ⚡ Native C System Daemons (`src/`)

Minimal replaces high-overhead shell daemon loops and external CLI wrappers with hyper-optimized, standalone native C daemons built with strict zero-subshell and zero-fork constraints:

### 1. Zero-Wakeup Battery Daemon (`src/minbat/`)
- Listens on `PF_NETLINK` for true zero-CPU idle operation.

### 2. Wayland Layer-Shell OSD Overlay (`src/minosd/`)
- Renders progress bars on a Wayland layer surface via `pixman` with 12px rounding.

### 3. In-Memory Wayland Clipboard Ring (`src/minclip/`)
- Implements Wayland `wl_data_device_manager` directly with zero disk I/O.

## 📁 Repository Structure
- `docs/` - IDE architecture and design documentation
- `src/ide/` - Rust source for `minimal-ide` sidecar
- `nvim/lua/ide/` - Neovim Lua plugin
- `hypr/` - Hyprland window manager configurations
