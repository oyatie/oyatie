use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelLifecyclePhase {
    Current,
    Pending,
    Unobserved,
    CommitUnknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelObservationScope {
    RespondingInstance,
}

model_object!(ModelLifecycle {
    desired_generation: ModelNumber,
    #[serde(deserialize_with = "required_field")]
    observed_generation: Option<ModelNumber>,
    phase: ModelLifecyclePhase,
    ready: bool,
    scope: ModelObservationScope,
});
model_object!(ModelPublished {
    receipt: ModelReceiptWire,
    lifecycle: ModelLifecycle,
});

impl ModelLifecycle {
    pub fn check_bounds(&self) -> Result<(), ModelError> {
        let valid = match self.phase {
            ModelLifecyclePhase::Current => {
                self.observed_generation == Some(self.desired_generation)
            }
            ModelLifecyclePhase::Pending => self
                .observed_generation
                .is_some_and(|value| value != self.desired_generation),
            ModelLifecyclePhase::Unobserved => self.observed_generation.is_none(),
            ModelLifecyclePhase::CommitUnknown => true,
        };
        if !valid || (self.ready && self.phase != ModelLifecyclePhase::Current) {
            return Err(ModelError::MalformedDocument);
        }
        Ok(())
    }
    pub fn encode(&self) -> Result<Vec<u8>, ModelError> {
        self.check_bounds()?;
        serde_json::to_vec(self).map_err(|_| ModelError::MalformedDocument)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, ModelError> {
        if bytes.len() > model_wire_limits()?.lifecycle {
            return Err(ModelError::MalformedDocument);
        }
        let value: Self =
            serde_json::from_slice(bytes).map_err(|_| ModelError::MalformedDocument)?;
        value.check_bounds()?;
        Ok(value)
    }
}
impl ModelPublished {
    pub fn encode(&self) -> Result<Vec<u8>, ModelError> {
        self.receipt.check_bounds()?;
        self.lifecycle.check_bounds()?;
        if self.receipt.generation > self.lifecycle.desired_generation {
            return Err(ModelError::MalformedDocument);
        }
        serde_json::to_vec(self).map_err(|_| ModelError::MalformedDocument)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, ModelError> {
        if bytes.len() > model_wire_limits()?.published {
            return Err(ModelError::MalformedDocument);
        }
        let value: Self =
            serde_json::from_slice(bytes).map_err(|_| ModelError::MalformedDocument)?;
        value.encode()?;
        Ok(value)
    }
}
