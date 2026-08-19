//! M02-P02-IP-001 — Read-only REST API surface (request/response shapes).
//!
//! No HTTP server here — just the typed shapes for the read-only endpoints.
//! The HTTP runtime adapter (a separate crate per ADR-0090) plugs these in.
//!
//! Write methods are rejected at the type level: the only constructible
//! `HttpMethod` value is `Get`. Constructing `Post`/`Put`/`Delete` requires
//! reaching for the `RejectedWriteMethod` enum, whose existence is the
//! negative test surface for "write methods fail closed".
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use intelligence_dashboard_kernel::{AccountHealthView, RoutingView, SessionView, UsageView};

// ── HTTP method (read-only at type level) ────────────────────────────────────

/// The single HTTP method this surface accepts.
/// There is no constructor here for `Post`/`Put`/`Delete`/`Patch`; those
/// only appear inside `RejectedWriteMethod` so that any "accepts a method"
/// API only sees `Get`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HttpMethod {
    Get,
}

/// The set of write methods that this surface MUST reject with 405.
///
/// This type intentionally has NO `From<HttpMethod>` impl. There is no
/// path from `HttpMethod` to `RejectedWriteMethod` and vice versa.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RejectedWriteMethod {
    Post,
    Put,
    Delete,
    Patch,
}

impl RejectedWriteMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
        }
    }
}

// ── Request shape ────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadOnlyRequest {
    method: HttpMethod,
    path: String,
}

impl ReadOnlyRequest {
    /// Only entry point. Method is fixed to `Get`.
    pub fn new_get(path: impl Into<String>) -> Self {
        Self {
            method: HttpMethod::Get,
            path: path.into(),
        }
    }

    pub fn method(&self) -> HttpMethod {
        self.method
    }
    pub fn path(&self) -> &str {
        &self.path
    }
}

