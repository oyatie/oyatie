//! Policy publication and authorization.

use crate::*;

use crate::Foundation;

impl Foundation {
    pub fn publish_policy(
        &mut self,
        version: PolicyVersion,
    ) -> Result<iam_policy_cedar_domain::PublishedPolicy, FoundationError> {
        let scope_tenant_id = match &version.scope {
            PolicyScope::Global => None,
            PolicyScope::Tenant(tenant_id) => Some(tenant_id.clone()),
        };
        if let Some(tenant_id) = &scope_tenant_id {
            self.require_tenant(tenant_id)?;
        }
        let published = self.policies.publish(version).map_err(map_policy_error)?;
        self.audit_chain.append_classifications(
            scope_tenant_id.unwrap_or_else(|| "ten_system".to_string()),
            "cedar.policy.publish",
            Plane::Control,
            Purpose::CoreService,
            vec![DataClass::InternalOnly],
            "ALLOW",
        )?;
        Ok(published)
    }

    pub fn authorize(
        &mut self,
        request: AuthorizationRequest,
    ) -> Result<AuthorizationDecision, FoundationError> {
        let user = self.require_user(&request.tenant_id, &request.user_id)?;
        let decision = self.policies.authorize(&AuthorizationQuery {
            subject: AuthorizationSubject {
                tenant_id: request.tenant_id.clone(),
                roles: user.roles.value.clone(),
            },
            action: request.action,
            resource: request.resource,
            attributes: request.attributes.into_iter().collect(),
        });
        self.audit_chain.append_classifications(
            request.tenant_id,
            "cedar.policy.authorize",
            Plane::Control,
            Purpose::CoreService,
            vec![DataClass::InternalOnly],
            if decision.allowed { "ALLOW" } else { "DENY" },
        )?;
        Ok(decision)
    }
}
