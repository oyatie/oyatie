#![forbid(unsafe_code)]
//! Drive file, folder, object metadata, ACL, KMS-shred, lifecycle, version, and trash invariants.
//!
//! The early Drive domain deliberately models security before storage adapters:
//! tenant ownership, object-level authorization, KMS-shred state, redacted
//! metadata views, and immutable audit emission. Runtime persistence and API
//! middleware land in later source-shaped layers.

use oya_office_authz_domain::{
    AclRole, AuthorizationDecision, AuthorizationRequest, DriveAction, ResourceRef,
};
use oya_office_kernel::{
    AuditEvent, DataClass, ObjectId, PrincipalId, RequestContext, RequestId, TenantId,
};

/// Stable crate identifier used by workspace and Buck2 scaffold verification.
pub const CRATE_NAME: &str = "oya-office-drive-domain";

/// Product vertical slice owned by this crate.
pub const VERTICAL_SLICE: &str = "drive";

/// Source-shaped architectural layer represented by this crate.
pub const ARCHITECTURE_LAYER: &str = "domain";

/// G080 Drive/contracts lane evidence version.
pub const G080_DRIVE_CONTRACT_LANE_VERSION: &str = "g080-drive-contracts-v1";

/// Build/source and security capability that must remain visible in the Drive lane.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DriveContractGateKind {
    /// Built-in Drive object domain and metadata contract.
    BuiltInDriveDomain,
    /// Object-level ACL authorization contract.
    AclAuthorization,
    /// KMS-shred content-access denial contract.
    KmsShred,
    /// Immutable audit projection contract.
    AuditEmission,
    /// Search/index request and result contract.
    SearchContract,
    /// Object version-history contract.
    VersionHistory,
    /// Trash/lifecycle transition contract.
    TrashLifecycle,
    /// Quota impact contract.
    Quota,
    /// Versioned API/event envelope contract.
    ApiEventContract,
    /// Tenant/object-aware Drive shell route contract.
    DriveShellRoute,
}

impl DriveContractGateKind {
    /// Returns the stable gate label used by docs and static verification.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BuiltInDriveDomain => "built-in-drive-domain",
            Self::AclAuthorization => "acl-authorization",
            Self::KmsShred => "kms-shred",
            Self::AuditEmission => "audit-emission",
            Self::SearchContract => "search-contract",
            Self::VersionHistory => "version-history",
            Self::TrashLifecycle => "trash-lifecycle",
            Self::Quota => "quota",
            Self::ApiEventContract => "api-event-contract",
            Self::DriveShellRoute => "drive-shell-route",
        }
    }
}

/// One launch-blocking Drive/contracts lane gate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DriveContractGate {
    kind: DriveContractGateKind,
    evidence: &'static str,
    tenant_scoped: bool,
    launch_blocking: bool,
}

impl DriveContractGate {
    /// Creates a static Drive contract gate.
    #[must_use]
    pub const fn new(
        kind: DriveContractGateKind,
        evidence: &'static str,
        tenant_scoped: bool,
        launch_blocking: bool,
    ) -> Self {
        Self {
            kind,
            evidence,
            tenant_scoped,
            launch_blocking,
        }
    }

    /// Returns the capability covered by this gate.
    #[must_use]
    pub const fn kind(self) -> DriveContractGateKind {
        self.kind
    }

    /// Returns the current evidence anchor for this gate.
    #[must_use]
    pub const fn evidence(self) -> &'static str {
        self.evidence
    }

    /// Returns true when the gate must carry tenant scope.
    #[must_use]
    pub const fn is_tenant_scoped(self) -> bool {
        self.tenant_scoped
    }

    /// Returns true when missing evidence blocks launch/parity claims.
    #[must_use]
    pub const fn is_launch_blocking(self) -> bool {
        self.launch_blocking
    }
}

const G080_DRIVE_CONTRACT_GATES: [DriveContractGate; 10] = [
    DriveContractGate::new(
        DriveContractGateKind::BuiltInDriveDomain,
        "DriveObjectMetadata, DriveObjectDescriptor, DriveObjectBinding",
        true,
        true,
    ),
    DriveContractGate::new(
        DriveContractGateKind::AclAuthorization,
        "DriveSecurityPolicy::authorize with DriveAcl and AclRole",
        true,
        true,
    ),
    DriveContractGate::new(
        DriveContractGateKind::KmsShred,
        "KmsKeyState::Shredded denies content actions",
        true,
        true,
    ),
    DriveContractGate::new(
        DriveContractGateKind::AuditEmission,
        "DriveSecurityDecision always carries AuditEvent",
        true,
        true,
    ),
    DriveContractGate::new(
        DriveContractGateKind::SearchContract,
        "DriveSearchObjectsRequest and search/index slice map",
        true,
        true,
    ),
    DriveContractGate::new(
        DriveContractGateKind::VersionHistory,
        "DriveVersionPointer requires object id and monotonic version number",
        true,
        true,
    ),
    DriveContractGate::new(
        DriveContractGateKind::TrashLifecycle,
        "DriveLifecycleState::Trashed is hidden from default listings",
        true,
        true,
    ),
    DriveContractGate::new(
        DriveContractGateKind::Quota,
        "DriveQuotaImpact carries tenant id, object id, and bytes",
        true,
        true,
    ),
    DriveContractGate::new(
        DriveContractGateKind::ApiEventContract,
        "DriveEventEnvelope carries schema, tenant, object, data class, and sequence",
        true,
        true,
    ),
    DriveContractGate::new(
        DriveContractGateKind::DriveShellRoute,
        "G080 Drive shell route contract is tenant/object-aware and SSR-only",
        true,
        true,
    ),
];

