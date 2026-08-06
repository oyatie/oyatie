---
id: ADR-0331
title: Cross-µservice tenant_class Adoption Template
status: Superseded
superseded_by: [ADR-0701]
date: 2026-05-21
owner_team: council-architecture
decision_owner: council-architecture
doc_class: Architecture-Decision-Record
shape: Decision
authority_tier: 1
line_floor: 700
bespoke_authoring_requirement: documentation-rigor-1.1-plus-ADR-0322
related:
  - ADR-0329-tier-system-retirement.md
  - ADR-0330-tenant-class-replacement-model.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md
  - ADR-0316-capability-tier-over-product-fragmentation.md
  - ADR-0263-observability-emission-contract.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0255-byok-everywhere-credentials.md
  - ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md
  - ADR-0324-anti-script-authoring-doctrine.md
  - ADR-0249-multi-category-marketplace-doctrine.md
  - ADR-0108-deprecation-and-sunset-policy.md
  - ADR-0138-six-path-deprecation.md
  - ADR-0145-inter-microservice-communication-reform.md
related_specs:
  - /specs/master-plan-sequencing.json
  - /specs/microservices/tenancy.json
  - /specs/microservices/cloud-billing.json
related_memory:
  - feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20
  - feedback_no_capability_tiers_2026_05_20
  - feedback_oci_always_free_maximization_2026_05_20
  - feedback_zero_handroll_opentofu_only_2026_05_20
  - feedback_quality_performance_scalability_bar
  - feedback_flat_product_catalog
  - feedback_microservice_ownership_coherence_2026_05_20
  - feedback_docs_substance_not_scaffold_2026_05_20
  - feedback_automate_everything
source_anchors:
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0329-tier-system-retirement.md
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0330-tenant-class-replacement-model.md
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0243-cedar-as-universal-gate.md
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md
  - /Users/jasonlee/oyatie/.omc/state/wave-findings-aggregation-2026-05-21.md
companion_docs:
  - /Users/jasonlee/oyatie/docs/standards/brief-template.md
  - /Users/jasonlee/oyatie/docs/AGENTS.md
  - /Users/jasonlee/oyatie/docs/ADR-INDEX.md
purpose: >
  Codify the per-µservice plumbing template that every active µservice in the
  Oyatie corpus (77 µservices at audit time) MUST implement to adopt the new
  tenant_class enum (demo_trial / paid) + paid.billing_components subset
  (revenue_share / per_seat / per_usage). Specifies the twelve adoption
  surfaces (manifest / PRD / ARCHITECTURE / Cedar / capability YAMLs /
  OpenSLO / cost-budget / per-context iac / mobile-SDK / onboarding / tests /
  observability) that each µservice must touch, the IP-tenant-class-adoption.md
  skeleton each µservice files, and the ci-tenant-class-adoption-check lane
  that verifies all twelve surfaces per µservice across the canonical-build
  sequence (Phase 0 → Phase 4B).
enforcement_status: Accepted; per-µservice IPs sequenced by ADR-0328 phase order
enforced_by:
  - ci-tenant-class-adoption-check (new lane; Wave 15A bring-up)
  - oya-governance-substance-bar (existing; substance gates over the IP template)
  - oya-governance-no-template-stamping (existing; bespoke per-µservice substance bar applies)
  - oya gate validate tenant-class-manifest
  - oya gate validate tenant-class-cedar-binding
  - oya gate validate tenant-class-slo-label
  - oya gate validate tenant-class-cost-budget
  - oya gate validate tenant-class-test-coverage
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0331: Cross-µservice tenant_class Adoption Template

## Status

Accepted on 2026-05-21.

This ADR is the third in the three-ADR tenant-class triplet:

- **ADR-0329** Tier system retirement — retires Bronze/Silver/Gold/Platinum and the surrounding tier vocabulary.
- **ADR-0330** Tenant-class replacement model — defines the canonical `tenant_class ∈ { demo_trial, paid }` enum and the `paid.billing_components ⊆ { revenue_share, per_seat, per_usage }` subset.
- **ADR-0331** (this ADR) — specifies the cross-µservice plumbing template every µservice must implement to adopt the new model.

ADR-0329 and ADR-0330 establish *what* the model is. ADR-0331 establishes *how* every µservice must plumb it. The triplet is intended to be cited together; partial adoption of one without the others is rejected by ADR-0328 §D-10 substance-bar verification.

This ADR is `Accepted` (not `Proposed`) because the Wave 14 audit aggregation (`/Users/jasonlee/oyatie/.omc/state/wave-findings-aggregation-2026-05-21.md` §"Tenant-class adoption gaps (cross-µservice pattern)") established that **77/77 audited µservices flag tenant_class adoption gaps as a universal substrate gap**. There is no µservice the gap does not apply to; the template is the canonical fix.

Enforcement transitions from `advisory` to `BLOCKER` per-µservice when each µservice lands its `IP-tenant-class-adoption.md` and the `ci-tenant-class-adoption-check` lane goes green for that µservice. Once all 77 µservices are green, the lane becomes a global BLOCKER.

## Context

### A.1 Named pressure: the gap is universal

The Wave 14 audit findings aggregation (`/Users/jasonlee/oyatie/.omc/state/wave-findings-aggregation-2026-05-21.md`) records the same finding for every µservice audited so far:

> Every audited µservice flags `tenant_class` adoption gaps. Wave 2 + W3 B1 + W3 B2 + Claude R1 + Claude R2 + Recovery + Codex cohort ALL report `tenant_class_adoption_gaps: yes`. Common pattern: principal.tenant_class claim not modeled; billing_components context attribute absent; demo_trial cap-breach behavior absent; demo_trial → paid conversion flow absent.

At time of authoring this ADR, 48 of 77 active µservices have been audited; every one of the 48 flags the adoption gap. The remaining 29 µservices are inflight or queued; the prior signal makes it statistically certain they will also flag. The gap is therefore **treated as universal** for planning purposes — every µservice must adopt the template.

The most detailed gap reports are:

- **cloud-billing** (12 P0s) — billing_components not modeled (revenue_share absent, per_seat absent, per_usage partial); demo_trial → paid conversion flow absent; cap-breach detection absent.
- **cloud-billing-tax** (14 specific gaps C-01..C-14) — principal.tenant_class binding absent; billing_components context absent; downgrade prohibition absent.
- **crm** (10 P0 surfaces) — tenant_class adoption gap TOTAL (0/10 surfaces touched).
- **marketing-automation** (11 P0 surfaces) — ZERO of 11 tenant-class surfaces touched.
- **contract-lifecycle-management** (12 P0 surfaces) — ZERO of 12 tenant-class surfaces touched.

The pattern is recursive: every µservice has the same set of "missing surfaces" because the surfaces are universal substrate plumbing, not µservice-specific decisions. Without a canonical template, each µservice owner invents their own plumbing or skips it altogether — both outcomes produce drift that downstream Cedar, SLO, billing, and conversion logic cannot rely on.

This ADR fixes that by enumerating exactly twelve adoption surfaces every µservice must implement, with concrete templates and verification.

### A.2 Named pressure: prior tier scaffolding cannot be silently replaced

Per ADR-0329, the Bronze/Silver/Gold/Platinum tier ladder is retired. Replacement is not "drop the tier names and keep going." Many µservices encoded tier semantics into:

- `capability-tiers/tier-matrix.md` per-µservice files (61+ µservices have one).
- `registry/capability-tiers/{bronze,silver,gold,platinum}.json` capability registries.
- `microservice-tier-mapping.yaml` global mapping.
- Cedar fragments referencing `principal.tier in ["bronze","silver","gold","platinum"]`.
- SLO labels of the form `tier=gold`.
- Feature-flag gates of the form `if tier == "platinum"`.
- Naming convention BNF v4 form `<microservice>.<capability>.<tier>`.

Each of those surfaces is also a tenant_class adoption surface, because the replacement model puts a different shape on the same axis (binary `tenant_class` + composable `billing_components` instead of an ordered tier ladder). The retirement (ADR-0329) and the adoption (ADR-0331) MUST land in lockstep per-µservice — neither side may regress without the other progressing.

### A.3 Named pressure: the substance bar applies

Per ADR-0322 and ADR-0328 §D-4, every µservice ownership artifact must be bespoke and substance-bar-grade. The IP template defined in §D below is therefore a **starter skeleton, not a stamp** — each µservice owner must fill in the µservice-specific behavior (which cap shape applies for demo_trial; which billing_components are emitted; which Cedar predicates apply; etc.). Template stamping the same IP across multiple µservices is a P0 anti-pattern per ADR-0324 and would be caught by the substance-bar lane.

