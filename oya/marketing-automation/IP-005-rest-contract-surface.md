---
doc_class: ImplementationPlan
ip_id: IP-005-rest-contract-surface
microservice: marketing-automation
related_adrs: [ADR-0253, ADR-0258, ADR-0263, ADR-0297, ADR-0321]
journey_id: J-MA-05-public-api-campaign-ops
status: proposed
date: 2026-05-20
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-005: Marketing Automation REST Contract Surface

## Context

This slice defines the first REST surface that Omar Watkins can smoke-test and rollback. It subsumes the public campaign APIs of Marketo, HubSpot, Mailchimp Premium, Iterable, and Braze with a tenant-scoped, versioned API that exposes explicit idempotency, dry-run, Cedar decision, and audit ids.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_api_idempotency` | `idempotency_key` | `text primary key` | Caller supplied key. |
| `marketing_api_idempotency` | `tenant_id` | `uuid not null` | Tenant partition. |
| `marketing_api_idempotency` | `route_id` | `text not null` | Contract route identifier. |
| `marketing_api_idempotency` | `request_hash` | `bytea not null` | Prevents replay mismatch. |
| `marketing_api_idempotency` | `response_body` | `jsonb` | Stored success or safe failure body. |
| `marketing_api_idempotency` | `expires_at` | `timestamptz not null` | 24h for writes, 7d for imports. |

## API Endpoints

REST endpoints in OpenAPI 3.2.0:

```http
POST /v1/marketing-automation/campaigns
POST /v1/marketing-automation/segments/preview
POST /v1/marketing-automation/journeys/{journey_id}:launch
GET  /v1/marketing-automation/audit-events/{event_id}
```

Example campaign create:

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-ma000001",
  "scope_id": "01HXMA_SCOPE",
  "campaign_key": "q2-upgrade",
  "purpose": "product_marketing",
  "channels": ["email", "in_app"],
  "dry_run": true
}
```

gRPC parity: `MarketingCampaignRestBridge.CreateCampaign` is internal only and exists to keep REST and worker command schemas identical.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"marketing.manager"` | `marketingAutomation::CreateCampaign` | `Campaign::*` | `tenant_id`, `scope_id`, `purpose`, `channels`, `dry_run` |
| `User::"marketing.manager"` | `marketingAutomation::LaunchJourney` | `Journey::*` | `tenant_id`, `journey_id`, `suppression_revision` |
| `Service::"api-gateway"` | `marketingAutomation::ReplayIdempotentWrite` | `IdempotencyRecord::*` | `request_hash`, `route_id` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Marketo Campaign | `MarketingCampaign` | `programId -> source_ref.id`; campaign channel becomes delivery plan. |
| HubSpot Campaign | `MarketingCampaign` | `campaignGuid -> source_ref.id`; owner id maps to principal ref. |
| Mailchimp Premium Campaign | `MarketingCampaign` | `campaign_id -> source_ref.id`; audience id maps through IP-001. |
| Iterable Campaign | `MarketingCampaign` | `campaignId -> source_ref.id`; template id becomes content ref. |
| Braze Campaign | `MarketingCampaign` | `campaign_id -> source_ref.id`; canvas id links to workflow template. |

## Workflow Steps

1. `AuthenticateGatewayPrincipal` requires API gateway service principal.
2. `ValidateContractVersion` rejects unsupported media versions.
3. `CheckIdempotency` returns prior response for duplicate key and matching hash.
4. `EvaluateCedar` authorizes the command.
5. `DispatchCommand` writes command row or enqueues async worker.
6. `ReturnAuditAwareResponse` includes `audit_event_id`.

Branches: no idempotency key returns `428`; dry-run returns preview without mutation; abuse score over threshold returns `429` with retry budget.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-API-WRITE-ACCEPTED` | `tenant_id`, `route_id`, `idempotency_key`, `audit_event_id` |
| `EVT-MARKETING-API-DRY-RUN` | `tenant_id`, `route_id`, `validation_errors[]`, `projection_hash` |
| `EVT-ERROR-MARKETING-API` | `route_id`, `status_code`, `recovery_branch` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Campaign write accepted | 70 ms | 280 ms | 650 ms | 600 rps/cell | 99.95% |
| Segment preview accepted | 90 ms | 500 ms | 1.2 s | 250 rps/cell | 99.9% |

## Failure Modes + Recovery

- Duplicate idempotency key with different body: return `409 idempotency_hash_mismatch` and emit no second mutation.
- API gateway abuse signal: throttle by tenant and source IP; preserve audit row.
- Contract version drift: return `426 unsupported_contract_version` with migration link.

## Migration Notes

Vendor APIs often accept partial campaign writes without clear idempotency semantics. Oyatie migration wrappers must synthesize idempotency keys from source id plus revision and must surface dry-run validation before writing campaign state.

## Cross-µservice Handoffs

- `api-gateway` terminates HTTP/3 and forwards request context.
- `abuse-defence` scores suspicious automation.
- `workflow-engine` receives launch commands.
- `audit-chain` stores route-level audit events.
- `sdk` generation consumes this REST contract after OpenAPI materialization.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-005-rest-contract-surface.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-005-rest-contract-surface.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].
