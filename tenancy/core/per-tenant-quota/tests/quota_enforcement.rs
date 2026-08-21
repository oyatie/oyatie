//! End-to-end quota behaviour across the public surface: resolve a tenant's
//! ceiling through the ports, enforce it through a ledger, and roll the
//! window — all with time as a parameter.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use tenancy_per_tenant_quota::inmemory::{
    InMemoryQuotaOverrideRepository, InMemoryTenantClassReader,
};
use tenancy_per_tenant_quota::{
    QuotaAllowance, QuotaKey, QuotaOutcome, QuotaPolicyCatalog, QuotaResource, QuotaSource,
    QuotaUsageError, QuotaUsecaseError, ResetWindow, US_HC_PACK, open_ledger, resolve,
    resolve_effective_quota, resolve_quota_sheet,
};

fn classes() -> InMemoryTenantClassReader {
    InMemoryTenantClassReader::new()
        .with_tenant("ten_trial", "trial")
        .with_tenant("ten_prod", "production")
        .with_tenant("ten_sandbox", "sandbox")
        .with_packed_tenant("ten_hc", "production", US_HC_PACK)
        // The spelling `Tenant.jurisdiction_code` uses in the OpenAPI
        // contract — the same pack, shouted.
        .with_packed_tenant("ten_hc_shouty", "production", "US-HC")
        // Bound to a pack the platform catalog does not declare.
        .with_packed_tenant("ten_typo", "production", "pack-us-hc")
        // A sandbox tenant carrying the regulated pack.
        .with_packed_tenant("ten_hc_sandbox", "sandbox", US_HC_PACK)
}

#[test]
fn each_plan_tier_gets_its_own_class_default() {
    let overrides = InMemoryQuotaOverrideRepository::new();
    let classes = classes();
    let resource = QuotaResource::RequestRatePerMinute;

    let trial = resolve(&classes, &overrides, &QuotaKey::new("ten_trial", resource)).unwrap();
    let production = resolve(&classes, &overrides, &QuotaKey::new("ten_prod", resource)).unwrap();
    let sandbox = resolve(
        &classes,
        &overrides,
        &QuotaKey::new("ten_sandbox", resource),
    )
    .unwrap();

    assert!(sandbox.effective < trial.effective);
    assert!(trial.effective < production.effective);
    for decision in [&trial, &production, &sandbox] {
        assert_eq!(decision.source, QuotaSource::ClassDefault);
        assert_eq!(decision.limit, decision.effective, "nothing clamped here");
        assert_eq!(decision.window, ResetWindow::Seconds(60));
    }
}

#[test]
fn the_whole_precedence_chain_is_walked_one_layer_at_a_time() {
    // A non-regulated product pack: it tightens the default but declares no
    // pack hard cap, so a tenant override may legitimately outbid it. That is
    // what makes all four layers observable in one chain.
    let resource = QuotaResource::CapabilityInvocationsPerDay;
    let catalog = QuotaPolicyCatalog::new()
        .with_class_default(
            "production",
            resource,
            QuotaAllowance::standard(500_000, resource),
        )
        .with_pack_override(
            "promo",
            resource,
            QuotaAllowance::standard(300_000, resource),
        );
    let classes = InMemoryTenantClassReader::new()
        .with_tenant("ten_prod", "production")
        .with_packed_tenant("ten_promo", "production", "promo");

    // Layer 1: class default.
    let plain = resolve_effective_quota(
        &catalog,
        &classes,
        &InMemoryQuotaOverrideRepository::new(),
        &QuotaKey::new("ten_prod", resource),
    )
    .unwrap();
    assert_eq!(plain.source, QuotaSource::ClassDefault);
    assert_eq!(plain.effective, 500_000);

    // Layer 2: the pack override tightens the class default.
    let packed = resolve_effective_quota(
        &catalog,
        &classes,
        &InMemoryQuotaOverrideRepository::new(),
        &QuotaKey::new("ten_promo", resource),
    )
    .unwrap();
    assert_eq!(packed.source, QuotaSource::PackOverride);
    assert_eq!(packed.effective, 300_000);

    // Layer 3: the tenant override displaces an undefended pack override.
    let overridden = resolve_effective_quota(
        &catalog,
        &classes,
        &InMemoryQuotaOverrideRepository::new().with_override("ten_promo", resource, 400_000),
        &QuotaKey::new("ten_promo", resource),
    )
    .unwrap();
    assert_eq!(overridden.source, QuotaSource::TenantOverride);
    assert_eq!(overridden.effective, 400_000);

    // Layer 4: the hard cap clamps the tenant override and owns the answer.
    let capped_catalog = catalog.clone().with_hard_cap(resource, 350_000);
    let capped = resolve_effective_quota(
        &capped_catalog,
        &classes,
        &InMemoryQuotaOverrideRepository::new().with_override("ten_promo", resource, 400_000),
        &QuotaKey::new("ten_promo", resource),
    )
    .unwrap();
    assert_eq!(capped.source, QuotaSource::HardCap);
    assert_eq!(capped.limit, 400_000, "the declared number survives");
    assert_eq!(capped.effective, 350_000, "the enforced number is clamped");
}

