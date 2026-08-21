//! OpenAI provider-adapter foundation for Intelligence dispatch.
//!
//! This crate is a deterministic source-level adapter seam. It builds metadata
//! envelopes for the OpenAI Responses API shape and maps provider outcome
//! metadata into the existing Intelligence `ProviderDispatchPort`; it does not
//! resolve credentials, perform network I/O, or carry raw prompt/output bytes.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod client;
pub mod modalities;
pub mod streaming;

pub use client::{
    OpenAiAdapterConfigError, OpenAiHttpMethod, OpenAiProviderAdapter, OpenAiProviderAdapterConfig,
    OpenAiProviderRequestEnvelope, OpenAiProviderStatus,
};
pub use modalities::{
    OpenAiModality, default_openai_modalities, modalities_for_capability,
    supports_declared_modalities,
};
pub use streaming::{
    OpenAiStreamChunkMetadata, OpenAiStreamChunkValidationFailure, OpenAiStreamEventKind,
    OpenAiStreamingMode, validate_stream_chunk,
};

#[cfg(test)]
mod tests {
    use super::*;
    use intelligence_dispatch_usecase::{ProviderDispatchPort, ProviderDispatchRequest};
    use intelligence_model_routing_domain::{CredentialMode, ModelProvider, RouteSelection};

    fn request(provider: ModelProvider) -> ProviderDispatchRequest {
        ProviderDispatchRequest {
            idempotency_key: "idem-1".to_owned(),
            tenant_id: "ten_a".to_owned(),
            content_ref: "contentref://prompt/1".to_owned(),
            route_selection: RouteSelection {
                provider,
                model_id: "gpt-5.4-mini".to_owned(),
                credential_mode: CredentialMode::BringYourOwnKey,
                evidence_refs: vec!["route:openai".to_owned()],
            },
            request_evidence_ref: "req:dispatch".to_owned(),
        }
    }

    #[test]
    fn builds_responses_envelope_and_returns_metadata_only_refs() {
        let config = OpenAiProviderAdapterConfig::new(
            "https://api.openai.com",
            "openbao://ten_a/openai/byok",
            "audit://tap/openai",
        )
        .with_streaming(OpenAiStreamingMode::ServerSentEvents {
            include_obfuscation: true,
        })
        .with_safety_identifier_ref("safety://user/hash-1");
        let mut adapter = OpenAiProviderAdapter::try_new(
            config,
            OpenAiProviderStatus::Accepted {
                provider_request_id_ref: "openai://responses/resp_1".to_owned(),
                output_ref: "contentref://openai/output/1".to_owned(),
                usage_ref: "usage://openai/resp_1".to_owned(),
            },
        )
        .expect("valid adapter config");

        let response = adapter
            .dispatch(request(ModelProvider::OpenAi))
            .expect("dispatch ok");
        let envelope = adapter.last_envelope().expect("recorded envelope");

        assert_eq!(envelope.method, OpenAiHttpMethod::Post);
        assert_eq!(envelope.path, "/v1/responses");
        assert_eq!(envelope.model_id, "gpt-5.4-mini");
        assert_eq!(envelope.input_ref, "contentref://prompt/1");
        assert_eq!(envelope.idempotency_key, "idem-1");
        assert!(envelope.streaming.is_enabled());
        assert_eq!(response.output_ref, "contentref://openai/output/1");
        assert!(
            response
                .provider_evidence_ref
                .contains("openai://responses/resp_1")
        );
    }

    #[test]
    fn rejects_non_openai_route_before_provider_envelope() {
        let mut adapter = valid_adapter(OpenAiProviderStatus::Timeout {
            evidence_ref: "openai:error:timeout".to_owned(),
        });

        let failure = adapter
            .dispatch(request(ModelProvider::Anthropic))
            .expect_err("route denied");

        assert_eq!(failure.reason, "openai:route-provider-mismatch");
        assert!(adapter.last_envelope().is_none());
    }

