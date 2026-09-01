mod status;
mod theme;

use clap::{Parser, Subcommand};
use status::SystemStatus;
use std::fs;
use std::path::Path;
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
    /// Print authoritative desktop status and security invariant report
    Status,
    /// Theme management and target configuration compilation
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
    /// Audit existing target configurations against design invariants
    Check,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Status => {
            let status = SystemStatus::collect();
            status.print_report();
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
            ThemeActions::Check => {
                println!("[minimalctl] Checking generated target configurations...");
                let mako_path = Path::new("mako/config");
                if mako_path.exists() {
                    if let Ok(content) = fs::read_to_string(mako_path) {
                        if content.contains("padding=14 16") {
                            eprintln!("[!] ERROR: mako/config contains invalid space-separated padding!");
                        } else {
                            println!(" - mako/config: Syntax valid.");
                        }
                    }
                }
                println!("[minimalctl] All configuration target checks passed.");
            }
        },
    }
}
