#[cfg(feature = "wasm")]
use crate::ai::wasm::error::WasmError;
#[cfg(feature = "wasm")]
use crate::ai::wasm::limits::WasmLimits;
use serde::Serialize;
#[cfg(feature = "wasm")]
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct ModuleInspection {
    pub path: String,
    pub size_bytes: usize,
    pub exports: Vec<ExportEntry>,
    pub imports: Vec<ImportEntry>,
}

#[derive(Debug, Serialize)]
pub struct ExportEntry {
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Serialize)]
pub struct ImportEntry {
    pub module: String,
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Serialize)]
pub struct ExecutionResult {
    pub function: String,
    pub output: serde_json::Value,
    pub fuel_consumed: u64,
    pub fuel_remaining: u64,
}

#[cfg(feature = "wasm")]
pub struct HostState {
    pub limits: WasmLimits,
    pub current_memory_bytes: usize,
}

#[cfg(feature = "wasm")]
impl HostState {
    pub fn new(limits: WasmLimits) -> Self {
        Self {
            limits,
            current_memory_bytes: 0,
        }
    }
}

#[cfg(feature = "wasm")]
impl wasmtime::ResourceLimiter for HostState {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        let max_bytes = (self.limits.max_memory_pages as usize) * 65536;
        if desired > max_bytes {
            Ok(false) // Reject linear memory growth beyond the configured limit
        } else {
            self.current_memory_bytes = desired;
            Ok(true)
        }
    }

    fn table_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        if desired > 10_000 {
            Ok(false)
        } else {
            Ok(true)
        }
    }
}

#[cfg(feature = "wasm")]
pub struct WasmEngine {
    engine: wasmtime::Engine,
}

#[cfg(feature = "wasm")]
impl WasmEngine {
    pub fn new(limits: &WasmLimits) -> Result<Self, WasmError> {
        let mut config = wasmtime::Config::new();
        config.consume_fuel(true);
        config.max_wasm_stack(limits.max_stack_bytes);

        let engine = wasmtime::Engine::new(&config)
            .map_err(|e| WasmError::CompilationFailed(e.to_string()))?;

        Ok(Self { engine })
    }

    pub fn inspect(&self, wasm_path: &Path) -> Result<ModuleInspection, WasmError> {
        let bytes = std::fs::read(wasm_path)
            .map_err(|e| WasmError::IoError(format!("Cannot read '{}': {}", wasm_path.display(), e)))?;

        let module = wasmtime::Module::new(&self.engine, &bytes)
            .map_err(|e| WasmError::CompilationFailed(e.to_string()))?;

        let mut exports = Vec::new();
        for export in module.exports() {
            let kind = match export.ty() {
                wasmtime::ExternType::Func(_) => "func",
                wasmtime::ExternType::Memory(_) => "memory",
                wasmtime::ExternType::Table(_) => "table",
                wasmtime::ExternType::Global(_) => "global",
                _ => "other",
            };
            exports.push(ExportEntry {
                name: export.name().to_string(),
                kind: kind.to_string(),
            });
        }

        let mut imports = Vec::new();
        for import in module.imports() {
            let kind = match import.ty() {
                wasmtime::ExternType::Func(_) => "func",
                wasmtime::ExternType::Memory(_) => "memory",
                wasmtime::ExternType::Table(_) => "table",
                wasmtime::ExternType::Global(_) => "global",
                _ => "other",
            };
            imports.push(ImportEntry {
                module: import.module().to_string(),
                name: import.name().to_string(),
                kind: kind.to_string(),
            });
        }

        Ok(ModuleInspection {
            path: wasm_path.display().to_string(),
            size_bytes: bytes.len(),
            exports,
            imports,
        })
    }

    pub fn exec_compute(
        &self,
        wasm_bytes: &[u8],
        func_name: &str,
        arg_i64: Option<i64>,
        limits: WasmLimits,
    ) -> Result<ExecutionResult, WasmError> {
        let module = wasmtime::Module::new(&self.engine, wasm_bytes)
            .map_err(|e| WasmError::CompilationFailed(e.to_string()))?;

        // In pure compute mode: zero host imports allowed.
        // If module requires imports, Instantiation fails closed immediately.
        let mut store = wasmtime::Store::new(&self.engine, HostState::new(limits.clone()));
        store
            .set_fuel(limits.fuel_budget)
            .map_err(|e| WasmError::ExecutionTrapped(format!("Failed to set fuel: {}", e)))?;
        store.limiter(|state| state);

        let instance = wasmtime::Instance::new(&mut store, &module, &[])
            .map_err(|e| WasmError::InstantiationFailed(e.to_string()))?;

        let func = instance
            .get_func(&mut store, func_name)
            .ok_or_else(|| WasmError::FunctionNotFound(func_name.to_string()))?;

        let ty = func.ty(&store);
        let param_count = ty.params().len();

        let mut params = Vec::new();
        if param_count == 1 {
            let input_val = match ty.params().next() {
                Some(wasmtime::ValType::I32) => wasmtime::Val::I32(arg_i64.unwrap_or(0) as i32),
                Some(wasmtime::ValType::I64) => wasmtime::Val::I64(arg_i64.unwrap_or(0)),
                _ => wasmtime::Val::I64(arg_i64.unwrap_or(0)),
            };
            params.push(input_val);
        } else if param_count > 1 {
            return Err(WasmError::ExecutionTrapped(format!(
                "Function '{}' expects {} parameters; pure compute prototype supports 0 or 1 parameter.",
                func_name, param_count
            )));
        }

        let mut results = Vec::new();
        for res_ty in ty.results() {
            match res_ty {
                wasmtime::ValType::I32 => results.push(wasmtime::Val::I32(0)),
                wasmtime::ValType::I64 => results.push(wasmtime::Val::I64(0)),
                wasmtime::ValType::F32 => results.push(wasmtime::Val::F32(0)),
                wasmtime::ValType::F64 => results.push(wasmtime::Val::F64(0)),
                _ => results.push(wasmtime::Val::I32(0)),
            }
        }

        func.call(&mut store, &params, &mut results)
            .map_err(|trap| {
                if let Ok(remaining) = store.get_fuel() {
                    if remaining == 0 {
                        return WasmError::FuelExhausted;
                    }
                }
                WasmError::ExecutionTrapped(trap.to_string())
            })?;

        let output_value = match results.first() {
            Some(wasmtime::Val::I32(v)) => serde_json::json!(v),
            Some(wasmtime::Val::I64(v)) => serde_json::json!(v),
            Some(wasmtime::Val::F32(v)) => serde_json::json!(v),
            Some(wasmtime::Val::F64(v)) => serde_json::json!(v),
            None => serde_json::Value::Null,
            _ => serde_json::Value::Null,
        };

        let remaining_fuel = store.get_fuel().unwrap_or(0);
        let fuel_consumed = limits.fuel_budget.saturating_sub(remaining_fuel);

        Ok(ExecutionResult {
            function: func_name.to_string(),
            output: output_value,
            fuel_consumed,
            fuel_remaining: remaining_fuel,
        })
    }
}

