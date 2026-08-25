//! Draft OCI Block Volume backend for the provider-block compatibility port.

#![forbid(unsafe_code)]

use storage_domain::{
    EncryptionMode, StorageProviderBlockCreateVolumeRequest, StorageProviderBlockError,
    StorageProviderBlockPort, StorageProviderBlockReceipt, StorageProviderKind, VolumeTier,
};

include!(concat!(env!("OUT_DIR"), "/lib.generated.rs"));
