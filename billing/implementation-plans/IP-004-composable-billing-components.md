---
ip_id: IP-004
microservice: cloud-billing
title: Composable billing_components — revenue_share + per_seat + per_usage subset
wave: Wave-15B-cloud-billing-spec-sprint
date: 2026-05-21
owner: axis-cloud-billing
status: drafted
priority: P0
binding_adrs: [ADR-0330, ADR-0329, ADR-0244, ADR-0243, ADR-0131]
counterpart_parity: [Stripe Billing, Recurly, Zuora Billing, Chargebee]
capabilities_touched:
  - cap.cloud.billing.mutate_billing_components
  - cap.cloud.billing.rev_share.settle
  - cap.cloud.billing.per_seat.read_seat_count
  - cap.cloud.billing.per_usage.set_soft_cap
billing_components: [revenue_share, per_seat, per_usage]
tenant_class_scope: paid
---

# IP-004 — Composable billing_components: revenue_share + per_seat + per_usage

## §A Objective

Document the composable `billing_components` semantics per ADR-0330 §B.11. A paid tenant carries an immutable `tenant_class = paid` enum value plus a mutable subset of `billing_components ⊆ {revenue_share, per_seat, per_usage}`. Any non-empty subset is valid: a single component, any pair, or all three. The components compose orthogonally — adding a component does not remove existing line-item streams; removing a component freezes its line-item streams without affecting others.


## §B Scope

In scope:

- The three canonical components: `revenue_share`, `per_seat`, `per_usage`.
- Composability rules: any non-empty subset; addition by contract amendment; removal by contract amendment with grace window.
- Per-component event kinds in `CloudBillingEventKind` proto3 (RevenueShare, RevenueShareReversal, SeatCount, Subscription).
- Per-component Cedar gates in `billing-components-gates.cedar` (156 lines).
- Cross-component invoice composition: a single monthly invoice may include line items from all three components.

Out of scope:

- demo_trial caps (covered by IP-005).
- Settlement computation for revenue_share (covered by IP-011 and `settlement-gates.cedar`).
- Per-seat enforcement at authentication time (gated by cloud-iam consuming `principal.billing_components`).

## §C Architecture

### §C.1 The closed-subset rule

`billing_components` is a Cedar set whose membership is strictly drawn from the three canonical strings. The closure is enforced by:

- proto3 `BillingComponent` enum (REVENUE_SHARE = 1, PER_SEAT = 2, PER_USAGE = 3) — wire-level closed.
- Cedar `cap.cloud.billing.deny_unknown_billing_component` (tenant-class-binding.cedar lines 79–88) — policy-level closed.
- Domain kernel: `BillingAccount` carries no explicit billing_components field; cloud-billing's source-of-truth is a separate `tenant_billing_components` table whose schema is `{tenant_id, component_string, contract_amendment_id, added_at_epoch_seconds, removed_at_epoch_seconds_nullable}` (table schema future IP).

### §C.2 Component 1 — revenue_share

`revenue_share` means "Oyatie takes a contractual percentage of the tenant's gross revenue and either remits the net to the tenant (when oyatie collected) or invoices the tenant for the platform share (when tenant collected)."

Typical contracts:

- Marketplace seller — tenant sells via Oyatie marketplace; Oyatie collects from buyer; remits seller-share to tenant; takes platform commission.
- Affiliate — tenant refers customers; Oyatie collects; pays referral commission to tenant.
- Reseller — tenant resells Oyatie SaaS to its own customers; Oyatie collects from tenant gross; remits net to tenant; takes platform share.

Event kinds:

- `RevenueShare` — positive revenue event (Oyatie owes tenant).
- `RevenueShareReversal` — clawback / chargeback (Oyatie reverses prior revenue event).

Settlement direction:

- `oyatie_pays` — Oyatie owes tenant (marketplace seller, affiliate).
- `oyatie_collects` — Tenant owes Oyatie (reseller platform share).

Cedar gate: `cap.cloud.billing.rev_share.settle`, `cap.cloud.billing.rev_share.publish_marketplace_listing`. Both require `"revenue_share" in resource.billing_components`.

### §C.3 Component 2 — per_seat

`per_seat` means "Oyatie invoices the tenant a recurring per-named-user license fee."

Seat-count snapshot semantics:

- Seat-count is captured at the period boundary by `cloud-billing-seat-counter` worker.
- Active seat is determined by cloud-iam principal authentication history.
- Over-seat detection compares `active_seat_count` to `seat_count_ceiling`.
- Grace window: 24h post-overage before write-deny via `cap.cloud.billing.per_seat.deny_over_ceiling`.

