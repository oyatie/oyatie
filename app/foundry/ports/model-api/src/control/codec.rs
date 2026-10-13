use super::*;

fn context_text(value: &str) -> Result<(), ModelError> {
    if value.trim().is_empty() || value.len() > MAX_MODEL_CONTEXT_BYTES {
        return Err(ModelError::InvalidContext);
    }
    Ok(())
}

macro_rules! codec {
    ($ty:ident, $limit:ident) => {
        impl $ty {
            pub fn decode(bytes: &[u8]) -> Result<Self, ModelError> {
                if bytes.len() > model_wire_limits()?.$limit {
                    return Err(ModelError::Limit {
                        path: stringify!($ty).into(),
                    });
                }
                let row: Self =
                    serde_json::from_slice(bytes).map_err(|_| ModelError::MalformedDocument)?;
                row.check_bounds()?;
                Ok(row)
            }
            pub fn encode(&self) -> Result<Vec<u8>, ModelError> {
                self.check_bounds()?;
                let bytes = serde_json::to_vec(self).map_err(|_| ModelError::MalformedDocument)?;
                if bytes.len() > model_wire_limits()?.$limit {
                    return Err(ModelError::Limit {
                        path: stringify!($ty).into(),
                    });
                }
                Ok(bytes)
            }
        }
    };
}

codec!(ModelWorkspace, workspace);
codec!(ModelDraftPage, draft_page);
codec!(ModelDraftWire, draft);
codec!(ModelReceiptWire, receipt);
codec!(CreateModelDraft, create);
codec!(SaveModelDraft, save);
codec!(PrepareModelDraft, prepare);
codec!(ModelValidation, validation);
codec!(ModelRefusal, refusal);

impl ModelWorkspace {
    pub fn check_bounds(&self) -> Result<(), ModelError> {
        self.document.check_bounds()
    }
}
impl CreateModelDraft {
    pub fn check_bounds(&self) -> Result<(), ModelError> {
        self.document.check_bounds()
    }
}
impl SaveModelDraft {
    pub fn check_bounds(&self) -> Result<(), ModelError> {
        self.document.check_bounds()
    }
}
impl PrepareModelDraft {
    pub fn check_bounds(&self) -> Result<(), ModelError> {
        Ok(())
    }
}
impl ModelRefusal {
    pub fn check_bounds(&self) -> Result<(), ModelError> {
        Ok(())
    }
}
impl ModelReceiptWire {
    pub fn check_bounds(&self) -> Result<(), ModelError> {
        context_text(&self.owner)?;
        if next_model_version(self.prior_generation.get())? != self.generation.get() {
            return Err(ModelError::MalformedDocument);
        }
        Ok(())
    }
}
impl ModelDraftWire {
    pub fn check_bounds(&self) -> Result<(), ModelError> {
        self.document.check_bounds()?;
        context_text(&self.owner)?;
        if !self.tenant_id.starts_with("ten_")
            || self.tenant_id.len() <= 4
            || self.tenant_id.len() > MAX_MODEL_CONTEXT_BYTES
        {
            return Err(ModelError::InvalidContext);
        }
        if let Some(receipt) = &self.publication {
            receipt.check_bounds()?;
            if receipt.owner != self.owner
                || receipt.draft_id != self.id
                || receipt.draft_version != self.version
                || receipt.prior_generation != self.base_generation
            {
                return Err(ModelError::MalformedDocument);
            }
        }
        Ok(())
    }
}
impl ModelDraftPage {
    pub fn check_bounds(&self) -> Result<(), ModelError> {
        if self.drafts.len() > MAX_MODEL_ITEMS {
            return Err(ModelError::Limit {
                path: "drafts".into(),
            });
        }
        if self.drafts.windows(2).any(|pair| pair[0].id >= pair[1].id)
            || self.drafts.iter().any(|draft| {
                draft
                    .published_generation
                    .is_some_and(|generation| generation <= draft.base_generation)
            })
            || self
                .next
                .is_some_and(|next| self.drafts.last().is_none_or(|draft| next != draft.id))
        {
            return Err(ModelError::MalformedDocument);
        }
        Ok(())
    }
}
impl ModelValidation {
    pub fn check_bounds(&self) -> Result<(), ModelError> {
        if self.diagnostics.len() > MAX_MODEL_DIAGNOSTICS {
            return Err(ModelError::Limit {
                path: "diagnostics".into(),
            });
        }
        if self.valid != self.diagnostics.is_empty() {
            return Err(ModelError::MalformedDocument);
        }
        for diagnostic in &self.diagnostics {
            if diagnostic.code.is_empty()
                || diagnostic.code.len() > MAX_MODEL_DIAGNOSTIC_CODE_BYTES
                || diagnostic.path.len() > MAX_MODEL_DIAGNOSTIC_PATH_BYTES
                || diagnostic.message.is_empty()
                || diagnostic.message.len() > MAX_MODEL_DIAGNOSTIC_MESSAGE_BYTES
            {
                return Err(ModelError::Limit {
                    path: "diagnostics".into(),
                });
            }
        }
        Ok(())
    }
}
