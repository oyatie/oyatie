---
doc_class: ImplementationPlan
impl_plan_id: IP-006-daily-note-template-gallery
milestone: M02-foundation
phase: P01-notes-foundation
status: pending
owner: axis-notes
acceptance_lanes: [cargo-check, cargo-test, oya-governance-port-location]
---

# IP-006: daily-note + template-gallery

## Intent

Land `oya-notes-daily-note-*` (auto-create on first access of user-local-day) + `oya-notes-template-gallery-*` (built-in + user-authored templates with `{{placeholder}}` substitution).

## Built-in Templates

| Template | Purpose |
|---|---|
| meeting-notes | structured meeting capture (attendees, agenda, decisions, action-items) |
| book-notes | book reading (title, author, summary, highlights, quotes) |
| recipe | recipe (ingredients, instructions, notes) |
| project-page | project landing (status, links, retro) |
| daily-journal | default daily-note template (today's intent, gratitudes, what happened) |
| 1-on-1 | reporting-line touchbase notes (objectives, blockers, growth) |
| brainstorm | unstructured ideation |
| code-review | code-review notes |

## Test Plan

- Idempotent daily-note auto-create (PRIMARY KEY (tenant_id, user_id, date)).
- Template placeholder substitution preserves frontmatter.
- Timezone authority per Open Q #4 — user-local from JWT claim; fallback UTC + correction job.

## Acceptance Gates

```bash
cargo check -p oya-notes-daily-note-kernel
cargo check -p oya-notes-template-gallery-kernel
```

## Next IP

[`IP-007-web-clipper-bridge.md`](IP-007-web-clipper-bridge.md)
