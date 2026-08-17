//! RFC 6376 §3.4 DKIM header and body canonicalization.
//!
//! Pure functions over already-parsed inputs — no network I/O, no DNS lookup,
//! no cryptographic operations.

/// RFC 6376 §3.4 canonicalization algorithm selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DkimCanonicalizationAlgorithm {
    /// RFC 6376 §3.4.1/§3.4.3 — folds header whitespace, lowercases names;
    /// strips trailing whitespace per line and collapses trailing blank lines.
    Relaxed,
    /// RFC 6376 §3.4.2/§3.4.4 — preserves headers verbatim; normalises body
    /// to exactly one trailing CRLF.
    Simple,
}

/// A single parsed mail header name/value pair (after RFC 2822 unfolding if
/// required by the caller).  Name and value are UTF-8 strings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawHeader {
    pub name: String,
    pub value: String,
}

/// Produce the canonical representation of the given headers per RFC 6376 §3.4.
///
/// Relaxed (§3.4.1): lowercased name, whitespace in value collapsed to single
/// SP, leading/trailing whitespace in value stripped.  Output per header:
/// `<name>:<value>\r\n`.
///
/// Simple (§3.4.2): header preserved verbatim; CRLF termination ensured.
///
/// No I/O; pure string transformation.
pub fn canonicalize_header(
    headers: &[RawHeader],
    algorithm: DkimCanonicalizationAlgorithm,
) -> String {
    let mut out = String::new();
    for h in headers {
        match algorithm {
            DkimCanonicalizationAlgorithm::Relaxed => {
                let name = h.name.to_ascii_lowercase();
                // Unfold: replace CRLF SP/HTAB sequences with a single SP.
                let unfolded = unfold_header_value(&h.value);
                // Collapse runs of whitespace to single SP; strip leading/trailing.
                let value = collapse_whitespace(&unfolded);
                out.push_str(&name);
                out.push(':');
                out.push_str(&value);
                out.push_str("\r\n");
            }
            DkimCanonicalizationAlgorithm::Simple => {
                out.push_str(&h.name);
                out.push(':');
                out.push_str(&h.value);
                if !out.ends_with("\r\n") {
                    out.push_str("\r\n");
                }
            }
        }
    }
    out
}

