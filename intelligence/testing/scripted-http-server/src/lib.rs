//! First-party scripted HTTP test server — the in-repo replacement for `httpmock`.
//!
//! See `Cargo.toml` for why this crate exists. Behaviourally this is the union of the
//! two hand-rolled servers the repo had already converged on independently:
//!
//! * `scripted_http_server` (ci/controller/github-adapter/tests/review_admission.rs) —
//!   blocking `std::net`, exact `Content-Length` framing, positional response script.
//! * `recording_multi_request_server` (intelligence/adapters/rest/src/lib.rs) —
//!   records the FULL raw request including headers, so a test can assert what
//!   actually went upstream.
//!
//! Each fixed a real gap in the other. This crate keeps both and closes the remaining
//! failure modes that made them awkward to share:
//!
//! * **Header recording.** Every request is parsed into [`RecordedRequest`] with its
//!   headers intact (names lowercased), so `Authorization` / `X-GitHub-Api-Version`
//!   assertions survive the port off httpmock's `when.header(..)` matcher.
//! * **Exact framing.** Bodies are read by `Content-Length` (`read_exact`) or by
//!   chunked decoding — never by one best-effort `read()` into a fixed buffer, which
//!   truncated bodies over 16 KiB and could split a body across TCP segments.
//! * **No hang on under-run.** Nothing joins the server thread, so a code path that
//!   makes FEWER requests than scripted fails on a count assertion instead of
//!   deadlocking the test.
//! * **Visible over-run.** A request past the end of the script is still recorded and
//!   answered with `500 scripted-http-server: script exhausted`, so an unexpected extra
//!   call shows up as a readable assertion failure rather than a connection refusal
//!   that lands nowhere.
//! * **Content routing.** [`ScriptedServer::start_with`] takes a closure over the parsed
//!   request, which is how a matcher-shaped test ports when its request ORDER is
//!   genuinely not fixed. [`ScriptedServer::start`] is the positional special case.
//! * **Concurrency.** Each accepted connection is handled on its own thread, so N
//!   in-flight clients are served simultaneously.
//! * **Streaming.** [`ScriptedResponse::chunks`] writes a `Transfer-Encoding: chunked`
//!   body one frame at a time, flushing between frames with an optional delay, which is
//!   what an SSE test needs to observe incremental delivery.
//!
//! ## Happens-before
//!
//! A request is pushed onto the trace BEFORE its response byte is written. Any client
//! that has observed the response has therefore already observed the recording, so a
//! test may call [`ScriptedServer::requests`] straight after awaiting the client with no
//! sleep and no polling.

#![forbid(unsafe_code)]

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// One request as the server actually received it off the wire.
#[derive(Clone, Debug)]
pub struct RecordedRequest {
    /// Request method, verbatim (`GET`, `POST`, ...).
    pub method: String,
    /// Raw request-target, path and query together (`/repos/o/r/pulls/42?page=1`).
    pub target: String,
    /// HTTP version token from the request line (`HTTP/1.1`).
    pub version: String,
    /// Headers in wire order. Names are lowercased; values are trimmed.
    pub headers: Vec<(String, String)>,
    /// Body bytes, framed by `Content-Length` or by chunked decoding.
    pub body: Vec<u8>,
}

impl RecordedRequest {
    /// Request-target with any query string removed.
    #[must_use]
    pub fn path(&self) -> &str {
        match self.target.split_once('?') {
            Some((path, _)) => path,
            None => &self.target,
        }
    }

    /// Raw query string, without the leading `?`.
    #[must_use]
    pub fn query(&self) -> Option<&str> {
        self.target.split_once('?').map(|(_, query)| query)
    }

    /// First value for a query parameter, percent-decoded.
    #[must_use]
    pub fn query_param(&self, name: &str) -> Option<String> {
        self.query()?.split('&').find_map(|pair| {
            let (key, value) = pair.split_once('=')?;
            (percent_decode(key) == name).then(|| percent_decode(value))
        })
    }

