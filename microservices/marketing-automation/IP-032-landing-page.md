---
doc_class: ImplementationPlan
ip_id: IP-032-landing-page
microservice: marketing-automation
related_adrs: [ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0263, ADR-0321, ADR-0328]
bounded_context: landing-page
journey_id: J-MA-32-marketing-landing-page-build
status: proposed
date: 2026-05-21
owner: axis-marketing-automation
tenant_class_aware: true
boundary_settled_by: ADR-MS-MA-002
---

# IP-032: Marketing Landing Page

## Context

Diana Alvarez (agency principal) needs landing pages with form attachment + conversion goal + A/B variant for her clients running webinar registration + ebook download + product trial campaigns. The boundary between marketing-attached landing pages (here) and tenant website root (delegated to sites) is settled by ADR-MS-MA-002: marketing-automation owns campaign-attached transient pages; sites owns the persistent website root.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_landing_page` | `page_id` | `uuid primary key` | Oyatie page id. |
| `marketing_landing_page` | `tenant_id` | `uuid not null` | Tenant partition. |
| `marketing_landing_page` | `slug` | `text not null` | URL path segment; unique per tenant. |
| `marketing_landing_page` | `template_id` | `uuid` | Reference to marketing_asset template. |
| `marketing_landing_page` | `attached_form_id` | `uuid` | Reference to marketing_form. |
| `marketing_landing_page` | `conversion_goal` | `text not null` | form_submit / meeting_booked / trial_started / asset_downloaded / custom. |
| `marketing_landing_page` | `seo_metadata` | `jsonb not null` | title, meta_description, og_tags, schema.org JSON-LD. |
| `marketing_landing_page` | `custom_html` | `text` | Optional override. |
| `marketing_landing_page` | `password_protected` | `boolean not null default false` | If true, requires `access_password_hash`. |
| `marketing_landing_page` | `access_password_hash` | `text` | Argon2id hash when password_protected. |
| `marketing_landing_page` | `locale_variants` | `jsonb not null` | Per-locale content per pack overlay. |
| `marketing_landing_page` | `published_at_hlc` | `hlc` | Immutable timestamp. |
| `marketing_landing_page_conversion` | `conversion_id` | `uuid primary key` | Conversion event id. |
| `marketing_landing_page_conversion` | `page_id` | `uuid not null` | FK. |
| `marketing_landing_page_conversion` | `subject_hash` | `text not null` | Hashed subject ref. |
| `marketing_landing_page_conversion` | `goal_reached` | `text not null` | Which conversion_goal fired. |
| `marketing_landing_page_conversion` | `occurred_at_hlc` | `hlc not null` | HLC stamp. |

## API Endpoints

REST `POST /v1/marketing-automation/landing-pages`:

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-ma000032",
  "slug": "may-2026-webinar",
  "template_id": "...",
  "attached_form_id": "...",
  "conversion_goal": "form_submit",
  "seo_metadata": {
    "title": "May 2026 Webinar — Acme Product",
    "meta_description": "Join us for a deep dive into Acme Product.",
    "og_tags": {"og:image": "https://drive.oyatie.dev/..."}
  }
}
```

REST `GET /landing/{tenant_slug}/{page_slug}` is the public render endpoint (Cedar gates by `password_protected` + `published` status).

REST `POST /v1/marketing-automation/landing-pages/{page_id}/conversions` records a conversion event (called by attached form on submit, or by tracking pixel on goal-reached).

gRPC `MarketingLandingPageService.Publish` mirrors REST.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"marketing.ops"` | `marketingAutomation::PublishLandingPage` | `MarketingLandingPage::page_id` | `tenant_class`, `active_landing_pages_count` |
| `Service::"page-renderer"` | `marketingAutomation::RenderLandingPage` | `MarketingLandingPage::page_id` | `subject_ip_country`, `password_supplied`, `published` |

Demo-trial gate: `tenant_class == 'demo_trial' && active_landing_pages_count >= 3` denies publish.

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| HubSpot Landing Page | `MarketingLandingPage` | content modules become `template_id` reference + per-region overrides. |
| HubSpot LP password protection | `MarketingLandingPage.password_protected` + `access_password_hash` | Hash format unified to Argon2id. |
| Marketo Landing Page | `MarketingLandingPage` | Marketo LP form attachment becomes `attached_form_id`. |
| Mailchimp Landing Page | `MarketingLandingPage` | Mailchimp LP is template-driven; preserved as `template_id` reference. |

## Workflow Steps

1. `ValidateSlugUniqueness` ensures (`tenant_id`, `slug`) is unique.
2. `ResolveTemplate` validates `template_id` exists in marketing-asset.
3. `ValidateAttachedForm` validates form is published if `attached_form_id` is set.
4. `ValidateSeoBlock` validates required SEO fields per pack overlay (e.g., ePrivacy requires cookie-consent disclosure).
5. `AuthorizePublish` calls Cedar.
6. `RenderPreview` produces preview HTML for the operator.
7. `Publish` transitions status; provisions tenant CNAME via sites contract.
8. `SealPublish` emits `EVT-MARKETING-LANDING-PAGE-PUBLISHED`.

Decision branches:
- Slug conflict → 409 `slug_conflict`.
- Form not published → 422 `attached_form_not_published`.
- SEO floor not met → 422 `seo_block_required`.
- Demo-trial cap exceeded → 429 `demo_trial_cap_hit`.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-LANDING-PAGE-CREATED` | `tenant_id`, `page_id`, `slug`, `tenant_class` |
| `EVT-MARKETING-LANDING-PAGE-PUBLISHED` | `page_id`, `published_at_hlc`, `tenant_class`, `cedar_decision_id` |
| `EVT-MARKETING-LANDING-PAGE-CONVERTED` | `page_id`, `conversion_id`, `subject_hash`, `goal_reached`, `occurred_at_hlc` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Publish landing page | 200 ms | 800 ms | 2 s | 50 rps/cell | 99.95% |
| Render landing page | 30 ms | 150 ms | 400 ms | 5000 rps/cell | 99.99% |
| Record conversion | 25 ms | 100 ms | 250 ms | 2000 rps/cell | 99.99% |

## Failure Modes + Recovery

- Sites CNAME provisioning failure → page remains `validated`; retry worker re-attempts; page is not publicly resolvable until success.
- Renderer cache miss → fall through to Postgres source-of-truth + warm cache.
- Conversion event during decommission window → event buffered until commit replay.

## Migration Notes

HubSpot LP export bundle includes module-tree JSON + template HTML + asset references. Import re-templates into Oyatie `marketing_asset.template` and rewrites asset URLs to drive µservice. SEO block is preserved verbatim. Password-protected pages re-hash the password to Argon2id.

Marketo LP and Mailchimp LP follow analogous patterns; mapping details in `migration-playbooks/from-marketo.md` §4.3 and `migration-playbooks/from-mailchimp.md` §4.3.

## Cross-µservice Handoffs

- `sites` provisions tenant CNAME and reverse-proxy routing (per ADR-MS-MA-002).
- `forms` substrate hosts attached form per ADR-MS-MA-003.
- `marketing-asset` provides template + image references.
- `drive` hosts large image assets.
- `attribution` consumes `EVT-MARKETING-LANDING-PAGE-CONVERTED` as a touch event.
- `audit-chain` seals every lifecycle event.
- `data-boundary` labels conversion subject hashes.
- `analytics` consumes page-view events for customer-analytics reports.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-032-landing-page.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/marketing-automation/IP-032-landing-page.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-032-landing-page.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-032-landing-page.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
