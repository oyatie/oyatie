//! Hand-written binding to `libfdb_c` (FoundationDB 7.3.79, API version
//! 730). This is the only module tree in the mail product where `unsafe` is
//! allowed; everything above it sees owned Rust types. Layers, bottom up:
//!
//! * [`sys`] — `unsafe extern "C"` declarations transcribed from `fdb_c.h`;
//!   never called from outside `ffi/`. Option and enum wire codes are the
//!   discriminants of the typed enums re-exported below.
//! * [`FdbError`] — typed `fdb_error_t` with the client's retry predicates.
//! * [`network`] — `fdb_select_api_version_impl(730, 730)`, one network
//!   thread per process behind a `OnceLock`, stopped through `atexit`.
//! * [`FdbFuture`] — an `FDBFuture` that implements `Future` by registering
//!   `fdb_future_set_callback` with the task's `Waker`; [`Ready`] copies
//!   values out. [`block_on`] is a park-based executor for tests and tools.
//! * [`Database`] and [`Transaction`] — owned handles with the get / set /
//!   clear / range / commit / read-version surface. Keys are built with
//!   [`crate::key`].

/// Linking: `#[link(name = "fdb_c")]` resolves against the `libfdb_c` the
/// lane provides (`.github/scripts/live-fdb.sh`); `cargo clippy --features
/// fdb` type-checks without the library, only test binaries link it.
pub mod sys;

mod database;
mod error;
mod future;
pub mod network;
mod options;
mod transaction;

pub use database::Database;
pub use error::FdbError;
pub use future::{FdbFuture, KeyValues, Kind, Ready, block_on};
pub use network::API_VERSION;
pub use options::{ErrorPredicate, KeySelector, RangeOptions, StreamingMode};
pub use transaction::Transaction;

/// ponytail: a hand-written binding stays cheaper than a vendored one only
/// while it is small. Past this many code lines under `src/ffi/` — neither
/// blank nor comment-only, `cat src/ffi/*.rs | grep -vE '^\s*$|^\s*//' | wc
/// -l` — replace this module with `foundationdb = "=0.11.0"` pinned in
/// `third-party/mail/fdb/BUCK`.
pub const VENDOR_ABOVE_CODE_LINES: usize = 600;

/// The same replacement when [`sys`] declares more than this many `libfdb_c`
/// functions (`grep -c 'pub fn fdb_' src/ffi/sys.rs`; `atexit` is the C
/// runtime's and is not counted). Every addition under `src/ffi/` remeasures
/// both numbers.
pub const VENDOR_ABOVE_FUNCTIONS: usize = 40;
