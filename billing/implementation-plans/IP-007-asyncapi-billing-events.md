---
ip_id: IP-007
microservice: cloud-billing
title: AsyncAPI 3.1.0 billing events — CloudEvents 1.0 envelope + protobuf payload
wave: Wave-15B-cloud-billing-spec-sprint
date: 2026-05-21
owner: axis-cloud-billing
status: drafted
priority: P0
binding_adrs: [ADR-0145, ADR-0263, ADR-0244, ADR-0131]
counterpart_parity: [Stripe Webhooks, Recurly webhooks, Zuora event-bus, Chargebee webhooks]
capabilities_touched:
  - cap.cloud.billing.emit_usage_event
billing_components: [per_seat, per_usage, revenue_share]
tenant_class_scope: both
---

# IP-007 — AsyncAPI 3.1.0 billing event stream

## §A Objective

Document the existing AsyncAPI 3.1.0 contract at `contracts/asyncapi/cloud/cloud-billing-events-v1.yaml` (50 lines) which declares the canonical event ingest channel using **CloudEvents 1.0** envelope + **Protobuf** payload (proto3 schema from `contracts/proto/cloud/billing/v1/cloud-billing-event-v1.proto`).

cloud-billing emits a domain event for every state-changing operation. The event stream is the source-of-truth for downstream consumers: finops-portal aggregations, audit-chain seal entries, cloud-iam principal cache invalidation, observability per-tenant cost metrics, and FOCUS / ERP export jobs.

## §B Scope

In scope:

- Channel `oya.cloud.billing` (CloudEvents source `oyatie://cloud/billing`).
- Message `cloud.billing.event.ingest.v1` — the canonical usage event ingest.
- CloudEvents 1.0 header schema (specversion, id, source, type, subject, time, datacontenttype).
- Protobuf payload schema (referenced via `proto/cloud/billing/v1/cloud-billing-event-v1.proto#/cloud.billing.v1.CloudBillingEventIngest`).
- Companion event types (not yet in AsyncAPI but documented here for full surface): `cloud.billing.invoice.issued.v1`, `cloud.billing.invoice.voided.v1`, `cloud.billing.credit_memo.issued.v1`, `cloud.billing.subscription.created.v1`, `cloud.billing.subscription.modified.v1`, `cloud.billing.settlement.computed.v1`, `cloud.billing.payout.initiated.v1`, `cloud.billing.tenant_class.converted.v1`, `cloud.billing.billing_components.mutated.v1`, `cloud.billing.cap.breach.detected.v1`.

Out of scope:

- REST API surface (IP-006).
- gRPC service surface (IP-008).
- audit-chain seal mechanics (IP-010).

## §C Architecture

### §C.1 CloudEvents 1.0 envelope

Per AsyncAPI line 25–46, the envelope shape is:

```
specversion: '1.0'
id: <ulid>
source: oyatie://cloud/billing
type: cloud.billing.event.ingest.v1
subject: tenant/{tenant_id}/resource/{resource_id}
time: <RFC 3339 timestamp>
datacontenttype: application/protobuf
```

The `source` is fixed (`oyatie://cloud/billing`); the `type` follows `cloud.billing.<entity>.<verb>.v<version>` dotted-snake-case per ADR-0263. `subject` carries the principal-bound path for downstream filtering. Time is RFC 3339; ULID id enables time-sortable de-dup at consumers.

### §C.2 Channel topology

| Channel address | Direction | Producers | Consumers |
|---|---|---|---|
| `oya.cloud.billing` | publish | Every Phase-0/1/2 µservice emitting usage | cloud-billing ingest-worker |
| `oya.cloud.billing.invoice` | publish | cloud-billing | finops-portal, audit-chain, ERP-export |
| `oya.cloud.billing.settlement` | publish | cloud-billing-settlement-worker | payments, audit-chain, finops-portal |
| `oya.cloud.billing.tenant_class` | publish | cloud-billing | cloud-iam, tenancy |
| `oya.cloud.billing.cap` | publish | cloud-billing-cap-watcher | observability-notification, finops-portal |
| `oya.cloud.billing.dunning` | publish | cloud-billing-dunning-worker | payments, observability-notification |

All channels broadcast via the messenger µservice's NATS-class substrate per ADR-0253.

### §C.3 Why CloudEvents

CloudEvents 1.0 is the CNCF-standard event envelope; choosing it gives:

