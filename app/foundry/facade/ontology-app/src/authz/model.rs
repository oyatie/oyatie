use super::{PolicyEnforcementPoint, principal_ref, resource_entities};
use foundry_caller_draft::Caller;
use policy_pdp_kernel::PolicyDecisionPoint;
use shared_platform_contracts_kernel::pdp::{
    AuthorizationRequest, Decision, EntityRef, PolicyVersion,
};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelOperation {
    Read,
    Author,
    Publish,
}

impl ModelOperation {
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Read => "foundry.ontology.model.read",
            Self::Author => "foundry.ontology.model.author",
            Self::Publish => "foundry.ontology.model.publish",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelDecision {
    pub tenant_id: String,
    pub principal_id: String,
    pub operation: ModelOperation,
    pub decision_id: String,
    pub policy_version: PolicyVersion,
    pub session_expiry: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelPepError {
    Denied,
    Unavailable,
}

impl PolicyEnforcementPoint {
    pub fn decide_model(
        &self,
        caller: &Caller,
        operation: ModelOperation,
        served_tenant: &str,
        session_expiry: i64,
    ) -> Result<ModelDecision, ModelPepError> {
        if caller.tenant_id != served_tenant {
            return Err(ModelPepError::Denied);
        }
        let resource = EntityRef {
            entity_type: "OntologyModel".into(),
            entity_id: served_tenant.into(),
        };
        let request = AuthorizationRequest {
            request_id: format!("{}:{served_tenant}", operation.slug()),
            tenant_id: caller.tenant_id.clone(),
            principal: principal_ref(caller),
            action: operation.slug().into(),
            resource: resource.clone(),
            context: BTreeMap::new(),
            min_policy_version: None,
        };
        let outcome = self
            .guard
            .authorize(
                &request,
                &resource_entities(caller, resource, served_tenant),
            )
            .map_err(|_| ModelPepError::Unavailable)?;
        if outcome.response.decision != Decision::Allow {
            return Err(ModelPepError::Denied);
        }
        Ok(ModelDecision {
            tenant_id: caller.tenant_id.clone(),
            principal_id: caller.principal_id.clone(),
            operation,
            decision_id: outcome.response.decision_id,
            policy_version: outcome.response.policy_version,
            session_expiry,
        })
    }
}
