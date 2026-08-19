# Minimal Dotfiles

Modular, high-performance Linux desktop environment architecture built for Hyprland, Neovim (NvChad), and an AI-native Zsh terminal workspace. Cyan/void palette contract defined in `palette.md`. Multi-distro install support.

---

## 🚀 Quick Start & Deployment

```bash
git clone https://github.com/Hypokoto/minimal.git ~/minimal
cd ~/minimal
./install.sh
```

- **`install.sh`**: Multi-distro package installer with distro detection (`pacman`/`apt`/`dnf`/`brew`). On Arch Linux (primary target), installs system binaries, fonts, services, and toolchains via `pacman` + `yay` (AUR). On other distros, skips system package installation but still deploys dotfiles via `deploy.sh`.
- **`deploy.sh`**: Atomic symlinking pipeline for user dotfiles. Performs binary presence verification, palette target compilation, atomic `.bak.$(date +%s)` rotations, parent directory creation precedence (preventing nested target directory bugs), NvChad custom overlaying, and font cache refreshes (`fc-cache -fv`).

---

## 🎨 Color System Automation (`palette.md`)

`palette.md` serves as the **single source of truth** for all visual color tokens:

```
Background: #0A0C12 | Surface: #11141D | Overlay: #1C2230 | Text: #F2F6FF
Muted:      #8D95B3 | Primary: #00D9FF | Secondary: #5B8CFF | Highlight: #A05CFF
Success:    #4DFF91 | Warning: #FFCC66 | Danger:    #FF5470 | Info:      #61E6FF
```

### Palette Generator (`hypr/colors.lua`)
`hypr/colors.lua` is a dual-compatible generator (executes identically under standard Lua 5.4 `lua` and LuaJIT 5.1 `luajit`). It parses `palette.md` and deterministically updates:
- **`hypr/colors.conf`**: Hyprland variable definitions (`$primary = rgba(00D9FFFF)`)
- **`rofi/theme.rasi`**: Centralized Rofi CSS tokens (`* { primary: #00D9FF; ... }`)
- **`btop/btop.theme`**: TTY resource monitor color mapping table
- **`kitty/kitty.conf`**: Terminal palette, borders, and tab bar colors
- **`mako/config`**: Notification frame, progress bar, and urgency colors

---

## 🛠️ Shell Subsystem (`zsh/`)

The Zsh shell environment is configured as a guarded, high-performance workspace:

### 1. Defensive Alias Architecture (`zsh/aliases.zsh`)
Every binary alias is wrapped in a `command -v <tool> >/dev/null 2>&1` check to guarantee that the shell never breaks if a package is uninstalled or missing:
- **Listing**: `ls`, `ll`, `la`, `l`, `lt`, `llt` → `eza` (with git & icon support)
- **File Viewing**: `cat`, `batp` → `bat` (syntax highlighting pager)
- **Disk Usage**: `df` → `duf`, `du` → `dust`
- **System Monitoring**: `top` → `btop`
- **Search**: `grep` → `ripgrep` (`rg`), `find` → `fd`
- **Safe Deletion**: `rm`, `tp`, `tl`, `tr` → `trash-cli` / `trash-put` (with `rmf` for raw `/bin/rm -iv`)
- **Safety Overrides**: `cp -iv`, `mv -iv`, `mkdir -pv`
- **Navigation & Reloader**: `..`, `...`, `....`, `-`, `reload`, `ezsh`, `ealias`

### 2. Agent Safety & Hooks (`zsh/.zshrc`)
- **Bracketed Paste Protection**: Enables `bracketed-paste-magic` so multi-line code snippets pasted by external AI agents (Aider, Claude Code, OpenCode) drop into the prompt buffer as editable text without auto-executing.
- **Engine Integrations**: `atuin` (SQLite-backed history search), `zoxide` (frecent directory jump), `starship` (Minimal prompt engine), `zsh-autosuggestions`, and `zsh-syntax-highlighting`.

---

## 🖥️ Desktop & Rofi Suite

