use std::fmt;

#[derive(Debug)]
pub enum WasmError {
    CompilationFailed(String),
    InstantiationFailed(String),
    FunctionNotFound(String),
    ExecutionTrapped(String),
    FuelExhausted,
    MemoryExceeded(String),
    PayloadTooLarge { size: usize, limit: usize },
    IoError(String),
    JsonError(String),
    FeatureDisabled,
}

impl fmt::Display for WasmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WasmError::CompilationFailed(msg) => write!(f, "WASM Compilation Failed: {}", msg),
            WasmError::InstantiationFailed(msg) => write!(f, "WASM Instantiation Failed: {}", msg),
            WasmError::FunctionNotFound(name) => write!(f, "Exported function '{}' not found in module", name),
            WasmError::ExecutionTrapped(msg) => write!(f, "WASM Execution Trapped: {}", msg),
            WasmError::FuelExhausted => write!(f, "Execution aborted: Fuel budget exhausted (instruction budget exceeded)"),
            WasmError::MemoryExceeded(msg) => write!(f, "Execution aborted: Memory limit exceeded ({})", msg),
            WasmError::PayloadTooLarge { size, limit } => {
                write!(f, "Input payload size ({} bytes) exceeds limit ({} bytes)", size, limit)
            }
            WasmError::IoError(msg) => write!(f, "I/O Error: {}", msg),
            WasmError::JsonError(msg) => write!(f, "JSON Serialization/Deserialization Error: {}", msg),
            WasmError::FeatureDisabled => {
                write!(f, "WASM runtime is disabled in this build. Recompile with: cargo build --features wasm")
            }
        }
    }
}

impl std::error::Error for WasmError {}
