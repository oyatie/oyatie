---
doc_class: ImplementationPlan
impl_plan_id: IP-011-collab-edit-loro
milestone: M02-foundation
phase: P01-notes-foundation
status: pending
owner: axis-notes
acceptance_lanes: [cargo-check, cargo-test, oya-governance-dual-context-isolation, loro-version-pin]
---

# IP-011: collab-edit via Loro 1.x LTS (Professional-tier only)

## Intent

Land `oya-notes-collab-edit-{kernel,domain,usecase,api,adapter,adapter-loro,worker,sdk,app}`. Per ADR-NOTES-0003: Loro 1.x LTS; Professional-tier only; E2E-tier (Personal) refused at type system.

## Broker

- Per-(tenant_id, note_id) session-affinity hashing.
- HPA min 3 max 30.
- Op-log persisted to Postgres `loro_op` table (TimescaleDB-style partition).
- Compaction at 1h idle: snapshot + truncate op-log.

## Convergence Conformance Test

`tests/e2e/loro-collab-convergence.rs` (AC-15): two-client concurrent edit; verifies converged state matches Loro 1.x reference implementation.

## Acceptance Gates

```bash
cargo check -p oya-notes-collab-edit-kernel
cargo check -p oya-notes-collab-edit-adapter-loro
cargo test --test loro-collab-convergence
cargo run -p oya-dev-cli -- gate validate dual-context-isolation --microservice notes
cargo run -p oya-dev-cli -- gate validate version-pinning-conformance
```

## Next IP

[`IP-012-import-export-pipelines.md`](IP-012-import-export-pipelines.md)
