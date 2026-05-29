---
doc_class: ImplementationPlan
ip_id: IP-042-webhook-subscription
microservice: marketing-automation
related_adrs: [ADR-0244, ADR-0253-amendment, ADR-0263, ADR-0321, ADR-0328]
bounded_context: webhook-subscription
journey_id: J-MA-42-tenant-integration-webhooks
status: proposed
date: 2026-05-21
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-042: Webhook Subscription

## Context

Tenant integrations subscribe to marketing-automation events (segment-materialized, form-submitted, lead-scored, lifecycle-progressed, attribution-reconciled, journey-step-advanced). HubSpot + Marketo + Mailchimp all expose webhook subscriptions with HMAC signing + retry policy. Oyatie's differentiator is HTTP/3 + QUIC delivery by default (counterparts default to HTTP/1.1).

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_webhook_subscription` | `subscription_id` | `uuid primary key` | Subscription id. |
| `marketing_webhook_subscription` | `tenant_id` | `uuid not null` | Tenant partition. |
| `marketing_webhook_subscription` | `target_url` | `text not null` | Subscriber endpoint. |
| `marketing_webhook_subscription` | `event_filter` | `text[] not null` | Array of event class names. |
| `marketing_webhook_subscription` | `signing_secret_id` | `text not null` | OpenBao reference. |
| `marketing_webhook_subscription` | `retry_policy` | `jsonb not null` | `{max_attempts: 6, backoff_ms: [60000, 300000, 900000, 3600000, 21600000, 86400000]}`. |
| `marketing_webhook_subscription` | `active` | `boolean not null default true` | Toggle. |
| `marketing_webhook_subscription` | `subscribed_at_hlc` | `hlc not null` | HLC. |
| `marketing_webhook_delivery` | `delivery_id` | `uuid primary key` | Per-delivery row. |
| `marketing_webhook_delivery` | `subscription_id` | `uuid not null` | FK. |
| `marketing_webhook_delivery` | `event_class` | `text not null` | Event being delivered. |
| `marketing_webhook_delivery` | `payload` | `jsonb not null` | Event payload. |
| `marketing_webhook_delivery` | `signature` | `text not null` | HMAC-SHA-256 over payload + timestamp. |
| `marketing_webhook_delivery` | `signed_timestamp` | `int8 not null` | Unix seconds; ±300s replay window. |
| `marketing_webhook_delivery` | `attempt_count` | `int not null default 0` | Retry counter. |
| `marketing_webhook_delivery` | `status` | `text not null` | pending / succeeded / failed_retry / failed_dead_letter. |
| `marketing_webhook_delivery` | `last_attempt_hlc` | `hlc` | HLC. |
| `marketing_webhook_delivery` | `last_response_code` | `int` | Last HTTP status received. |

## API Endpoints

REST `POST /v1/marketing-automation/webhooks/subscriptions`:

```json
{
  "tenant_id": "...",
  "target_url": "https://api.acme.io/oyatie/webhooks",
  "event_filter": [
    "EVT-MARKETING-FORM-SUBMITTED",
    "EVT-MARKETING-LEAD-SCORED",
    "EVT-MARKETING-LIFECYCLE-PROGRESSED",
    "EVT-MARKETING-ATTRIBUTION-RECONCILED"
  ]
}
```

Response includes `signing_secret` once (not retrievable thereafter; stored in OpenBao with ≤60s lease).

REST `POST /v1/marketing-automation/webhooks/subscriptions/{subscription_id}:rotate-secret` generates a new secret.

REST `GET /v1/marketing-automation/webhooks/deliveries/{delivery_id}` returns delivery log.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"marketing.ops"` | `marketingAutomation::SubscribeWebhook` | `MarketingWebhookSubscription::*` | `tenant_class`, `webhook_subscriptions_count`, `target_url_allowlist_ok` |
| `Service::"webhook-dispatcher"` | `marketingAutomation::DeliverWebhook` | `MarketingWebhookDelivery::*` | `event_class`, `subscription_active` |

Demo-trial gate: `tenant_class == 'demo_trial' && webhook_subscriptions_count >= 3` denies subscribe.

## Workflow Steps

1. `ValidateTargetUrl` checks https-only + DNS resolves + not in deny-list (no internal addresses).
2. `MintSigningSecret` generates 256-bit secret + stores in OpenBao.
3. `PersistSubscription` writes row.
4. On event matching subscription, `EnqueueDelivery` writes pending delivery row.
5. `Sign` computes HMAC-SHA-256(secret, payload || signed_timestamp).
6. `Deliver` posts to target_url over HTTP/3 with `oya-signature: sha256=<hex>` + `oya-timestamp: <unix>` headers.
7. On 2xx, mark succeeded; on 4xx, mark `failed_dead_letter` (no retry on 4xx); on 5xx or timeout, schedule retry per policy.
8. After max attempts, mark `failed_dead_letter` and emit `EVT-MARKETING-WEBHOOK-DELIVERY-FAILED`.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-WEBHOOK-SUBSCRIBED` | `subscription_id`, `target_url`, `event_filter`, `tenant_class` |
| `EVT-MARKETING-WEBHOOK-DELIVERY-ATTEMPTED` | `delivery_id`, `subscription_id`, `event_class`, `attempt_count` |
| `EVT-MARKETING-WEBHOOK-DELIVERY-SUCCEEDED` | `delivery_id`, `response_code`, `transport: http3_or_h2_fallback` |
| `EVT-MARKETING-WEBHOOK-DELIVERY-FAILED` | `delivery_id`, `attempt_count`, `last_response_code`, `dead_letter: bool` |
| `EVT-MARKETING-WEBHOOK-SECRET-ROTATED` | `subscription_id`, `principal_id` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Subscribe | 40 ms | 150 ms | 400 ms | 100 rps/cell | 99.95% |
| Deliver (first attempt) | 80 ms | 400 ms | 1.5 s | 5000 rps/cell | 99.9% |
| Retry queue depth | < 100 | < 1000 | < 5000 | n/a | n/a |

## Failure Modes + Recovery

- Subscriber endpoint unreachable → retry per policy; after max attempts, dead-letter.
- Subscriber endpoint signs with wrong secret → 4xx response treated as client error; alert tenant operator.
- Webhook flood (event volume exceeds delivery throughput) → admission control; defer non-critical events.
- HTTP/3 negotiation failure → fall back to HTTP/2 (transport recorded in audit event).

## Migration Notes

HubSpot Webhooks + Marketo Webhooks + Mailchimp Webhooks all use HMAC-SHA-256 signing; migration preserves subscription configuration. Signing secrets are re-minted (not exported from vendors).

## Cross-µservice Handoffs

- `openbao` stores per-subscription signing secret.
- `audit-chain` seals every delivery attempt.
- `finops` consumes per_usage `webhook_deliveries` meter.
- Outbound event publishers in this µservice + other µservices feed the dispatch queue.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-042-webhook-subscription.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-042-webhook-subscription.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-042-webhook-subscription.md` matched [`attribution`, `finops`, `per_usage`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-042-webhook-subscription.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
