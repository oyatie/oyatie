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

### Axis products (7)

| Product | PRD | Owning team | Status |
|---|---|---|---|
| SaaS Platform | [`saas-platform/PRD.md`](saas-platform/PRD.md) | `axis-saas` | draft |
| Workspace | [`workspace/PRD.md`](workspace/PRD.md) | `axis-workspace` | draft |
| Foundry (agent runtime + engineering platform) | [`foundry/PRD.md`](foundry/PRD.md) | `axis-foundry` | draft |
| Cloud Provider | [`cloud/PRD.md`](cloud/PRD.md) | `axis-cloud` | draft |
| Search | [`search/PRD.md`](search/PRD.md) | `axis-search` | draft |
| Ads + Analytics | [`ads-analytics/PRD.md`](ads-analytics/PRD.md) | `axis-ads-analytics` | draft |
| Vertical Industry Cloud | (see vertical products below; vertical PRDs are the unit) | per-vertical teams | umbrella |

### Vertical products (14)

| Product | PRD | Owning team | Status |
|---|---|---|---|
| Corporate (HR / payroll / GL / mail / comms) | [`vertical-corporate/PRD.md`](vertical-corporate/PRD.md) | `vertical-corporate` | draft |
| Healthcare (clinical / ambulatory / HL7-FHIR) | [`vertical-healthcare/PRD.md`](vertical-healthcare/PRD.md) | `vertical-healthcare` | draft |
| Industrial (MES / OEE / ISA-95 / OPC UA) | [`vertical-industrial/PRD.md`](vertical-industrial/PRD.md) | `vertical-industrial` | draft |
| Logistics (shipment / dock / EDI / route) | [`vertical-logistics/PRD.md`](vertical-logistics/PRD.md) | `vertical-logistics` | draft |
| Fintech (PG / open-banking / KYC / AML) | [`vertical-fintech/PRD.md`](vertical-fintech/PRD.md) | `vertical-fintech` | draft |
| Legal (regulated corpus / contracts) | [`vertical-legal/PRD.md`](vertical-legal/PRD.md) | `vertical-legal` | draft |
| Retail (POS / inventory / promotions) | [`vertical-retail/PRD.md`](vertical-retail/PRD.md) | `vertical-retail` | skeleton |
| Education (LMS) | [`vertical-education/PRD.md`](vertical-education/PRD.md) | `vertical-education` | skeleton |
| Public Sector (forms / 조달청 + global gov) | [`vertical-public-sector/PRD.md`](vertical-public-sector/PRD.md) | `vertical-public-sector` | skeleton |
| Hospitality (PMS) | [`vertical-hospitality/PRD.md`](vertical-hospitality/PRD.md) | `vertical-hospitality` | skeleton |
| Construction (project mgmt) | [`vertical-construction/PRD.md`](vertical-construction/PRD.md) | `vertical-construction` | skeleton |
| Real Estate (leasing) | [`vertical-real-estate/PRD.md`](vertical-real-estate/PRD.md) | `vertical-real-estate` | skeleton |
| Agriculture (traceability) | [`vertical-agriculture/PRD.md`](vertical-agriculture/PRD.md) | `vertical-agriculture` | skeleton |
| Food (supply-chain compliance) | [`vertical-food/PRD.md`](vertical-food/PRD.md) | `vertical-food` | skeleton |

### Cross-product utilities

These are not "products" in the customer sense but are catalog-tracked for consistency:

| Catalog entity | Path | Notes |
|---|---|---|
| Regional packs | [`../regional-packs/`](../regional-packs/) | One pack per locale (KR / JP / US / EU / IN / BR / KSA / UAE / AU / SG / …); each is its own folder with `PACK.md` + `i18n/` + `regulatory/` + `payment-rails/` + `identity/` + `tax/` |
| Engineering teams | [`../teams/`](../teams/) | One charter per team |

## Required sections per product PRD (validated by `oya gate validate product-prd-json`)

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
11. **Open questions** — council-pending
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