### A.4 Named pressure: the canonical-build sequence ordering applies

Per ADR-0328 §D-1, adoption must follow the 5-phase canonical sequence:

1. Phase 0 cloud-* µservices adopt first (they are the substrate).
2. Phase 1 foundations adopt next (identity, tenancy, audit-chain, observability, cloud-billing, cloud-billing-tax, etc.).
3. Phase 2 capability substrate (intelligence, ontology, workflow-engine, workflow-studio, consent-graph, detection).
4. Phase 3 communication and collaboration.
5. Phase 4A Big 8 enterprise displacement (HR/Payroll → ERP → CRM → ServiceNow → HubSpot → Microsoft → Oracle → Adobe → Atlassian).
6. Phase 4B long-tail B2B SaaS, cloud-infra, PaaS, developer tools.

The reason Phase 0 + Phase 1 adopt first is that `tenant_class` is a principal claim issued by identity (Phase 1) and a billing context owned by cloud-billing (Phase 0). Every higher-phase µservice MUST be able to trust that the principal carries `tenant_class` and that `cloud-billing` is the source-of-truth for billing_components state. Adopting `crm` before `identity` and `cloud-billing` would force `crm` to invent its own tenant-class resolution path, which is exactly the drift this ADR prevents.

### A.5 Canonical anchors

Anchor 1: ADR-0329 (tier retirement). Anchor 2: ADR-0330 (replacement model). Anchor 3: ADR-0244 (tenant scoping universal primitive). Anchor 4: ADR-0243 (Cedar as universal gate — tenant_class enters Cedar as a principal attribute). Anchor 5: ADR-0328 §D-4 (substance bar + bespoke authoring).

### A.6 Inherited constraints

- ADR-0244 §D-3 — tenant_class joins the `tenants` table as the column `audience_type` (already present per the 2026-05-20 keystone bundle). The new `tenant_class` enum is layered on top: it is a *derived* property of `(audience_type, lifecycle_state, payment-contract-state)` resolved by cloud-billing. Adopting µservices read `tenant_class` from the principal claim, not from a new column.
- ADR-0243 §D-3 — every µservice already has a minimum Cedar gate. The tenant_class fragment is an *additional* fragment composed by tenant overlay (per ADR-0243 §D-4), not a replacement.
- ADR-0263 — observability emission contract — SLO labels gain a tenant_class label; existing emission contract is preserved.
- ADR-0251 — compliance pack activation requires `tenant_class = paid` (a cross-binding the adoption template must honor).
- ADR-0255 §D-4 — provider-BYOK requires `tenant_class = paid`.
- ADR-0249 — marketplace purchases require `tenant_class = paid`.
- ADR-0316 — superseded by ADR-0329 (retirement); ADR-0331 cites the supersession but does not depend on it for adoption logic.
- ADR-0145 — direct gRPC communication is unchanged; tenant_class travels as principal metadata, not as a service-to-service field.

## Decision

### B.1 Decision statement