- Cross-µservice envelope uniformity (every Oyatie event channel uses the same schema).
- Off-the-shelf SDK support for downstream consumers.
- Vendor-neutral interop for tenant-side webhooks (tenants can subscribe via api-gateway with CloudEvents semantics).
- Built-in `id` + `source` + `time` for replay/dedup without per-channel schema work.

### §C.4 Why Protobuf payload (not JSON)

Per `defaultContentType: application/cloudevents+protobuf` and `datacontenttype: application/protobuf`:

- Strong typing across cross-µservice and cross-runtime (Rust producers + Rust consumers).
- Wire compactness (cloud-billing emits >100M events/day at scale; JSON would dominate network cost).
- Schema evolution discipline (proto3 backward-compatibility rules).
- Single source of schema truth (the `.proto` file is the cited schema in AsyncAPI — no schema duplication).

### §C.5 Event taxonomy (full list)

| Event type | Producer | Carrier | Cedar gate at emit |
|---|---|---|---|
| `cloud.billing.event.ingest.v1` | any Phase-0/1/2 µservice | CloudBillingEventCreate (proto) | cap.cloud.billing.emit_usage_event |
| `cloud.billing.invoice.issued.v1` | cloud-billing-invoice-worker | Invoice (proto) | cap.cloud.billing.issue_invoice |
| `cloud.billing.invoice.voided.v1` | oyatie-finance-operator (via API) | Invoice + reason | cap.cloud.billing.void_invoice |
| `cloud.billing.credit_memo.issued.v1` | oyatie-finance-operator | CreditMemo + reason + original_invoice_id | cap.cloud.billing.issue_credit_memo |
| `cloud.billing.subscription.created.v1` | tenant-admin | Subscription | cap.cloud.billing.create_subscription |
| `cloud.billing.subscription.modified.v1` | tenant-admin | Subscription + op (CHANGE_PLAN/PAUSE/RESUME/CANCEL) | cap.cloud.billing.modify_subscription |
| `cloud.billing.settlement.computed.v1` | cloud-billing-settlement-worker | SettlementStatement | cap.cloud.billing.compute_settlement |
| `cloud.billing.payout.initiated.v1` | cloud-billing-settlement-worker | statement_id + payment_handle | cap.cloud.billing.initiate_payout |
| `cloud.billing.tenant_class.converted.v1` | tenant-admin | tenant_id + new_class + billing_components | cap.cloud.billing.convert_tenant |
| `cloud.billing.billing_components.mutated.v1` | tenant-admin | tenant_id + op + component + amendment | cap.cloud.billing.mutate_billing_components |
| `cloud.billing.cap.breach.detected.v1` | cloud-billing-cap-watcher | tenant_id + axis + actual + ceiling | — (system event) |
| `cloud.billing.cap.soft_breach.detected.v1` | cloud-billing-cap-watcher | tenant_id + axis + percentage | — |
| `cloud.billing.trial.expiring.v1` | cloud-billing-trial-worker | tenant_id + expires_at | — |
| `cloud.billing.reservation.purchased.v1` | tenant-finops-admin | Reservation | cap.cloud.billing.purchase_reservation |
| `cloud.billing.reservation.converted.v1` | tenant-finops-admin | Reservation | cap.cloud.billing.convert_reservation |
| `cloud.billing.fx_lock.recorded.v1` | cloud-billing-fx-worker | lock_id + base/quote + rate | — |
| `cloud.billing.focus_export.completed.v1` | cloud-billing-export-worker | tenant_id + period + destination_object_ref | — |
| `cloud.billing.erp_export.completed.v1` | cloud-billing-export-worker | tenant_id + period + connector_kind | — |

### §C.6 Replay and dedup

Every event carries a ULID `id` and a `subject` with tenant scoping. Consumers dedup by `(source, id)` tuple per CloudEvents 1.0 §3.1.1. The ingest channel additionally honors `idempotency_key` inside the payload (per IP-001 §D.2) — this is double-belt-and-suspenders for the ingest direction where consumers may replay due to message-broker at-least-once delivery semantics.

audit-chain (IP-010) consumes the event stream and seals each event by hash (Ed25519 + Merkle); the seal hash is returned as `AuditChainHeader.audit_chain_hash` on the gRPC response (per `cloud-billing.proto` line 117–121).

## §D Lifecycle

### §D.1 Event publication

