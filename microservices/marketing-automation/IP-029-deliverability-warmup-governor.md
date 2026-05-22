---
doc_class: ImplementationPlan
ip_id: IP-029-deliverability-warmup-governor
microservice: marketing-automation
related_adrs: [ADR-0253, ADR-0263, ADR-0273, ADR-0297, ADR-0321]
journey_id: J-MA-29-domain-warmup-before-campaign
status: proposed
date: 2026-05-20
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-029: Deliverability Warmup Governor

## Context

This net-new slice protects tenant sending reputation before high-volume campaign launch. It displaces Marketo deliverability packs, HubSpot email health, Mailchimp Premium delivery optimization, Iterable deliverability analytics, and Braze email deliverability controls while handing actual send execution to the mail microservice.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_deliverability_warmup` | `warmup_id` | `uuid primary key` | One domain/channel warmup plan. |
| `marketing_deliverability_warmup` | `tenant_id` | `uuid not null` | Tenant partition. |
| `marketing_deliverability_warmup` | `domain_ref` | `text not null` | DKIM/SPF/DMARC domain from mail owner. |
| `marketing_deliverability_warmup` | `daily_send_cap` | `integer not null` | Dynamic cap by reputation. |
| `marketing_deliverability_warmup` | `bounce_rate_ppm` | `integer not null default 0` | Parts per million. |
| `marketing_deliverability_warmup` | `complaint_rate_ppm` | `integer not null default 0` | Abuse signal. |
| `marketing_deliverability_warmup` | `state` | `text not null` | `warming`, `healthy`, `paused`, `blocked`. |

## API Endpoints

REST `POST /v1/marketing-automation/deliverability/warmups/{warmup_id}:admit-send`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-ma000001",
  "domain_ref": "mail-domain:example.com",
  "campaign_id": "cmp_q2_upgrade",
  "requested_recipients": 25000,
  "send_window": "2026-05-21T15:00:00Z/2026-05-21T17:00:00Z"
}
```

gRPC `DeliverabilityGovernor.AdmitSend(AdmitSendRequest)` returns `admitted_count`, `deferred_count`, and `pause_reason`.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `Service::"journey-runner"` | `marketingAutomation::AdmitSendVolume` | `DeliverabilityWarmup::*` | `tenant_id`, `domain_ref`, `requested_recipients` |
| `User::"marketing.ops"` | `marketingAutomation::PauseWarmup` | `DeliverabilityWarmup::*` | `state`, `bounce_rate_ppm`, `complaint_rate_ppm` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Marketo Email Deliverability Program | `DeliverabilityWarmup` | program status maps to `state`. |
| HubSpot Email Health | `DeliverabilityWarmup` | health score maps to cap policy input. |
| Mailchimp Premium Delivery Optimization | `DeliverabilityWarmup` | send-time optimization stays advisory. |
| Iterable Deliverability Analytics | `DeliverabilityWarmup` | bounce/complaint rates map to ppm fields. |
| Braze Email Deliverability | `DeliverabilityWarmup` | IP/domain pool maps to `domain_ref`. |

## Workflow Steps

1. `LoadDomainReputation` reads mail-owned DKIM/SPF/DMARC health.
2. `ComputeCap` derives daily cap from warmup state and complaint ppm.
3. `AdmitOrDefer` splits recipient batch.
4. `PublishSendBudget` sends admitted count to workflow-engine.
5. `UpdateWarmupState` pauses on bounce/complaint thresholds.

Branches: DMARC failure blocks all marketing mail; complaint spike pauses warmup; tenant admin override requires Cedar and audit.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-DELIVERABILITY-ADMITTED` | `tenant_id`, `warmup_id`, `admitted_count`, `deferred_count` |
| `EVT-MARKETING-DELIVERABILITY-PAUSED` | `tenant_id`, `domain_ref`, `bounce_rate_ppm`, `complaint_rate_ppm` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Admit send volume | 25 ms | 100 ms | 250 ms | 2k checks/s/cell | 99.99% |
| Warmup metric update | 80 ms | 350 ms | 900 ms | 500 updates/min/cell | 99.95% |

## Failure Modes + Recovery

- Mail domain health unavailable: fail closed to deferred send and retry after 5 minutes.
- Complaint rate threshold exceeded: pause state and require explicit resume.
- Vendor migration lacks domain history: start at conservative cap of 500/day/domain.

## Migration Notes

Vendor health scores are not portable. Marketo, HubSpot, Mailchimp Premium, Iterable, and Braze exports provide evidence inputs only; Oyatie recomputes caps from mail-owned bounce and complaint streams.

## Cross-µservice Handoffs

- `mail` owns DKIM/SPF/DMARC and bounce events.
- `abuse-defence` supplies complaint and trap signals.
- `workflow-engine` receives admitted send budget.
- `audit-chain` seals admission and pause events.
- `finops` receives deferred-send cost impact.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-029-deliverability-warmup-governor.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-029-deliverability-warmup-governor.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-029-deliverability-warmup-governor.md` matched [`finops`, `cost`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-029-deliverability-warmup-governor.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
