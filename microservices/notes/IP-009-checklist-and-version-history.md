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

## Next IP

[`IP-010-search-and-graph-view.md`](IP-010-search-and-graph-view.md)
