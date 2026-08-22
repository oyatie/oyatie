---
ip_id: IP-005
microservice: cloud-billing
title: demo_trial tenant_class — zero-cost path with caps, grace, conversion
wave: Wave-15B-cloud-billing-spec-sprint
date: 2026-05-21
owner: axis-cloud-billing
status: drafted
priority: P0
binding_adrs: [ADR-0330, ADR-0329, ADR-0244, ADR-0243, ADR-0251, ADR-0316_RETIRED]
counterpart_parity: [Stripe Billing trials, AWS Free Tier, Recurly free trials, Vercel Hobby tier]
capabilities_touched:
  - cap.cloud.billing.read_tenant_class
  - cap.cloud.billing.convert_tenant
  - cap.cloud.billing.demo_trial.deny_compliance_pack_activation
  - cap.cloud.billing.demo_trial.deny_byok_opt_in
  - cap.cloud.billing.demo_trial.deny_marketplace_listing
billing_components: []
tenant_class_scope: demo_trial
---

# IP-005 — demo_trial tenant_class: caps, grace, conversion

## §A Objective

Document the demo_trial tenant_class behavior per ADR-0330 §B.3. demo_trial is the canonical zero-cost trial path: a tenant gets the **full** product surface (no degraded mode, no feature lockout, no second-class UX) bounded by usage caps, time caps, and a closed set of Cedar-denied actions (compliance pack activation, BYOK opt-in, marketplace listing publishing).


## §B Scope

In scope:

- demo_trial caps: usage caps (per-axis), time caps (trial expiry), spending caps (always $0 since no billing).
- Grace window semantics: 7-day write-deny after cap breach, with conversion path remaining open.
- Cedar-denied actions for demo_trial: compliance pack activation, BYOK, marketplace listing (publish + non-free purchase).
- Conversion path demo_trial → paid (one-way, atomic).
- Suspension state interaction.

Out of scope:

- Paid tenant_class semantics (covered by IP-004 composable billing_components).
- Compliance pack catalog (covered by `compliance-packs` µservice).
- BYOK provider catalog (covered by `cloud-iam` BYOK module).

## §C Architecture

### §C.1 Trial fairness rule (Outcome 1 from PRD)

Per PRD outcome 1: "A demo_trial tenant gets the full product surface, full quality bar, full performance budget, and full architectural posture." This rule is binding on every µservice; cloud-billing enforces the no-feature-lockout rule structurally:

- No demo_trial-only Cedar permits (only the three denies above).
- No demo_trial-only rate cards (free-tier is `unit_price_micros = 0` rather than a separate code path).
- No demo_trial-only SLO targets (the same availability targets apply).

This rule is CI-enforced by `governance-paid-quality-bar-parity` (per ADR-0330 §B.3.5).

### §C.2 Cap structure

| Cap axis | Default cap for demo_trial | Refresh cadence | Cedar enforcement |
|---|---|---|---|
| Compute cpu-hour | 100 vCPU-hour/month | Monthly reset | `cap.cloud.billing.demo_trial.deny_write_during_grace` after breach |
| GPU gpu-hour | 4 H100-hour/month (Tier-A1 ARM equivalent on OCI Always Free) | Monthly reset | Same |
| Storage gib-storage-month | 100 GiB-month | Monthly accumulation | Same |
| Egress gib-egress | 100 GiB/month | Monthly reset | Same |
| API requests | 1M/month | Monthly reset | Same |
| LLM tokens | 1M tokens (input+output combined)/month | Monthly reset | Same |
| Time cap (trial duration) | 30 days from tenant creation | One-shot | Conversion CTA on day 21 / 28 / 30 |
| Tenant count per organization | 1 per organization domain | One-shot | tenancy µservice enforces |

