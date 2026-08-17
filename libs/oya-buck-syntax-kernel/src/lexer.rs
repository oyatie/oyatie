//! Sound Starlark lexer for the BUCK subset the gates consume (ADR-0549).
//!
//! Soundness properties (each pinned by a fixture in `lib.rs`):
//! - comments (`#` to end of line) are trivia: their content can never open/close a string,
//!   change delimiter depth, or be mistaken for a call/dep token (the comment-blind class);
//! - string literals are lexed with full escape state: `\\` pairs, `\"`, and the
//!   backslash-newline CONTINUATION inside a string (the #693 LOW-2 detect-gap vector
//!   `"third-party//:k\` + newline + `ube"` cooks to `third-party//:kube`);
//! - raw strings (`r"…"`) keep backslashes verbatim while still honoring the
//!   backslash-shields-the-quote rule, and triple-quoted strings may span newlines;
//! - newlines are statement separators ONLY at bracket depth zero (Python implicit line
//!   joining); a backslash-newline outside any string is explicit line joining;
//! - every token carries an exact byte span over the ORIGINAL text, so edits are span-accurate.
//!
//! The lexer is total over its input except for structurally undelimitable text (an
//! unterminated string), which is a hard [`LexError`] — fail-closed, never a guess.

/// A half-open byte range `[start, end)` over the original source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// The source slice this span covers. Returns an empty string when out of range
    /// (defensive; spans produced by this lexer are always in range and on char boundaries).
    pub fn slice<'a>(&self, text: &'a str) -> &'a str {
        text.get(self.start..self.end).unwrap_or("")
    }
}

/// One lexed token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    /// An identifier or keyword (`name`, `glob`, `for`, `in`, ...).
    Ident(String),
    /// A string literal with its COOKED value (escapes resolved, continuations joined).
    Str(String),
    /// An integer-ish literal, kept raw (the gates never do arithmetic).
    Int(String),
    /// A single punctuation/delimiter character: `( ) [ ] { } , = : + . % * - | & < > ! ; @ \`.
    Punct(char),
    /// A multi-character operator (`==`, `!=`, `<=`, `>=`, `+=`, `-=`, `//`, `**`), kept raw so
    /// `a == b` can never be misread as the kwarg `a = (= b)`.
    Op(String),
    /// A statement separator: a newline at bracket depth zero.
    Newline,
}

/// A lexing failure: the text cannot be soundly delimited. Fail-closed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub offset: usize,
    pub message: String,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "lex error at byte {}: {}", self.offset, self.message)
    }
}

impl std::error::Error for LexError {}

/// The lex result: tokens plus comment trivia spans (needed by edit primitives to place
/// insertions without splitting a trailing comment from its line).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LexOutput {
    pub tokens: Vec<Token>,
    pub comments: Vec<Span>,
}

/// Lex `text`. Comments become trivia; newlines inside brackets are joined; strings are cooked.
pub fn lex(text: &str) -> Result<LexOutput, LexError> {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut tokens: Vec<Token> = Vec::new();
    let mut comments: Vec<Span> = Vec::new();
    let mut depth: usize = 0;
    let mut i = 0usize;

    while i < len {
        let b = bytes[i];
        // Whitespace other than newline.
        if b == b' ' || b == b'\t' || b == b'\r' {
            i += 1;
            continue;
        }
        // Newline: a statement separator at depth 0, joined inside brackets.
        if b == b'\n' {
            if depth == 0 {
                // Collapse runs of newlines into one separator token.
                if !matches!(tokens.last().map(|t| &t.kind), Some(TokenKind::Newline)) {
                    tokens.push(Token {
                        kind: TokenKind::Newline,
                        span: Span::new(i, i + 1),
                    });
                }
            }
            i += 1;
            continue;
        }
        // Explicit line joining: backslash immediately followed by a newline (outside strings).
        if b == b'\\' && i + 1 < len && bytes[i + 1] == b'\n' {
            i += 2;
            continue;
        }
        // Comment: trivia to end of line. The newline itself is handled by the loop.
        if b == b'#' {
            let start = i;
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
            comments.push(Span::new(start, i));
            continue;
        }
        // String literal (optionally r/b prefixed, single/double, triple-quoted).
        if b == b'"' || b == b'\'' || is_string_prefix(bytes, i) {
            let (value, end) = lex_string(text, i)?;
            tokens.push(Token {
                kind: TokenKind::Str(value),
                span: Span::new(i, end),
            });
            i = end;
            continue;
        }
        // Identifier.
        if b == b'_' || b.is_ascii_alphabetic() {
            let start = i;
            while i < len && (bytes[i] == b'_' || bytes[i].is_ascii_alphanumeric()) {
                i += 1;
            }
            let ident = &text[start..i];
            tokens.push(Token {
                kind: TokenKind::Ident(ident.to_owned()),
                span: Span::new(start, i),
            });
            continue;
        }
        // Number (kept raw; we never evaluate arithmetic).
        if b.is_ascii_digit() {
            let start = i;
            while i < len
                && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'.')
            {
                i += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Int(text[start..i].to_owned()),
                span: Span::new(start, i),
            });
            continue;
        }
        // Multi-char operators first so `a == b` never lexes as `=` `=`.
        if let Some(op_len) = match_multichar_op(bytes, i) {
            tokens.push(Token {
                kind: TokenKind::Op(text[i..i + op_len].to_owned()),
                span: Span::new(i, i + op_len),
            });
            i += op_len;
            continue;
        }
        // Single punctuation. Track bracket depth for newline joining (saturating: a stray
        // closer cannot underflow; the PARSER decides whether the structure balances).
        let ch = match text[i..].chars().next() {
            Some(c) => c,
            None => break,
        };
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            _ => {}
        }
        tokens.push(Token {
            kind: TokenKind::Punct(ch),
            span: Span::new(i, i + ch.len_utf8()),
        });
        i += ch.len_utf8();
    }

    Ok(LexOutput { tokens, comments })
}

