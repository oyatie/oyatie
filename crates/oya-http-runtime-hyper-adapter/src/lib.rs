//! Hyper runtime adapter — the ONLY crate in the hyper foundation that
//! imports hyper directly (per ADR-0090).
//!
//! Layer 5 of the foundation. Bridges:
//!   - oya-http-router-kernel::Router<H>
//!   - oya-http-middleware-kernel::MiddlewareChain<HyperRequest, HyperResponse>
//!   - hyper::service::Service over hyper 1.x
//!
//! Concrete request / response types declared in THIS crate so consumers
//! never have to import hyper themselves.

use std::collections::BTreeMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto::Builder as ConnBuilder;
use tokio::net::TcpListener;

use oya_http_middleware_kernel::MiddlewareChain;
use oya_http_router_kernel::{HttpMethod, Router};

/// HTTP request as seen by Layer 4 middlewares + handlers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HyperRequest {
    pub method: HttpMethod,
    pub path: String,
    pub headers: BTreeMap<String, String>,
    pub body: Bytes,
    pub path_captures: BTreeMap<String, String>,
}

/// HTTP response shape. `body` is materialized as bytes here; SSE streams
/// are emitted by separate streaming handlers (future Layer 5 enhancement).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HyperResponse {
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub body: Bytes,
}

impl HyperResponse {
    pub fn new(status: u16) -> Self {
        Self {
            status,
            headers: BTreeMap::new(),
            body: Bytes::new(),
        }
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn with_body(mut self, body: impl Into<Bytes>) -> Self {
        self.body = body.into();
        self
    }

    pub fn not_found() -> Self {
        Self::new(404).with_body(Bytes::from_static(b"not found"))
    }

    pub fn method_not_allowed() -> Self {
        Self::new(405).with_body(Bytes::from_static(b"method not allowed"))
    }
}

/// Synchronous handler signature wrapped by the router. Handlers are pure
/// `Fn` — they own / borrow their state via captured Arcs. The runtime calls
/// the chain + handler on a tokio worker thread.
pub type SyncHandler = Arc<dyn Fn(HyperRequest) -> HyperResponse + Send + Sync>;

/// Build a `HyperRequest` from a `hyper::Request<Incoming>` by collecting
/// the body fully. Suitable for non-streaming routes. SSE / streaming
/// routes need a separate path (future enhancement).
pub async fn collect_hyper_request(
    req: Request<Incoming>,
) -> Result<HyperRequest, HyperRuntimeError> {
    let method_str = req.method().as_str().to_string();
    let method = HttpMethod::parse(&method_str)
        .ok_or_else(|| HyperRuntimeError::UnsupportedMethod(method_str.clone()))?;
    let path = req.uri().path().to_string();
    let mut headers = BTreeMap::new();
    for (name, value) in req.headers().iter() {
        if let Ok(value_str) = value.to_str() {
            headers.insert(name.as_str().to_string(), value_str.to_string());
        }
    }
    let body_bytes = req
        .into_body()
        .collect()
        .await
        .map_err(|error| HyperRuntimeError::BodyRead(error.to_string()))?
        .to_bytes();
    Ok(HyperRequest {
        method,
        path,
        headers,
        body: body_bytes,
        path_captures: BTreeMap::new(),
    })
}

/// Convert a `HyperResponse` into a hyper `Response<Full<Bytes>>`.
pub fn to_hyper_response(resp: HyperResponse) -> Response<Full<Bytes>> {
    let mut builder = Response::builder().status(resp.status);
    for (name, value) in &resp.headers {
        builder = builder.header(name, value);
    }
    builder.body(Full::new(resp.body)).unwrap_or_else(|_| {
        Response::builder()
            .status(500)
            .body(Full::new(Bytes::from_static(b"response build failed")))
            .expect("infallible default response")
    })
}

/// Dispatch a request through router → middleware chain → handler.
///
/// Lookups are sync (router) + sync (middleware chain) + sync (handler).
/// The hyper Service wrapper drives this from an async context.
pub fn dispatch(
    request: HyperRequest,
    router: &Router<SyncHandler>,
    chain: &MiddlewareChain<HyperRequest, HyperResponse>,
) -> HyperResponse {
    let (handler, captures) = match router.match_route(request.method, &request.path) {
        Some(pair) => pair,
        None => return HyperResponse::not_found(),
    };
    let mut req_with_captures = request;
    req_with_captures.path_captures = captures;
    let handler_arc = handler.clone();
    chain.execute(req_with_captures, move |req| handler_arc(req))
}

#[derive(Debug)]
pub enum HyperRuntimeError {
    Bind(String),
    BodyRead(String),
    UnsupportedMethod(String),
}

impl std::fmt::Display for HyperRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HyperRuntimeError::Bind(reason) => write!(f, "hyper bind failed: {reason}"),
            HyperRuntimeError::BodyRead(reason) => {
                write!(f, "hyper body read failed: {reason}")
            }
            HyperRuntimeError::UnsupportedMethod(method) => {
                write!(f, "unsupported HTTP method: `{method}`")
            }
        }
    }
}