/// Returns the G080 Drive/contracts lane gates.
#[must_use]
pub const fn g080_drive_contract_gates() -> [DriveContractGate; 10] {
    G080_DRIVE_CONTRACT_GATES
}

/// Drive security baseline validation error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveSecurityError {
    message: String,
}

impl DriveSecurityError {
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

impl core::fmt::Display for DriveSecurityError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.message())
    }
}

impl std::error::Error for DriveSecurityError {}

/// Drive object category used for ACL, privacy, and future format-routing policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DriveObjectKind {
    /// Folder/library container.
    Folder,
    /// Docs document object.
    Document,
    /// Sheets workbook object.
    Spreadsheet,
    /// Slides presentation object.
    Presentation,
    /// Generic binary or uploaded file.
    Binary,
}

/// Redacted KMS key reference. The raw value is intentionally not exposed in
/// public metadata views.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KmsKeyRef(String);

impl KmsKeyRef {
    /// Creates a stable KMS key reference identifier.
    pub fn new(value: impl Into<String>) -> Result<Self, DriveSecurityError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(DriveSecurityError::new(
                "kms key reference must not be empty",
            ));
        }
        if trimmed.len() > 128 {
            return Err(DriveSecurityError::new("kms key reference is too long"));
        }
        if !trimmed.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        }) {
            return Err(DriveSecurityError::new(
                "kms key reference contains an invalid character",
            ));
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// Returns the internal key reference for privileged storage/KMS adapters.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// KMS state attached to a Drive object.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KmsKeyState {
    /// Content can be decrypted by authorized paths.
    Active,
    /// Deletion is scheduled but not yet cryptographically final.
    PendingDeletion,
    /// Key material has been shredded; content access must be denied.
    Shredded,
}

impl KmsKeyState {
    /// Returns true when content operations may use the key.
    #[must_use]
    pub const fn allows_content_access(self) -> bool {
        matches!(self, Self::Active)
    }
}

/// Tenant-owned Drive object metadata. Storage pointers and key references are
/// private implementation details and must not appear in public API views.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveObjectMetadata {
    tenant_id: TenantId,
    object_id: ObjectId,
    owner_id: PrincipalId,
    kind: DriveObjectKind,
    data_class: DataClass,
    kms_key_ref: KmsKeyRef,
    kms_key_state: KmsKeyState,
    storage_pointer: String,
}

impl DriveObjectMetadata {
    /// Creates object metadata with tenant, owner, data classification, KMS, and storage state.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        tenant_id: TenantId,
        object_id: ObjectId,
        owner_id: PrincipalId,
        kind: DriveObjectKind,
        data_class: DataClass,
        kms_key_ref: KmsKeyRef,
        kms_key_state: KmsKeyState,
        storage_pointer: String,
    ) -> Self {
        Self {
            tenant_id,
            object_id,
            owner_id,
            kind,
            data_class,
            kms_key_ref,
            kms_key_state,
            storage_pointer,
        }
    }

    /// Returns the owning tenant.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns the Drive object identifier.
    #[must_use]
    pub const fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    /// Returns the object owner.
    #[must_use]
    pub const fn owner_id(&self) -> &PrincipalId {
        &self.owner_id
    }

    /// Returns the object kind.
    #[must_use]
    pub const fn kind(&self) -> DriveObjectKind {
        self.kind
    }

    /// Returns the data classification.
    #[must_use]
    pub const fn data_class(&self) -> DataClass {
        self.data_class
    }

    /// Returns KMS key state without exposing key material or key reference.
    #[must_use]
    pub const fn kms_key_state(&self) -> KmsKeyState {
        self.kms_key_state
    }

    /// Updates KMS state for shred/lifecycle tests and future workers.
    pub fn set_kms_key_state(&mut self, kms_key_state: KmsKeyState) {
        self.kms_key_state = kms_key_state;
    }

    /// Returns true when the object storage pointer is present for privileged storage adapters.
    #[must_use]
    pub fn has_storage_pointer(&self) -> bool {
        !self.storage_pointer.is_empty()
    }

    /// Returns true when the object has a non-empty internal KMS reference.
    #[must_use]
    pub fn has_kms_key_ref(&self) -> bool {
        !self.kms_key_ref.as_str().is_empty()
    }

    /// Returns a public metadata view that omits storage pointer and KMS reference.
    #[must_use]
    pub fn public_view(&self) -> DriveObjectPublicView {
        DriveObjectPublicView {
            object_id: self.object_id.clone(),
            kind: self.kind,
            data_class: self.data_class,
            kms_key_state: self.kms_key_state,
        }
    }

    /// Returns a Drive binding for Docs, Sheets, Slides, and launch-point contracts.
    #[must_use]
    pub fn binding(&self) -> DriveObjectBinding {
        DriveObjectBinding::new(
            self.tenant_id.clone(),
            self.object_id.clone(),
            self.kind,
            self.data_class,
        )
    }
}