    /// First value for a header. Lookup is case-insensitive.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        let wanted = name.to_ascii_lowercase();
        self.headers
            .iter()
            .find(|(header, _)| *header == wanted)
            .map(|(_, value)| value.as_str())
    }

    /// Every value for a header, in wire order. Lookup is case-insensitive.
    #[must_use]
    pub fn header_values(&self, name: &str) -> Vec<&str> {
        let wanted = name.to_ascii_lowercase();
        self.headers
            .iter()
            .filter(|(header, _)| *header == wanted)
            .map(|(_, value)| value.as_str())
            .collect()
    }

    /// Whether a header is present at all, regardless of value.
    #[must_use]
    pub fn has_header(&self, name: &str) -> bool {
        self.header(name).is_some()
    }

    /// Body decoded as UTF-8, lossily.
    #[must_use]
    pub fn body_string(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    /// Body parsed as JSON.
    ///
    /// # Panics
    /// Panics when the body is not valid JSON — in a test that is the assertion.
    #[must_use]
    pub fn json(&self) -> serde_json::Value {
        serde_json::from_slice(&self.body).unwrap_or_else(|error| {
            panic!(
                "scripted-http-server: request body for {} {} is not JSON: {error}: {}",
                self.method,
                self.target,
                self.body_string()
            )
        })
    }

    /// `METHOD target` — the shape most path assertions want.
    #[must_use]
    pub fn line(&self) -> String {
        format!("{} {}", self.method, self.target)
    }
}

/// One frame of a chunked response body.
#[derive(Clone, Debug)]
pub struct Chunk {
    data: Vec<u8>,
    delay_before: Duration,
}

impl Chunk {
    /// A frame written as soon as the previous one flushed.
    #[must_use]
    pub fn new(data: impl Into<Vec<u8>>) -> Self {
        Self {
            data: data.into(),
            delay_before: Duration::ZERO,
        }
    }

    /// A frame written only after `delay` has elapsed since the previous flush.
    #[must_use]
    pub fn after(delay: Duration, data: impl Into<Vec<u8>>) -> Self {
        Self {
            data: data.into(),
            delay_before: delay,
        }
    }
}

#[derive(Clone, Debug)]
enum Body {
    /// A body of known length, sent with `Content-Length`.
    Fixed(Vec<u8>),
    /// Frames sent with `Transfer-Encoding: chunked`, flushed one at a time.
    Chunked(Vec<Chunk>),
}

/// A scripted response. Built fluently; framed correctly on the way out.
#[derive(Clone, Debug)]
pub struct ScriptedResponse {
    status: u16,
    reason: String,
    headers: Vec<(String, String)>,
    body: Body,
    delay: Duration,
    /// Record the request, then close the socket without writing anything.
    hangup: bool,
    /// Emit these bytes verbatim, bypassing all framing. For malformed-response tests.
    raw: Option<Vec<u8>>,
}

impl ScriptedResponse {
    /// A response with the given status and an empty body.
    #[must_use]
    pub fn status(status: u16) -> Self {
        Self {
            status,
            reason: reason_phrase(status).to_owned(),
            headers: Vec::new(),
            body: Body::Fixed(Vec::new()),
            delay: Duration::ZERO,
            hangup: false,
            raw: None,
        }
    }

    /// `200 OK` with an empty body.
    #[must_use]
    pub fn ok() -> Self {
        Self::status(200)
    }

    /// Emit `bytes` verbatim with no status line, headers or framing added.
    ///
    /// For tests that assert the client's behaviour against a MALFORMED response.
    /// Nothing here is validated — framing is the caller's problem, which is exactly
    /// the point.
    #[must_use]
    pub fn raw(bytes: impl Into<Vec<u8>>) -> Self {
        let mut response = Self::status(200);
        response.raw = Some(bytes.into());
        response
    }

    /// Record the request and then close the connection without writing a byte.
    #[must_use]
    pub fn hangup() -> Self {
        let mut response = Self::status(200);
        response.hangup = true;
        response
    }

