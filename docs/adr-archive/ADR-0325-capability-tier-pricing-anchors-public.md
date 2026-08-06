---
id: ADR-0325
status: Superseded
date: 2026-05-20
owner: council-product
owners:
  - council-product
  - council-architecture
  - council-marketing
  - council-finance
  - council-go-to-market
  - council-engineering
  - axis-billing
  - axis-tenancy
  - axis-policy-engine
  - axis-workflow-engine
  - axis-foundry
  - ops-compliance
supersedes: []
amends:
  - ADR-0249-multi-category-marketplace-doctrine.md (adds per-category public price anchors)
  - ADR-0314-marketplace-as-universal-deal-settlement.md (binds settlement pricing to public tier anchors)
  - ADR-0316-capability-tier-over-product-fragmentation.md (publishes the per-tier price bands declared but not anchored in ADR-0316)
superseded_by: [ADR-0700]
related:
  - ADR-0132
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0245
  - ADR-0249
  - ADR-0251
  - ADR-0252
  - ADR-0253
  - ADR-0254
  - ADR-0255
  - ADR-0263
  - ADR-0313
  - ADR-0314
  - ADR-0315
  - ADR-0316
  - ADR-0317
  - ADR-0321
  - ADR-0322
  - ADR-0326
  - ADR-0327
related_specs:
  - /specs/pricing/tier-anchor-bands.json
  - /specs/pricing/per-category-anchors.json
  - /specs/products/capability-tier-registry.json
  - /specs/tenant-model.json
  - /specs/compliance-pack-schema.json
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/standards/pricing-doctrine.md
  - docs/decisions/ADR-0316-capability-tier-over-product-fragmentation.md
  - docs/marketing/pricing-anchors-public-2026-05-20.md
inbound_citations:
  - docs/decisions/ADR-0316-capability-tier-over-product-fragmentation.md
  - docs/marketing/pricing-anchors-public-2026-05-20.md
doc_class: Architecture-Decision-Record
shape: Decision
authority_tier: 1
purpose: >
  Make the per-tier (Bronze / Silver / Gold / Platinum) pricing bands
  PUBLIC and bind them by name and number to specific marketplace
  categories, so that prospective tenants can self-quote without sales
  contact and so that downstream billing and settlement code can resolve
  the price for any (category, tier, tenant-pack, residency) tuple
  deterministically. ADR-0316 defined the tier shape without anchoring the
  monetary bands; this ADR fills the gap with a named anchor table, a
  named annual escalation policy, a named multi-currency parity rule, a
  named compliance-pack uplift schedule, and a named residency-uplift
  schedule that composes with ADR-0326 per-tenant data residency rules.
enforcement_status: blocker-day-one
enforced_by:
  - oya-governance-pricing-tier-anchor-public
  - oya-governance-pricing-tier-anchor-currency-parity
  - oya-governance-pricing-compliance-pack-uplift
  - oya-governance-pricing-residency-uplift
  - oya-governance-pricing-anchor-drift-monitor
decision_owner: council-product
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


> **Disposition light-edit (2026-08-06):** Context re-triage Accept: Capability tier pricing anchors

# ADR-0325: Capability Tier Pricing Anchors Public

## Status

Proposed (2026-05-20). The anchors below are effective on the publication
date and remain stable for ≥365 days unless the drift-monitor (D-9) flags
a market deviation greater than the documented thresholds, in which case
the anchor revision protocol (D-10) applies.

## Context

### Named pressure

ADR-0316 declared the Bronze / Silver / Gold / Platinum tier shape but
explicitly deferred the monetary anchor question to a successor ADR. In
the intervening period (2026-05-19 to 2026-05-20) the post-keystone work
produced multi-category marketplace registries (per ADR-0249) and the
universal-settlement substrate (per ADR-0314) — neither of which can
operate deterministically without a price anchor that downstream
microservices can consume.

Customer-facing pressure also accumulated: the B2B-leader coverage doctrine
(ADR-0321) names 13 leader-coverage product surfaces, each of which must be
priced in the marketplace catalog. Sales conversations stalled because no
public anchor existed, forcing per-conversation pricing improvisation —
which the GTM council flagged as both a deal-velocity blocker and a
brand-trust concern.

The Stripe-class quality bar (per
`feedback_quality_performance_scalability_bar.md`) demands self-serve
quoting; Stripe's own pricing pages are public, anchored, and trivially
self-quotable. This ADR meets the same bar.

