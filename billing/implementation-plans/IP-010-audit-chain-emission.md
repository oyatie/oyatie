---
ip_id: IP-010
microservice: cloud-billing
title: Audit-chain emission — Ed25519 + Merkle anchor per billing transaction
wave: Wave-15B-cloud-billing-spec-sprint
date: 2026-05-21
owner: axis-cloud-billing
status: drafted
priority: P0
binding_adrs: [ADR-0263, ADR-0244, ADR-0243, ADR-0145, ADR-0252]
counterpart_parity: [Stripe Sigma audit trail, Recurly audit history, Zuora audit log, SOX-404 evidence chains]
capabilities_touched:
  - cap.cloud.billing.issue_invoice
  - cap.cloud.billing.convert_tenant
  - cap.cloud.billing.compute_settlement
  - cap.cloud.billing.initiate_payout
  - cap.cloud.billing.void_invoice
  - cap.cloud.billing.issue_credit_memo
billing_components: [per_seat, per_usage, revenue_share]
tenant_class_scope: both
---

# IP-010 — Audit-chain seal emission per billing transaction

## §A Objective

Document the cloud-billing audit-chain emission contract per ADR-0263. Every state-mutating operation in cloud-billing seals an event into the immutable audit-chain via the `AuditChainHeader` field on every gRPC mutation response. The seal is Ed25519-signed and anchored into a per-tenant Merkle tree whose root is published to a hash-chained external anchor (a public ledger or per-tenant transparency log).

The audit-chain is the canonical evidence substrate for SOX-404 segregation-of-duties, ASC 606 revenue recognition, K-FSI billing audit, MAS-TRM logbook, PCI-DSS retention, GDPR data-flow accuracy, and EU-AI-Act billing-decisions-impact log.

## §B Scope

In scope:

- `AuditChainHeader` proto message: `{audit_chain_hash, event_class, sealed_at_epoch_seconds}`.
- Per-operation seal events (one per mutating RPC).
- Cross-µservice seal chaining (cloud-billing → audit-chain → external anchor).
- HLC default + TrueTime opt-in for fin-grade timestamping (ADR-0252).
- Fail-closed semantics when audit-chain is unreachable.

Out of scope:

- audit-chain µservice internals (Merkle tree construction, Ed25519 key rotation, transparency-log anchor selection — owned by audit-chain).
- Long-term retention storage (cloud-storage µservice).

## §C Architecture

### §C.1 Seal-on-mutation rule

Every state-mutating RPC in `cloud-billing.proto` returns a response containing `AuditChainHeader audit = N`. The contract is:

- ConvertTenantToPaidResponse.audit
- MutateBillingComponentsResponse.audit
- CreateBillingAccountResponse.audit
- EmitUsageEventResponse.audit
- IssueInvoiceResponse.invoice.audit (nested on Invoice)
- VoidInvoiceResponse.audit
- IssueCreditMemoResponse.audit
- PurchaseReservationResponse.reservation.audit (nested on Reservation)
- ConvertReservationResponse.audit
- ComputeSettlementResponse.statement.audit (nested on SettlementStatement)
- InitiatePayoutResponse.audit
- CreateSubscriptionResponse.subscription.audit (nested on Subscription)
- ModifySubscriptionResponse.audit
- GetFxLockResponse.audit (FX lock is itself an evidence event)
- RetryDunningResponse.audit

A response missing `AuditChainHeader` would fail the proto3 compile-time `governance-audit-chain-completeness` lint (per ADR-0263 §C-2 lint registration).

### §C.2 AuditChainHeader fields

```
message AuditChainHeader {
  bytes audit_chain_hash = 1;        // 32 bytes: SHA-256 over (event_id || event_class || tenant_id || resource_id || sealed_at || Ed25519_signature)
  string event_class = 2;            // lowercase dotted snake-case per ADR-0263: e.g. "cloud.billing.invoice.issued"
  int64 sealed_at_epoch_seconds = 3; // HLC tick at seal time
}
```

