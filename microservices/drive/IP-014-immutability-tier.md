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

## ChangeSet metadata

```yaml
changeset_id: CS-DRIVE-IP-014-immutability-tier
depends_on_changesets: [CS-DRIVE-IP-003-file-store-kernel-domain, CS-DRIVE-IP-010-permissions]
parallel_safe_with_changesets: [CS-DRIVE-IP-011-search-index]
enables: []
acceptance_status: ga
load_bearing: true
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Object under WORM refuses purge by any principal including tenant-root | `cargo nextest run --test worm_refuses_purge` |
| AC-02 | Tenant-root cannot release WORM ahead of retention floor | `cargo nextest run --test worm_refuses_tenant_root` |
| AC-03 | Retention is monotonic — cannot be shortened after applied | `cargo nextest run --test worm_retention_monotonic` |
| AC-04 | Legal-hold preserves object + versions past retention expiry | `cargo nextest run --test legal_hold_preserves` |
| AC-05 | Hourly integrity scan detects + alerts any WORM-bypass attempt | `cargo run -p oya-dev-cli -- gate validate worm-integrity-scan --microservice drive` |
| AC-06 | Multi-layer enforcement gate green (app + DB + object-store) | `cargo run -p oya-dev-cli -- gate validate worm-enforcement-multi-layer --microservice drive` |

## Build Sequence

1. Kernel: `ImmutabilityTier`, `RetentionPolicy`, `LegalHold`, `IntegrityScanner` ports.
2. Domain: `WormObject`, `RetentionBound`, `Hold`, `ScanVerdict`.
3. Postgres adapter with `BEFORE DELETE/UPDATE` triggers enforcing WORM.
4. S3 Object Lock compliance-mode binding at storage layer (defence-in-depth).
5. Worker that runs hourly integrity scan; emits audit-chain record.
6. `cargo nextest run -p oya-drive-immutability-tier-*`.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-drive FR | FR-12 (WORM), FR-21 (legal hold) |
| PRD-drive AC | AC-09, AC-10, AC-14 |
| ADR | ADR-DRIVE-0006 |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| Application-layer bug bypasses WORM | Defence-in-depth: DB trigger + S3 Object Lock compliance-mode block at storage |
| Clock-skew shortens retention | Retention bound never decreases; monotonic invariant test |
| Legal-hold orphaned after compliance-officer departure | Per-tenant hold registry; mandatory hand-off ceremony |

## References

- ADR-DRIVE-0006.
- PRD-drive §FR-12; §FR-21; AC-09; AC-10; AC-14.
- SEC 17a-4(f) (Records to be made by certain exchange members, brokers and dealers).
- FINRA Rule 4511 (General Requirements).
- HIPAA §164.316 (Policies and procedures and documentation requirements).
- AWS S3 Object Lock — Compliance mode documentation.
