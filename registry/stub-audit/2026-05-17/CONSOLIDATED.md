---
audit_id: stub-audit-2026-05-17
status: in-progress
trigger: user_directive_2026-05-17 "audit every PRD, ADR, every IP, every milestone, every gate, every spec and see if any of them are in contradiction to our standards. Fix those."
companion_directive: full-production-only, no MVPs, no stubs
authority_chain:
  - specs/agent-durable-goal.json#operating_principles/OP-11
  - specs/decision-principles.json
  - specs/forbidden-operations.json
---

# Consolidated OP-11 Stub Audit — 2026-05-17

## Surface audited (3 of 5 classes complete)

| Class | Files | Findings | Critical | High | Status |
|---|---|---|---|---|---|
| ADRs | 101 | 196 | 3 | 74 | **complete** |
| Specs | 64 | 180 | 1 | 155 | **complete** |
| IPs | 207 | 307 | 0 | 88 | **complete** |
| PRDs | 13 | 89 | 9 | 44 | **complete** |
| Standards + master-plan | 42+3 | 75 | **54** | 16 | **complete** |
| **TOTAL** | **433** | **847** | **67** | **377** | **all 5 done** |

## Standards-class critical findings (54)

`master-plan-sequencing.json` declares `git`, `gh`, `manual-branch`, `manual-rebase`,
`manual-merge`, `manual-push` as `forbidden_primitives` — but the named enforcer
`oya-governance-banned-primitives` does not exist as a `.github/workflows/` file,
and `git-workflow.md §4` explicitly grants advisory-only status. **Agents demonstrably
use `git`/`gh` routinely** (including this entire session) — the ban is structurally
unenforceable.

29 standards docs cite `enforced_by:` lanes with no workflow file. 9 reference
`oya-check-*` kernel crates that don't exist. `docs/standards/m02-exit-gate-validators.md`
itself has 8 findings (7 missing kernels + BLOCKER flip deferred with 12 of 14 tests
absent). `docs/standards/error-handling.md` has 4 unwired lanes (error-boundary,
no-unwrap-prod, silent-failure, audit-emission).

## Critical findings (4)

| ID | File | Issue |
|---|---|---|
| C-01 | `specs/governance-amendment.json` | Accepted spec with `enforcement.mechanical_components[1].status = "deferred (depends on M02-P04 audit-chain substrate)"` |
| C-02 | `docs/decisions/ADR-0008.md` line 194 | Open questions section in accepted ADR — BEHAVIORAL_TENANT_PRODUCT cross-tenant aggregate flow undecided |
| C-03 | `docs/decisions/ADR-0015.md` line 185 | Open questions section in accepted ADR — sub-context naming inside an axis unresolved |
| C-04 | `docs/decisions/ADR-0053.md` line 138 | RESOLVED 2026-05-18 — concrete ULID `EVT-ADR-LAND-0053-01HXXMKPRGRITICM00000000000000` substituted in by SWEEP-C |

## PRD-class critical findings (9)

Both markdown PRDs (`docs/products/cloud/PRD.md`, `docs/products/foundry/PRD.md`) are
missing 6 required template sections each (§2a acceptance criteria, §9b verification
commands, §11b competitive, §11c best practices, §11d patterns/anti-patterns, §11e goals).
All 4 enterprise PRDs (`platform.json`, `hr.json`, `payroll.json`, `accounting.json`) have
under-structured `user_experience`, `best_practices`, `patterns`, `anti_patterns` blocks
plus missing `production_readiness_gates` — payroll is the most critical given its KR
statutory compliance claims with no production_readiness_gates defined.

sub-PRDs (`platform`, `mail`, `messenger`, `calendar`) carry unsourced competitive
claims (no `source_evidence_refs`) violating BP-07 in workflow.json.

`workflow-studio.json` has 2 open deferrals (CRDT library choice `yrs OR loro`, JS vs
Rust-WASM canvas).

## Industry-pattern audit (separate follow-on, currently in flight)

Per user directive 2026-05-17 "ensure our corporate saas and workflow, as well as our
other products are spec'd to industry leading products", 5 parallel audit-and-fix agents
are running:

