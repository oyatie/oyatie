---
ip_id: IP-009
microservice: cloud-billing
title: Cedar policy fragments — six-file authorization model
wave: Wave-15B-cloud-billing-spec-sprint
date: 2026-05-21
owner: axis-cloud-billing
status: drafted
priority: P0
binding_adrs: [ADR-0243, ADR-0244, ADR-0330, ADR-0263, ADR-0131]
counterpart_parity: [Stripe platform policies, Recurly user roles, AWS IAM, OpenFGA]
capabilities_touched: all cap.cloud.billing.* gates
billing_components: [per_seat, per_usage, revenue_share]
tenant_class_scope: both
---

# IP-009 — Cedar policy fragments: six-file authorization model

## §A Objective

Document the six-file Cedar policy bundle that authorizes every cloud-billing state mutation and read. Per ADR-0243 cedar-as-universal-gate, **every** state-mutation surface in cloud-billing is gated by a named Cedar capability — there is no inline `if` guard, no per-service authorization shortcut.

The six files total 881 lines across 70+ named gates and cover: master permits, billing_component-conditional gates, demo_trial deny semantics, conversion atomicity, revenue_share settlement, and the principal/resource/context attribute schema.

## §B Scope

In scope:

- The six Cedar files:
  - `cloud-billing.cedar` (195 lines, 14 gates) — master permits + cross-cutting denies.
  - `billing-components-gates.cedar` (156 lines, 12 gates) — per-component conditional gates.
  - `conversion-gates.cedar` (142 lines, 9 gates) — demo_trial → paid atomicity.
  - `demo-trial-gates.cedar` (174 lines, 13 gates) — demo_trial deny rules + suspension.
  - `settlement-gates.cedar` (126 lines, 9 gates) — revenue_share settlement + clawback + sovereign.
  - `tenant-class-binding.cedar` (88 lines, 4 gates + attribute schema) — closure rules + cross-tenant defense.
- Principal / resource / context attribute schemas.
- Gate naming convention `cap.cloud.billing.<entity>.<verb>` or `cap.cloud.billing.<component>.<verb>`.
- Deny-precedence: any `forbid` policy overrides any `permit` per Cedar default semantics.

Out of scope:

- Cedar evaluator runtime (lives in `oya-cedar-eval-kernel`).
- Per-gate test fixtures (covered in IP-009-test-fixtures, future).
- cloud-iam principal claim binding (covered by cloud-iam µservice).

## §C Architecture

### §C.1 Six-file layout rationale

The six-file split is by concern, not by entity:

- **Master permits** (`cloud-billing.cedar`): the "any tenant-admin can issue invoice" type rules — broad, named-action permits.
- **Component conditionals** (`billing-components-gates.cedar`): "only if `per_seat` is active" type rules.
- **Conversion atomicity** (`conversion-gates.cedar`): "demo_trial → paid one-way, atomic" type rules.
- **Demo-trial denies** (`demo-trial-gates.cedar`): "demo_trial cannot do X" type rules.
- **Settlement specifics** (`settlement-gates.cedar`): revenue_share + sovereign-invoice rules that don't fit the per-component pattern.
- **Attribute schema** (`tenant-class-binding.cedar`): principal/resource/context shape contract.

This layout means a reader auditing "what gates apply during demo_trial conversion" can find them in `conversion-gates.cedar` + `demo-trial-gates.cedar` without grepping all six.

### §C.2 Attribute schema (per `tenant-class-binding.cedar`)

```
principal.tenant_id : EntityUid             // ten_* or demo_* prefix
principal.tenant_class : String              // closed enum "demo_trial" | "paid"
principal.billing_components : Set<String>   // closed subset of canonical 3
principal.cap_breached : Bool                // transient demo_trial state
principal.roles : Set<String>                // tenant-admin, oyatie-finance-operator, etc.
principal.byok_modes : Record<String, String>
principal.compliance_packs : Set<String>

resource.tenant_id : EntityUid
resource.tenant_class : String               // snapshot at audited operation (ADR-0330 §B.12.2)
resource.billing_components : Set<String>

context.contract_id : String                 // required for conversion
context.contract_amendment_id : String       // required for billing_components mutation
context.has_reviewer_approval : Bool         // required for void / credit_memo / payout
context.target_tenant_class : String         // for downgrade-attempt detection
```

### §C.3 Closure rules (anti-drift)

- `cap.cloud.billing.deny_unknown_tenant_class` forbids `mutate_tenant_class` unless `context.target_tenant_class` is one of the two canonical values.
- `cap.cloud.billing.deny_unknown_billing_component` forbids `mutate_billing_components` unless `context.component` is one of the three canonical values.
- `cap.cloud.billing.deny_cross_tenant_access` forbids any action where `principal.tenant_id != resource.tenant_id` (except oyatie-internal-operator escape).

### §C.4 Two-person-rule gates

