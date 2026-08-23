---
doc_class: PipelineSpec
shape: pipeline
length_cap: 150
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Enforce a `last_verified:` date on every runbook under `docs/runbooks/**/*.md`
  with severity-aware staleness thresholds: >90 days = advisory, >180 days = HIGH,
  >365 days = BLOCKER. Auto-generate a refresh-PR template when a threshold is
  crossed. Extends the existing `governance-runbook-freshness-kernel`.
planned_enforcement_ref: governance-runbook-freshness
extends_crates:
  - governance-runbook-freshness-kernel
  - governance-runbook-index-kernel
companion_docs:
  - INDEX.md
  - doc-freshness-discipline.md
  - ../../docs/RUNBOOKS-INDEX.md
doc_status: published
---

# Pipeline: runbook freshness + auto-refresh-PR

> **ADRs:** ADR-0052, ADR-0053, ADR-0054.

## 1. Purpose

The extant `governance-runbook-freshness-kernel` already validates `last_verified:` per severity (Sev-1 ≤ 90d, Sev-2 ≤ 180d, Sev-3/4 ≤ 365d). This pipeline adds:

1. A graduated severity ladder (advisory → HIGH → BLOCKER) so operations are not silently broken at the cliff.
2. Auto-generation of a refresh-PR template the moment any threshold is crossed.
3. mdbook rollup chapter `/operations/runbook-health` showing the freshness landscape.

## 2. Inputs

- Every `docs/runbooks/**/*.md` frontmatter: `last_verified: YYYY-MM-DD`, `severity_scope:` (Sev-1..Sev-4 or unscoped), `owner:`, `status:`, `next_review:`.
- `docs/RUNBOOKS-INDEX.md` (validated by `governance-runbook-index-kernel`).
- Current date (CI build clock).

## 3. Outputs

- Freshness report JSON `docs/machine-readable/runbook-freshness.json` per nightly run.
- Auto-generated refresh PR (one per stale runbook) using the template at `docs/templates/runbook-refresh-pr.md` with body pre-filled from the previous `last_verified` author, the runbook owner, and the change log since.
- mdbook chapter `docs/site/src/operations/runbook-health.md` (overall freshness landscape; per-Sev distribution; per-owner backlog).

## 4. Severity ladder

| Sev scope | Advisory (warn) | HIGH (lane fail) | BLOCKER (merge gate) |
|---|---|---|---|
| Sev-1 | > 60 days | > 90 days | > 120 days |
| Sev-2 | > 120 days | > 180 days | > 240 days |
| Sev-3/4 / unscoped | > 270 days | > 365 days | > 450 days |

The extant kernel handles the HIGH threshold; this pipeline extends to advisory + BLOCKER tiers via additional records in the report.

## 5. Trigger matrix

| Event | Action |
|---|---|
| Per-PR touching `docs/runbooks/**` | Re-run kernel; if `last_verified:` not bumped on a touched file, advisory comment posted. |
| Nightly | Full sweep; auto-create refresh PRs for every advisory-tier and HIGH-tier runbook; update mdbook chapter. |
| Weekly (Monday 09:00 KST) | Email digest to runbook owners with their backlog count. |

## 6. Auto-refresh-PR template

```markdown
# Refresh runbook: <path>

**Severity:** <Sev-N>  **Owner:** <team>  **Days since last verification:** <N>

## Checklist (must check at least one verification action below before merging)

- [ ] Walked through the runbook end-to-end in a non-prod environment on <date>.
- [ ] Confirmed every cited tool / command / dashboard / alert exists.
- [ ] Confirmed every linked ADR is current (not Superseded).
- [ ] Updated `last_verified:` to today's date.
- [ ] Updated `next_review:` to today + half-the-Sev-window.

## Changes since last verification

<auto-summarized commit diff against runbook path>
```

## 7. Validation gates (extending `governance-runbook-freshness`)

The existing kernel's errors remain authoritative. New gate additions:

1. **Refresh-PR existence.** If a runbook is HIGH-stale and no open refresh-PR exists, the lane raises HIGH and the nightly job opens one.
2. **Refresh-PR aging.** If a refresh-PR is itself > 14 days old without merge, escalate to owning team lead.

## 8. Out-of-scope

- Runbook content quality (covered by `governance-runbook-index-kernel` discoverability checks).
- Per-Sev playbook completeness (covered by `governance-slo-coverage-kernel`).
- Incident postmortem freshness (covered by `governance-incident-template-completeness`).
