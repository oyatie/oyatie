//! Action-invocation authorization: fail-closed gating of an
//! [`ActionInvocationRequest`](crate::ActionInvocationRequest) against a
//! caller-supplied [`ActionPolicyDecision`](crate::ActionPolicyDecision).

use crate::definitions::{
    ActionInvocationReceipt, ActionInvocationRequest, ActionPolicyDecision,
    validate_ontology_tenant,
};
use crate::error::OntologyEngineError;

use super::{OntologyEngine, ontology_scoped_key};

impl OntologyEngine {
    pub fn authorize_action_invocation(
        &self,
        request: ActionInvocationRequest,
        decision: ActionPolicyDecision,
    ) -> Result<ActionInvocationReceipt, OntologyEngineError> {
        validate_ontology_tenant(&request.tenant_id)?;
        if request.principal_id.trim().is_empty() {
            return Err(OntologyEngineError::EmptyPrincipalId);
        }
        if request.idempotency_key.trim().is_empty() {
            return Err(OntologyEngineError::EmptyIdempotencyKey);
        }
        if !request.entity_id.starts_with("ent_") {
            return Err(OntologyEngineError::InvalidEntityId);
        }
        if decision.decision_id.trim().is_empty() {
            return Err(OntologyEngineError::EmptyDecisionId);
        }
        if decision.tenant_id != request.tenant_id {
            return Err(OntologyEngineError::TenantMismatch);
        }
        if decision.principal_id != request.principal_id {
            return Err(OntologyEngineError::PrincipalMismatch);
        }
        let action = self
            .action_types
            .get(&ontology_scoped_key(
                &request.tenant_id,
                &request.action_id.value,
            ))
            .ok_or(OntologyEngineError::UnknownActionType)?;
        if !decision
            .allowed_surfaces
            .iter()
            .any(|surface| surface == &action.surface)
        {
            return Err(OntologyEngineError::AuthorizationDenied);
        }
        if decision.autonomy_tier > action.max_autonomy_tier {
            return Err(OntologyEngineError::AutonomyTierExceeded);
        }
        Ok(ActionInvocationReceipt {
            decision_id: decision.decision_id,
            tenant_id: request.tenant_id,
            principal_id: request.principal_id,
            action_id: request.action_id.value,
            entity_id: request.entity_id,
            idempotency_key: request.idempotency_key,
            audit_event_type: action.audit_event_type.clone(),
            occurred_at_epoch_seconds: request.requested_at_epoch_seconds,
            schema_version: 1,
        })
    }
}
