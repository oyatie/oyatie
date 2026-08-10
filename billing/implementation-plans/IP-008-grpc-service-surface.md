---
ip_id: IP-008
microservice: cloud-billing
title: gRPC service surface — proto3 inter-microservice contracts
wave: Wave-15B-cloud-billing-spec-sprint
date: 2026-05-21
owner: axis-cloud-billing
status: drafted
priority: P0
binding_adrs: [ADR-0145, ADR-0253, ADR-0243, ADR-0263, ADR-0330, ADR-0131]
counterpart_parity: [Stripe API, Recurly API, Zuora SOAP/REST, Chargebee API]
capabilities_touched:
  - cap.cloud.billing.read_tenant_class
  - cap.cloud.billing.convert_tenant
  - cap.cloud.billing.mutate_billing_components
  - cap.cloud.billing.emit_usage_event
  - cap.cloud.billing.issue_invoice
  - cap.cloud.billing.purchase_reservation
  - cap.cloud.billing.compute_settlement
  - cap.cloud.billing.modify_subscription
billing_components: [per_seat, per_usage, revenue_share]
tenant_class_scope: both
---

# IP-008 — gRPC service surface: 11 services, 700-line proto3 contract

## §A Objective

Document the existing proto3 gRPC contract at `billing/contracts/proto/cloud-billing.proto` (700 lines). gRPC is the **canonical** inter-microservice communication per ADR-0145 (direct gRPC + 3 invariants — no Workflow + Ontology forced-adapter rule). REST (IP-006) is the outside-facing translation; AsyncAPI (IP-007) is the event channel; gRPC is the synchronous request/response substrate that every Phase-0/1/2 µservice uses to talk to cloud-billing.

The contract carries 11 service definitions, 40+ message types, 8 enums, and Cedar action bindings inline-commented for every state-mutating RPC.

## §B Scope

In scope:

- 11 services: TenantClassApi, BillingAccountApi, MeteringApi, InvoiceApi, ReservationApi, SettlementApi, SubscriptionApi, SeatCountApi, FxLockApi, DunningApi, ExportApi.
- 8 enums: TenantClass, BillingComponent, DeploymentContext, CloudBillingEventKind, InvoiceState, SettlementDirection, SettlementState, SubscriptionState, TaxInvoiceFormat.
- Common message types: Money, BillingPeriod, MeterUnit, AuditChainHeader.
- Per-service request/response pairs (RPCs).
- Cedar action binding (inline comment on every state-mutation RPC).

Out of scope:

- REST translation layer (IP-006).
- Event channels (IP-007).
- Per-language client codegen (tonic + prost for Rust).

## §C Architecture

### §C.1 Why direct gRPC + 3 invariants (ADR-0145)

ADR-0145 retired the "Workflow + Ontology adapter layer" mandate. The 3 invariants that replace the adapter rule:

1. **Tenant scoping** — every state-mutation RPC carries `tenant_id` in its request message; the Cedar evaluator at the service-boundary interceptor matches against principal claims.
2. **Idempotency** — every state-mutation RPC carries an `idempotency_key` field or equivalent (the kernel handles replay).
3. **Audit-chain emission** — every state-mutation RPC returns an `AuditChainHeader` containing the seal hash.

These three invariants are encoded in the proto3 message shapes directly. The contract is self-policing: any RPC missing one of the three is rejected at code-review time.

### §C.2 Service catalog

#### TenantClassApi (cloud-iam → cloud-billing read path)

| RPC | Cedar | Producer | Consumer |
|---|---|---|---|
| GetTenantClass | cap.cloud.billing.read_tenant_class | cloud-iam (at principal issuance) | — |
| ConvertTenantToPaid | cap.cloud.billing.convert_tenant | tenancy / tenant-admin | cloud-iam, finops-portal |
| MutateBillingComponents | cap.cloud.billing.mutate_billing_components | tenant-admin | cloud-iam, finops-portal |

#### BillingAccountApi (account lifecycle)

| RPC | Cedar | Producer | Consumer |
|---|---|---|---|
| CreateBillingAccount | (implicit at tenant creation) | tenancy | finops-portal |
| GetBillingAccount | (read) | any tenant-scoped reader | — |

#### MeteringApi (usage event ingest)

| RPC | Cedar | Producer | Consumer |
|---|---|---|---|
| EmitUsageEvent | cap.cloud.billing.emit_usage_event | Phase-0/1/2 µservice | metering kernel |
| GetUsageEvent | (read) | finops-portal | — |
| ReadMeterAggregate | (read) | finops-portal | — |

#### InvoiceApi (invoice lifecycle)

| RPC | Cedar | Producer | Consumer |
|---|---|---|---|
| IssueInvoice | cap.cloud.billing.issue_invoice | cloud-billing-invoice-worker | audit-chain |
| GetInvoice | (read) | finops-portal | — |
| VoidInvoice | cap.cloud.billing.void_invoice | oyatie-finance-operator | audit-chain |
| IssueCreditMemo | cap.cloud.billing.issue_credit_memo | oyatie-finance-operator | audit-chain |

