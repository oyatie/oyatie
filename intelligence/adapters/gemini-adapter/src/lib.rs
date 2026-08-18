#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, HashSet};
use std::pin::Pin;
use std::sync::Arc;

use bytes::Bytes;
use intelligence_translation_kernel::{
    anthropic_messages_to_gemini_generate_content,
    gemini_generate_content_response_to_anthropic_message_response,
    gemini_generate_content_response_to_openai_chat_response,
    openai_chat_to_gemini_generate_content,
};
use tracing::debug;

pub type GeminiByteStream =
    Pin<Box<dyn futures_core::Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static>>;

const GEMINI_DEFAULT_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";

pub const HOP_BY_HOP: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailers",
    "transfer-encoding",
    "upgrade",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GeminiAdapterError {
    InvalidRequest(String), // data_class: INTERNAL_ONLY
    UpstreamError {
        status: u16,  // data_class: INTERNAL_ONLY
        body: String, // data_class: INTERNAL_ONLY
    },
    TransportError(String), // data_class: INTERNAL_ONLY
    RateLimited {
        retry_after_secs: Option<u64>, // data_class: INTERNAL_ONLY
    },
}

#[derive(Clone, Debug)]
pub struct GeminiProxyRequest {
    pub body: Vec<u8>,                           // data_class: INTERNAL_ONLY
    pub extra_headers: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug)]
pub struct GeminiProxyResponse {
    pub status: u16,                       // data_class: INTERNAL_ONLY
    pub headers: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
    pub body: Vec<u8>,                     // data_class: INTERNAL_ONLY
}

pub struct GeminiApiKeyAdapter {
    base_url: String,           // data_class: INTERNAL_ONLY
    http: Arc<reqwest::Client>, // data_class: INTERNAL_ONLY
}

impl GeminiApiKeyAdapter {
    pub fn new(http: Arc<reqwest::Client>) -> Self {
        Self {
            base_url: GEMINI_DEFAULT_BASE_URL.to_string(),
            http,
        }
    }

