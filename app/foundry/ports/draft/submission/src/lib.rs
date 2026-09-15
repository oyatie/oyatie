#![forbid(unsafe_code)]

use std::{collections::BTreeMap, future::Future, pin::Pin};

use serde::{Deserialize, Serialize};

pub mod conformance;

/// The canonical JSON request may not exceed the HTTP surface's body budget.
/// Typed consumers receive the same bound before edit construction or policy.
pub const MAX_SUBMISSION_BYTES: usize = 2 * 1024 * 1024;

/// The existing Foundry action surface. Identity and tenant are deliberately
/// absent: each submission verifies its credential and obtains a fresh policy
/// decision before admitting registered edits. Values are not authority.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SubmitRequest {
    pub object_ref: String,                   // data_class: TENANT_SCOPED
    pub action_type: String,                  // data_class: INTERNAL_ONLY
    pub idempotency_key: String,              // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64,       // data_class: INTERNAL_ONLY
    pub properties: BTreeMap<String, String>, // data_class: PROPERTY_VALUE_PRIVACY_CLASS
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SubmitResponse {
    pub outcome: &'static str, // data_class: INTERNAL_ONLY
    pub ordinal: u64,          // data_class: INTERNAL_ONLY
    pub deduplicated: bool,    // data_class: INTERNAL_ONLY
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poison_reason: Option<String>, // data_class: INTERNAL_ONLY
}

/// Coarse failures preserve the served refusal taxonomy without exposing
/// credentials, storage errors, or policy-engine internals.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SubmitError {
    Credential,
    UnservedTenant,
    TenantMismatch,
    Authorization,
    Surface { cause: &'static str },
    Refused { gate: String, cause: &'static str },
    Conflict,
    Unavailable,
}

pub type Submission<'a> =
    Pin<Box<dyn Future<Output = Result<SubmitResponse, SubmitError>> + Send + 'a>>;

/// An accepted response follows durable append and fold. A mirror failure must
/// remain accepted and observable as projection lag; a poisoned fold must be
/// reported as poisoned. Retry identity is tenant-scoped and payload-stable.
/// No caller-supplied policy decision, principal, or tenant can bypass the
/// verifier and policy enforcement point, including on a deduplicated retry.
pub trait ActionSubmitter: Send + Sync {
    fn submit<'a>(&'a self, credential: &'a str, request: SubmitRequest) -> Submission<'a>;

    /// Constrain a source handoff to its original tenant. This value is only
    /// an equality constraint against the credential's verified tenant, never
    /// authority to select a different destination. A mismatch appends nothing.
    fn submit_in_tenant<'a>(
        &'a self,
        credential: &'a str,
        expected_tenant: &'a str,
        request: SubmitRequest,
    ) -> Submission<'a>;
}