#### ReservationApi

| RPC | Cedar | Producer | Consumer |
|---|---|---|---|
| PurchaseReservation | cap.cloud.billing.purchase_reservation | tenant-finops-admin | finops-portal |
| ConvertReservation | cap.cloud.billing.convert_reservation | tenant-finops-admin | finops-portal |

#### SettlementApi (revenue_share)

| RPC | Cedar | Producer | Consumer |
|---|---|---|---|
| ComputeSettlement | cap.cloud.billing.compute_settlement | cloud-billing-settlement-worker | finops-portal |
| InitiatePayout | cap.cloud.billing.initiate_payout | cloud-billing-settlement-worker | payments |

#### SubscriptionApi

| RPC | Cedar | Producer | Consumer |
|---|---|---|---|
| CreateSubscription | (implicit at conversion) | tenant-admin | cloud-iam |
| GetSubscription | (read) | finops-portal | — |
| ModifySubscription | cap.cloud.billing.modify_subscription | tenant-admin | cloud-iam |

#### SeatCountApi

| RPC | Cedar | Producer | Consumer |
|---|---|---|---|
| GetSeatCount | cap.cloud.billing.per_seat.read_seat_count | cloud-iam | — |

#### FxLockApi

| RPC | Cedar | Producer | Consumer |
|---|---|---|---|
| GetFxLock | (read) | cloud-billing-settlement-worker | audit-chain |

#### DunningApi

| RPC | Cedar | Producer | Consumer |
|---|---|---|---|
| GetDunningStatus | (read) | cloud-billing-dunning-worker | — |
| RetryDunning | (implicit) | oyatie-finance-operator | payments |

#### ExportApi

| RPC | Cedar | Producer | Consumer |
|---|---|---|---|
| TriggerFocusExport | (read/write) | finops-portal | cloud-storage |
| TriggerErpExport | (read/write) | tenant-admin | external ERP connector |

### §C.3 HTTP/3 + QUIC transport (ADR-0253)

All gRPC services run on HTTP/3 + QUIC (gRPC-over-HTTP/3). Cross-region calls use connection migration for cell-failover scenarios (per ADR-0248 cellular architecture).

### §C.4 AuditChainHeader contract

Every state-mutation response includes `AuditChainHeader audit = N`:

```proto
message AuditChainHeader {
  bytes audit_chain_hash = 1;        // 32 bytes (SHA-256 over Ed25519 seal)
  string event_class = 2;            // lowercase dotted snake-case per ADR-0263
  int64 sealed_at_epoch_seconds = 3;
}
```

The caller can chain audit hashes for cross-µservice transactions. The seal is mandatory; if audit-chain is unreachable, the RPC fails closed (no silent emit).

### §C.5 Closed enums