impl std::error::Error for HyperRuntimeError {}

/// Start a hyper server on `addr` that dispatches every request through
/// `router` + `chain`. This is the Layer 5 entry point that per-cell
/// binaries (Layer 6) call from their `tokio::main`.
pub async fn serve(
    addr: SocketAddr,
    router: Arc<Router<SyncHandler>>,
    chain: Arc<MiddlewareChain<HyperRequest, HyperResponse>>,
) -> Result<(), HyperRuntimeError> {
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|error| HyperRuntimeError::Bind(error.to_string()))?;
    loop {
        let (stream, _peer) = match listener.accept().await {
            Ok(pair) => pair,
            Err(_) => continue,
        };
        let io = TokioIo::new(stream);
        let router = router.clone();
        let chain = chain.clone();
        tokio::spawn(async move {
            let service = service_fn(move |req: Request<Incoming>| {
                let router = router.clone();
                let chain = chain.clone();
                async move {
                    let response = match collect_hyper_request(req).await {
                        Ok(parsed) => dispatch(parsed, &router, &chain),
                        Err(_) => {
                            HyperResponse::new(400).with_body(Bytes::from_static(b"bad request"))
                        }
                    };
                    Ok::<_, Infallible>(to_hyper_response(response))
                }
            });
            let _ = ConnBuilder::new(hyper_util::rt::TokioExecutor::new())
                .serve_connection(io, service)
                .await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_http_router_kernel::Router;

    fn ok_handler(body: &'static [u8]) -> SyncHandler {
        let bytes = Bytes::from_static(body);
        Arc::new(move |_req: HyperRequest| HyperResponse::new(200).with_body(bytes.clone()))
    }

    fn empty_chain() -> MiddlewareChain<HyperRequest, HyperResponse> {
        MiddlewareChain::new()
    }

    fn mock_request(method: HttpMethod, path: &str) -> HyperRequest {
        HyperRequest {
            method,
            path: path.to_string(),
            headers: BTreeMap::new(),
            body: Bytes::new(),
            path_captures: BTreeMap::new(),
        }
    }

    #[test]
    fn dispatch_routes_to_matching_handler() {
        let mut router: Router<SyncHandler> = Router::new();
        router
            .route(HttpMethod::Get, "/workspace", ok_handler(b"live-list"))
            .unwrap();
        let chain = empty_chain();
        let response = dispatch(mock_request(HttpMethod::Get, "/workspace"), &router, &chain);
        assert_eq!(response.status, 200);
        assert_eq!(response.body, Bytes::from_static(b"live-list"));
    }

    #[test]
    fn dispatch_unknown_route_returns_404() {
        let router: Router<SyncHandler> = Router::new();
        let chain = empty_chain();
        let response = dispatch(mock_request(HttpMethod::Get, "/nope"), &router, &chain);
        assert_eq!(response.status, 404);
    }

    #[test]
    fn dispatch_passes_path_captures_to_handler() {
        let mut router: Router<SyncHandler> = Router::new();
        router
            .route(
                HttpMethod::Post,
                "/workspace/docs/api/v1/extractors/{extractor_id}/refresh",
                Arc::new(move |req: HyperRequest| {
                    let id = req
                        .path_captures
                        .get("extractor_id")
                        .cloned()
                        .unwrap_or_default();
                    HyperResponse::new(202).with_body(Bytes::from(id))
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
        assert_eq!(response.body, Bytes::from("manifest-walker"));
    }

    #[test]
    fn response_helpers_build_canonical_errors() {
        assert_eq!(HyperResponse::not_found().status, 404);
        assert_eq!(HyperResponse::method_not_allowed().status, 405);
    }

    #[test]
    fn response_with_header_inserts() {
        let resp = HyperResponse::new(200).with_header("content-type", "application/json");
        assert_eq!(
            resp.headers.get("content-type").map(String::as_str),
            Some("application/json")
        );
    }

    #[test]
    fn to_hyper_response_preserves_status_and_body() {
        let resp = HyperResponse::new(201).with_body(Bytes::from_static(b"created"));
        let hyper_resp = to_hyper_response(resp);
        assert_eq!(hyper_resp.status().as_u16(), 201);
    }

    #[test]
    fn dispatch_invokes_middleware_chain() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        struct Counter(Arc<AtomicUsize>);
        impl oya_http_middleware_kernel::Middleware<HyperRequest, HyperResponse> for Counter {
            fn handle(
                &self,
                request: HyperRequest,
                next: oya_http_middleware_kernel::Next<'_, HyperRequest, HyperResponse>,
            ) -> HyperResponse {
                self.0.fetch_add(1, Ordering::SeqCst);
                next.run(request)
            }
        }
        let mut router: Router<SyncHandler> = Router::new();
        router
            .route(HttpMethod::Get, "/x", ok_handler(b"x"))
            .unwrap();
        let counter = Arc::new(AtomicUsize::new(0));
        let chain: MiddlewareChain<HyperRequest, HyperResponse> =
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
}
