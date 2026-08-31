//! Display metadata for ontology definitions: operator-facing rendering
//! hints, never semantic. Every field is optional; a present field must be
//! non-blank (validated at the engine boundary). Display metadata evolves
//! freely across revisions — it is exactly the class of field the
//! frozen-field law does not cover.

use crate::error::OntologyEngineError;

/// Rendering hints shared by every definition kind. `display_name` is the
/// label for kinds whose identity field is not already a display name
/// (properties, links, actions); entity types keep their existing required
/// `display_name` and use the rest.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DisplayMetadata {
    pub display_name: Option<String>, // data_class: INTERNAL_ONLY
    pub description: Option<String>,  // data_class: INTERNAL_ONLY
    pub icon: Option<String>,         // data_class: INTERNAL_ONLY
    pub color: Option<String>,        // data_class: INTERNAL_ONLY
    pub plural_name: Option<String>,  // data_class: INTERNAL_ONLY
}

impl DisplayMetadata {
    /// A present field must carry content: blank display strings render as
    /// nothing while looking configured, so they are refused with the
    /// field named.
    pub fn validate(&self) -> Result<(), OntologyEngineError> {
        for (field, value) in [
            ("display_name", &self.display_name),
            ("description", &self.description),
            ("icon", &self.icon),
            ("color", &self.color),
            ("plural_name", &self.plural_name),
        ] {
            if let Some(text) = value
                && text.trim().is_empty()
            {
                return Err(OntologyEngineError::BlankDisplayField {
                    field: field.to_string(),
                });
            }
        }
        Ok(())
    }
}

/// Validate an optional display block, treating `None` as trivially valid.
pub(crate) fn check_display_integrity(
    display: Option<&DisplayMetadata>,
) -> Result<(), OntologyEngineError> {
    display.map_or(Ok(()), DisplayMetadata::validate)
}