- Workflow Studio vs n8n editor + Zapier + Make + Retool + Power Automate
- Enterprise platform (platform, hr, payroll, accounting) vs Workday + ADP + NetSuite + QuickBooks + SAP + BambooHR + Gusto + Xero + Sage
- platform (platform, mail, messenger, calendar) vs Google Workspace + Microsoft 365 + Slack + Proton + Outlook + Gmail
- Foundry vs Palantir Foundry + AWS Bedrock Agents + OpenAI Assistants + LangChain + GitHub Copilot Workspace
- Cloud + meta-hyperscaler-pattern audit (AWS Well-Architected, Google SRE, Azure Well-Architected)

These will land separate PRs that update each PRD with adopted industry patterns +
avoided anti-patterns + hyperscaler bar.

## Dominant cross-audit pattern: aspirational enforcement without implementation

The single largest OP-11 violation class (~340 of 683 findings) is *aspirational enforcement*:

- ADRs cite fitness lanes (`oya-governance-X`) that don't exist on tree — ~130 of 188 non-critical ADR findings
- Specs declare production bars (e.g., "p99 ≤ 10ms") without an enforcement reference — ~30 of specs findings
- IPs name AC-IDs without `test_id` references — 207 of 207 IPs (every single one)
- Master-plan-sequencing names `forbidden_primitives` but no CI guard prevents them (`git`, `gh` are used routinely)

### Worst offenders by aspirational-enforcement count

| Rank | Artifact | Aspirational refs |
|---|---|---|
| 1 | ADR-0107 | 22 missing fitness crates |
| 2 | ADR-0019 | 11 missing fitness crates |
| 3 | ADR-0053 | 6 missing + 1 placeholder |
| 4 | masterplan.json | 153 tracking-stub statuses (legitimate lifecycle, not drift) |

## Tracked-backlog vs in-PR-fixable

| Category | Count | Action |
|---|---|---|
| Tracked-stub IPs (status=stub/scaffolded) | 59 | Legitimate backlog — promote per priority |
| Master-plan tracking-stubs | 153 | Legitimate index lifecycle |
| Aspirational fitness-lane refs (ADR → crate missing) | 130 | **In-PR fixable**: either ship the crate or remove the citation |
| Open questions in accepted ADRs | 2 | **In-PR fixable now**: resolve inline or downgrade status=proposed |
| Placeholder in accepted ADR | 1 | RESOLVED 2026-05-18 — concrete ULID populated by SWEEP-C |
| Deferred enforcement in accepted spec | 1 | **In-PR fixable now**: ship the enforcement or status=proposed |
| Missing AC-IDs across all IPs | 207 | **Structural multi-week**: AC-ID + test_id retrofit across 207 plans |
| `forbidden_primitives` with no CI guard | 6 | **In-PR fixable**: ship `oya-governance-forbidden-primitive-usage` lane |

## Structural fix proposal — chained-enforcement gate

New fitness lane `oya-governance-aspirational-enforcement-detection` that scans:
- Every ADR for `oya-governance-*` references → fails if crate doesn't exist
- Every spec with `"production_bars"` or `"enforcement_mode": "mechanical"` → fails if no `enforcement_ref` field
- Every IP with `acceptance_criteria` → fails if any AC-ID missing `test_id`
- Every spec with `forbidden_primitives` → fails if no `enforcement_lane_id` field

This lane makes the chained-enforcement contract executable: cite the lane in the ADR ⇒ the crate must exist ⇒ the workflow must wire it ⇒ the test must pass.

## Pending audits

- 13 PRDs (background agent in flight)
- 42 standards docs + 3 master-plan specs (background agent in flight)

## Next actions (priority-ordered)

1. **Fix C-01 through C-04** (4 critical findings) — in-PR per OP-11. Owner: this session if budget allows; else next session.
2. **File the structural fitness lane** as `F-FITNESS-ASPIRATIONAL-ENFORCEMENT-DETECTION` with concrete scope.
3. **Triage the 130 aspirational fitness-lane refs**: per-ADR audit of which lanes should ship vs which citations should be removed.
4. **Multi-week**: retrofit AC-IDs + test_ids across 207 IPs.

## Audit raw data

- `registry/stub-audit/2026-05-17/adrs.jsonl` (196 rows)
- `registry/stub-audit/2026-05-17/specs.jsonl` (180 rows)
- `registry/stub-audit/2026-05-17/ips.jsonl` (307 rows)
- `registry/stub-audit/2026-05-17/prds.jsonl` (pending)
- `registry/stub-audit/2026-05-17/standards-and-plans.jsonl` (pending)
