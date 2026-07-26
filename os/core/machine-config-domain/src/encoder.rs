//! A tiny, dependency-free document encoder/decoder modeling the YAML
//! multi-document boundary that Talos uses (`---` separated documents, each with
//! an `apiVersion`/`kind` header).
//!
//! This is deliberately not a full YAML implementation: it models just enough
//! structure (the `---` separators and the `kind:` / `version:` header keys) to
//! split, identify, and round-trip the documents that the multi-document
//! [`crate::container::Config`] holds.

use crate::document::{ConfigVersion, DocumentMeta};
use os_kernel::error::{Error, Result};

/// The YAML document separator.
pub const SEPARATOR: &str = "---";

/// One encoded document: its parsed metadata header plus the raw body text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedDocument {
    /// The parsed `apiVersion` + `kind` header.
    pub meta: DocumentMeta,
    /// The full document body (including the header lines).
    pub body: String,
}

impl EncodedDocument {
    /// Build an encoded document from metadata and a body.
    pub fn new(meta: DocumentMeta, body: impl Into<String>) -> Self {
        EncodedDocument {
            meta,
            body: body.into(),
        }
    }
}

/// Find the value of a simple top-level `key: value` line in a document body.
fn scalar_field<'a>(body: &'a str, key: &str) -> Option<&'a str> {
    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(key) {
            let rest = rest.trim_start();
            if let Some(value) = rest.strip_prefix(':') {
                return Some(value.trim().trim_matches('"').trim_matches('\''));
            }
        }
    }
    None
}

/// Parse a single document body into its [`DocumentMeta`].
///
/// Recognizes the legacy form (`version: v1alpha1`, no `kind`, which defaults to
/// kind `v1alpha1`) and the typed form (`apiVersion: v1alpha1` + `kind: Foo`).
fn parse_meta(body: &str) -> Result<DocumentMeta> {
    let version_str = scalar_field(body, "apiVersion")
        .or_else(|| scalar_field(body, "version"))
        .ok_or_else(|| Error::parse("document is missing apiVersion/version"))?;
    let version = ConfigVersion::parse(version_str)?;
    let kind = scalar_field(body, "kind").unwrap_or("v1alpha1").to_string();
    if kind.is_empty() {
        return Err(Error::parse("document kind is empty"));
    }
    Ok(DocumentMeta { version, kind })
}

/// Split a multi-document string into its component document bodies.
///
/// Leading/trailing empty documents (produced by a leading `---` or trailing
/// separator) are dropped.
fn split_documents(input: &str) -> Vec<String> {
    let mut docs = Vec::new();
    let mut current = String::new();
    for line in input.lines() {
        if line.trim() == SEPARATOR {
            if !current.trim().is_empty() {
                docs.push(current.trim_end().to_string());
            }
            current = String::new();
        } else {
            current.push_str(line);
            current.push('\n');
        }
    }
    if !current.trim().is_empty() {
        docs.push(current.trim_end().to_string());
    }
    docs
}

/// Decode a multi-document string into [`EncodedDocument`]s, parsing each
/// document's metadata header.
pub fn decode_documents(input: &str) -> Result<Vec<EncodedDocument>> {
    let mut out = Vec::new();
    for body in split_documents(input) {
        let meta = parse_meta(&body)?;
        out.push(EncodedDocument::new(meta, body));
    }
    if out.is_empty() {
        return Err(Error::parse("no documents found"));
    }
    Ok(out)
}

/// Encode a slice of documents back into the `---`-separated multi-document
/// form, with the separator between (but not before the first) document.
pub fn encode_documents(docs: &[EncodedDocument]) -> String {
    let mut out = String::new();
    for (i, doc) in docs.iter().enumerate() {
        if i > 0 {
            out.push_str(SEPARATOR);
            out.push('\n');
        }
        out.push_str(doc.body.trim_end());
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const MULTI: &str = "version: v1alpha1\nmachine:\n  type: controlplane\n---\napiVersion: v1alpha1\nkind: SideroLinkConfig\napiUrl: grpc://example\n";

    #[test]
    fn decode_splits_and_parses_meta() {
        let docs = decode_documents(MULTI).unwrap();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].meta.kind, "v1alpha1");
        assert_eq!(docs[0].meta.version, ConfigVersion::V1Alpha1);
        assert_eq!(docs[1].meta.kind, "SideroLinkConfig");
    }

    #[test]
    fn legacy_document_defaults_kind() {
        let docs = decode_documents("version: v1alpha1\nmachine: {}\n").unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].meta.kind, "v1alpha1");
    }

    #[test]
    fn round_trip_preserves_documents() {
        let docs = decode_documents(MULTI).unwrap();
        let encoded = encode_documents(&docs);
        let redecoded = decode_documents(&encoded).unwrap();
        assert_eq!(docs, redecoded);
        // Re-encoding the separator count is exactly one for two docs.
        assert_eq!(encoded.matches(SEPARATOR).count(), 1);
    }

    #[test]
    fn missing_version_is_error() {
        assert!(decode_documents("machine:\n  type: worker\n").is_err());
    }

    #[test]
    fn empty_input_is_error() {
        assert!(decode_documents("\n---\n").is_err());
    }

    #[test]
    fn quoted_values_are_unquoted() {
        let docs = decode_documents("apiVersion: \"v1alpha1\"\nkind: 'KmsgLogConfig'\n").unwrap();
        assert_eq!(docs[0].meta.kind, "KmsgLogConfig");
    }
}