/// Public metadata view safe for APIs and audit-adjacent logs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveObjectPublicView {
    object_id: ObjectId,
    kind: DriveObjectKind,
    data_class: DataClass,
    kms_key_state: KmsKeyState,
}

impl DriveObjectPublicView {
    /// Returns the object id.
    #[must_use]
    pub const fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    /// Returns the object kind.
    #[must_use]
    pub const fn kind(&self) -> DriveObjectKind {
        self.kind
    }

    /// Returns the data classification.
    #[must_use]
    pub const fn data_class(&self) -> DataClass {
        self.data_class
    }

    /// Returns KMS state without exposing the KMS key reference.
    #[must_use]
    pub const fn kms_key_state(&self) -> KmsKeyState {
        self.kms_key_state
    }

    /// Returns true when content access can use an active KMS key.
    #[must_use]
    pub const fn has_active_kms_key(&self) -> bool {
        self.kms_key_state.allows_content_access()
    }
}

/// ACL entry binding a principal to a Drive role.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AclEntry {
    principal_id: PrincipalId,
    role: AclRole,
}

impl AclEntry {
    /// Creates an ACL entry.
    #[must_use]
    pub const fn new(principal_id: PrincipalId, role: AclRole) -> Self {
        Self { principal_id, role }
    }

    /// Creates an owner ACL entry.
    #[must_use]
    pub const fn owner(principal_id: PrincipalId) -> Self {
        Self::new(principal_id, AclRole::Owner)
    }

    /// Creates a viewer ACL entry.
    #[must_use]
    pub const fn viewer(principal_id: PrincipalId) -> Self {
        Self::new(principal_id, AclRole::Viewer)
    }

    /// Creates an editor ACL entry.
    #[must_use]
    pub const fn editor(principal_id: PrincipalId) -> Self {
        Self::new(principal_id, AclRole::Editor)
    }

    /// Returns the ACL principal.
    #[must_use]
    pub const fn principal_id(&self) -> &PrincipalId {
        &self.principal_id
    }

    /// Returns the ACL role.
    #[must_use]
    pub const fn role(&self) -> AclRole {
        self.role
    }
}

/// Object ACL. Every ACL must include an owner entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveAcl {
    tenant_id: TenantId,
    object_id: ObjectId,
    entries: Vec<AclEntry>,
}

impl DriveAcl {
    /// Creates a Drive ACL and verifies that it has an owner.
    pub fn new(
        tenant_id: TenantId,
        object_id: ObjectId,
        entries: Vec<AclEntry>,
    ) -> Result<Self, DriveSecurityError> {
        if entries.is_empty() {
            return Err(DriveSecurityError::new(
                "drive acl requires at least one entry",
            ));
        }
        if !entries.iter().any(|entry| entry.role() == AclRole::Owner) {
            return Err(DriveSecurityError::new("drive acl requires an owner entry"));
        }
        Ok(Self {
            tenant_id,
            object_id,
            entries,
        })
    }

    /// Returns the ACL tenant.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns the ACL object id.
    #[must_use]
    pub const fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    /// Returns ACL entries.
    #[must_use]
    pub fn entries(&self) -> &[AclEntry] {
        self.entries.as_slice()
    }

    /// Returns the highest listed role for a principal in insertion order.
    #[must_use]
    pub fn role_for(&self, principal_id: &PrincipalId) -> Option<AclRole> {
        self.entries
            .iter()
            .find(|entry| entry.principal_id() == principal_id)
            .map(AclEntry::role)
    }

    /// Returns a new ACL with a share grant applied.
    pub fn with_share_grant(&self, grant: DriveShareGrant) -> Result<Self, DriveSecurityError> {
        let mut entries = self.entries.clone();
        entries.retain(|entry| entry.principal_id() != grant.principal_id());
        entries.push(AclEntry::new(grant.principal_id().clone(), grant.role()));
        Self::new(self.tenant_id.clone(), self.object_id.clone(), entries)
    }

    /// Returns a new ACL with a non-owner principal revoked.
    pub fn revoke_share(
        &self,
        revocation: &DriveShareRevocation,
    ) -> Result<Self, DriveSecurityError> {
        let revoked_role = self.role_for(revocation.principal_id());
        if revoked_role == Some(AclRole::Owner) {
            return Err(DriveSecurityError::new(
                "drive acl cannot revoke owner through share revocation",
            ));
        }
        let mut entries = self.entries.clone();
        entries.retain(|entry| entry.principal_id() != revocation.principal_id());
        Self::new(self.tenant_id.clone(), self.object_id.clone(), entries)
    }
}

/// Drive library identifier. Libraries are top-level tenant-owned collections.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DriveLibraryId(ObjectId);

impl DriveLibraryId {
    /// Creates a library id from an object id.
    #[must_use]
    pub const fn from_object_id(object_id: ObjectId) -> Self {
        Self(object_id)
    }