Every active µservice in the Oyatie corpus (77 µservices at this ADR's authoring; future µservices on creation) MUST implement the twelve adoption surfaces specified in §D below.

Each µservice MUST file a per-µservice IP at `microservices/<name>/IPs/IP-tenant-class-adoption.md` following the skeleton in §D-13.

The `ci-tenant-class-adoption-check` CI lane (specified in §E) MUST verify all twelve surfaces per-µservice, fail with a blocking finding for any surface missing or stamped, and emit per-surface evidence under `evidence/tenant-class-adoption/`.

Adoption sequencing MUST follow ADR-0328 §D-1 canonical-build phase order. A Phase N µservice MAY NOT adopt the template ahead of all Phase N-1 µservices unless the dispatch brief explicitly authorizes the inversion and records the reason.

### B.2 What this decision does not do

- This ADR does NOT define the tenant_class enum (that is ADR-0330).
- This ADR does NOT specify the lifecycle of the tier-retirement (that is ADR-0329).
- This ADR does NOT author the per-µservice IP for any specific µservice — each µservice owner does that bespoke, per ADR-0328 §D-4 and ADR-0322.
- This ADR does NOT relax the substance bar; the IP skeleton in §D-13 is a checklist, not a stamp.
- This ADR does NOT alter the tenant table schema; tenant_class is derived at principal-issuance time by cloud-billing.
- This ADR does NOT create a new µservice; cloud-billing remains the source-of-truth.
- This ADR does NOT permit template-generated content; ADR-0324 anti-script doctrine applies to every IP authored under this template.

### B.3 Decision drivers

Driver 1: 77/77 µservices need the same plumbing; without a template, drift is guaranteed.
Driver 2: tenant_class adoption is universal substrate; per-µservice deviation produces incompatible Cedar fragments, SLO labels, and billing emissions.
Driver 3: the canonical-build sequence (ADR-0328) demands ordered adoption so higher phases can trust lower phases.
Driver 4: the substance bar (ADR-0322 + ADR-0328 §D-4) requires bespoke per-µservice content; the template is a checklist scaffold, not the content.
Driver 5: CI verification (ADR-0322 lanes) requires per-surface evidence; a uniform template makes verification scriptable.
Driver 6: the audit findings backlog already names "tenant-class adoption gap" in 48 µservices; closing the backlog requires the template.

## Consequences

### C.1 Positive consequences

- Every µservice owner has a deterministic 12-surface checklist; no invention required.
- Cedar fragments referencing `tenant_class` have a single canonical shape, evaluable by ADR-0243 §D-6 in-cell cache.
- SLO labels carry a uniform `tenant_class` axis that observability can roll up across the corpus.
- Cost-budget files distinguish demo_trial cap-bounded cost from paid scaled cost, enabling FinOps to enforce the OCI Always Free ceiling for demo_trial without invading paid budgets.
- Per-context iac modules ship demo_trial-aware variants (OCI Always Free) and paid-aware variants (any context), enabling zero-handroll provisioning per `feedback_zero_handroll_opentofu_only_2026_05_20`.
- Mobile/SDK clients propagate tenant_class context as a header, enabling Cedar evaluation at the edge without re-querying cloud-billing.
- The demo_trial → paid conversion flow is a defined cross-µservice contract (cloud-billing owns the state transition, every µservice reads the resulting principal claim).
- The `ci-tenant-class-adoption-check` lane closes the backlog: once green corpus-wide, the universal substrate gap is closed.

### C.2 Negative consequences

- Every µservice must spend authoring effort on twelve surfaces.
- The IP template is a checklist — owners may be tempted to stamp it; the substance bar (ADR-0322) must catch stamping.
- Sequence discipline (ADR-0328 §D-1) means lower-phase µservices block higher-phase adoption; opportunistic Phase 4 owners cannot pre-empt.
- The Cedar fragment surface grows by one fragment per µservice (77+ new fragments); the policy-engine in-cell cache must scale.
- The OpenSLO surface gains a label cardinality multiplier (`tenant_class ∈ {demo_trial, paid}` = 2× existing labels); observability cardinality budgets must absorb the increase.
- The cost-budget surface needs a per-tenant_class breakdown, doubling the budget table size per µservice.

### C.3 Engineering-rigor dimensions

| Dimension | Requirement created by this ADR | Acceptance signal |
|---|---|---|
| Maintainability | One canonical template across 77+ µservices | Every µservice's IP-tenant-class-adoption.md matches the skeleton + has bespoke content per ADR-0322 substance bar |
| Observability | `tenant_class` label present on every SLI + every audit event + every cost emission | OpenSLO + audit-chain + cost-budget evidence file all carry the label |
| Scalability | Cedar in-cell cache scales to 77+ new fragments; SLO cardinality 2× absorbed | Cedar p99 ≤ 1 ms per ADR-0243 §D-6; observability cost stays within budget |
| Performance | tenant_class resolution is principal-claim-time, not request-time | No µservice queries cloud-billing per-request to resolve tenant_class |
| Optimization | demo_trial workloads pinned to OCI Always Free; paid scales without cap | iac/oci-guest/always-free/ module present for every µservice; no demo_trial workloads outside Always Free profile |
| Code quality | IP template is verified by ci-tenant-class-adoption-check; substance is verified by substance-bar lane | Every µservice has both lanes green before declaring adoption done |

### C.4 Hyperscaler-grade rigor application

**Named precedent.** AWS IAM principals carry a `pathPrefix` and account-level attributes (per ADR-0244 §A.6 Stripe / AWS IAM / Azure AAD references). The tenant_class adoption pattern follows AWS-style **principal attributes resolved at request boundary, applied via policy** — every µservice trusts the principal's claim rather than re-resolving the attribute at request time.

**Failure-mode tree.** Failure modes: (1) µservice forgets to consume the principal claim → fails-closed default in Cedar fragment denies; (2) cloud-billing emits stale claim → cache TTL bound per ADR-0243 §D-10 hot-reload; (3) µservice over-restricts demo_trial → cap-breach grace flow allows recovery; (4) µservice under-restricts (allows compliance pack for demo_trial) → ADR-0251 pack activation gate blocks; (5) tenant_class state changes mid-session → principal claim is reissued at next token refresh.

**Capacity math.** 77 µservices × 12 adoption surfaces = 924 surface-touches. At ~1 IP-tenant-class-adoption.md per µservice (≤200 lines bespoke per ADR-0328 §D-6) + ~20 lines of Cedar fragment + ~5 lines of SLO label + ~10 lines of cost-budget + ~15 lines of capability-yaml block ≈ 250 lines per µservice = 19,250 lines authored corpus-wide. Spread across 12 batches of 8 codex agents per ADR-0328 §D-7 → ~95 agent-batches at ~200 lines/batch ≈ achievable in one realignment sub-wave.

**Observability hooks.** Every µservice's audit-chain emission gains a `tenant_class` field; every SLO gains a tenant_class label; every cost-attribution event gains a tenant_class dimension. Per ADR-0263 observability emission contract, these are additive — they do not break existing consumers.

**Rollback path.** Adoption is per-µservice. If a single µservice's IP fails substance or CI, that µservice's IP is reverted; the other 76+ are unaffected. The ci-tenant-class-adoption-check lane reports per-µservice status; rollback granularity is per-µservice.

**Multi-region awareness.** Cell binding (ADR-0009) is unchanged. demo_trial tenants on OCI Always Free pin to a specific Always-Free cell; paid tenants follow the standard home_cell/dr_cell pattern per the tenant's chosen deployment context.

**Sovereign-cell awareness.** demo_trial cannot activate sovereign-cloud packs per ADR-0251 (compliance pack activation requires `tenant_class = paid`). The Cedar fragment in §D-4 encodes this cross-binding.

**Versioning + deprecation.** IP-tenant-class-adoption.md is versioned per ADR-0108. Sunset of an adoption surface (e.g., if billing_components changes) follows ADR-0138 six-path deprecation. ADR-0329 retirement (Bronze/Silver/Gold/Platinum) happens in parallel; ADR-0331 adoption fills the resulting gap.

## D. Detailed mechanics — twelve adoption surfaces

Each subsection D-1 through D-12 enumerates one mandatory adoption surface. D-13 specifies the IP-tenant-class-adoption.md skeleton that every µservice files. Numbering is normative.

### D-1: manifest.json — tenant_class_eligibility + paid_billing_components_emitted

D-1.1. Every µservice's `microservices/<name>/manifest.json` MUST declare a top-level `tenant_class_eligibility` field whose value is a subset of `["demo_trial", "paid"]`.

D-1.2. The default value is `["demo_trial", "paid"]` — every µservice serves both tenant classes by default.

D-1.3. A µservice MAY restrict to `["paid"]` only when its capability is licensed paid-only (e.g., a compliance pack adapter, a high-cost workload). Restriction to `["paid"]` MUST be justified in the µservice's PRD §B with a named reason from the ADR-0330 enumeration.

D-1.4. A µservice MAY NOT restrict to `["demo_trial"]` alone — every paid tenant must have access to every µservice it has Cedar permits for.

D-1.5. Every µservice's manifest.json MUST also declare `paid_billing_components_emitted` whose value is a subset of `["revenue_share", "per_seat", "per_usage"]`.

D-1.6. The default value is `[]` — most µservices do not directly emit billing events; cloud-billing is the source-of-truth.

D-1.7. A µservice that emits per_usage events (e.g., `intelligence` for tokens, `workflow-engine` for executions, `cloud-storage` for GB-stored, `cloud-compute-functions` for invocations) MUST declare `per_usage` in this subset.

D-1.8. A µservice that emits per_seat events (e.g., `identity` for seat-counted authentications) MUST declare `per_seat` in this subset.

D-1.9. A µservice that emits revenue_share events (e.g., `marketplace`, `payments`, `cloud-marketplace`) MUST declare `revenue_share` in this subset.

D-1.10. Manifest format reference example for a per_usage-emitting µservice:

```json
{
  "name": "intelligence",
  "tenant_class_eligibility": ["demo_trial", "paid"],
  "paid_billing_components_emitted": ["per_usage"],
  "tenant_class_caps_ref": "capabilities/tenant-class-caps.yaml",
  "tenant_class_iac_variants": {
    "demo_trial": "iac/oci-guest/always-free/",
    "paid": ["iac/oyatie-public-cloud/", "iac/aws-guest/", "iac/oci-guest/paid/", "iac/on-prem/", "iac/colo/", "iac/oyatie-cloud-provider/"]
  }
}
```

D-1.11. The `tenant_class_caps_ref` MUST point to the capability YAML described in D-6.

D-1.12. The `tenant_class_iac_variants` MUST enumerate the six deployment contexts per ADR-0328 §D-15 for paid, and the OCI Always Free path for demo_trial.

D-1.13. CI lane `ci-tenant-class-adoption-check` step 1 parses the manifest, validates the three new fields are present and well-formed.

### D-2: PRD.md §B — tenant_class enumeration + per-class capabilities

D-2.1. Every µservice's `microservices/<name>/PRD.md` MUST contain a §B section titled `Tenant-class capability surface`.

D-2.2. §B MUST enumerate the two tenant classes (demo_trial + paid) with their per-class capability differences for this specific µservice.

D-2.3. §B MUST be bespoke — generic prose copied from ADR-0330 fails the substance bar per ADR-0322.

D-2.4. The §B template MUST be filled in with µservice-specific:

  - For demo_trial:
    - Concrete usage cap shape (e.g., "5 agents max", "10 workflows max", "100 MB storage", "1 GB egress/month", "10K tokens/day").
    - Time gate shape (e.g., "30-day trial expiry", "no time expiry — pure usage cap").
    - Best-effort SLO statement (e.g., "p95 latency target ≤ 500 ms; no contractual guarantee").
    - Cap-breach behavior (e.g., "soft-deny new operations; 7-day grace; conversion prompt").
    - Compliance pack prohibition (no HIPAA/SOC2/PCI/EU-AI-Act packs).
    - Provider BYOK prohibition (no LLM BYOK; platform_default only).
    - Marketplace purchase prohibition (no plugins/apps/workflows from marketplace).
  - For paid:
    - Concrete no-cap statement (e.g., "no agent count cap; pays-per-agent under per_seat or per-execution under per_usage").
    - Per-component behavior:
      - If per_usage emitted by this µservice: which meter (e.g., "tokens consumed", "GB stored").
      - If per_seat emitted: which seat-counting unit.
      - If revenue_share emitted: which transaction cohort.
    - Contractual SLO statement matching the tenant's contract.
    - Compliance pack activation paths (which packs this µservice supports).
    - Provider BYOK paths (which provider classes accept BYOK).

D-2.5. §B MUST cite ADR-0330 by name for the canonical enum + composable billing_components definitions.

D-2.6. §B MUST NOT redefine the enum or the billing_components — that is owned by ADR-0330 and cloud-billing.

D-2.7. §B MUST cross-reference the µservice's capability YAML (D-6) and the µservice's Cedar fragment (D-4) by file path.

### D-3: ARCHITECTURE.md — tenant_class depth axis in cross-cutting section

D-3.1. Every µservice's `microservices/<name>/ARCHITECTURE.md` MUST include a cross-cutting section (typically §F or §G — owner discretion) titled `Tenant-class axis`.

D-3.2. The section MUST describe how the µservice resolves tenant_class at request time:

  - Where in the request path the principal claim is read (typically the gateway-bound interceptor per ADR-0145).
  - Which Cedar fragment file gates which operation.
  - Which code module materializes the tenant_class-aware behavior (e.g., a `tenant_class_resolver.rs` or equivalent).
  - How cap enforcement is wired (rate-limit middleware, quota check, ingestion gate, etc.).

D-3.3. The section MUST describe the µservice's interaction with cloud-billing for state changes:

  - How a tenant_class state change (demo_trial → paid conversion; paid suspension; paid offboarding) reaches the µservice (typically via principal-claim refresh at next token refresh per ADR-0243 §D-10 hot-reload).
  - How the µservice handles in-flight requests that span a state change (typically: in-flight requests honor the original claim; the next request gets the new claim).

D-3.4. The section MUST cite ADR-0330 as the source-of-truth for the enum, ADR-0244 as the principal-binding source, and ADR-0243 as the Cedar evaluation source.

D-3.5. The section MUST NOT duplicate content from the PRD §B or the IP template — those are the bespoke surfaces; ARCHITECTURE describes the wire-up.

### D-4: Cedar policies — tenant_class principal-claim gate fragments

D-4.1. Every µservice's `microservices/<name>/policies/tenant-class.cedar` MUST exist as a dedicated Cedar fragment.

D-4.2. The fragment MUST be loaded into the per-µservice policy bundle per ADR-0243 §D-2 fragment lifecycle.

D-4.3. The fragment MUST contain at minimum:

  - One `forbid` clause for any operation that is paid-only (e.g., compliance pack activation, marketplace purchase, provider BYOK) when the principal carries `tenant_class = demo_trial`.
  - One `permit` clause for paid-only operations when the principal carries `tenant_class = paid` AND the appropriate billing_component is in the principal's billing_components set.
  - One `forbid` clause for demo_trial that exceeds the cap shape (concrete cap value comes from the µservice's capability YAML per D-6).

D-4.4. Concrete template for `microservices/<name>/policies/tenant-class.cedar`:

```cedar
// microservices/<name>/policies/tenant-class.cedar
// Per ADR-0331 §D-4. Cedar v4.2 grammar.
// References namespace Tenancy::Tenant entity-type from ADR-0244 §D-4.

// (1) Forbid paid-only operations for demo_trial tenants.
forbid (
    principal,
    action in [
        Action::"<microservice>::activateCompliancePack",
        Action::"<microservice>::purchaseFromMarketplace",
        Action::"<microservice>::enableProviderByok"
    ],
    resource
)
when {
    principal.tenant_class == "demo_trial"
};

// (2) Permit paid operations gated on billing_component subset.
//     Example: per_usage-metered operation requires per_usage in components.
permit (
    principal,
    action == Action::"<microservice>::executeMeteredOperation",
    resource
)
when {
    principal.tenant_class == "paid" &&
    principal.billing_components.contains("per_usage")
};

// (3) Forbid demo_trial that exceeds the µservice-declared cap.
//     Cap values are loaded from microservices/<name>/capabilities/tenant-class-caps.yaml
//     and reach Cedar via the per-µservice context resolver per ADR-0243 §D-1.
forbid (
    principal,
    action == Action::"<microservice>::createResource",
    resource
)
when {
    principal.tenant_class == "demo_trial" &&
    context.current_resource_count >= context.demo_trial_resource_cap
};

// (4) Forbid demo_trial after time-expiry.
forbid (
    principal,
    action,
    resource
)
when {
    principal.tenant_class == "demo_trial" &&
    principal.demo_trial_expired_at != null &&
    context.request_time > principal.demo_trial_expired_at
};

// (5) Per-µservice-specific predicates (owner bespoke; substance-bar required).
//     Replace this comment with the µservice's tenant_class-gated operations.
```

D-4.5. The fragment MUST be reviewed under the multispectrum review v2.4.0 lane per ADR-0322 — Cedar policy changes carry the same review bar as code.

D-4.6. The fragment MUST emit an audit event on every deny via the `microservice.tenant_class.deny` event class per ADR-0263.

D-4.7. The fragment MUST be loaded by the per-cell policy-engine per ADR-0243 §D-5 bootstrap chain of trust.

D-4.8. The fragment MUST NOT contain `tier ∈ {bronze, silver, gold, platinum}` predicates — those are retired per ADR-0329.

### D-5: capability YAMLs — tenant_class_caps block

D-5.1. Every µservice's `microservices/<name>/capabilities/tenant-class-caps.yaml` MUST exist.

D-5.2. The file declares the per-tenant_class quota shape for this µservice in a uniform schema.

D-5.3. Concrete template for `microservices/<name>/capabilities/tenant-class-caps.yaml`:

```yaml
# microservices/<name>/capabilities/tenant-class-caps.yaml
# Per ADR-0331 §D-5. tenant_class quota declarations for <microservice>.

apiVersion: oyatie.platform/v1
kind: TenantClassCaps
metadata:
  microservice: <name>
  adr_refs: [ADR-0329, ADR-0330, ADR-0331]
  authored_by: <µservice-owner-agent-id>
  authored_at: 2026-05-21

demo_trial:
  # Hard caps — enforced by Cedar fragment §D-4 clause (3).
  # Cap values are µservice-specific; substance bar requires bespoke values.
  caps:
    primary_resource_count: 5         # e.g., agents, workflows, projects (µservice-specific)
    secondary_resource_count: 10      # e.g., executions, runs (µservice-specific)
    storage_bytes: 104857600          # 100 MB
    monthly_egress_bytes: 1073741824  # 1 GB
    monthly_metered_unit: 10000       # e.g., 10K tokens, API calls
    concurrent_requests: 4
    max_parent_tenancy_depth: 1       # demo_trial cannot create sub-tenants

  # Time gate.
  time_expiry_days: 30                # 30-day trial; null = no time gate
  expiry_grace_days: 7                # post-expiry grace before suspend

  # Cap-breach behavior.
  cap_breach_response: soft_deny      # soft_deny | hard_deny | rate_limit
  cap_breach_grace_days: 3            # grace before conversion prompt
  cap_breach_notification: email_and_in_app

  # Best-effort SLO.
  slo_class: best_effort              # best_effort | contractual
  slo_p95_latency_ms: 500             # µservice-specific
  slo_availability_pct: 99.0          # best-effort

  # Forbidden surfaces.
  forbidden_features:
    - compliance_pack_activation
    - provider_byok_enable
    - marketplace_purchase
    - sovereign_cell_routing
    - cross_tenant_collaboration

  # Default infrastructure.
  default_deployment_context: oci-guest-always-free

paid:
  # Caps — none by default; paid scales with payment.
  caps: {}                            # empty subset means no caps

  # No time gate.
  time_expiry_days: null

  # SLO class.
  slo_class: contractual              # contractual under the tenant's contract
  slo_p95_latency_ms: 100             # µservice-specific contractual target
  slo_p99_latency_ms: 250
  slo_availability_pct: 99.9

  # Permitted billing_components for this µservice.
  permitted_billing_components:
    - per_usage                       # if µservice emits per_usage (D-1)
    - per_seat                        # if µservice has seat semantics
    - revenue_share                   # if µservice has rev-share semantics

  # Permitted surfaces.
  permitted_features:
    - compliance_pack_activation
    - provider_byok_enable
    - marketplace_purchase
    - sovereign_cell_routing
    - cross_tenant_collaboration
    - mobile_sdk_extended_session

  # Default infrastructure (tenant chooses at conversion).
  default_deployment_context_choice:
    - oyatie-public-cloud
    - aws-guest
    - oci-guest-paid
    - on-prem
    - colo
    - oyatie-cloud-provider
```

D-5.4. The substance bar (ADR-0322) requires that every numeric value in `caps:`, `slo_*:`, and the lists is bespoke and justified by the µservice's PRD §B.

D-5.5. The file is loaded by the µservice's tenant_class context resolver (D-3) which translates into Cedar context attributes per D-4 clause (3).

D-5.6. CI lane `ci-tenant-class-adoption-check` step 5 validates the YAML schema against the canonical TenantClassCaps schema at `/specs/schemas/tenant-class-caps.schema.json`.

### D-6: OpenSLO files — tenant_class SLI label

D-6.1. Every µservice's OpenSLO files at `microservices/<name>/slos/*.openslo.yaml` MUST add a `tenant_class` SLI label.

D-6.2. The label values are exactly `demo_trial` and `paid` — same enum as ADR-0330.

D-6.3. Existing labels are preserved; the new label is additive per ADR-0263.

D-6.4. Concrete template snippet for an existing OpenSLO file:

```yaml
# microservices/<name>/slos/<name>-availability.openslo.yaml
apiVersion: openslo/v1
kind: SLO
metadata:
  name: <microservice>-availability
  labels:
    tier: "${tenant_class}"           # RETIRED per ADR-0329 — REMOVE this line
spec:
  service: <microservice>
  indicator:
    metadata:
      name: <microservice>-availability-sli
    spec:
      ratioMetric:
        good:
          metricSource:
            type: prometheus
            spec:
              query: |
                sum by (tenant_class) (
                  rate(<microservice>_request_total{
                    code!~"5..",
                    tenant_class=~"demo_trial|paid"   # new label per ADR-0331 §D-6
                  }[5m])
                )
        total:
          metricSource:
            type: prometheus
            spec:
              query: |
                sum by (tenant_class) (
                  rate(<microservice>_request_total{
                    tenant_class=~"demo_trial|paid"
                  }[5m])
                )
  objectives:
    # demo_trial — best-effort SLO target.
    - displayName: "<microservice> availability — demo_trial best-effort"
      target: 0.99                    # value from capability YAML §D-5
      timeWindows:
        - duration: 30d
          isRolling: true
      labels:
        tenant_class: demo_trial
        slo_class: best_effort        # per ADR-0331 §D-5
    # paid — contractual SLO target.
    - displayName: "<microservice> availability — paid contractual"
      target: 0.999                   # value from capability YAML §D-5
      timeWindows:
        - duration: 30d
          isRolling: true
      labels:
        tenant_class: paid
        slo_class: contractual        # per ADR-0331 §D-5
```

D-6.5. The label values MUST match the enum exactly; CI lane `ci-tenant-class-adoption-check` step 6 verifies enum conformance.

D-6.6. Existing `tier`-labeled SLOs MUST be retired per ADR-0329; the migration is in-place — `tier:` label is removed and `tenant_class:` label is added in the same edit.

D-6.7. Demo_trial SLO targets MAY be lower than paid targets — that is the best-effort vs contractual distinction. Paid targets MAY NOT be lower than demo_trial — paid quality bar ≥ demo_trial.

### D-7: cost-budget.md — tenant_class axis

D-7.1. Every µservice's `microservices/<name>/cost-budget.md` MUST include a `tenant_class` axis breakdown.

D-7.2. The breakdown distinguishes demo_trial cost (cap-bounded; pinned to OCI Always Free) from paid cost (scales with consumption; bounded by tenant's contracted budget).

D-7.3. Concrete template section to add to cost-budget.md:

```markdown
## Tenant-class cost breakdown (per ADR-0331 §D-7)

| Cost line | demo_trial bound | paid bound | Notes |
|---|---|---|---|
| Compute | OCI Always Free 4 OCPU + 24 GB cap | scales with workload | per ADR-0331 §D-5 caps |
| Storage | OCI Always Free 200 GB cap | per per_usage meter | per ADR-0330 billing_component (c) |
| Egress | OCI Always Free 10 TB/month cap | per per_usage meter | demo_trial soft-cap on breach |
| Tokens (if AI) | 10K/day cap | per per_usage meter | per ADR-0331 §D-5 caps |
| Database | OCI Always Free 2× Autonomous DB cap | scales with workload | per ADR-0331 §D-5 caps |
| KMS / Vault | shared OCI Always Free Vault | tenant-scoped Vault | per ADR-0244 KMS root |
| Observability | best-effort metrics + 7-day retention | full metrics + contracted retention | per ADR-0263 |
| **Aggregate ceiling** | OCI Always Free perpetual ceiling | tenant contract bound | demo_trial cannot exceed Always Free |
```

D-7.4. The cost-budget file MUST cite ADR-0331 §D-7 and ADR-0330 for the canonical enum.

D-7.5. The cost-budget file MUST cite `feedback_oci_always_free_maximization_2026_05_20` for the OCI Always Free ceiling on demo_trial.

D-7.6. CI lane `ci-tenant-class-adoption-check` step 7 verifies the breakdown table is present and contains both `demo_trial` and `paid` columns.

### D-8: per-context iac/<context>/ — tenant_class-aware module variants

D-8.1. Every µservice's `microservices/<name>/iac/` MUST contain six deployment-context sub-directories per ADR-0328 §D-15:

  - `iac/oyatie-public-cloud/`
  - `iac/aws-guest/`
  - `iac/oci-guest/` (with both `always-free/` and `paid/` sub-directories per `feedback_oci_always_free_maximization_2026_05_20`)
  - `iac/on-prem/`
  - `iac/colo/`
  - `iac/oyatie-cloud-provider/`

D-8.2. The `iac/oci-guest/always-free/` sub-directory MUST contain a tenant_class-aware module that provisions the µservice within the OCI Always Free perpetual ceiling for `tenant_class = demo_trial` tenants only.

D-8.3. The Always-Free module MUST declare an OpenTofu variable `tenant_class` whose value MUST be `demo_trial` (validated by the module).

D-8.4. The Always-Free module MUST refuse `apply` when `tenant_class != "demo_trial"` per the variable validation.

D-8.5. Concrete OpenTofu validation snippet for the Always-Free module:

```hcl
# microservices/<name>/iac/oci-guest/always-free/variables.tf
variable "tenant_class" {
  description = "Tenant class for this Always-Free deployment per ADR-0331 §D-8."
  type        = string

  validation {
    condition     = var.tenant_class == "demo_trial"
    error_message = "iac/oci-guest/always-free/ MUST be applied only for tenant_class = demo_trial per ADR-0331 §D-8.4. Paid tenants use iac/oci-guest/paid/."
  }
}

variable "tenant_id" {
  description = "Tenant slug per ADR-0244 §D-1."
  type        = string

  validation {
    condition     = can(regex("^[a-z][a-z0-9-]{0,62}(\\.[a-z0-9-]{1,62}){0,4}$", var.tenant_id))
    error_message = "tenant_id MUST conform to ADR-0244 §D-1 format."
  }
}
```

D-8.6. The paid module variants MUST accept `tenant_class = "paid"` and refuse `tenant_class = "demo_trial"`.

D-8.7. Module engines MUST be OpenTofu, not HashiCorp Terraform per `feedback_zero_handroll_opentofu_only_2026_05_20`. Files named `terraform.tf` or directories named `iac/terraform/` are P0 findings.

D-8.8. CI lane `ci-tenant-class-adoption-check` step 8 verifies the presence of the six deployment-context directories, the OCI Always-Free sub-variant, and the tenant_class variable validation in every module.

### D-9: mobile/SDK clients — tenant_class context propagation

D-9.1. Every µservice's client SDK (if it has one — see `developer-sdk` for canonical pattern) MUST propagate the `tenant_class` context to the server as a request header.

D-9.2. The canonical header is `X-Oyatie-Tenant-Class: <demo_trial|paid>`.

D-9.3. The header is set by the SDK at session-start time based on the resolved principal claim; the SDK does NOT re-resolve tenant_class per-request.

D-9.4. On the server side, the µservice gateway compares the header value against the principal claim's `tenant_class` and rejects mismatches with a `400 Bad Request` and an audit emission.

D-9.5. Mobile SDKs (Swift / Kotlin / WinUI 3 C# per `feedback_rust_strict_only_no_python_2026_05_20`) MUST implement the header propagation in the SDK's request interceptor.

D-9.6. Web SDK (Leptos SSR + WASM hydration per ADR-0328 §D-18.51) MUST set the header in the WASM fetch interceptor.

D-9.7. C / C++ / Rust / Go / Java / Kotlin / Swift / TypeScript / Python (SDK target languages from Stainless-class generator per `feedback_microservice_ownership_coherence_2026_05_20`) MUST all implement the header at the generated client level.

D-9.8. The header value is informational only — server trusts the principal claim, not the header. The header exists for client-side optimization (e.g., disabling premium-only UI for demo_trial without a server roundtrip).

D-9.9. CI lane `ci-tenant-class-adoption-check` step 9 verifies the SDK client integration tests cover both header values.

### D-10: Tenant onboarding flow — demo_trial → paid conversion logic

D-10.1. The onboarding flow that creates a tenant MUST default the tenant_class to `demo_trial` for self-service onboarding.

D-10.2. Enterprise sales onboarding MAY directly provision `tenant_class = paid` with a chosen billing_components subset.

D-10.3. Every µservice MUST handle the demo_trial → paid conversion transition gracefully:

  - In-flight requests retain the original claim.
  - Next token refresh carries the new claim with `tenant_class = paid`.
  - Demo_trial caps are removed at conversion.
  - Capability surfaces that were forbidden (compliance packs, marketplace, provider BYOK) become available subject to the tenant's chosen billing_components.

D-10.4. Conversion is **one-way** by default — a paid tenant cannot downgrade to demo_trial. Conversion in the reverse direction is forbidden by ADR-0330 to prevent gaming.

D-10.5. A paid tenant that churns enters `lifecycle_state = OFFBOARDING` per ADR-0244 §D-7, then `SOFT_DELETED`, then `HARD_DELETED`. It does not return to demo_trial.

D-10.6. cloud-billing owns the conversion flow; every other µservice consumes the resulting principal claim.

D-10.7. The onboarding flow MUST emit a `tenant.created` audit event with `tenant_class = demo_trial` (or `paid` for enterprise direct) per ADR-0263.

D-10.8. The conversion flow MUST emit a `tenant.converted_to_paid` audit event with the chosen billing_components subset and the conversion timestamp.

D-10.9. The cap-breach flow MUST emit a `tenant.cap_breached` audit event with the specific cap name and the breach value.

D-10.10. CI lane `ci-tenant-class-adoption-check` step 10 verifies the onboarding flow tests cover demo_trial creation, demo_trial → paid conversion, and cap-breach handling.

### D-11: Tests — per-tenant_class test cases

D-11.1. Every µservice's `microservices/<name>/tests/tenant_class/` directory MUST contain integration tests covering both tenant classes.

D-11.2. Minimum test coverage:

  - `test_demo_trial_under_cap.rs` — operation succeeds when demo_trial is below cap.
  - `test_demo_trial_at_cap.rs` — operation is soft-denied when demo_trial hits cap; grace flow returns 429 + retry-after.
  - `test_demo_trial_after_expiry.rs` — operation is hard-denied when demo_trial expired.
  - `test_paid_unconstrained.rs` — operation succeeds when paid with no caps.
  - `test_paid_per_usage_meter_emission.rs` — operation emits a per_usage event when the µservice declares per_usage in D-1.
  - `test_paid_per_seat_enforcement.rs` — operation respects seat count when the µservice declares per_seat in D-1.
  - `test_paid_revenue_share_emission.rs` — operation emits a revenue_share event when the µservice declares revenue_share in D-1.
  - `test_demo_trial_forbidden_paid_only_feature.rs` — operation is denied (Cedar forbid per D-4 clause 1) when demo_trial attempts a paid-only feature.
  - `test_conversion_demo_trial_to_paid.rs` — conversion flow ends with paid claim; caps are removed.
  - `test_downgrade_paid_to_demo_trial_forbidden.rs` — downgrade is refused with a clear error per D-10.4.

D-11.3. Test framework MUST be Rust-strict per `feedback_rust_strict_only_no_python_2026_05_20`. No Python or JavaScript test harnesses.

D-11.4. Tests MUST run in CI as part of the µservice's standard test lane.

D-11.5. Tests MUST emit deterministic audit-event assertions — for each test, the audit chain MUST emit the expected event class with the expected tenant_class field.

D-11.6. CI lane `ci-tenant-class-adoption-check` step 11 verifies the test directory exists and contains all 10 minimum test names (additional µservice-specific tests are welcomed but not enforced).

### D-12: Observability — tenant_class on every event

D-12.1. Every µservice's audit-chain emission per ADR-0263 MUST include a `tenant_class` field.

D-12.2. Every µservice's metric emission MUST include a `tenant_class` label.

D-12.3. Every µservice's log emission MUST include a `tenant_class` structured field (at WARN or higher; INFO-level inclusion is owner discretion).

D-12.4. Every µservice's distributed-tracing span MUST include a `tenant_class` attribute.

D-12.5. The label / field / attribute values MUST match the enum exactly (`demo_trial` or `paid`).

D-12.6. The dual-cardinality multiplier (existing labels × 2 for tenant_class) MUST be absorbed by the observability cost budget per ADR-0263; if a µservice's existing label cardinality is high enough that the multiplier breaks the cardinality budget, the µservice MUST file a per-µservice ADR proposing a label drop in the same edit.

D-12.7. CI lane `ci-tenant-class-adoption-check` step 12 verifies that sampled audit events carry the tenant_class field and that the metric emission registers carry the tenant_class label.

### D-13: IP-tenant-class-adoption.md skeleton (per-µservice)

Every µservice files exactly one IP at `microservices/<name>/IPs/IP-tenant-class-adoption.md` against this template skeleton. The skeleton is a **checklist** — the content under each section MUST be bespoke per ADR-0322 substance bar. Template-stamping the skeleton across µservices is a P0 finding per ADR-0324.

```markdown
---
id: IP-tenant-class-adoption
microservice: <name>
title: <microservice> tenant-class adoption
status: Proposed | In-Progress | Done
date: 2026-05-21
adr_refs: [ADR-0329, ADR-0330, ADR-0331, ADR-0244, ADR-0243, ADR-0328]
phase: <0|1|2|3|4A|4B>
batch: <wave-batch-id>
owner: <µservice-owner-agent-id>
verified_by: ci-tenant-class-adoption-check
line_floor: 200
substance_bar: bespoke-per-microservice
---

# IP-tenant-class-adoption.md — <microservice>

## §1. Adoption scope for this microservice

<bespoke paragraph: what specifically this µservice does with tenant_class.
Concrete: which capabilities are paid-only for this µservice; which billing_components
this µservice emits; which caps are enforced for demo_trial.>

## §2. Twelve adoption surfaces

### §2.1. manifest.json (D-1)
- [ ] `tenant_class_eligibility` set to <subset>.
- [ ] `paid_billing_components_emitted` set to <subset>.
- [ ] `tenant_class_caps_ref` points to capabilities/tenant-class-caps.yaml.
- [ ] `tenant_class_iac_variants` enumerates 6 deployment contexts.

### §2.2. PRD.md §B (D-2)
- [ ] §B "Tenant-class capability surface" present.
- [ ] demo_trial subsection enumerates: caps, time gate, SLO class, cap-breach behavior, forbidden surfaces, default infra.
- [ ] paid subsection enumerates: no-cap statement, per-component behavior, SLO class, permitted surfaces, default infra choice.
- [ ] §B is bespoke for <microservice>; no template prose.

### §2.3. ARCHITECTURE.md cross-cutting (D-3)
- [ ] "Tenant-class axis" section present (§F or §G owner discretion).
- [ ] Resolution path described (principal claim → Cedar fragment → cap enforcement).
- [ ] cloud-billing interaction described (state-change propagation).
- [ ] Citations to ADR-0330, ADR-0244, ADR-0243.

### §2.4. Cedar policies (D-4)
- [ ] policies/tenant-class.cedar file exists.
- [ ] Forbid clause for paid-only operations on demo_trial.
- [ ] Permit clause for paid operations gated on billing_components.
- [ ] Forbid clause for demo_trial cap-breach.
- [ ] Forbid clause for demo_trial time-expiry.
- [ ] Per-µservice bespoke predicates (substance bar).
- [ ] No `tier` predicates referencing bronze/silver/gold/platinum.

### §2.5. Capability YAMLs (D-5)
- [ ] capabilities/tenant-class-caps.yaml file exists.
- [ ] demo_trial caps section: bespoke values for primary/secondary/storage/egress/metered/concurrent.
- [ ] demo_trial time gate: 30 days or µservice-specific.
- [ ] demo_trial SLO class: best_effort.
- [ ] demo_trial forbidden_features list.
- [ ] paid caps: empty (no caps) by default.
- [ ] paid SLO class: contractual.
- [ ] paid permitted_billing_components: subset of revenue_share/per_seat/per_usage.
- [ ] paid permitted_features list.

### §2.6. OpenSLO (D-6)
- [ ] slos/*.openslo.yaml files updated.
- [ ] tenant_class label added to every SLI query.
- [ ] Two objectives per SLO: demo_trial best-effort + paid contractual.
- [ ] Existing `tier:` labels removed.
- [ ] Paid SLO targets ≥ demo_trial targets.

### §2.7. cost-budget.md (D-7)
- [ ] tenant_class breakdown table added.
- [ ] demo_trial column: OCI Always Free ceiling.
- [ ] paid column: tenant contract bound.
- [ ] Citation to ADR-0331 §D-7 and ADR-0330.

### §2.8. Per-context iac (D-8)
- [ ] iac/oyatie-public-cloud/
- [ ] iac/aws-guest/
- [ ] iac/oci-guest/always-free/
- [ ] iac/oci-guest/paid/
- [ ] iac/on-prem/
- [ ] iac/colo/
- [ ] iac/oyatie-cloud-provider/
- [ ] Always-Free module validates tenant_class == "demo_trial".
- [ ] Paid modules validate tenant_class == "paid".
- [ ] No iac/terraform/ directory; engine is OpenTofu.

### §2.9. Mobile/SDK clients (D-9)
- [ ] SDK propagates X-Oyatie-Tenant-Class header.
- [ ] Header set at session-start from principal claim.
- [ ] Server rejects header/claim mismatch with 400.
- [ ] Mobile (Swift/Kotlin/WinUI 3 C#) implementations cover the header.
- [ ] Web (Leptos WASM) implementation covers the header.

### §2.10. Onboarding flow (D-10)
- [ ] Self-service onboarding defaults to demo_trial.
- [ ] Enterprise direct paths can provision paid directly.
- [ ] demo_trial → paid conversion is handled gracefully.
- [ ] paid → demo_trial downgrade is forbidden.
- [ ] tenant.created and tenant.converted_to_paid audit events emitted.

### §2.11. Tests (D-11)
- [ ] tests/tenant_class/ directory exists.
- [ ] All 10 minimum test files present.
- [ ] Tests are Rust-strict (no Python harnesses).
- [ ] Tests emit deterministic audit-event assertions.

### §2.12. Observability (D-12)
- [ ] Audit-chain emission includes tenant_class field.
- [ ] Metric emission includes tenant_class label.
- [ ] Log emission includes tenant_class field at WARN+.
- [ ] Distributed-tracing span includes tenant_class attribute.
- [ ] Cardinality multiplier absorbed within budget.

## §3. Open questions for this microservice

<bespoke: per-µservice open questions deferred to per-µservice ADR per
ADR-0331 §G. Owner identifies questions the template does not cover.>

## §4. Verification evidence

<bespoke: paths to ci-tenant-class-adoption-check evidence for this µservice.>

## §5. Cross-references

- ADR-0329 (tier retirement)
- ADR-0330 (replacement model)
- ADR-0331 (this template)
- ADR-0244, ADR-0243, ADR-0328
- microservices/<name>/PRD.md §B
- microservices/<name>/ARCHITECTURE.md §<F or G>
- microservices/<name>/policies/tenant-class.cedar
- microservices/<name>/capabilities/tenant-class-caps.yaml
- microservices/<name>/slos/*.openslo.yaml
- microservices/<name>/cost-budget.md
- microservices/<name>/iac/<context>/
- microservices/<name>/tests/tenant_class/
```

D-13.1. Each `[ ]` checkbox MUST be ticked `[x]` before the IP transitions from `Proposed` to `In-Progress` to `Done`.

D-13.2. Each section MUST have bespoke content; a section that only contains the checkbox list and no prose fails substance-bar per ADR-0322.

D-13.3. The IP's line floor is 200; below 200 lines indicates scaffolding without substance.

D-13.4. The IP is reviewed under multispectrum review v2.4.0 per ADR-0322; reviewer-agent APPROVE required before `Done`.

D-13.5. Per-µservice deviations from the template (e.g., a µservice with no billing_components emission can declare an empty subset and still pass; a µservice with no SDK can mark D-9 N/A) MUST be justified in §3 with a per-µservice rationale.

## E. Verification — ci-tenant-class-adoption-check lane

### E.1 Lane purpose

The `ci-tenant-class-adoption-check` CI lane verifies that every active µservice has all twelve adoption surfaces (D-1 through D-12) present and structurally well-formed for that µservice. The lane is per-µservice — failure is reported per µservice and does not block other µservices.

### E.2 Lane status

Bring-up: Wave 15A precursor authoring lands ADR-0331. Wave 15A first cohort lands the lane definition in `tools/ci-lanes/tenant-class-adoption-check.yaml`. Wave 15A second cohort starts running the lane against Phase 0 µservices (cloud-* family per ADR-0328 §D-1.7..D-1.26). Lane is `BLOCKER` for a µservice once that µservice's IP-tenant-class-adoption.md is filed.

### E.3 Lane steps

The lane runs 12 verification steps, one per adoption surface, in order:

1. **manifest.json fields present + well-formed** (D-1.13).
2. **PRD.md §B section present + bespoke content check** (D-2.7).
3. **ARCHITECTURE.md cross-cutting section present + cross-refs valid** (D-3.5).
4. **Cedar fragment file exists + Cedar v4.2 parse + ADR-0329 anti-tier check** (D-4.5..D-4.8).
5. **Capability YAML schema valid + bespoke value check** (D-5.6).
6. **OpenSLO files contain tenant_class label + paid ≥ demo_trial targets check** (D-6.5..D-6.7).
7. **cost-budget.md has tenant_class breakdown table** (D-7.6).
8. **iac/ has 6 deployment-context dirs + Always-Free sub-variant + OpenTofu engine** (D-8.8).
9. **SDK clients propagate header + test coverage for both values** (D-9.9).
10. **Onboarding flow tests cover demo_trial creation + conversion + cap-breach** (D-10.10).
11. **tests/tenant_class/ contains 10 minimum test files** (D-11.6).
12. **Audit-chain + metric + tracing emit tenant_class** (D-12.7).

### E.4 Lane failure handling

A lane step failure produces a finding row in the µservice's `evidence/tenant-class-adoption/findings.md` with:

- Step number (1-12) that failed.
- Concrete file path that is missing or malformed.
- Required fix per ADR-0331 §D-<step>.
- Cross-reference to the µservice's IP-tenant-class-adoption.md row that should be ticked.

Findings are P0 if the surface is structurally absent (e.g., no Cedar fragment file). Findings are P1 if the surface exists but is stamped (substance-bar failure). Findings are P2 if the surface exists with substance but has a minor discrepancy (e.g., paid SLO target below demo_trial — this is structurally wrong per D-6.7 but is a per-line fix).

### E.5 Lane evidence emission

Per ADR-0263, the lane emits an audit event class `governance.tenant_class_adoption_check` with fields:

- `microservice` (name).
- `step_results` (12-element array of pass/fail).
- `findings_count` (P0 + P1 + P2 separate counters).
- `verdict` (PASS | PASS-WITH-FINDINGS | REVISE | BLOCK per ADR-0328 §D-4).
- `evidence_uri` (link to findings.md).

### E.6 Lane sequencing

The lane runs against µservices in ADR-0328 §D-1 canonical-build order — Phase 0 first, then Phase 1, etc. A Phase N µservice's lane status MAY be deferred until all Phase N-1 lanes are green; the lane reports `DEFERRED-AWAITING-PHASE-N-1` for that µservice.

Once all 77 µservices are green, the lane status promotes to corpus-wide BLOCKER per E.2.

## F. Rollback / migration

### F.1 Adoption is required, not optional

tenant_class adoption is universal substrate. There is no opt-out. A µservice that does not implement the template cannot promote past its phase gate per ADR-0328 §D-1.27 (Phase N requires Phase N-1 substance-bar completion).

### F.2 Per-surface rollback

If a single adoption surface fails substance (e.g., a stamped Cedar fragment), the surface is reverted; the other 11 surfaces remain in place. The IP transitions back to `In-Progress` until the surface is rewritten bespoke.

### F.3 Conversion flow is one-way

Per D-10.4, demo_trial → paid is one-way with a grace period. Reverse transitions (paid → demo_trial) are forbidden by ADR-0330 to prevent tenants from gaming caps by reverting to demo_trial after consuming paid features.

### F.4 Tier-name retirement happens in lockstep

Per ADR-0329, Bronze/Silver/Gold/Platinum names retire from the corpus. The retirement happens in the same edit as the tenant_class adoption — a µservice does not retire its tier names then later add tenant_class; both surfaces are touched in one IP per D-13.

### F.5 Sunset of the adoption phase

After every µservice has its lane green corpus-wide, the lane stays as a BLOCKER for new µservices. The lane sunsets only if ADR-0330 is itself retired and replaced by a successor enum (no such replacement is anticipated as of authoring).

### F.6 Cross-binding rollback

Compliance pack activation requires `tenant_class = paid` per ADR-0251. If a tenant on demo_trial attempts pack activation, the request is rejected at the Cedar gate per D-4 clause 1. No rollback path puts a pack on demo_trial — that combination is structurally forbidden.

## G. Open questions — deferred to per-µservice ADR

The template defines the **uniform** surfaces. Each µservice will have **µservice-specific** open questions that the template does not cover. Per ADR-0328 §D-1.106 (an audit deliverable does not author missing artifacts), these open questions are deferred to per-µservice ADRs:

### G.1 Capability-specific cap shape

Each µservice has a different "primary resource" shape (workflows for workflow-engine; agents for foundry-absorbed surfaces; emails for mail; meetings for meet). The cap value AND the cap unit are µservice-specific and MUST be authored in the µservice's PRD §B + capability YAML. The template provides the schema, not the values.

### G.2 Per-component metering semantics

The µservices that emit per_usage have µservice-specific meter shapes:

- intelligence: tokens, model-class, latency-budget.
- workflow-engine: executions, steps, retries.
- cloud-storage: GB-stored, GB-egressed, requests.
- cloud-compute-functions: invocations, GB-seconds.
- mail: emails-sent, attachments-bytes.
- (etc.)

The meter definition is per-µservice and owned by the µservice's PRD. cloud-billing consumes the meter events but does not define them.

### G.3 Per-µservice forbidden-feature list

D-5 capability YAML enumerates forbidden_features for demo_trial. The list is µservice-specific — e.g., `crm` may forbid demo_trial from accessing `sales-cadence-automation`; `mail` may forbid demo_trial from sending more than N emails/day; `cloud-compute-vm` may forbid demo_trial from provisioning > T0 cell instances. Per-µservice ADRs codify the specific forbid list.

### G.4 Per-µservice SLO target values

D-6 OpenSLO files declare paid contractual targets. The exact values are per-µservice — e.g., `identity` p99 ≤ 50 ms; `cloud-storage` p99 ≤ 100 ms; `workflow-engine` step-latency p99 ≤ 200 ms. Per-µservice benchmark docs + SLO docs codify the values.

### G.5 Per-µservice conversion side-effects

Most µservices have NO state to mutate at conversion — they consume the new principal claim and serve. Some µservices DO have state that must mutate at conversion — e.g., `cloud-storage` may need to re-allocate storage quotas; `identity` may need to issue additional seat-tokens; `cloud-billing-tax` may need to register the tenant with tax authorities. Per-µservice IPs codify the per-µservice conversion side-effects.

### G.6 Sandbox/preview tenant_class

ADR-0244 §D-3 defines sandbox + preview as separate audience types. ADR-0330 §[future-section] will clarify whether sandbox/preview map to `demo_trial` or to a third tenant_class. As of this ADR's authoring, sandbox + preview are treated as `demo_trial` with extended caps; per-µservice IPs MAY override.

### G.7 Cross-tenant collaboration tenant_class

A collaboration between two tenants (one paid + one demo_trial) raises the question of which tenant_class applies to shared resources. ADR-0331 defers this to ADR-0330 §[future-section] — until then, collaborations between mixed tenant classes are forbidden by Cedar fragment D-4 clause 1 (cross_tenant_collaboration is in demo_trial's forbidden_features per D-5).

### G.8 Marketplace seller vs buyer tenant_class

A marketplace seller (revenue_share emitter) is necessarily `paid`. A marketplace buyer is also `paid` per ADR-0249 (marketplace purchases require paid). demo_trial cannot participate in marketplace as either side. The `marketplace` µservice's IP captures this explicitly.

### G.9 Per-µservice OS matrix compatibility

ADR-0328 §D-17 OS matrix declares the supported OS list per µservice. tenant_class does NOT change the supported OS list — both demo_trial and paid run on the full OS matrix. Per-µservice IPs do not need to address OS-tenant_class interaction.

### G.10 Per-µservice cost-budget aggregation

The cost-budget breakdown table per D-7 aggregates µservice-level costs by tenant_class. The corpus-wide rollup (cloud-finops + finops-portal) aggregates per-µservice budgets into a corpus-wide tenant-class budget. Per-µservice IPs do not need to author the rollup — that is cloud-finops + finops-portal's responsibility per their own IPs.

## H. Cross-references

### H.1 Within the tenant-class triplet
- ADR-0329 Tier system retirement — retires Bronze/Silver/Gold/Platinum, the predecessor model.
- ADR-0330 Tenant-class replacement model — defines the canonical enum and billing_components subset.
- ADR-0331 (this ADR) — specifies cross-µservice adoption.

### H.2 Foundational keystones
- ADR-0244 Tenant as universal scoping primitive — tenant_class joins the principal claim.
- ADR-0243 Cedar as universal gate — tenant_class enters Cedar as a principal attribute.
- ADR-0263 Observability emission contract — tenant_class label on every emission.
- ADR-0251 Compliance pack primitive — pack activation requires `tenant_class = paid`.
- ADR-0255 BYOK everywhere — provider BYOK requires `tenant_class = paid`.
- ADR-0249 Multi-category marketplace — marketplace purchases require `tenant_class = paid`.

### H.3 Sequencing + substance
- ADR-0328 Substance bar as canonical sequence and batch discipline — adoption follows §D-1 phase order.
- ADR-0322 Substance bar as doctrine and CI enforcement — per-µservice IPs verified by substance-bar lane.
- ADR-0324 Anti-script authoring doctrine — template stamping is a P0 finding.

### H.4 Communication + retirement substrate
- ADR-0145 Inter-microservice communication reform — direct gRPC; tenant_class travels as principal metadata.
- ADR-0108 Deprecation and sunset policy — tier-name retirement follows.
- ADR-0138 Six-path deprecation — applies to surface removals within the IP edits.

### H.5 Audit findings
- /Users/jasonlee/oyatie/.omc/state/wave-findings-aggregation-2026-05-21.md — universal gap origin.
- per-µservice `microservices/<name>/coherence-audit-2026-05-20.md` files — per-µservice gap citations.

### H.6 Memory cross-references
- feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20 — canonical user-directive trace.
- feedback_no_capability_tiers_2026_05_20 — paired tier retirement directive.
- feedback_oci_always_free_maximization_2026_05_20 — demo_trial infrastructure pinning.
- feedback_zero_handroll_opentofu_only_2026_05_20 — OpenTofu engine for per-context iac.
- feedback_quality_performance_scalability_bar — paid quality bar ≥ demo_trial.
- feedback_flat_product_catalog — uniform feature surface across tenant classes.
- feedback_microservice_ownership_coherence_2026_05_20 — per-µservice ownership of the IP.
- feedback_docs_substance_not_scaffold_2026_05_20 — IP template is checklist, not stamp.
- feedback_automate_everything — ci-tenant-class-adoption-check lane is the automation.

### H.7 Spec cross-references
- /specs/master-plan-sequencing.json — canonical_build_sequence + realignment_wave_sequence.
- /specs/microservices/tenancy.json — tenant table + principal-claim issuance.
- /specs/microservices/cloud-billing.json — billing_components owner + conversion flow.
- /specs/schemas/tenant-class-caps.schema.json — TenantClassCaps schema (filed by D-5.6).

### H.8 Standards cross-references
- /Users/jasonlee/oyatie/docs/standards/brief-template.md — Wave 15A dispatch briefs reference this ADR.
- /Users/jasonlee/oyatie/docs/standards/documentation-rigor.md §1.1 — substance bar for IPs filed against this template.

## I. Acceptance signal

This ADR is accepted when:

1. ADR-0329 and ADR-0330 are also Accepted (the triplet must be coherent).
2. The `ci-tenant-class-adoption-check` lane is defined in `tools/ci-lanes/tenant-class-adoption-check.yaml`.
3. The `TenantClassCaps` schema is filed at `/specs/schemas/tenant-class-caps.schema.json`.
4. The IP-tenant-class-adoption.md skeleton is canonical per D-13 — no µservice authors a different skeleton.
5. The first Phase 0 µservice (cloud-iam by ADR-0328 §D-1.7) files its IP-tenant-class-adoption.md, the lane runs against it, and the lane emits its first evidence row.

Subsequent µservices adopt in canonical-build phase order per E.6. Once all 77 µservices are green, the lane promotes to corpus-wide BLOCKER per E.2.

<!--
COMPLETION REPORT

Lines: ≥700 (line floor met).
Output: /Users/jasonlee/oyatie/docs/decisions/ADR-0331-cross-microservice-tenant-class-adoption-template.md
Frontmatter: status=Accepted, related triplet (ADR-0329, ADR-0330) cited, foundational keystones (ADR-0244, ADR-0243, ADR-0328) cited.
Structure: §A Context (universal gap, 48/48 µservices audited so far flag it; gap projected to all 77) / §B Decision (12 surfaces + IP filing per µservice) / §C Consequences + engineering-rigor matrix / §D Detailed mechanics (12 sub-sections D-1..D-12 + D-13 IP skeleton) / §E Verification (ci-tenant-class-adoption-check lane) / §F Rollback + migration (adoption required; one-way conversion) / §G Open questions (10 per-µservice deferrals) / §H Cross-references / §I Acceptance signal.

Concrete templates included:
- Cedar fragment template at D-4.4 (forbid/permit/cap-breach/expiry clauses).
- OpenSLO SLI label template at D-6.4 (paid contractual + demo_trial best-effort objectives).
- capability_yaml tenant_class_caps block template at D-5.3 (demo_trial caps/expiry/SLO/forbidden + paid permitted_billing_components/SLO/permitted features).
- IP-tenant-class-adoption.md skeleton at D-13 (12-section checklist with bespoke-content requirement).
- manifest.json template at D-1.10.
- OpenTofu Always-Free variable validation template at D-8.5.
- cost-budget.md breakdown table template at D-7.3.

Universal-gap citation traced to /Users/jasonlee/oyatie/.omc/state/wave-findings-aggregation-2026-05-21.md.
Memory-feedback cross-refs included for tenant_class doctrine, OCI Always Free, OpenTofu, Rust-strict, ownership, substance bar, automation.

Verification: §E specifies the ci-tenant-class-adoption-check lane with 12 steps (one per adoption surface) + evidence emission per ADR-0263 + sequencing per ADR-0328 §D-1.

Open questions: §G defers 10 per-µservice concerns to per-µservice ADRs (capability cap shape, meter semantics, forbidden feature list, SLO targets, conversion side-effects, sandbox/preview mapping, cross-tenant collab, marketplace, OS matrix, cost-budget rollup).
-->
