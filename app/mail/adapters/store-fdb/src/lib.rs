//! FoundationDB store adapter for the mail product.
//!
//! Two builds share this crate:
//!
//! * the default build links no native library and compiles to a refusing
//!   shim — [`FdbStore::open`] returns [`mail_kernel::Error::Unavailable`];
//! * `--features fdb` compiles the hand-written `libfdb_c` binding under
//!   [`ffi`] and the constructor connects.
//!
//! `unsafe` is denied crate-wide and allowed only inside `ffi/`, the sole
//! place a raw `libfdb_c` pointer is visible. The [`key`] encoder is plain
//! Rust and compiled in both builds, so its tests run under the workspace
//! verdict even when the native library is absent.
#![deny(unsafe_code)]

#[cfg(feature = "fdb")]
#[allow(unsafe_code)]
pub mod ffi;
pub mod key;
mod store;

pub use store::{FdbConfig, FdbStore, UNAVAILABLE_REASON};