    #[test]
    fn rejects_raw_secret_like_credential_handles() {
        let config = OpenAiProviderAdapterConfig::new(
            "https://api.openai.com",
            "sk-test-raw-secret",
            "audit://tap/openai",
        );

        let error = OpenAiProviderAdapter::try_new(
            config,
            OpenAiProviderStatus::RateLimited {
                evidence_ref: "openai:error:rate-limit".to_owned(),
            },
        )
        .expect_err("raw secret handle rejected");

        assert_eq!(
            error,
            OpenAiAdapterConfigError::RawCredentialMaterialRejected
        );
    }

    #[test]
    fn maps_rate_limit_and_server_error_distinctly() {
        let mut rate_limited = valid_adapter(OpenAiProviderStatus::RateLimited {
            evidence_ref: "openai:error:429".to_owned(),
        });
        let mut server_error = valid_adapter(OpenAiProviderStatus::ServerError {
            evidence_ref: "openai:error:500".to_owned(),
        });

        assert_eq!(
            rate_limited
                .dispatch(request(ModelProvider::OpenAi))
                .expect_err("rate limit")
                .reason,
            "openai:rate_limit"
        );
        assert_eq!(
            server_error
                .dispatch(request(ModelProvider::OpenAi))
                .expect_err("server error")
                .reason,
            "openai:server_error"
        );
    }

    #[test]
    fn envelope_and_refs_never_contain_raw_prompt_output_or_secret_bytes() {
        let mut adapter = valid_adapter(OpenAiProviderStatus::Accepted {
            provider_request_id_ref: "openai://responses/resp_1".to_owned(),
            output_ref: "contentref://openai/output/1".to_owned(),
            usage_ref: "usage://openai/resp_1".to_owned(),
        });

        let response = adapter
            .dispatch(request(ModelProvider::OpenAi))
            .expect("dispatch ok");
        let debug = format!("{:?}{:?}", adapter.last_envelope(), response);

        assert!(!debug.contains("sk-test"));
        assert!(!debug.contains("write an email to the customer"));
        assert!(!debug.contains("raw model answer"));
    }

    #[test]
    fn rejects_raw_prompt_shaped_content_refs_before_envelope() {
        let mut raw_prompt = request(ModelProvider::OpenAi);
        raw_prompt.content_ref = "write an email to the customer".to_owned();
        let mut adapter = valid_adapter(OpenAiProviderStatus::Accepted {
            provider_request_id_ref: "openai://responses/resp_1".to_owned(),
            output_ref: "contentref://openai/output/1".to_owned(),
            usage_ref: "usage://openai/resp_1".to_owned(),
        });

        let failure = adapter
            .dispatch(raw_prompt)
            .expect_err("raw prompt rejected");

        assert_eq!(failure.reason, "openai:content_ref_must_be_opaque");
        assert!(adapter.last_envelope().is_none());
    }

    #[test]
    fn rejects_blank_safety_identifier_refs_in_config() {
        let config = OpenAiProviderAdapterConfig::new(
            "https://api.openai.com",
            "secretref://ten_a/openai/byok",
            "audit://tap/openai",
        )
        .with_safety_identifier_ref(" ");

        let error = OpenAiProviderAdapter::try_new(
            config,
            OpenAiProviderStatus::Timeout {
                evidence_ref: "openai:error:timeout".to_owned(),
            },
        )
        .expect_err("blank safety identifier rejected");

        assert_eq!(error, OpenAiAdapterConfigError::EmptySafetyIdentifierRef);
    }

    #[test]
    fn validates_stream_chunk_metadata_refs() {
        let chunk = OpenAiStreamChunkMetadata {
            sequence: 1,
            event_kind: OpenAiStreamEventKind::ResponseOutputTextDelta,
            chunk_ref: "chunkref://openai/resp_1/1".to_owned(),
            response_ref: "openai://responses/resp_1".to_owned(),
        };

        assert_eq!(validate_stream_chunk(&chunk), Ok(()));
    }

    fn valid_adapter(status: OpenAiProviderStatus) -> OpenAiProviderAdapter {
        OpenAiProviderAdapter::try_new(
            OpenAiProviderAdapterConfig::new(
                "https://api.openai.com",
                "secretref://ten_a/openai/byok",
                "audit://tap/openai",
            ),
            status,
        )
        .expect("valid adapter config")
    }
}
