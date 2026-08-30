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
