//! Server-Sent Events (SSE) framing helper — pure std-only.
//!
//! Layer 3 of the hyper foundation. Produces the on-wire `text/event-stream`
//! byte format from typed events. The hyper-runtime adapter (Layer 5) wraps
//! the byte stream in a `hyper::Body`.
//!
//! Format per WHATWG SSE spec:
//!   - `id: <event-id>\n`
//!   - `event: <event-name>\n`
//!   - `data: <line>\n` (repeated per `\n` in the payload)
//!   - blank line `\n` terminates the event
//!   - `retry: <ms>\n` (optional retry hint to the browser)
//!   - `: <comment>\n` (optional heartbeat / keepalive comment)

/// One SSE event ready for serialization.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct SseEvent {
    /// `id:` field. Browser uses it for Last-Event-ID resume.
    pub id: Option<String>,
    /// `event:` field (event type name). When absent the client fires
    /// the default `message` event.
    pub event: Option<String>,
    /// `data:` field payload. Multi-line strings are emitted as one
    /// `data:` line per `\n`.
    pub data: String,
    /// `retry:` reconnection-delay hint in milliseconds (optional).
    pub retry_ms: Option<u64>,
}

impl SseEvent {
    pub fn data(payload: impl Into<String>) -> Self {
        Self {
            id: None,
            event: None,
            data: payload.into(),
            retry_ms: None,
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn with_event(mut self, event: impl Into<String>) -> Self {
        self.event = Some(event.into());
        self
    }

    pub fn with_retry_ms(mut self, retry_ms: u64) -> Self {
        self.retry_ms = Some(retry_ms);
        self
    }

    /// Serialize this event to its on-wire string form (terminator included).
    pub fn render(&self) -> String {
        let mut out = String::new();
        if let Some(id) = &self.id
            && !id.is_empty()
        {
            out.push_str("id: ");
            out.push_str(id);
            out.push('\n');
        }
        if let Some(event) = &self.event
            && !event.is_empty()
        {
            out.push_str("event: ");
            out.push_str(event);
            out.push('\n');
        }
        if let Some(retry_ms) = self.retry_ms {
            out.push_str("retry: ");
            out.push_str(&retry_ms.to_string());
            out.push('\n');
        }
        for line in self.data.split('\n') {
            out.push_str("data: ");
            out.push_str(line);
            out.push('\n');
        }
        out.push('\n');
        out
    }
}

/// Render a heartbeat comment line. Keepalive without payload.
pub fn render_heartbeat(comment: &str) -> String {
    let mut out = String::new();
    out.push_str(": ");
    out.push_str(comment);
    out.push('\n');
    out.push('\n');
    out
}

/// The canonical content-type header value for an SSE stream.
pub const SSE_CONTENT_TYPE: &str = "text/event-stream";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_minimal_data_only() {
        let event = SseEvent::data("hello");
        let rendered = event.render();
        assert_eq!(rendered, "data: hello\n\n");
    }

    #[test]
    fn render_with_id_event_retry() {
        let event = SseEvent::data("hello")
            .with_id("42")
            .with_event("ping")
            .with_retry_ms(5000);
        let rendered = event.render();
        assert!(rendered.starts_with("id: 42\n"));
        assert!(rendered.contains("event: ping\n"));
        assert!(rendered.contains("retry: 5000\n"));
        assert!(rendered.contains("data: hello\n"));
        assert!(rendered.ends_with("\n\n"));
    }

    #[test]
    fn multi_line_data_emits_multiple_data_lines() {
        let event = SseEvent::data("line one\nline two\nline three");
        let rendered = event.render();
        let data_lines: Vec<&str> = rendered
            .lines()
            .filter(|l| l.starts_with("data:"))
            .collect();
        assert_eq!(data_lines.len(), 3);
        assert_eq!(data_lines[0], "data: line one");
        assert_eq!(data_lines[1], "data: line two");
        assert_eq!(data_lines[2], "data: line three");
    }

    #[test]
    fn empty_id_field_is_omitted() {
        let event = SseEvent::data("hello").with_id("");
        let rendered = event.render();
        assert!(!rendered.contains("id:"));
    }

    #[test]
    fn empty_event_field_is_omitted() {
        let event = SseEvent::data("hello").with_event("");
        let rendered = event.render();
        assert!(!rendered.contains("event:"));
    }

    #[test]
    fn heartbeat_render_shape() {
        let h = render_heartbeat("keep-alive");
        assert_eq!(h, ": keep-alive\n\n");
    }

    #[test]
    fn event_terminator_is_blank_line() {
        let event = SseEvent::data("x");
        assert!(event.render().ends_with("\n\n"));
    }

    #[test]
    fn content_type_is_canonical() {
        assert_eq!(SSE_CONTENT_TYPE, "text/event-stream");
    }
}
