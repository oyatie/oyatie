//! Action-type definitions and the invocation records they exchange.

use crate::action_parameters::ActionParameterDefinition;
use crate::error::OntologyEngineError;

use super::identifiers::{ActionTypeId, AutonomyTier, EntityTypeId, validate_ontology_tenant};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionTypeDefinition {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub id: ActionTypeId,                // data_class: INTERNAL_ONLY
    pub entity_type: EntityTypeId,       // data_class: INTERNAL_ONLY
    pub surface: String,                 // data_class: INTERNAL_ONLY
    pub max_autonomy_tier: AutonomyTier, // data_class: INTERNAL_ONLY
    pub audit_event_type: String,        // data_class: INTERNAL_ONLY
    pub revision: u32,                   // data_class: INTERNAL_ONLY
    /// Declared parameter schema. Empty means the action takes no
    /// parameters; submissions carrying any value are then non-conformant.
    pub parameters: Vec<ActionParameterDefinition>, // data_class: INTERNAL_ONLY
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionPolicyDecision {
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
    pub autonomy_tier: AutonomyTier,   // data_class: INTERNAL_ONLY
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionInvocationRequest {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub principal_id: String,            // data_class: INTERNAL_ONLY
    pub action_id: ActionTypeId,         // data_class: INTERNAL_ONLY
    pub entity_id: String,               // data_class: INTERNAL_ONLY
    pub idempotency_key: String,         // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionInvocationReceipt {
    pub decision_id: String,            // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub principal_id: String,           // data_class: INTERNAL_ONLY
    pub action_id: String,              // data_class: INTERNAL_ONLY
    pub entity_id: String,              // data_class: INTERNAL_ONLY
    pub idempotency_key: String,        // data_class: INTERNAL_ONLY
    pub audit_event_type: String,       // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub schema_version: u32,            // data_class: INTERNAL_ONLY
}

impl ActionTypeDefinition {
    pub fn new(
        tenant_id: impl Into<String>,
        id: ActionTypeId,
        entity_type: EntityTypeId,
        surface: impl Into<String>,
        max_autonomy_tier: AutonomyTier,
        audit_event_type: impl Into<String>,
    ) -> Result<Self, OntologyEngineError> {
        let tenant_id = tenant_id.into();
        validate_ontology_tenant(&tenant_id)?;
        let surface = surface.into();
        if surface.trim().is_empty() {
            return Err(OntologyEngineError::EmptySurface);
        }
        let audit_event_type = audit_event_type.into();
        if audit_event_type.trim().is_empty() {
            return Err(OntologyEngineError::EmptyAuditEventType);
        }
        Ok(Self {
            tenant_id,
            id,
            entity_type,
            surface,
            max_autonomy_tier,
            audit_event_type,
            revision: 1,
            parameters: Vec::new(),
        })
    }

    /// Attach the declared parameter schema. Returns `self` for chaining,
    /// mirroring [`EntityTypeDefinition::with_pillar`].
    pub fn with_parameters(mut self, parameters: Vec<ActionParameterDefinition>) -> Self {
        self.parameters = parameters;
        self
    }
}
