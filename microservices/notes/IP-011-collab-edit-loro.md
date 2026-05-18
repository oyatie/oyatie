---
doc_class: ImplementationPlan
impl_plan_id: IP-011-collab-edit-loro
milestone: M02-foundation
phase: P01-notes-foundation
status: pending
owner: axis-notes
acceptance_lanes: [cargo-check, cargo-test, oya-governance-dual-context-isolation, loro-version-pin]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

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

## ChangeSet metadata

```yaml
changeset_id: CS-NOTES-IP-011-collab-edit-loro
depends_on_changesets: [CS-NOTES-IP-003-note-store-kernel-domain, CS-NOTES-IP-008-share-link-and-embed, CS-NOTES-IP-010-search-and-graph-view]
parallel_safe_with_changesets: [CS-NOTES-IP-012-import-export-pipelines]
enables: []
acceptance_status: ga
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | E2E (Personal-tier) note refused at the type system from entering collab-edit | `cargo nextest run -p oya-notes-collab-edit-kernel -- personal_tier_refused` |
| AC-02 | Two-client concurrent edit converges to identical state matching Loro 1.x reference impl | `cargo test --test loro-collab-convergence` |
| AC-03 | Session-affinity hashing by `(tenant_id, note_id)` deterministic | `cargo nextest run -p oya-notes-collab-edit-domain -- session_affinity` |
| AC-04 | Op-log compaction at 1h idle emits snapshot + truncates log | `cargo nextest run -p oya-notes-collab-edit-worker -- compaction_idle_1h` |
| AC-05 | `oya gate validate version-pinning-conformance` exits 0 (Loro 1.x LTS) | governance lane |

## Build Sequence

1. Kernel: `CollabSession`, `OpLog`, `Snapshot` ports.
2. Domain: `Op`, `Version`, `SessionAffinity`.
3. Adapter: `-adapter-loro` pinned to Loro 1.x LTS (ADR-NOTES-0003).
4. Worker: idle compaction at 1h; snapshot to Postgres `loro_op` table.
5. `cargo test --test loro-collab-convergence`.
6. `cargo run -p oya-dev-cli -- gate validate dual-context-isolation --microservice notes`.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-notes FR | FR-17 (collab-edit), FR-19 (dual-context) |
| PRD-notes AC | AC-15 (convergence) |
| ADR | ADR-NOTES-0003 (Loro), ADR-WS-0001 (Loro alignment across products) |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| Personal-tier note silently routed into collab-edit | Compile-time refusal; type-system invariant |
| Op-log unbounded growth | Idle compaction + monthly snapshot retention |
| Loro version skew across clients | LTS pin + `version-pinning-conformance` gate |

## References

- Loro CRDT documentation (`loro.dev/docs`).
- "A comprehensive study of Convergent and Commutative Replicated Data Types" — Shapiro et al. (INRIA RR-7506, 2011).
- Yjs CRDT reference (yjs.dev) — comparative.
- ADR-NOTES-0003 (Loro), ADR-WS-0001 (Loro alignment).

## Next IP

[`IP-012-import-export-pipelines.md`](IP-012-import-export-pipelines.md)