/// True if `bytes[i]` begins a string prefix form: `r"`, `b"`, `rb"`, `br"` (either quote char,
/// possibly triple). The prefix character(s) must be immediately followed by a quote.
fn is_string_prefix(bytes: &[u8], i: usize) -> bool {
    let mut j = i;
    let mut seen = 0usize;
    while seen < 2 {
        match bytes.get(j) {
            Some(b'r') | Some(b'b') | Some(b'R') | Some(b'B') => {
                j += 1;
                seen += 1;
            }
            _ => break,
        }
    }
    seen > 0 && matches!(bytes.get(j), Some(b'"') | Some(b'\''))
}

fn match_multichar_op(bytes: &[u8], i: usize) -> Option<usize> {
    const OPS: [&[u8]; 9] = [
        b"==", b"!=", b"<=", b">=", b"+=", b"-=", b"//", b"**", b"->",
    ];
    OPS.iter()
        .find(|op| bytes.len() >= i + op.len() && &bytes[i..i + op.len()] == **op)
        .map(|op| op.len())
}

/// Lex one string literal starting at `start` (at the prefix if any). Returns the COOKED value
/// and the byte offset just past the closing quote(s).
///
/// Cooking rules (non-raw): `\` + newline is a CONTINUATION (both consumed, nothing emitted —
/// the #693 LOW-2 vector); `\n`/`\t`/`\r`/`\0`/`\\`/`\"`/`\'` map to their characters; an
/// unknown escape keeps the escaped character verbatim (mirrors the prior gate parsers, so
/// migration is behavior-preserving). Raw strings keep `\` + char verbatim, but the backslash
/// still shields a quote from terminating the literal (Python/Starlark raw-string rule).
fn lex_string(text: &str, start: usize) -> Result<(String, usize), LexError> {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = start;
    let mut raw = false;
    // Consume the prefix (r/b in either order, at most one of each).
    while i < len {
        match bytes[i] {
            b'r' | b'R' => {
                raw = true;
                i += 1;
            }
            b'b' | b'B' => {
                i += 1;
            }
            _ => break,
        }
    }
    let quote = match bytes.get(i) {
        Some(b'"') => b'"',
        Some(b'\'') => b'\'',
        _ => {
            return Err(LexError {
                offset: start,
                message: "expected a string quote".to_owned(),
            });
        }
    };
    // Triple-quoted?
    let triple = i + 2 < len && bytes[i + 1] == quote && bytes[i + 2] == quote;
    i += if triple { 3 } else { 1 };

    let mut value = String::new();
    while i < len {
        let b = bytes[i];
        if b == b'\\' {
            // Escape handling — identical structural rule for raw and cooked: the next
            // character can never terminate the string.
            let Some(&next) = bytes.get(i + 1) else {
                return Err(LexError {
                    offset: start,
                    message: "unterminated string escape at end of input".to_owned(),
                });
            };
            if raw {
                value.push('\\');
                // Push the escaped char verbatim (multi-byte safe).
                let ch_len = utf8_len(next);
                if let Some(s) = text.get(i + 1..i + 1 + ch_len) {
                    value.push_str(s);
                }
                i += 1 + ch_len;
                continue;
            }
            // Cooked-string escapes per the Starlark spec buck2 implements (verified against
            // buck2 uquery semantics — review F1: `"k\x75be"` IS `kube` to buck2, so keeping
            // the escape verbatim let a denylisted dep hide by formatting). Any escape class
            // NOT implemented here is a hard LexError — fail-closed, never a guess
            // (ADR-0548 D7 / ADR-0549 D1).
            match next {
                b'\n' => { /* backslash-newline continuation: emit nothing */ }
                b'n' => value.push('\n'),
                b't' => value.push('\t'),
                b'r' => value.push('\r'),
                b'a' => value.push('\x07'),
                b'b' => value.push('\x08'),
                b'f' => value.push('\x0C'),
                b'v' => value.push('\x0B'),
                b'\\' => value.push('\\'),
                b'"' => value.push('"'),
                b'\'' => value.push('\''),
                // \xXX — exactly two hex digits.
                b'x' => {
                    let code = read_hex(bytes, i + 2, 2).ok_or_else(|| LexError {
                        offset: i,
                        message: "\\x escape requires exactly two hex digits".to_owned(),
                    })?;
                    value.push(char_for_code(code, i)?);
                    i += 4;
                    continue;
                }
                // \uXXXX — exactly four hex digits.
                b'u' => {
                    let code = read_hex(bytes, i + 2, 4).ok_or_else(|| LexError {
                        offset: i,
                        message: "\\u escape requires exactly four hex digits".to_owned(),
                    })?;
                    value.push(char_for_code(code, i)?);
                    i += 6;
                    continue;
                }
                // \UXXXXXXXX — exactly eight hex digits.
                b'U' => {
                    let code = read_hex(bytes, i + 2, 8).ok_or_else(|| LexError {
                        offset: i,
                        message: "\\U escape requires exactly eight hex digits".to_owned(),
                    })?;
                    value.push(char_for_code(code, i)?);
                    i += 10;
                    continue;
                }
                // \NNN — one to three octal digits (\0 is the one-digit case).
                b'0'..=b'7' => {
                    let mut code: u32 = 0;
                    let mut digits = 0usize;
                    while digits < 3 {
                        match bytes.get(i + 1 + digits) {
                            Some(d @ b'0'..=b'7') => {
                                code = code * 8 + u32::from(d - b'0');
                                digits += 1;
                            }
                            _ => break,
                        }
                    }
                    value.push(char_for_code(code, i)?);
                    i += 1 + digits;
                    continue;
                }
                other => {
                    // Unimplemented escape class: refuse outright. Verbatim pass-through (the
                    // prior parsers' behavior) silently mis-cooks against buck2 semantics.
                    return Err(LexError {
                        offset: i,
                        message: format!(
                            "unsupported escape sequence `\\{}` — refusing to guess its Starlark semantics (fail-closed)",
                            (other as char).escape_default()
                        ),
                    });
                }
            }
            i += 2;
            continue;
        }
        if b == quote {
            if triple {
                if i + 2 < len && bytes[i + 1] == quote && bytes[i + 2] == quote {
                    return Ok((value, i + 3));
                }
                // A lone quote inside a triple string is content.
                value.push(quote as char);
                i += 1;
                continue;
            }
            return Ok((value, i + 1));
        }
        if b == b'\n' && !triple {
            return Err(LexError {
                offset: start,
                message: "unterminated string literal (bare newline)".to_owned(),
            });
        }
        // Ordinary content (multi-byte safe).
        let ch_len = utf8_len(b);
        if let Some(s) = text.get(i..i + ch_len) {
            value.push_str(s);
        }
        i += ch_len;
    }
    Err(LexError {
        offset: start,
        message: "unterminated string literal".to_owned(),
    })
}

