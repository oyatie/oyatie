---
id: ADR-0052
title: "Canonical inventory ledger for the grit/icm cutover"
status: Superseded
doc_status: published
date: 2026-05-12
owners:
  - council-architecture
  - foundry
supersedes: []
superseded_by: [ADR-0118]
doc_class: DecisionRecord
purpose: |
  Canonical inventory ledger for the grit/icm cutover; classifies every file/dir/script under oyatie/ and bominal/agents/ + bominal/docs/ scope by closed-set action (KEEP/KEEP+ANNOTATE/REPLACE-WITH-GRIT/REPLACE-WITH-ICM/REPLACE-WITH-HELPER/ARCHIVE/DELETE/FLAG-FOR-USER).
planned_enforcement_ref: oya-governance-inventory-tracker
related:
  - ADR-0053
  - ADR-0054
tags:
  - cross-cutting
  - tooling
  - inventory
  - grit
  - icm
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0052: Canonical Inventory Ledger for the grit/icm Cutover

> **Status:** Accepted
> **Date:** 2026-05-12
> **Owners:** `council-architecture`, `foundry` — see [`teams/`](../teams/)
> **Supersedes:** — **Superseded-by:** —
> **Siblings (parallel wave):** ADR-0053 (sanctioned primitives), ADR-0054 (scaffold-claim)

---

## Status

Accepted. This ADR satisfies acceptance criterion **A2** of `.omc/plans/ralplan-oyatie-sst-consolidation.md`: "A canonical inventory ledger ADR classifies every file/dir/script in scope by closed-set action before any deletion or archive move executes."

Planned enforcement: `oya-governance-inventory-tracker` remains advisory until the CI lane exists.

---

## Context

The grit/icm cutover (per `.omc/plans/ralplan-oyatie-sst-consolidation.md`, Option A — strict-phased, archive-first) requires that **no artifact leaves the active path without a committed ADR row classifying it**. This is §Constraints item 3 of the plan, stated as: "Inventory precedes deletion."

Before this ADR, no single authoritative ledger classified all 223 artifact rows now in scope. The `ultragoal/` orchestration glue (`ledger.jsonl`, `goals.json`, `codex-goal-*.json`, `G004-reconciliation-blocker.md`, `PAUSE.md`) was treated as live coordination state even though its semantic content is wholly absorbed by `grit claim`/`grit done`/`grit watch` state and `icm store -t goals-oyatie`. The absence of a committed classification ledger created two risks:

1. **Premature deletion** — an agent or human could remove an archive-class artifact before its function was confirmed to be replaced elsewhere.
2. **Missing-boundary ambiguity** — seven cross-boundary artifacts between `bominal/` and `oyatie/` lacked a disposition ruling, leaving their authority-home undefined.

