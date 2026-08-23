---
doc_class: PipelineSpec
shape: pipeline
length_cap: 150
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Every PR that touches a canonical doc (Tier-1 or Tier-2 per DOC-CATALOG.md)
  auto-emits a `docs/CHANGELOG.md` row. The row is templated from the PR body +
  commit metadata; the `governance-changelog-row` lane fails the PR if
  the row is missing or malformed. CHANGELOG drift dies at the door.
planned_enforcement_ref: governance-changelog-row
extends_crates:
  - governance-doc-catalog-kernel
companion_docs:
  - INDEX.md
  - ../../docs/CHANGELOG.md
  - ../../docs/DOC-CATALOG.md
doc_status: published
---

# Pipeline: CHANGELOG row auto-emission

> **ADRs:** ADR-0052, ADR-0053, ADR-0054.

## 1. Purpose

`docs/CHANGELOG.md` is the human-readable trail of canonical-doc evolution. Today it is hand-maintained, which guarantees drift. This pipeline closes the loop: any PR touching a doc listed in `docs/DOC-CATALOG.md` Tier-1 or Tier-2 MUST emit a CHANGELOG row, and the row shape is tracked by planned advisory lane `governance-changelog-row`.

## 2. Inputs

- PR body parsed for the `## Changelog` block (required for canonical-doc PRs).
- Commit metadata: SHA, author (mapped through CODEOWNERS), date, primary `doc.<id>` touched.
- `docs/DOC-CATALOG.md` machine-readable mirror at `docs/machine-readable/catalog.json` (the doc-id lookup table).
- Existing `docs/CHANGELOG.md` for de-dup and ordering.

## 3. Row shape

```markdown
## YYYY-MM-DD — <PR-NNNN>

- **doc.<id>** (Tier <N>): <one-line summary from PR body's `## Changelog` block>.
  - Authors: <author handles>
  - ADRs cited: <ADR-#### list, comma-separated, or "none">
  - Related lanes: <fitness-lane ids touched>
  - Commit: <SHA>
```

Rows are grouped by date descending; multiple rows per day are allowed.

## 4. Trigger matrix

| Event | Action |
|---|---|
| Per-PR touching any Tier-1/Tier-2 doc | Lane runs; if `## Changelog` block missing in PR body → BLOCKER. If row not appended to `docs/CHANGELOG.md` → BLOCKER. |
| On-merge | Row finalized (PR number + merge SHA filled in if `PR-NNNN` was placeholder). |
| Nightly | Sweep CHANGELOG for duplicate rows, out-of-order dates, missing ADR citations on doc.adr_index touches. |

## 5. Validation gates (`governance-changelog-row`)

1. **Row presence.** Every Tier-1/Tier-2 doc touch produces a corresponding row in the same PR (BLOCKER).
2. **Row schema.** Row parses against the shape above; missing fields → BLOCKER.
3. **doc-id resolution.** The `doc.<id>` field exists in `catalog.json` (BLOCKER on typo).
4. **Author attribution.** `Authors:` field non-empty and resolves to a real GitHub handle in CODEOWNERS (HIGH).
5. **ADR-citation linkage.** When `doc.adr_index` is touched, the row's `ADRs cited:` field must list the touched ADRs (HIGH; cross-fed from the ADR-citation linkage check).
6. **Ordering.** New rows append at top; out-of-order dates → HIGH.

## 6. PR template integration

`docs/templates/pull-request-template.md` is extended with a required `## Changelog` block. The hook refuses PR creation (via pre-push) if the block is absent for a canonical-doc-touching diff.

## 7. Auto-fill draft

For PRs that touch a canonical doc, the bot opens a draft `## Changelog` block in the PR body containing:

- `**doc.<id>** (Tier <N>): <placeholder summary derived from PR title>`
- `Authors: <auto from git commit author>`
- `ADRs cited: <auto-detected from PR body ADR-#### references>`

The author edits the placeholder summary before merge.

## 8. Out-of-scope

- Source-code-only CHANGELOG (covered by per-crate `CHANGELOG.md` files, owned by `intelligence-api-semver-kernel`).
- Marketing-facing release notes (separate `docs/RELEASE-NOTES.md`, hand-curated).
- ADR-pack CHANGELOG (handled inline in each ADR's status-transition history).
