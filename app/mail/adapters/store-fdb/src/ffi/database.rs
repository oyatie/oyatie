//! `FDBDatabase`: one handle per cluster file, shared across threads.

use std::ffi::CString;
use std::path::Path;
use std::ptr::NonNull;

use super::error::{FdbError, check};
use super::transaction::Transaction;
use super::{network, sys};

pub struct Database {
    raw: NonNull<sys::FDBDatabase>,
}

// SAFETY: the C API documents FDBDatabase as safe to share across threads.
unsafe impl Send for Database {}
unsafe impl Sync for Database {}

impl Database {
    /// Starts the client network if needed, then opens `cluster_file`.
    /// `fdb_create_database` succeeds before any coordinator is reached; the
    /// first operation reports connectivity.
    pub fn open(cluster_file: &Path) -> Result<Self, FdbError> {
        network::start()?;
        let path = CString::new(cluster_file.as_os_str().as_encoded_bytes())
            .map_err(|_| FdbError::from_code(2000))?;
        let mut raw = std::ptr::null_mut();
        // SAFETY: path is NUL-terminated and outlives the call; out-pointer valid.
        unsafe { check(sys::fdb_create_database(path.as_ptr(), &mut raw))? };
        Ok(Self {
            raw: NonNull::new(raw).ok_or_else(|| FdbError::from_code(2000))?,
        })
    }

    pub fn create_transaction(&self) -> Result<Transaction, FdbError> {
        let mut raw = std::ptr::null_mut();
        // SAFETY: raw database is live; out-pointer valid.
        unsafe {
            check(sys::fdb_database_create_transaction(
                self.raw.as_ptr(),
                &mut raw,
            ))?
        };
        NonNull::new(raw)
            .map(Transaction::from_raw)
            .ok_or_else(|| FdbError::from_code(2000))
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        // SAFETY: destroyed exactly once, here.
        unsafe { sys::fdb_database_destroy(self.raw.as_ptr()) }
    }
}