1. Cloud-billing (or upstream Phase-0/1/2 µservice) emits a CloudEvents envelope wrapping a proto3-encoded payload.
2. messenger µservice routes to subscribers per channel filter.
3. Consumers process (with at-least-once delivery).
4. audit-chain seals (IP-010) within the per-event SLO window.

### §D.2 Event consumption (audit-chain)

1. audit-chain reads `oya.cloud.billing.*` channels.
2. Each event is sealed with Ed25519 signature + appended to per-tenant Merkle tree.
3. Seal hash is fed back to cloud-billing via gRPC `AuditChainHeader.audit_chain_hash` on the next mutation response.

### §D.3 Failure modes

- Schema drift (consumer expects proto3 v1 but cloud-billing emits v2): proto3 backward compatibility rules prevent this; new fields are added with new field numbers; old fields are not renumbered.
- Out-of-order delivery: messenger guarantees per-channel ordering; consumers honor `time` for ordering hints but treat `id` as the dedup key.
- Replay storm: messenger throttles per-subscriber; cloud-billing's ingest worker honors `idempotency_key` payload field.

## §E Cedar Policy Bindings

- `cap.cloud.billing.emit_usage_event` (cloud-billing.cedar lines 80–89) — guards production into `oya.cloud.billing`.
- audit-chain consumer side uses its own Cedar gates (out of scope here).

## §F Evidence

### §F.1 Source files

- `/Users/jasonlee/oyatie/contracts/asyncapi/cloud/cloud-billing-events-v1.yaml` (50 lines).
- `/Users/jasonlee/oyatie/contracts/asyncapi/cloud/cloud-billing-events-v1.meta.yaml` (governance metadata).
- `/Users/jasonlee/oyatie/contracts/proto/cloud/billing/v1/cloud-billing-event-v1.proto` (referenced schema).
- `/Users/jasonlee/oyatie/billing/contracts/proto/cloud-billing.proto` lines 117–121 (AuditChainHeader), 223–263 (event ingest + retrieval messages).

### §F.2 ADR anchors

- ADR-0145 inter-µservice communication reform: direct gRPC + 3 invariants; AsyncAPI is the cross-µservice event channel surface.
- ADR-0263 audit-chain seal hash emission.
- ADR-0244 tenant scoping (every event carries tenant_id via subject path).
- ADR-0253 messenger NATS-class substrate.

## §G Counterpart parity

| Counterpart | Their event model | Oyatie equivalent | Delta |
|---|---|---|---|
| Stripe Webhooks | Webhook endpoints registered per account; events delivered HTTP POST | AsyncAPI channels + messenger-routed CloudEvents; tenants subscribe via api-gateway webhook adapter | Stripe is HTTP-push-only; oyatie has internal pub/sub + tenant-facing HTTP webhook adapter. Internal events stay on messenger. |
| Stripe Webhooks | Event type strings: `invoice.created`, `invoice.payment_succeeded`, `customer.subscription.updated` | Event type strings: `cloud.billing.invoice.issued.v1`, `cloud.billing.subscription.modified.v1` | Stripe lacks version in type; oyatie embeds version per CloudEvents convention. |
| Stripe Webhooks | At-least-once delivery; signature on each POST | At-least-once delivery; Ed25519 audit seal | Same delivery semantics; oyatie's seal is stronger (Merkle anchor). |
| Recurly webhooks | XML payloads, HTTP POST | Protobuf payloads, CloudEvents envelope | Recurly is older XML-based; oyatie is modern proto-based. |
| Zuora event bus | Internal Kafka-based event bus + outbound notifications | messenger NATS-class internal + tenant-webhook adapter | Same topology; different broker substrate. |
| Chargebee webhooks | JSON payloads, "events" API with replay endpoint | Protobuf payloads, replay via CloudEvents `id` dedup | Same replay story. |
| AWS EventBridge | CloudEvents-compatible event bus with custom rules | messenger channel filters + CloudEvents | Direct architectural parity. |

## §H Open questions

- Whether to expose tenant-facing CloudEvents webhook endpoint at `/v1/cloud/billing/webhooks` for self-registration. Current decision: implement as part of api-gateway webhook-adapter feature; cloud-billing only publishes internal events.
- Whether to add per-tenant event-replay endpoint for disaster recovery. Current decision: audit-chain owns replay capability via its sealed log; cloud-billing does not duplicate.
