#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtocolShape {
    AnthropicMessages,
    OpenAiChatCompletions,
    GeminiGenerateContent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackendClass {
    AnthropicSubscription,
    OpenAiCompatible,
    GeminiNative,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TranslationMode {
    PassThrough,
    AnthropicToOpenAi,
    OpenAiToAnthropic,
    AnthropicToGemini,
    OpenAiToGemini,
    GeminiToAnthropic,
    GeminiToOpenAi,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelCapability {
    OneMillionContext,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct RoutePolicy {
    pub forced_backend: Option<BackendClass>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteRequest {
    pub protocol: ProtocolShape,
    pub model: String,
    pub route_policy: RoutePolicy,
    pub tenant_default_backend: Option<BackendClass>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoutingDecision {
    pub backend: BackendClass,
    pub upstream_model: String,
    pub translation_mode: TranslationMode,
    pub provider_prefix: Option<String>,
    pub capabilities: Vec<ModelCapability>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModelRoutingError {
    EmptyModel,
    UnknownModel { model: String },
}

#[derive(Clone, Debug, Default)]
pub struct ModelRouter {
    prefix_registry: ProviderPrefixRegistry,
}

impl ModelRouter {
    pub fn route(&self, request: RouteRequest) -> Result<RoutingDecision, ModelRoutingError> {
        let requested_model = request.model.trim();
        if requested_model.is_empty() {
            return Err(ModelRoutingError::EmptyModel);
        }

        let prefix_match = self.prefix_registry.classify(requested_model);
        let (provider_prefix, model_without_prefix, prefix_backend) = match prefix_match {
            Some(prefix_match) => (
                Some(prefix_match.prefix.to_string()),
                prefix_match.model_without_prefix,
                Some(prefix_match.backend),
            ),
            None => (None, requested_model, None),
        };

        let normalized = normalize_model(model_without_prefix);
        let classifier_backend = classify_model(&normalized.upstream_model);
        let backend = prefix_backend
            .or(request.route_policy.forced_backend)
            .or(classifier_backend)
            .or(request.tenant_default_backend)
            .ok_or_else(|| ModelRoutingError::UnknownModel {
                model: requested_model.to_string(),
            })?;

        Ok(RoutingDecision {
            backend,
            upstream_model: normalized.upstream_model,
            translation_mode: translation_mode(request.protocol, backend),
            provider_prefix,
            capabilities: normalized.capabilities,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct ProviderPrefixRegistry;

impl ProviderPrefixRegistry {
    fn classify<'a>(&self, model: &'a str) -> Option<ProviderPrefixMatch<'a>> {
        let (prefix, rest) = model.split_once(':')?;
        let backend = match prefix {
            "openai" | "codex" => BackendClass::OpenAiCompatible,
            "claude" | "anthropic" => BackendClass::AnthropicSubscription,
            "gemini" | "google" => BackendClass::GeminiNative,
            _ => return None,
        };
        if rest.trim().is_empty() {
            return None;
        }
        Some(ProviderPrefixMatch {
            prefix,
            model_without_prefix: rest.trim(),
            backend,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ProviderPrefixMatch<'a> {
    prefix: &'a str,
    model_without_prefix: &'a str,
    backend: BackendClass,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NormalizedModel {
    upstream_model: String,
    capabilities: Vec<ModelCapability>,
}

fn normalize_model(model: &str) -> NormalizedModel {
    let mut raw = model.trim().to_string();
    let mut capabilities = Vec::new();
    let mut lower = raw.to_ascii_lowercase();

    if lower.ends_with("[1m]") {
        raw.truncate(raw.len() - "[1m]".len());
        lower.truncate(lower.len() - "[1m]".len());
        capabilities.push(ModelCapability::OneMillionContext);
    }

    let alias_had_one_million_suffix = matches!(lower.as_str(), "fable1m" | "opus1m" | "sonnet1m");
    if alias_had_one_million_suffix {
        lower.truncate(lower.len() - 2);
        raw.truncate(raw.len() - 2);
        capabilities.push(ModelCapability::OneMillionContext);
    }

    let upstream_model = match lower.as_str() {
        "fable" => "claude-sonnet-4-5".to_string(),
        "opus" => "claude-opus-4-5".to_string(),
        "opus47" => "claude-opus-4-7".to_string(),
        "opus46" => "claude-opus-4-6".to_string(),
        "sonnet" => "claude-sonnet-4-5".to_string(),
        "haiku" => "claude-haiku-3-5".to_string(),
        _ => raw,
    };

    capabilities.sort_by_key(|capability| match capability {
        ModelCapability::OneMillionContext => 1,
    });
    capabilities.dedup();

    NormalizedModel {
        upstream_model,
        capabilities,
    }
}

fn classify_model(model: &str) -> Option<BackendClass> {
    let lower = model.to_ascii_lowercase();
    if lower.starts_with("claude-")
        || lower == "fable"
        || lower == "opus"
        || lower == "sonnet"
        || lower == "haiku"
    {
        return Some(BackendClass::AnthropicSubscription);
    }
    if lower.starts_with("gpt")
        || lower.starts_with("o1")
        || lower.starts_with("o3")
        || lower.starts_with("o4")
        || lower.starts_with("codex-")
        || lower.starts_with("text-embedding-")
    {
        return Some(BackendClass::OpenAiCompatible);
    }
    if lower.starts_with("gemini-") {
        return Some(BackendClass::GeminiNative);
    }
    None
}

fn translation_mode(protocol: ProtocolShape, backend: BackendClass) -> TranslationMode {
    match (protocol, backend) {
        (ProtocolShape::AnthropicMessages, BackendClass::AnthropicSubscription)
        | (ProtocolShape::OpenAiChatCompletions, BackendClass::OpenAiCompatible)
        | (ProtocolShape::GeminiGenerateContent, BackendClass::GeminiNative) => {
            TranslationMode::PassThrough
        }
        (ProtocolShape::AnthropicMessages, BackendClass::OpenAiCompatible) => {
            TranslationMode::AnthropicToOpenAi
        }
        (ProtocolShape::OpenAiChatCompletions, BackendClass::AnthropicSubscription) => {
            TranslationMode::OpenAiToAnthropic
        }
        (ProtocolShape::AnthropicMessages, BackendClass::GeminiNative) => {
            TranslationMode::AnthropicToGemini
        }
        (ProtocolShape::OpenAiChatCompletions, BackendClass::GeminiNative) => {
            TranslationMode::OpenAiToGemini
        }
        (ProtocolShape::GeminiGenerateContent, BackendClass::AnthropicSubscription) => {
            TranslationMode::GeminiToAnthropic
        }
        (ProtocolShape::GeminiGenerateContent, BackendClass::OpenAiCompatible) => {
            TranslationMode::GeminiToOpenAi
        }
    }
}
