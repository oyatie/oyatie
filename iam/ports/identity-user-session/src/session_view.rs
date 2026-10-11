use super::{ActiveUserMembership, AuthenticatedSubject, SessionRefusal};
use serde::{Deserialize, Serialize};

/// Fresh IAM HTTP response, consumed only after a trusted IAM transport read.
/// Role identifiers are provisioned data and never a product PDP decision.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SessionView {
    pub tenant_id: String,
    pub user_id: String,
    pub expires_at: i64,
    pub roles: Vec<String>,
}

impl SessionView {
    /// The adapter must obtain this response from IAM afresh for every live
    /// operation. Deserialization does not establish the response's authority.
    pub fn into_active_membership(
        self,
        expected_tenant: &str,
        now_epoch_seconds: i64,
    ) -> Result<ActiveUserMembership, SessionRefusal> {
        let subject = AuthenticatedSubject::new(self.user_id, self.tenant_id, self.expires_at)?;
        ActiveUserMembership::from_verified_subject(
            subject,
            self.roles,
            expected_tenant,
            now_epoch_seconds,
        )
    }
}

impl From<&ActiveUserMembership> for SessionView {
    fn from(value: &ActiveUserMembership) -> Self {
        Self {
            tenant_id: value.tenant_id().to_owned(),
            user_id: value.user_id().to_owned(),
            expires_at: value.expires_at_epoch_seconds(),
            roles: value.roles().to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view() -> SessionView {
        SessionView {
            tenant_id: "ten_acme".into(),
            user_id: "usr_alice".into(),
            expires_at: 200,
            roles: vec!["foundry-operator".into()],
        }
    }

    #[test]
    fn trusted_response_conversion_enforces_native_boundary() {
        let active = view()
            .into_active_membership("ten_acme", 100)
            .expect("current");
        assert_eq!(active.user_id(), "usr_alice");
        assert_eq!(active.roles(), ["foundry-operator"]);
        assert_eq!(
            view().into_active_membership("ten_other", 100),
            Err(SessionRefusal::TenantMismatch)
        );
        assert_eq!(
            view().into_active_membership("ten_acme", 200),
            Err(SessionRefusal::Expired)
        );
        let mut invalid = view();
        invalid.user_id = "workload".into();
        assert_eq!(
            invalid.into_active_membership("ten_acme", 100),
            Err(SessionRefusal::InvalidSubject)
        );
        for roles in [
            vec!["foundry-operator".into(), "foundry-operator".into()],
            vec![" ".into()],
            vec!["a".repeat(257)],
        ] {
            let mut invalid = view();
            invalid.roles = roles;
            assert_eq!(
                invalid.into_active_membership("ten_acme", 100),
                Err(SessionRefusal::MembershipUnavailable)
            );
        }
    }
}
