//! Tenant-extraction middleware — Layer 4 infrastructure.
//!
//! Reads the `x-tenant-id` (or configured) request header, validates it via
//! `tenancy_kernel::TenantSlug`, and either:
//!   - injects the validated slug as a path-capture-style "tenant_id" key for
//!     handlers, or
//!   - short-circuits with HTTP 400 when the route requires a tenant and the
//!     header is missing/invalid.
//!
//! Per ADR-0095 (Phase 7 of M01-P13-IP-002): grammar lives in
//! `tenancy-kernel`, not here. This middleware extracts; the kernel
//! validates. Defense in depth — anyone who bypasses the middleware (e.g.,
//! a test, a debug path) still cannot construct an invalid `TenantSlug`.

use oya_http_middleware_kernel::{HttpRequest, HttpResponse, Middleware, Next};
use tenancy_kernel::{TenantKernelError, TenantSlug};

pub const TENANT_ID_HEADER: &str = "x-tenant-id";
/// Path-captures key used to surface the validated tenant id to handlers.
pub const TENANT_ID_CAPTURE_KEY: &str = "tenant_id";

#[derive(Clone, Debug)]
pub struct TenantMiddleware {
    header_name: String,
    required: bool,
}

impl Default for TenantMiddleware {
    fn default() -> Self {
        Self {
            header_name: TENANT_ID_HEADER.into(),
            required: true,
        }
    }
}

impl TenantMiddleware {
    pub fn required() -> Self {
        Self::default()
    }

    pub fn optional() -> Self {
        Self {
            header_name: TENANT_ID_HEADER.into(),
            required: false,
        }
    }

    pub fn with_header(mut self, name: impl Into<String>) -> Self {
        self.header_name = name.into();
        self
    }

    fn extract_tenant(
        &self,
        request: &HttpRequest,
    ) -> Result<Option<TenantSlug>, TenantHeaderError> {
        match request.headers.get(&self.header_name) {
            None if self.required => Err(TenantHeaderError::Missing),
            None => Ok(None),
            Some(value) => {
                let slug =
                    TenantSlug::try_new(value.as_str()).map_err(TenantHeaderError::InvalidSlug)?;
                Ok(Some(slug))
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantHeaderError {
    Missing,
    InvalidSlug(TenantKernelError),
}

impl TenantHeaderError {
    pub fn message(&self) -> String {
        match self {
            TenantHeaderError::Missing => format!("missing {} header", TENANT_ID_HEADER),
            TenantHeaderError::InvalidSlug(TenantKernelError::TenantSlugEmpty) => {
                format!("empty {} header value", TENANT_ID_HEADER)
            }
            TenantHeaderError::InvalidSlug(TenantKernelError::TenantSlugTooLong { actual }) => {
                format!(
                    "{} header value too long: {} bytes (max 128)",
                    TENANT_ID_HEADER, actual
                )
            }
            TenantHeaderError::InvalidSlug(TenantKernelError::TenantSlugInvalidChar) => format!(
                "{} header contains non-alphanumeric / non-dash / non-underscore character",
                TENANT_ID_HEADER
            ),
            TenantHeaderError::InvalidSlug(_) => {
                format!("invalid {} header", TENANT_ID_HEADER)
            }
        }
    }
}

impl Middleware<HttpRequest, HttpResponse> for TenantMiddleware {
    fn handle(
        &self,
        mut request: HttpRequest,
        next: Next<'_, HttpRequest, HttpResponse>,
    ) -> HttpResponse {
        match self.extract_tenant(&request) {
            Err(error) => HttpResponse::new(400)
                .with_header("content-type", "application/json")
                .with_body(format!("{{\"error\":\"{}\"}}", error.message()).into_bytes()),
            Ok(Some(slug)) => {
                request
                    .path_captures
                    .insert(TENANT_ID_CAPTURE_KEY.to_string(), slug.into_inner());
                next.run(request)
            }
            Ok(None) => next.run(request),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_http_middleware_kernel::MiddlewareChain;
    use oya_http_router_kernel::HttpMethod;
    use std::collections::BTreeMap;
    use tenancy_kernel::TENANT_SLUG_MAX_LEN;

    fn req(headers: &[(&str, &str)]) -> HttpRequest {
        let mut h = BTreeMap::new();
        for (k, v) in headers {
            h.insert((*k).to_string(), (*v).to_string());
        }
        HttpRequest {
            method: HttpMethod::Get,
            path: "/x".into(),
            headers: h,
            body: Vec::new(),
            path_captures: BTreeMap::new(),
            matched_template: None,
        }
    }

    fn ok_terminal(req: HttpRequest) -> HttpResponse {
        // Echo the captured tenant_id into the response body for assertion.
        let tenant = req
            .path_captures
            .get(TENANT_ID_CAPTURE_KEY)
            .cloned()
            .unwrap_or_default();
        HttpResponse::new(200).with_body(tenant.into_bytes())
    }

    #[test]
    fn required_missing_header_returns_400() {
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(TenantMiddleware::required()));
        let response = chain.execute(req(&[]), ok_terminal);
        assert_eq!(response.status, 400);
    }

    #[test]
    fn optional_missing_header_passes_through() {
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(TenantMiddleware::optional()));
        let response = chain.execute(req(&[]), ok_terminal);
        assert_eq!(response.status, 200);
        assert!(response.body.is_empty());
    }

    #[test]
    fn valid_tenant_injected_into_path_captures() {
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(TenantMiddleware::required()));
        let response = chain.execute(req(&[(TENANT_ID_HEADER, "acme-co")]), ok_terminal);
        assert_eq!(response.status, 200);
        assert_eq!(response.body, b"acme-co".to_vec());
    }

    #[test]
    fn empty_tenant_rejected_400() {
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(TenantMiddleware::required()));
        let response = chain.execute(req(&[(TENANT_ID_HEADER, "")]), ok_terminal);
        assert_eq!(response.status, 400);
    }

    #[test]
    fn invalid_char_rejected_400() {
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(TenantMiddleware::required()));
        let response = chain.execute(req(&[(TENANT_ID_HEADER, "abc/def")]), ok_terminal);
        assert_eq!(response.status, 400);
    }

    #[test]
    fn too_long_rejected_400() {
        let oversize = "a".repeat(TENANT_SLUG_MAX_LEN + 1);
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(TenantMiddleware::required()));
        let response = chain.execute(req(&[(TENANT_ID_HEADER, &oversize)]), ok_terminal);
        assert_eq!(response.status, 400);
    }