Cap values are configurable per `microservices/tenancy/configs/demo-trial-caps.json` (not in this IP's scope); the rules of enforcement are encoded here.

### §C.3 OCI Always Free maximization

Per `feedback_oci_always_free_maximization_2026_05_20`: demo_trial deployments use OCI Always Free tier as the cost-floor. The OCI Always Free resources (2× Ampere A1 ARM 4 OCPU + 24GB, 2× Autonomous DB, 200GB block, 10GB obj, 10TB egress, Vault, LB, Streaming) are sufficient for a typical demo_trial workload. cloud-billing emits zero-cost CloudBillingEvents during demo_trial (rate-card `unit_price_micros = 0` for the always-free workload class).

The IaC module `microservices/cloud-billing/iac/oci-guest/always-free/` provisions the OCI Always Free substrate for demo_trial tenants. Conversion to paid replaces the always-free rate-cards with paid rate-cards via cloud-iam-issued role rotation.

### §C.4 Grace window (7-day write-deny)

When a cap is breached:

1. Cloud-billing's cap-watcher emits `cloud.billing.cap.breach.detected` event.
2. Tenant state is marked `cap_breached = true` on principal claims.
3. Cedar `cap.cloud.billing.deny_demo_trial_writes_after_cap_breach` (tenant-class-binding.cedar lines 38–52) and `cap.cloud.billing.demo_trial.deny_write_during_grace` (demo-trial-gates.cedar lines 110–123) deny writes.
4. Reads remain permitted via `cap.cloud.billing.demo_trial.permit_read_during_grace`.
5. Conversion to paid is permitted via `cap.cloud.billing.demo_trial.permit_conversion_during_grace`.
6. After 7 days, tenant state transitions to `suspended` (cloud-billing-suspend-worker emits `tenant.state.suspended`).
7. Suspended tenants get 30-day retention window for data export + conversion before deletion.

### §C.5 Conversion path (one-way)

`conversion-gates.cedar` (142 lines) enforces:

- Source must be `demo_trial` (cannot convert paid → demo_trial; `cap.cloud.billing.conversion.deny_downgrade`).
- Target must be `paid` (cannot self-convert demo_trial → demo_trial or paid → paid; `cap.cloud.billing.conversion.deny_demo_to_demo`, `cap.cloud.billing.conversion.deny_paid_to_paid_via_convert`).
- Contract id required (no implicit conversion; `cap.cloud.billing.conversion.deny_missing_contract`).
- Initial billing_components must be a valid subset (must declare at least one of revenue_share/per_seat/per_usage).
- Atomicity invariant: transaction is committed or rolled back (`cap.cloud.billing.conversion.deny_partial_state`).
- Audit-chain seal required (`cap.cloud.billing.conversion.require_audit_chain_seal`).

Conversion is permitted during cap-breach grace and during suspension retention window. This gives the tenant a low-friction path to recover from a cap breach by signing the paid contract.

### §C.6 Denied capabilities for demo_trial

Per ADR-0330 §B.3.6 / §B.3.7 / §B.3.8 the demo_trial tenant class explicitly cannot:

- Activate any compliance pack (`cap.cloud.billing.demo_trial.deny_compliance_pack_activation`). Compliance pack activation is a paid-tenant-only operation because compliance packs impose substrate cost (signed audit chain rotation, key-rotation cadence, retention period extension) that the demo_trial cost model cannot absorb.
- Configure BYOK for LLM / payment / KMS / identity providers (`cap.cloud.billing.demo_trial.deny_byok_opt_in`). BYOK requires per-tenant key management, key-rotation infra, and break-glass procedures whose substrate cost is paid-tier.
- Publish marketplace listings (`cap.cloud.billing.demo_trial.deny_marketplace_listing`). Marketplace seller status requires KYC, tax registration, and revenue_share contract — all paid-tier features.

Demo_trial tenants **may** purchase free marketplace listings (`cap.cloud.billing.demo_trial.permit_free_listing_consumption`); the deny is on the publish side.

## §D Lifecycle

### §D.1 Demo trial signup

1. Free-tier signup flow creates tenant with `tenant_class = demo_trial`.
2. tenancy µservice emits `tenant.created` event.
3. cloud-billing creates a BillingAccount with `regional_pack = pack-electronic-tax` (default) and `payment_method = pm_demo_trial` placeholder.
4. cloud-iam emits principal tokens with `tenant_class = demo_trial`, `billing_components = []`, `cap_breached = false`.

### §D.2 Soft cap (80% breach)

1. cloud-billing-cap-watcher detects 80% of any cap.
2. `cloud.billing.cap.soft_breach.detected` event fires.
3. observability-notification µservice notifies tenant-admin.
4. No Cedar deny yet.

### §D.3 Hard cap (100% breach + grace start)

1. cloud-billing-cap-watcher detects 100% of any cap.
2. `cloud.billing.cap.breach.detected` event fires.
3. Principal claim `cap_breached = true` propagated by cloud-iam.
4. 7-day grace window begins.
5. Cedar denies writes; reads remain.

### §D.4 Trial expiry (30-day)

1. cloud-billing emits `cloud.billing.trial.expiring` at day 21, 28, 30.
2. If no conversion by day 30, tenant transitions to `suspended` with 30-day retention.
3. After 60 days total (30-day trial + 30-day retention), tenant deletion is triggered by tenancy µservice DSR cascade (IP-013).

### §D.5 Conversion

1. Tenant signs paid contract via crm.
2. tenant-admin submits `ConvertTenantToPaidRequest`.
3. Cedar `cap.cloud.billing.conversion.permit` evaluates.
4. On permit, cloud-billing atomically:
   - Updates `tenant_class` to `paid`.
   - Sets `billing_components` from `context.initial_billing_components_subset_of(...)`.
   - Records contract_id.
   - Emits `tenant.class.converted_to_paid` to audit-chain.
   - Triggers cloud-iam principal cache invalidation.
5. Principal tokens refresh within 30 seconds.

## §E Cedar Policy Bindings

Master conversion gates (in `conversion-gates.cedar`):

- `cap.cloud.billing.conversion.permit`
- `cap.cloud.billing.conversion.deny_downgrade`
- `cap.cloud.billing.conversion.deny_demo_to_demo`
- `cap.cloud.billing.conversion.deny_paid_to_paid_via_convert`
- `cap.cloud.billing.conversion.deny_unknown_target_class`
- `cap.cloud.billing.conversion.deny_missing_contract`
- `cap.cloud.billing.conversion.permit_during_grace`
- `cap.cloud.billing.conversion.permit_during_suspension`
- `cap.cloud.billing.conversion.deny_partial_state`
- `cap.cloud.billing.conversion.require_audit_chain_seal`

Demo-trial gates (in `demo-trial-gates.cedar`):

- `cap.cloud.billing.demo_trial.deny_compliance_pack_activation`
- `cap.cloud.billing.paid.permit_compliance_pack_activation`
- `cap.cloud.billing.demo_trial.deny_byok_opt_in`
- `cap.cloud.billing.paid.permit_byok_opt_in`
- `cap.cloud.billing.demo_trial.deny_marketplace_listing`
- `cap.cloud.billing.demo_trial.deny_paid_listing_purchase`
- `cap.cloud.billing.demo_trial.permit_free_listing_consumption`
- `cap.cloud.billing.demo_trial.permit_within_caps`
- `cap.cloud.billing.demo_trial.deny_write_during_grace`
- `cap.cloud.billing.demo_trial.permit_read_during_grace`
- `cap.cloud.billing.demo_trial.permit_conversion_during_grace`
- `cap.cloud.billing.suspended.deny_all_writes`
- `cap.cloud.billing.suspended.permit_read_during_retention`

## §F Evidence

### §F.1 Source files

- `/Users/jasonlee/oyatie/microservices/cloud-billing/policies/demo-trial-gates.cedar` (174 lines).
- `/Users/jasonlee/oyatie/microservices/cloud-billing/policies/conversion-gates.cedar` (142 lines).
- `/Users/jasonlee/oyatie/microservices/cloud-billing/policies/tenant-class-binding.cedar` (88 lines).
- `/Users/jasonlee/oyatie/microservices/cloud-billing/contracts/proto/cloud-billing.proto` lines 143–156 (ConvertTenantToPaidRequest/Response), 127–141 (GetTenantClassResponse with `cap_breached`, `trial_expires_at_epoch_seconds`, `grace_window_expires_at_epoch_seconds`).
- `/Users/jasonlee/oyatie/microservices/cloud-billing/iac/oci-guest/always-free/` (IaC for demo_trial OCI deployment).

### §F.2 ADR anchors

- ADR-0330 §B.3 (demo_trial doctrine).
- ADR-0329 (tenant-class binary enum).
- ADR-0251 (compliance packs — paid only).
- ADR-0255 §D-4 (BYOK opt-in — paid only).
- ADR-0249 (marketplace — paid only publish; free purchase OK).
- ADR-0316 (RETIRED — replaced tier model).

## §G Counterpart parity

| Counterpart | Their concept | Oyatie equivalent | Delta |
|---|---|---|---|
| Stripe Billing | Free trial with `trial_period_days` on Subscription | demo_trial tenant_class with 30-day time cap | Stripe's trial is subscription-scoped; oyatie's is tenant-scoped. |
| Stripe Billing | `pause_payment_collection` mid-trial | Cap-breach grace state with read-only access | Behavioral parity. |
| AWS Free Tier | Per-service free quotas (e.g. 750h EC2 t2.micro, 5GB S3) | Per-axis demo_trial caps (compute / storage / egress / requests / tokens) | AWS quotas are per-service; oyatie's are per-axis (cleaner cross-µservice budget tracking). |
| AWS Free Tier | "Always Free" tier (12-month limited free + indefinite always-free) | OCI Always Free as substrate floor for demo_trial; 30-day product-level trial | Oyatie maps to OCI's Always Free substrate. |
| Vercel | "Hobby" plan vs "Pro" plan with feature lockouts (no SSO, no logs retention beyond 1 day, etc.) | demo_trial has caps but **no feature lockouts** — PRD outcome 1 | Oyatie is more honest: no degraded mode, just hard caps. |
| Recurly | Account states: active, paused, expired, canceled | tenant_states: active, cap_breached, suspended, retention | Direct parity. |
| Heroku | "Eco" dyno → free-tier-equivalent that sleeps after 30 min idle | OCI Always Free Ampere A1 — does not sleep but is OCI-tier scaled | Different substrate strategy; same outcome. |

## §H Open questions

- Whether to expose a "demo_trial extension" — a 30-day grace extension granted by Customer Success on request. Current decision: no — conversion is the canonical extension path; this enforces revenue discipline. Revisit if Sales reports lost conversions due to inflexible cap.
- Whether `cap_breached` should reset on cap-axis-reset (monthly) or persist until tenant-admin acknowledgment. Current decision: monthly auto-reset; cap_breached is computed against the current month's cap usage.