#[cfg(all(test, feature = "wasm"))]
mod tests {
    use super::*;

    #[test]
    fn test_pure_compute_execution() {
        let wat = r#"
            (module
                (func (export "calculate") (param i64) (result i64)
                    local.get 0
                    i64.const 42
                    i64.add
                )
            )
        "#;
        let wasm_bytes = wat::parse_str(wat).expect("WAT parse failed");
        let limits = WasmLimits::default();
        let engine = WasmEngine::new(&limits).expect("Engine initialization failed");

        let result = engine
            .exec_compute(&wasm_bytes, "calculate", Some(10), limits)
            .expect("Execution failed");

        assert_eq!(result.function, "calculate");
        assert_eq!(result.output, serde_json::json!(52));
        assert!(result.fuel_consumed > 0, "Fuel must be consumed during execution");
        assert!(result.fuel_remaining > 0, "Fuel remaining should be > 0");
    }

    #[test]
    fn test_fuel_exhaustion_infinite_loop() {
        let wat = r#"
            (module
                (func (export "spin") (param i64) (result i64)
                    (loop $infinite
                        br $infinite
                    )
                    local.get 0
                )
            )
        "#;
        let wasm_bytes = wat::parse_str(wat).expect("WAT parse failed");
        let limits = WasmLimits {
            fuel_budget: 5_000, // Small instruction budget
            ..Default::default()
        };
        let engine = WasmEngine::new(&limits).expect("Engine initialization failed");

        let err = engine
            .exec_compute(&wasm_bytes, "spin", Some(0), limits)
            .expect_err("Infinite loop must trap due to fuel exhaustion");

        match err {
            WasmError::FuelExhausted => {}
            other => panic!("Expected WasmError::FuelExhausted, got: {:?}", other),
        }
    }

    #[test]
    fn test_linear_memory_limit_rejection() {
        let wat = r#"
            (module
                (memory 1)
                (func (export "request_huge_memory") (param i64) (result i64)
                    i32.const 1000 ;; request to grow by 1000 pages (~64 MiB)
                    memory.grow
                    i64.extend_i32_s
                )
            )
        "#;
        let wasm_bytes = wat::parse_str(wat).expect("WAT parse failed");
        let limits = WasmLimits {
            max_memory_pages: 10, // Cap at 10 pages (~640 KiB)
            ..Default::default()
        };
        let engine = WasmEngine::new(&limits).expect("Engine initialization failed");

        let result = engine
            .exec_compute(&wasm_bytes, "request_huge_memory", None, limits)
            .expect("Execution should complete and return -1 on memory growth rejection");

        // In WebAssembly, memory.grow returns -1 on failure/rejection
        assert_eq!(
            result.output,
            serde_json::json!(-1),
            "Memory growth exceeding limits must be rejected by ResourceLimiter and return -1"
        );
    }

    #[test]
    fn test_unauthorized_imports_fail_closed() {
        let wat = r#"
            (module
                (import "host_env" "dangerous_call" (func $danger (result i64)))
                (func (export "exploit") (result i64)
                    call $danger
                )
            )
        "#;
        let wasm_bytes = wat::parse_str(wat).expect("WAT parse failed");
        let limits = WasmLimits::default();
        let engine = WasmEngine::new(&limits).expect("Engine initialization failed");

        let err = engine
            .exec_compute(&wasm_bytes, "exploit", None, limits)
            .expect_err("Module with unauthorized host imports must fail closed");

        match err {
            WasmError::InstantiationFailed(msg) => {
                assert!(
                    msg.contains("import"),
                    "Error should explicitly state import failure, got: {}",
                    msg
                );
            }
            other => panic!("Expected InstantiationFailed, got: {:?}", other),
        }
    }

    #[test]
    fn test_zero_arg_compute() {
        let wat = r#"
            (module
                (func (export "constant_value") (result i32)
                    i32.const 99
                )
            )
        "#;
        let wasm_bytes = wat::parse_str(wat).expect("WAT parse failed");
        let limits = WasmLimits::default();
        let engine = WasmEngine::new(&limits).expect("Engine initialization failed");

        let result = engine
            .exec_compute(&wasm_bytes, "constant_value", None, limits)
            .expect("Execution failed");

        assert_eq!(result.output, serde_json::json!(99));
        assert!(result.fuel_consumed > 0);
    }
}
