use crate::{OntologyReadError, relationships::identifier};

/// Captured human detail intent; the source checks the type on the returned object.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectPointQuery {
    pub entity_type: String, // data_class: INTERNAL_ONLY
    pub object_ref: String,  // data_class: TENANT_SCOPED
    pub revision: u32,       // data_class: INTERNAL_ONLY
}

impl ObjectPointQuery {
    pub fn validate(&self) -> Result<(), OntologyReadError> {
        if !identifier(&self.entity_type, "ety_") || !identifier(&self.object_ref, "ent_") {
            return Err(OntologyReadError::InvalidQuery);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_intent_preserves_exact_bounded_ids_and_native_revision_range() {
        for suffix in [
            "record",
            "+%&=#/?",
            "기록",
            "a b",
            "\u{feff}",
            &"가".repeat(84),
            &"x".repeat(252),
        ] {
            for revision in [0, u32::MAX] {
                assert_eq!(
                    ObjectPointQuery {
                        entity_type: format!("ety_{suffix}"),
                        object_ref: format!("ent_{suffix}"),
                        revision,
                    }
                    .validate(),
                    Ok(()),
                    "{suffix:?}"
                );
            }
        }
        for id in [
            "",
            "ety_",
            "ent_wrong",
            " ety_a",
            "ety_a\u{85}",
            "ety_a\n",
            "ety_\u{7f}",
            &format!("ety_{}", "가".repeat(85)),
            &format!("ety_{}", "x".repeat(253)),
        ] {
            assert_eq!(
                ObjectPointQuery {
                    entity_type: id.into(),
                    object_ref: "ent_a".into(),
                    revision: 1
                }
                .validate(),
                Err(OntologyReadError::InvalidQuery),
                "{id:?}"
            );
        }
        for id in [
            "",
            "ent_",
            "ety_wrong",
            "ent_a ",
            "ent_a\u{9f}",
            &format!("ent_{}", "x".repeat(253)),
        ] {
            assert_eq!(
                ObjectPointQuery {
                    entity_type: "ety_a".into(),
                    object_ref: id.into(),
                    revision: 1
                }
                .validate(),
                Err(OntologyReadError::InvalidQuery),
                "{id:?}"
            );
        }
    }
}