    /// Override the reason phrase on the status line.
    #[must_use]
    pub fn reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = reason.into();
        self
    }

    /// Append a response header. Repeat the call to send a header twice.
    #[must_use]
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// A body of known length. Sets `Content-Length`.
    #[must_use]
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = Body::Fixed(body.into());
        self
    }

    /// A `text/plain` body of known length.
    #[must_use]
    pub fn text(self, body: impl Into<String>) -> Self {
        self.header("Content-Type", "text/plain; charset=utf-8")
            .body(body.into().into_bytes())
    }

    /// An `application/json` body of known length.
    #[must_use]
    pub fn json(self, value: &serde_json::Value) -> Self {
        self.header("Content-Type", "application/json")
            .body(value.to_string().into_bytes())
    }

    /// A `Transfer-Encoding: chunked` body, one frame per [`Chunk`], flushed
    /// individually so the client observes incremental delivery.
    #[must_use]
    pub fn chunks(mut self, chunks: Vec<Chunk>) -> Self {
        self.body = Body::Chunked(chunks);
        self
    }

    /// An SSE body: `text/event-stream`, chunked, one frame per event.
    #[must_use]
    pub fn sse(self, events: Vec<Chunk>) -> Self {
        self.header("Content-Type", "text/event-stream")
            .header("Cache-Control", "no-cache")
            .chunks(events)
    }

    /// Wait this long after reading the request before writing any response byte.
    #[must_use]
    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    fn write_to(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        if self.delay > Duration::ZERO {
            thread::sleep(self.delay);
        }
        if self.hangup {
            let _ = stream.shutdown(Shutdown::Both);
            return Ok(());
        }
        if let Some(raw) = &self.raw {
            stream.write_all(raw)?;
            return stream.flush();
        }

        let mut head = format!("HTTP/1.1 {} {}\r\n", self.status, self.reason);
        for (name, value) in &self.headers {
            head.push_str(&format!("{name}: {value}\r\n"));
        }
        // `Connection: close` is what stops a pooling client (reqwest) from reusing a
        // socket the script has finished with. The original in-repo servers that omitted
        // it depended on socket-drop to break keep-alive, which is a race.
        if !self.has_header("connection") {
            head.push_str("Connection: close\r\n");
        }
        match &self.body {
            Body::Fixed(bytes) => {
                if !self.has_header("content-length") {
                    head.push_str(&format!("Content-Length: {}\r\n", bytes.len()));
                }
                head.push_str("\r\n");
                stream.write_all(head.as_bytes())?;
                stream.write_all(bytes)?;
                stream.flush()
            }
            Body::Chunked(chunks) => {
                if !self.has_header("transfer-encoding") {
                    head.push_str("Transfer-Encoding: chunked\r\n");
                }
                head.push_str("\r\n");
                stream.write_all(head.as_bytes())?;
                stream.flush()?;
                for chunk in chunks {
                    if chunk.delay_before > Duration::ZERO {
                        thread::sleep(chunk.delay_before);
                    }
                    stream.write_all(format!("{:x}\r\n", chunk.data.len()).as_bytes())?;
                    stream.write_all(&chunk.data)?;
                    stream.write_all(b"\r\n")?;
                    stream.flush()?;
                }
                stream.write_all(b"0\r\n\r\n")?;
                stream.flush()
            }
        }
    }

    fn has_header(&self, lowercase_name: &str) -> bool {
        self.headers
            .iter()
            .any(|(name, _)| name.to_ascii_lowercase() == lowercase_name)
    }
}

type Responder = Arc<dyn Fn(&RecordedRequest) -> ScriptedResponse + Send + Sync>;

/// A blocking, recording, scripted HTTP/1.1 server bound to an ephemeral loopback port.
///
/// Dropping the server stops the accept loop and closes the listening socket.
pub struct ScriptedServer {
    base_url: String,
    port: u16,
    requests: Arc<Mutex<Vec<RecordedRequest>>>,
    shutdown: Arc<AtomicBool>,
}

