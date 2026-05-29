---
doc_class: ImplementationPlan
ip_id: IP-043-marketing-calendar
microservice: marketing-automation
related_adrs: [ADR-0244, ADR-0263, ADR-0321, ADR-0328]
bounded_context: marketing-calendar
journey_id: J-MA-43-multi-channel-calendar
status: proposed
date: 2026-05-21
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-043: Marketing Calendar

## Context

Marcus Chen coordinates multi-channel campaigns (email + landing-page + social + ad) across his team. HubSpot Marketing Calendar + Marketo Calendar + Mailchimp Content Studio Calendar all expose timeline views. The differentiator is conflict-detection per (channel, audience overlap, week) — counterparts surface visualization only; Oyatie flags overlapping audience × week conflicts as auditable events requiring acknowledgement.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_calendar_entry` | `entry_id` | `uuid primary key` | Calendar entry id. |
| `marketing_calendar_entry` | `tenant_id` | `uuid not null` | Tenant partition. |
| `marketing_calendar_entry` | `entry_kind` | `text not null` | email_send / landing_page_launch / workflow_canvas_publish / social_post / ad_campaign / meeting / blog_post. |
| `marketing_calendar_entry` | `subject_id` | `uuid not null` | FK to entry-kind-specific aggregate. |
| `marketing_calendar_entry` | `audience_descriptor` | `jsonb not null` | Segment ids / list ids / account ids. |
| `marketing_calendar_entry` | `scheduled_at` | `tstzrange not null` | Time window. |
| `marketing_calendar_entry` | `channel` | `text not null` | email / web / social / paid / etc. |
| `marketing_calendar_entry` | `status` | `text not null` | scheduled / live / completed / cancelled. |
| `marketing_calendar_conflict` | `conflict_id` | `uuid primary key` | Conflict row. |
| `marketing_calendar_conflict` | `entry_id_a` | `uuid not null` | First entry. |
| `marketing_calendar_conflict` | `entry_id_b` | `uuid not null` | Second entry. |
| `marketing_calendar_conflict` | `audience_overlap_pct` | `numeric(5,2) not null` | Estimated audience overlap. |
| `marketing_calendar_conflict` | `detected_at_hlc` | `hlc not null` | HLC. |
| `marketing_calendar_conflict` | `resolution` | `text` | acknowledged / rescheduled / null. |

## API Endpoints

REST `POST /v1/marketing-automation/calendar/entries` schedules an entry.

REST `GET /v1/marketing-automation/calendar/entries?start={iso}&end={iso}&channel={ch}` lists.

REST `POST /v1/marketing-automation/calendar/conflicts/{conflict_id}:acknowledge` records operator acknowledgement.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"marketing.ops"` | `marketingAutomation::ScheduleCalendarEntry` | `MarketingCalendarEntry::*` | `tenant_class` |
| `Service::"conflict-detector"` | `marketingAutomation::DetectConflict` | `MarketingCalendarConflict::*` | tenant scope |

## Workflow Steps

1. `ResolveAudienceDescriptor` expands segment / list / account references.
2. `EnumerateOverlaps` finds existing entries with overlapping (audience, channel, week) tuples.
3. `EstimateAudienceOverlap` computes intersection cardinality (using HyperLogLog sketches for performance at scale).
4. If overlap > 30%, write `marketing_calendar_conflict` row.
5. `PersistEntry` writes the entry.
6. `EmitSchedule` emits `EVT-MARKETING-CALENDAR-SCHEDULED` (+ `EVT-MARKETING-CALENDAR-CONFLICT-DETECTED` if conflicts).

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-CALENDAR-SCHEDULED` | `entry_id`, `entry_kind`, `subject_id`, `scheduled_at`, `tenant_class` |
| `EVT-MARKETING-CALENDAR-CONFLICT-DETECTED` | `conflict_id`, `entry_id_a`, `entry_id_b`, `audience_overlap_pct` |
| `EVT-MARKETING-CALENDAR-CONFLICT-ACKNOWLEDGED` | `conflict_id`, `principal_id`, `resolution` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Schedule entry (with conflict detect) | 150 ms | 700 ms | 2 s | 100 rps/cell | 99.9% |
| List calendar entries | 30 ms | 120 ms | 300 ms | 500 rps/cell | 99.95% |

## Failure Modes + Recovery

- HyperLogLog sketch unavailable → fall back to exact count (slower); flag operator that estimate is exact for this entry.
- Audience descriptor references retired segment → 422 `audience_unresolved`.
- Conflict notification delivery failure → buffer; retry until acknowledged.

## Migration Notes

HubSpot Marketing Calendar export uses iCal; Marketo Calendar exports via Marketo API; Mailchimp Content Calendar exports via Mailchimp API. All preserve entry kind + subject reference + scheduled window. Vendor-side conflict detection is absent so Oyatie initializes with no pre-existing conflicts.

## Cross-µservice Handoffs

- `calendar` µservice provides meeting-scheduler primitive (entry_kind == 'meeting').
- `email`, `landing-page`, `workflow-canvas`, `social`, `advertising-platform` provide subject aggregates.
- `audit-chain` seals every change.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-043-marketing-calendar.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-043-marketing-calendar.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].
