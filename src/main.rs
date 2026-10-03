mod audit;
mod doctor;
mod ide;
mod status;

use audit::AuditReport;
use clap::{Parser, Subcommand};
use doctor::DoctorReport;
use ide::{IdeConfig, IdeServer, FilesystemWatcher};
use status::SystemStatus;
use std::fs;
use std::path::PathBuf;

fn is_repo_root(path: &std::path::Path) -> bool {
    path.join("labwc").is_dir()
        && path.join("packages").is_dir()
        && path.join("Cargo.toml").is_file()
}

fn find_repo_root() -> PathBuf {
    // 1. Search upwards from current working directory
    if let Ok(mut curr) = std::env::current_dir() {
        loop {
            if is_repo_root(&curr) {
                return curr;
            }
            if !curr.pop() {
                break;
            }
        }
    }

    // 2. Search upwards from binary executable location (handles ~/.local/bin/minimalctl)
    if let Ok(exe_path) =
        std::env::var("HOME").map(|h| PathBuf::from(h).join(".local/bin/minimalctl"))
    {
        if let Ok(canonical_exe) = fs::canonicalize(&exe_path) {
            let mut curr = canonical_exe;
            while curr.pop() {
                if is_repo_root(&curr) {
                    return curr;
                }
            }
        }
    }

    // 3. Fallback to standard dotfiles home location (~/minimal)
    if let Ok(home) = std::env::var("HOME") {
        let minimal_dir = PathBuf::from(&home).join("minimal");
        if is_repo_root(&minimal_dir) {
            return minimal_dir;
        }
    }

    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

#[derive(Parser)]
#[command(name = "minimalctl")]
#[command(about = "Native diagnostic and audit control plane for Minimal OS", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run single-pass operational diagnostic suite across desktop invariants
    Doctor,
    /// Run security, syntax, and permission audit suite
    Audit,
    /// Print authoritative desktop status and socket inventory
    Status,
    /// Launch the Neovim IDE server (JSON-RPC over stdio or Unix socket)
    Ide {
        /// Workspace root (defaults to repo root discovery)
        #[arg(long)]
        root: Option<PathBuf>,
        /// Run in stdio JSON-RPC mode (used by the Neovim plugin)
        #[arg(long)]
        stdio: bool,
        /// Unix socket path (alternative to --stdio)
        #[arg(long)]
        socket: Option<PathBuf>,
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
            let report = DoctorReport::run(&root);
            if report.fail_count > 0 {
                std::process::exit(1);
            }
        }
        Commands::Audit => {
            let report = AuditReport::run(&root);
            if report.fail_count > 0 {
                std::process::exit(1);
            }
        }
        Commands::Ide { root: ide_root, stdio, socket } => {
            let workspace = ide_root.unwrap_or(root);
            let config = IdeConfig::default();
            let mut server = IdeServer::new(workspace.clone(), config);

            // Spawn filesystem watcher and feed events into server
            let (watcher_tx, watcher_rx) = std::sync::mpsc::channel();
            let mut watcher = FilesystemWatcher::new(workspace, watcher_tx);
            if let Err(e) = watcher.start() {
                eprintln!("[minimal-ide] watcher start failed: {e}");
            }

            // Spawn background thread to recompute diffs on file change
            let _watcher_thread = std::thread::spawn(move || {
                for _event in watcher_rx {
                    // Events forwarded; server.recompute_diffs() called from RPC thread
                }
            });

            if stdio || socket.is_none() {
                if let Err(e) = server.run_stdio() {
                    eprintln!("[minimal-ide] stdio error: {e}");
                    std::process::exit(1);
                }
            } else if let Some(sock_path) = socket {
                if let Err(e) = server.run_socket(&sock_path) {
                    eprintln!("[minimal-ide] socket error: {e}");
                    std::process::exit(1);
                }
            }
        }
    }
}
