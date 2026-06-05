---
doc_class: Template
template_id: TPL-PR
status: pending approval
purpose: |
  Canonical PR body for every change. 5 author-owned H2 sections plus a lead-owned `## Code Review` added at merge time. Dual-audience: identical content readable by both reviewer agents and human reviewers. Enforces RFC-2119 normative form where requirements are stated.
lift_target: oyatie/docs/templates/pull-request-template.md
supersedes: docs/templates/pull-request-template.md
enforcing_fitness_lane: oya-governance-pr-shape (delegates to `traceability-validator`)
owner_team: axis-foundry + council-architecture
related:
  - docs/AGENTS.md  # §PR shape + §Done-Definition
  - docs/STANDARDS-AND-TEMPLATES.md  # §2
  - /templates/checklists/done-definition-checklist.md
  - /templates/checklists/pr-review-checklist.md
rfc_2119_active: true
---

<!-- author-owned: fill 5 sections below before requesting review. lead-owned: `## Code Review` at merge. -->

## Issue

`Closes #<n>` (or `Refs #<n>` if not closing). Change class **MUST** be named on the same line: `feature | bugfix | refactor | migration | docs | chore | capability | plugin | runbook | ADR | pack-update`.

## Summary

- 1-3 bullets on **what + why**. The diff already shows the *what*; this section adds the *why*.
- Cite the canonical authority you read first per `docs/AGENTS.md §Pre-flight checklist` item 2.

<!-- agent-instructions:start -->
**Agent path** (read this fork if you are a Claude/Codex/Gemini/Foundry agent):
- Authoring a PR **MUST** use the current lane primitives: isolated `git worktree`, short-lived `git` branch, `gh pr create --base dev`, and lane-owned Buck2/Prow evidence. Retired local VCS/governance wrappers are not SCM or CI authority.
- The `## Verification` block **MUST** paste actual tool output, not a hand-wave. Paste the captured stdout/stderr from the Buck2 targets and remote PR checks that prove the claim.
- The `## Code Review` H2 **MUST NOT** be added by the worker agent; only the lead reviewer agent (per change-class table in `docs/AGENTS.md §Per-change-class reviewer agents`) signs it at merge time. Adding it as a worker is a `guard-pr-merge-review.mjs` violation.
<!-- agent-instructions:end -->

**Human path:** PR body uses 5 H2 sections; CI `traceability-validator` fails the gate if any section is missing or empty. Reviewer-agent verdict is pasted into `## Code Review` at merge.

## Verification

Each line below **MUST** be present with a pass/fail token (`PASS` / `FAIL`) and the actual command output excerpt.

- `buck2 build //:<lane-owned-target>` — `<PASS|FAIL>` — `<excerpt>`
- `buck2 build //:buck2-authority-policy-check` — `<PASS|FAIL|N/A>` — `<excerpt>`
- `buck2 build //:repo-hygiene-automation-check` — `<PASS|FAIL|N/A>` — `<excerpt>`
- `buck2 build //:kubernetes-native-anti-pattern-check` — `<PASS|FAIL|N/A>` — `<excerpt>`
- Remote PR checks: `<Prow/GitHub adapter contexts + PASS|FAIL each>`
- Per-change-class fitness lanes: `<list Buck2/Prow lanes + PASS|FAIL each>`
- Per-change-class reviewer agent: `<agent-name>` — verdict `<APPROVE|REQUEST CHANGES>`

## Traceability

- Catalog records touched: `<list under registry/catalog/>`
- Cross-axis contracts touched: `<list under contracts/>` (per `docs/DESIGN.md §10`)
- ADRs cited: `<ADR-NNNN list>` (legacy ADR-NNNN forbidden in active text per `docs/ADR-CONSOLIDATION-PLAN.md`)
- `MISTAKES-LEDGER` row referenced (if regression-class): `MFL-NNNN`
- Cross-axis review label applied (if cross-axis contract change): `<label>` (see `/templates/checklists/cross-axis-contract-change-checklist.md`)
- Implementation Plan ID (if executing an IP): `IP-NNN-<slug>` from `.omc/plans/milestones/M*/phases/P*/`
- Lane-owned paths/symbols (agent path): `<file::Identifier or path list>`

## Evidence

- Audit-chain emission ID: `EVT-<topic>-<ulid>` (per ADR-0003)
- Foundation-bypass referenced (if any): `<bypass-id>` + renewal date
- Per-pack regulator-watch impact (if any): `<oya-pack-XX.regulator list>`
- Distroless image build (if shipping a binary): `<image:tag>` + Cosign attestation digest
- SBOM artifact: `<path|registry-ref>` (Syft/CycloneDX)
- SLSA provenance level achieved: `L1 | L2 | L3`

<!-- merge-gate: lead reviewer adds `## Code Review` below at merge; `guard-pr-merge-review.mjs` refuses without it. -->

## Code Review _(lead-only — never as worker)_

- Reviewer agent: `<rust-reviewer | non-rust-exception-reviewer | database-reviewer | security-reviewer | privacy-reviewer | tdd-guide | silent-failure-hunter | doc-updater | doc-style-reviewer | capability-reviewer | perf-reviewer>`
- Verdict: `<APPROVE | REQUEST CHANGES>`
- Resolved items: `<list>`
- Deferred items: `<list with owners + follow-up issue refs>`
- Linus good-taste audit row: `<special cases eliminated | "none — no candidates">`
