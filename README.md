# Minimal Dotfiles

Modular, high-performance Linux desktop environment architecture built for Hyprland, Neovim (NvChad), Quickshell, and an AI-native Zsh terminal workspace. Features a refined, ultra-fast Quickshell presentation layer, native Rust theme control plane (`minimalctl`), Hyprtasking workspace overview, and hy3 tabbed window grouping.

---

## 📚 Documentation & User Guides

- 📖 **[User Guide](docs/user-guide.md)** — Keybindings, Workspaces, Launchers, Window Rescue, Game Mode, Night Light, Screen OCR, Theme Switching.
- 🏗️ **[System Architecture](docs/architecture.md)** — Rust `minimalctl` control plane, atomic writes, zero-drift FNV-1a content hash engine.
- ⌨️ **[Keybindings Reference](docs/keybindings.md)** — Complete Vim spatial navigation & shortcut cheat sheet.
- 🩺 **[Troubleshooting Guide](docs/troubleshooting.md)** — Operational diagnostics (`minimalctl doctor`), verification, and emergency window recovery.

---

## 🚀 Quick Start & Deployment

```bash
git clone https://github.com/Hypokoto/minimal.git ~/minimal
cd ~/minimal
./install.sh
```

- **`install.sh`**: Multi-distro package installer with distro detection (`pacman`/`apt`/`dnf`/`brew`). On Arch Linux (primary target), installs system binaries, fonts, services, and toolchains via `pacman` + `yay` (AUR). On other distros, skips system package installation but still deploys dotfiles via `deploy.sh`.
- **`deploy.sh`**: Idempotent deployment pipeline for user dotfiles. Performs binary presence verification, palette target compilation (`minimalctl theme apply obsidian`), atomic symlinking, NvChad custom overlaying, and font cache refreshes (`fc-cache -fv`). Logged to `/tmp/minimal-deploy.log`.

---

## 🎨 Single-Source-of-Truth Theme Engine (`minimalctl`)

`themes/*.toml` (e.g. `themes/obsidian.toml`) and the `minimalctl` native Rust control plane serve as the **single source of truth** for all desktop color palettes and theme targets.

```bash
# Build & apply theme transaction across all running components live:
minimalctl theme apply [theme]

# Verify theme definition and check for target drift:
minimalctl theme verify

# Run operational diagnostics suite:
minimalctl doctor
```

`minimalctl theme apply` compiles 1 canonical source (`themes/*.toml`) into 7 active runtime targets live without logging out or killing session states:
- `hypr/colors.conf` — Hyprland `$variable` definitions (reloaded live via `hyprctl`)
- `kitty/kitty.conf` — Terminal palette & chrome (reloaded live via `SIGUSR1`)
- `btop/btop.theme` — btop TUI color table
- `starship/starship.toml` — Starship shell prompt (reloaded live on next prompt)
- `tmux/tmux.conf` & `~/.tmux.conf` — Tmux status bar & pane borders (reloaded live via `tmux source-file` + `tmux refresh-client`)
- `nvim/lua/themes/minimal.lua` — Neovim NvChad Base46 theme
- `~/.config/quickshell/theme.json` — Quickshell presentation layer (reloaded live via `quickshell ipc call minimal-shell reloadTheme`)

---

## 🐚 Quickshell Presentation Layer (`quickshell/`)

Minimal replaces legacy bar and menu daemons with a unified, lightweight Quickshell layer (`shell.qml`):
- **Top Bar (`Bar.qml`)**: Workspaces, window title, sys-tray, network, audio, volume, battery, and clock.
- **App Launcher (`LauncherWindow.qml`)**: 5-column grid app launcher with freedesktop icon evaluation, fuzzy search, and right-click context menu (Open / Pin / Unpin).
- **Clipboard History Manager (`ClipboardWindow.qml`)**: Quickshell clipboard overlay backed by `cliphist` + `wl-clipboard` featuring fuzzy search, text & image previews, item deletion, and clear history (`SUPER + X`).
- **Control Center (`ControlCenterWindow.qml`)**: Quick toggles for Wi-Fi, Bluetooth, Audio, Brightness, Game Mode, Night Light, and System Usage graphs (`SUPER + N`).
- **Workspace Overview (`Hyprtasking`)**: 3x3 interactive workspace grid overview (`SUPER + T` / `SUPER + ~`).
- **Session Menu (`SidebarWindow.qml`)**: Power menu overlay for shutdown, reboot, suspend, lock, and logout (`SUPER + Escape`).

---

## 🛠️ Zsh Subsystem (`zsh/`)

The Zsh shell environment is configured as a guarded, high-performance workspace:

