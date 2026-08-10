---
ip_id: IP-011
microservice: cloud-billing
title: Revenue attribution — cost-center model + revenue_share settlement pipeline
wave: Wave-15B-cloud-billing-spec-sprint
date: 2026-05-21
owner: axis-cloud-billing
status: drafted
priority: P0
binding_adrs: [ADR-0242, ADR-0330, ADR-0244, ADR-0263, ADR-0145]
counterpart_parity: [Stripe platform fees, Recurly revenue recognition, AWS Billing Conductor, FinOps Framework]
capabilities_touched:
  - cap.cloud.billing.compute_settlement
  - cap.cloud.billing.initiate_payout
  - cap.cloud.billing.settlement.clawback.record
  - cap.cloud.billing.settlement.affiliate_payout
  - cap.cloud.billing.settlement.beps_export
billing_components: [revenue_share, per_seat, per_usage]
tenant_class_scope: paid
---

# IP-011 — Revenue attribution + revenue_share settlement pipeline

## §A Objective

Document the revenue attribution model per ADR-0242 KS#1 oyatie-is-a-tenant doctrine and ADR-0330 §B.5 settlement engine. cloud-billing is the source-of-truth for **commercial state attribution**: every dollar entering or leaving Oyatie's ledger is attributed to a cost-center via deterministic rules. Cost-centers are the unit of revenue accounting; tenants are the unit of access scope.

The settlement engine drives the revenue_share component — computing commission rates, FX adjustments, clawbacks, and payout initiation for marketplace sellers, affiliates, and resellers.

## §B Scope

In scope:

- Cost-center hierarchy: per-tenant sub-scopes that attribute usage to internal cost-centers.
- Revenue_share settlement state machine: COMPUTED → PAYOUT_INITIATED → SETTLED.
- Settlement direction: `oyatie_pays` (Oyatie owes tenant) vs `oyatie_collects` (tenant owes Oyatie).
- Commission rate computation: contract_rate × gross_amount + FX adjustment − clawback = net_amount.
- Clawback / chargeback: dispute-period bounded reversal of prior revenue events.
- BEPS Pillar Two export: cross-jurisdiction revenue allocation for global minimum tax compliance.
- Affiliate payout (negative revenue_share): Oyatie pays affiliate per referral.

Out of scope:

- Per-jurisdiction tax computation (IP-003).
- Payment-method specifics (payments µservice).
- ERP reconciliation (IP-015).

## §C Architecture

### §C.1 oyatie-is-a-tenant doctrine (ADR-0242)

Oyatie itself is a reserved-namespace tenant: `ten_oyatie_*`. This means:

- Oyatie's own revenue (from paid tenants) is accounted in `ten_oyatie_finance` cost-centers.
- Oyatie's own cost (substrate cost from AWS/OCI/on-prem) is accounted in `ten_oyatie_substrate` cost-centers.
- Oyatie's marketplace platform fees collected from sellers are revenue in `ten_oyatie_marketplace_revenue` and payouts to sellers are cost in `ten_oyatie_marketplace_payouts`.

There is no carve-out — Oyatie's books reconcile through the same cloud-billing pipeline as every other tenant.

### §C.2 Sub-scope cost-center model

Each tenant declares a hierarchical cost-center tree:

```
tenant_id: ten_alpha
├── cost-center: ten_alpha/engineering
│   ├── ten_alpha/engineering/backend
│   ├── ten_alpha/engineering/data
│   └── ten_alpha/engineering/ml-platform
├── cost-center: ten_alpha/sales
│   ├── ten_alpha/sales/north-america
│   └── ten_alpha/sales/emea
└── cost-center: ten_alpha/finance
```

Every CloudBillingEvent carries an optional `cost_center` field (defaulting to `ten_alpha/_default` when absent). Attribution rules in cloud-billing-attribution-engine (separate worker) match resource tags → cost_center. Mismatches surface in finops-portal as "unattributed cost" with reconciliation actions.

### §C.3 Revenue_share settlement state machine

```
                       ┌──────────────┐
                       │  SETTLEMENT_ │
                       │  STATE_      │
                       │  UNSPECIFIED │
                       └──────┬───────┘
                              │ ComputeSettlement
                              ▼
                       ┌──────────────┐
                       │   COMPUTED   │
                       └──────┬───────┘
                              │ InitiatePayout
                              ▼
                       ┌──────────────┐
                       │   PAYOUT_    │
                       │   INITIATED  │
                       └──────┬───────┘
                              │ payment handle resolves
                              ▼
                       ┌──────────────┐
                       │    SETTLED   │
                       └──────────────┘
```

