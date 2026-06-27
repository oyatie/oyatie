---
doc_status: published
---

# Oyatie — Per-Product PRD Index

Each Oyatie product (an axis or a vertical) has its own PRD here. Every PRD follows [`_TEMPLATE.md`](_TEMPLATE.md). Per-product PRDs are intentionally **slice-level**: they name the bounded context, list the kernel entities + data structures, declare optimization practices, list regional-pack seams, declare in-house vs external dep posture, and define success metrics for that slice only. Cross-cutting concerns live in the consolidated docs one level up.

## Reading order

1. [`../README.md`](../README.md) — overall doc map.
2. [`../PRD.md`](../PRD.md) — cross-product PRD (the cohesion thesis + 7 axes).
3. [`../DESIGN.md`](../DESIGN.md) — architecture + cross-axis contracts.
4. This index.
5. The per-product PRD relevant to your work.

## Per-product PRDs

### Authored product PRDs

| Product | PRD | Owning team | Status |
|---|---|---|---|
| SaaS Platform | [`saas-platform/PRD.md`](saas-platform/PRD.md) | `axis-saas` | planning-closed-contract-authored |
| Cloud Provider | [`cloud/PRD.md`](cloud/PRD.md) | `axis-cloud` | draft |
| Foundry | [`foundry/PRD.md`](foundry/PRD.md) | `axis-foundry` | draft |
| ERP Coverage | [`erp-coverage/PRD.md`](erp-coverage/PRD.md) | `axis-product + axis-architecture + axis-erp-parity` | draft |
| Workplace Integration | [`workplace-integration/PRD.md`](workplace-integration/PRD.md) | `axis-workflow + axis-application-shell + axis-identity + axis-tenancy + axis-compliance` | draft |

### Planned product PRD slots (not yet authored)

These rows keep the product-lane roster visible without linking to PRD files that do not exist yet. Add a link only in the same change that lands the corresponding `docs/products/<product>/PRD.md` file.

| Product | PRD slot | Owning team | Status |
|---|---|---|---|
| Workspace | `docs/products/workspace/PRD.md` | `axis-workspace` | draft slot; PRD not authored |
| Intelligence Platform (cloud-intelligence + oya-intelligence) | `docs/products/intelligence-platform/PRD.md` | `cloud-intelligence / oya-intelligence` | draft slot; PRD not authored |
| Search | `docs/products/search/PRD.md` | `axis-search` | draft slot; PRD not authored |
| Ads + Analytics | `docs/products/ads-analytics/PRD.md` | `axis-ads-analytics` | draft slot; PRD not authored |
| Corporate (HR / payroll / GL / mail / comms) | `docs/products/vertical-corporate/PRD.md` | `vertical-corporate` | draft slot; PRD not authored |
| Healthcare (clinical / ambulatory / HL7-FHIR) | `docs/products/vertical-healthcare/PRD.md` | `vertical-healthcare` | draft slot; PRD not authored |
| Industrial (MES / OEE / ISA-95 / OPC UA) | `docs/products/vertical-industrial/PRD.md` | `vertical-industrial` | draft slot; PRD not authored |
| Logistics (shipment / dock / EDI / route) | `docs/products/vertical-logistics/PRD.md` | `vertical-logistics` | draft slot; PRD not authored |
| Fintech (PG / open-banking / KYC / AML) | `docs/products/vertical-fintech/PRD.md` | `vertical-fintech` | draft slot; PRD not authored |
| Legal (regulated corpus / contracts) | `docs/products/vertical-legal/PRD.md` | `vertical-legal` | draft slot; PRD not authored |
| Retail (POS / inventory / promotions) | `docs/products/vertical-retail/PRD.md` | `vertical-retail` | skeleton slot; PRD not authored |
| Education (LMS) | `docs/products/vertical-education/PRD.md` | `vertical-education` | skeleton slot; PRD not authored |
| Public Sector (forms / 조달청 + global gov) | `docs/products/vertical-public-sector/PRD.md` | `vertical-public-sector` | skeleton slot; PRD not authored |
| Hospitality (PMS) | `docs/products/vertical-hospitality/PRD.md` | `vertical-hospitality` | skeleton slot; PRD not authored |
| Construction (project mgmt) | `docs/products/vertical-construction/PRD.md` | `vertical-construction` | skeleton slot; PRD not authored |
| Real Estate (leasing) | `docs/products/vertical-real-estate/PRD.md` | `vertical-real-estate` | skeleton slot; PRD not authored |
| Agriculture (traceability) | `docs/products/vertical-agriculture/PRD.md` | `vertical-agriculture` | skeleton slot; PRD not authored |
| Food (supply-chain compliance) | `docs/products/vertical-food/PRD.md` | `vertical-food` | skeleton slot; PRD not authored |

### Cross-product utilities

These are not "products" in the customer sense but are catalog-tracked for consistency:

| Catalog entity | Path | Notes |
|---|---|---|
| Regional packs | [`../localization-packs/INDEX.md`](../localization-packs/INDEX.md), [`../../packs/`](../../packs/) | Documentation-pack index plus repo-root regional pack artifacts. This is traceability only and does not claim pack runtime readiness. |
| Engineering teams | [`../teams/`](../teams/) | One charter per team |

## Required sections per product PRD (validated by the product PRD Rust gate in `oya-ci-required`)

1. **North star** — what + who + why
2. **Target users** — personas + value-exchange
3. **In/out of scope** — by wave (preview / stable / GA)
4. **Architecture overview** — bounded context, layered structure, surfaces, seams, cross-axis dependencies
5. **Data structures** — kernel entities, aggregates, persistence layout, event schemas, index touchpoints, audit-chain emission, schema-migration policy
6. **Optimization practices** — cell-routing, sharding, caching, batching, idempotency, hot-path benchmarks, agent-driven optimization, FinOps
7. **Regional pack interactions** — which seams the product plugs
8. **In-house vs external dependency posture** — license-tier table
9. **Success metrics** — per-wave targets + structural metrics
10. **Risks + mitigations** — slice-level risk register
11. **Open questions** — founder-/governance-pending
12. **Decision log** — slice-level decisions
13. **Sources scanned** — fresh

## Templates for sub-artifacts

- Capability record YAML — per [`registry/capability-templates/`](../../registry/capability-templates/)
- Catalog record YAML — per [`registry/catalog/`](../../registry/catalog/)
- ADR shape — per [`decisions/`](../decisions/) (use `decisions/_template.md` if present)

## Update protocol

A product PRD is updated whenever:
- The product's catalog entry changes
- A new capability is registered
- A cross-axis contract changes
- A regional pack onboards / changes
- A wave-gate passes for the product

See [`../DOC-CATALOG.md §2.5`](../DOC-CATALOG.md) for the full event → doc mapping.
