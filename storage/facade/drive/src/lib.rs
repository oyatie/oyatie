//! Workspace drive kernel.
//!
//! Typed kernel records for the W-Workspace-Preview Drive surface named by
//! `docs/products/workspace/PRD.md` and ADR-0029. The kernel owns object/folder
//! identities, permission grants, conservative data-class defaults, and the
//! path-resolution seam consumed by Search and intelligence consumers.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const DRIVE_OBJECT_SCHEMA_VERSION: u32 = 1;
const DRIVE_FOLDER_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DriveError {
    InvalidObjectId,
    InvalidFolderId,
    InvalidTenantId,
    InvalidRegion,
    InvalidPath,
    InvalidObjectStorageKey,
    InvalidMimeType,
    InvalidKmsShredKeyId,
    InvalidPermissionSubject,
    EmptyPermissionSet,
    MissingOwnerGrant,
    InvalidDataClass,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DriveObjectCreate {
    pub id: String,                           // data_class: INTERNAL_ONLY
    pub folder_id: String,                    // data_class: INTERNAL_ONLY
    pub path: String,                         // data_class: PII_QUASI_IDENTIFIER
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub region: String,                       // data_class: INTERNAL_ONLY
    pub data_class: Option<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub object_storage_key: String,           // data_class: INTERNAL_ONLY
    pub size_bytes: u64,                      // data_class: INTERNAL_ONLY
    pub mime_type: String,                    // data_class: INTERNAL_ONLY
    pub kms_shred_key_id: String,             // data_class: INTERNAL_ONLY
    pub permissions: PermissionSet,           // data_class: PII_IDENTIFYING
    pub created_at_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DriveFolderCreate {
    pub id: String,                           // data_class: INTERNAL_ONLY
    pub parent_id: Option<String>,            // data_class: INTERNAL_ONLY
    pub path: String,                         // data_class: PII_QUASI_IDENTIFIER
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub region: String,                       // data_class: INTERNAL_ONLY
    pub data_class: Option<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub permissions: PermissionSet,           // data_class: PII_IDENTIFYING
    pub created_at_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DriveObject {
    pub id: Classified<String>,                    // data_class: INTERNAL_ONLY
    pub folder_id: Classified<String>,             // data_class: INTERNAL_ONLY
    pub path: Classified<String>,                  // data_class: PII_QUASI_IDENTIFIER
    pub tenant_id: Classified<String>,             // data_class: INTERNAL_ONLY
    pub region: Classified<String>,                // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>,  // data_class: INTERNAL_ONLY
    pub object_storage_key: Classified<String>,    // data_class: INTERNAL_ONLY
    pub size_bytes: Classified<u64>,               // data_class: INTERNAL_ONLY
    pub mime_type: Classified<String>,             // data_class: INTERNAL_ONLY
    pub kms_shred_key_id: Classified<String>,      // data_class: INTERNAL_ONLY
    pub permissions: Classified<PermissionSet>,    // data_class: PII_IDENTIFYING
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DriveFolder {
    pub id: Classified<String>,                    // data_class: INTERNAL_ONLY
    pub parent_id: Classified<Option<String>>,     // data_class: INTERNAL_ONLY
    pub path: Classified<String>,                  // data_class: PII_QUASI_IDENTIFIER
    pub tenant_id: Classified<String>,             // data_class: INTERNAL_ONLY
    pub region: Classified<String>,                // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>,  // data_class: INTERNAL_ONLY
    pub permissions: Classified<PermissionSet>,    // data_class: PII_IDENTIFYING
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PermissionSet {
    pub grants: Vec<PermissionGrant>, // data_class: PII_IDENTIFYING
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PermissionGrant {
    pub subject_ref: Classified<String>, // data_class: PII_IDENTIFYING
    pub role: Classified<DriveRole>,     // data_class: INTERNAL_ONLY
    pub granted_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DriveRole {
    Viewer,
    Commenter,
    Editor,
    Owner,
}

pub trait DrivePathProvider {
    fn object_storage_key_for_path(
        &self,
        tenant_id: &str,
        path: &str,
    ) -> Result<Option<String>, DriveError>;
}

impl DriveObject {
    pub fn new(input: DriveObjectCreate) -> Result<Self, DriveError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_drive_data_class());
        validate_non_empty(&input.id, DriveError::InvalidObjectId)?;
        validate_non_empty(&input.folder_id, DriveError::InvalidFolderId)?;
        validate_drive_path(&input.path, false)?;
        validate_non_empty(&input.tenant_id, DriveError::InvalidTenantId)?;
        validate_non_empty(&input.region, DriveError::InvalidRegion)?;
        validate_non_empty(
            &input.object_storage_key,
            DriveError::InvalidObjectStorageKey,
        )?;
        validate_non_empty(&input.mime_type, DriveError::InvalidMimeType)?;
        validate_non_empty(&input.kms_shred_key_id, DriveError::InvalidKmsShredKeyId)?;
        input.permissions.validate()?;

        Ok(Self {
            id: internal(input.id),
            folder_id: internal(input.folder_id),
            path: Classified::new(input.path, path_data_class()),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            data_class: internal(data_class),
            object_storage_key: internal(input.object_storage_key),
            size_bytes: internal(input.size_bytes),
            mime_type: internal(input.mime_type),
            kms_shred_key_id: internal(input.kms_shred_key_id),
            permissions: Classified::new(input.permissions, permission_data_class()),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: internal(DRIVE_OBJECT_SCHEMA_VERSION),
        })
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }
}

impl DriveFolder {
    pub fn new(input: DriveFolderCreate) -> Result<Self, DriveError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_drive_data_class());
        validate_non_empty(&input.id, DriveError::InvalidFolderId)?;
        if let Some(parent_id) = input.parent_id.as_deref() {
            validate_non_empty(parent_id, DriveError::InvalidFolderId)?;
        }
        validate_drive_path(&input.path, true)?;
        validate_non_empty(&input.tenant_id, DriveError::InvalidTenantId)?;
        validate_non_empty(&input.region, DriveError::InvalidRegion)?;
        input.permissions.validate()?;

        Ok(Self {
            id: internal(input.id),
            parent_id: internal(input.parent_id),
            path: Classified::new(input.path, path_data_class()),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            data_class: internal(data_class),
            permissions: Classified::new(input.permissions, permission_data_class()),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: internal(DRIVE_FOLDER_SCHEMA_VERSION),
        })
    }
}

impl PermissionSet {
    pub fn new(grants: Vec<PermissionGrant>) -> Result<Self, DriveError> {
        let set = Self { grants };
        set.validate()?;
        Ok(set)
    }

    pub fn validate(&self) -> Result<(), DriveError> {
        if self.grants.is_empty() {
            return Err(DriveError::EmptyPermissionSet);
        }
        if !self
            .grants
            .iter()
            .any(|grant| grant.role.value == DriveRole::Owner)
        {
            return Err(DriveError::MissingOwnerGrant);
        }
        for grant in &self.grants {
            validate_non_empty(
                &grant.subject_ref.value,
                DriveError::InvalidPermissionSubject,
            )?;
        }
        Ok(())
    }

    pub fn role_for_subject(&self, subject_ref: &str) -> Option<DriveRole> {
        self.grants
            .iter()
            .filter(|grant| grant.subject_ref.value == subject_ref)
            .map(|grant| grant.role.value)
            .max()
    }

    pub fn can_view(&self, subject_ref: &str) -> bool {
        self.role_for_subject(subject_ref).is_some()
    }

    pub fn can_edit(&self, subject_ref: &str) -> bool {
        matches!(
            self.role_for_subject(subject_ref),
            Some(DriveRole::Editor | DriveRole::Owner)
        )
    }
}

impl PermissionGrant {
    pub fn new(
        subject_ref: String,
        role: DriveRole,
        granted_at_epoch_seconds: u64,
    ) -> Result<Self, DriveError> {
        validate_non_empty(&subject_ref, DriveError::InvalidPermissionSubject)?;
        Ok(Self {
            subject_ref: Classified::new(subject_ref, permission_data_class()),
            role: internal(role),
            granted_at_epoch_seconds: internal(granted_at_epoch_seconds),
        })
    }
}

pub fn default_workspace_drive_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn path_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_quasi_identifier()
}

pub fn permission_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn workspace_drive_data_class_from_legacy(
    data_class: DataClass,
) -> Result<PrivacyDataClass, DriveError> {
    PrivacyDataClass::new(data_class).map_err(|_| DriveError::InvalidDataClass)
}

fn validate_drive_path(path: &str, allow_root: bool) -> Result<(), DriveError> {
    if path.trim() != path || !path.starts_with('/') || path.contains("//") {
        return Err(DriveError::InvalidPath);
    }
    if path == "/" {
        return if allow_root {
            Ok(())
        } else {
            Err(DriveError::InvalidPath)
        };
    }
    if path.ends_with('/') || path.chars().any(char::is_control) {
        return Err(DriveError::InvalidPath);
    }
    if path
        .split('/')
        .skip(1)
        .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(DriveError::InvalidPath);
    }
    Ok(())
}

fn validate_non_empty(value: &str, error: DriveError) -> Result<(), DriveError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

// ---------------------------------------------------------------------------
// M03-P06-IP-003 — workspace.drive.{put,get} STAGING surface (per-object
// KMS-shred reference + per-tenant cell routing + per-permission ACL).
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DriveSurfaceOp {
    Put,
    Get,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DriveSurfaceStaging {
    pub object_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub region: Classified<String>,     // data_class: INTERNAL_ONLY
    pub op: Classified<DriveSurfaceOp>, // data_class: INTERNAL_ONLY
    pub kms_shred_key_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub per_object_kms_shred: Classified<bool>, // data_class: INTERNAL_ONLY
    pub audit_emit_on_get: Classified<bool>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

impl DriveSurfaceStaging {
    pub fn new(
        object_id: String,
        tenant_id: String,
        region: String,
        op: DriveSurfaceOp,
        kms_shred_key_id: String,
    ) -> Result<Self, DriveError> {
        validate_non_empty(&object_id, DriveError::InvalidObjectId)?;
        validate_non_empty(&tenant_id, DriveError::InvalidTenantId)?;
        validate_non_empty(&region, DriveError::InvalidRegion)?;
        validate_non_empty(&kms_shred_key_id, DriveError::InvalidKmsShredKeyId)?;
        Ok(Self {
            object_id: internal(object_id),
            tenant_id: internal(tenant_id),
            region: internal(region),
            op: internal(op),
            kms_shred_key_id: internal(kms_shred_key_id),
            per_object_kms_shred: internal(true),
            audit_emit_on_get: internal(matches!(op, DriveSurfaceOp::Get)),
            schema_version: internal(DRIVE_OBJECT_SCHEMA_VERSION),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_boundary_kernel::{DataClassification, OperationalDataClass};

    fn owner_grant() -> PermissionGrant {
        PermissionGrant::new(
            "user:owner@example.com".into(),
            DriveRole::Owner,
            1_700_000_000,
        )
        .unwrap()
    }

    fn viewer_grant() -> PermissionGrant {
        PermissionGrant::new(
            "user:viewer@example.com".into(),
            DriveRole::Viewer,
            1_700_000_001,
        )
        .unwrap()
    }

    fn permission_set() -> PermissionSet {
        PermissionSet::new(vec![owner_grant(), viewer_grant()]).unwrap()
    }

    fn object_input() -> DriveObjectCreate {
        DriveObjectCreate {
            id: "object-1".into(),
            folder_id: "folder-1".into(),
            path: "/team/plan.md".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            data_class: None,
            object_storage_key: "tenant-1/team/plan.md".into(),
            size_bytes: 42,
            mime_type: "text/markdown".into(),
            kms_shred_key_id: "kms-key-1".into(),
            permissions: permission_set(),
            created_at_epoch_seconds: 1_700_000_010,
        }
    }

    fn folder_input() -> DriveFolderCreate {
        DriveFolderCreate {
            id: "folder-1".into(),
            parent_id: None,
            path: "/team".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            data_class: None,
            permissions: permission_set(),
            created_at_epoch_seconds: 1_700_000_000,
        }
    }

    #[test]
    fn drive_object_defaults_to_identifying_class_and_classifies_path() {
        let object = DriveObject::new(object_input()).unwrap();

        assert_eq!(
            object.privacy_data_class().data_class(),
            DataClass::PiiIdentifying
        );
        assert_eq!(
            object.path.data_class,
            DataClassification::Privacy(path_data_class())
        );
        assert_eq!(object.schema_version.value, 1);
    }

    #[test]
    fn drive_object_rejects_root_or_parent_traversal_paths() {
        let mut invalid = object_input();
        invalid.path = "/".into();
        assert_eq!(DriveObject::new(invalid), Err(DriveError::InvalidPath));

        let mut invalid = object_input();
        invalid.path = "/team/../secret.md".into();
        assert_eq!(DriveObject::new(invalid), Err(DriveError::InvalidPath));
    }

    #[test]
    fn drive_folder_allows_root_path_and_preserves_permissions() {
        let mut input = folder_input();
        input.path = "/".into();
        let folder = DriveFolder::new(input).unwrap();

        assert_eq!(folder.path.value, "/");
        assert_eq!(
            folder.permissions.data_class,
            DataClassification::Privacy(permission_data_class())
        );
    }

    #[test]
    fn permission_set_requires_owner_and_exposes_role_capabilities() {
        let permissions = permission_set();

        assert!(permissions.can_edit("user:owner@example.com"));
        assert!(permissions.can_view("user:viewer@example.com"));
        assert!(!permissions.can_edit("user:viewer@example.com"));

        let missing_owner = PermissionSet::new(vec![viewer_grant()]);
        assert_eq!(missing_owner, Err(DriveError::MissingOwnerGrant));
    }

    #[test]
    fn surface_staging_pins_per_object_kms_shred_and_audit_on_get() {
        let put = DriveSurfaceStaging::new(
            "obj-1".into(),
            "tenant-1".into(),
            "region-alpha1".into(),
            DriveSurfaceOp::Put,
            "kms-key-1".into(),
        )
        .unwrap();
        assert!(put.per_object_kms_shred.value);
        assert!(!put.audit_emit_on_get.value);
        assert_eq!(put.op.value, DriveSurfaceOp::Put);

        let get = DriveSurfaceStaging::new(
            "obj-1".into(),
            "tenant-1".into(),
            "region-alpha1".into(),
            DriveSurfaceOp::Get,
            "kms-key-1".into(),
        )
        .unwrap();
        assert!(get.audit_emit_on_get.value);
    }

    #[test]
    fn surface_staging_rejects_empty_kms_or_tenant() {
        assert_eq!(
            DriveSurfaceStaging::new(
                "obj-1".into(),
                "".into(),
                "region-alpha1".into(),
                DriveSurfaceOp::Put,
                "kms-key-1".into(),
            ),
            Err(DriveError::InvalidTenantId)
        );
        assert_eq!(
            DriveSurfaceStaging::new(
                "obj-1".into(),
                "tenant-1".into(),
                "region-alpha1".into(),
                DriveSurfaceOp::Put,
                "".into(),
            ),
            Err(DriveError::InvalidKmsShredKeyId)
        );
    }

    #[test]
    fn legacy_data_class_conversion_rejects_operational_markers() {
        assert_eq!(
            workspace_drive_data_class_from_legacy(DataClass::Audit),
            Err(DriveError::InvalidDataClass)
        );
        assert_eq!(
            DataClassification::from(OperationalDataClass::Audit).privacy_data_class(),
            None
        );
    }
}
