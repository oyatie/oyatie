//! Streaming response collection across fanned-out nodes.
//!
//! Server-streaming methods (`Logs`, `Dmesg`, `List`, `Watch`) emit many
//! messages per call. When proxied, apid interleaves the messages from every
//! node, tagging each with the node it came from so the client can demultiplex.
//! [`StreamCollector`] accumulates those tagged messages and can render them as
//! a flat [`StreamResponse`].

use crate::error::ApiError;
use crate::response::{Response, StreamResponse};

/// One streamed message tagged with its originating node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamItem {
    /// The node that produced this message (empty for a local-only stream).
    pub node: String,
    /// The message payload.
    pub response: Response,
}

/// Accumulates streamed messages from one or more nodes.
#[derive(Debug, Clone, Default)]
pub struct StreamCollector {
    items: Vec<StreamItem>,
    /// Per-node terminal errors (a node's stream that ended in failure).
    errors: Vec<(String, ApiError)>,
}

impl StreamCollector {
    /// An empty collector.
    pub fn new() -> Self {
        StreamCollector {
            items: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Record one message from `node`.
    pub fn push(&mut self, node: impl Into<String>, response: Response) {
        self.items.push(StreamItem {
            node: node.into(),
            response,
        });
    }

    /// Record an entire node stream (each response tagged with `node`).
    pub fn extend(&mut self, node: impl Into<String>, responses: Vec<Response>) {
        let node = node.into();
        for r in responses {
            self.items.push(StreamItem {
                node: node.clone(),
                response: r,
            });
        }
    }

    /// Record a terminal error for one node's stream.
    pub fn push_error(&mut self, node: impl Into<String>, error: ApiError) {
        self.errors.push((node.into(), error));
    }

    /// All collected messages, in arrival order.
    pub fn items(&self) -> &[StreamItem] {
        &self.items
    }

    /// Total number of messages collected.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether no messages were collected.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// The per-node terminal errors recorded.
    pub fn errors(&self) -> &[(String, ApiError)] {
        &self.errors
    }

    /// Number of distinct nodes that contributed at least one message.
    pub fn node_count(&self) -> usize {
        let mut seen: Vec<&str> = Vec::new();
        for item in &self.items {
            if !seen.contains(&item.node.as_str()) {
                seen.push(item.node.as_str());
            }
        }
        seen.len()
    }

    /// Flatten into a [`StreamResponse`], prefixing each message with its node
    /// when more than one node contributed (so multi-node logs stay
    /// attributable).
    pub fn into_stream(self) -> StreamResponse {
        let multi = self.node_count() > 1;
        let mut s = StreamResponse::new();
        for item in self.items {
            if multi && !item.node.is_empty() {
                s.push(Response::ok(format!(
                    "{}: {}",
                    item.node,
                    item.response.body()
                )));
            } else {
                s.push(item.response);
            }
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_single_node_stream_unprefixed() {
        let mut c = StreamCollector::new();
        c.extend("", vec![Response::ok("a"), Response::ok("b")]);
        assert_eq!(c.len(), 2);
        assert_eq!(c.node_count(), 1);
        let s = c.into_stream();
        assert_eq!(s.join().body(), "a\nb");
    }

    #[test]
    fn multi_node_messages_are_prefixed() {
        let mut c = StreamCollector::new();
        c.push("10.0.0.1", Response::ok("log1"));
        c.push("10.0.0.2", Response::ok("log2"));
        assert_eq!(c.node_count(), 2);
        let s = c.into_stream();
        assert_eq!(s.join().body(), "10.0.0.1: log1\n10.0.0.2: log2");
    }

    #[test]
    fn records_per_node_errors() {
        let mut c = StreamCollector::new();
        c.push("10.0.0.1", Response::ok("ok"));
        c.push_error("10.0.0.2", ApiError::unavailable("dropped"));
        assert_eq!(c.errors().len(), 1);
        assert_eq!(c.errors()[0].0, "10.0.0.2");
    }

    #[test]
    fn empty_collector() {
        let c = StreamCollector::new();
        assert!(c.is_empty());
        assert_eq!(c.node_count(), 0);
    }
}
