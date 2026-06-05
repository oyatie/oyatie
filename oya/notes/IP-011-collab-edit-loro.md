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
buck2 build //:quality-lane-registry-authority-check # lane=dual-context-isolation --microservice notes
buck2 build //:quality-lane-registry-authority-check # lane=version-pinning-conformance
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
6. `buck2 build //:quality-lane-registry-authority-check # lane=dual-context-isolation --microservice notes`.

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


## A. Problem
`IP-011: collab-edit via Loro 1.x LTS (Professional-tier only)` is not a generic implementation packet; it closes the `011 collab edit loro` gap for `notes` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: Note, PersonalNoteRef, ProfessionalNoteRef, tag-graph, backlink graph, Loro CRDT, MLS key package, share-link, E2E refusal.

## B. Approach
Personal-tier plaintext exclusion is enforced before collaboration or AI: MLS/OpenMLS key material stays client-side, Loro CRDT is Professional-only unless an encrypted client flow exists. The implementation must keep the µservice boundary intact: contracts remain under `microservices/notes/contracts/openapi/notes.yaml` / `microservices/notes/contracts/proto/notes.proto`, policy decisions remain in `microservices/notes/policy/tenant-scope.cedar`, operational proof remains in `microservices/notes/slos/note-open-latency.openslo.yaml`, and the parity claim is checked against `microservices/notes/competitor-parity-matrix.md`.

## C. Deliverables
- `microservices/notes/PRD.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/ARCHITECTURE.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/contracts/openapi/notes.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/contracts/proto/notes.proto` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/contracts/asyncapi/notes-events.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/policy/tenant-scope.cedar` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/slos/note-open-latency.openslo.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/runbooks/sync-conflict-resolution.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/catalog/oya-notes-note-store-kernel.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/competitor-parity-matrix.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/policy/e2e-personal-tier-default.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/catalog/oya-notes-e2e-key-management-adapter-mls.yaml` — verify/update as the authoritative artifact for this IP.
- Named code targets declared by this IP and `manifest.json` must be created only when the implementation PR actually adds the crates/types; this scrub does not pretend source files exist.

## D. Implementation Steps
1. Read `microservices/notes/PRD.md` and `microservices/notes/ARCHITECTURE.md` to confirm the bounded context, tenant class, and first-ship milestone for `notes`.
2. Diff the declared contract in `microservices/notes/contracts/openapi/notes.yaml` and `microservices/notes/contracts/proto/notes.proto` against the IP title so every endpoint/message has a matching domain type or explicit backlog gap.
3. Check `microservices/notes/policy/tenant-scope.cedar` plus adjacent Cedar/policy files before adding any mutation, share, webhook, agent, AI, or cross-tenant path.
4. Wire observability to `microservices/notes/slos/note-open-latency.openslo.yaml` and the relevant dashboard/runbook; no acceptance claim counts without a metric or sealed evidence path.
5. Update the catalog/capability record such as `microservices/notes/catalog/oya-notes-note-store-kernel.yaml` so the service registry can discover the new boundary.
6. Run the IP-specific test/gate commands listed above; if a source crate is absent, record the absent crate as implementation debt rather than faking a green result.

## E. Acceptance
- Local artifact links resolve for `microservices/notes/PRD.md`, `microservices/notes/ARCHITECTURE.md`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/policy/tenant-scope.cedar`, `microservices/notes/slos/note-open-latency.openslo.yaml`, and `microservices/notes/competitor-parity-matrix.md`.
- The implementation exposes no cross-tenant, cross-pack, credential, E2E, or vendor-call path without the policy file cited in this IP.
- At least one targeted unit/contract/gate command verifies the named behavior, and any skipped command is documented with the missing artifact.
- The final PR includes evidence that counterpart parity is improved or explicitly marks the remaining gap.

## F. Evidence
- `microservices/notes/PRD.md`
- `microservices/notes/ARCHITECTURE.md`
- `microservices/notes/contracts/openapi/notes.yaml`
- `microservices/notes/contracts/proto/notes.proto`
- `microservices/notes/contracts/asyncapi/notes-events.yaml`
- `microservices/notes/policy/tenant-scope.cedar`
- `microservices/notes/slos/note-open-latency.openslo.yaml`
- `microservices/notes/runbooks/sync-conflict-resolution.md`
- `microservices/notes/catalog/oya-notes-note-store-kernel.yaml`
- `microservices/notes/competitor-parity-matrix.md`
- `microservices/notes/competitor-parity-matrix.md` — counterpart gap table used for the comparison below.

## G. Counterparts
| Counterpart pressure | Oyatie closure for this IP |
|---|---|
| Apple Notes, Google Keep, OneNote, Notion, Bear, Obsidian, Standard Notes, Evernote, Roam, Logseq, Joplin, Reflect, Tana, Mem, and Heptabase | Notion and OneNote define workspace/collab parity; Obsidian/Roam/Logseq define backlink and graph parity; Standard Notes and Apple Notes define privacy pressure; Evernote/Bear/Google Keep define capture/import expectations. This IP closes the relevant gap by binding `011 collab edit loro` to concrete `notes` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