Event kind: `SeatCount` (records active/over-seat snapshot per period).

Cedar gates: `cap.cloud.billing.per_seat.read_seat_count`, `cap.cloud.billing.per_seat.add_seat`, `cap.cloud.billing.per_seat.deny_over_ceiling`.

### §C.4 Component 3 — per_usage

`per_usage` means "Oyatie invoices the tenant for metered consumption."

Usage axis:

- Compute (cpu-hour, gpu-hour, pod-minute).
- Storage (gib-storage-month).
- Network (gib-egress).
- API surface (request, token).

Soft cap vs hard cap (per Cedar `billing-components-gates.cedar`):

- Soft cap: alert at 80% (notification via observability-notification µservice).
- Hard cap: write-deny at 100% via `cap.cloud.billing.per_usage.deny_above_hard_cap`.

Event kinds: `Usage`, `ResourceCreated`, `ResourceTerminated`, `Reservation`, `Commitment`, `Credit`.

### §C.5 Composition examples

| Tenant archetype | tenant_class | billing_components |
|---|---|---|
| Free trial signup | demo_trial | ∅ (caps in lieu of billing) |
| SaaS enterprise customer (named users, no usage) | paid | {per_seat} |
| PAYG/usage-only customer (no seats) | paid | {per_usage} |
| Hybrid SaaS (named seats + overage usage) | paid | {per_seat, per_usage} |
| Marketplace seller | paid | {per_seat, revenue_share} |
| Complex reseller (sells under their brand, pays platform share) | paid | {revenue_share, per_seat, per_usage} |
| Affiliate referrer | paid | {revenue_share} |

### §C.6 Mutation transaction

Adding or removing a component is a Cedar-gated single transaction:

1. Tenant-admin (or oyatie-finance-operator) submits `MutateBillingComponentsRequest {tenant_id, op: ADD|REMOVE, component, contract_amendment_id}`.
2. Cedar evaluates `cap.cloud.billing.mutate_billing_components`:
   - `principal.tenant_id == resource.tenant_id`
   - `resource.tenant_class == "paid"`
   - `context.contract_amendment_id != ""`
   - `principal.has_role("tenant-admin") || principal.has_role("oyatie-finance-operator")`
3. cloud-billing applies the mutation, emits audit-chain event (per ADR-0263), refreshes principal cache via cloud-iam invalidation, and returns the new component set + audit hash.
4. Principal tokens refresh within 30 seconds (ADR-0255 §D-3 short-cycle token rotation).

Removal generates a settlement-event for any in-flight line items so no per_usage event lands without an active component to absorb it.

## §D Lifecycle

### §D.1 Initial component assignment at conversion

When demo_trial → paid conversion happens, the contract names the initial component subset (per `conversion-gates.cedar` line 26–30: `context.initial_billing_components_subset_of(["revenue_share", "per_seat", "per_usage"])`).

### §D.2 Adding a component mid-life

Tenant signs contract amendment → tenant-admin submits MutateBillingComponentsRequest with `op = ADD` → cloud-billing emits `tenant.billing_components.added` event → cloud-iam refreshes principal claims.

### §D.3 Removing a component mid-life

Tenant signs contract amendment with effective date → tenant-admin submits MutateBillingComponentsRequest with `op = REMOVE` → cloud-billing computes outstanding line items, generates final invoice for the removed component, emits `tenant.billing_components.removed` event → cloud-iam refreshes claims.

### §D.4 Failure modes

- Demo_trial tenant attempts mutation → `cap.cloud.billing.deny_demo_trial_billing_components_mutation` forbids (always denies for demo_trial).
- Missing contract_amendment_id → `cap.cloud.billing.mutate_billing_components` permit fails (does not match).
- Unknown component string → `cap.cloud.billing.deny_unknown_billing_component` forbids.
- Cap-breach grace state → `cap.cloud.billing.deny_demo_trial_writes_after_cap_breach` forbids mutation.

## §E Cedar Policy Bindings

Master permits (in `cloud-billing.cedar`):

- `cap.cloud.billing.mutate_billing_components`
- `cap.cloud.billing.deny_demo_trial_billing_components_mutation`

Per-component gates (in `billing-components-gates.cedar`):

