use crate::Error;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ObjectRef {
    pub tenant: String,
    pub object: String,
    pub revision: u32,
}

impl ObjectRef {
    pub fn message(&self) -> Result<String, Error> {
        self.validate()?;
        serde_json::to_string(&serde_json::json!({"oyatie_object": self}))
            .map_err(|_| Error::Invalid("invalid object reference".into()))
    }

    pub fn from_message(body: &str) -> Option<Self> {
        let value: serde_json::Value = serde_json::from_str(body).ok()?;
        let object: Self = serde_json::from_value(value.get("oyatie_object")?.clone()).ok()?;
        object.validate().ok()?;
        Some(object)
    }

    pub fn validate(&self) -> Result<(), Error> {
        if self.tenant.is_empty()
            || self.object.is_empty()
            || self.revision == 0
            || self.tenant.len() > 256
            || self.object.len() > 256
        {
            return Err(Error::Invalid("invalid object reference".into()));
        }
        Ok(())
    }
}
