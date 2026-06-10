//! Typed response envelopes returned by backends and aggregated by the proxy.
//!
//! A single backend produces a [`Response`]. When apid fans a request out to
//! multiple nodes it tags each backend's result with the originating node into a
//! [`NodeResponse`], and the proxy collects those into the multi-node reply that
//! `talosctl` renders one block per node. Streaming methods accumulate into a
//! [`StreamResponse`].

use crate::error::ApiError;

/// A single unary backend response: a body plus an optional error.
///
/// In real apid the body is a marshalled protobuf message; here it is an opaque
/// string so the routing/aggregation logic can be tested directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    body: String,
}

impl Response {
    /// A successful response carrying `body`.
    pub fn ok(body: impl Into<String>) -> Self {
        Response { body: body.into() }
    }

    /// An empty successful response.
    pub fn empty() -> Self {
        Response {
            body: String::new(),
        }
    }

    /// The response body.
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Whether the body is empty.
    pub fn is_empty(&self) -> bool {
        self.body.is_empty()
    }
}

/// A backend response tagged with the node that produced it.
///
/// When apid fans out, each leg can independently succeed or fail; the per-node
/// error is carried inline (rather than aborting the whole call) so a partial
/// failure on one node still returns the other nodes' data — matching apid's
/// behavior of embedding per-node errors in the aggregated reply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeResponse {
    /// The endpoint that produced this leg.
    pub node: String,
    /// The successful payload, if this leg succeeded.
    pub response: Option<Response>,
    /// The per-node error, if this leg failed.
    pub error: Option<ApiError>,
}

impl NodeResponse {
    /// A successful leg from `node`.
    pub fn ok(node: impl Into<String>, response: Response) -> Self {
        NodeResponse {
            node: node.into(),
            response: Some(response),
            error: None,
        }
    }

    /// A failed leg from `node`.
    pub fn failed(node: impl Into<String>, error: ApiError) -> Self {
        NodeResponse {
            node: node.into(),
            response: None,
            error: Some(error),
        }
    }

    /// Whether this leg succeeded.
    pub fn is_ok(&self) -> bool {
        self.error.is_none()
    }
}

/// A server-streaming response: an ordered sequence of message bodies.
///
/// Models `Logs`, `Dmesg`, `List` and `Watch` where the server emits multiple
/// messages over one call. The collector in [`stream`](crate::stream) builds
/// these incrementally.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StreamResponse {
    messages: Vec<Response>,
}

impl StreamResponse {
    /// An empty stream.
    pub fn new() -> Self {
        StreamResponse {
            messages: Vec::new(),
        }
    }

    /// Build a stream from an iterator of responses.
    pub fn from_messages(iter: impl IntoIterator<Item = Response>) -> Self {
        StreamResponse {
            messages: iter.into_iter().collect(),
        }
    }

    /// Append one message to the stream.
    pub fn push(&mut self, response: Response) {
        self.messages.push(response);
    }

    /// The messages emitted so far.
    pub fn messages(&self) -> &[Response] {
        &self.messages
    }

    /// Number of messages in the stream.
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Whether the stream is empty.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Collapse the stream into a single newline-joined [`Response`].
    pub fn join(&self) -> Response {
        let mut body = String::new();
        for (i, m) in self.messages.iter().enumerate() {
            if i > 0 {
                body.push('\n');
            }
            body.push_str(m.body());
        }
        Response::ok(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_basics() {
        assert!(Response::empty().is_empty());
        assert_eq!(Response::ok("v1.7.0").body(), "v1.7.0");
    }

    #[test]
    fn node_response_ok_and_failed() {
        let ok = NodeResponse::ok("10.0.0.1", Response::ok("hi"));
        assert!(ok.is_ok());
        assert_eq!(ok.response.as_ref().unwrap().body(), "hi");

        let bad = NodeResponse::failed("10.0.0.2", ApiError::unavailable("down"));
        assert!(!bad.is_ok());
        assert_eq!(bad.error.as_ref().unwrap().grpc_code(), "Unavailable");
    }

    #[test]
    fn stream_push_and_join() {
        let mut s = StreamResponse::new();
        assert!(s.is_empty());
        s.push(Response::ok("line1"));
        s.push(Response::ok("line2"));
        assert_eq!(s.len(), 2);
        assert_eq!(s.join().body(), "line1\nline2");
    }

    #[test]
    fn stream_from_messages() {
        let s = StreamResponse::from_messages([Response::ok("a"), Response::ok("b")]);
        assert_eq!(s.messages().len(), 2);
        assert_eq!(s.join().body(), "a\nb");
    }
}
