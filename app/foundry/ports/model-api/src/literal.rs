use data_ontology_kernel::FiniteDouble;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ModelValue {
    String {
        value: String,
    },
    Integer {
        #[serde(with = "decimal")]
        value: i64,
    },
    Double {
        #[serde(with = "double")]
        value: FiniteDouble,
    },
    Boolean {
        value: bool,
    },
    Date {
        year: i32,
        month: u8,
        day: u8,
    },
    Timestamp {
        #[serde(with = "decimal")]
        epoch_millis: i64,
    },
    Array {
        items: Vec<ModelValue>,
    },
    Struct {
        #[serde(serialize_with = "struct_fields")]
        fields: std::collections::BTreeMap<String, ModelValue>,
    },
}

impl<'de> serde::Deserialize<'de> for ModelValue {
    fn deserialize<D: serde::Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        struct Object;
        impl<'de> serde::de::Visitor<'de> for Object {
            type Value = ModelValue;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a model literal object")
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                map: A,
            ) -> Result<Self::Value, A::Error> {
                let fields = <LiteralFields as serde::Deserialize>::deserialize(
                    serde::de::value::MapAccessDeserializer::new(map),
                )?;
                ModelValue::try_from(fields).map_err(serde::de::Error::custom)
            }
        }
        decoder.deserialize_map(Object)
    }
}

model_object!(ModelValueField {
    name: String,
    value: ModelValue,
});

#[derive(serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum LiteralFields {
    String {
        value: String,
    },
    Integer {
        #[serde(with = "decimal")]
        value: i64,
    },
    Double {
        #[serde(with = "double")]
        value: FiniteDouble,
    },
    Boolean {
        value: bool,
    },
    Date {
        year: i32,
        month: u8,
        day: u8,
    },
    Timestamp {
        #[serde(with = "decimal")]
        epoch_millis: i64,
    },
    Array {
        items: Vec<ModelValue>,
    },
    Struct {
        fields: Vec<ModelValueField>,
    },
}

impl TryFrom<LiteralFields> for ModelValue {
    type Error = &'static str;

    fn try_from(fields: LiteralFields) -> Result<Self, Self::Error> {
        let value = match fields {
            LiteralFields::String { value } => Self::String { value },
            LiteralFields::Integer { value } => Self::Integer { value },
            LiteralFields::Double { value } => Self::Double { value },
            LiteralFields::Boolean { value } => Self::Boolean { value },
            LiteralFields::Date { year, month, day } => Self::Date { year, month, day },
            LiteralFields::Timestamp { epoch_millis } => Self::Timestamp { epoch_millis },
            LiteralFields::Array { items } => Self::Array { items },
            LiteralFields::Struct { fields } => {
                let mut values = std::collections::BTreeMap::new();
                for field in fields {
                    if values.insert(field.name, field.value).is_some() {
                        return Err("duplicate literal struct field");
                    }
                }
                Self::Struct { fields: values }
            }
        };
        if let Self::Date { year, month, day } = &value {
            data_ontology_kernel::CalendarDate::new(*year, *month, *day)
                .map_err(|_| "invalid calendar date")?;
        }
        Ok(value)
    }
}

fn struct_fields<S: serde::Serializer>(
    fields: &std::collections::BTreeMap<String, ModelValue>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    use serde::ser::SerializeSeq;
    #[derive(serde::Serialize)]
    struct Field<'a> {
        name: &'a str,
        value: &'a ModelValue,
    }
    let mut rows = serializer.serialize_seq(Some(fields.len()))?;
    for (name, value) in fields {
        rows.serialize_element(&Field { name, value })?;
    }
    rows.end()
}

mod decimal {
    pub(super) fn serialize<S: serde::Serializer>(
        value: &i64,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&value.to_string())
    }

    pub(super) fn deserialize<'de, D: serde::Deserializer<'de>>(
        decoder: D,
    ) -> Result<i64, D::Error> {
        let text = <String as serde::Deserialize>::deserialize(decoder)?;
        let parsed = text.parse::<i64>().ok();
        parsed
            .filter(|value| value.to_string() == text)
            .ok_or_else(|| serde::de::Error::custom("expected canonical i64 decimal string"))
    }
}

mod double {
    use data_ontology_kernel::FiniteDouble;

    pub(super) fn serialize<S: serde::Serializer>(
        value: &FiniteDouble,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(value.get())
    }

    pub(super) fn deserialize<'de, D: serde::Deserializer<'de>>(
        decoder: D,
    ) -> Result<FiniteDouble, D::Error> {
        let value = <f64 as serde::Deserialize>::deserialize(decoder)?;
        FiniteDouble::new(value).map_err(|_| serde::de::Error::custom("expected finite double"))
    }
}