- `TenantClass`: UNSPECIFIED, DEMO_TRIAL, PAID. UNSPECIFIED at the wire level signals upgrade-required.
- `BillingComponent`: UNSPECIFIED, REVENUE_SHARE, PER_SEAT, PER_USAGE.
- `DeploymentContext`: UNSPECIFIED, OYATIE_PUBLIC_CLOUD, GUEST_ON_AWS, GUEST_ON_OCI, ON_PREM, COLO, OYATIE_AS_CLOUD_PROVIDER.
- `CloudBillingEventKind`: 11 variants (including REVENUE_SHARE, REVENUE_SHARE_REVERSAL, SEAT_COUNT, SUBSCRIPTION beyond domain crate's 6).
- `InvoiceState`: UNSPECIFIED, ISSUED, PAID, OVERDUE, VOID.
- `SettlementDirection`: UNSPECIFIED, OYATIE_PAYS, OYATIE_COLLECTS.
- `SettlementState`: UNSPECIFIED, COMPUTED, PAYOUT_INITIATED, SETTLED.
- `SubscriptionState`: UNSPECIFIED, CREATED, ACTIVE, PAST_DUE, PAUSED, CANCELED.
- `TaxInvoiceFormat`: 8 variants matching IP-003.

### §C.6 Rust-only codegen (ADR feedback_rust_strict_only_no_python)

`option go_package = ""` and `option java_package = ""` are explicitly blank because Rust-strict-only is the directive. Codegen uses `tonic` + `prost`; clients are generated into per-consumer Rust crates (`oya-cloud-billing-grpc-client`).

## §D Lifecycle

### §D.1 Cross-µservice call flow (tenant-admin convert)

1. tenant-admin calls finops-portal "Convert to Paid" button.
2. finops-portal calls `ConvertTenantToPaid` gRPC on cloud-billing.
3. cloud-billing's gRPC interceptor evaluates Cedar `cap.cloud.billing.convert_tenant`.
4. On permit, cloud-billing executes atomic transaction (update tenant_class, set billing_components, emit audit event).
5. Response includes `AuditChainHeader` with seal hash.
6. cloud-iam asynchronously refreshes principal cache from the AsyncAPI event channel (IP-007).

### §D.2 Cross-µservice call flow (Phase-0 µservice emit usage)

1. cloud-storage emits `EmitUsageEvent` gRPC on cloud-billing for an object PUT.
2. cloud-billing's interceptor evaluates Cedar `cap.cloud.billing.emit_usage_event` (group-membership check `phase-0-microservice`).
3. cloud-billing ingests into `CloudBillingLedger::ingest`, which routes through the metering kernel.
4. Response `EmitUsageEventResponse {event_id, idempotent_replay, audit}` carries the seal hash.

### §D.3 Failure modes

- Cedar denial → gRPC `PERMISSION_DENIED` with the failing capability name in `metadata`.
- Idempotency key collision → `ALREADY_EXISTS` with the original event id.
- Kernel invariant violation → `INVALID_ARGUMENT` with `CloudBillingError.message()`.
- audit-chain unreachable → `UNAVAILABLE` (fail closed).

## §E Cedar Policy Bindings

Each state-mutation RPC binds to a named Cedar capability (see §C.2 service catalog). The Cedar action name is inline-commented in `cloud-billing.proto`:

```
service InvoiceApi {
  // Cedar: cap.cloud.billing.issue_invoice
  rpc IssueInvoice(IssueInvoiceRequest) returns (IssueInvoiceResponse);
  rpc GetInvoice(GetInvoiceRequest) returns (GetInvoiceResponse);
  // Cedar: cap.cloud.billing.void_invoice
  rpc VoidInvoice(VoidInvoiceRequest) returns (VoidInvoiceResponse);
  // Cedar: cap.cloud.billing.issue_credit_memo
  rpc IssueCreditMemo(IssueCreditMemoRequest) returns (IssueCreditMemoResponse);
}
```

This is the spec-level binding; the runtime enforcement is in the gRPC interceptor that reads the principal STS claim and calls the Cedar evaluator before invoking the RPC handler.

## §F Evidence

### §F.1 Source files

- `/Users/jasonlee/oyatie/billing/contracts/proto/cloud-billing.proto` (700 lines).
- `/Users/jasonlee/oyatie/contracts/proto/cloud/billing/v1/cloud-billing-event-v1.proto` (event ingest schema, AsyncAPI-referenced).
- `/Users/jasonlee/oyatie/contracts/proto/cloud/billing/v1/cloud-billing-event-v1.meta.yaml` (governance metadata).

### §F.2 Cedar fragment cross-reference

- `/Users/jasonlee/oyatie/billing/policies/cloud-billing.cedar` (master permits).
- `/Users/jasonlee/oyatie/billing/policies/billing-components-gates.cedar`.
- `/Users/jasonlee/oyatie/billing/policies/conversion-gates.cedar`.
- `/Users/jasonlee/oyatie/billing/policies/demo-trial-gates.cedar`.
- `/Users/jasonlee/oyatie/billing/policies/settlement-gates.cedar`.
- `/Users/jasonlee/oyatie/billing/policies/tenant-class-binding.cedar`.

### §F.3 ADR anchors

- ADR-0145 direct gRPC + 3 invariants.
- ADR-0253 HTTP/3 + QUIC default.
- ADR-0243 cedar-as-universal-gate.
- ADR-0263 audit-chain seal hash.
- ADR-0330 tenant_class + billing_components canonical.

## §G Counterpart parity

| Counterpart | Their inter-service protocol | Oyatie equivalent | Delta |
|---|---|---|---|
| Stripe | Internal: undisclosed. External: REST + webhooks | gRPC internal + REST external + AsyncAPI events | Stripe is external-only API surface; oyatie has explicit internal gRPC. |
| Recurly | REST + webhooks | Same as Stripe | Same. |
| Zuora | SOAP + REST + Kafka | gRPC + REST + AsyncAPI | Oyatie is modern proto3 vs Zuora's SOAP legacy. |
| Chargebee | REST + webhooks + GraphQL | gRPC + REST + AsyncAPI | No GraphQL plans (per Rust-strict + complexity-budget directives). |
| AWS Billing | Internal: proprietary RPC. External: REST + EventBridge | gRPC + REST + CloudEvents AsyncAPI | Direct architectural parity. |

## §H Open questions

- Whether to expose a streaming RPC `StreamUsageEvents` for high-volume Phase-2 consumers. Current decision: no — the AsyncAPI channel is the streaming substrate; gRPC unary is the synchronous request/response.
- Whether to add `ListInvoices` paginated RPC. Current decision: defer to IP-006-extension; finops-portal currently reads invoices via `ReadMeterAggregate` rolled-up surface.
