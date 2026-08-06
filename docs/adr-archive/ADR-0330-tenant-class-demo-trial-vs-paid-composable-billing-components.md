---
id: ADR-0330
title: Tenant Class — demo_trial vs paid with Composable Billing Components
status: Superseded
planning_impact: true
date: 2026-05-21
owner_team: council-architecture
related:
  - ADR-0329
  - ADR-0331
  - ADR-0244
  - ADR-0243
  - ADR-0251
  - ADR-0316
  - ADR-0255
  - ADR-0255-amendment
  - ADR-0249
  - ADR-0328
  - ADR-0064
  - ADR-0108
  - ADR-0216
  - ADR-0218
supersedes: []
amends:
  - ADR-0316-capability-tier-over-product-fragmentation.md
superseded_by: [ADR-702]
related_specs:
  - /specs/tenant-model.json
  - /specs/master-plan-sequencing.json
  - /specs/microservices/manifest-schema.json
  - /specs/billing/billing-component-schema.json
  - /specs/cedar-fragment-schema.json
companion_docs:
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0329-tier-system-retirement.md
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0331-per-microservice-tenant-class-adoption.md
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0243-cedar-as-universal-gate.md
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0251-compliance-pack-cell-certification-levels.md
inbound_citations: []
decision_owner: council-architecture
co_owners:
  - council-product
  - council-engineering
  - council-finance
  - council-privacy
  - ops-compliance
  - ops-sre-reliability
  - axis-tenancy
  - axis-policy-engine
  - axis-audit-chain
doc_class: Architecture-Decision-Record
shape: Decision
authority_tier: 1
line_floor: 800
bespoke_authoring_requirement: documentation-rigor-1.1-plus-ADR-0322
enforcement_status: Accepted; CI lane `ci-tenant-class-adoption-check` becomes BLOCKER once per-µservice adoption ADR-0331 lands
enforced_by:
  - oya-governance-tenant-class-enum-closed
  - oya-governance-billing-components-subset-closed
  - oya-governance-cloud-billing-source-of-truth
  - oya-governance-iam-principal-tenant-class-claim
  - oya-governance-cedar-tenant-class-attribute-coverage
  - oya-governance-audit-chain-tenant-class-transition
  - oya-governance-demo-trial-cap-enforcement
  - oya-governance-paid-quality-bar-parity
source_anchors:
  - /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md
  - /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_capability_tiers_2026_05_20.md
  - /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_oci_always_free_maximization_2026_05_20.md
  - /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_quality_performance_scalability_bar.md
  - /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_flat_product_catalog.md
purpose: >
  Codify the canonical tenant-class model that replaces the retired
  Bronze/Silver/Gold/Platinum capability-tier system. There are exactly two
  tenant classes — demo_trial and paid — and the paid class carries a
  composable billing_components set drawn from {revenue_share, per_seat,
  per_usage}. The quality bar, capability surface, and architectural posture
  are uniform across both tenant classes. cloud-billing is the source of
  truth for tenant_class and billing_components; cloud-iam emits them as
  principal claims; Cedar policies gate behavior; audit-chain records every
  transition.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0330: Tenant Class — demo_trial vs paid with Composable Billing Components

## Status

Accepted on 2026-05-21.

This ADR is the replacement model for the retired ADR-0316 capability-tier
doctrine. It is the foundational decision behind ADR-0329 (tier system
retirement) and ADR-0331 (per-microservice tenant-class adoption). It
encodes the binary tenant-class enum and the composable billing-components
set as canonical primitives.

Acceptance is final because the user directive on 2026-05-20 was explicit
and sequential: "There are only demo/trial and paid per seat + usage"
followed by "and revenue share" followed by the binding correction "tenant
class is demo trial, paid only. revenue share + per seat + per usage". The
directive is captured in memory feedback_tenant_class_demo_trial_vs_paid_
per_seat_usage_2026_05_20. The replacement model is therefore not advisory
or pending consensus — it is the canonical model effective immediately,
with per-µservice plumbing rollout governed by ADR-0331.

No new microservice is introduced by this decision. cloud-billing already
exists and gains owner-of-record responsibility for tenant_class state and
billing_components composition.

The companion retirement decisions are recorded in ADR-0329; this ADR
provides the positive shape that the retired tier system is replaced with.

## Context

### A.1 Named pressure: tier feature gating fought oyatie's "everyone is shared" doctrine

The retired ADR-0316 doctrine treated CRM-class / marketing-automation-class
/ HR-class / ERP-class / ITSM-class etc. surfaces as projections over the
shared substrate via Bronze, Silver, Gold, and Platinum capability tiers.
The model worked for the question "how do we model many product surfaces
without fragmenting microservices?" but it secretly answered a second
question that the user never wanted asked: "how do paying customers get
stratified into capability bands?"

The corpus accumulated 60+ per-µservice capability-tiers/tier-matrix.md
files, a centralized registry/capability-tiers/{bronze,silver,gold,
platinum}.json dataset, naming convention BNF v4 rules N-014/N-015 that
embedded `.<tier>` segments into qualified names, and four-deliverable
Wave 2 audits that included capability-tier-deltas-vs-counterparts-
2026-05-20.md per microservice. The presence of these artifacts shaped
agent expectations: agents started writing tier-segmented performance
benchmarks, tier-segmented SLOs, tier-segmented localization clauses,
tier-segmented compliance evidence, and tier-segmented marketplace offers.

