//! Cloud IAM kernel.
//!
//! Cloud IAM is the cloud-customer-facing IAM/SAML/OIDC/STs contract. It keeps
//! role, federation, and STS session values typed while delegating short-lived
//! credential issuance to the platform identity kernel so the two surfaces stay
//! in lockstep.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

use cell_region::RegionCode;
use iam_identity_domain::{
    CredentialRequest, CredentialRequestKind, IdentityError, MAX_TOKEN_TTL_SECONDS, Principal,
    issue_credential,
};
use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass, Purpose};

const IAM_PRINCIPAL_SCHEMA_VERSION: u32 = 1;
const IAM_ROLE_SCHEMA_VERSION: u32 = 1;
const STS_SESSION_SCHEMA_VERSION: u32 = 1;
const IDENTITY_PROVIDER_SCHEMA_VERSION: u32 = 1;
const IAM_PROVIDER_IDP_SYNC_SCHEMA_VERSION: u32 = 1;
const IAM_PROVIDER_IDP_REGISTRY_EVIDENCE_SCHEMA_VERSION: u32 = 1;
const TENANT_ID_PREFIX: &str = "ten_";
const USER_ID_PREFIX: &str = "usr_";
const SERVICE_PRINCIPAL_PREFIX: &str = "sp_";
const ROLE_ID_PREFIX: &str = "role_";
const IDENTITY_PROVIDER_PREFIX: &str = "idp_";
const CEDAR_POLICY_PREFIX: &str = "pol_";
const STS_SESSION_PREFIX: &str = "sts_";
const REGIONAL_PACK_PREFIX: &str = "pack-";
const CERT_REF_PREFIX: &str = "cert/";
const JWKS_REF_PREFIX: &str = "jwks/";
const CLOUD_SCOPE_PREFIX: &str = "cloud.";
const CLOUD_IAM_CAPABILITY_ID: &str = "cap.cloud.iam";
const OCI_IDP_EVIDENCE_REF_PREFIX: &str = "oci-iam-idp://";
const SELFHOSTED_IDP_EVIDENCE_REF_PREFIX: &str = "selfhosted-idp://";

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct IamPrincipalId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct IamRoleId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CedarPolicyId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RoleName {
    pub value: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ScopeRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SubjectUri {
    pub value: String, // data_class: PII_QUASI_IDENTIFIER
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct IamIdentityProviderId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct StsSessionId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamPlacementBoundary {
    pub tenant_id: CloudIamBoundaryTenantId, // data_class: INTERNAL_ONLY
    pub cell_id: CloudIamBoundaryCellId,     // data_class: INTERNAL_ONLY
    pub region_id: CloudIamBoundaryRegionId, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CloudIamBoundaryTenantId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CloudIamBoundaryCellId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CloudIamBoundaryRegionId {
    pub value: String, // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum IamPrincipalKind {
    User,
    ServiceAccount,
    Role,
    Federated,
    External,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MfaState {
    NotRequired,
    Required,
    Enrolled,
    Verified,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum IdentityProviderKind {
    Saml,
    Oidc,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CloudIamProviderKind {
    OciIdentityDomain,
    SelfHostedOidcControlPlane,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum IamProviderIdentityProviderOperation {
    Upsert,
    Delete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum IamProviderIdentityProviderSyncStatus {
    Synchronized,
    DeleteSynchronized,
}

impl IdentityProviderKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Saml => "saml",
            Self::Oidc => "oidc",
        }
    }
}

impl CloudIamProviderKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::OciIdentityDomain => "oci_identity_domain",
            Self::SelfHostedOidcControlPlane => "selfhosted_oidc_control_plane",
        }
    }
}

impl IamProviderIdentityProviderOperation {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Upsert => "upsert",
            Self::Delete => "delete",
        }
    }
}

