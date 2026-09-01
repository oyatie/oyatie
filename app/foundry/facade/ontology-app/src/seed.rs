//! The registry seed: the per-tenant `OntologyEngine` the fold and the
//! writer both read as the law in force.
//!
//! The seed is CODE, not configuration, and that is forced rather than
//! chosen: a handwritten yaml or json is inadmissible at a capability root,
//! and `OntologyEngine` carries no serialization. The durable, operator-
//! authored registry is the Ontology Manager vertical's charter; this module
//! is the seam it will replace and must not quietly grow into it.

use data_boundary_kernel::{DataClass, PrivacyDataClass};
use data_ontology_kernel::{
    ActionTypeDefinition, ActionTypeId, AutonomyTier, EntityTypeDefinition, EntityTypeId,
    EntityTypePropertyDefinition, OntologyEngine, OntologyEngineError, PropertyTier,
};

/// Why a tenant could not be seeded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SeedError {
    /// The tenant id is not the kernel's `ten_` vocabulary, or a definition
    /// the seed declares was refused by the kernel.
    Refused(OntologyEngineError),
    /// The label this seed classifies its properties with is not a privacy
    /// class. Unreachable while the seed names `InternalOnly`, and typed
    /// rather than unwrapped so it stays that way.
    DataClassRefused,
}

impl std::fmt::Display for SeedError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Refused(error) => write!(formatter, "the registry refused the seed: {error:?}"),
            Self::DataClassRefused => {
                write!(formatter, "the seed's data class is not a privacy class")
            }
        }
    }
}

impl From<OntologyEngineError> for SeedError {
    fn from(error: OntologyEngineError) -> Self {
        Self::Refused(error)
    }
}

fn internal() -> Result<PrivacyDataClass, SeedError> {
    PrivacyDataClass::try_from(DataClass::InternalOnly).map_err(|_| SeedError::DataClassRefused)
}

/// The registry for one tenant. Every definition here is law the writer
/// stamps against and the fold re-checks, so the seed is deliberately the
/// smallest coherent ontology rather than a demonstration.
pub fn registry_for(tenant_id: &str) -> Result<OntologyEngine, SeedError> {
    build(tenant_id)
}

fn build(tenant_id: &str) -> Result<OntologyEngine, SeedError> {
    let mut engine = OntologyEngine::default();
    let record = EntityTypeId::new("ety_record")?;
    engine.register_entity_type(
        EntityTypeDefinition::new(
            tenant_id,
            record.clone(),
            "Record",
            vec![
                EntityTypePropertyDefinition::new("name", PropertyTier::Scalar, internal()?, true)?,
                EntityTypePropertyDefinition::new(
                    "note",
                    PropertyTier::Scalar,
                    internal()?,
                    false,
                )?,
            ],
            1,
        )?
        .with_title_property("name"),
    )?;
    engine.register_action_type(ActionTypeDefinition::new(
        tenant_id,
        ActionTypeId::new("aty_record_write")?,
        record,
        "ops-console",
        AutonomyTier::T1Assist,
        "record.written",
    )?)?;
    Ok(engine)
}
