//! Composition root for the ops workspace-shell cell.
//!
//! Layer 6 of the hyper foundation per ADR-0090. Wires the REST handlers
//! (`console-workspace-shell-rest`) through `oya-http-router-kernel` +
//! `oya-http-middleware-kernel` + `oya-http-runtime-hyper-adapter` into a
//! hyper service ready for `tokio::main` boot.
//!
//! Type migration note (per ADR-0092): kernel types renamed from
//! `HyperRequest`/`HyperResponse` to `HttpRequest`/`HttpResponse`. Body
//! type is now `Vec<u8>` end-to-end at this layer; the hyper adapter
//! converts to/from `bytes::Bytes` at its boundary so this runtime no
//! longer needs to depend on `bytes`.

// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` to assert
// invariants under the `cfg(test)` exemption (production code is Tier 1).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::sync::{Arc, RwLock};

/// Ingress authentication middleware for the workspace-shell cell (AUTH-005 increment-1).
///
/// The composition root previously seeded an EMPTY middleware chain, so every
/// internal-tier ops route (`GET /workspace`, `GET /workspace/api/v1/surfaces`)
/// served to any unauthenticated caller even though the contract
/// (`contracts/ops-workspace-shell-v1.openapi.yaml`) declares `401`/`403` for
/// both. This module restores a DEFAULT-DENY authn gate.
///
/// Trust boundary (OWASP LLM01 / forgeable-authz): a caller cannot fabricate a
/// verified identity from `x-*` headers — only a verifier that proved an
/// unforgeable credential can mint a [`VerifiedPrincipal`]. The gate keys on the
/// router-set `matched_template`, never the raw request path, so a public-path
/// string cannot spoof the public bypass.
///
/// Scope note: this is authn only. PDP/Cedar RBAC so `/surfaces` requires an
/// *admin* (not merely any verified principal) is AUTH-005 increment-2, a
/// required follow-on tracked separately.
mod authz {
    use std::collections::BTreeMap;
    use std::sync::Arc;

    use oya_http_middleware_kernel::{HttpRequest, HttpResponse, Middleware, Next};

    /// A caller whose bearer credential a [`PrincipalAuthenticator`] has VERIFIED.
    ///
    /// The `subject` field is private with only a `pub(crate)` constructor, so a
    /// handler cannot fabricate one from caller-supplied headers — only a verifier
    /// that proved an unforgeable credential mints it.
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct VerifiedPrincipal {
        subject: String, // data_class: INTERNAL_ONLY
    }

    impl VerifiedPrincipal {
        /// Mint a verified principal. Crate-private: only an authenticator in this
        /// crate can construct one (no public constructor → unforgeable).
        pub(crate) fn new(subject: impl Into<String>) -> Self {
            Self {
                subject: subject.into(),
            }
        }

        /// The verified caller's subject (trusted; derived from the credential,
        /// never a caller-supplied header).
        #[must_use]
        pub fn subject(&self) -> &str {
            &self.subject
        }
    }

    /// Ingress authentication PORT: derive a [`VerifiedPrincipal`] from the request
    /// headers by checking an UNFORGEABLE credential. `None` ⇒ no verified
    /// principal ⇒ `401` (default-deny).
    pub trait PrincipalAuthenticator: Send + Sync {
        /// Verify the caller's credential against `headers` (lowercased at boundary).
        /// `None` ⇒ `401`.
        fn verify(&self, headers: &BTreeMap<String, String>) -> Option<VerifiedPrincipal>;
    }

    /// Reference [`PrincipalAuthenticator`]: a single configured bearer token bound
    /// to one subject, compared in constant time. An empty/unset token verifies
    /// NOTHING — there is no allow-all path.
    #[derive(Clone, Debug)]
    pub struct ConfiguredBearerAuthenticator {
        token: String,   // data_class: SECRET
        subject: String, // data_class: INTERNAL_ONLY
    }

    impl ConfiguredBearerAuthenticator {
        #[must_use]
        pub fn new(token: impl Into<String>, subject: impl Into<String>) -> Self {
            Self {
                token: token.into(),
                subject: subject.into(),
            }
        }
    }

    impl PrincipalAuthenticator for ConfiguredBearerAuthenticator {
        fn verify(&self, headers: &BTreeMap<String, String>) -> Option<VerifiedPrincipal> {
            let configured = self.token.trim();
            if configured.is_empty() {
                return None;
            }
            let presented = headers
                .get("authorization")
                .and_then(|value| value.strip_prefix("Bearer "))?;
            if constant_time_eq(presented.as_bytes(), configured.as_bytes()) {
                Some(VerifiedPrincipal::new(self.subject.clone()))
            } else {
                None
            }
        }
    }

