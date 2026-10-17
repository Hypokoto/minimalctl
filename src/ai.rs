use clap::Subcommand;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub mod wasm;

#[derive(Subcommand)]
pub enum AiCommands {
    /// Manage Model Context Protocol (MCP) servers in mcp.json
    Mcp {
        #[command(subcommand)]
        action: McpAction,
    },
    /// Manage AI Plugins in plugin.json
    Plugin {
        #[command(subcommand)]
        action: PluginAction,
    },
    /// Manage AI Skills
    Skill {
        #[command(subcommand)]
        action: EntityAction,
    },
    /// Initialize AI sandbox configuration (.mcp.json, .ai/settings.json, .ai/plugin.json) in current directory
    Init {
        /// Overwrite existing configuration files
        #[arg(short, long)]
        force: bool,
    },
    /// Launch an AI harness in the secure bubblewrap sandbox
    Run {
        /// The harness executable (e.g. `agy`, `antigravity-cli`, `claude`)
        harness: Option<String>,
        /// Arguments passed to the harness
        args: Vec<String>,
    },
    /// Manage and execute isolated WebAssembly tools
    Wasm {
        #[command(subcommand)]
        action: wasm::WasmAction,
    },
}

#[derive(Subcommand)]
pub enum McpAction {
    /// List configured MCP servers from mcp.json
    List,
    /// Connect or register a new MCP server in mcp.json
    Connect {
        /// Identifier/name for the server
        name: String,
        /// Command to execute (e.g. npx, node, uvx, python)
        command: Option<String>,
        /// Arguments for the command
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Remove an MCP server from mcp.json by name
    Remove {
        /// Name of the MCP server to remove
        name: String,
    },
}

#[derive(Subcommand)]
pub enum PluginAction {
    /// List configured plugins from plugin.json and available installed plugins
    List,
    /// Add or enable a plugin in plugin.json
    Add {
        /// Name of installed plugin (e.g. science) or path to local plugin directory
        name_or_path: String,
    },
    /// Remove or disable a plugin from plugin.json
    Remove {
        /// Name of plugin to remove
        name: String,
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
    let _ = fs::create_dir_all(dir.join("mcps"));
    let _ = fs::create_dir_all(dir.join("skills"));
    let _ = fs::create_dir_all(dir.join("plugins"));
    dir
}

pub fn handle_ai(cmd: &AiCommands) {
    let sandbox_dir = get_sandbox_dir();

    match cmd {
        AiCommands::Mcp { action } => handle_mcp(action),
        AiCommands::Plugin { action } => handle_plugin(action),
        AiCommands::Skill { action } => handle_entity(&sandbox_dir.join("skills"), action, "Skill"),
        AiCommands::Init { force } => handle_init(*force),
        AiCommands::Run { harness, args } => run_harness(harness, args),
        AiCommands::Wasm { action } => wasm::handle_wasm(action),
    }
}

fn find_mcp_config() -> (PathBuf, bool) {
    let pwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let global_config = PathBuf::from(&home).join(".config/ai-sandbox/mcp.json");

    let candidates = [
        pwd.join(".mcp.json"),
        pwd.join("mcp.json"),
        global_config,
    ];

    for path in &candidates {
        if path.exists() {
            return (path.clone(), true);
        }
    }

    (pwd.join(".mcp.json"), false)
}

fn handle_mcp(action: &McpAction) {
    let (config_path, exists) = find_mcp_config();

    match action {
        McpAction::List => {
            if !exists {
                println!("No active mcp.json found.");
                println!("Run 'minimalctl ai init' to scaffold a project configuration,");
                println!("or add a server with 'minimalctl ai mcp connect <name> [cmd] [args...]'.");
                return;
            }

            println!("Active MCP Config: {}", config_path.display());
            match fs::read_to_string(&config_path) {
                Ok(content) => match serde_json::from_str::<Value>(&content) {
                    Ok(val) => {
                        if let Some(servers) = val.get("mcpServers").and_then(|s| s.as_object()) {
                            if servers.is_empty() {
                                println!("  (No servers defined in mcpServers)");
                            } else {
                                for (name, def) in servers {
                                    let cmd = def.get("command").and_then(|c| c.as_str()).unwrap_or("<unknown>");
                                    let args_str = def.get("args")
                                        .and_then(|a| a.as_array())
                                        .map(|arr| {
                                            arr.iter()
                                                .filter_map(|x| x.as_str())
                                                .collect::<Vec<_>>()
                                                .join(" ")
                                        })
                                        .unwrap_or_default();
                                    println!("  - [{}] {} {}", name, cmd, args_str);
                                }
                            }
                        } else {
                            println!("  No valid 'mcpServers' object found in config.");
                        }
                    }
                    Err(e) => eprintln!("Failed to parse JSON in {}: {}", config_path.display(), e),
                },
                Err(e) => eprintln!("Failed to read {}: {}", config_path.display(), e),
            }
        }
        McpAction::Connect { name, command, args } => {
            let mut val: Value = if exists {
                fs::read_to_string(&config_path)
                    .ok()
                    .and_then(|c| serde_json::from_str(&c).ok())
                    .unwrap_or_else(|| json!({ "mcpServers": {} }))
            } else {
                json!({ "mcpServers": {} })
            };

            let (cmd_str, final_args) = match command {
                Some(c) => (c.clone(), args.clone()),
                None => (
                    "npx".to_string(),
                    vec!["-y".to_string(), format!("@modelcontextprotocol/server-{}", name)],
                ),
            };

            let server_obj = json!({
                "command": cmd_str,
                "args": final_args
            });

            if let Some(obj) = val.as_object_mut() {
                let servers = obj.entry("mcpServers").or_insert_with(|| json!({}));
                if let Some(s_map) = servers.as_object_mut() {
                    s_map.insert(name.clone(), server_obj);
                }
            }

            if let Ok(pretty) = serde_json::to_string_pretty(&val) {
                if let Err(e) = fs::write(&config_path, format!("{}\n", pretty)) {
                    eprintln!("Failed to write {}: {}", config_path.display(), e);
                } else {
                    println!("Registered MCP server '{}' in {}", name, config_path.display());
                }
            }
        }
        McpAction::Remove { name } => {
            if !exists {
                eprintln!("No active mcp.json found.");
                return;
            }

            let mut val: Value = match fs::read_to_string(&config_path)
                .ok()
                .and_then(|c| serde_json::from_str(&c).ok())
            {
                Some(v) => v,
                None => {
                    eprintln!("Failed to parse {}.", config_path.display());
                    return;
                }
            };

            let mut removed = false;
            if let Some(obj) = val.as_object_mut() {
                if let Some(servers) = obj.get_mut("mcpServers").and_then(|s| s.as_object_mut()) {
                    if servers.remove(name).is_some() {
                        removed = true;
                    }
                }
            }

            if removed {
                if let Ok(pretty) = serde_json::to_string_pretty(&val) {
                    let _ = fs::write(&config_path, format!("{}\n", pretty));
                    println!("Removed MCP server '{}' from {}", name, config_path.display());
                }
            } else {
                eprintln!("Server '{}' not found in {}", name, config_path.display());
            }
        }
    }
}

fn find_plugin_config() -> (PathBuf, bool) {
    let pwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let global_config = PathBuf::from(&home).join(".config/ai-sandbox/plugin.json");

    let candidates = [
        pwd.join(".ai/plugin.json"),
        pwd.join("plugin.json"),
        global_config,
    ];

    for path in &candidates {
        if path.exists() {
            return (path.clone(), true);
        }
    }

    if pwd.join(".ai").exists() {
        (pwd.join(".ai/plugin.json"), false)
    } else {
        (pwd.join("plugin.json"), false)
    }
}

fn handle_plugin(action: &PluginAction) {
    let (config_path, exists) = find_plugin_config();
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let host_plugins_dir = PathBuf::from(&home).join(".gemini/config/plugins");

    match action {
        PluginAction::List => {
            println!("Active Plugin Config: {}", if exists { config_path.display().to_string() } else { "None (using global default)".to_string() });

            if exists {
                if let Ok(content) = fs::read_to_string(&config_path) {
                    if let Ok(val) = serde_json::from_str::<Value>(&content) {
                        if let Some(plugins) = val.get("plugins").and_then(|p| p.as_array()) {
                            if plugins.is_empty() {
                                println!("  (No plugins active in project config)");
                            } else {
                                println!("Active Plugins:");
                                for p in plugins {
                                    if let Some(name) = p.as_str() {
                                        println!("  - [x] {}", name);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if host_plugins_dir.exists() {
                println!("\nInstalled Host Plugins (~/.gemini/config/plugins):");
                if let Ok(entries) = fs::read_dir(&host_plugins_dir) {
                    for entry in entries.flatten() {
                        if entry.path().is_dir() {
                            println!("  * {}", entry.file_name().to_string_lossy());
                        }
                    }
                }
            }
        }
        PluginAction::Add { name_or_path } => {
            let mut val: Value = if exists {
                fs::read_to_string(&config_path)
                    .ok()
                    .and_then(|c| serde_json::from_str(&c).ok())
                    .unwrap_or_else(|| json!({ "plugins": [] }))
            } else {
                json!({ "plugins": [] })
            };

            if let Some(obj) = val.as_object_mut() {
                let plugins = obj.entry("plugins").or_insert_with(|| json!([]));
                if let Some(arr) = plugins.as_array_mut() {
                    let already_present = arr.iter().any(|v| v.as_str() == Some(name_or_path.as_str()));
                    if !already_present {
                        arr.push(json!(name_or_path));
                    }
                }
            }

            if let Some(parent) = config_path.parent() {
                let _ = fs::create_dir_all(parent);
            }

            if let Ok(pretty) = serde_json::to_string_pretty(&val) {
                if let Err(e) = fs::write(&config_path, format!("{}\n", pretty)) {
                    eprintln!("Failed to write {}: {}", config_path.display(), e);
                } else {
                    println!("Added plugin '{}' to {}", name_or_path, config_path.display());
                }
            }
        }
        PluginAction::Remove { name } => {
            if !exists {
                eprintln!("No active plugin.json found.");
                return;
            }

            let mut val: Value = match fs::read_to_string(&config_path)
                .ok()
                .and_then(|c| serde_json::from_str(&c).ok())
            {
                Some(v) => v,
                None => {
                    eprintln!("Failed to parse {}.", config_path.display());
                    return;
                }
            };

            let mut removed = false;
            if let Some(obj) = val.as_object_mut() {
                if let Some(arr) = obj.get_mut("plugins").and_then(|p| p.as_array_mut()) {
                    let prev_len = arr.len();
                    arr.retain(|x| x.as_str() != Some(name.as_str()));
                    removed = arr.len() < prev_len;
                }
            }

            if removed {
                if let Ok(pretty) = serde_json::to_string_pretty(&val) {
                    let _ = fs::write(&config_path, format!("{}\n", pretty));
                    println!("Removed plugin '{}' from {}", name, config_path.display());
                }
            } else {
                eprintln!("Plugin '{}' not found in {}", name, config_path.display());
            }
        }
    }
}

fn handle_init(force: bool) {
    let pwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mcp_path = pwd.join(".mcp.json");
    let ai_dir = pwd.join(".ai");
    let settings_path = ai_dir.join("settings.json");
    let plugin_path = ai_dir.join("plugin.json");

    if !force && mcp_path.exists() && settings_path.exists() && plugin_path.exists() {
        println!("AI sandbox configuration already exists in {}.", pwd.display());
        println!("Use '--force' to overwrite existing configuration.");
        return;
    }

    let _ = fs::create_dir_all(&ai_dir);

    // 1. .mcp.json
    if force || !mcp_path.exists() {
        let default_mcp = json!({
            "mcpServers": {
                "filesystem": {
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-filesystem", "."]
                }
            }
        });
        if let Ok(pretty) = serde_json::to_string_pretty(&default_mcp) {
            if let Err(e) = fs::write(&mcp_path, format!("{}\n", pretty)) {
                eprintln!("Failed to write {}: {}", mcp_path.display(), e);
            } else {
                println!("[OK] Created {}", mcp_path.display());
            }
        }
    }

    // 2. .ai/settings.json
    if force || !settings_path.exists() {
        let default_settings = json!({
            "trustedWorkspaces": ["."],
            "defaultHarness": "agy",
            "sandbox": {
                "allowNetwork": true,
                "roBinds": [],
                "rwBinds": []
            }
        });
        if let Ok(pretty) = serde_json::to_string_pretty(&default_settings) {
            if let Err(e) = fs::write(&settings_path, format!("{}\n", pretty)) {
                eprintln!("Failed to write {}: {}", settings_path.display(), e);
            } else {
                println!("[OK] Created {}", settings_path.display());
            }
        }
    }

    // 3. .ai/plugin.json
    if force || !plugin_path.exists() {
        let default_plugins = json!({
            "plugins": []
        });
        if let Ok(pretty) = serde_json::to_string_pretty(&default_plugins) {
            if let Err(e) = fs::write(&plugin_path, format!("{}\n", pretty)) {
                eprintln!("Failed to write {}: {}", plugin_path.display(), e);
            } else {
                println!("[OK] Created {}", plugin_path.display());
            }
        }
    }

    println!("\nProject AI sandbox initialized. Launch with: minimalctl ai run");
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
                if let Err(e) = std::os::unix::fs::symlink(source, &dest) {
                    eprintln!("Failed to link {}: {}", source, e);
                } else {
                    println!("Linked {} to {}", source, dest.display());
                }
            } else if source.starts_with("http") || source.starts_with("git@") {
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
            let has_agy = Path::new("/usr/sbin/agy").exists()
                || Command::new("which").arg("agy").output().map(|o| o.status.success()).unwrap_or(false);

            if pwd.join(".gemini").exists() || has_agy {
                println!("Auto-detected Antigravity environment. Using 'agy'.");
                "agy".to_string()
            } else if pwd.join("claude.json").exists() || pwd.join(".claude").exists() {
                println!("Auto-detected Claude environment. Using 'claude'.");
                "claude".to_string()
            } else {
                println!("No harness specified and no auto-detection matched.");
                println!("Available harnesses:");
                println!("  1) agy");
                println!("  2) claude");
                print!("Select a harness (1 or 2, or type custom name): ");
                io::stdout().flush().unwrap();

                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                let input = input.trim();

                match input {
                    "1" => "agy".to_string(),
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

    let ai_script_path = if Path::new(&home).join(".local/bin/ai").exists() {
        PathBuf::from(&home).join(".local/bin/ai")
    } else if pwd.join("scripts/ai").exists() {
        pwd.join("scripts/ai")
    } else {
        PathBuf::from(format!("{}/minimal/scripts/ai", home))
    };

    let mut ai_script = Command::new(&ai_script_path);
    ai_script.arg(&harness).args(args);

    let err = ai_script.exec();
    eprintln!("Failed to exec {}: {}", ai_script_path.display(), err);
    std::process::exit(1);
}