#[test]
fn a_regulated_pack_cannot_be_bought_past_by_a_tenant_override() {
    // Every resource us-hc regulates, not just the one that happened to have
    // a pack hard cap: the earlier version of this test asserted the
    // invariant for SeatCount only, while CapabilityInvocationsPerDay — the
    // one resource the pack actually declares a ceiling on — was outbiddable.
    let catalog = QuotaPolicyCatalog::platform_defaults();
    let classes = classes();
    for (resource, ceiling, bid) in [
        (QuotaResource::SeatCount, 250_u64, 4_000_u64),
        (QuotaResource::ApiCallsPerDay, 1_000_000, 9_000_000),
        (
            QuotaResource::CapabilityInvocationsPerDay,
            50_000,
            5_000_000,
        ),
    ] {
        let overrides =
            InMemoryQuotaOverrideRepository::new().with_override("ten_hc", resource, bid);
        let decision = resolve_effective_quota(
            &catalog,
            &classes,
            &overrides,
            &QuotaKey::new("ten_hc", resource),
        )
        .unwrap();
        assert_eq!(decision.source, QuotaSource::HardCap, "{resource}");
        assert_eq!(decision.limit, bid, "{resource}: the bid is still recorded");
        assert_eq!(decision.effective, ceiling, "{resource} was bought past");
        assert_eq!(decision.pack.as_deref(), Some(US_HC_PACK));
    }
}

#[test]
fn a_shouted_pack_spelling_binds_the_same_regulated_ceiling() {
    // `Tenant.jurisdiction_code` spells it `US-HC`. Matching pack identifiers
    // byte-exactly would make that spelling silently disable the pack while
    // the decision still claimed the pack had been applied.
    let catalog = QuotaPolicyCatalog::platform_defaults();
    let classes = classes();
    let overrides = InMemoryQuotaOverrideRepository::new()
        .with_override("ten_hc", QuotaResource::ApiCallsPerDay, 9_000_000)
        .with_override("ten_hc_shouty", QuotaResource::ApiCallsPerDay, 9_000_000);
    let key = |tenant| QuotaKey::new(tenant, QuotaResource::ApiCallsPerDay);

    let canonical =
        resolve_effective_quota(&catalog, &classes, &overrides, &key("ten_hc")).unwrap();
    let shouty =
        resolve_effective_quota(&catalog, &classes, &overrides, &key("ten_hc_shouty")).unwrap();

    assert_eq!(shouty.effective, 1_000_000);
    assert_eq!(shouty.source, QuotaSource::HardCap);
    assert_eq!(
        shouty.pack.as_deref(),
        Some(US_HC_PACK),
        "the decision reports the canonical spelling, not the input one"
    );
    assert_eq!(canonical.effective, shouty.effective);
}