    /// Constant-time byte comparison: length-independent so neither a length nor a
    /// first-differing-byte position leaks via timing. Mirrors
    /// `intelligence/adapters/rest::constant_time_eq`.
    pub(super) fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
        let max_len = a.len().max(b.len());
        let mut diff = a.len() ^ b.len();
        for index in 0..max_len {
            let left = a.get(index).copied().unwrap_or(0);
            let right = b.get(index).copied().unwrap_or(0);
            diff |= (left ^ right) as usize;
        }
        diff == 0
    }

    /// Default-deny ingress authn middleware.
    ///
    /// A route is PROTECTED unless its router-set `matched_template` is in `public`.
    /// `matched_template = None` ⇒ fail-closed (protected).
    pub struct AuthzMiddleware {
        pub(super) authenticator: Arc<dyn PrincipalAuthenticator>,
        pub(super) public: Vec<String>,
    }

    impl AuthzMiddleware {
        #[must_use]
        pub fn new(authenticator: Arc<dyn PrincipalAuthenticator>, public: Vec<String>) -> Self {
            Self {
                authenticator,
                public,
            }
        }

        fn is_public(&self, template: Option<&str>) -> bool {
            match template {
                Some(t) => self.public.iter().any(|p| p == t),
                None => false,
            }
        }
    }

    impl Middleware<HttpRequest, HttpResponse> for AuthzMiddleware {
        fn handle(
            &self,
            request: HttpRequest,
            next: Next<'_, HttpRequest, HttpResponse>,
        ) -> HttpResponse {
            if self.is_public(request.matched_template.as_deref()) {
                return next.run(request);
            }
            match self.authenticator.verify(&request.headers) {
                // increment-2 (PDP/Cedar RBAC) will gate on the principal here.
                Some(_principal) => next.run(request),
                None => unauthorized(),
            }
        }
    }

    /// Generic `401` with `WWW-Authenticate: Bearer` and a non-leaky JSON body.
    pub(super) fn unauthorized() -> HttpResponse {
        HttpResponse::new(401)
            .with_header("www-authenticate", "Bearer")
            .with_header("content-type", "application/json")
            .with_body(b"{\"error\":\"unauthorized\"}".to_vec())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::{build_chain, build_dev_catalog, build_router};
        use console_workspace_shell_rest::SHELL_HEALTH_ROUTE;
        use oya_http_middleware_kernel::MiddlewareChain;
        use oya_http_router_kernel::HttpMethod;
        use oya_http_runtime_hyper_adapter::dispatch;

        const TOKEN: &str = "test-admin-token";

        fn protected_chain() -> MiddlewareChain<HttpRequest, HttpResponse> {
            let auth: Arc<dyn PrincipalAuthenticator> =
                Arc::new(ConfiguredBearerAuthenticator::new(TOKEN, "ops-admin"));
            build_chain(auth)
        }

        fn request(method: HttpMethod, path: &str, bearer: Option<&str>) -> HttpRequest {
            let mut headers = BTreeMap::new();
            if let Some(token) = bearer {
                headers.insert("authorization".to_string(), format!("Bearer {token}"));
            }
            HttpRequest {
                method,
                path: path.to_string(),
                headers,
                body: Vec::new(),
                path_captures: BTreeMap::new(),
                matched_template: None,
            }
        }

        #[test]
        fn unauthenticated_protected_route_is_401() {
            let router = build_router(build_dev_catalog()).unwrap();
            let resp = dispatch(
                request(HttpMethod::Get, "/workspace", None),
                &router,
                &protected_chain(),
            );
            assert_eq!(resp.status, 401);
            assert_eq!(
                resp.headers.get("www-authenticate").map(String::as_str),
                Some("Bearer")
            );
        }

        #[test]
        fn forged_bearer_is_401() {
            let router = build_router(build_dev_catalog()).unwrap();
            let resp = dispatch(
                request(
                    HttpMethod::Get,
                    "/workspace/api/v1/surfaces",
                    Some("not-the-token"),
                ),
                &router,
                &protected_chain(),
            );
            assert_eq!(resp.status, 401);
        }

        #[test]
        fn valid_bearer_reaches_handler_200() {
            let router = build_router(build_dev_catalog()).unwrap();
            let resp = dispatch(
                request(HttpMethod::Get, "/workspace/api/v1/surfaces", Some(TOKEN)),
                &router,
                &protected_chain(),
            );
            assert_eq!(resp.status, 200);
        }

        #[test]
        fn public_health_route_needs_no_auth() {
            let router = build_router(build_dev_catalog()).unwrap();
            let resp = dispatch(
                request(HttpMethod::Get, SHELL_HEALTH_ROUTE, None),
                &router,
                &protected_chain(),
            );
            assert_eq!(resp.status, 200);
        }

        #[test]
        fn empty_configured_token_denies_every_protected_route() {
            let auth: Arc<dyn PrincipalAuthenticator> =
                Arc::new(ConfiguredBearerAuthenticator::new("", "ops-admin"));
            let chain = build_chain(auth);
            let router = build_router(build_dev_catalog()).unwrap();
            let denied = dispatch(
                request(HttpMethod::Get, "/workspace", Some("anything")),
                &router,
                &chain,
            );
            assert_eq!(denied.status, 401);
            let health = dispatch(
                request(HttpMethod::Get, SHELL_HEALTH_ROUTE, None),
                &router,
                &chain,
            );
            assert_eq!(health.status, 200);
        }

        #[test]
        fn none_matched_template_fails_closed_even_for_public_path() {
            let chain = protected_chain();
            let req = request(HttpMethod::Get, SHELL_HEALTH_ROUTE, None);
            assert!(req.matched_template.is_none());
            let resp = chain.execute(req, |_| HttpResponse::new(200));
            assert_eq!(resp.status, 401);
        }

        #[test]
        fn constant_time_eq_equal() {
            assert!(constant_time_eq(b"correct-horse", b"correct-horse"));
        }

        #[test]
        fn constant_time_eq_diff_len() {
            assert!(!constant_time_eq(b"abc", b"abcd"));
        }

        #[test]
        fn constant_time_eq_one_byte_diff() {
            assert!(!constant_time_eq(b"abc", b"abd"));
        }
    }
}

