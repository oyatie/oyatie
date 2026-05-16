---
doc_class: Index
status: Accepted
date: 2026-05-13
purpose: |
  Canonical index of all oyatie PRD files under docs/prds/.
  Every µservice has exactly one PRD. PRDs drive all downstream
  phase specs, impl plans, and µservice scaffolds.
doc_status: published
---

# PRD Index

All PRDs live under `docs/prds/`. Every µservice must have a PRD before
its first impl plan can be authored (per `feedback_autonomous_implementation_artifacts.md`).

## Shared Substrate PRDs (always-on; underpin every product)

| PRD file | µservice | Milestone | Status | Sales segment |
|---|---|---|---|---|
| [`tenancy.md`](tenancy.md) | `tenancy` | M02-substrate-ready | Accepted | shared-substrate |
| [`ontology.md`](ontology.md) | `ontology` | M02-substrate-ready | Accepted | shared-substrate |
| [`workflow.md`](workflow.md) | `workflow` | M02-substrate-ready | Accepted | shared-substrate + B2B hero product |

## B2B Application Shell

| PRD file | µservice | Milestone | Status | Sales segment |
|---|---|---|---|---|
| [`application.md`](application.md) | `application` | M03-first-paying-tenant | Accepted | Enterprise |

## Enterprise µservices (M03 first-paying-tenant scope)

| PRD file | µservice | Milestone | Status | Sales segment |
|---|---|---|---|---|
| [`hr.md`](hr.md) | `hr` | M03-first-paying-tenant | Accepted | Enterprise |
| [`payroll.md`](payroll.md) | `payroll` | M03-first-paying-tenant | Accepted | Enterprise |
| [`accounting.md`](accounting.md) | `accounting` | M03-first-paying-tenant | Accepted | Enterprise |
| [`connect.md`](connect.md) | `connect` | M03-first-paying-tenant | Accepted | Enterprise |

## Notes

- **Sales segment** is GTM/marketing segmentation ONLY — not architectural
  grouping (per `feedback_flat_product_catalog.md`). Every µservice is flat
  in the catalog; a tenant enables any subset à-la-carte.
- **Workflow Studio** ships as both shared substrate (M02) and end-user B2B
  product (M03 GA). It is oyatie's first hero product
  (per `feedback_workflow_studio_scope.md`).
- **Connect Personal** context is scaffolded in M03 but not GA until
  post-crypto-audit; deferred per `feedback_flat_product_catalog.md` §"Deferred".
- M04+ µservices (Healthcare, FinTech expansion, GRC, ATS, Procurement,
  Manufacturing, Logistics, etc.) will have PRDs authored in Wave 3+.

## PRD Authoring Rules

1. Start from `docs/templates/prd-template.md` (TPL-PRD). Never from scratch.
2. Every PRD must include: `## Competitive Benchmark`, `## Performance Targets`,
   `## Horizontal Scalability` sections (per `feedback_quality_performance_scalability_bar.md`).
3. Every PRD's BC listing must include layer mapping + port traits in kernel
   (per `feedback_clean_architecture_requirements.md`).
4. Every PRD must reference Bominal ADR inheritance or oyatie override
   (per `feedback_bominal_inheritance_precedence.md`).
5. No PRD may use retired glossary: no "platform", no "Object Graph",
   no "Shell", no "Product Group", no "Arm", no `shared|vertical` BNF slot.
