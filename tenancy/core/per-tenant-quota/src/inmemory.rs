//! In-memory adapters for the two read ports.
//!
//! These are the deterministic test/bootstrap doubles for what will become a
//! Postgres-backed tenant read model and override table. They are also the
//! reference semantics the real adapters must reproduce: an unknown tenant is
//! [`QuotaUsecaseError::UnknownTenant`], and an unreachable store is
//! [`QuotaUsecaseError::PersistenceUnavailable`] — never "no override", which
//! would silently promote an outage into a class-default quota.

use std::collections::BTreeMap;

use crate::kernel::{
    QuotaKey, QuotaOverrideRepository, QuotaResource, QuotaUsecaseError, TenantClassReader,
};

/// In-memory tenant class/pack read model.
///
/// `Default` is hand-written, not derived: the derive would zero `available`
/// and hand back the *unavailable* adapter, so `Default::default()` would be a
/// permanently-down store wearing the name of an empty working one.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InMemoryTenantClassReader {
    tenants: BTreeMap<String, (String, Option<String>)>,
    available: bool,
}

impl Default for InMemoryTenantClassReader {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryTenantClassReader {
    /// An empty, reachable reader: every lookup is `UnknownTenant`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tenants: BTreeMap::new(),
            available: true,
        }
    }

    /// A reader standing in for an unreachable store: every lookup is
    /// `PersistenceUnavailable`.
    #[must_use]
    pub fn unavailable() -> Self {
        Self {
            tenants: BTreeMap::new(),
            available: false,
        }
    }

    /// Register a tenant with no pack binding.
    #[must_use]
    pub fn with_tenant(mut self, tenant_id: &str, class: &str) -> Self {
        self.tenants
            .insert(tenant_id.to_owned(), (class.to_owned(), None));
        self
    }

    /// Register a tenant bound to a regulatory/product pack.
    #[must_use]
    pub fn with_packed_tenant(mut self, tenant_id: &str, class: &str, pack: &str) -> Self {
        self.tenants.insert(
            tenant_id.to_owned(),
            (class.to_owned(), Some(pack.to_owned())),
        );
        self
    }

    fn row(&self, tenant_id: &str) -> Result<&(String, Option<String>), QuotaUsecaseError> {
        if !self.available {
            return Err(QuotaUsecaseError::PersistenceUnavailable);
        }
        self.tenants
            .get(tenant_id)
            .ok_or(QuotaUsecaseError::UnknownTenant)
    }
}

impl TenantClassReader for InMemoryTenantClassReader {
    fn class(&self, tenant_id: &str) -> Result<String, QuotaUsecaseError> {
        self.row(tenant_id).map(|(class, _)| class.clone())
    }

    fn pack(&self, tenant_id: &str) -> Result<Option<String>, QuotaUsecaseError> {
        self.row(tenant_id).map(|(_, pack)| pack.clone())
    }
}

/// In-memory per-tenant override table, keyed by `(tenant_id, resource)`.
///
/// `Default` is hand-written for the same reason as
/// [`InMemoryTenantClassReader`]: a derived one would report every lookup as
/// `PersistenceUnavailable`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InMemoryQuotaOverrideRepository {
    overrides: BTreeMap<(String, String), u64>,
    available: bool,
}

impl Default for InMemoryQuotaOverrideRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryQuotaOverrideRepository {
    /// An empty, reachable repository: every lookup is `Ok(None)`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            overrides: BTreeMap::new(),
            available: true,
        }
    }

    /// A repository standing in for an unreachable store.
    #[must_use]
    pub fn unavailable() -> Self {
        Self {
            overrides: BTreeMap::new(),
            available: false,
        }
    }

    /// Record a typed override.
    #[must_use]
    pub fn with_override(mut self, tenant_id: &str, resource: QuotaResource, limit: u64) -> Self {
        self.overrides
            .insert((tenant_id.to_owned(), resource.as_str().to_owned()), limit);
        self
    }
}