Three actions require `context.has_reviewer_approval == true`:

- `cap.cloud.billing.void_invoice` — voiding an issued invoice.
- `cap.cloud.billing.issue_credit_memo` — issuing a credit memo.
- `cap.cloud.billing.initiate_payout` (human path; worker path bypasses).

The reviewer is enforced to be a different principal via `context.reviewer_principal_id != principal.principal_id` (settlement-gates.cedar line 60).

### §C.5 Sovereign / on-prem / colo overlays

`cap.cloud.billing.settlement.sovereign_invoice` (settlement-gates.cedar lines 116–126) permits sovereign-invoice issuance only when:

- `resource.tenant_class == "paid"`
- `resource.deployment_context in ["on-prem", "colo", "guest-on-oci"]`
- `resource.sovereign_pack_active == true`
- principal is in `cloud-billing-invoice-worker` group

This gate runs alongside the standard `cap.cloud.billing.issue_invoice`. Sovereign invoices need both gates to pass.

### §C.6 Cap-breach / suspension / grace gates

- `cap.cloud.billing.deny_demo_trial_writes_after_cap_breach` denies emit_usage_event / issue_invoice / purchase_reservation / create_subscription / mutate_billing_components for cap_breached demo_trial principals.
- `cap.cloud.billing.demo_trial.permit_read_during_grace` permits reads during grace.
- `cap.cloud.billing.suspended.deny_all_writes` denies all writes for suspended tenants.
- `cap.cloud.billing.suspended.permit_read_during_retention` permits reads during retention window.
- `cap.cloud.billing.demo_trial.permit_conversion_during_grace` keeps conversion path open during grace.

These five gates encode the demo_trial lifecycle state machine at the policy layer.

## §D Lifecycle

### §D.1 Gate evaluation flow

1. gRPC interceptor extracts STS principal claims (from JWT signed by cloud-iam).
2. Interceptor constructs Cedar `Request { principal, action, resource, context }`.
3. Cedar evaluator runs all six files (deny precedence applied).
4. On permit, handler proceeds; on deny, gRPC `PERMISSION_DENIED` with capability name.

### §D.2 Adding a new gate (process)

1. Identify the entity/action/component intersection.
2. Pick the right file by concern.
3. Add gate with `@id("cap.cloud.billing.<entity>.<verb>")`.
4. Add fixture test in `microservices/cloud-billing/policies/_tests/`.
5. Update the proto3 `// Cedar:` inline comment on the corresponding RPC.

### §D.3 Failure modes

- Gate missing for a new RPC: gRPC interceptor falls back to default-deny.
- Gate names colliding: `@id` uniqueness is checked at Cedar policy-bundle compile time.
- Attribute schema drift: tenant-class-binding.cedar is the source-of-truth; any other file referencing an attribute not declared here fails compile.

## §E Cedar Policy Bindings (full enumeration)

### §E.1 cloud-billing.cedar (master permits + denies)

- cap.cloud.billing.read_tenant_class
- cap.cloud.billing.convert_tenant
- cap.cloud.billing.deny_paid_downgrade
- cap.cloud.billing.mutate_billing_components
- cap.cloud.billing.deny_demo_trial_billing_components_mutation
- cap.cloud.billing.emit_usage_event
- cap.cloud.billing.issue_invoice
- cap.cloud.billing.void_invoice
- cap.cloud.billing.issue_credit_memo
- cap.cloud.billing.purchase_reservation
- cap.cloud.billing.convert_reservation
- cap.cloud.billing.compute_settlement
- cap.cloud.billing.initiate_payout
- cap.cloud.billing.read_invoice
- cap.cloud.billing.read_settlement_statement

### §E.2 billing-components-gates.cedar (per-component)

- cap.cloud.billing.rev_share.settle
- cap.cloud.billing.rev_share.deny_without_component
- cap.cloud.billing.rev_share.publish_marketplace_listing
- cap.cloud.billing.rev_share.deny_demo_trial_marketplace
- cap.cloud.billing.per_seat.read_seat_count
- cap.cloud.billing.per_seat.deny_without_component
- cap.cloud.billing.per_seat.add_seat
- cap.cloud.billing.per_seat.deny_over_ceiling
- cap.cloud.billing.per_usage.set_soft_cap
- cap.cloud.billing.per_usage.set_hard_cap
- cap.cloud.billing.per_usage.deny_above_hard_cap
- cap.cloud.billing.per_usage.read_meter_aggregate

### §E.3 conversion-gates.cedar (conversion atomicity)

- cap.cloud.billing.conversion.permit
- cap.cloud.billing.conversion.deny_downgrade
- cap.cloud.billing.conversion.deny_demo_to_demo
- cap.cloud.billing.conversion.deny_paid_to_paid_via_convert
- cap.cloud.billing.conversion.deny_unknown_target_class
- cap.cloud.billing.conversion.deny_missing_contract
- cap.cloud.billing.conversion.permit_during_grace
- cap.cloud.billing.conversion.permit_during_suspension
- cap.cloud.billing.conversion.deny_partial_state
- cap.cloud.billing.conversion.require_audit_chain_seal

