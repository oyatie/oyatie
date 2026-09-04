use std::fmt;

use crate::{IdempotencyKey, ResourceName, ResourceProvider};

use super::violation;

/// A single conformance divergence: which check failed and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceViolation {
    pub check: &'static str, // data_class: INTERNAL_ONLY
    pub detail: String,      // data_class: INTERNAL_ONLY
}

impl fmt::Display for ConformanceViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.check, self.detail)
    }
}

impl std::error::Error for ConformanceViolation {}

/// What a service supplies to run the harness: a fresh provider per check
/// plus deterministic, ordinal-indexed fixtures. Distinct ordinals MUST
/// yield distinct payloads (`resource_payload(a) != resource_payload(b)` for
/// `a != b`).
pub trait ConformanceFixture {
    /// The provider under test.
    type Provider: ResourceProvider;

    /// A FRESH, empty provider (checks never share state).
    fn fresh_provider(&self) -> Self::Provider;

    /// The collection the harness exercises (slug form).
    fn collection(&self) -> &str;

    /// A deterministic resource payload for `ordinal`.
    fn resource_payload(&self, ordinal: u32) -> <Self::Provider as ResourceProvider>::Resource;

    /// A deterministic resource name for `ordinal`.
    fn resource_name(&self, ordinal: u32) -> Result<ResourceName, ConformanceViolation> {
        ResourceName::new(self.collection(), format!("res-{ordinal:04}"))
            .map_err(|error| violation("fixture", error.to_string()))
    }

    /// A deterministic client-UUID idempotency key for `ordinal`.
    fn idempotency_key(&self, ordinal: u32) -> Result<IdempotencyKey, ConformanceViolation> {
        IdempotencyKey::new(format!("00000000-0000-4000-8000-{ordinal:012x}"))
            .map_err(|error| violation("fixture", error.to_string()))
    }

    /// The expected ORN recorded in operation ledger rows for `name`.
    fn resource_orn(&self, name: &ResourceName) -> String {
        format!(
            "orn:oya:local-test:account-test:{}:{}/{}",
            name.collection(),
            name.collection(),
            name.resource_id()
        )
    }

    /// The expected tenant/account/project scope recorded in operation ledger rows.
    fn tenant_account_project(&self) -> &str {
        "tenant-test/account-test/project-test"
    }

    /// The expected region/cell placement recorded in operation ledger rows.
    fn region_cell(&self) -> &str {
        "local-test/cell-0001"
    }

    /// The expected principal recorded in operation ledger rows.
    fn principal(&self) -> &str {
        "principal:test-harness"
    }
}
