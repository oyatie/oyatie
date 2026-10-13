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
    pub(crate) fn native(&self) -> ValueTypeDeclaration {
        match self {
            Self::Scalar { kind } => ValueTypeDeclaration::Scalar((*kind).into()),
            Self::Array { element } => ValueTypeDeclaration::Array {
                element: Box::new(element.native()),
            },
            Self::Struct { fields } => ValueTypeDeclaration::Struct(StructSchema {
                fields: fields
                    .iter()
                    .map(|field| StructFieldDeclaration {
                        name: field.name.clone(),
                        required: field.required,
                        value_type: field.value_type.native(),
                    })
                    .collect(),
            }),
        }
    }
}

impl From<&ValueTypeDeclaration> for ModelValueType {
    fn from(value: &ValueTypeDeclaration) -> Self {
        match value {
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
