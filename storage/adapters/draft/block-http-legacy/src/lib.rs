//! Deprecated HTTP-shaped block metadata compatibility boundary.
//!
//! This remains a local P0 fixture; block promotion is a later independent
//! evidence-backed lane.

use std::collections::BTreeMap;

use data_boundary_kernel::{DataClass, parse_data_class_label};
use network_residency::{ResidencyClass, parse_residency_class_label};
use storage_domain::{
    BlockVolume, CloudStorageCatalog, CloudStorageError, EncryptionMode, StorageRepo, VolumeCreate,
    VolumePerformance, VolumeState, VolumeTier,
};

include!(concat!(env!("OUT_DIR"), "/lib.generated.rs"));
