//! Deprecated HTTP-shaped object metadata compatibility boundary.
//!
//! This is a local P0 fixture, not the sold storage facade. P1 replaces it
//! with the canonical protobuf transaction model.

use std::collections::BTreeMap;

use compute_resource::ResourceId;
use data_boundary_kernel::{DataClass, parse_data_class_label};
use secrets_kms_domain::KmsPurpose;
use storage_domain::{
    CloudStorageCatalog, CloudStorageError, ObjectCreate, ObjectEncryptionBindingCreate, ObjectKey,
    StorageRepo, StoredObject,
};

include!(concat!(env!("OUT_DIR"), "/lib.generated.rs"));

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    include!(concat!(env!("OUT_DIR"), "/tests.generated.rs"));
}