    /// Returns the underlying object id.
    #[must_use]
    pub const fn as_object_id(&self) -> &ObjectId {
        &self.0
    }
}

/// Drive folder identifier. Folder IDs are object IDs with folder semantics.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DriveFolderId(ObjectId);

impl DriveFolderId {
    /// Creates a folder id from an object id.
    #[must_use]
    pub const fn from_object_id(object_id: ObjectId) -> Self {
        Self(object_id)
    }

    /// Returns the underlying object id.
    #[must_use]
    pub const fn as_object_id(&self) -> &ObjectId {
        &self.0
    }
}

/// Tenant-private Drive path. Paths are request/body metadata, never public URL/query PII.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DrivePath(String);

impl DrivePath {
    /// Creates a validated absolute Drive path.
    pub fn new(value: impl Into<String>) -> Result<Self, DriveSecurityError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(DriveSecurityError::new("drive path must not be empty"));
        }
        if !trimmed.starts_with('/') {
            return Err(DriveSecurityError::new("drive path must be absolute"));
        }
        if trimmed.len() > 1024 {
            return Err(DriveSecurityError::new("drive path is too long"));
        }
        if trimmed.contains('\n') || trimmed.contains('\r') || trimmed.contains('\0') {
            return Err(DriveSecurityError::new(
                "drive path contains an invalid control character",
            ));
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// Returns the path string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Object descriptor used by Drive metadata, format, search, and API lanes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveObjectDescriptor {
    library_id: DriveLibraryId,
    folder_id: DriveFolderId,
    path: DrivePath,
    name: String,
    mime_type: String,
    size_bytes: u64,
    schema_version: u32,
}

impl DriveObjectDescriptor {
    /// Creates a Drive object descriptor.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        library_id: DriveLibraryId,
        folder_id: DriveFolderId,
        path: DrivePath,
        name: impl Into<String>,
        mime_type: impl Into<String>,
        size_bytes: u64,
        schema_version: u32,
    ) -> Result<Self, DriveSecurityError> {
        let name = name.into();
        let mime_type = mime_type.into();
        if name.trim().is_empty() {
            return Err(DriveSecurityError::new(
                "drive object name must not be empty",
            ));
        }
        if mime_type.trim().is_empty() {
            return Err(DriveSecurityError::new(
                "drive object mime type must not be empty",
            ));
        }
        if schema_version == 0 {
            return Err(DriveSecurityError::new(
                "drive object schema version must be at least 1",
            ));
        }
        Ok(Self {
            library_id,
            folder_id,
            path,
            name: name.trim().to_owned(),
            mime_type: mime_type.trim().to_owned(),
            size_bytes,
            schema_version,
        })
    }

    /// Returns the library id.
    #[must_use]
    pub const fn library_id(&self) -> &DriveLibraryId {
        &self.library_id
    }

    /// Returns the folder id.
    #[must_use]
    pub const fn folder_id(&self) -> &DriveFolderId {
        &self.folder_id
    }

    /// Returns the tenant-private path.
    #[must_use]
    pub const fn path(&self) -> &DrivePath {
        &self.path
    }

    /// Returns the display name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the MIME type.
    #[must_use]
    pub fn mime_type(&self) -> &str {
        self.mime_type.as_str()
    }

    /// Returns object size in bytes.
    #[must_use]
    pub const fn size_bytes(&self) -> u64 {
        self.size_bytes
    }

    /// Returns metadata schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }
}

/// Drive share grant contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveShareGrant {
    principal_id: PrincipalId,
    role: AclRole,
    granted_by: PrincipalId,
    granted_at_epoch_seconds: u64,
}

impl DriveShareGrant {
    /// Creates a share grant. Owner grants are intentionally excluded from this path.
    pub fn new(
        principal_id: PrincipalId,
        role: AclRole,
        granted_by: PrincipalId,
        granted_at_epoch_seconds: u64,
    ) -> Result<Self, DriveSecurityError> {
        if role == AclRole::Owner {
            return Err(DriveSecurityError::new(
                "drive share grants cannot create owner role",
            ));
        }
        Ok(Self {
            principal_id,
            role,
            granted_by,
            granted_at_epoch_seconds,
        })
    }

    /// Returns the shared principal.
    #[must_use]
    pub const fn principal_id(&self) -> &PrincipalId {
        &self.principal_id
    }

    /// Returns granted role.
    #[must_use]
    pub const fn role(&self) -> AclRole {
        self.role
    }

    /// Returns granting principal.
    #[must_use]
    pub const fn granted_by(&self) -> &PrincipalId {
        &self.granted_by
    }

    /// Returns grant time.
    #[must_use]
    pub const fn granted_at_epoch_seconds(&self) -> u64 {
        self.granted_at_epoch_seconds
    }
}

/// Drive share revocation contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveShareRevocation {
    principal_id: PrincipalId,
    revoked_by: PrincipalId,
    revoked_at_epoch_seconds: u64,
}

impl DriveShareRevocation {
    /// Creates a share revocation.
    pub fn new(
        principal_id: PrincipalId,
        revoked_by: PrincipalId,
        revoked_at_epoch_seconds: u64,
    ) -> Result<Self, DriveSecurityError> {
        if principal_id == revoked_by {
            return Err(DriveSecurityError::new(
                "drive share revocation must be requested by a distinct principal",
            ));
        }
        Ok(Self {
            principal_id,
            revoked_by,
            revoked_at_epoch_seconds,
        })
    }

