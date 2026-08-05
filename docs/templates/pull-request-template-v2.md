---
doc_class: Template
template_id: TPL-PR
status: Accepted
date: 2026-05-12
purpose: |
  Canonical PR body for every change. 5 author-owned H2 sections plus a lead-owned `## Code Review` added at merge time. Dual-audience: identical content readable by both reviewer agents and human reviewers. Enforces RFC-2119 normative form where requirements are stated.
supersedes: docs/templates/pull-request-template.md
header_note: "Supersedes prior docs/templates/pull-request-template.md once reviewed."
enforcing_fitness_lane: oya-governance-pr-shape (delegates to `traceability-validator`)
owner_team: axis-foundry + council-architecture
related:
  - docs/AGENTS.md  # §PR shape + §Done-Definition
  - docs/STANDARDS-AND-TEMPLATES.md  # §2
  - docs/checklists/done-definition-checklist.md
  - docs/checklists/pr-review-checklist.md
adrs_cited:
  - ADR-0052  # inventory ledger (traceability row)
  - ADR-0053  # sanctioned primitives (agent path)
rfc_2119_active: true
doc_status: published
---

<!-- Supersedes prior docs/templates/pull-request-template.md once reviewed. -->

<!-- author-owned: fill 5 sections below before requesting review. lead-owned: `## Code Review` at merge. -->

## Issue

`Closes #<n>` (or `Refs #<n>` if not closing). Change class **MUST** be named on the same line: `feature | bugfix | refactor | migration | docs | chore | capability | plugin | runbook | ADR | pack-update`.

## Summary

- 1-3 bullets on **what + why**. The diff already shows the *what*; this section adds the *why*.
- Cite the canonical authority you read first per `docs/AGENTS.md §Pre-flight checklist` item 2.

<!-- agent-instructions:start -->
**Agent path** (read this fork if you are a Claude/Codex/Gemini agent):
- The `## Verification` block **MUST** paste actual tool output or cloud-ci/GitHub status evidence, not a hand-wave. Legacy `oya-tooling-agent-read` capture is optional bridge evidence only.
- The `## Code Review` H2 **MUST NOT** be added by the worker agent; only the lead reviewer agent (per change-class table in `docs/AGENTS.md §Per-change-class reviewer agents`) signs it at merge time. Adding it as a worker is a `guard-pr-merge-review.mjs` violation.
- Before closeout, the worker/reviewer **MUST** harvest agent observations: review chat, review notes, scratch/workspace notes, PR comments, and Kanban comments; dedupe against active cards; then create/link Kanban work or document duplicates/no-action rationale in `## Evidence`.
<!-- agent-instructions:end -->

**Human path:** PR body uses 5 H2 sections; CI `traceability-validator` fails the gate if any section is missing or empty. Reviewer-agent verdict is pasted into `## Code Review` at merge.

## Verification

Each line below **MUST** be present with a pass/fail token (`PASS` / `FAIL`) and the actual command output excerpt.

- `cargo nextest run --workspace --all-features --no-fail-fast` — `<PASS|FAIL>` — `<excerpt>`
- `cargo clippy --workspace --all-features --all-targets -- -D warnings` — `<PASS|FAIL>` — `<excerpt>`
- `cargo deny check` — `<PASS|FAIL>` — `<excerpt>`
- `oya-ci-required` protected status — `<PASS|FAIL>` — `<status URL or cloud-ci packet excerpt>`
- Legacy local bridge evidence, if run (`oya verify` / `oya gate validate`) — `<PASS|FAIL|N/A>` — `<excerpt or rationale>`
- Per-change-class fitness lanes: `<list lanes + PASS|FAIL each>`
- Per-change-class reviewer agent: `<agent-name>` — verdict `<APPROVE|REQUEST CHANGES>`

## Traceability

- Catalog records touched: `<list under registry/catalog/>`
- Cross-axis contracts touched: `<list under contracts/>` (per `docs/DESIGN.md §10`)
- ADRs cited: `<ADR-#### list>` (legacy ADR-#### forbidden in active text per `docs/ADR-CONSOLIDATION-PLAN.md`)
- `MISTAKES-LEDGER` row referenced (if regression-class): `MFL-NNNN`
- Cross-axis review label applied (if cross-axis contract change): `<label>` (see `docs/checklists/cross-axis-contract-change-checklist.md`)
- Implementation Plan ID (if executing an IP): `IP-NNN-<slug>` from `.omc/plans/milestones/M*/phases/P*/`
- Inventory ledger row (if migration-class): `INV-NNNN` (per ADR-0052)

## Evidence

- Audit-chain emission ID: `EVT-<topic>-<ulid>` (per ADR-0003)
- Foundation-bypass referenced (if any): `<bypass-id>` + renewal date
- Per-pack regulator-watch impact (if any): `<oya-pack-XX.regulator list>`
- Distroless image build (if shipping a binary): `<image:tag>` + Cosign attestation digest
- SBOM artifact: `<path|registry-ref>` (Syft/CycloneDX)
- SLSA provenance level achieved: `L1 | L2 | L3`
- Post-merge product-completion packet (after squash merge):
  - promoted SHA + `oya-ci-required` status URL
  - rollout verification + rollback note
  - observability check + browser UX/user-story evidence
  - release-governance/release-note impact (Release Please applies only when a live repo config/workflow exists)
- Agent-observation harvest:
  - source contexts reviewed: `<chat|review notes|scratch|PR|Kanban|N/A>`
  - outcome: `<new/linked Kanban card ids | duplicate/no-action rationale>`
  - new/linked card fields present: source context, classification, affected card/PR/artifact, acceptance criteria, verification path, suggested owner/profile, dependencies/conflict notes

<!-- merge-gate: lead reviewer adds `## Code Review` below at merge; `guard-pr-merge-review.mjs` refuses without it. -->

## Code Review

_(lead-only — never as worker)_

- Reviewer agent: `<rust-reviewer | typescript-reviewer | python-reviewer | database-reviewer | security-reviewer | privacy-reviewer | tdd-guide | silent-failure-hunter | doc-updater | doc-style-reviewer | capability-reviewer | perf-reviewer>`
- Verdict: `<APPROVE | REQUEST CHANGES>`
- Resolved items: `<list>`
- Deferred items: `<list with owners + follow-up issue refs>`
- Linus good-taste audit row: `<special cases eliminated | "none — no candidates">`
