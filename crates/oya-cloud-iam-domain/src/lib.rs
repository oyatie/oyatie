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

use oya_cloud_region_domain::RegionCode;
use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass, Purpose};
use oya_identity_domain::{
    CredentialRequest, CredentialRequestKind, IdentityError, MAX_TOKEN_TTL_SECONDS, Principal,
    issue_credential,
};

const IAM_PRINCIPAL_SCHEMA_VERSION: u32 = 1;
const IAM_ROLE_SCHEMA_VERSION: u32 = 1;
const STS_SESSION_SCHEMA_VERSION: u32 = 1;
const IDENTITY_PROVIDER_SCHEMA_VERSION: u32 = 1;
const IAM_PROVIDER_IDP_SYNC_SCHEMA_VERSION: u32 = 1;
const TENANT_ID_PREFIX: &str = "ten_";
const USER_ID_PREFIX: &str = "usr_";
const SERVICE_PRINCIPAL_PREFIX: &str = "sp_";
const ROLE_ID_PREFIX: &str = "role_";
const IDENTITY_PROVIDER_PREFIX: &str = "idp_";
const CEDAR_POLICY_PREFIX: &str = "pol_";
const STS_SESSION_PREFIX: &str = "sts_";
const REGIONAL_PACK_PREFIX: &str = "oya-pack-";
const CERT_REF_PREFIX: &str = "cert/";
const JWKS_REF_PREFIX: &str = "jwks/";
const CLOUD_SCOPE_PREFIX: &str = "cloud.";
const CLOUD_IAM_CAPABILITY_ID: &str = "cap.cloud.iam";

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
    pub provider_identity_provider_ref: String, // data_class: INTERNAL_ONLY
    pub actor: String,                  // data_class: INTERNAL_ONLY
    pub idempotency_key: String,        // data_class: INTERNAL_ONLY
    pub provider_evidence_ref: String,  // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub schema_version: u32,            // data_class: PUBLIC
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
        validate_provider_ref(
            &provider_evidence_ref,
            IamProviderIdentityProviderError::InvalidProviderEvidenceRef,
        )?;
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
            provider_identity_provider_ref: input.provider_identity_provider_ref,
            actor: input.actor,
            idempotency_key: input.idempotency_key,
            provider_evidence_ref,
            occurred_at_epoch_seconds: input.requested_at_epoch_seconds,
            schema_version: IAM_PROVIDER_IDP_SYNC_SCHEMA_VERSION,
        })
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
            region_pack: "oya-pack-alpha".to_string(),
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
            region_pack: "oya-pack-alpha".to_string(),
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
            region_pack: "oya-pack-alpha".to_string(),
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
                region_pack: "oya-pack-alpha".to_string(),
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
    fn creates_role_with_cedar_policy_and_assumes_sts_session_via_platform_identity() {
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
                region_pack: "oya-pack-alpha".to_string(),
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
}
