use std::os::raw::c_void;
use std::thread;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};

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

// Opaque struct for uv_async_t (must be large enough to hold libuv's uv_async_t).
// In libuv, uv_async_t is usually around 128 bytes. We allocate 256 just to be safe.
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

extern "C" fn async_cb(_handle: *mut c_void) {
    // This runs on Neovim's main thread!
    unsafe {
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

        let loop_ptr = &raw mut main_loop as *mut c_void;

        let async_ptr = &raw mut ASYNC_HANDLE as *mut UvAsync as *mut c_void;
        let res = uv_async_init(loop_ptr, async_ptr, async_cb);
        if res != 0 {
            return res;
        }

        INITIALIZED = true;
        let async_ptr_val = async_ptr as usize;

        // Spawn a background thread to simulate external IPC trigger (Tracer Bullet)
        WORKER_THREAD = Some(thread::spawn(move || {
            let async_ptr = async_ptr_val as *mut c_void;
            while !SHUTDOWN_FLAG.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_secs(2));
                if SHUTDOWN_FLAG.load(Ordering::SeqCst) { break; }

                // Ping Neovim main thread!
                uv_async_send(async_ptr);
            }
        }));

        0
    }
}
