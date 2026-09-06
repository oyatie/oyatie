use super::*;

/// Wrap a typed `Handler` (with associated `Error`) into the closure-shaped
/// `SyncHandler` the router holds. Renders errors via the handler's
/// `From<Error> for HttpResponse` impl at call time.
///
/// This is the canonical bridge between the kernel `Handler` trait and the
/// router's handler-type-erasure. ADR-0094.
pub fn handler_to_sync<H>(handler: H) -> SyncHandler
where
    H: Handler + 'static,
{
    let handler = Arc::new(handler);
    Arc::new(move |req: HttpRequest| call_into_response(handler.as_ref(), req))
}

/// Default per-request body cap (1 MiB) when `ServerConfig` is constructed
/// via `default()`. Per ADR-0092 + S3 security finding: NEVER read an
/// unbounded request body. Routes that legitimately need larger bodies
/// MUST override via `ServerConfig::with_max_body_bytes`.
pub const DEFAULT_MAX_BODY_BYTES: usize = 1024 * 1024;

/// Default header-read timeout. Hyper's `http1.header_read_timeout` budget;
/// closes Slowloris-style attacks that drip headers one byte at a time.
pub const DEFAULT_HEADER_READ_TIMEOUT: Duration = Duration::from_secs(15);

/// Default keep-alive idle timeout. Connections idle longer than this are
/// dropped to bound concurrent-connection count under load.
pub const DEFAULT_KEEPALIVE_TIMEOUT: Duration = Duration::from_secs(60);

/// Server-level configuration: body cap + connection timeouts. Per ADR-0092
/// Phase 8 (S3 + S4): both fields are mandatory at the seam; defaults are
/// safe but conservative.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServerConfig {
    pub max_body_bytes: usize,
    pub header_read_timeout: Duration,
    pub keepalive_timeout: Duration,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            max_body_bytes: DEFAULT_MAX_BODY_BYTES,
            header_read_timeout: DEFAULT_HEADER_READ_TIMEOUT,
            keepalive_timeout: DEFAULT_KEEPALIVE_TIMEOUT,
        }
    }
}

impl ServerConfig {
    pub fn with_max_body_bytes(mut self, max: usize) -> Self {
        self.max_body_bytes = max;
        self
    }

    pub fn with_header_read_timeout(mut self, dur: Duration) -> Self {
        self.header_read_timeout = dur;
        self
    }

    pub fn with_keepalive_timeout(mut self, dur: Duration) -> Self {
        self.keepalive_timeout = dur;
        self
    }
}

/// Build an `HttpRequest` from a `hyper::Request<Incoming>` by collecting
/// the body fully, bounded by `max_body_bytes`. Bodies exceeding the cap
/// fail with `HyperRuntimeError::BodyTooLarge`; the caller renders 413.
/// This closes the S3 security finding (unbounded body → OOM DoS).
///
/// Boundary conversion: hyper `Bytes` body → kernel `Vec<u8>`. Allocates
/// once, bounded by max_body_bytes.
pub async fn collect_hyper_request(
    req: Request<Incoming>,
    max_body_bytes: usize,
) -> Result<HttpRequest, HyperRuntimeError> {
    let method_str = req.method().as_str().to_string();
    let method = HttpMethod::parse(&method_str)
        .ok_or_else(|| HyperRuntimeError::UnsupportedMethod(method_str.clone()))?;
    let path = req.uri().path().to_string();
    let mut headers = BTreeMap::new();
    // ADR-0092 Phase 10:
    //   * S1: hyper normalizes header names to lowercase already; we
    //     additionally `.to_ascii_lowercase()` defensively so direct-
    //     constructor tests cannot create case-divergent maps.
    //   * S2: non-UTF8 header value is REJECTED with 400 BadHeader, not
    //     silently dropped. Silent drops mask attack signal.
    for (name, value) in req.headers().iter() {
        let name_lower = name.as_str().to_ascii_lowercase();
        match value.to_str() {
            Ok(value_str) => {
                headers.insert(name_lower, value_str.to_string());
            }
            Err(_) => {
                return Err(HyperRuntimeError::NonUtf8HeaderValue {
                    header_name: name_lower,
                });
            }
        }
    }
    let body_bytes = collect_body_with_limit(req.into_body(), max_body_bytes).await?;
    Ok(HttpRequest {
        method,
        path,
        headers,
        body: body_bytes,
        path_captures: BTreeMap::new(),
        matched_template: None,
    })
}

