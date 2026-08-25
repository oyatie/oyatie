//! Draft S3 backend for the storage provider-object compatibility port.
//!
//! The adapter emits deterministic command and receipt projections only; it
//! has no credentialed network execution or durability authority.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use storage_domain::{
    StorageProviderKind, StorageProviderObjectError, StorageProviderObjectGetRequest,
    StorageProviderObjectPort, StorageProviderObjectPutRequest, StorageProviderObjectReceipt,
};

include!(concat!(env!("OUT_DIR"), "/lib.generated.rs"));

#[cfg(test)]
mod tests {
    include!(concat!(env!("OUT_DIR"), "/tests.generated.rs"));
}
