---
doc_class: CrossReferenceIndex
shape: index
status: Accepted
authority_tier: 1
length_cap: 200
date: 2026-05-12
purpose: |
  Master index of every authoritative doc in the project. Every agent treats this as the
  one-stop lookup table for "which doc governs X". Derived from MASTERPLAN + existing
  oyatie/docs/ + the parallel composer outputs (docs/standards, docs/templates,
  docs/governance-lanes).
canonical_authority: docs/CONSTITUTION.md
foundation: ADR-0053 (sanctioned primitives), ADR-0054 (scaffold-claim)
related:
  - docs/agents/AGENT-ENTRY-POINT.md
  - docs/AGENTS.md
  - docs/DOC-CATALOG.md
  - docs/RACI-OWNERSHIP.md
doc_status: published
---

# Cross-Reference Index

> The one-stop lookup. When you need to know "which doc governs X / who owns it / which lane enforces it / when was it last verified", read the row here. Do not duplicate content — link. Foundation: ADR-0053 (sanctioned primitives) and ADR-0054 (scaffold-claim).

## Master table

| Path | Doc-class | Purpose | Owner (RACI) | Lifecycle (DOC-CATALOG) | Enforced-by lane | Last-verified | Cross-refs (cited by) |
|---|---|---|---|---|---|---|---|
| `docs/CONSTITUTION.md` | Constitution | Mission, decision rights, prohibitions, amendments | Founder + council-architecture | Tier 0; amend-only-by-Founder | `oya-governance-authority-cohesion` | 2026-05-12 | AGENTS, README, MASTERPLAN, every ADR |
| `docs/AGENTS.md` | Operating-Contract | Pre-flight + during-change + done-definition (D1–D18) | council-architecture | Tier 1; amend-by-PR | `traceability-validator`; independent review evidence (`F-PR5-06` enforcement gap) | 2026-05-12 | CONSTITUTION, README, every agent-kickoff file |
| `docs/README.md` | Redirect | Bootstrap routing | axis-foundry | Tier 1; thin (≤25 lines) | `oya-governance-redirect-thinness` | 2026-05-12 | CONSTITUTION |
| `docs/DESIGN.md` | Design | Architecture, planes, cross-axis contracts | council-architecture | Tier 1 | `oya-governance-cohesion` | 2026-05-12 | AGENTS, MASTERPLAN, every architectural ADR |
| `docs/SPEC.md` | Specification | Surface enumeration (capabilities, APIs, events, indexes, ad slots) | council-architecture | Tier 1 | `oya-governance-spec-surface` | 2026-05-12 | DESIGN, PRD |
| `docs/PRD.md` | PRD | North star, axes, scope, success metrics, decision log | Founder + council-architecture | Tier 1 | `oya-governance-prd-coverage` | 2026-05-12 | MASTERPLAN, ROADMAP |
| `docs/ROADMAP.md` | Roadmap | Wave sequence + per-wave gate criteria | council-architecture | Tier 1 | `oya-governance-roadmap-gate` | 2026-05-12 | MASTERPLAN milestone gates |
| `docs/DOC-CATALOG.md` | Catalog | Per-doc lifecycle + update protocol | council-architecture | Tier 1 | `oya-governance-doc-catalog` | 2026-05-12 | every authoritative doc |
| `docs/DOC-UPDATE-PROTOCOL.md` | Protocol | Update-trigger taxonomy + automation hooks | council-architecture | Tier 1 | `oya-governance-doc-freshness` | 2026-05-12 | DOC-CATALOG |
| `docs/ADR-INDEX.md` | ADR Index | Index of all ADRs (Accepted/Superseded) | crew-adr-promotion | Tier 1 | `oya-governance-adr-shape`, `-adr-citation` | 2026-05-12 | DESIGN, every PR `## Traceability` |
| `docs/CHANGELOG.md` | Ledger | Per-PR canonical-doc-touch row | axis-foundry (auto) | Tier 1 (append-only) | `oya-governance-changelog-row` | 2026-05-12 | every PR D18 |
| `docs/MISTAKES-LEDGER.md` | Ledger | Failure-mode catalog + mechanical preventions | council-architecture | Tier 1 (append-only) | `oya-governance-mistakes-ledger-cite` | 2026-05-12 | PR `## Traceability` |
| `docs/GLOSSARY.md` | Glossary | Canonical vocabulary | council-architecture | Tier 1 | `oya-governance-glossary` | 2026-05-12 | every doc using domain terms |
| `docs/RACI-OWNERSHIP.md` | RACI | Per-domain RACI rows | council-architecture | Tier 1 | `oya-governance-raci-coverage` | 2026-05-12 | AGENTS, MASTERPLAN |
| `docs/RISK-REGISTER.md` | Register | Top risks RM-NN with owner + status | council-architecture | Tier 1 | `oya-governance-risk-coverage` | 2026-05-12 | MASTERPLAN §9 |
| `docs/PRIVACY-PROGRAM.md` / `docs/security-program/security-program.json` / `docs/COMPLIANCE-MATRIX.md` | Programs | Cross-cutting compliance | council-privacy / ops-security / ops-compliance | Tier 1 | `oya-governance-privacy-coverage`, `-security-coverage` | 2026-05-12 | AGENTS pre-flight |
| `docs/RELEASE-MANAGEMENT.md` / `docs/INCIDENT-MANAGEMENT.md` | Programs | Release + incident process | ops-sre-reliability | Tier 1 | `oya-governance-release-readiness` | 2026-05-12 | AGENTS, runbooks |
| `docs/RUNBOOKS-INDEX.md` | Index | Runbook discovery | ops-sre-reliability | Tier 1 | `oya-governance-runbook-index-resolves` | 2026-05-12 | every runbook |
| `docs/SLO-CATALOG.md` | Catalog | Service-level objectives | ops-sre-reliability | Tier 1 | `oya-governance-error-budget-gate` | 2026-05-12 | RELEASE-MGMT |
| `docs/STANDARDS-AND-TEMPLATES.md` | Index | Standards + templates index (post-lift) | council-architecture | Tier 1 | `oya-governance-standards-index` | 2026-05-12 | AGENTS, docs/standards/INDEX.md |
| `docs/decisions/ADR-*.md` | ADR | Architectural decisions | per-ADR-owner | Tier 2 | `oya-governance-adr-shape` | per-ADR | DESIGN, IPs, MASTERPLAN |
| `docs/standards/*.md` (lifted) | Standard | Cross-cutting norm | per-standard-owner | Tier 2 | the lane named in each standard's frontmatter | 2026-05-12 | AGENTS, IPs |
| `docs/templates/*.md\|yaml\|json` (lifted) | Template | Canonical doc/record shape | axis-foundry + per-template-owner | Tier 2 | `oya-governance-<class>-shape` | 2026-05-12 | every PR |
| `templates/checklists/*.md` (lifted) | Checklist | Per-class verification | axis-foundry + per-checklist-owner | Tier 2 | independent reviewer evidence + per-lane CI | 2026-05-12 | every PR |
| `docs/products/<axis>/` | Per-axis PRD | Per-axis product spec | per-axis lead | Tier 2 | `oya-governance-prd-coverage` | per-axis | PRD, DESIGN |
| `docs/teams/` | Team charter | Per-team norms | per-team lead | Tier 2 | `oya-governance-team-charter` | per-team | RACI |
| `docs/regional-packs/` | Regional pack | Per-region adaptations | regional-packs lead | Tier 2 | `oya-governance-regional-pack` | per-region | ADR-0010 |
| `docs/runbooks/` | Runbook | Per-service operations | ops-sre-reliability + axis lead | Tier 2 | `oya-governance-runbook-index-resolves` | per-runbook | RUNBOOKS-INDEX |

