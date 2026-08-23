//! HTTP router kernel — pure std-only path-template matcher.
//!
//! Layer 1 of the hyper foundation (user-issued 2026-05-14: "hyper for
//! framework; switch from axum to hyper everywhere; thats our backbone;
//! fits our support everything ourselves with 0 to minimal dependency").
//!
//! No HTTP framework deps. Generic over handler type `H` so the same Router
//! type can hold function pointers, boxed closures, or trait objects depending
//! on how the consuming runtime crate composes it. The runtime
//! http-runtime-hyper-adapter (Layer 5) is responsible for converting
//! `hyper::Request<Body>` → `(HttpMethod, path_str)` and back.
//!
//! Path templates use the OpenAPI 3.x style `{name}` placeholders:
//!   `/workspace/docs/api/v1/extractors/{extractor_id}/refresh`
//!
//! Match semantics:
//!   - exact segment matches first (e.g., `bar` matches `bar` only)
//!   - placeholder segments match any non-empty segment and capture it
//!   - longer templates beat shorter templates with the same prefix (FCFS
//!     tie-break for routes registered in insertion order)
//!
//! ADR-0092 Phase 4 change: `match_route` returns the matched template as a
//! third tuple element so consumers (telemetry middleware in particular) can
//! use the static template as a metric label instead of reconstructing it
//! from the raw path. Eliminates the F-MULTI-Q1 quality bug AND the S6
//! metric-label-injection security class in one move.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

/// HTTP method enum. Covers the OpenAPI operation set. Not bound to any
/// external HTTP crate so this kernel stays std-only.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

impl HttpMethod {
    pub fn name(self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
        }
    }

    /// Parse a method-name string (case-insensitive ASCII) into a method.
    pub fn parse(input: &str) -> Option<Self> {
        let upper = input.to_ascii_uppercase();
        match upper.as_str() {
            "GET" => Some(HttpMethod::Get),
            "POST" => Some(HttpMethod::Post),
            "PUT" => Some(HttpMethod::Put),
            "PATCH" => Some(HttpMethod::Patch),
            "DELETE" => Some(HttpMethod::Delete),
            "HEAD" => Some(HttpMethod::Head),
            "OPTIONS" => Some(HttpMethod::Options),
            _ => None,
        }
    }
}

/// Parsed path template. Stored as a normalized template string + a vec of
/// segment kinds for fast matching.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteTemplate {
    raw: String,
    segments: Vec<Segment>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Segment {
    Literal(String),
    Placeholder(String), // captured name (without braces)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RouterError {
    InvalidTemplate {
        template: String,
        reason: String,
    },
    DuplicateRoute {
        method: HttpMethod,
        template: String,
    },
}

impl RouteTemplate {
    pub fn parse(input: &str) -> Result<Self, RouterError> {
        if !input.starts_with('/') {
            return Err(RouterError::InvalidTemplate {
                template: input.to_string(),
                reason: "must start with `/`".to_string(),
            });
        }
        let mut segments = Vec::new();
        for raw_seg in input.split('/').skip(1) {
            if raw_seg.is_empty() {
                // trailing or repeated slash; preserve the empty segment via a literal "".
                // For routing purposes, empty literal only matches another empty segment.
                segments.push(Segment::Literal(String::new()));
                continue;
            }
            if let Some(stripped) = raw_seg.strip_prefix('{') {
                let Some(name) = stripped.strip_suffix('}') else {
                    return Err(RouterError::InvalidTemplate {
                        template: input.to_string(),
                        reason: format!("segment `{raw_seg}` has `{{` without closing `}}`"),
                    });
                };
                if name.is_empty() {
                    return Err(RouterError::InvalidTemplate {
                        template: input.to_string(),
                        reason: "placeholder name is empty".to_string(),
                    });
                }
                segments.push(Segment::Placeholder(name.to_string()));
            } else {
                segments.push(Segment::Literal(raw_seg.to_string()));
            }
        }
        Ok(RouteTemplate {
            raw: input.to_string(),
            segments,
        })
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// Attempt to match `path` against this template. Returns the captured
    /// placeholder values on match.
    ///
    /// Per ADR-0092 Phase 9 (S5 security): dot-segments (`.`, `..`) are
    /// REJECTED as placeholder captures by default — a captured `..` would
    /// open path-traversal vectors for downstream handlers that interpret
    /// the capture as a filesystem name. Routes that legitimately need
    /// dot-segments must accept them as literal templates, not placeholders.
    pub fn match_path(&self, path: &str) -> Option<BTreeMap<String, String>> {
        if !path.starts_with('/') {
            return None;
        }
        let mut path_segments = path.split('/').skip(1);
        let mut captures = BTreeMap::new();
        for segment in &self.segments {
            let actual = path_segments.next()?;
            match segment {
                Segment::Literal(value) => {
                    if value != actual {
                        return None;
                    }
                }
                Segment::Placeholder(name) => {
                    if actual.is_empty() {
                        return None;
                    }
                    // S5 defense: reject dot-segments as captures.
                    if actual == "." || actual == ".." {
                        return None;
                    }
                    captures.insert(name.clone(), actual.to_string());
                }
            }
        }
        if path_segments.next().is_some() {
            return None;
        }
        Some(captures)
    }
}

/// Method + path-template + handler triple.
pub struct Router<H> {
    routes: Vec<(HttpMethod, RouteTemplate, H)>,
}

impl<H> Default for Router<H> {
    fn default() -> Self {
        Self { routes: Vec::new() }
    }
}

impl<H> Router<H> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn route(
        &mut self,
        method: HttpMethod,
        template: &str,
        handler: H,
    ) -> Result<(), RouterError> {
        let parsed = RouteTemplate::parse(template)?;
        if self
            .routes
            .iter()
            .any(|(m, t, _)| *m == method && t.raw == parsed.raw)
        {
            return Err(RouterError::DuplicateRoute {
                method,
                template: parsed.raw,
            });
        }
        self.routes.push((method, parsed, handler));
        Ok(())
    }

