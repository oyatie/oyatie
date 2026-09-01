//! The wire shapes of the write surface.
//!
//! `occurred_at_epoch_seconds` is the caller's, not this process's, and
//! that is deliberate: the writer derives every payload byte from the
//! request so a retry is byte-identical and deduplicates. If the facade
//! stamped its own clock, the same request sent twice would produce
//! different bytes under one idempotency key — a loud conflict where the
//! caller expected a dedup. The tenant is absent from this shape by
//! design; it comes from the credential and nothing in the body can move
//! it.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// One Action submission.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubmitRequest {
    /// The `ent_`-prefixed object this Action acts on.
    pub object_ref: String, // data_class: TENANT_SCOPED
    /// The `aty_`-prefixed action type, which must be registered.
    pub action_type: String, // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
    /// The caller's own timestamp — the retry contract depends on it.
    pub occurred_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    /// Property values to write, as declared strings. Typed values ride
    /// the wire's richer vocabulary once the surface admits them; the
    /// legacy string carrier is what the seeded registry declares today.
    pub properties: BTreeMap<String, String>, // data_class: PROPERTY_VALUE_PRIVACY_CLASS
}

/// What became of an accepted submission.
#[derive(Clone, Debug, Serialize)]
pub struct SubmitResponse {
    /// `applied` or `poisoned` — the projection's own verdict, reported
    /// honestly rather than flattened into success.
    pub outcome: &'static str, // data_class: INTERNAL_ONLY
    pub ordinal: u64, // data_class: INTERNAL_ONLY
    /// Whether the log recognized this as a byte-identical retry.
    pub deduplicated: bool, // data_class: INTERNAL_ONLY
    /// Present only when the projection refused the entry it accepted into
    /// the log: the typed reason, never a classified value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poison_reason: Option<String>, // data_class: INTERNAL_ONLY
}

/// A refusal, in the shape the operator procedure documents.
#[derive(Clone, Debug, Serialize)]
pub struct RefusalBody {
    /// Which gate refused: authorization, parameters, admission, or the
    /// surface itself.
    pub gate: String, // data_class: INTERNAL_ONLY
    pub cause: String, // data_class: INTERNAL_ONLY
}
