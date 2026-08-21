//! Quota usecase: the port-driven entry points.
//!
//! This layer does exactly two things — read the tenant's class and pack
//! through [`TenantClassReader`], read the tenant override through
//! [`QuotaOverrideRepository`] — and then hands both to the pure
//! [`crate::domain`] precedence chain. Enforcement stays local to each
//! service; tenancy is the source of truth for the number and its provenance.

use crate::domain::{QuotaLedger, QuotaPolicyCatalog, resolve_from_policy};
use crate::kernel::{
    QuotaDecision, QuotaKey, QuotaOverrideRepository, QuotaResource, QuotaUsecaseError,
    TenantClassReader,
};

/// Resolve one quota using the platform catalog.
///
/// The compatibility entry point: the scaffold's signature, kept working. It
/// now consults the tenant's class instead of discarding it, so a tenant
/// without an override gets its class default attributed to
/// `ClassDefault` rather than a zero attributed to the same layer.
///
/// The catalog is the process-wide
/// [`QuotaPolicyCatalog::platform_defaults_ref`] singleton, not a fresh build
/// per call: this sits on the admission path of every request, and rebuilding
/// ~33 map entries and ~27 owned strings per call to read one number is a cost
/// nobody asked for.
///
/// Callers with their own policy data should use
/// [`resolve_effective_quota`], which takes the catalog explicitly.
///
/// # Errors
/// Whatever the ports report, plus
/// [`QuotaUsecaseError::UnknownResource`] / [`QuotaUsecaseError::NoPolicyForClass`]
/// from the precedence chain.
pub fn resolve<C: TenantClassReader, O: QuotaOverrideRepository>(
    classes: &C,
    overrides: &O,
    key: &QuotaKey,
) -> Result<QuotaDecision, QuotaUsecaseError> {
    resolve_effective_quota(
        QuotaPolicyCatalog::platform_defaults_ref(),
        classes,
        overrides,
        key,
    )
}

/// Resolve one quota against an explicit catalog (IP-022 §D.4).
///
/// Returns the declared limit, the enforced ceiling, the soft threshold, the
/// reset window, and the provenance layer that produced the enforced number.
///
/// # Errors
/// - [`QuotaUsecaseError::UnknownTenant`] / [`QuotaUsecaseError::PersistenceUnavailable`]
///   from the ports.
/// - [`QuotaUsecaseError::UnknownResource`] when the key names a resource
///   outside the closed set.
/// - [`QuotaUsecaseError::UnknownPack`] when the tenant record names a pack
///   the catalog does not declare.
/// - [`QuotaUsecaseError::NoPolicyForClass`] when the catalog declares no
///   default for the tenant's class.
pub fn resolve_effective_quota<C: TenantClassReader, O: QuotaOverrideRepository>(
    catalog: &QuotaPolicyCatalog,
    classes: &C,
    overrides: &O,
    key: &QuotaKey,
) -> Result<QuotaDecision, QuotaUsecaseError> {
    let class = classes.class(&key.tenant_id)?;
    let pack = classes.pack(&key.tenant_id)?;
    let tenant_override = overrides.lookup(key)?;
    resolve_from_policy(catalog, &class, pack.as_deref(), key, tenant_override)
}

/// Resolve the tenant's whole quota sheet, in [`QuotaResource::ALL`] order —
/// what `GET /v1/tenants/{tid}/quotas` (REST follow-up IP-026) serves.
///
/// # Errors
/// As [`resolve_effective_quota`]; the first failing resource aborts the
/// sheet, because a partial sheet would read as a complete one.
pub fn resolve_quota_sheet<C: TenantClassReader, O: QuotaOverrideRepository>(
    catalog: &QuotaPolicyCatalog,
    classes: &C,
    overrides: &O,
    tenant_id: &str,
) -> Result<Vec<QuotaDecision>, QuotaUsecaseError> {
    let class = classes.class(tenant_id)?;
    let pack = classes.pack(tenant_id)?;
    let mut sheet = Vec::with_capacity(QuotaResource::ALL.len());
    for resource in QuotaResource::ALL {
        let key = QuotaKey::new(tenant_id, resource);
        let tenant_override = overrides.lookup(&key)?;
        sheet.push(resolve_from_policy(
            catalog,
            &class,
            pack.as_deref(),
            &key,
            tenant_override,
        )?);
    }
    Ok(sheet)
}

