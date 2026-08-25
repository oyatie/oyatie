//! Draft OCI Object Storage backend for the provider-object compatibility port.

#![forbid(unsafe_code)]

use storage_domain::{
    StorageProviderKind, StorageProviderObjectError, StorageProviderObjectGetRequest,
    StorageProviderObjectPort, StorageProviderObjectPutRequest, StorageProviderObjectReceipt,
};

include!(concat!(env!("OUT_DIR"), "/lib.generated.rs"));
