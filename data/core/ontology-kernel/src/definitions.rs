//! Ontology type-plane definitions: identifier newtypes, cardinality and
//! autonomy vocabularies, and the entity/link/action type definitions plus
//! the invocation request/decision/receipt records they exchange.

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

use crate::action_parameters::ActionParameterDefinition;
use crate::error::OntologyEngineError;
use crate::pillar::OntologyPillar;
use crate::property::PropertyTier;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct EntityTypeId {
    pub value: String, // data_class: INTERNAL_ONLY
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LinkTypeId {
    pub value: String, // data_class: INTERNAL_ONLY
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ActionTypeId {
    pub value: String, // data_class: INTERNAL_ONLY
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum LinkCardinality {
    OneToOne,
    OneToMany,
    ManyToMany,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AutonomyTier {
    T0Suggest,
    T1Assist,
    T2ExecuteWithApproval,
    T3Autonomous,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntityTypePropertyDefinition {
    pub name: String,       // data_class: INTERNAL_ONLY
    pub tier: PropertyTier, // data_class: INTERNAL_ONLY
    pub data_class: PrivacyDataClass,
    pub required: bool, // data_class: INTERNAL_ONLY
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntityTypeDefinition {
    pub tenant_id: String,                             // data_class: INTERNAL_ONLY
    pub id: EntityTypeId,                              // data_class: INTERNAL_ONLY
    pub display_name: Classified<String>,              // data_class: INTERNAL_ONLY
    pub properties: Vec<EntityTypePropertyDefinition>, // data_class: INTERNAL_ONLY
    pub revision: u32,                                 // data_class: INTERNAL_ONLY
    /// Optional pillar annotation for org/person isolation (Bominal-ADR-0132).
    /// `None` means the entity type is pillar-agnostic and does not
    /// participate in cross-pillar link rejection.
    pub pillar: Option<OntologyPillar>, // data_class: INTERNAL_ONLY
    /// Name of the property that identifies instances of this type. Must
    /// name a declared `required` property; immutable once set (re-keying a
    /// population is a breaking change). `None` means instances are keyed
    /// only by their `ent_`-prefixed id.
    pub primary_key_property: Option<String>, // data_class: INTERNAL_ONLY
    /// Name of the property rendered as the default human-readable label.
    /// Must name a declared property; freely changeable across revisions.
    pub title_property: Option<String>, // data_class: INTERNAL_ONLY
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinkTypeDefinition {
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub id: LinkTypeId,                 // data_class: INTERNAL_ONLY
    pub from_entity_type: EntityTypeId, // data_class: INTERNAL_ONLY
    pub to_entity_type: EntityTypeId,   // data_class: INTERNAL_ONLY
    pub cardinality: LinkCardinality,   // data_class: INTERNAL_ONLY
    pub allow_cross_tenant: bool,       // data_class: INTERNAL_ONLY
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionTypeDefinition {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub id: ActionTypeId,                // data_class: INTERNAL_ONLY
    pub entity_type: EntityTypeId,       // data_class: INTERNAL_ONLY
    pub surface: String,                 // data_class: INTERNAL_ONLY
    pub max_autonomy_tier: AutonomyTier, // data_class: INTERNAL_ONLY
    pub audit_event_type: String,        // data_class: INTERNAL_ONLY
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

impl EntityTypeId {
    pub fn new(value: impl Into<String>) -> Result<Self, OntologyEngineError> {
        prefixed_ontology_id(value.into(), "ety_", OntologyEngineError::InvalidTypeId)
            .map(|value| Self { value })
    }
}
impl LinkTypeId {
    pub fn new(value: impl Into<String>) -> Result<Self, OntologyEngineError> {
        prefixed_ontology_id(value.into(), "lty_", OntologyEngineError::InvalidLinkTypeId)
            .map(|value| Self { value })
    }
}
impl ActionTypeId {
    pub fn new(value: impl Into<String>) -> Result<Self, OntologyEngineError> {
        prefixed_ontology_id(
            value.into(),
            "aty_",
            OntologyEngineError::InvalidActionTypeId,
        )
        .map(|value| Self { value })
    }
}
impl EntityTypePropertyDefinition {
    pub fn new(
        name: impl Into<String>,
        tier: PropertyTier,
        data_class: PrivacyDataClass,
        required: bool,
    ) -> Result<Self, OntologyEngineError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(OntologyEngineError::EmptyPropertyName);
        }
        Ok(Self {
            name,
            tier,
            data_class,
            required,
        })
    }
}
impl EntityTypeDefinition {
    pub fn new(
        tenant_id: impl Into<String>,
        id: EntityTypeId,
        display_name: impl Into<String>,
        properties: Vec<EntityTypePropertyDefinition>,
        revision: u32,
    ) -> Result<Self, OntologyEngineError> {
        let tenant_id = tenant_id.into();
        validate_ontology_tenant(&tenant_id)?;
        let display_name = display_name.into();
        if display_name.trim().is_empty() {
            return Err(OntologyEngineError::EmptyDisplayName);
        }
        if properties.is_empty() {
            return Err(OntologyEngineError::EmptyProperties);
        }
        Ok(Self {
            tenant_id,
            id,
            display_name: Classified::new(display_name, DataClass::InternalOnly),
            properties,
            revision,
            pillar: None,
            primary_key_property: None,
            title_property: None,
        })
    }

    /// Annotate this entity type with an [`OntologyPillar`] for org/person
    /// isolation enforcement (Bominal-ADR-0132). Returns `self` for chaining.
    /// Entity types without a pillar annotation are pillar-agnostic and do not
    /// participate in cross-pillar link rejection.
    pub fn with_pillar(mut self, pillar: OntologyPillar) -> Self {
        self.pillar = Some(pillar);
        self
    }

    /// Designate the primary-key property. Integrity (declared, required,
    /// immutable across revisions) is enforced at registration/evolution.
    pub fn with_primary_key_property(mut self, name: impl Into<String>) -> Self {
        self.primary_key_property = Some(name.into());
        self
    }

    /// Designate the title property. Integrity (declared) is enforced at
    /// registration/evolution.
    pub fn with_title_property(mut self, name: impl Into<String>) -> Self {
        self.title_property = Some(name.into());
        self
    }
}
impl LinkTypeDefinition {
    pub fn new(
        tenant_id: impl Into<String>,
        id: LinkTypeId,
        from_entity_type: EntityTypeId,
        to_entity_type: EntityTypeId,
        cardinality: LinkCardinality,
        allow_cross_tenant: bool,
    ) -> Result<Self, OntologyEngineError> {
        let tenant_id = tenant_id.into();
        validate_ontology_tenant(&tenant_id)?;
        Ok(Self {
            tenant_id,
            id,
            from_entity_type,
            to_entity_type,
            cardinality,
            allow_cross_tenant,
        })
    }
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

fn prefixed_ontology_id(
    value: String,
    prefix: &str,
    error: OntologyEngineError,
) -> Result<String, OntologyEngineError> {
    if value.starts_with(prefix) && value.len() > prefix.len() {
        Ok(value)
    } else {
        Err(error)
    }
}
pub(crate) fn validate_ontology_tenant(tenant_id: &str) -> Result<(), OntologyEngineError> {
    if tenant_id.starts_with("ten_") && tenant_id.len() > "ten_".len() {
        Ok(())
    } else {
        Err(OntologyEngineError::InvalidTenantId)
    }
}