    #[test]
    fn with_header_override() {
        let mw = TenantMiddleware::required().with_header("x-acme-tenant");
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(mw));
        let response = chain.execute(req(&[("x-acme-tenant", "tt")]), ok_terminal);
        assert_eq!(response.status, 200);
        assert_eq!(response.body, b"tt".to_vec());
    }

    #[test]
    fn tenant_header_error_messages() {
        assert!(
            TenantHeaderError::Missing
                .message()
                .to_lowercase()
                .contains("missing")
        );
        assert!(
            TenantHeaderError::InvalidSlug(TenantKernelError::TenantSlugEmpty)
                .message()
                .contains("empty")
        );
        assert!(
            TenantHeaderError::InvalidSlug(TenantKernelError::TenantSlugTooLong { actual: 200 })
                .message()
                .contains("128")
        );
        assert!(
            TenantHeaderError::InvalidSlug(TenantKernelError::TenantSlugInvalidChar)
                .message()
                .contains("character")
        );
    }

    // F3 adversarial + F5 defense-in-depth: prove the middleware delegates
    // grammar to TenantSlug, NOT a local inline check. If TenantSlug stops
    // accepting "acme-co", this test fails — single source of truth.
    #[test]
    fn middleware_grammar_matches_kernel_tenant_slug_grammar() {
        // Both should accept identical inputs.
        let valid_inputs = ["a", "acme", "acme-co", "acme_co_1", "ABC_123-XYZ"];
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(TenantMiddleware::required()));
        for v in valid_inputs {
            assert!(TenantSlug::try_new(v).is_ok(), "kernel rejected {v}");
            let response = chain.execute(req(&[(TENANT_ID_HEADER, v)]), ok_terminal);
            assert_eq!(response.status, 200, "middleware rejected {v}");
        }
        // Both should reject identical inputs.
        let invalid_inputs = ["ab cd", "abc/def", "..", ".", "acme.co"];
        for v in invalid_inputs {
            assert!(TenantSlug::try_new(v).is_err(), "kernel accepted {v}");
            let response = chain.execute(req(&[(TENANT_ID_HEADER, v)]), ok_terminal);
            assert_eq!(response.status, 400, "middleware accepted {v}");
        }
    }

    // F3 adversarial: tenant slug value reaches the handler's path-captures
    // exactly as constructed — no mutation or escape applied.
    #[test]
    fn captured_tenant_id_reaches_handler_unchanged() {
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(TenantMiddleware::required()));
        let response = chain.execute(
            req(&[(TENANT_ID_HEADER, "tenant_42_alpha-beta")]),
            ok_terminal,
        );
        assert_eq!(response.status, 200);
        assert_eq!(response.body, b"tenant_42_alpha-beta".to_vec());
    }
}
