use serde::{Deserialize, Serialize};

use crate::OntologyReadError;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EntityTypeLabel {
    pub entity_type: String,  // data_class: INTERNAL_ONLY
    pub revision: u32,        // data_class: INTERNAL_ONLY
    pub display_name: String, // data_class: INTERNAL_ONLY
}

#[derive(Deserialize)]
#[serde(remote = "EntityTypeLabel", deny_unknown_fields)]
struct EntityTypeLabelFields {
    entity_type: String,  // data_class: INTERNAL_ONLY
    revision: u32,        // data_class: INTERNAL_ONLY
    display_name: String, // data_class: INTERNAL_ONLY
}

impl<'de> Deserialize<'de> for EntityTypeLabel {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ObjectRow;
        impl<'de> serde::de::Visitor<'de> for ObjectRow {
            type Value = EntityTypeLabel;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("an entity type label object")
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                map: A,
            ) -> Result<Self::Value, A::Error> {
                EntityTypeLabelFields::deserialize(serde::de::value::MapAccessDeserializer::new(
                    map,
                ))
            }
        }
        deserializer.deserialize_map(ObjectRow)
    }
}

impl EntityTypeLabel {
    pub fn validate(&self) -> Result<(), OntologyReadError> {
        if !self.entity_type.starts_with("ety_")
            || self.entity_type.len() <= 4
            || self.display_name.trim().is_empty()
        {
            return Err(OntologyReadError::SourceUnavailable);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_wire_requires_one_complete_object_and_preserves_native_unicode_text() {
        let wire =
            r#"{"entity_type":"ety_Ω<script>","revision":0,"display_name":"기록 <img src=x>"}"#;
        let row: EntityTypeLabel = serde_json::from_str(wire).unwrap();
        assert_eq!(row.validate(), Ok(()));
        assert_eq!(
            serde_json::to_value(row).unwrap(),
            serde_json::from_str::<serde_json::Value>(wire).unwrap()
        );
        for invalid in [
            r#"["ety_record",1,"Record"]"#,
            r#"{"entity_type":"ety_record","revision":1}"#,
            r#"{"entity_type":"ety_record","display_name":"Record"}"#,
            r#"{"revision":1,"display_name":"Record"}"#,
            r#"{"entity_type":"ety_record","revision":1,"display_name":"Record","extra":true}"#,
            r#"{"entity_type":"ety_record","revision":1,"display_name":"Record","display_name":"Other"}"#,
            r#"{"entity_type":"ety_record","revision":-1,"display_name":"Record"}"#,
            r#"{"entity_type":"ety_record","revision":4294967296,"display_name":"Record"}"#,
        ] {
            assert!(
                serde_json::from_str::<EntityTypeLabel>(invalid).is_err(),
                "{invalid}"
            );
        }
        for (entity_type, display_name) in [
            ("ety_", "Record"),
            ("ent_record", "Record"),
            ("ety_record", "\u{2003}\t"),
        ] {
            assert_eq!(
                EntityTypeLabel {
                    entity_type: entity_type.into(),
                    revision: 0,
                    display_name: display_name.into()
                }
                .validate(),
                Err(OntologyReadError::SourceUnavailable)
            );
        }
    }
}
