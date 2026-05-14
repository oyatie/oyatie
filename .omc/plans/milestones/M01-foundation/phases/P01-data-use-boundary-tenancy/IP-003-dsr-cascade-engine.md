---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P01-IP-003
title: dsr.cascade.execute ≤30d cascade engine
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Ship the DSR cascade engine that satisfies SPEC §2 DSR rows with proof-of-erasure per affected store.
evidence_ref: ../../../../../evidence/foundation/m01-p01-ip-003-dsr-cascade-engine.json
---

# M01-P01-IP-003 — dsr.cascade.execute ≤30d cascade engine

## Purpose
Ship the DSR cascade engine that satisfies SPEC §2 DSR rows with proof-of-erasure per affected store. The runtime surface is `dsr.cascade.execute`; it closes the higher-level `tenant.dsr.cascade` requirement through the current clean-architecture crates `oya-dsr-domain` and `oya-dsr-application`.

## Symbols-to-grit-claim
```
crates/oya-dsr-domain/src/lib.rs::DsrRequest
crates/oya-dsr-domain/src/lib.rs::DsrCompletionRecord
crates/oya-dsr-domain/src/lib.rs::ErasureProof
crates/oya-dsr-application/src/lib.rs::execute_dsr_cascade_from_api
crates/oya-dsr-application/tests/dsr_cascade_execute_api.rs::dsr_cascade_execute_requires_proof_of_erasure_per_affected_store
```
`grit claim` returned the known ADR-0054 scaffold FK failure for this mixed symbol/doc scope; fallback lock rows are `01KRKG9G0ZG4PMEY9P9J66GTFB` and `01KRKGF5VPTX1PJZE22F8M8BPM`.

## Agent-prerequisites
IP-002 tenant kernel merged.

## Acceptance-test-commands
```
cargo test -p oya-dsr-domain
cargo test -p oya-dsr-application --test dsr_cascade_execute_api
cargo clippy -p oya-dsr-domain -- -D warnings
cargo clippy -p oya-dsr-application -- -D warnings
```

## Done-criteria
- All acceptance-test commands return 0.
- Distroless image built (if IP ships a deployed binary); not applicable because this IP ships library/application boundary crates only.
- No provider-specific deps outside adapter crates (per MASTERPLAN §2 Directive 4).
- All direct deps current LTS or have ADR-tracked exception (Directive 8).
- PR "good-taste audit" section non-empty (Directive 7).

## Completion-evidence
- `DsrSlaTier::Preview` is locked to 30 days and rejects `30d + 1s`.
- `execute_dsr_cascade_from_api` proves multi-store cascades produce a proof id per affected store and reject completed acknowledgements missing proof fields.
- SPEC and machine-readable contract rows now point DSR runtime ownership at `oya-dsr-application` / `oya-dsr-domain` current crate names.
- Fresh evidence file: [`.omc/evidence/foundation/m01-p01-ip-003-dsr-cascade-engine.json`](../../../../../evidence/foundation/m01-p01-ip-003-dsr-cascade-engine.json).

## Rollback-procedure
`grit done` is atomic per-symbol; if a subsequent IP regresses, revert the merge commit. For crates that other IPs already depend on, follow the per-crate split unwind documented in ADR-0015 §7.

## Next-IP-pointer
M01-P02-IP-001 (identity kernel)

## Icm-store-payload
```
icm store -t context-oyatie -c 'DSR cascade ≤30d demonstrated; proof-of-erasure per affected store green; M01-P01 acceptance gate ready' -i critical -k 'M01,P01,IP-003,dsr-cascade,complete'
```

## Decision-log (Linus good-taste row)
DSR cascade is a single engine traversing the per-tenant ER graph — eliminates N per-axis DSR re-implementations.