The source data for this ledger was gathered in `.omc/scratch/inventory-draft-oyatie-cutover.md` (READ-ONLY; that file is not modified by this ADR's lift).

A Critic iter-2 finding corrected one phantom-path entry: `oyatie/.omx/ultragoal/` does not exist in the repository; the entry is noted as "phantom path; not present; no action" in the ledger below.

---

## Decision

This ADR **IS** the inventory ledger. The table in the §Inventory Ledger section below is the canonical, authoritative, committed classification of every in-scope artifact. It uses only values from the closed action set:

`KEEP` | `KEEP+ANNOTATE` | `REPLACE-WITH-GRIT` | `REPLACE-WITH-ICM` | `REPLACE-WITH-HELPER` | `ARCHIVE` | `DELETE` | `FLAG-FOR-USER`

Each inventory row carries an `Archived at` column. The value remains `null` until P6 stamps ARCHIVE-class rows with the archive timestamp; planned future target rows that do not exist yet use `n/a`.

Rules that follow from this ADR:

1. **No ARCHIVE-class artifact MAY be moved** until this ADR is merged to `main`.
2. **No DELETE-class artifact MAY be removed** until the corresponding ARCHIVE move has its own merged PR (per plan §P6→P7 gate).
3. **KEEP+ANNOTATE artifacts MUST receive their annotation** no later than plan phase P4 (agent-instruction rewrite).
4. **FLAG-FOR-USER items** are not actioned by automated agents; they require explicit human decision before any agent touches them.
5. The `oya-governance-inventory-tracker` lane **MUST** verify that every row with `Classification ≠ KEEP` has a corresponding successor-IP issue or completed-phase marker before P7 deletion is permitted to merge.

---

## Decision Drivers

1. **Inventory precedes deletion** — §Constraints item 3 of the RALPLAN plan; the temporal ordering is a hard requirement, not a convention.
2. **Rollback safety** — archiving orchestration glue before deleting it keeps `git revert` cheap; a committed ledger means the pre-deletion state is always recoverable without archaeological git-log search.
3. **Agent-flow correctness** — agents executing subsequent plan phases consult this ADR to determine which artifacts are safe to reference, modify, or ignore; an ambiguous disposition blocks agent execution.

---

## Alternatives Considered

**Alt 1 — Inline comments in the plan file (`.omc/plans/ralplan-oyatie-sst-consolidation.md`).**
- Pros: no new file; decisions co-located with phasing.
- Cons: plan files are READ-ONLY per agent convention; comments in plan files are not tracked as authoritative decisions; `oya-governance-inventory-tracker` cannot validate a non-ADR format. Rejected: violates §Constraints item 2 (authoritative = repo-tracked in `docs/`).

**Alt 2 — A standalone JSON ledger at `docs/machine-readable/inventory-grit-cutover.json`.**
- Pros: machine-parseable natively; easier for CI lane to query.
- Cons: JSON has no narrative context; supersession graph and fitness-lane enforcement require the ADR frontmatter shape; a separate JSON file would need a companion ADR anyway. Rejected: the ADR itself with the embedded table is the canonical shape; the fitness lane can parse the markdown table.

**Alt 3 — Status quo (no ledger; proceed phase-by-phase trusting the plan).**
- Pros: no work required.
- Cons: violates §Constraints item 3 literally; the two risk scenarios described in §Context (premature deletion, boundary ambiguity) remain live. Rejected by plan consensus.

---

## Why Chosen

This ADR satisfies:

- **(a) Spec acceptance criterion A2** — "A canonical inventory ledger ADR classifies every file/dir/script in scope." This ADR is that document.
- **(b) Master Plan principles** — P1 (inventory precedes deletion), P3 (authoritative = repo-tracked), P7 (reshape data to eliminate special cases: `G004-reconciliation-blocker.md` and `PAUSE.md` disappear because grit's data model has no place for them, not because we add a shim).
- **(c) Prior ADRs** — builds on ADR-0015 (flat-crates; explains why all `crates/` entries are KEEP), ADR-0019 (doc catalog; explains why all `docs/` entries are KEEP), ADR-0025 (Foundry as engineering platform; why `oya-governance-*` lanes are the enforcement vehicle).
- **(d) Beats alternatives** — Alt 1 breaks the authoritative-tracking invariant; Alt 2 produces an orphaned JSON with no narrative; Alt 3 is the failure mode the plan was written to prevent.

Sibling ADRs ADR-0053 (sanctioned primitives closed set) and ADR-0054 (grit scaffold-claim pattern) land in the same wave and are cross-cited here because their acceptance is a precondition for the REPLACE-WITH-GRIT and REPLACE-WITH-ICM action classes to have executable meaning.

---

## Consequences

### Positive

- Every subsequent plan phase (P3–P10) can cite a stable ADR row as authority for why a specific artifact is being archived, annotated, or deleted.
- `oya-governance-inventory-tracker` has a parseable, versioned source of truth; classification drift is detectable as a CI failure.
- The seven cross-boundary artifacts have explicit dispositions; the `bominal` ↔ `oyatie` boundary ambiguity is resolved before any doc rewrites begin.
- The phantom-path finding (`oyatie/.omx/ultragoal/` does not exist) is committed as a fact, preventing a future agent from wasting time searching for it.

### Negative

- The ledger now covers 223 existing artifact rows across two repositories; any future artifact added to either scope without a corresponding ledger update will trigger an `oya-governance-inventory-tracker` gap warning. This requires process discipline on all contributors.
- The 15 ARCHIVE-class rows cannot be moved until this ADR merges, which is a hard sequencing constraint that blocks P6 in the plan.
- Maintaining the ledger in a markdown table limits programmatic query ergonomics; the `oya-governance-inventory-tracker` lane must implement its own markdown-table parser.

### Neutral

- The existing-artifact classification counts (201 KEEP, 5 KEEP+ANNOTATE, 15 ARCHIVE, 2 DELETE, 0 FLAG-FOR-USER, 0 REPLACE-WITH-*) reflect the reconciled ledger state after the 2026-05-14 archive move and exclude planned future target rows. The helper target `tools/oya-tooling-agent-read/` is recorded separately as a planned `REPLACE-WITH-HELPER` row because it is created in P2, after this inventory ADR lands.
- The P7 review reconciled one stale file-specific KEEP row for an absent Bominal evidence log: the evidence directory is authoritative and KEEP, but that specific file was absent before P7 cleanup and is not counted as an existing artifact row.
- All `oyatie/crates/` artifacts are KEEP; the flat-crates architecture per ADR-0015 is not disturbed by this cutover.

---

## Inventory Ledger

**Classification closed set:** `KEEP` | `KEEP+ANNOTATE` | `REPLACE-WITH-GRIT` | `REPLACE-WITH-ICM` | `REPLACE-WITH-HELPER` | `ARCHIVE` | `DELETE` | `FLAG-FOR-USER`

**Existing-artifact summary counts** (excludes planned future target rows):

| Classification | Count |
|---|---|
| KEEP | 201 |
| KEEP+ANNOTATE | 5 |
| REPLACE-WITH-GRIT | 0 |
| REPLACE-WITH-ICM | 0 |
| REPLACE-WITH-HELPER | 0 |
| ARCHIVE | 15 |
| DELETE | 2 |
| FLAG-FOR-USER | 0 |
| **TOTAL** | **223** |

**Planned target rows** (not included in the existing-artifact count):

| Path | Type | Classification | Archived at | Maps to spec criterion | Notes |
|---|---|---|---|---|---|
| oyatie/tools/oya-tooling-agent-read/ | dir | REPLACE-WITH-HELPER | n/a | A4 | New read-only, audit-emitting helper target; scaffolded in P2 via ADR-0054; replaces agent read-side `git`/`gh` access with sanctioned `log`, `diff`, `pr-view`, and `pr-comments` verbs. |

---

### oyatie/ — Root-level files

| Path | Type | Classification | Archived at | Maps to spec criterion | Notes |
|---|---|---|---|---|---|
| oyatie/Cargo.toml | file | KEEP | null | A8 | Workspace manifest; flat-crates architecture preserved per ADR-0015 |
| oyatie/Cargo.lock | file | KEEP | null | A8 | Dependency lock file; authoritative |
| oyatie/deny.toml | file | KEEP | null | A8 | Supply-chain policy per ADR-0039 |
| oyatie/README.md | file | KEEP | null | A8 | Project summary |
| oyatie/CLAUDE.md | file | KEEP+ANNOTATE | null | A5 | Agent-instruction home; needs rewrite to remove rtk git/gh references; add sanctioned-primitives section naming grit+icm+oya-tooling-agent-read |
| oyatie/AGENTS.md | file | KEEP+ANNOTATE | null | A5 | Agent-instruction redirect to docs/AGENTS.md; same annotation needs as CLAUDE.md |
| oyatie/.aider.conventions.md | file | KEEP | null | A8 | Code convention guidance |
| oyatie/.gitignore | file | KEEP | null | A8 | Version-control housekeeping |
| oyatie/.windsurfrules | file | KEEP | null | A8 | Windsurf IDE configuration |
| oyatie/WINUI3_KOREAN_PAYROLL_minimum-shippable-tier_PROMPT.md | file | KEEP | null | A8 | Product-context reference; not authoritative SoT |

### oyatie/ — Root-level directories (core)

| Path | Type | Classification | Archived at | Maps to spec criterion | Notes |
|---|---|---|---|---|---|
| oyatie/crates/ | dir | KEEP | null | A8 | 142 crates including 7 suspect oya-foundry-*-kernel crates; all KEEP per Constraint 6 (fitness/policy kernels, not coordination kernels); flat-crates architecture per ADR-0015 |
| oyatie/docs/ | dir | KEEP | null | A8 | Canonical product-content SoT per Layer 1; all subdirs + 140+ files KEEP |
| oyatie/scripts/ | dir | KEEP | null | A8 | Build/lint/release helpers (5 scripts); humans + sanctioned CI only; KEEP |
| oyatie/contracts/ | dir | KEEP | null | A8 | Cross-axis contract files (OpenAPI/Proto/AsyncAPI); 20+ files; KEEP |
| oyatie/registry/ | dir | KEEP | null | A8 | Catalog + capability records; machine-readable registry; KEEP |
| oyatie/registry/capability-templates/ | dir | KEEP | null | A8 | Capability templates plus eval-runs/eval-sets; KEEP |
| oyatie/infra/ | dir | KEEP | null | A8 | Policy-as-code (kyverno); 1 file; KEEP |

### oyatie/ — Hidden/session directories

| Path | Type | Classification | Archived at | Maps to spec criterion | Notes |
|---|---|---|---|---|---|
| oyatie/.grit/ | dir | KEEP | null | A8 | grit local state (worktrees, locks, symbols); .gitignored session ephemera; KEEP (managed by grit itself) |
| oyatie/.omc/ | dir | KEEP | null | A8 | OMC plans + state; .gitignored for state subdirs; non-authoritative; KEEP (session-scoped tooling) |
| oyatie/.omx/ | dir | KEEP | null | A8 | Working state only (metrics.json, notepad.md); .gitignored; non-authoritative ephemera; KEEP |
| oyatie/.omx/ultragoal/ | dir | — | n/a | — | **Phantom path; not present in repository; no action.** (Critic iter-2 finding: this path does not exist.) |
| oyatie/.rtk/ | dir | KEEP | null | A8 | RTK token filters (filters.toml); personal config; KEEP |
| oyatie/.github/ | dir | KEEP | null | A8 | GitHub Actions + Copilot instructions; (1 file: copilot-instructions.md); KEEP |

---

### oyatie/docs/ — Top-level authority files

| Path | Type | Classification | Archived at | Maps to spec criterion | Notes |
|---|---|---|---|---|---|
| oyatie/docs/CONSTITUTION.md | file | KEEP | null | A1, A8 | Project frame; canonical product authority; declares authority chain per ADR-0001 |
| oyatie/docs/PRD.md | file | KEEP+ANNOTATE | null | A1 | 33.4K; canonical product PRD (7 axes); KEEP+ANNOTATE: add bidirectional cite to bominal/docs/consolidated/PRD.md as portfolio parent |
| oyatie/docs/DESIGN.md | file | KEEP | null | A8 | 72.6K; canonical architecture design |
| oyatie/docs/SPEC.md | file | KEEP | null | A8 | 43.6K; product specification |
| oyatie/docs/ROADMAP.md | file | KEEP | null | A8 | Product roadmap; gates Foundry on Foundation completion |
| oyatie/docs/README.md | file | KEEP | null | A8 | Docs portal homepage |
| oyatie/docs/ADR-INDEX.md | file | KEEP | null | A8 | Master index of all ADRs; must be updated with ADR-0052 (this ADR), ADR-0053, ADR-0054 |
| oyatie/docs/ADR-CONSOLIDATION-PLAN.md | file | KEEP | null | A8 | ADR consolidation strategy |
| oyatie/docs/ADR-LEGACY-REGRESSION-MAPPING.md | file | KEEP | null | A8 | Legacy-to-current mapping; 43.6K |
| oyatie/docs/CHANGELOG.md | file | KEEP | null | A8 | Version history |

### oyatie/docs/ — Quality machinery

| Path | Type | Classification | Archived at | Maps to spec criterion | Notes |
|---|---|---|---|---|---|
| oyatie/docs/CONTRADICTION-LEDGER.md | file | KEEP | null | A8 | 77 tracked contradictions; OPEN ledger entries (LEDG-008/017/021/024) remain open per Constraint 9 |
| oyatie/docs/MISTAKES-LEDGER.md | file | KEEP | null | A8 | 13 active mistakes; each backed by CI fitness lane |
| oyatie/docs/RACI-OWNERSHIP.md | file | KEEP | null | A8 | Ownership mapping; authority cohesion enforcement per ADR-0001 |

### oyatie/docs/ — Product-quality & governance

| Path | Type | Classification | Archived at | Maps to spec criterion | Notes |
|---|---|---|---|---|---|
| oyatie/docs/COMPLIANCE-MATRIX.md | file | KEEP | null | A8 | Compliance tracking |
| oyatie/docs/COMPETITIVE-GAP-ANALYSIS.md | file | KEEP | null | A8 | Market position analysis |
| oyatie/docs/DOC-CATALOG.md | file | KEEP | null | A8 | Documentation inventory per ADR-0019 |
| oyatie/docs/DOC-UPDATE-PROTOCOL.md | file | KEEP | null | A8 | Doc maintenance protocol per ADR-0019 |
| oyatie/docs/DOCUMENTATION.md | file | KEEP | null | A8 | Documentation guide |
| oyatie/docs/FINOPS-PLAN.md | file | KEEP | null | A8 | Financial operations roadmap |
| oyatie/docs/GLOSSARY.md | file | KEEP | null | A8 | Terminology canon per ADR-0018 |
| oyatie/docs/GTM-PLAN.md | file | KEEP | null | A8 | Go-to-market strategy |
| oyatie/docs/HIRING-CAPACITY-PLAN.md | file | KEEP | null | A8 | Staffing roadmap |
| oyatie/docs/INCIDENT-MANAGEMENT.md | file | KEEP | null | A8 | Incident response policy |
| oyatie/docs/INTERNATIONALIZATION.md | file | KEEP | null | A8 | i18n strategy; Korean morphology per ADR-0048 |
| oyatie/docs/LEGAL-IP-LEDGER.md | file | KEEP | null | A8 | IP + legal tracking |
| oyatie/docs/PRIVACY-PROGRAM.md | file | KEEP | null | A8 | Privacy governance; 25KB |
| oyatie/docs/QA-TEST-STRATEGY.md | file | KEEP | null | A8 | Test strategy |
| oyatie/docs/RELEASE-MANAGEMENT.md | file | KEEP | null | A8 | Release process per ADR-0041 |
| oyatie/docs/RISK-REGISTER.md | file | KEEP | null | A8 | Risk ledger |
| docs/security-program/security-program.json | file | KEEP | null | A8 | Security governance |
| docs/security-program/OWNERS | file | KEEP | null | A8 | Security governance subtree ownership marker (ops-security) |
| oyatie/docs/SLO-CATALOG.md | file | KEEP | null | A8 | Service-level objectives |
| oyatie/docs/STANDARDS-AND-TEMPLATES.md | file | KEEP | null | A8 | Standards index |
| oyatie/docs/TOOLCHAIN.md | file | KEEP | null | A8 | Engineering tooling guide |
| oyatie/docs/VENDOR-PARTNER-LEDGER.md | file | KEEP | null | A8 | Vendor + partner tracking |
| oyatie/docs/RUNBOOKS-INDEX.md | file | KEEP | null | A8 | Index to 200+ operational runbooks |
| oyatie/docs/AGENTS.md | file | KEEP+ANNOTATE | null | A5 | Agent instruction home; redirect-class; same annotation needs as root AGENTS.md |

### oyatie/docs/decisions/ — ADR files

| Path | Type | Classification | Archived at | Maps to spec criterion | Notes |
|---|---|---|---|---|---|
| oyatie/docs/decisions/ADR-0001 through ADR-0051 | files (51×) | KEEP | null | A8 | All accepted architectural decisions; KEEP unchanged |
| oyatie/docs/decisions/RETIRED.md | file | KEEP | null | A8 | Retirement record for superseded ADRs |
| oyatie/docs/decisions/README.md | file | KEEP | null | A8 | ADR README |

### oyatie/docs/checklists/ (24 files)

| Path | Type | Classification | Archived at | Maps to spec criterion | Notes |
|---|---|---|---|---|---|
| oyatie/docs/checklists/ | dir | KEEP | null | A8 | Operational checklists (adr-promotion, audit-readiness, build-vs-buy, etc.); 24 files; all KEEP |

### oyatie/docs/products/ (axis PRDs)

| Path | Type | Classification | Archived at | Maps to spec criterion | Notes |
|---|---|---|---|---|---|
| oyatie/docs/products/ | dir | KEEP | null | A8 | 7-axis + 14-vertical product family; 17 PRDs + 1 template + README; all KEEP |
| oyatie/docs/products/_TEMPLATE.md | file | KEEP | null | A8 | Axis PRD template |
| oyatie/docs/products/saas-platform/PRD.md | file | KEEP | null | A8 | SaaS Axis PRD |
| oyatie/docs/products/foundry/PRD.md | file | KEEP | null | A8 | Foundry Axis PRD; engineering platform per ADR-0025 |
| oyatie/docs/products/workspace/PRD.md | file | KEEP | null | A8 | Workspace Axis PRD (Axis 2 per 2026-05-09 reframing) |
| oyatie/docs/products/cloud/PRD.md | file | KEEP | null | A8 | Cloud Axis PRD |
| oyatie/docs/products/search/PRD.md | file | KEEP | null | A8 | Search Axis PRD |
| oyatie/docs/products/ads-analytics/PRD.md | file | KEEP | null | A8 | Ads + Analytics Axis PRD |
| oyatie/docs/products/vertical-*/PRD.md | files (14×) | KEEP | null | A8 | 14 vertical-industry PRDs (healthcare, fintech, agriculture, construction, etc.); all KEEP |

### oyatie/docs/regional-packs/

| Path | Type | Classification | Archived at | Maps to spec criterion | Notes |
|---|---|---|---|---|---|
| oyatie/docs/regional-packs/ | dir | KEEP | null | A8 | Region-specific regulatory/compliance packs per ADR-0010 |
| oyatie/docs/regional-packs/oya-pack-kr/PACK.md | file | KEEP | null | A8 | Korean regional pack (fintech regulatory, morphology ADR-0048) |
| oyatie/docs/regional-packs/_TEMPLATE.md | file | KEEP | null | A8 | Regional pack template |

### oyatie/docs/raw/ (working drafts; non-authoritative)

| Path | Type | Classification | Archived at | Maps to spec criterion | Notes |
|---|---|---|---|---|---|
| oyatie/docs/raw/ | dir | KEEP | null | A8 | Working-draft corpus; 5 files; non-authoritative until promoted; per Lane 3 of trace, agentic-delivery-fabric-executable-prd.md becomes ground-zero for new agentic-pipeline spec (promote in-place, do not move to bominal) |
| oyatie/docs/raw/agentic-delivery-fabric-executable-prd.md | file | KEEP | null | A8 | Draft agentic-pipeline spec; cite bominal but promote in oyatie per trace finding |
| oyatie/docs/raw/agentic-delivery-foundry-critical-challenge.md | file | KEEP | null | A8 | Foundry challenge analysis |
| oyatie/docs/raw/agentic-delivery-vcs-cicd-report.md | file | KEEP | null | A8 | VCS/CI-CD assessment |
| oyatie/docs/raw/big-tech-dev-cycle-agentic-optimization.md | file | KEEP | null | A8 | Optimization study |
| oyatie/docs/raw/claude-code-backup-comprehensive-analysis.md | file | KEEP | null | A8 | Claude Code analysis |

### oyatie/docs/runbooks/ (200+ operational runbooks)

| Path | Type | Classification | Archived at | Maps to spec criterion | Notes |
|---|---|---|---|---|---|
| oyatie/docs/runbooks/ | dir | KEEP | null | A8 | 200+ runbooks (incident response, operational playbooks); all KEEP; organized by axis + cross-microservice |
| oyatie/docs/runbooks/*.md | files (200+) | KEEP | null | A8 | All incident/operational runbooks; list-only at depth 2 due to size (200+ files) |
| oyatie/docs/runbooks/ads/ | subdir | KEEP | null | A8 | ads microservice runbooks (auction-engine, click-fraud, data-use-boundary) |
| oyatie/docs/runbooks/cloud/ | subdir | KEEP | null | A8 | cloud microservice runbooks (billing, cell-isolation, DCops, IAM, KMS, region-failover) |
| oyatie/docs/runbooks/foundry/ | subdir | KEEP | null | A8 | foundry runbooks (autonomy-ceiling, capability-eval, cost-ceiling, prompt-injection, sandbox-escape) |
| oyatie/docs/runbooks/search/ | subdir | KEEP | null | A8 | search microservice runbooks (crawler, index-corruption, RTBF, SERP-quality) |
| oyatie/docs/runbooks/workspace/ | subdir | KEEP | null | A8 | microservice runbooks (doc-CRDT, drive-permission, mail, Meet SFU, recording) |
| oyatie/docs/runbooks/vertical-fintech/ | subdir | KEEP | null | A8 | Fintech vertical runbooks (AML, CDE-isolation, PCI) |
| oyatie/docs/runbooks/vertical-healthcare/ | subdir | KEEP | null | A8 | Healthcare vertical runbooks (clinical-safety, PHI-leak) |
| oyatie/docs/runbooks/vertical-industrial/ | subdir | KEEP | null | A8 | Industrial vertical runbooks (OT-safety) |
| oyatie/docs/runbooks/vertical-logistics/ | subdir | KEEP | null | A8 | Logistics vertical runbooks (EDI-counterparty) |
| oyatie/docs/runbooks/cross-microservice/ | subdir | KEEP | null | A8 | Cross-axis coordination runbooks (audit-chain-integrity, cohesion-fitness, DSR-cascade, regional-pack) |

### oyatie/docs/site/ (public documentation site)

| Path | Type | Classification | Archived at | Maps to spec criterion | Notes |
|---|---|---|---|---|---|
| oyatie/docs/site/ | dir | KEEP | null | A8 | Public docs site source (mdBook) |
| oyatie/docs/site/src/SUMMARY.md | file | KEEP | null | A8 | Site navigation |
| oyatie/docs/site/src/introduction.md | file | KEEP | null | A8 | Introduction |
| oyatie/docs/site/src/concepts/cohesion-thesis.md | file | KEEP | null | A8 | Foundational concept |
| oyatie/docs/site/src/guides/ | subdir | KEEP | null | A8 | Operator guides (operate-a-tenant, etc.) |
| oyatie/docs/site/src/tutorials/ | subdir | KEEP | null | A8 | First-capability tutorial |
| oyatie/docs/site/src/admin/ | subdir | KEEP | null | A8 | Tenant admin guide |
| oyatie/docs/site/src/plugins/ | subdir | KEEP | null | A8 | Plugin authoring guide |
| oyatie/docs/site/src/studio/ | subdir | KEEP | null | A8 | Workflow studio guide |

### oyatie/docs/standards/ (21 standard documents)

| Path | Type | Classification | Archived at | Maps to spec criterion | Notes |
|---|---|---|---|---|---|
| oyatie/docs/standards/ | dir | KEEP | null | A8 | Engineering standards; all KEEP |
| oyatie/docs/standards/api-design.md | file | KEEP | null | A8 | API design standard per ADR-0037 |
| oyatie/docs/standards/code-style.md | file | KEEP | null | A8 | Rust/code style guide |
| oyatie/docs/standards/code-review.md | file | KEEP | null | A8 | Code review process |
| oyatie/docs/standards/commit-message.md | file | KEEP | null | A8 | Commit message convention |
| oyatie/docs/standards/testing.md | file | KEEP | null | A8 | Testing standard |
| oyatie/docs/standards/security-review.md | file | KEEP | null | A8 | Security review checklist |
| oyatie/docs/standards/privacy-review.md | file | KEEP | null | A8 | Privacy review checklist |
| oyatie/docs/standards/schema-migration.md | file | KEEP | null | A8 | DB migration playbook |
| oyatie/docs/standards/release.md | file | KEEP | null | A8 | Release procedure per ADR-0041 |
| oyatie/docs/standards/incident-severity.md | file | KEEP | null | A8 | Severity classification per incident-management |
| oyatie/docs/standards/on-call.md | file | KEEP | null | A8 | On-call runbook |
| oyatie/docs/standards/capability-authoring.md | file | KEEP | null | A8 | Foundry capability authoring per ADR-0021 |
| oyatie/docs/standards/plugin-authoring.md | file | KEEP | null | A8 | Plugin substrate authoring per ADR-0036 |
| oyatie/docs/standards/ci-lanes.md | file | KEEP | null | A8 | CI lane definitions (fitness lanes per ADR-0003) |
| oyatie/docs/standards/doc-style.md | file | KEEP | null | A8 | Documentation style guide |
| oyatie/docs/standards/error-handling.md | file | KEEP | null | A8 | Error handling convention |
| oyatie/docs/standards/logging-tracing.md | file | KEEP | null | A8 | Observability standard per ADR-0042 |
| oyatie/docs/standards/fintech-compliance.md | file | KEEP | null | A8 | Fintech regulatory compliance |
| oyatie/docs/standards/prevention-doctrine.md | file | KEEP | null | A8 | Prevention-first operational philosophy |
| oyatie/docs/standards/migration-playbook.md | file | KEEP | null | A8 | Schema/service migration |
| oyatie/docs/standards/brand-voice.md | file | KEEP | null | A8 | Brand voice standard |

### oyatie/docs/teams/ (21 team charters)

| Path | Type | Classification | Archived at | Maps to spec criterion | Notes |
|---|---|---|---|---|---|
| oyatie/docs/teams/ | dir | KEEP | null | A8 | Team ownership charters; all KEEP |
| oyatie/docs/teams/README.md | file | KEEP | null | A8 | Teams index |
| oyatie/docs/teams/axis-*/CHARTER.md | files (7×) | KEEP | null | A8 | 7-axis team charters (SaaS, Workspace, Foundry, Cloud, Search, Ads, vertical) |
| oyatie/docs/teams/council-*/CHARTER.md | files (2×) | KEEP | null | A8 | Council charters (Architecture, Privacy) |
| oyatie/docs/teams/platform-*/CHARTER.md | files (5×) | KEEP | null | A8 | Platform team charters (API/SDK, audit/evidence, eventing, privacy/DUB, tenancy/identity) |
| oyatie/docs/teams/ops-*/CHARTER.md | files (5×) | KEEP | null | A8 | Ops team charters (compliance, DR, finops, security, SRE/reliability) |
| oyatie/docs/teams/gtm-*/CHARTER.md | files (4×) | KEEP | null | A8 | GTM team charters (customer-success, marketing, partnerships, sales/SE) |
| oyatie/docs/teams/crew-adr-promotion/CHARTER.md | file | KEEP | null | A8 | ADR promotion crew charter |
| oyatie/docs/teams/tactical-first-vertical-pilot/CHARTER.md | file | KEEP | null | A8 | Vertical pilot team charter |
| oyatie/docs/teams/regional-packs/CHARTER.md | file | KEEP | null | A8 | Regional packs team charter |
| oyatie/docs/teams/vertical-*/CHARTER.md | files (14×) | KEEP | null | A8 | 14 vertical team charters (healthcare, fintech, agricultural, etc.) |

### oyatie/docs/templates/ (9 templates)

| Path | Type | Classification | Archived at | Maps to spec criterion | Notes |
|---|---|---|---|---|---|
| oyatie/docs/templates/ | dir | KEEP | null | A8 | Document templates; all KEEP |
| oyatie/docs/templates/adr-template.md | file | KEEP | null | A8 | ADR template |
| oyatie/docs/templates/adr-supersession-template.md | file | KEEP | null | A8 | ADR supersession template |
| oyatie/docs/templates/dpia-template.md | file | KEEP | null | A8 | Data-protection impact assessment template |
| oyatie/docs/templates/evidence-pack-template.md | file | KEEP | null | A8 | Regulatory evidence-pack template |
| oyatie/docs/templates/incident-postmortem-template.md | file | KEEP | null | A8 | Incident postmortem template |
| oyatie/docs/templates/migration-runbook-template.md | file | KEEP | null | A8 | Migration runbook template |
| oyatie/docs/templates/pull-request-template.md | file | KEEP | null | A8 | PR template |
| oyatie/docs/templates/regional-pack-template.md | file | KEEP | null | A8 | Regional pack template |
| oyatie/docs/templates/runbook-template.md | file | KEEP | null | A8 | Runbook template |
| oyatie/docs/templates/team-charter-template.md | file | KEEP | null | A8 | Team charter template |
| oyatie/docs/templates/threat-model-template.md | file | KEEP | null | A8 | Threat model template |

### oyatie/docs/wiki/

| Path | Type | Classification | Archived at | Maps to spec criterion | Notes |
|---|---|---|---|---|---|
| oyatie/docs/wiki/quickref/README.md | file | KEEP | null | A8 | Quick reference index |

### oyatie/docs/machine-readable/

| Path | Type | Classification | Archived at | Maps to spec criterion | Notes |
|---|---|---|---|---|---|
| oyatie/docs/machine-readable/ | dir | KEEP | null | A8 | Machine-readable artifact mirrors (auto-generated); KEEP |

---

### bominal/docs/consolidated/ (portfolio parent PRD + artifacts)

| Path | Type | Classification | Archived at | Maps to spec criterion | Notes |
|---|---|---|---|---|---|
| bominal/docs/consolidated/PRD.md | file | KEEP+ANNOTATE | null | A1 | 97.2K; portfolio-parent PRD (7 axes, brand "Oyatie", oyatie.com); KEEP+ANNOTATE: add bidirectional cite to oyatie/docs/PRD.md as canonical implementation home |
| bominal/docs/consolidated/CONSTITUTION.md | file | KEEP | null | A8 | Portfolio constitution; cross-cites oyatie authority chain |
| bominal/docs/consolidated/README.md | file | KEEP | null | A8 | Portfolio docs README |
| bominal/docs/consolidated/*.md | files (30×) | KEEP | null | A8 | All other consolidated docs (GLOSSARY, COMPLIANCE, ROADMAP, ADR-INDEX, standards, etc.); no modification needed; all KEEP |

### bominal/docs/ (other subdirs — depth-2 listing)

| Path | Type | Classification | Archived at | Maps to spec criterion | Notes |
|---|---|---|---|---|---|
| bominal/docs/agents/ | dir | KEEP | null | A8 | Agent-corpus docs; unchanged by cutover |
| bominal/docs/architecture/ | dir | KEEP | null | A8 | Architecture decision lanes + templates |
| bominal/docs/business/ | dir | KEEP | null | A8 | Business strategy + competitive analysis |
| bominal/docs/design-system/ | dir | KEEP | null | A8 | Design system docs |
| bominal/docs/domain-atlas/ | dir | KEEP | null | A8 | Domain knowledge organization |
| bominal/docs/engineering/ | dir | KEEP | null | A8 | Engineering playbooks + audits |
| bominal/docs/handbook/ | dir | KEEP | null | A8 | Company handbook |
| bominal/docs/healthcare/ | dir | KEEP | null | A8 | Healthcare-specific corpus |
| bominal/docs/integration/ | dir | KEEP | null | A8 | Integration guides |
| bominal/docs/observability/ | dir | KEEP | null | A8 | Observability strategy |
| bominal/docs/operations/ | dir | KEEP | null | A8 | Operational playbooks |
| bominal/docs/platform/ | dir | KEEP | null | A8 | Platform architecture docs |
| bominal/docs/products/ | dir | KEEP | null | A8 | Product strategy docs (other verticals) |
| bominal/docs/raw/ | dir | KEEP | null | A8 | Raw research + drafts |
| bominal/docs/rfcs/ | dir | KEEP | null | A8 | RFCs |
| bominal/docs/roadmap/ | dir | KEEP | null | A8 | Roadmap lane definitions + slices |
| bominal/docs/runbooks/ | dir | KEEP | null | A8 | Operational runbooks |
| bominal/docs/security/ | dir | KEEP | null | A8 | Security guidance |
| bominal/docs/status/ | dir | KEEP | null | A8 | Status tracking |
| bominal/docs/superpowers/ | dir | KEEP | null | A8 | Agent superpowers corpus + plans/specs |
| bominal/docs/wiki/ | dir | KEEP | null | A8 | Wiki knowledge base |

### bominal/agents/ultragoal/ — Active orchestration glue (ARCHIVE targets)

| Path | Type | Classification | Archived at | Maps to spec criterion | Notes |
|---|---|---|---|---|---|
| bominal/agents/ultragoal/ledger.jsonl | file | ARCHIVE | 2026-05-14T12:57:54Z | A3 | Orchestration ledger; moved to archive/pre-grit-cutover-2026-05-12/ then deleted; function absorbed by grit watch + lock state |
| bominal/agents/ultragoal/goals.json | file | ARCHIVE | 2026-05-14T12:57:54Z | A3 | Goal state file; moved to archive/pre-grit-cutover-2026-05-12/ then deleted; function absorbed by grit claim --intent + icm store -t goals-oyatie |
| bominal/agents/ultragoal/goals.before-stale-g001-recovery.20260509T015645Z.json | file | ARCHIVE | 2026-05-14T12:57:54Z | A3 | Backup goal state; moved to archive |
| bominal/agents/ultragoal/codex-goal-G001-active.json | file | ARCHIVE | 2026-05-14T12:57:54Z | A3 | Codex goal G001; 9× goal state files total; all moved to archive then deleted; function absorbed by grit claim + icm store |
| bominal/agents/ultragoal/codex-goal-G001-fresh-reconciliation.json | file | ARCHIVE | 2026-05-14T12:57:54Z | A3 | Codex goal variant |
| bominal/agents/ultragoal/codex-goal-G001-stop-10-active.json | file | ARCHIVE | 2026-05-14T12:57:54Z | A3 | Codex goal variant |
| bominal/agents/ultragoal/codex-goal-G001-stop-10-null.json | file | ARCHIVE | 2026-05-14T12:57:54Z | A3 | Codex goal variant |
| bominal/agents/ultragoal/codex-goal-G001-stop-hook-retry.json | file | ARCHIVE | 2026-05-14T12:57:54Z | A3 | Codex goal variant |
| bominal/agents/ultragoal/codex-goal-G002-active.json | file | ARCHIVE | 2026-05-14T12:57:54Z | A3 | Codex goal G002 |
| bominal/agents/ultragoal/codex-goal-G002-final-complete.json | file | ARCHIVE | 2026-05-14T12:57:54Z | A3 | Codex goal G002 variant |
| bominal/agents/ultragoal/codex-goal-G004-paused-mismatch.json | file | ARCHIVE | 2026-05-14T12:57:54Z | A3 | Codex goal G004 |
| bominal/agents/ultragoal/codex-goal-implementation-run-blocked.json | file | ARCHIVE | 2026-05-14T12:57:54Z | A3 | Codex goal variant |
| bominal/agents/ultragoal/G004-reconciliation-blocker.md | file | ARCHIVE | 2026-05-14T12:57:54Z | A3 | Objective-state mismatch marker; not needed under grit (no objective-state concept); moved to archive then deleted |
| bominal/agents/ultragoal/PAUSE.md | file | ARCHIVE | 2026-05-14T12:57:54Z | A3 | Agent pause marker; not a grit verb; agents halt via release or TTL expiry under grit; moved to archive then deleted |
| bominal/agents/ultragoal/ledger.before-stale-g001-recovery.20260509T015645Z.jsonl | file | ARCHIVE | 2026-05-14T12:57:54Z | A3 | Backup ledger state; moved to archive |

### bominal/agents/ultragoal/ — Active planning documents (KEEP)

| Path | Type | Classification | Archived at | Maps to spec criterion | Notes |
|---|---|---|---|---|---|
| bominal/agents/ultragoal/2026-05-12-foundry-ultragoal-mega-plan.md | file | KEEP | null | A8 | Mega-plan (97.2K); active planning document; canonically versioned 2026-05-12; KEEP; reference from grit session context |
| bominal/agents/ultragoal/foundry-agentic-substrate-master.md | file | KEEP | null | A8 | Agentic substrate analysis (97.7K); active planning; KEEP |
| bominal/agents/ultragoal/brief.md | file | KEEP | null | A8 | Planning brief; KEEP |
| bominal/agents/ultragoal/oyatie-product-delivery-baseline.md | file | KEEP | null | A8 | Product baseline (pre-2026-05-09 reframing); KEEP as historical reference; note that axis count is now 7, not 6 |
| bominal/agents/ultragoal/oyatie-product-delivery-implementation-plan.md | file | KEEP | null | A8 | Implementation plan (44.4K); cross-boundary candidate per trace Lane 2; per spec, KEEP-IN-PLACE in bominal (do not copy to oyatie); add forward-ref from oyatie/docs/README.md |
| bominal/agents/ultragoal/latest-source-register.md | file | KEEP | null | A8 | Regulatory sourcing (23.7K); cross-boundary per trace; per spec, KEEP-IN-PLACE in bominal + compress to thin oyatie pointer; add cite from oyatie/docs/ |
| bominal/agents/ultragoal/README.md | file | KEEP | null | A8 | Directory README |
| bominal/agents/ultragoal/requirement-trace.md | file | KEEP | null | A8 | Requirement traceability |
| bominal/agents/ultragoal/validator-inventory.md | file | KEEP | null | A8 | Validator inventory |
| bominal/agents/ultragoal/ci-agentic-flow.json | file | KEEP | null | A8 | CI flow state (155.3K); active metadata; KEEP |
| bominal/agents/ultragoal/ci-agentic-flow.md | file | KEEP | null | A8 | CI flow documentation |
| bominal/agents/ultragoal/final-delivery-evidence.md | file | KEEP | null | A8 | Evidence summary; KEEP for audit trail |
| bominal/agents/ultragoal/implementation-docs-final-evidence.md | file | KEEP | null | A8 | Implementation evidence; KEEP |
| bominal/agents/ultragoal/implementation-docs-quality-gate.json | file | KEEP | null | A8 | Quality gate metadata; KEEP |
| bominal/agents/ultragoal/final-readiness-20260512T034457Z.json | file | KEEP | null | A8 | Readiness metadata (timestamp-tagged); KEEP |

### bominal/agents/ultragoal/ — Evidence + subdirs

| Path | Type | Classification | Archived at | Maps to spec criterion | Notes |
|---|---|---|---|---|---|
| bominal/agents/ultragoal/evidence/ | dir | KEEP | null | A8 | Evidence trail (logs, analysis, decisions); KEEP for audit |
| bominal/agents/ultragoal/evidence/ | dir (contents) | KEEP | null | A8 | All evidence files; KEEP |

### bominal/agents/ultragoal/ — Error output files (DELETE)

| Path | Type | Classification | Archived at | Maps to spec criterion | Notes |
|---|---|---|---|---|---|
| bominal/agents/ultragoal/G001-stop-hook-complete-attempt.err | file | DELETE | null | A3 | Error log output; operational ephemera; removed during P7 cleanup at 2026-05-14T13:26:13Z |
| bominal/agents/ultragoal/G001-stop-hook-complete-attempt.out | file | DELETE | null | A3 | Output log; ephemera; removed during P7 cleanup at 2026-05-14T13:26:13Z |

### bominal/agents/ultragoal/ — Archive subdirs (pre-existing)

| Path | Type | Classification | Archived at | Maps to spec criterion | Notes |
|---|---|---|---|---|---|
| bominal/agents/ultragoal/archive/ | dir | KEEP | null | A8 | Pre-existing archive of earlier planning phases (pre-oyatie-delivery, pre-rust, planning-complete); KEEP; these are historical snapshots |
| bominal/agents/ultragoal/archive/pre-oyatie-product-delivery-20260512T013650Z/ | dir | KEEP | null | A8 | Earlier snapshot; KEEP |
| bominal/agents/ultragoal/archive/pre-rust-clean-architecture-20260512T091941Z/ | dir | KEEP | null | A8 | Earlier snapshot; KEEP |
| bominal/agents/ultragoal/archive/planning-complete-20260512T160118Z/ | dir | KEEP | null | A8 | Earlier snapshot; KEEP |

### bominal/agents/ultragoal/ — Remaining subdirs

| Path | Type | Classification | Archived at | Maps to spec criterion | Notes |
|---|---|---|---|---|---|
| bominal/agents/ultragoal/issue-priority-pipeline/ | dir | KEEP | null | A8 | Issue pipeline data; KEEP as working reference |
| bominal/agents/ultragoal/legacy/ | dir | KEEP | null | A8 | Legacy artifacts; KEEP as historical reference |
| bominal/agents/ultragoal/proof-slices/ | dir | KEEP | null | A8 | Proof slices; KEEP as evidence trail |
| bominal/agents/ultragoal/sub-plans/ | dir | KEEP | null | A8 | Sub-plan hierarchy; KEEP as planning reference |

### bominal/agents/ — Other subdirs (unchanged)

| Path | Type | Classification | Archived at | Maps to spec criterion | Notes |
|---|---|---|---|---|---|
| bominal/agents/compatibility/ | dir | KEEP | null | A8 | Compatibility tracking; unchanged |
| bominal/agents/forks/ | dir | KEEP | null | A8 | Forked codebases; unchanged |
| bominal/agents/hooks/ | dir | KEEP | null | A8 | Agent hook definitions; unchanged |
| bominal/agents/memory/ | dir | KEEP | null | A8 | Agent memory corpus; unchanged |
| bominal/agents/runtime/ | dir | KEEP | null | A8 | Agent runtime config; unchanged |
| bominal/agents/settings/ | dir | KEEP | null | A8 | Agent settings; A6 audit required (any git/gh calls must route through grit+icm+oya-tooling-agent-read) |
| bominal/agents/skills/ | dir | KEEP | null | A8 | Agent skills; A6 audit required |
| bominal/agents/specs/ | dir | KEEP | null | A8 | Agent spec docs; unchanged |

---

### Cross-boundary artifacts (FLAG-FOR-USER)

Per trace Lane 2 §Cross-boundary rule audit:

| Source | Destination | Scope | Boundary | Action | Flag Reason |
|---|---|---|---|---|---|
| oyatie/CLAUDE.md RTK section | Remove agent-side `rtk git`/`rtk gh` references | shared-config → updated | same-scope | KEEP+ANNOTATE oyatie/CLAUDE.md; add sanctioned-primitives section | Agent ban is project-level; global ~/ RTK is user's personal token optimization. User must decide whether to extend ban to global config. **Default: edit oyatie/ only; flag global RTK for user decision.** |
| `~/.claude/CLAUDE.md` RTK section (if agent instructions reference it) | Remove from agent flow (if applicable) | personal-config → personal-config | **OUT-OF-SCOPE** | FLAG-FOR-USER | Per spec §Non-Goals: "Rewriting ~/.claude/CLAUDE.md (user-machine config). The agentic-pipeline rules land in oyatie/CLAUDE.md and oyatie/AGENTS.md only, unless the user explicitly broadens the rule." Do not edit user global config without explicit request. |

---

### Authoritative-tracked invariant audit

Files currently in `.gitignored` paths that ANY part of the corpus treats as authoritative:

| File | Location | Status | Recommendation |
|---|---|---|---|
| (none identified) | - | - | All authoritative state appears tracked or properly ephemeral. .gitignored dirs (`.grit/`, `.omx/`, `.omc/state/`) are session-scoped ephemera or external tool state, not canonical authority. Per Constraint 2, all canonical authority is repo-tracked. |

---

## Follow-ups

1. **ADR-0053 — Sanctioned primitives closed set.** Owner: `council-architecture` + `foundry`. Lands in the same wave (parallel). Defines the executable meaning of `REPLACE-WITH-GRIT` and `REPLACE-WITH-ICM` classification classes.

2. **ADR-0054 — grit scaffold-claim pattern.** Owner: `foundry`. Lands in the same wave (parallel). Defines the `icm-coordination-lock` fallback for new-crate phases where `grit symbols` cannot index `Cargo.toml::workspace_members`.

3. **P3 — Bidirectional PRD citation.** Owner: `foundry` (P3 executor). Apply `KEEP+ANNOTATE` to `oyatie/docs/PRD.md` and `bominal/docs/consolidated/PRD.md` per rows above. Gate: this ADR merged.

4. **P4 — Agent-instruction rewrite.** Owner: `foundry` (P4 executor). Apply `KEEP+ANNOTATE` to `oyatie/CLAUDE.md`, `oyatie/AGENTS.md`, `oyatie/docs/AGENTS.md`. Gate: ADR-0053 merged (sanctioned-primitives closed set must be committed before agent-instruction rewrites reference it).

5. **P6 — Archive moves.** Owner: human orchestrator + `foundry`. Move all 15 `ARCHIVE`-class rows from `bominal/agents/ultragoal/` to `archive/pre-grit-cutover-2026-05-12/`. Gate: this ADR merged + `oya-governance-archive-orphan` lane scaffolded.

6. **P7 — Deletion.** Owner: human orchestrator. Remove 2 `DELETE`-class rows. Gate: three checks per RALPLAN pre-mortem item 2 — (a) banned-primitives lane green post-P6, (b) `oya-governance-archive-orphan` lane confirms no living references to archived paths, (c) every ARCHIVE-class row has a non-null `Archived at` timestamp in this ADR's ledger.

7. **FLAG-FOR-USER — Global RTK extension.** Owner: human principal. Decide whether to extend the agent-instruction grit/icm primitive ban to `~/.claude/CLAUDE.md`. Default per spec §Non-Goals: no; scope is `oyatie/` only until the user explicitly broadens it.

8. **`oya-governance-inventory-tracker` lane.** Owner: `foundry`. Implement markdown-table parser that validates every row in this ADR has a classification value from the closed set; emit CI failure on gap. Required before P7 deletion gate (item 6 above).

---

## References

- Source spec (READ-ONLY): `.omc/scratch/inventory-draft-oyatie-cutover.md`
- Plan: `.omc/plans/ralplan-oyatie-sst-consolidation.md` (acceptance criterion A2)
- ADR-0001: cohesion thesis — authority chain declaration
- ADR-0015: architectural flattening target — flat-crates; explains all `crates/` KEEP rows
- ADR-0019: doc catalog and update protocol — explains all `docs/` KEEP rows
- ADR-0025: Foundry as engineering platform — `oya-governance-*` enforcement vehicle
- ADR-0039: supply chain security — `deny.toml` KEEP rationale
- ADR-0053: sanctioned primitives (sibling, parallel wave) — defines REPLACE-WITH-GRIT / REPLACE-WITH-ICM action classes
- ADR-0054: grit scaffold-claim pattern (sibling, parallel wave) — icm-coordination-lock fallback for new-crate phases
- Critic iter-2 finding: `oyatie/.omx/ultragoal/` is a phantom path; corrected in §Inventory Ledger above
