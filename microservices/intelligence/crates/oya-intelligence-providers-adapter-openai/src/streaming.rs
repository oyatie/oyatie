#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OpenAiStreamingMode {
    Disabled,
    ServerSentEvents { include_obfuscation: bool },
}

impl OpenAiStreamingMode {
    pub fn is_enabled(self) -> bool {
        matches!(self, Self::ServerSentEvents { .. })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum OpenAiStreamEventKind {
    Error,
    ResponseCompleted,
    ResponseCreated,
    ResponseOutputTextDelta,
    ResponseOutputTextDone,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenAiStreamChunkMetadata {
    pub sequence: u64,                     // data_class: INTERNAL_ONLY
    pub event_kind: OpenAiStreamEventKind, // data_class: PUBLIC
    pub chunk_ref: String,                 // data_class: INTERNAL_ONLY
    pub response_ref: String,              // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum OpenAiStreamChunkValidationFailure {
    EmptyChunkRef,
    EmptyResponseRef,
}

pub fn validate_stream_chunk(
    chunk: &OpenAiStreamChunkMetadata,
) -> Result<(), Vec<OpenAiStreamChunkValidationFailure>> {
    let mut failures = Vec::new();
    if chunk.chunk_ref.trim().is_empty() {
        failures.push(OpenAiStreamChunkValidationFailure::EmptyChunkRef);
    }
    if chunk.response_ref.trim().is_empty() {
        failures.push(OpenAiStreamChunkValidationFailure::EmptyResponseRef);
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures)
    }
}
