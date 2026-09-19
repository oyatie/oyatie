//! Process-wide client lifecycle: API version selection, the network thread,
//! and its shutdown. `libfdb_c` permits exactly one of each per process.

use std::sync::{Mutex, OnceLock};
use std::thread::JoinHandle;

use super::error::{FdbError, check};
use super::sys;

/// Header version this binding was transcribed against; also the runtime
/// version requested, so behaviour never floats with the installed library.
pub const API_VERSION: i32 = 730;

static STARTED: OnceLock<Result<(), FdbError>> = OnceLock::new();
static NETWORK_THREAD: Mutex<Option<JoinHandle<Result<(), FdbError>>>> = Mutex::new(None);

/// Selects the API version, sets up the network and runs it on a dedicated
/// thread. Idempotent: every call after the first returns the first outcome.
/// `fdb_stop_network` is registered with `atexit` so the thread is joined
/// before the runtime tears down.
pub fn start() -> Result<(), FdbError> {
    *STARTED.get_or_init(|| {
        // SAFETY: called at most once per process through OnceLock; the
        // network thread outlives every handle because stop runs at exit.
        unsafe {
            check(sys::fdb_select_api_version_impl(API_VERSION, API_VERSION))?;
            check(sys::fdb_setup_network())?;
        }
        let handle = std::thread::Builder::new()
            .name("fdb-network".into())
            // SAFETY: fdb_run_network blocks until fdb_stop_network; it is
            // the only caller and runs after setup succeeded.
            .spawn(|| unsafe { check(sys::fdb_run_network()) })
            .map_err(|_| FdbError::spawn_failed())?;
        *NETWORK_THREAD
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) = Some(handle);
        // SAFETY: atexit is the C runtime's; the handler only stops a network
        // this function started.
        unsafe {
            sys::atexit(stop_at_exit);
        }
        Ok(())
    })
}

unsafe extern "C" fn stop_at_exit() {
    // By the time atexit handlers run, the main thread's C++ thread-locals
    // inside libfdb_c may already be destroyed; a fresh thread has its own.
    // A spawn failure here must not panic inside an `extern "C"` handler.
    if let Ok(handle) = std::thread::Builder::new().spawn(stop) {
        let _ = handle.join();
    }
}

/// Stops the network thread and joins it. Safe to call more than once; the
/// process cannot start a network again afterwards.
pub fn stop() -> Result<(), FdbError> {
    let handle = NETWORK_THREAD
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .take();
    let Some(handle) = handle else {
        return Ok(());
    };
    // SAFETY: a network was started by `start` and is stopped once.
    unsafe { check(sys::fdb_stop_network())? };
    handle
        .join()
        .unwrap_or_else(|_| Err(FdbError::spawn_failed()))
}

impl FdbError {
    /// `operation_failed` (1000): the client could not run its network thread.
    const fn spawn_failed() -> Self {
        Self::from_code(1000)
    }
}
