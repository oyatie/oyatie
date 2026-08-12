---
doc_class: Checklist
checklist_id: CHK-DOCFRESH
status: pending approval
purpose: |
  Per-doc-class staleness budget and auto-update path. Walked monthly by the doc-freshness fitness lane and at every `EVT-WAVE-GATE-PASSED` per `docs/DOC-CATALOG.md`.
lift_target: oyatie/templates/checklists/doc-freshness.md
enforcing_fitness_lane: oya-governance-doc-freshness
owner_team: council-architecture
related:
  - docs/DOC-CATALOG.md
  - docs/DOC-UPDATE-PROTOCOL.md
  - .omc/plans/MASTERPLAN.md  # §2 principle 10 auto-doc
---

# Doc Freshness Checklist

> One row per doc class. Each row names a staleness budget + auto-update path (auto-generated mirror, agent re-emission, or manual re-author).

## Staleness budget by doc class

| Doc class | Staleness budget | Auto-update path | Lane |
|---|---|---|---|
| `Constitution` | 1 quarter | Manual; council-architecture only | `oya-governance-authority-cohesion` |
| `Operating-Contract` (AGENTS.md) | 1 quarter | Manual; axis-foundry + council-architecture | `oya-governance-authority-cohesion` |
| `MasterPlan` | 1 quarter | Manual; council-architecture | `oya-governance-plan-hierarchy` |
| `MilestoneIndex` | 2 weeks | Status auto-recomputed from phase INDEX rollup | `oya-governance-plan-hierarchy` |
| `PhaseIndex` | 1 week | Status auto-recomputed from IP rollup | `oya-governance-plan-hierarchy` |
| `ImplementationPlan` | per-PR | Author-emitted; agent re-emits on `grit done` | `oya-governance-plan-hierarchy` |
| `PRD` | 1 quarter (EVT-AXIS-SCOPE-CHANGE, EVT-PRICING-CHANGE, EVT-VERTICAL-ADDED) | Council-only | `prd-internal-consistency` |
| `Design` (docs/DESIGN.md) | 1 month | Council-only | `design-contracts-mirror` |
| `Spec` (docs/SPEC.md) | 1 week | Agent auto-PR for additions; manual for deletions | `spec-contract-mirror` |
| `Roadmap` | 2 weeks | Agent rebalance; manual band-promotion | `roadmap-band-totals` |
| `ADR-INDEX` | per ADR event | Agent re-emit from `decisions/` | `adr-index-completeness` |
| `ADR` (individual) | immutable once Accepted (status transitions only) | Manual via supersession | `adr-shape` |
| `RiskRegister` | weekly (events: EVT-RISK-MATERIALIZED, EVT-INCIDENT-CLOSED, EVT-AUDIT-FINDING) | Agent for low/med; manual for catastrophic | `risk-register-coverage` |
| `ComplianceMatrix` | monthly (events: EVT-REGULATORY-CHANGE, EVT-AUDIT-FINDING) | Manual | `compliance-matrix-coverage` |
| `SecurityProgram` | 1 quarter | Manual | `security-controls-coverage` |
| `PrivacyProgram` | monthly | Manual | `privacy-class-taxonomy-coverage` |
| `RunbooksIndex` | weekly | Agent re-emit | `runbook-discoverability` |
| `Runbook` (individual) | `last_verified` ≤ 90 days | Drill-triggered manual; new-runbook-on-postmortem auto-stub | `runbook-index-resolves` |
| `SloCatalog` | weekly | Agent re-emit from observability config | `slo-surface-coverage` |
| `ReleaseManagement` | monthly | Manual | `release-lane-coverage` |
| `QaTestStrategy` | 1 quarter | Manual | `qa-coverage-by-class` |
| `RaciOwnership` | 1 quarter (event: EVT-HIRE-NEW-TEAM-LEAD) | Agent sync from CODEOWNERS | `raci-team-coverage` |
| `IncidentManagement` | per Sev-1/2 + 1 quarter | Manual | `incident-template-completeness` |
| `MistakesLedger` | monthly + per-mistake-event | Manual append; quarterly council review | `oya-governance-mistakes-ledger-cite` |
| `ChangeLog` | per commit | System-emitted | `changelog-completeness` |
| `Glossary` | monthly | Agent extracts new terms; humans rename | `glossary-cross-doc-coverage` |
| `Doc-Catalog` | per change + monthly | Manual | `doc-catalog-self-coverage` |
| `Capability` (record) | per publish | Author | `capability-schema-validator` |
| `Postmortem` | per-incident, then immutable | Manual | `postmortem-shape` |
| `PRFAQ` | once accepted, immutable (or supersession) | Manual | `(advisory)` |
| `DesignDoc` | per-phase | Manual | `oya-governance-design-doc-shape` (advisory) |

## Walk procedure

- [ ] **F1** For each row above, compute `now - last_updated_at`. If `> staleness budget`, the doc is **stale**.
- [ ] **F2** For each stale doc, the auto-update path emits a draft (or a `BLOCKED_ON_HUMAN_ORCHESTRATOR` event for council-only docs). *Lane:* `oya-governance-doc-freshness`.
- [ ] **F3** No `<!-- forward-reference: wave-1 -->` markers point at artifacts that should have existed by `now`. *Lane:* `oya-governance-forward-reference`.
- [ ] **F4** `docs/DOC-CATALOG.md` itself walked: every row's `update_trigger` events for the past period accounted for. *Lane:* `oya-governance-doc-catalog`.
- [ ] **F5** `docs/CHANGELOG.md` rows present for every doc touched since last walk. *Lane:* `oya-governance-changelog-row`.
- [ ] **F6** Orphans: every file under `docs/` has either an `update_trigger` row in `DOC-CATALOG.md` OR an explicit `excludes:` entry in upstream contract. *Lane:* `oya-governance-orphan-detection`.

## Anti-patterns

- Bumping `last_updated_at` without a substantive change. *(forbidden — `oya-governance-doc-freshness` detects via diff size.)*
- Marking a doc `auto-updated` then editing manually. *(authority drift; council-architecture audit.)*
- Forward-referencing into wave-2/3 docs when the phase is supposed to ship wave-1.
