---
doc_class: TemplateIndex
status: pending approval
purpose: |
  Catalog of canonical templates and checklists for oyatie that lift to docs/templates/ and templates/checklists/ on approval. Single navigation point for human authors, agent capabilities, and fitness lanes that enforce template shape.
lift_target: oyatie/docs/STANDARDS-AND-TEMPLATES.md  # §Templates + §Checklists sections updated to reference these IDs
authority_chain_declaration: |
  docs/CONSTITUTION.md > docs/AGENTS.md > docs/DOC-CATALOG.md > docs/STANDARDS-AND-TEMPLATES.md > /templates/INDEX.md (this file, working draft)
related:
  - docs/CONSTITUTION.md
  - docs/AGENTS.md
  - docs/DOC-CATALOG.md
  - docs/STANDARDS-AND-TEMPLATES.md
  - docs/RACI-OWNERSHIP.md
  - .omc/plans/MASTERPLAN.md
  - .omc/scratch/hyperscaler-best-practices-2026-05-12.md
---

# Templates + Checklists INDEX

> All entries are `status: pending approval`. Lift on Council-Architecture + Founder sign-off. Each row names the fitness lane (or explicit `(advisory)` marker) that enforces it.

## Templates (`/templates/*.md|*.yaml|*.json`)

| ID | File | Purpose (1-line) | Lift target | Owner (per `docs/RACI-OWNERSHIP.md`) | Enforcing fitness lane |
|---|---|---|---|---|---|
| TPL-PR | `pull-request-template.md` | 5-section PR body + `oya-ci-required` verification + reviewer evidence (`F-PR5-06` bounded until trusted producer is live). | `docs/templates/pull-request-template.md` | `platform-governance` + `council-architecture` | `oya-governance-pr-shape` (`traceability-validator`) |
| TPL-ADR | `adr-template.md` | Architecture Decision Record with autogen-friendly frontmatter. | `docs/templates/adr-template.md` | `crew-adr-promotion` | `oya-governance-adr-shape` |
| TPL-IP | `implementation-plan-template.md` | Per-IP plan under `milestones/M*/phases/P*/`. | `docs/templates/implementation-plan-template.md` | `council-architecture` (cross-axis) | `oya-governance-plan-hierarchy` |
| TPL-PHASE | `phase-index-template.md` | Phase INDEX (≤50 lines). | `docs/templates/phase-index-template.md` | `council-architecture` | `oya-governance-plan-hierarchy` |
| TPL-MILE | `milestone-index-template.md` | Milestone INDEX (≤100 lines). | `docs/templates/milestone-index-template.md` | `council-architecture` | `oya-governance-plan-hierarchy` |
| TPL-RUNBOOK | `runbook-template.md` | Diátaxis-aligned ops runbook, dual-audience. | `docs/templates/runbook-template.md` | `ops-sre-reliability` | `oya-governance-runbook-index-resolves` |
| TPL-CAP | `capability-record-template.yaml` | Foundry capability record (T1-T4, eval-set, audit topic, Cosign slot). | `docs/templates/capability-record-template.yaml` | `axis-foundry` | `oya-governance-capability-publish` |
| TPL-EVB | `evidence-bundle-template.json` | Phase-00-shape evidence bundle, CI-validatable. | `docs/templates/evidence-bundle-template.json` | `ops-compliance` + `axis-foundry` | `oya-governance-audit-emission` |
| TPL-MFL | `mistakes-ledger-row-template.md` | `MFL-NNNN` row with mechanical-prevention field. | `docs/templates/mistakes-ledger-row-template.md` | `council-architecture` | `oya-governance-mistakes-ledger-cite` |
| TPL-PM | `postmortem-template.md` | Google-SRE blameless postmortem with regulator notification matrix. | `docs/templates/postmortem-template.md` | `ops-sre-reliability` | `oya-governance-postmortem-shape` |
| TPL-DD | `design-doc-template.md` | Google-style design doc (problem/goals/non-goals/detailed design/alternatives). | `docs/templates/design-doc-template.md` | `council-architecture` | `oya-governance-design-doc-shape` (advisory at draft; lane on lift) |
| TPL-PRFAQ | `prfaq-template.md` | Amazon Working-Backwards PRFAQ. | `docs/templates/prfaq-template.md` | `council-architecture` + `gtm-marketing` | `(advisory)` for drafts; PRD-level adoption gates by Founder + Council. |

