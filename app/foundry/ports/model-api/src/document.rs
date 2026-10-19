use crate::{ModelError, ModelPrivacy, ModelTier, ModelValue, ModelValueType};

pub const MAX_MODEL_BYTES: usize = 1_048_576;
pub const MAX_MODEL_ITEMS: usize = 256;
pub const MAX_MODEL_TEXT_BYTES: usize = 4096;
pub const MAX_MODEL_VALUE_NODES: usize = 4096;

model_object!(ModelDisplay {
    display_name: Option<String>,
    description: Option<String>,
    icon: Option<String>,
    color: Option<String>,
    plural_name: Option<String>,
});

model_object!(ModelProperty {
    name: String,
    tier: ModelTier,
    data_class: ModelPrivacy,
    required: bool,
    value_type: Option<ModelValueType>,
    display: Option<ModelDisplay>,
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "crate::wire::present_default")]
    default_value: Option<ModelValue>,
});

model_object!(ModelEntity {
    id: String,
    display_name: String,
    properties: Vec<ModelProperty>,
    revision: u32,
    pillar: Option<ModelPillar>,
    primary_key_property: Option<String>,
    title_property: Option<String>,
    display: Option<ModelDisplay>,
});

model_object!(ModelLink {
    id: String,
    from_entity_type: String,
    to_entity_type: String,
    cardinality: ModelCardinality,
    allow_cross_tenant: bool,
    revision: u32,
    display: Option<ModelDisplay>,
});

model_object!(ModelNamedStringType {
    id: String,
    revision: u32,
    display_name: String,
    members: Vec<String>,
});

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct ModelDocument {
    pub entities: Vec<ModelEntity>,
    pub links: Vec<ModelLink>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub named_string_types: Vec<ModelNamedStringType>,
}
impl<'de> serde::Deserialize<'de> for ModelDocument {
    fn deserialize<D: serde::Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Fields {
            entities: Vec<ModelEntity>,
            links: Vec<ModelLink>,
            #[serde(default)]
            named_string_types: Vec<ModelNamedStringType>,
        }
        struct Object;
        impl<'de> serde::de::Visitor<'de> for Object {
            type Value = ModelDocument;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a ModelDocument object")
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                map: A,
            ) -> Result<Self::Value, A::Error> {
                let fields = <Fields as serde::Deserialize>::deserialize(
                    serde::de::value::MapAccessDeserializer::new(map),
                )?;
                Ok(ModelDocument {
                    entities: fields.entities,
                    links: fields.links,
                    named_string_types: fields.named_string_types,
                })
            }
        }
        decoder.deserialize_map(Object)
    }
}

model_object!(ModelBaseline {
    document: ModelDocument,
    prior_entity_revisions: Vec<ModelEntity>,
});

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelPillar {
    Org,
    Person,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelCardinality {
    OneToOne,
    OneToMany,
    ManyToMany,
}

impl ModelDocument {
    pub fn decode(bytes: &[u8]) -> Result<Self, ModelError> {
        if bytes.len() > MAX_MODEL_BYTES {
            return Err(ModelError::Limit {
                path: "document".into(),
            });
        }
        let model: Self =
            serde_json::from_slice(bytes).map_err(|_| ModelError::MalformedDocument)?;
        model.check_bounds()?;
        Ok(model)
    }

    pub fn encode(&self) -> Result<Vec<u8>, ModelError> {
        self.check_bounds()?;
        serde_json::to_vec(self).map_err(|_| ModelError::MalformedDocument)
    }
}