pub use authz::{
    AuthzMiddleware, ConfiguredBearerAuthenticator, PrincipalAuthenticator, VerifiedPrincipal,
};

use console_workspace_shell_kernel::{
    InMemorySurfaceCatalog, SurfaceCatalogPort, SurfaceState, VisibilityTier,
};
use console_workspace_shell_rest::{
    LIST_ALL_SURFACES_ROUTE, LIST_LIVE_SURFACES_ROUTE, SHELL_HEALTH_ROUTE,
};
use console_workspace_shell_usecase::{
    ListAllSurfacesUseCase, ListLiveSurfacesUseCase, ShellHealthUseCase,
};
use oya_http_middleware_kernel::MiddlewareChain;
use oya_http_router_kernel::{HttpMethod, Router, RouterError};
use oya_http_runtime_hyper_adapter::{HttpRequest, HttpResponse, SyncHandler};

/// Shared catalog state. Wrapped in `RwLock` so handlers can clone-snapshot
/// without mutating shared state. Per-cell composition swaps this for a
/// durable port implementation once that crate lands (Layer 4-ish).
pub type SharedCatalog = Arc<RwLock<InMemorySurfaceCatalog>>;

/// Recover from a poisoned `RwLock` read by extracting the inner state.
///
/// ADR-0083 Tier 1: avoid `.expect("catalog poisoned")` — a poisoned lock means
/// a *writer* panicked while holding the write lock; readers can still safely
/// observe the catalog snapshot. Recover by taking the inner guard from the
/// `PoisonError`. This is the canonical non-panicking recovery for a
/// snapshot-reader at the HTTP boundary.
fn read_or_recover<T: Clone>(lock: &RwLock<T>) -> T {
    match lock.read() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    }
}

