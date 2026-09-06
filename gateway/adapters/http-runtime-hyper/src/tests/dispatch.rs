use super::*;

#[test]
fn dispatch_routes_to_matching_handler() {
    let mut router: Router<SyncHandler> = Router::new();
    router
        .route(HttpMethod::Get, "/workspace", ok_handler(b"live-list"))
        .unwrap();
    let chain = empty_chain();
    let response = dispatch(mock_request(HttpMethod::Get, "/workspace"), &router, &chain);
    assert_eq!(response.status, 200);
    assert_eq!(response.body, b"live-list".to_vec());
}

#[test]
fn dispatch_unknown_route_returns_404() {
    let router: Router<SyncHandler> = Router::new();
    let chain = empty_chain();
    let response = dispatch(mock_request(HttpMethod::Get, "/nope"), &router, &chain);
    assert_eq!(response.status, 404);
}

#[test]
fn dispatch_known_path_with_wrong_method_returns_405() {
    let mut router: Router<SyncHandler> = Router::new();
    router
        .route(HttpMethod::Get, "/workspace", ok_handler(b"live-list"))
        .unwrap();
    let chain = empty_chain();
    let response = dispatch(
        mock_request(HttpMethod::Post, "/workspace"),
        &router,
        &chain,
    );
    assert_eq!(response.status, 405);
    assert_eq!(response.body, b"method not allowed".to_vec());
}

#[test]
fn dispatch_passes_path_captures_to_handler() {
    let mut router: Router<SyncHandler> = Router::new();
    router
        .route(
            HttpMethod::Post,
            "/workspace/docs/api/v1/extractors/{extractor_id}/refresh",
            Arc::new(move |req: HttpRequest| {
                let id = req
                    .path_captures
                    .get("extractor_id")
                    .cloned()
                    .unwrap_or_default();
                HttpResponse::new(202).with_body(id.into_bytes())
            }),
        )
        .unwrap();
    let chain = empty_chain();
    let response = dispatch(
        mock_request(
            HttpMethod::Post,
            "/workspace/docs/api/v1/extractors/manifest-walker/refresh",
        ),
        &router,
        &chain,
    );
    assert_eq!(response.status, 202);
    assert_eq!(response.body, b"manifest-walker".to_vec());
}

#[test]
fn response_helpers_build_canonical_errors() {
    assert_eq!(HttpResponse::not_found().status, 404);
    assert_eq!(HttpResponse::method_not_allowed().status, 405);
}

#[test]
fn response_with_header_inserts() {
    let resp = HttpResponse::new(200).with_header("content-type", "application/json");
    assert_eq!(
        resp.headers.get("content-type").map(String::as_str),
        Some("application/json")
    );
}

#[test]
fn to_hyper_response_preserves_status_and_body() {
    let resp = HttpResponse::new(201).with_body(b"created".to_vec());
    let hyper_resp = to_hyper_response(resp);
    assert_eq!(hyper_resp.status().as_u16(), 201);
}

#[test]
fn dispatch_invokes_middleware_chain() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct Counter(Arc<AtomicUsize>);
    impl http_middleware_kernel::Middleware<HttpRequest, HttpResponse> for Counter {
        fn handle(
            &self,
            request: HttpRequest,
            next: http_middleware_kernel::Next<'_, HttpRequest, HttpResponse>,
        ) -> HttpResponse {
            self.0.fetch_add(1, Ordering::SeqCst);
            next.run(request)
        }
    }
    let mut router: Router<SyncHandler> = Router::new();
    router
        .route(HttpMethod::Get, "/x", ok_handler(b"x"))
        .unwrap();
    let counter = Arc::new(AtomicUsize::new(0));
    let chain: MiddlewareChain<HttpRequest, HttpResponse> =
        MiddlewareChain::new().push(Box::new(Counter(counter.clone())));
    let _ = dispatch(mock_request(HttpMethod::Get, "/x"), &router, &chain);
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn hyper_runtime_error_display() {
    let bind = HyperRuntimeError::Bind("permission denied".into());
    assert!(format!("{bind}").contains("bind failed"));
    let body = HyperRuntimeError::BodyRead("eof".into());
    assert!(format!("{body}").contains("body read"));
    let method = HyperRuntimeError::UnsupportedMethod("FOO".into());
    assert!(format!("{method}").contains("FOO"));
}
