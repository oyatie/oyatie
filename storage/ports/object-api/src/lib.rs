//! Deprecated source-compatibility crate for the pre-P0 object HTTP package.
//!
//! New work must not depend on this HTTP-shaped contract. The implementation
//! is quarantined in the owner-local draft adapter until the P1 facade lands.

pub use storage_object_http_legacy_draft::*;
