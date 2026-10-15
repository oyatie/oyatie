use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelWireLimits {
    pub workspace: usize,
    pub draft_page: usize,
    pub draft: usize,
    pub receipt: usize,
    pub create: usize,
    pub save: usize,
    pub prepare: usize,
    pub validation: usize,
    pub refusal: usize,
    pub lifecycle: usize,
    pub published: usize,
}

fn encoded_length<T: serde::Serialize>(value: &T) -> Result<usize, ModelError> {
    serde_json::to_vec(value)
        .map(|bytes| bytes.len())
        .map_err(|_| ModelError::MalformedDocument)
}

fn document_bound<T: serde::Serialize>(value: &T, empty: usize) -> Result<usize, ModelError> {
    encoded_length(value)?
        .checked_sub(empty)
        .and_then(|overhead| MAX_MODEL_BYTES.checked_add(overhead))
        .ok_or(ModelError::MalformedDocument)
}

pub fn model_wire_limits() -> Result<ModelWireLimits, ModelError> {
    // NUL is the maximal six-byte JSON expansion for each admitted source byte.
    let context = "\0".repeat(MAX_MODEL_CONTEXT_BYTES);
    let number = ModelNumber::new(MAX_MODEL_VERSION)?;
    let document = ModelDocument {
        entities: vec![],
        links: vec![],
    };
    let empty = encoded_length(&document)?;
    let receipt = ModelReceiptWire {
        generation: number,
        prior_generation: number,
        draft_id: number,
        draft_version: number,
        owner: context.clone(),
    };
    let lifecycle = ModelLifecycle {
        desired_generation: number,
        observed_generation: Some(number),
        phase: ModelLifecyclePhase::CommitUnknown,
        ready: false,
        scope: ModelObservationScope::RespondingInstance,
    };
    let draft = ModelDraftWire {
        id: number,
        tenant_id: context.clone(),
        owner: context,
        base_generation: number,
        version: number,
        document: document.clone(),
        publication: Some(receipt.clone()),
    };
    let summary = ModelDraftSummary {
        id: number,
        version: number,
        base_generation: number,
        published_generation: Some(number),
    };
    let diagnostics = vec![
        ModelDiagnostic {
            code: "\0".repeat(MAX_MODEL_DIAGNOSTIC_CODE_BYTES),
            path: "\0".repeat(MAX_MODEL_DIAGNOSTIC_PATH_BYTES),
            message: "\0".repeat(MAX_MODEL_DIAGNOSTIC_MESSAGE_BYTES),
        };
        MAX_MODEL_DIAGNOSTICS
    ];
    let refusal = [
        ModelRefusalCode::Unauthenticated,
        ModelRefusalCode::Forbidden,
        ModelRefusalCode::DraftNotFound,
        ModelRefusalCode::DraftConflict,
        ModelRefusalCode::HeadConflict,
        ModelRefusalCode::DraftClosed,
        ModelRefusalCode::MalformedRequest,
        ModelRefusalCode::BodyTooLarge,
        ModelRefusalCode::UnsupportedContentType,
        ModelRefusalCode::ModelUnavailable,
    ]
    .into_iter()
    .map(|error| encoded_length(&ModelRefusal { error }))
    .collect::<Result<Vec<_>, _>>()?
    .into_iter()
    .max()
    .ok_or(ModelError::MalformedDocument)?;
    Ok(ModelWireLimits {
        workspace: document_bound(
            &ModelWorkspace {
                generation: number,
                document: document.clone(),
                capabilities: ModelCapabilities {
                    author: false,
                    publish: false,
                },
            },
            empty,
        )?,
        draft_page: encoded_length(&ModelDraftPage {
            drafts: vec![summary; MAX_MODEL_ITEMS],
            next: Some(number),
        })?,
        draft: document_bound(&draft, empty)?,
        receipt: encoded_length(&receipt)?,
        create: document_bound(
            &CreateModelDraft {
                expected_generation: number,
                document: document.clone(),
            },
            empty,
        )?,
        save: document_bound(
            &SaveModelDraft {
                expected_version: number,
                expected_base: number,
                document,
            },
            empty,
        )?,
        prepare: encoded_length(&PrepareModelDraft {
            expected_version: number,
            expected_base: number,
        })?,
        validation: encoded_length(&ModelValidation {
            draft_id: number,
            draft_version: number,
            base_generation: number,
            valid: false,
            diagnostics,
        })?,
        refusal,
        lifecycle: encoded_length(&lifecycle)?,
        published: encoded_length(&ModelPublished { receipt, lifecycle })?,
    })
}
