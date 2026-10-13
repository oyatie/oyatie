use crate::{MAX_MODEL_VERSION, ModelError};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ModelNumber(u64);

impl ModelNumber {
    pub fn new(value: u64) -> Result<Self, ModelError> {
        if value == 0 || value > MAX_MODEL_VERSION {
            return Err(ModelError::MalformedDocument);
        }
        Ok(Self(value))
    }
    pub const fn get(self) -> u64 {
        self.0
    }
    pub fn parse(value: &str) -> Result<Self, ModelError> {
        if value.is_empty()
            || value.len() > 19
            || value.starts_with('0')
            || !value.bytes().all(|byte| byte.is_ascii_digit())
        {
            return Err(ModelError::MalformedDocument);
        }
        value
            .parse()
            .map_err(|_| ModelError::MalformedDocument)
            .and_then(Self::new)
    }
}
impl serde::Serialize for ModelNumber {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}
impl<'de> serde::Deserialize<'de> for ModelNumber {
    fn deserialize<D: serde::Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        let value = <String as serde::Deserialize>::deserialize(decoder)?;
        Self::parse(&value)
            .map_err(|_| serde::de::Error::custom("expected canonical model decimal string"))
    }
}
