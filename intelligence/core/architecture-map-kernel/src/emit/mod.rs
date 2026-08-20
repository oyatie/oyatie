//! Diagram emitters for the architecture-map kernel.
//!
//! Each submodule exposes `Emitter::render(&ArchitectureMap) -> String`,
//! producing a deterministic textual representation in its target syntax.
//! Output ordering is stable across runs (nodes sorted by id, edges in
//! insertion order) so diffs stay reviewable.

pub mod d2;
pub mod graphviz;
pub mod mermaid;

/// Sanitize a `NodeId` into a token safe for diagram syntax that
/// requires bare identifiers (Graphviz, Mermaid node IDs).
///
/// Strategy: replace any character that is not `[A-Za-z0-9_]` with `_`,
/// and prefix with `n_` if the first char would otherwise be a digit.
/// Result is deterministic but **not** injective across all inputs;
/// callers needing reversibility should keep their own id map.
pub(crate) fn sanitize_id(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len() + 2);
    for (i, c) in raw.chars().enumerate() {
        let keep = c.is_ascii_alphanumeric() || c == '_';
        if !keep {
            out.push('_');
        } else if i == 0 && c.is_ascii_digit() {
            out.push_str("n_");
            out.push(c);
        } else {
            out.push(c);
        }
    }
    if out.is_empty() {
        out.push_str("n_empty");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_alpha_passthrough() {
        assert_eq!(sanitize_id("ops"), "ops");
    }

    #[test]
    fn sanitize_slash_to_underscore() {
        assert_eq!(sanitize_id("ops/docs-portal"), "ops_docs_portal");
    }

    #[test]
    fn sanitize_leading_digit_prefixed() {
        assert_eq!(sanitize_id("9-svc"), "n_9_svc");
    }

    #[test]
    fn sanitize_empty_returns_sentinel() {
        assert_eq!(sanitize_id(""), "n_empty");
    }
}
