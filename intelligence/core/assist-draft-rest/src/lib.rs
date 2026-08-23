//! Intelligence assist-draft REST boundary foundation.
//!
//! This crate defines a framework-free, source-level REST route facade for the
//! assist-draft API boundary. It maps method/path checks to stable HTTP-shaped
//! responses and delegates accepted POST bodies to `intelligence-assist-draft-api`.
//! It performs no HTTP serving, socket binding, serialization framework work,
//! prompt rendering, model/provider calls, builder mutation, network I/O,
//! durable idempotency storage, durable audit-chain emission, queue processing,
//! or runtime execution.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use intelligence_assist_draft_api::*;

pub const ASSIST_DRAFT_REST_ROUTE: &str = "/v1/intelligence/assist-drafts";
pub const ASSIST_DRAFT_REST_METHOD: AssistDraftRestMethod = AssistDraftRestMethod::Post;
pub const ASSIST_DRAFT_REST_CONTRACT_REF: &str = "contracts/openapi/intelligence-assist-draft-v1.yaml#/paths/~1v1~1intelligence~1assist-drafts/post";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssistDraftRestMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftRestRequest {
    pub method: AssistDraftRestMethod, // data_class: PUBLIC
    pub path: String,                  // data_class: PUBLIC
    pub request_id: String,            // data_class: INTERNAL_ONLY
    pub body: AssistDraftApiRequest,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftRestResponse {
    pub status_code: u16,          // data_class: PUBLIC
    pub body: AssistDraftRestBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssistDraftRestBody {
    Success(Box<AssistDraftApiSuccessResponse>),
    Error(Box<AssistDraftApiErrorResponse>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftRestError {
    pub reason: String, // data_class: INTERNAL_ONLY
}

pub struct AssistDraftRestService {
    api: IntelligenceAssistDraftApi,
}

impl AssistDraftRestService {
    pub fn new(api: IntelligenceAssistDraftApi) -> Self {
        Self { api }
    }

    pub fn handle(
        &mut self,
        request: AssistDraftRestRequest,
    ) -> Result<AssistDraftRestResponse, AssistDraftRestError> {
        if request.path != ASSIST_DRAFT_REST_ROUTE {
            return Ok(rest_error_response(
                404,
                AssistDraftApiErrorCode::UnsafeMetadata,
                "Assist-draft REST route was not found",
                request.request_id,
                "route",
            ));
        }
        if request.method != ASSIST_DRAFT_REST_METHOD {
            return Ok(rest_error_response(
                405,
                AssistDraftApiErrorCode::UnsafeMetadata,
                "Assist-draft REST method is not allowed",
                request.request_id,
                "method",
            ));
        }
        match self.api.submit(request.body) {
            Ok(success) => Ok(AssistDraftRestResponse {
                status_code: success.http_status_code(),
                body: AssistDraftRestBody::Success(Box::new(success)),
            }),
            Err(error) => Ok(AssistDraftRestResponse {
                status_code: error.status_code(),
                body: AssistDraftRestBody::Error(Box::new(
                    error.error_response(request.request_id),
                )),
            }),
        }
    }

    pub fn api_dispatch_count(&self) -> usize {
        self.api.dispatch_count()
    }
}

fn rest_error_response(
    status_code: u16,
    code: AssistDraftApiErrorCode,
    message: &str,
    request_id: String,
    field: &str,
) -> AssistDraftRestResponse {
    AssistDraftRestResponse {
        status_code,
        body: AssistDraftRestBody::Error(Box::new(AssistDraftApiErrorResponse {
            error: AssistDraftApiErrorBody {
                code: code.as_str().to_owned(),
                message: message.to_owned(),
                message_localized: None,
                request_id: if request_id.trim().is_empty()
                    || contains_unsafe_debug_material(&request_id)
                {
                    "assist-draft-rest:redacted-request-id".to_owned()
                } else {
                    request_id
                },
                details: vec![AssistDraftApiErrorDetail {
                    field: field.to_owned(),
                    issue: code.as_str().to_owned(),
                }],
                retry_after_seconds: None,
            },
        })),
    }
}

fn contains_unsafe_debug_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("raw prompt")
        || lower.contains("raw output")
        || lower.contains("write an email")
        || lower.contains("document=")
        || lower.contains("sk-")
        || lower.contains("secret=")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_constants_match_assist_draft_contract() {
        assert_eq!(ASSIST_DRAFT_REST_ROUTE, "/v1/intelligence/assist-drafts");
        assert_eq!(ASSIST_DRAFT_REST_METHOD, AssistDraftRestMethod::Post);
    }

    #[test]
    fn post_handler_maps_api_accepted_to_202_response() {
        let mut rest = AssistDraftRestService::new(valid_api());
        let response = rest.handle(valid_rest_request()).expect("rest response");
        assert_eq!(response.status_code, 202);
        assert!(matches!(response.body, AssistDraftRestBody::Success(_)));
    }

    #[test]
    fn method_or_path_mismatch_never_calls_api() {
        let mut rest = AssistDraftRestService::new(valid_api());
        let mut request = valid_rest_request();
        request.path = "/v1/intelligence/other".to_owned();
        let response = rest.handle(request).expect("not found");
        assert_eq!(response.status_code, 404);
        assert_eq!(rest.api_dispatch_count(), 0);
    }

    #[test]
    fn rest_error_body_is_structured_and_redaction_safe() {
        let mut rest = AssistDraftRestService::new(valid_api());
        let mut request = valid_rest_request();
        request.body.body.request.prompt_ref =
            "raw prompt: write an email with sk-secret".to_owned();
        let response = rest.handle(request).expect("bad request");
        assert_eq!(response.status_code, 400);
        let rendered = format!("{response:?}");
        assert!(!rendered.contains("raw prompt"));
        assert!(!rendered.contains("write an email"));
        assert!(!rendered.contains("sk-secret"));
    }

    fn valid_api() -> IntelligenceAssistDraftApi {
        IntelligenceAssistDraftApi::new(valid_adapter())
    }

    fn valid_adapter() -> IntelligenceAssistDraftAdapter {
        IntelligenceAssistDraftAdapter::try_new(
            AssistDraftExecutorAdapterConfig::new(
                "https://assist-draft-executor.internal",
                "credential-handle:assist-draft:1",
                "audit-tap:assist-draft:1",
                "draft-executor:assist-draft:workflow-studio",
            ),
            AssistDraftExecutorStatus::Accepted {
                executor_request_ref: "draft-request:assist-draft:1".to_owned(),
                draft_ref: "draft:assist-draft:1".to_owned(),
                evidence_ref: "executor-evidence:assist-draft:accepted".to_owned(),
            },
        )
        .expect("adapter")
    }

    fn valid_rest_request() -> AssistDraftRestRequest {
        AssistDraftRestRequest {
            method: AssistDraftRestMethod::Post,
            path: ASSIST_DRAFT_REST_ROUTE.to_owned(),
            request_id: "request:api:assist-draft:1".to_owned(),
            body: valid_api_request(),
        }
    }

    fn valid_api_request() -> AssistDraftApiRequest {
        AssistDraftApiRequest {
            boundary: AssistDraftApiBoundaryContext {
                request_id: "request:api:assist-draft:1".to_owned(),
                tenant_id: "tenant:alpha".to_owned(),
                idempotency_key: "idempotency:assist-draft:1".to_owned(),
                trace_context_ref: "trace:assist-draft:1".to_owned(),
            },
            principal: AssistDraftApiPrincipal {
                tenant_id: "tenant:alpha".to_owned(),
                principal_id: "principal:builder-owner".to_owned(),
            },
            authorization: AssistDraftApiAuthorization {
                tenant_id: "tenant:alpha".to_owned(),
                principal_id: "principal:builder-owner".to_owned(),
                decision_id: "decision:assist-draft:1".to_owned(),
                evidence_ref: "policy:assist-draft:allow".to_owned(),
                allowed_surfaces: vec![ASSIST_DRAFT_API_SURFACE.to_owned()],
            },
            body: valid_domain_request(),
        }
    }

    fn valid_domain_request() -> DomainAssistDraftRequest {
        DomainAssistDraftRequest {
            principal_id: "principal:builder-owner".to_owned(),
            brand_surface_ref: "brand-surface:workflow-studio:assist".to_owned(),
            locale: "en-US".to_owned(),
            prompt_context_refs: vec!["context-snippet:workflow-studio:canvas-1".to_owned()],
            policy_decision: AssistDraftPolicyDecision {
                decision_id: "decision:assist-draft:1".to_owned(),
                tenant_id: "tenant:alpha".to_owned(),
                principal_id: "principal:builder-owner".to_owned(),
                ai_assist_enabled: true,
                explicit_automation_allowed: false,
                allowed_builder_surfaces: vec![AssistDraftBuilderSurface::WorkflowStudio],
                allowed_draft_kinds: vec![AssistDraftKind::WorkflowDraft],
                allowed_audiences: vec![AssistDraftAudience::TenantBuilder],
                allowed_data_classes: vec![
                    AssistDraftDataClass::Internal,
                    AssistDraftDataClass::Public,
                ],
                allowed_actions: vec![
                    AssistDraftAction::CreateDraft,
                    AssistDraftAction::ExplainDraft,
                ],
                allowed_locales: vec!["en-US".to_owned()],
                max_prompt_context_refs: 4,
                evidence_ref: "policy:assist-draft:allow".to_owned(),
                prompt_registry_snapshot_ref: "prompt-registry:assist-draft:v1".to_owned(),
                cost_floor_disclosure_ref: "cost-floor:assist-draft:workflow-studio".to_owned(),
                builder_capability_scope_ref: "builder-scope:workflow-studio:draft".to_owned(),
            },
            request: AssistDraftRequest {
                tenant_id: "tenant:alpha".to_owned(),
                principal_id: "principal:builder-owner".to_owned(),
                context_id: "context://workflow-studio/canvas-1".to_owned(),
                builder_surface: AssistDraftBuilderSurface::WorkflowStudio,
                draft_kind: AssistDraftKind::WorkflowDraft,
                audience: AssistDraftAudience::TenantBuilder,
                invocation_mode: AssistDraftInvocationMode::UserInvoked,
                review_gate: AssistDraftReviewGate::HumanReviewRequired,
                prompt_ref: "prompt://assist-draft/req-1".to_owned(),
                target_builder_ref: "builder://workflow-studio/canvas-1".to_owned(),
                output_contract_ref: "workflow-spec://contracts/v1".to_owned(),
                consent_grant_ref: "consent:assist-draft:1".to_owned(),
                budget_evidence_ref: "budget:assist-draft:1".to_owned(),
                policy_decision_ref: "policy:assist-draft:allow".to_owned(),
                model_route_ref: "model-route:assist-draft:1".to_owned(),
                guardrail_evidence_ref: "guardrail:assist-draft:allow".to_owned(),
                request_evidence_ref: "request:assist-draft:1".to_owned(),
                trace_context_ref: "trace:assist-draft:1".to_owned(),
                data_classes: vec![AssistDraftDataClass::Internal, AssistDraftDataClass::Public],
                requested_actions: vec![
                    AssistDraftAction::CreateDraft,
                    AssistDraftAction::ExplainDraft,
                ],
                additional_evidence_refs: Vec::new(),
            },
        }
    }
}