impl IamProviderIdentityProviderSyncStatus {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Synchronized => "synchronized",
            Self::DeleteSynchronized => "delete_synchronized",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityProviderCreate {
    pub id: String,                        // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub region_pack: String,               // data_class: INTERNAL_ONLY
    pub kind: IdentityProviderKind,        // data_class: PUBLIC
    pub issuer_uri: String,                // data_class: INTERNAL_ONLY
    pub audience: String,                  // data_class: INTERNAL_ONLY
    pub verification_material_ref: String, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityProviderUpdate {
    pub id: String,                        // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub region_pack: String,               // data_class: INTERNAL_ONLY
    pub kind: IdentityProviderKind,        // data_class: PUBLIC
    pub issuer_uri: String,                // data_class: INTERNAL_ONLY
    pub audience: String,                  // data_class: INTERNAL_ONLY
    pub verification_material_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityProvider {
    pub id: Classified<IamIdentityProviderId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,         // data_class: INTERNAL_ONLY
    pub region_pack: Classified<String>,       // data_class: INTERNAL_ONLY
    pub kind: Classified<IdentityProviderKind>, // data_class: PUBLIC
    pub issuer_uri: Classified<String>,        // data_class: INTERNAL_ONLY
    pub audience: Classified<String>,          // data_class: INTERNAL_ONLY
    pub verification_material_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,       // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IamProviderIdentityProviderSyncRequest {
    pub request_id: String,                     // data_class: INTERNAL_ONLY
    pub provider_identity_provider_ref: String, // data_class: INTERNAL_ONLY
    pub tenant_id: String,                      // data_class: INTERNAL_ONLY
    pub actor: String,                          // data_class: INTERNAL_ONLY
    pub idempotency_key: String,                // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
    pub operation: IamProviderIdentityProviderOperation, // data_class: PUBLIC
    pub identity_provider: IdentityProvider,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IamProviderIdentityProviderSyncReceipt {
    pub provider: CloudIamProviderKind, // data_class: PUBLIC
    pub operation: IamProviderIdentityProviderOperation, // data_class: PUBLIC
    pub sync_status: IamProviderIdentityProviderSyncStatus, // data_class: PUBLIC
    pub request_id: String,             // data_class: INTERNAL_ONLY
    pub provider_request_id: String,    // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub identity_provider_id: String,   // data_class: INTERNAL_ONLY
    pub identity_provider_kind: IdentityProviderKind, // data_class: PUBLIC
    pub region_pack: String,            // data_class: INTERNAL_ONLY
    pub issuer_ref: String,             // data_class: INTERNAL_ONLY
    pub audience_ref: String,           // data_class: INTERNAL_ONLY
    pub verification_material_ref: String, // data_class: INTERNAL_ONLY
    pub provider_identity_provider_ref: String, // data_class: INTERNAL_ONLY
    pub actor: String,                  // data_class: INTERNAL_ONLY
    pub idempotency_key: String,        // data_class: INTERNAL_ONLY
    pub provider_evidence_ref: String,  // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub schema_version: u32,            // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IdentityProviderRegistryRawMaterialCounters {
    pub raw_provider_document_bytes: u64, // data_class: INTERNAL_ONLY
    pub credential_material_bytes: u64,   // data_class: INTERNAL_ONLY
    pub assertion_material_bytes: u64,    // data_class: INTERNAL_ONLY
    pub sts_material_bytes: u64,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityProviderRegistryRecord {
    pub tenant_id: String,                            // data_class: INTERNAL_ONLY
    pub identity_provider_id: String,                 // data_class: INTERNAL_ONLY
    pub identity_provider_kind: IdentityProviderKind, // data_class: PUBLIC
    pub region_pack: String,                          // data_class: INTERNAL_ONLY
    pub issuer_ref: String,                           // data_class: INTERNAL_ONLY
    pub audience_ref: String,                         // data_class: INTERNAL_ONLY
    pub verification_material_ref: String,            // data_class: INTERNAL_ONLY
    pub provider: CloudIamProviderKind,               // data_class: PUBLIC
    pub operation: IamProviderIdentityProviderOperation, // data_class: PUBLIC
    pub sync_status: IamProviderIdentityProviderSyncStatus, // data_class: PUBLIC
    pub provider_identity_provider_ref: String,       // data_class: INTERNAL_ONLY
    pub provider_request_id: String,                  // data_class: INTERNAL_ONLY
    pub actor: String,                                // data_class: INTERNAL_ONLY
    pub idempotency_key: String,                      // data_class: INTERNAL_ONLY
    pub provider_evidence_ref: String,                // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64,               // data_class: INTERNAL_ONLY
    pub raw_provider_document_bytes: u64,             // data_class: INTERNAL_ONLY
    pub credential_material_bytes: u64,               // data_class: INTERNAL_ONLY
    pub assertion_material_bytes: u64,                // data_class: INTERNAL_ONLY
    pub sts_material_bytes: u64,                      // data_class: INTERNAL_ONLY
    pub schema_version: u32,                          // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityProviderRegistrySnapshot {
    pub snapshot_id: String,                          // data_class: INTERNAL_ONLY
    pub tenant_id: String,                            // data_class: INTERNAL_ONLY
    pub records: Vec<IdentityProviderRegistryRecord>, // data_class: INTERNAL_ONLY
    pub raw_material_counters: IdentityProviderRegistryRawMaterialCounters, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                                                // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityProviderRegistrySnapshotCommit {
    pub snapshot_id: String,                // data_class: INTERNAL_ONLY
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub persisted_record_count: usize,      // data_class: INTERNAL_ONLY
    pub max_occurred_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityProviderRegistryEvidenceEvent {
    event_id: String,                                   // data_class: INTERNAL_ONLY
    tenant_id: String,                                  // data_class: INTERNAL_ONLY
    actor: String,                                      // data_class: INTERNAL_ONLY
    provider: CloudIamProviderKind,                     // data_class: PUBLIC
    operation: IamProviderIdentityProviderOperation,    // data_class: PUBLIC
    sync_status: IamProviderIdentityProviderSyncStatus, // data_class: PUBLIC
    identity_provider_id: String,                       // data_class: INTERNAL_ONLY
    provider_request_id: String,                        // data_class: INTERNAL_ONLY
    provider_evidence_ref: String,                      // data_class: INTERNAL_ONLY
    idempotency_key: String,                            // data_class: INTERNAL_ONLY
    occurred_at_epoch_seconds: u64,                     // data_class: INTERNAL_ONLY
    schema_version: u32,                                // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityProviderRegistryEvidenceReceipt {
    event_id: String,               // data_class: INTERNAL_ONLY
    tenant_id: String,              // data_class: INTERNAL_ONLY
    provider: CloudIamProviderKind, // data_class: PUBLIC
    provider_evidence_ref: String,  // data_class: INTERNAL_ONLY
    idempotency_key: String,        // data_class: INTERNAL_ONLY
    schema_version: u32,            // data_class: PUBLIC
}

pub trait IdentityProviderRegistrySnapshotRepository {
    fn persist_snapshot(
        &mut self,
        snapshot: IdentityProviderRegistrySnapshot,
    ) -> Result<IdentityProviderRegistrySnapshotCommit, CloudIamError>;
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMemoryIdentityProviderRegistrySnapshotRepository {
    snapshots: BTreeMap<String, IdentityProviderRegistrySnapshot>,
    records: BTreeMap<(String, String), IdentityProviderRegistryRecord>,
    idempotency_keys: BTreeSet<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IamProviderIdentityProviderError {
    InvalidProviderIdentityProviderRef,
    InvalidProviderRequestId,
    InvalidProviderEvidenceRef,
    InvalidIdempotencyKey,
    InvalidActorRef,
    InvalidRequestShape(CloudIamError),
    ProviderRejected {
        provider: CloudIamProviderKind, // data_class: PUBLIC
        reason: String,                 // data_class: INTERNAL_ONLY
    },
    ProviderUnavailable {
        provider: CloudIamProviderKind, // data_class: PUBLIC
        reason: String,                 // data_class: INTERNAL_ONLY
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IamPrincipalCreate {
    pub id: String,                                       // data_class: INTERNAL_ONLY
    pub tenant_id: String,                                // data_class: INTERNAL_ONLY
    pub kind: IamPrincipalKind,                           // data_class: PUBLIC
    pub display_name: String,                             // data_class: PII_QUASI_IDENTIFIER
    pub external_subject: Option<String>,                 // data_class: PII_QUASI_IDENTIFIER
    pub identity_provider_id: Option<String>,             // data_class: INTERNAL_ONLY
    pub region_pack: String,                              // data_class: INTERNAL_ONLY
    pub mfa_state: MfaState,                              // data_class: INTERNAL_ONLY
    pub last_authenticated_at_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,                    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IamPrincipal {
    pub id: Classified<IamPrincipalId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub kind: Classified<IamPrincipalKind>, // data_class: PUBLIC
    pub display_name: Classified<String>, // data_class: PII_QUASI_IDENTIFIER
    pub external_subject: Classified<Option<SubjectUri>>, // data_class: PII_QUASI_IDENTIFIER
    pub identity_provider_id: Classified<Option<IamIdentityProviderId>>, // data_class: INTERNAL_ONLY
    pub region_pack: Classified<String>, // data_class: INTERNAL_ONLY
    pub mfa_state: Classified<MfaState>, // data_class: INTERNAL_ONLY
    pub last_authenticated_at_epoch_seconds: Classified<Option<u64>>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IamRoleCreate {
    pub id: String,                    // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub region: String,                // data_class: PUBLIC
    pub name: String,                  // data_class: PUBLIC
    pub cedar_policy_id: String,       // data_class: INTERNAL_ONLY
    pub cedar_policy_version: String,  // data_class: INTERNAL_ONLY
    pub assumable_by: Vec<String>,     // data_class: INTERNAL_ONLY
    pub max_session_duration_sec: u32, // data_class: INTERNAL_ONLY
    pub data_class: DataClass,         // data_class: PUBLIC
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IamRole {
    pub id: Classified<IamRoleId>,      // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>, // data_class: PUBLIC
    pub name: Classified<RoleName>,     // data_class: PUBLIC
    pub cedar_policy_id: Classified<CedarPolicyId>, // data_class: INTERNAL_ONLY
    pub cedar_policy_version: Classified<String>, // data_class: INTERNAL_ONLY
    pub assumable_by: Classified<Vec<IamPrincipalId>>, // data_class: INTERNAL_ONLY
    pub max_session_duration_sec: Classified<u32>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: PUBLIC
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssumeRoleRequest {
    pub session_id: String,           // data_class: INTERNAL_ONLY
    pub tenant_id: String,            // data_class: INTERNAL_ONLY
    pub role_id: String,              // data_class: INTERNAL_ONLY
    pub assumed_by: String,           // data_class: INTERNAL_ONLY
    pub external_id: Option<String>,  // data_class: INTERNAL_ONLY
    pub requested_duration_sec: u32,  // data_class: INTERNAL_ONLY
    pub scopes: Vec<String>,          // data_class: INTERNAL_ONLY
    pub issued_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StsSession {
    pub id: Classified<StsSessionId>,        // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub assumed_role: Classified<IamRoleId>, // data_class: INTERNAL_ONLY
    pub assumed_by: Classified<IamPrincipalId>, // data_class: INTERNAL_ONLY
    pub external_id: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub issued_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub expires_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub scopes: Classified<Vec<ScopeRef>>,   // data_class: INTERNAL_ONLY
    pub token_fingerprint: Classified<String>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudIamError {
    InvalidTenantId,
    InvalidPrincipalId,
    InvalidRoleId,
    InvalidProviderId,
    InvalidCedarPolicyId,
    InvalidRoleName,
    InvalidScope,
    InvalidSubjectUri,
    InvalidRegionalPack,
    InvalidIssuerUri,
    InvalidAudience,
    InvalidVerificationMaterialRef,
    InvalidSessionId,
    InvalidExternalId,
    InvalidDataClass,
    InvalidSemver,
    InvalidSessionDuration,
    MissingAssumablePrincipal,
    DuplicateAssumablePrincipal,
    DuplicateScope,
    PrincipalKindMismatch,
    PrincipalCannotAssumeRole,
    MfaNotVerified,
    ExternalIdRequired,
    ProviderRequired,
    ProviderTenantMismatch,
    ProviderInUse,
    InvalidProviderEvidenceRef,
    InvalidIdentityProviderRegistrySnapshotId,
    InvalidIdentityProviderRegistrySnapshotSchemaVersion,
    EmptyIdentityProviderRegistrySnapshot,
    DuplicateIdentityProviderRegistrySnapshot,
    DuplicateIdentityProviderRegistryRecord,
    IdentityProviderRegistryRawMaterialForbidden,
    ProviderMismatch,
    MissingExternalSubject,
    UnexpectedExternalSubject,
    DuplicateProvider,
    DuplicatePrincipal,
    DuplicateRole,
    DuplicateSession,
    UnknownProvider,
    UnknownPrincipal,
    UnknownRole,
    TrustPolicyDenied,
    TenantMismatch,
    PlatformIdentityRejected(IdentityError),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct IamDirectory {
    providers: BTreeMap<IamIdentityProviderId, IdentityProvider>,
    principals: BTreeMap<IamPrincipalId, IamPrincipal>,
    roles: BTreeMap<IamRoleId, IamRole>,
    sessions: BTreeMap<StsSessionId, StsSession>,
}

pub trait IamProviderIdentityProviderPort {
    fn provider_kind(&self) -> CloudIamProviderKind;

    fn sync_identity_provider(
        &self,
        input: IamProviderIdentityProviderSyncRequest,
    ) -> Result<IamProviderIdentityProviderSyncReceipt, IamProviderIdentityProviderError>;
}

impl IamIdentityProviderId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudIamError> {
        prefixed_id(
            value.into(),
            IDENTITY_PROVIDER_PREFIX,
            CloudIamError::InvalidProviderId,
        )
        .map(|value| Self { value })
    }
}

impl IamPrincipalId {
    pub fn new_for_kind(
        value: impl Into<String>,
        kind: IamPrincipalKind,
    ) -> Result<Self, CloudIamError> {
        let value = value.into();
        let valid = match kind {
            IamPrincipalKind::User => {
                value.starts_with(USER_ID_PREFIX) && value.len() > USER_ID_PREFIX.len()
            }
            IamPrincipalKind::ServiceAccount
            | IamPrincipalKind::Federated
            | IamPrincipalKind::External => {
                value.starts_with(SERVICE_PRINCIPAL_PREFIX)
                    && value.len() > SERVICE_PRINCIPAL_PREFIX.len()
            }
            IamPrincipalKind::Role => {
                value.starts_with(ROLE_ID_PREFIX) && value.len() > ROLE_ID_PREFIX.len()
            }
        };
        if valid {
            Ok(Self { value })
        } else {
            Err(CloudIamError::InvalidPrincipalId)
        }
    }
}

impl IamRoleId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudIamError> {
        prefixed_id(value.into(), ROLE_ID_PREFIX, CloudIamError::InvalidRoleId)
            .map(|value| Self { value })
    }
}

impl CedarPolicyId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudIamError> {
        prefixed_id(
            value.into(),
            CEDAR_POLICY_PREFIX,
            CloudIamError::InvalidCedarPolicyId,
        )
        .map(|value| Self { value })
    }
}

impl RoleName {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudIamError> {
        let value = value.into();
        validate_canonical_segment(&value, CloudIamError::InvalidRoleName)?;
        Ok(Self { value })
    }
}

impl ScopeRef {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudIamError> {
        let value = value.into();
        if value.starts_with(CLOUD_SCOPE_PREFIX)
            && value.len() > CLOUD_SCOPE_PREFIX.len()
            && !value.bytes().any(|byte| byte.is_ascii_whitespace())
        {
            Ok(Self { value })
        } else {
            Err(CloudIamError::InvalidScope)
        }
    }
}

impl SubjectUri {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudIamError> {
        let value = value.into();
        if (value.starts_with("saml://") || value.starts_with("oidc://"))
            && value.len() > "saml://".len()
        {
            Ok(Self { value })
        } else {
            Err(CloudIamError::InvalidSubjectUri)
        }
    }
}

impl StsSessionId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudIamError> {
        prefixed_id(
            value.into(),
            STS_SESSION_PREFIX,
            CloudIamError::InvalidSessionId,
        )
        .map(|value| Self { value })
    }
}

impl IdentityProvider {
    pub fn new(input: IdentityProviderCreate) -> Result<Self, CloudIamError> {
        let id = IamIdentityProviderId::new(input.id)?;
        validate_tenant_id(&input.tenant_id)?;
        validate_regional_pack(&input.region_pack)?;
        validate_https_uri(&input.issuer_uri)?;
        validate_non_empty(&input.audience, CloudIamError::InvalidAudience)?;
        validate_verification_material(input.kind, &input.verification_material_ref)?;
        Ok(Self {
            id: internal(id),
            tenant_id: internal(input.tenant_id),
            region_pack: internal(input.region_pack),
            kind: public(input.kind),
            issuer_uri: internal(input.issuer_uri),
            audience: internal(input.audience),
            verification_material_ref: internal(input.verification_material_ref),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(IDENTITY_PROVIDER_SCHEMA_VERSION),
        })
    }
}

impl IamProviderIdentityProviderSyncRequest {
    pub fn validate(&self) -> Result<(), IamProviderIdentityProviderError> {
        validate_provider_ref(
            &self.request_id,
            IamProviderIdentityProviderError::InvalidProviderRequestId,
        )?;
        validate_provider_ref(
            &self.provider_identity_provider_ref,
            IamProviderIdentityProviderError::InvalidProviderIdentityProviderRef,
        )?;
        validate_provider_ref(
            &self.idempotency_key,
            IamProviderIdentityProviderError::InvalidIdempotencyKey,
        )?;
        validate_tenant_id(&self.tenant_id)
            .map_err(IamProviderIdentityProviderError::InvalidRequestShape)?;
        infer_principal_id(self.actor.clone())
            .map_err(|_| IamProviderIdentityProviderError::InvalidActorRef)?;
        if self.identity_provider.tenant_id.value != self.tenant_id {
            return Err(IamProviderIdentityProviderError::InvalidRequestShape(
                CloudIamError::ProviderTenantMismatch,
            ));
        }
        Ok(())
    }
}

impl IamProviderIdentityProviderSyncReceipt {
    pub fn from_request(
        provider: CloudIamProviderKind,
        input: IamProviderIdentityProviderSyncRequest,
        provider_request_id: impl Into<String>,
        provider_evidence_ref: impl Into<String>,
    ) -> Result<Self, IamProviderIdentityProviderError> {
        input.validate()?;
        let provider_request_id = provider_request_id.into();
        let provider_evidence_ref = provider_evidence_ref.into();
        validate_provider_ref(
            &provider_request_id,
            IamProviderIdentityProviderError::InvalidProviderRequestId,
        )?;
        validate_provider_evidence_ref(&provider_evidence_ref)?;
        let sync_status = match input.operation {
            IamProviderIdentityProviderOperation::Upsert => {
                IamProviderIdentityProviderSyncStatus::Synchronized
            }
            IamProviderIdentityProviderOperation::Delete => {
                IamProviderIdentityProviderSyncStatus::DeleteSynchronized
            }
        };
        Ok(Self {
            provider,
            operation: input.operation,
            sync_status,
            request_id: input.request_id,
            provider_request_id,
            tenant_id: input.tenant_id,
            identity_provider_id: input.identity_provider.id.value.value,
            identity_provider_kind: input.identity_provider.kind.value,
            region_pack: input.identity_provider.region_pack.value,
            issuer_ref: input.identity_provider.issuer_uri.value,
            audience_ref: input.identity_provider.audience.value,
            verification_material_ref: input.identity_provider.verification_material_ref.value,
            provider_identity_provider_ref: input.provider_identity_provider_ref,
            actor: input.actor,
            idempotency_key: input.idempotency_key,
            provider_evidence_ref,
            occurred_at_epoch_seconds: input.requested_at_epoch_seconds,
            schema_version: IAM_PROVIDER_IDP_SYNC_SCHEMA_VERSION,
        })
    }
}

impl IdentityProviderRegistryRawMaterialCounters {
    fn ensure_metadata_only(self) -> Result<(), CloudIamError> {
        if self.raw_provider_document_bytes == 0
            && self.credential_material_bytes == 0
            && self.assertion_material_bytes == 0
            && self.sts_material_bytes == 0
        {
            Ok(())
        } else {
            Err(CloudIamError::IdentityProviderRegistryRawMaterialForbidden)
        }
    }
}

impl IdentityProviderRegistryRecord {
    fn from_receipt(receipt: IamProviderIdentityProviderSyncReceipt) -> Self {
        Self {
            tenant_id: receipt.tenant_id,
            identity_provider_id: receipt.identity_provider_id,
            identity_provider_kind: receipt.identity_provider_kind,
            region_pack: receipt.region_pack,
            issuer_ref: receipt.issuer_ref,
            audience_ref: receipt.audience_ref,
            verification_material_ref: receipt.verification_material_ref,
            provider: receipt.provider,
            operation: receipt.operation,
            sync_status: receipt.sync_status,
            provider_identity_provider_ref: receipt.provider_identity_provider_ref,
            provider_request_id: receipt.provider_request_id,
            actor: receipt.actor,
            idempotency_key: receipt.idempotency_key,
            provider_evidence_ref: receipt.provider_evidence_ref,
            occurred_at_epoch_seconds: receipt.occurred_at_epoch_seconds,
            raw_provider_document_bytes: 0,
            credential_material_bytes: 0,
            assertion_material_bytes: 0,
            sts_material_bytes: 0,
            schema_version: receipt.schema_version,
        }
    }

    fn key(&self) -> (String, String) {
        (self.tenant_id.clone(), self.identity_provider_id.clone())
    }
}

impl IdentityProviderRegistrySnapshot {
    pub fn from_receipts(
        snapshot_id: impl Into<String>,
        tenant_id: impl Into<String>,
        receipts: Vec<IamProviderIdentityProviderSyncReceipt>,
        raw_material_counters: IdentityProviderRegistryRawMaterialCounters,
    ) -> Result<Self, CloudIamError> {
        let snapshot_id = snapshot_id.into();
        let tenant_id = tenant_id.into();
        validate_registry_ref(
            &snapshot_id,
            CloudIamError::InvalidIdentityProviderRegistrySnapshotId,
        )?;
        validate_tenant_id(&tenant_id)?;
        raw_material_counters.ensure_metadata_only()?;
        if receipts.is_empty() {
            return Err(CloudIamError::EmptyIdentityProviderRegistrySnapshot);
        }

        let mut seen_idempotency_keys = BTreeSet::new();
        let mut seen_records = BTreeSet::new();
        let mut records = Vec::with_capacity(receipts.len());
        for receipt in receipts {
            if receipt.tenant_id != tenant_id {
                return Err(CloudIamError::TenantMismatch);
            }
            if !seen_idempotency_keys.insert(receipt.idempotency_key.clone()) {
                return Err(CloudIamError::DuplicateIdentityProviderRegistrySnapshot);
            }
            let record = IdentityProviderRegistryRecord::from_receipt(receipt);
            if !seen_records.insert(record.key()) {
                return Err(CloudIamError::DuplicateIdentityProviderRegistryRecord);
            }
            records.push(record);
        }

        Ok(Self {
            snapshot_id,
            tenant_id,
            records,
            raw_material_counters,
            schema_version: IAM_PROVIDER_IDP_SYNC_SCHEMA_VERSION,
        })
    }

    fn validate_for_persistence(&self) -> Result<(), CloudIamError> {
        validate_registry_ref(
            &self.snapshot_id,
            CloudIamError::InvalidIdentityProviderRegistrySnapshotId,
        )?;
        validate_tenant_id(&self.tenant_id)?;
        if self.schema_version != IAM_PROVIDER_IDP_SYNC_SCHEMA_VERSION {
            return Err(CloudIamError::InvalidIdentityProviderRegistrySnapshotSchemaVersion);
        }
        if self.records.is_empty() {
            return Err(CloudIamError::EmptyIdentityProviderRegistrySnapshot);
        }
        self.raw_material_counters.ensure_metadata_only()?;
        let mut seen_idempotency_keys = BTreeSet::new();
        let mut seen_records = BTreeSet::new();
        for record in &self.records {
            if record.tenant_id != self.tenant_id {
                return Err(CloudIamError::TenantMismatch);
            }
            validate_registry_ref(
                &record.provider_identity_provider_ref,
                CloudIamError::InvalidProviderEvidenceRef,
            )?;
            validate_registry_provider_evidence_ref(&record.provider_evidence_ref)?;
            validate_registry_ref(
                &record.provider_request_id,
                CloudIamError::InvalidProviderEvidenceRef,
            )?;
            validate_registry_ref(
                &record.idempotency_key,
                CloudIamError::InvalidProviderEvidenceRef,
            )?;
            infer_principal_id(record.actor.clone())
                .map_err(|_| CloudIamError::InvalidPrincipalId)?;
            if !seen_idempotency_keys.insert(record.idempotency_key.clone()) {
                return Err(CloudIamError::DuplicateIdentityProviderRegistrySnapshot);
            }
            if !seen_records.insert(record.key()) {
                return Err(CloudIamError::DuplicateIdentityProviderRegistryRecord);
            }
            if record.raw_provider_document_bytes != 0
                || record.credential_material_bytes != 0
                || record.assertion_material_bytes != 0
                || record.sts_material_bytes != 0
            {
                return Err(CloudIamError::IdentityProviderRegistryRawMaterialForbidden);
            }
            if record.schema_version != IAM_PROVIDER_IDP_SYNC_SCHEMA_VERSION {
                return Err(CloudIamError::InvalidIdentityProviderRegistrySnapshotSchemaVersion);
            }
        }
        Ok(())
    }
}

impl IdentityProviderRegistryEvidenceEvent {
    pub fn from_sync_receipt(
        expected_tenant_id: &str,
        expected_provider: CloudIamProviderKind,
        receipt: IamProviderIdentityProviderSyncReceipt,
    ) -> Result<Self, CloudIamError> {
        validate_tenant_id(expected_tenant_id)?;
        if receipt.tenant_id != expected_tenant_id {
            return Err(CloudIamError::TenantMismatch);
        }
        if receipt.provider != expected_provider {
            return Err(CloudIamError::ProviderMismatch);
        }
        if receipt.schema_version != IAM_PROVIDER_IDP_SYNC_SCHEMA_VERSION {
            return Err(CloudIamError::InvalidIdentityProviderRegistrySnapshotSchemaVersion);
        }
        validate_registry_provider_evidence_ref(&receipt.provider_evidence_ref)?;
        validate_registry_ref(
            &receipt.provider_request_id,
            CloudIamError::InvalidProviderEvidenceRef,
        )?;
        validate_registry_ref(
            &receipt.request_id,
            CloudIamError::InvalidProviderEvidenceRef,
        )?;
        validate_registry_ref(
            &receipt.idempotency_key,
            CloudIamError::InvalidProviderEvidenceRef,
        )?;
        infer_principal_id(receipt.actor.clone()).map_err(|_| CloudIamError::InvalidPrincipalId)?;

        let event_id = format!(
            "evt_cloud_iam_idp_registry_{}_{}_{}",
            receipt.tenant_id,
            receipt.provider.label(),
            receipt.idempotency_key
        );
        validate_registry_ref(
            &event_id,
            CloudIamError::InvalidIdentityProviderRegistrySnapshotId,
        )?;

        Ok(Self {
            event_id,
            tenant_id: receipt.tenant_id,
            actor: receipt.actor,
            provider: receipt.provider,
            operation: receipt.operation,
            sync_status: receipt.sync_status,
            identity_provider_id: receipt.identity_provider_id,
            provider_request_id: receipt.provider_request_id,
            provider_evidence_ref: receipt.provider_evidence_ref,
            idempotency_key: receipt.idempotency_key,
            occurred_at_epoch_seconds: receipt.occurred_at_epoch_seconds,
            schema_version: IAM_PROVIDER_IDP_REGISTRY_EVIDENCE_SCHEMA_VERSION,
        })
    }

    pub fn event_id(&self) -> &str {
        &self.event_id
    }

    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    pub fn actor(&self) -> &str {
        &self.actor
    }

    pub fn provider(&self) -> CloudIamProviderKind {
        self.provider
    }

    pub fn operation(&self) -> IamProviderIdentityProviderOperation {
        self.operation
    }

    pub fn sync_status(&self) -> IamProviderIdentityProviderSyncStatus {
        self.sync_status
    }

    pub fn identity_provider_id(&self) -> &str {
        &self.identity_provider_id
    }

    pub fn provider_request_id(&self) -> &str {
        &self.provider_request_id
    }

    pub fn provider_evidence_ref(&self) -> &str {
        &self.provider_evidence_ref
    }

    pub fn idempotency_key(&self) -> &str {
        &self.idempotency_key
    }

    pub fn occurred_at_epoch_seconds(&self) -> u64 {
        self.occurred_at_epoch_seconds
    }

    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }

    pub fn receipt(&self) -> IdentityProviderRegistryEvidenceReceipt {
        IdentityProviderRegistryEvidenceReceipt {
            event_id: self.event_id.clone(),
            tenant_id: self.tenant_id.clone(),
            provider: self.provider,
            provider_evidence_ref: self.provider_evidence_ref.clone(),
            idempotency_key: self.idempotency_key.clone(),
            schema_version: self.schema_version,
        }
    }
}

impl IdentityProviderRegistryEvidenceReceipt {
    pub fn event_id(&self) -> &str {
        &self.event_id
    }

    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    pub fn provider(&self) -> CloudIamProviderKind {
        self.provider
    }

    pub fn provider_evidence_ref(&self) -> &str {
        &self.provider_evidence_ref
    }

    pub fn idempotency_key(&self) -> &str {
        &self.idempotency_key
    }

    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }
}

impl IdentityProviderRegistrySnapshotRepository
    for InMemoryIdentityProviderRegistrySnapshotRepository
{
    fn persist_snapshot(
        &mut self,
        snapshot: IdentityProviderRegistrySnapshot,
    ) -> Result<IdentityProviderRegistrySnapshotCommit, CloudIamError> {
        snapshot.validate_for_persistence()?;
        if self.snapshots.contains_key(&snapshot.snapshot_id)
            || snapshot
                .records
                .iter()
                .any(|record| self.idempotency_keys.contains(&record.idempotency_key))
        {
            return Err(CloudIamError::DuplicateIdentityProviderRegistrySnapshot);
        }
        if snapshot
            .records
            .iter()
            .any(|record| self.records.contains_key(&record.key()))
        {
            return Err(CloudIamError::DuplicateIdentityProviderRegistryRecord);
        }

        let max_occurred_at_epoch_seconds = snapshot
            .records
            .iter()
            .map(|record| record.occurred_at_epoch_seconds)
            .max()
            .unwrap_or(0);
        let commit = IdentityProviderRegistrySnapshotCommit {
            snapshot_id: snapshot.snapshot_id.clone(),
            tenant_id: snapshot.tenant_id.clone(),
            persisted_record_count: snapshot.records.len(),
            max_occurred_at_epoch_seconds,
            schema_version: snapshot.schema_version,
        };

        for record in &snapshot.records {
            self.idempotency_keys.insert(record.idempotency_key.clone());
            self.records.insert(record.key(), record.clone());
        }
        self.snapshots
            .insert(snapshot.snapshot_id.clone(), snapshot);
        Ok(commit)
    }
}

impl InMemoryIdentityProviderRegistrySnapshotRepository {
    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }

    pub fn record(
        &self,
        tenant_id: &str,
        identity_provider_id: &str,
    ) -> Option<&IdentityProviderRegistryRecord> {
        self.records
            .get(&(tenant_id.to_string(), identity_provider_id.to_string()))
    }
}

impl IamPrincipal {
    pub fn new(input: IamPrincipalCreate) -> Result<Self, CloudIamError> {
        let id = IamPrincipalId::new_for_kind(input.id, input.kind)?;
        validate_tenant_id(&input.tenant_id)?;
        validate_non_empty(&input.display_name, CloudIamError::InvalidPrincipalId)?;
        validate_regional_pack(&input.region_pack)?;
        validate_principal_external_shape(
            input.kind,
            input.external_subject.as_deref(),
            input.identity_provider_id.as_deref(),
        )?;
        let external_subject = input.external_subject.map(SubjectUri::new).transpose()?;
        let identity_provider_id = input
            .identity_provider_id
            .map(IamIdentityProviderId::new)
            .transpose()?;
        Ok(Self {
            id: internal(id),
            tenant_id: internal(input.tenant_id),
            kind: public(input.kind),
            display_name: Classified::new(input.display_name, display_name_class(input.kind)),
            external_subject: Classified::new(external_subject, DataClass::PiiQuasiIdentifier),
            identity_provider_id: internal(identity_provider_id),
            region_pack: internal(input.region_pack),
            mfa_state: internal(input.mfa_state),
            last_authenticated_at_epoch_seconds: internal(
                input.last_authenticated_at_epoch_seconds,
            ),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(IAM_PRINCIPAL_SCHEMA_VERSION),
        })
    }

    fn platform_principal(&self) -> Result<Principal, CloudIamError> {
        match self.kind.value {
            IamPrincipalKind::User => {
                Principal::human(self.tenant_id.value.clone(), self.id.value.value.clone())
            }
            IamPrincipalKind::ServiceAccount
            | IamPrincipalKind::Federated
            | IamPrincipalKind::External => Principal::service(
                self.tenant_id.value.clone(),
                self.id.value.value.clone(),
                CLOUD_IAM_CAPABILITY_ID.to_string(),
            ),
            IamPrincipalKind::Role => return Err(CloudIamError::PrincipalCannotAssumeRole),
        }
        .map_err(CloudIamError::PlatformIdentityRejected)
    }

    fn can_attempt_assume_role(&self) -> Result<(), CloudIamError> {
        match self.kind.value {
            IamPrincipalKind::Role => Err(CloudIamError::PrincipalCannotAssumeRole),
            IamPrincipalKind::User | IamPrincipalKind::Federated | IamPrincipalKind::External => {
                if self.mfa_state.value == MfaState::Verified {
                    Ok(())
                } else {
                    Err(CloudIamError::MfaNotVerified)
                }
            }
            IamPrincipalKind::ServiceAccount => Ok(()),
        }
    }
}

impl IamRole {
    pub fn new(input: IamRoleCreate) -> Result<Self, CloudIamError> {
        let id = IamRoleId::new(input.id)?;
        validate_tenant_id(&input.tenant_id)?;
        let region = RegionCode::new(input.region).map_err(|_| CloudIamError::InvalidRoleId)?;
        let name = RoleName::new(input.name)?;
        let cedar_policy_id = CedarPolicyId::new(input.cedar_policy_id)?;
        validate_semver(&input.cedar_policy_version)?;
        let assumable_by = typed_principal_ids(input.assumable_by)?;
        validate_duration(input.max_session_duration_sec)?;
        let data_class = public_data_class(input.data_class)?;
        Ok(Self {
            id: internal(id),
            tenant_id: internal(input.tenant_id),
            region: public(region),
            name: public(name),
            cedar_policy_id: internal(cedar_policy_id),
            cedar_policy_version: internal(input.cedar_policy_version),
            assumable_by: internal(assumable_by),
            max_session_duration_sec: internal(input.max_session_duration_sec),
            data_class: public(data_class),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(IAM_ROLE_SCHEMA_VERSION),
        })
    }
}

impl StsSession {
    fn new(
        request: AssumeRoleRequest,
        role: &IamRole,
        principal: &IamPrincipal,
    ) -> Result<Self, CloudIamError> {
        let id = StsSessionId::new(request.session_id)?;
        validate_tenant_id(&request.tenant_id)?;
        if request.tenant_id != role.tenant_id.value
            || request.tenant_id != principal.tenant_id.value
        {
            return Err(CloudIamError::TenantMismatch);
        }
        let requested_role_id = IamRoleId::new(request.role_id)?;
        if requested_role_id != role.id.value {
            return Err(CloudIamError::UnknownRole);
        }
        let assumed_by = IamPrincipalId::new_for_kind(request.assumed_by, principal.kind.value)?;
        if assumed_by != principal.id.value {
            return Err(CloudIamError::UnknownPrincipal);
        }
        if !role.assumable_by.value.contains(&assumed_by) {
            return Err(CloudIamError::TrustPolicyDenied);
        }
        principal.can_attempt_assume_role()?;
        validate_external_id(principal.kind.value, request.external_id.as_deref())?;
        if request.requested_duration_sec > role.max_session_duration_sec.value {
            return Err(CloudIamError::InvalidSessionDuration);
        }
        validate_duration(request.requested_duration_sec)?;
        let scopes = typed_scopes(request.scopes)?;
        let credential = issue_credential(CredentialRequest {
            principal: principal.platform_principal()?,
            kind: CredentialRequestKind::Sts,
            purpose: Purpose::CapabilityInvocation,
            scopes: scopes.iter().map(|scope| scope.value.clone()).collect(),
            ttl_seconds: u64::from(request.requested_duration_sec),
            issued_at_epoch_seconds: request.issued_at_epoch_seconds,
        })
        .map_err(CloudIamError::PlatformIdentityRejected)?;
        Ok(Self {
            id: internal(id),
            tenant_id: internal(request.tenant_id),
            assumed_role: internal(role.id.value.clone()),
            assumed_by: internal(assumed_by),
            external_id: internal(request.external_id),
            issued_at_epoch_seconds: internal(request.issued_at_epoch_seconds),
            expires_at_epoch_seconds: internal(
                request.issued_at_epoch_seconds + u64::from(request.requested_duration_sec),
            ),
            scopes: internal(scopes),
            token_fingerprint: internal(credential.token_fingerprint.value),
            data_class: public(public_data_class(DataClass::Public)?),
            schema_version: public(STS_SESSION_SCHEMA_VERSION),
        })
    }
}

impl IamDirectory {
    pub fn register_identity_provider(
        &mut self,
        input: IdentityProviderCreate,
    ) -> Result<IdentityProvider, CloudIamError> {
        let provider = IdentityProvider::new(input)?;
        if self.providers.contains_key(&provider.id.value) {
            return Err(CloudIamError::DuplicateProvider);
        }
        self.providers
            .insert(provider.id.value.clone(), provider.clone());
        Ok(provider)
    }

    pub fn list_identity_providers(
        &self,
        tenant_id: &str,
    ) -> Result<Vec<IdentityProvider>, CloudIamError> {
        validate_tenant_id(tenant_id)?;
        Ok(self
            .providers
            .values()
            .filter(|provider| provider.tenant_id.value == tenant_id)
            .cloned()
            .collect())
    }

    pub fn update_identity_provider(
        &mut self,
        input: IdentityProviderUpdate,
    ) -> Result<IdentityProvider, CloudIamError> {
        let provider_id = IamIdentityProviderId::new(input.id.clone())?;
        validate_tenant_id(&input.tenant_id)?;
        let existing = self
            .providers
            .get(&provider_id)
            .ok_or(CloudIamError::UnknownProvider)?;
        if existing.tenant_id.value != input.tenant_id {
            return Err(CloudIamError::ProviderTenantMismatch);
        }
        let provider = IdentityProvider::new(IdentityProviderCreate {
            id: input.id,
            tenant_id: input.tenant_id,
            region_pack: input.region_pack,
            kind: input.kind,
            issuer_uri: input.issuer_uri,
            audience: input.audience,
            verification_material_ref: input.verification_material_ref,
            created_at_epoch_seconds: existing.created_at_epoch_seconds.value,
        })?;
        self.providers.insert(provider_id, provider.clone());
        Ok(provider)
    }

    pub fn delete_identity_provider(
        &mut self,
        tenant_id: &str,
        provider_id: &str,
    ) -> Result<IdentityProvider, CloudIamError> {
        validate_tenant_id(tenant_id)?;
        let provider_id = IamIdentityProviderId::new(provider_id.to_string())?;
        {
            let existing = self
                .providers
                .get(&provider_id)
                .ok_or(CloudIamError::UnknownProvider)?;
            if existing.tenant_id.value != tenant_id {
                return Err(CloudIamError::ProviderTenantMismatch);
            }
        }
        if self
            .principals
            .values()
            .any(|principal| principal.identity_provider_id.value.as_ref() == Some(&provider_id))
        {
            return Err(CloudIamError::ProviderInUse);
        }
        self.providers
            .remove(&provider_id)
            .ok_or(CloudIamError::UnknownProvider)
    }

    pub fn create_principal(
        &mut self,
        input: IamPrincipalCreate,
    ) -> Result<IamPrincipal, CloudIamError> {
        let principal = IamPrincipal::new(input)?;
        if self.principals.contains_key(&principal.id.value) {
            return Err(CloudIamError::DuplicatePrincipal);
        }
        if let Some(provider_id) = principal.identity_provider_id.value.as_ref() {
            let provider = self
                .providers
                .get(provider_id)
                .ok_or(CloudIamError::UnknownProvider)?;
            if provider.tenant_id.value != principal.tenant_id.value {
                return Err(CloudIamError::ProviderTenantMismatch);
            }
        }
        self.principals
            .insert(principal.id.value.clone(), principal.clone());
        Ok(principal)
    }

    pub fn create_role(&mut self, input: IamRoleCreate) -> Result<IamRole, CloudIamError> {
        let role = IamRole::new(input)?;
        if self.roles.contains_key(&role.id.value) {
            return Err(CloudIamError::DuplicateRole);
        }
        for principal_id in &role.assumable_by.value {
            let principal = self
                .principals
                .get(principal_id)
                .ok_or(CloudIamError::UnknownPrincipal)?;
            if principal.tenant_id.value != role.tenant_id.value {
                return Err(CloudIamError::TenantMismatch);
            }
            if principal.kind.value == IamPrincipalKind::Role {
                return Err(CloudIamError::PrincipalCannotAssumeRole);
            }
        }
        self.roles.insert(role.id.value.clone(), role.clone());
        Ok(role)
    }

    pub fn assume_role(&mut self, request: AssumeRoleRequest) -> Result<StsSession, CloudIamError> {
        let session_id = StsSessionId::new(request.session_id.clone())?;
        if self.sessions.contains_key(&session_id) {
            return Err(CloudIamError::DuplicateSession);
        }
        let role_id = IamRoleId::new(request.role_id.clone())?;
        let role = self.roles.get(&role_id).ok_or(CloudIamError::UnknownRole)?;
        let principal = self
            .principals
            .values()
            .find(|principal| principal.id.value.value == request.assumed_by)
            .ok_or(CloudIamError::UnknownPrincipal)?;
        let session = StsSession::new(request, role, principal)?;
        self.sessions
            .insert(session.id.value.clone(), session.clone());
        Ok(session)
    }

    pub fn principal(&self, id: &IamPrincipalId) -> Option<&IamPrincipal> {
        self.principals.get(id)
    }

    pub fn role(&self, id: &IamRoleId) -> Option<&IamRole> {
        self.roles.get(id)
    }

    pub fn session(&self, id: &StsSessionId) -> Option<&StsSession> {
        self.sessions.get(id)
    }
}

fn validate_principal_external_shape(
    kind: IamPrincipalKind,
    external_subject: Option<&str>,
    provider_id: Option<&str>,
) -> Result<(), CloudIamError> {
    match kind {
        IamPrincipalKind::Federated | IamPrincipalKind::External => {
            if external_subject.is_none() {
                return Err(CloudIamError::MissingExternalSubject);
            }
            if provider_id.is_none() {
                return Err(CloudIamError::ProviderRequired);
            }
            Ok(())
        }
        IamPrincipalKind::User | IamPrincipalKind::ServiceAccount | IamPrincipalKind::Role => {
            if external_subject.is_some() || provider_id.is_some() {
                Err(CloudIamError::UnexpectedExternalSubject)
            } else {
                Ok(())
            }
        }
    }
}

fn validate_external_id(
    kind: IamPrincipalKind,
    external_id: Option<&str>,
) -> Result<(), CloudIamError> {
    match (kind, external_id) {
        (IamPrincipalKind::External, Some(value)) => {
            validate_non_empty(value, CloudIamError::InvalidExternalId)
        }
        (IamPrincipalKind::External, None) => Err(CloudIamError::ExternalIdRequired),
        (_, Some(value)) => validate_non_empty(value, CloudIamError::InvalidExternalId),
        (_, None) => Ok(()),
    }
}

fn typed_principal_ids(values: Vec<String>) -> Result<Vec<IamPrincipalId>, CloudIamError> {
    if values.is_empty() {
        return Err(CloudIamError::MissingAssumablePrincipal);
    }
    let mut seen = BTreeSet::new();
    let mut typed = Vec::with_capacity(values.len());
    for value in values {
        let principal_id = infer_principal_id(value)?;
        if !seen.insert(principal_id.clone()) {
            return Err(CloudIamError::DuplicateAssumablePrincipal);
        }
        typed.push(principal_id);
    }
    Ok(typed)
}

fn infer_principal_id(value: String) -> Result<IamPrincipalId, CloudIamError> {
    if value.starts_with(USER_ID_PREFIX) {
        IamPrincipalId::new_for_kind(value, IamPrincipalKind::User)
    } else if value.starts_with(SERVICE_PRINCIPAL_PREFIX) {
        IamPrincipalId::new_for_kind(value, IamPrincipalKind::ServiceAccount)
    } else if value.starts_with(ROLE_ID_PREFIX) {
        IamPrincipalId::new_for_kind(value, IamPrincipalKind::Role)
    } else {
        Err(CloudIamError::InvalidPrincipalId)
    }
}

fn typed_scopes(values: Vec<String>) -> Result<Vec<ScopeRef>, CloudIamError> {
    if values.is_empty() {
        return Err(CloudIamError::InvalidScope);
    }
    let mut seen = BTreeSet::new();
    let mut typed = Vec::with_capacity(values.len());
    for value in values {
        let scope = ScopeRef::new(value)?;
        if !seen.insert(scope.clone()) {
            return Err(CloudIamError::DuplicateScope);
        }
        typed.push(scope);
    }
    Ok(typed)
}

fn public_data_class(data_class: DataClass) -> Result<PrivacyDataClass, CloudIamError> {
    let data_class =
        PrivacyDataClass::new(data_class).map_err(|_| CloudIamError::InvalidDataClass)?;
    if data_class.data_class() == DataClass::Public {
        Ok(data_class)
    } else {
        Err(CloudIamError::InvalidDataClass)
    }
}

fn validate_duration(duration: u32) -> Result<(), CloudIamError> {
    if duration > 0 && u64::from(duration) <= MAX_TOKEN_TTL_SECONDS {
        Ok(())
    } else {
        Err(CloudIamError::InvalidSessionDuration)
    }
}

fn validate_tenant_id(value: &str) -> Result<(), CloudIamError> {
    if value.starts_with(TENANT_ID_PREFIX) && value.len() > TENANT_ID_PREFIX.len() {
        Ok(())
    } else {
        Err(CloudIamError::InvalidTenantId)
    }
}

fn validate_regional_pack(value: &str) -> Result<(), CloudIamError> {
    if value.starts_with(REGIONAL_PACK_PREFIX) && value.len() > REGIONAL_PACK_PREFIX.len() {
        Ok(())
    } else {
        Err(CloudIamError::InvalidRegionalPack)
    }
}

fn validate_https_uri(value: &str) -> Result<(), CloudIamError> {
    if value.starts_with("https://") && value.len() > "https://".len() {
        Ok(())
    } else {
        Err(CloudIamError::InvalidIssuerUri)
    }
}

fn validate_verification_material(
    kind: IdentityProviderKind,
    value: &str,
) -> Result<(), CloudIamError> {
    let expected_prefix = match kind {
        IdentityProviderKind::Saml => CERT_REF_PREFIX,
        IdentityProviderKind::Oidc => JWKS_REF_PREFIX,
    };
    if value.starts_with(expected_prefix) && value.len() > expected_prefix.len() {
        Ok(())
    } else {
        Err(CloudIamError::InvalidVerificationMaterialRef)
    }
}

fn validate_semver(value: &str) -> Result<(), CloudIamError> {
    let parts = value.split('.').collect::<Vec<_>>();
    if parts.len() == 3 && parts.iter().all(|part| part.parse::<u64>().is_ok()) {
        Ok(())
    } else {
        Err(CloudIamError::InvalidSemver)
    }
}

fn validate_canonical_segment(value: &str, error: CloudIamError) -> Result<(), CloudIamError> {
    if value.trim().is_empty()
        || value.starts_with('-')
        || value.ends_with('-')
        || value.contains("--")
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(error);
    }
    Ok(())
}

fn prefixed_id(value: String, prefix: &str, error: CloudIamError) -> Result<String, CloudIamError> {
    if value.starts_with(prefix) && value.len() > prefix.len() {
        Ok(value)
    } else {
        Err(error)
    }
}

fn validate_non_empty(value: &str, error: CloudIamError) -> Result<(), CloudIamError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_provider_ref(
    value: &str,
    error: IamProviderIdentityProviderError,
) -> Result<(), IamProviderIdentityProviderError> {
    if value.trim().is_empty()
        || value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
    {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_provider_evidence_ref(value: &str) -> Result<(), IamProviderIdentityProviderError> {
    if is_provider_evidence_ref(value) {
        Ok(())
    } else {
        Err(IamProviderIdentityProviderError::InvalidProviderEvidenceRef)
    }
}

fn validate_registry_ref(value: &str, error: CloudIamError) -> Result<(), CloudIamError> {
    if value.trim().is_empty()
        || value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
    {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_registry_provider_evidence_ref(value: &str) -> Result<(), CloudIamError> {
    if is_provider_evidence_ref(value) {
        Ok(())
    } else {
        Err(CloudIamError::InvalidProviderEvidenceRef)
    }
}

fn is_provider_evidence_ref(value: &str) -> bool {
    provider_evidence_segments(value, OCI_IDP_EVIDENCE_REF_PREFIX, 3)
        || provider_evidence_segments(value, SELFHOSTED_IDP_EVIDENCE_REF_PREFIX, 5)
}

fn provider_evidence_segments(value: &str, prefix: &str, expected_segments: usize) -> bool {
    let Some(rest) = value.strip_prefix(prefix) else {
        return false;
    };
    let segments = rest.split('/').collect::<Vec<_>>();
    segments.len() == expected_segments
        && segments
            .iter()
            .all(|segment| is_safe_provider_ref_segment(segment))
}

fn is_safe_provider_ref_segment(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'_'
        })
}

fn display_name_class(kind: IamPrincipalKind) -> DataClass {
    match kind {
        IamPrincipalKind::User | IamPrincipalKind::Federated | IamPrincipalKind::External => {
            DataClass::PiiQuasiIdentifier
        }
        IamPrincipalKind::ServiceAccount | IamPrincipalKind::Role => DataClass::Public,
    }
}

fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provider_create() -> IdentityProviderCreate {
        IdentityProviderCreate {
            id: "idp_alpha_saml".to_string(),
            tenant_id: "ten_alpha".to_string(),
            region_pack: "pack-alpha".to_string(),
            kind: IdentityProviderKind::Saml,
            issuer_uri: "https://idp.alpha.example/saml".to_string(),
            audience: "urn:oyatie:cloud".to_string(),
            verification_material_ref: "cert/alpha-saml-signing".to_string(),
            created_at_epoch_seconds: 1_700_000_000,
        }
    }

    fn service_principal_create() -> IamPrincipalCreate {
        IamPrincipalCreate {
            id: "sp_cloud_provisioner".to_string(),
            tenant_id: "ten_alpha".to_string(),
            kind: IamPrincipalKind::ServiceAccount,
            display_name: "cloud provisioner".to_string(),
            external_subject: None,
            identity_provider_id: None,
            region_pack: "pack-alpha".to_string(),
            mfa_state: MfaState::NotRequired,
            last_authenticated_at_epoch_seconds: None,
            created_at_epoch_seconds: 1_700_000_001,
        }
    }

    fn user_principal_create() -> IamPrincipalCreate {
        IamPrincipalCreate {
            id: "usr_alice".to_string(),
            tenant_id: "ten_alpha".to_string(),
            kind: IamPrincipalKind::User,
            display_name: "Alice".to_string(),
            external_subject: None,
            identity_provider_id: None,
            region_pack: "pack-alpha".to_string(),
            mfa_state: MfaState::Verified,
            last_authenticated_at_epoch_seconds: Some(1_700_000_002),
            created_at_epoch_seconds: 1_700_000_001,
        }
    }

    fn role_create() -> IamRoleCreate {
        IamRoleCreate {
            id: "role_compute_admin".to_string(),
            tenant_id: "ten_alpha".to_string(),
            region: "region-alpha1".to_string(),
            name: "compute-admin".to_string(),
            cedar_policy_id: "pol_cloud_compute_admin".to_string(),
            cedar_policy_version: "1.0.0".to_string(),
            assumable_by: vec!["sp_cloud_provisioner".to_string(), "usr_alice".to_string()],
            max_session_duration_sec: 900,
            data_class: DataClass::Public,
            created_at_epoch_seconds: 1_700_000_003,
        }
    }

    fn directory_with_role() -> IamDirectory {
        let mut directory = IamDirectory::default();
        directory
            .create_principal(service_principal_create())
            .expect("service principal registers");
        directory
            .create_principal(user_principal_create())
            .expect("user principal registers");
        directory
            .create_role(role_create())
            .expect("role registers");
        directory
    }

    #[test]
    fn registers_saml_provider_and_federated_principal_under_same_tenant() {
        let mut directory = IamDirectory::default();
        directory
            .register_identity_provider(provider_create())
            .expect("SAML provider registers");

        let principal = directory
            .create_principal(IamPrincipalCreate {
                id: "sp_federated_alice".to_string(),
                tenant_id: "ten_alpha".to_string(),
                kind: IamPrincipalKind::Federated,
                display_name: "Federated Alice".to_string(),
                external_subject: Some("saml://idp.alpha.example/alice".to_string()),
                identity_provider_id: Some("idp_alpha_saml".to_string()),
                region_pack: "pack-alpha".to_string(),
                mfa_state: MfaState::Verified,
                last_authenticated_at_epoch_seconds: Some(1_700_000_010),
                created_at_epoch_seconds: 1_700_000_005,
            })
            .expect("federated principal binds to provider");

        assert_eq!(principal.kind.value, IamPrincipalKind::Federated);
        assert_eq!(
            principal
                .external_subject
                .value
                .expect("subject captured")
                .value,
            "saml://idp.alpha.example/alice"
        );
    }

    #[test]
    fn creates_role_with_cedar_policy_and_assumes_sts_session_via_module_identity() {
        let mut directory = directory_with_role();

        let session = directory
            .assume_role(AssumeRoleRequest {
                session_id: "sts_compute_admin_001".to_string(),
                tenant_id: "ten_alpha".to_string(),
                role_id: "role_compute_admin".to_string(),
                assumed_by: "sp_cloud_provisioner".to_string(),
                external_id: None,
                requested_duration_sec: 600,
                scopes: vec![
                    "cloud.compute.write".to_string(),
                    "cloud.iam.read".to_string(),
                ],
                issued_at_epoch_seconds: 1_700_000_100,
            })
            .expect("trusted service principal can assume role");

        assert_eq!(session.assumed_role.value.value, "role_compute_admin");
        assert_eq!(session.expires_at_epoch_seconds.value, 1_700_000_700);
        assert!(session.token_fingerprint.value.starts_with("sts1:"));
    }

    #[test]
    fn rejects_role_without_cedar_policy_or_known_trusted_principal() {
        let policy_error = IamRole::new(IamRoleCreate {
            cedar_policy_id: "bad".to_string(),
            ..role_create()
        })
        .expect_err("role creation requires Cedar policy id");
        assert_eq!(policy_error, CloudIamError::InvalidCedarPolicyId);

        let mut directory = IamDirectory::default();
        directory
            .create_principal(service_principal_create())
            .expect("service principal registers");
        let unknown_error = directory
            .create_role(IamRoleCreate {
                assumable_by: vec!["sp_missing".to_string()],
                ..role_create()
            })
            .expect_err("trust policy principals must exist");
        assert_eq!(unknown_error, CloudIamError::UnknownPrincipal);
    }

    #[test]
    fn sts_rejects_sessions_longer_than_role_or_platform_limit() {
        let mut directory = directory_with_role();

        let role_limit_error = directory
            .assume_role(AssumeRoleRequest {
                session_id: "sts_too_long".to_string(),
                tenant_id: "ten_alpha".to_string(),
                role_id: "role_compute_admin".to_string(),
                assumed_by: "sp_cloud_provisioner".to_string(),
                external_id: None,
                requested_duration_sec: 901,
                scopes: vec!["cloud.compute.write".to_string()],
                issued_at_epoch_seconds: 1_700_000_100,
            })
            .expect_err("role max session duration is authoritative");
        assert_eq!(role_limit_error, CloudIamError::InvalidSessionDuration);

        let platform_limit_error = IamRole::new(IamRoleCreate {
            max_session_duration_sec: 3_601,
            ..role_create()
        })
        .expect_err("cloud IAM cannot exceed platform STS max TTL");
        assert_eq!(platform_limit_error, CloudIamError::InvalidSessionDuration);
    }

    #[test]
    fn sts_rejects_untrusted_principal_and_duplicate_session_id() {
        let mut directory = directory_with_role();
        directory
            .create_principal(IamPrincipalCreate {
                id: "sp_untrusted".to_string(),
                ..service_principal_create()
            })
            .expect("untrusted principal registers");

        let trust_error = directory
            .assume_role(AssumeRoleRequest {
                session_id: "sts_untrusted".to_string(),
                tenant_id: "ten_alpha".to_string(),
                role_id: "role_compute_admin".to_string(),
                assumed_by: "sp_untrusted".to_string(),
                external_id: None,
                requested_duration_sec: 300,
                scopes: vec!["cloud.iam.read".to_string()],
                issued_at_epoch_seconds: 1_700_000_100,
            })
            .expect_err("trust policy denies principal");
        assert_eq!(trust_error, CloudIamError::TrustPolicyDenied);

        let request = AssumeRoleRequest {
            session_id: "sts_duplicate".to_string(),
            tenant_id: "ten_alpha".to_string(),
            role_id: "role_compute_admin".to_string(),
            assumed_by: "sp_cloud_provisioner".to_string(),
            external_id: None,
            requested_duration_sec: 300,
            scopes: vec!["cloud.iam.read".to_string()],
            issued_at_epoch_seconds: 1_700_000_100,
        };
        directory
            .assume_role(request.clone())
            .expect("first session succeeds");
        let duplicate_error = directory
            .assume_role(request)
            .expect_err("session ids are idempotency keys");
        assert_eq!(duplicate_error, CloudIamError::DuplicateSession);
    }

    #[test]
    fn user_and_federated_assume_role_require_verified_mfa() {
        let mut directory = IamDirectory::default();
        directory
            .create_principal(IamPrincipalCreate {
                mfa_state: MfaState::Enrolled,
                ..user_principal_create()
            })
            .expect("user registers");
        directory
            .create_role(IamRoleCreate {
                assumable_by: vec!["usr_alice".to_string()],
                ..role_create()
            })
            .expect("role registers");

        let error = directory
            .assume_role(AssumeRoleRequest {
                session_id: "sts_mfa".to_string(),
                tenant_id: "ten_alpha".to_string(),
                role_id: "role_compute_admin".to_string(),
                assumed_by: "usr_alice".to_string(),
                external_id: None,
                requested_duration_sec: 300,
                scopes: vec!["cloud.iam.read".to_string()],
                issued_at_epoch_seconds: 1_700_000_100,
            })
            .expect_err("human principal must have verified MFA");

        assert_eq!(error, CloudIamError::MfaNotVerified);
    }

    #[test]
    fn external_principal_requires_external_id_when_assuming_role() {
        let mut directory = IamDirectory::default();
        directory
            .register_identity_provider(IdentityProviderCreate {
                id: "idp_partner_oidc".to_string(),
                kind: IdentityProviderKind::Oidc,
                issuer_uri: "https://partner.example/oidc".to_string(),
                verification_material_ref: "jwks/partner".to_string(),
                ..provider_create()
            })
            .expect("OIDC provider registers");
        directory
            .create_principal(IamPrincipalCreate {
                id: "sp_external_partner".to_string(),
                tenant_id: "ten_alpha".to_string(),
                kind: IamPrincipalKind::External,
                display_name: "Partner".to_string(),
                external_subject: Some("oidc://partner.example/sub-1".to_string()),
                identity_provider_id: Some("idp_partner_oidc".to_string()),
                region_pack: "pack-alpha".to_string(),
                mfa_state: MfaState::Verified,
                last_authenticated_at_epoch_seconds: Some(1_700_000_050),
                created_at_epoch_seconds: 1_700_000_040,
            })
            .expect("external principal registers");
        directory
            .create_role(IamRoleCreate {
                assumable_by: vec!["sp_external_partner".to_string()],
                ..role_create()
            })
            .expect("role registers");

        let error = directory
            .assume_role(AssumeRoleRequest {
                session_id: "sts_external".to_string(),
                tenant_id: "ten_alpha".to_string(),
                role_id: "role_compute_admin".to_string(),
                assumed_by: "sp_external_partner".to_string(),
                external_id: None,
                requested_duration_sec: 300,
                scopes: vec!["cloud.iam.read".to_string()],
                issued_at_epoch_seconds: 1_700_000_100,
            })
            .expect_err("external assume role requires external id");

        assert_eq!(error, CloudIamError::ExternalIdRequired);
    }

    #[test]
    fn rejects_non_cloud_scopes_and_non_public_role_metadata_class() {
        let scope_error = ScopeRef::new("foundry.invoke".to_string())
            .expect_err("cloud STS sessions carry cloud scopes only");
        assert_eq!(scope_error, CloudIamError::InvalidScope);

        let class_error = IamRole::new(IamRoleCreate {
            data_class: DataClass::Audit,
            ..role_create()
        })
        .expect_err("role metadata class must stay public privacy metadata");
        assert_eq!(class_error, CloudIamError::InvalidDataClass);
    }

    #[test]
    fn provider_identity_provider_sync_requests_validate_refs_and_receipts() {
        let identity_provider =
            IdentityProvider::new(provider_create()).expect("identity provider fixture is valid");
        let request = IamProviderIdentityProviderSyncRequest {
            request_id: "iam-idp-sync-001".to_string(),
            provider_identity_provider_ref: "oci-iam-idp://identity-domain-alpha/idp_alpha_saml"
                .to_string(),
            tenant_id: "ten_alpha".to_string(),
            actor: "sp_cloud_provisioner".to_string(),
            idempotency_key: "idem-iam-idp-sync-001".to_string(),
            requested_at_epoch_seconds: 1_700_000_020,
            operation: IamProviderIdentityProviderOperation::Upsert,
            identity_provider,
        };

        request.validate().expect("provider sync request is valid");
        assert_eq!(request.operation.label(), "upsert");
        assert_eq!(
            CloudIamProviderKind::OciIdentityDomain.label(),
            "oci_identity_domain"
        );

        let receipt = IamProviderIdentityProviderSyncReceipt::from_request(
            CloudIamProviderKind::OciIdentityDomain,
            request.clone(),
            "oci-iam-1700000020-iam-idp-sync-001",
            "oci-iam-idp://identity-domain-alpha/idp_alpha_saml/iam-idp-sync-001",
        )
        .expect("provider sync receipt is valid");

        assert_eq!(receipt.provider, CloudIamProviderKind::OciIdentityDomain);
        assert_eq!(
            receipt.sync_status,
            IamProviderIdentityProviderSyncStatus::Synchronized
        );
        assert_eq!(receipt.identity_provider_id, "idp_alpha_saml");
        assert_eq!(receipt.tenant_id, "ten_alpha");
        assert_eq!(receipt.occurred_at_epoch_seconds, 1_700_000_020);

        let mut mismatched_tenant = request.clone();
        mismatched_tenant.tenant_id = "ten_beta".to_string();
        assert_eq!(
            mismatched_tenant.validate(),
            Err(IamProviderIdentityProviderError::InvalidRequestShape(
                CloudIamError::ProviderTenantMismatch
            ))
        );

        let mut blank_provider_ref = request;
        blank_provider_ref.provider_identity_provider_ref = " ".to_string();
        assert_eq!(
            blank_provider_ref.validate(),
            Err(IamProviderIdentityProviderError::InvalidProviderIdentityProviderRef)
        );
    }

    #[test]
    fn idp_registry_sync_receipt_converts_to_metadata_only_evidence_event() {
        let identity_provider =
            IdentityProvider::new(provider_create()).expect("identity provider fixture is valid");
        let request = IamProviderIdentityProviderSyncRequest {
            request_id: "iam-idp-sync-001".to_string(),
            provider_identity_provider_ref: "oci-iam-idp://identity-domain-alpha/idp_alpha_saml"
                .to_string(),
            tenant_id: "ten_alpha".to_string(),
            actor: "sp_cloud_provisioner".to_string(),
            idempotency_key: "idem-iam-idp-sync-001".to_string(),
            requested_at_epoch_seconds: 1_700_000_020,
            operation: IamProviderIdentityProviderOperation::Upsert,
            identity_provider,
        };
        let receipt = IamProviderIdentityProviderSyncReceipt::from_request(
            CloudIamProviderKind::OciIdentityDomain,
            request,
            "oci-iam-1700000020-iam-idp-sync-001",
            "oci-iam-idp://identity-domain-alpha/idp_alpha_saml/iam-idp-sync-001",
        )
        .expect("provider sync receipt is valid");

        let event = IdentityProviderRegistryEvidenceEvent::from_sync_receipt(
            "ten_alpha",
            CloudIamProviderKind::OciIdentityDomain,
            receipt.clone(),
        )
        .expect("metadata-only evidence event is valid");

        assert_eq!(
            event.event_id(),
            "evt_cloud_iam_idp_registry_ten_alpha_oci_identity_domain_idem-iam-idp-sync-001"
        );
        assert_eq!(event.tenant_id(), "ten_alpha");
        assert_eq!(event.actor(), "sp_cloud_provisioner");
        assert_eq!(event.provider(), CloudIamProviderKind::OciIdentityDomain);
        assert_eq!(
            event.operation(),
            IamProviderIdentityProviderOperation::Upsert
        );
        assert_eq!(
            event.sync_status(),
            IamProviderIdentityProviderSyncStatus::Synchronized
        );
        assert_eq!(event.identity_provider_id(), "idp_alpha_saml");
        assert_eq!(
            event.provider_request_id(),
            "oci-iam-1700000020-iam-idp-sync-001"
        );
        assert_eq!(
            event.provider_evidence_ref(),
            "oci-iam-idp://identity-domain-alpha/idp_alpha_saml/iam-idp-sync-001"
        );
        assert_eq!(event.idempotency_key(), "idem-iam-idp-sync-001");
        assert_eq!(event.occurred_at_epoch_seconds(), 1_700_000_020);
        assert_eq!(
            event.schema_version(),
            IAM_PROVIDER_IDP_REGISTRY_EVIDENCE_SCHEMA_VERSION
        );

        let evidence_receipt = event.receipt();
        assert_eq!(evidence_receipt.event_id(), event.event_id());
        assert_eq!(evidence_receipt.tenant_id(), "ten_alpha");
        assert_eq!(
            evidence_receipt.provider(),
            CloudIamProviderKind::OciIdentityDomain
        );
        assert_eq!(
            evidence_receipt.provider_evidence_ref(),
            "oci-iam-idp://identity-domain-alpha/idp_alpha_saml/iam-idp-sync-001"
        );
        assert_eq!(evidence_receipt.idempotency_key(), "idem-iam-idp-sync-001");
        assert_eq!(
            evidence_receipt.schema_version(),
            IAM_PROVIDER_IDP_REGISTRY_EVIDENCE_SCHEMA_VERSION
        );

        let tenant_mismatch_error = IdentityProviderRegistryEvidenceEvent::from_sync_receipt(
            "ten_beta",
            CloudIamProviderKind::OciIdentityDomain,
            receipt.clone(),
        )
        .expect_err("evidence event is tenant-bound");
        assert_eq!(tenant_mismatch_error, CloudIamError::TenantMismatch);

        let provider_mismatch_error = IdentityProviderRegistryEvidenceEvent::from_sync_receipt(
            "ten_alpha",
            CloudIamProviderKind::SelfHostedOidcControlPlane,
            receipt.clone(),
        )
        .expect_err("evidence event is provider-bound");
        assert_eq!(provider_mismatch_error, CloudIamError::ProviderMismatch);

        let mut missing_evidence_ref = receipt.clone();
        missing_evidence_ref.provider_evidence_ref.clear();
        let missing_evidence_ref_error = IdentityProviderRegistryEvidenceEvent::from_sync_receipt(
            "ten_alpha",
            CloudIamProviderKind::OciIdentityDomain,
            missing_evidence_ref,
        )
        .expect_err("evidence event requires provider evidence ref");
        assert_eq!(
            missing_evidence_ref_error,
            CloudIamError::InvalidProviderEvidenceRef
        );

        let mut token_shaped_evidence_ref = receipt.clone();
        token_shaped_evidence_ref.provider_evidence_ref =
            "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhbGljZSJ9.signature".to_string();
        let token_shaped_evidence_ref_error =
            IdentityProviderRegistryEvidenceEvent::from_sync_receipt(
                "ten_alpha",
                CloudIamProviderKind::OciIdentityDomain,
                token_shaped_evidence_ref,
            )
            .expect_err("evidence event rejects token-shaped provider evidence");
        assert_eq!(
            token_shaped_evidence_ref_error,
            CloudIamError::InvalidProviderEvidenceRef
        );

        let mut schema_mismatch = receipt;
        schema_mismatch.schema_version = IAM_PROVIDER_IDP_SYNC_SCHEMA_VERSION + 1;
        let schema_mismatch_error = IdentityProviderRegistryEvidenceEvent::from_sync_receipt(
            "ten_alpha",
            CloudIamProviderKind::OciIdentityDomain,
            schema_mismatch,
        )
        .expect_err("evidence event rejects receipt schema drift");
        assert_eq!(
            schema_mismatch_error,
            CloudIamError::InvalidIdentityProviderRegistrySnapshotSchemaVersion
        );
    }

    #[test]
    fn idp_registry_snapshot_rejects_raw_material_and_duplicate_idempotency() {
        let identity_provider =
            IdentityProvider::new(provider_create()).expect("identity provider fixture is valid");
        let sync_request = IamProviderIdentityProviderSyncRequest {
            request_id: "iam-idp-sync-001".to_string(),
            provider_identity_provider_ref: "oci-iam-idp://identity-domain-alpha/idp_alpha_saml"
                .to_string(),
            tenant_id: "ten_alpha".to_string(),
            actor: "sp_cloud_provisioner".to_string(),
            idempotency_key: "idem-iam-idp-sync-001".to_string(),
            requested_at_epoch_seconds: 1_700_000_020,
            operation: IamProviderIdentityProviderOperation::Upsert,
            identity_provider,
        };
        let receipt = IamProviderIdentityProviderSyncReceipt::from_request(
            CloudIamProviderKind::OciIdentityDomain,
            sync_request.clone(),
            "oci-iam-1700000020-iam-idp-sync-001",
            "oci-iam-idp://identity-domain-alpha/idp_alpha_saml/iam-idp-sync-001",
        )
        .expect("provider sync receipt is valid");
        let token_shaped_evidence_error = IamProviderIdentityProviderSyncReceipt::from_request(
            CloudIamProviderKind::OciIdentityDomain,
            sync_request,
            "oci-iam-1700000020-iam-idp-sync-001",
            "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhbGljZSJ9.signature",
        )
        .expect_err("token-shaped provider evidence material is not an opaque evidence ref");
        assert_eq!(
            token_shaped_evidence_error,
            IamProviderIdentityProviderError::InvalidProviderEvidenceRef
        );

        let snapshot = IdentityProviderRegistrySnapshot::from_receipts(
            "idp-registry-snapshot-001",
            "ten_alpha",
            vec![receipt.clone()],
            IdentityProviderRegistryRawMaterialCounters::default(),
        )
        .expect("metadata-only snapshot is valid");

        let mut repository = InMemoryIdentityProviderRegistrySnapshotRepository::default();
        let commit = repository
            .persist_snapshot(snapshot.clone())
            .expect("metadata-only snapshot persists");

        assert_eq!(commit.snapshot_id, "idp-registry-snapshot-001");
        assert_eq!(commit.tenant_id, "ten_alpha");
        assert_eq!(commit.persisted_record_count, 1);
        assert_eq!(commit.max_occurred_at_epoch_seconds, 1_700_000_020);
        assert_eq!(commit.schema_version, IAM_PROVIDER_IDP_SYNC_SCHEMA_VERSION);
        assert_eq!(repository.snapshot_count(), 1);

        let record = repository
            .record("ten_alpha", "idp_alpha_saml")
            .expect("record persisted by tenant and provider");
        assert_eq!(record.provider, CloudIamProviderKind::OciIdentityDomain);
        assert_eq!(record.tenant_id, "ten_alpha");
        assert_eq!(record.identity_provider_id, "idp_alpha_saml");
        assert_eq!(record.identity_provider_kind, IdentityProviderKind::Saml);
        assert_eq!(record.region_pack, "pack-alpha");
        assert_eq!(record.issuer_ref, "https://idp.alpha.example/saml");
        assert_eq!(record.audience_ref, "urn:oyatie:cloud");
        assert_eq!(record.verification_material_ref, "cert/alpha-saml-signing");
        assert_eq!(
            record.provider_identity_provider_ref,
            "oci-iam-idp://identity-domain-alpha/idp_alpha_saml"
        );
        assert_eq!(
            record.provider_request_id,
            "oci-iam-1700000020-iam-idp-sync-001"
        );
        assert_eq!(record.actor, "sp_cloud_provisioner");
        assert_eq!(record.idempotency_key, "idem-iam-idp-sync-001");
        assert_eq!(
            record.provider_evidence_ref,
            "oci-iam-idp://identity-domain-alpha/idp_alpha_saml/iam-idp-sync-001"
        );
        assert_eq!(record.occurred_at_epoch_seconds, 1_700_000_020);
        assert_eq!(record.schema_version, IAM_PROVIDER_IDP_SYNC_SCHEMA_VERSION);
        assert_eq!(
            record.operation,
            IamProviderIdentityProviderOperation::Upsert
        );
        assert_eq!(
            record.sync_status,
            IamProviderIdentityProviderSyncStatus::Synchronized
        );
        assert_eq!(record.raw_provider_document_bytes, 0);
        assert_eq!(record.credential_material_bytes, 0);
        assert_eq!(record.assertion_material_bytes, 0);
        assert_eq!(record.sts_material_bytes, 0);

        let duplicate_error = repository
            .persist_snapshot(snapshot)
            .expect_err("duplicate idempotency key cannot be persisted twice");
        assert_eq!(
            duplicate_error,
            CloudIamError::DuplicateIdentityProviderRegistrySnapshot
        );

        let second_identity_provider = IdentityProvider::new(IdentityProviderCreate {
            id: "idp_beta_saml".to_string(),
            issuer_uri: "https://idp.beta.example/saml".to_string(),
            verification_material_ref: "cert/beta-saml-signing".to_string(),
            ..provider_create()
        })
        .expect("second identity provider fixture is valid");
        let reused_idempotency_request = IamProviderIdentityProviderSyncRequest {
            request_id: "iam-idp-sync-002".to_string(),
            provider_identity_provider_ref: "oci-iam-idp://identity-domain-alpha/idp_beta_saml"
                .to_string(),
            tenant_id: "ten_alpha".to_string(),
            actor: "sp_cloud_provisioner".to_string(),
            idempotency_key: "idem-iam-idp-sync-001".to_string(),
            requested_at_epoch_seconds: 1_700_000_021,
            operation: IamProviderIdentityProviderOperation::Upsert,
            identity_provider: second_identity_provider,
        };
        let reused_idempotency_receipt = IamProviderIdentityProviderSyncReceipt::from_request(
            CloudIamProviderKind::OciIdentityDomain,
            reused_idempotency_request,
            "oci-iam-1700000021-iam-idp-sync-002",
            "oci-iam-idp://identity-domain-alpha/idp_beta_saml/iam-idp-sync-002",
        )
        .expect("second provider sync receipt is valid");
        let reused_idempotency_snapshot = IdentityProviderRegistrySnapshot::from_receipts(
            "idp-registry-snapshot-reused-idempotency",
            "ten_alpha",
            vec![reused_idempotency_receipt],
            IdentityProviderRegistryRawMaterialCounters::default(),
        )
        .expect("second snapshot has a different snapshot id and provider record");
        let reused_idempotency_error = repository
            .persist_snapshot(reused_idempotency_snapshot)
            .expect_err("idempotency key reuse is rejected independently of snapshot id");
        assert_eq!(
            reused_idempotency_error,
            CloudIamError::DuplicateIdentityProviderRegistrySnapshot
        );

        let leak_cases = [
            IdentityProviderRegistryRawMaterialCounters {
                raw_provider_document_bytes: 1,
                ..IdentityProviderRegistryRawMaterialCounters::default()
            },
            IdentityProviderRegistryRawMaterialCounters {
                credential_material_bytes: 1,
                ..IdentityProviderRegistryRawMaterialCounters::default()
            },
            IdentityProviderRegistryRawMaterialCounters {
                assertion_material_bytes: 1,
                ..IdentityProviderRegistryRawMaterialCounters::default()
            },
            IdentityProviderRegistryRawMaterialCounters {
                sts_material_bytes: 1,
                ..IdentityProviderRegistryRawMaterialCounters::default()
            },
        ];

        for (index, counters) in leak_cases.into_iter().enumerate() {
            let leaky_snapshot = IdentityProviderRegistrySnapshot::from_receipts(
                format!("idp-registry-snapshot-raw-{index}"),
                "ten_alpha",
                vec![receipt.clone()],
                counters,
            )
            .expect_err("raw provider documents, credentials, assertions, and STS are forbidden");
            assert_eq!(
                leaky_snapshot,
                CloudIamError::IdentityProviderRegistryRawMaterialForbidden
            );
        }

        let empty_snapshot_error = IdentityProviderRegistrySnapshot::from_receipts(
            "idp-registry-snapshot-empty",
            "ten_alpha",
            Vec::new(),
            IdentityProviderRegistryRawMaterialCounters::default(),
        )
        .expect_err("empty registry snapshots cannot create false evidence");
        assert_eq!(
            empty_snapshot_error,
            CloudIamError::EmptyIdentityProviderRegistrySnapshot
        );

        let tenant_mismatch_error = IdentityProviderRegistrySnapshot::from_receipts(
            "idp-registry-snapshot-wrong-tenant",
            "ten_beta",
            vec![receipt.clone()],
            IdentityProviderRegistryRawMaterialCounters::default(),
        )
        .expect_err("registry snapshots are tenant-bound");
        assert_eq!(tenant_mismatch_error, CloudIamError::TenantMismatch);

        let mut duplicate_record_receipt = receipt.clone();
        duplicate_record_receipt.idempotency_key = "idem-iam-idp-sync-002".to_string();
        let duplicate_record_error = IdentityProviderRegistrySnapshot::from_receipts(
            "idp-registry-snapshot-duplicate-record",
            "ten_alpha",
            vec![receipt.clone(), duplicate_record_receipt],
            IdentityProviderRegistryRawMaterialCounters::default(),
        )
        .expect_err("same provider record cannot appear twice in one snapshot");
        assert_eq!(
            duplicate_record_error,
            CloudIamError::DuplicateIdentityProviderRegistryRecord
        );

        let public_snapshot_template = IdentityProviderRegistrySnapshot::from_receipts(
            "idp-registry-snapshot-public-template",
            "ten_alpha",
            vec![receipt.clone()],
            IdentityProviderRegistryRawMaterialCounters::default(),
        )
        .expect("template snapshot starts valid");

        let mut public_empty_snapshot = public_snapshot_template.clone();
        public_empty_snapshot.snapshot_id = "idp-registry-snapshot-public-empty".to_string();
        public_empty_snapshot.records.clear();
        let public_empty_error = InMemoryIdentityProviderRegistrySnapshotRepository::default()
            .persist_snapshot(public_empty_snapshot)
            .expect_err("repository rechecks non-empty invariant on public snapshots");
        assert_eq!(
            public_empty_error,
            CloudIamError::EmptyIdentityProviderRegistrySnapshot
        );

        let mut public_tenant_mismatch = public_snapshot_template.clone();
        public_tenant_mismatch.snapshot_id =
            "idp-registry-snapshot-public-tenant-mismatch".to_string();
        public_tenant_mismatch.records[0].tenant_id = "ten_beta".to_string();
        let public_tenant_mismatch_error =
            InMemoryIdentityProviderRegistrySnapshotRepository::default()
                .persist_snapshot(public_tenant_mismatch)
                .expect_err("repository rechecks record tenant against snapshot tenant");
        assert_eq!(public_tenant_mismatch_error, CloudIamError::TenantMismatch);

        let mut public_duplicate_idempotency = public_snapshot_template.clone();
        public_duplicate_idempotency.snapshot_id =
            "idp-registry-snapshot-public-duplicate-idempotency".to_string();
        let mut second_public_record = public_duplicate_idempotency.records[0].clone();
        second_public_record.identity_provider_id = "idp_gamma_saml".to_string();
        public_duplicate_idempotency
            .records
            .push(second_public_record);
        let public_duplicate_idempotency_error =
            InMemoryIdentityProviderRegistrySnapshotRepository::default()
                .persist_snapshot(public_duplicate_idempotency)
                .expect_err("repository rechecks duplicate idempotency in public snapshots");
        assert_eq!(
            public_duplicate_idempotency_error,
            CloudIamError::DuplicateIdentityProviderRegistrySnapshot
        );

        let mut public_duplicate_record = public_snapshot_template.clone();
        public_duplicate_record.snapshot_id =
            "idp-registry-snapshot-public-duplicate-record".to_string();
        let mut duplicate_public_record = public_duplicate_record.records[0].clone();
        duplicate_public_record.idempotency_key = "idem-iam-idp-sync-public-duplicate".to_string();
        public_duplicate_record
            .records
            .push(duplicate_public_record);
        let public_duplicate_record_error =
            InMemoryIdentityProviderRegistrySnapshotRepository::default()
                .persist_snapshot(public_duplicate_record)
                .expect_err("repository rechecks duplicate provider records in public snapshots");
        assert_eq!(
            public_duplicate_record_error,
            CloudIamError::DuplicateIdentityProviderRegistryRecord
        );

        let mut public_invalid_snapshot_id = public_snapshot_template;
        public_invalid_snapshot_id.snapshot_id = " ".to_string();
        let public_invalid_snapshot_id_error =
            InMemoryIdentityProviderRegistrySnapshotRepository::default()
                .persist_snapshot(public_invalid_snapshot_id)
                .expect_err("repository rechecks snapshot id format on public snapshots");
        assert_eq!(
            public_invalid_snapshot_id_error,
            CloudIamError::InvalidIdentityProviderRegistrySnapshotId
        );

        let mut public_invalid_snapshot_schema = IdentityProviderRegistrySnapshot::from_receipts(
            "idp-registry-snapshot-public-invalid-schema",
            "ten_alpha",
            vec![receipt.clone()],
            IdentityProviderRegistryRawMaterialCounters::default(),
        )
        .expect("template snapshot starts valid");
        public_invalid_snapshot_schema.schema_version = IAM_PROVIDER_IDP_SYNC_SCHEMA_VERSION + 1;
        let public_invalid_snapshot_schema_error =
            InMemoryIdentityProviderRegistrySnapshotRepository::default()
                .persist_snapshot(public_invalid_snapshot_schema)
                .expect_err("repository rechecks snapshot schema version");
        assert_eq!(
            public_invalid_snapshot_schema_error,
            CloudIamError::InvalidIdentityProviderRegistrySnapshotSchemaVersion
        );

        let mut public_invalid_record_schema = IdentityProviderRegistrySnapshot::from_receipts(
            "idp-registry-snapshot-public-invalid-record-schema",
            "ten_alpha",
            vec![receipt.clone()],
            IdentityProviderRegistryRawMaterialCounters::default(),
        )
        .expect("template snapshot starts valid");
        public_invalid_record_schema.records[0].schema_version =
            IAM_PROVIDER_IDP_SYNC_SCHEMA_VERSION + 1;
        let public_invalid_record_schema_error =
            InMemoryIdentityProviderRegistrySnapshotRepository::default()
                .persist_snapshot(public_invalid_record_schema)
                .expect_err("repository rechecks record schema version");
        assert_eq!(
            public_invalid_record_schema_error,
            CloudIamError::InvalidIdentityProviderRegistrySnapshotSchemaVersion
        );

        let mut public_token_shaped_evidence_ref = IdentityProviderRegistrySnapshot::from_receipts(
            "idp-registry-snapshot-public-token-evidence-ref",
            "ten_alpha",
            vec![receipt.clone()],
            IdentityProviderRegistryRawMaterialCounters::default(),
        )
        .expect("template snapshot starts valid");
        public_token_shaped_evidence_ref.records[0].provider_evidence_ref =
            "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhbGljZSJ9.signature".to_string();
        let public_token_shaped_evidence_ref_error =
            InMemoryIdentityProviderRegistrySnapshotRepository::default()
                .persist_snapshot(public_token_shaped_evidence_ref)
                .expect_err("repository rejects token-shaped evidence refs on public snapshots");
        assert_eq!(
            public_token_shaped_evidence_ref_error,
            CloudIamError::InvalidProviderEvidenceRef
        );

        let mut tampered_snapshot = IdentityProviderRegistrySnapshot::from_receipts(
            "idp-registry-snapshot-tampered",
            "ten_alpha",
            vec![receipt],
            IdentityProviderRegistryRawMaterialCounters::default(),
        )
        .expect("metadata-only snapshot starts valid");
        tampered_snapshot.records[0].credential_material_bytes = 1;
        let tampered_error = InMemoryIdentityProviderRegistrySnapshotRepository::default()
            .persist_snapshot(tampered_snapshot)
            .expect_err("repository rejects post-construction raw-material mutation");
        assert_eq!(
            tampered_error,
            CloudIamError::IdentityProviderRegistryRawMaterialForbidden
        );
    }
}
