//! The engine-level error vocabulary shared by every kernel operation.

use crate::definitions::LinkCardinality;

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
    /// isolation. Both endpoints must share the same [`OntologyPillar`](crate::OntologyPillar), or at
    /// least one must be pillar-agnostic (`pillar: None`).
    CrossPillarLink,
    /// The candidate revision is not strictly greater than the stored revision.
    /// [`OntologyEngine::evolve_entity_type`](crate::OntologyEngine::evolve_entity_type) requires
    /// `candidate.revision > stored.revision`.
    NonMonotonicRevision,
    /// The candidate definition removes or mutates an existing property, or
    /// introduces a new property with `required: true`.
    /// [`OntologyEngine::evolve_entity_type`](crate::OntologyEngine::evolve_entity_type) only allows additive changes:
    /// every prior property must remain with unchanged `tier`, `data_class`,
    /// and `required` flag, and a new property must be optional — every
    /// object projected under the prior revision lacks it, so a required
    /// new property would invalidate the existing population.
    IncompatibleSchemaEvolution,
    /// The candidate definition changes the stored [`OntologyPillar`](crate::OntologyPillar)
    /// annotation (including adding or removing it). Link types were
    /// endpoint-validated against the stored pillar at registration time
    /// ([`OntologyEngineError::CrossPillarLink`]), so the pillar is
    /// immutable under [`OntologyEngine::evolve_entity_type`](crate::OntologyEngine::evolve_entity_type).
    PillarChangedOnEvolution,
    /// `register_link_instance` was called with a [`LinkTypeId`] that has not
    /// been registered for the tenant. Register the link type before creating
    /// instances of it.
    UnknownLinkType,
    /// An instance names a definition property with `required: true` that is
    /// absent from the instance's property set
    /// ([`OntologyEngine::check_instance_conformance`](crate::OntologyEngine::check_instance_conformance)).
    MissingRequiredProperty {
        /// Name of the required property the instance lacks.
        name: String,
    },
    /// An instance carries a property its entity type definition does not
    /// declare. Conformance is fail-closed on vocabulary: evolve the type
    /// (additively) before writing the property.
    UndeclaredProperty {
        /// Name of the property the definition does not declare.
        name: String,
    },
    /// An instance property's [`PropertyTier`](crate::PropertyTier) differs
    /// from the tier its definition declares.
    PropertyTierMismatch {
        /// Name of the mismatched property.
        name: String,
    },
    /// An instance property's data class differs from the class its
    /// definition declares.
    PropertyDataClassMismatch {
        /// Name of the mismatched property.
        name: String,
    },
    /// A link instance would violate the [`LinkCardinality`] declared on the
    /// [`LinkTypeDefinition`]. The `cardinality` field carries the constraint
    /// that was violated.
    CardinalityViolation {
        /// The cardinality constraint that was violated.
        cardinality: LinkCardinality,
    },
}