The hash is computed by audit-chain (not cloud-billing). Cloud-billing constructs the event, sends to audit-chain via gRPC `Seal` RPC, and embeds the returned hash in the response.

### §C.3 Fail-closed semantics

When audit-chain is unreachable, the mutation RPC fails with gRPC `UNAVAILABLE` and a metadata key `audit_chain_unreachable: true`. No mutation completes without a seal. This is the core SOX-404 invariant: no money state changes without evidence.

The single exception is `EmitUsageEvent` during a documented audit-chain outage window — usage events are buffered in a local at-least-once write-ahead log (`cloud-billing-uel-buffer`) and replayed when audit-chain returns. This is gated by ADR-0263 §D-5 outage-tolerance clause.

### §C.4 HLC default, TrueTime opt-in

Per ADR-0252:

- HLC (Hybrid Logical Clock) is the default for `sealed_at_epoch_seconds`. HLC provides causal ordering across cells without atomic-clock cost.
- TrueTime is opt-in per tenant for fin-grade scenarios (K-FSI, MAS-TRM, BCBS-239). TrueTime provides bounded uncertainty (~7ms) suitable for cross-jurisdiction tax-timing precision.

The choice is per-tenant; the `AuditChainHeader.sealed_at_epoch_seconds` field carries the timestamp regardless of source clock — provenance is in the parallel `clock_provenance` field of the underlying audit-chain entry (not exposed in `AuditChainHeader` directly to keep the proto small).

### §C.5 Event class taxonomy (ADR-0263 §C-1)

cloud-billing's event classes (lowercase dotted snake-case, per ADR-0263):

- cloud.billing.invoice.issued
- cloud.billing.invoice.voided
- cloud.billing.credit_memo.issued
- cloud.billing.tenant_class.converted
- cloud.billing.billing_components.added
- cloud.billing.billing_components.removed
- cloud.billing.usage_event.ingested
- cloud.billing.reservation.purchased
- cloud.billing.reservation.converted
- cloud.billing.settlement.computed
- cloud.billing.payout.initiated
- cloud.billing.subscription.created
- cloud.billing.subscription.modified
- cloud.billing.subscription.canceled
- cloud.billing.cap.breach.detected
- cloud.billing.trial.expired
- cloud.billing.fx_lock.recorded
- cloud.billing.focus_export.completed
- cloud.billing.erp_export.completed

### §C.6 Cross-µservice seal chaining

When cloud-billing performs an operation that transitively involves another µservice (e.g. convert_tenant cascades to cloud-iam principal refresh), each µservice emits its own seal. The seals are chained via `previous_seal_hash` in the underlying audit-chain entry (out of the AuditChainHeader proto for compactness). A single conversion produces ~4 chained seals: cloud-billing tenant_class.converted → cloud-iam principal.cache.invalidated → tenancy tenant.class.updated → finops-portal billing.context.refreshed.

This is the SOX-404 segregation-of-duties evidence: each µservice has its own signing key; cross-µservice operations leave a multi-party chain.

## §D Lifecycle

### §D.1 Seal emission flow (issue_invoice)

1. cloud-billing-invoice-worker calls `IssueInvoice` gRPC.
2. Cedar `cap.cloud.billing.issue_invoice` evaluates and permits.
3. `cloud-billing-domain::CloudBillingLedger::generate_invoice` executes.
4. cloud-billing constructs `AuditEvent { event_class: "cloud.billing.invoice.issued", tenant_id, resource_id: invoice.id, payload: invoice (proto-encoded), occurred_at: HLC }`.
5. cloud-billing calls audit-chain gRPC `Seal(AuditEvent)`.
6. audit-chain signs with tenant-specific Ed25519 key, appends to Merkle tree, returns `{audit_chain_hash, sealed_at_epoch_seconds}`.
7. cloud-billing embeds in `IssueInvoiceResponse.invoice.audit`.
8. Response returned to caller.

### §D.2 Outage-tolerance for usage events

