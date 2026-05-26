---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M03-P03-IP-001
title: Cloud Data services kernel + adapters
status: complete (provider-neutral-kernel-green; provider runtime adapters not claimed)
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Ship managed Postgres/Citus/pgvector/Valkey/Kafka/ClickHouse via provider-agnostic data kernel.
---

# M03-P03-IP-001 — Cloud Data services kernel + adapters

## Purpose
Ship managed Postgres/Citus/pgvector/Valkey/Kafka/ClickHouse via provider-agnostic data kernel.

## Symbols-to-grit-claim
```
crates/oya-cloud-data-kernel/src/lib.rs::DataService
crates/oya-cloud-data-kernel/src/lib.rs::DatabaseEngine
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
Phase INDEX read; parent milestone INDEX read; MASTERPLAN §2 principles understood; M01-P08 ≥ P5 merged.

## Acceptance-test-commands
```
cargo test -p oya-cloud-data-kernel --all-features
cargo test -p oya-cloud-data-domain --all-features
cargo run -q -p oya-dev-cli -- gate validate cohesion
oya verify --ci-required
```

Legacy scaffold note: the generated `cargo run -p oya-governance-cohesion -- <owning-crate-glob>` and `scripts/check.sh` commands do not map to current workspace packages/files; current canonical equivalents are `oya-dev-cli -- gate validate cohesion` and `oya verify --ci-required`.

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
icm store -t context-oyatie -c 'M03-P03-IP-001 Cloud Data services kernel + adapters shipped; acceptance commands green' -i high -k 'M03-P03-IP-001,complete'
```

## Progress
- 2026-05-21: Verified `DataService`, `DatabaseEngine`, and `provision_data_service` are implemented and exported by `oya-cloud-data-kernel` with provider-neutral, I/O-free admission checks for Postgres, Citus, pgvector, Valkey, Kafka, and ClickHouse engine families.
- 2026-05-21: Targeted cloud data kernel tests pass 21/21 and cloud data domain tests pass 8/8. This IP does not claim deployed provider database/cache/stream adapters, live managed data service provisioning, or app/API transport wiring.

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP: Postgres, Citus, pgvector, Valkey, Kafka, and ClickHouse share one provider-neutral `DatabaseEngine`/`DataServiceKind` mapping and one `DataService` admission path instead of branching into provider-specific data-service models.
Rejected: adding provider SDK calls or provisioning side effects to the data kernel; adapters must own provider-specific database/cache/stream runtime behavior.
Rejected: claiming deployed runtime readiness from pure kernel/domain tests; live provider provisioning smoke and app/API wiring remain follow-up slices.