#[test]
fn a_tenant_bound_to_an_undeclared_pack_fails_closed() {
    // `pack-us-hc` is a third spelling from IP-009 that this catalog does not
    // declare. Resolving it anyway would drop the regulated ceiling entirely
    // and report `pack: Some(...)` as if it had been applied.
    let catalog = QuotaPolicyCatalog::platform_defaults();
    let classes = classes();
    let overrides = InMemoryQuotaOverrideRepository::new().with_override(
        "ten_typo",
        QuotaResource::ApiCallsPerDay,
        9_000_000,
    );
    assert_eq!(
        resolve_effective_quota(
            &catalog,
            &classes,
            &overrides,
            &QuotaKey::new("ten_typo", QuotaResource::ApiCallsPerDay),
        )
        .unwrap_err(),
        QuotaUsecaseError::UnknownPack {
            pack: "pack-us-hc".to_owned()
        }
    );
    assert!(resolve_quota_sheet(&catalog, &classes, &overrides, "ten_typo").is_err());
}

#[test]
fn binding_a_regulated_pack_never_raises_a_sandbox_tenants_ceiling() {
    // IP-022 §A exists because "sandbox tenants can exceed shared substrate
    // limits". A pack layer that *replaced* the class default would take this
    // tenant from 200 to the pack's 50_000.
    let catalog = QuotaPolicyCatalog::platform_defaults();
    let classes = classes();
    let overrides = InMemoryQuotaOverrideRepository::new();
    let resource = QuotaResource::CapabilityInvocationsPerDay;

    let plain = resolve_effective_quota(
        &catalog,
        &classes,
        &overrides,
        &QuotaKey::new("ten_sandbox", resource),
    )
    .unwrap();
    let packed = resolve_effective_quota(
        &catalog,
        &classes,
        &overrides,
        &QuotaKey::new("ten_hc_sandbox", resource),
    )
    .unwrap();

    assert_eq!(plain.effective, 200);
    assert_eq!(packed.effective, 200, "the pack escalated a sandbox tenant");
    assert_eq!(packed.source, QuotaSource::ClassDefault);

    let sheet = resolve_quota_sheet(&catalog, &classes, &overrides, "ten_hc_sandbox").unwrap();
    let plain_sheet = resolve_quota_sheet(&catalog, &classes, &overrides, "ten_sandbox").unwrap();
    for (packed, plain) in sheet.iter().zip(&plain_sheet) {
        assert!(
            packed.effective <= plain.effective,
            "{}: pack raised {} to {}",
            packed.resource,
            plain.effective,
            packed.effective
        );
    }
}

#[test]
fn enforcement_admits_up_to_the_ceiling_then_refuses_and_recovers_on_release() {
    let catalog = QuotaPolicyCatalog::platform_defaults();
    let classes = classes();
    let overrides = InMemoryQuotaOverrideRepository::new();
    let mut ledger = open_ledger(
        &catalog,
        &classes,
        &overrides,
        &QuotaKey::new("ten_sandbox", QuotaResource::SeatCount),
        0,
    )
    .unwrap();
    assert_eq!(ledger.limit(), 3);

    assert!(ledger.reserve(3).unwrap().is_admitted());
    assert_eq!(
        ledger.reserve(1).unwrap(),
        QuotaOutcome::RefusedHardLimit {
            requested: 1,
            available: 0
        }
    );

    ledger.release(1).unwrap();
    assert!(ledger.reserve(1).unwrap().is_admitted());
    assert_eq!(ledger.used(), 3);
}

#[test]
fn soft_and_hard_thresholds_are_exact_at_both_boundaries() {
    let catalog = QuotaPolicyCatalog::new().with_class_default(
        "bespoke",
        QuotaResource::ApiCallsPerDay,
        QuotaAllowance::new(10, 50, ResetWindow::Seconds(86_400)).unwrap(),
    );
    let classes = InMemoryTenantClassReader::new().with_tenant("ten_x", "bespoke");
    let overrides = InMemoryQuotaOverrideRepository::new();
    let key = QuotaKey::new("ten_x", QuotaResource::ApiCallsPerDay);

    let decision = resolve_effective_quota(&catalog, &classes, &overrides, &key).unwrap();
    assert_eq!(decision.effective, 10);
    assert_eq!(decision.soft_threshold, 5);

    let mut ledger = open_ledger(&catalog, &classes, &overrides, &key, 0).unwrap();
    // At the soft threshold: quiet.
    assert!(!ledger.reserve(5).unwrap().warns());
    // One above it: warns, still admitted.
    let crossed = ledger.reserve(1).unwrap();
    assert!(crossed.warns());
    assert!(crossed.is_admitted());
    // Up to the hard limit exactly: admitted.
    assert!(ledger.reserve(4).unwrap().is_admitted());
    // One beyond it: refused.
    assert!(!ledger.reserve(1).unwrap().is_admitted());
}

