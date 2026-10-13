use std::collections::BTreeSet;

use serde::{Deserialize, Deserializer, Serialize};

use crate::OntologyReadError;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntitySchema {
    pub entity_type: String,                 // data_class: INTERNAL_ONLY
    pub revision: u32,                       // data_class: INTERNAL_ONLY
    pub properties: Vec<PropertyDefinition>, // data_class: INTERNAL_ONLY
    #[serde(deserialize_with = "nullable")]
    pub primary_key_property: Option<String>, // data_class: INTERNAL_ONLY
    #[serde(deserialize_with = "nullable")]
    pub title_property: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PropertyDefinition {
    pub name: String,       // data_class: INTERNAL_ONLY
    pub tier: PropertyTier, // data_class: INTERNAL_ONLY
    pub data_class: String, // data_class: INTERNAL_ONLY
    pub required: bool,     // data_class: INTERNAL_ONLY
    /// Explicit null preserves the native legacy string contract.
    #[serde(deserialize_with = "nullable")]
    pub value_type: Option<ValueTypeDeclaration>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyTier {
    Scalar,
    Vector,
    Timeseries,
    Geo,
    Ciphertext,
    Struct,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ValueTypeDeclaration {
    String {},
    Integer {},
    Double {},
    Boolean {},
    Date {},
    Timestamp {},
    Array { element: Box<Self> },
    Struct { fields: Vec<StructFieldDeclaration> },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructFieldDeclaration {
    pub name: String,                     // data_class: INTERNAL_ONLY
    pub required: bool,                   // data_class: INTERNAL_ONLY
    pub value_type: ValueTypeDeclaration, // data_class: INTERNAL_ONLY
}

// Missing nullable metadata is unavailable; only an explicit null means None.
fn nullable<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

impl EntitySchema {
    pub fn validate(&self) -> Result<(), OntologyReadError> {
        let mut names = BTreeSet::new();
        if !self.entity_type.starts_with("ety_")
            || self.entity_type.len() <= 4
            || self.properties.is_empty()
        {
            return Err(OntologyReadError::SourceUnavailable);
        }
        for property in &self.properties {
            if property.name.trim().is_empty()
                || !names.insert(property.name.as_str())
                || !canonical_privacy_label(&property.data_class)
            {
                return Err(OntologyReadError::SourceUnavailable);
            }
            if let Some(value_type) = &property.value_type
                && (value_type.tier() != property.tier || !value_type.valid_at_depth(1))
            {
                return Err(OntologyReadError::SourceUnavailable);
            }
        }
        if self
            .title_property
            .as_ref()
            .is_some_and(|name| !names.contains(name.as_str()))
            || self.primary_key_property.as_ref().is_some_and(|name| {
                !self.properties.iter().any(|property| {
                    &property.name == name
                        && property.required
                        && (property.value_type.is_none() || property.tier == PropertyTier::Scalar)
                })
            })
        {
            return Err(OntologyReadError::SourceUnavailable);
        }
        Ok(())
    }
}

impl ValueTypeDeclaration {
    fn tier(&self) -> PropertyTier {
        match self {
            Self::Array { .. } => PropertyTier::Vector,
            Self::Struct { .. } => PropertyTier::Struct,
            Self::String {}
            | Self::Integer {}
            | Self::Double {}
            | Self::Boolean {}
            | Self::Date {}
            | Self::Timestamp {} => PropertyTier::Scalar,
        }
    }

    fn valid_at_depth(&self, depth: usize) -> bool {
        if depth > 8 {
            return false;
        }
        match self {
            Self::Array { element } => element.valid_at_depth(depth + 1),
            Self::Struct { fields } => {
                let mut names = BTreeSet::new();
                !fields.is_empty()
                    && fields.iter().all(|field| {
                        !field.name.trim().is_empty()
                            && names.insert(field.name.as_str())
                            && field.value_type.valid_at_depth(depth + 1)
                    })
            }
            Self::String {}
            | Self::Integer {}
            | Self::Double {}
            | Self::Boolean {}
            | Self::Date {}
            | Self::Timestamp {} => true,
        }
    }
}

fn canonical_privacy_label(label: &str) -> bool {
    matches!(
        label,
        "INTERNAL_ONLY"
            | "PHI"
            | "PII_IDENTIFYING"
            | "PII_QUASI_IDENTIFIER"
            | "PCI"
            | "FINANCIAL"
            | "FINANCIAL_REGULATED_CREDIT"
            | "BEHAVIORAL_TENANT_PRODUCT"
            | "BEHAVIORAL_ADS"
            | "DECLARED_PREFERENCE"
            | "SEARCH_QUERY"
            | "PUBLIC"
            | "SENSITIVE_PIPA_ART23"
    )
}