/// Build the Router used by the workspace-shell cell.
///
/// ADR-0083 Tier 1: returns `Result<_, RouterError>` so the three
/// `router.route(...)` calls propagate via `?` instead of `.expect(...)`. The
/// binary entry-point (`main.rs`) and tests adapt with `?` / `.unwrap()`
/// respectively.
pub fn build_router(catalog: SharedCatalog) -> Result<Router<SyncHandler>, RouterError> {
    let mut router: Router<SyncHandler> = Router::new();

    // GET /workspace — Live + principal-tier-scoped.
    let cat_live = catalog.clone();
    router.route(
        HttpMethod::Get,
        LIST_LIVE_SURFACES_ROUTE,
        Arc::new(move |_req: HttpRequest| -> HttpResponse {
            let snapshot = read_or_recover(&cat_live);
            let response =
                ListLiveSurfacesUseCase::new(snapshot).execute(VisibilityTier::InternalPublic);
            let surfaces: Vec<console_workspace_shell_adapter::WireSurface> = response
                .surfaces
                .iter()
                .map(console_workspace_shell_adapter::WireSurface::from_kernel)
                .collect();
            let count = surfaces.len();
            let wire = console_workspace_shell_adapter::WireSurfaceListResponse { surfaces, count };
            json_response(&surface_list_json(&wire))
        }),
    )?;

    // GET /workspace/api/v1/surfaces — admin-only catalog dump.
    let cat_all = catalog.clone();
    router.route(
        HttpMethod::Get,
        LIST_ALL_SURFACES_ROUTE,
        Arc::new(move |_req: HttpRequest| -> HttpResponse {
            let snapshot = read_or_recover(&cat_all);
            let response = ListAllSurfacesUseCase::new(snapshot).execute(None, None);
            let surfaces: Vec<console_workspace_shell_adapter::WireSurface> = response
                .surfaces
                .iter()
                .map(console_workspace_shell_adapter::WireSurface::from_kernel)
                .collect();
            let count = surfaces.len();
            let wire = console_workspace_shell_adapter::WireSurfaceListResponse { surfaces, count };
            json_response(&surface_list_json(&wire))
        }),
    )?;

    // GET /workspace/api/v1/health
    let cat_health = catalog.clone();
    router.route(
        HttpMethod::Get,
        SHELL_HEALTH_ROUTE,
        Arc::new(move |_req: HttpRequest| -> HttpResponse {
            let snapshot = read_or_recover(&cat_health);
            let response =
                ShellHealthUseCase::new(snapshot, env!("CARGO_PKG_VERSION"), None).execute();
            json_response(&format!(
                "{{\"status\":\"{}\",\"surface_count\":{},\"version\":\"{}\"}}",
                response.status, response.surface_count, response.version
            ))
        }),
    )?;

    Ok(router)
}

/// Build the middleware chain with a DEFAULT-DENY authn gate (AUTH-005
/// increment-1). Every route is protected except [`SHELL_HEALTH_ROUTE`]; the
/// gate verifies an unforgeable bearer via `authenticator` and short-circuits
/// unauthenticated callers with `401` before the handler runs. Cedar/tenant/
/// telemetry/deadline middlewares are pushed alongside this one as they land.
pub fn build_chain(
    authenticator: Arc<dyn PrincipalAuthenticator>,
) -> MiddlewareChain<HttpRequest, HttpResponse> {
    MiddlewareChain::new().push(Box::new(AuthzMiddleware::new(
        authenticator,
        vec![SHELL_HEALTH_ROUTE.to_string()],
    )))
}

/// Pre-seed a dev catalog with one example surface so the cell can answer
/// queries out of the box. Real composition root reads persistent state.
pub fn build_dev_catalog() -> SharedCatalog {
    let mut catalog = InMemorySurfaceCatalog::new();
    let _ = catalog.register_surface(console_workspace_shell_kernel::Surface {
        id: console_workspace_shell_kernel::SurfaceId("docs-portal".into()),
        canonical_route: "/workspace/docs".into(),
        visibility_tier: VisibilityTier::InternalPublic,
        state: SurfaceState::ReservedComingSoon,
        owning_bc_id: "ops/docs-portal".into(),
        cedar_fragments: vec!["ops-internal-public".into()],
        openapi_contract: Some("contracts/ops-docs-v1.openapi.yaml".into()),
        retired_redirects_to: None,
    });
    Arc::new(RwLock::new(catalog))
}

fn json_response(body: &str) -> HttpResponse {
    HttpResponse::new(200)
        .with_header("content-type", "application/json")
        .with_body(body.to_string().into_bytes())
}

