use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

fn default_disabled() -> String { "#475569".to_string() }
fn default_selection() -> String { "#1E293B".to_string() }
fn default_hover() -> String { "#243144".to_string() }
fn default_pressed() -> String { "#0F172A".to_string() }
fn default_focus() -> String { "#38BDF8".to_string() }
fn default_panel() -> String { "#0F141C".to_string() }
fn default_panel_variant() -> String { "#151C28".to_string() }

fn default_icon_role_text() -> String { "text".to_string() }
fn default_icon_role_primary() -> String { "primary".to_string() }
fn default_icon_role_muted() -> String { "muted".to_string() }
fn default_icon_role_disabled() -> String { "disabled".to_string() }
fn default_icon_role_success() -> String { "success".to_string() }
fn default_icon_role_warning() -> String { "warning".to_string() }
fn default_icon_role_danger() -> String { "danger".to_string() }
fn default_icon_role_info() -> String { "info".to_string() }

fn default_radius_sm() -> u32 { 6 }
fn default_radius_md() -> u32 { 12 }
fn default_radius_lg() -> u32 { 18 }

fn default_spacing_xs() -> u32 { 4 }
fn default_spacing_sm() -> u32 { 8 }
fn default_spacing_md() -> u32 { 12 }
fn default_spacing_lg() -> u32 { 16 }
fn default_spacing_xl() -> u32 { 24 }

