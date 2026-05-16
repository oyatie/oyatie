---
doc_class: TemplateIndex
status: Accepted
date: 2026-05-12
purpose: |
  Catalog of canonical templates and checklists for oyatie that lift to docs/templates/ and docs/checklists/ on approval. Single navigation point for human authors, agent capabilities, and fitness lanes that enforce template shape.
authority_chain_declaration: |
  docs/CONSTITUTION.md > docs/AGENTS.md > docs/DOC-CATALOG.md > docs/STANDARDS-AND-TEMPLATES.md > docs/templates/INDEX.md (this file)
related:
  - docs/CONSTITUTION.md
  - docs/AGENTS.md
  - docs/DOC-CATALOG.md
  - docs/STANDARDS-AND-TEMPLATES.md
  - docs/RACI-OWNERSHIP.md
  - .omc/plans/MASTERPLAN.md
  - .omc/scratch/hyperscaler-best-practices-2026-05-12.md
adrs_cited:
  - ADR-0052  # inventory ledger
  - ADR-0053  # sanctioned primitives
  - ADR-0054  # scaffold-claim pattern
doc_status: published
---

# Templates + Checklists INDEX

> Lifted 2026-05-12 per Stage 1 Wave 2. Status: `Accepted`. Each row names the fitness lane (or explicit `(advisory)` marker) that enforces it.

## Templates (`docs/templates/*.md|*.yaml|*.json`)

| ID | File | Purpose (1-line) | Owner (per `docs/RACI-OWNERSHIP.md`) | Enforcing fitness lane |
|---|---|---|---|---|
| TPL-PR | `pull-request-template-v2.md` | 5-section PR body + agent/human fork + RFC-2119 normative form. | `axis-foundry` + `council-architecture` | `oya-foundry-fitness-pr-shape` (`traceability-validator`) |
| TPL-ADR | `adr-template-v2.md` | Architecture Decision Record with autogen-friendly frontmatter. | `crew-adr-promotion` | `oya-foundry-fitness-adr-shape` |
| TPL-IP | `implementation-plan-template.md` | Per-IP plan under `milestones/M*/phases/P*/`. | `council-architecture` (cross-axis) | `oya-foundry-fitness-plan-hierarchy` |
| TPL-PHASE | `phase-index-template.md` | Phase INDEX (≤50 lines). | `council-architecture` | `oya-foundry-fitness-plan-hierarchy` |
| TPL-MILE | `milestone-index-template.md` | Milestone INDEX (≤100 lines). | `council-architecture` | `oya-foundry-fitness-plan-hierarchy` |
| TPL-RUNBOOK | `runbook-template-v2.md` | Diátaxis-aligned ops runbook, dual-audience. | `ops-sre-reliability` | `oya-foundry-fitness-runbook-index-resolves` |
| TPL-CAP | `capability-record-template-v2.yaml` | Foundry capability record (T1-T4, eval-set, audit topic, Cosign slot). | `axis-foundry` | `oya-foundry-fitness-capability-publish` |
| TPL-EVB | `evidence-bundle-template.json` | Phase-00-shape evidence bundle, CI-validatable. | `ops-compliance` + `axis-foundry` | `oya-foundry-fitness-audit-emission` |
| TPL-MFL | `mistakes-ledger-row-template.md` | `MFL-NNNN` row with mechanical-prevention field. | `council-architecture` | `oya-foundry-fitness-mistakes-ledger-cite` |
| TPL-PM | `postmortem-template.md` | Google-SRE blameless postmortem with regulator notification matrix. | `ops-sre-reliability` | `oya-foundry-fitness-postmortem-shape` |
| TPL-DD | `design-doc-template.md` | Google-style design doc (problem/goals/non-goals/detailed design/alternatives). | `council-architecture` | `oya-foundry-fitness-design-doc-shape` (advisory at draft; lane on lift) |
| TPL-PRFAQ | `prfaq-template.md` | Amazon Working-Backwards PRFAQ. | `council-architecture` + `gtm-marketing` | `(advisory)` for drafts; PRD-level adoption gates by Founder + Council. |

## Checklists (`docs/checklists/*.md`)

| ID | File | Purpose | Owner | Verification path |
|---|---|---|---|---|
| CHK-DONE | `done-definition-checklist.md` | Extends `docs/AGENTS.md` D1-D18 with per-change-class variants. | `axis-foundry` + `council-architecture` | `guard-pr-merge-review.mjs` + per-lane CI status |
| CHK-PRE | `pre-flight-checklist.md` | Per-change-class preconditions. | `council-architecture` | `oya-foundry-fitness-pr-shape` |
| CHK-PHASE | `per-phase-completion-checklist.md` | Phase-internal verification. | `council-architecture` | `oya-foundry-fitness-plan-hierarchy` |
| CHK-IP | `per-implementation-plan-checklist.md` | IP-internal verification. | `council-architecture` | `oya-foundry-fitness-plan-hierarchy` |
| CHK-DOCFRESH | `doc-freshness-checklist.md` | Per-doc-class staleness budget + auto-update path. | `council-architecture` | `oya-foundry-fitness-doc-freshness` |
| CHK-KICKOFF | `agent-kickoff-checklist.md` | Agent's first 5 actions before any `grit claim`. | `axis-foundry` | `oya-foundry-fitness-banned-primitives` |
| CHK-COMPLETE | `agent-completion-checklist.md` | Agent's last 5 actions before `grit done`. | `axis-foundry` | `oya-foundry-fitness-banned-primitives` + `oya-foundry-fitness-audit-emission` |
| CHK-PRREV | `pr-review-checklist.md` | Reviewer agent's per-change-class verification. | `axis-foundry` + per change-class team | `guard-pr-merge-review.mjs` |
| CHK-REL | `release-readiness-checklist.md` | Milestone-level release gate (wave-gate alignment). | `ops-sre-reliability` + `council-architecture` | `oya-foundry-fitness-release-readiness` |
| CHK-INV | `inventory-update-checklist.md` | Every cutover/migration phase inventory ledger update (ADR-0052). | `axis-foundry` | `oya-foundry-fitness-inventory-tracker` |
| CHK-XAXIS | `cross-axis-contract-change-checklist.md` | Cross-axis contract change cascade. | `council-architecture` | `oya-foundry-fitness-cross-axis-notify` |
| CHK-ESC | `escalation-checklist.md` | When agent halts and emits `BLOCKED_ON_HUMAN_ORCHESTRATOR`. | `council-architecture` | `oya-foundry-fitness-banned-primitives` (audits halt events) |