    /// Returns revoked principal.
    #[must_use]
    pub const fn principal_id(&self) -> &PrincipalId {
        &self.principal_id
    }

    /// Returns revoking principal.
    #[must_use]
    pub const fn revoked_by(&self) -> &PrincipalId {
        &self.revoked_by
    }

    /// Returns revocation time.
    #[must_use]
    pub const fn revoked_at_epoch_seconds(&self) -> u64 {
        self.revoked_at_epoch_seconds
    }
}

/// KMS-shred binding attached to object content and metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KmsShredBinding {
    object_id: ObjectId,
    key_ref: KmsKeyRef,
    key_state: KmsKeyState,
    data_class: DataClass,
}

impl KmsShredBinding {
    /// Creates a KMS-shred binding.
    #[must_use]
    pub const fn new(
        object_id: ObjectId,
        key_ref: KmsKeyRef,
        key_state: KmsKeyState,
        data_class: DataClass,
    ) -> Self {
        Self {
            object_id,
            key_ref,
            key_state,
            data_class,
        }
    }

    /// Returns object id.
    #[must_use]
    pub const fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    /// Returns internal key reference for privileged KMS adapters.
    #[must_use]
    pub const fn key_ref(&self) -> &KmsKeyRef {
        &self.key_ref
    }

    /// Returns key state.
    #[must_use]
    pub const fn key_state(&self) -> KmsKeyState {
        self.key_state
    }

    /// Returns data class.
    #[must_use]
    pub const fn data_class(&self) -> DataClass {
        self.data_class
    }

    /// Returns true when content access is allowed.
    #[must_use]
    pub const fn allows_content_access(&self) -> bool {
        self.key_state.allows_content_access()
    }
}

/// Drive object lifecycle state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DriveLifecycleState {
    /// Object is active and visible in normal Drive listings.
    Active,
    /// Object is trashed and hidden from default listings.
    Trashed,
    /// Object is held for compliance and cannot be purged.
    RetentionHold,
    /// Object has been purged after KMS-shred/lifecycle completion.
    Purged,
}

impl DriveLifecycleState {
    /// Returns true when the state should appear in default object listings.
    #[must_use]
    pub const fn is_visible_to_default_lists(self) -> bool {
        matches!(self, Self::Active)
    }
}

/// Pointer to a Drive object version and backing storage revision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveVersionPointer {
    object_id: ObjectId,
    version_number: u64,
    storage_revision: String,
}

impl DriveVersionPointer {
    /// Creates a version pointer.
    pub fn new(
        object_id: ObjectId,
        version_number: u64,
        storage_revision: impl Into<String>,
    ) -> Result<Self, DriveSecurityError> {
        let storage_revision = storage_revision.into();
        if version_number == 0 {
            return Err(DriveSecurityError::new(
                "drive version number must be at least 1",
            ));
        }
        if storage_revision.trim().is_empty() {
            return Err(DriveSecurityError::new(
                "drive storage revision must not be empty",
            ));
        }
        Ok(Self {
            object_id,
            version_number,
            storage_revision: storage_revision.trim().to_owned(),
        })
    }

    /// Returns object id.
    #[must_use]
    pub const fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    /// Returns version number.
    #[must_use]
    pub const fn version_number(&self) -> u64 {
        self.version_number
    }

    /// Returns storage revision.
    #[must_use]
    pub fn storage_revision(&self) -> &str {
        self.storage_revision.as_str()
    }
}

/// Drive quota impact contract for writes, uploads, imports, and versioning.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveQuotaImpact {
    tenant_id: TenantId,
    object_id: ObjectId,
    size_bytes: u64,
}

impl DriveQuotaImpact {
    /// Creates quota impact for a Drive object.
    #[must_use]
    pub const fn new(tenant_id: TenantId, object_id: ObjectId, size_bytes: u64) -> Self {
        Self {
            tenant_id,
            object_id,
            size_bytes,
        }
    }

    /// Returns tenant id.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns object id.
    #[must_use]
    pub const fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    /// Returns size impact.
    #[must_use]
    pub const fn size_bytes(&self) -> u64 {
        self.size_bytes
    }
}

/// Preview/render contract for Drive objects and Office file previews.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrivePreviewContract {
    object_id: ObjectId,
    data_class: DataClass,
    rendition_mime_type: String,
    stale_after_seconds: u64,
}

impl DrivePreviewContract {
    /// Creates a preview contract.
    pub fn new(
        object_id: ObjectId,
        data_class: DataClass,
        rendition_mime_type: impl Into<String>,
        stale_after_seconds: u64,
    ) -> Result<Self, DriveSecurityError> {
        let rendition_mime_type = rendition_mime_type.into();
        if rendition_mime_type.trim().is_empty() {
            return Err(DriveSecurityError::new(
                "drive preview rendition mime type must not be empty",
            ));
        }
        if stale_after_seconds == 0 {
            return Err(DriveSecurityError::new(
                "drive preview stale-after seconds must be greater than zero",
            ));
        }
        Ok(Self {
            object_id,
            data_class,
            rendition_mime_type: rendition_mime_type.trim().to_owned(),
            stale_after_seconds,
        })
    }

