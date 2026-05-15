---
purpose: Auto-backfilled purpose for INDEX.md
---

---
doc_class: MilestoneIndex
parent: ../../MASTERPLAN.md
id: M04
title: Vertical-Pilot (Korea-first design-partner pack)
wave: W-Vertical-Pilot
status: gated on M03
owner: vertical-corporate (or council-elected) + tactical-first-vertical-pilot + gtm-customer-success
purpose: Run one vertical end-to-end on M01-M03 stack as a design-partner pilot with a KR Group tenant.
acceptance_authority: docs/ROADMAP.md §2.6
---

# M04 — Vertical-Pilot Korea

## Purpose
Prove the cohesion thesis: a single KR design-partner tenant runs end-to-end across SaaS workflows, Cloud substrate, Workspace collaboration, Foundry agent runtime, and Search/RAG, with full audit-chain emission and KR regulatory compliance. Validates [`docs/PRD.md`](../../../docs/PRD.md) §4.1 first-commercial-wave metrics.

## Status
**gated on M03.** All four axis previews stable enough to host a tenant.

## Scope
Council elects the vertical (likely `vertical-corporate` — HR/payroll/GL/mail — per [`docs/ROADMAP.md`](../../../docs/ROADMAP.md) §2.6). Four phases: capability pack authoring, regulatory binding (KR pack), design-partner onboarding, evidence collection + retention measurement.

## Dependencies
- **Hard:** M03 acceptance gate passed.
- **Hard:** M-CC-P07 hyperscaler-practice adoption ≥ Working-Backwards PRFAQ + Design Doc gates active.
- **Council decision required:** vertical election per [`docs/PRD.md`](../../../docs/PRD.md) §8 question 4.

## Acceptance gate
- One pilot tenant runs end-to-end with full audit-chain emission on every regulated capability invocation (100% per [`docs/PRD.md`](../../../docs/PRD.md) §4.1).
- Pilot retention ≥ 80% over 8 weeks of pilot operation.
- ≥ 50K Foundry agent runs/week at ≥ 99.5% success during pilot window.
- Zero tenant-data egress without consent receipt (hard zero per [`docs/PRD.md`](../../../docs/PRD.md) §4.2).
- KR pack control evidence at "Evidenced" in [`docs/COMPLIANCE-MATRIX.md`](../../../docs/COMPLIANCE-MATRIX.md) for tenant-onboarding-relevant controls.

## Phases
| ID | Title | Status | Index |
|---|---|---|---|
| P01 | Vertical Election + Capability Pack Authoring | stub | [`phases/P01-vertical-capability-pack/INDEX.md`](phases/P01-vertical-capability-pack/INDEX.md) |
| P02 | KR Regulatory Pack Binding (PIPA + CSAP + K-ISMS-P + KCMVP) | stub | [`phases/P02-kr-regulatory-binding/INDEX.md`](phases/P02-kr-regulatory-binding/INDEX.md) |
| P03 | Design-Partner Tenant Onboarding + Workflow Authoring | stub | [`phases/P03-design-partner-onboarding/INDEX.md`](phases/P03-design-partner-onboarding/INDEX.md) |
| P04 | Evidence Collection + Retention Measurement + Audit Pack | stub | [`phases/P04-evidence-retention-audit/INDEX.md`](phases/P04-evidence-retention-audit/INDEX.md) |

## Parallelism strategy
P01 and P02 run in parallel (capability authoring is code; regulatory binding is doc/control evidence). P03 starts when P01 ≥ 80% and P02 KR pack ≥ "Bound, evidence pending". P04 runs continuously alongside P03 (evidence collection is concurrent with tenant operations). Target: 3 agents in P01 (per capability family), 2 agents in P02 (per control family), 2 agents in P03 (onboarding + workflow studio support), 1 agent in P04 (evidence harvest).

## Hyperscaler practices adopted
- AWS Working-Backwards / PRFAQ: pilot tenant launch is a PRFAQ-gated event.
- Google Design Doc for the capability pack.
- SRE postmortem-blameless for any pilot incident.
- Microsoft 1ES pipelines for KR pack regression suite.
- Oracle Engineering-Excellence-Council reviews capability-pack PRs.

## Agent-navigability-pointer
First-claim seed (post council election): `crates/oya-vertical-<elected>-pack-kernel/src/lib.rs::CapabilityPack` (after P01 IP-001 scaffold-claim). If vertical-corporate is elected: `crates/oya-vertical-corporate-payroll-kernel/src/lib.rs::PayrollClose`.
