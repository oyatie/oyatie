---
doc_class: ImplementationPlan
impl_plan_id: IP-009-checklist-and-version-history
milestone: M02-foundation
phase: P01-notes-foundation
status: pending
owner: axis-notes
acceptance_lanes: [cargo-check, cargo-test, oya-governance-port-location]
---


# IP-009: checklist + version-history

## Intent

Land `oya-notes-checklist-*` (parses `- [ ]` items from note body; emits `ChecklistItemEmitted` to tasks µservice) + `oya-notes-version-history-*` (linear append-only version timeline; restore-to-version).

## Checklist Semantics

- Parse `- [ ]` (open) and `- [x]` (done).
- `@due(YYYY-MM-DD)` annotation extracts to `due_hint`.
- `@assignee(@user)` annotation extracts to `assignee_ref` (Ontology lookup).
- Idempotency: `item_id` derived from `sha256(note_id + line_position + text)`.

## Version History

- Linear (not branched per PRD §FR-12).
- Compacted at 90d for inactive notes (keep last 30 versions + monthly snapshots).
- Restore tx-locked with version-pointer-fence.

## Acceptance Gates

```bash
cargo check -p oya-notes-checklist-kernel
cargo check -p oya-notes-version-history-kernel
```

## ChangeSet metadata

```yaml
changeset_id: CS-NOTES-IP-009-checklist-and-version-history
depends_on_changesets: [CS-NOTES-IP-003-note-store-kernel-domain]
parallel_safe_with_changesets: [CS-NOTES-IP-008-share-link-and-embed, CS-NOTES-IP-010-search-and-graph-view]
enables: [CS-NOTES-IP-011-collab-edit-loro]
acceptance_status: ga
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | `- [ ]` parser correctly identifies open vs done items | `cargo nextest run -p oya-notes-checklist-domain -- parse_open_done` |
| AC-02 | `@due(YYYY-MM-DD)` extracts to `due_hint`; ISO 8601 conformance | `cargo nextest run -p oya-notes-checklist-domain -- due_iso8601` |
| AC-03 | `item_id = sha256(note_id + line_position + text)` deterministic across re-runs | `cargo nextest run -p oya-notes-checklist-domain -- item_id_determinism` |
| AC-04 | `ChecklistItemEmitted` event delivered to `tasks` µservice via Workflow event bus | `cargo nextest run --test e2e_checklist_to_tasks` |
| AC-05 | Version history linear (not branched); restore is tx-locked with version-pointer fence | `cargo nextest run -p oya-notes-version-history-domain -- linear_and_fenced` |
| AC-06 | Compaction at 90d idle keeps last 30 versions + monthly snapshots | `cargo nextest run -p oya-notes-version-history-domain -- compaction_90d` |

## Build Sequence

1. Kernel: `ChecklistParser`, `VersionStore`, `RestoreOrchestrator` ports.
2. Domain: `ChecklistItem`, `DueHint`, `AssigneeRef`, `Version`, `RestorePointer`.
3. Usecase: `EmitChecklistItem`, `RecordVersion`, `RestoreToVersion`, `CompactVersions`.
4. Cross-µservice event emission to `tasks`.
5. `cargo nextest run -p oya-notes-checklist-* -p oya-notes-version-history-*`.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-notes FR | FR-10 (checklist→tasks), FR-12 (version history), FR-23 (`ChecklistItemEmitted`) |
| PRD-notes NFR | NFR perf — note-open p95 ≤ 50ms warm |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| Duplicate `ChecklistItemEmitted` events on re-edit | Idempotency via `item_id` derived from sha256 |
| Restore loses concurrent edits | Version-pointer fence + tx lock |
| Compaction deletes a version still under legal hold | Compaction checks hold ledger first |

## References

- CommonMark task-list-item extension (`spec.commonmark.org/0.31.2/#task-list-items`).
- GitHub Flavored Markdown — task lists.
- ISO 8601 date format.
- Apple Notes checklist semantics (Apple Support).
- ADR-NOTES-0006 (note-to-task bridge).

## Next IP

[`IP-010-search-and-graph-view.md`](IP-010-search-and-graph-view.md)


## A. Problem
`IP-009: checklist + version-history` is not a generic implementation packet; it closes the `009 checklist and version history` gap for `notes` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: Note, PersonalNoteRef, ProfessionalNoteRef, tag-graph, backlink graph, Loro CRDT, MLS key package, share-link, E2E refusal.

## B. Approach
Note-store domain rules preserve immutable Personal/Professional context, retention, hold, and audit boundaries before higher-level note features attach. The implementation must keep the µservice boundary intact: contracts remain under `microservices/notes/contracts/openapi/notes.yaml` / `microservices/notes/contracts/proto/notes.proto`, policy decisions remain in `microservices/notes/policy/tenant-scope.cedar`, operational proof remains in `microservices/notes/slos/note-open-latency.openslo.yaml`, and the parity claim is checked against `microservices/notes/competitor-parity-matrix.md`.

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
- `microservices/notes/policy/dual-context-isolation.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/slos/note-create-latency.openslo.yaml` — verify/update as the authoritative artifact for this IP.
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
| Apple Notes, Google Keep, OneNote, Notion, Bear, Obsidian, Standard Notes, Evernote, Roam, Logseq, Joplin, Reflect, Tana, Mem, and Heptabase | Notion and OneNote define workspace/collab parity; Obsidian/Roam/Logseq define backlink and graph parity; Standard Notes and Apple Notes define privacy pressure; Evernote/Bear/Google Keep define capture/import expectations. This IP closes the relevant gap by binding `009 checklist and version history` to concrete `notes` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
