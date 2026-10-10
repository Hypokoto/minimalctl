use clap::Subcommand;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Subcommand)]
pub enum AiCommands {
    /// Manage Model Context Protocol (MCP) servers
    Mcp {
        #[command(subcommand)]
        action: EntityAction,
    },
    /// Manage AI Skills and Plugins
    Skill {
        #[command(subcommand)]
        action: EntityAction,
    },
    /// Launch an AI harness in the secure bubblewrap sandbox
    Run {
        /// The harness executable (e.g. `coderabbit`, `antigravity-cli`, `claude`)
        harness: Option<String>,
        /// Arguments passed to the harness
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum EntityAction {
    /// List installed items
    List,
    /// Connect/Install a new item from a path or repository
    Connect { source: String },
    /// Remove an installed item by name
    Remove { name: String },
    /// Search available items
    Search { query: String },
}

fn get_sandbox_dir() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    let dir = PathBuf::from(home).join(".local/share/ai-sandbox");
    fs::create_dir_all(dir.join("mcps")).unwrap();
    fs::create_dir_all(dir.join("skills")).unwrap();
    fs::create_dir_all(dir.join("plugins")).unwrap();
    dir
}

pub fn handle_ai(cmd: &AiCommands) {
    let sandbox_dir = get_sandbox_dir();

    match cmd {
        AiCommands::Mcp { action } => handle_entity(&sandbox_dir.join("mcps"), action, "MCP"),
        AiCommands::Skill { action } => handle_entity(&sandbox_dir.join("skills"), action, "Skill"),
        AiCommands::Run { harness, args } => run_harness(harness, args),
    }
}

fn handle_entity(dir: &Path, action: &EntityAction, kind: &str) {
    match action {
        EntityAction::List => {
            println!("Installed {}s in {}:", kind, dir.display());
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    println!("  - {}", entry.file_name().to_string_lossy());
                }
            }
        }
        EntityAction::Connect { source } => {
            // Simplified logic: symlink or clone the source into the registry
            println!("Connecting {} from '{}'...", kind, source);
            let name = match Path::new(source).file_name() {
                Some(n) => n,
                None => {
                    eprintln!("Invalid source path.");
                    return;
                }
            };
            let dest = dir.join(name);

            if Path::new(source).exists() {
                // local path, create symlink
                if let Err(e) = std::os::unix::fs::symlink(source, &dest) {
                    eprintln!("Failed to link {}: {}", source, e);
                } else {
                    println!("Linked {} to {}", source, dest.display());
                }
            } else if source.starts_with("http") || source.starts_with("git@") {
                // git clone
                let status = Command::new("git")
                    .arg("clone")
                    .arg(source)
                    .arg(&dest)
                    .status();
                if let Ok(s) = status {
                    if s.success() {
                        println!("Cloned {} to {}", source, dest.display());
                    } else {
                        eprintln!("Failed to clone {}", source);
                    }
                } else {
                    eprintln!("Failed to invoke git");
                }
            } else {
                eprintln!("Unknown source format (not a local path or git URL).");
            }
        }
        EntityAction::Remove { name } => {
            let target_path = Path::new(name);
            if target_path.is_absolute() || target_path.components().any(|c| !matches!(c, std::path::Component::Normal(_))) {
                eprintln!("Invalid name: path traversal blocked.");
                return;
            }

            let target = dir.join(name);
            if target.exists() {
                if let Err(e) = fs::remove_dir_all(&target).or_else(|_| fs::remove_file(&target)) {
                    eprintln!("Failed to remove {}: {}", name, e);
                } else {
                    println!("Removed {} '{}'", kind, name);
                }
            } else {
                eprintln!("{} '{}' not found.", kind, name);
            }
        }
        EntityAction::Search { query } => {
            println!("Searching global registry for {} matching '{}'...", kind, query);
            println!("(Search backend not yet implemented - this would query a central MCP/Skill registry)");
        }
    }
}

fn run_harness(harness_opt: &Option<String>, args: &[String]) {
    use std::io::{self, Write};
    use std::os::unix::process::CommandExt;

    let home = std::env::var("HOME").expect("HOME not set");
    let pwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from(&home));

    let harness = match harness_opt {
        Some(h) => h.clone(),
        None => {
            if pwd.join(".gemini").exists() {
                println!("Auto-detected .gemini/ directory. Using 'antigravity-cli'.");
                "antigravity-cli".to_string()
            } else if pwd.join("claude.json").exists() {
                println!("Auto-detected claude.json. Using 'claude'.");
                "claude".to_string()
            } else {
                println!("No harness specified and no auto-detection matched.");
                println!("Available harnesses:");
                println!("  1) antigravity-cli");
                println!("  2) claude");
                print!("Select a harness (1 or 2, or type custom name): ");
                io::stdout().flush().unwrap();

                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                let input = input.trim();

                match input {
                    "1" => "antigravity-cli".to_string(),
                    "2" => "claude".to_string(),
                    _ if !input.is_empty() => input.to_string(),
                    _ => {
                        eprintln!("Invalid selection. Aborting.");
                        std::process::exit(1);
                    }
                }
            }
        }
    };

    let mut ai_script = Command::new(format!("{}/minimal/scripts/ai", home));
    ai_script.arg(&harness).args(args);

    let err = ai_script.exec();
    eprintln!("Failed to exec scripts/ai: {}", err);
    std::process::exit(1);
}