1. cloud-storage calls `EmitUsageEvent`.
2. cloud-billing attempts audit-chain Seal; fails with UNAVAILABLE.
3. cloud-billing writes to local `cloud-billing-uel-buffer` (per-cell append-only WAL).
4. cloud-billing returns success with `AuditChainHeader { audit_chain_hash: zeros, event_class: "cloud.billing.usage_event.ingested.buffered" }`.
5. Cap-watcher observes UEL buffer depth; if > threshold, paging fires.
6. Background reconciliation worker drains UEL when audit-chain returns; replaces buffered seal-hash placeholder with actual seal.

This narrow exception preserves availability for high-volume usage emission while keeping the invariant for slower, money-impactful operations (invoice / settlement / convert).

### §D.3 Failure modes

- audit-chain unreachable for mutating RPC (non-usage): RPC fails with UNAVAILABLE; caller retries.
- Seal hash mismatch (audit-chain returns different hash on idempotent retry): cloud-billing fails the response; operator investigation.
- Clock skew between cloud-billing HLC and audit-chain HLC: HLC tolerates up to 250ms skew; beyond that, sealed_at falls back to caller's HLC.
- Ed25519 key compromise: audit-chain rotates keys per tenant; old keys remain in transparency log for replay verification.

## §E Cedar Policy Bindings

Audit-chain emission is not Cedar-gated at the cloud-billing side (it's a downstream consequence of permit'd mutations). audit-chain has its own Cedar gates for read access (out of scope here).

`cap.cloud.billing.conversion.require_audit_chain_seal` (conversion-gates.cedar lines 134–142) is the meta-gate that fails-closed when audit-chain is unreachable for convert_tenant operations — the only Cedar gate that directly references audit-chain availability.

## §F Evidence

### §F.1 Source files

- `/Users/jasonlee/oyatie/microservices/cloud-billing/contracts/proto/cloud-billing.proto` lines 117–121 (AuditChainHeader message) + 12 RPC responses embedding it.
- `/Users/jasonlee/oyatie/microservices/cloud-billing/policies/conversion-gates.cedar` lines 134–142 (audit-chain availability gate).

### §F.2 ADR anchors

- ADR-0263 audit-chain seal-hash binding (master).
- ADR-0252 HLC default + TrueTime opt-in.
- ADR-0244 tenant scoping (per-tenant Ed25519 key).
- ADR-0145 direct gRPC + 3 invariants (3rd invariant = audit-chain emission).

## §G Counterpart parity

| Counterpart | Their audit model | Oyatie equivalent | Delta |
|---|---|---|---|
| Stripe Sigma | SQL-queryable event store with append-only semantics | audit-chain immutable Merkle log + queryable via audit-chain query API | Oyatie's chain is cryptographically anchored; Stripe Sigma is database-level append-only without external anchor. |
| Recurly audit history | Per-account event log with admin-readable changes | Per-tenant audit-chain entries | Same scope; oyatie is signed. |
| Zuora audit log | "Action log" per object with user attribution | audit-chain with principal_id + seal hash | Direct parity. |
| Chargebee event timeline | Per-customer activity timeline | Per-tenant audit-chain query | Same UX, different substrate. |
| SOX-404 evidence | Standard requirement: who, what, when, why, evidence chain | event_class + tenant_id + principal_id + sealed_at + audit_chain_hash + Merkle proof | Oyatie meets SOX-404 evidence requirements at-substrate-level rather than via reporting bolt-on. |
| AWS CloudTrail | Per-region trail of API calls with optional Lake aggregation | audit-chain per-tenant + per-µservice chain | Architectural parity; oyatie's chain is cryptographically stronger. |

## §H Open questions

- Whether to expose `AuditChainHeader.merkle_proof_uri` for tenants that want to verify proofs themselves. Current decision: defer to audit-chain; cloud-billing only embeds the seal hash.
- Whether buffered usage events should emit a placeholder seal hash or zero bytes. Current decision: zero bytes + event_class suffix `.buffered`; the reconciliation worker replaces with actual seal once audit-chain returns.