### Named constraints

- **C-1 Marketplace alignment** — per ADR-0249, every marketplace category
  must have a canonical anchor; ADR-0325 anchors all six categories
  (plugin, app, workflow, agent, model, dataset).
- **C-2 Settlement determinism** — per ADR-0314, the settlement substrate
  must resolve a price from a (category, tier, pack, residency) tuple
  without round-tripping to a config service.
- **C-3 BYOK compatibility** — per ADR-0255 §D-4 (BYOK doctrine), tenants
  that supply their own LLM credentials see a downward price adjustment;
  the anchor includes the named BYOK adjustment.
- **C-4 Compliance-pack uplift** — per ADR-0251, compliance packs
  (HIPAA / GDPR / SOC2 / CSAP / PCI / EU-AI-Act) carry uplift; the anchor
  includes the per-pack table.
- **C-5 Residency uplift** — per ADR-0326 (companion ADR), per-tenant data
  residency requirements carry uplift; the anchor schedules residency
  uplift bands.
- **C-6 Multi-currency parity** — per ADR-0242 (oyatie tenant doctrine),
  oyatie itself is a tenant; the canonical-base + pack model from
  `feedback_canonical_base_localization.md` requires that the anchor be
  published in USD as the canonical base and projected into pack-defined
  local currencies via a parity table that updates monthly.
- **C-7 Public-doctrine binding** — the anchors are PUBLIC; ADR-0244
  tenancy doctrine permits a tenant to see only its own data, but the
  anchor itself is published outside any tenant boundary (it is a property
  of the substrate, not of a tenant).

### Named prior incidents

- **Incident I-1 (2026-04-30)**: B2B leader-coverage discovery call stalled
  at the pricing question; sales team improvised a quote inconsistent with
  the eventual anchor; required a follow-up apology and reset.
- **Incident I-2 (2026-05-09)**: marketplace settlement attempt produced
  three different prices for the same (category, tier) tuple because three
  caller microservices each had their own hard-coded anchors; postmortem at
  `docs/postmortems/postmortem-settlement-price-divergence-2026-05-09.md`.
- **Incident I-3 (2026-05-15)**: a CSAP-pack customer in KR requested a
  quote; without a residency uplift table, the quote omitted residency cost
  and was retracted within 48 hours after engineering computed the actual
  cost.

## Decision

The per-tier anchor table is below. All amounts in USD, expressed as
monthly recurring revenue (MRR) per tenant per category. Annual
prepayment carries a 12% discount (D-3); BYOK carries 15% discount on the
LLM-cost component (D-4).

| Tier      | Plugin | App    | Workflow | Agent  | Model  | Dataset |
|-----------|--------|--------|----------|--------|--------|---------|
| Bronze    | $39    | $99    | $149     | $199   | $249   | $79     |
| Silver    | $149   | $399   | $599     | $799   | $999   | $299    |
| Gold      | $599   | $1,499 | $2,499   | $3,499 | $4,499 | $1,199  |
| Platinum  | $2,499 | $5,999 | $9,999   | $14,999| $19,999| $4,999  |

Compliance-pack uplift (multiplicative on the base):

- HIPAA pack: ×1.25
- GDPR pack: ×1.10
- SOC2 pack: ×1.15
- CSAP pack: ×1.30
- PCI pack: ×1.20
- EU-AI-Act pack: ×1.25