## Canonical Templates (2026-05-13 — BNF v4.1 + Workflow Studio + Clean-Arch)

Added by Wave 2 Templates partition executor. These 7 templates are the
authoritative scaffold for all planning and product artifacts. Starting
from these templates is mandatory per `feedback_autonomous_implementation_artifacts.md`.

| ID | File | Purpose (1-line) | Owner | Enforcing fitness lane |
|---|---|---|---|---|
| TPL-ADR | `adr-template.md` | ADR with BNF v4.1 naming-justification block, Bominal inheritance citation, clean-arch impact section, and concrete file-path consequences. SUPERSEDES `adr-template-v2.md` for new ADRs. | `council-architecture` | `oya-foundry-fitness-adr-shape` |
| TPL-PHASE-SPEC | `phase-spec-template.md` | Phase spec with entry/exit gates, in/out scope, impl-plan list, acceptance commands, clean-arch compliance block, grit symbols, ICM fields. | `council-architecture` | `oya-foundry-fitness-plan-hierarchy` |
| TPL-IMPL | `impl-plan-template.md` | Implementation plan with concrete file targets, crate BNF justification, code skeleton, clean-arch compliance block, load-test section, grit symbols, ICM payload, halt conditions. SUPERSEDES `implementation-plan-template.md` for new IPs. | `council-architecture` | `oya-foundry-fitness-plan-hierarchy` |
| TPL-PRD | `prd-template.md` | PRD with functional/non-functional requirements, BC layer map, port traits in kernel, Workflow+Ontology integration, Competitive Benchmark, Performance Targets, Horizontal Scalability. | `council-architecture` | `oya-check-benchmark-cli` + `oya-check-perf-budget-cli` |
| TPL-MS | `microservice-template.md` | µservice scaffold: naming-justification, Cargo workspace entries, 12-layer skeleton, BC list, `[package.metadata.oya]` schema, test scaffolding, grit symbols. | `council-architecture` | `oya-foundry-fitness-plan-hierarchy` |
| TPL-BC-REG | `bounded-context-registration-template.md` | BC registration schema for `docs/standards/bounded-contexts.md`: name justification, owner µservice, entities, Workflow events, Ontology types, acceptance criteria. | `council-architecture` | `oya-shared-bounded-contexts-check-cli` (LEAN-A2) |
| TPL-MILE-README | `milestone-readme-template.md` | Milestone README for `.omc/plans/milestones/M0X-*/README.md`: intent, entry/exit gates, phase index table, risk register, Bominal ADR citations, agent-navigability pointer. SUPERSEDES `milestone-index-template.md` for new milestones. | `council-architecture` | `oya-foundry-fitness-plan-hierarchy` |

Glossary enforced in all 7 templates:
- "shared" not "platform" (retired per `feedback_glossary_shared_not_platform.md`)
- "Ontology" not "Object Graph" (renamed per `feedback_glossary_ontology_not_object_graph.md`)
- "Application" not "Shell" (override #8 per `feedback_bominal_inheritance_precedence.md`)
- flat µservice catalog; no "Product Group" / "Arm"
- BNF v4.1: `oya-<microservice>[-<bc-tokens>]-<layer>` (no `shared|vertical` slot)

---

## Conflicts resolved with existing oyatie docs

- `pull-request-template.md`, `adr-template.md`, `capability-record-template.yaml`, `runbook-template.md` already existed at `docs/templates/`. Lifted variants are saved as `*-v2` files and supersede the prior files once reviewed.
- `docs/templates/migration-runbook-template.md`, `docs/templates/dpia-template.md`, and `docs/templates/team-charter-template.md` are out of scope of this delivery; preserved as-is.
- No checklist conflicts: all 12 checklists are new additions to `docs/checklists/`.

## ADR citations

- **ADR-0052** — inventory ledger contract; governs CHK-INV row schema.
- **ADR-0053** — sanctioned primitives (`grit`, `icm`, `oya-tooling-agent-read`); governs CHK-KICKOFF, CHK-COMPLETE, CHK-ESC.
- **ADR-0054** — scaffold-claim pattern; governs TPL-IP grit-claim symbol requirements.
