//! Ontology kernel: data-classed entities, property-tier semantics, and pillar isolation.
//!
//! # Registration invariants
//!
//! ## Endpoint-reference validation
//!
//! [`OntologyEngine::register_link_type`] and
//! [`OntologyEngine::register_action_type`] validate that every
//! [`EntityTypeId`] referenced by the definition was previously registered for
//! the same tenant via [`OntologyEngine::register_entity_type`]. A dangling
//! reference (an `EntityTypeId` that has not been registered) is rejected with
//! [`OntologyEngineError::UnknownEntityTypeEndpoint`].
//!
//! ## Pillar-consistency enforcement (Bominal-ADR-0132)
//!
//! [`OntologyEngine::register_link_type`] enforces the org/person isolation
//! boundary. If both the `from_entity_type` and `to_entity_type` carry an
//! [`OntologyPillar`] annotation (via [`EntityTypeDefinition::with_pillar`])
//! and those pillars differ, the registration is rejected with
//! [`OntologyEngineError::CrossPillarLink`]. Entity types with no pillar
//! annotation (`pillar: None`) are pillar-agnostic and do not trigger this
//! check.
//!
//! ## New error variants
//!
//! | Variant | Trigger |
//! |---------|---------|
//! | [`OntologyEngineError::UnknownEntityTypeEndpoint`] | A `LinkTypeDefinition` or `ActionTypeDefinition` references an `EntityTypeId` not registered for the same tenant. |
//! | [`OntologyEngineError::CrossPillarLink`] | A `LinkTypeDefinition` binds an org-pillar endpoint to a person-pillar endpoint (or vice versa). |

// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod pillar;
pub use pillar::{OntologyPillar, UnknownPillarLabel};

use std::collections::BTreeMap;

use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PropertyTier {
    Scalar,
    Vector,
    Timeseries,
    Geo,
    Ciphertext,
    Struct,
}

