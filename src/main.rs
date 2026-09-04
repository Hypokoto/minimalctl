mod doctor;
mod status;
mod theme;

use clap::{Parser, Subcommand};
use doctor::DoctorReport;
use status::SystemStatus;
use std::fs;
use std::path::{Path, PathBuf};
use theme::Theme;

fn find_repo_root() -> PathBuf {
    // 1. Search upwards from current working directory
    if let Ok(mut curr) = std::env::current_dir() {
        loop {
            if curr.join("quickshell").exists() && curr.join("themes").exists() {
                return curr;
            }
            if !curr.pop() {
                break;
            }
        }
    }

    // 2. Search upwards from binary executable location (handles ~/.local/bin/minimalctl)
    if let Ok(exe_path) = std::env::var("HOME").map(|h| PathBuf::from(h).join(".local/bin/minimalctl")) {
        if let Ok(canonical_exe) = fs::canonicalize(&exe_path) {
            let mut curr = canonical_exe;
            while curr.pop() {
                if curr.join("quickshell").exists() && curr.join("themes").exists() {
                    return curr;
                }
            }
        }
    }

    // 3. Fallback to standard dotfiles home location (~/minimal)
    if let Ok(home) = std::env::var("HOME") {
        let minimal_dir = PathBuf::from(&home).join("minimal");
        if minimal_dir.join("themes").exists() {
            return minimal_dir;
        }
    }

    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn resolve_theme_path<P: AsRef<Path>>(input: P, root: &Path) -> PathBuf {
    let input_path = input.as_ref();

    if input_path.exists() {
        return input_path.to_path_buf();
    }

    let rel_root = root.join(input_path);
    if rel_root.exists() {
        return rel_root;
    }

    let file_name = input_path.file_name().unwrap_or(input_path.as_os_str());
    let in_themes = root.join("themes").join(file_name);
    if in_themes.exists() {
        return in_themes;
    }

    let clean_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let with_toml = root.join("themes").join(format!("{}.toml", clean_stem));
    if with_toml.exists() {
        return with_toml;
    }

    input_path.to_path_buf()
}

#[derive(Parser)]
#[command(name = "minimalctl")]
#[command(about = "Native control plane and theme compiler for Minimal OS Shell", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print authoritative desktop status and socket inventory
    Status,
    /// Run single-pass operational diagnostic suite across desktop invariants
    Doctor,
    /// Theme management, target configuration compilation, and drift verification
    Theme {
        #[command(subcommand)]
        action: ThemeActions,
    },
    /// Icon theme color inspection and dynamic role modification
    Icon {
        #[command(subcommand)]
        action: IconActions,
    },
    /// Repository configuration verification suite
    Config {
        #[command(subcommand)]
        action: ConfigActions,
    },
}

#[derive(Subcommand)]
enum ConfigActions {
    /// Verify repository symlinks and configuration file structure
    Verify,
}

#[derive(Subcommand, Clone)]
enum IconActions {
    /// List icon roles, token mappings, and resolved hex colors
    List,
    /// Get resolved color for a specific icon role (e.g. minimalctl icon get active)
    Get {
        role: String,
    },
    /// Set color or token for an icon role and apply live (e.g. minimalctl icon set active #7DD3FC)
    Set {
        role: String,
        color: String,
    },
    /// Reset icon roles to theme default tokens
    Reset,
}

#[derive(Subcommand)]
enum ThemeActions {
    /// Compile target configurations from specified theme TOML file
    Build {
        #[arg(default_value = "themes/obsidian.toml")]
        path: String,
    },
    /// List available themes in themes/
    List,
    /// Display currently active theme details and content hash
    Current,
    /// Verify theme definition validity and check target configs for theme drift
    Verify,
    /// Compare color tokens between active theme and target theme
    Diff {
        #[arg(default_value = "themes/obsidian.toml")]
        path_a: String,
        #[arg(default_value = "themes/obsidian.toml")]
        path_b: String,
    },
    /// Roll back target configurations to the previous state before the last theme transaction
    Rollback,
    /// Compile target configurations and perform target-aware hot reloads
    Apply {
        #[arg(default_value = "themes/obsidian.toml")]
        path: String,
    },
    /// Run single-pass diagnostic checklist across theme and icon system pipelines
    Doctor,
    /// Manage icon role colors (alias for minimalctl icon)
    Icon {
        #[command(subcommand)]
        action: IconActions,
    },
}

