//! Workspace drive kernel.
//!
//! Typed kernel records for the W-Workspace-Preview Drive surface named by
//! `docs/products/workspace/PRD.md` and ADR-0029. The kernel owns object/folder
//! identities, permission grants, conservative data-class defaults, and the
//! path-resolution seam consumed by Search and Foundry.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

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
// DRIVE-BUILD-001 — source-locked file/folder CRUD replay slice.
//
// This intentionally covers only the first RED-backed Drive fixture:
// `drive_file_folder_crud_contract_fixture`. Upload/download, share-link,
// permissions, preview/search, scan engines, sync, immutability workflows, and
// ontology projection stay deferred to follow-up Build cards.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DriveContextKind {
    Personal,
    Work,
    AdminAudit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DriveScanState {
    Pending,
    Clean,
    Flagged,
    Quarantined,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DriveFileLifecycleState {
    Active,
    SoftDeleted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DriveFileDeleteOutcome {
    SoftDeleted,
    LegalHoldDenied,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DriveSourceLockedFolderCreate {
    pub folder_id: String,                   // data_class: INTERNAL_ONLY
    pub parent_folder_id: Option<String>,    // data_class: INTERNAL_ONLY
    pub tenant_or_person_scope: String,      // data_class: INTERNAL_ONLY
    pub context_kind: DriveContextKind,      // data_class: INTERNAL_ONLY
    pub path: String,                        // data_class: PII_QUASI_IDENTIFIER
    pub permission_inheritance_mode: String, // data_class: INTERNAL_ONLY
    pub retention_policy_id: String,         // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DriveSourceLockedFileCreate {
    pub file_id: String,                      // data_class: INTERNAL_ONLY
    pub folder_id: String,                    // data_class: INTERNAL_ONLY
    pub tenant_or_person_scope: String,       // data_class: INTERNAL_ONLY
    pub context_kind: DriveContextKind,       // data_class: INTERNAL_ONLY
    pub path: String,                         // data_class: PII_QUASI_IDENTIFIER
    pub object_version_ref: String,           // data_class: INTERNAL_ONLY
    pub content_hash: String,                 // data_class: INTERNAL_ONLY
    pub mime_type: String,                    // data_class: INTERNAL_ONLY
    pub size_bytes: u64,                      // data_class: INTERNAL_ONLY
    pub retention_policy_id: String,          // data_class: INTERNAL_ONLY
    pub scan_state: DriveScanState,           // data_class: INTERNAL_ONLY
    pub data_class: Option<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub created_by: String,                   // data_class: PII_IDENTIFYING
    pub created_at_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
    pub audit_event_id: String,               // data_class: AUDIT
    pub legal_hold_open: bool,                // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DriveSourceLockedFolder {
    pub folder_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub parent_folder_id: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub tenant_or_person_scope: Classified<String>, // data_class: INTERNAL_ONLY
    pub context_kind: Classified<DriveContextKind>, // data_class: INTERNAL_ONLY
    pub path: Classified<String>,      // data_class: PII_QUASI_IDENTIFIER
    pub permission_inheritance_mode: Classified<String>, // data_class: INTERNAL_ONLY
    pub retention_policy_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DriveSourceLockedFile {
    pub file_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub folder_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub tenant_or_person_scope: Classified<String>, // data_class: INTERNAL_ONLY
    pub context_kind: Classified<DriveContextKind>, // data_class: INTERNAL_ONLY
    pub path: Classified<String>,      // data_class: PII_QUASI_IDENTIFIER
    pub object_version_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub content_hash: Classified<String>, // data_class: INTERNAL_ONLY
    pub mime_type: Classified<String>, // data_class: INTERNAL_ONLY
    pub size_bytes: Classified<u64>,   // data_class: INTERNAL_ONLY
    pub retention_policy_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub scan_state: Classified<DriveScanState>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub created_by: Classified<String>, // data_class: PII_IDENTIFYING
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub created_audit_event_id: Classified<String>, // data_class: AUDIT
    pub legal_hold_open: Classified<bool>, // data_class: INTERNAL_ONLY
    pub lifecycle_state: Classified<DriveFileLifecycleState>, // data_class: INTERNAL_ONLY
    pub last_mutation_audit_event_id: Classified<Option<String>>, // data_class: AUDIT
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

impl DriveSourceLockedFolder {
    pub fn new(input: DriveSourceLockedFolderCreate) -> Result<Self, DriveError> {
        validate_non_empty(&input.folder_id, DriveError::InvalidFolderId)?;
        if let Some(parent_folder_id) = input.parent_folder_id.as_deref() {
            validate_non_empty(parent_folder_id, DriveError::InvalidFolderId)?;
        }
        validate_non_empty(&input.tenant_or_person_scope, DriveError::InvalidTenantId)?;
        validate_drive_path(&input.path, true)?;
        validate_non_empty(
            &input.permission_inheritance_mode,
            DriveError::InvalidPermissionSubject,
        )?;
        validate_non_empty(
            &input.retention_policy_id,
            DriveError::InvalidObjectStorageKey,
        )?;

        Ok(Self {
            folder_id: internal(input.folder_id),
            parent_folder_id: internal(input.parent_folder_id),
            tenant_or_person_scope: internal(input.tenant_or_person_scope),
            context_kind: internal(input.context_kind),
            path: Classified::new(input.path, path_data_class()),
            permission_inheritance_mode: internal(input.permission_inheritance_mode),
            retention_policy_id: internal(input.retention_policy_id),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: internal(DRIVE_FOLDER_SCHEMA_VERSION),
        })
    }
}

impl DriveSourceLockedFile {
    pub fn new(input: DriveSourceLockedFileCreate) -> Result<Self, DriveError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_drive_data_class());
        validate_non_empty(&input.file_id, DriveError::InvalidObjectId)?;
        validate_non_empty(&input.folder_id, DriveError::InvalidFolderId)?;
        validate_non_empty(&input.tenant_or_person_scope, DriveError::InvalidTenantId)?;
        validate_drive_path(&input.path, false)?;
        validate_object_version_ref(&input.object_version_ref)?;
        validate_content_hash(&input.content_hash)?;
        validate_non_empty(&input.mime_type, DriveError::InvalidMimeType)?;
        validate_non_empty(
            &input.retention_policy_id,
            DriveError::InvalidObjectStorageKey,
        )?;
        validate_non_empty(&input.created_by, DriveError::InvalidPermissionSubject)?;
        validate_audit_event_id(&input.audit_event_id)?;

        Ok(Self {
            file_id: internal(input.file_id),
            folder_id: internal(input.folder_id),
            tenant_or_person_scope: internal(input.tenant_or_person_scope),
            context_kind: internal(input.context_kind),
            path: Classified::new(input.path, path_data_class()),
            object_version_ref: internal(input.object_version_ref),
            content_hash: internal(input.content_hash),
            mime_type: internal(input.mime_type),
            size_bytes: internal(input.size_bytes),
            retention_policy_id: internal(input.retention_policy_id),
            scan_state: internal(input.scan_state),
            data_class: internal(data_class),
            created_by: Classified::new(input.created_by, permission_data_class()),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            created_audit_event_id: Classified::new(input.audit_event_id, DataClass::Audit),
            legal_hold_open: internal(input.legal_hold_open),
            lifecycle_state: internal(DriveFileLifecycleState::Active),
            last_mutation_audit_event_id: Classified::new(None, DataClass::Audit),
            schema_version: internal(DRIVE_OBJECT_SCHEMA_VERSION),
        })
    }

    pub fn is_visible_to_readers(&self) -> bool {
        self.lifecycle_state.value == DriveFileLifecycleState::Active
            && self.scan_state.value == DriveScanState::Clean
            && !self.created_audit_event_id.value.trim().is_empty()
            && matches!(
                self.last_mutation_audit_event_id.value.as_deref(),
                Some(audit_event_id) if audit_event_id.starts_with("audit:drive:")
            )
    }

    pub fn record_scan_verdict(
        &mut self,
        scan_state: DriveScanState,
        audit_event_id: String,
    ) -> Result<(), DriveError> {
        validate_audit_event_id(&audit_event_id)?;
        self.scan_state = internal(scan_state);
        self.last_mutation_audit_event_id = Classified::new(Some(audit_event_id), DataClass::Audit);
        Ok(())
    }

    pub fn soft_delete(
        &mut self,
        audit_event_id: String,
    ) -> Result<DriveFileDeleteOutcome, DriveError> {
        validate_audit_event_id(&audit_event_id)?;
        if self.legal_hold_open.value {
            self.last_mutation_audit_event_id =
                Classified::new(Some(audit_event_id), DataClass::Audit);
            return Ok(DriveFileDeleteOutcome::LegalHoldDenied);
        }
        self.lifecycle_state = internal(DriveFileLifecycleState::SoftDeleted);
        self.last_mutation_audit_event_id = Classified::new(Some(audit_event_id), DataClass::Audit);
        Ok(DriveFileDeleteOutcome::SoftDeleted)
    }
}

fn validate_object_version_ref(value: &str) -> Result<(), DriveError> {
    validate_non_empty(value, DriveError::InvalidObjectStorageKey)?;
    if value.starts_with("object-version:drive:") {
        Ok(())
    } else {
        Err(DriveError::InvalidObjectStorageKey)
    }
}

fn validate_content_hash(value: &str) -> Result<(), DriveError> {
    validate_non_empty(value, DriveError::InvalidObjectStorageKey)?;
    let digest = value
        .strip_prefix("sha256:")
        .ok_or(DriveError::InvalidObjectStorageKey)?;
    if digest.len() == 64 && digest.chars().all(|ch| ch.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(DriveError::InvalidObjectStorageKey)
    }
}

fn validate_audit_event_id(value: &str) -> Result<(), DriveError> {
    validate_non_empty(value, DriveError::InvalidObjectStorageKey)?;
    if value.starts_with("audit:drive:") {
        Ok(())
    } else {
        Err(DriveError::InvalidObjectStorageKey)
    }
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
    use oya_data_boundary_kernel::{DataClassification, OperationalDataClass};

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

    #[test]
    fn drive_file_folder_crud_contract_fixture_records_source_locked_metadata_and_audit_visibility()
    {
        let folder = DriveSourceLockedFolder::new(DriveSourceLockedFolderCreate {
            folder_id: "folder-source-lock-1".into(),
            parent_folder_id: None,
            tenant_or_person_scope: "tenant:workspace-alpha".into(),
            context_kind: DriveContextKind::Work,
            path: "/team".into(),
            permission_inheritance_mode: "explicit".into(),
            retention_policy_id: "retention-standard".into(),
            created_at_epoch_seconds: 1_700_000_000,
        })
        .unwrap();

        assert_eq!(
            folder.tenant_or_person_scope.value,
            "tenant:workspace-alpha"
        );
        assert_eq!(folder.context_kind.value, DriveContextKind::Work);
        assert_eq!(folder.path.value, "/team");

        let pending = DriveSourceLockedFile::new(DriveSourceLockedFileCreate {
            file_id: "file-source-lock-1".into(),
            folder_id: folder.folder_id.value.clone(),
            tenant_or_person_scope: folder.tenant_or_person_scope.value.clone(),
            context_kind: DriveContextKind::Work,
            path: "/team/source-map.md".into(),
            object_version_ref: "object-version:drive:file-source-lock-1:v1".into(),
            content_hash: "sha256:6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d"
                .into(),
            mime_type: "text/markdown".into(),
            size_bytes: 2048,
            retention_policy_id: "retention-standard".into(),
            scan_state: DriveScanState::Pending,
            data_class: None,
            created_by: "user:owner@example.com".into(),
            created_at_epoch_seconds: 1_700_000_010,
            audit_event_id: "audit:drive:file-created:001".into(),
            legal_hold_open: false,
        })
        .unwrap();

        assert_eq!(
            pending.tenant_or_person_scope.value,
            "tenant:workspace-alpha"
        );
        assert_eq!(pending.context_kind.value, DriveContextKind::Work);
        assert_eq!(
            pending.content_hash.value,
            "sha256:6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d"
        );
        assert_eq!(
            pending.object_version_ref.value,
            "object-version:drive:file-source-lock-1:v1"
        );
        assert_eq!(
            pending.created_audit_event_id.value,
            "audit:drive:file-created:001"
        );
        assert!(!pending.is_visible_to_readers());

        let mut clean_without_scan_audit = pending.clone();
        clean_without_scan_audit.scan_state = internal(DriveScanState::Clean);
        assert!(!clean_without_scan_audit.is_visible_to_readers());

        let mut clean = pending.clone();
        clean
            .record_scan_verdict(DriveScanState::Clean, "audit:drive:scan-clean:001".into())
            .unwrap();
        assert!(clean.is_visible_to_readers());
    }

    #[test]
    fn drive_file_folder_crud_contract_fixture_denies_legal_hold_delete_and_soft_deletes_without_bypass()
     {
        let mut held = DriveSourceLockedFile::new(DriveSourceLockedFileCreate {
            file_id: "file-held".into(),
            folder_id: "folder-source-lock-1".into(),
            tenant_or_person_scope: "tenant:workspace-alpha".into(),
            context_kind: DriveContextKind::Work,
            path: "/team/held.pdf".into(),
            object_version_ref: "object-version:drive:file-held:v1".into(),
            content_hash: "sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
                .into(),
            mime_type: "application/pdf".into(),
            size_bytes: 4096,
            retention_policy_id: "retention-legal".into(),
            scan_state: DriveScanState::Clean,
            data_class: None,
            created_by: "user:owner@example.com".into(),
            created_at_epoch_seconds: 1_700_000_020,
            audit_event_id: "audit:drive:file-created:held".into(),
            legal_hold_open: true,
        })
        .unwrap();

        assert_eq!(
            held.soft_delete("audit:drive:file-delete:held".into()),
            Ok(DriveFileDeleteOutcome::LegalHoldDenied)
        );
        assert_eq!(held.lifecycle_state.value, DriveFileLifecycleState::Active);
        assert_eq!(
            held.last_mutation_audit_event_id.value,
            Some("audit:drive:file-delete:held".to_string())
        );

        let mut deletable = held.clone();
        deletable.legal_hold_open = internal(false);
        assert_eq!(
            deletable.soft_delete("audit:drive:file-delete:ok".into()),
            Ok(DriveFileDeleteOutcome::SoftDeleted)
        );
        assert_eq!(
            deletable.lifecycle_state.value,
            DriveFileLifecycleState::SoftDeleted
        );
        assert_eq!(
            deletable.last_mutation_audit_event_id.value,
            Some("audit:drive:file-delete:ok".to_string())
        );
    }
}
