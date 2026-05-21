---
doc_class: ImplementationPlan
ip_id: IP-030-cross-channel-frequency-cap
microservice: marketing-automation
related_adrs: [ADR-0243, ADR-0263, ADR-0272, ADR-0321]
journey_id: J-MA-30-cross-channel-fatigue-control
status: proposed
date: 2026-05-20
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-030: Cross-Channel Frequency Cap

## Context

This net-new slice prevents campaign fatigue across email, SMS, push, in-app, and webhook handoffs. It displaces Marketo communication limits, HubSpot frequency safeguards, Mailchimp Premium contact rating heuristics, Iterable frequency capping, and Braze rate limits with a tenant-visible ledger.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_frequency_window` | `window_id` | `uuid primary key` | Per subject-purpose-channel window. |
| `marketing_frequency_window` | `tenant_id` | `uuid not null` | Tenant partition. |
| `marketing_frequency_window` | `subject_ref` | `text not null` | Hashed subject. |
| `marketing_frequency_window` | `purpose` | `text not null` | Consent purpose. |
| `marketing_frequency_window` | `channel` | `text not null` | Channel name or `all`. |
| `marketing_frequency_window` | `max_touches` | `integer not null` | Cap for window. |
| `marketing_frequency_window` | `touch_count` | `integer not null default 0` | Current count. |
| `marketing_frequency_window` | `window_expires_at` | `timestamptz not null` | Rolling expiration. |

## API Endpoints

REST `POST /v1/marketing-automation/frequency:reserve-touch`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-ma000001",
  "subject_ref": "profile_hash_98f1",
  "purpose": "product_marketing",
  "channel": "email",
  "campaign_id": "cmp_q2_upgrade",
  "window": "P7D",
  "max_touches": 3
}
```

gRPC `MarketingFrequencyService.ReserveTouch(ReserveTouchRequest)` returns `reserved`, `remaining_touches`, and `window_expires_at`.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `Service::"journey-runner"` | `marketingAutomation::ReserveFrequencyTouch` | `FrequencyWindow::*` | `tenant_id`, `subject_ref`, `purpose`, `channel`, `campaign_id` |
| `User::"marketing.admin"` | `marketingAutomation::ChangeFrequencyCap` | `FrequencyPolicy::*` | `old_cap`, `new_cap`, `pack_id`, `justification` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Marketo Communication Limit | `MarketingFrequencyPolicy` | limit maps to purpose/channel cap. |
| HubSpot Frequency Safeguard | `MarketingFrequencyPolicy` | send frequency maps to rolling window. |
| Mailchimp Premium Contact Rating | `MarketingFrequencySignal` | rating is advisory, not a cap. |
| Iterable Frequency Cap | `MarketingFrequencyPolicy` | message type maps to purpose. |
| Braze Rate Limit | `MarketingFrequencyWindow` | canvas/campaign cap maps to per-subject reservation. |

## Workflow Steps

1. `ResolveSubjectWindow` loads current subject-purpose-channel window.
2. `EvaluateCapPolicy` merges tenant and pack rules.
3. `ReserveTouch` increments count atomically when below cap.
4. `DenyOrDefer` returns delay until window expiration when above cap.
5. `SealReservation` records reservation proof for send node.

Branches: cap exceeded defers send; legal notice purpose bypass requires Cedar; missing subject hash denies.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-FREQUENCY-TOUCH-RESERVED` | `tenant_id`, `subject_ref_hash`, `purpose`, `channel`, `remaining_touches` |
| `EVT-MARKETING-FREQUENCY-CAP-DENIED` | `tenant_id`, `campaign_id`, `window_expires_at`, `cap` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Reserve touch | 9 ms | 45 ms | 100 ms | 40k reservations/s/cell | 99.99% |
| Change cap policy | 50 ms | 220 ms | 500 ms | 100 changes/min/cell | 99.95% |

## Failure Modes + Recovery

- Counter write contention: retry with compare-and-swap up to 3 times, then defer send.
- Valkey cache miss: read Postgres source of truth and refresh cache.
- Incorrect tenant cap migration: preserve vendor cap in shadow mode until admin approval.

## Migration Notes

Vendor systems cap by campaign, list, message type, or canvas. Oyatie migrates those into a single subject-purpose-channel window so cross-channel fatigue cannot hide behind product boundaries.

## Cross-µservice Handoffs

- `data-boundary` hashes subject references.
- `workflow-engine` consumes reservation decisions before send nodes.
- `mail`, `messenger`, and `notification` receive deferred-send reasons.
- `consent` supplies purpose compatibility.
- `audit-chain` seals reservation and denial events.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-030-cross-channel-frequency-cap.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-030-cross-channel-frequency-cap.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].
