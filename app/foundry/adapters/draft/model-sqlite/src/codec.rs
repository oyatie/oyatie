use crate::encode;
use foundry_model_api::*;

pub(crate) struct RowBounds {
    pub binding: usize,
    pub draft: usize,
    pub publication: usize,
}

pub(crate) fn row_bounds() -> Result<RowBounds, ModelError> {
    // NUL encodes to six JSON bytes per source byte, the maximal string expansion.
    let context = "\0".repeat(MAX_MODEL_CONTEXT_BYTES);
    let document = ModelDocument {
        named_string_types: vec![],
        entities: Vec::new(),
        links: Vec::new(),
    };
    let document_bytes = encode(&document)?.len();
    let receipt = PublicationReceipt {
        generation: MAX_MODEL_VERSION,
        prior_generation: MAX_MODEL_VERSION,
        draft_id: MAX_MODEL_VERSION,
        draft_version: MAX_MODEL_VERSION,
        owner: context.clone(),
    };
    let binding = ModelBinding {
        tenant_id: context.clone(),
        store_id: context.clone(),
        baseline_id: context.clone(),
    };
    let draft = ModelDraft {
        id: MAX_MODEL_VERSION,
        tenant_id: context.clone(),
        owner: context,
        base_generation: MAX_MODEL_VERSION,
        version: MAX_MODEL_VERSION,
        document: document.clone(),
        publication: Some(receipt.clone()),
    };
    let publication = ModelPublication {
        generation: MAX_MODEL_VERSION,
        document,
        receipt: Some(receipt),
    };
    Ok(RowBounds {
        binding: encode(&binding)?.len(),
        draft: document_envelope_bound(&draft, document_bytes)?,
        publication: document_envelope_bound(&publication, document_bytes)?,
    })
}

fn document_envelope_bound<T: serde::Serialize>(
    row: &T,
    empty_document_bytes: usize,
) -> Result<usize, ModelError> {
    encode(row)?
        .len()
        .checked_sub(empty_document_bytes)
        .and_then(|overhead| MAX_MODEL_BYTES.checked_add(overhead))
        .ok_or(ModelError::CorruptState)
}

pub(crate) fn decode<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    max_bytes: usize,
) -> Result<T, ModelError> {
    if bytes.len() > max_bytes {
        return Err(ModelError::CorruptState);
    }
    serde_json::from_slice(bytes).map_err(|_| ModelError::CorruptState)
}
