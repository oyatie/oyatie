#![forbid(unsafe_code)]
//! Provider-neutral object storage and metadata storage port contracts.
//!
//! This crate is intentionally runtime-dependency-free at scaffold time. It models
//! Drive upload/download intent boundaries without coupling Oya Office to one object-store vendor.

use oya_office_kernel::{CellId, DataClass, ObjectId, TenantId};

/// Stable crate identifier used by workspace and Buck2 scaffold verification.
pub const CRATE_NAME: &str = "oya-office-storage-kernel";

/// Product vertical slice owned by this crate.
pub const VERTICAL_SLICE: &str = "platform";

/// Canonical ADR-0056 architectural layer represented by this crate. Port-trait
/// declarations + pure typed contracts live in the `kernel` layer.
pub const ARCHITECTURE_LAYER: &str = "kernel";

/// Storage validation error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoragePortError {
    message: String,
}

impl StoragePortError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Returns the validation message.
    #[must_use]
    pub fn message(&self) -> &str {
        self.message.as_str()
    }
}

impl core::fmt::Display for StoragePortError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.message())
    }
}

impl std::error::Error for StoragePortError {}

/// Provider-neutral object storage key.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StorageObjectKey(String);

impl StorageObjectKey {
    /// Creates a storage object key. Keys are internal and never public API paths.
    pub fn new(value: impl Into<String>) -> Result<Self, StoragePortError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(StoragePortError::new("storage key must not be empty"));
        }
        if trimmed.starts_with('/') || trimmed.contains("..") {
            return Err(StoragePortError::new(
                "storage key must be relative and normalized",
            ));
        }
        if trimmed.len() > 1024 {
            return Err(StoragePortError::new("storage key is too long"));
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// Returns key string for privileged storage adapters.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Inclusive byte range for bounded downloads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ByteRange {
    start: u64,
    end_inclusive: u64,
}

impl ByteRange {
    /// Creates an inclusive byte range.
    pub fn new(start: u64, end_inclusive: u64) -> Result<Self, StoragePortError> {
        if end_inclusive < start {
            return Err(StoragePortError::new(
                "byte range end must be greater than or equal to start",
            ));
        }
        Ok(Self {
            start,
            end_inclusive,
        })
    }

    /// Returns start offset.
    #[must_use]
    pub const fn start(self) -> u64 {
        self.start
    }

    /// Returns inclusive end offset.
    #[must_use]
    pub const fn end_inclusive(self) -> u64 {
        self.end_inclusive
    }

    /// Returns range length.
    #[must_use]
    pub const fn length(self) -> u64 {
        self.end_inclusive - self.start + 1
    }
}

/// Drive upload intent consumed by object-store adapters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UploadIntent {
    tenant_id: TenantId,
    cell_id: CellId,
    object_id: ObjectId,
    storage_key: StorageObjectKey,
    max_size_bytes: u64,
    data_class: DataClass,
}

impl UploadIntent {
    /// Creates an upload intent.
    pub fn new(
        tenant_id: TenantId,
        cell_id: CellId,
        object_id: ObjectId,
        storage_key: StorageObjectKey,
        max_size_bytes: u64,
        data_class: DataClass,
    ) -> Result<Self, StoragePortError> {
        if max_size_bytes == 0 {
            return Err(StoragePortError::new(
                "upload max size must be greater than zero",
            ));
        }
        Ok(Self {
            tenant_id,
            cell_id,
            object_id,
            storage_key,
            max_size_bytes,
            data_class,
        })
    }

    /// Returns tenant id.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns serving or home cell id.
    #[must_use]
    pub const fn cell_id(&self) -> &CellId {
        &self.cell_id
    }

    /// Returns object id.
    #[must_use]
    pub const fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    /// Returns storage key.
    #[must_use]
    pub const fn storage_key(&self) -> &StorageObjectKey {
        &self.storage_key
    }

    /// Returns max allowed upload size.
    #[must_use]
    pub const fn max_size_bytes(&self) -> u64 {
        self.max_size_bytes
    }

    /// Returns data class.
    #[must_use]
    pub const fn data_class(&self) -> DataClass {
        self.data_class
    }
}

/// Drive download intent consumed by object-store adapters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DownloadIntent {
    tenant_id: TenantId,
    cell_id: CellId,
    object_id: ObjectId,
    storage_key: StorageObjectKey,
    byte_range: Option<ByteRange>,
    data_class: DataClass,
}

impl DownloadIntent {
    /// Creates a download intent.
    #[must_use]
    pub const fn new(
        tenant_id: TenantId,
        cell_id: CellId,
        object_id: ObjectId,
        storage_key: StorageObjectKey,
        byte_range: Option<ByteRange>,
        data_class: DataClass,
    ) -> Self {
        Self {
            tenant_id,
            cell_id,
            object_id,
            storage_key,
            byte_range,
            data_class,
        }
    }

    /// Returns tenant id.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns serving or home cell id.
    #[must_use]
    pub const fn cell_id(&self) -> &CellId {
        &self.cell_id
    }

    /// Returns object id.
    #[must_use]
    pub const fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    /// Returns storage key.
    #[must_use]
    pub const fn storage_key(&self) -> &StorageObjectKey {
        &self.storage_key
    }

    /// Returns optional byte range.
    #[must_use]
    pub const fn byte_range(&self) -> Option<ByteRange> {
        self.byte_range
    }

    /// Returns data class.
    #[must_use]
    pub const fn data_class(&self) -> DataClass {
        self.data_class
    }
}

/// Storage adapter operation class for metrics and auth policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StorageOperation {
    /// Upload object content.
    Upload,
    /// Download object content.
    Download,
}

impl StorageOperation {
    /// Returns product metric name for the operation.
    #[must_use]
    pub const fn metric_name(self) -> &'static str {
        match self {
            Self::Upload => "oya_office_drive_storage_upload_seconds",
            Self::Download => "oya_office_drive_storage_download_seconds",
        }
    }
}

#[cfg(test)]
mod tests {
    use oya_office_kernel::{CellId, DataClass, ObjectId, TenantId};

    use super::{
        ARCHITECTURE_LAYER, ByteRange, CRATE_NAME, DownloadIntent, StorageObjectKey, UploadIntent,
        VERTICAL_SLICE,
    };

    #[test]
    fn scaffold_identity_is_declared() {
        assert!(!ARCHITECTURE_LAYER.is_empty());
        assert!(!CRATE_NAME.is_empty());
        assert!(!VERTICAL_SLICE.is_empty());
    }

    #[test]
    fn upload_download_intents_are_tenant_cell_and_object_scoped() {
        let key = StorageObjectKey::new("tenant-alpha/drive/object-1/content")
            .expect("valid storage key");
        let upload = UploadIntent::new(
            TenantId::new("tenant-alpha").expect("valid tenant id"),
            CellId::new("cell-us-1").expect("valid cell id"),
            ObjectId::new("object-1").expect("valid object id"),
            key.clone(),
            4096,
            DataClass::Confidential,
        )
        .expect("valid upload");

        let download = DownloadIntent::new(
            upload.tenant_id().clone(),
            upload.cell_id().clone(),
            upload.object_id().clone(),
            key,
            Some(ByteRange::new(0, 1023).expect("valid range")),
            upload.data_class(),
        );

        assert_eq!(download.tenant_id().as_str(), "tenant-alpha");
        assert_eq!(download.cell_id().as_str(), "cell-us-1");
        assert_eq!(download.byte_range().expect("range").length(), 1024);
    }
}