fn main() {
    let cli = Cli::parse();
    let root = find_repo_root();

    match cli.command {
        Commands::Status => {
            let status = SystemStatus::collect();
            status.print_report();
        }
        Commands::Doctor => {
            let report = DoctorReport::run();
            if report.fail_count > 0 {
                std::process::exit(1);
            }
        }
        Commands::Theme { action } => match action {
            ThemeActions::Build { path } => {
                let resolved = resolve_theme_path(&path, &root);
                println!(
                    "[minimalctl] Loading theme definition from: {}",
                    resolved.display()
                );
                match Theme::load_from_file(&resolved) {
                    Ok(theme) => {
                        println!(
                            "[minimalctl] Theme '{}' validated successfully.",
                            theme.meta.name
                        );
                        match theme.build_all(&root) {
                            Ok(_) => {
                                println!("[minimalctl] All target configurations compiled cleanly.")
                            }
                            Err(e) => eprintln!("[minimalctl] Build error: {}", e),
                        }
                    }
                    Err(e) => eprintln!("[minimalctl] Theme validation error: {}", e),
                }
            }
            ThemeActions::Current => {
                let active_theme_path = root.join("themes/obsidian.toml");
                match Theme::load_from_file(&active_theme_path) {
                    Ok(theme) => {
                        println!("=== ACTIVE THEME CONFIGURATION ===");
                        println!(" - Name:        {}", theme.meta.name);
                        println!(" - Author:      {}", theme.meta.author);
                        println!(" - Description: {}", theme.meta.description);
                        println!(" - Source:      {}", active_theme_path.display());
                        println!(" - Content Hash: {}", theme.compute_hash());
                        println!(" - Primary:     {}", theme.tokens.primary);
                        println!(" - Surface:     {}", theme.tokens.surface);
                        println!(" - Background:  {}", theme.tokens.background);
                    }
                    Err(e) => {
                        eprintln!("[!] Failed to load active theme: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            ThemeActions::Diff { path_a, path_b } => {
                let resolved_a = resolve_theme_path(&path_a, &root);
                let resolved_b = resolve_theme_path(&path_b, &root);
                match (
                    Theme::load_from_file(&resolved_a),
                    Theme::load_from_file(&resolved_b),
                ) {
                    (Ok(t_a), Ok(t_b)) => {
                        println!(
                            "=== THEME TOKEN DIFF: {} vs {} ===",
                            t_a.meta.name, t_b.meta.name
                        );
                        let pairs = [
                            ("background", &t_a.tokens.background, &t_b.tokens.background),
                            ("surface", &t_a.tokens.surface, &t_b.tokens.surface),
                            ("overlay", &t_a.tokens.overlay, &t_b.tokens.overlay),
                            ("text", &t_a.tokens.text, &t_b.tokens.text),
                            ("muted", &t_a.tokens.muted, &t_b.tokens.muted),
                            ("primary", &t_a.tokens.primary, &t_b.tokens.primary),
                            ("secondary", &t_a.tokens.secondary, &t_b.tokens.secondary),
                            ("highlight", &t_a.tokens.highlight, &t_b.tokens.highlight),
                            ("success", &t_a.tokens.success, &t_b.tokens.success),
                            ("warning", &t_a.tokens.warning, &t_b.tokens.warning),
                            ("danger", &t_a.tokens.danger, &t_b.tokens.danger),
                            ("info", &t_a.tokens.info, &t_b.tokens.info),
                        ];
                        for (token, val_a, val_b) in pairs.iter() {
                            let status = if val_a == val_b {
                                "MATCH"
                            } else {
                                "DIFFERENT"
                            };
                            println!(" - {:<12}: {:<10} -> {:<10} [{}]", token, val_a, val_b, status);
                        }
                    }
                    (Err(e), _) | (_, Err(e)) => {
                        eprintln!("[!] Diff error loading theme: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            ThemeActions::Rollback => {
                println!("[minimalctl] Restoring target configuration rollback backup...");
                if let Err(e) = Theme::perform_rollback(&root) {
                    eprintln!("[!] Rollback failed: {}", e);
                    std::process::exit(1);
                }
            }
            ThemeActions::Apply { path } => {
                let resolved = resolve_theme_path(&path, &root);
                println!(
                    "[minimalctl] Applying theme transaction from: {}",
                    resolved.display()
                );
                match Theme::load_from_file(&resolved) {
                    Ok(theme) => {
                        let _ = theme.backup_state(&root);
                        if let Err(e) = theme.apply_runtime(&root) {
                            eprintln!("[!] Transaction aborted: {}", e);
                            std::process::exit(1);
                        }
                    }
                    Err(e) => {
                        eprintln!("[!] Theme validation error (Transaction aborted): {}", e);
                        std::process::exit(1);
                    }
                }
            }
            ThemeActions::Doctor => {
                if let Err(_) = Theme::run_doctor(&root) {
                    std::process::exit(1);
                }
            }
            ThemeActions::List => {
                println!("=== AVAILABLE THEMES ===");
                let themes_dir = root.join("themes");
                if let Ok(entries) = fs::read_dir(&themes_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                            if let Ok(theme) = Theme::load_from_file(&path) {
                                println!(
                                    " - {:<12} ({}): {}",
                                    theme.meta.name,
                                    path.file_name().unwrap().to_string_lossy(),
                                    theme.meta.description
                                );
                            }
                        }
                    }
                }
            }
            ThemeActions::Verify => {
                println!("[minimalctl] Verifying theme definition and target drift...");
                let mut drift = false;
                let active_theme_path = root.join("themes/obsidian.toml");
                match Theme::load_from_file(&active_theme_path) {
                    Ok(theme) => {
                        println!(" - themes/obsidian.toml: Valid TOML, all hex tokens verified.");

                        if let Ok(content) = fs::read_to_string(root.join("kitty/kitty.conf")) {
                            if content != theme.generate_kitty_conf() {
                                eprintln!("[!] DRIFT: kitty/kitty.conf differs from compiled obsidian.toml output!");
                                drift = true;
                            } else {
                                println!(" - kitty/kitty.conf: In sync with source.");
                            }
                        }

                        if let Ok(content) = fs::read_to_string(root.join("starship/starship.toml"))
                        {
                            if content != theme.generate_starship_toml() {
                                eprintln!("[!] DRIFT: starship/starship.toml differs from compiled obsidian.toml output!");
                                drift = true;
                            } else {
                                println!(" - starship/starship.toml: In sync with source.");
                            }
                        }

                        if let Ok(content) = fs::read_to_string(root.join("btop/btop.theme")) {
                            if content != theme.generate_btop_theme() {
                                eprintln!("[!] DRIFT: btop/btop.theme differs from compiled obsidian.toml output!");
                                drift = true;
                            } else {
                                println!(" - btop/btop.theme: In sync with source.");
                            }
                        }

                        if let Ok(content) = fs::read_to_string(root.join("hypr/colors.conf")) {
                            if content != theme.generate_hypr_colors() {
                                eprintln!("[!] DRIFT: hypr/colors.conf differs from compiled obsidian.toml output!");
                                drift = true;
                            } else {
                                println!(" - hypr/colors.conf: In sync with source.");
                            }
                        }

                        if let Ok(content) = fs::read_to_string(root.join("tmux/tmux.conf")) {
                            if content != theme.generate_tmux_conf() {
                                eprintln!("[!] DRIFT: tmux/tmux.conf differs from compiled obsidian.toml output!");
                                drift = true;
                            } else {
                                println!(" - tmux/tmux.conf: In sync with source.");
                            }
                        }

                        let nvim_theme_file = root.join("nvim/lua/themes/minimal.lua");
                        if nvim_theme_file.exists() {
                            if let Ok(content) = fs::read_to_string(&nvim_theme_file) {
                                if content != theme.generate_nvim_theme() {
                                    eprintln!("[!] DRIFT: nvim/lua/themes/minimal.lua differs from compiled obsidian.toml output!");
                                    drift = true;
                                } else {
                                    println!(
                                        " - nvim/lua/themes/minimal.lua: In sync with source."
                                    );
                                }
                            }
                        }

                        let quickshell_theme_file = std::env::var("HOME")
                            .map(|h| std::path::PathBuf::from(h).join(".config/quickshell/theme.json"))
                            .unwrap_or_else(|_| root.join("quickshell/theme.json"));
                        if quickshell_theme_file.exists() {
                            if let Ok(content) = fs::read_to_string(&quickshell_theme_file) {
                                if content != theme.generate_quickshell_theme() {
                                    eprintln!("[!] DRIFT: ~/.config/quickshell/theme.json differs from compiled obsidian.toml output!");
                                    drift = true;
                                } else {
                                    println!(" - ~/.config/quickshell/theme.json: In sync with source.");
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[!] ERROR: Failed to parse themes/obsidian.toml: {}", e);
                        drift = true;
                    }
                }

                if drift {
                    eprintln!("[!] Theme verification FAILED: Drift or syntax error detected.");
                    std::process::exit(1);
                } else {
                    println!(
                        "[minimalctl] Theme verification PASSED: Zero drift across all targets."
                    );
                }
            }
            ThemeActions::Icon { action } => {
                handle_icon_action(action, &root);
            }
        },
        Commands::Icon { action } => {
            handle_icon_action(action, &root);
        }
        Commands::Config { action } => match action {
            ConfigActions::Verify => {
                println!("=== REPOSITORY CONFIGURATION VERIFICATION ===");
                let required_dirs = [
                    "hypr", "quickshell", "kitty", "tmux", "zsh", "starship", "btop", "themes", "icons",
                ];
                let mut valid = true;

                for dir in required_dirs.iter() {
                    let path = root.join(dir);
                    if path.exists() && path.is_dir() {
                        println!(" [PASS] Directory structure intact: {}", dir);
                    } else {
                        eprintln!(" [FAIL] Missing core directory: {}", dir);
                        valid = false;
                    }
                }

                let keybinds_file = root.join("hypr/keybinds.lua");
                if keybinds_file.exists() {
                    if let Ok(content) = fs::read_to_string(&keybinds_file) {
                        if content.contains("hl.bind") {
                            println!(" [PASS] Hyprland keybindings configuration valid");
                        } else {
                            eprintln!(" [WARN] Hyprland keybindings file missing hl.bind calls");
                        }
                    }
                } else {
                    eprintln!(" [FAIL] Missing hypr/keybinds.lua");
                    valid = false;
                }

                if valid {
                    println!("[minimalctl] Repository configuration verification PASSED.");
                } else {
                    eprintln!("[minimalctl] Repository configuration verification FAILED.");
                    std::process::exit(1);
                }
            }
        },
    }
}

fn handle_icon_action(action: IconActions, root: &Path) {
    let active_theme_path = root.join("themes/obsidian.toml");
    let mut theme = match Theme::load_from_file(&active_theme_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("[!] Failed to load active theme: {}", e);
            std::process::exit(1);
        }
    };

    match action {
        IconActions::List => {
            println!("=== MINIMAL ICON COLOR ROLES ===");
            let roles = theme.icon_roles.clone().unwrap_or_else(|| theme::IconRoles {
                default: "text".into(),
                active: "primary".into(),
                muted: "muted".into(),
                disabled: "disabled".into(),
                success: "success".into(),
                warning: "warning".into(),
                error: "danger".into(),
                info: "info".into(),
            });

            let items = [
                ("default", &roles.default),
                ("active", &roles.active),
                ("muted", &roles.muted),
                ("disabled", &roles.disabled),
                ("success", &roles.success),
                ("warning", &roles.warning),
                ("error", &roles.error),
                ("info", &roles.info),
            ];

            for (name, val) in items.iter() {
                let resolved = theme.resolve_icon_color(val);
                println!(" - {:<10} : {:<12} ({})", name, val, resolved);
            }
        }
        IconActions::Get { role } => {
            let roles = theme.icon_roles.clone().unwrap_or_else(|| theme::IconRoles {
                default: "text".into(),
                active: "primary".into(),
                muted: "muted".into(),
                disabled: "disabled".into(),
                success: "success".into(),
                warning: "warning".into(),
                error: "danger".into(),
                info: "info".into(),
            });

            let val = match role.to_lowercase().as_str() {
                "default" => &roles.default,
                "active" => &roles.active,
                "muted" => &roles.muted,
                "disabled" => &roles.disabled,
                "success" => &roles.success,
                "warning" => &roles.warning,
                "error" | "danger" => &roles.error,
                "info" => &roles.info,
                _ => {
                    eprintln!(
                        "[!] Unknown icon role '{}'. Valid roles: default, active, muted, disabled, success, warning, error, info",
                        role
                    );
                    std::process::exit(1);
                }
            };
            let resolved = theme.resolve_icon_color(val);
            println!("{}", resolved);
        }
        IconActions::Set { role, color } => {
            if let Err(e) = theme.update_icon_role(&role, &color) {
                eprintln!("[!] Error updating icon role: {}", e);
                std::process::exit(1);
            }

            if let Err(e) = theme.save_to_file(&active_theme_path) {
                eprintln!("[!] Error saving theme file: {}", e);
                std::process::exit(1);
            }

            println!(
                "[minimalctl] Updated icon role '{}' to '{}' in {}",
                role,
                color,
                active_theme_path.display()
            );

            let _ = theme.backup_state(root);
            if let Err(e) = theme.apply_runtime(root) {
                eprintln!("[!] Failed to apply runtime theme transaction: {}", e);
                std::process::exit(1);
            }
        }
        IconActions::Reset => {
            theme.icon_roles = Some(theme::IconRoles {
                default: "text".into(),
                active: "primary".into(),
                muted: "muted".into(),
                disabled: "disabled".into(),
                success: "success".into(),
                warning: "warning".into(),
                error: "danger".into(),
                info: "info".into(),
            });

            if let Err(e) = theme.save_to_file(&active_theme_path) {
                eprintln!("[!] Error saving theme file: {}", e);
                std::process::exit(1);
            }

            println!(
                "[minimalctl] Reset icon roles to theme defaults in {}",
                active_theme_path.display()
            );

            let _ = theme.backup_state(root);
            if let Err(e) = theme.apply_runtime(root) {
                eprintln!("[!] Failed to apply runtime theme transaction: {}", e);
                std::process::exit(1);
            }
        }
    }
}
