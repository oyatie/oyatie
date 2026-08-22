---
ip_id: IP-012
microservice: cloud-billing
title: Tenant onboarding billing flow — demo_trial signup → paid conversion
wave: Wave-15B-cloud-billing-spec-sprint
date: 2026-05-21
owner: axis-cloud-billing
status: drafted
priority: P0
binding_adrs: [ADR-0330, ADR-0329, ADR-0244, ADR-0243, ADR-0263, ADR-0255]
counterpart_parity: [Stripe checkout signup, AWS account signup, Recurly subscription create, Vercel signup-to-pro]
capabilities_touched:
  - cap.cloud.billing.convert_tenant
  - cap.cloud.billing.mutate_billing_components
  - cap.cloud.billing.conversion.permit
billing_components: [revenue_share, per_seat, per_usage]
tenant_class_scope: both
---

# IP-012 — Tenant onboarding billing flow

## §A Objective

Document the end-to-end onboarding pipeline that gets a brand-new prospect from "first signup" to "paid tenant generating monthly invoice." This is the runtime composition of IP-001 (BillingAccount creation), IP-005 (demo_trial caps), IP-004 (billing_components composition), IP-006/008 (REST/gRPC surfaces), and IP-010 (audit-chain seal) — but composed as a user-facing journey rather than a per-IP feature.

The journey has 5 phases: Phase-0 prospect signup → Phase-1 demo_trial active → Phase-2 cap-aware → Phase-3 conversion → Phase-4 paid lifecycle. Each phase has explicit handoffs to sibling µservices.

## §B Scope

In scope:

- Phase-0 signup flow (interaction with tenancy µservice).
- Phase-1 demo_trial steady-state (interaction with cloud-iam, observability, finops-portal).
- Phase-2 cap-aware transitions (soft-breach, hard-breach, grace, suspension).
- Phase-3 conversion transaction (atomicity, audit-chain seal, principal cache refresh).
- Phase-4 paid lifecycle (per-component activation, invoice cadence, settlement, dunning).
- Cross-µservice handoffs.

Out of scope:

- Tenancy µservice internals (tenant id generation, namespace reservation).
- payments µservice integration (payment method capture).
- crm contract-management surface.

## §C Architecture

### §C.1 Phase-0 prospect signup

1. Prospect lands on signup page (rendered by `product-app-marketing`).
2. Submits email + organization name.
3. tenancy µservice creates tenant: `tenant_id = ten_<ULID>` (or `demo_<ULID>` for demo_trial reserved-namespace per ADR-0244 §C-4).
4. tenancy emits `tenant.created` event.
5. cloud-billing observes event; creates BillingAccount:

```
BillingAccountCreate {
  id: "ba_<tenant_short_id>",
  tenant_id: <tenant_id>,
  region: <prospect_region>,
  regional_pack: "pack-electronic-tax", // default — refined per locale
  payment_method: "pm_demo_trial_placeholder",
  credit_balance: Money { currency: "OYC", minor_units: 0 },
  state: BillingAccountState::Active,
  data_class: DataClass::Financial,
  created_at_epoch_seconds: now(),
}
```

6. cloud-billing emits `cloud.billing.account.created.v1`.
7. cloud-iam issues principal STS token: `{tenant_id, tenant_class: demo_trial, billing_components: [], cap_breached: false}`.

### §C.2 Phase-1 demo_trial steady-state

- Prospect uses the product freely.
- Usage events flow: Phase-0/1/2 µservices emit `EmitUsageEvent` for every consumption.
- cloud-billing's `CloudBillingLedger::ingest` records into per-tenant ledger.
- Rate-card lookup returns `unit_price_micros = 0` for the OCI Always Free workload class (per IP-005 §C.3).
- Cap-watcher tracks cumulative usage per axis.
- finops-portal shows real-time consumption with cap proximity indicators.

### §C.3 Phase-2 cap-aware transitions

#### §C.3.1 Soft breach (80%)

- cap-watcher emits `cloud.billing.cap.soft_breach.detected.v1`.
- observability-notification µservice notifies tenant-admin via email + in-product banner.
- No Cedar deny.

#### §C.3.2 Hard breach (100% + grace start)

- cap-watcher emits `cloud.billing.cap.breach.detected.v1`.
- cloud-billing updates tenant state to `cap_breached = true`.
- cloud-iam invalidates principal cache; next token refresh carries `cap_breached: true`.
- Cedar `cap.cloud.billing.deny_demo_trial_writes_after_cap_breach` denies all writes.
- Reads remain via `cap.cloud.billing.demo_trial.permit_read_during_grace`.
- 7-day grace timer starts.

#### §C.3.3 Suspension (after grace)

