//! ADR-0142 — CRDT portability seam trait.
//!
//! Vendor-neutral surface for swapping the underlying CRDT runtime (Loro
//! today; Yrs/Automerge as candidates). Per ADR-0173 Tier I-asterisk
//! registration of `loro-crdt` in `registry/vendor-lockin-phaseout/index.json`,
//! the workspace MUST host this kernel trait so the vendor-lockin discipline
//! validator can resolve the `seam_adapter_trait` reference.
//!
//! Adapter implementations live in sibling crates:
//! - `collab-crdt-loro-adapter` — Loro 1.x adapter (queued).
//! - `collab-crdt-yrs-adapter`  — Yrs Rust port (queued per ADR-0142).
//!
//! Kernel-tier per ADR-0083: zero production deps. Tests use stdlib only.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fmt;

/// Opaque document identifier the host µservice attaches to every CRDT doc.
///
/// The adapter does NOT mint these — the host coordinates ULID minting via
/// `shared-ulid-id-kernel`. Kernel holds only the by-value handle.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct CrdtDocId(pub String); // data_class: INTERNAL_ONLY

impl fmt::Display for CrdtDocId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Opaque change unit emitted from one peer + applied at another.
///
/// Encoded payload is adapter-defined (Loro's snapshot encoding differs from
/// Yrs's update encoding); the host treats it as opaque bytes and signs
/// it for audit chain emission per ADR-0145 Invariant 1.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrdtChange {
    pub doc_id: CrdtDocId, // data_class: INTERNAL_ONLY
    pub payload: Vec<u8>,  // data_class: INTERNAL_ONLY
}

/// Stable error surface for adapter failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CrdtPortabilityError {
    /// Encoding or wire-format mismatch (typically from cross-adapter replay).
    PayloadInvalid(String),
    /// Document handle unknown to the underlying runtime.
    UnknownDocument(CrdtDocId),
    /// Underlying runtime returned an error string the kernel doesn't classify.
    Adapter(String),
}

impl fmt::Display for CrdtPortabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PayloadInvalid(reason) => write!(f, "crdt payload invalid: {reason}"),
            Self::UnknownDocument(doc) => write!(f, "crdt doc unknown: {doc}"),
            Self::Adapter(reason) => write!(f, "crdt adapter error: {reason}"),
        }
    }
}

impl std::error::Error for CrdtPortabilityError {}

/// Vendor-neutral CRDT runtime trait.
///
/// Every concrete adapter (Loro, Yrs, Automerge) implements this trait. The
/// host µservice (notes, sheets, slides, docs) depends on the trait, never on
/// the concrete adapter. Per ADR-0173, this trait IS the seam.
pub trait CrdtPortabilityRuntime {
    /// Initialise a fresh CRDT document under the given id.
    fn create_doc(&mut self, doc_id: CrdtDocId) -> Result<(), CrdtPortabilityError>;

    /// Apply an incoming change to the named document.
    fn apply_change(&mut self, change: &CrdtChange) -> Result<(), CrdtPortabilityError>;

    /// Emit the current state of the document as an opaque payload.
    fn snapshot(&self, doc_id: &CrdtDocId) -> Result<CrdtChange, CrdtPortabilityError>;
}

/// In-memory test stub.
///
/// Sufficient for kernel tests and for any host µservice that wants to write
/// unit tests against the trait without pulling in a real adapter. Not for
/// production use.
#[derive(Default)]
pub struct InMemoryCrdtRuntime {
    docs: std::collections::BTreeMap<CrdtDocId, Vec<u8>>, // data_class: INTERNAL_ONLY
}

impl CrdtPortabilityRuntime for InMemoryCrdtRuntime {
    fn create_doc(&mut self, doc_id: CrdtDocId) -> Result<(), CrdtPortabilityError> {
        self.docs.entry(doc_id).or_default();
        Ok(())
    }

    fn apply_change(&mut self, change: &CrdtChange) -> Result<(), CrdtPortabilityError> {
        match self.docs.get_mut(&change.doc_id) {
            Some(state) => {
                state.extend_from_slice(&change.payload);
                Ok(())
            }
            None => Err(CrdtPortabilityError::UnknownDocument(change.doc_id.clone())),
        }
    }

    fn snapshot(&self, doc_id: &CrdtDocId) -> Result<CrdtChange, CrdtPortabilityError> {
        match self.docs.get(doc_id) {
            Some(state) => Ok(CrdtChange {
                doc_id: doc_id.clone(),
                payload: state.clone(),
            }),
            None => Err(CrdtPortabilityError::UnknownDocument(doc_id.clone())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_memory_runtime_roundtrips_change() {
        let mut runtime = InMemoryCrdtRuntime::default();
        let doc = CrdtDocId("01JABCDEFGHIJKLMNPQRSTUVWX".to_string());
        runtime.create_doc(doc.clone()).expect("create ok");
        runtime
            .apply_change(&CrdtChange {
                doc_id: doc.clone(),
                payload: b"hello".to_vec(),
            })
            .expect("apply ok");
        let snap = runtime.snapshot(&doc).expect("snapshot ok");
        assert_eq!(snap.payload, b"hello");
    }

    #[test]
    fn unknown_doc_surfaces_error() {
        let runtime = InMemoryCrdtRuntime::default();
        let doc = CrdtDocId("01JMISSINGZZZZZZZZZZZZZZZZ".to_string());
        let err = runtime.snapshot(&doc).unwrap_err();
        assert!(matches!(err, CrdtPortabilityError::UnknownDocument(_)));
    }

    #[test]
    fn error_display_renders_human_messages() {
        let payload_err = CrdtPortabilityError::PayloadInvalid("bad-magic".to_string());
        assert!(format!("{payload_err}").contains("bad-magic"));
    }
}
