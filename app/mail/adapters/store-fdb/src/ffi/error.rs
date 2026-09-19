//! Typed `fdb_error_t` with the client's own retry classification.

use std::ffi::{CStr, c_int};

use super::options::ErrorPredicate;
use super::sys;

/// A non-zero `fdb_error_t`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FdbError {
    code: i32,
}

impl FdbError {
    pub(super) const fn from_code(code: i32) -> Self {
        Self { code }
    }

    pub const fn code(self) -> i32 {
        self.code
    }

    /// `fdb_error_predicate`: `Retryable` means retry after `on_error`;
    /// `MaybeCommitted` means the commit may have applied and the caller must
    /// resolve through its own idempotency record first.
    pub fn satisfies(self, predicate: ErrorPredicate) -> bool {
        // SAFETY: pure function of two integers.
        unsafe { sys::fdb_error_predicate(predicate as c_int, self.code) != 0 }
    }
}

impl std::fmt::Display for FdbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // SAFETY: fdb_get_error returns a pointer to a static NUL-terminated
        // string for every code, including unknown ones.
        let text = unsafe { CStr::from_ptr(sys::fdb_get_error(self.code)) };
        write!(f, "fdb error {}: {}", self.code, text.to_string_lossy())
    }
}

impl std::error::Error for FdbError {}

/// Maps a raw return code to `Result`.
pub(super) fn check(code: sys::FdbError) -> Result<(), FdbError> {
    if code == 0 {
        Ok(())
    } else {
        Err(FdbError::from_code(code))
    }
}
