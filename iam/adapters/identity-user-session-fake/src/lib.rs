//! Hermetic fake adapters for the IAM user session port. Never compose these
//! into a serving sign-in route.

#![forbid(unsafe_code)]

use std::future::Future;
use std::pin::Pin;

use iam_identity_user_session::{
    AuthenticatedSubject, MembershipRecord, MembershipStoreError, SessionRefusal,
    UserCredentialVerifier, UserMembershipStore,
};

/// One exact fixture credential. This has no production credential authority.
pub struct FixtureCredentialVerifier {
    credential: String,
    subject: AuthenticatedSubject,
}

impl FixtureCredentialVerifier {
    pub fn new(credential: impl Into<String>, subject: AuthenticatedSubject) -> Self {
        Self {
            credential: credential.into(),
            subject,
        }
    }
}

impl UserCredentialVerifier for FixtureCredentialVerifier {
    fn verify<'a>(
        &'a self,
        presented: Option<&'a str>,
        _expected_tenant: &'a str,
        _now_epoch_seconds: i64,
    ) -> Pin<Box<dyn Future<Output = Result<AuthenticatedSubject, SessionRefusal>> + Send + 'a>>
    {
        Box::pin(async move {
            let Some(presented) = presented else {
                return Err(SessionRefusal::Unauthenticated);
            };
            if presented != self.credential {
                return Err(SessionRefusal::InvalidCredential);
            }
            Ok(self.subject.clone())
        })
    }
}

/// A complete response from the membership port, including a storage fault.
pub struct FixtureMembershipStore {
    result: Result<Vec<MembershipRecord>, MembershipStoreError>,
}

impl FixtureMembershipStore {
    pub fn records(records: Vec<MembershipRecord>) -> Self {
        Self {
            result: Ok(records),
        }
    }

    pub fn failure(error: MembershipStoreError) -> Self {
        Self { result: Err(error) }
    }
}

impl UserMembershipStore for FixtureMembershipStore {
    fn candidates<'a>(
        &'a self,
        _tenant_id: &'a str,
        _user_id: &'a str,
    ) -> Pin<
        Box<dyn Future<Output = Result<Vec<MembershipRecord>, MembershipStoreError>> + Send + 'a>,
    > {
        Box::pin(async move { self.result.clone() })
    }
}

#[cfg(test)]
mod tests {
    use iam_identity_user_session::{MembershipSessionAuthorizer, UserSessionAuthorizer};

    use super::*;

    fn record(active: bool) -> MembershipRecord {
        MembershipRecord {
            user_id: "usr_alice".into(),
            tenant_id: "ten_acme".into(),
            active,
            roles: vec!["foundry-operator".into()],
        }
    }

    fn verifier(exp: i64) -> FixtureCredentialVerifier {
        FixtureCredentialVerifier::new(
            "test-human-credential",
            AuthenticatedSubject::new("usr_alice", "ten_acme", exp).expect("valid subject"),
        )
    }

    #[tokio::test]
    async fn active_native_binding_grants_only_its_tenant_until_expiry() {
        let authorizer = MembershipSessionAuthorizer::new(
            verifier(300),
            FixtureMembershipStore::records(vec![record(true)]),
        );
        let grant = authorizer
            .authorize(Some("test-human-credential"), "ten_acme", 200)
            .await
            .expect("active membership");
        assert_eq!(grant.user_id(), "usr_alice");
        assert_eq!(grant.tenant_id(), "ten_acme");
        assert_eq!(grant.expires_at_epoch_seconds(), 300);
        assert_eq!(grant.roles(), &["foundry-operator"]);
        assert_eq!(grant.ensure_current(299), Ok(()));
        assert_eq!(grant.ensure_current(300), Err(SessionRefusal::Expired));
    }

    #[tokio::test]
    async fn credential_and_tenant_denials_happen_before_membership_grant() {
        let authorizer = MembershipSessionAuthorizer::new(
            verifier(300),
            FixtureMembershipStore::records(vec![record(true)]),
        );
        for (presented, tenant, expected) in [
            (None, "ten_acme", SessionRefusal::Unauthenticated),
            (Some("wrong"), "ten_acme", SessionRefusal::InvalidCredential),
            (
                Some("test-human-credential"),
                "ten_other",
                SessionRefusal::TenantMismatch,
            ),
        ] {
            assert_eq!(
                authorizer.authorize(presented, tenant, 200).await,
                Err(expected)
            );
        }
        assert_eq!(
            authorizer
                .authorize(Some("test-human-credential"), "ten_acme", 300)
                .await,
            Err(SessionRefusal::Expired)
        );
    }

    #[tokio::test]
    async fn missing_inactive_ambiguous_and_unavailable_membership_deny() {
        let cases = [
            (
                FixtureMembershipStore::records(vec![]),
                SessionRefusal::MissingMembership,
            ),
            (
                FixtureMembershipStore::records(vec![record(false)]),
                SessionRefusal::InactiveMembership,
            ),
            (
                FixtureMembershipStore::records(vec![record(true), record(true)]),
                SessionRefusal::AmbiguousMembership,
            ),
            (
                FixtureMembershipStore::failure(MembershipStoreError::Unavailable),
                SessionRefusal::MembershipUnavailable,
            ),
            (
                FixtureMembershipStore::failure(MembershipStoreError::Corrupt),
                SessionRefusal::MembershipUnavailable,
            ),
        ];
        for (store, expected) in cases {
            let authorizer = MembershipSessionAuthorizer::new(verifier(300), store);
            assert_eq!(
                authorizer
                    .authorize(Some("test-human-credential"), "ten_acme", 200)
                    .await,
                Err(expected)
            );
        }
    }

    #[tokio::test]
    async fn mismatched_store_record_and_invalid_native_subject_deny() {
        let authorizer = MembershipSessionAuthorizer::new(
            verifier(300),
            FixtureMembershipStore::records(vec![MembershipRecord {
                tenant_id: "ten_other".into(),
                ..record(true)
            }]),
        );
        assert_eq!(
            authorizer
                .authorize(Some("test-human-credential"), "ten_acme", 200)
                .await,
            Err(SessionRefusal::MembershipUnavailable)
        );
        assert_eq!(
            AuthenticatedSubject::new("not-a-user", "ten_acme", 300),
            Err(SessionRefusal::InvalidSubject)
        );
    }
}
