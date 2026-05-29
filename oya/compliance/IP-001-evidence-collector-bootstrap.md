---
microservice: compliance
ip: IP-001
title: Evidence collector bootstrap (kernel + trait + in-memory test impl)
status: Drafting
authority_tier: 3
owner: axis-compliance
date: 2026-05-18
related_adrs: [ADR-0209, ADR-0145, ADR-0131]
---

# IP-001 — Evidence collector bootstrap

## Purpose

Land the kernel-level coverage matrix (`oya-shared-compliance-evidence-kernel::required_artifacts_for`) into the µservice's domain + use-case + REST API, plus a trait-driven collector abstraction with one in-memory test implementation per artifact kind. This IP is the foundation for IP-002..IP-015.

## Acceptance criteria

1. `oya-compliance-domain` crate compiles and exposes:
   - `EvidenceLedger` (append-only artifact store).
   - `CollectorOrchestrator` (trait-object collector registry + dispatch).
   - `CoverageReporter` (per-tenant × per-framework coverage rollup).
2. `oya-compliance-usecase` crate compiles and exposes:
   - `RegisterCollectorUseCase`, `EmitArtifactUseCase`, `ReportCoverageUseCase`.
3. In-memory test implementations of all 9 artifact-kind collectors registered under `tests/in_memory_collectors.rs`.
4. Integration test `tests/coverage_end_to_end.rs` populates two tenants, runs each collector once, asserts `coverage_gaps` returns empty for the active framework set.
5. Cross-tenant isolation invariant covered: a tenant's collector cannot stamp another tenant's `tenant_id` (kernel-level reject).
6. ≥ 10 unit tests across domain + usecase crates.

## Architecture sketch

```
oya-compliance-domain/
  src/
    lib.rs                  // re-export
    evidence_ledger.rs      // append-only store
    collector_registry.rs   // trait-object registry
    coverage_reporter.rs    // rollup over EvidenceLedger
oya-compliance-usecase/
  src/
    lib.rs
    register_collector.rs
    emit_artifact.rs
    report_coverage.rs
```

## Implementation steps

1. **Domain — `EvidenceLedger`.** Append-only `Vec<EvidenceArtifact>` (kernel type from `oya-shared-compliance-evidence-kernel`). Insertions are `Result`-returning; reject duplicate `artifact_id` for same tenant.
2. **Domain — `CollectorRegistry`.** Holds `Vec<Box<dyn EvidenceCollector>>`; lookup by `EvidenceArtifactKind`.
3. **Domain — `CoverageReporter`.** Wraps kernel `coverage_gaps`; adds per-framework aggregation; per-tenant + per-microservice rollup.
4. **Use-case — `RegisterCollectorUseCase`.** Idempotent registration; rejects duplicate (kind, microservice) pair.
5. **Use-case — `EmitArtifactUseCase`.** Calls `EvidenceCollector::collect` → seal-validate → ledger-insert → emit `EVT-COMPLIANCE-ARTIFACT-EMITTED` (downstream wires via outbox).
6. **Use-case — `ReportCoverageUseCase`.** Walks ledger; calls reporter; returns coverage gaps.
7. **Tests.** Build collectors for each `EvidenceArtifactKind`; populate two tenants; assert zero cross-tenant leakage in coverage rollup.

## Cross-tenant isolation test

```rust
#[test]
fn cross_tenant_artifact_does_not_close_other_tenant_gap() {
    let mut ledger = EvidenceLedger::new();
    // tenant_a emits full SOC 2 set.
    for kind in required_artifacts_for(ComplianceFramework::Soc2TypeII) {
        ledger.append(artifact_for("tenant_a", kind)).unwrap();
    }
    // Coverage for tenant_b stays empty.
    let gaps = reporter.coverage_gaps("tenant_b", ComplianceFramework::Soc2TypeII).unwrap();
    assert_eq!(gaps.len(), required_artifacts_for(ComplianceFramework::Soc2TypeII).len());
}
```

## Risk + mitigation

- **Risk:** trait-object dispatch overhead on hot path. **Mitigation:** kernel keeps trait surface small (3 methods); benchmarking lane validates.
- **Risk:** ledger memory blowup at fleet scale. **Mitigation:** ledger backs onto SeaweedFS adapter in IP-006; in-memory test impl is for `cfg(test)`.

## Acceptance evidence

`evidence/ip-001-collector-bootstrap-acceptance.json` records:
- Tests run + outcome.
- Cross-tenant invariant test outcome.
- Kernel + domain + usecase crate build hash.

## Cross-references

- ADR-0209 — substrate authority.
- `oya-shared-compliance-evidence-kernel` — kernel.
- IP-005 — audit-chain seal coverage.
- IP-006 — SeaweedFS storage (replaces in-memory ledger).
