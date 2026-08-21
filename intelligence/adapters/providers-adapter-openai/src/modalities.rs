use intelligence_model_routing_domain::ModelCapability;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum OpenAiModality {
    Embedding,
    StructuredJson,
    TextGeneration,
    ToolUse,
    Vision,
}

pub fn default_openai_modalities() -> Vec<OpenAiModality> {
    sorted_modalities(vec![OpenAiModality::TextGeneration])
}

pub fn modalities_for_capability(capability: ModelCapability) -> Vec<OpenAiModality> {
    match capability {
        ModelCapability::ChatCompletion => vec![OpenAiModality::TextGeneration],
        ModelCapability::Embedding => vec![OpenAiModality::Embedding],
        ModelCapability::JsonMode => vec![OpenAiModality::StructuredJson],
        ModelCapability::ToolUse => vec![OpenAiModality::TextGeneration, OpenAiModality::ToolUse],
        ModelCapability::Vision => vec![OpenAiModality::TextGeneration, OpenAiModality::Vision],
    }
}

pub fn supports_declared_modalities(
    declared: &[OpenAiModality],
    required: &[OpenAiModality],
) -> bool {
    required
        .iter()
        .all(|required_modality| declared.contains(required_modality))
}

pub fn sorted_modalities(mut modalities: Vec<OpenAiModality>) -> Vec<OpenAiModality> {
    modalities.sort();
    modalities.dedup();
    modalities
}