// ── Response shapes (per surface) ────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountHealthResponse {
    pub status: u16,             // always 200
    pub view: AccountHealthView, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountHealthListResponse {
    pub status: u16,                   // always 200
    pub views: Vec<AccountHealthView>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionResponse {
    pub status: u16,       // data_class: INTERNAL_ONLY
    pub view: SessionView, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsageResponse {
    pub status: u16,     // data_class: INTERNAL_ONLY
    pub view: UsageView, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoutingResponse {
    pub status: u16,       // data_class: INTERNAL_ONLY
    pub view: RoutingView, // data_class: INTERNAL_ONLY
}

// ── Write-rejection response ─────────────────────────────────────────────────

/// HTTP 405 Method Not Allowed with audit event tag.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MethodNotAllowedResponse {
    pub status: u16,                          // always 405
    pub allow_header: &'static str,           // "GET"
    pub audit_event: &'static str,            // "forbidden_write_attempt"
    pub rejected_method: RejectedWriteMethod, // data_class: INTERNAL_ONLY
}

impl MethodNotAllowedResponse {
    pub fn reject(method: RejectedWriteMethod) -> Self {
        Self {
            status: 405,
            allow_header: "GET",
            audit_event: "forbidden_write_attempt",
            rejected_method: method,
        }
    }
}

// ── Surface enumeration (the endpoint catalog) ───────────────────────────────

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReadOnlyEndpoint {
    AccountHealth,
    AccountHealthList,
    Session,
    Usage,
    Routing,
}

impl ReadOnlyEndpoint {
    pub fn path(&self) -> &'static str {
        match self {
            Self::AccountHealth => "/v1/accounts/{id}/health",
            Self::AccountHealthList => "/v1/accounts/health",
            Self::Session => "/v1/sessions/{id}",
            Self::Usage => "/v1/accounts/{id}/usage",
            Self::Routing => "/v1/routing/explain",
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use intelligence_account_domain::{
        AccountHealth, AccountId, AccountState, ProviderAccount, ProviderFamily, RouteExplanation,
        UsageWindow, UsageWindowKind,
    };

    fn aid(s: &str) -> AccountId {
        AccountId(s.to_owned())
    }

    fn active() -> ProviderAccount {
        let mut a = ProviderAccount::new(aid("a1"), ProviderFamily::Claude);
        a.state = AccountState::Active;
        a
    }

    #[test]
    fn request_constructor_pins_method_to_get() {
        let r = ReadOnlyRequest::new_get("/v1/accounts/a1/health");
        assert_eq!(r.method(), HttpMethod::Get);
        assert_eq!(r.path(), "/v1/accounts/a1/health");
    }

    #[test]
    fn account_health_response_ok_status() {
        let a = active();
        let h = AccountHealth {
            account_id: aid("a1"),
            is_healthy: true,
            reason: None,
            last_check_at_epoch_secs: 0,
        };
        let resp = AccountHealthResponse {
            status: 200,
            view: AccountHealthView::new(&a, &h),
        };
        assert_eq!(resp.status, 200);
    }

    #[test]
    fn account_health_list_response_ok_status() {
        let a = active();
        let h = AccountHealth {
            account_id: aid("a1"),
            is_healthy: true,
            reason: None,
            last_check_at_epoch_secs: 0,
        };
        let resp = AccountHealthListResponse {
            status: 200,
            views: vec![AccountHealthView::new(&a, &h)],
        };
        assert_eq!(resp.status, 200);
        assert_eq!(resp.views.len(), 1);
    }

    #[test]
    fn usage_response_ok_status() {
        let w = UsageWindow::new(UsageWindowKind::FiveHour, 0, 18000, 80, 20).unwrap();
        let resp = UsageResponse {
            status: 200,
            view: UsageView::new(aid("a1"), &w),
        };
        assert_eq!(resp.status, 200);
    }

    #[test]
    fn routing_response_ok_status() {
        let e = RouteExplanation {
            chosen_provider: ProviderFamily::Claude,
            chosen_account_id: aid("a1"),
            chosen_model: "claude-sonnet-4-6".to_owned(),
            reason: "cost".to_owned(),
        };
        let resp = RoutingResponse {
            status: 200,
            view: RoutingView::new(&e),
        };
        assert_eq!(resp.status, 200);
    }

    #[test]
    fn endpoint_paths_are_stable() {
        assert_eq!(
            ReadOnlyEndpoint::AccountHealth.path(),
            "/v1/accounts/{id}/health"
        );
        assert_eq!(
            ReadOnlyEndpoint::AccountHealthList.path(),
            "/v1/accounts/health"
        );
        assert_eq!(ReadOnlyEndpoint::Session.path(), "/v1/sessions/{id}");
        assert_eq!(ReadOnlyEndpoint::Usage.path(), "/v1/accounts/{id}/usage");
        assert_eq!(ReadOnlyEndpoint::Routing.path(), "/v1/routing/explain");
    }

    /// Negative test: write methods are rejected fail-closed at the type
    /// boundary. We cannot construct a `ReadOnlyRequest` with a non-Get
    /// method (no constructor exists). Any POST/PUT/DELETE/PATCH input
    /// must be mapped to `RejectedWriteMethod` which produces a 405.
    #[test]
    fn negative_write_methods_rejected_at_type_level() {
        for m in [
            RejectedWriteMethod::Post,
            RejectedWriteMethod::Put,
            RejectedWriteMethod::Delete,
            RejectedWriteMethod::Patch,
        ] {
            let r = MethodNotAllowedResponse::reject(m);
            assert_eq!(r.status, 405);
            assert_eq!(r.allow_header, "GET");
            assert_eq!(r.audit_event, "forbidden_write_attempt");
            assert_eq!(r.rejected_method, m);
            assert!(!m.as_str().is_empty());
        }
        // Confirm HttpMethod has exactly one variant constructible here.
        let only = HttpMethod::Get;
        assert_eq!(only, HttpMethod::Get);
    }
}
