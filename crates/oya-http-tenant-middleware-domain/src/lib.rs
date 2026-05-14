//! Tenant-extraction middleware — Layer 4.
//!
//! Reads the `x-tenant-id` (or configured) request header, validates it, and
//! either:
//!   - injects it as a path-capture-style "tenant_id" key for handlers, or
//!   - short-circuits with HTTP 400 when the route requires a tenant and the
//!     header is missing/invalid.
//!
//! Std-only logic; concrete types from Layer 5 hyper-runtime-adapter so the
//! middleware composes cleanly into the chain.

use bytes::Bytes;
use oya_http_middleware_kernel::{HyperRequest, HyperResponse, Middleware, Next};

/// Tenant id, validated. Internally a 64-char SHA-256-style string up to
/// `MAX_TENANT_ID_LEN` (configurable per cell). Stored as String so callers
/// don't need a separate dep.
pub const TENANT_ID_HEADER: &str = "x-tenant-id";
pub const MAX_TENANT_ID_LEN: usize = 128;
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

    fn extract_tenant(&self, request: &HyperRequest) -> Result<Option<String>, TenantError> {
        match request.headers.get(&self.header_name) {
            None if self.required => Err(TenantError::Missing),
            None => Ok(None),
            Some(value) => {
                if value.is_empty() {
                    return Err(TenantError::Empty);
                }
                if value.len() > MAX_TENANT_ID_LEN {
                    return Err(TenantError::TooLong {
                        actual: value.len(),
                    });
                }
                if !value
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
                {
                    return Err(TenantError::InvalidChar);
                }
                Ok(Some(value.clone()))
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantError {
    Missing,
    Empty,
    TooLong { actual: usize },
    InvalidChar,
}

impl TenantError {
    pub fn message(&self) -> &'static str {
        match self {
            TenantError::Missing => "missing x-tenant-id header",
            TenantError::Empty => "empty x-tenant-id header value",
            TenantError::TooLong { .. } => "x-tenant-id header exceeds 128 chars",
            TenantError::InvalidChar => {
                "x-tenant-id header contains non-alphanumeric / non-dash / non-underscore character"
            }
        }
    }
}

impl Middleware<HyperRequest, HyperResponse> for TenantMiddleware {
    fn handle(
        &self,
        mut request: HyperRequest,
        next: Next<'_, HyperRequest, HyperResponse>,
    ) -> HyperResponse {
        match self.extract_tenant(&request) {
            Err(error) => HyperResponse::new(400)
                .with_header("content-type", "application/json")
                .with_body(Bytes::from(format!(
                    "{{\"error\":\"{}\"}}",
                    error.message()
                ))),
            Ok(Some(tenant_id)) => {
                request
                    .path_captures
                    .insert(TENANT_ID_CAPTURE_KEY.to_string(), tenant_id);
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

    fn req(headers: &[(&str, &str)]) -> HyperRequest {
        let mut h = BTreeMap::new();
        for (k, v) in headers {
            h.insert((*k).to_string(), (*v).to_string());
        }
        HyperRequest {
            method: HttpMethod::Get,
            path: "/x".into(),
            headers: h,
            body: Bytes::new(),
            path_captures: BTreeMap::new(),
        }
    }

    fn ok_terminal(req: HyperRequest) -> HyperResponse {
        // Echo the captured tenant_id into the response body for assertion.
        let tenant = req
            .path_captures
            .get(TENANT_ID_CAPTURE_KEY)
            .cloned()
            .unwrap_or_default();
        HyperResponse::new(200).with_body(Bytes::from(tenant))
    }

    #[test]
    fn required_missing_header_returns_400() {
        let chain: MiddlewareChain<HyperRequest, HyperResponse> =
            MiddlewareChain::new().push(Box::new(TenantMiddleware::required()));
        let response = chain.execute(req(&[]), ok_terminal);
        assert_eq!(response.status, 400);
    }

    #[test]
    fn optional_missing_header_passes_through() {
        let chain: MiddlewareChain<HyperRequest, HyperResponse> =
            MiddlewareChain::new().push(Box::new(TenantMiddleware::optional()));
        let response = chain.execute(req(&[]), ok_terminal);
        assert_eq!(response.status, 200);
        assert_eq!(response.body, Bytes::from_static(b""));
    }

    #[test]
    fn valid_tenant_injected_into_path_captures() {
        let chain: MiddlewareChain<HyperRequest, HyperResponse> =
            MiddlewareChain::new().push(Box::new(TenantMiddleware::required()));
        let response = chain.execute(req(&[(TENANT_ID_HEADER, "acme-co")]), ok_terminal);
        assert_eq!(response.status, 200);
        assert_eq!(response.body, Bytes::from_static(b"acme-co"));
    }

    #[test]
    fn empty_tenant_rejected_400() {
        let chain: MiddlewareChain<HyperRequest, HyperResponse> =
            MiddlewareChain::new().push(Box::new(TenantMiddleware::required()));
        let response = chain.execute(req(&[(TENANT_ID_HEADER, "")]), ok_terminal);
        assert_eq!(response.status, 400);
    }

    #[test]
    fn invalid_char_rejected_400() {
        let chain: MiddlewareChain<HyperRequest, HyperResponse> =
            MiddlewareChain::new().push(Box::new(TenantMiddleware::required()));
        let response = chain.execute(req(&[(TENANT_ID_HEADER, "abc/def")]), ok_terminal);
        assert_eq!(response.status, 400);
    }

    #[test]
    fn too_long_rejected_400() {
        let oversize = "a".repeat(MAX_TENANT_ID_LEN + 1);
        let chain: MiddlewareChain<HyperRequest, HyperResponse> =
            MiddlewareChain::new().push(Box::new(TenantMiddleware::required()));
        let response = chain.execute(req(&[(TENANT_ID_HEADER, &oversize)]), ok_terminal);
        assert_eq!(response.status, 400);
    }

    #[test]
    fn with_header_override() {
        let mw = TenantMiddleware::required().with_header("x-acme-tenant");
        let chain: MiddlewareChain<HyperRequest, HyperResponse> =
            MiddlewareChain::new().push(Box::new(mw));
        let response = chain.execute(req(&[("x-acme-tenant", "tt")]), ok_terminal);
        assert_eq!(response.status, 200);
        assert_eq!(response.body, Bytes::from_static(b"tt"));
    }

    #[test]
    fn tenant_error_messages() {
        assert!(TenantError::Missing.message().contains("missing"));
        assert!(TenantError::Empty.message().contains("empty"));
        assert!(
            TenantError::TooLong { actual: 200 }
                .message()
                .contains("128")
        );
        assert!(TenantError::InvalidChar.message().contains("character"));
    }
}
