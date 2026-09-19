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
//!   clear / range / atomic-op / watch / commit / on_error / conflict-range
//!   surface the store needs, plus [`Transaction::set_versionstamped_key`]
//!   for append-only audit keys built with [`crate::key`].

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

/// Vendoring criterion, part one. A hand-written binding stays cheaper than a
/// vendored one only while it is small: when the Rust under `src/ffi/`
/// exceeds this many code lines — lines that are neither blank nor
/// comment-only, `cat src/ffi/*.rs | grep -vE '^\s*$|^\s*//' | wc -l`;
/// comments are the safety argument a reviewer reads either way and a
/// vendored crate would not remove them — this module is replaced by
/// `foundationdb = "=0.11.0"` pinned in `third-party/mail/fdb/BUCK`, with its
/// transitive set and Reindeer fixups enumerated in that change.
pub const VENDOR_ABOVE_CODE_LINES: usize = 600;

/// Vendoring criterion, part two: the same replacement happens when [`sys`]
/// declares more than this many `libfdb_c` functions
/// (`grep -c 'pub fn fdb_' src/ffi/sys.rs`; `atexit` is the C runtime's and
/// is not counted). Every addition under `src/ffi/` re-measures both numbers
/// and states them in its PR text.
pub const VENDOR_ABOVE_FUNCTIONS: usize = 40;
