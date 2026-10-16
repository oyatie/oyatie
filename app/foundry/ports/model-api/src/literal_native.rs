use crate::{ModelError, ModelValue};
use data_ontology_kernel::{CalendarDate, PropertyValue};

impl ModelValue {
    pub(crate) fn native(&self) -> Result<PropertyValue, ModelError> {
        Ok(match self {
            Self::String { value } => PropertyValue::String(value.clone()),
            Self::Integer { value } => PropertyValue::Integer(*value),
            Self::Double { value } => PropertyValue::Double(*value),
            Self::Boolean { value } => PropertyValue::Boolean(*value),
            Self::Date { year, month, day } => PropertyValue::Date(
                CalendarDate::new(*year, *month, *day)
                    .map_err(|_| ModelError::MalformedDocument)?,
            ),
            Self::Timestamp { epoch_millis } => PropertyValue::Timestamp {
                epoch_millis: *epoch_millis,
            },
            Self::Array { items } => {
                PropertyValue::Array(items.iter().map(Self::native).collect::<Result<_, _>>()?)
            }
            Self::Struct { fields } => PropertyValue::Struct(
                fields
                    .iter()
                    .map(|(name, value)| Ok((name.clone(), value.native()?)))
                    .collect::<Result<_, ModelError>>()?,
            ),
        })
    }
}

impl From<&PropertyValue> for ModelValue {
    fn from(value: &PropertyValue) -> Self {
        match value {
            PropertyValue::String(value) => Self::String {
                value: value.clone(),
            },
            PropertyValue::Integer(value) => Self::Integer { value: *value },
            PropertyValue::Double(value) => Self::Double { value: *value },
            PropertyValue::Boolean(value) => Self::Boolean { value: *value },
            PropertyValue::Date(value) => Self::Date {
                year: value.year(),
                month: value.month(),
                day: value.day(),
            },
            PropertyValue::Timestamp { epoch_millis } => Self::Timestamp {
                epoch_millis: *epoch_millis,
            },
            PropertyValue::Array(items) => Self::Array {
                items: items.iter().map(Into::into).collect(),
            },
            PropertyValue::Struct(fields) => Self::Struct {
                fields: fields
                    .iter()
                    .map(|(name, value)| (name.clone(), value.into()))
                    .collect(),
            },
        }
    }
}
