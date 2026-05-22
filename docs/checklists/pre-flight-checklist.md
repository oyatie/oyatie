---
doc_class: Checklist
checklist_id: CHK-PRE
status: Accepted
date: 2026-05-12
purpose: |
  Every change class precondition. Walked before any agent claims a symbol or any human stages a commit. Extends `docs/AGENTS.md §Pre-flight checklist`.
enforcing_fitness_lane: oya-governance-pr-shape
owner_team: council-architecture
related:
  - docs/AGENTS.md
  - docs/CONSTITUTION.md
  - docs/checklists/done-definition-checklist.md
  - docs/checklists/agent-kickoff-checklist.md
adrs_cited:
  - ADR-0052  # inventory ledger (migration pre-flight)
  - ADR-0053  # sanctioned primitives (agent tool restrictions)
  - ADR-0054  # scaffold-claim (symbol pre-flight)
doc_status: published
---

# Pre-flight Checklist

> Walk before any change. Each row names a verification path (lane / command / advisory).

## Universal (every change class)

- [ ] **P1** Identify the change class on a single line in the eventual PR `## Issue`. *Class:* `feature | bugfix | refactor | migration | docs | chore | capability | plugin | runbook | ADR | pack-update`. *Lane:* `traceability-validator`.
- [ ] **P2** Read the canonical authority for the class (`docs/AGENTS.md §Canonical doc map`). *Verification:* PR `## Traceability` cites the doc(s) read.
- [ ] **P3** Confirm Data Use Boundary. Every new kernel-struct field carries `data_class`. *Lane:* `oya-governance-data-class`.
- [ ] **P4** Confirm autonomy ceiling. Capability bindings declare T1/T2/T3/T4. *Lane:* `oya-governance-autonomy-ceiling`.
- [ ] **P5** Confirm license posture. *Command:* `cargo deny check`.
- [ ] **P6** Search `docs/MISTAKES-LEDGER.md` for the failure-mode class. *Verification:* PR `## Traceability` cites the `MFL-NNNN` row OR a "no prior row" note.
- [ ] **P7** Identify the per-change-class reviewer agent (per `docs/AGENTS.md §Per-change-class reviewer agents`). *Lane:* `guard-pr-merge-review.mjs`.
- [ ] **P8** For cross-axis contract changes: apply cross-axis review label + notify consumer-axis teams. *Lane:* `oya-governance-cross-axis-notify`. Use `docs/checklists/cross-axis-contract-change-checklist.md`.
- [ ] **P9** For hook / harness / CLI changes: run the harness self-test first. *Command:* `npm --prefix /Users/home/.codex test` (per harness).

## Per-change-class additions

### feature
- [ ] PRFAQ authored (if new-product / new-axis / new-vertical scope) per `docs/templates/prfaq-template.md`. *(advisory; Founder + Council approval gate)*
- [ ] Design doc authored per `docs/templates/design-doc-template.md`; reviewers named. *Lane:* `oya-governance-design-doc-shape` (advisory).

### bugfix
- [ ] Reproduction recipe documented (input + observed + expected). *(advisory)*
- [ ] Failing regression test written **before** the fix (TDD per `superpowers:test-driven-development`). *Lane:* `oya-governance-qa-coverage`.

### refactor
- [ ] No behavior change asserted; public API surface unchanged. *Command:* `cargo public-api --diff`.
- [ ] Linus good-taste candidate identified (special case to delete). *(advisory)*

### migration
- [ ] Inventory ledger row drafted per `docs/checklists/inventory-update-checklist.md` (ADR-0052). *Lane:* `oya-governance-inventory-tracker`.
- [ ] Rollback boundary identified; rollback command named. *Lane:* `oya-governance-schema-migration` (for schema migrations).

### docs
- [ ] `docs/DOC-CATALOG.md` trigger event named in PR `## Issue`. *Lane:* `oya-governance-doc-catalog`.
- [ ] `doc-class:` taxonomy honored per `docs/standards/doc-style.md`. *Lane:* `oya-governance-doc-style`.

### chore
- [ ] Confirm no production behavior change. *(advisory)*

### capability
- [ ] Capability record drafted from `docs/templates/capability-record-template-v2.yaml`. *Lane:* `capability-schema-validator`.
- [ ] Eval set scaffolds present (golden + adversarial + linguistic). *Lane:* `oya-governance-capability-publish`.
- [ ] Cedar policy stub + runtime-gate stub present if tier ≥ T2. *Lane:* `oya-governance-autonomy-ceiling`.

### plugin
- [ ] Plugin manifest validates. *Lane:* `oya-governance-plugin-manifest`.
- [ ] External-network allowlist enumerated. *Lane:* `oya-governance-plugin-network`.

### runbook
- [ ] Trigger / SLO impact / mitigation steps drafted from `docs/templates/runbook-template-v2.md`. *Lane:* `oya-governance-runbook-index-resolves`.
- [ ] Drill scheduled (date). *(advisory)*

### ADR
- [ ] Next-free ADR slot identified by reading `docs/ADR-INDEX.md`. *(advisory)*
- [ ] ≥ 2 viable alternatives drafted per `docs/templates/adr-template-v2.md §Alternatives considered`. *Lane:* `oya-governance-adr-shape`.

### pack-update
- [ ] Regulator-watch impact summarized. *Lane:* `oya-governance-compliance-matrix`.
- [ ] Regional pack file path enumerated under `regional-packs/<pack>/`. *Lane:* `oya-governance-pack-coverage`.

## Stop conditions

If any row above cannot be checked, **do not** claim a symbol. Either: (a) escalate per `docs/checklists/escalation-checklist.md`, or (b) author the missing artifact first (PRFAQ → DD → IP cascade).
