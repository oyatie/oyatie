//! Span-accurate safe-edit primitives over parsed BUCK text (ADR-0549).
//!
//! Every primitive computes a byte-exact edit from PARSED spans (never substring heuristics),
//! so comments, strings with escaped quotes/parens, and unusual indentation cannot corrupt the
//! result — the classes that produced the historical missing-comma / double-comma corruption
//! vectors (FRIC-1781190000) and forced the comment-bearing-block refusals (ADR-0545) and the
//! BUCK `--fix` refusal-only descope (ADR-0547 D6, FRIC-1781200001).
//!
//! Primitives REFUSE (return [`EditError`]) instead of guessing. Callers MUST route the result
//! through the [`crate::harness`] write-through guard before persisting (reparse + semantic
//! validation + pre-image rollback).

use crate::lexer::Span;
use crate::parser::{CallExpr, DictExpr, ListExpr};

/// A refused edit: applying it could not be proven sound from the parsed spans.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditError {
    pub message: String,
}

impl std::fmt::Display for EditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "edit refused: {}", self.message)
    }
}

impl std::error::Error for EditError {}

fn err(message: impl Into<String>) -> EditError {
    EditError {
        message: message.into(),
    }
}

/// Replace `span` with `replacement`. Refuses an out-of-range or boundary-invalid span.
pub fn replace_span(text: &str, span: Span, replacement: &str) -> Result<String, EditError> {
    if span.end < span.start || span.end > text.len() {
        return Err(err(format!(
            "span {}..{} out of range for text of {} bytes",
            span.start,
            span.end,
            text.len()
        )));
    }
    if !text.is_char_boundary(span.start) || !text.is_char_boundary(span.end) {
        return Err(err("span does not fall on char boundaries"));
    }
    let mut out = String::with_capacity(text.len() + replacement.len());
    out.push_str(&text[..span.start]);
    out.push_str(replacement);
    out.push_str(&text[span.end..]);
    Ok(out)
}

/// Insert `insertion` at byte `offset`. Refuses out-of-range/boundary-invalid offsets.
pub fn insert_at(text: &str, offset: usize, insertion: &str) -> Result<String, EditError> {
    replace_span(text, Span::new(offset, offset), insertion)
}

/// Insert a new keyword argument into a call, immediately before its closing paren, supplying
/// the separating comma iff the last existing argument lacks one. `kwarg_src` is the argument
/// source WITHOUT leading indentation or trailing comma (e.g. `mapped_srcs = {\n        ...\n    }`).
///
/// Sound under comments: the comma decision reads the PARSED last-arg comma offset, never a
/// trimmed-text heuristic, so `deps = [],  # trailing comment` (the FRIC-1781190000 probe that
/// forced the prior refusal guard) gets a correct single comma after `]` — never a double comma,
/// never a comma swallowed by the comment.
pub fn insert_kwarg(text: &str, call: &CallExpr, kwarg_src: &str) -> Result<String, EditError> {
    let close = call.close_paren;
    if close >= text.len() || !text.is_char_boundary(close) {
        return Err(err("call closing paren offset out of range"));
    }
    match call.args.last() {
        None => {
            // Empty call: `kind()` -> `kind(\n    kwarg,\n)`.
            insert_at(text, close, &format!("\n    {kwarg_src},\n"))
        }
        Some(last) => {
            if last.comma.is_some() {
                // Trailing comma already present: append the new kwarg before `)`.
                insert_at(text, close, &format!("    {kwarg_src},\n"))
            } else {
                // No trailing comma: add one right after the last argument's value end (NOT
                // before the `)`, which may be separated from the value by a comment).
                let with_comma = insert_at(text, last.span.end, ",")?;
                // The close paren shifted by 1.
                insert_at(&with_comma, close + 1, &format!("    {kwarg_src},\n"))
            }
        }
    }
}

/// Insert a `"key": "value"` entry at the FRONT of a dict literal. Refuses a comprehension
/// dict (a comprehension admits no extra entries — inserting one is a parse error by
/// construction, the corruption shape the prior round-trip validator caught only after the
/// fact).
pub fn insert_dict_entry(
    text: &str,
    dict: &DictExpr,
    key: &str,
    value: &str,
) -> Result<String, EditError> {
    if dict.comprehension.is_some() {
        return Err(err(
            "cannot insert an entry into a dict comprehension; edit the variable assembly site instead",
        ));
    }
    let insertion = format!("\n        \"{key}\": \"{value}\",");
    insert_at(text, dict.open_brace + 1, &insertion)
}

/// Remove element `index` from a list literal, including exactly one adjacent comma (the
/// element's own trailing comma when present, else the PREVIOUS element's comma for a last
/// element) and the surrounding line whitespace when the element sits on its own line.
pub fn remove_list_element(text: &str, list: &ListExpr, index: usize) -> Result<String, EditError> {
    let Some(element) = list.elements.get(index) else {
        return Err(err(format!(
            "list element index {index} out of range ({} elements)",
            list.elements.len()
        )));
    };
    let mut start = element.value.span.start;
    let mut end = element.value.span.end;
    if let Some(comma) = element.comma {
        // Include the trailing comma.
        end = comma + 1;
    } else if index > 0 {
        // Last element without trailing comma: include the previous element's comma.
        if let Some(prev_comma) = list.elements.get(index - 1).and_then(|prev| prev.comma) {
            start = prev_comma;
        }
    }
    // Widen to consume an element-on-its-own-line: leading indentation back to the line start
    // and the trailing newline, ONLY when both sides are pure whitespace (otherwise leave the
    // neighbors untouched — e.g. a trailing comment stays).
    let bytes = text.as_bytes();
    let line_start = text[..start].rfind('\n').map(|p| p + 1).unwrap_or(0);
    let leading_is_ws = text[line_start..start]
        .chars()
        .all(|c| c == ' ' || c == '\t');
    let mut line_end = end;
    while line_end < bytes.len() && (bytes[line_end] == b' ' || bytes[line_end] == b'\t') {
        line_end += 1;
    }
    let trailing_is_newline = bytes.get(line_end) == Some(&b'\n');
    if leading_is_ws && trailing_is_newline && line_start > list.open_bracket {
        start = line_start;
        end = line_end + 1;
    }
    replace_span(text, Span::new(start, end), "")
}