The 2026-05-20 user directive ("we don't have tiers") cut through that
accumulation in one sentence. The follow-up directive ("there are only
demo/trial and paid per seat + usage … and revenue share … tenant class
is demo trial, paid only. revenue share + per seat + per usage") locked
the replacement model into a two-axis primitive: a binary tenant_class
enum on one axis, and a composable billing_components subset on the other
axis. The composability is the load-bearing detail: revenue_share,
per_seat, and per_usage are not separate tenant classes — they are billing
components that any paid tenant can adopt in any combination as part of
their commercial contract.

### A.2 Named pressure: stratified quality conflicts with the industry-leader bar

Oyatie's quality, performance, and scalability bar memory directive
(feedback_quality_performance_scalability_bar) requires every microservice
to ship at Stripe / Palantir / Linear class. That memory is reinforced by
the no-silent-regression doctrine (feedback_no_silent_regression) and by
the keystone bundle of ADRs 0242 through 0255. A tier system that gates
"better quality" behind Silver or Gold or Platinum implicitly tells the
substrate teams that Bronze-class deliveries are acceptable. It also
opens the door for vendor counterparts (Stripe vs Stripe Plus, AWS Basic
vs Business vs Enterprise support, Salesforce Essentials vs Professional
vs Enterprise) to colonize Oyatie's internal model — which is precisely
the kind of accidental mimicry the drift-too-big directive (feedback_
drift_too_big_2026_05_20) calls out.

The binary demo_trial vs paid model collapses the "what does the customer
get?" question into a single answer: the customer gets the full capability
surface, the full quality bar, the full performance budget, and the full
architectural posture. The only legitimate distinction between the two
classes is "has the customer agreed to pay?", which is a commercial fact,
not an engineering fact.

### A.3 Named pressure: composable billing matches how real customers buy

Real Oyatie customer shapes do not slot cleanly into "Silver" or "Gold".
A marketplace seller pays via revenue-share; a B2B enterprise buys named
seats; a developer team consumes API metering; an enterprise reseller
combines all three. The composable billing_components model recognizes
that pricing is a commercial overlay on top of a uniform product, not an
ordering of products into "more capable" bands. A customer who pays only
revenue_share gets the same product surface as a customer who pays
revenue_share + per_seat + per_usage. The difference is the contract
shape, the invoice shape, and the settlement cadence — not the capability
surface.

This matches how vendor anchors actually operate when stripped of their
marketing tiers. Stripe sells revenue-share on payments AND per-usage on
infrastructure AND per-seat on dashboards — the "Standard" and "Plus"
tiers are sales-collateral overlays, not product fragments. Palantir
Foundry charges per-seat licensing AND per-usage on compute AND revenue-
share on operationalized ML — combined per-customer. Snowflake charges
per-usage primarily but also per-seat on dashboards and revenue-share on
the data marketplace. Oyatie's composable billing_components model
encodes this honest shape directly, without the marketing tier overlay.

### A.4 Named pressure: OCI Always Free is a deployment-context fact, not a tier

The OCI Always Free maximization directive (feedback_oci_always_free_
maximization_2026_05_20) recognizes that OCI's perpetually-free tier
(4 OCPU Ampere + 24 GB RAM + 200 GB block + 2 Autonomous DBs + 10 TB
egress + Vault + LB + Streaming + Functions + API Gateway + WAF + Bastion)
is uniquely substantial across hyperscalers. The previous tier mapping
("Bronze = Always Free; Silver/Gold/Platinum = paid OCI") conflated two
orthogonal facts: which OpenTofu module composes the resources (an IaC
concern), and which class of customer pays for them (a commercial concern).

The binary demo_trial vs paid model separates these concerns cleanly.
demo_trial tenants default to the iac/oci-guest/always-free/ OpenTofu
module on the oci-guest deployment context — because that is the cheapest
sustainable substrate for free customers. paid tenants pick any of the
six deployment contexts (oyatie-public-cloud, guest-on-aws, guest-on-oci,
on-prem, colo, oyatie-as-cloud-provider) and any OpenTofu module within
the chosen context. The OCI Always Free profile is no longer a "Bronze
tier" — it is an OpenTofu module composition that the demo_trial class
prefers by default. paid tenants on OCI may also use it for sandbox or
dev sub-tenancies, but cannot run production workloads inside the
Always Free ceiling.

### A.5 Named pressure: composability fights schema explosion

The retired tier system attempted to encode commercial nuance via a
4-element discrete enum (Bronze, Silver, Gold, Platinum). That model
collapses under real-world composability. A "Gold marketplace seller
with internal team and metered API usage" cannot be expressed in a single
tier value; it requires three orthogonal billing axes — which is exactly
what the composable billing_components set provides. The replacement
model uses 2 (tenant_class) × 2³ (billing_components subset) = 9 distinct
states (1 demo_trial state + 8 paid permutations) where the 8 paid
permutations span the empty set, the three singletons, the three pairs,
and the full triple. This is a richer expressive surface than the retired
4-tier enum, and every state has a sharp commercial meaning.

### A.6 Sibling decisions in flight

This ADR is co-authored with ADR-0329 (tier system retirement) and
ADR-0331 (per-microservice tenant_class adoption). ADR-0329 records the
retirement deliverables: deletion of registry/capability-tiers/, deletion
of 60+ per-µservice capability-tiers/tier-matrix.md files, retraction of
13+ Wave 2 capability-tier-deltas-vs-counterparts docs, amendment of
ADR-0316 to mark Superseded, amendment of naming convention BNF v4 to
drop N-014/N-015 .<tier> segments, amendment of ADR-0328 §D-19 to
reword without "OCI Bronze" terminology. ADR-0331 records the per-
microservice plumbing: which test paths exercise demo_trial cap-hit
behavior, which test paths exercise paid no-cap behavior, which Cedar
fragments cover tenant_class attribute matching, and which µservices
emit usage events to cloud-billing for the per_usage component.

ADR-0330 (this ADR) provides the positive primitive that 0329 retires
toward and that 0331 plumbs into the microservices.

### A.7 Authority chain context

Per ADR-0244 (tenant-as-universal-scoping-primitive), every audited row
in oyatie already carries tenant context. The tenant_class field extends
that primitive: every row that carries tenant_id now also carries
tenant_class, and every Cedar policy that already reads tenant_id can
read tenant_class with no additional wiring beyond a one-line attribute
addition. Per ADR-0243 (cedar-as-universal-gate), the runtime gate for
tenant-class-conditional behavior is a Cedar policy evaluation against
the principal claim — never an inline `if tenant_class == "demo_trial"`
guard in microservice code. Per ADR-0247 (self-modification doctrine),
Foundry-issued principals also carry tenant_class (oyatie.foundry.* is
itself a reserved-namespace tenant operating under the paid class with
all three billing components disabled at the system level).

### A.8 Customer-shape grounding

The replacement model deliberately accepts that tenant_class is a coarse
discriminator. It is intentionally coarse: oyatie's commercial story
does not require finer granularity at the engineering layer. Finer
discriminators (industry vertical, compliance pack, BYOK status, support
level, region, etc.) exist as orthogonal primitives that compose with
tenant_class — not as sub-tiers of tenant_class. This compositional
posture is what lets the binary enum stay closed without forcing every
customer-specific behavior into a tenant-class branch.

## Decision

The decision is recorded as a numbered set of normative clauses. Every
clause is a load-bearing commitment; downstream microservice work,
governance lanes, and CI checks bind to clause numbers. Numbering is
immutable once accepted.

### B.1 The tenant_class enum

1. **B.1.1** The tenant_class field on every oyatie tenant principal is a
   closed enum with exactly two members: `demo_trial` and `paid`.
2. **B.1.2** No third value of tenant_class may be introduced without
   superseding this ADR. Proposals to add a value (for example,
   `internal`, `partner`, `nonprofit`, `enterprise`) must be folded into
   one of the two existing values plus a composable orthogonal primitive
   (typically a billing_component, a compliance_pack, or a support level).
3. **B.1.3** The tenant_class value of an existing tenant may change in
   exactly one direction: `demo_trial` may convert to `paid`. A `paid`
   tenant may never demote to `demo_trial`. Rationale: downgrade would
   silently erase usage history, billing ledger entries, compliance
   posture, and committed seat counts — operations that have to flow
   through explicit churn, refund, and data-deletion paths, not through
   a class-flip.
4. **B.1.4** The tenant_class field is mandatory on every tenant record;
   no oyatie tenant exists without a tenant_class value.
5. **B.1.5** The tenant_class enum is closed at the type system layer.
   Each microservice's Rust code uses `#[non_exhaustive] enum TenantClass`
   only when the consumer is downstream of cloud-billing's
   tenant-model crate; the cloud-billing crate itself uses the exhaustive
   form so that the compiler rejects any unknown variant at the source
   of truth.

### B.2 The billing_components set for paid tenants

6. **B.2.1** The billing_components field is meaningful only when
   tenant_class = paid. On demo_trial tenants, billing_components is
   defined as the empty set and is not user-mutable.
7. **B.2.2** When tenant_class = paid, billing_components is a subset of
   the closed set `{ revenue_share, per_seat, per_usage }`.
8. **B.2.3** The subset may be empty (a paid tenant with no billing
   components is a valid configuration during contract setup — for
   example, a signed letter-of-intent customer before billing has been
   configured), but cloud-billing emits a non-blocking advisory event
   when a paid tenant has been in `billing_components = {}` for more
   than 7 days, prompting commercial follow-up.
9. **B.2.4** The subset may include any single component, any pair, or
   all three components. The 8 valid combinations are:
   - `{}` — paid, no billing component configured yet (transient)
   - `{revenue_share}` — pure marketplace seller / B2C operator
   - `{per_seat}` — pure B2B enterprise named-user model
   - `{per_usage}` — pure pay-as-you-go developer / metered consumption
   - `{revenue_share, per_seat}` — reseller with internal team
   - `{revenue_share, per_usage}` — marketplace seller with metered ops
   - `{per_seat, per_usage}` — enterprise with consumption workload
   - `{revenue_share, per_seat, per_usage}` — complex enterprise reseller
10. **B.2.5** A paid tenant's billing_components may be added or removed
    mid-contract through a contract amendment recorded in cloud-billing.
    Removal triggers a clean settlement of any outstanding billing under
    the removed component before the change takes effect.
11. **B.2.6** Each billing_component has an independent contract-terms
    record in cloud-billing: revenue_share carries a commission rate and
    a category mapping; per_seat carries a per-seat monthly or annual
    price and a seat-count ceiling; per_usage carries a per-meter
    price-per-unit table and an optional soft-cap configuration.
12. **B.2.7** Billing_components are orthogonal. No component blocks
    another; no component requires another; no component depends on
    another at the configuration layer.

### B.3 demo_trial tenant semantics

13. **B.3.1** demo_trial tenants pay $0 to oyatie for the trial window.
14. **B.3.2** demo_trial tenants default to the OCI Always Free OpenTofu
    module under the oci-guest deployment context. The module composes
    only Always Free OCI resources (2 Ampere A1 instances totaling 4
    OCPU + 24 GB RAM, 200 GB block storage, 2 Autonomous Databases ×
    20 GB each, 10 TB egress, 1 LB, Vault, Streaming, Functions, API
    Gateway, WAF, Bastion).
15. **B.3.3** demo_trial tenants may opt into a different deployment
    context at trial-creation time (for example, an enterprise prospect
    wants their trial on guest-on-aws), but the trial-specific caps
    still apply, and the default cost-control posture remains "no paid
    resources provisioned".
16. **B.3.4** demo_trial tenants are subject to two cap families: a time
    cap (default 30 days, configurable to 60 or 90 days per global
    policy or per-microservice override) and a usage cap (per-microservice
    limits on resource units; for example, N agents in oya-agentic-agent,
    N workflows in oya-workflow-engine, N seats in cloud-iam, N MLS
    groups in oya-messaging-mls, N GB in cloud-data-store).
17. **B.3.5** demo_trial tenants receive best-effort SLO posture. The
    µservice's published SLO targets remain the same (uniform quality
    bar), but there is no contractual SLO commitment. Service is provided
    as-is with self-serve / community support only.
18. **B.3.6** demo_trial tenants may not activate any compliance pack
    (HIPAA, GDPR-DPO, SOC2-Type-II, PCI-DSS, EU-AI-Act, CSAP, K-ISMS,
    ISMS-P, etc.). Compliance activation requires the contractual,
    paid posture per ADR-0251.
19. **B.3.7** demo_trial tenants may not opt into BYOK for LLM providers,
    payment providers, or KMS roots. BYOK requires tenant_class = paid
    per ADR-0255 §D-4.
20. **B.3.8** demo_trial tenants may not list listings on the marketplace
    as sellers (no revenue_share applies); they may consume free
    marketplace listings for evaluation but cannot purchase paid listings.
21. **B.3.9** demo_trial tenants receive the same product surface, the
    same UX shell, the same agent dispatch, the same workflow engine,
    the same ontology, the same audit-chain semantics, and the same
    observability stack as paid tenants. The only operational differences
    are the caps in B.3.4, the SLO posture in B.3.5, and the gates in
    B.3.6 through B.3.8.
22. **B.3.10** A demo_trial tenant approaching a cap (80% threshold) is
    notified via the tenant's primary contact channel with a clear
    upgrade-to-paid call-to-action that includes the billing_components
    selection workflow.
23. **B.3.11** A demo_trial tenant that breaches a cap enters a grace
    window (default 7 days). During grace, the tenant retains read
    access to its data and the conversion-to-paid flow remains open;
    write paths are blocked at the API gateway via a Cedar deny policy.
24. **B.3.12** A demo_trial tenant whose grace window expires without
    conversion enters a suspension state. Data is retained per the
    tenant data-retention policy (default 90 days after suspension)
    before being purged. Conversion to paid during the retention window
    restores full access.
25. **B.3.13** demo_trial tenants are scoped to a reserved tenant_id
    prefix `demo_` for top-level tenants; sub-tenancies use the same
    convention. The prefix is a soft hint for operational dashboards;
    enforcement remains via the tenant_class field, not the prefix.
26. **B.3.14** demo_trial tenant_class is a property of the tenant, not
    a property of a user account. A user who belongs to multiple tenants
    sees different tenant_class postures across their workspace context
    switches.

### B.4 paid tenant semantics

27. **B.4.1** paid tenants execute a commercial contract with oyatie or
    with an authorized reseller. The contract names the billing_components
    in effect at activation time.
28. **B.4.2** paid tenants choose any of the six canonical deployment
    contexts at provisioning time: oyatie-public-cloud, guest-on-aws,
    guest-on-oci, on-prem, colo, oyatie-as-cloud-provider. The choice
    is bound to the tenant record and is governed by ADR-0215.