### 1. Defensive Alias Architecture (`zsh/aliases.zsh`)
Every binary alias is wrapped in a `command -v <tool> >/dev/null 2>&1` check to guarantee that the shell never breaks if a package is uninstalled:
- **Listing**: `ls`, `ll`, `la`, `l`, `lt`, `llt` → `eza` (with git & icon support)
- **File Viewing**: `cat`, `batp` → `bat` (syntax highlighting pager)
- **Disk Usage**: `df` → `duf`, `du` → `dust`
- **System Monitoring**: `top` → `btop`
- **Search**: `grep` → `ripgrep` (`rg`), `find` → `fd`
- **Safe Deletion**: `rm`, `tp`, `tl`, `tr` → `trash-cli` / `trash-put` (with `rmf` for raw `/bin/rm -iv`)
- **Navigation & Reloader**: `..`, `...`, `....`, `-`, `reload`, `ezsh`, `ealias`

### 2. Rust Security & Networking Layer (`zsh/sec.zsh`)
Dedicated security and networking aliases:
- **Recon & Fuzzing**: `rustscan` (fast Nmap hand-off) and `feroxbuster` (web content discovery).
- **Monitoring & Discovery**: `sniffnet` (TUI traffic monitor), `bandwhich`, `trippy` (traceroute), and `netscanner` (ARP LAN discovery).
- **Analysis**: `hexyl` (hex viewer) and `cargo-audit` (RustSec CVE scanner).

---

## 📁 Repository Structure

```
minimal/
├── install.sh                     # Arch system setup script (pacman + yay + deploy)
├── deploy.sh                      # Idempotent symlinker & NvChad overlay pipeline
├── LICENSE                        # GNU General Public License v3.0
├── README.md                      # Repository documentation
├── Cargo.toml / src/              # minimalctl native Rust control plane
├── btop/
│   └── btop.theme                 # Compiled from themes/*.toml
├── fastfetch/
│   └── config.jsonc               # Fastfetch system info layout
├── git/
│   └── config                     # Git & delta syntax pager configuration
├── hypr/
│   ├── hyprland.lua               # Core Hyprland compositor entry point (Lua)
│   ├── keybinds.lua               # Global keybindings & media pipeline (Lua)
│   ├── monitors.lua               # Display rules & monitor topology (Lua)
│   └── wallpaper/                 # Wallpaper picker & daemon scripts
├── kitty/
│   └── kitty.conf                 # Terminal config & compiled colors
├── nvim/
│   ├── ftplugin/java.lua          # Prioritized JDK 21 OpenJDK discovery logic
│   ├── lua/chadrc.lua             # NvChad entry point
│   └── lua/themes/minimal.lua    # Base46 theme compiled from themes/*.toml
├── quickshell/
│   ├── shell.qml                  # Main IPC entry point & service initialization
│   ├── modules/
│   │   ├── bar/Bar.qml            # Top bar component
│   │   ├── launcher/              # Grid launcher component
│   │   ├── clipboard/             # Clipboard history manager component
│   │   ├── controlcenter/        # Quick settings control center
│   │   └── osd/                   # OSD volume and brightness overlays
│   └── services/                  # Quickshell singletons (CliphistService, Pywal, etc.)
├── starship/
│   └── starship.toml              # Starship prompt configuration
├── themes/
│   ├── obsidian.toml              # Default dark cyan/void theme source
│   ├── gruvbox.toml
│   ├── synthwave.toml
│   ├── catpuccin.toml
│   └── ayu-dark.toml
├── tmux/
│   └── tmux.conf                  # Tmux terminal multiplexer configuration
├── yazi/
│   └── yazi.toml                  # Yazi file manager configuration
└── zsh/
    ├── .zshrc                     # Main Zsh configuration
    └── aliases.zsh                # Defensive guarded CLI aliases module
```

---

## ⌨️ Keybindings Quick Reference

| Key Combo | Action | Component |
|---|---|---|
| `SUPER + Return` | Launch Kitty terminal | Terminal |
| `SUPER + Space` | Toggle Quickshell App Grid Launcher | Launcher |
| `SUPER + X` | Toggle Quickshell Clipboard Manager | Clipboard |
| `SUPER + N` | Toggle Quickshell Control Center | Quick Settings |
| `SUPER + T` / `SUPER + ~` | Toggle Hyprtasking 3x3 workspace grid overview | Workspaces |
| `SUPER + G` | Toggle Hyprland window tabbed grouping (`hy3`) | Windows |
| `SUPER + Q` | Close active window | Window Control |
| `SUPER + F` | Toggle fullscreen window mode | Window Control |
| `SUPER + V` | Toggle floating window mode | Window Control |
| `SUPER + B` | Toggle Quickshell Top Bar | Bar |
| `SUPER + Escape` | Toggle Quickshell Session / Power menu | Session |
| `SUPER + ALT + Escape` | Lock screen (`hyprlock`) | Security |
| `SUPER + SHIFT + G` | Toggle Game Mode (disable blur/animations) | Performance |
| `SUPER + SHIFT + X` | Screen OCR text extractor to clipboard | Utilities |
