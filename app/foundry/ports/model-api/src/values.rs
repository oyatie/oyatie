#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelTier {
    Scalar,
    Vector,
    Timeseries,
    Geo,
    Ciphertext,
    Struct,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelScalar {
    String,
    Integer,
    Double,
    Boolean,
    Date,
    Timestamp,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelPrivacy(data_boundary_kernel::PrivacyDataClass);

impl ModelPrivacy {
    pub const fn new(class: data_boundary_kernel::PrivacyDataClass) -> Self {
        Self(class)
    }
    pub const fn internal_only() -> Self {
        Self(data_boundary_kernel::PrivacyDataClass::internal_only())
    }
    pub const fn native(self) -> data_boundary_kernel::PrivacyDataClass {
        self.0
    }
}

impl serde::Serialize for ModelPrivacy {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.0.data_class().pascal_label())
    }
}
impl<'de> serde::Deserialize<'de> for ModelPrivacy {
    fn deserialize<D: serde::Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        let label = <String as serde::Deserialize>::deserialize(decoder)?;
        data_boundary_kernel::parse_data_class_pascal_label(&label)
            .filter(|class| class.pascal_label() == label)
            .and_then(|class| data_boundary_kernel::PrivacyDataClass::new(class).ok())
            .map(Self)
            .ok_or_else(|| serde::de::Error::custom("expected a native privacy class"))
    }
}

model_object!(ModelStructField {
    name: String,
    value_type: ModelValueType,
    required: bool,
});

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "shape", rename_all = "snake_case", deny_unknown_fields)]
pub enum ModelValueType {
    Scalar { kind: ModelScalar },
    NamedString { type_id: String, revision: u32 },
    Array { element: Box<ModelValueType> },
    Struct { fields: Vec<ModelStructField> },
}
