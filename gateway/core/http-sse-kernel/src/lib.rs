//! Server-Sent Events (SSE) framing kernel — pure std-only.
//!
//! Layer 3 of the hyper foundation. Produces the on-wire `text/event-stream`
//! byte format from typed events. The hyper-runtime adapter (Layer 5) wraps
//! the byte stream in a `hyper::Body`. Renamed from `-domain` to `-kernel`
//! per ADR-0056 v4.1 canonical 12-layer enum (this module is pure types +
//! a serializer; no business logic; layer = kernel).
//!
//! Per ADR-0092 Phase 9 (S7 + S8 security): `render()` sanitizes dangerous
//! bytes (CR, NUL, other control chars except LF in `data:` payloads) at
//! output time. This is defense-in-depth — even if a caller injects raw
//! bytes via the constructor, the on-wire output cannot smuggle a synthetic
//! event field. Callers handling fully-untrusted input SHOULD additionally
//! use the `try_*` constructors which return `Result<_, SseError>` on
//! invalid input.
//!
//! Format per WHATWG SSE spec:
//!   - `id: <event-id>\n`
//!   - `event: <event-name>\n`
//!   - `data: <line>\n` (repeated per `\n` in the payload)
//!   - blank line `\n` terminates the event
//!   - `retry: <ms>\n` (optional retry hint to the browser)
//!   - `: <comment>\n` (optional heartbeat / keepalive comment)
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// One SSE event ready for serialization.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct SseEvent {
    /// `id:` field. Browser uses it for Last-Event-ID resume.
    // data_class: INTERNAL_ONLY
    pub id: Option<String>, // data_class: INTERNAL_ONLY
    /// `event:` field (event type name). When absent the client fires
    /// the default `message` event.
    // data_class: INTERNAL_ONLY
    pub event: Option<String>, // data_class: INTERNAL_ONLY
    /// `data:` field payload. Multi-line strings are emitted as one
    /// `data:` line per `\n`.
    // data_class: INTERNAL_ONLY
    pub data: String, // data_class: INTERNAL_ONLY
    /// `retry:` reconnection-delay hint in milliseconds (optional).
    // data_class: INTERNAL_ONLY
    pub retry_ms: Option<u64>, // data_class: INTERNAL_ONLY
}