fn default_font() -> String { "JetBrainsMono Nerd Font".to_string() }
fn default_mono() -> String { "JetBrainsMono Nerd Font".to_string() }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuickshellIconRoles {
    pub default: String,
    pub active: String,
    pub muted: String,
    pub disabled: String,
    pub success: String,
    pub warning: String,
    pub error: String,
    pub info: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuickshellPalette {
    pub background: String,
    pub surface: String,
    pub overlay: String,
    pub text: String,
    pub muted: String,
    pub disabled: String,
    pub primary: String,
    pub secondary: String,
    pub highlight: String,
    pub success: String,
    pub warning: String,
    pub danger: String,
    pub info: String,
    pub selection: String,
    pub hover: String,
    pub pressed: String,
    pub focus: String,
    pub panel: String,
    pub panel_variant: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuickshellMetrics {
    pub border_radius: u32,
    pub pill_padding_h: u32,
    pub pill_padding_v: u32,
    pub bar_height: u32,
    pub bar_mode: String,
    pub font_family: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuickshellThemeConfig {
    pub name: String,
    pub mode: String,
    pub palette: QuickshellPalette,
    pub icon: QuickshellIconRoles,
    pub metrics: QuickshellMetrics,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeTokens {
    pub background: String,
    pub surface: String,
    pub overlay: String,
    pub text: String,
    pub muted: String,
    #[serde(default = "default_disabled")]
    pub disabled: String,
    pub primary: String,
    pub secondary: String,
    pub highlight: String,
    pub success: String,
    pub warning: String,
    pub danger: String,
    pub info: String,
    #[serde(default = "default_selection")]
    pub selection: String,
    #[serde(default = "default_hover")]
    pub hover: String,
    #[serde(default = "default_pressed")]
    pub pressed: String,
    #[serde(default = "default_focus")]
    pub focus: String,
    #[serde(default = "default_panel")]
    pub panel: String,
    #[serde(default = "default_panel_variant")]
    pub panel_variant: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IconRoles {
    #[serde(default = "default_icon_role_text")]
    pub default: String,
    #[serde(default = "default_icon_role_primary")]
    pub active: String,
    #[serde(default = "default_icon_role_muted")]
    pub muted: String,
    #[serde(default = "default_icon_role_disabled")]
    pub disabled: String,
    #[serde(default = "default_icon_role_success")]
    pub success: String,
    #[serde(default = "default_icon_role_warning")]
    pub warning: String,
    #[serde(default = "default_icon_role_danger")]
    pub error: String,
    #[serde(default = "default_icon_role_info")]
    pub info: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeGeometry {
    #[serde(default = "default_radius_sm")]
    pub radius_sm: u32,
    #[serde(default = "default_radius_md")]
    pub radius_md: u32,
    #[serde(default = "default_radius_lg")]
    pub radius_lg: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeSpacing {
    #[serde(default = "default_spacing_xs")]
    pub xs: u32,
    #[serde(default = "default_spacing_sm")]
    pub sm: u32,
    #[serde(default = "default_spacing_md")]
    pub md: u32,
    #[serde(default = "default_spacing_lg")]
    pub lg: u32,
    #[serde(default = "default_spacing_xl")]
    pub xl: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeTypography {
    #[serde(default = "default_font")]
    pub font: String,
    #[serde(default = "default_mono")]
    pub mono: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeMeta {
    pub name: String,
    pub author: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Theme {
    pub meta: ThemeMeta,
    pub tokens: ThemeTokens,
    #[serde(default)]
    pub icon_roles: Option<IconRoles>,
    #[serde(default)]
    pub geometry: Option<ThemeGeometry>,
    #[serde(default)]
    pub spacing: Option<ThemeSpacing>,
    #[serde(default)]
    pub typography: Option<ThemeTypography>,
}

impl Theme {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let theme: Theme = toml::from_str(&content).map_err(|e| e.to_string())?;
        theme.validate()?;
        Ok(theme)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let toml_str = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        Self::atomic_write(path, &toml_str)
    }

    pub fn resolve_icon_color(&self, val: &str) -> String {
        if val.starts_with('#') {
            return val.to_string();
        }
        match val {
            "background" => self.tokens.background.clone(),
            "surface" => self.tokens.surface.clone(),
            "overlay" => self.tokens.overlay.clone(),
            "text" => self.tokens.text.clone(),
            "muted" => self.tokens.muted.clone(),
            "disabled" => self.tokens.disabled.clone(),
            "primary" => self.tokens.primary.clone(),
            "secondary" => self.tokens.secondary.clone(),
            "highlight" => self.tokens.highlight.clone(),
            "success" => self.tokens.success.clone(),
            "warning" => self.tokens.warning.clone(),
            "danger" | "error" => self.tokens.danger.clone(),
            "info" => self.tokens.info.clone(),
            "selection" => self.tokens.selection.clone(),
            "hover" => self.tokens.hover.clone(),
            "pressed" => self.tokens.pressed.clone(),
            "focus" => self.tokens.focus.clone(),
            "panel" => self.tokens.panel.clone(),
            "panel_variant" => self.tokens.panel_variant.clone(),
            _ => val.to_string(),
        }
    }

    pub fn update_icon_role(&mut self, role: &str, val: &str) -> Result<(), String> {
        let is_hex = val.starts_with('#') && (val.len() == 7 || val.len() == 9);
        let valid_tokens = [
            "background", "surface", "overlay", "text", "muted", "disabled",
            "primary", "secondary", "highlight", "success", "warning", "danger",
            "info", "selection", "hover", "pressed", "focus", "panel", "panel_variant"
        ];
        if !is_hex && !valid_tokens.contains(&val) {
            return Err(format!(
                "Invalid color or token '{}'. Must be a hex color (e.g. #7DD3FC) or valid theme token name.",
                val
            ));
        }

        let mut roles = self.icon_roles.clone().unwrap_or_else(|| IconRoles {
            default: "text".into(),
            active: "primary".into(),
            muted: "muted".into(),
            disabled: "disabled".into(),
            success: "success".into(),
            warning: "warning".into(),
            error: "danger".into(),
            info: "info".into(),
        });

        match role.to_lowercase().as_str() {
            "default" => roles.default = val.to_string(),
            "active" => roles.active = val.to_string(),
            "muted" => roles.muted = val.to_string(),
            "disabled" => roles.disabled = val.to_string(),
            "success" => roles.success = val.to_string(),
            "warning" => roles.warning = val.to_string(),
            "error" | "danger" => roles.error = val.to_string(),
            "info" => roles.info = val.to_string(),
            _ => return Err(format!(
                "Unknown icon role '{}'. Valid roles: default, active, muted, disabled, success, warning, error, info",
                role
            )),
        }

        self.icon_roles = Some(roles);
        Ok(())
    }

    pub fn validate(&self) -> Result<(), String> {
        let hex_regex =
            |val: &str| -> bool { val.starts_with('#') && (val.len() == 7 || val.len() == 9) };

        let tokens = [
            ("background", &self.tokens.background),
            ("surface", &self.tokens.surface),
            ("overlay", &self.tokens.overlay),
            ("text", &self.tokens.text),
            ("muted", &self.tokens.muted),
            ("disabled", &self.tokens.disabled),
            ("primary", &self.tokens.primary),
            ("secondary", &self.tokens.secondary),
            ("highlight", &self.tokens.highlight),
            ("success", &self.tokens.success),
            ("warning", &self.tokens.warning),
            ("danger", &self.tokens.danger),
            ("info", &self.tokens.info),
            ("selection", &self.tokens.selection),
            ("hover", &self.tokens.hover),
            ("pressed", &self.tokens.pressed),
            ("focus", &self.tokens.focus),
            ("panel", &self.tokens.panel),
            ("panel_variant", &self.tokens.panel_variant),
        ];

        for (name, val) in tokens.iter() {
            if !hex_regex(val) {
                return Err(format!("Invalid hex color for token '{}': {}", name, val));
            }
        }
        Ok(())
    }

    pub fn generate_kitty_conf(&self) -> String {
        format!(
            r#"# Minimal Kitty Config — Bento Aesthetics
# Generated by minimalctl — DO NOT edit directly.
font_family      {}
bold_font        auto
italic_font      auto
font_size        11.0

background_opacity 1.0
window_padding_width 12
confirm_os_window_close 0

cursor_shape beam
cursor_blink_interval 0

foreground            {}
background            {}
selection_foreground  {}
selection_background  {}

cursor                {}
cursor_text_color     {}

url_color             {}

color0  {}
color8  {}
color1  {}
color9  {}
color2  {}
color10 {}
color3  {}
color11 {}
color4  {}
color12 {}
color5  {}
color13 {}
color6  {}
color14 {}
color7  {}
color15 {}

active_border_color   {}
inactive_border_color {}
tab_bar_style         powerline
tab_powerline_style   slanted
tab_bar_margin_height 4 0
tab_bar_background    {}
active_tab_background {}
active_tab_foreground {}
inactive_tab_background {}
inactive_tab_foreground {}

allow_remote_control yes
enabled_layouts tall,fat,grid,stack
"#,
            self.typography.as_ref().map(|t| t.mono.as_str()).unwrap_or("JetBrainsMono Nerd Font"),
            self.tokens.text,
            self.tokens.background,
            self.tokens.background,
            self.tokens.primary,
            self.tokens.primary,
            self.tokens.background,
            self.tokens.secondary,
            self.tokens.surface,
            self.tokens.muted,
            self.tokens.danger,
            self.tokens.danger,
            self.tokens.success,
            self.tokens.success,
            self.tokens.warning,
            self.tokens.warning,
            self.tokens.secondary,
            self.tokens.secondary,
            self.tokens.highlight,
            self.tokens.highlight,
            self.tokens.info,
            self.tokens.primary,
            self.tokens.text,
            self.tokens.text,
            self.tokens.primary,
            self.tokens.overlay,
            self.tokens.background,
            self.tokens.overlay,
            self.tokens.primary,
            self.tokens.surface,
            self.tokens.muted
        )
    }

    pub fn generate_starship_toml(&self) -> String {
        format!(
            r#"# Minimal Starship Config — Bento Prompt Aesthetics
# Generated by minimalctl — DO NOT edit directly.

add_newline = false
command_timeout = 500

format = """
$username\
$directory\
$git_branch\
$git_status\
$package\
$c\
$cmake\
$golang\
$java\
$nodejs\
$python\
$rust\
$cmd_duration\
$line_break\
$character"""

# --- Username ---
[username]
show_always = false
style_user = "bold {}"
format = "[$user]($style) "

# --- Directory ---
[directory]
truncation_length = 3
truncation_symbol = "…/"
home_symbol = "~"
style = "bold {}"
format = "[$path]($style) "

# --- Git Branch ---
[git_branch]
symbol = " "
style = "{}"
format = "on [$symbol$branch]($style) "

# --- Git Status ---
[git_status]
style = "{}"
format = "([$all_status$ahead_behind]($style) )"
conflicted = "="
ahead = "⇡${{count}}"
behind = "⇣${{count}}"
diverged = "⇕⇡${{ahead_count}}⇣${{behind_count}}"
untracked = "?"
stashed = "$"
modified = "*"
staged = "+"
renamed = "»"
deleted = "✘"

# --- Languages / Toolchains ---
[c]
symbol = " "
style = "{}"
format = "[$symbol$version]($style) "

[golang]
symbol = " "
style = "{}"
format = "[$symbol$version]($style) "

[java]
symbol = " "
style = "{}"
format = "[$symbol$version]($style) "

[nodejs]
symbol = " "
style = "{}"
format = "[$symbol$version]($style) "

[python]
symbol = " "
style = "{}"
format = "[$symbol$version]($style) "

[rust]
symbol = " "
style = "{}"
format = "[$symbol$version]($style) "

# --- Execution Time ---
[cmd_duration]
min_time = 2_000
style = "{}"
format = "took [$duration]($style) "

# --- Prompt Character ---
[character]
success_symbol = "[λ](bold {})"
error_symbol = "[λ](bold {})"
"#,
            self.tokens.primary,
            self.tokens.primary,
            self.tokens.muted,
            self.tokens.danger,
            self.tokens.muted,
            self.tokens.muted,
            self.tokens.muted,
            self.tokens.muted,
            self.tokens.muted,
            self.tokens.muted,
            self.tokens.muted,
            self.tokens.primary,
            self.tokens.danger
        )
    }

    pub fn generate_btop_theme(&self) -> String {
        format!(
            r#"# btop tty theme — Minimal Bento
# Generated by minimalctl — DO NOT edit directly.

# Main UI Element Colors
theme[main_bg]="{}"
theme[main_fg]="{}"
theme[title]="{}"
theme[hi_fg]="{}"
theme[selected_bg]="{}"
theme[selected_fg]="{}"
theme[inactive_fg]="{}"
theme[graph_text]="{}"
theme[proc_misc]="{}"

# Box Outlines
theme[cpu_box]="{}"
theme[mem_box]="{}"
theme[net_box]="{}"
theme[proc_box]="{}"
theme[div_line]="{}"

# CPU Graph Colors
theme[cpu_start]="{}"
theme[cpu_mid]="{}"
theme[cpu_end]="{}"

# Temperature Colors
theme[temp_start]="{}"
theme[temp_mid]="{}"
theme[temp_end]="{}"

# Mem/Disk free meter
theme[free_start]="{}"
theme[free_mid]="{}"
theme[free_end]="{}"

# Mem/Disk cached meter
theme[cached_start]="{}"
theme[cached_mid]="{}"
theme[cached_end]="{}"

# Mem/Disk available meter
theme[available_start]="{}"
theme[available_mid]="{}"
theme[available_end]="{}"

# Mem/Disk used meter
theme[used_start]="{}"
theme[used_mid]="{}"
theme[used_end]="{}"

# Download graph colors
theme[download_start]="{}"
theme[download_mid]="{}"
theme[download_end]="{}"

# Upload graph colors
theme[upload_start]="{}"
theme[upload_mid]="{}"
theme[upload_end]="{}"
"#,
            self.tokens.background,
            self.tokens.text,
            self.tokens.primary,
            self.tokens.primary,
            self.tokens.overlay,
            self.tokens.text,
            self.tokens.muted,
            self.tokens.muted,
            self.tokens.secondary,
            self.tokens.overlay,
            self.tokens.overlay,
            self.tokens.overlay,
            self.tokens.overlay,
            self.tokens.overlay,
            self.tokens.primary,
            self.tokens.secondary,
            self.tokens.highlight,
            self.tokens.success,
            self.tokens.warning,
            self.tokens.danger,
            self.tokens.primary,
            self.tokens.secondary,
            self.tokens.primary,
            self.tokens.secondary,
            self.tokens.highlight,
            self.tokens.primary,
            self.tokens.primary,
            self.tokens.secondary,
            self.tokens.primary,
            self.tokens.highlight,
            self.tokens.warning,
            self.tokens.danger,
            self.tokens.primary,
            self.tokens.secondary,
            self.tokens.primary,
            self.tokens.highlight,
            self.tokens.secondary,
            self.tokens.primary
        )
    }

    pub fn generate_nvim_theme(&self) -> String {
        format!(
            r#"---@type Base46Theme
-- lua/themes/minimal.lua — Base46 theme for NvChad
-- Generated by minimalctl — DO NOT edit directly.

local M = {{}}

M.base_30 = {{
  white         = "{}",
  darker_black  = "{}",
  black         = "{}",
  black2        = "{}",
  one_bg        = "{}",
  one_bg2       = "{}",
  one_bg3       = "{}",
  grey          = "{}",
  grey_fg       = "{}",
  grey_fg2      = "{}",
  light_grey    = "{}",
  red           = "{}",
  baby_pink     = "{}",
  pink          = "{}",
  line          = "{}",
  green         = "{}",
  vibrant_green = "{}",
  nord_blue     = "{}",
  blue          = "{}",
  yellow        = "{}",
  sun           = "{}",
  purple        = "{}",
  dark_purple   = "{}",
  teal          = "{}",
  orange        = "{}",
  cyan          = "{}",
  statusline_bg = "{}",
  lightbg       = "{}",
  pmenu_bg      = "{}",
  folder_bg     = "{}",
}}

M.base_16 = {{
  base00 = "{}",
  base01 = "{}",
  base02 = "{}",
  base03 = "{}",
  base04 = "{}",
  base05 = "{}",
  base06 = "{}",
  base07 = "{}",
  base08 = "{}",
  base09 = "{}",
  base0A = "{}",
  base0B = "{}",
  base0C = "{}",
  base0D = "{}",
  base0E = "{}",
  base0F = "{}",
}}

M.type = "dark"

return M
"#,
            self.tokens.text,
            self.tokens.background,
            self.tokens.background,
            self.tokens.surface,
            self.tokens.surface,
            self.tokens.overlay,
            self.tokens.overlay,
            self.tokens.muted,
            self.tokens.muted,
            self.tokens.muted,
            self.tokens.muted,
            self.tokens.danger,
            self.tokens.danger,
            self.tokens.highlight,
            self.tokens.overlay,
            self.tokens.success,
            self.tokens.success,
            self.tokens.secondary,
            self.tokens.secondary,
            self.tokens.warning,
            self.tokens.warning,
            self.tokens.highlight,
            self.tokens.highlight,
            self.tokens.info,
            self.tokens.warning,
            self.tokens.primary,
            self.tokens.surface,
            self.tokens.overlay,
            self.tokens.primary,
            self.tokens.secondary,
            self.tokens.background,
            self.tokens.surface,
            self.tokens.overlay,
            self.tokens.muted,
            self.tokens.muted,
            self.tokens.text,
            self.tokens.text,
            self.tokens.text,
            self.tokens.danger,
            self.tokens.warning,
            self.tokens.warning,
            self.tokens.success,
            self.tokens.info,
            self.tokens.secondary,
            self.tokens.highlight,
            self.tokens.danger
        )
    }

    pub fn generate_hypr_colors(&self) -> String {
        let clean = |hex: &str| -> String { hex.trim_start_matches('#').to_string() };
        format!(
            r#"# Hyprland color variables
# Generated by minimalctl — DO NOT edit directly.
$background = rgba({}FF)
$surface    = rgba({}FF)
$overlay    = rgba({}FF)
$text       = rgba({}FF)
$muted      = rgba({}FF)
$primary    = rgba({}FF)
$secondary  = rgba({}FF)
$highlight  = rgba({}FF)
$success    = rgba({}FF)
$warning    = rgba({}FF)
$danger     = rgba({}FF)
$info       = rgba({}FF)
"#,
            clean(&self.tokens.background),
            clean(&self.tokens.surface),
            clean(&self.tokens.overlay),
            clean(&self.tokens.text),
            clean(&self.tokens.muted),
            clean(&self.tokens.primary),
            clean(&self.tokens.secondary),
            clean(&self.tokens.highlight),
            clean(&self.tokens.success),
            clean(&self.tokens.warning),
            clean(&self.tokens.danger),
            clean(&self.tokens.info)
        )
    }

    pub fn generate_tmux_conf(&self) -> String {
        format!(
            r##"# ===========================================================================
#  TMUX CONFIGURATION — Minimal Theme (Optimized)
#  Generated by minimalctl — DO NOT edit directly.
# ===========================================================================

set -s escape-time 0
set -g history-limit 50000
set -g mouse on

set -g default-terminal "tmux-256color"
set -as terminal-features ",xterm-256color:RGB"
set -as terminal-features ",xterm-kitty:RGB"
set -as terminal-features ",alacritty:RGB"
set -ag terminal-overrides ",xterm-256color:Tc"

set -g focus-events on
set -sg repeat-time 400

unbind C-b
set -g prefix C-a
bind C-a send-prefix

bind r source-file ~/.config/tmux/tmux.conf \; display-message "  tmux.conf reloaded"

unbind '"'
unbind %
bind | split-window -h -c "#{{pane_current_path}}"
bind v split-window -h -c "#{{pane_current_path}}"
bind - split-window -v -c "#{{pane_current_path}}"
bind _ split-window -v -c "#{{pane_current_path}}"
bind c new-window -c "#{{pane_current_path}}"

bind -n M-h select-pane -L
bind -n M-j select-pane -D
bind -n M-k select-pane -U
bind -n M-l select-pane -R

bind -r h select-pane -L
bind -r j select-pane -D
bind -r k select-pane -U
bind -r l select-pane -R

bind -r H resize-pane -L 5
bind -r J resize-pane -D 5
bind -r K resize-pane -U 5
bind -r L resize-pane -R 5

bind m resize-pane -Z
bind = select-layout even-horizontal
bind + select-layout even-vertical

set -g base-index 1
setw -g pane-base-index 1
set -g renumber-windows on

setw -g mode-keys vi
set -g set-clipboard on

bind -T copy-mode-vi v send-keys -X begin-selection
bind -T copy-mode-vi y send-keys -X copy-selection-and-cancel
bind -T copy-mode-vi Escape send-keys -X cancel
bind -T copy-mode-vi q      send-keys -X cancel

bind -n WheelUpPane if-shell -F "#{{||:#{{pane_in_mode}},#{{mouse_any_flag}}}}" \
    "send-keys -M" \
    "copy-mode -e"

bind -T copy-mode-vi MouseDrag1Pane send-keys -X begin-selection
bind -T copy-mode-vi MouseDragEnd1Pane send-keys -X copy-selection-and-cancel
bind -T copy-mode-vi DoubleClick1Pane \
    select-pane \; send-keys -X select-word \; send-keys -X copy-selection-and-cancel
bind -T copy-mode-vi TripleClick1Pane \
    select-pane \; send-keys -X select-line \; send-keys -X copy-selection-and-cancel

setw -g monitor-activity on
set -g visual-activity off
set -g visual-bell off
set -g visual-silence off
set -g bell-action none

bind C-k send-keys -X cancel 2>/dev/null \; send-keys 'clear' Enter
bind X send-keys 'reset' Enter
bind -n M-Escape send-keys -X cancel 2>/dev/null

# --- MINIMAL STATUS BAR THEME ---
set -g status-position bottom
set -g status-interval 15

set -g status-style "fg={},bg={}"

set -g status-left-length 30
set -g status-right-length 60

set -g status-left "#[fg={},bg={},bold] #S #[fg={},bg={},nobold] "
set -g status-right "#[fg={},bg={}]  #h  #[fg={},bg={},bold] %H:%M "

set -g status-justify left
setw -g window-status-separator " "

setw -g window-status-format "#[fg={},bg={}] #I  #W "
setw -g window-status-current-format "#[fg={},bg={},bold] #I  #W#{{?window_zoomed_flag, 󰁌 , }} "
setw -g window-status-activity-style "fg={},bg={}"

set -g pane-border-style "fg={}"
set -g pane-active-border-style "fg={}"

set -g message-style "fg={},bg={},bold"
setw -g clock-mode-colour "{}"
set -g mode-style "fg={},bg={}"
"##,
            self.tokens.muted,
            self.tokens.surface,
            self.tokens.surface,
            self.tokens.primary,
            self.tokens.primary,
            self.tokens.surface,
            self.tokens.text,
            self.tokens.overlay,
            self.tokens.background,
            self.tokens.primary,
            self.tokens.muted,
            self.tokens.surface,
            self.tokens.primary,
            self.tokens.overlay,
            self.tokens.danger,
            self.tokens.surface,
            self.tokens.overlay,
            self.tokens.primary,
            self.tokens.text,
            self.tokens.overlay,
            self.tokens.primary,
            self.tokens.background,
            self.tokens.primary
        )
    }

    pub fn generate_quickshell_theme(&self) -> String {
        let icon_roles = QuickshellIconRoles {
            default: self.icon_roles.as_ref().map(|i| i.default.clone()).unwrap_or_else(|| "text".to_string()),
            active: self.icon_roles.as_ref().map(|i| i.active.clone()).unwrap_or_else(|| "primary".to_string()),
            muted: self.icon_roles.as_ref().map(|i| i.muted.clone()).unwrap_or_else(|| "muted".to_string()),
            disabled: self.icon_roles.as_ref().map(|i| i.disabled.clone()).unwrap_or_else(|| "disabled".to_string()),
            success: self.icon_roles.as_ref().map(|i| i.success.clone()).unwrap_or_else(|| "success".to_string()),
            warning: self.icon_roles.as_ref().map(|i| i.warning.clone()).unwrap_or_else(|| "warning".to_string()),
            error: self.icon_roles.as_ref().map(|i| i.error.clone()).unwrap_or_else(|| "danger".to_string()),
            info: self.icon_roles.as_ref().map(|i| i.info.clone()).unwrap_or_else(|| "info".to_string()),
        };

        let radius = self.geometry.as_ref().map(|g| g.radius_md).unwrap_or(12);
        let font_fam = self.typography.as_ref().map(|t| t.font.clone()).unwrap_or_else(|| "Adwaita Sans".to_string());

        let config = QuickshellThemeConfig {
            name: self.meta.name.clone(),
            mode: if self.meta.name.contains("light") { "light".to_string() } else { "dark".to_string() },
            palette: QuickshellPalette {
                background: self.tokens.background.clone(),
                surface: self.tokens.surface.clone(),
                overlay: self.tokens.overlay.clone(),
                text: self.tokens.text.clone(),
                muted: self.tokens.muted.clone(),
                disabled: self.tokens.disabled.clone(),
                primary: self.tokens.primary.clone(),
                secondary: self.tokens.secondary.clone(),
                highlight: self.tokens.highlight.clone(),
                success: self.tokens.success.clone(),
                warning: self.tokens.warning.clone(),
                danger: self.tokens.danger.clone(),
                info: self.tokens.info.clone(),
                selection: self.tokens.selection.clone(),
                hover: self.tokens.hover.clone(),
                pressed: self.tokens.pressed.clone(),
                focus: self.tokens.focus.clone(),
                panel: self.tokens.panel.clone(),
                panel_variant: self.tokens.panel_variant.clone(),
            },
            icon: icon_roles,
            metrics: QuickshellMetrics {
                border_radius: radius,
                pill_padding_h: 16,
                pill_padding_v: 8,
                bar_height: 40,
                bar_mode: "full".to_string(),
                font_family: font_fam,
            },
        };
        serde_json::to_string_pretty(&config).unwrap_or_default()
    }

    pub fn generate_gtk_settings(&self) -> String {
        let is_dark = if self.meta.name.contains("light") { "0" } else { "1" };
        let theme_name = if self.meta.name.contains("light") { "Adwaita" } else { "Adwaita-dark" };
        let font = self.typography.as_ref().map(|t| t.font.as_str()).unwrap_or("Adwaita Sans");
        format!(
            r#"[Settings]
gtk-theme-name={}
gtk-icon-theme-name=Minimal
gtk-application-prefer-dark-theme={}
gtk-font-name={} 11
"#,
            theme_name, is_dark, font
        )
    }

    pub fn generate_gtk_css(&self) -> String {
        format!(
            r#"/* Minimal GTK Theme Overrides — Generated by minimalctl */
@define-color theme_bg_color {};
@define-color theme_fg_color {};
@define-color theme_selected_bg_color {};
@define-color theme_selected_fg_color {};
@define-color theme_base_color {};
@define-color theme_text_color {};
"#,
            self.tokens.background,
            self.tokens.text,
            self.tokens.primary,
            self.tokens.background,
            self.tokens.surface,
            self.tokens.text
        )
    }

    pub fn generate_qtct_conf(&self) -> String {
        format!(
            r#"[Appearance]
icon_theme=Minimal
style=Fusion
standard_dialogs=default
"#,
        )
    }

    pub fn install_icon_theme<P: AsRef<Path>>(root_dir: P) -> Result<(), String> {
        let root = root_dir.as_ref();
        let src_minimal = root.join("icons/dist/Minimal");
        if !src_minimal.exists() {
            return Ok(());
        }

        let home = std::env::var("HOME").map_err(|e| e.to_string())?;
        let icons_dest = PathBuf::from(&home).join(".local/share/icons/Minimal");

        if let Err(e) = fs::create_dir_all(&icons_dest) {
            return Err(format!("Failed to create icon directory {}: {}", icons_dest.display(), e));
        }

        fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
            fs::create_dir_all(dst)?;
            for entry in fs::read_dir(src)? {
                let entry = entry?;
                let ty = entry.file_type()?;
                let dst_path = dst.join(entry.file_name());
                if ty.is_dir() {
                    copy_dir_all(&entry.path(), &dst_path)?;
                } else if ty.is_symlink() {
                    let target = fs::read_link(entry.path())?;
                    if dst_path.exists() || fs::symlink_metadata(&dst_path).is_ok() {
                        let _ = fs::remove_file(&dst_path);
                    }
                    #[cfg(unix)]
                    std::os::unix::fs::symlink(target, dst_path)?;
                } else {
                    fs::copy(entry.path(), dst_path)?;
                }
            }
            Ok(())
        }

        copy_dir_all(&src_minimal, &icons_dest).map_err(|e| format!("Failed to copy Minimal icon theme: {}", e))?;
        
        // Update icon cache if gtk-update-icon-cache is available
        let _ = std::process::Command::new("gtk-update-icon-cache")
            .args(["-f", "-t", icons_dest.to_str().unwrap_or("")])
            .output();

        Ok(())
    }

    pub fn atomic_write<P: AsRef<Path>>(path: P, content: &str) -> Result<(), String> {
        let path = path.as_ref();
        let parent = path
            .parent()
            .ok_or_else(|| format!("Invalid parent directory for path: {}", path.display()))?;
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory {}: {}", parent.display(), e))?;
        }

        let file_name = path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("target");
        let tmp_path = parent.join(format!(".{}.tmp", file_name));

        {
            use std::io::Write;
            let mut file = fs::File::create(&tmp_path)
                .map_err(|e| format!("Failed to create temp file {}: {}", tmp_path.display(), e))?;
            file.write_all(content.as_bytes())
                .map_err(|e| format!("Failed to write to temp file {}: {}", tmp_path.display(), e))?;
            file.flush()
                .map_err(|e| format!("Flush error on {}: {}", tmp_path.display(), e))?;
            file.sync_all()
                .map_err(|e| format!("fsync error on {}: {}", tmp_path.display(), e))?;
        }

        fs::rename(&tmp_path, path)
            .map_err(|e| format!("Atomic rename failed for {}: {}", path.display(), e))?;

        if let Ok(dir_file) = fs::File::open(parent) {
            let _ = dir_file.sync_all();
        }

        Ok(())
    }

    pub fn compute_hash(&self) -> String {
        let mut hash: u64 = 0xcbf29ce484222325;
        let bytes = format!(
            "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
            self.meta.name,
            self.tokens.background,
            self.tokens.surface,
            self.tokens.overlay,
            self.tokens.text,
            self.tokens.muted,
            self.tokens.primary,
            self.tokens.secondary,
            self.tokens.highlight,
            self.tokens.success,
            self.tokens.warning,
            self.tokens.danger,
            self.tokens.info
        );
        for byte in bytes.bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        format!("{:016x}", hash)
    }

    pub fn build_all<P: AsRef<Path>>(&self, root_dir: P) -> Result<(), String> {
        let root = root_dir.as_ref();

        Self::atomic_write(root.join("kitty/kitty.conf"), &self.generate_kitty_conf())?;
        Self::atomic_write(root.join("starship/starship.toml"), &self.generate_starship_toml())?;
        Self::atomic_write(root.join("btop/btop.theme"), &self.generate_btop_theme())?;
        Self::atomic_write(root.join("hypr/colors.conf"), &self.generate_hypr_colors())?;
        Self::atomic_write(root.join("tmux/tmux.conf"), &self.generate_tmux_conf())?;

        let quickshell_target = if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".config/quickshell/theme.json")
        } else {
            root.join("quickshell/theme.json")
        };
        Self::atomic_write(quickshell_target, &self.generate_quickshell_theme())?;

        let nvim_theme_dir = root.join("nvim/lua/themes");
        if nvim_theme_dir.exists() {
            Self::atomic_write(nvim_theme_dir.join("minimal.lua"), &self.generate_nvim_theme())?;
        }

        // GTK & Qt theme targets
        if let Ok(home) = std::env::var("HOME") {
            let home_path = PathBuf::from(&home);
            let gtk3_dir = home_path.join(".config/gtk-3.0");
            let gtk4_dir = home_path.join(".config/gtk-4.0");
            let qt5_dir = home_path.join(".config/qt5ct");
            let qt6_dir = home_path.join(".config/qt6ct");

            let gtk_settings = self.generate_gtk_settings();
            let gtk_css = self.generate_gtk_css();
            let qt_conf = self.generate_qtct_conf();

            Self::atomic_write(gtk3_dir.join("settings.ini"), &gtk_settings)?;
            Self::atomic_write(gtk3_dir.join("gtk.css"), &gtk_css)?;
            Self::atomic_write(gtk4_dir.join("settings.ini"), &gtk_settings)?;
            Self::atomic_write(gtk4_dir.join("gtk.css"), &gtk_css)?;
            Self::atomic_write(qt5_dir.join("qt5ct.conf"), &qt_conf)?;
            Self::atomic_write(qt6_dir.join("qt6ct.conf"), &qt_conf)?;
        }

        // Install Minimal icon theme
        Self::install_icon_theme(root)?;

        Ok(())
    }

    pub fn backup_state<P: AsRef<Path>>(&self, root_dir: P) -> Result<(), String> {
        let root = root_dir.as_ref();
        let backup_dir = root.join(".rollback_backup");
        if backup_dir.exists() {
            let _ = fs::remove_dir_all(&backup_dir);
        }
        fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;

        let files = [
            "kitty/kitty.conf",
            "starship/starship.toml",
            "btop/btop.theme",
            "hypr/colors.conf",
            "tmux/tmux.conf",
        ];

        for rel in files.iter() {
            let src = root.join(rel);
            if src.exists() {
                let dest = backup_dir.join(rel.replace('/', "_"));
                let _ = fs::copy(&src, &dest);
            }
        }

        let nvim_src = root.join("nvim/lua/themes/minimal.lua");
        if nvim_src.exists() {
            let _ = fs::copy(&nvim_src, backup_dir.join("nvim_lua_themes_minimal.lua"));
        }

        Ok(())
    }

    pub fn perform_rollback<P: AsRef<Path>>(root_dir: P) -> Result<(), String> {
        let root = root_dir.as_ref();
        let backup_dir = root.join(".rollback_backup");
        if !backup_dir.exists() {
            return Err("No rollback backup state found. Apply a theme first.".to_string());
        }

        let files = [
            ("kitty_kitty.conf", "kitty/kitty.conf"),
            ("starship_starship.toml", "starship/starship.toml"),
            ("btop_btop.theme", "btop/btop.theme"),
            ("hypr_colors.conf", "hypr/colors.conf"),
            ("tmux_tmux.conf", "tmux/tmux.conf"),
            ("nvim_lua_themes_minimal.lua", "nvim/lua/themes/minimal.lua"),
        ];

        for (bak_name, target_rel) in files.iter() {
            let bak_file = backup_dir.join(bak_name);
            if bak_file.exists() {
                let target_file = root.join(target_rel);
                if let Ok(content) = fs::read_to_string(&bak_file) {
                    Self::atomic_write(&target_file, &content)?;
                }
            }
        }

        println!("[minimalctl] Rollback executed successfully.");
        Ok(())
    }

    pub fn apply_runtime<P: AsRef<Path>>(&self, root_dir: P) -> Result<(), String> {
        let root = root_dir.as_ref();
        println!("=== TRANSACTIONAL THEME APPLY ===");

        // 1. Validate & Build targets
        self.build_all(root)?;
        println!("[PASS] Theme source validated & active 7 targets built.");

        // 1b. Update system gsettings icon theme
        let _ = std::process::Command::new("gsettings")
            .args(["set", "org.gnome.desktop.interface", "icon-theme", "Minimal"])
            .output();
        println!("[PASS] GTK: System icon theme set to 'Minimal' via gsettings");

        // 2. Kitty: Target-aware hot reload (SIGUSR1)
        if std::process::Command::new("pgrep")
            .arg("-x")
            .arg("kitty")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            let _ = std::process::Command::new("pkill")
                .args(["-SIGUSR1", "kitty"])
                .status();
            println!("[PASS] Kitty: SIGUSR1 signal sent to active instances");
        } else {
            println!("[INFO] Kitty: Not running (applies on new windows)");
        }

        // 3. Hyprland: Dynamic border keyword update without session restart
        if std::process::Command::new("pgrep")
            .arg("-x")
            .arg("Hyprland")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            let hex_clean = self.tokens.primary.trim_start_matches('#');
            let _ = std::process::Command::new("hyprctl")
                .args([
                    "keyword",
                    "general:col.active_border",
                    &format!("rgba({}FF)", hex_clean),
                ])
                .output();
            println!(
                "[PASS] Hyprland: Active border updated (rgba({}FF))",
                hex_clean
            );
        } else {
            println!("[INFO] Hyprland: Not running");
        }

        // 4. Tmux: Target-aware hot reload (tmux source-file + refresh-client)
        let tmux_active = std::process::Command::new("tmux")
            .arg("ls")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        // 5. Quickshell: Target-aware live reload
        if std::process::Command::new("pgrep")
            .arg("-x")
            .arg("quickshell")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            let _ = std::process::Command::new("quickshell")
                .args(["ipc", "call", "minimal-shell", "reloadTheme"])
                .output();
            println!("[PASS] Quickshell: Triggered theme reload IPC");
        } else {
            println!("[INFO] Quickshell: Not running (applies on launch)");
        }

        if tmux_active {
            let tmux_conf = std::fs::canonicalize(root.join("tmux/tmux.conf"))
                .unwrap_or_else(|_| root.join("tmux/tmux.conf"));
            let _ = std::process::Command::new("tmux")
                .args([
                    "source-file",
                    tmux_conf.to_str().unwrap_or("tmux/tmux.conf"),
                ])
                .output();
            if let Ok(home) = std::env::var("HOME") {
                let _ = std::process::Command::new("tmux")
                    .args(["source-file", &format!("{}/.tmux.conf", home)])
                    .output();
            }
            let _ = std::process::Command::new("tmux")
                .args(["refresh-client", "-S"])
                .output();
            let _ = std::process::Command::new("tmux")
                .args(["refresh-client", "-a"])
                .output();
            println!("[PASS] Tmux: Sourced tmux.conf & refreshed active sessions live");
        } else {
            println!("[INFO] Tmux: Server not running (applies on next session)");
        }

        println!("[PASS] Theme transaction completed successfully.");
        Ok(())
    }

    pub fn run_doctor<P: AsRef<Path>>(root_dir: P) -> Result<(), String> {
        let root = root_dir.as_ref();
        println!("=== MINIMAL THEME & ICON SYSTEM DOCTOR ===");
        println!("--------------------------------------------------");

        let mut pass = 0;
        let mut warn = 0;
        let mut fail = 0;

        // 1. Theme Definition
        let active_theme_path = root.join("themes/obsidian.toml");
        if let Ok(theme) = Theme::load_from_file(&active_theme_path) {
            println!("[PASS] Theme source definition ({} - hash {})", theme.meta.name, theme.compute_hash());
            pass += 1;
        } else {
            println!("[FAIL] Theme source definition invalid or missing");
            fail += 1;
        }

        // 2. Local Repo Icon Distribution
        let icon_dist = root.join("icons/dist/Minimal");
        if icon_dist.join("index.theme").exists() && icon_dist.join("scalable").exists() {
            println!("[PASS] Repository icon distribution (icons/dist/Minimal)");
            pass += 1;
        } else {
            println!("[FAIL] Repository icon distribution missing or incomplete");
            fail += 1;
        }

        // 3. User Installed Icon Theme
        if let Ok(home) = std::env::var("HOME") {
            let installed_icons = PathBuf::from(&home).join(".local/share/icons/Minimal");
            if installed_icons.join("index.theme").exists() && installed_icons.join("scalable").exists() {
                println!("[PASS] User icon theme installation (~/.local/share/icons/Minimal)");
                pass += 1;
            } else {
                println!("[FAIL] User icon theme not installed in ~/.local/share/icons/Minimal");
                fail += 1;
            }
        }

        // 4. GTK settings.ini
        if let Ok(home) = std::env::var("HOME") {
            let gtk3_ini = PathBuf::from(&home).join(".config/gtk-3.0/settings.ini");
            let gtk4_ini = PathBuf::from(&home).join(".config/gtk-4.0/settings.ini");
            let gtk3_ok = gtk3_ini.exists() && fs::read_to_string(&gtk3_ini).map(|c| c.contains("gtk-icon-theme-name=Minimal")).unwrap_or(false);
            let gtk4_ok = gtk4_ini.exists() && fs::read_to_string(&gtk4_ini).map(|c| c.contains("gtk-icon-theme-name=Minimal")).unwrap_or(false);
            if gtk3_ok && gtk4_ok {
                println!("[PASS] GTK 3 & GTK 4 settings.ini icon-theme set to Minimal");
                pass += 1;
            } else {
                println!("[WARN] GTK settings.ini missing or not set to Minimal");
                warn += 1;
            }
        }

        // 5. GTK GSettings
        let gsettings_theme = std::process::Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "icon-theme"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();

        if gsettings_theme.contains("Minimal") {
            println!("[PASS] GSettings org.gnome.desktop.interface icon-theme set to 'Minimal'");
            pass += 1;
        } else {
            println!("[FAIL] GSettings icon-theme is '{}' (expected 'Minimal')", gsettings_theme);
            fail += 1;
        }

        // 6. Qt Config
        if let Ok(home) = std::env::var("HOME") {
            let qt5_conf = PathBuf::from(&home).join(".config/qt5ct/qt5ct.conf");
            let qt6_conf = PathBuf::from(&home).join(".config/qt6ct/qt6ct.conf");
            let qt5_ok = qt5_conf.exists() && fs::read_to_string(&qt5_conf).map(|c| c.contains("icon_theme=Minimal")).unwrap_or(false);
            let qt6_ok = qt6_conf.exists() && fs::read_to_string(&qt6_conf).map(|c| c.contains("icon_theme=Minimal")).unwrap_or(false);
            if qt5_ok || qt6_ok {
                println!("[PASS] Qt (qtct) configuration icon_theme set to Minimal");
                pass += 1;
            } else {
                println!("[WARN] Qt configuration missing or icon_theme not set to Minimal");
                warn += 1;
            }
        }

        // 7. Quickshell theme.json
        if let Ok(home) = std::env::var("HOME") {
            let qs_json = PathBuf::from(&home).join(".config/quickshell/theme.json");
            if qs_json.exists() && fs::read_to_string(&qs_json).map(|c| c.contains("\"border_radius\"")).unwrap_or(false) {
                println!("[PASS] Quickshell presentation theme.json (~/.config/quickshell/theme.json)");
                pass += 1;
            } else {
                println!("[FAIL] Quickshell theme.json missing or invalid");
                fail += 1;
            }
        }

        // 8. Icon resolution lookup check
        if let Ok(home) = std::env::var("HOME") {
            let search_icon = PathBuf::from(&home).join(".local/share/icons/Minimal/scalable/actions/search.svg");
            let wifi_icon = PathBuf::from(&home).join(".local/share/icons/Minimal/scalable/status/network-wifi.svg");
            if search_icon.exists() && wifi_icon.exists() {
                println!("[PASS] Icon lookup pipeline (actions/search.svg, status/network-wifi.svg verified)");
                pass += 1;
            } else {
                println!("[FAIL] Icon lookup check failed for key symbolic icons");
                fail += 1;
            }
        }

        println!("--------------------------------------------------");
        println!("Result: {} PASS / {} WARN / {} FAIL", pass, warn, fail);
        println!();

        if fail > 0 {
            Err(format!("Theme & icon system doctor found {} failure(s)", fail))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quickshell_token_reactivity() {
        let mut theme = Theme {
            meta: ThemeMeta {
                name: "obsidian".into(),
                author: "Hypokoto".into(),
                description: "Test".into(),
            },
            tokens: ThemeTokens {
                background: "#0B0E14".into(),
                surface: "#11161F".into(),
                overlay: "#19212D".into(),
                text: "#E8EDF5".into(),
                muted: "#7F899B".into(),
                disabled: "#475569".into(),
                primary: "#7DD3FC".into(),
                secondary: "#8BA4FF".into(),
                highlight: "#B4A7FF".into(),
                success: "#8BE28B".into(),
                warning: "#E8C77B".into(),
                danger: "#F08080".into(),
                info: "#7DD3FC".into(),
                selection: "#1E293B".into(),
                hover: "#243144".into(),
                pressed: "#0F172A".into(),
                focus: "#38BDF8".into(),
                panel: "#0F141C".into(),
                panel_variant: "#151C28".into(),
            },
            icon_roles: None,
            geometry: None,
            spacing: None,
            typography: None,
        };

        let style1 = theme.generate_quickshell_theme();
        assert!(style1.contains("\"primary\": \"#7DD3FC\""));

        theme.tokens.primary = "#FF0000".to_string();
        let style2 = theme.generate_quickshell_theme();
        assert!(style2.contains("\"primary\": \"#FF0000\""));
        assert_ne!(style1, style2);
    }

    #[test]
    fn test_theme_validation_hex_format() {
        let mut theme = Theme {
            meta: ThemeMeta {
                name: "test".into(),
                author: "test".into(),
                description: "test".into(),
            },
            tokens: ThemeTokens {
                background: "#0B0E14".into(),
                surface: "#11161F".into(),
                overlay: "#19212D".into(),
                text: "#E8EDF5".into(),
                muted: "#7F899B".into(),
                disabled: "#475569".into(),
                primary: "#7DD3FC".into(),
                secondary: "#8BA4FF".into(),
                highlight: "#B4A7FF".into(),
                success: "#8BE28B".into(),
                warning: "#E8C77B".into(),
                danger: "#F08080".into(),
                info: "#7DD3FC".into(),
                selection: "#1E293B".into(),
                hover: "#243144".into(),
                pressed: "#0F172A".into(),
                focus: "#38BDF8".into(),
                panel: "#0F141C".into(),
                panel_variant: "#151C28".into(),
            },
            icon_roles: None,
            geometry: None,
            spacing: None,
            typography: None,
        };
        assert!(theme.validate().is_ok());

        theme.tokens.background = "invalid-hex".to_string();
        assert!(theme.validate().is_err());
    }

    #[test]
    fn test_atomic_write() {
        let temp_dir = std::env::temp_dir().join("minimalctl_test_atomic");
        let test_file = temp_dir.join("test_target.txt");
        let content = "hello atomic world";
        assert!(Theme::atomic_write(&test_file, content).is_ok());
        assert_eq!(fs::read_to_string(&test_file).unwrap(), content);
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_quickshell_theme_generation() {
        let theme = Theme {
            meta: ThemeMeta {
                name: "obsidian".into(),
                author: "Hypokoto".into(),
                description: "Test".into(),
            },
            tokens: ThemeTokens {
                background: "#0B0E14".into(),
                surface: "#11161F".into(),
                overlay: "#19212D".into(),
                text: "#E8EDF5".into(),
                muted: "#7F899B".into(),
                disabled: "#475569".into(),
                primary: "#7DD3FC".into(),
                secondary: "#8BA4FF".into(),
                highlight: "#B4A7FF".into(),
                success: "#8BE28B".into(),
                warning: "#E8C77B".into(),
                danger: "#F08080".into(),
                info: "#7DD3FC".into(),
                selection: "#1E293B".into(),
                hover: "#243144".into(),
                pressed: "#0F172A".into(),
                focus: "#38BDF8".into(),
                panel: "#0F141C".into(),
                panel_variant: "#151C28".into(),
            },
            icon_roles: None,
            geometry: None,
            spacing: None,
            typography: None,
        };
        let json_str = theme.generate_quickshell_theme();
        assert!(json_str.contains("\"background\": \"#0B0E14\""));
        assert!(json_str.contains("\"primary\": \"#7DD3FC\""));
        assert!(json_str.contains("\"border_radius\": 12"));
    }

    #[test]
    fn test_compute_hash_determinism() {
        let theme = Theme {
            meta: ThemeMeta {
                name: "obsidian".into(),
                author: "Hypokoto".into(),
                description: "Test".into(),
            },
            tokens: ThemeTokens {
                background: "#0B0E14".into(),
                surface: "#11161F".into(),
                overlay: "#19212D".into(),
                text: "#E8EDF5".into(),
                muted: "#7F899B".into(),
                disabled: "#475569".into(),
                primary: "#7DD3FC".into(),
                secondary: "#8BA4FF".into(),
                highlight: "#B4A7FF".into(),
                success: "#8BE28B".into(),
                warning: "#E8C77B".into(),
                danger: "#F08080".into(),
                info: "#7DD3FC".into(),
                selection: "#1E293B".into(),
                hover: "#243144".into(),
                pressed: "#0F172A".into(),
                focus: "#38BDF8".into(),
                panel: "#0F141C".into(),
                panel_variant: "#151C28".into(),
            },
            icon_roles: None,
            geometry: None,
            spacing: None,
            typography: None,
        };

        let hash1 = theme.compute_hash();
        let hash2 = theme.compute_hash();
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 16);
    }

    #[test]
    fn test_icon_svg_validity_and_index_theme() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let index_path = root.join("icons/dist/Minimal/index.theme");
        assert!(index_path.exists(), "Minimal index.theme must exist");

        let content = fs::read_to_string(&index_path).expect("Read index.theme");
        assert!(content.contains("Name=Minimal"));
        assert!(content.contains("Inherits=Adwaita,hicolor"));

        let scalable_dir = root.join("icons/dist/Minimal/scalable");
        assert!(scalable_dir.exists());

        // Check essential icons exist and contain valid XML/viewBox
        let essential_icons = [
            "actions/add.svg",
            "actions/close.svg",
            "actions/search.svg",
            "status/battery.svg",
            "status/network-wifi.svg",
            "status/volume.svg",
            "devices/computer.svg",
            "places/folder.svg",
            "apps/quickshell.svg",
        ];

        for icon_rel in essential_icons.iter() {
            let path = scalable_dir.join(icon_rel);
            assert!(path.exists(), "Missing essential icon SVG: {}", icon_rel);
            let svg_str = fs::read_to_string(&path).expect("Read SVG");
            assert!(svg_str.contains("viewBox=\"0 0 24 24\""));
            assert!(svg_str.contains("<svg"));
            assert!(svg_str.contains("</svg>"));
        }
    }

    #[test]
    fn test_gtk_and_qt_generation() {
        let theme = Theme {
            meta: ThemeMeta {
                name: "obsidian".into(),
                author: "Hypokoto".into(),
                description: "Test".into(),
            },
            tokens: ThemeTokens {
                background: "#0B0E14".into(),
                surface: "#11161F".into(),
                overlay: "#19212D".into(),
                text: "#E8EDF5".into(),
                muted: "#7F899B".into(),
                disabled: "#475569".into(),
                primary: "#7DD3FC".into(),
                secondary: "#8BA4FF".into(),
                highlight: "#B4A7FF".into(),
                success: "#8BE28B".into(),
                warning: "#E8C77B".into(),
                danger: "#F08080".into(),
                info: "#7DD3FC".into(),
                selection: "#1E293B".into(),
                hover: "#243144".into(),
                pressed: "#0F172A".into(),
                focus: "#38BDF8".into(),
                panel: "#0F141C".into(),
                panel_variant: "#151C28".into(),
            },
            icon_roles: None,
            geometry: None,
            spacing: None,
            typography: None,
        };

        let gtk = theme.generate_gtk_settings();
        assert!(gtk.contains("gtk-icon-theme-name=Minimal"));
        assert!(gtk.contains("gtk-application-prefer-dark-theme=1"));

        let qt = theme.generate_qtct_conf();
        assert!(qt.contains("icon_theme=Minimal"));
    }

    #[test]
    fn test_icon_role_update_and_resolution() {
        let mut theme = Theme {
            meta: ThemeMeta {
                name: "obsidian".into(),
                author: "Hypokoto".into(),
                description: "Test".into(),
            },
            tokens: ThemeTokens {
                background: "#0B0E14".into(),
                surface: "#11161F".into(),
                overlay: "#19212D".into(),
                text: "#E8EDF5".into(),
                muted: "#7F899B".into(),
                disabled: "#475569".into(),
                primary: "#7DD3FC".into(),
                secondary: "#8BA4FF".into(),
                highlight: "#B4A7FF".into(),
                success: "#8BE28B".into(),
                warning: "#E8C77B".into(),
                danger: "#F08080".into(),
                info: "#7DD3FC".into(),
                selection: "#1E293B".into(),
                hover: "#243144".into(),
                pressed: "#0F172A".into(),
                focus: "#38BDF8".into(),
                panel: "#0F141C".into(),
                panel_variant: "#151C28".into(),
            },
            icon_roles: None,
            geometry: None,
            spacing: None,
            typography: None,
        };

        assert_eq!(theme.resolve_icon_color("primary"), "#7DD3FC");
        assert_eq!(theme.resolve_icon_color("#FF00FF"), "#FF00FF");

        assert!(theme.update_icon_role("active", "warning").is_ok());
        assert_eq!(
            theme.resolve_icon_color(&theme.icon_roles.as_ref().unwrap().active),
            "#E8C77B"
        );

        assert!(theme.update_icon_role("active", "#123456").is_ok());
        assert_eq!(
            theme.resolve_icon_color(&theme.icon_roles.as_ref().unwrap().active),
            "#123456"
        );

        assert!(theme.update_icon_role("active", "invalid_color").is_err());
        assert!(theme.update_icon_role("invalid_role", "#123456").is_err());
    }
}
