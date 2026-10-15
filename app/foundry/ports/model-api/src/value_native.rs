use crate::*;
use data_ontology_kernel::{
    PropertyTier, ScalarType, StructFieldDeclaration, StructSchema, ValueTypeDeclaration,
};

macro_rules! native_enum {
    ($local:ident, $native:ident, $($variant:ident),+ $(,)?) => {
        impl From<$native> for $local {
            fn from(value: $native) -> Self { match value { $($native::$variant => Self::$variant),+ } }
        }
        impl From<$local> for $native {
            fn from(value: $local) -> Self { match value { $($local::$variant => Self::$variant),+ } }
        }
    }
}
native_enum!(
    ModelTier,
    PropertyTier,
    Scalar,
    Vector,
    Timeseries,
    Geo,
    Ciphertext,
    Struct
);
native_enum!(
    ModelScalar,
    ScalarType,
    String,
    Integer,
    Double,
    Boolean,
    Date,
    Timestamp
);
native_enum!(ModelPillar, OntologyPillar, Org, Person);
native_enum!(
    ModelCardinality,
    LinkCardinality,
    OneToOne,
    OneToMany,
    ManyToMany
);

use data_ontology_kernel::{LinkCardinality, OntologyPillar};

impl ModelValueType {
    pub(crate) fn native(
        &self,
        registry: &data_ontology_kernel::OntologyEngine,
        tenant: &str,
        path: &str,
    ) -> Result<ValueTypeDeclaration, ModelError> {
        self.native_at_depth(registry, tenant, path, 1)
    }

    fn native_at_depth(
        &self,
        registry: &data_ontology_kernel::OntologyEngine,
        tenant: &str,
        path: &str,
        depth: usize,
    ) -> Result<ValueTypeDeclaration, ModelError> {
        Ok(match self {
            Self::Scalar { kind } => ValueTypeDeclaration::Scalar((*kind).into()),
            Self::NamedString { type_id, revision } => {
                if depth != 1 {
                    return Err(crate::native::native_error(
                        path,
                        data_ontology_kernel::OntologyEngineError::UnsupportedNamedStringPosition,
                    ));
                }
                let reference = data_ontology_kernel::NamedStringTypeRef::new(
                    tenant,
                    data_ontology_kernel::NamedStringTypeId::new(type_id)
                        .map_err(|e| crate::native::native_error(path, e))?,
                    *revision,
                )
                .map_err(|e| crate::native::native_error(path, e))?;
                registry
                    .named_string_declaration(&reference)
                    .map_err(|e| crate::native::native_error(path, e))?
            }
            Self::Array { element } => ValueTypeDeclaration::Array {
                element: Box::new(element.native_at_depth(
                    registry,
                    tenant,
                    &format!("{path}.element"),
                    depth + 1,
                )?),
            },
            Self::Struct { fields } => ValueTypeDeclaration::Struct(StructSchema {
                fields: fields
                    .iter()
                    .map(|field| {
                        Ok(StructFieldDeclaration {
                            name: field.name.clone(),
                            required: field.required,
                            value_type: field.value_type.native_at_depth(
                                registry,
                                tenant,
                                &format!("{path}.fields.{}.value_type", field.name),
                                depth + 1,
                            )?,
                        })
                    })
                    .collect::<Result<Vec<_>, ModelError>>()?,
            }),
        })
    }
}

impl From<&ValueTypeDeclaration> for ModelValueType {
    fn from(value: &ValueTypeDeclaration) -> Self {
        match value {
            ValueTypeDeclaration::NamedString(definition) => Self::NamedString {
                type_id: definition.reference().id().as_str().into(),
                revision: definition.reference().revision(),
            },
            ValueTypeDeclaration::Scalar(kind) => Self::Scalar {
                kind: (*kind).into(),
            },
            ValueTypeDeclaration::Array { element } => Self::Array {
                element: Box::new(element.as_ref().into()),
            },
            ValueTypeDeclaration::Struct(schema) => Self::Struct {
                fields: schema
                    .fields
                    .iter()
                    .map(|field| ModelStructField {
                        name: field.name.clone(),
                        required: field.required,
                        value_type: (&field.value_type).into(),
                    })
                    .collect(),
            },
        }
    }
}