    /// Find the first registered handler matching `(method, path)`.
    ///
    /// Returns `(handler, captures, matched_template_str)`. The template
    /// string is the raw template as registered (e.g. `/users/{user_id}`),
    /// suitable as a low-cardinality metric label. Consumers MUST NOT use
    /// the captured values themselves in labels — that re-introduces the
    /// S6 metric-label-injection class.
    pub fn match_route(
        &self,
        method: HttpMethod,
        path: &str,
    ) -> Option<(&H, BTreeMap<String, String>, &str)> {
        for (m, template, handler) in &self.routes {
            if *m != method {
                continue;
            }
            if let Some(captures) = template.match_path(path) {
                return Some((handler, captures, template.as_str()));
            }
        }
        None
    }

    /// Return true when `path` matches a registered template for any method.
    ///
    /// Runtime adapters use this to distinguish a real unknown route (404)
    /// from a known route with the wrong method (405). This intentionally
    /// returns only a boolean so callers cannot accidentally use captured
    /// path values as metric labels or error details.
    pub fn path_matches_any_method(&self, path: &str) -> bool {
        self.routes
            .iter()
            .any(|(_, template, _)| template.match_path(path).is_some())
    }

    pub fn count(&self) -> usize {
        self.routes.len()
    }

    /// Iterate over (method, template_string) pairs of every registered route.
    pub fn routes(&self) -> impl Iterator<Item = (HttpMethod, &str)> {
        self.routes.iter().map(|(m, t, _)| (*m, t.raw.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_method_names_round_trip() {
        for m in [
            HttpMethod::Get,
            HttpMethod::Post,
            HttpMethod::Put,
            HttpMethod::Patch,
            HttpMethod::Delete,
            HttpMethod::Head,
            HttpMethod::Options,
        ] {
            let n = m.name();
            assert_eq!(HttpMethod::parse(n), Some(m));
            assert_eq!(
                HttpMethod::parse(&n.to_ascii_lowercase()),
                Some(m),
                "method `{n}` should parse case-insensitively"
            );
        }
    }

    #[test]
    fn http_method_parse_unknown_is_none() {
        assert!(HttpMethod::parse("BREW").is_none());
    }

    #[test]
    fn route_template_parse_literal_only() {
        let template = RouteTemplate::parse("/foo/bar").unwrap();
        assert_eq!(template.segments.len(), 2);
    }

    #[test]
    fn route_template_parse_placeholder() {
        let template =
            RouteTemplate::parse("/workspace/docs/api/v1/extractors/{extractor_id}/refresh")
                .unwrap();
        assert_eq!(template.segments.len(), 7);
    }

    #[test]
    fn route_template_must_start_with_slash() {
        let result = RouteTemplate::parse("workspace");
        assert!(matches!(result, Err(RouterError::InvalidTemplate { .. })));
    }

    #[test]
    fn route_template_unclosed_placeholder_errors() {
        let result = RouteTemplate::parse("/foo/{id");
        assert!(matches!(result, Err(RouterError::InvalidTemplate { .. })));
    }

    #[test]
    fn route_template_empty_placeholder_errors() {
        let result = RouteTemplate::parse("/foo/{}");
        assert!(matches!(result, Err(RouterError::InvalidTemplate { .. })));
    }

    #[test]
    fn match_path_literal_exact() {
        let template = RouteTemplate::parse("/foo/bar").unwrap();
        assert!(template.match_path("/foo/bar").is_some());
        assert!(template.match_path("/foo/baz").is_none());
    }

    #[test]
    fn match_path_placeholder_captures() {
        let template = RouteTemplate::parse("/users/{user_id}/posts/{post_id}").unwrap();
        let captures = template.match_path("/users/42/posts/7").unwrap();
        assert_eq!(captures.get("user_id").map(String::as_str), Some("42"));
        assert_eq!(captures.get("post_id").map(String::as_str), Some("7"));
    }

    #[test]
    fn match_path_rejects_extra_segments() {
        let template = RouteTemplate::parse("/foo").unwrap();
        assert!(template.match_path("/foo/bar").is_none());
    }

    #[test]
    fn match_path_rejects_missing_segments() {
        let template = RouteTemplate::parse("/foo/bar").unwrap();
        assert!(template.match_path("/foo").is_none());
    }

    #[test]
    fn match_path_rejects_empty_placeholder_value() {
        let template = RouteTemplate::parse("/foo/{id}").unwrap();
        assert!(template.match_path("/foo/").is_none());
    }

    #[test]
    fn router_registers_and_matches() {
        let mut router: Router<&'static str> = Router::new();
        router
            .route(HttpMethod::Get, "/workspace", "list_live")
            .unwrap();
        router
            .route(HttpMethod::Get, "/workspace/api/v1/surfaces", "list_all")
            .unwrap();
        let (handler, captures, template) =
            router.match_route(HttpMethod::Get, "/workspace").unwrap();
        assert_eq!(*handler, "list_live");
        assert!(captures.is_empty());
        assert_eq!(template, "/workspace");
        let (handler, _, template) = router
            .match_route(HttpMethod::Get, "/workspace/api/v1/surfaces")
            .unwrap();
        assert_eq!(*handler, "list_all");
        assert_eq!(template, "/workspace/api/v1/surfaces");
    }

    #[test]
    fn router_method_discriminates() {
        let mut router: Router<&'static str> = Router::new();
        router
            .route(HttpMethod::Get, "/workspace", "get_handler")
            .unwrap();
        router
            .route(HttpMethod::Post, "/workspace", "post_handler")
            .unwrap();
        let (handler, _, _) = router.match_route(HttpMethod::Post, "/workspace").unwrap();
        assert_eq!(*handler, "post_handler");
    }

    #[test]
    fn router_path_matches_any_method_distinguishes_method_mismatch_from_unknown_path() {
        let mut router: Router<&'static str> = Router::new();
        router
            .route(
                HttpMethod::Get,
                "/v1/modules/{namespace}/{name}/{system}/versions",
                "get",
            )
            .unwrap();

        assert!(router.path_matches_any_method("/v1/modules/oyatie/vpc/opentofu/versions"));
        assert!(!router.path_matches_any_method("/v1/modules/oyatie/vpc"));
        assert!(!router.path_matches_any_method("/v1/modules/oyatie/../opentofu/versions"));
    }

    #[test]
    fn router_duplicate_errors() {
        let mut router: Router<&'static str> = Router::new();
        router.route(HttpMethod::Get, "/workspace", "a").unwrap();
        let result = router.route(HttpMethod::Get, "/workspace", "b");
        assert!(matches!(result, Err(RouterError::DuplicateRoute { .. })));
    }

    #[test]
    fn router_capture_in_handler_match() {
        let mut router: Router<&'static str> = Router::new();
        router
            .route(
                HttpMethod::Post,
                "/workspace/docs/api/v1/extractors/{extractor_id}/refresh",
                "refresh",
            )
            .unwrap();
        let (handler, captures, template) = router
            .match_route(
                HttpMethod::Post,
                "/workspace/docs/api/v1/extractors/foo-bar/refresh",
            )
            .unwrap();
        assert_eq!(*handler, "refresh");
        assert_eq!(
            captures.get("extractor_id").map(String::as_str),
            Some("foo-bar")
        );
        assert_eq!(
            template,
            "/workspace/docs/api/v1/extractors/{extractor_id}/refresh"
        );
    }

    #[test]
    fn router_no_match_returns_none() {
        let router: Router<&'static str> = Router::new();
        assert!(router.match_route(HttpMethod::Get, "/anything").is_none());
    }

    #[test]
    fn router_routes_iterator() {
        let mut router: Router<&'static str> = Router::new();
        router.route(HttpMethod::Get, "/a", "a").unwrap();
        router.route(HttpMethod::Post, "/b", "b").unwrap();
        let routes: Vec<(HttpMethod, &str)> = router.routes().collect();
        assert_eq!(routes.len(), 2);
        assert!(routes.contains(&(HttpMethod::Get, "/a")));
        assert!(routes.contains(&(HttpMethod::Post, "/b")));
    }

    // F3 adversarial: matched_template is the REGISTERED string, not the raw
    // path. A captured value that happens to look like a literal segment of
    // another route MUST NOT cause the matched_template to be the wrong one.
    #[test]
    fn matched_template_is_registered_template_not_raw_path() {
        let mut router: Router<&'static str> = Router::new();
        router
            .route(
                HttpMethod::Get,
                "/users/{user_id}/posts/{post_id}",
                "uid_pid",
            )
            .unwrap();
        // Captured user_id = "5"; literal "5" never appears as a segment so
        // both numerical captures stay isolated. The returned template must be
        // the original `/users/{user_id}/posts/{post_id}`, NOT the raw path.
        let (handler, captures, template) = router
            .match_route(HttpMethod::Get, "/users/5/posts/5")
            .unwrap();
        assert_eq!(*handler, "uid_pid");
        assert_eq!(captures.get("user_id").map(String::as_str), Some("5"));
        assert_eq!(captures.get("post_id").map(String::as_str), Some("5"));
        assert_eq!(template, "/users/{user_id}/posts/{post_id}");
        // The raw path "/users/5/posts/5" MUST NOT appear as the template;
        // that would re-introduce the S6 metric-label-injection class.
        assert_ne!(template, "/users/5/posts/5");
    }

    // F3 adversarial + S5 security: dot-segments in placeholder captures
    // are REJECTED at the router. `/users/..` MUST NOT match `/users/{id}`
    // — that would open path traversal for handlers that pass user_id
    // into a filesystem lookup.
    #[test]
    fn match_path_rejects_dot_dot_in_capture() {
        let template = RouteTemplate::parse("/users/{id}").unwrap();
        assert!(template.match_path("/users/..").is_none());
    }

    #[test]
    fn match_path_rejects_single_dot_in_capture() {
        let template = RouteTemplate::parse("/users/{id}").unwrap();
        assert!(template.match_path("/users/.").is_none());
    }

    // F3 adversarial: dot-prefixed values that are NOT bare dots are fine
    // — a user id like `.bashrc` (legit unusual case) still matches.
    #[test]
    fn match_path_accepts_dot_prefix_not_bare_dot() {
        let template = RouteTemplate::parse("/users/{id}").unwrap();
        let captures = template.match_path("/users/.bashrc").unwrap();
        assert_eq!(captures.get("id").map(String::as_str), Some(".bashrc"));
    }

    #[test]
    fn match_path_accepts_literal_dot_in_middle() {
        let template = RouteTemplate::parse("/users/{id}").unwrap();
        let captures = template.match_path("/users/foo.bar").unwrap();
        assert_eq!(captures.get("id").map(String::as_str), Some("foo.bar"));
    }

    // F3 adversarial + S5: nested traversal attempts via multiple placeholders.
    #[test]
    fn match_path_rejects_dot_dot_in_first_capture() {
        let template = RouteTemplate::parse("/{tenant}/posts/{id}").unwrap();
        assert!(template.match_path("/../posts/7").is_none());
    }

    #[test]
    fn match_path_rejects_dot_dot_in_second_capture() {
        let template = RouteTemplate::parse("/{tenant}/posts/{id}").unwrap();
        assert!(template.match_path("/acme/posts/..").is_none());
    }

    // F3 adversarial: sensitive capture value (looks like an API key) does
    // NOT appear in the matched_template. Even if the value happens to
    // contain template-delimiter chars, the template remains static.
    #[test]
    fn matched_template_excludes_sensitive_capture_values() {
        let mut router: Router<&'static str> = Router::new();
        router
            .route(HttpMethod::Get, "/api/v1/keys/{key_id}", "lookup")
            .unwrap();
        let sensitive = "sk-live-abc123def456ghi789";
        let (_, captures, template) = router
            .match_route(HttpMethod::Get, &format!("/api/v1/keys/{}", sensitive))
            .unwrap();
        assert_eq!(captures.get("key_id").map(String::as_str), Some(sensitive));
        assert_eq!(template, "/api/v1/keys/{key_id}");
        assert!(
            !template.contains(sensitive),
            "matched_template MUST NOT contain the captured sensitive value"
        );
    }
}
