---
doc_class: ImplementationPlan
impl_plan_id: IP-009-checklist-and-version-history
milestone: M02-foundation
phase: P01-notes-foundation
status: pending
owner: axis-notes
acceptance_lanes: [cargo-check, cargo-test, oya-governance-port-location]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

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