    /// Returns object id.
    #[must_use]
    pub const fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    /// Returns data class.
    #[must_use]
    pub const fn data_class(&self) -> DataClass {
        self.data_class
    }

    /// Returns rendition MIME type.
    #[must_use]
    pub fn rendition_mime_type(&self) -> &str {
        self.rendition_mime_type.as_str()
    }

    /// Returns stale-after TTL in seconds.
    #[must_use]
    pub const fn stale_after_seconds(&self) -> u64 {
        self.stale_after_seconds
    }
}

/// Minimal Drive object binding shared by Docs, Sheets, and Slides domain crates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveObjectBinding {
    tenant_id: TenantId,
    object_id: ObjectId,
    kind: DriveObjectKind,
    data_class: DataClass,
}

impl DriveObjectBinding {
    /// Creates a Drive object binding from validated kernel IDs.
    #[must_use]
    pub const fn new(
        tenant_id: TenantId,
        object_id: ObjectId,
        kind: DriveObjectKind,
        data_class: DataClass,
    ) -> Self {
        Self {
            tenant_id,
            object_id,
            kind,
            data_class,
        }
    }

    /// Returns the tenant that owns the bound Drive object.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns the bound Drive object id.
    #[must_use]
    pub const fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    /// Returns the bound Drive object kind.
    #[must_use]
    pub const fn kind(&self) -> DriveObjectKind {
        self.kind
    }

    /// Returns the bound Drive object data class.
    #[must_use]
    pub const fn data_class(&self) -> DataClass {
        self.data_class
    }
}

/// Authorization decision paired with its mandatory audit event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveSecurityDecision {
    authorization: AuthorizationDecision,
    audit_event: AuditEvent,
}

impl DriveSecurityDecision {
    fn new(authorization: AuthorizationDecision, audit_event: AuditEvent) -> Self {
        Self {
            authorization,
            audit_event,
        }
    }

    /// Returns the authorization decision.
    #[must_use]
    pub const fn authorization(&self) -> &AuthorizationDecision {
        &self.authorization
    }

    /// Returns the audit event emitted for the decision.
    #[must_use]
    pub const fn audit_event(&self) -> &AuditEvent {
        &self.audit_event
    }
}

/// Pure Drive security policy for tenant isolation, ACL, KMS-shred, and audit emission.
pub struct DriveSecurityPolicy;

impl DriveSecurityPolicy {
    /// Authorizes a Drive action and always returns an audit-bound decision.
    #[must_use]
    pub fn authorize(
        context: &RequestContext,
        object: &DriveObjectMetadata,
        acl: &DriveAcl,
        action: DriveAction,
        event_id: RequestId,
    ) -> DriveSecurityDecision {
        let request = AuthorizationRequest::new(
            context.clone(),
            ResourceRef::drive_object(
                object.tenant_id().clone(),
                object.object_id().clone(),
                object.data_class(),
            ),
            action,
        );

        let authorization = if context.tenant_id() != object.tenant_id() {
            AuthorizationDecision::deny(
                "drive-tenant-isolation",
                "cross-tenant drive access denied",
            )
        } else if acl.tenant_id() != object.tenant_id() || acl.object_id() != object.object_id() {
            AuthorizationDecision::deny("drive-acl-integrity", "drive acl does not match object")
        } else if Self::content_action_requires_active_key(action)
            && !object.kms_key_state().allows_content_access()
        {
            AuthorizationDecision::deny("drive-kms-shred", "drive object kms key is shredded")
        } else if let Some(role) = acl.role_for(context.principal_id()) {
            if role.allows_drive_action(action) {
                AuthorizationDecision::allow("drive-acl")
            } else {
                AuthorizationDecision::deny(
                    "drive-acl",
                    "principal is not permitted to perform drive action",
                )
            }
        } else {
            AuthorizationDecision::deny("drive-acl", "principal has no drive acl entry")
        };

        let audit_event = authorization.to_audit_event(&request, event_id);
        DriveSecurityDecision::new(authorization, audit_event)
    }