impl ScriptedServer {
    /// Serve `responses` positionally: request N gets response N.
    ///
    /// A request past the end of the script is recorded and answered `500`, so an
    /// unexpected extra call surfaces as a count assertion rather than vanishing.
    #[must_use]
    pub fn start(responses: Vec<ScriptedResponse>) -> Self {
        let script = Mutex::new(VecDeque::from(responses));
        Self::start_with(move |request| {
            script
                .lock()
                .expect("scripted-http-server: script mutex poisoned")
                .pop_front()
                .unwrap_or_else(|| {
                    ScriptedResponse::status(500).text(format!(
                        "scripted-http-server: script exhausted at {} {}",
                        request.method, request.target
                    ))
                })
        })
    }

    /// Serve every request with the same response.
    #[must_use]
    pub fn start_always(response: ScriptedResponse) -> Self {
        Self::start_with(move |_| response.clone())
    }

    /// Serve requests by CONTENT rather than by position: `responder` sees the parsed
    /// request and picks the response. This is the port target for a matcher-shaped test
    /// whose request order is genuinely not fixed.
    #[must_use]
    pub fn start_with<F>(responder: F) -> Self
    where
        F: Fn(&RecordedRequest) -> ScriptedResponse + Send + Sync + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").expect("scripted-http-server: bind");
        let address = listener
            .local_addr()
            .expect("scripted-http-server: local_addr");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let shutdown = Arc::new(AtomicBool::new(false));

        let responder: Responder = Arc::new(responder);
        let accept_requests = Arc::clone(&requests);
        let accept_shutdown = Arc::clone(&shutdown);
        thread::spawn(move || {
            for connection in listener.incoming() {
                if accept_shutdown.load(Ordering::SeqCst) {
                    break;
                }
                let Ok(stream) = connection else { continue };
                let responder = Arc::clone(&responder);
                let requests = Arc::clone(&accept_requests);
                // One thread per connection: N simultaneous in-flight clients are all
                // served, which a single sequential accept loop cannot do.
                thread::spawn(move || serve_connection(stream, &responder, &requests));
            }
        });

        Self {
            base_url: format!("http://{address}"),
            port: address.port(),
            requests,
            shutdown,
        }
    }

    /// `http://127.0.0.1:<port>`, with no trailing slash.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// The ephemeral port the server bound.
    #[must_use]
    pub fn port(&self) -> u16 {
        self.port
    }

    /// `base_url` joined with `path` (which should start with `/`).
    #[must_use]
    pub fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    /// Every request received so far, in arrival order.
    ///
    /// Safe to call immediately after awaiting the client: a request is recorded before
    /// its response is written.
    #[must_use]
    pub fn requests(&self) -> Vec<RecordedRequest> {
        self.requests
            .lock()
            .expect("scripted-http-server: request trace poisoned")
            .clone()
    }

    /// Number of requests received so far.
    #[must_use]
    pub fn request_count(&self) -> usize {
        self.requests
            .lock()
            .expect("scripted-http-server: request trace poisoned")
            .len()
    }

    /// `METHOD target` for every request, in arrival order.
    #[must_use]
    pub fn request_lines(&self) -> Vec<String> {
        self.requests().iter().map(RecordedRequest::line).collect()
    }

    /// Requests whose path CONTAINS `needle`.
    #[must_use]
    pub fn requests_matching(&self, needle: &str) -> Vec<RecordedRequest> {
        self.requests()
            .into_iter()
            .filter(|request| request.path().contains(needle))
            .collect()
    }

    /// Block until at least `count` requests have arrived, or `timeout` elapses.
    ///
    /// Only needed when asserting on a request the test does NOT await — a
    /// fire-and-forget background call. Returns the trace either way, so the caller
    /// still asserts the count and a timeout shows up as a normal assertion failure.
    #[must_use]
    pub fn wait_for_requests(&self, count: usize, timeout: Duration) -> Vec<RecordedRequest> {
        let deadline = std::time::Instant::now() + timeout;
        loop {
            let seen = self.requests();
            if seen.len() >= count || std::time::Instant::now() >= deadline {
                return seen;
            }
            thread::sleep(Duration::from_millis(2));
        }
    }
}