    pub fn with_base_url(http: Arc<reqwest::Client>, base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            http,
        }
    }

    pub async fn proxy_generate_content(
        &self,
        api_key: &str,
        model: &str,
        request: GeminiProxyRequest,
    ) -> Result<GeminiProxyResponse, GeminiAdapterError> {
        let url = self.generate_content_url(model);
        debug!(url = %url, "proxying Gemini generateContent API-key request");
        let resp = self.send(api_key, url, request, false).await?;
        let status = resp.status().as_u16();
        if status == 429 {
            return Err(GeminiAdapterError::RateLimited {
                retry_after_secs: retry_after_secs(&resp),
            });
        }
        let headers = filtered_response_headers(resp.headers());
        let body = resp
            .bytes()
            .await
            .map_err(|e| GeminiAdapterError::TransportError(e.to_string()))?
            .to_vec();
        if status >= 400 {
            return Err(GeminiAdapterError::UpstreamError {
                status,
                body: String::from_utf8_lossy(&body).to_string(),
            });
        }
        Ok(GeminiProxyResponse {
            status,
            headers,
            body,
        })
    }

    pub async fn proxy_stream_generate_content(
        &self,
        api_key: &str,
        model: &str,
        request: GeminiProxyRequest,
    ) -> Result<(u16, BTreeMap<String, String>, GeminiByteStream), GeminiAdapterError> {
        let url = self.stream_generate_content_url(model);
        debug!(url = %url, "opening Gemini streamGenerateContent SSE stream");
        let resp = self.send(api_key, url, request, true).await?;
        let status = resp.status().as_u16();
        if status == 429 {
            return Err(GeminiAdapterError::RateLimited {
                retry_after_secs: retry_after_secs(&resp),
            });
        }
        let headers = filtered_response_headers(resp.headers());
        Ok((status, headers, Box::pin(resp.bytes_stream())))
    }

    /// Translate an OpenAI-compatible chat-completions request to Gemini
    /// generateContent, proxy it, then translate the provider response back to
    /// OpenAI-compatible shape. Translation lives at the adapter boundary so
    /// REST/gateway callers do not construct provider-native bodies directly.
    pub async fn proxy_openai_chat(
        &self,
        api_key: &str,
        body: Vec<u8>,
        extra_headers: BTreeMap<String, String>,
    ) -> Result<GeminiProxyResponse, GeminiAdapterError> {
        let payload: serde_json::Value = serde_json::from_slice(&body)
            .map_err(|e| GeminiAdapterError::InvalidRequest(e.to_string()))?;
        let translated = openai_chat_to_gemini_generate_content(&payload)
            .map_err(|e| GeminiAdapterError::InvalidRequest(format!("{e:?}")))?;
        let model = translated.model.clone();
        let native_body = serde_json::to_vec(&translated)
            .map_err(|e| GeminiAdapterError::InvalidRequest(e.to_string()))?;
        let native_response = self
            .proxy_generate_content(
                api_key,
                &model,
                GeminiProxyRequest {
                    body: native_body,
                    extra_headers,
                },
            )
            .await?;
        self.translate_response(native_response, &model, GeminiResponseShape::OpenAi)
    }

    /// Translate an Anthropic Messages request to Gemini generateContent, proxy
    /// it, then translate the provider response back to Anthropic Messages
    /// shape. Translation lives at the adapter boundary.
    pub async fn proxy_anthropic_messages(
        &self,
        api_key: &str,
        body: Vec<u8>,
        extra_headers: BTreeMap<String, String>,
    ) -> Result<GeminiProxyResponse, GeminiAdapterError> {
        let payload: serde_json::Value = serde_json::from_slice(&body)
            .map_err(|e| GeminiAdapterError::InvalidRequest(e.to_string()))?;
        let translated = anthropic_messages_to_gemini_generate_content(&payload)
            .map_err(|e| GeminiAdapterError::InvalidRequest(format!("{e:?}")))?;
        let model = translated.model.clone();
        let native_body = serde_json::to_vec(&translated)
            .map_err(|e| GeminiAdapterError::InvalidRequest(e.to_string()))?;
        let native_response = self
            .proxy_generate_content(
                api_key,
                &model,
                GeminiProxyRequest {
                    body: native_body,
                    extra_headers,
                },
            )
            .await?;
        self.translate_response(native_response, &model, GeminiResponseShape::Anthropic)
    }

    fn translate_response(
        &self,
        native_response: GeminiProxyResponse,
        model: &str,
        response_shape: GeminiResponseShape,
    ) -> Result<GeminiProxyResponse, GeminiAdapterError> {
        let payload: serde_json::Value =
            serde_json::from_slice(&native_response.body).map_err(|e| {
                GeminiAdapterError::UpstreamError {
                    status: native_response.status,
                    body: format!("invalid upstream JSON: {e}"),
                }
            })?;
        let translated = match response_shape {
            GeminiResponseShape::OpenAi => {
                gemini_generate_content_response_to_openai_chat_response(&payload, model)
            }
            GeminiResponseShape::Anthropic => {
                gemini_generate_content_response_to_anthropic_message_response(&payload, model)
            }
        };
        let body =
            serde_json::to_vec(&translated).map_err(|e| GeminiAdapterError::UpstreamError {
                status: native_response.status,
                body: format!("response translation failed: {e}"),
            })?;
        Ok(GeminiProxyResponse {
            status: native_response.status,
            headers: native_response.headers,
            body,
        })
    }

    fn generate_content_url(&self, model: &str) -> String {
        format!(
            "{}/models/{}:generateContent",
            self.base_url.trim_end_matches('/'),
            sanitize_model_path_segment(model)
        )
    }

    fn stream_generate_content_url(&self, model: &str) -> String {
        format!(
            "{}/models/{}:streamGenerateContent?alt=sse",
            self.base_url.trim_end_matches('/'),
            sanitize_model_path_segment(model)
        )
    }

    async fn send(
        &self,
        api_key: &str,
        url: String,
        request: GeminiProxyRequest,
        stream: bool,
    ) -> Result<reqwest::Response, GeminiAdapterError> {
        let hop_by_hop = hop_by_hop_set();
        let connection_tokens = connection_tokens(&request.extra_headers);
        let mut req_builder = self
            .http
            .post(&url)
            .header("x-goog-api-key", api_key)
            .body(request.body);
        if stream {
            req_builder = req_builder.header("Accept", "text/event-stream");
        }

        for (k, v) in &request.extra_headers {
            let key_lower = k.to_ascii_lowercase();
            if matches!(
                key_lower.as_str(),
                "authorization"
                    | "host"
                    | "content-length"
                    | "user-agent"
                    | "x-goog-api-key"
                    | "x-google-api-key"
            ) {
                continue;
            }
            if hop_by_hop.contains(key_lower.as_str()) {
                continue;
            }
            if connection_tokens.contains(&key_lower) {
                continue;
            }
            req_builder = req_builder.header(k.as_str(), v.as_str());
        }

        req_builder
            .send()
            .await
            .map_err(|e| GeminiAdapterError::TransportError(e.to_string()))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GeminiResponseShape {
    OpenAi,
    Anthropic,
}

fn sanitize_model_path_segment(model: &str) -> String {
    model
        .trim()
        .trim_start_matches("models/")
        .split('/')
        .next_back()
        .unwrap_or(model)
        .to_string()
}

fn hop_by_hop_set() -> HashSet<&'static str> {
    HOP_BY_HOP.iter().copied().collect()
}

fn connection_tokens(headers: &BTreeMap<String, String>) -> HashSet<String> {
    let mut tokens = HashSet::new();
    for (k, v) in headers {
        if k.eq_ignore_ascii_case("connection") {
            tokens.extend(v.split(',').map(|t| t.trim().to_ascii_lowercase()));
        }
    }
    tokens
}

fn retry_after_secs(resp: &reqwest::Response) -> Option<u64> {
    resp.headers()
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
}

fn filtered_response_headers(headers: &reqwest::header::HeaderMap) -> BTreeMap<String, String> {
    let hop_by_hop = hop_by_hop_set();
    let response_connection_tokens: HashSet<String> = headers
        .get("connection")
        .and_then(|v| v.to_str().ok())
        .map(|v| {
            v.split(',')
                .map(|t| t.trim().to_ascii_lowercase())
                .collect()
        })
        .unwrap_or_default();

    let mut filtered = BTreeMap::new();
    for (k, v) in headers {
        let key_lower = k.as_str().to_ascii_lowercase();
        if hop_by_hop.contains(key_lower.as_str()) {
            continue;
        }
        if response_connection_tokens.contains(&key_lower) {
            continue;
        }
        if let Ok(val) = v.to_str() {
            filtered.insert(k.as_str().to_string(), val.to_string());
        }
    }
    filtered
}