/// Collect a hyper body to `Vec<u8>` with a hard byte cap. Used by
/// `collect_hyper_request`; exposed for tests + future per-route overrides.
pub async fn collect_body_with_limit<B>(
    body: B,
    max_bytes: usize,
) -> Result<Vec<u8>, HyperRuntimeError>
where
    B: Body<Data = Bytes> + Send + Unpin,
    B::Error: std::fmt::Display + std::error::Error + Send + Sync + 'static,
{
    let limited = Limited::new(body, max_bytes);
    let collected = limited.collect().await.map_err(|err| {
        // Limited returns a boxed error; we can't downcast cleanly without
        // a dep on a specific error type, so detect via the display string —
        // the upstream LengthLimitError::ContentLengthMismatch / OverLimit
        // both contain "limit" in their messages.
        let msg = err.to_string();
        if msg.to_lowercase().contains("limit") || msg.to_lowercase().contains("too large") {
            HyperRuntimeError::BodyTooLarge { max_bytes }
        } else {
            HyperRuntimeError::BodyRead(msg)
        }
    })?;
    Ok(collected.to_bytes().to_vec())
}

/// Convert an `HttpResponse` into a hyper `Response<Full<Bytes>>`.
///
/// Boundary conversion: kernel `Vec<u8>` body → hyper `Bytes`. Zero-copy via
/// `Bytes::from(Vec<u8>)` (Bytes takes ownership of the buffer).
pub fn to_hyper_response(resp: HttpResponse) -> Response<Full<Bytes>> {
    response::convert(resp, None)
}

/// Dispatch a request through router → middleware chain → handler.
///
/// Lookups are sync (router) + sync (middleware chain) + sync (handler).
/// The hyper Service wrapper drives this from an async context.
pub fn dispatch(
    request: HttpRequest,
    router: &Router<SyncHandler>,
    chain: &MiddlewareChain<HttpRequest, HttpResponse>,
) -> HttpResponse {
    let (handler, captures, template) = match router.match_route(request.method, &request.path) {
        Some(triple) => triple,
        None if router.path_matches_any_method(&request.path) => {
            return HttpResponse::method_not_allowed();
        }
        None => return HttpResponse::not_found(),
    };
    let template_owned = template.to_string();
    let mut req_with_captures = request;
    req_with_captures.path_captures = captures;
    req_with_captures.matched_template = Some(template_owned);
    let handler_arc = handler.clone();
    chain.execute(req_with_captures, move |req| handler_arc(req))
}

#[derive(Debug)]
pub enum HyperRuntimeError {
    Bind(String),
    BodyRead(String),
    BodyTooLarge { max_bytes: usize },
    UnsupportedMethod(String),
    NonUtf8HeaderValue { header_name: String },
    Config(String),
    Connection(String),
    Runtime(String),
}

impl HyperRuntimeError {
    /// Status code the runtime should emit when converting this error to a
    /// client-facing response.
    pub fn status_code(&self) -> u16 {
        match self {
            HyperRuntimeError::Bind(_) => 500,
            HyperRuntimeError::BodyRead(_) => 400,
            HyperRuntimeError::BodyTooLarge { .. } => 413,
            HyperRuntimeError::UnsupportedMethod(_) => 405,
            HyperRuntimeError::NonUtf8HeaderValue { .. } => 400,
            HyperRuntimeError::Config(_) => 500,
            HyperRuntimeError::Connection(_) => 500,
            HyperRuntimeError::Runtime(_) => 500,
        }
    }
}

impl std::fmt::Display for HyperRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HyperRuntimeError::Bind(reason) => write!(f, "hyper bind failed: {reason}"),
            HyperRuntimeError::BodyRead(reason) => {
                write!(f, "hyper body read failed: {reason}")
            }
            HyperRuntimeError::BodyTooLarge { max_bytes } => {
                write!(f, "request body exceeded max {max_bytes} bytes")
            }
            HyperRuntimeError::UnsupportedMethod(method) => {
                write!(f, "unsupported HTTP method: `{method}`")
            }
            HyperRuntimeError::NonUtf8HeaderValue { header_name } => {
                write!(f, "header `{header_name}` contains non-UTF-8 bytes")
            }
            HyperRuntimeError::Config(reason) => {
                write!(f, "hyper server configuration failed: {reason}")
            }
            HyperRuntimeError::Connection(reason) => {
                write!(f, "hyper connection failed: {reason}")
            }
            HyperRuntimeError::Runtime(reason) => write!(f, "tokio runtime failed: {reason}"),
        }
    }
}

impl std::error::Error for HyperRuntimeError {}

impl From<HyperRuntimeError> for HttpResponse {
    fn from(err: HyperRuntimeError) -> Self {
        HttpResponse::new(err.status_code()).with_body(err.to_string().into_bytes())
    }
}