fn surface_list_json(
    response: &console_workspace_shell_adapter::WireSurfaceListResponse,
) -> String {
    // Minimal hand-rolled JSON; no serde per 0-to-minimal-deps policy.
    let surfaces = response
        .surfaces
        .iter()
        .map(|s| {
            format!(
                "{{\"id\":\"{}\",\"canonical_route\":\"{}\",\"visibility_tier\":\"{}\",\"state\":\"{}\",\"owning_bc_id\":\"{}\"}}",
                escape_json(&s.id),
                escape_json(&s.canonical_route),
                s.visibility_tier,
                s.state,
                escape_json(&s.owning_bc_id),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"surfaces\":[{}],\"count\":{}}}",
        surfaces, response.count
    )
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_http_router_kernel::HttpMethod;
    use oya_http_runtime_hyper_adapter::dispatch;
    use std::collections::BTreeMap;

    const TEST_TOKEN: &str = "test-admin-token";

    fn mock_request(method: HttpMethod, path: &str) -> HttpRequest {
        HttpRequest {
            method,
            path: path.to_string(),
            headers: BTreeMap::new(),
            body: Vec::new(),
            path_captures: BTreeMap::new(),
            matched_template: None,
        }
    }

    /// A request carrying a valid `Authorization: Bearer <TEST_TOKEN>` header so
    /// the authn gate in [`test_chain`] lets it through to the handler.
    fn mock_request_with_bearer(method: HttpMethod, path: &str) -> HttpRequest {
        let mut req = mock_request(method, path);
        req.headers
            .insert("authorization".to_string(), format!("Bearer {TEST_TOKEN}"));
        req
    }

    /// Middleware chain seeded with a single configured-bearer authn gate that
    /// accepts [`TEST_TOKEN`]. Public routes still pass without a bearer.
    fn test_chain() -> MiddlewareChain<HttpRequest, HttpResponse> {
        let auth: Arc<dyn PrincipalAuthenticator> =
            Arc::new(ConfiguredBearerAuthenticator::new(TEST_TOKEN, "ops-admin"));
        build_chain(auth)
    }

    #[test]
    fn build_router_registers_three_routes() {
        let catalog = build_dev_catalog();
        let router = build_router(catalog).unwrap();
        assert_eq!(router.count(), 3);
    }

    #[test]
    fn list_live_surfaces_returns_200_json() {
        let catalog = build_dev_catalog();
        let router = build_router(catalog).unwrap();
        let chain = test_chain();
        let response = dispatch(
            mock_request_with_bearer(HttpMethod::Get, "/workspace"),
            &router,
            &chain,
        );
        assert_eq!(response.status, 200);
        assert_eq!(
            response.headers.get("content-type").map(String::as_str),
            Some("application/json")
        );
        // Seeded surface is ReservedComingSoon → list_live filters Live-only, so 0 results.
        let body_text = std::str::from_utf8(&response.body).unwrap();
        assert!(body_text.contains("\"count\":0"));
    }

    #[test]
    fn list_all_surfaces_includes_seeded_surface() {
        let catalog = build_dev_catalog();
        let router = build_router(catalog).unwrap();
        let chain = test_chain();
        let response = dispatch(
            mock_request_with_bearer(HttpMethod::Get, "/workspace/api/v1/surfaces"),
            &router,
            &chain,
        );
        assert_eq!(response.status, 200);
        let body_text = std::str::from_utf8(&response.body).unwrap();
        assert!(body_text.contains("\"count\":1"));
        assert!(body_text.contains("docs-portal"));
    }

    #[test]
    fn shell_health_returns_status_healthy() {
        let catalog = build_dev_catalog();
        let router = build_router(catalog).unwrap();
        let chain = test_chain();
        let response = dispatch(
            mock_request(HttpMethod::Get, "/workspace/api/v1/health"),
            &router,
            &chain,
        );
        assert_eq!(response.status, 200);
        let body_text = std::str::from_utf8(&response.body).unwrap();
        assert!(body_text.contains("\"status\":\"healthy\""));
        assert!(body_text.contains("\"surface_count\":1"));
    }

    #[test]
    fn unknown_route_returns_404() {
        let catalog = build_dev_catalog();
        let router = build_router(catalog).unwrap();
        let chain = test_chain();
        let response = dispatch(mock_request(HttpMethod::Get, "/nope"), &router, &chain);
        assert_eq!(response.status, 404);
    }

    #[test]
    fn shared_catalog_mutation_visible_to_handlers() {
        let catalog = build_dev_catalog();
        // Register a second surface in the shared catalog.
        catalog
            .write()
            .unwrap()
            .register_surface(console_workspace_shell_kernel::Surface {
                id: console_workspace_shell_kernel::SurfaceId("tenant-mgmt".into()),
                canonical_route: "/workspace/tenants".into(),
                visibility_tier: VisibilityTier::InternalPublic,
                state: SurfaceState::Live,
                owning_bc_id: "ops/tenant-mgmt".into(),
                cedar_fragments: vec![],
                openapi_contract: None,
                retired_redirects_to: None,
            })
            .unwrap();
        let router = build_router(catalog).unwrap();
        let chain = test_chain();
        let response = dispatch(
            mock_request_with_bearer(HttpMethod::Get, "/workspace/api/v1/surfaces"),
            &router,
            &chain,
        );
        let body_text = std::str::from_utf8(&response.body).unwrap();
        assert!(body_text.contains("\"count\":2"));
        assert!(body_text.contains("tenant-mgmt"));
    }
}