Multiple packs compose multiplicatively up to a cap of ×1.85 (the cap
prevents pathological multi-pack stacking from producing a quote that
exceeds the next tier's anchor).

Residency uplift (multiplicative on the post-pack base):

- Default (multi-region): ×1.00
- Single-region (in-region only): ×1.10
- Sovereign cell (per ADR-0246): ×1.30
- Air-gapped tenant cell: ×1.65

These four schedules (base, pack, residency, multi-pack cap) combined with
the BYOK adjustment and the prepayment discount form the deterministic
price resolution function (D-5).

## Consequences

Publishing the per-tier anchor table with multiplicative compliance-pack uplifts and deterministic BYOK and prepayment adjustments means quotes resolve from a single public price function, so pricing becomes auditable and reproducible across tenants; the detailed mechanics, SLO implications, and migration path below enumerate the operational consequences of these public anchors.

## Detailed Mechanics

### D-1 Tier semantics recap

Per ADR-0316:

- **Bronze** — single-team workgroup; ≤25 active users; ≤5 capability
  surfaces enabled; substrate SLOs at the default class.
- **Silver** — single-business-unit; ≤250 active users; ≤25 capability
  surfaces; substrate SLOs at the silver class (higher availability target).
- **Gold** — multi-business-unit; ≤2,500 active users; ≤100 capability
  surfaces; substrate SLOs at the gold class plus sub-second p99 latency.
- **Platinum** — enterprise; unlimited active users; unlimited surfaces;
  substrate SLOs at the platinum class plus dedicated cell allocation per
  ADR-0246 Tier-0/1.

The anchor table above prices each tier accordingly; the steep multiplier
from Bronze to Platinum reflects the substrate-cost differential (Platinum
tenants consume up to 100× substrate resources per active user vs Bronze).

### D-2 Per-category anchor rationale

The per-category MRR anchors reflect the relative substrate cost and
relative customer value of each category:

- **Plugin** — lightest. A plugin is an inert capability extension that
  runs in-process with an existing workflow. Anchor reflects substrate
  cost only.
- **App** — heavier. An app composes plugins, ontology projections, and
  UX shell manifests. Anchor reflects substrate + composition cost.
- **Workflow** — heaviest among the non-AI categories. A workflow runs
  arbitrarily long-running orchestrations, each carrying observability,
  retry, and compensation cost.
- **Agent** — heavier than workflow because agents incur inference cost
  and reviewer-attestation cost on top of workflow substrate.
- **Model** — highest among non-dataset categories because hosting and
  serving models carries GPU/accelerator cost and a baseline ML lifecycle
  cost (per ADR-0308).
- **Dataset** — distinct from plugins because storage, lineage, and
  consent obligations dominate; sits between Plugin and App in price.

The anchors are seated on the W3-G market analysis at
`docs/marketing/market-analysis-w3-g-2026-05-19.md` which surveys 27
comparable B2B SaaS pricing tables.

### D-3 Annual prepayment discount

A tenant that prepays 12 months in advance receives a flat 12% discount on
the post-uplift price. The discount is applied at settlement time by the
`oya-billing-discount-resolver` microservice (per ADR-0314). The 12%
figure is set such that:

- Customer cash-flow cost (12 months early payment) is a wash with a
  ~6% time-value adjustment.
- Oyatie's working-capital benefit covers the remainder.
- Net retention impact (modeled by council-finance) is positive ~3.5%.

The discount is renewable annually but never expands beyond 12% — the
anchor table is the headline price, and no published combination of
discounts can land below the (anchor × 0.88) floor without an exception
per D-10.

### D-4 BYOK adjustment

Per ADR-0255 §D-4 BYOK doctrine, a tenant supplying its own LLM credentials
receives a 15% discount on the LLM-cost component of its price. The
LLM-cost component is itemised per category in the anchor breakdown table
at `/specs/pricing/per-category-anchors.json`. The BYOK discount applies
only when:

- The tenant's `provider_credential_mode == "byok"` or `"byok_required_by_pack"`.
- The BYOK keys have been activated and proven against an actual provider
  call in the trailing 30 days.
- The BYOK discount is not stacked with any other LLM-cost-specific
  discount.

### D-5 Price resolution function

The deterministic price resolution function is named
`resolve_price(category, tier, packs, residency, byok_mode, prepayment) -> Decimal`
and lives in `crates/oya-billing-price-resolver/`. Computation:

1. Look up the anchor `base = ANCHOR[tier][category]`.
2. Compose pack uplifts: `pack_factor = min(prod(pack_uplift[p] for p in packs), 1.85)`.
3. Compose residency uplift: `residency_factor = RESIDENCY[residency]`.
4. Sub-total = `base × pack_factor × residency_factor`.
5. If `byok_mode == "byok" or "byok_required_by_pack"`, apply BYOK
   discount only on the LLM-cost component of `sub-total` (the LLM-cost
   share is itemised per category).
6. If `prepayment == true`, multiply the post-BYOK total by 0.88.
7. Return the final `Decimal` rounded to whole USD cents.

The resolver is the only authoritative source for any quoted price; all
sales-facing tools call it, and all settlement tools call it. There is no
alternative path.

### D-6 Multi-currency parity

The anchors are USD-canonical. Local-currency display is computed by:

- The treasury substrate publishes a daily exchange rate table at
  `/specs/pricing/fx-rates.json`.
- Localisation packs (per the canonical-base-localization doctrine in
  `feedback_canonical_base_localization.md`) declare which currencies their
  tenants see.
- The price resolver applies the FX rate at quote time and pins the rate
  to the quoted contract for the duration of the contract (1 year for
  annual, 1 month for monthly).
- Currency rounding rules per ISO 4217 apply: JPY rounds to whole yen, USD
  rounds to whole cents, etc.

The FX table updates daily; price drift caused by FX is monitored but not
governed by this ADR — it is a treasury-substrate concern.

### D-7 Pack uplift composition

Pack uplifts compose multiplicatively per D-5 step 2, capped at ×1.85.
Rationale for the cap:

- Without a cap, six packs at ×1.30 each would yield ×4.83, exceeding
  the next tier's anchor and breaking tier-pricing semantics.
- ×1.85 was chosen so that even a HIPAA + CSAP + GDPR + PCI + SOC2 +
  EU-AI-Act stack (the maximal six-pack stack) lands within a single tier
  band.
- Compliance-cost recovery is preserved because each individual pack's
  uplift covers its incremental substrate cost; the cap is a customer-
  protection mechanism, not a cost-recovery mechanism.

### D-8 Residency uplift schedule

Per ADR-0326 (companion), residency tiers are:

- `multi_region` (default): the tenant accepts the global default cell
  topology; no uplift.
- `single_region`: tenant's data stays within a named region; ×1.10
  reflects the loss of cross-region capacity sharing.
- `sovereign_cell`: dedicated cell with sovereign substrate per ADR-0246
  Tier-1 / Tier-2; ×1.30 reflects the dedicated cell cost.
- `airgapped_cell`: a fully air-gapped cell with no shared substrate;
  ×1.65 reflects the operational overhead of a dedicated control plane.

Residency choices are declared in the tenant manifest at provision time
and may be upgraded (but not downgraded mid-contract) via the residency
attestation protocol in ADR-0326.

### D-9 Anchor drift monitor

`oya-governance-pricing-anchor-drift-monitor` watches:

- Monthly net-revenue divergence between the anchor and the realised
  median quote (a divergence > 8% flags review).
- Competitive pricing surveys (quarterly market scan) where the anchor
  becomes >15% out of band vs the comparable competitor median.
- Customer churn rate against the price-sensitive segment (segmentation
  per `docs/marketing/segmentation-w3-g-2026-05-19.md`).

A drift detection emits `governance.pricing.anchor.drift.detected` at WARN
severity; council-product reviews and either confirms the anchor or
initiates an anchor revision per D-10.

### D-10 Anchor revision protocol

Anchor revisions:

- Are proposed by council-product with concurrence from council-finance
  and council-go-to-market.
- Land as an amendment to this ADR (a successor ADR is not required for
  routine revisions; a successor ADR is required for tier-shape changes).
- Must respect a ≥90-day notice period for any upward revision and ≤30-day
  notice for any downward revision.
- Trigger an `anchor.revision.scheduled` event on the audit chain followed
  by `anchor.revision.effective` on the effective date.
- May not exceed ±25% per revision per category; larger revisions require
  a tier-shape successor ADR.

In-flight contracts honour the anchor in force at the time of the
contract's signature; new contracts use the revised anchor.

### D-11 Marketplace category onboarding to the anchor

When a new marketplace category is introduced (per ADR-0249), the
category must be added to the anchor table before it can be priced:

- Council-product authors a category-anchor proposal as a Tier-2
  artifact (≥500 lines per ADR-0322) naming the category's substrate-
  cost rationale.
- Council-finance concurs on the per-tier anchor values.
- Council-go-to-market concurs on the public-launch narrative.
- The amendment lands as an edit to this ADR with explicit
  `anchor_table_revision: <new-revision-id>` frontmatter.
- The price-resolver microservice's smoke test is updated to cover
  the new category's 24-tuple anchor set.

The onboarding sequence guarantees no category appears in the
marketplace without a deterministic price.

### D-12 Itemised LLM-cost share per category

The BYOK discount (D-4) applies to the LLM-cost component of the
price. The itemised share by category (the LLM-cost share is the
fraction of the per-tier MRR that is attributable to LLM inference
and accelerator cost):

| Category   | LLM-cost share | Rationale                                  |
|------------|----------------|--------------------------------------------|
| Plugin     | 5%             | Plugins rarely invoke LLMs directly         |
| App        | 20%            | Apps compose plugins; some LLM-mediated UX  |
| Workflow   | 35%            | Workflows often orchestrate LLM steps       |
| Agent      | 65%            | Agents are LLM-centric                      |
| Model      | 80%            | Model-hosting is dominated by inference     |
| Dataset    | 10%            | Datasets carry storage/lineage; minimal LLM |

A BYOK tenant on the `Agent` category at the Silver tier
($799 MRR) sees a discount of `$799 × 0.65 × 0.15 = $77.90`. The
discount is itemised on the invoice per the universal-settlement
substrate (ADR-0314) requirement for line-item transparency.

### D-13 Quote validity and snapshot

Every quote emitted by the price-resolver is valid for 30 days
from emission and carries:

- The anchor revision ID active at emission time.
- The compliance-pack composition at emission time.
- The residency tier at emission time.
- The BYOK eligibility flag at emission time.
- The FX rate (where non-USD).
- The quote ID (UUID v7).

A signed contract within the validity window pins the quote's values
for the contract duration. Re-quoting after validity expiry uses
current anchor and FX values.

### D-14 Mid-contract pack or residency changes

A tenant that activates a new pack or upgrades residency mid-contract
sees:

- A prorated price adjustment from the activation date to the next
  renewal.
- The adjustment computed by the price-resolver using the active
  anchor revision.
- Per ADR-0326, residency upgrades are permitted mid-contract;
  downgrades are not.
- An audit event `pricing.mid_contract.adjustment.applied`.

Pack deactivation mid-contract carries no refund unless the contract
explicitly allows it; the council-finance default policy is that
pack deactivation refunds only the unused service window for the
deactivated pack.

## Cedar Policy Hooks

```cedar
// Fragment: cedar/pricing/anchor-table-read-public.cedar
permit (
  principal,
  action == Pricing::"read_anchor",
  resource is PriceAnchor
);
// Public read; the price anchors are public per the ADR's headline decision.
```

```cedar
// Fragment: cedar/pricing/price-resolver-may-quote.cedar
permit (
  principal == Service::"oyatie.billing.price_resolver",
  action == Pricing::"resolve",
  resource is QuoteRequest
) when {
  context.category in ["plugin", "app", "workflow", "agent", "model", "dataset"] &&
  context.tier in ["bronze", "silver", "gold", "platinum"] &&
  context.residency in ["multi_region", "single_region", "sovereign_cell", "airgapped_cell"]
};
```

```cedar
// Fragment: cedar/pricing/anchor-revision-requires-council.cedar
forbid (
  principal,
  action == Pricing::"revise_anchor",
  resource is PriceAnchor
) when {
  context.council_product_concurrence == false ||
  context.council_finance_concurrence == false ||
  context.council_gtm_concurrence == false
};
```

```cedar
// Fragment: cedar/pricing/byok-discount-eligibility.cedar
permit (
  principal == Service::"oyatie.billing.price_resolver",
  action == Discount::"apply_byok",
  resource is Quote
) when {
  context.tenant.provider_credential_mode in ["byok", "byok_required_by_pack"] &&
  context.tenant.byok_keys_activated_within_30d == true &&
  context.no_other_llm_specific_discount == true
};
```

```cedar
// Fragment: cedar/pricing/prepayment-floor.cedar
forbid (
  principal == Service::"oyatie.billing.price_resolver",
  action == Pricing::"emit_quote",
  resource is Quote
) when {
  context.quoted_amount < context.anchor_amount * 0.88
};
```

## Audit Event Classes Emitted

| Class                                                | Severity | Source crate                                  |
|------------------------------------------------------|----------|-----------------------------------------------|
| pricing.anchor.read                                  | INFO     | oya-billing-price-resolver                    |
| pricing.anchor.resolved                              | INFO     | oya-billing-price-resolver                    |
| pricing.anchor.drift.detected                        | WARN     | oya-governance-pricing-anchor-drift-monitor   |
| pricing.anchor.revision.scheduled                    | INFO     | oya-governance-pricing-tier-anchor-public     |
| pricing.anchor.revision.effective                    | INFO     | oya-governance-pricing-tier-anchor-public     |
| pricing.pack_uplift.capped                           | INFO     | oya-billing-price-resolver                    |
| pricing.residency_uplift.applied                     | INFO     | oya-billing-price-resolver                    |
| pricing.byok_discount.applied                        | INFO     | oya-billing-price-resolver                    |
| pricing.byok_discount.rejected                       | INFO     | oya-billing-price-resolver                    |
| pricing.prepayment_floor.breach_blocked              | BLOCKER  | oya-billing-price-resolver                    |
| pricing.currency_parity.fx_rate_pinned               | INFO     | oya-billing-price-resolver                    |
| pricing.anchor.public_change.notice_served           | INFO     | oya-governance-pricing-tier-anchor-public     |

Each class carries tenant context per ADR-0244 and authority-chain
attestation per ADR-0246; anchor public-read events are tenant-agnostic
(scoped to `oyatie.public`).

## SLO Implications

`microservices/billing/price-resolver/slos/price-resolver.openslo.yaml`:

- `price_resolution_p99_latency`: ≤ 50 ms (settlement path is hot).
- `price_resolution_availability`: ≥ 99.99% (settlement is critical path).
- `anchor_table_correctness`: 100% (smoke-test injects known tuples and
  verifies expected resolutions every 5 minutes).
- `anchor_drift_monitor_freshness`: anchor drift is recomputed at most 24
  hours stale.
- `byok_discount_application_correctness`: ≥ 99.95% (auditing against
  expected discount).

## Migration Path / Phased Rollout

- **Phase 0 (T-0, ADR Proposed)**: anchors are PUBLIC the day of
  publication; the price-resolver microservice begins shadow-mode quoting
  against current production pricing.
- **Phase 1 (T+14 days)**: price-resolver becomes the authoritative quote
  source for all new contracts.
- **Phase 2 (T+30 days)**: existing contracts begin migration to anchored
  prices at renewal (no mid-contract change for existing customers; new
  pricing applies at next renewal).
- **Phase 3 (T+60 days)**: drift monitor active in WARN; alerts council-
  product weekly.
- **Phase 4 (T+90 days)**: first scheduled anchor review per the annual
  cadence; ADR eligible for promotion per ADR-0327.

## Failure Modes + Recovery

### F-1: Resolver outage during quote

The price resolver is unavailable during a quote attempt. Recovery: the
quote path returns a structured `pricing_unavailable` error with the
anchor URL embedded; the sales agent can fall back to public anchor
display and a 24-hour follow-up quote.

### F-2: FX rate desynchronised

The treasury substrate's FX table is stale. Recovery: the price resolver
refuses to emit a localised quote until the FX table is no more than 48
hours stale; falls back to USD quoting in the interim.

### F-3: Drift monitor false positive

The drift monitor flags a divergence that turns out to be a one-time
discount campaign rather than a structural shift. Recovery: council-
product files a `drift.false_positive.acknowledged` event with the
campaign reference; the drift monitor's baseline excludes the campaign
window.

### F-4: Customer attempts to stack discounts

A customer's contract proposal attempts to stack BYOK + prepayment +
manual sales discount below the anchor × 0.88 floor. Recovery: the
prepayment floor Cedar fragment BLOCKs the quote; the sales path returns
the floor as the minimum quotable amount; exceptions require council-
product + council-finance concurrence per D-10.

### F-5: Pack uplift cap collision

A tenant with seven enabled packs encounters the ×1.85 cap and the cap
ceiling collides with a pack's individual uplift. Recovery: the resolver
applies the cap and emits `pricing.pack_uplift.capped` event; the
council-finance reviews the case quarterly to determine whether the cap
needs revision.

### F-6: Anchor publication corruption

An anchor publication PR introduces a typo (e.g. $399 → $39). Recovery:
the substance-bar lane (per ADR-0322) requires bespoke content; the
no-template-stamping detector catches mass copy-paste errors; a final
human review by council-product + council-finance is mandatory for any
anchor publication PR.

## Verification

Named CI checks:

- `oya-governance-pricing-tier-anchor-public`
- `oya-governance-pricing-tier-anchor-currency-parity`
- `oya-governance-pricing-compliance-pack-uplift`
- `oya-governance-pricing-residency-uplift`
- `oya-governance-pricing-anchor-drift-monitor`

Named crates:

- `oya-governance-pricing-tier-anchor-public`
- `oya-governance-pricing-tier-anchor-currency-parity`
- `oya-governance-pricing-compliance-pack-uplift`
- `oya-governance-pricing-residency-uplift`
- `oya-governance-pricing-anchor-drift-monitor`
- `oya-billing-price-resolver` (settlement-time consumer)

Verification fixtures live at `tests/billing/price-resolver/` and include
the 24-tuple anchor smoke test (6 categories × 4 tiers), pack-stacking
scenarios up to the cap, residency uplift transitions, BYOK eligibility
boundary tests, and drift-monitor synthetic-divergence triggers.

## Cross-References

### Other ADRs

- ADR-0132 (suite dissolution) — capability-tier-anchored pricing replaces
  per-suite pricing.
- ADR-0242 (oyatie tenant) — public anchor publishing scope.
- ADR-0243 (Cedar universal gate) — Cedar fragment convention.
- ADR-0244 (tenant scoping) — tenant context on audit events.
- ADR-0245 (substrate-product layering) — anchor is a substrate concern.
- ADR-0249 (multi-category marketplace) — six categories anchored here.
- ADR-0251 (compliance packs) — pack uplift table.
- ADR-0252 (HLC default) — quote timestamps under HLC.
- ADR-0253 (HTTP/3 default) — quote API transport.
- ADR-0254 (K8s + Cloud Hypervisor) — substrate cost factors.
- ADR-0255 (intelligence two-layer) — BYOK basis.
- ADR-0263 (audit-event registry) — class registration.
- ADR-0313 (conglomerate tenant hierarchy) — multi-tenant pricing roll-up.
- ADR-0314 (marketplace settlement) — settlement-resolver consumer.
- ADR-0315 (ERP coverage) — ERP-related categories anchored.
- ADR-0316 (capability tier) — tier shape pre-requisite.
- ADR-0317 (role-based projection) — projection-bundle pricing absorbs into
  capability surfaces.
- ADR-0321 (B2B leader coverage) — leader-coverage surfaces priced under
  this anchor.
- ADR-0322 (substance bar) — anchor publication artifacts subject.
- ADR-0326 (per-tenant residency) — residency uplift binding.
- ADR-0327 (wave-3 completion) — promotion gates consume.

### Standards

- `docs/standards/pricing-doctrine.md` (to be authored as a Tier-2 companion
  document in the W2 wave; pre-existing pricing notes consolidated).
- `docs/standards/documentation-rigor.md` §3.2 density schedule.

### Microservices

- `microservices/billing/price-resolver/` — settlement-time consumer.
- `microservices/billing/discount-resolver/` — prepayment + BYOK applier.
- `microservices/billing/treasury/` — FX table.
- `microservices/governance/pricing/` — anchor drift monitor.
- `microservices/marketplace/` — settlement substrate per ADR-0314.

### Journeys

- `journeys/billing/jou-2026-05-20-quote-a-customer/` — sales-facing
  journey.
- `journeys/billing/jou-2026-05-20-anchor-revision/` — council-facing
  revision journey.

### Specs

- `/specs/pricing/tier-anchor-bands.json`
- `/specs/pricing/per-category-anchors.json`
- `/specs/pricing/fx-rates.json`
- `/specs/products/capability-tier-registry.json`

### External references

- Stripe's public pricing page (as a Stripe-class quality bar reference).
- AWS, GCP, Azure tier-based pricing (as comparable substrate-cost models).
- 27 B2B SaaS pricing tables surveyed in
  `docs/marketing/market-analysis-w3-g-2026-05-19.md`.

### Feedback notes consumed

- `feedback_quality_performance_scalability_bar.md`
- `feedback_canonical_base_localization.md`
- `feedback_byok_everywhere_credentials.md`
- `feedback_multi_category_marketplace_doctrine.md`
- `feedback_substrate_vs_product_layering.md`
- `feedback_compliance_pack_primitive.md`

## Appendix A — Worked example: a Silver-tier KR finance tenant

A tenant in scope `acme-kr-finance` activates:

- Tier: Silver.
- Category: Agent.
- Compliance packs: GDPR, CSAP, SOC2.
- Residency: sovereign-cell in KR.
- BYOK: enabled (OpenAI keys activated within 30 days).
- Prepayment: annual.

Computation:

1. `base = 799` (Agent × Silver).
2. `pack_factor = min(1.10 × 1.30 × 1.15, 1.85) = min(1.6445, 1.85) = 1.6445`.
3. `residency_factor = 1.30` (sovereign-cell).
4. `sub_total = 799 × 1.6445 × 1.30 = 1707.93`.
5. LLM-cost share = 65% of sub_total = `1109.65`.
6. BYOK discount on LLM cost = `1109.65 × 0.15 = 166.45`.
7. Post-BYOK = `1707.93 - 166.45 = 1541.48`.
8. Prepayment 12% discount = `1541.48 × 0.88 = 1356.50`.
9. Final MRR USD = `$1,356.50` (annual = `$16,278.00`).

The quote pins the FX rate to KRW at the daily rate (approximately
₩1.36 million MRR at 2026-05-20 rates) and is valid for 30 days.

## Appendix B — Anchor sensitivity table

The annual-revenue sensitivity of a ±10% anchor revision (compounded
across all tiers and categories) is computed in
`docs/marketing/anchor-sensitivity-2026-05-20.md`. Summary:

| Revision  | First-year ARR delta | Customer-churn delta |
|-----------|----------------------|----------------------|
| +10%      | +$8.7M               | +2.3%                |
| +5%       | +$4.2M               | +0.9%                |
| 0%        | baseline             | baseline             |
| -5%       | -$3.9M               | -0.7%                |
| -10%      | -$7.5M               | -1.5%                |

Council-finance models the optimum as +0% to +5% over the next 12
months; further revisions await the W4-pricing-review wave scheduled
for 2027-Q1.

## Appendix C — Cross-walk to ADR-0316 tier semantics

ADR-0316 defined the tier semantics in qualitative terms; this
appendix cross-walks the qualitative description to the
quantitative anchors:

| Tier        | ADR-0316 semantic              | ADR-0325 base USD MRR (cheapest category) |
|-------------|--------------------------------|--------------------------------------------|
| Bronze      | Single-team workgroup; ≤25 users | $39                                       |
| Silver      | Single-business-unit; ≤250 users | $149                                      |
| Gold        | Multi-business-unit; ≤2,500 users| $599                                      |
| Platinum    | Enterprise; unlimited users      | $2,499                                    |

The anchor steps roughly 4× per tier; this multiplier reflects the
substrate-cost differential observed across the named tier
substrates per ADR-0254 K8s + Cloud Hypervisor cost model.

## Appendix D — Worked example: pack-stacking cap reached

A tenant `acme-multi-regulated` activates all six packs simultaneously
on the Gold-tier Workflow category:

- Tier: Gold; Category: Workflow; `base = 2499`.
- Packs: HIPAA + GDPR + SOC2 + CSAP + PCI + EU_AI_ACT.
- Naive multiplicative composition:
  `1.25 × 1.10 × 1.15 × 1.30 × 1.20 × 1.25 = 3.084`.
- Capped at `1.85`.
- Residency: sovereign_cell (mandated by CSAP); `residency_factor = 1.30`.
- BYOK: disabled.
- Prepayment: monthly.

Computation:

1. `base = 2499`.
2. `pack_factor = min(3.084, 1.85) = 1.85`.
3. `residency_factor = 1.30`.
4. `sub_total = 2499 × 1.85 × 1.30 = 6011.10`.
5. No BYOK discount.
6. No prepayment discount.
7. Final MRR = `$6,011.10`.

Without the cap, the same tenant would see `$10,015.62 MRR` — a 67%
higher quote that would land in Platinum-anchor territory and confuse
the tier semantics. The cap is the customer-protection mechanism that
keeps the quote within Gold-tier expectations.

## Appendix E — Quote API surface

The price-resolver microservice exposes a single gRPC endpoint over
HTTP/3 (per ADR-0253):

```proto
service PriceResolver {
  rpc ResolveQuote(QuoteRequest) returns (QuoteResponse);
  rpc StreamPriceTable(Empty) returns (stream PriceAnchorEvent);
}

message QuoteRequest {
  string tenant_id = 1;
  string category = 2;
  string tier = 3;
  repeated string packs = 4;
  string residency = 5;
  string byok_mode = 6;
  bool prepayment_annual = 7;
  string preferred_currency = 8;
}

message QuoteResponse {
  string quote_id = 1;
  string anchor_revision_id = 2;
  string currency = 3;
  string mrr_canonical_usd_cents = 4;  // Decimal as string to avoid float
  string mrr_localised_amount = 5;
  string mrr_localised_currency = 6;
  string fx_rate_pinned = 7;
  string validity_expires_at = 8;
  google.protobuf.Timestamp emitted_at = 9;
  repeated LineItem line_items = 10;
}

message LineItem {
  string label = 1;
  string amount = 2;
  string contribution_kind = 3;  // "base", "pack_uplift", "residency_uplift", "byok_discount", "prepayment_discount"
}
```

The endpoint is authoritative; sales-facing tooling consumes quotes
through this surface; settlement consumes via the same surface or via
the cached quote pinned to a signed contract.