    const fn content_action_requires_active_key(action: DriveAction) -> bool {
        matches!(
            action,
            DriveAction::Read | DriveAction::Write | DriveAction::Export | DriveAction::Delete
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{ARCHITECTURE_LAYER, CRATE_NAME, VERTICAL_SLICE};

    #[test]
    fn scaffold_identity_is_declared() {
        assert!(!ARCHITECTURE_LAYER.is_empty());
        assert!(!CRATE_NAME.is_empty());
        assert!(!VERTICAL_SLICE.is_empty());
    }
}

#[cfg(test)]
mod drive_security_baseline_tests {
    use oya_office_authz_domain::DriveAction;
    use oya_office_kernel::{
        AuditAction, AuditOutcome, CellId, DataClass, ObjectId, PrincipalId, RequestContext,
        RequestId, TenantId,
    };

    use super::{
        AclEntry, DriveAcl, DriveObjectKind, DriveObjectMetadata, DriveSecurityPolicy, KmsKeyRef,
        KmsKeyState,
    };

    fn context(tenant: &str, principal: &str) -> RequestContext {
        RequestContext::new(
            RequestId::new("req-drive-security").expect("valid request id"),
            TenantId::new(tenant).expect("valid tenant id"),
            PrincipalId::new(principal).expect("valid principal id"),
            CellId::new("iad-1").expect("valid cell id"),
        )
    }

    fn metadata() -> DriveObjectMetadata {
        DriveObjectMetadata::new(
            TenantId::new("tenant-alpha").expect("valid tenant id"),
            ObjectId::new("drive-object-1").expect("valid object id"),
            PrincipalId::new("owner-1").expect("valid principal id"),
            DriveObjectKind::Document,
            DataClass::Confidential,
            KmsKeyRef::new("kms-tenant-alpha-drive-object-1").expect("valid kms key"),
            KmsKeyState::Active,
            "s3://tenant-alpha/private/drive-object-1".to_owned(),
        )
    }

    #[test]
    fn cross_tenant_access_is_denied_and_audited() {
        let object = metadata();
        let acl = DriveAcl::new(
            object.tenant_id().clone(),
            object.object_id().clone(),
            vec![AclEntry::owner(object.owner_id().clone())],
        )
        .expect("valid acl");

        let decision = DriveSecurityPolicy::authorize(
            &context("tenant-beta", "owner-1"),
            &object,
            &acl,
            DriveAction::Read,
            RequestId::new("evt-cross-tenant").expect("valid event id"),
        );

        assert!(decision.authorization().is_denied());
        assert_eq!(decision.audit_event().outcome(), AuditOutcome::Denied);
        assert_eq!(decision.audit_event().tenant_id().as_str(), "tenant-beta");
    }

    #[test]
    fn drive_authorization_allows_owner_and_emits_complete_audit_event() {
        let object = metadata();
        let acl = DriveAcl::new(
            object.tenant_id().clone(),
            object.object_id().clone(),
            vec![AclEntry::owner(object.owner_id().clone())],
        )
        .expect("valid acl");

        let decision = DriveSecurityPolicy::authorize(
            &context("tenant-alpha", "owner-1"),
            &object,
            &acl,
            DriveAction::Write,
            RequestId::new("evt-owner-write").expect("valid event id"),
        );

        assert!(decision.authorization().is_allowed());
        assert_eq!(decision.authorization().policy_id(), "drive-acl");
        assert_eq!(decision.authorization().reason(), None);
        assert_eq!(
            decision.audit_event().event_id().as_str(),
            "evt-owner-write"
        );
        assert_eq!(
            decision.audit_event().request_id().as_str(),
            "req-drive-security"
        );
        assert_eq!(decision.audit_event().tenant_id().as_str(), "tenant-alpha");
        assert_eq!(decision.audit_event().actor().as_str(), "owner-1");
        assert_eq!(decision.audit_event().action(), AuditAction::DriveWrite);
        assert_eq!(
            decision
                .audit_event()
                .resource()
                .expect("drive object resource")
                .as_str(),
            "drive-object-1"
        );
        assert_eq!(decision.audit_event().data_class(), DataClass::Confidential);
        assert_eq!(decision.audit_event().outcome(), AuditOutcome::Allowed);
        assert_eq!(decision.audit_event().reason(), None);
    }

    #[test]
    fn shredded_kms_key_blocks_content_access_even_for_owner() {
        let mut object = metadata();
        object.set_kms_key_state(KmsKeyState::Shredded);
        let acl = DriveAcl::new(
            object.tenant_id().clone(),
            object.object_id().clone(),
            vec![AclEntry::owner(object.owner_id().clone())],
        )
        .expect("valid acl");

        let decision = DriveSecurityPolicy::authorize(
            &context("tenant-alpha", "owner-1"),
            &object,
            &acl,
            DriveAction::Export,
            RequestId::new("evt-kms-shred").expect("valid event id"),
        );

        assert!(decision.authorization().is_denied());
        assert_eq!(
            decision.authorization().reason(),
            Some("drive object kms key is shredded")
        );
    }

    #[test]
    fn object_metadata_public_view_does_not_expose_storage_pointer_or_kms_ref() {
        let object = metadata();
        let view = object.public_view();
        let debug_view = format!("{view:?}");

        assert_eq!(view.object_id().as_str(), "drive-object-1");
        assert_eq!(view.data_class(), DataClass::Confidential);
        assert!(view.has_active_kms_key());
        assert!(!debug_view.contains("s3://"));
        assert!(!debug_view.contains("kms-tenant-alpha"));
        assert!(object.has_storage_pointer());
        assert!(object.has_kms_key_ref());
    }
}

#[cfg(test)]
mod drive_object_contract_tests {
    use oya_office_authz_domain::AclRole;
    use oya_office_kernel::{DataClass, ObjectId, PrincipalId, TenantId};

    use super::{
        DriveAcl, DriveContractGateKind, DriveFolderId, DriveLibraryId, DriveLifecycleState,
        DriveObjectDescriptor, DrivePath, DrivePreviewContract, DriveQuotaImpact, DriveShareGrant,
        DriveShareRevocation, DriveVersionPointer, G080_DRIVE_CONTRACT_LANE_VERSION, KmsKeyRef,
        KmsKeyState, KmsShredBinding, g080_drive_contract_gates,
    };

    #[test]
    fn object_descriptor_requires_folder_library_and_safe_body_path() {
        let descriptor = DriveObjectDescriptor::new(
            DriveLibraryId::from_object_id(ObjectId::new("library-1").expect("valid library id")),
            DriveFolderId::from_object_id(ObjectId::new("folder-1").expect("valid folder id")),
            DrivePath::new("/Workplace/e-sign-queue/example.docx").expect("valid path"),
            "example.docx",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            42,
            1,
        )
        .expect("valid descriptor");

        assert_eq!(descriptor.folder_id().as_object_id().as_str(), "folder-1");
        assert_eq!(descriptor.schema_version(), 1);
        assert!(DrivePath::new("relative/path").is_err());
    }

    #[test]
    fn share_grants_can_be_revoked_without_removing_last_owner() {
        let owner = PrincipalId::new("owner-1").expect("valid owner");
        let viewer = PrincipalId::new("viewer-1").expect("valid viewer");
        let object_id = ObjectId::new("object-1").expect("valid object id");
        let acl = DriveAcl::new(
            TenantId::new("tenant-alpha").expect("valid tenant id"),
            object_id.clone(),
            vec![super::AclEntry::owner(owner.clone())],
        )
        .expect("valid acl");

        let grant = DriveShareGrant::new(viewer.clone(), AclRole::Viewer, owner.clone(), 10)
            .expect("valid grant");
        let updated = acl.with_share_grant(grant).expect("grant applied");
        assert_eq!(updated.role_for(&viewer), Some(AclRole::Viewer));

        let revocation = DriveShareRevocation::new(viewer.clone(), owner, 11).expect("revocation");
        let revoked = updated.revoke_share(&revocation).expect("revoked");
        assert_eq!(revoked.role_for(&viewer), None);
    }

    #[test]
    fn kms_version_lifecycle_quota_contracts_are_tenant_scoped() {
        let object_id = ObjectId::new("object-1").expect("valid object id");
        let kms = KmsShredBinding::new(
            object_id.clone(),
            KmsKeyRef::new("kms-tenant-alpha-object-1").expect("valid kms key"),
            KmsKeyState::Active,
            DataClass::Confidential,
        );
        assert!(kms.allows_content_access());

        let version =
            DriveVersionPointer::new(object_id.clone(), 3, "storage-rev-3").expect("valid version");
        assert_eq!(version.version_number(), 3);

        assert!(!DriveLifecycleState::Trashed.is_visible_to_default_lists());

        let quota = DriveQuotaImpact::new(
            TenantId::new("tenant-alpha").expect("valid tenant id"),
            object_id,
            2048,
        );
        assert_eq!(quota.size_bytes(), 2048);

        let preview = DrivePreviewContract::new(
            ObjectId::new("object-2").expect("valid object id"),
            DataClass::Internal,
            "image/png",
            300,
        )
        .expect("valid preview");
        assert_eq!(preview.rendition_mime_type(), "image/png");
    }

    #[test]
    fn g080_drive_contract_lane_covers_required_security_and_product_gates() {
        let gates = g080_drive_contract_gates();

        assert_eq!(G080_DRIVE_CONTRACT_LANE_VERSION, "g080-drive-contracts-v1");
        assert_eq!(gates.len(), 10);
        assert!(gates.iter().all(|gate| gate.is_tenant_scoped()));
        assert!(gates.iter().all(|gate| gate.is_launch_blocking()));
        assert!(
            gates
                .iter()
                .all(|gate| !gate.evidence().is_empty() && !gate.kind().as_str().is_empty())
        );
        assert!(
            gates
                .iter()
                .any(|gate| gate.kind() == DriveContractGateKind::BuiltInDriveDomain)
        );
        assert!(gates.iter().any(
            |gate| gate.kind() == DriveContractGateKind::AclAuthorization
                && gate.evidence().contains("DriveSecurityPolicy::authorize")
        ));
        assert!(
            gates
                .iter()
                .any(|gate| gate.kind() == DriveContractGateKind::KmsShred
                    && gate.evidence().contains("KmsKeyState::Shredded"))
        );
        assert!(
            gates
                .iter()
                .any(|gate| gate.kind() == DriveContractGateKind::AuditEmission)
        );
        assert!(
            gates
                .iter()
                .any(|gate| gate.kind() == DriveContractGateKind::SearchContract)
        );
        assert!(
            gates
                .iter()
                .any(|gate| gate.kind() == DriveContractGateKind::VersionHistory)
        );
        assert!(
            gates
                .iter()
                .any(|gate| gate.kind() == DriveContractGateKind::TrashLifecycle)
        );
        assert!(
            gates
                .iter()
                .any(|gate| gate.kind() == DriveContractGateKind::Quota)
        );
        assert!(
            gates
                .iter()
                .any(|gate| gate.kind() == DriveContractGateKind::ApiEventContract)
        );
        assert!(
            gates
                .iter()
                .any(|gate| gate.kind() == DriveContractGateKind::DriveShellRoute)
        );
    }
}