/// Resolve a quota and open a consumption ledger against its *effective*
/// ceiling, with the window opened at the caller-supplied instant.
///
/// Time is a parameter here for the same reason it is one everywhere else in
/// this crate: a ledger that read a clock could not be replayed in a test.
///
/// # Errors
/// As [`resolve_effective_quota`].
pub fn open_ledger<C: TenantClassReader, O: QuotaOverrideRepository>(
    catalog: &QuotaPolicyCatalog,
    classes: &C,
    overrides: &O,
    key: &QuotaKey,
    window_start: u64,
) -> Result<QuotaLedger, QuotaUsecaseError> {
    let decision = resolve_effective_quota(catalog, classes, overrides, key)?;
    Ok(QuotaLedger::from_decision(&decision, window_start))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::US_HC_PACK;
    use crate::inmemory::{InMemoryQuotaOverrideRepository, InMemoryTenantClassReader};
    use crate::kernel::QuotaSource;

    #[test]
    fn the_compat_entry_point_now_reports_a_real_class_default() {
        let classes = InMemoryTenantClassReader::new().with_tenant("ten_alpha", "trial");
        let overrides = InMemoryQuotaOverrideRepository::new();
        let decision = resolve(
            &classes,
            &overrides,
            &QuotaKey::new("ten_alpha", QuotaResource::SeatCount),
        )
        .unwrap();
        assert_eq!(decision.source, QuotaSource::ClassDefault);
        assert_eq!(decision.effective, 5);
        assert_eq!(decision.class, "trial");
    }

    #[test]
    fn a_tenant_override_flows_through_the_ports() {
        let classes = InMemoryTenantClassReader::new().with_tenant("ten_alpha", "production");
        let overrides = InMemoryQuotaOverrideRepository::new().with_override(
            "ten_alpha",
            QuotaResource::SeatCount,
            900,
        );
        let decision = resolve(
            &classes,
            &overrides,
            &QuotaKey::new("ten_alpha", QuotaResource::SeatCount),
        )
        .unwrap();
        assert_eq!(decision.source, QuotaSource::TenantOverride);
        assert_eq!(decision.effective, 900);
    }

    #[test]
    fn an_override_for_another_tenant_does_not_leak() {
        let classes = InMemoryTenantClassReader::new()
            .with_tenant("ten_alpha", "production")
            .with_tenant("ten_beta", "production");
        let overrides = InMemoryQuotaOverrideRepository::new().with_override(
            "ten_beta",
            QuotaResource::SeatCount,
            900,
        );
        let decision = resolve(
            &classes,
            &overrides,
            &QuotaKey::new("ten_alpha", QuotaResource::SeatCount),
        )
        .unwrap();
        assert_eq!(decision.source, QuotaSource::ClassDefault);
        assert_eq!(decision.effective, 500);
    }

    #[test]
    fn an_unknown_tenant_propagates_from_the_class_port() {
        let classes = InMemoryTenantClassReader::new();
        let overrides = InMemoryQuotaOverrideRepository::new();
        assert_eq!(
            resolve(
                &classes,
                &overrides,
                &QuotaKey::new("ten_ghost", QuotaResource::SeatCount)
            )
            .unwrap_err(),
            QuotaUsecaseError::UnknownTenant
        );
    }

    #[test]
    fn an_unavailable_override_store_is_not_treated_as_no_override() {
        let classes = InMemoryTenantClassReader::new().with_tenant("ten_alpha", "production");
        let overrides = InMemoryQuotaOverrideRepository::unavailable();
        assert_eq!(
            resolve(
                &classes,
                &overrides,
                &QuotaKey::new("ten_alpha", QuotaResource::SeatCount)
            )
            .unwrap_err(),
            QuotaUsecaseError::PersistenceUnavailable
        );
    }

    #[test]
    fn the_quota_sheet_covers_every_resource_in_declaration_order() {
        let catalog = QuotaPolicyCatalog::platform_defaults();
        let classes = InMemoryTenantClassReader::new().with_packed_tenant(
            "ten_alpha",
            "production",
            US_HC_PACK,
        );
        let overrides = InMemoryQuotaOverrideRepository::new().with_override(
            "ten_alpha",
            QuotaResource::ApiCallsPerDay,
            9_000_000,
        );
        let sheet = resolve_quota_sheet(&catalog, &classes, &overrides, "ten_alpha").unwrap();
        let resources: Vec<_> = sheet.iter().map(|d| d.resource).collect();
        assert_eq!(resources, QuotaResource::ALL.to_vec());
        let api = sheet
            .iter()
            .find(|d| d.resource == QuotaResource::ApiCallsPerDay)
            .unwrap();
        assert_eq!(api.source, QuotaSource::HardCap);
        assert_eq!(api.effective, 1_000_000, "the us-hc pack cap binds");
        let capability = sheet
            .iter()
            .find(|d| d.resource == QuotaResource::CapabilityInvocationsPerDay)
            .unwrap();
        assert_eq!(capability.source, QuotaSource::PackOverride);
    }

    #[test]
    fn open_ledger_enforces_the_resolved_ceiling() {
        let catalog = QuotaPolicyCatalog::platform_defaults();
        let classes = InMemoryTenantClassReader::new().with_tenant("ten_alpha", "sandbox");
        let overrides = InMemoryQuotaOverrideRepository::new();
        let mut ledger = open_ledger(
            &catalog,
            &classes,
            &overrides,
            &QuotaKey::new("ten_alpha", QuotaResource::SeatCount),
            42,
        )
        .unwrap();
        assert_eq!(ledger.limit(), 3, "the sandbox seat ceiling");
        assert_eq!(ledger.window_start(), 42);
        assert!(ledger.reserve(3).unwrap().is_admitted());
        assert!(!ledger.reserve(1).unwrap().is_admitted());
    }
}
