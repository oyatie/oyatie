---
doc_class: DisciplineSpec
shape: discipline
length_cap: 150
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Define the per-doc-class staleness budget for every Oyatie doc, auto-PR
  generator when a threshold is crossed, and the
  `governance-doc-freshness` lane that enforces it: BLOCKER for
  Constitutional + Operating-Contract; HIGH for Reference; Decision-Record is
  never-stale; Working-Draft has 30-day budget.
planned_enforcement_ref: governance-doc-freshness
extends_crates:
  - governance-runbook-freshness-kernel
  - governance-doc-catalog-kernel
  - governance-documentation-system-kernel
companion_docs:
  - INDEX.md
  - runbook-freshness-pipeline.md
  - ../../docs/DOC-CATALOG.md
doc_status: published
---

# Discipline: doc freshness

> **ADRs:** ADR-0052, ADR-0053, ADR-0054.

## 1. Purpose

The `governance-runbook-freshness-kernel` proves the freshness pattern; this discipline extends it to every doc class. No Tier-1 / Tier-2 doc may rot silently. Per-doc-class budgets enforce a cadence; threshold crossings auto-generate refresh PRs; the lane fails merges that introduce stale-doc references.

## 2. Doc-class taxonomy (per `docs/DOC-CATALOG.md` + `docs/standards/doc-style.md`)

| Doc class | Examples | Staleness budget |
|---|---|---|
| **Constitutional** | `CONSTITUTION.md`, `AGENTS.md` | 365 days |
| **Operating-Contract** | `DOC-CATALOG.md`, `AGENTS.md`, `RELEASE-MANAGEMENT.md` | 90 days |
| **Reference** | `SPEC.md`, `GLOSSARY.md`, `DOC-CATALOG.md` | 90 days |
| **Decision-Record** | `decisions/ADR-*.md` | never-stale (Accepted ADRs are durable) |
| **Working-Draft** | `.omc/plans/**/*.md`, `.omc/drafts/**/*.md` | 30 days |

## 3. Staleness-budget table (verbatim, the SLA)

| Doc class | Budget | Lane severity on breach | Auto-PR generated? |
|---|---|---|---|
| Constitutional | 365 days | BLOCKER | yes (founder + council-architecture review required) |
| Operating-Contract | 90 days | BLOCKER | yes |
| Reference | 90 days | HIGH | yes |
| Decision-Record | never-stale | n/a (status-bound, not date-bound) | no |
| Working-Draft | 30 days | advisory | no (drafts allowed to drop) |

## 4. Required frontmatter

Every doc declares:

```yaml
doc_class: Constitutional | Operating-Contract | Reference | Decision-Record | Working-Draft
last_verified: 2026-05-12
next_review: 2026-08-10
owner: council-architecture
```

Decision-Record docs are exempt from `last_verified:` / `next_review:` requirements; their freshness is bound to status (Proposed/Accepted/Superseded/Deprecated/Rejected).

## 5. Inputs

- Every `docs/**/*.md` frontmatter.
- Every `.omc/plans/**/*.md` frontmatter (for working-draft tracking).
- Current date (CI build clock).

## 6. Outputs

- Freshness report `docs/machine-readable/doc-freshness.json` (nightly).
- mdbook chapter `docs/site/src/operations/doc-health.md` (per-doc-class freshness landscape).
- Auto-generated refresh PRs (one per breached doc) using `docs/templates/doc-refresh-pr.md`.

## 7. Validation gates (`governance-doc-freshness`)

1. **Frontmatter completeness.** Every Tier-1/Tier-2 doc has `doc_class:`, `last_verified:`, `next_review:`, `owner:` (BLOCKER).
2. **Class validity.** `doc_class:` ∈ the five allowed values (BLOCKER).
3. **Future date ban.** `last_verified:` MUST NOT exceed today's CI date (BLOCKER; same as the extant runbook kernel's `FutureLastVerified`).
4. **Staleness enforcement.** Per the table above; severity per class.
5. **Refresh-PR existence.** If a doc is at or past its budget, an open refresh-PR MUST exist (HIGH); if missing, the nightly job opens one.
6. **next_review consistency.** `next_review:` ≤ `last_verified` + budget (HIGH).

## 8. Auto-refresh-PR template (`docs/templates/doc-refresh-pr.md`)

```markdown
# Refresh: <doc path>

**Doc class:** <Constitutional | Operating-Contract | Reference | Working-Draft>
**Owner:** <team>  **Days past budget:** <N>

## Checklist

- [ ] Walked the doc top-to-bottom; every section still reflects current state.
- [ ] Every cited ADR confirmed current (not Superseded).
- [ ] Every cited contract/event/topic resolves.
- [ ] Updated `last_verified:` to today's date.
- [ ] Updated `next_review:` to today + budget.
- [ ] Updated CHANGELOG row per `changelog-pipeline.md`.

## Auto-summarized changes since last verification

<diff summary>
```

## 9. Linkage to runbook-freshness

`runbook-freshness-pipeline.md` is the Sev-aware variant for `docs/runbooks/**`. This discipline covers the broader doc tree using the doc-class taxonomy instead of Sev scope. Both lanes coexist; runbooks satisfy both.

## 10. Out-of-scope

- Per-paragraph freshness (granular; covered by per-section anchor links and ADR cites).
- Auto-rewriting doc bodies (out of scope; refresh PRs are bot-opened but human-edited).
- External-doc freshness (vendor docs are linked, not vendored).
