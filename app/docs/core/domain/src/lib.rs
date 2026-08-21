//! Workspace docs kernel.
//!
//! Typed kernel records for the W-Workspace-Preview Docs surface named by
//! `docs/products/workspace/PRD.md` and ADR-0029. This crate models document
//! metadata, CRDT snapshot references, permission grants, and the read seam
//! consumed by Search and Foundry without owning protocol or storage adapters.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const DOC_SCHEMA_VERSION: u32 = 1;
const MIN_SNAPSHOT_BYTES: u64 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocError {
    InvalidDocId,
    InvalidTenantId,
    InvalidRegion,
    InvalidDrivePath,
    InvalidSnapshotId,
    InvalidSnapshotStorageKey,
    InvalidStateHash,
    EmptySnapshot,
    EmptyVersionHistory,
    VersionSnapshotMismatch,
    InvalidVersionId,
    InvalidAuthorRef,
    InvalidTimeOrder,
    InvalidPermissionSubject,
    EmptyPermissionSet,
    MissingOwnerGrant,
    InvalidDataClass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DocContentType {
    Document,
    Sheet,
    Slide,
    SitePage,
    Note,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DocRole {
    Viewer,
    Commenter,
    Editor,
    Owner,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocCreate {
    pub id: String,                            // data_class: INTERNAL_ONLY
    pub drive_path: String,                    // data_class: PII_QUASI_IDENTIFIER
    pub tenant_id: String,                     // data_class: INTERNAL_ONLY
    pub region: String,                        // data_class: INTERNAL_ONLY
    pub data_class: Option<PrivacyDataClass>,  // data_class: INTERNAL_ONLY
    pub crdt_snapshot: CrdtSnapshotRef,        // data_class: PII_IDENTIFYING
    pub content_type: DocContentType,          // data_class: INTERNAL_ONLY
    pub permissions: DocPermissionSet,         // data_class: PII_IDENTIFYING
    pub version_history: Vec<VersionRef>,      // data_class: PII_IDENTIFYING
    pub indexed_at_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,         // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: u64,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Doc {
    pub id: Classified<String>,                   // data_class: INTERNAL_ONLY
    pub drive_path: Classified<String>,           // data_class: PII_QUASI_IDENTIFIER
    pub tenant_id: Classified<String>,            // data_class: INTERNAL_ONLY
    pub region: Classified<String>,               // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub crdt_snapshot: Classified<CrdtSnapshotRef>, // data_class: PII_IDENTIFYING
    pub content_type: Classified<DocContentType>, // data_class: INTERNAL_ONLY
    pub permissions: Classified<DocPermissionSet>, // data_class: PII_IDENTIFYING
    pub version_history: Classified<Vec<VersionRef>>, // data_class: PII_IDENTIFYING
    pub indexed_at_epoch_seconds: Classified<Option<u64>>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrdtSnapshotRef {
    pub snapshot_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub storage_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub state_hash: Classified<String>,  // data_class: INTERNAL_ONLY
    pub byte_len: Classified<u64>,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionRef {
    pub version_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub snapshot_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub author_ref: Classified<String>,  // data_class: PII_IDENTIFYING
    pub state_hash: Classified<String>,  // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocPermissionSet {
    pub grants: Vec<DocPermissionGrant>, // data_class: PII_IDENTIFYING
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocPermissionGrant {
    pub subject_ref: Classified<String>, // data_class: PII_IDENTIFYING
    pub role: Classified<DocRole>,       // data_class: INTERNAL_ONLY
    pub granted_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
}

pub trait DocReader {
    fn read_doc_text(&self, tenant_id: &str, doc_id: &str) -> Result<Option<String>, DocError>;
}

impl Doc {
    pub fn new(input: DocCreate) -> Result<Self, DocError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_doc_data_class());
        validate_non_empty(&input.id, DocError::InvalidDocId)?;
        validate_drive_path(&input.drive_path)?;
        validate_non_empty(&input.tenant_id, DocError::InvalidTenantId)?;
        validate_non_empty(&input.region, DocError::InvalidRegion)?;
        validate_time_order(
            input.created_at_epoch_seconds,
            input.updated_at_epoch_seconds,
        )?;
        input.crdt_snapshot.validate()?;
        input.permissions.validate()?;
        validate_version_history(&input.crdt_snapshot, &input.version_history)?;

        Ok(Self {
            id: internal(input.id),
            drive_path: Classified::new(input.drive_path, path_data_class()),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            data_class: internal(data_class),
            crdt_snapshot: Classified::new(input.crdt_snapshot, data_class),
            content_type: internal(input.content_type),
            permissions: Classified::new(input.permissions, permission_data_class()),
            version_history: Classified::new(input.version_history, data_class),
            indexed_at_epoch_seconds: internal(input.indexed_at_epoch_seconds),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            updated_at_epoch_seconds: internal(input.updated_at_epoch_seconds),
            schema_version: internal(DOC_SCHEMA_VERSION),
        })
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }
}

impl CrdtSnapshotRef {
    pub fn new(
        snapshot_id: String,
        storage_key: String,
        state_hash: String,
        byte_len: u64,
    ) -> Result<Self, DocError> {
        validate_non_empty(&snapshot_id, DocError::InvalidSnapshotId)?;
        validate_non_empty(&storage_key, DocError::InvalidSnapshotStorageKey)?;
        validate_non_empty(&state_hash, DocError::InvalidStateHash)?;
        if byte_len < MIN_SNAPSHOT_BYTES {
            return Err(DocError::EmptySnapshot);
        }
        Ok(Self {
            snapshot_id: internal(snapshot_id),
            storage_key: internal(storage_key),
            state_hash: internal(state_hash),
            byte_len: internal(byte_len),
        })
    }

    fn validate(&self) -> Result<(), DocError> {
        validate_non_empty(&self.snapshot_id.value, DocError::InvalidSnapshotId)?;
        validate_non_empty(&self.storage_key.value, DocError::InvalidSnapshotStorageKey)?;
        validate_non_empty(&self.state_hash.value, DocError::InvalidStateHash)?;
        if self.byte_len.value < MIN_SNAPSHOT_BYTES {
            return Err(DocError::EmptySnapshot);
        }
        Ok(())
    }
}

impl VersionRef {
    pub fn new(
        version_id: String,
        snapshot_id: String,
        author_ref: String,
        state_hash: String,
        created_at_epoch_seconds: u64,
    ) -> Result<Self, DocError> {
        validate_non_empty(&version_id, DocError::InvalidVersionId)?;
        validate_non_empty(&snapshot_id, DocError::InvalidSnapshotId)?;
        validate_non_empty(&author_ref, DocError::InvalidAuthorRef)?;
        validate_non_empty(&state_hash, DocError::InvalidStateHash)?;
        Ok(Self {
            version_id: internal(version_id),
            snapshot_id: internal(snapshot_id),
            author_ref: Classified::new(author_ref, permission_data_class()),
            state_hash: internal(state_hash),
            created_at_epoch_seconds: internal(created_at_epoch_seconds),
        })
    }
}

impl DocPermissionSet {
    pub fn new(grants: Vec<DocPermissionGrant>) -> Result<Self, DocError> {
        let set = Self { grants };
        set.validate()?;
        Ok(set)
    }

    pub fn validate(&self) -> Result<(), DocError> {
        if self.grants.is_empty() {
            return Err(DocError::EmptyPermissionSet);
        }
        if !self
            .grants
            .iter()
            .any(|grant| grant.role.value == DocRole::Owner)
        {
            return Err(DocError::MissingOwnerGrant);
        }
        for grant in &self.grants {
            validate_non_empty(&grant.subject_ref.value, DocError::InvalidPermissionSubject)?;
        }
        Ok(())
    }

    pub fn role_for_subject(&self, subject_ref: &str) -> Option<DocRole> {
        self.grants
            .iter()
            .filter(|grant| grant.subject_ref.value == subject_ref)
            .map(|grant| grant.role.value)
            .max()
    }

    pub fn can_read(&self, subject_ref: &str) -> bool {
        self.role_for_subject(subject_ref).is_some()
    }

    pub fn can_write(&self, subject_ref: &str) -> bool {
        matches!(
            self.role_for_subject(subject_ref),
            Some(DocRole::Editor | DocRole::Owner)
        )
    }
}

impl DocPermissionGrant {
    pub fn new(
        subject_ref: String,
        role: DocRole,
        granted_at_epoch_seconds: u64,
    ) -> Result<Self, DocError> {
        validate_non_empty(&subject_ref, DocError::InvalidPermissionSubject)?;
        Ok(Self {
            subject_ref: Classified::new(subject_ref, permission_data_class()),
            role: internal(role),
            granted_at_epoch_seconds: internal(granted_at_epoch_seconds),
        })
    }
}

pub fn default_workspace_doc_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn path_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_quasi_identifier()
}

pub fn permission_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn workspace_doc_data_class_from_legacy(
    data_class: DataClass,
) -> Result<PrivacyDataClass, DocError> {
    PrivacyDataClass::new(data_class).map_err(|_| DocError::InvalidDataClass)
}

fn validate_version_history(
    snapshot: &CrdtSnapshotRef,
    versions: &[VersionRef],
) -> Result<(), DocError> {
    let Some(latest) = versions.last() else {
        return Err(DocError::EmptyVersionHistory);
    };
    for version in versions {
        validate_non_empty(&version.version_id.value, DocError::InvalidVersionId)?;
        validate_non_empty(&version.snapshot_id.value, DocError::InvalidSnapshotId)?;
        validate_non_empty(&version.author_ref.value, DocError::InvalidAuthorRef)?;
        validate_non_empty(&version.state_hash.value, DocError::InvalidStateHash)?;
    }
    if latest.snapshot_id.value != snapshot.snapshot_id.value {
        return Err(DocError::VersionSnapshotMismatch);
    }
    Ok(())
}

fn validate_drive_path(path: &str) -> Result<(), DocError> {
    if path.trim() != path
        || !path.starts_with('/')
        || path == "/"
        || path.ends_with('/')
        || path.contains("//")
        || path.chars().any(char::is_control)
    {
        return Err(DocError::InvalidDrivePath);
    }
    if path
        .split('/')
        .skip(1)
        .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(DocError::InvalidDrivePath);
    }
    Ok(())
}

fn validate_time_order(created_at: u64, updated_at: u64) -> Result<(), DocError> {
    if updated_at < created_at {
        Err(DocError::InvalidTimeOrder)
    } else {
        Ok(())
    }
}

fn validate_non_empty(value: &str, error: DocError) -> Result<(), DocError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_boundary_kernel::{DataClassification, OperationalDataClass};

    fn snapshot() -> CrdtSnapshotRef {
        CrdtSnapshotRef::new(
            "snap-1".into(),
            "tenant/doc/snap-1".into(),
            "sha256:abc".into(),
            42,
        )
        .unwrap()
    }

    fn owner_grant() -> DocPermissionGrant {
        DocPermissionGrant::new(
            "user:owner@example.com".into(),
            DocRole::Owner,
            1_700_000_000,
        )
        .unwrap()
    }

    fn viewer_grant() -> DocPermissionGrant {
        DocPermissionGrant::new(
            "user:viewer@example.com".into(),
            DocRole::Viewer,
            1_700_000_001,
        )
        .unwrap()
    }

    fn permissions() -> DocPermissionSet {
        DocPermissionSet::new(vec![owner_grant(), viewer_grant()]).unwrap()
    }

    fn version() -> VersionRef {
        VersionRef::new(
            "v1".into(),
            "snap-1".into(),
            "user:owner@example.com".into(),
            "sha256:abc".into(),
            1_700_000_010,
        )
        .unwrap()
    }

    fn doc_input() -> DocCreate {
        DocCreate {
            id: "doc-1".into(),
            drive_path: "/team/plan.oyadoc".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            data_class: None,
            crdt_snapshot: snapshot(),
            content_type: DocContentType::Document,
            permissions: permissions(),
            version_history: vec![version()],
            indexed_at_epoch_seconds: None,
            created_at_epoch_seconds: 1_700_000_000,
            updated_at_epoch_seconds: 1_700_000_010,
        }
    }

    #[test]
    fn doc_defaults_to_identifying_class_and_classifies_path_as_quasi() {
        let doc = Doc::new(doc_input()).unwrap();

        assert_eq!(
            doc.privacy_data_class().data_class(),
            DataClass::PiiIdentifying
        );
        assert_eq!(
            doc.drive_path.data_class,
            DataClassification::Privacy(path_data_class())
        );
        assert_eq!(doc.schema_version.value, 1);
    }

    #[test]
    fn doc_rejects_empty_snapshot_and_stale_version_head() {
        let empty_snapshot = CrdtSnapshotRef::new(
            "snap-1".into(),
            "tenant/doc/snap-1".into(),
            "sha256:abc".into(),
            0,
        );
        assert_eq!(empty_snapshot, Err(DocError::EmptySnapshot));

        let mut invalid = doc_input();
        invalid.crdt_snapshot = CrdtSnapshotRef::new(
            "snap-2".into(),
            "tenant/doc/snap-2".into(),
            "sha256:def".into(),
            42,
        )
        .unwrap();
        assert_eq!(Doc::new(invalid), Err(DocError::VersionSnapshotMismatch));
    }

    #[test]
    fn doc_rejects_drive_path_traversal_and_time_reversal() {
        let mut invalid = doc_input();
        invalid.drive_path = "/team/../secret.oyadoc".into();
        assert_eq!(Doc::new(invalid), Err(DocError::InvalidDrivePath));

        let mut invalid = doc_input();
        invalid.updated_at_epoch_seconds = invalid.created_at_epoch_seconds - 1;
        assert_eq!(Doc::new(invalid), Err(DocError::InvalidTimeOrder));
    }

    #[test]
    fn permissions_require_owner_and_distinguish_read_write() {
        let set = permissions();

        assert!(set.can_write("user:owner@example.com"));
        assert!(set.can_read("user:viewer@example.com"));
        assert!(!set.can_write("user:viewer@example.com"));
        assert_eq!(
            DocPermissionSet::new(vec![viewer_grant()]),
            Err(DocError::MissingOwnerGrant)
        );
    }

    #[test]
    fn legacy_data_class_conversion_rejects_operational_markers() {
        assert_eq!(
            workspace_doc_data_class_from_legacy(DataClass::Audit),
            Err(DocError::InvalidDataClass)
        );
        assert_eq!(
            DataClassification::from(OperationalDataClass::Audit).privacy_data_class(),
            None
        );
    }
}

// ---------------------------------------------------------------------------
// M03-P06-IP — workspace.docs STAGING surface markers (SPEC §4 rows).
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocsSurfaceStaging {
    pub doc_id: Classified<String>,            // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,         // data_class: INTERNAL_ONLY
    pub yrs_state_vector: Classified<Vec<u8>>, // data_class: INTERNAL_ONLY
}

impl DocsSurfaceStaging {
    pub fn new(doc_id: String, tenant_id: String, yrs_state_vector: Vec<u8>) -> Self {
        Self {
            doc_id: Classified::new(doc_id, DataClass::InternalOnly),
            tenant_id: Classified::new(tenant_id, DataClass::InternalOnly),
            yrs_state_vector: Classified::new(yrs_state_vector, DataClass::InternalOnly),
        }
    }
}

#[cfg(test)]
mod m03_p06_tests {
    use super::*;

    fn sample() -> DocsSurfaceStaging {
        DocsSurfaceStaging::new("docs-1".into(), "docs-1".into(), vec![])
    }

    #[test]
    fn surface_staging_constructor_sets_internal_only() {
        let s = sample();
        assert_eq!(s.doc_id.data_class, DataClass::InternalOnly.into());
    }

    #[test]
    fn surface_staging_round_trip_equality() {
        assert_eq!(sample(), sample());
    }
}
