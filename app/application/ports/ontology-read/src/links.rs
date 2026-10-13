use serde::{Deserialize, Serialize};

use crate::OntologyReadError;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct LinkSchema {
    pub link_type: String,            // data_class: INTERNAL_ONLY
    pub from_entity_type: String,     // data_class: INTERNAL_ONLY
    pub to_entity_type: String,       // data_class: INTERNAL_ONLY
    pub cardinality: LinkCardinality, // data_class: INTERNAL_ONLY
    pub allow_cross_tenant: bool,     // data_class: INTERNAL_ONLY
    pub revision: u32,                // data_class: INTERNAL_ONLY
}

#[derive(Deserialize)]
#[serde(remote = "LinkSchema", deny_unknown_fields)]
struct LinkSchemaFields {
    link_type: String,            // data_class: INTERNAL_ONLY
    from_entity_type: String,     // data_class: INTERNAL_ONLY
    to_entity_type: String,       // data_class: INTERNAL_ONLY
    cardinality: LinkCardinality, // data_class: INTERNAL_ONLY
    allow_cross_tenant: bool,     // data_class: INTERNAL_ONLY
    revision: u32,                // data_class: INTERNAL_ONLY
}

impl<'de> Deserialize<'de> for LinkSchema {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ObjectRow;
        impl<'de> serde::de::Visitor<'de> for ObjectRow {
            type Value = LinkSchema;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a link schema object")
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                map: A,
            ) -> Result<Self::Value, A::Error> {
                LinkSchemaFields::deserialize(serde::de::value::MapAccessDeserializer::new(map))
            }
        }
        deserializer.deserialize_map(ObjectRow)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkCardinality {
    OneToOne,
    OneToMany,
    ManyToMany,
}

impl<'de> Deserialize<'de> for LinkCardinality {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "one_to_one" => Ok(Self::OneToOne),
            "one_to_many" => Ok(Self::OneToMany),
            "many_to_many" => Ok(Self::ManyToMany),
            _ => Err(serde::de::Error::unknown_variant(
                &value,
                &["one_to_one", "one_to_many", "many_to_many"],
            )),
        }
    }
}

impl LinkSchema {
    pub fn validate(&self) -> Result<(), OntologyReadError> {
        for (value, prefix) in [
            (&self.link_type, "lty_"),
            (&self.from_entity_type, "ety_"),
            (&self.to_entity_type, "ety_"),
        ] {
            if !value.starts_with(prefix) || value.len() <= prefix.len() {
                return Err(OntologyReadError::SourceUnavailable);
            }
        }
        Ok(())
    }
}