- cloud-billing-suspend-worker transitions tenant to `suspended` state.
- 30-day retention window starts.
- Cedar `cap.cloud.billing.suspended.deny_all_writes` denies all writes.
- Reads remain via `cap.cloud.billing.suspended.permit_read_during_retention`.

#### §C.3.4 Deletion (after retention)

- tenancy µservice triggers DSR cascade (IP-013).
- All tenant data deleted; audit-chain entries retained per regulatory retention.

### §C.4 Phase-3 conversion transaction

Conversion is atomic across cloud-billing + crm + tenancy + cloud-iam. The transaction:

1. crm presents contract terms to tenant-admin via finops-portal upgrade button.
2. tenant-admin accepts; crm produces `contract_id`.
3. tenant-admin (or oyatie-finance-operator on behalf) calls `ConvertTenantToPaid` gRPC:

```
ConvertTenantToPaidRequest {
  tenant_id: <ten_*>,
  contract_id: <crm_contract_id>,
  billing_components: [PER_SEAT, PER_USAGE], // initial set
  retain_trial_usage: false,                   // contract default
  deployment_context: OYATIE_PUBLIC_CLOUD,
}
```

4. Cedar `cap.cloud.billing.conversion.permit` evaluates:
   - principal.tenant_id == resource.tenant_id ✓
   - resource.tenant_class == "demo_trial" ✓
   - context.target_tenant_class == "paid" ✓
   - context.contract_id != "" ✓
   - context.initial_billing_components_subset_of canonical ✓
   - principal.has_role("tenant-admin") ✓
5. cloud-billing transactionally:
   - Updates tenant_class to "paid".
   - Sets billing_components to {per_seat, per_usage}.
   - Resets cap_breached to false (cap state irrelevant for paid).
   - Stamps contract_id.
6. Emits `cloud.billing.tenant_class.converted.v1` event.
7. audit-chain seals the conversion event; seal hash embedded in response.
8. cloud-iam observes event; invalidates principal cache.
9. Next principal STS token refresh (within 30 seconds per ADR-0255 §D-3): `{tenant_id, tenant_class: paid, billing_components: [per_seat, per_usage], cap_breached: false}`.

Atomicity: any failure rolls back via `cap.cloud.billing.conversion.deny_partial_state`.

### §C.5 Phase-4 paid lifecycle

#### §C.5.1 First monthly invoice

- Monthly close at period_end_epoch_seconds.
- cloud-billing-invoice-worker aggregates usage events per axis.
- For per_seat: cloud-billing-seat-counter snapshots active_seat_count.
- For per_usage: cloud-billing-aggregator rolls up per-meter consumption.
- Tax engine (cloud-billing-tax µservice, IP-003) resolves tax_profile_ref per line item.
- `IssueInvoice` gRPC produces immutable invoice + audit-chain seal.
- Invoice PDF rendered by cloud-billing-pdf-renderer; storage handle in `Invoice.pdf_object_ref`.
- payments µservice charges payment_method.

#### §C.5.2 Component mutation mid-life

- Tenant signs contract amendment adding `revenue_share` (e.g. tenant becomes a marketplace seller).
- tenant-admin calls `MutateBillingComponents { op: ADD, component: REVENUE_SHARE, contract_amendment_id }`.
- Cedar `cap.cloud.billing.mutate_billing_components` evaluates and permits.
- billing_components updated to {per_seat, per_usage, revenue_share}.
- cloud-marketplace observes; tenant can now publish marketplace listings.
- audit-chain seal.

#### §C.5.3 Dunning flow on payment failure

- payments µservice returns `payment.failed` for monthly invoice.
- cloud-billing-dunning-worker initiates retry schedule (RetryDunning RPC).
- Per dunning policy: retry on days 3, 7, 14, 21, 28.
- If still unpaid after 30 days: tenant_class remains paid but `tenant.state` transitions to `overdue` (different from suspended).
- After 60 days: optional suspension per contract terms.

#### §C.5.4 Subscription modifications

- tenant-admin calls `ModifySubscription` for plan changes / pauses / cancellations.
- Cedar `cap.cloud.billing.modify_subscription` evaluates.
- Proration computed per Subscription.proration_behavior.
- audit-chain seal.

## §D Lifecycle

The five-phase journey is summarized as a state machine:

```
[New prospect]
     │ tenancy.create_tenant
     ▼
[demo_trial_active] ◄─────────┐
     │                         │
     ├─ usage events           │
     ├─ soft breach (80%)      │
     │                         │
     ▼                         │
[cap_breached_grace] (7d)      │
     │                         │
     ├─ convert → ─────────────┼─→ [paid_active]
     │                         │        │
     ▼                         │        ├─ invoice issued
[suspended_retention] (30d)    │        ├─ subscription modified
     │                         │        ├─ component mutated
     ├─ convert → ─────────────┘        ├─ settlement computed
     │                                  ├─ payout initiated
     ▼                                  ├─ dunning attempts
[deleted]                               │
                                        ▼
                                  [paid_active] (steady-state)
                                        │
                                        ├─ subscription canceled → [canceled]
                                        ├─ payment failures → [overdue] → [suspended]
                                        └─ contract end → [final_invoice → settled]
```