29. **B.4.3** paid tenants are not subject to default usage caps. The
    µservice scales with usage; usage is metered when per_usage is in
    the billing_components set; usage is uncapped (but optionally soft-
    capped at the tenant's request) otherwise.
30. **B.4.4** paid tenants receive contractual SLO posture per their
    contract. The SLO targets are uniform across all paid tenants
    (industry-leader bar); the legal commitment to those targets is
    what differs from demo_trial.
31. **B.4.5** paid tenants may activate any compliance pack that is
    applicable to their deployment context, industry vertical, and
    jurisdiction, per ADR-0251.
32. **B.4.6** paid tenants may opt into BYOK for LLM providers, payment
    providers, KMS roots, identity providers, and any other oyatie
    BYOK-eligible surface, per ADR-0255 §D-4.
33. **B.4.7** paid tenants may list, sell, and purchase on the
    marketplace per ADR-0249. Marketplace sales by paid tenants
    automatically engage the revenue_share billing component for that
    tenant.
34. **B.4.8** paid tenants receive the same product surface, the same
    UX shell, the same agent dispatch, the same workflow engine, the
    same ontology, the same audit-chain semantics, and the same
    observability stack as demo_trial tenants. The differences are the
    no-cap default in B.4.3, the contractual SLO in B.4.4, and the
    unlocked gates in B.4.5 through B.4.7.
35. **B.4.9** paid tenants may have any number of seats (per_seat
    component), any meter usage (per_usage component), and any number
    of revenue-generating events (revenue_share component) within the
    contract's commercial terms.
36. **B.4.10** paid tenants may operate sub-tenancies (for example, an
    enterprise paid tenant with multiple BUs, each with its own scoped
    sub-tenant). Each sub-tenant inherits tenant_class = paid; the
    parent tenant aggregates the sub-tenancy billing into a single
    invoice unless the contract specifies separate invoicing.

### B.5 revenue_share component

37. **B.5.1** The revenue_share component applies when oyatie earns a
    percentage of revenue routed through oyatie's surfaces. The
    component is independently activatable; it does not require
    per_seat or per_usage to also be active.
38. **B.5.2** Customer shapes that engage revenue_share include:
    marketplace sellers (plugins / apps / workflows / agents / models /
    datasets sold through ADR-0249's marketplace); B2C consumer-product
    operators (customer builds a consumer app on oyatie; consumers pay
    the customer; oyatie takes a cut of consumer-facing transactions);
    embedded SaaS resellers (customer white-labels oyatie into their
    own SaaS sold downstream); affiliate / channel partners (customer
    drives signups to oyatie's products; oyatie pays the customer a
    referral share — this is a "negative" revenue_share).
39. **B.5.3** The commission rate per revenue_share contract is set by
    the per-marketplace-category ADR (one ADR per category — pending
    authoring as Wave 15K work). Default ranges by category are
    referenced in §G as open questions.
40. **B.5.4** The revenue-share computation occurs in cloud-billing.
    Every transaction routed through an oyatie surface emits a
    revenue-event with: tenant_id, transaction_id, gross_amount,
    currency, category, timestamp, idempotency_key. cloud-billing
    cohorts revenue-events per the rev-share contract terms and
    computes oyatie's share at monthly close.
41. **B.5.5** Settlement occurs monthly. cloud-billing emits a
    settlement-statement event to the payments microservice, which
    initiates the payout (or invoice, if oyatie owes the tenant).
42. **B.5.6** Clawback and chargeback handling is baked into the
    settlement flow. A clawback (where a downstream consumer refunds
    a transaction after oyatie has paid the tenant's share) is
    recorded as a reverse revenue-event and netted in the next
    settlement window.
43. **B.5.7** revenue_share tenants must integrate with the audit-chain
    microservice for transaction provenance. Every revenue-event is
    a first-class audit-chain entry; settlement statements cite the
    underlying audit-chain hashes.
44. **B.5.8** revenue_share tenants are subject to FX accounting rules.
    Multi-currency revenue is recorded at the transaction-time FX rate
    (per the cloud-billing-tax FX feed) and settled in the tenant's
    contracted settlement currency; FX delta is recorded as a
    settlement-FX-adjustment line item.
45. **B.5.9** revenue_share tenants may not unilaterally suspend
    revenue-event reporting. Doing so triggers an audit-chain integrity
    violation and a Cedar deny on further transaction routing.
46. **B.5.10** Negative revenue_share (affiliate / referral) operates
    under the same settlement plumbing but with oyatie as the payor
    and the tenant as the payee. cloud-billing's settlement-statement
    event carries a `direction` field with values `oyatie_collects`
    or `oyatie_pays`.

### B.6 per_seat component

47. **B.6.1** The per_seat component applies when the tenant pays for
    named user licenses. The component is independently activatable;
    it does not require revenue_share or per_usage to also be active.
48. **B.6.2** A seat is one named human user (or one named non-human
    principal — service accounts, headless bots, scheduled jobs — when
    the contract specifies). A multi-user organization is N seats.
49. **B.6.3** Seat counting is the responsibility of cloud-iam (the
    issuer of principals) and identity (the user-record store).
    cloud-billing reads seat counts via a documented cloud-iam API at
    the monthly invoice close.
50. **B.6.4** Deactivated users drop from the seat count after a
    configurable grace window (default 7 days) to absorb accidental
    deactivations and re-activations.
51. **B.6.5** Over-seat principals (where the active-seat count
    exceeds the contract's seat ceiling) fail to authenticate after the
    contract's grace window. cloud-iam emits a fail-closed deny with
    a clear "seat ceiling exceeded" error and a tenant-admin call-to-
    action.
52. **B.6.6** Seat pricing is set per the contract. Monthly invoices
    are the default cadence; annual prepay is an option that produces
    a single invoice at contract anniversary plus monthly true-up
    invoices for seats added mid-cycle.
53. **B.6.7** Seat add and remove operations are authority-tier 2
    operations on cloud-iam, requiring tenant-admin authorization
    governed by Cedar policy.
54. **B.6.8** Multi-tenant users (one human associated with multiple
    tenants) consume one seat per tenant. There is no cross-tenant
    seat pooling.

### B.7 per_usage component

55. **B.7.1** The per_usage component applies when the tenant is metered
    on resource consumption. The component is independently activatable;
    it does not require revenue_share or per_seat to also be active.
56. **B.7.2** Each oyatie microservice declares its meter shape in its
    PRD. The meter shape is an enum (closed per-microservice) of
    metered units the microservice exposes. Examples: oya-intelligence-
    inference emits `llm_tokens_consumed` (input + output, by model);
    oya-workflow-engine emits `workflow_executions`; cloud-data-store
    emits `gb_stored` and `gb_egress`; oya-cloud-api-gateway emits
    `api_calls`; oya-intelligence-inference also emits `gpu_seconds`;
    oya-search-index emits `vector_search_queries`.
57. **B.7.3** Meter events are emitted via the canonical observability
    contract (per ADR-0130 agentic SLO-gated promotion). Each event
    carries: tenant_id, meter_unit, quantity, timestamp, idempotency_
    key, optional pricing_dimension (model name, region, tier).
58. **B.7.4** cloud-billing aggregates meter events continuously and
    surfaces hourly / daily / weekly visibility in the finops-portal
    microservice.
59. **B.7.5** Per-tenant invoice is generated at monthly close. The
    invoice line items group by meter_unit and pricing_dimension.
60. **B.7.6** The per_usage component has no default cap. The tenant
    may opt into a soft cap via cloud-billing configuration: at the
    soft-cap threshold, an alert is emitted; the tenant may set a hard
    cap as well, at which point further usage above the hard cap is
    Cedar-denied at the gateway.
61. **B.7.7** Meter pricing is set per the contract. Different tenants
    may have different per-unit prices for the same meter (volume
    discounts, custom-negotiated pricing, regional differences).
62. **B.7.8** Meter idempotency keys are required on every emitted
    event. cloud-billing deduplicates on (tenant_id, meter_unit,
    idempotency_key) within a 7-day window.
63. **B.7.9** Meter clawbacks (where a usage event was emitted in
    error and must be reversed) are supported via a `correction_for`
    field that references the original event's idempotency_key.
    cloud-billing applies the correction in the next monthly close.
64. **B.7.10** Per_usage tenants may request a usage-projection report
    (next month's projected invoice based on rolling 30-day usage),
    surfaced via the finops-portal.

### B.8 Composability examples (informative)

65. **B.8.1** Pure enterprise customer. Customer is a 500-seat
    organization buying oyatie as an internal collaboration / agentic-
    automation substrate. Configuration: `tenant_class = paid`,
    `billing_components = {per_seat}`. Invoice cadence: monthly per-seat
    invoice. Marketplace consumption: pay-per-purchase but no revenue-
    share on sells. Compliance: SOC2-Type-II pack activated; HIPAA
    pack on the healthcare-BU sub-tenant.
66. **B.8.2** Pay-as-you-go developer team. Customer is a 5-person
    dev team building an integration product on oyatie. Configuration:
    `tenant_class = paid`, `billing_components = {per_usage}`. Invoice
    cadence: monthly per-meter invoice. Pricing-dimension breakdown:
    LLM tokens by model, API calls, workflow executions, GB stored.
67. **B.8.3** Marketplace seller. Customer is an indie developer
    selling 12 plugins on oyatie's marketplace. Configuration:
    `tenant_class = paid`, `billing_components = {revenue_share}`.
    Invoice cadence: monthly settlement statement (oyatie pays the
    seller their share). Audit-chain integration is mandatory.
68. **B.8.4** Enterprise with consumption workload. Customer is a
    300-seat ML team running large inference workloads. Configuration:
    `tenant_class = paid`, `billing_components = {per_seat, per_usage}`.
    Invoice cadence: monthly invoice combining per-seat + per-meter
    line items. Both components are billed independently and totaled
    at the bottom of the invoice.
69. **B.8.5** Reseller with internal team. Customer is a consulting
    firm that white-labels oyatie for downstream clients and has its
    own 20-person team using oyatie internally. Configuration:
    `tenant_class = paid`, `billing_components = {revenue_share,
    per_seat}`. Invoice cadence: monthly per-seat invoice + monthly
    revenue-share settlement statement.
70. **B.8.6** Complex enterprise reseller. Customer is an enterprise
    SI partner with 1,500 seats, white-labeled offerings to downstream
    customers, and metered API usage. Configuration: `tenant_class =
    paid`, `billing_components = {revenue_share, per_seat, per_usage}`.
    Invoice cadence: combined monthly invoice + monthly settlement
    statement.
71. **B.8.7** Free trial converting to paid. Prospect starts as
    `tenant_class = demo_trial`, `billing_components = {}`. At Day 23,
    prospect converts via the contract flow. Tenant becomes
    `tenant_class = paid`, `billing_components = {per_seat, per_usage}`
    per the chosen contract. Trial usage history is retained but is
    not retroactively billed.
72. **B.8.8** Negative revenue_share affiliate partner. Customer is a
    content creator who drives signups via referral links. Configuration:
    `tenant_class = paid`, `billing_components = {revenue_share}`,
    contract direction = `oyatie_pays`. Customer is also entitled to a
    seat for their own oyatie account access; this is provided at $0
    under the partner's contract addendum and is not part of per_seat
    billing.

### B.9 Quality bar parity (uniform across tenant classes)

73. **B.9.1** Performance SLO targets, scalability targets, security
    posture, observability coverage, accessibility compliance, and
    localization coverage are uniform across demo_trial and paid
    tenants.
74. **B.9.2** The "uniform" claim is CI-enforced via the existing
    industry-leader-grade governance lanes (oya-governance-quality-bar,
    oya-governance-performance-bar, oya-governance-substance-bar). No
    new lane is needed to enforce the no-stratification rule; the
    existing lanes already reject stratified delivery.
75. **B.9.3** Microservice tests must include both a demo_trial path
    and a paid path. The paths exercise identical functional behavior;
    they differ only in cap-hit semantics, SLO commitment, and gate
    posture (compliance / BYOK / marketplace-sell).
76. **B.9.4** Counterexample: a microservice that intentionally returns
    a "feature locked for trial" message on demo_trial tenants fails
    the parity check. Cap-hit is acceptable ("you have reached your
    trial limit of N workflows; upgrade to continue"); feature lockout
    is not.
77. **B.9.5** Counterexample: a microservice that delivers higher
    accuracy / lower latency / richer telemetry to paid tenants fails
    the parity check. The accuracy, latency, and telemetry posture is
    uniform.
78. **B.9.6** Acceptable difference: SLO commitment posture differs.
    The SLO target (for example, p99 < 200ms) is the same for both
    classes; the contractual penalty for missing it is paid-only.

### B.10 cloud-billing as keystone owner

79. **B.10.1** cloud-billing is the source-of-truth for the tenant_class
    field. Other microservices may cache tenant_class; mutations occur
    only via cloud-billing's tenant-class-mutate API.
80. **B.10.2** cloud-billing owns the billing_components set on every
    paid tenant. Add and remove operations are recorded as audit-chain
    events.
81. **B.10.3** cloud-billing owns the per-component contract-terms
    record (commission rates, seat pricing, meter pricing, soft-cap
    configurations, FX rules).
82. **B.10.4** cloud-billing owns the demo_trial → paid conversion
    flow. The conversion is a single atomic transaction that updates
    tenant_class, sets the initial billing_components, records the
    contract terms, and emits a tenant-class-transition event to the
    audit-chain.
83. **B.10.5** cloud-billing owns the cap-breach detection and
    grace-period flow for demo_trial tenants. Cap-breach triggers a
    tenant-cap-breach event consumed by cloud-iam (to apply Cedar
    write-deny) and by the notifications microservice (to send the
    upgrade CTA).
84. **B.10.6** cloud-billing owns the time-expiry detection for
    demo_trial tenants. At T-7 days, T-3 days, and T-0, cloud-billing
    emits notifications via the notifications microservice.
85. **B.10.7** cloud-billing owns the rev-share settlement payout
    flow. At monthly close, cloud-billing emits a settlement-statement
    event to the payments microservice, which initiates the payout
    or invoice direction depending on the settlement direction.
86. **B.10.8** cloud-billing owns the rev-share clawback / chargeback
    handling. Clawback events are netted in the next settlement.
87. **B.10.9** cloud-billing owns the per-tenant billing_components
    mutation flow. Mutations are contract-amendment-gated; clean
    settlement of removed components precedes the mutation.
88. **B.10.10** cloud-billing exposes a tenant-class read API
    consumed by cloud-iam and identity at principal-issuance time.
    The API returns the tenant_class and billing_components atomically.
89. **B.10.11** cloud-billing publishes a tenant-class-mutated event
    on the canonical event substrate (per ADR-0145 inter-microservice
    communication reform: direct gRPC + 3 invariants); other
    microservices subscribe to refresh their caches.
90. **B.10.12** cloud-billing's tenant-class state is replicated to
    every deployment context (oyatie-public-cloud, guest-on-aws,
    guest-on-oci, on-prem, colo, oyatie-as-cloud-provider) per the
    multi-context platform doctrine; eventual consistency window is
    ≤ 30 seconds globally.

### B.11 Cedar policy gate templates

91. **B.11.1** Every Cedar policy that gates a behavior conditional on
    tenant_class reads tenant_class from the principal claim. The
    claim name is `principal.tenant_class`. Principal claims also
    carry `principal.tenant_id` (per ADR-0244) and `principal.billing_
    components` (a Cedar set type).
92. **B.11.2** Cedar gate template 1: deny compliance pack activation
    for demo_trial. `forbid(principal, action == Action::"activate_
    compliance_pack", resource) when { principal.tenant_class ==
    "demo_trial" };`
93. **B.11.3** Cedar gate template 2: deny BYOK opt-in for demo_trial.
    `forbid(principal, action == Action::"configure_byok_provider",
    resource) when { principal.tenant_class == "demo_trial" };`
94. **B.11.4** Cedar gate template 3: deny marketplace listing for
    demo_trial. `forbid(principal, action == Action::"publish_
    marketplace_listing", resource) when { principal.tenant_class ==
    "demo_trial" };`
95. **B.11.5** Cedar gate template 4: deny demo_trial write after cap
    breach. `forbid(principal, action in [Action::"create", Action::
    "update", Action::"delete"], resource) when { principal.tenant_
    class == "demo_trial" && principal.cap_breached == true };`
96. **B.11.6** Cedar gate template 5: deny paid downgrade. `forbid
    (principal, action == Action::"change_tenant_class", resource)
    when { principal.tenant_class == "paid" && context.target_tenant_
    class == "demo_trial" };`
97. **B.11.7** Cedar gate template 6: gate revenue-share-only actions.
    `forbid(principal, action == Action::"settle_rev_share", resource)
    when { !(principal.tenant_class == "paid" && "revenue_share" in
    principal.billing_components) };`
98. **B.11.8** Cedar gate templates 1 through 6 ship as canonical
    Cedar fragments in the policy-engine microservice's fragment
    catalog. Every microservice that needs tenant-class-gated behavior
    references one of these fragments rather than authoring an
    inline policy.

### B.12 Cross-cutting invariants

99. **B.12.1** No microservice may inline a `match tenant_class` in
    business logic that would alter the functional surface. Functional
    surface uniformity is part of the quality-bar parity rule in B.9.
100. **B.12.2** Every audited row that carries `tenant_id` also carries
    `tenant_class` and, when applicable, the `billing_components`
    snapshot at the time of the audited operation. This extends
    ADR-0244's tenant-scoping primitive.
101. **B.12.3** No new µservice surface may invent a tenant-class
    primitive of its own. The enum and the billing_components set are
    canonical and shared.
102. **B.12.4** Demo_trial tenants are real tenants. They are not test
    fixtures, not synthetic accounts, not sample data. They are
    first-class principals with real data, real audit-chain entries,
    real observability footprint, and real Cedar evaluation. Their
    only differences from paid tenants are commercial.
103. **B.12.5** Quality, performance, scalability, security, and
    accessibility CI lanes do not stratify by tenant_class. A test
    that runs on a demo_trial fixture meets the same pass bar as the
    same test on a paid fixture.
104. **B.12.6** All microservice tests are required to run against
    both a demo_trial fixture and a paid fixture. CI lane `ci-tenant-
    class-adoption-check` verifies dual-fixture coverage and rejects
    microservices with single-fixture tests.

## Consequences

### C.1 Direct corpus impact

The replacement model directly retires the tier-system corpus listed in
ADR-0329's retirement deliverable: ADR-0316 marked Superseded; 60+ per-
µservice capability-tiers/tier-matrix.md files deleted; registry/
capability-tiers/{bronze,silver,gold,platinum}.json retired and replaced
with registry/tenant-classes/{demo-trial,paid}.json plus registry/
billing-components/{revenue-share,per-seat,per-usage}.json; ADR-0328
§D-19 amended to remove "OCI Bronze" wording; naming convention BNF v4
N-014/N-015 amended to drop the `.<tier>` segment; 13+ already-authored
Wave 2 capability-tier-deltas-vs-counterparts-2026-05-20.md docs
retracted; brief-template.md §3.x tier anchors scrubbed; 32 already-
authored Phase 0 capability-tier-deltas deliverables retracted.

### C.2 Brief-template amendments

The agent brief template gains a new anchor: §3.X-tenant-class. The
anchor declares which tenant_class paths the microservice exercises in
tests, which Cedar fragments it composes, and which billing_components
it emits events to (for per_usage). The anchor replaces the retired
§3.X-capability-tier anchor in the same numerical position.

### C.3 CI lane updates

The existing oya-governance-* lanes that referenced tier-segmented
behavior are amended:
- `oya-governance-capability-tier-registry-shape` is RETIRED.
- `oya-governance-capability-tier-cedar-coverage` is REPLACED by
  `oya-governance-cedar-tenant-class-attribute-coverage`.
- `oya-governance-capability-tier-ontology-projection-pin` is RETIRED
  because ontology projection no longer pins to tiers.
- `oya-governance-capability-tier-workflow-template-coverage` is
  REPLACED by `oya-governance-workflow-template-uniform-coverage`,
  which verifies that workflow templates do not stratify by tenant
  class.

New lanes introduced by this ADR:
- `ci-tenant-class-adoption-check` (BLOCKER once ADR-0331 lands)
- `oya-governance-tenant-class-enum-closed`
- `oya-governance-billing-components-subset-closed`
- `oya-governance-cloud-billing-source-of-truth`
- `oya-governance-iam-principal-tenant-class-claim`
- `oya-governance-cedar-tenant-class-attribute-coverage`
- `oya-governance-audit-chain-tenant-class-transition`
- `oya-governance-demo-trial-cap-enforcement`
- `oya-governance-paid-quality-bar-parity`

### C.4 Per-microservice plumbing

Every microservice gains a small but uniform plumbing footprint:
- Read tenant_class and billing_components from the principal claim.
- Branch behavior on cap-hit (for demo_trial) without altering
  functional surface.
- Emit usage events to cloud-billing if the microservice contributes
  to per_usage meters.
- Compose canonical Cedar fragments rather than inline policies.
- Test against both tenant_class fixtures.

The per-microservice plumbing details are recorded in ADR-0331.

### C.5 Documentation consequences

The retired tier-segmented docs (capability-authoring.md, autonomy-
ceiling.md tier sections, hyperscaler-best-practices.md tier sections,
observability.md tier sections, on-call.md tier sections, workflow-
substrate-engine.md tier bindings, asyncapi-3-1-authoring.md per-tier
event grants) are amended to use the tenant_class binary plus
billing_components composability vocabulary. Sections that exclusively
described tier mechanics are retracted; sections that described
behaviors now mapped to tenant_class are reworded.

### C.6 Naming convention BNF v4 amendment

The N-014 and N-015 rules previously required the form `<microservice>.
<capability>.<tier>`. After this ADR, the canonical form is
`<microservice>.<capability>` without the `.<tier>` suffix. Where a
qualifier is required to disambiguate behavior between demo_trial and
paid contexts in a name (rare), the form is `<microservice>.
<capability>.<tenant_class>` (for example, `cloud-billing.invoice.
paid`, but this form should be avoided in favor of polymorphic naming).

### C.7 Marketplace impact (ADR-0249 alignment)

Marketplace listings remain category-segmented (plugins, apps,
workflows, agents, models, datasets) per ADR-0249. The previous tier
overlay on marketplace listings is retired. Marketplace purchases by
paid tenants engage revenue_share automatically; marketplace
consumption by demo_trial tenants is restricted to free listings only.

### C.8 Compliance pack impact (ADR-0251 alignment)

Compliance pack activation is gated by tenant_class = paid per B.3.6.
The retired tier-gated activation (Silver activates SOC2-Type-II;
Gold activates HIPAA; Platinum activates PCI-DSS) is replaced by a
flat "any pack is available to any paid tenant in the applicable
jurisdiction / vertical" rule. Per-tenant pack-eligibility is governed
by ADR-0251's per-cell certification model, not by tenant_class
sub-bands.

### C.9 BYOK impact (ADR-0255 amendment alignment)

BYOK opt-in is gated by tenant_class = paid per B.3.7 and per ADR-0255
§D-4. The `provider_credential_mode` field has values `platform_
default`, `byok`, and `byok_required_by_pack`. demo_trial tenants are
restricted to `platform_default`; paid tenants may opt into any mode
applicable to their compliance posture.

### C.10 Audit-chain impact

Audit-chain extends its event schema to record:
- `tenant_class_transition_event`: emitted on demo_trial → paid
  conversion, with the old class, new class, contract identifier, and
  initial billing_components.
- `billing_components_mutation_event`: emitted on add or remove of a
  component, with the old set, new set, contract amendment identifier,
  and any settlement events that the mutation triggered.
- `cap_breach_event`: emitted on demo_trial cap-hit, with the cap
  type, current value, ceiling value, and grace window start time.
- `grace_window_expiry_event`: emitted at end of demo_trial grace
  window without conversion.

### C.11 finops-portal impact

The finops-portal microservice gains views for:
- Per-tenant tenant_class status and billing_components set.
- Per-tenant per-meter usage (when per_usage is active).
- Per-tenant seat count (when per_seat is active).
- Per-tenant rev-share running balance (when revenue_share is active).
- Per-tenant upcoming-invoice projection.

### C.12 Substance bar parity

Per ADR-0322 substance bar, this ADR is itself substance-bar grade
(line floor 800; bespoke clauses, not templated text). The replacement
plumbing in ADR-0331 inherits the substance bar.

## D. Implementation footprint

### D.1 cloud-billing crate boundaries

The cloud-billing microservice gains the following crate
responsibilities (each as a new module within the existing crate
layout per ADR-0131 per-microservice flat layout):

- `cloud-billing/src/tenant_class.rs`: defines the closed enum
  `enum TenantClass { DemoTrial, Paid }`, its serde representation,
  its Cedar serialization helper, and its audit-chain emission helper.
- `cloud-billing/src/billing_components.rs`: defines the
  `BillingComponent` enum (RevenueShare, PerSeat, PerUsage), the
  `BillingComponentSet` newtype wrapping a HashSet<BillingComponent>,
  and its membership-query helpers (`contains_revenue_share()`, etc.).
- `cloud-billing/src/conversion.rs`: the demo_trial → paid conversion
  transaction. Validates contract record presence, applies the initial
  billing_components, writes the audit-chain event, publishes the
  tenant-class-mutated event.
- `cloud-billing/src/cap_breach.rs`: cap-breach detection for
  demo_trial tenants. Polls per-microservice usage meters at a
  configurable cadence (default every 5 minutes); emits cap-breach
  events when a meter exceeds its ceiling.
- `cloud-billing/src/grace_window.rs`: grace-window state machine.
  Starts on cap-breach; default 7 days; emits expiry events; supports
  early conversion to paid.
- `cloud-billing/src/settlement.rs`: monthly settlement engine for
  revenue_share. Cohorts revenue-events per contract terms; computes
  oyatie's share; emits settlement-statement events; handles
  clawback netting; handles FX adjustment.
- `cloud-billing/src/seat_counter.rs`: per_seat counting integration
  with cloud-iam. Reads active-seat counts; computes monthly invoice
  line items; emits over-seat alerts.
- `cloud-billing/src/meter_aggregator.rs`: per_usage meter aggregation.
  Subscribes to meter events from all microservices; deduplicates on
  idempotency keys; aggregates to hourly/daily/weekly buckets;
  computes monthly invoice line items.

### D.2 cloud-iam principal-claim emission

cloud-iam, on every principal issuance (login, token refresh, service-
account token mint), reads tenant_class and billing_components from
cloud-billing's read API and embeds them as principal claims:
- `principal.tenant_class`: the tenant_class string value.
- `principal.tenant_id`: the tenant identifier (per ADR-0244).
- `principal.billing_components`: a JSON array of billing-component
  strings (Cedar set type at the policy layer).
- `principal.cap_breached`: a boolean indicating whether the tenant
  is currently in cap-breach state (relevant only for demo_trial).

Token expiry windows for the claim refresh: principal tokens carry a
TTL of 1 hour by default; tenant-class mutations propagate via the
event substrate within 30 seconds, prompting clients to refresh.

### D.3 identity microservice integration

The identity microservice (the user-record store distinct from
cloud-iam's principal-issuance role) is updated to:
- Bind every user to a tenant_id and read the tenant's tenant_class
  for UX-layer state.
- Surface tenant_class in the user profile API (read-only for end
  users; write-only via the tenant-admin contract amendment flow).
- Expose per-tenant active seat counts for cloud-iam's seat-counter
  consumption.

### D.4 audit-chain microservice integration

audit-chain registers the new event schemas defined in C.10:
- `tenant_class_transition_event`
- `billing_components_mutation_event`
- `cap_breach_event`
- `grace_window_expiry_event`

These events become first-class entries in the audit-chain hash
ledger; cloud-billing's settlement statements cite the underlying
audit-chain hashes for provenance.

### D.5 cloud-billing-tax integration

The cloud-billing-tax microservice (distinct from cloud-billing per
ADR-0131 single-concern layout) applies billing-component-specific
tax treatment:
- per_seat invoices apply jurisdictional SaaS tax rules (per the
  ship-from / ship-to / tenant-billing-address triplet).
- per_usage invoices apply jurisdictional consumption tax rules
  (often different from SaaS rules; for example, EU digital services
  VAT vs US-state SaaS tax variability).
- revenue_share settlements apply jurisdictional payee-side tax
  withholding rules; backup withholding for missing W-9/W-8 forms in
  US contexts; VAT reverse-charge in EU contexts; per-country
  withholding in APAC contexts.

cloud-billing-tax exposes a per-component tax-computation API that
cloud-billing calls at invoice and settlement time.

### D.6 payments microservice integration

The payments microservice handles the money movement:
- Per_seat and per_usage invoices: charge the tenant's stored
  payment method (card, ACH, wire, BYOK PSP per ADR-0255 §D-4).
- Revenue_share settlements with direction `oyatie_pays`: initiate
  payouts to the tenant's payout method (bank account, card payout
  rail, fiat or stable-coin per the contract).
- Revenue_share settlements with direction `oyatie_collects`:
  invoice the tenant via the same channel as per_seat / per_usage.

### D.7 cloud-data-store invariants

The cloud-data-store microservice, which owns row-level tenant
scoping per ADR-0244, extends its row schema to include `tenant_
class` and (when applicable) `billing_components_snapshot`. These
columns are populated by triggers from cloud-billing's
tenant-class-mutated event subscription; they enable historical
analytics on tenant-class transitions without joining back to
cloud-billing.

### D.8 Per-microservice tests covering both paths

Every microservice that touches tenant_class (which is every
microservice, per B.12.2) ships tests covering:
- demo_trial path: a fixture tenant with `tenant_class = demo_trial`
  and the configured caps; tests exercise normal operation, cap-hit
  alerts at 80%, cap-breach state at 100%, grace-window behavior,
  and conversion to paid.
- paid path: a fixture tenant with `tenant_class = paid` and a
  representative billing_components set; tests exercise normal
  operation under uncapped or soft-capped conditions and verify the
  same functional surface as demo_trial.

CI lane `ci-tenant-class-adoption-check` enforces dual-fixture
coverage.

### D.9 demo_trial → paid conversion flow

The conversion flow is a single atomic transaction in cloud-billing:
1. Validate the contract record (signed by both parties; legal
   terms include the billing_components and per-component contract
   terms).
2. Run a settlement step on any demo_trial-period usage that the
   contract specifies as billable (rare; default is "trial usage is
   free").
3. Atomically update the tenant row: set `tenant_class = paid`,
   set `billing_components` from the contract, write the audit-chain
   `tenant_class_transition_event`.
4. Publish the `tenant-class-mutated` event on the event substrate.
5. Notify the tenant of successful conversion via the notifications
   microservice.
6. Refresh all active principal tokens belonging to the tenant
   (force token refresh within 30 seconds).

The conversion is idempotent: a second conversion attempt against
an already-paid tenant is a no-op with a successful response.

### D.10 Cap-breach detection and grace-period flow

The grace-period state machine for demo_trial tenants:
1. Polling: cloud-billing polls per-microservice usage meters every
   5 minutes for demo_trial tenants.
2. 80% threshold: emit a warning notification; record an audit-chain
   `cap_breach_warning_event`.
3. 100% threshold: emit a cap-breach notification; update the
   `principal.cap_breached` claim to true within 30 seconds (via
   token refresh trigger); enter grace state.
4. Grace state: 7 days by default. During grace, read paths remain
   open; write paths are denied via Cedar policy template 4.
5. Conversion during grace: cap_breached flag clears on successful
   conversion; tokens refresh.
6. Grace expiry without conversion: suspend the tenant; data is
   retained per the data-retention policy (default 90 days post-
   suspension); read access is denied; conversion remains possible
   during the retention window.
7. Retention expiry without conversion: data is purged per the
   tenant data-purge policy (per ADR-0008 data-use-boundary).

### D.11 Clawback / chargeback handling for revenue_share

The clawback flow:
1. A downstream consumer initiates a refund or chargeback on a
   transaction routed through oyatie's surfaces.
2. The payment processor (PSP) emits a refund-event or chargeback-
   event to cloud-billing.
3. cloud-billing records a `revenue_event_reversal` entry tied to
   the original revenue_event's idempotency key.
4. At the next monthly settlement, cloud-billing nets the reversal
   against the tenant's revenue share. If the net is negative
   (tenant owes oyatie back its previously-paid share), the
   amount is invoiced; if grace timeline applies, it is netted
   against future settlements.
5. The audit-chain records the reversal as a first-class event;
   the original revenue_event remains in the chain for provenance.

### D.12 Monthly settlement payout via payments microservice

The monthly settlement engine in cloud-billing:
1. At month-end (configurable per tenant; default last calendar day
   of the month, UTC), gather all revenue_events for the tenant in
   the settlement window.
2. Apply contract terms: commission rate per category, currency
   conversion at transaction-time FX rate, applicable taxes per
   cloud-billing-tax.
3. Compute oyatie's share, tenant's share, withheld taxes, FX
   adjustments, clawback nettings.
4. Generate the settlement statement (PDF + JSON via the document-
   generation microservice).
5. Emit `settlement_statement_event` to the payments microservice
   with direction (`oyatie_pays` or `oyatie_collects`) and
   destination payment method.
6. payments microservice executes the money movement; emits
   `payout_completed_event` or `invoice_issued_event` back to
   cloud-billing.
7. cloud-billing closes the settlement window; emits `settlement_
   closed_event` to the audit-chain.

### D.13 BNF for tenant_class + billing_components

```
tenant_class ::= "demo_trial" | "paid"

billing_components ::= empty_set | nonempty_set

empty_set ::= "{}"

nonempty_set ::= "{" component_list "}"

component_list ::= component
                |  component "," component_list

component ::= "revenue_share"
            | "per_seat"
            | "per_usage"

# Subset cardinality constraint:
# - When tenant_class = "demo_trial", billing_components must be empty_set
# - When tenant_class = "paid", billing_components is a subset of
#   { revenue_share, per_seat, per_usage } with 0 ≤ cardinality ≤ 3
# - Each component appears at most once in the set
```

### D.14 JSON schema for the tenant record

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "/specs/tenant-model.json",
  "type": "object",
  "required": ["tenant_id", "tenant_class", "billing_components", "created_at"],
  "properties": {
    "tenant_id": {
      "type": "string",
      "pattern": "^(demo_)?[a-z0-9][a-z0-9-]{2,62}[a-z0-9]$"
    },
    "tenant_class": {
      "type": "string",
      "enum": ["demo_trial", "paid"]
    },
    "billing_components": {
      "type": "array",
      "uniqueItems": true,
      "items": {
        "type": "string",
        "enum": ["revenue_share", "per_seat", "per_usage"]
      },
      "minItems": 0,
      "maxItems": 3
    },
    "contract_id": {
      "type": ["string", "null"]
    },
    "deployment_context": {
      "type": "string",
      "enum": [
        "oyatie-public-cloud",
        "guest-on-aws",
        "guest-on-oci",
        "on-prem",
        "colo",
        "oyatie-as-cloud-provider"
      ]
    },
    "compliance_packs": {
      "type": "array",
      "items": { "type": "string" }
    },
    "byok_modes": {
      "type": "object",
      "additionalProperties": {
        "type": "string",
        "enum": ["platform_default", "byok", "byok_required_by_pack"]
      }
    },
    "created_at": { "type": "string", "format": "date-time" },
    "tenant_class_changed_at": { "type": "string", "format": "date-time" },
    "trial_expires_at": { "type": ["string", "null"], "format": "date-time" },
    "cap_breached": { "type": "boolean" },
    "grace_window_expires_at": { "type": ["string", "null"], "format": "date-time" }
  },
  "allOf": [
    {
      "if": { "properties": { "tenant_class": { "const": "demo_trial" } } },
      "then": {
        "properties": {
          "billing_components": { "maxItems": 0 },
          "compliance_packs": { "maxItems": 0 }
        },
        "required": ["trial_expires_at"]
      }
    },
    {
      "if": { "properties": { "tenant_class": { "const": "paid" } } },
      "then": {
        "required": ["contract_id"]
      }
    }
  ]
}
```

### D.15 Invoice event shape (per_seat)

```json
{
  "event_type": "invoice_issued",
  "event_version": "1.0",
  "event_id": "evt_01HZ...",
  "tenant_id": "acme",
  "tenant_class": "paid",
  "billing_component": "per_seat",
  "invoice_id": "inv_2026_05",
  "period_start": "2026-05-01T00:00:00Z",
  "period_end": "2026-05-31T23:59:59Z",
  "currency": "USD",
  "seat_count_average": 312.4,
  "seat_count_peak": 318,
  "price_per_seat_per_month": "25.00",
  "subtotal": "7810.00",
  "tax_amount": "624.80",
  "total_amount": "8434.80",
  "due_at": "2026-06-15T23:59:59Z",
  "audit_chain_hash": "sha256:..."
}
```

### D.16 Invoice event shape (per_usage)

```json
{
  "event_type": "invoice_issued",
  "event_version": "1.0",
  "event_id": "evt_01HZ...",
  "tenant_id": "acme",
  "tenant_class": "paid",
  "billing_component": "per_usage",
  "invoice_id": "inv_2026_05_usage",
  "period_start": "2026-05-01T00:00:00Z",
  "period_end": "2026-05-31T23:59:59Z",
  "currency": "USD",
  "line_items": [
    {
      "meter_unit": "llm_tokens_input",
      "pricing_dimension": "claude-opus-4-7",
      "quantity": 12500000,
      "unit_price": "0.000015",
      "amount": "187.50"
    },
    {
      "meter_unit": "llm_tokens_output",
      "pricing_dimension": "claude-opus-4-7",
      "quantity": 4200000,
      "unit_price": "0.000075",
      "amount": "315.00"
    },
    {
      "meter_unit": "workflow_executions",
      "pricing_dimension": "default",
      "quantity": 8412,
      "unit_price": "0.005",
      "amount": "42.06"
    },
    {
      "meter_unit": "gb_stored",
      "pricing_dimension": "object_storage",
      "quantity": 218.4,
      "unit_price": "0.022",
      "amount": "4.80"
    }
  ],
  "subtotal": "549.36",
  "tax_amount": "43.95",
  "total_amount": "593.31",
  "due_at": "2026-06-15T23:59:59Z",
  "audit_chain_hash": "sha256:..."
}
```

### D.17 Settlement event shape (revenue_share)

```json
{
  "event_type": "settlement_statement_issued",
  "event_version": "1.0",
  "event_id": "evt_01HZ...",
  "tenant_id": "indie_plugins",
  "tenant_class": "paid",
  "billing_component": "revenue_share",
  "statement_id": "stmt_2026_05",
  "period_start": "2026-05-01T00:00:00Z",
  "period_end": "2026-05-31T23:59:59Z",
  "currency": "USD",
  "direction": "oyatie_pays",
  "gross_revenue_events_count": 1284,
  "gross_revenue_amount": "18420.50",
  "clawback_amount": "320.00",
  "net_revenue_amount": "18100.50",
  "category_breakdown": [
    {
      "category": "marketplace_plugin_sale",
      "commission_rate": "0.20",
      "gross": "16200.00",
      "oyatie_share": "3240.00",
      "tenant_share": "12960.00"
    },
    {
      "category": "marketplace_workflow_sale",
      "commission_rate": "0.15",
      "gross": "2220.50",
      "oyatie_share": "333.08",
      "tenant_share": "1887.42"
    }
  ],
  "tenant_share_total": "14847.42",
  "fx_adjustments": "0.00",
  "tax_withholding": "0.00",
  "net_payout_amount": "14847.42",
  "payout_method": "ach",
  "scheduled_payout_at": "2026-06-10T00:00:00Z",
  "audit_chain_hash": "sha256:..."
}
```

### D.18 Cap-breach event shape

```json
{
  "event_type": "cap_breach",
  "event_version": "1.0",
  "event_id": "evt_01HZ...",
  "tenant_id": "demo_acme",
  "tenant_class": "demo_trial",
  "microservice": "oya-workflow-engine",
  "cap_name": "workflow_count",
  "cap_ceiling": 50,
  "current_value": 50,
  "breach_at": "2026-05-21T14:32:00Z",
  "grace_window_start": "2026-05-21T14:32:00Z",
  "grace_window_end": "2026-05-28T14:32:00Z",
  "upgrade_cta_url": "https://oyatie.com/upgrade/demo_acme",
  "audit_chain_hash": "sha256:..."
}
```

### D.19 Tenant-class-transition event shape

```json
{
  "event_type": "tenant_class_transition",
  "event_version": "1.0",
  "event_id": "evt_01HZ...",
  "tenant_id": "acme",
  "old_tenant_class": "demo_trial",
  "new_tenant_class": "paid",
  "transition_at": "2026-05-21T15:45:00Z",
  "initial_billing_components": ["per_seat", "per_usage"],
  "contract_id": "ctr_2026_acme_001",
  "actor_principal_id": "usr_admin_001",
  "audit_chain_hash": "sha256:..."
}
```

### D.20 Billing-components-mutation event shape

```json
{
  "event_type": "billing_components_mutation",
  "event_version": "1.0",
  "event_id": "evt_01HZ...",
  "tenant_id": "acme",
  "tenant_class": "paid",
  "old_billing_components": ["per_seat"],
  "new_billing_components": ["per_seat", "per_usage"],
  "added": ["per_usage"],
  "removed": [],
  "mutation_at": "2026-06-15T10:00:00Z",
  "contract_amendment_id": "amd_2026_acme_001",
  "actor_principal_id": "usr_admin_001",
  "pre_mutation_settlement_required": false,
  "audit_chain_hash": "sha256:..."
}
```

### D.21 Cedar fragment shipping bundle

The policy-engine microservice ships a Cedar fragment bundle named
`tenant-class.cedar` containing the 6 templates in B.11. The bundle
is versioned (v1.0 as of this ADR's acceptance); microservices
reference the bundle by version in their fragment composition spec.

```cedar
// tenant-class.cedar v1.0

@id("oya-tc-001")
forbid (
  principal,
  action == Action::"activate_compliance_pack",
  resource
) when {
  principal.tenant_class == "demo_trial"
};

@id("oya-tc-002")
forbid (
  principal,
  action == Action::"configure_byok_provider",
  resource
) when {
  principal.tenant_class == "demo_trial"
};

@id("oya-tc-003")
forbid (
  principal,
  action == Action::"publish_marketplace_listing",
  resource
) when {
  principal.tenant_class == "demo_trial"
};

@id("oya-tc-004")
forbid (
  principal,
  action in [
    Action::"create",
    Action::"update",
    Action::"delete"
  ],
  resource
) when {
  principal.tenant_class == "demo_trial" &&
  principal.cap_breached == true
};

@id("oya-tc-005")
forbid (
  principal,
  action == Action::"change_tenant_class",
  resource
) when {
  principal.tenant_class == "paid" &&
  context.target_tenant_class == "demo_trial"
};

@id("oya-tc-006")
forbid (
  principal,
  action == Action::"settle_rev_share",
  resource
) when {
  !(
    principal.tenant_class == "paid" &&
    "revenue_share" in principal.billing_components
  )
};
```

### D.22 Rust enum sketch

```rust
// crates/oya-cloud-billing-domain/src/tenant_class.rs

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantClass {
    DemoTrial,
    Paid,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingComponent {
    RevenueShare,
    PerSeat,
    PerUsage,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BillingComponentSet(HashSet<BillingComponent>);

impl BillingComponentSet {
    pub fn empty() -> Self { Self(HashSet::new()) }
    pub fn contains(&self, c: &BillingComponent) -> bool { self.0.contains(c) }
    pub fn insert(&mut self, c: BillingComponent) -> bool { self.0.insert(c) }
    pub fn remove(&mut self, c: &BillingComponent) -> bool { self.0.remove(c) }
    pub fn len(&self) -> usize { self.0.len() }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Tenant {
    pub tenant_id: String,
    pub tenant_class: TenantClass,
    pub billing_components: BillingComponentSet,
    pub contract_id: Option<String>,
    pub trial_expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub cap_breached: bool,
    pub grace_window_expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Tenant {
    pub fn require_paid(&self) -> Result<(), Forbidden> {
        match self.tenant_class {
            TenantClass::Paid => Ok(()),
            TenantClass::DemoTrial => Err(Forbidden::DemoTrialDenied),
        }
    }

    pub fn requires_revenue_share(&self) -> bool {
        self.tenant_class == TenantClass::Paid
            && self.billing_components.contains(&BillingComponent::RevenueShare)
    }
}
```

### D.23 Migration footprint

The migration footprint applied during ADR-0329's retirement wave:
- Every tenant row in cloud-billing's tenant table is backfilled
  with a tenant_class value: tenants whose previous tier was Bronze
  become demo_trial; tenants whose previous tier was Silver, Gold,
  or Platinum become paid; billing_components are seeded from the
  tenant's pre-existing billing contract per the migration mapping
  table in ADR-0329 §D.
- The migration is reversible only via re-running the inverse map
  (paid → tier inferred from prior contract); the inverse is
  documented as a one-time emergency rollback path, not a steady-
  state operation.

### D.24 Demo_trial conversion UX flow

The conversion UX, surfaced via the tenant-admin console:
1. Tenant admin clicks "Upgrade to Paid" on the workspace banner.
2. UX walks the admin through:
   a. Choosing the deployment context (default oyatie-public-cloud).
   b. Choosing the billing_components set (multi-select).
   c. Per-component contract terms input (commission category for
      revenue_share, seat count and price for per_seat, meter price
      table for per_usage).
   d. Reviewing and accepting the contract terms.
   e. Stored payment method or billing-arrangement setup.
3. Backend executes the conversion transaction in D.9.
4. UX confirms success with a "Welcome to Paid" page; surfaces the
   new caps (or no-caps), the new SLO commitment, the new
   compliance pack and BYOK availability.

### D.25 Deployment context interaction

Each deployment context interacts with tenant_class as follows:
- oyatie-public-cloud: both classes welcome; demo_trial default;
  paid scales up.
- guest-on-aws: paid only (no demo_trial on AWS to avoid AWS-side
  trial-cost ambiguity; AWS Free Tier is 12-month-limited, not
  Always Free, so demo_trial on AWS is not cost-zero sustainable).
- guest-on-oci: both classes welcome; demo_trial defaults to OCI
  Always Free module; paid can use Always Free for sandbox /dev
  sub-tenancies plus paid modules for production.
- on-prem: paid only (on-prem deployments are contractual by
  nature).
- colo: paid only (same reasoning).
- oyatie-as-cloud-provider: both classes welcome; demo_trial runs
  on a reserved free-tier capacity pool; paid scales out across
  the available capacity.

### D.26 Cross-µservice event subscriptions

The microservices that subscribe to cloud-billing's tenant-class
events:
- cloud-iam (refresh principal claims on tenant-class-mutated).
- identity (update user-profile UX state).
- audit-chain (write tenant-class-transition entries).
- cloud-billing-tax (recompute applicable tax rules).
- finops-portal (refresh dashboards).
- payments (initiate payouts on settlement events).
- notifications (send conversion / cap-breach CTAs).
- workflow-engine (refresh per-workflow caps for demo_trial).
- intelligence-inference (refresh per-tenant model-access matrix).
- marketplace (refresh listing eligibility).

Each subscription is documented in the subscribing microservice's
PRD and tested per the canonical event-substrate test harness.

## E. Verification

### E.1 ci-tenant-class-adoption-check (BLOCKER lane)

The `ci-tenant-class-adoption-check` lane enforces per-microservice
adoption. The lane runs against every microservice and verifies:
- The microservice's Rust code imports `TenantClass` and
  `BillingComponentSet` from the cloud-billing-domain crate (no
  local re-definition).
- The microservice's PRD declares its tenant_class behavior
  (cap definitions for demo_trial, meter definitions for per_usage).
- The microservice's tests include both a demo_trial fixture and a
  paid fixture.
- The microservice's Cedar fragments compose canonical fragments
  from the tenant-class.cedar bundle when tenant-class gating
  applies (no inline tenant_class string comparison).
- The microservice's audit-chain emissions include `tenant_class`
  on every audited row.
- The microservice's observability emissions include `tenant_class`
  as a label on relevant metrics (no stratified SLO targets, but
  labels for analytics).

The lane becomes BLOCKER on the dev branch once ADR-0331 lands.
Until then, the lane runs in advisory mode and emits warnings.

### E.2 Quality bar parity lane

The `oya-governance-paid-quality-bar-parity` lane verifies that no
microservice stratifies functional surface by tenant_class. The lane
inspects the microservice's API surface, comparing the response
shape for demo_trial fixtures vs paid fixtures and rejecting any
divergence in feature surface (cap-hit messages are exempt; capability
absence is not).

### E.3 Enum closure lanes

Three CI lanes enforce the enum closures:
- `oya-governance-tenant-class-enum-closed`: rejects any
  microservice that introduces a tenant_class value outside
  {demo_trial, paid}.
- `oya-governance-billing-components-subset-closed`: rejects any
  microservice that introduces a billing component outside
  {revenue_share, per_seat, per_usage}.
- `oya-governance-cloud-billing-source-of-truth`: rejects any
  microservice that mutates tenant_class or billing_components
  outside cloud-billing's mutate API.

### E.4 Cedar attribute coverage lane

The `oya-governance-cedar-tenant-class-attribute-coverage` lane runs
against the policy-engine fragment catalog and verifies that every
fragment that references tenant_class uses the canonical attribute
name `principal.tenant_class` (no aliases) and that the canonical
fragment bundle is referenced rather than re-implemented.

### E.5 Audit-chain transition lane

The `oya-governance-audit-chain-tenant-class-transition` lane runs
against audit-chain's event schema registry and verifies the four
new event types are registered: `tenant_class_transition_event`,
`billing_components_mutation_event`, `cap_breach_event`,
`grace_window_expiry_event`.

### E.6 Demo-trial cap enforcement lane

The `oya-governance-demo-trial-cap-enforcement` lane runs against
each microservice that declares a cap and verifies:
- The cap is enforced at the API gateway via Cedar policy template
  4 plus the microservice's own cap-check.
- The cap value is configurable per-tenant within the contractual
  range.
- The cap-breach event is emitted to cloud-billing within 60 seconds
  of cap reaching 100%.
- The 80% warning event is emitted at the 80% threshold.

### E.7 IAM principal claim lane

The `oya-governance-iam-principal-tenant-class-claim` lane runs
against cloud-iam's principal-issuance contract and verifies that
every principal token carries the four claims: `tenant_id`,
`tenant_class`, `billing_components`, `cap_breached`.

### E.8 Manual verification (Wave 15J completion)

Wave 15J completion criteria require manual verification of:
- ADR-0329 retirement deliverables completed (registry deleted;
  tier-matrix files deleted; tier-deltas docs retracted; ADR-0316
  marked Superseded; BNF v4 amended; brief-template scrubbed).
- ADR-0331 per-microservice adoption applied to all 60+ microservices.
- One end-to-end demo_trial → paid conversion executed in a staging
  environment with full audit-chain trace.

## F. Rollback

### F.1 Composability rollback (low cost)

The composability rollback path applies to per-tenant billing-
components mutations: a tenant who added per_usage and wants to
remove it requests the mutation; cloud-billing executes a clean
settlement on the per_usage component for any open period; the
component is removed; the contract amendment is recorded.

This rollback is part of normal operations, not an exceptional event.

### F.2 Tenant-class conversion rollback (prohibited)

A paid tenant cannot demote to demo_trial. Demotion would erase
billing history, audit-chain provenance, compliance posture, and
committed seat counts in a way that is operationally incoherent.
The legitimate paths for a paid tenant who no longer wants paid
service are:
- Contract churn: tenant terminates the contract; cloud-billing
  closes out the final invoice / settlement; tenant becomes a
  former tenant with retained read-only access for the contractual
  data-retention window; eventual data purge.
- Contract reduction: tenant removes billing_components to lower
  the cost (for example, remove per_usage to stop metered consumption,
  retaining per_seat for the team). The tenant remains paid.
- Demo creation: tenant creates a separate demo_trial tenant
  alongside the paid one (different tenant_id, separate data store);
  the paid tenant continues unchanged.

### F.3 Component mutation rollback (transactional)

A billing_components mutation that is in flight (between
amendment-signed and component-active) may be rolled back by the
contract amendment counterparty before the activation moment. After
activation, rollback follows the F.1 path.

### F.4 ADR rollback (catastrophic, requires Superseded ADR)

If the binary tenant_class model is determined to be wrong (which
would require user directive reversal), this ADR is marked
Superseded by a successor ADR. The successor ADR must specify the
new tenant-class model, the migration map from {demo_trial, paid}
into the new model, and the per-microservice plumbing impact.

Such a rollback is considered extraordinary; the user directive
that originated this ADR was explicit and sequential, and the
keystone bundle has been written to amplify the binary model.

## G. Open questions

### G.1 revenue_share commission rates by category

The per-marketplace-category commission rates are not fixed in this
ADR. They are deferred to per-marketplace-category ADRs (one per
category in ADR-0249's six-category split: plugins, apps, workflows,
agents, models, datasets). Default ranges for negotiating the
per-category ADRs:
- Plugins: 15–25% (Apple App Store anchor)
- Apps: 15–25% (App Store / Play Store anchor)
- Workflows: 10–20% (n8n / Zapier anchor)
- Agents: 20–30% (premium for execution complexity)
- Models: 20–30% (Hugging Face anchor)
- Datasets: 10–20% (Snowflake Marketplace anchor)

These ranges are anchors, not commitments. Each per-category ADR
finalizes the canonical rate.

### G.2 FX accounting for rev-share

The FX accounting question: should rev-share be settled at
transaction-time FX rate, end-of-month FX rate, or contractually-
agreed rate? Default per B.5.8 is transaction-time FX with a
settlement-FX-adjustment line item for FX delta. Alternative
contractual mechanisms (hedged rate, fixed-rate quarterly true-up,
stable-coin settlement) are open per-tenant negotiation but require
audit-chain integration to remain canonical.

### G.3 Multi-tenant downgrade prohibition rationale

The downgrade prohibition in B.1.3 has been challenged in one prior
review: "what if a customer wants to step back to a free trial of
new features?" The answer: that path is to create a separate
demo_trial tenant, not to demote the paid tenant. The prohibition
remains because (a) billing history erasure is not a feature,
(b) compliance posture erasure is not a feature, and (c) the
"feature trial" pattern is better served by sub-tenancy or by
beta-feature flag rather than by tenant-class flipping. If a future
product requirement clarifies a legitimate downgrade case, B.1.3
will be amended via a successor ADR.

### G.4 Negative revenue_share affiliate clarity

The negative revenue_share case (oyatie pays the tenant for
referrals) is functionally distinct from the standard positive
revenue_share (oyatie collects from the tenant). The current model
expresses this as a `direction` field on the settlement event; an
open question is whether the model should split the component into
two named components (`revenue_share_outbound` and `revenue_share_
inbound`) for clarity. Current decision: keep one component with a
direction field; reassess if the affiliate-side volume justifies
the split.

### G.5 Sub-tenant tenant_class inheritance

Whether a sub-tenant of a paid parent must also be paid (vs being
able to be demo_trial for evaluation of a sub-team) is unsettled.
Current default: sub-tenants inherit the parent's tenant_class.
Override: parent-tenant admin may explicitly mark a sub-tenant as
demo_trial for evaluation purposes, with the caveat that the
sub-tenant is subject to demo_trial caps and posture. This is a
non-blocking open question to resolve when sub-tenancy plumbing
matures.

### G.6 Per-microservice cap defaults

The per-microservice cap values for demo_trial tenants are not
fixed in this ADR. They are deferred to ADR-0331 (per-microservice
tenant-class adoption) and to per-microservice PRDs. Default
guidance:
- oya-workflow-engine: 50 workflows, 1000 executions/month
- oya-agentic-agent: 10 agents, 5000 invocations/month
- cloud-iam: 10 seats
- cloud-data-store: 10 GB stored
- oya-messaging-mls: 100 MLS groups, 1000 messages/day
- oya-intelligence-inference: 1M tokens/month total across models

These defaults match the OCI Always Free ceiling so demo_trial
tenants fit within the free infrastructure budget.

### G.7 Conversion from suspension after retention expiry

If a demo_trial tenant exhausts its grace window without conversion
and enters suspension, data is retained for 90 days. The open
question: should a suspended tenant be reactivatable as paid during
the retention window (D.10 step 6) without going through a fresh
sign-up, or should reactivation require a fresh sign-up with the
prior data restored? Current default: reactivation during the
retention window restores the existing tenant_id and data. After
the retention window, a fresh sign-up is required.

### G.8 demo_trial extension policy

Whether a demo_trial tenant who is engaged but not yet ready to
commit can extend the trial window (beyond the default 30 days) is
unsettled. Current default: trial extension is a commercial decision
made by the oyatie sales team; the trial_expires_at field is
mutable by tenant-admin oyatie staff (cloud-billing's
extend_trial API). Extensions are bounded by a per-tenant maximum
(default 180 days from initial trial creation) to avoid open-ended
freeloading.

### G.9 Component cardinality changes mid-month

When a billing_components mutation happens mid-month, how is the
invoice prorated? Current default per D.20: the pre-mutation
component is settled cleanly for the partial month before the
mutation takes effect; the post-mutation component starts a new
billing period at the mutation moment. This produces two partial
invoice entries for the affected month. Alternative (full-month
proration with a single invoice) is operationally simpler but
loses clean settlement boundaries; the current default favors
clean boundaries.

### G.10 Account-level vs tenant-level billing

A single oyatie account that owns multiple tenants (typical
agency / consulting pattern) may want consolidated invoicing
across tenants. The open question: is the consolidated invoice a
feature of cloud-billing or of finops-portal? Current intent: the
invoice issuance remains per-tenant in cloud-billing; finops-portal
provides a consolidated view across tenants for the account.
Account-level payment methods are supported (one card pays for
N tenants).

## H. Cross-references

### H.1 Retired ADR
- ADR-0316 capability-tier-over-product-fragmentation — RETIRED.
  Marked Superseded by this ADR (via ADR-0329 retirement deliverable).

### H.2 Sibling ADRs
- ADR-0329 tier system retirement — sibling decision recording the
  retirement deliverables (registry deletion, tier-matrix file
  deletion, tier-deltas doc retraction, BNF v4 amendment, brief-
  template scrub, etc.).
- ADR-0331 per-microservice tenant-class adoption — sibling decision
  recording the per-microservice plumbing impact (which test paths,
  which Cedar fragments, which event subscriptions, which caps).

### H.3 Foundational ADRs preserved
- ADR-0244 tenant-as-universal-scoping-primitive — tenant_class
  joins tenant_id as a principal claim and as an audited-row column.
- ADR-0243 cedar-as-universal-gate — the runtime gate for
  tenant-class-conditional behavior is a Cedar policy evaluation
  against the principal claim, never inline microservice code.
- ADR-0251 compliance-pack-cell-certification-levels — compliance
  pack activation requires tenant_class = paid (B.3.6).
- ADR-0255 (and amendment) — intelligence-as-two-layer-ai-substrate
  and library-first-network-opt-in-clarification — BYOK opt-in
  requires tenant_class = paid (B.3.7 + B.4.6).
- ADR-0249 multi-category-marketplace-doctrine — marketplace listing
  publication requires tenant_class = paid (B.4.7); marketplace
  consumption by demo_trial is restricted to free listings.
- ADR-0247 self-modification-doctrine — oyatie.foundry.* principals
  operate as a reserved-namespace paid tenant.
- ADR-0242 oyatie-is-a-tenant — extends to "every oyatie principal
  carries a tenant_class".
- ADR-0245 substrate-vs-product-layering — substrate microservices
  serve all tenant classes uniformly.
- ADR-0064 canonical-base-localization — localization clauses
  previously scoped per-tier are rescoped per-tenant-class where
  applicable (most localization is uniform across classes).

### H.4 Sequence ADRs
- ADR-0328 substance-bar-as-canonical-sequence-and-batch-discipline —
  §D-19 amended to reword "OCI Bronze = Always Free" without tier
  language; this ADR is one of the keystone-bundle replacement
  decisions referenced in ADR-0328's wave 1.
- ADR-0322 substance-bar-as-doctrine-and-ci-enforcement — substance-
  bar grade applies to this ADR and to the per-microservice
  adoption work in ADR-0331.
- ADR-0327 wave-3-completion-criteria-and-promotion-gates — Wave 15J
  completion criteria (E.8) feed into the wave-3 promotion gates.

### H.5 Operational ADRs
- ADR-0108 sunset doctrine — ADR-0316's marking as Superseded
  follows ADR-0108 sunset rules.
- ADR-0215 multi-context platform — the six deployment contexts
  referenced in B.4.2 and D.25 are defined here.
- ADR-0216 open integration / no vendor lock-in — uniform capability
  surface across tenant classes preserves the no-lock-in posture.
- ADR-0218 tenant-granular-control — per-tenant deployment-context
  choice and billing_components selection.
- ADR-0008 data-use-boundary — tenant data-retention windows in
  D.10 follow the data-use-boundary doctrine.
- ADR-0130 agentic-SLO-gated promotion — meter event emission
  follows the canonical observability contract.
- ADR-0131 per-microservice flat layout — cloud-billing internal
  module layout in D.1 follows the flat layout doctrine.
- ADR-0132 product-platform-and-bundle-dissolution — confirms no
  per-tier product fragmentation.
- ADR-0145 inter-microservice communication reform — event
  substrate for tenant-class-mutated events follows direct gRPC +
  3 invariants.

### H.6 Memory directive sources
- feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20 —
  primary user directive establishing the binary class + composable
  components model.
- feedback_no_capability_tiers_2026_05_20 — user directive
  establishing the tier-system retirement.
- feedback_oci_always_free_maximization_2026_05_20 — OCI Always
  Free as the default infrastructure for demo_trial tenants.
- feedback_quality_performance_scalability_bar — uniform industry-
  leader quality bar across tenant classes.
- feedback_flat_product_catalog — "everyone is shared"; no
  feature gating between paying customers.
- feedback_drift_too_big_2026_05_20 — tier system as part of the
  drift to reconcile.
- feedback_multi_context_provider_agnostic_2026_05_20 — six
  deployment contexts framed for tenant-class interaction.
- feedback_zero_handroll_opentofu_only_2026_05_20 — OpenTofu
  provisioning for both tenant classes.
- feedback_microservice_ownership_coherence_2026_05_20 — one agent
  owns one microservice end-to-end including tenant-class plumbing.

### H.7 Specs touched
- /specs/tenant-model.json — schema in D.14 lands here.
- /specs/billing/billing-component-schema.json — new spec for the
  per-component contract terms record.
- /specs/microservices/manifest-schema.json — gains a tenant_class
  adoption manifest section per ADR-0331.
- /specs/cedar-fragment-schema.json — the tenant-class.cedar
  bundle in D.21 lands here.
- /specs/master-plan-sequencing.json — wave 15J ledger entry
  citing this ADR.
- /specs/markdown-retirement-policy.json — retirement of
  tier-matrix files cited here.

### H.8 Standards docs touched
- docs/standards/capability-authoring.md — scrubbed of tier
  segments by ADR-0329.
- docs/standards/autonomy-ceiling.md — same.
- docs/standards/hyperscaler-best-practices.md — same.
- docs/standards/observability.md — same.
- docs/standards/on-call.md — same.
- docs/standards/workflow-substrate-engine.md — same.
- docs/standards/asyncapi-3-1-authoring.md — per-tier event grants
  rescoped to per-tenant-class where applicable.
- docs/standards/naming-convention-bnf-v4.md — N-014/N-015 amended
  to drop `.<tier>` segment.
- docs/standards/brief-template.md — §3.X anchor amended to
  tenant-class plus billing-components.
- docs/standards/documentation-rigor.md — this ADR follows §1.1
  bespoke-authoring requirement.

### H.9 Registry artifacts touched
- registry/capability-tiers/ — deleted by ADR-0329.
- registry/tenant-classes/{demo-trial,paid}.json — created by this
  ADR (deliverable in ADR-0329's positive replacement set).
- registry/billing-components/{revenue-share,per-seat,per-usage}.json
  — created by this ADR (deliverable in ADR-0329's positive
  replacement set).

### H.10 Microservice ownership
- cloud-billing — keystone owner of tenant_class and
  billing_components per B.10.
- cloud-iam — principal-claim emitter per D.2.
- identity — user-profile tenant_class surface per D.3.
- audit-chain — transition / mutation event recorder per D.4.
- cloud-billing-tax — per-component tax treatment per D.5.
- payments — settlement money movement per D.6.
- cloud-data-store — tenant_class + billing_components_snapshot on
  audited rows per D.7.
- finops-portal — tenant-facing dashboards per C.11.
- notifications — conversion / cap-breach CTAs per D.10 / D.24.
- workflow-engine, agentic-agent, messaging-mls, intelligence-
  inference, search-index, marketplace — per-microservice plumbing
  per D.8 and per ADR-0331.

---

End of ADR-0330.

<!--
COMPLETION REPORT

owner: sole-owner (this dispatch)
output_file: /Users/jasonlee/oyatie/docs/decisions/ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components.md
line_count_target: ≥800 lines substantive
sections_authored: A (Context, 8 subsections), B (Decision, 104 numbered clauses across 12 subsections), C (Consequences, 12 subsections), D (Implementation footprint, 26 subsections with full BNF + JSON schema + Cedar fragments + Rust enum sketch + 6 event shapes), E (Verification, 8 lanes / subsections), F (Rollback, 4 subsections), G (Open questions, 10 numbered questions), H (Cross-references, 10 subsections covering retired ADRs, sibling ADRs, foundational ADRs, sequence ADRs, operational ADRs, memory directive sources, specs, standards docs, registry artifacts, microservice ownership)
key_primitives_encoded:
  - tenant_class closed enum {demo_trial, paid}
  - billing_components closed subset of {revenue_share, per_seat, per_usage}
  - cloud-billing as source-of-truth (B.10)
  - cloud-iam principal-claim emission (D.2)
  - Cedar fragment bundle tenant-class.cedar v1.0 (D.21 / B.11)
  - audit-chain 4 new event types (C.10 / D.4)
  - uniform quality bar parity (B.9)
  - demo_trial → paid conversion atomic flow (D.9)
  - cap-breach + grace-window state machine (D.10)
  - rev-share monthly settlement + clawback netting (D.11 / D.12)
  - 8 composability examples (B.8)
  - per-deployment-context interaction matrix (D.25)
  - 6 Cedar fragment templates (B.11.2 through B.11.7 + D.21)
  - BNF for tenant_class + billing_components (D.13)
  - JSON schema for tenant record (D.14)
  - Rust enum sketch (D.22)
  - 6 detailed event shapes (D.15-D.20)
cross_references_completed:
  retired: ADR-0316
  siblings: ADR-0329, ADR-0331
  foundational: ADR-0244, ADR-0243, ADR-0251, ADR-0255 (+ amendment), ADR-0249, ADR-0247, ADR-0242, ADR-0245, ADR-0064
  sequence: ADR-0328, ADR-0322, ADR-0327
  operational: ADR-0108, ADR-0215, ADR-0216, ADR-0218, ADR-0008, ADR-0130, ADR-0131, ADR-0132, ADR-0145
  memory: 9 feedback memory files cited
  specs: 6 specs touched
  standards: 11 standards docs touched
  registry: 3 registry families touched
ci_lanes_introduced: 9 (ci-tenant-class-adoption-check + 8 oya-governance-* lanes)
no_commits: confirmed (this dispatch authored only)
no_parallel_writes_outside_target: confirmed
no_scripting: confirmed
no_placeholder: confirmed (every clause is substantive)
-->