### §E.4 demo-trial-gates.cedar (demo_trial behavior)

- cap.cloud.billing.demo_trial.deny_compliance_pack_activation
- cap.cloud.billing.paid.permit_compliance_pack_activation
- cap.cloud.billing.demo_trial.deny_byok_opt_in
- cap.cloud.billing.paid.permit_byok_opt_in
- cap.cloud.billing.demo_trial.deny_marketplace_listing
- cap.cloud.billing.demo_trial.deny_paid_listing_purchase
- cap.cloud.billing.demo_trial.permit_free_listing_consumption
- cap.cloud.billing.demo_trial.permit_within_caps
- cap.cloud.billing.demo_trial.deny_write_during_grace
- cap.cloud.billing.demo_trial.permit_read_during_grace
- cap.cloud.billing.demo_trial.permit_conversion_during_grace
- cap.cloud.billing.suspended.deny_all_writes
- cap.cloud.billing.suspended.permit_read_during_retention

### §E.5 settlement-gates.cedar (revenue_share settlement)

- cap.cloud.billing.settlement.compute.permit
- cap.cloud.billing.settlement.compute.deny_demo_trial
- cap.cloud.billing.settlement.compute.deny_paid_without_revshare
- cap.cloud.billing.settlement.payout.permit_worker
- cap.cloud.billing.settlement.payout.permit_human_with_approval
- cap.cloud.billing.settlement.clawback.record
- cap.cloud.billing.settlement.clawback.dispute_period
- cap.cloud.billing.settlement.affiliate_payout
- cap.cloud.billing.settlement.beps_export
- cap.cloud.billing.settlement.sovereign_invoice

### §E.6 tenant-class-binding.cedar (closures + cross-cutting)

- cap.cloud.billing.deny_demo_trial_writes_after_cap_breach
- cap.cloud.billing.deny_cross_tenant_access
- cap.cloud.billing.deny_unknown_tenant_class
- cap.cloud.billing.deny_unknown_billing_component

## §F Evidence

### §F.1 Source files (all six present and substantive)

- `/Users/jasonlee/oyatie/microservices/cloud-billing/policies/cloud-billing.cedar` (195 lines).
- `/Users/jasonlee/oyatie/microservices/cloud-billing/policies/billing-components-gates.cedar` (156 lines).
- `/Users/jasonlee/oyatie/microservices/cloud-billing/policies/conversion-gates.cedar` (142 lines).
- `/Users/jasonlee/oyatie/microservices/cloud-billing/policies/demo-trial-gates.cedar` (174 lines).
- `/Users/jasonlee/oyatie/microservices/cloud-billing/policies/settlement-gates.cedar` (126 lines).
- `/Users/jasonlee/oyatie/microservices/cloud-billing/policies/tenant-class-binding.cedar` (88 lines).

### §F.2 ADR anchors

- ADR-0243 cedar-as-universal-gate.
- ADR-0244 tenant scoping primitive.
- ADR-0330 §B.11 tenant_class + billing_components canonical.
- ADR-0263 audit-chain seal hash binding.

## §G Counterpart parity

| Counterpart | Their policy model | Oyatie equivalent | Delta |
|---|---|---|---|
| Stripe | Platform policy: charge_type ∈ {direct, destination, separate}; per-connected-account permissions | Per-tenant role-based Cedar gates + billing_components attribute checks | Stripe's rules are a special-case of revenue_share; oyatie's model handles general N-component composability. |
| Recurly | User roles: admin, billing, support, reporting | tenant-admin, tenant-finance-reader, tenant-finops-admin, oyatie-finance-operator | Same role pattern; Cedar adds attribute-based conditions. |
| AWS IAM | JSON IAM policies on resources with `Condition` clauses | Cedar policies on entities with `when {}` clauses | Cedar is more expressive than IAM (subset/membership operators); same authorization model. |
| OpenFGA | Relationship-based authorization graphs (Zanzibar-style) | Cedar entity hierarchies + attribute-based conditions | Cedar covers both attribute-based (ABAC) and relationship-based (ReBAC) at policy layer. |
| Stripe permissions API | Per-API-key scope tags ("read", "write") | Per-action Cedar gates with named capabilities | Oyatie has finer-grained gates (70+) vs Stripe's broad scopes. |

## §H Open questions

- Whether to add a `cap.cloud.billing.read_tax_registration_id` gate as separate from `read_invoice`. Current decision: tax_registration_id is FINANCIAL_REGULATED_CREDIT and inherits invoice's gate; revisit if PCI-DSS Level 1 scope minimization requires.
- Whether settlement gates should split into `commission_compute` vs `clawback_record` files. Current decision: keep one file because the workers cross-call within the same crate.
