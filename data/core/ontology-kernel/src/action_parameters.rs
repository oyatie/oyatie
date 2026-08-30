//! Typed action parameters: the declared input schema of an
//! [`ActionTypeDefinition`](crate::ActionTypeDefinition), reusing the
//! property-tier and privacy-data-class vocabulary of the type plane.

use data_boundary_kernel::PrivacyDataClass;

use crate::error::OntologyEngineError;
use crate::property::PropertyTier;

/// One declared parameter of an action type: name, tier, data class, and
/// whether a submission must carry it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionParameterDefinition {
    pub name: String,       // data_class: INTERNAL_ONLY
    pub tier: PropertyTier, // data_class: INTERNAL_ONLY
    pub data_class: PrivacyDataClass,
    pub required: bool, // data_class: INTERNAL_ONLY
}

impl ActionParameterDefinition {
    pub fn new(
        name: impl Into<String>,
        tier: PropertyTier,
        data_class: PrivacyDataClass,
        required: bool,
    ) -> Result<Self, OntologyEngineError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(OntologyEngineError::EmptyParameterName);
        }
        Ok(Self {
            name,
            tier,
            data_class,
            required,
        })
    }
}
