//! A human credential is authorized for one active IAM tenant membership.
//!
//! The shell consumes only [`UserSessionAuthorizer`]. Credential verification
//! and the membership lookup are separate IAM-owned ports so a missing or
//! failed store can never be interpreted as a permitted user. Membership is
//! not a Foundry role grant; product permissions still require their PDP.

#![forbid(unsafe_code)]

use std::future::Future;
use std::pin::Pin;

use iam_identity_domain::{Principal, provisioned_role_is_valid};

mod session_view;
pub use session_view::SessionView;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SessionRefusal {
    Unauthenticated,
    InvalidCredential,
    Expired,
    InvalidSubject,
    TenantMismatch,
    MissingMembership,
    InactiveMembership,
    AmbiguousMembership,
    MembershipUnavailable,
}

/// Output of an IAM human credential verifier. The tenant and canonical user
/// id must come from the verified credential or its server-side binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthenticatedSubject {
    user_id: String,
    tenant_id: String,
    expires_at_epoch_seconds: i64,
}

impl AuthenticatedSubject {
    pub fn new(
        user_id: impl Into<String>,
        tenant_id: impl Into<String>,
        expires_at_epoch_seconds: i64,
    ) -> Result<Self, SessionRefusal> {
        let user_id = user_id.into();
        let tenant_id = tenant_id.into();
        Principal::human(tenant_id.clone(), user_id.clone())
            .map_err(|_| SessionRefusal::InvalidSubject)?;
        Ok(Self {
            user_id,
            tenant_id,
            expires_at_epoch_seconds,
        })
    }
}

/// Only a credential verifier may produce the input to membership resolution.
/// A workload or operator bearer must not implement this human-user port.
pub trait UserCredentialVerifier: Send + Sync {
    fn verify<'a>(
        &'a self,
        presented: Option<&'a str>,
        expected_tenant: &'a str,
        now_epoch_seconds: i64,
    ) -> Pin<Box<dyn Future<Output = Result<AuthenticatedSubject, SessionRefusal>> + Send + 'a>>;
}

/// One provisioned account binding from the IAM membership store.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MembershipRecord {
    pub user_id: String,
    pub tenant_id: String,
    pub active: bool,
    /// Role identifiers explicitly provisioned by IAM. Foundry's own policy
    /// engine still decides whether any role permits an Ontology operation.
    pub roles: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MembershipStoreError {
    Unavailable,
    Corrupt,
}

/// A fallible read is required: the SCIM `UserStore` read signatures cannot
/// distinguish absence from a storage outage and are not used for sign-in.
pub trait UserMembershipStore: Send + Sync {
    fn candidates<'a>(
        &'a self,
        tenant_id: &'a str,
        user_id: &'a str,
    ) -> Pin<
        Box<dyn Future<Output = Result<Vec<MembershipRecord>, MembershipStoreError>> + Send + 'a>,
    >;
}

/// The only identity result the shell may retain. No credential or product
/// permission is carried. Re-authorize through [`UserSessionAuthorizer`] for
/// every live use; [`Self::ensure_current`] checks expiry only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveUserMembership {
    user_id: String,
    tenant_id: String,
    expires_at_epoch_seconds: i64,
    roles: Vec<String>,
}

impl ActiveUserMembership {
    fn from_verified_subject(
        subject: AuthenticatedSubject,
        roles: Vec<String>,
        expected_tenant: &str,
        now_epoch_seconds: i64,
    ) -> Result<Self, SessionRefusal> {
        validate_subject(&subject, expected_tenant, now_epoch_seconds)?;
        let mut seen = std::collections::BTreeSet::new();
        if roles
            .iter()
            .any(|role| !provisioned_role_is_valid(role) || !seen.insert(role))
        {
            return Err(SessionRefusal::MembershipUnavailable);
        }
        Ok(Self {
            user_id: subject.user_id,
            tenant_id: subject.tenant_id,
            expires_at_epoch_seconds: subject.expires_at_epoch_seconds,
            roles,
        })
    }

    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    pub fn expires_at_epoch_seconds(&self) -> i64 {
        self.expires_at_epoch_seconds
    }

    pub fn roles(&self) -> &[String] {
        &self.roles
    }

    pub fn ensure_current(&self, now_epoch_seconds: i64) -> Result<(), SessionRefusal> {
        if now_epoch_seconds >= self.expires_at_epoch_seconds {
            Err(SessionRefusal::Expired)
        } else {
            Ok(())
        }
    }
}

pub trait UserSessionAuthorizer: Send + Sync {
    fn authorize<'a>(
        &'a self,
        presented: Option<&'a str>,
        expected_tenant: &'a str,
        now_epoch_seconds: i64,
    ) -> Pin<Box<dyn Future<Output = Result<ActiveUserMembership, SessionRefusal>> + Send + 'a>>;
}

/// Common fail-closed ordering for a credential verifier and fallible native
/// membership store. The concrete credential method remains an adapter choice.
pub struct MembershipSessionAuthorizer<V, S> {
    verifier: V,
    store: S,
}

impl<V, S> MembershipSessionAuthorizer<V, S> {
    pub fn new(verifier: V, store: S) -> Self {
        Self { verifier, store }
    }
}

impl<V: UserCredentialVerifier, S: UserMembershipStore> UserSessionAuthorizer
    for MembershipSessionAuthorizer<V, S>
{
    fn authorize<'a>(
        &'a self,
        presented: Option<&'a str>,
        expected_tenant: &'a str,
        now_epoch_seconds: i64,
    ) -> Pin<Box<dyn Future<Output = Result<ActiveUserMembership, SessionRefusal>> + Send + 'a>>
    {
        Box::pin(async move {
            let subject = self
                .verifier
                .verify(presented, expected_tenant, now_epoch_seconds)
                .await?;
            validate_subject(&subject, expected_tenant, now_epoch_seconds)?;
            let candidates = self
                .store
                .candidates(&subject.tenant_id, &subject.user_id)
                .await
                .map_err(|_| SessionRefusal::MembershipUnavailable)?;
            let [record] = candidates.as_slice() else {
                return Err(if candidates.is_empty() {
                    SessionRefusal::MissingMembership
                } else {
                    SessionRefusal::AmbiguousMembership
                });
            };
            if record.tenant_id != subject.tenant_id || record.user_id != subject.user_id {
                return Err(SessionRefusal::MembershipUnavailable);
            }
            if !record.active {
                return Err(SessionRefusal::InactiveMembership);
            }
            ActiveUserMembership::from_verified_subject(
                subject,
                record.roles.clone(),
                expected_tenant,
                now_epoch_seconds,
            )
        })
    }
}

fn validate_subject(
    subject: &AuthenticatedSubject,
    expected_tenant: &str,
    now_epoch_seconds: i64,
) -> Result<(), SessionRefusal> {
    if now_epoch_seconds >= subject.expires_at_epoch_seconds {
        return Err(SessionRefusal::Expired);
    }
    if subject.tenant_id != expected_tenant {
        return Err(SessionRefusal::TenantMismatch);
    }
    Ok(())
}
