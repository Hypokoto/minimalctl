mod doctor;
mod status;
mod theme;

use clap::{Parser, Subcommand};
use doctor::DoctorReport;
use status::SystemStatus;
use std::fs;
use theme::Theme;

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
    /// Verify theme definition validity and check target configs for theme drift
    Verify,
    /// Compile target configurations and perform target-aware hot reloads
    Apply {
        #[arg(default_value = "themes/obsidian.toml")]
        path: String,
    },
}

fn main() {
    let cli = Cli::parse();

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
                println!("[minimalctl] Loading theme definition from: {}", path);
                match Theme::load_from_file(&path) {
                    Ok(theme) => {
                        println!("[minimalctl] Theme '{}' validated successfully.", theme.meta.name);
                        match theme.build_all(".") {
                            Ok(_) => println!("[minimalctl] All target configurations compiled cleanly."),
                            Err(e) => eprintln!("[minimalctl] Build error: {}", e),
                        }
                    }
                    Err(e) => eprintln!("[minimalctl] Theme validation error: {}", e),
                }
            }
            ThemeActions::Apply { path } => {
                println!("[minimalctl] Applying theme transaction from: {}", path);
                match Theme::load_from_file(&path) {
                    Ok(theme) => {
                        if let Err(e) = theme.apply_runtime(".") {
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
            ThemeActions::List => {
                println!("=== AVAILABLE THEMES ===");
                if let Ok(entries) = fs::read_dir("themes") {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                            if let Ok(theme) = Theme::load_from_file(&path) {
                                println!(" - {} ({}): {}", theme.meta.name, path.display(), theme.meta.description);
                            }
                        }
                    }
                }
            }
            ThemeActions::Verify => {
                println!("[minimalctl] Verifying theme definition and target drift...");
                let mut drift = false;
                match Theme::load_from_file("themes/obsidian.toml") {
                    Ok(theme) => {
                        println!(" - themes/obsidian.toml: Valid TOML, all hex tokens verified.");

                        if let Ok(content) = fs::read_to_string("mako/config") {
                            if content.contains("padding=14 16") {
                                eprintln!("[!] ERROR: mako/config contains invalid space-separated padding!");
                                drift = true;
                            } else if content != theme.generate_mako_config() {
                                eprintln!("[!] DRIFT: mako/config differs from compiled obsidian.toml output!");
                                drift = true;
                            } else {
                                println!(" - mako/config: In sync with source.");
                            }
                        }

                        if let Ok(content) = fs::read_to_string("rofi/theme.rasi") {
                            if content != theme.generate_rofi_theme() {
                                eprintln!("[!] DRIFT: rofi/theme.rasi differs from compiled obsidian.toml output!");
                                drift = true;
                            } else {
                                println!(" - rofi/theme.rasi: In sync with source.");
                            }
                        }

                        if let Ok(content) = fs::read_to_string("kitty/kitty.conf") {
                            if content != theme.generate_kitty_conf() {
                                eprintln!("[!] DRIFT: kitty/kitty.conf differs from compiled obsidian.toml output!");
                                drift = true;
                            } else {
                                println!(" - kitty/kitty.conf: In sync with source.");
                            }
                        }

                        if let Ok(content) = fs::read_to_string("waybar/style.css") {
                            if content != theme.generate_waybar_style() {
                                eprintln!("[!] DRIFT: waybar/style.css differs from compiled obsidian.toml output!");
                                drift = true;
                            } else {
                                println!(" - waybar/style.css: In sync with source.");
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
                    println!("[minimalctl] Theme verification PASSED: Zero drift across all targets.");
                }
            }
        },
    }
}
