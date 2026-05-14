//! HTTP router kernel — pure std-only path-template matcher.
//!
//! Layer 1 of the hyper foundation (user-issued 2026-05-14: "hyper for
//! framework; switch from axum to hyper everywhere; thats our backbone;
//! fits our support everything ourselves with 0 to minimal dependency").
//!
//! No HTTP framework deps. Generic over handler type `H` so the same Router
//! type can hold function pointers, boxed closures, or trait objects depending
//! on how the consuming runtime crate composes it. The runtime
//! oya-http-runtime-hyper-adapter (Layer 5) is responsible for converting
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
    pub fn match_route(
        &self,
        method: HttpMethod,
        path: &str,
    ) -> Option<(&H, BTreeMap<String, String>)> {
        for (m, template, handler) in &self.routes {
            if *m != method {
                continue;
            }
            if let Some(captures) = template.match_path(path) {
                return Some((handler, captures));
            }
        }
        None
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
        let (handler, captures) = router.match_route(HttpMethod::Get, "/workspace").unwrap();
        assert_eq!(*handler, "list_live");
        assert!(captures.is_empty());
        let (handler, _) = router
            .match_route(HttpMethod::Get, "/workspace/api/v1/surfaces")
            .unwrap();
        assert_eq!(*handler, "list_all");
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
        let (handler, _) = router.match_route(HttpMethod::Post, "/workspace").unwrap();
        assert_eq!(*handler, "post_handler");
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
        let (handler, captures) = router
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
}
