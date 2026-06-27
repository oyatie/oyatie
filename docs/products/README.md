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

This index is a traceability surface, not a runtime-readiness claim. It lists only product PRD files that exist in this repository. Product or vertical concepts that do not yet have a `docs/products/<product>/PRD.md` file remain governed by the cross-product PRD plus the machine-readable `specs/microservices/*.json` surfaces until a PRD file is added.

### Available product PRDs

| Product surface | PRD | Owning team / axis | Status | Traceability note |
|---|---|---|---|---|
| SaaS Platform | [`saas-platform/PRD.md`](saas-platform/PRD.md) | `axis-saas` | planning-closed-contract-authored | Existing product PRD. |
| Cloud Provider | [`cloud/PRD.md`](cloud/PRD.md) | `axis-cloud` | draft | Existing product PRD; target/non-claim posture remains governed by cloud specs and gates. |
| Foundry / Intelligence Platform | [`foundry/PRD.md`](foundry/PRD.md) | `axis-foundry` / intelligence substrate | draft | Existing PRD; Foundry naming/retirement details remain governed by the PRD and microservice specs. |
| ERP Coverage | [`erp-coverage/PRD.md`](erp-coverage/PRD.md) | `axis-erp-coverage` | draft | Existing ERP composition PRD. |
| Workplace Integration | [`workplace-integration/PRD.md`](workplace-integration/PRD.md) | `axis-workplace-integration` | draft target/non-claim | Existing cross-cutting workplace integration PRD. |

### Spec-backed surfaces without product PRD files yet

These surfaces are intentionally not linked to missing `docs/products/<name>/PRD.md` files. Use the machine-readable specs under [`../../specs/microservices/`](../../specs/microservices/) until a product PRD exists.

| Surface family | Current authority | Traceability note |
|---|---|---|
| Workspace apps (mail, calendar, messenger, workflow, workflow studio, etc.) | [`../../specs/microservices/`](../../specs/microservices/) plus the Workplace Integration PRD | No separate Workspace product PRD file exists in this tree. |
| Search / RAG | [`../../specs/microservices/`](../../specs/microservices/) and cross-product docs | No separate Search product PRD file exists in this tree. |
| Ads + analytics | [`../../specs/microservices/`](../../specs/microservices/) and cross-product docs | No separate Ads/analytics product PRD file exists in this tree. |
| Vertical domains | [`../../specs/microservices/`](../../specs/microservices/) plus `erp-coverage/PRD.md` where applicable | The previous vertical PRD links were placeholders; do not add product-readiness claims without files and gates. |

### Regional pack traceability

Regional pack documentation lives in [`../localization-packs/INDEX.md`](../localization-packs/INDEX.md). Repository-local pack directories live under [`../../packs/`](../../packs/) and are checked by `scripts/tests/product_region_traceability_check.py`.

### Cross-product utilities

These are not "products" in the customer sense but are catalog-tracked for consistency:

| Catalog entity | Path | Notes |
|---|---|---|
| Localization pack catalog | [`../localization-packs/INDEX.md`](../localization-packs/INDEX.md) | Region/pack traceability only; not a runtime readiness claim. |
| Engineering teams | [`../teams/`](../teams/) | One charter per team. |

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
