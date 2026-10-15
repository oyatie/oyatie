use crate::ModelError;
use data_ontology_kernel::{
    EntityTypePropertyDefinition, OntologyEngine, StructFieldDeclaration, ValueTypeDeclaration,
};
use serde::{
    Serialize, Serializer,
    ser::{SerializeSeq, SerializeStruct},
};
use std::{
    io::{self, Write},
    sync::LazyLock,
};

pub const MAX_ONTOLOGY_SCHEMA_BYTES: usize = 1_048_576;

pub fn schema_bytes(engine: &OntologyEngine, tenant: &str) -> Result<Vec<u8>, ModelError> {
    let mut output = SchemaBuffer::default();
    match serde_json::to_writer(&mut output, &SchemaRows(engine, tenant)) {
        Ok(()) => Ok(output.bytes),
        Err(_) if output.exceeded => Err(ModelError::Limit {
            path: "schemas".into(),
        }),
        Err(_) => Err(ModelError::MalformedDocument),
    }
}

#[derive(Default)]
struct SchemaBuffer {
    bytes: Vec<u8>,
    exceeded: bool,
}
impl Write for SchemaBuffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let Some(length) = self
            .bytes
            .len()
            .checked_add(bytes.len())
            .filter(|length| *length <= MAX_ONTOLOGY_SCHEMA_BYTES)
        else {
            self.exceeded = true;
            return Err(io::Error::other("ontology schema response bound exceeded"));
        };
        if length > self.bytes.capacity() {
            let capacity = self
                .bytes
                .capacity()
                .saturating_mul(2)
                .max(8192)
                .max(length)
                .min(MAX_ONTOLOGY_SCHEMA_BYTES);
            self.bytes.reserve_exact(capacity - self.bytes.len());
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct SchemaRows<'a>(&'a OntologyEngine, &'a str);
impl Serialize for SchemaRows<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut rows = serializer.serialize_seq(None)?;
        for definition in self.0.entity_types(self.1) {
            rows.serialize_element(&EntitySchemaRow {
                entity_type: &definition.id.value,
                revision: definition.revision,
                properties: PropertyRows(&definition.properties),
                primary_key_property: definition.primary_key_property.as_deref(),
                title_property: definition.title_property.as_deref(),
            })?;
        }
        rows.end()
    }
}

#[derive(Serialize)]
struct EntitySchemaRow<'a> {
    entity_type: &'a str,                  // data_class: INTERNAL_ONLY
    revision: u32,                         // data_class: INTERNAL_ONLY
    properties: PropertyRows<'a>,          // data_class: INTERNAL_ONLY
    primary_key_property: Option<&'a str>, // data_class: INTERNAL_ONLY
    title_property: Option<&'a str>,       // data_class: INTERNAL_ONLY
}

struct PropertyRows<'a>(&'a [EntityTypePropertyDefinition]);
impl Serialize for PropertyRows<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut rows = serializer.serialize_seq(Some(self.0.len()))?;
        for property in self.0 {
            rows.serialize_element(&PropertyDefinitionRow {
                name: &property.name,
                tier: property.tier.wire_label(),
                data_class: property.data_class.label(),
                required: property.required,
                value_type: property.value_type.as_ref().map(DeclaredValueType),
            })?;
        }
        rows.end()
    }
}

#[derive(Serialize)]
struct PropertyDefinitionRow<'a> {
    name: &'a str,                             // data_class: INTERNAL_ONLY
    tier: &'static str,                        // data_class: INTERNAL_ONLY
    data_class: &'static str,                  // data_class: INTERNAL_ONLY
    required: bool,                            // data_class: INTERNAL_ONLY
    value_type: Option<DeclaredValueType<'a>>, // data_class: INTERNAL_ONLY
}

// A fixed two-key probe preserves both supported serde_json map orders without cloning schema data.
static LEGACY_INSERTION_ORDER: LazyLock<bool> = LazyLock::new(|| {
    let mut map = serde_json::Map::new();
    map.insert("kind".into(), serde_json::Value::Null);
    map.insert("element".into(), serde_json::Value::Null);
    map.keys().next().is_some_and(|key| key == "kind")
});

struct DeclaredValueType<'a>(&'a ValueTypeDeclaration);
impl Serialize for DeclaredValueType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // The old JSON map's key order depends on serde_json feature unification.
        let mut row = serializer.serialize_struct(
            "ValueType",
            match self.0 {
                ValueTypeDeclaration::NamedString(_) => 5,
                ValueTypeDeclaration::Scalar(_) => 1,
                _ => 2,
            },
        )?;
        match self.0 {
            ValueTypeDeclaration::NamedString(definition) => {
                if *LEGACY_INSERTION_ORDER {
                    row.serialize_field("kind", "named_string")?;
                    row.serialize_field("type_id", definition.reference().id().as_str())?;
                    row.serialize_field("revision", &definition.reference().revision())?;
                    row.serialize_field("display_name", &definition.display_name().value)?;
                    row.serialize_field("members", definition.members())?;
                } else {
                    row.serialize_field("display_name", &definition.display_name().value)?;
                    row.serialize_field("kind", "named_string")?;
                    row.serialize_field("members", definition.members())?;
                    row.serialize_field("revision", &definition.reference().revision())?;
                    row.serialize_field("type_id", definition.reference().id().as_str())?;
                }
            }
            ValueTypeDeclaration::Scalar(_) => row.serialize_field("kind", self.0.type_label())?,
            ValueTypeDeclaration::Array { element } => {
                if *LEGACY_INSERTION_ORDER {
                    row.serialize_field("kind", "array")?;
                }
                row.serialize_field("element", &DeclaredValueType(element))?;
                if !*LEGACY_INSERTION_ORDER {
                    row.serialize_field("kind", "array")?;
                }
            }
            ValueTypeDeclaration::Struct(schema) => {
                if *LEGACY_INSERTION_ORDER {
                    row.serialize_field("kind", "struct")?;
                }
                row.serialize_field("fields", &StructFields(&schema.fields))?;
                if !*LEGACY_INSERTION_ORDER {
                    row.serialize_field("kind", "struct")?;
                }
            }
        }
        row.end()
    }
}

struct StructFields<'a>(&'a [StructFieldDeclaration]);
impl Serialize for StructFields<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut fields = serializer.serialize_seq(Some(self.0.len()))?;
        for field in self.0 {
            #[derive(Serialize)]
            struct Field<'a> {
                name: &'a str,
                required: bool,
                value_type: DeclaredValueType<'a>,
            }
            fields.serialize_element(&Field {
                name: &field.name,
                required: field.required,
                value_type: DeclaredValueType(&field.value_type),
            })?;
        }
        fields.end()
    }
}

#[cfg(test)]
mod tests;
