//! Foundry blob port: content-addressed bytes.
//!
//! One shared blob seam serves Foundry attachments and workbook bytes (and,
//! per the platform direction, Drive and mail attachments through the same
//! shape). Addresses are computed by the port from content, never assigned by
//! an adapter — so no adapter can hand back an address whose bytes it cannot
//! produce, and the conformance suite holds every adapter to that arithmetic.
//!
//! Blobs are tenant-scoped even though addressing is by content: a digest one
//! tenant stored must read as absent to every other, because cross-tenant
//! deduplication would leak the existence of another tenant's content.
#![forbid(unsafe_code)]

mod blob;
mod store;

pub mod conformance;

pub use blob::{BlobRef, BlobRefError};
pub use store::{BlobStore, BlobStoreError};
