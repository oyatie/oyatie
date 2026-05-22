---
doc_class: ImplementationPlan
ip_id: IP-050-cms-seam
microservice: marketing-automation
related_adrs: [ADR-0244, ADR-0263, ADR-0321, ADR-0328]
bounded_context: cms-seam
journey_id: J-MA-50-cms-overlap-seam
status: proposed
date: 2026-05-21
owner: axis-marketing-automation
tenant_class_aware: true
delegation_destination: sites + design-collaboration µservices
open_question_settled: Q-007
---

# IP-050: CMS Seam

## Context

HubSpot CMS Hub combines marketing landing pages, website pages, and the website CMS. Oyatie splits CMS responsibility: marketing-attached landing pages live in this µservice (IP-032); tenant website root lives in sites; content authoring + design collaboration lives in design-collaboration. This seam declares the contract.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_cms_page_ref` | `ref_id` | `uuid primary key` | Reference id. |
| `marketing_cms_page_ref` | `tenant_id` | `uuid not null` | Tenant. |
| `marketing_cms_page_ref` | `sites_page_id` | `uuid not null` | FK to sites.page. |
| `marketing_cms_page_ref` | `campaign_id` | `uuid` | Optional campaign binding. |
| `marketing_cms_page_ref` | `created_at_hlc` | `hlc not null` | HLC. |

## API Endpoints

REST `POST /v1/marketing-automation/cms/page-refs` references a sites page from a campaign.

## Cross-µservice Handoffs

- `sites` owns tenant website root.
- `design-collaboration` owns collaborative content authoring.
- `audit-chain` seals events.
- `marketing-calendar` displays CMS publication events.