## §E Cedar Policy Bindings (full journey)

| Phase | Capabilities evaluated |
|---|---|
| Phase-0 signup | (implicit at tenancy µservice; cloud-billing creates account passively) |
| Phase-1 demo_trial | cap.cloud.billing.demo_trial.permit_within_caps, cap.cloud.billing.emit_usage_event |
| Phase-2 soft breach | (no Cedar; notification only) |
| Phase-2 hard breach | cap.cloud.billing.deny_demo_trial_writes_after_cap_breach, cap.cloud.billing.demo_trial.deny_write_during_grace |
| Phase-2 read during grace | cap.cloud.billing.demo_trial.permit_read_during_grace |
| Phase-2 suspension | cap.cloud.billing.suspended.deny_all_writes, cap.cloud.billing.suspended.permit_read_during_retention |
| Phase-3 conversion | cap.cloud.billing.conversion.permit, cap.cloud.billing.conversion.require_audit_chain_seal, plus 6 deny rules |
| Phase-3 conversion during grace | cap.cloud.billing.conversion.permit_during_grace |
| Phase-3 conversion during suspension | cap.cloud.billing.conversion.permit_during_suspension |
| Phase-4 first invoice | cap.cloud.billing.issue_invoice |
| Phase-4 component mutation | cap.cloud.billing.mutate_billing_components |
| Phase-4 subscription modify | cap.cloud.billing.modify_subscription |
| Phase-4 settlement | cap.cloud.billing.compute_settlement, cap.cloud.billing.initiate_payout |

## §F Evidence

### §F.1 Source files

- All six Cedar fragments under `microservices/cloud-billing/policies/`.
- proto3 ConvertTenantToPaid + MutateBillingComponents + IssueInvoice RPCs.
- `cloud-billing-domain::BillingAccount::new` (lines 530–551).
- IaC `microservices/cloud-billing/iac/oci-guest/always-free/` (demo_trial substrate).

### §F.2 ADR anchors

- ADR-0330 §B.10.4 conversion transaction.
- ADR-0329 binary tenant_class.
- ADR-0244 tenant scoping.
- ADR-0243 cedar-gated mutations.
- ADR-0263 audit-chain seal.
- ADR-0255 §D-3 30-second principal cache refresh.

## §G Counterpart parity

| Counterpart | Their onboarding | Oyatie equivalent | Delta |
|---|---|---|---|
| Stripe checkout | Card-based signup with optional trial period | Email-first signup → demo_trial → paid contract | Stripe couples signup with payment method; oyatie defers payment-method capture to conversion. |
| AWS account signup | Email + credit-card-on-file required at signup | Email-only at signup; no payment method until conversion | Oyatie is friendlier for evaluation; matches Vercel/Netlify "Hobby" pattern. |
| Recurly subscription create | `POST /sites/{id}/subscriptions` with Account + Plan + PaymentMethod | `ConvertTenantToPaid` with contract + billing_components | Recurly bundles signup + plan; oyatie separates demo_trial from plan. |
| Vercel "Hobby → Pro" upgrade | Upgrade button → enter payment → instant upgrade | Convert button → contract signing → ConvertTenantToPaid → 30s token refresh | Direct parity in UX; oyatie's Cedar audit-chain is stronger. |
| Linear "Free → Plus" upgrade | Workspace owner → billing page → plan change | Similar but tenant_id is org-rooted | Direct parity. |
| Notion "Free → Plus" upgrade | Workspace settings → upgrade plan | Same pattern | Direct parity. |
| Heroku "Hobby → Production" | Add credit card → upgrade dyno → instant | Direct parity | Heroku's "Eco" dynos sleep; oyatie's OCI Always Free doesn't. |
| Stripe onboarding | Hosted onboarding flow for connected accounts (Know Your Customer) | Marketplace seller onboarding = demo_trial → paid + revenue_share component | Same conceptual flow; KYC delegated to cloud-iam + crm. |

## §H Open questions

- Whether to support "instant conversion" via card-on-file at signup (skip demo_trial). Current decision: no — demo_trial is the canonical trial path; instant paid signup is supported by the same Convert RPC with zero-day-trial.
- Whether to support "tenant merge" — multiple demo_trial accounts merge into one paid account at conversion. Current decision: defer; complex consent flow; revisit if customer-acquisition needs it.
- Whether to expose a "convert preview" RPC that returns what the first invoice would look like before commitment. Current decision: yes — `PreviewSubscription` planned for IP-012-extension.