/// Produce the canonical representation of the message body per RFC 6376 §3.4.
///
/// An empty body (or all-whitespace body after processing) becomes a single
/// `\r\n` for both relaxed and simple modes.  No I/O; pure byte transformation.
pub fn canonicalize_body(body: &[u8], algorithm: DkimCanonicalizationAlgorithm) -> Vec<u8> {
    match algorithm {
        DkimCanonicalizationAlgorithm::Relaxed => canonicalize_body_relaxed(body),
        DkimCanonicalizationAlgorithm::Simple => canonicalize_body_simple(body),
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// RFC 6376 §3.4.3 — relaxed body canonicalization.
fn canonicalize_body_relaxed(body: &[u8]) -> Vec<u8> {
    // Split into CRLF-terminated lines (also handle bare LF for robustness).
    let lines = split_lines(body);

    let mut canonical: Vec<Vec<u8>> = lines
        .into_iter()
        .map(|line| {
            // Remove trailing SP and HTAB from each line.
            let stripped = line
                .iter()
                .rev()
                .skip_while(|&&b| b == b' ' || b == b'\t')
                .collect::<Vec<_>>();
            stripped.into_iter().rev().copied().collect()
        })
        .collect();

    // Remove trailing empty lines.
    while canonical
        .last()
        .map(|l: &Vec<u8>| l.is_empty())
        .unwrap_or(false)
    {
        canonical.pop();
    }

    if canonical.is_empty() {
        return b"\r\n".to_vec();
    }

    // Re-join with CRLF and append one trailing CRLF.
    let mut out = Vec::new();
    for line in &canonical {
        out.extend_from_slice(line);
        out.extend_from_slice(b"\r\n");
    }
    out
}

/// RFC 6376 §3.4.4 — simple body canonicalization.
fn canonicalize_body_simple(body: &[u8]) -> Vec<u8> {
    if body.is_empty() {
        return b"\r\n".to_vec();
    }

    let lines = split_lines(body);

    // Remove trailing blank lines.
    let mut lines = lines;
    while lines
        .last()
        .map(|l: &Vec<u8>| l.is_empty())
        .unwrap_or(false)
    {
        lines.pop();
    }

    if lines.is_empty() {
        return b"\r\n".to_vec();
    }

    let mut out = Vec::new();
    for line in &lines {
        out.extend_from_slice(line);
        out.extend_from_slice(b"\r\n");
    }
    out
}

/// Split body bytes into lines (without their CRLF/LF terminators).
fn split_lines(body: &[u8]) -> Vec<Vec<u8>> {
    let mut lines = Vec::new();
    let mut current = Vec::new();
    let mut i = 0;
    while i < body.len() {
        if body[i] == b'\r' && i + 1 < body.len() && body[i + 1] == b'\n' {
            lines.push(current.clone());
            current.clear();
            i += 2;
        } else if body[i] == b'\n' {
            lines.push(current.clone());
            current.clear();
            i += 1;
        } else {
            current.push(body[i]);
            i += 1;
        }
    }
    // Push any remaining content (no trailing newline in input).
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// Unfold RFC 2822 header value: replace CRLF SP / CRLF HTAB (and bare LF SP
/// / LF HTAB) with a single SP.
fn unfold_header_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\r' && i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
            // CRLF — skip if followed by SP/HTAB (folding); otherwise preserve.
            if i + 2 < bytes.len() && (bytes[i + 2] == b' ' || bytes[i + 2] == b'\t') {
                out.push(' ');
                i += 3; // skip CR LF WSP
            } else {
                i += 2; // bare CRLF — skip
            }
        } else if bytes[i] == b'\n' {
            if i + 1 < bytes.len() && (bytes[i + 1] == b' ' || bytes[i + 1] == b'\t') {
                out.push(' ');
                i += 2;
            } else {
                i += 1;
            }
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

/// Collapse all runs of SP/HTAB to a single SP; strip leading/trailing
/// whitespace.
fn collapse_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_ws = false;
    for ch in s.chars() {
        if ch == ' ' || ch == '\t' {
            if !in_ws {
                in_ws = true;
                out.push(' ');
            }
        } else {
            in_ws = false;
            out.push(ch);
        }
    }
    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rh(name: &str, value: &str) -> RawHeader {
        RawHeader {
            name: name.into(),
            value: value.into(),
        }
    }

    // --- Header canonicalization ---

    #[test]
    fn relaxed_header_lowercases_name() {
        let result = canonicalize_header(
            &[rh("Subject", " Hello")],
            DkimCanonicalizationAlgorithm::Relaxed,
        );
        assert_eq!(result, "subject:Hello\r\n");
    }

    #[test]
    fn relaxed_header_collapses_whitespace() {
        let result = canonicalize_header(
            &[rh("Subject", "  foo  bar  ")],
            DkimCanonicalizationAlgorithm::Relaxed,
        );
        assert_eq!(result, "subject:foo bar\r\n");
    }

    #[test]
    fn relaxed_header_strips_folded_whitespace() {
        // Folded value: value contains CRLF SP (RFC 2822 folding)
        let result = canonicalize_header(
            &[rh("Subject", " foo\r\n bar")],
            DkimCanonicalizationAlgorithm::Relaxed,
        );
        // CRLF SP → single SP; leading SP stripped; result = "foo bar"
        assert_eq!(result, "subject:foo bar\r\n");
    }

    #[test]
    fn simple_header_preserves_verbatim() {
        let result = canonicalize_header(
            &[rh("Subject", " Hello")],
            DkimCanonicalizationAlgorithm::Simple,
        );
        assert_eq!(result, "Subject: Hello\r\n");
    }

    #[test]
    fn multiple_headers_canonicalized_in_order() {
        let headers = vec![
            rh("From", " alice@example.com"),
            rh("To", " bob@example.com"),
        ];
        let result = canonicalize_header(&headers, DkimCanonicalizationAlgorithm::Relaxed);
        assert_eq!(result, "from:alice@example.com\r\nto:bob@example.com\r\n");
    }

    // --- Body canonicalization ---

    #[test]
    fn relaxed_body_strips_trailing_whitespace() {
        let body = b"foo   \r\nbar\r\n";
        let result = canonicalize_body(body, DkimCanonicalizationAlgorithm::Relaxed);
        assert_eq!(result, b"foo\r\nbar\r\n");
    }

    #[test]
    fn relaxed_body_collapses_trailing_blank_lines() {
        let body = b"foo\r\n\r\n\r\n";
        let result = canonicalize_body(body, DkimCanonicalizationAlgorithm::Relaxed);
        assert_eq!(result, b"foo\r\n");
    }

    #[test]
    fn relaxed_body_empty_yields_crlf() {
        let result = canonicalize_body(b"", DkimCanonicalizationAlgorithm::Relaxed);
        assert_eq!(result, b"\r\n");
    }

    #[test]
    fn simple_body_collapses_trailing_crlf() {
        let body = b"foo\r\n\r\n\r\n";
        let result = canonicalize_body(body, DkimCanonicalizationAlgorithm::Simple);
        assert_eq!(result, b"foo\r\n");
    }

    #[test]
    fn simple_body_empty_yields_crlf() {
        let result = canonicalize_body(b"", DkimCanonicalizationAlgorithm::Simple);
        assert_eq!(result, b"\r\n");
    }

    #[test]
    fn simple_body_single_crlf_unchanged() {
        let result = canonicalize_body(b"hello\r\n", DkimCanonicalizationAlgorithm::Simple);
        assert_eq!(result, b"hello\r\n");
    }

    #[test]
    fn relaxed_body_strips_tab_trailing_whitespace() {
        let body = b"line\t\t\r\n";
        let result = canonicalize_body(body, DkimCanonicalizationAlgorithm::Relaxed);
        assert_eq!(result, b"line\r\n");
    }
}
