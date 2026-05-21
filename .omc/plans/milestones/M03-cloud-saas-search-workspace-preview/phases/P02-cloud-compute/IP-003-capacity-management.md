---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M03-P02-IP-003
title: Capacity management (reserved / committed-use / spot)
status: complete (provider-neutral-kernel-green; provider procurement/runtime adapters not claimed)
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Bring reserved + committed-use + spot capacity management to stable.
---

# M03-P02-IP-003 — Capacity management (reserved / committed-use / spot)

## Purpose
Bring reserved + committed-use + spot capacity management to stable.

## Symbols-to-grit-claim
```
crates/oya-cloud-capacity-kernel/src/lib.rs::ReservedCapacity
crates/oya-cloud-capacity-kernel/src/lib.rs::CommittedUseContract
crates/oya-cloud-capacity-kernel/src/lib.rs::SpotPool
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
Phase INDEX read; parent milestone INDEX read; MASTERPLAN §2 principles understood; M01-P08 ≥ P5 merged.

## Acceptance-test-commands
```
cargo test -p oya-cloud-capacity-kernel --all-features
cargo run -q -p oya-dev-cli -- gate validate cohesion
oya verify --ci-required
```

Legacy scaffold note: the generated `cargo run -p oya-foundry-fitness-cohesion -- <owning-crate-glob>` and `scripts/check.sh` commands do not map to current workspace packages/files; current canonical equivalents are `oya-dev-cli -- gate validate cohesion` and `oya verify --ci-required`.

## Done-criteria
- All acceptance-test commands return 0.
- Distroless image built (if IP ships a deployed binary); size < per-binary budget per `docs/standards/image-size-budgets.md`.
- No provider-specific deps outside adapter crates (Directive 4).
- All direct deps current LTS or have ADR-tracked exception (Directive 8).
- PR "good-taste audit" section non-empty (Directive 7).
- Distroless + provider-coupling + LTS-dependency lanes green on PR.

## Rollback-procedure
`grit done` is atomic per-symbol; if a subsequent IP regresses, revert the merge commit. For crates that other IPs depend on, follow per-crate split unwind per ADR-0015 §7.

## Next-IP-pointer
Next IP in this phase's INDEX list (or first IP of next phase if phase complete).

## Icm-store-payload
```
icm store -t context-oyatie -c 'M03-P02-IP-003 Capacity management (reserved / committed-use / spot) shipped; acceptance commands green' -i high -k 'M03-P02-IP-003,complete'
```

## Progress
- 2026-05-21: Verified `ReservedCapacity`, `CommittedUseContract`, and `SpotPool` are implemented and exported by `oya-cloud-capacity-kernel` with provider-neutral, I/O-free invariants for reserved capacity, committed-use discounts, and spot/preemptible pool admission.
- 2026-05-21: Targeted capacity-kernel tests pass 42/42; cohesion/fmt/clippy are green. This IP does not claim deployed capacity procurement APIs, provider discount purchase flows, or live spot/preemptible provider smoke.

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP: reserved, committed-use, and spot capacity share the same provider-neutral `CapacityClass`/`RegionId` kernel vocabulary instead of branching into provider-specific capacity models.
Rejected: adding provider SDK calls or procurement side effects to the capacity kernel; adapters must own provider-specific purchase/interruption APIs.
Rejected: claiming deployed runtime readiness from pure kernel tests; live provider spot/preemptible smoke and app/API wiring remain follow-up slices.