impl QuotaOverrideRepository for InMemoryQuotaOverrideRepository {
    fn lookup(&self, key: &QuotaKey) -> Result<Option<u64>, QuotaUsecaseError> {
        if !self.available {
            return Err(QuotaUsecaseError::PersistenceUnavailable);
        }
        // Normalise through the closed vocabulary so a kebab-case REST key and
        // a snake_case stored key address the same row.
        let resource = QuotaResource::parse(&key.resource)?;
        Ok(self
            .overrides
            .get(&(key.tenant_id.clone(), resource.as_str().to_owned()))
            .copied())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unknown_tenant_is_distinguished_from_an_unreachable_store() {
        assert_eq!(
            InMemoryTenantClassReader::new()
                .class("ten_ghost")
                .unwrap_err(),
            QuotaUsecaseError::UnknownTenant
        );
        assert_eq!(
            InMemoryTenantClassReader::unavailable()
                .class("ten_ghost")
                .unwrap_err(),
            QuotaUsecaseError::PersistenceUnavailable
        );
    }

    #[test]
    fn a_tenant_without_a_pack_reports_none_not_an_error() {
        let reader = InMemoryTenantClassReader::new().with_tenant("ten_alpha", "production");
        assert_eq!(reader.class("ten_alpha").unwrap(), "production");
        assert_eq!(reader.pack("ten_alpha").unwrap(), None);
    }

    #[test]
    fn a_packed_tenant_reports_its_pack() {
        let reader =
            InMemoryTenantClassReader::new().with_packed_tenant("ten_alpha", "production", "us-hc");
        assert_eq!(reader.pack("ten_alpha").unwrap().as_deref(), Some("us-hc"));
    }

    #[test]
    fn an_override_lookup_normalises_the_resource_spelling() {
        let repo = InMemoryQuotaOverrideRepository::new().with_override(
            "ten_alpha",
            QuotaResource::ApiCallsPerDay,
            77,
        );
        let kebab = QuotaKey {
            tenant_id: "ten_alpha".to_owned(),
            resource: "api-calls-per-day".to_owned(),
        };
        assert_eq!(repo.lookup(&kebab).unwrap(), Some(77));
    }

    #[test]
    fn an_unreachable_override_store_errors_rather_than_reporting_no_override() {
        let repo = InMemoryQuotaOverrideRepository::unavailable();
        assert_eq!(
            repo.lookup(&QuotaKey::new("ten_alpha", QuotaResource::SeatCount))
                .unwrap_err(),
            QuotaUsecaseError::PersistenceUnavailable
        );
    }

    #[test]
    fn default_is_the_empty_reachable_adapter_not_the_unavailable_one() {
        // A derived `Default` would zero `available` and silently hand back a
        // permanently-down store.
        let reader = InMemoryTenantClassReader::default().with_tenant("ten_a", "production");
        assert_eq!(reader.class("ten_a").unwrap(), "production");
        assert_eq!(
            reader.class("ten_ghost").unwrap_err(),
            QuotaUsecaseError::UnknownTenant,
            "an unknown tenant, not an outage"
        );

        let repo = InMemoryQuotaOverrideRepository::default().with_override(
            "ten_a",
            QuotaResource::SeatCount,
            42,
        );
        assert_eq!(
            repo.lookup(&QuotaKey::new("ten_a", QuotaResource::SeatCount))
                .unwrap(),
            Some(42)
        );
    }

    #[test]
    fn default_and_new_are_the_same_adapter() {
        assert_eq!(
            InMemoryTenantClassReader::default(),
            InMemoryTenantClassReader::new()
        );
        assert_ne!(
            InMemoryTenantClassReader::default(),
            InMemoryTenantClassReader::unavailable()
        );
        assert_eq!(
            InMemoryQuotaOverrideRepository::default(),
            InMemoryQuotaOverrideRepository::new()
        );
        assert_ne!(
            InMemoryQuotaOverrideRepository::default(),
            InMemoryQuotaOverrideRepository::unavailable()
        );
    }

    #[test]
    fn a_missing_override_is_none() {
        let repo = InMemoryQuotaOverrideRepository::new();
        assert_eq!(
            repo.lookup(&QuotaKey::new("ten_alpha", QuotaResource::SeatCount))
                .unwrap(),
            None
        );
    }
}