#[test]
fn a_window_reset_forgives_settled_use_but_not_in_flight_reservations() {
    let catalog = QuotaPolicyCatalog::platform_defaults();
    let classes = classes();
    let overrides = InMemoryQuotaOverrideRepository::new();
    let mut ledger = open_ledger(
        &catalog,
        &classes,
        &overrides,
        &QuotaKey::new("ten_sandbox", QuotaResource::RequestRatePerMinute),
        1_700_000_000,
    )
    .unwrap();
    assert_eq!(ledger.limit(), 30);

    ledger.reserve(28).unwrap();
    ledger.commit(25).unwrap();
    assert!(!ledger.reserve(5).unwrap().is_admitted());

    assert!(!ledger.advance_to(1_700_000_059).unwrap());
    assert!(ledger.advance_to(1_700_000_060).unwrap());
    assert_eq!(ledger.committed(), 0);
    assert_eq!(ledger.reserved(), 3);
    assert!(ledger.reserve(5).unwrap().is_admitted());
}

#[test]
fn a_storage_quota_never_resets_because_stored_bytes_are_a_stock() {
    let catalog = QuotaPolicyCatalog::platform_defaults();
    let classes = classes();
    let overrides = InMemoryQuotaOverrideRepository::new();
    let mut ledger = open_ledger(
        &catalog,
        &classes,
        &overrides,
        &QuotaKey::new("ten_trial", QuotaResource::StorageBytes),
        0,
    )
    .unwrap();
    ledger.reserve(1_000).unwrap();
    ledger.commit(1_000).unwrap();
    assert!(!ledger.advance_to(u64::MAX).unwrap());
    assert_eq!(ledger.committed(), 1_000);
}

#[test]
fn releasing_more_than_is_reserved_is_refused_across_the_public_surface() {
    let catalog = QuotaPolicyCatalog::platform_defaults();
    let classes = classes();
    let overrides = InMemoryQuotaOverrideRepository::new();
    let mut ledger = open_ledger(
        &catalog,
        &classes,
        &overrides,
        &QuotaKey::new("ten_prod", QuotaResource::SeatCount),
        0,
    )
    .unwrap();
    ledger.reserve(2).unwrap();
    assert_eq!(
        ledger.release(3).unwrap_err(),
        QuotaUsageError::ReleaseWithoutReservation {
            requested: 3,
            reserved: 2
        }
    );
    assert_eq!(ledger.reserved(), 2);
    assert_eq!(ledger.remaining(), 498);
}

#[test]
fn a_quota_sheet_is_all_or_nothing_when_a_class_has_no_policy() {
    let catalog = QuotaPolicyCatalog::new().with_class_default(
        "partial",
        QuotaResource::SeatCount,
        QuotaAllowance::standard(10, QuotaResource::SeatCount),
    );
    let classes = InMemoryTenantClassReader::new().with_tenant("ten_x", "partial");
    let overrides = InMemoryQuotaOverrideRepository::new();
    assert!(resolve_quota_sheet(&catalog, &classes, &overrides, "ten_x").is_err());
}

#[test]
fn resolution_is_deterministic_across_repeated_calls() {
    let catalog = QuotaPolicyCatalog::platform_defaults();
    let classes = classes();
    let overrides = InMemoryQuotaOverrideRepository::new().with_override(
        "ten_hc",
        QuotaResource::ApiCallsPerDay,
        9_000_000,
    );
    let first = resolve_quota_sheet(&catalog, &classes, &overrides, "ten_hc").unwrap();
    let second = resolve_quota_sheet(&catalog, &classes, &overrides, "ten_hc").unwrap();
    assert_eq!(first, second);
}