/// Read EXACTLY `n` hex digits starting at `bytes[start]`; `None` if any is missing/invalid.
fn read_hex(bytes: &[u8], start: usize, n: usize) -> Option<u32> {
    let mut code: u32 = 0;
    for offset in 0..n {
        let digit = match bytes.get(start + offset)? {
            d @ b'0'..=b'9' => u32::from(d - b'0'),
            d @ b'a'..=b'f' => u32::from(d - b'a') + 10,
            d @ b'A'..=b'F' => u32::from(d - b'A') + 10,
            _ => return None,
        };
        code = code * 16 + digit;
    }
    Some(code)
}

/// Convert an escape code point to a char, failing closed on invalid scalar values
/// (surrogates, out-of-range) instead of substituting.
fn char_for_code(code: u32, offset: usize) -> Result<char, LexError> {
    char::from_u32(code).ok_or_else(|| LexError {
        offset,
        message: format!("escape code point U+{code:04X} is not a valid Unicode scalar value"),
    })
}

/// UTF-8 encoded length of the character whose first byte is `b` (continuation bytes -> 1 so the
/// scan always makes progress; such input is already invalid mid-char positioning).
fn utf8_len(b: u8) -> usize {
    if b < 0x80 {
        1
    } else if b >= 0xF0 {
        4
    } else if b >= 0xE0 {
        3
    } else if b >= 0xC0 {
        2
    } else {
        1
    }
}