- **Hyprland Engine**: Modern block window rules (`windowrule`), gesture workspace switching, and isolated hardware environment flags (`WLR_DRM_NO_ATOMIC` wrapped in configurable toggles).
- **OSD Latency Optimization**: `osd-volume.sh` and `osd-brightness.sh` execute through `makoctl` using explicit replaceable notification IDs (`-r 91110` / `-r 91111`) and synchronous hint `-h string:x-canonical-private-synchronous:osd` to eliminate stack delays.
- **Rofi Integration Suite**:
  - `rofi/scripts/clipboard.sh`: Interfaced with `cliphist` + `wl-clipboard` (list, decode, copy, `Alt+Delete` deletion).
  - `rofi/scripts/powermenu.sh`: Interfaced with `hyprctl dispatch exit`, `systemctl suspend`, `reboot`, and `poweroff`.
  - `rofi/scripts/wallpaper.sh`: Bridge to `hypr/wallpaper/picker.sh` for `awww` hot-reloading.

---

## 📁 Repository Structure

```
minimal/
├── install.sh                     # Multi-distro package installer (pacman/apt/dnf/brew)
├── deploy.sh                      # Idempotent symlinker & NvChad overlay pipeline
├── palette.md                     # Color contract single source of truth
├── README.md                      # Repository documentation
├── btop/
│   └── btop.theme                 # Compiled from palette.md
├── fastfetch/
│   └── config.jsonc               # Fastfetch system info layout
├── git/
│   └── config                     # Git & delta syntax pager configuration
├── hypr/
│   ├── colors.conf                # Generated Hyprland color variables
│   ├── colors.lua                 # Palette compiler (Lua 5.1 & 5.4 compatible)
│   ├── hyprland.conf              # Core Hyprland compositor entry point
│   ├── hypridle.conf / hyprlock.conf
│   ├── keybinds.conf              # Global keybindings & media pipeline
│   ├── monitors.conf             # Display rules & monitor topology
│   ├── wallpaper/                 # awww wallpaper picker & daemon scripts
│   └── scripts/
│       ├── osd-volume.sh          # Volume OSD widget (-r 91110)
│       └── osd-brightness.sh      # Brightness OSD widget (-r 91111)
├── kitty/
│   └── kitty.conf                 # Terminal config & compiled colors
├── mako/
│   └── config                     # Notification daemon config & compiled colors
├── nvim/
│   ├── ftplugin/java.lua          # Prioritized JDK 21 OpenJDK discovery logic
│   ├── lua/autocmds.lua           # Auto-open snacks.dashboard on buffer close
│   ├── lua/chadrc.lua             # NvChad entry point
│   ├── lua/mappings.lua           # Guarded close_buffer and nvim-tree toggle
│   └── lua/themes/minimal.lua    # Base46 theme matching palette.md
├── rofi/
│   ├── theme.rasi                 # Centralized Rofi CSS token theme
│   └── scripts/
│       ├── clipboard.sh           # Cliphist clipboard manager
│       ├── powermenu.sh           # Hyprland session power menu
│       └── wallpaper.sh           # Wallpaper dashboard bridge
├── starship/
│   └── starship.toml              # Starship prompt configuration
├── tmux/
│   └── tmux.conf                  # Tmux terminal multiplexer configuration
├── waybar/
│   ├── config.jsonc               # Waybar panel layout
│   └── style.css                  # Waybar CSS styling
├── yazi/
│   └── yazi.toml                  # Yazi file manager configuration
└── zsh/
    ├── .zshrc                     # Main Zsh configuration
    └── aliases.zsh                # Defensive guarded CLI aliases module
```

---

## ⌨️ Keybindings Reference

| Key Combo | Action |
|---|---|
| `SUPER + Return` | Launch Kitty terminal |
| `SUPER + Space` | Launch Rofi launcher |
| `SUPER + Q` | Close active window |
| `SUPER + F` | Toggle fullscreen |
| `SUPER + V` | Toggle floating mode |
| `SUPER + X` | Launch Rofi clipboard history (`cliphist`) |
| `SUPER + W` | Launch Rofi wallpaper picker (`awww`) |
| `SUPER + Escape` | Launch Rofi power menu |
| `SUPER + SHIFT + L` | Lock screen (`hyprlock`) |
| `SUPER + H / J / K / L` | Vim-style window focus navigation |
| `SUPER + 1..9` | Switch workspace |
| `SUPER + SHIFT + 1..9` | Move window to workspace |
| `XF86Audio*` / `XF86MonBrightness*` | Media controls + zero-latency OSD |
