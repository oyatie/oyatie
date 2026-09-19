//! FoundationDB store adapter for the mail product.
//!
//! Two builds share this crate:
//!
//! * the default build links no native library and compiles to a refusing
//!   shim — [`FdbStore::open`] returns [`mail_kernel::Error::Unavailable`];
//! * `--features fdb` compiles the hand-written `libfdb_c` binding under
//!   [`ffi`] and the constructor connects.
//!
//! `unsafe` is denied crate-wide and allowed only inside `ffi/`, which is the
//! sole place a raw `libfdb_c` pointer is visible. The [`key`] encoder is
//! plain Rust and compiled in both builds so its tests run under the
//! workspace verdict. Errors are the kernel's [`mail_kernel::Error`], the
//! type every `mail-api` port returns; the adapter that implements those
//! ports lands in a later step, and this crate exists so the build, lint and
//! live lane are proven first.
#![deny(unsafe_code)]

#[cfg(feature = "fdb")]
#[allow(unsafe_code)]
pub mod ffi;
pub mod key;
mod store;

pub use store::{FdbConfig, FdbStore, UNAVAILABLE_REASON};
