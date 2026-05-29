---
doc_class: ImplementationPlan
ip_id: IP-045-marketing-asset
microservice: marketing-automation
related_adrs: [ADR-0244, ADR-0263, ADR-0321, ADR-0328]
bounded_context: marketing-asset
journey_id: J-MA-45-template-and-design-asset-library
status: proposed
date: 2026-05-21
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-045: Marketing Asset

## Context

Marketing assets are templates + files + design blocks + snippets + brand kit. HubSpot Design Manager + Files + Snippets + Templates + Custom Modules + Themes + Brand Kit + Marketo Design Studio + Mailchimp Content Studio cover the canonical surface. Differentiator is per-locale variants resolved per pack overlay — counterparts treat localisation as per-asset duplication; Oyatie resolves at render time.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_asset` | `asset_id` | `uuid primary key` | Asset id. |
| `marketing_asset` | `tenant_id` | `uuid not null` | Tenant partition. |
| `marketing_asset` | `kind` | `text not null` | template / file / design_block / snippet / brand_kit. |
| `marketing_asset` | `name` | `text not null` | Unique per (tenant, kind). |
| `marketing_asset` | `payload_storage_ref` | `text not null` | drive reference for large objects; inline for small. |
| `marketing_asset` | `payload_inline` | `jsonb` | Inline payload (≤ 64 KB). |
| `marketing_asset` | `locale_variants` | `jsonb not null default '{}'` | Per-locale payload overrides. |
| `marketing_asset` | `accessibility_audit_score` | `numeric(4,2)` | For template + design_block. |
| `marketing_asset` | `version` | `int not null` | Monotonic. |
| `marketing_asset` | `status` | `text not null` | draft / published / retired. |
| `marketing_asset` | `published_at_hlc` | `hlc` | HLC. |

## API Endpoints

REST `POST /v1/marketing-automation/marketing-assets` creates an asset.

REST `POST /v1/marketing-automation/marketing-assets/{asset_id}:publish` requires accessibility score ≥ 85.00 for templates + design_blocks.

REST `GET /v1/marketing-automation/marketing-assets/{asset_id}?locale=fr-CA` resolves locale variant.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"marketing.ops"` | `marketingAutomation::CreateMarketingAsset` | `MarketingAsset::*` | `tenant_class` |
| `User::"marketing.ops"` | `marketingAutomation::PublishMarketingAsset` | `MarketingAsset::asset_id` | `accessibility_audit_score`, `tenant_class` |

## Workflow Steps

1. `ValidateNameUnique` per (tenant_id, kind, name).
2. `StorePayload` writes inline or to drive depending on size.
3. `ResolveLocaleVariants` for templates with per-pack disclosure blocks.
4. `RunAccessibilityAudit` for template + design_block.
5. `AuthorizePublish` calls Cedar.
6. `Publish` transitions status + emits event.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-ASSET-CREATED` | `asset_id`, `kind`, `name`, `tenant_class` |
| `EVT-MARKETING-ASSET-PUBLISHED` | `asset_id`, `version`, `accessibility_audit_score`, `locale_variants_count` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Create asset | 60 ms | 250 ms | 700 ms | 100 rps/cell | 99.95% |
| Publish asset | 100 ms | 400 ms | 1.2 s | 50 rps/cell | 99.95% |
| Resolve asset (with locale) | 15 ms | 60 ms | 150 ms | 5000 rps/cell | 99.99% |

## Failure Modes + Recovery

- Drive unreachable on resolve → cached payload served if available; otherwise 502 + retry.
- Accessibility audit failure → 422 with remediation hints; asset remains draft.
- Locale variant missing for pack overlay → fall back to canonical; emit warning event.

## Migration Notes

HubSpot Design Manager exports Theme + Template + Module HTML/CSS/JS bundles; Marketo Design Studio exports image + template ZIPs; Mailchimp Content Studio exports template HTML. All preserve as `marketing_asset` rows with original payload as `payload_inline` or `payload_storage_ref`.

## Cross-µservice Handoffs

- `drive` hosts large file objects.
- `design-collaboration` for collaborative template editing.
- `email` + `landing-page` consume templates.
- `audit-chain` seals events.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-045-marketing-asset.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-045-marketing-asset.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].
