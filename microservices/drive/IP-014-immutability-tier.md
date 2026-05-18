---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-drive-foundation
impl_plan_id: IP-014-immutability-tier
status: pending
execution_unit: ChangeSet
owner: axis-drive + compliance + ops-security
acceptance_lanes: [cargo-build, cargo-nextest, oya-governance-worm-enforcement-multi-layer, oya-governance-worm-retention-monotonic]
---

# IP-014: immutability-tier BC — WORM compliance-mode + legal hold + 2-person rule + periodic integrity scan

## Intent

Stand up `oya-drive-immutability-tier-*` BC per ADR-DRIVE-0006. Defence-in-depth WORM enforcement across application + DB + object-store layers; legal-hold cascade; 2-person rule on any release path; hourly integrity scan.

## Crates

`oya-drive-immutability-tier-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,app}` (8 crates).

## Acceptance Gates

```bash
cargo nextest run --test worm_refuses_purge
cargo nextest run --test worm_refuses_tenant_root
cargo nextest run --test worm_retention_monotonic
cargo nextest run --test legal_hold_preserves
cargo run -p oya-dev-cli -- gate validate worm-integrity-scan --microservice drive
cargo run -p oya-dev-cli -- gate validate worm-enforcement-multi-layer --microservice drive
```

## References

- ADR-DRIVE-0006.
- PRD-drive §FR-12; §FR-21; AC-09; AC-10; AC-14.
- SEC 17a-4(f); FINRA Rule 4511; HIPAA §164.316.
