use super::*;
use http_router_kernel::Router;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::convert::Infallible;

fn ok_handler(body: &'static [u8]) -> SyncHandler {
    Arc::new(move |_req: HttpRequest| HttpResponse::new(200).with_body(body.to_vec()))
}

fn empty_chain() -> MiddlewareChain<HttpRequest, HttpResponse> {
    MiddlewareChain::new()
}

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

mod dispatch;
mod request_limits;
mod tls_policy;
mod transport;