impl Drop for ScriptedServer {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        // Unblock the accept loop so it observes the flag and drops the listener.
        let _ = TcpStream::connect(("127.0.0.1", self.port));
    }
}

fn serve_connection(
    mut stream: TcpStream,
    responder: &Responder,
    requests: &Arc<Mutex<Vec<RecordedRequest>>>,
) {
    let Some(request) = read_request(&mut stream) else {
        // A bare connect with no request line — the shutdown poke, or a client that
        // gave up. Nothing to record.
        return;
    };

    // Record BEFORE responding. Every assertion a ported test makes about what the
    // client sent reads this trace, so a connection that is served but not recorded
    // makes those assertions vacuous rather than failing them — the exact false-green
    // this helper exists to replace httpmock's `mock.assert()` with.
    requests
        .lock()
        .expect("scripted server request trace poisoned")
        .push(request.clone());

    let response = responder(&request);
    // A client that hung up mid-exchange is a legitimate test scenario (cancellation,
    // timeout), so a write error here is not a server fault and must not panic a
    // detached thread.
    let _ = response.write_to(&mut stream);
    let _ = stream.shutdown(Shutdown::Write);
}

fn read_request(stream: &mut TcpStream) -> Option<RecordedRequest> {
    let mut reader = BufReader::new(stream);

    let mut request_line = String::new();
    if reader.read_line(&mut request_line).ok()? == 0 {
        return None;
    }
    let mut parts = request_line.split_whitespace();
    let method = parts.next()?.to_owned();
    let target = parts.next().unwrap_or("/").to_owned();
    let version = parts.next().unwrap_or("HTTP/1.1").to_owned();

    let mut headers: Vec<(String, String)> = Vec::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).ok()? == 0 {
            break;
        }
        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.push((name.trim().to_ascii_lowercase(), value.trim().to_owned()));
        }
    }

    let chunked = headers
        .iter()
        .any(|(name, value)| name == "transfer-encoding" && value.contains("chunked"));
    let content_length: usize = headers
        .iter()
        .find(|(name, _)| name == "content-length")
        .and_then(|(_, value)| value.parse().ok())
        .unwrap_or(0);

    let body = if chunked {
        read_chunked_body(&mut reader)
    } else {
        // read_exact, not a best-effort read: a body may arrive across several TCP
        // segments, and a body over any fixed buffer size must not be truncated.
        let mut body = vec![0u8; content_length];
        reader.read_exact(&mut body).ok()?;
        body
    };

    Some(RecordedRequest {
        method,
        target,
        version,
        headers,
        body,
    })
}

fn read_chunked_body(reader: &mut BufReader<&mut TcpStream>) -> Vec<u8> {
    let mut body = Vec::new();
    loop {
        let mut size_line = String::new();
        if reader.read_line(&mut size_line).is_err() {
            break;
        }
        let size_token = size_line.trim();
        let size_token = size_token.split(';').next().unwrap_or("");
        let Ok(size) = usize::from_str_radix(size_token, 16) else {
            break;
        };
        if size == 0 {
            // Consume the trailer section.
            loop {
                let mut trailer = String::new();
                match reader.read_line(&mut trailer) {
                    Ok(0) => break,
                    Ok(_) if trailer.trim().is_empty() => break,
                    Ok(_) => {}
                    Err(_) => break,
                }
            }
            break;
        }
        let mut chunk = vec![0u8; size];
        if reader.read_exact(&mut chunk).is_err() {
            break;
        }
        body.extend_from_slice(&chunk);
        let mut crlf = [0u8; 2];
        if reader.read_exact(&mut crlf).is_err() {
            break;
        }
    }
    body
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                out.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).unwrap_or("");
                match u8::from_str_radix(hex, 16) {
                    Ok(byte) => {
                        out.push(byte);
                        index += 3;
                    }
                    Err(_) => {
                        out.push(bytes[index]);
                        index += 1;
                    }
                }
            }
            byte => {
                out.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        413 => "Payload Too Large",
        422 => "Unprocessable Entity",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        _ => "Status",
    }
}
