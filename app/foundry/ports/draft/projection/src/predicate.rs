//! Property predicates: the indexed read surface. `Equals` is exact
//! typed equality; `Range` is inclusive and class-scoped — comparison
//! only means something within one [`StorageClass`], so a mixed-class
//! range is refused at construction and a class-mismatched stored value
//! is refused at match time (schema drift is loud, never a silent
//! false). An absent property is simply no match.

use data_ontology_kernel::{ObjectEntity, PropertyValue};

use crate::store::ProjectionStoreError;

/// Why a predicate could not be constructed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PredicateError {
    BlankProperty,
    /// `from` and `to` carry different storage classes.
    MixedStorageClasses,
    /// `from` sorts after `to`; an inverted range is ambiguity, not an
    /// empty result.
    InvertedRange,
}

/// A type-scoped filter over one declared property.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PropertyPredicate {
    Equals {
        property: String,     // data_class: INTERNAL_ONLY
        value: PropertyValue, // data_class: PII_IDENTIFYING
    },
    Range {
        property: String,    // data_class: INTERNAL_ONLY
        from: PropertyValue, // data_class: PII_IDENTIFYING
        to: PropertyValue,   // data_class: PII_IDENTIFYING
    },
}

impl PropertyPredicate {
    pub fn equals(
        property: impl Into<String>,
        value: PropertyValue,
    ) -> Result<Self, PredicateError> {
        let property = non_blank(property.into())?;
        Ok(Self::Equals { property, value })
    }

    /// An inclusive range; both bounds must share one storage class and
    /// `from` must not sort after `to`.
    pub fn range(
        property: impl Into<String>,
        from: PropertyValue,
        to: PropertyValue,
    ) -> Result<Self, PredicateError> {
        let property = non_blank(property.into())?;
        if from.storage_class() != to.storage_class() {
            return Err(PredicateError::MixedStorageClasses);
        }
        if from > to {
            return Err(PredicateError::InvertedRange);
        }
        Ok(Self::Range { property, from, to })
    }

    /// Whether `entity` matches. `Ok(false)` is a real answer; a
    /// class-mismatched stored value under a `Range` is an error.
    pub(crate) fn matches(&self, entity: &ObjectEntity) -> Result<bool, ProjectionStoreError> {
        match self {
            Self::Equals { property, value } => Ok(entity
                .properties
                .get(property)
                .is_some_and(|stored| &stored.value.value == value)),
            Self::Range { property, from, to } => {
                let Some(stored) = entity.properties.get(property) else {
                    return Ok(false);
                };
                let stored = &stored.value.value;
                if stored.storage_class() != from.storage_class() {
                    return Err(ProjectionStoreError::ClassMismatch {
                        property: property.clone(),
                    });
                }
                Ok(stored >= from && stored <= to)
            }
        }
    }
}

fn non_blank(property: String) -> Result<String, PredicateError> {
    if property.trim().is_empty() || property.trim() != property {
        return Err(PredicateError::BlankProperty);
    }
    Ok(property)
}