/// SSE construction / validation errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SseError {
    /// Field contains a CR byte (`\r`); some clients treat CR as a line
    /// terminator and would split the event.
    CarriageReturnInField { field: &'static str },
    /// Field contains a NUL byte; smuggling vector against text-stream
    /// parsers.
    NullByteInField { field: &'static str },
    /// `id` or `event` field contains a newline; these are single-line
    /// fields per the SSE spec and a `\n` would terminate them early.
    NewlineInInlineField { field: &'static str },
}

impl SseEvent {
    /// Construct an event with the given data payload. Ergonomic; assumes
    /// caller-controlled input. For untrusted input use `try_data`.
    ///
    /// On dangerous input (CR / NUL) this constructor still succeeds; the
    /// `render()` step sanitizes dangerous bytes (defense in depth). For
    /// callers who want fail-fast detection, use `try_data` instead.
    pub fn data(payload: impl Into<String>) -> Self {
        Self {
            id: None,
            event: None,
            data: payload.into(),
            retry_ms: None,
        }
    }

    /// Construct an event with the given data payload. Returns `Err` if the
    /// payload contains CR or NUL bytes. Suitable for untrusted input.
    pub fn try_data(payload: impl Into<String>) -> Result<Self, SseError> {
        let payload = payload.into();
        validate_data(&payload)?;
        Ok(Self::data(payload))
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// As `with_id` but rejects CR/LF/NUL bytes that would terminate the
    /// inline field early.
    pub fn try_with_id(mut self, id: impl Into<String>) -> Result<Self, SseError> {
        let id = id.into();
        validate_inline_field(&id, "id")?;
        self.id = Some(id);
        Ok(self)
    }

    pub fn with_event(mut self, event: impl Into<String>) -> Self {
        self.event = Some(event.into());
        self
    }

    pub fn try_with_event(mut self, event: impl Into<String>) -> Result<Self, SseError> {
        let event = event.into();
        validate_inline_field(&event, "event")?;
        self.event = Some(event);
        Ok(self)
    }

    pub fn with_retry_ms(mut self, retry_ms: u64) -> Self {
        self.retry_ms = Some(retry_ms);
        self
    }

    /// Serialize this event to its on-wire string form (terminator included).
    ///
    /// Per ADR-0092 Phase 9 (S7 + S8): dangerous bytes are sanitized at
    /// render time as defense-in-depth. CR is stripped; NUL is stripped;
    /// other control bytes < 0x20 (except `\n`, which is the line splitter
    /// for `data:`) are stripped. Inline fields (`id:`, `event:`, `retry:`)
    /// additionally strip `\n` because they are single-line per spec.
    pub fn render(&self) -> String {
        let mut out = String::new();
        if let Some(id) = &self.id
            && !id.is_empty()
        {
            out.push_str("id: ");
            out.push_str(&sanitize_inline(id));
            out.push('\n');
        }
        if let Some(event) = &self.event
            && !event.is_empty()
        {
            out.push_str("event: ");
            out.push_str(&sanitize_inline(event));
            out.push('\n');
        }
        if let Some(retry_ms) = self.retry_ms {
            out.push_str("retry: ");
            out.push_str(&retry_ms.to_string());
            out.push('\n');
        }
        let sanitized = sanitize_data(&self.data);
        for line in sanitized.split('\n') {
            out.push_str("data: ");
            out.push_str(line);
            out.push('\n');
        }
        out.push('\n');
        out
    }
}

/// Render a heartbeat comment line. Keepalive without payload.
///
/// Sanitizes the comment for CR / NUL / LF — a `\n` would terminate the
/// comment and could inject a synthetic event field on the next line.
pub fn render_heartbeat(comment: &str) -> String {
    let mut out = String::new();
    out.push_str(": ");
    out.push_str(&sanitize_inline(comment));
    out.push('\n');
    out.push('\n');
    out
}

/// As `render_heartbeat` but returns `Err` if the comment contains CR /
/// NUL / LF rather than silently sanitizing.
pub fn try_render_heartbeat(comment: &str) -> Result<String, SseError> {
    validate_inline_field(comment, "comment")?;
    Ok(render_heartbeat(comment))
}

/// The canonical content-type header value for an SSE stream.
pub const SSE_CONTENT_TYPE: &str = "text/event-stream";

// ---- Sanitization + validation helpers ----

fn sanitize_inline(s: &str) -> String {
    s.chars()
        .filter(|c| {
            let b = *c as u32;
            // Strip CR, NUL, LF, and other C0 controls.
            *c != '\r' && *c != '\0' && *c != '\n' && !(b > 0 && b < 0x20)
        })
        .collect()
}

fn sanitize_data(s: &str) -> String {
    s.chars()
        .filter(|c| {
            // Preserve LF (data line splitter); strip CR, NUL, other C0.
            *c == '\n' || (*c != '\r' && *c != '\0' && !((*c as u32) > 0 && (*c as u32) < 0x20))
        })
        .collect()
}

fn validate_data(s: &str) -> Result<(), SseError> {
    if s.contains('\r') {
        return Err(SseError::CarriageReturnInField { field: "data" });
    }
    if s.contains('\0') {
        return Err(SseError::NullByteInField { field: "data" });
    }
    Ok(())
}

fn validate_inline_field(s: &str, field: &'static str) -> Result<(), SseError> {
    if s.contains('\r') {
        return Err(SseError::CarriageReturnInField { field });
    }
    if s.contains('\0') {
        return Err(SseError::NullByteInField { field });
    }
    if s.contains('\n') {
        return Err(SseError::NewlineInInlineField { field });
    }
    Ok(())
}

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

    // ---- Phase 9 (S7 + S8) injection-defense fixtures ----

    // F3 adversarial: CR in data is STRIPPED at render time — even if a
    // caller bypassed try_data, the on-wire stream cannot smuggle a fake
    // event field via `\r`.
    #[test]
    fn render_strips_cr_in_data_payload() {
        let event = SseEvent::data("hello\rid: 999");
        let rendered = event.render();
        assert!(!rendered.contains('\r'));
        // No injected "id: 999" line at the EVENT level — it's still on a
        // single data: line because render strips CR.
        assert!(rendered.contains("data: helloid: 999\n"));
    }

    #[test]
    fn render_strips_null_in_data_payload() {
        let event = SseEvent::data("hello\0world");
        let rendered = event.render();
        assert!(!rendered.contains('\0'));
        assert!(rendered.contains("data: helloworld"));
    }

    #[test]
    fn render_strips_cr_in_id_field() {
        let event = SseEvent::data("payload").with_id("42\rinjected: yes");
        let rendered = event.render();
        assert!(!rendered.contains('\r'));
        // id line is preserved but the injection bytes after \r are merged
        // into the same id line — no smuggled `injected:` field.
        assert!(rendered.contains("id: 42injected: yes\n"));
        // Crucially: no `\n` between the id and the injected text.
        let id_line = rendered.lines().find(|l| l.starts_with("id:")).unwrap();
        assert!(id_line.contains("42injected: yes"));
    }

    #[test]
    fn render_strips_newline_in_inline_event_field() {
        let event = SseEvent::data("payload").with_event("ping\nevent: pwned");
        let rendered = event.render();
        // Newline stripped from inline `event:` field; the injection attempt
        // collapses into one event line.
        let event_lines: Vec<&str> = rendered
            .lines()
            .filter(|l| l.starts_with("event:"))
            .collect();
        assert_eq!(event_lines.len(), 1);
        assert!(event_lines[0].contains("pingevent: pwned"));
    }

    #[test]
    fn render_heartbeat_strips_newline_in_comment() {
        let h = render_heartbeat("ok\nevent: pwned");
        // Single comment line; no injected event line. The smuggled bytes
        // `event: pwned` remain INSIDE the comment line (clients ignore
        // lines starting with `:`); what matters is no LINE starts with
        // `event:`.
        let lines: Vec<&str> = h.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].starts_with(": "));
        for line in h.lines() {
            assert!(
                !line.starts_with("event:"),
                "smuggled event line: `{}`",
                line
            );
        }
    }

    #[test]
    fn render_heartbeat_strips_null_in_comment() {
        let h = render_heartbeat("ok\0boom");
        assert!(!h.contains('\0'));
        assert!(h.starts_with(": okboom"));
    }

    // F3 adversarial: try_data rejects CR/NUL with specific error variants.
    #[test]
    fn try_data_rejects_cr() {
        let err = SseEvent::try_data("hello\rinjection").unwrap_err();
        assert_eq!(err, SseError::CarriageReturnInField { field: "data" });
    }

    #[test]
    fn try_data_rejects_null() {
        let err = SseEvent::try_data("hello\0null").unwrap_err();
        assert_eq!(err, SseError::NullByteInField { field: "data" });
    }

    #[test]
    fn try_data_accepts_clean_payload_including_newlines() {
        // newlines in data are LEGITIMATE — they split into multiple data: lines.
        let event = SseEvent::try_data("line one\nline two").unwrap();
        assert_eq!(event.data, "line one\nline two");
    }

    #[test]
    fn try_with_id_rejects_newline() {
        let err = SseEvent::data("payload")
            .try_with_id("42\nid: 999")
            .unwrap_err();
        assert_eq!(err, SseError::NewlineInInlineField { field: "id" });
    }

    #[test]
    fn try_with_event_rejects_cr() {
        let err = SseEvent::data("payload")
            .try_with_event("ping\rinject")
            .unwrap_err();
        assert_eq!(err, SseError::CarriageReturnInField { field: "event" });
    }

    #[test]
    fn try_render_heartbeat_rejects_newline() {
        let err = try_render_heartbeat("ok\nevent: pwned").unwrap_err();
        assert_eq!(err, SseError::NewlineInInlineField { field: "comment" });
    }

    #[test]
    fn try_render_heartbeat_accepts_clean_comment() {
        let h = try_render_heartbeat("keep-alive").unwrap();
        assert_eq!(h, ": keep-alive\n\n");
    }

    // F3 adversarial: render does NOT strip legitimate ASCII printable chars.
    #[test]
    fn render_preserves_legitimate_payload_bytes() {
        let event = SseEvent::data("payload-with-symbols!@#$%^&*()");
        let rendered = event.render();
        assert!(rendered.contains("payload-with-symbols!@#$%^&*()"));
    }

    // F3 adversarial: defense in depth proven — even SseEvent::data() (the
    // lenient constructor) cannot produce a render output that smuggles a
    // synthetic event field. Iterates a bunch of injection-shaped payloads.
    #[test]
    fn defense_in_depth_render_never_emits_injected_field() {
        let attacks = [
            "ok\rid: 9999",
            "ok\0\nevent: pwned\n",
            "ok\x01\x02\x03event-shaped",
            "ok\x7f\x1bsmuggled",
        ];
        for attack in attacks {
            let event = SseEvent::data(attack);
            let rendered = event.render();
            assert!(!rendered.contains('\r'), "attack `{attack}` left CR");
            assert!(!rendered.contains('\0'), "attack `{attack}` left NUL");
            // Every line that begins with `id: `, `event: `, `retry: `, or
            // `:` (heartbeat) MUST originate from an explicit setter; the
            // data payload alone must never produce such a line.
            for line in rendered.lines() {
                if line.starts_with("id: ")
                    || line.starts_with("event: ")
                    || line.starts_with("retry: ")
                {
                    panic!(
                        "attack `{}` produced unexpected event field line: `{}`",
                        attack, line
                    );
                }
            }
        }
    }
}
