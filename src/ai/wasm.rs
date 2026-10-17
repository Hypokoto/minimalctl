use clap::Subcommand;
use std::path::PathBuf;

pub mod engine;
pub mod error;
pub mod limits;

#[derive(Subcommand, Debug)]
pub enum WasmAction {
    /// Inspect a WebAssembly module (exports, imports, size, safety) without execution
    Inspect {
        /// Path to the .wasm file
        path: PathBuf,
    },
    /// Execute an exported compute function with fuel and memory boundaries
    Exec {
        /// Path to the .wasm file
        path: PathBuf,
        /// Exported function name to call
        #[arg(short, long)]
        func: String,
        /// Optional integer argument passed to the function
        #[arg(short, long)]
        arg: Option<i64>,
        /// Fuel instruction budget (default: 10_000_000)
        #[arg(long, default_value = "10000000")]
        fuel: u64,
        /// Max memory pages (1 page = 64 KiB, default: 512 = 32 MiB)
        #[arg(long, default_value = "512")]
        max_pages: u32,
    },
}

pub fn handle_wasm(action: &WasmAction) {
    #[cfg(not(feature = "wasm"))]
    {
        let _ = action;
        eprintln!("Error: {}", error::WasmError::FeatureDisabled);
        std::process::exit(1);
    }

    #[cfg(feature = "wasm")]
    {
        use engine::WasmEngine;
        use limits::WasmLimits;

        let default_limits = WasmLimits::default();
        let engine = match WasmEngine::new(&default_limits) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Failed to initialize WASM engine: {}", e);
                std::process::exit(1);
            }
        };

        match action {
            WasmAction::Inspect { path } => {
                match engine.inspect(path) {
                    Ok(info) => {
                        println!("{}", serde_json::to_string_pretty(&info).unwrap());
                    }
                    Err(e) => {
                        eprintln!("Inspection failed: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            WasmAction::Exec {
                path,
                func,
                arg,
                fuel,
                max_pages,
            } => {
                let limits = WasmLimits {
                    fuel_budget: *fuel,
                    max_memory_pages: *max_pages,
                    ..Default::default()
                };

                let wasm_bytes = match std::fs::read(path) {
                    Ok(b) => b,
                    Err(e) => {
                        eprintln!("Failed to read {}: {}", path.display(), e);
                        std::process::exit(1);
                    }
                };

                match engine.exec_compute(&wasm_bytes, func, *arg, limits) {
                    Ok(result) => {
                        println!("{}", serde_json::to_string_pretty(&result).unwrap());
                    }
                    Err(e) => {
                        eprintln!("Execution failed: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
    }
}