## Working drafts (.omc/) — not authoritative until lifted

| Path | Doc-class | Lift target | Status |
|---|---|---|---|
| `.omc/plans/MASTERPLAN.md` | MasterPlan | `docs/MASTERPLAN.md` | pending approval |
| `.omc/plans/milestones/<MNN>/INDEX.md` | MilestoneIndex | `docs/plans/milestones/<MNN>/INDEX.md` | pending approval per-milestone |
| `.omc/plans/milestones/<MNN>/phases/<PNN>/INDEX.md` | PhaseIndex | same path under `docs/` | pending approval per-phase |
| `.omc/plans/milestones/<MNN>/phases/<PNN>/IP-*.md` | ImplementationPlan | same path under `docs/` | pending approval per-IP |
| `.omc/standards/INDEX.md` + `*.md` | StandardsIndex / Standard | `docs/standards/` | pending approval per-standard |
| `/templates/INDEX.md` + `*.md|yaml|json` | TemplateIndex / Template | `docs/templates/`, `templates/checklists/` | pending approval per-template |
| `.omc/governance-lanes/*.md` | FitnessLane | `docs/governance-lanes/<lane>.md` | pending approval per-lane |

## Agent-kickoff files (this directory)

| File | Purpose | Reading order |
|---|---|---|
| [`INDEX.md`](INDEX.md) | Catalog + reading order | 1 |
| [`AGENT-ENTRY-POINT.md`](AGENT-ENTRY-POINT.md) | Single page a fresh agent reads first | 2 |
| [`AGENT-DECISION-TREE.md`](AGENT-DECISION-TREE.md) | Per-decision flowchart | 3 |
| [`AGENT-TOOL-PROTOCOL.md`](AGENT-TOOL-PROTOCOL.md) | Tool-by-tool calling convention | 4 |
| [`AGENT-FAILURE-RECOVERY.md`](AGENT-FAILURE-RECOVERY.md) | R1–R7 recoveries | 6 |
| [`CROSS-REFERENCE-INDEX.md`](CROSS-REFERENCE-INDEX.md) | This file | 8 |
| [`AGENT-CHEAT-SHEET.md`](AGENT-CHEAT-SHEET.md) | Printable 1-pager | 9 |
| [`HUMAN-OPERATOR-GUIDE.md`](HUMAN-OPERATOR-GUIDE.md) | For humans when matrix matches | 10 |
| [`ESCALATION-MATRIX.md`](ESCALATION-MATRIX.md) | 3 retained halt cases | 11 |

## How to use this index

1. Need to know which doc governs a change? Look up the doc-class column.
2. Need to know who owns the decision? Owner column.
3. Need to know which fitness lane will block your PR? Enforced-by-lane column.
4. Need to know if the doc is stale? Last-verified column; rerun the doc-freshness check if older than the staleness budget in CHK-DOCFRESH.
5. Need to know what else cites this doc (impact analysis)? Cross-refs column.

## Lane → standard → doc resolution

The complete lane catalog lives at [`docs/governance-lanes/`](../governance-lanes/) (parallel composer output). Each lane's `.md` declares its `governs:` path; this index is the inverse mapping (doc → lane).
