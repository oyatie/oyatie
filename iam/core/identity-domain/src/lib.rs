//! Identity kernel: user records and purpose-bound short-lived tokens.

pub mod hr_employee_status;
pub use hr_employee_status::{EmployeeStatus, UnknownEmployeeStatus};

pub mod identity_employment_classification;
pub use identity_employment_classification::{
    EmploymentClassification, UnknownEmploymentClassification,
};

pub mod credential_revocation;
pub use credential_revocation::{
    CredentialStatus, RevocationError, RevocationLedger, RevocationReason, UnknownRevocationReason,
    token_fingerprint,
};

use std::fmt;

use oya_data_boundary_kernel::{Classified, DataClass, Purpose};

pub const MAX_TOKEN_TTL_SECONDS: u64 = 60 * 60;
pub const USER_SCHEMA_VERSION: u32 = 1;
pub const IDP_BINDING_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct UserId(String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdpBinding {
    pub region_pack: Classified<String>, // data_class: INTERNAL_ONLY
    pub identity_provider_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub external_subject: Classified<String>, // data_class: PII_IDENTIFYING
    pub bound_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct User {
    pub tenant_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub id: Classified<UserId>,        // data_class: PII_IDENTIFYING
    pub primary_identifier: Classified<String>, // data_class: PII_IDENTIFYING
    pub display_name: Classified<String>, // data_class: PII_QUASI_IDENTIFIER
    pub roles: Classified<Vec<String>>, // data_class: INTERNAL_ONLY
    pub idp_binding: Classified<IdpBinding>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    pub tenant_id: String,
    pub user_id: String,
    pub purpose: Purpose,
    pub issued_at_epoch_seconds: u64,
    pub expires_at_epoch_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Principal {
    Human {
        tenant_id: Classified<String>,
        user_id: Classified<String>,
    },
    ServicePrincipal {
        tenant_id: Classified<String>,
        service_principal_id: Classified<String>,
        owning_capability_id: Classified<String>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialRequestKind {
    Sts,
    LongLivedApiKey,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CredentialRequest {
    pub principal: Principal,
    pub kind: CredentialRequestKind,
    pub purpose: Purpose, // data_class: INTERNAL_ONLY
    pub scopes: Vec<String>,
    pub ttl_seconds: u64,
    pub issued_at_epoch_seconds: u64,
}

#[derive(Clone, Eq, PartialEq)]
pub struct StsCredential {
    pub principal: Principal,
    pub purpose: Classified<Purpose>, // data_class: INTERNAL_ONLY
    pub scopes: Classified<Vec<String>>,
    pub issued_at_epoch_seconds: Classified<u64>,
    pub expires_at_epoch_seconds: Classified<u64>,
    pub token_fingerprint: Classified<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IdentityError {
    InvalidTenantId,
    InvalidUserId,
    InvalidRegionPack,
    InvalidIdentityProviderId,
    InvalidServicePrincipalId,
    InvalidCapabilityId,
    EmptyPrimaryIdentifier,
    EmptyExternalSubject,
    TokenTtlTooLong,
    TokenTtlZero,
    MissingCredentialScope,
    LongLivedCredentialForbidden,
}

impl UserId {
    pub fn new(value: impl Into<String>) -> Result<Self, IdentityError> {
        let value = value.into();
        validate_user_id(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl IdpBinding {
    pub fn new(
        region_pack: String,
        identity_provider_id: String,
        external_subject: String,
        bound_at_epoch_seconds: u64,
    ) -> Result<Self, IdentityError> {
        validate_region_pack(&region_pack)?;
        validate_identity_provider_id(&identity_provider_id)?;
        if external_subject.trim().is_empty() {
            return Err(IdentityError::EmptyExternalSubject);
        }

        Ok(Self {
            region_pack: Classified::new(region_pack, DataClass::InternalOnly),
            identity_provider_id: Classified::new(identity_provider_id, DataClass::InternalOnly),
            external_subject: Classified::new(external_subject, DataClass::PiiIdentifying),
            bound_at_epoch_seconds: Classified::new(
                bound_at_epoch_seconds,
                DataClass::InternalOnly,
            ),
            schema_version: Classified::new(IDP_BINDING_SCHEMA_VERSION, DataClass::Public),
        })
    }
}

impl User {
    pub fn new(
        tenant_id: String,
        id: String,
        primary_identifier: String,
        display_name: String,
        roles: Vec<String>,
        idp_binding: IdpBinding,
    ) -> Result<Self, IdentityError> {
        validate_tenant_id(&tenant_id)?;
        let id = UserId::new(id)?;
        if primary_identifier.trim().is_empty() {
            return Err(IdentityError::EmptyPrimaryIdentifier);
        }
        Ok(Self {
            tenant_id: Classified::new(tenant_id, DataClass::InternalOnly),
            id: Classified::new(id, DataClass::PiiIdentifying),
            primary_identifier: Classified::new(primary_identifier, DataClass::PiiIdentifying),
            display_name: Classified::new(display_name, DataClass::PiiQuasiIdentifier),
            roles: Classified::new(roles, DataClass::InternalOnly),
            idp_binding: Classified::new(idp_binding, DataClass::InternalOnly),
            schema_version: Classified::new(USER_SCHEMA_VERSION, DataClass::Public),
        })
    }

    pub fn tenant_id(&self) -> &str {
        &self.tenant_id.value
    }

    pub fn user_id(&self) -> &UserId {
        &self.id.value
    }

    pub fn idp_binding(&self) -> &IdpBinding {
        &self.idp_binding.value
    }
}

impl Principal {
    pub fn human(tenant_id: String, user_id: String) -> Result<Self, IdentityError> {
        validate_tenant_id(&tenant_id)?;
        validate_user_id(&user_id)?;
        Ok(Self::Human {
            tenant_id: Classified::new(tenant_id, DataClass::InternalOnly),
            user_id: Classified::new(user_id, DataClass::PiiIdentifying),
        })
    }

    pub fn service(
        tenant_id: String,
        service_principal_id: String,
        owning_capability_id: String,
    ) -> Result<Self, IdentityError> {
        validate_tenant_id(&tenant_id)?;
        if !service_principal_id.starts_with("sp_") || service_principal_id.len() <= 3 {
            return Err(IdentityError::InvalidServicePrincipalId);
        }
        if !owning_capability_id.starts_with("cap.") {
            return Err(IdentityError::InvalidCapabilityId);
        }
        Ok(Self::ServicePrincipal {
            tenant_id: Classified::new(tenant_id, DataClass::InternalOnly),
            service_principal_id: Classified::new(service_principal_id, DataClass::InternalOnly),
            owning_capability_id: Classified::new(owning_capability_id, DataClass::InternalOnly),
        })
    }

    fn fingerprint_material(&self) -> String {
        match self {
            Self::Human { tenant_id, user_id } => {
                format!("human:{}:{}", tenant_id.value, user_id.value)
            }
            Self::ServicePrincipal {
                tenant_id,
                service_principal_id,
                owning_capability_id,
            } => format!(
                "service:{}:{}:{}",
                tenant_id.value, service_principal_id.value, owning_capability_id.value
            ),
        }
    }
}

impl StsCredential {
    pub fn is_active(&self, now_epoch_seconds: u64) -> bool {
        now_epoch_seconds < self.expires_at_epoch_seconds.value
    }
}

impl fmt::Debug for StsCredential {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StsCredential")
            .field("principal", &self.principal)
            .field("purpose", &self.purpose)
            .field("scopes", &self.scopes)
            .field("issued_at_epoch_seconds", &self.issued_at_epoch_seconds)
            .field("expires_at_epoch_seconds", &self.expires_at_epoch_seconds)
            .field("token_fingerprint", &self.token_fingerprint.value)
            .finish()
    }
}

pub fn issue_token(
    tenant_id: String,
    user_id: String,
    purpose: Purpose,
    ttl_seconds: u64,
    issued_at_epoch_seconds: u64,
) -> Result<Token, IdentityError> {
    validate_tenant_id(&tenant_id)?;
    if ttl_seconds == 0 {
        return Err(IdentityError::TokenTtlZero);
    }
    if ttl_seconds > MAX_TOKEN_TTL_SECONDS {
        return Err(IdentityError::TokenTtlTooLong);
    }
    Ok(Token {
        tenant_id,
        user_id,
        purpose,
        issued_at_epoch_seconds,
        expires_at_epoch_seconds: issued_at_epoch_seconds + ttl_seconds,
    })
}

pub fn issue_credential(request: CredentialRequest) -> Result<StsCredential, IdentityError> {
    if request.kind == CredentialRequestKind::LongLivedApiKey {
        return Err(IdentityError::LongLivedCredentialForbidden);
    }
    if request.scopes.is_empty() || request.scopes.iter().any(|scope| scope.trim().is_empty()) {
        return Err(IdentityError::MissingCredentialScope);
    }
    let token = issue_token(
        principal_tenant_id(&request.principal).to_string(),
        principal_subject_id(&request.principal).to_string(),
        request.purpose,
        request.ttl_seconds,
        request.issued_at_epoch_seconds,
    )?;
    let fingerprint = credential_fingerprint(&request.principal, &request.scopes, &token);
    Ok(StsCredential {
        principal: request.principal,
        purpose: Classified::new(token.purpose, DataClass::InternalOnly),
        scopes: Classified::new(request.scopes, DataClass::InternalOnly),
        issued_at_epoch_seconds: Classified::new(
            token.issued_at_epoch_seconds,
            DataClass::InternalOnly,
        ),
        expires_at_epoch_seconds: Classified::new(
            token.expires_at_epoch_seconds,
            DataClass::InternalOnly,
        ),
        token_fingerprint: Classified::new(fingerprint, DataClass::InternalOnly),
    })
}

fn validate_tenant_id(tenant_id: &str) -> Result<(), IdentityError> {
    if tenant_id.starts_with("ten_") && tenant_id.len() > 4 {
        Ok(())
    } else {
        Err(IdentityError::InvalidTenantId)
    }
}

fn validate_user_id(user_id: &str) -> Result<(), IdentityError> {
    if user_id.starts_with("usr_") && user_id.len() > 4 {
        Ok(())
    } else {
        Err(IdentityError::InvalidUserId)
    }
}

fn validate_region_pack(region_pack: &str) -> Result<(), IdentityError> {
    if region_pack.starts_with("oya-pack-") && region_pack.len() > "oya-pack-".len() {
        Ok(())
    } else {
        Err(IdentityError::InvalidRegionPack)
    }
}

fn validate_identity_provider_id(identity_provider_id: &str) -> Result<(), IdentityError> {
    if identity_provider_id.starts_with("idp_") && identity_provider_id.len() > 4 {
        Ok(())
    } else {
        Err(IdentityError::InvalidIdentityProviderId)
    }
}

fn principal_tenant_id(principal: &Principal) -> &str {
    match principal {
        Principal::Human { tenant_id, .. } | Principal::ServicePrincipal { tenant_id, .. } => {
            &tenant_id.value
        }
    }
}

fn principal_subject_id(principal: &Principal) -> &str {
    match principal {
        Principal::Human { user_id, .. } => &user_id.value,
        Principal::ServicePrincipal {
            service_principal_id,
            ..
        } => &service_principal_id.value,
    }
}

fn credential_fingerprint(principal: &Principal, scopes: &[String], token: &Token) -> String {
    let mut state = 0xcbf29ce484222325_u64;
    fn feed(state: &mut u64, bytes: &[u8]) {
        for byte in bytes {
            *state ^= u64::from(*byte);
            *state = state.wrapping_mul(0x100000001b3);
        }
    }
    feed(&mut state, principal.fingerprint_material().as_bytes());
    feed(&mut state, token.purpose.pascal_label().as_bytes());
    for scope in scopes {
        feed(&mut state, scope.as_bytes());
    }
    feed(
        &mut state,
        token.issued_at_epoch_seconds.to_string().as_bytes(),
    );
    feed(
        &mut state,
        token.expires_at_epoch_seconds.to_string().as_bytes(),
    );
    format!("sts1:{state:016x}")
}
