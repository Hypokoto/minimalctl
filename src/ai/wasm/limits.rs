#[derive(Debug, Clone)]
pub struct WasmLimits {
    /// Fuel units allocated for guest instruction execution.
    /// Default: 10_000_000 instructions
    pub fuel_budget: u64,
    /// Maximum linear memory pages (1 page = 64 KiB).
    /// Default: 512 pages = 32 MiB
    pub max_memory_pages: u32,
    /// Maximum input/output payload size in bytes.
    /// Default: 1 MiB
    pub max_payload_bytes: usize,
    /// Maximum stack size per execution in bytes.
    /// Default: 1 MiB
    pub max_stack_bytes: usize,
}

impl Default for WasmLimits {
    fn default() -> Self {
        Self {
            fuel_budget: 10_000_000,
            max_memory_pages: 512, // 32 MiB
            max_payload_bytes: 1024 * 1024, // 1 MiB
            max_stack_bytes: 1024 * 1024, // 1 MiB
        }
    }
}