Per proto3 `SettlementState` enum (lines 71–76 of cloud-billing.proto).

### §C.4 Settlement statement composition

`SettlementStatement` (cloud-billing.proto lines 414–430):

```
{
  statement_id, tenant_id, contract_id,
  settlement_window_start_epoch_seconds, settlement_window_end_epoch_seconds,
  direction: oyatie_pays | oyatie_collects,
  gross_amount: Money,
  commission_rate: f64,
  commission_amount: Money,
  fx_adjustment: Money,
  clawback_amount: Money,
  net_amount: Money,
  payout_method_ref: String,
  state: SettlementState,
  audit: AuditChainHeader,
}
```

Computation:

- `commission_amount = gross_amount * commission_rate`.
- `net_amount = gross_amount - commission_amount + fx_adjustment - clawback_amount` (when direction = oyatie_pays).
- `net_amount = commission_amount + fx_adjustment - clawback_amount` (when direction = oyatie_collects).

The arithmetic is deterministic; FX adjustment uses the FX lock (FxLockApi) from the settlement window's reference date.

### §C.5 Clawback / chargeback semantics

Per `settlement-gates.cedar` lines 64–83:

- Clawback is recording a `RevenueShareReversal` event against a prior `RevenueShare` event.
- Permitted only within the contract's dispute period (`context.original_event_age_seconds <= context.dispute_period_seconds`).
- Default dispute period: 90 days (matches Visa/Mastercard chargeback window).
- Permitted principals: payments-psp-webhook-handler group (auto-clawback from PSP) or oyatie-finance-operator (manual).

When a clawback lands, the next monthly settlement nets it against the gross_amount.

### §C.6 Affiliate payout (negative revenue_share)

Per `settlement-gates.cedar` lines 87–98:

- Affiliate contracts carry `contract_direction = "affiliate"` and `direction = "oyatie_pays"`.
- cloud-billing-settlement-worker computes referral commission per contract terms.
- Initiates payout via payments µservice.

This is structurally identical to marketplace seller payout but with different contract terms (referral-rate-per-referral vs platform-take-percentage).

### §C.7 BEPS Pillar Two export

Per `settlement-gates.cedar` lines 102–112:

- For global minimum tax compliance (OECD BEPS Pillar Two = 15% global minimum effective tax).
- Permitted only for tenants with `resource.beps_eligible == true` (i.e. multi-jurisdiction tenants).
- Requires oyatie-finance-operator with reviewer approval (two-person rule).
- Exports settlement data per jurisdiction for the tenant's transfer-pricing model.

### §C.8 Sovereign invoice issuance

Per `settlement-gates.cedar` lines 116–126:

- Sovereign deployments (on-prem, colo, guest-on-oci) with sovereign_pack_active.
- Settlement statements convert to local-jurisdiction invoices (with the deployment context's TaxInvoiceFormat).
- Issued by cloud-billing-invoice-worker.

## §D Lifecycle

### §D.1 Marketplace settlement (monthly)

1. Cloud-billing-settlement-worker scans `RevenueShare` events for the prior calendar month per tenant.
2. Aggregates by `(tenant_id, contract_id)`.
3. Looks up commission rate from contract.
4. Looks up FX lock for the settlement-window reference date.
5. Subtracts any `RevenueShareReversal` events landed within dispute period.
6. Calls Cedar `cap.cloud.billing.compute_settlement` (gate validates `tenant_class = paid` AND `"revenue_share" in billing_components`).
7. Computes `SettlementStatement` and persists with `state = COMPUTED`.
8. Emits `cloud.billing.settlement.computed.v1` event.
9. Audit-chain seal embedded in response.

### §D.2 Payout initiation

1. cloud-billing-settlement-worker filters statements with `state = COMPUTED` and `direction = oyatie_pays`.
2. Calls Cedar `cap.cloud.billing.settlement.payout.permit_worker` (gate validates worker membership).
3. Constructs `InitiatePayoutRequest { statement_id }`.
4. payments µservice handles the actual money movement and returns `payment_handle`.
5. State transitions to `PAYOUT_INITIATED`.
6. Audit-chain seal embedded.

### §D.3 Payout settlement (async confirmation)

1. payments µservice emits `payment.settled` or `payment.failed` events.
2. cloud-billing observes; on `payment.settled`, state transitions to `SETTLED`; on `payment.failed`, state remains `PAYOUT_INITIATED` and a retry workflow kicks in.

### §D.4 Failure modes

- Demo_trial tenant attempts settlement → `cap.cloud.billing.settlement.compute.deny_demo_trial` forbids.
- Paid tenant without `revenue_share` component → `cap.cloud.billing.settlement.compute.deny_paid_without_revshare` forbids.
- Clawback outside dispute period → `cap.cloud.billing.settlement.clawback.dispute_period` forbids.
- audit-chain unreachable → fail closed (per IP-010).

## §E Cedar Policy Bindings

- cap.cloud.billing.compute_settlement (cloud-billing.cedar lines 149–159)
- cap.cloud.billing.initiate_payout (cloud-billing.cedar lines 161–170)
- cap.cloud.billing.settlement.compute.permit (settlement-gates.cedar lines 8–17)
- cap.cloud.billing.settlement.compute.deny_demo_trial (lines 19–26)
- cap.cloud.billing.settlement.compute.deny_paid_without_revshare (lines 28–36)
- cap.cloud.billing.settlement.payout.permit_worker (lines 40–49)
- cap.cloud.billing.settlement.payout.permit_human_with_approval (lines 51–60)
- cap.cloud.billing.settlement.clawback.record (lines 64–74)
- cap.cloud.billing.settlement.clawback.dispute_period (lines 76–83)
- cap.cloud.billing.settlement.affiliate_payout (lines 87–98)
- cap.cloud.billing.settlement.beps_export (lines 102–112)
- cap.cloud.billing.settlement.sovereign_invoice (lines 116–126)

## §F Evidence

### §F.1 Source files

- `/Users/jasonlee/oyatie/billing/contracts/proto/cloud-billing.proto` lines 71–76 (SettlementState enum), 65–69 (SettlementDirection), 408–444 (settlement messages + RPCs), 669–674 (SettlementApi service).
- `/Users/jasonlee/oyatie/billing/policies/settlement-gates.cedar` (126 lines, 9 gates).
- `/Users/jasonlee/oyatie/billing/policies/cloud-billing.cedar` lines 149–195 (settlement + read gates).

### §F.2 ADR anchors

- ADR-0242 oyatie-is-a-tenant (Oyatie's own books in same pipeline).
- ADR-0330 §B.5 settlement engine.
- ADR-0244 tenant scoping (cost-center is a tenant sub-scope).
- ADR-0263 audit-chain seal on every settlement.
- ADR-0145 direct gRPC.

## §G Counterpart parity

| Counterpart | Their concept | Oyatie equivalent | Delta |
|---|---|---|---|
| Stripe | `application_fee_amount` per Charge; payouts to connected_account | revenue_share commission_rate × gross_amount = commission_amount | Stripe deducts at charge time; oyatie aggregates monthly. |
| Stripe | Standard/Express/Custom account types | Single `revenue_share` component on paid tenant_class | Oyatie collapses Stripe's three account types into one component with direction enum. |
| AWS Billing Conductor | Per-linked-account allocation rules with custom rates | Cost-center hierarchy + attribution-engine | AWS Conductor is for chargeback within an org; oyatie's attribution is the same shape per tenant. |
| FinOps Framework | "Showback" + "Chargeback" cost allocation patterns | finops-portal reads cost-center attribution; cloud-billing enforces the binding | Direct parity. |
| Recurly revenue recognition | ASC 606 module computing recognized revenue per period | Recognition computed at FOCUS / ERP export (IP-015) | Oyatie separates recognition from settlement; Recurly bundles. |
| Chargebee | Refunds + write-offs as separate concepts | Clawback (RevenueShareReversal) covers both | Oyatie collapses into one event type with explicit reason. |
| OECD BEPS Pillar Two | 15% global minimum effective tax with per-jurisdiction reconciliation | `cap.cloud.billing.settlement.beps_export` + per-jurisdiction allocation | Oyatie supports natively via cost-center attribution. |

## §H Open questions

- Whether to expose the cost-center hierarchy via a separate `CostCenterApi` service. Current decision: defer — cost-center binding is done via resource tags + attribution rules; explicit API can be added in IP-011-extension.
- Whether clawback should auto-trigger re-issuance of original invoice (credit memo) or stand alone as a settlement-level adjustment. Current decision: stand alone in settlement; original invoice immutable; credit memo issued only for tenant-direct refund.
