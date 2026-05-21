---
doc_class: ImplementationPlan
ip_id: IP-039-email-tracking
microservice: marketing-automation
related_adrs: [ADR-0244, ADR-0251, ADR-0263, ADR-0321, ADR-0328]
bounded_context: email-tracking
journey_id: J-MA-39-email-engagement-telemetry
status: proposed
date: 2026-05-21
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-039: Email Tracking

## Context

Email tracking ingests open + click + reply + bounce events for marketing emails. HubSpot Email Tracking (Sales + Marketing), Marketo Email Insights, and Mailchimp Click/Open Reports are universal. This slice respects GPC + DNT + Apple Mail Privacy Protection — counterparts treat MPP-confirmed opens as ordinary opens, inflating open-rate metrics. Oyatie aggregates MPP-confirmed opens separately.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_email_tracking_event` | `event_id` | `uuid primary key` | Event id. |
| `marketing_email_tracking_event` | `tenant_id` | `uuid not null` | Tenant partition. |
| `marketing_email_tracking_event` | `email_id` | `uuid not null` | FK to email. |
| `marketing_email_tracking_event` | `subject_hash` | `text not null` | Subject ref. |
| `marketing_email_tracking_event` | `kind` | `text not null` | open / click / reply / bounce / spam_complaint. |
| `marketing_email_tracking_event` | `link_target` | `text` | For click events. |
| `marketing_email_tracking_event` | `link_position` | `int` | Click position in email. |
| `marketing_email_tracking_event` | `bounce_kind` | `text` | hard / soft. |
| `marketing_email_tracking_event` | `mpp_aggregated` | `boolean not null default false` | Apple Mail Privacy Protection. |
| `marketing_email_tracking_event` | `privacy_signal` | `text` | gpc / dnt / none. |
| `marketing_email_tracking_event` | `recorded_at_hlc` | `hlc not null` | HLC stamp. |

## API Endpoints

The tracking pixel + click redirector are public endpoints under `/track/`:

- `GET /track/open/{tenant_id}/{email_id}/{subject_token}.gif` — returns 1x1 pixel; records open.
- `GET /track/click/{tenant_id}/{email_id}/{link_token}` — 302 redirects to `link_target`; records click.
- `POST /webhooks/mail/bounce` (from mail µservice) — records bounce.
- `POST /webhooks/mail/spam-complaint` — records spam complaint.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `Service::"tracking-pixel-receiver"` | `marketingAutomation::RecordTrackingEvent` | `MarketingEmailTrackingEvent::*` | `kind`, `pack_overlay`, `privacy_signal` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| HubSpot Email Open / Click | `MarketingEmailTrackingEvent` | HubSpot pixel maps to `kind == 'open'`. |
| HubSpot Email Reply | `MarketingEmailTrackingEvent.kind == 'reply'` | Sales Hub Reply Tracking; needs inbound IMAP/SMTP connect. |
| Marketo Email Open / Click | `MarketingEmailTrackingEvent` | Marketo `EMAIL_OPENED` event maps. |
| Mailchimp Click / Open | `MarketingEmailTrackingEvent` | Mailchimp `email.clicked` event maps. |

## Workflow Steps

1. `ResolveSubjectToken` decodes the subject_token (HMAC-signed) to recover subject_hash + email_id.
2. `DetectPrivacySignal` reads request headers for GPC + DNT.
3. `DetectMPP` inspects the IP range + User-Agent for Apple MPP indicators.
4. If `privacy_signal == 'gpc'` and pack overlay enforces GPC honor → record event with `aggregated_only: true` (no subject-level row).
5. If `mpp_detected`, record event with `mpp_aggregated: true` (subject-level row retained for the aggregate cohort, not for the individual subject).
6. Otherwise record normally.
7. `EmitTrackingEvent` writes row and emits `EVT-MARKETING-EMAIL-TRACKING-*`.
8. `FeedAttribution` posts the event to attribution touch ingestion.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-EMAIL-TRACKING-OPEN` | `tenant_id`, `email_id`, `subject_hash`, `mpp_aggregated`, `privacy_signal` |
| `EVT-MARKETING-EMAIL-TRACKING-CLICK` | `email_id`, `subject_hash`, `link_target`, `link_position`, `privacy_signal` |
| `EVT-MARKETING-EMAIL-TRACKING-REPLY` | `email_id`, `subject_hash`, `reply_intent: detected_or_unknown` |
| `EVT-MARKETING-EMAIL-TRACKING-BOUNCE` | `email_id`, `subject_hash`, `bounce_kind` |
| `EVT-MARKETING-EMAIL-TRACKING-SPAM-COMPLAINT` | `email_id`, `subject_hash`, `complaint_source` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Record open (pixel) | 8 ms | 30 ms | 80 ms | 50000 rps/cell | 99.99% |
| Record click (redirect) | 12 ms | 50 ms | 120 ms | 20000 rps/cell | 99.99% |

## Failure Modes + Recovery

- Subject token forged or expired → return pixel/redirect but skip event recording; emit `EVT-MARKETING-TRACKING-TOKEN-INVALID`.
- Click target host unreachable for prefetch validation → record click anyway; redirect with `noopener noreferrer`.
- Spam complaint feedback loop unavailable → buffer events; mail µservice replays when feedback loop recovers.

## Migration Notes

HubSpot, Marketo, Mailchimp track each open/click in vendor-proprietary stores. Migration imports historical events with `migrated_from: <vendor>` flag and original event ID for replay. MPP detection logic is unique to Oyatie — vendors do not flag MPP-confirmed opens distinctly.

## Cross-µservice Handoffs

- `mail` posts bounce + spam-complaint webhooks.
- `attribution` consumes tracking events as touch events.
- `data-boundary` labels subject_hash by data_class.
- `audit-chain` seals every event.
- `email-tracking` events feed `customer-analytics.email_engagement_report`.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-039-email-tracking.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-039-email-tracking.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-039-email-tracking.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-039-email-tracking.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