## Checklists (`/templates/checklists/*.md`)

| ID | File | Purpose | Lift target | Owner | Verification path |
|---|---|---|---|---|---|
| CHK-DONE | `done-definition-checklist.md` | Extends `docs/AGENTS.md` D1-D19 with per-change-class variants. | `templates/checklists/done-definition.md` | `platform-governance` + `council-architecture` | `oya-ci-required` + per-lane CI status |
| CHK-PRE | `pre-flight-checklist.md` | Per-change-class preconditions. | `templates/checklists/pre-flight.md` | `council-architecture` | `oya-governance-pr-shape` |
| CHK-PHASE | `per-phase-completion-checklist.md` | Phase-internal verification. | `templates/checklists/per-phase-completion.md` | `council-architecture` | `oya-governance-plan-hierarchy` |
| CHK-IP | `per-implementation-plan-checklist.md` | IP-internal verification. | `templates/checklists/per-implementation-plan.md` | `council-architecture` | `oya-governance-plan-hierarchy` |
| CHK-DOCFRESH | `doc-freshness-checklist.md` | Per-doc-class staleness budget + auto-update path. | `templates/checklists/doc-freshness.md` | `council-architecture` | `oya-governance-doc-freshness` |
| CHK-KICKOFF | `agent-kickoff-checklist.md` | Agent's first 5 actions before claiming task-local work. | `templates/checklists/agent-kickoff.md` | `platform-governance` | `oya-governance-banned-primitives` |
| CHK-COMPLETE | `agent-completion-checklist.md` | Agent's last 5 actions before completion evidence is recorded. | `templates/checklists/agent-completion.md` | `platform-governance` | `oya-governance-banned-primitives` + `oya-governance-audit-emission` |
| CHK-PRREV | `pr-review-checklist.md` | Reviewer agent's per-change-class verification. | `templates/checklists/pr-review.md` | `platform-governance` + per change-class team | trusted reviewer evidence producer (target; bounded by `F-PR5-06`) |
| CHK-REL | `release-readiness-checklist.md` | Milestone-level release gate (wave-gate alignment). | `templates/checklists/release-readiness.md` | `ops-sre-reliability` + `council-architecture` | `oya-governance-release-readiness` |
| CHK-INV | `inventory-update-checklist.md` | Every cutover/migration phase inventory ledger update. | `templates/checklists/inventory-update.md` | `axis-foundry` | `oya-governance-inventory-tracker` |
| CHK-XAXIS | `cross-axis-contract-change-checklist.md` | Cross-axis contract change cascade. | `templates/checklists/cross-axis-contract-change.md` | `council-architecture` | `oya-governance-cross-axis-notify` |
| CHK-ESC | `escalation-checklist.md` | When agent halts and emits `BLOCKED_ON_HUMAN_ORCHESTRATOR`. | `templates/checklists/escalation.md` | `council-architecture` | `oya-governance-banned-primitives` (audits halt events) |
| CHK-SWARM-RITUAL | `swarm-agent-ritual.md` | Per-dispatch Tier-2 ritual (diagram + digraph + role-scaled receipt). Forever home; session rules MUST cite this path. | `templates/checklists/swarm-agent-ritual.md` | `platform-governance` | INV-DOC-9 / process_meta session rule (`integ/ci`) |

## Conflicts discovered with existing oyatie docs

- Existing slim templates exist at `docs/templates/{pull-request-template.md, adr-template.md, capability-record-template.yaml, runbook-template.md, incident-postmortem-template.md}`. New variants here are **supersets**; lift replaces in-place (preserving CI-validator hooks). See per-file `lift_target:` + `supersedes:` frontmatter.
- Canonical checklist home is `templates/checklists/` (docs/checklists/ deleted; dual-home closed); new variants here add: `done-definition`, `pre-flight`, `per-phase-completion`, `per-implementation-plan`, `doc-freshness`, `agent-kickoff`, `agent-completion`, `pr-review`, `release-readiness`, `inventory-update`, `escalation`. The existing `cross-axis-contract-change.md` is extended (not replaced); see CHK-XAXIS frontmatter.
- `docs/templates/migration-runbook-template.md` and `docs/templates/dpia-template.md` and `docs/templates/team-charter-template.md` are out of scope of this delivery; preserve as-is.