impl PropertyTier {
    pub const fn wire_label(self) -> &'static str {
        match self {
            Self::Scalar => "scalar",
            Self::Vector => "vector",
            Self::Timeseries => "timeseries",
            Self::Geo => "geo",
            Self::Ciphertext => "ciphertext",
            Self::Struct => "struct",
        }
    }

    pub const fn all_tiers() -> [Self; 6] {
        [
            Self::Scalar,
            Self::Vector,
            Self::Timeseries,
            Self::Geo,
            Self::Ciphertext,
            Self::Struct,
        ]
    }

    pub const fn object_graph_property_tiers() -> [Self; 5] {
        [
            Self::Vector,
            Self::Timeseries,
            Self::Geo,
            Self::Ciphertext,
            Self::Struct,
        ]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectProperty {
    pub name: String,              // data_class: INTERNAL_ONLY
    pub value: Classified<String>, // data_class: PROPERTY_VALUE_PRIVACY_CLASS
    pub tier: PropertyTier,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectEntity {
    pub tenant_id: String,                            // data_class: INTERNAL_ONLY
    pub id: String,                                   // data_class: INTERNAL_ONLY
    pub entity_type: Classified<String>,              // data_class: INTERNAL_ONLY
    pub properties: BTreeMap<String, ObjectProperty>, // data_class: PROPERTY_VALUE_PRIVACY_CLASS
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectEntityUpsertOutcome {
    Created,
    Updated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectPropertyUpsertOutcome {
    Inserted,
    Updated,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ObjectGraph {
    entities: BTreeMap<ObjectEntityKey, ObjectEntity>, // data_class: PROPERTY_VALUE_PRIVACY_CLASS
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct ObjectEntityKey {
    tenant_id: String, // data_class: INTERNAL_ONLY
    id: String,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObjectGraphError {
    InvalidEntityId,
    EmptyEntityType,
    MissingProperties,
    EmptyPropertyName,
    InvalidDataClass,
}

impl ObjectEntity {
    pub fn new(
        tenant_id: String,
        id: String,
        entity_type: String,
        properties: Vec<ObjectProperty>,
    ) -> Result<Self, ObjectGraphError> {
        if !id.starts_with("ent_") {
            return Err(ObjectGraphError::InvalidEntityId);
        }
        if entity_type.trim().is_empty() {
            return Err(ObjectGraphError::EmptyEntityType);
        }
        if properties.is_empty() {
            return Err(ObjectGraphError::MissingProperties);
        }
        let mut by_name = BTreeMap::new();
        for property in properties {
            validate_property(&property)?;
            by_name.insert(property.name.clone(), property);
        }
        Ok(Self {
            tenant_id,
            id,
            entity_type: Classified::new(entity_type, DataClass::InternalOnly),
            properties: by_name,
        })
    }

    pub fn upsert_property(
        &mut self,
        property: ObjectProperty,
    ) -> Result<ObjectPropertyUpsertOutcome, ObjectGraphError> {
        validate_property(&property)?;
        let outcome = if self
            .properties
            .insert(property.name.clone(), property)
            .is_some()
        {
            ObjectPropertyUpsertOutcome::Updated
        } else {
            ObjectPropertyUpsertOutcome::Inserted
        };
        Ok(outcome)
    }
}

impl ObjectGraph {
    pub fn upsert_entity(
        &mut self,
        entity: ObjectEntity,
    ) -> Result<ObjectEntityUpsertOutcome, ObjectGraphError> {
        validate_entity_key(&entity.tenant_id, &entity.id)?;
        if entity.properties.is_empty() {
            return Err(ObjectGraphError::MissingProperties);
        }
        if entity.entity_type.value.trim().is_empty() {
            return Err(ObjectGraphError::EmptyEntityType);
        }
        for property in entity.properties.values() {
            validate_property(property)?;
        }

        let key = ObjectEntityKey {
            tenant_id: entity.tenant_id.clone(),
            id: entity.id.clone(),
        };
        let outcome = if self.entities.insert(key, entity).is_some() {
            ObjectEntityUpsertOutcome::Updated
        } else {
            ObjectEntityUpsertOutcome::Created
        };
        Ok(outcome)
    }

    pub fn get(&self, tenant_id: &str, entity_id: &str) -> Option<&ObjectEntity> {
        self.entities.get(&ObjectEntityKey {
            tenant_id: tenant_id.to_string(),
            id: entity_id.to_string(),
        })
    }

    pub fn entities_for_tenant(&self, tenant_id: &str) -> impl Iterator<Item = &ObjectEntity> {
        self.entities
            .range(
                ObjectEntityKey {
                    tenant_id: tenant_id.to_string(),
                    id: String::new(),
                }..,
            )
            .map_while(move |(key, entity)| (key.tenant_id == tenant_id).then_some(entity))
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }
}

impl ObjectProperty {
    pub fn new(
        name: String,
        value: String,
        tier: PropertyTier,
        data_class: PrivacyDataClass,
    ) -> Self {
        Self::new_with_privacy_data_class(name, value, tier, data_class)
    }

    /// Compatibility constructor for request/import seams that still carry raw
    /// `DataClass` labels. Canonical object properties take
    /// `PrivacyDataClass`, and this path fails closed for operational markers
    /// and subject markers.
    pub fn try_from_legacy_data_class(
        name: String,
        value: String,
        tier: PropertyTier,
        data_class: DataClass,
    ) -> Result<Self, ObjectGraphError> {
        let data_class = PrivacyDataClass::try_from(data_class)
            .map_err(|_| ObjectGraphError::InvalidDataClass)?;
        Ok(Self::new(name, value, tier, data_class))
    }

    pub fn new_with_privacy_data_class(
        name: String,
        value: String,
        tier: PropertyTier,
        data_class: PrivacyDataClass,
    ) -> Self {
        Self {
            name,
            value: Classified::new(value, data_class),
            tier,
        }
    }
}

fn validate_property(property: &ObjectProperty) -> Result<(), ObjectGraphError> {
    if property.name.trim().is_empty() {
        return Err(ObjectGraphError::EmptyPropertyName);
    }
    Ok(())
}

fn validate_entity_key(tenant_id: &str, entity_id: &str) -> Result<(), ObjectGraphError> {
    if tenant_id.trim().is_empty() || !entity_id.starts_with("ent_") {
        return Err(ObjectGraphError::InvalidEntityId);
    }
    Ok(())
}

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
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OntologyEngineError {
    InvalidTenantId,
    InvalidTypeId,
    InvalidLinkTypeId,
    InvalidActionTypeId,
    EmptyDisplayName,
    EmptyProperties,
    EmptyPropertyName,
    DuplicateEntityType,
    DuplicateLinkType,
    DuplicateActionType,
    UnknownEntityType,
    UnknownActionType,
    EmptySurface,
    EmptyAuditEventType,
    EmptyDecisionId,
    EmptyPrincipalId,
    EmptyIdempotencyKey,
    PrincipalMismatch,
    TenantMismatch,
    AuthorizationDenied,
    AutonomyTierExceeded,
    InvalidEntityId,
    /// A `LinkTypeDefinition` or `ActionTypeDefinition` references an
    /// [`EntityTypeId`] endpoint that has not been registered for the same
    /// tenant. Register all endpoint entity types before registering link or
    /// action types that reference them.
    UnknownEntityTypeEndpoint,
    /// A `LinkTypeDefinition` binds an org-pillar endpoint to a person-pillar
    /// endpoint (or vice versa), violating Bominal-ADR-0132 org/person
    /// isolation. Both endpoints must share the same [`OntologyPillar`], or at
    /// least one must be pillar-agnostic (`pillar: None`).
    CrossPillarLink,
    /// The candidate revision is not strictly greater than the stored revision.
    /// [`OntologyEngine::evolve_entity_type`] requires
    /// `candidate.revision > stored.revision`.
    NonMonotonicRevision,
    /// The candidate definition removes or mutates an existing property.
    /// [`OntologyEngine::evolve_entity_type`] only allows additive changes:
    /// every prior property must remain with unchanged `tier`, `data_class`,
    /// and `required` flag. New properties may be introduced freely.
    IncompatibleSchemaEvolution,
    /// `register_link_instance` was called with a [`LinkTypeId`] that has not
    /// been registered for the tenant. Register the link type before creating
    /// instances of it.
    UnknownLinkType,
    /// A link instance would violate the [`LinkCardinality`] declared on the
    /// [`LinkTypeDefinition`]. The `cardinality` field carries the constraint
    /// that was violated.
    CardinalityViolation {
        /// The cardinality constraint that was violated.
        cardinality: LinkCardinality,
    },
}

/// Outcome returned by [`OntologyEngine::register_link_instance`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LinkInstanceOutcome {
    /// The instance was freshly inserted into the registry.
    Registered,
    /// The identical `(link_type_id, from_entity_id, to_entity_id)` tuple
    /// already existed; no state change occurred.
    AlreadyExists,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OntologyEngine {
    entity_types: BTreeMap<OntologyScopedKey, EntityTypeDefinition>,
    link_types: BTreeMap<OntologyScopedKey, LinkTypeDefinition>,
    action_types: BTreeMap<OntologyScopedKey, ActionTypeDefinition>,
    /// Full 4-tuple registry for idempotency checks.
    /// Key: (tenant_id, link_type_id, from_entity_id, to_entity_id)
    /// data_class: INTERNAL_ONLY
    link_instances: BTreeMap<(String, String, String, String), ()>,
    /// Outbound index: at most one outbound edge per (tenant, link_type, from) for OneToOne.
    /// Key: (tenant_id, link_type_id, from_entity_id)
    /// data_class: INTERNAL_ONLY
    link_outbound: BTreeMap<(String, String, String), ()>,
    /// Inbound index: at most one inbound edge per (tenant, link_type, to) for OneToOne/OneToMany.
    /// Key: (tenant_id, link_type_id, to_entity_id)
    /// data_class: INTERNAL_ONLY
    link_inbound: BTreeMap<(String, String, String), ()>,
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct OntologyScopedKey {
    tenant_id: String,
    id: String,
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
        })
    }
}
impl OntologyEngine {
    pub fn register_entity_type(
        &mut self,
        definition: EntityTypeDefinition,
    ) -> Result<EntityTypeId, OntologyEngineError> {
        let key = ontology_scoped_key(&definition.tenant_id, &definition.id.value);
        if self.entity_types.contains_key(&key) {
            return Err(OntologyEngineError::DuplicateEntityType);
        }
        let id = definition.id.clone();
        self.entity_types.insert(key, definition);
        Ok(id)
    }

    /// Register or evolve an entity type definition.
    ///
    /// - **First registration** (id unknown for the tenant): behaves identically
    ///   to [`register_entity_type`](Self::register_entity_type) — inserts the
    ///   definition and returns `Ok(id)`. `DuplicateEntityType` is never
    ///   returned by this method.
    /// - **Evolution** (id already registered): requires
    ///   `definition.revision > stored.revision` (strict monotonicity) and that
    ///   every prior property is retained with unchanged `tier`, `data_class`,
    ///   and `required` flag. New properties may be introduced freely. On
    ///   success the stored definition is replaced with `definition`.
    ///
    /// # Errors
    ///
    /// | Error | Condition |
    /// |-------|-----------|
    /// | [`OntologyEngineError::InvalidTenantId`] | Tenant id fails prefix check. |
    /// | [`OntologyEngineError::EmptyDisplayName`] | `display_name` is blank. |
    /// | [`OntologyEngineError::EmptyProperties`] | `properties` is empty. |
    /// | [`OntologyEngineError::EmptyPropertyName`] | A property name is blank. |
    /// | [`OntologyEngineError::NonMonotonicRevision`] | `definition.revision <= stored.revision`. |
    /// | [`OntologyEngineError::IncompatibleSchemaEvolution`] | A prior property was removed or mutated. |
    pub fn evolve_entity_type(
        &mut self,
        definition: EntityTypeDefinition,
    ) -> Result<EntityTypeId, OntologyEngineError> {
        let key = ontology_scoped_key(&definition.tenant_id, &definition.id.value);
        match self.entity_types.get(&key) {
            None => {
                // First registration: identical to register_entity_type.
                let id = definition.id.clone();
                self.entity_types.insert(key, definition);
                Ok(id)
            }
            Some(stored) => {
                // Revision monotonicity check.
                if definition.revision <= stored.revision {
                    return Err(OntologyEngineError::NonMonotonicRevision);
                }
                // Backward-compatibility check.
                check_schema_compatibility(stored, &definition)?;
                let id = definition.id.clone();
                self.entity_types.insert(key, definition);
                Ok(id)
            }
        }
    }
    pub fn register_link_type(
        &mut self,
        definition: LinkTypeDefinition,
    ) -> Result<LinkTypeId, OntologyEngineError> {
        // st1: endpoint-reference validation
        let from_def = self
            .entity_types
            .get(&ontology_scoped_key(
                &definition.tenant_id,
                &definition.from_entity_type.value,
            ))
            .ok_or(OntologyEngineError::UnknownEntityTypeEndpoint)?;
        let to_def = self
            .entity_types
            .get(&ontology_scoped_key(
                &definition.tenant_id,
                &definition.to_entity_type.value,
            ))
            .ok_or(OntologyEngineError::UnknownEntityTypeEndpoint)?;
        // st2: pillar-consistency enforcement (Bominal-ADR-0132)
        if let (Some(from_pillar), Some(to_pillar)) = (from_def.pillar, to_def.pillar)
            && from_pillar != to_pillar
        {
            return Err(OntologyEngineError::CrossPillarLink);
        }
        let key = ontology_scoped_key(&definition.tenant_id, &definition.id.value);
        if self.link_types.contains_key(&key) {
            return Err(OntologyEngineError::DuplicateLinkType);
        }
        let id = definition.id.clone();
        self.link_types.insert(key, definition);
        Ok(id)
    }
    pub fn register_action_type(
        &mut self,
        definition: ActionTypeDefinition,
    ) -> Result<ActionTypeId, OntologyEngineError> {
        // st1: endpoint-reference validation
        if !self.has_entity_type(&definition.tenant_id, &definition.entity_type) {
            return Err(OntologyEngineError::UnknownEntityTypeEndpoint);
        }
        let key = ontology_scoped_key(&definition.tenant_id, &definition.id.value);
        if self.action_types.contains_key(&key) {
            return Err(OntologyEngineError::DuplicateActionType);
        }
        let id = definition.id.clone();
        self.action_types.insert(key, definition);
        Ok(id)
    }
    pub fn entity_type(&self, tenant_id: &str, id: &EntityTypeId) -> Option<&EntityTypeDefinition> {
        self.entity_types
            .get(&ontology_scoped_key(tenant_id, &id.value))
    }
    /// Return the [`LinkTypeDefinition`] registered for `tenant_id` and `id`,
    /// or `None` if no such link type has been registered.
    pub fn link_type(&self, tenant_id: &str, id: &LinkTypeId) -> Option<&LinkTypeDefinition> {
        self.link_types
            .get(&ontology_scoped_key(tenant_id, &id.value))
    }
    /// Return the [`ActionTypeDefinition`] registered for `tenant_id` and `id`,
    /// or `None` if no such action type has been registered.
    pub fn action_type(&self, tenant_id: &str, id: &ActionTypeId) -> Option<&ActionTypeDefinition> {
        self.action_types
            .get(&ontology_scoped_key(tenant_id, &id.value))
    }

    /// Register a directed link instance from `from_entity_id` to `to_entity_id`
    /// under `link_type_id` for `tenant_id`.
    ///
    /// # Behaviour
    ///
    /// 1. Rejects an unknown `link_type_id` with [`OntologyEngineError::UnknownLinkType`].
    /// 2. Is **idempotent** for the identical `(link_type_id, from_entity_id, to_entity_id)`
    ///    tuple — returns `Ok(LinkInstanceOutcome::AlreadyExists)` without mutation.
    /// 3. Enforces [`LinkCardinality`]:
    ///    - `OneToOne`: rejects a second outbound from `from_entity_id` **and** a second
    ///      inbound into `to_entity_id`.
    ///    - `OneToMany`: rejects a second inbound into `to_entity_id`; fan-out is permitted.
    ///    - `ManyToMany`: no restriction.
    pub fn register_link_instance(
        &mut self,
        tenant_id: &str,
        link_type_id: &LinkTypeId,
        from_entity_id: &str,
        to_entity_id: &str,
    ) -> Result<LinkInstanceOutcome, OntologyEngineError> {
        // Step 1: look up the link type definition.
        let link_def = self
            .link_types
            .get(&ontology_scoped_key(tenant_id, &link_type_id.value))
            .ok_or(OntologyEngineError::UnknownLinkType)?;
        let cardinality = link_def.cardinality;

        // Step 2: idempotency — identical 4-tuple already registered.
        let instance_key = (
            tenant_id.to_string(),
            link_type_id.value.clone(),
            from_entity_id.to_string(),
            to_entity_id.to_string(),
        );
        if self.link_instances.contains_key(&instance_key) {
            return Ok(LinkInstanceOutcome::AlreadyExists);
        }

        // Step 3: cardinality enforcement.
        let outbound_key = (
            tenant_id.to_string(),
            link_type_id.value.clone(),
            from_entity_id.to_string(),
        );
        let inbound_key = (
            tenant_id.to_string(),
            link_type_id.value.clone(),
            to_entity_id.to_string(),
        );
        match cardinality {
            LinkCardinality::OneToOne => {
                if self.link_outbound.contains_key(&outbound_key) {
                    return Err(OntologyEngineError::CardinalityViolation { cardinality });
                }
                if self.link_inbound.contains_key(&inbound_key) {
                    return Err(OntologyEngineError::CardinalityViolation { cardinality });
                }
            }
            LinkCardinality::OneToMany => {
                if self.link_inbound.contains_key(&inbound_key) {
                    return Err(OntologyEngineError::CardinalityViolation { cardinality });
                }
            }
            LinkCardinality::ManyToMany => {}
        }

        // Step 4: insert into all three indices.
        self.link_instances.insert(instance_key, ());
        self.link_outbound.insert(outbound_key, ());
        self.link_inbound.insert(inbound_key, ());

        Ok(LinkInstanceOutcome::Registered)
    }

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
    fn has_entity_type(&self, tenant_id: &str, id: &EntityTypeId) -> bool {
        self.entity_types
            .contains_key(&ontology_scoped_key(tenant_id, &id.value))
    }
}
/// Check that `candidate` is a backward-compatible evolution of `prior`.
///
/// Rules:
/// - Every property in `prior` must exist in `candidate` with identical
///   `tier`, `data_class`, and `required` flag.
/// - New properties in `candidate` that are absent from `prior` are permitted.
/// - Revision monotonicity is **not** checked here; the caller is responsible.
fn check_schema_compatibility(
    prior: &EntityTypeDefinition,
    candidate: &EntityTypeDefinition,
) -> Result<(), OntologyEngineError> {
    // Build a lookup map from the candidate's property list.
    let candidate_map: std::collections::BTreeMap<&str, &EntityTypePropertyDefinition> = candidate
        .properties
        .iter()
        .map(|p| (p.name.as_str(), p))
        .collect();

    for prior_prop in &prior.properties {
        match candidate_map.get(prior_prop.name.as_str()) {
            None => return Err(OntologyEngineError::IncompatibleSchemaEvolution),
            Some(cand_prop) => {
                if cand_prop.tier != prior_prop.tier
                    || cand_prop.data_class != prior_prop.data_class
                    || cand_prop.required != prior_prop.required
                {
                    return Err(OntologyEngineError::IncompatibleSchemaEvolution);
                }
            }
        }
    }
    Ok(())
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
fn validate_ontology_tenant(tenant_id: &str) -> Result<(), OntologyEngineError> {
    if tenant_id.starts_with("ten_") && tenant_id.len() > "ten_".len() {
        Ok(())
    } else {
        Err(OntologyEngineError::InvalidTenantId)
    }
}
fn ontology_scoped_key(tenant_id: &str, id: &str) -> OntologyScopedKey {
    OntologyScopedKey {
        tenant_id: tenant_id.to_string(),
        id: id.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_property_accepts_privacy_data_classes() {
        let property = ObjectProperty::new(
            "email".into(),
            "worker@example.com".into(),
            PropertyTier::Scalar,
            PrivacyDataClass::try_from(DataClass::PiiIdentifying).unwrap(),
        );

        assert_eq!(property.name, "email");
        assert_eq!(
            property.value.data_class.compatibility_data_class(),
            DataClass::PiiIdentifying
        );
    }

    #[test]
    fn object_property_rejects_operational_and_subject_markers() {
        for data_class in [DataClass::Audit, DataClass::Secret, DataClass::Children] {
            assert_eq!(
                ObjectProperty::try_from_legacy_data_class(
                    "marker".into(),
                    "not a privacy class".into(),
                    PropertyTier::Scalar,
                    data_class,
                ),
                Err(ObjectGraphError::InvalidDataClass)
            );
        }
    }

    #[test]
    fn property_tier_contract_exposes_five_object_graph_tiers() {
        let tiers = PropertyTier::object_graph_property_tiers();

        assert_eq!(tiers.len(), 5);
        assert_eq!(
            tiers.map(PropertyTier::wire_label),
            ["vector", "timeseries", "geo", "ciphertext", "struct"]
        );
        assert_eq!(
            PropertyTier::all_tiers().map(PropertyTier::wire_label),
            [
                "scalar",
                "vector",
                "timeseries",
                "geo",
                "ciphertext",
                "struct"
            ]
        );
    }

    #[test]
    fn object_entity_upsert_inserts_and_updates_property_by_name() {
        let mut entity = ObjectEntity::new(
            "tenant_a".into(),
            "ent_profile".into(),
            "profile".into(),
            vec![ObjectProperty::new(
                "embedding".into(),
                "[0.1,0.2]".into(),
                PropertyTier::Vector,
                PrivacyDataClass::try_from(DataClass::PiiQuasiIdentifier).unwrap(),
            )],
        )
        .unwrap();

        assert_eq!(
            entity.upsert_property(ObjectProperty::new(
                "last_seen".into(),
                "2026-05-14T00:00:00Z".into(),
                PropertyTier::Timeseries,
                PrivacyDataClass::try_from(DataClass::BehavioralTenantProduct).unwrap(),
            )),
            Ok(ObjectPropertyUpsertOutcome::Inserted)
        );
        assert_eq!(
            entity.upsert_property(ObjectProperty::new(
                "embedding".into(),
                "[0.3,0.4]".into(),
                PropertyTier::Vector,
                PrivacyDataClass::try_from(DataClass::PiiQuasiIdentifier).unwrap(),
            )),
            Ok(ObjectPropertyUpsertOutcome::Updated)
        );

        assert_eq!(entity.properties.len(), 2);
        assert_eq!(
            entity.properties["embedding"].value.value,
            "[0.3,0.4]".to_string()
        );
        assert_eq!(
            entity.properties["last_seen"].tier,
            PropertyTier::Timeseries
        );
    }

    #[test]
    fn object_entity_upsert_rejects_empty_property_name_without_mutation() {
        let mut entity = ObjectEntity::new(
            "tenant_a".into(),
            "ent_profile".into(),
            "profile".into(),
            vec![ObjectProperty::new(
                "location".into(),
                "{\"lat\":37.0,\"lng\":127.0}".into(),
                PropertyTier::Geo,
                PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap(),
            )],
        )
        .unwrap();

        assert_eq!(
            entity.upsert_property(ObjectProperty::new(
                " ".into(),
                "invalid".into(),
                PropertyTier::Struct,
                PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap(),
            )),
            Err(ObjectGraphError::EmptyPropertyName)
        );
        assert_eq!(entity.properties.len(), 1);
        assert!(entity.properties.contains_key("location"));
    }

    #[test]
    fn object_graph_upsert_creates_and_updates_entity_by_tenant_and_id() {
        let mut graph = ObjectGraph::default();
        let created_entity = ObjectEntity::new(
            "tenant_a".into(),
            "ent_profile".into(),
            "profile".into(),
            vec![ObjectProperty::new(
                "embedding".into(),
                "[0.1,0.2]".into(),
                PropertyTier::Vector,
                PrivacyDataClass::try_from(DataClass::PiiQuasiIdentifier).unwrap(),
            )],
        )
        .unwrap();
        let updated_entity = ObjectEntity::new(
            "tenant_a".into(),
            "ent_profile".into(),
            "profile".into(),
            vec![ObjectProperty::new(
                "location".into(),
                "{\"lat\":37.0,\"lng\":127.0}".into(),
                PropertyTier::Geo,
                PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap(),
            )],
        )
        .unwrap();

        assert_eq!(
            graph.upsert_entity(created_entity),
            Ok(ObjectEntityUpsertOutcome::Created)
        );
        assert_eq!(
            graph.upsert_entity(updated_entity),
            Ok(ObjectEntityUpsertOutcome::Updated)
        );

        assert_eq!(graph.len(), 1);
        let stored = graph
            .get("tenant_a", "ent_profile")
            .expect("entity exists after upsert");
        assert!(stored.properties.contains_key("location"));
        assert!(!stored.properties.contains_key("embedding"));
    }

    #[test]
    fn object_graph_upsert_keeps_tenants_row_isolated() {
        let mut graph = ObjectGraph::default();
        for tenant_id in ["tenant_a", "tenant_b"] {
            let entity = ObjectEntity::new(
                tenant_id.into(),
                "ent_profile".into(),
                "profile".into(),
                vec![ObjectProperty::new(
                    "config".into(),
                    tenant_id.into(),
                    PropertyTier::Struct,
                    PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap(),
                )],
            )
            .unwrap();
            assert_eq!(
                graph.upsert_entity(entity),
                Ok(ObjectEntityUpsertOutcome::Created)
            );
        }

        assert_eq!(graph.len(), 2);
        assert_eq!(
            graph.get("tenant_a", "ent_profile").unwrap().properties["config"]
                .value
                .value,
            "tenant_a"
        );
        assert_eq!(
            graph.get("tenant_b", "ent_profile").unwrap().properties["config"]
                .value
                .value,
            "tenant_b"
        );
        assert_eq!(graph.entities_for_tenant("tenant_a").count(), 1);
        assert_eq!(graph.entities_for_tenant("tenant_b").count(), 1);
    }
}

#[cfg(test)]
mod backbone_tests {
    use super::*;
    fn property(name: &str) -> EntityTypePropertyDefinition {
        EntityTypePropertyDefinition::new(
            name,
            PropertyTier::Scalar,
            PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap(),
            true,
        )
        .unwrap()
    }
    fn patient_type() -> EntityTypeDefinition {
        EntityTypeDefinition::new(
            "ten_clinic",
            EntityTypeId::new("ety_patient").unwrap(),
            "Patient",
            vec![property("mrn")],
            1,
        )
        .unwrap()
    }
    #[test]
    fn ontology_engine_registers_entity_types_and_rejects_conflicts() {
        let mut engine = OntologyEngine::default();
        let id = engine.register_entity_type(patient_type()).unwrap();
        assert_eq!(id.value, "ety_patient");
        assert!(engine.entity_type("ten_clinic", &id).is_some());
        assert_eq!(
            engine.register_entity_type(patient_type()),
            Err(OntologyEngineError::DuplicateEntityType)
        );
    }
    #[test]
    fn ontology_engine_type_checks_links_before_registration() {
        let mut engine = OntologyEngine::default();
        let patient = engine.register_entity_type(patient_type()).unwrap();
        let appointment = engine
            .register_entity_type(
                EntityTypeDefinition::new(
                    "ten_clinic",
                    EntityTypeId::new("ety_appointment").unwrap(),
                    "Appointment",
                    vec![property("starts_at")],
                    1,
                )
                .unwrap(),
            )
            .unwrap();
        let link = LinkTypeDefinition::new(
            "ten_clinic",
            LinkTypeId::new("lty_patient_appointment").unwrap(),
            patient.clone(),
            appointment,
            LinkCardinality::OneToMany,
            false,
        )
        .unwrap();
        assert_eq!(
            engine.register_link_type(link),
            Ok(LinkTypeId {
                value: "lty_patient_appointment".to_string()
            })
        );
        let unknown = LinkTypeDefinition::new(
            "ten_clinic",
            LinkTypeId::new("lty_unknown").unwrap(),
            patient,
            EntityTypeId::new("ety_missing").unwrap(),
            LinkCardinality::OneToOne,
            false,
        )
        .unwrap();
        assert_eq!(
            engine.register_link_type(unknown),
            Err(OntologyEngineError::UnknownEntityTypeEndpoint)
        );
    }
    #[test]
    fn ontology_engine_gates_action_invocation_by_policy_and_autonomy() {
        let mut engine = OntologyEngine::default();
        let patient = engine.register_entity_type(patient_type()).unwrap();
        engine
            .register_action_type(
                ActionTypeDefinition::new(
                    "ten_clinic",
                    ActionTypeId::new("aty_discharge_patient").unwrap(),
                    patient,
                    "ontology.action.discharge_patient",
                    AutonomyTier::T1Assist,
                    "EVT-ONTOLOGY-ACTION-INVOKED",
                )
                .unwrap(),
            )
            .unwrap();
        let request = ActionInvocationRequest {
            tenant_id: "ten_clinic".to_string(),
            principal_id: "usr_alice".to_string(),
            action_id: ActionTypeId::new("aty_discharge_patient").unwrap(),
            entity_id: "ent_patient_001".to_string(),
            idempotency_key: "idem-001".to_string(),
            requested_at_epoch_seconds: 1_779_523_600,
        };
        let decision = ActionPolicyDecision {
            decision_id: "dec_001".to_string(),
            tenant_id: "ten_clinic".to_string(),
            principal_id: "usr_alice".to_string(),
            allowed_surfaces: vec!["ontology.action.discharge_patient".to_string()],
            autonomy_tier: AutonomyTier::T1Assist,
        };
        let receipt = engine
            .authorize_action_invocation(request.clone(), decision.clone())
            .unwrap();
        assert_eq!(receipt.audit_event_type, "EVT-ONTOLOGY-ACTION-INVOKED");
        let denied = engine
            .authorize_action_invocation(
                request.clone(),
                ActionPolicyDecision {
                    allowed_surfaces: vec!["ontology.action.other".to_string()],
                    ..decision.clone()
                },
            )
            .unwrap_err();
        assert_eq!(denied, OntologyEngineError::AuthorizationDenied);
        let too = engine
            .authorize_action_invocation(
                request,
                ActionPolicyDecision {
                    autonomy_tier: AutonomyTier::T3Autonomous,
                    ..decision
                },
            )
            .unwrap_err();
        assert_eq!(too, OntologyEngineError::AutonomyTierExceeded);
    }

    // --- st1: endpoint-reference validation tests ---

    #[test]
    fn link_type_with_dangling_from_endpoint_rejected() {
        let mut engine = OntologyEngine::default();
        // Register only the "to" entity type; "from" is missing.
        engine
            .register_entity_type(
                EntityTypeDefinition::new(
                    "ten_clinic",
                    EntityTypeId::new("ety_appointment").unwrap(),
                    "Appointment",
                    vec![property("starts_at")],
                    1,
                )
                .unwrap(),
            )
            .unwrap();
        let link = LinkTypeDefinition::new(
            "ten_clinic",
            LinkTypeId::new("lty_missing_from").unwrap(),
            EntityTypeId::new("ety_patient").unwrap(), // not registered
            EntityTypeId::new("ety_appointment").unwrap(),
            LinkCardinality::OneToMany,
            false,
        )
        .unwrap();
        assert_eq!(
            engine.register_link_type(link),
            Err(OntologyEngineError::UnknownEntityTypeEndpoint)
        );
    }

    #[test]
    fn link_type_with_dangling_to_endpoint_rejected() {
        let mut engine = OntologyEngine::default();
        // Register only the "from" entity type; "to" is missing.
        engine.register_entity_type(patient_type()).unwrap();
        let link = LinkTypeDefinition::new(
            "ten_clinic",
            LinkTypeId::new("lty_missing_to").unwrap(),
            EntityTypeId::new("ety_patient").unwrap(),
            EntityTypeId::new("ety_appointment").unwrap(), // not registered
            LinkCardinality::OneToMany,
            false,
        )
        .unwrap();
        assert_eq!(
            engine.register_link_type(link),
            Err(OntologyEngineError::UnknownEntityTypeEndpoint)
        );
    }

    #[test]
    fn action_type_with_dangling_entity_type_rejected() {
        let mut engine = OntologyEngine::default();
        // No entity types registered at all.
        let action = ActionTypeDefinition::new(
            "ten_clinic",
            ActionTypeId::new("aty_discharge").unwrap(),
            EntityTypeId::new("ety_patient").unwrap(), // not registered
            "ontology.action.discharge",
            AutonomyTier::T1Assist,
            "EVT-DISCHARGE",
        )
        .unwrap();
        assert_eq!(
            engine.register_action_type(action),
            Err(OntologyEngineError::UnknownEntityTypeEndpoint)
        );
    }

    #[test]
    fn valid_link_and_action_type_registers_after_endpoints_present() {
        let mut engine = OntologyEngine::default();
        let patient = engine.register_entity_type(patient_type()).unwrap();
        let appointment = engine
            .register_entity_type(
                EntityTypeDefinition::new(
                    "ten_clinic",
                    EntityTypeId::new("ety_appointment").unwrap(),
                    "Appointment",
                    vec![property("starts_at")],
                    1,
                )
                .unwrap(),
            )
            .unwrap();
        // Both endpoints present: link type should register successfully.
        let link_id = engine
            .register_link_type(
                LinkTypeDefinition::new(
                    "ten_clinic",
                    LinkTypeId::new("lty_patient_appointment").unwrap(),
                    patient.clone(),
                    appointment,
                    LinkCardinality::OneToMany,
                    false,
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(link_id.value, "lty_patient_appointment");
        // Endpoint present: action type should register successfully.
        let action_id = engine
            .register_action_type(
                ActionTypeDefinition::new(
                    "ten_clinic",
                    ActionTypeId::new("aty_discharge").unwrap(),
                    patient,
                    "ontology.action.discharge",
                    AutonomyTier::T1Assist,
                    "EVT-DISCHARGE",
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(action_id.value, "aty_discharge");
    }

    // --- st2: pillar-consistency tests ---

    fn patient_type_with_pillar(pillar: OntologyPillar) -> EntityTypeDefinition {
        EntityTypeDefinition::new(
            "ten_hr",
            EntityTypeId::new("ety_person").unwrap(),
            "Person",
            vec![property("name")],
            1,
        )
        .unwrap()
        .with_pillar(pillar)
    }

    fn org_type_with_pillar(pillar: OntologyPillar) -> EntityTypeDefinition {
        EntityTypeDefinition::new(
            "ten_hr",
            EntityTypeId::new("ety_company").unwrap(),
            "Company",
            vec![property("name")],
            1,
        )
        .unwrap()
        .with_pillar(pillar)
    }

    fn agnostic_entity_type(id: &str) -> EntityTypeDefinition {
        EntityTypeDefinition::new(
            "ten_hr",
            EntityTypeId::new(id).unwrap(),
            "Agnostic",
            vec![property("name")],
            1,
        )
        .unwrap()
        // no with_pillar call — pillar: None
    }

    #[test]
    fn cross_pillar_link_org_to_person_rejected() {
        let mut engine = OntologyEngine::default();
        engine
            .register_entity_type(org_type_with_pillar(OntologyPillar::Org))
            .unwrap();
        engine
            .register_entity_type(patient_type_with_pillar(OntologyPillar::Person))
            .unwrap();
        let link = LinkTypeDefinition::new(
            "ten_hr",
            LinkTypeId::new("lty_org_person").unwrap(),
            EntityTypeId::new("ety_company").unwrap(),
            EntityTypeId::new("ety_person").unwrap(),
            LinkCardinality::OneToMany,
            false,
        )
        .unwrap();
        assert_eq!(
            engine.register_link_type(link),
            Err(OntologyEngineError::CrossPillarLink)
        );
    }

    #[test]
    fn cross_pillar_link_person_to_org_rejected() {
        let mut engine = OntologyEngine::default();
        engine
            .register_entity_type(patient_type_with_pillar(OntologyPillar::Person))
            .unwrap();
        engine
            .register_entity_type(org_type_with_pillar(OntologyPillar::Org))
            .unwrap();
        let link = LinkTypeDefinition::new(
            "ten_hr",
            LinkTypeId::new("lty_person_org").unwrap(),
            EntityTypeId::new("ety_person").unwrap(),
            EntityTypeId::new("ety_company").unwrap(),
            LinkCardinality::ManyToMany,
            false,
        )
        .unwrap();
        assert_eq!(
            engine.register_link_type(link),
            Err(OntologyEngineError::CrossPillarLink)
        );
    }

    #[test]
    fn same_pillar_link_org_to_org_accepted() {
        let mut engine = OntologyEngine::default();
        engine
            .register_entity_type(org_type_with_pillar(OntologyPillar::Org))
            .unwrap();
        let subsidiary = EntityTypeDefinition::new(
            "ten_hr",
            EntityTypeId::new("ety_subsidiary").unwrap(),
            "Subsidiary",
            vec![property("name")],
            1,
        )
        .unwrap()
        .with_pillar(OntologyPillar::Org);
        engine.register_entity_type(subsidiary).unwrap();
        let link = LinkTypeDefinition::new(
            "ten_hr",
            LinkTypeId::new("lty_parent_subsidiary").unwrap(),
            EntityTypeId::new("ety_company").unwrap(),
            EntityTypeId::new("ety_subsidiary").unwrap(),
            LinkCardinality::OneToMany,
            false,
        )
        .unwrap();
        assert!(engine.register_link_type(link).is_ok());
    }

    #[test]
    fn same_pillar_link_person_to_person_accepted() {
        let mut engine = OntologyEngine::default();
        engine
            .register_entity_type(patient_type_with_pillar(OntologyPillar::Person))
            .unwrap();
        let contact = EntityTypeDefinition::new(
            "ten_hr",
            EntityTypeId::new("ety_contact").unwrap(),
            "Contact",
            vec![property("email")],
            1,
        )
        .unwrap()
        .with_pillar(OntologyPillar::Person);
        engine.register_entity_type(contact).unwrap();
        let link = LinkTypeDefinition::new(
            "ten_hr",
            LinkTypeId::new("lty_person_contact").unwrap(),
            EntityTypeId::new("ety_person").unwrap(),
            EntityTypeId::new("ety_contact").unwrap(),
            LinkCardinality::OneToOne,
            false,
        )
        .unwrap();
        assert!(engine.register_link_type(link).is_ok());
    }

    #[test]
    fn pillar_agnostic_link_both_none_accepted() {
        let mut engine = OntologyEngine::default();
        engine
            .register_entity_type(agnostic_entity_type("ety_agnostic_a"))
            .unwrap();
        engine
            .register_entity_type(agnostic_entity_type("ety_agnostic_b"))
            .unwrap();
        let link = LinkTypeDefinition::new(
            "ten_hr",
            LinkTypeId::new("lty_agnostic").unwrap(),
            EntityTypeId::new("ety_agnostic_a").unwrap(),
            EntityTypeId::new("ety_agnostic_b").unwrap(),
            LinkCardinality::ManyToMany,
            false,
        )
        .unwrap();
        assert!(engine.register_link_type(link).is_ok());
    }

    #[test]
    fn one_pillar_agnostic_endpoint_accepted() {
        let mut engine = OntologyEngine::default();
        engine
            .register_entity_type(org_type_with_pillar(OntologyPillar::Org))
            .unwrap();
        engine
            .register_entity_type(agnostic_entity_type("ety_agnostic_b"))
            .unwrap();
        let link = LinkTypeDefinition::new(
            "ten_hr",
            LinkTypeId::new("lty_org_agnostic").unwrap(),
            EntityTypeId::new("ety_company").unwrap(),
            EntityTypeId::new("ety_agnostic_b").unwrap(),
            LinkCardinality::OneToMany,
            false,
        )
        .unwrap();
        assert!(engine.register_link_type(link).is_ok());
    }

    #[test]
    fn all_link_cardinality_variants_accepted_same_pillar() {
        for (link_id, cardinality) in [
            ("lty_one_one", LinkCardinality::OneToOne),
            ("lty_one_many", LinkCardinality::OneToMany),
            ("lty_many_many", LinkCardinality::ManyToMany),
        ] {
            let mut engine = OntologyEngine::default();
            engine
                .register_entity_type(
                    EntityTypeDefinition::new(
                        "ten_hr",
                        EntityTypeId::new("ety_from").unwrap(),
                        "From",
                        vec![property("x")],
                        1,
                    )
                    .unwrap()
                    .with_pillar(OntologyPillar::Org),
                )
                .unwrap();
            engine
                .register_entity_type(
                    EntityTypeDefinition::new(
                        "ten_hr",
                        EntityTypeId::new("ety_to").unwrap(),
                        "To",
                        vec![property("y")],
                        1,
                    )
                    .unwrap()
                    .with_pillar(OntologyPillar::Org),
                )
                .unwrap();
            let link = LinkTypeDefinition::new(
                "ten_hr",
                LinkTypeId::new(link_id).unwrap(),
                EntityTypeId::new("ety_from").unwrap(),
                EntityTypeId::new("ety_to").unwrap(),
                cardinality,
                false,
            )
            .unwrap();
            assert!(
                engine.register_link_type(link).is_ok(),
                "cardinality {:?} should be accepted",
                cardinality
            );
        }
    }
}

#[cfg(test)]
mod schema_evolution_tests {
    use super::*;
    use oya_data_boundary_kernel::{DataClass, PrivacyDataClass};

    fn internal() -> PrivacyDataClass {
        PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap()
    }

    fn pii() -> PrivacyDataClass {
        PrivacyDataClass::try_from(DataClass::PiiIdentifying).unwrap()
    }

    fn prop(
        name: &str,
        tier: PropertyTier,
        data_class: PrivacyDataClass,
        required: bool,
    ) -> EntityTypePropertyDefinition {
        EntityTypePropertyDefinition::new(name, tier, data_class, required).unwrap()
    }

    fn base_def(
        revision: u32,
        extra_props: Vec<EntityTypePropertyDefinition>,
    ) -> EntityTypeDefinition {
        let mut props = vec![prop("name", PropertyTier::Scalar, internal(), true)];
        props.extend(extra_props);
        EntityTypeDefinition::new(
            "ten_test",
            EntityTypeId::new("ety_thing").unwrap(),
            "Thing",
            props,
            revision,
        )
        .unwrap()
    }

    // --- check_schema_compatibility unit tests ---

    #[test]
    fn additive_property_is_accepted() {
        let prior = base_def(1, vec![]);
        let candidate = base_def(2, vec![prop("email", PropertyTier::Scalar, pii(), false)]);
        assert_eq!(check_schema_compatibility(&prior, &candidate), Ok(()));
    }

    #[test]
    fn tier_mutation_rejected() {
        let prior = base_def(1, vec![]);
        // Change "name" from Scalar → Vector.
        let candidate = EntityTypeDefinition::new(
            "ten_test",
            EntityTypeId::new("ety_thing").unwrap(),
            "Thing",
            vec![prop("name", PropertyTier::Vector, internal(), true)],
            2,
        )
        .unwrap();
        assert_eq!(
            check_schema_compatibility(&prior, &candidate),
            Err(OntologyEngineError::IncompatibleSchemaEvolution)
        );
    }

    #[test]
    fn data_class_mutation_rejected() {
        let prior = base_def(1, vec![]);
        // Change "name" from InternalOnly → PiiIdentifying.
        let candidate = EntityTypeDefinition::new(
            "ten_test",
            EntityTypeId::new("ety_thing").unwrap(),
            "Thing",
            vec![prop("name", PropertyTier::Scalar, pii(), true)],
            2,
        )
        .unwrap();
        assert_eq!(
            check_schema_compatibility(&prior, &candidate),
            Err(OntologyEngineError::IncompatibleSchemaEvolution)
        );
    }

    #[test]
    fn required_flip_rejected() {
        let prior = base_def(1, vec![]);
        // Flip "name" required: true → false.
        let candidate = EntityTypeDefinition::new(
            "ten_test",
            EntityTypeId::new("ety_thing").unwrap(),
            "Thing",
            vec![prop("name", PropertyTier::Scalar, internal(), false)],
            2,
        )
        .unwrap();
        assert_eq!(
            check_schema_compatibility(&prior, &candidate),
            Err(OntologyEngineError::IncompatibleSchemaEvolution)
        );
    }

    #[test]
    fn property_removal_rejected() {
        let prior = base_def(
            1,
            vec![prop("code", PropertyTier::Scalar, internal(), true)],
        );
        // candidate drops "code".
        let candidate = base_def(2, vec![]);
        assert_eq!(
            check_schema_compatibility(&prior, &candidate),
            Err(OntologyEngineError::IncompatibleSchemaEvolution)
        );
    }

    // --- OntologyEngine::evolve_entity_type tests ---

    #[test]
    fn first_registration_via_evolve_succeeds() {
        let mut engine = OntologyEngine::default();
        let def = base_def(1, vec![]);
        let id = engine.evolve_entity_type(def).unwrap();
        assert_eq!(id.value, "ety_thing");
        assert!(engine.entity_type("ten_test", &id).is_some());
    }

    #[test]
    fn monotonic_additive_evolution_accepted_and_stored() {
        let mut engine = OntologyEngine::default();
        engine.evolve_entity_type(base_def(1, vec![])).unwrap();

        let v2 = base_def(2, vec![prop("email", PropertyTier::Scalar, pii(), false)]);
        let id = engine.evolve_entity_type(v2).unwrap();
        assert_eq!(id.value, "ety_thing");

        let stored = engine.entity_type("ten_test", &id).unwrap();
        assert_eq!(stored.revision, 2);
        assert_eq!(stored.properties.len(), 2);
        assert!(stored.properties.iter().any(|p| p.name == "email"));
    }

    #[test]
    fn equal_revision_rejected_with_non_monotonic() {
        let mut engine = OntologyEngine::default();
        engine.evolve_entity_type(base_def(1, vec![])).unwrap();
        assert_eq!(
            engine.evolve_entity_type(base_def(1, vec![])),
            Err(OntologyEngineError::NonMonotonicRevision)
        );
    }

    #[test]
    fn lower_revision_rejected_with_non_monotonic() {
        let mut engine = OntologyEngine::default();
        engine.evolve_entity_type(base_def(5, vec![])).unwrap();
        assert_eq!(
            engine.evolve_entity_type(base_def(3, vec![])),
            Err(OntologyEngineError::NonMonotonicRevision)
        );
    }

    #[test]
    fn breaking_change_higher_revision_rejected() {
        let mut engine = OntologyEngine::default();
        engine.evolve_entity_type(base_def(1, vec![])).unwrap();

        // revision 1 → 2 but "name" tier is mutated.
        let breaking = EntityTypeDefinition::new(
            "ten_test",
            EntityTypeId::new("ety_thing").unwrap(),
            "Thing",
            vec![prop("name", PropertyTier::Vector, internal(), true)],
            2,
        )
        .unwrap();
        assert_eq!(
            engine.evolve_entity_type(breaking),
            Err(OntologyEngineError::IncompatibleSchemaEvolution)
        );
        // Stored definition must be unchanged after the rejection.
        let id = EntityTypeId::new("ety_thing").unwrap();
        let stored = engine.entity_type("ten_test", &id).unwrap();
        assert_eq!(stored.revision, 1);
    }
}
