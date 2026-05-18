---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-sites-foundation
impl_plan_id: IP-002-site-bc-kernel
status: pending
execution_unit: ChangeSet
owner: axis-sites
acceptance_lanes: [cargo-build, cargo-nextest, oya-governance-layer-correctness, oya-governance-port-location]
---

# IP-002: site BC — kernel + domain + usecase + api

## Intent

Author the `site` bounded-context's Layer-B crates (kernel + domain + usecase + api). Defines `Site`, `SiteVisibility`, `SiteOwner` entities; port traits `SiteRepository` + `RetentionPolicyResolver` + `LegalHoldStore`; usecases `create_site`, `update_site`, `publish_site`, `unpublish_site`, `delete_site`, `apply_legal_hold`, `release_legal_hold`. Zero I/O at kernel/domain; usecases consume via ports.

## ChangeSet boundary

4 new crates: `oya-sites-site-{kernel,domain,usecase,api}`. ~3000 LOC Rust. Workspace `Cargo.toml` updates + 4 new `Cargo.toml`s. Unit tests in each crate. Per-tenant scoping invariants test corpus.

## Crate Naming

`oya-sites-site-{kernel,domain,usecase,api}` per PRD §"Bounded Contexts" naming justification.

## Acceptance Gates

```bash
cargo build -p oya-sites-site-kernel -p oya-sites-site-domain -p oya-sites-site-usecase -p oya-sites-site-api
cargo nextest run -p oya-sites-site-kernel -p oya-sites-site-domain -p oya-sites-site-usecase
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice sites
cargo run -p oya-dev-cli -- gate validate port-location --microservice sites
```

## Test Plan

- Unit: `Site` entity validation; visibility transitions; ownership invariants.
- Unit: port-trait contracts mocked.
- Unit: tenant-scoping invariant (Tenant-A's repo never returns Tenant-B's site).
- Unit: legal-hold transitions cannot bypass 2-person-rule.

## Halt Conditions

- LEAN layer-correctness refuses — root-cause; do not mask.
- Port-location lane refuses — port traits must live in kernel only.

## Next IP

[`IP-003-page-bc-kernel.md`](IP-003-page-bc-kernel.md)

## References

- ADR-0105 (13-layer enum); ADR-0106 (usecase rename); ADR-0131; ADR-0140.
- PRD §"Bounded Contexts".
- Bominal ADR-0028 (audit-chain).