- `cap.cloud.billing.rev_share.settle`
- `cap.cloud.billing.rev_share.deny_without_component`
- `cap.cloud.billing.rev_share.publish_marketplace_listing`
- `cap.cloud.billing.rev_share.deny_demo_trial_marketplace`
- `cap.cloud.billing.per_seat.read_seat_count`
- `cap.cloud.billing.per_seat.deny_without_component`
- `cap.cloud.billing.per_seat.add_seat`
- `cap.cloud.billing.per_seat.deny_over_ceiling`
- `cap.cloud.billing.per_usage.set_soft_cap`
- `cap.cloud.billing.per_usage.set_hard_cap`
- `cap.cloud.billing.per_usage.deny_above_hard_cap`
- `cap.cloud.billing.per_usage.read_meter_aggregate`

Attribute schema (per `tenant-class-binding.cedar`):

- `principal.billing_components: Set<String>` subset of canonical 3.
- `resource.billing_components: Set<String>` snapshot at audited operation.

## §F Evidence

### §F.1 Source files

- `/Users/jasonlee/oyatie/billing/contracts/proto/cloud-billing.proto` lines 26–31 (BillingComponent enum), 158–175 (MutateBillingComponentsRequest/Response), 631–638 (TenantClassApi service).
- `/Users/jasonlee/oyatie/billing/policies/billing-components-gates.cedar` (156 lines, 12 named gates).
- `/Users/jasonlee/oyatie/billing/policies/tenant-class-binding.cedar` lines 78–88 (closure rule).
- `/Users/jasonlee/oyatie/billing/policies/cloud-billing.cedar` lines 56–76 (mutate / deny-demo gates).

### §F.2 ADR anchors

- ADR-0330 §B.11: canonical composability rule.
- ADR-0329: tenant-class doctrine (cloud-billing = source-of-truth).
- ADR-0244: tenant scoping on every resource.
- ADR-0243: cedar-as-universal-gate.
- ADR-0316 (retired): replaced tier model.

## §G Counterpart parity

| Counterpart | Their concept | Oyatie equivalent | Delta |
|---|---|---|---|
| Stripe Billing | `Subscription` with `SubscriptionItem[]` each carrying a Price | `Subscription` with billing_components on the tenant; per-component line items composed at invoice time | Stripe attaches metering at SubscriptionItem level; oyatie attaches at tenant level with composability — single source of truth for "which streams does this tenant generate?" |
| Stripe Billing | "Pricing" model: flat / per_seat / metered / volume / graduated / package | per_seat + per_usage closed subset; pricing curve carried in rate-card | Oyatie's component set is smaller; complex pricing curves live in the rate card, not the component model. |
| Stripe | Platform takes application_fee from connected accounts | `revenue_share` component on a tenant; SettlementApi computes commission | Direct parity. Stripe = oyatie's revenue_share. |
| Recurly | `RatePlan` with `RatePlanCharge[]` (recurring / usage / one-time) | `billing_components` (per_seat = recurring; per_usage = usage; revenue_share = platform-fee) | Recurly attaches everything to a plan; oyatie keeps plan opaque and surfaces the three commercial models orthogonally. |
| Zuora Billing | `Charge.Model ∈ {FlatFee, PerUnit, Volume, Tiered, Overage, ...}` | Pricing curve lives in rate-card (opaque ref); billing_components govern the commercial relationship | Zuora exposes the pricing-model variety; oyatie keeps it inside the rate card. |
| Chargebee | Plans, addons, metered components, in-app subscriptions | per_seat (plan + addons) + per_usage (metered components) + revenue_share (platform fee) | Same 3-axis composition with different vocabulary. |
| AWS Marketplace | "Listing Type": Bring-Your-Own-License, SaaS Subscription, SaaS Contract, Free Trial, Private Offer | Pack overlays (free-tier = demo_trial; SaaS Contract = paid+per_seat; SaaS Subscription = paid+per_usage; Free Trial = demo_trial; Private Offer = paid+custom) | AWS encodes the listing type; oyatie encodes the commercial model on the tenant. |

## §H Open questions

- Whether to add a fourth component `flat_fee` for "one-time / annual contract value with no metering or seats." Current decision: model flat fees as a per-period per_usage rate-card with single-charge cardinality; revisit if downstream FinOps needs distinct event class.
- Whether revenue_share should split into `marketplace_revshare` and `affiliate_revshare`. Current decision: keep one component; settlement contract carries the direction (`oyatie_pays` / `oyatie_collects`) and the contract_direction (`marketplace` / `affiliate` / `reseller`).
