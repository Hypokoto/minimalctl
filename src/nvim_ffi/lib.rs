use std::os::raw::c_void;
use std::thread;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};

#[path = "../ipc_shm.rs"]
pub mod ipc_shm;
use ipc_shm::ShmBuffer;

// libuv symbols exposed by Neovim
extern "C" {
    fn uv_async_init(
        loop_ptr: *mut c_void,
        async_ptr: *mut c_void,
        cb: extern "C" fn(*mut c_void),
    ) -> i32;
    fn uv_async_send(async_ptr: *mut c_void) -> i32;

    // Neovim's global main_loop struct. The first field is the uv_loop_t.
    static mut main_loop: c_void;
}

#[repr(C, align(8))]
pub struct UvAsync {
    _data: [u8; 256],
}

// Global state
static mut ASYNC_HANDLE: UvAsync = UvAsync { _data: [0; 256] };
static mut LUA_CALLBACK: Option<extern "C" fn()> = None;
static mut INITIALIZED: bool = false;
static SHUTDOWN_FLAG: AtomicBool = AtomicBool::new(false);
static mut WORKER_THREAD: Option<thread::JoinHandle<()>> = None;

// For the C-API to read the latest state
static mut LATEST_SHM: Option<ShmBuffer> = None;
static mut LATEST_SEQ: u64 = 0;
static mut LATEST_BYTES: Vec<u8> = Vec::new();

extern "C" fn async_cb(_handle: *mut c_void) {
    // This runs on Neovim's main thread!
    unsafe {
        // We pull the latest payload inside the main thread to avoid race conditions with Lua reading it
        if let Some(shm) = &LATEST_SHM {
            if let Some((new_seq, bytes)) = shm.read_payload(LATEST_SEQ) {
                LATEST_SEQ = new_seq;
                LATEST_BYTES = bytes;
            }
        }

        if let Some(cb) = LUA_CALLBACK {
            cb();
        }
    }
}

#[no_mangle]
pub extern "C" fn minimal_nvim_shutdown() {
    SHUTDOWN_FLAG.store(true, Ordering::SeqCst);
    unsafe {
        if let Some(handle) = std::ptr::replace(&raw mut WORKER_THREAD, None) {
            let _ = handle.join();
        }
    }
}

#[no_mangle]
pub extern "C" fn minimal_nvim_init(cb: extern "C" fn()) -> i32 {
    unsafe {
        if INITIALIZED {
            LUA_CALLBACK = Some(cb);
            return 0; // Already initialized
        }

        LUA_CALLBACK = Some(cb);
        SHUTDOWN_FLAG.store(false, Ordering::SeqCst);

        // Open SHM Buffer
        let shm_result = ShmBuffer::open_read_only();
        if let Ok(shm) = shm_result {
            LATEST_SHM = Some(shm);
        } else {
            return -2; // Failed to open SHM
        }

        let loop_ptr = &raw mut main_loop as *mut c_void;

        let async_ptr = &raw mut ASYNC_HANDLE as *mut UvAsync as *mut c_void;
        let res = uv_async_init(loop_ptr, async_ptr, async_cb);
        if res != 0 {
            return res;
        }

        INITIALIZED = true;
        let async_ptr_val = async_ptr as usize;

        WORKER_THREAD = Some(thread::spawn(move || {
            let async_ptr = async_ptr_val as *mut c_void;
            let mut local_seq = 0;
            // We need our own shm instance in the background thread for polling
            let shm = ShmBuffer::open_read_only().unwrap();

            while !SHUTDOWN_FLAG.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(50));

                // Poll for changes
                if let Some((new_seq, _)) = shm.read_payload(local_seq) {
                    local_seq = new_seq;
                    unsafe {
                        uv_async_send(async_ptr);
                    }
                }
            }
        }));

        0
    }
}

// Zero-Copy C API for LuaJIT
#[no_mangle]
pub extern "C" fn minimal_nvim_get_total_additions() -> u32 {
    unsafe {
        if LATEST_BYTES.is_empty() { return 0; }
        if let Ok(state) = rkyv::check_archived_root::<ipc_shm::IdeState>(&LATEST_BYTES) {
            return state.total_additions as u32;
        }
        0
    }
}

#[no_mangle]
pub extern "C" fn minimal_nvim_get_num_files() -> u32 {
    unsafe {
        if LATEST_BYTES.is_empty() { return 0; }
        if let Ok(state) = rkyv::check_archived_root::<ipc_shm::IdeState>(&LATEST_BYTES) {
            return state.changed_files.len() as u32;
        }
        0
    }
}

#[no_mangle]
pub extern "C" fn minimal_nvim_get_file_path(file_idx: u32, len_out: *mut usize) -> *const u8 {
    unsafe {
        if LATEST_BYTES.is_empty() { return std::ptr::null(); }
        if let Ok(state) = rkyv::check_archived_root::<ipc_shm::IdeState>(&LATEST_BYTES) {
            if (file_idx as usize) < state.changed_files.len() {
                let s = &state.changed_files[file_idx as usize].path;
                if !len_out.is_null() {
                    *len_out = s.len();
                }
                return s.as_ptr();
            }
        }
        std::ptr::null()
    }
}

#[no_mangle]
pub extern "C" fn minimal_nvim_get_num_hunks(file_idx: u32) -> u32 {
    unsafe {
        if LATEST_BYTES.is_empty() { return 0; }
        if let Ok(state) = rkyv::check_archived_root::<ipc_shm::IdeState>(&LATEST_BYTES) {
            if (file_idx as usize) < state.changed_files.len() {
                return state.changed_files[file_idx as usize].hunks.len() as u32;
            }
        }
        0
    }
}

#[no_mangle]
pub extern "C" fn minimal_nvim_get_hunk_new_start(file_idx: u32, hunk_idx: u32) -> u32 {
    unsafe {
        if LATEST_BYTES.is_empty() { return 0; }
        if let Ok(state) = rkyv::check_archived_root::<ipc_shm::IdeState>(&LATEST_BYTES) {
            if let Some(file) = state.changed_files.get(file_idx as usize) {
                if let Some(hunk) = file.hunks.get(hunk_idx as usize) {
                    return hunk.new_start as u32;
                }
            }
        }
        0
    }
}

#[no_mangle]
pub extern "C" fn minimal_nvim_get_num_lines(file_idx: u32, hunk_idx: u32) -> u32 {
    unsafe {
        if LATEST_BYTES.is_empty() { return 0; }
        if let Ok(state) = rkyv::check_archived_root::<ipc_shm::IdeState>(&LATEST_BYTES) {
            if let Some(file) = state.changed_files.get(file_idx as usize) {
                if let Some(hunk) = file.hunks.get(hunk_idx as usize) {
                    return hunk.lines.len() as u32;
                }
            }
        }
        0
    }
}

#[no_mangle]
pub extern "C" fn minimal_nvim_get_line_tag(file_idx: u32, hunk_idx: u32, line_idx: u32) -> u8 {
    unsafe {
        if LATEST_BYTES.is_empty() { return b' '; }
        if let Ok(state) = rkyv::check_archived_root::<ipc_shm::IdeState>(&LATEST_BYTES) {
            if let Some(file) = state.changed_files.get(file_idx as usize) {
                if let Some(hunk) = file.hunks.get(hunk_idx as usize) {
                    if let Some(line) = hunk.lines.get(line_idx as usize) {
                        if !line.tag.is_empty() {
                            return line.tag.as_bytes()[0];
                        }
                    }
                }
            }
        }
        b' '
    }
}
