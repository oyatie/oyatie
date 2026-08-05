use crate::error::{Result, ServiceError, ServiceErrorKind};
use crate::usecase::{CommandEnvelope, PolicyPort};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CedarInspectionPlanPolicyFixture {
    policy_fragment: String,
}

impl CedarInspectionPlanPolicyFixture {
    pub fn from_fragment(fragment: impl Into<String>) -> Result<Self> {
        let policy_fragment = fragment.into();
        if !policy_fragment.contains("principal.tenant_id == resource.tenant_id") {
            return Err(ServiceError::configuration(
                "inspection-plan policy fixture requires Cedar tenant equality rule",
            ));
        }
        if !policy_fragment.contains("Action::\"inspection-plan.approve\"") {
            return Err(ServiceError::configuration(
                "inspection-plan policy fixture requires inspection-plan approval action",
            ));
        }
        Ok(Self { policy_fragment })
    }

    fn resource_tenant<'a>(&self, envelope: &'a CommandEnvelope) -> Result<&'a str> {
        let _policy_fragment = &self.policy_fragment;
        envelope
            .command
            .resource_id()
            .as_str()
            .split_once(':')
            .map(|(tenant, _resource)| tenant)
            .filter(|tenant| !tenant.is_empty())
            .ok_or_else(|| {
                ServiceError::validation(
                    "resource_id",
                    "inspection-plan policy fixture expects tenant_id:resource_id shape",
                )
            })
    }
}

impl PolicyPort for CedarInspectionPlanPolicyFixture {
    fn authorize(&self, envelope: &CommandEnvelope) -> Result<()> {
        let principal_tenant = envelope.context.actor.tenant_id.as_str();
        let resource_tenant = self.resource_tenant(envelope)?;
        if principal_tenant == resource_tenant {
            Ok(())
        } else {
            Err(ServiceError::new(
                ServiceErrorKind::Authorization,
                "Cedar inspection-plan tenant equality policy denied cross-tenant command",
            ))
        }
    }
}
