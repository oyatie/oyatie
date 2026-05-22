---
doc_status: published
last_audited: 2026-05-20
---

# User Journeys Guide

This directory is the journey spine for `j01` through `j175`. Journey numbers are sequencing, not priority labels: lower numbers establish platform invariants that later journeys reuse.

## Sequencing Map

| Range | Theme | How to use it |
|---|---|---|
| `j01`-`j20` | Life-safety and critical-path trust | Read first for emergency, crisis, account recovery, disaster, delegated-agent, and high-risk boundary behavior. |
| `j21`-`j35` | Foundation workflow expansion | Bridges early trust journeys into ordinary tenant workflows and product-surface buildout. |
| `j36`-`j50` | Hero product journeys | Anchors first visible tenant value across collaboration, workflow, and business operations. |
| `j51`-`j75` | Cross-product operations | Exercises handoffs across workspace, workflow-engine, ontology, audit-chain, and compliance surfaces. |
| `j76`-`j90` | Locale and pack overlays | Tests regional packs, language, residency, accessibility, and jurisdiction-specific behavior. |
| `j91`-`j100` | Pack rollout and first action | Proves tenant onboarding can activate packs and reach first useful work without hidden tribal knowledge. |
| `j101`-`j125` | Cross-tenant ecosystem flows | Covers suppliers, payments, disruption, marketplace, RFQ, secondment, and tenant-to-tenant coordination. |
| `j126`-`j150` | Audit, HR, personal-work boundary, and creator/economy flows | Connects regulated enterprise operations with personal tenant continuity and marketplace income. |
| `j151`-`j175` | Extended persona and industry coverage | Broadens blue/pink/green/gold/gray-collar scenarios and validates universality across work styles. |

## Current Inventory

- Journey directories discovered: 175.

- Highest journey directory discovered: `j175`.

- Reports and catalogs in this directory summarize completed batches; journey directories remain the source for per-journey story, UX, handshake, and implementation-plan artifacts.


## Reading Rules

- Start with the range map, then open the exact `jNN-*` directory.

- Read the journey README first, then story, UX flow, handshake, schema, and implementation slices.

- Follow binding ADRs before changing a journey. Life-safety, vulnerable-user, personal/work boundary, and cross-tenant journeys usually bind to ADR-0292 through ADR-0321.

- Do not renumber journeys. If a journey is replaced, leave a supersession note and point to the successor.
