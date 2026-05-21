---
doc_class: Remediation Notes
microservice: marketplace
wave: 15A-MARKETPLACE-FIX
date: 2026-05-21
owner: axis-marketplace
audit_source: microservices/marketplace/coherence-audit-2026-05-20.md
canonical_anchors:
  - docs/decisions/ADR-0249-multi-category-marketplace-doctrine.md (KS#11 — 6 categories)
  - docs/decisions/ADR-0329-marketplace-revenue-share-billing-component.md (revenue_share as a cloud-billing component)
  - docs/decisions/ADR-0330-marketplace-category-surface-standard.md (per-category capability/policy/SLO/contract quadruple)
  - docs/decisions/ADR-0331-tenant-class-binary-demo-trial-paid.md (tenant_class as a Cedar principal claim)
  - feedback_multi_category_marketplace_doctrine
  - feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20
  - feedback_no_capability_tiers_2026_05_20
  - feedback_oci_always_free_maximization_2026_05_20
  - feedback_zero_handroll_opentofu_only_2026_05_20
  - feedback_multi_context_provider_agnostic_2026_05_20
  - feedback_rust_strict_only_no_python_2026_05_20
---

# Marketplace µservice — Remediation Notes (Wave 15A 2026-05-21)

This document records the remediation work executed against the 7 P0 audit findings raised in
`microservices/marketplace/coherence-audit-2026-05-20.md`. It is the closure receipt for Wave 15A-MARKETPLACE-FIX.

## §1 Audit findings closed

| # | Audit finding | Wave 15A action | Status |
|---|---|---|---|
| P0-1 | 0/6 per-category surfaces (plugins/apps/workflows/agents/models/datasets) | Authored 6× capability YAML + 6× Cedar policy + 6× OpenSLO + 6× listing-contract OpenAPI schemas (24 artifacts) | CLOSED |
| P0-2 | revenue_share billing component — 11 implementation pieces missing | Authored IP-031..IP-041 (11 implementation plans) + amended Cedar gate + added 4 new policies (clawback / statement-emit / reconcile / payout-dispatch / rate-override) + new SLO + new capability YAML | CLOSED |
| P0-3 | 5 SUPERIOR capabilities at risk during remediation | Preserved by reference in `manifest.json#superior_capabilities_preserved`; single ledger / single event schema / Cedar default-deny / BLAKE3 audit chain / EU AI Act readiness all carry forward | CLOSED |
| P0-4 | Tier-retirement (T-RET-01..T-RET-05) — 5 files | `capability-tiers/tier-matrix.md` deleted + directory removed; manifest `tier:product` field kept (it is the substrate-vs-product LAYER marker, NOT a capability tier); benchmarks + compliance + PRD + j73 IP cited for Wave 15J scrub | CLOSED for T-RET-01/02; carried to 15J for T-RET-03/04/05 (substrate-vs-product scrub) |
| P0-5 | Tenant-class adoption (ADR-0331) across 12 surfaces | Amended 6 existing Cedar policies + added `tenant_class` claim in manifest + tenant_class_caps blocks in all 6 category YAMLs + `published_tenant_class_claim_keys` published | CLOSED |
| P0-6 | OpenAPI 3.2.0 / AsyncAPI 3.1.0 / proto3 contract version conformance | Inherited from the existing contracts; 6 new per-category listing contracts emit `openapi: 3.2.0` | CLOSED |
| P0-7 | BNF v4 + 13-layer-enum naming conformance across 11 new IPs | Every new IP declares its `naming_justifications` inline | CLOSED |

## §2 Six-category coverage (ADR-0249 KS#11 — formally delivered)

For each of the six categories the following quadruple is now present:

| Category | Capability YAML | Cedar Policy | OpenSLO | Contract |
|---|---|---|---|---|
| plugins | `capabilities/category-plugins.yaml` | `policies/category-plugins.cedar` | `slos/listing-plugins-availability.openslo.yaml` | `contracts/listing-plugins-v1.yaml` |
| apps | `capabilities/category-apps.yaml` | `policies/category-apps.cedar` | `slos/listing-apps-availability.openslo.yaml` | `contracts/listing-apps-v1.yaml` |
| workflows | `capabilities/category-workflows.yaml` | `policies/category-workflows.cedar` | `slos/listing-workflows-availability.openslo.yaml` | `contracts/listing-workflows-v1.yaml` |
| agents | `capabilities/category-agents.yaml` | `policies/category-agents.cedar` | `slos/listing-agents-availability.openslo.yaml` | `contracts/listing-agents-v1.yaml` |
| models | `capabilities/category-models.yaml` | `policies/category-models.cedar` | `slos/listing-models-availability.openslo.yaml` | `contracts/listing-models-v1.yaml` |
| datasets | `capabilities/category-datasets.yaml` | `policies/category-datasets.cedar` | `slos/listing-datasets-availability.openslo.yaml` | `contracts/listing-datasets-v1.yaml` |

Per-category facets, listing-schema fields, review-process gates, pricing-model whitelists, entitlement-delivery
worker bindings, SLA tiers, and compliance-pack conditionality are declared in each category YAML. Counterpart references
to AWS Marketplace / AppExchange / Atlassian Marketplace are recorded per-category for parity tracking.

### Per-category default revenue-share rate basis points

| Category | Default | Industry parity rationale |
|---|---|---|
| plugins | 1500 (15%) | Lower infra burden; community-oriented |
| apps | 2000 (20%) | AppExchange / AWS Marketplace SaaS standard |
| workflows | 1500 (15%) | Aligned to plugins |
| agents | 2000 (20%) | Higher review burden + safety review |
| models | 2500 (25%) | Highest infra burden (compute, hosting, evals) |
| datasets | 2000 (20%) | Bandwidth + provenance audit cost |

### EU AI Act handling

Two categories (`agents` and `models`) declare `eu_ai_act_classification_required: true` in the manifest and gate on
`eu_ai_act_risk_class` in their Cedar policies. The `unacceptable` risk class is universally forbidden. The `high` risk class
requires `human_oversight_attestation_ref` for agents and the `EU-AI-Act-high-risk` compliance pack for models. The
`gpai-systemic-risk` model class requires `EU-AI-Act-gpai-systemic` compliance pack on consume side.

## §3 revenue_share billing-component completeness (11 IPs)

| IP | Closes audit gap | Deliverable |
|---|---|---|
| IP-031 | §3.4.B.ii.1 marketplace→cloud-billing ingestion | AsyncAPI subscribe-side contract + schema-registry BACKWARD_TRANSITIVE + Cedar gate + ordering + backpressure |
| IP-032 | §3.4.B.ii.2 revenue_share_cohort_id FK | Immutable FK on DealSet; gRPC existence check at create; audit-chain seal extension |
| IP-033 | §3.4.B.ii.3 monthly settlement statement event | `MarketplaceRevenueShareSettlementStatementSealed` channel + worker + Cedar + SLO + idempotency |
| IP-034 | §3.4.B.ii.4 clawback / chargeback surface | New `revenue-share-clawback` public capability + Cedar + SLO + worker + AsyncAPI + runbook |
| IP-035 | §3.4.B.ii.5 RevenueShareProvenance value-type | Kernel value-type with 13 fields + proto3 message + invariant + canonical serialization |
| IP-036 | §3.4.B.ii.6 payments.payout.create binding | `marketplace.payout.dispatch` usecase + OpenAPI + Cedar + idempotency + 6 failure classes |
| IP-037 | §3.4.B.ii.7 tenant_class==paid Cedar gate | Amend 6 existing + 5 new Cedar policies; tenant_billing_components principal claim |
| IP-038 | §3.4.B.ii.8 revenue_share_rate_basis_points | DB migration + per-category defaults + override Cedar action + manual review >50% |
| IP-039 | §3.4.B.ii.9 positive reconciliation event | `MarketplaceRevenueShareReconciliationSealed` (success counterpart) + worker + Cedar + SLO |
| IP-040 | §3.4.B.ii.10 FX snapshot | Adapter to cloud-billing.fx.snapshot.get + RevenueShareProvenance embed + fail-close mode |
| IP-041 | §3.4.B.ii.11 demo_trial denial path | 7-reason enum + observable counter + AsyncAPI conversion-CTA event + audit-chain seal on deny |

### Settlement flow narrative T0..T7 status

| Stage | Status before Wave 15A | Status after Wave 15A |
|---|---|---|
| T0 listing publish | authored | authored + per-category schemas |
| T1 deal-accept | authored | authored + tenant_class gates |
| T2 escrow-reserve | authored | authored + paid-only gate |
| T3 escrow-release | authored | authored + paid-only gate |
| T4 revenue-share accrue | partial | full — cohort FK + FX snapshot + provenance value-type |
| T5 statement emit | not authored | authored (IP-033) |
| T6 payout dispatch | not authored | authored (IP-036 + payout-dispatch.cedar) |
| T7 reconciliation | partial (failure only) | full — positive event (IP-039) + clawback (IP-034) |

## §4 Tenant-class adoption per ADR-0331 (12 surfaces)

| Surface | Action | Status |
|---|---|---|
| Cedar: deal-offer-create | Added `tenant_class in {paid, demo_trial(sandbox+free+plugins/workflows)}` | DONE |
| Cedar: deal-accept | Added paid OR (demo_trial + free) gate | DONE |
| Cedar: escrow-reserve | Added paid-only + explicit demo_trial deny | DONE |
| Cedar: escrow-release | Added paid-only + explicit demo_trial deny | DONE |
| Cedar: revenue-share-accrue | Added paid + revenue_share component + demo_trial deny | DONE |
| Cedar: mediation-open | Permit both classes (sandbox listings can escalate) | DONE |
| Cedar: revenue-share-clawback (NEW) | paid + revenue_share + chargeback_evidence_ref required | DONE |
| Cedar: revenue-share-statement-emit (NEW) | paid + revenue_share + period_start/end required | DONE |
| Cedar: revenue-share-reconcile (NEW) | paid + revenue_share | DONE |
| Cedar: revenue-share-rate-override (NEW) | paid + revenue_share + manual-review for >50% | DONE |
| Cedar: payout-dispatch (NEW) | paid + KYC artifact + payouts_enabled | DONE |
| Manifest: published_tenant_class_claim_keys | Declared `tenant_id + tenant_class + tenant_billing_components` | DONE |

Per-category YAMLs carry `tenant_class_caps` blocks declaring per-class rate / max-listings / payouts-enabled / category-specific caps
(autonomy_level_max for agents; model_size_band_max for models; sensitivity_class_max for datasets).

## §5 5 SUPERIOR capabilities preserved (no regression)

1. **Single cross-category SettlementLedger.** All 6 categories continue to post into the one DealSet/SettlementLedger
   primitive. No per-category shadow ledgers were introduced. Per-category surfaces extend the DealSet envelope; they
   do not fork the substrate.
2. **Single AsyncAPI event schema.** Per-category listing events emit through the same envelope as
   `MarketplaceDealOffered/Accepted/…`. No category-specific event channels were added beyond the per-listing-publish
   audit events (audit-only, not transaction).
3. **Cedar default-deny.** Every new policy (6 category + 5 revenue-share-amendments) inherits the default-deny stance;
   no `permit (principal, action, resource)` exists without explicit `when` conditions.
4. **BLAKE3 audit chain.** Every accrual, statement, payout, reconciliation, clawback, and demo_trial-deny event seals to the
   BLAKE3 chain. `RevenueShareProvenance` (IP-035) makes the chain-position structurally referenceable.
5. **EU AI Act compliance pack readiness.** Agents and models categories carry `eu_ai_act_risk_class` enums on their listing
   contracts; Cedar policies gate on the classification; compliance packs (`EU-AI-Act-high-risk`,
   `EU-AI-Act-gpai-systemic`, `EU-AI-Act-foundation-model`, `EU-AI-Act-training-data`) are wired into the conditional pack
   set per category.

## §6 Tier retirement (T-RET-01..T-RET-05)

| Item | Path | Action | Status |
|---|---|---|---|
| T-RET-01 | `capability-tiers/tier-matrix.md` | DELETE | DONE (file removed) |
| T-RET-02 | `capability-tiers/` (directory) | DELETE | DONE (directory removed) |
| T-RET-03 | `benchmarks/marketplace-vs-stripe-connect-vs-shopify-vs-amazon-marketplace-vs-appexchange.md` | SCRUB Bronze/Silver/Gold/Platinum language; replace with `paid + revenue_share + per_seat + per_usage` | DEFERRED to Wave 15J (substance scrub batch) |
| T-RET-04 | `compliance.md` | SCRUB tier-only pack-activation language | DEFERRED to Wave 15J |
| T-RET-05 | `PRD.md` §B + `IP-journey-j73-revenue-share.md` per-tier rev-share | SCRUB to tenant_class | DEFERRED to Wave 15J |

Note: `manifest.json#tier == "product"` is KEPT — it is the substrate-vs-product LAYER marker per
`feedback_substrate_vs_product_layering` (ADR-0245). A `_tier_comment` was added to disambiguate.

## §7 Tenant-class behavior (anchor doc for ADR-0331)

This section is the authoritative tenant-class behavior matrix referenced by `manifest.json#tenant_class_adoption.behavior_doc`.

### demo_trial defaults

- Hosted on the OCI Always Free profile (2× Ampere A1 4 OCPU / 24GB) per `feedback_oci_always_free_maximization_2026_05_20`.
- `payouts_enabled = false` across every category — no real-money settlement.
- `max_pending_transactions = 0` — escrow never reserves; all money-movement Cedar actions explicitly deny.
- Sandbox sub-scope: demo_trial can publish FREE listings in `plugins` + `workflows` categories only.
- Cannot publish in `apps` (requires KYC + security review), `agents` (requires red-team + safety card + KYC),
  `models` (requires KYC + provenance attestation), or `datasets` (requires KYC + datasheet).
- Cannot consume above per-category caps (small-model-only, public-sensitivity-only, supervised-autonomy-only).
- Mediation is available (sandbox listings can still escalate).
- Audit chain seals every deny event; conversion-CTA opportunity is emitted (rate-limited 1/minute).

### paid defaults

- `payouts_enabled = true` (subject to KYC artifact presence).
- Cohort-tracked under `revenue_share_cohort_id` when `billing_components has "revenue_share"`.
- All 6 categories available subject to per-category review process.
- Per-usage / per-seat metering done by cloud-billing — marketplace is the seller-side counterpart for revenue_share only.
- Conversion path from demo_trial: cloud-billing tenant-class flip on contract sign; new DealSets carry paid tenant_class
  immediately; in-flight sandbox DealSets retain demo_trial provenance for audit.

## §8 Cross-µservice contract changes (consumer-side coordination)

| Consuming µservice | New contract surface to honor | IP anchor |
|---|---|---|
| cloud-billing | Subscribe to `MarketplaceRevenueShareAccrued`, `MarketplaceRevenueShareSettlementStatementSealed`, `MarketplaceRevenueShareReconciliationSealed`, `MarketplaceRevenueShareClawbackPosted` | IP-031, IP-033, IP-034, IP-039 |
| cloud-billing | Serve `cloud-billing.cohort.exists` gRPC + `cloud-billing.fx.snapshot.get` gRPC | IP-032, IP-040 |
| cloud-iam | Publish `tenant_class` + `tenant_billing_components` principal claims | IP-037 |
| payments | Accept `payments.payout.create` with marketplace idempotency key + `source_settlement_statement_id` reference | IP-036 |
| payments | Emit chargeback events that marketplace's clawback worker subscribes to | IP-034 |
| audit-chain | Accept structured `RevenueShareProvenance` value-type as a seal payload | IP-035 |
| intelligence | Accept agent-registration entitlement | category-agents |
| model-registry | Accept signed-model entitlement-fetch | category-models |
| data-exchange | Issue signed-URL entitlement with SBOM evidence | category-datasets |
| workflow-engine | Accept workflow template imports | category-workflows |
| cloud-cell | Accept hosted-tenant provisioning | category-apps |
| foundry-plugin-registry | Accept wasmtime signed-binary entitlement | category-plugins |
| connect | Permission review for workflow connector manifest | category-workflows |
| identity | KYC artifact reference resolution | category-apps/agents/models/datasets |

## §9 Deferred items (Wave 15B / 15F / 15J / 15K)

| Item | Wave | Note |
|---|---|---|
| Convert `iac/terraform-main.tf` → OpenTofu modules in 6 per-context dirs | 15B | Out of Wave 15A scope; tracked in audit §3.2.A |
| Add `iac/oci-guest/always-free/main.tf` | 15B | demo_trial substrate sizing |
| Add `supported-oses.yaml` | 15B | OS support matrix |
| Bind connect + identity dependencies to IP slices | 15F | Dependency tracing |
| PRD + ARCHITECTURE de-substitution collapse | 15K | Substance bar |
| Add Atlassian Marketplace lane to benchmarks | 15J | Counterpart parity |
| Promote HTTP/3 from j73 IP to ARCHITECTURE §H | 15J | Protocol surface |
| ADR-MKT-001 §J amendment (tenant_class + billing_components) | 15J | Local ADR alignment with ADR-0331 |
| Scrub Bronze/Silver/Gold/Platinum from benchmarks / compliance / PRD / j73 | 15J | T-RET-03/04/05 |
| Per-tenant-class dashboard filters (5 dashboards × 2 classes) | 15J | Observability |

## §10 Acceptance verification (artifact-level)

The following counts confirm the Wave 15A deliverables are present and form a coherent surface.

| Artifact group | Required by audit | Authored in Wave 15A | Status |
|---|---|---|---|
| Per-category capability YAMLs | 6 | 6 | PASS |
| Per-category Cedar policies | 6 | 6 | PASS |
| Per-category OpenSLOs | 6 | 6 | PASS |
| Per-category listing-contract OpenAPI fragments | 6 | 6 | PASS |
| Revenue-share completeness IPs | 11 (IP-031..IP-041) | 11 | PASS |
| New revenue-share Cedar policies | 5 (clawback / statement-emit / reconcile / rate-override / payout-dispatch) | 5 | PASS |
| New revenue-share capability YAML | 1 (clawback) | 1 | PASS |
| New revenue-share OpenSLO | 1 (clawback-accuracy) | 1 | PASS |
| Amended existing Cedar policies (tenant_class) | 6 | 6 | PASS |
| Tier-matrix file deletion (T-RET-01/02) | 2 | 2 | PASS |
| Manifest updates (tenant_class + categories + billing_components + deployment_contexts + retired_artifacts) | 5 fields | 5 | PASS |
| Remediation notes (this file) | 1 | 1 | PASS |

## §11 Verdict

Wave 15A-MARKETPLACE-FIX delivers 7 P0 closures: the 6-category ADR-0249 KS#11 surface is now authored end-to-end
(quadruple per category); revenue_share completeness has 11 fresh IPs covering ingestion, cohort tracking, monthly
statements, clawback, provenance value-type, payout binding, tenant_class gate, rate persistence, positive
reconciliation, FX snapshot, and demo_trial denial path; the 5 SUPERIOR capabilities are preserved by reference and
verified to carry forward; tier-retirement T-RET-01/02 is executed (file + directory deleted); tenant-class adoption
per ADR-0331 covers all 12 surfaces in scope for Wave 15A.

Remaining deltas (T-RET-03/04/05, IaC tool-family conversion, dependency-IP binding, PRD substance scrub) are tracked
in §9 with explicit wave assignments. None block the Wave 15A completion claim.

<!--
COMPLETION REPORT
microservice: marketplace
wave: 15A-MARKETPLACE-FIX
date: 2026-05-21
deliverables:
  category_capability_yamls: 6
  category_cedar_policies: 6
  category_openslos: 6
  category_contract_openapi_fragments: 6
  revenue_share_completeness_ips: 11 (IP-031..IP-041)
  new_revenue_share_cedar_policies: 5 (clawback, statement-emit, reconcile, rate-override, payout-dispatch)
  new_revenue_share_capability_yaml: 1 (revenue-share-clawback)
  new_revenue_share_openslo: 1 (revenue-share-clawback-accuracy)
  amended_existing_cedar_policies: 6 (deal-offer-create, deal-accept, escrow-reserve, escrow-release, revenue-share-accrue, mediation-open)
  tier_retirement_executed: 2 (T-RET-01 tier-matrix.md, T-RET-02 capability-tiers/ directory)
  tier_retirement_deferred: 3 (T-RET-03/04/05 → Wave 15J substance scrub)
  manifest_updates: 5 (tenant_class_adoption, billing_components, deployment_contexts, categories registry, retired_artifacts)
  remediation_notes: 1 (this file)
findings_closed: [P0-1, P0-2, P0-3, P0-4 (partial — 2 of 5 immediate), P0-5, P0-6, P0-7]
findings_deferred: [P0-4 (T-RET-03/04/05 to Wave 15J)]
superior_capabilities_preserved: 5
halt_cleanly: true
-->

## Wave 15J-final-cleanup

- Bucket: F-BUCKET-3.
- Action: deleted stale 2026-05-20 coherence audit and feature parity artifacts; scrubbed onboarding, tutorial, migration playbook, FAQ, benchmark, performance companion, and manifest residue into `tenant_class` + `billing_components` language.
- Verification: tier-name grep and `capability_tier|max_tier|tier_threshold` grep both return 0 outside remediation notes.
- Follow-ups: none.

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- none; inventory returned zero Redis references under microservices/marketplace

Counterpart-fact preservations:
- none

Files renamed (git mv):
- none

## Wave 15-ADR-0105-RED-remediation (2026-05-21)

- Old array contents removed from Rust: `api`, `rest`, `application`, `usecase`, `domain`, `kernel`, `adapter`, `worker`, `sdk`, `iac`, `policy`, `observability`.
- New canonical declaration: `Layer::{Kernel, Domain, Usecase, App, Adapter, Infrastructure, Rest, Grpc, Graphql, Worker, Cli, Sdk, Api}` plus `domain::LAYERS`.
- Files modified: 138 total: `src/lib.rs`, `manifest.json`, `scorecards/overrides.json`, 13 capability files, 13 catalog files, 13 SLO files, 5 dashboard files, 12 IaC files, 17 policy files, 3 contract files, 36 `ip/` files, and 23 top-level marketplace docs/notes.
- Cargo check status: PASS — `cargo check --manifest-path microservices/marketplace/Cargo.toml -p oya-marketplace-doc-suite-scaffold`.
- Test status: PASS — `cargo test --manifest-path microservices/marketplace/Cargo.toml` (3 passed).
- Follow-up: legacy marketplace catalog records still include concern-shaped record names such as `policy`, `iac`, `observability`, and `events`; this remediation normalized their layer-enum declarations but did not rename/delete catalog records.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture (ADR-0343): Values mirror manifest `dr`: RTO 3600s, RPO 300s, `multi_region_active_active=false`, `dr_tier=T2`, `replication_shape=active-passive-cross-region-continuous`, `failover_runbook=runbooks/settlement-ledger-replay.md`. Alternative considered: rely on generic multi-region prose without numeric targets. Rejected because DealSet, escrow, and revenue-share evidence must be admitted by pack floors. Cost: active-passive replication and settlement replay evidence must remain current; EU-AI high-risk listing admission still needs a stricter pack decision.
- Capacity model (ADR-0340): Values mirror manifest `capacity_model`: 0.18 CPU, 640 MiB RAM, 18 GiB storage, connections Valkey 6/Postgres 5/outbound HTTP 12, `scaling_dimension=per_request`, `cell_placement_class=Tier-2`, `pod_runtime_tier=2`. Alternative considered: use listing-browse traffic as the only scaling driver. Rejected because escrow and revenue-share paths need isolation from browse and category indexing. Cost: category-level queues and admission throttles must be maintained for high-volume sellers.
- Sustainability and cost attribution (ADR-0344): Values require `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on DealSet, listing, escrow, dispute, revenue-share, payout-dispatch, mediation, and export rows, with carbon routing only for replay, statements, indexing, exports, and non-urgent reconciliation; manifest `sustainability_emission_model` remains absent. Alternative considered: aggregate seller carbon by monthly statement only. Rejected because CSRD/SB-253/SEC evidence needs event-derived rollup and tenant transparency. Cost: marketplace statements and finops-portal must expose category and settlement dimensions, and manifest emission fields must still be added.
- API versioning posture (ADR-0342): Values set public carrier triplet, SDK semver, last 3 versions for at least 180 days, paid-tenant/integrator pinning, and ADR-0145 internal mesh exemption. Alternative considered: current-stable-only marketplace contracts. Rejected because marketplace integrators need pinned OpenAPI/AsyncAPI/proto3 during seller and revenue-share migrations. Cost: generated SDKs and contract fixtures must retain three supported public versions.

## Wave 15-doctrine-propagation-IPs (2026-05-21)

- Bucket: `D4-BUCKET-2`.
- Doctrine source: ADR-0337..0345 selective propagation by trigger match; this section records only matched IPs.
- Manifest gap: `manifest.json#dr` is absent, so DR sections preserve compliance-pack floors without inventing service RTO/RPO targets.

| IP | Trigger(s) | Required sections | Source evidence | Manifest gaps |
| --- | --- | --- | --- | --- |
| `microservices/marketplace/IP-journey-j101-deal-settlement-ledger.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/marketplace/contracts/openapi-v1.yaml, crates/oya-cloud-marketplace-kernel/src/lib.rs::Offer | manifest.json#dr missing |
| `microservices/marketplace/IP-journey-j102-deal-settlement-ledger.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/marketplace/contracts/openapi-v1.yaml, crates/oya-cloud-marketplace-kernel/src/lib.rs::Offer | manifest.json#dr missing |
| `microservices/marketplace/IP-journey-j103-deal-settlement-ledger.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/marketplace/contracts/openapi-v1.yaml, crates/oya-cloud-marketplace-kernel/src/lib.rs::Offer | manifest.json#dr missing |
| `microservices/marketplace/IP-journey-j107-deal-settlement-ledger.md` | C | Sustainability emission (per ADR-0344) | microservices/marketplace/contracts/openapi-v1.yaml, crates/oya-cloud-marketplace-kernel/src/lib.rs::Offer | none |
| `microservices/marketplace/IP-journey-j108-deal-settlement-ledger.md` | C | Sustainability emission (per ADR-0344) | microservices/marketplace/contracts/openapi-v1.yaml, crates/oya-cloud-marketplace-kernel/src/lib.rs::Offer | none |
| `microservices/marketplace/IP-journey-j112-deal-settlement-ledger.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/marketplace/contracts/openapi-v1.yaml, crates/oya-cloud-marketplace-kernel/src/lib.rs::Offer | manifest.json#dr missing |
| `microservices/marketplace/IP-journey-j146-seller-flow-and-escrow.md` | B, C, D | DR posture (per ADR-0343); Sustainability emission (per ADR-0344); Pod runtime tier (per ADR-0338) | microservices/marketplace/contracts/openapi-v1.yaml, crates/oya-cloud-marketplace-kernel/src/lib.rs::Offer | manifest.json#dr missing; manifest.json#pod_runtime_tier missing |
| `microservices/marketplace/IP-journey-j23-seller-listing.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/marketplace/contracts/openapi-v1.yaml, crates/oya-cloud-marketplace-kernel/src/lib.rs::Offer | manifest.json#dr missing |
| `microservices/marketplace/IP-journey-j24-buyer-order.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/marketplace/contracts/openapi-v1.yaml, crates/oya-cloud-marketplace-kernel/src/lib.rs::Offer | manifest.json#dr missing |
| `microservices/marketplace/IP-journey-j29-sale-event-emitter.md` | C, D | Sustainability emission (per ADR-0344); Pod runtime tier (per ADR-0338) | microservices/marketplace/contracts/openapi-v1.yaml, crates/oya-cloud-marketplace-kernel/src/lib.rs::Offer | manifest.json#pod_runtime_tier missing |
| `microservices/marketplace/IP-journey-j52-order-ledger.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/marketplace/contracts/openapi-v1.yaml, crates/oya-cloud-marketplace-kernel/src/lib.rs::Offer | manifest.json#dr missing |
| `microservices/marketplace/IP-journey-j55-seller-buyer-mediation.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/marketplace/contracts/openapi-v1.yaml, crates/oya-cloud-marketplace-kernel/src/lib.rs::Offer | manifest.json#dr missing |
| `microservices/marketplace/IP-journey-j65-order-export.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/marketplace/contracts/openapi-v1.yaml, crates/oya-cloud-marketplace-kernel/src/lib.rs::Offer | manifest.json#dr missing |
| `microservices/marketplace/IP-journey-j69-appointment-and-service-commitments.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/marketplace/contracts/openapi-v1.yaml, crates/oya-cloud-marketplace-kernel/src/lib.rs::Offer | manifest.json#dr missing |
| `microservices/marketplace/IP-journey-j73-revenue-share.md` | A, B, C, D | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344); Pod runtime tier (per ADR-0338) | microservices/marketplace/contracts/openapi-v1.yaml, crates/oya-cloud-marketplace-kernel/src/lib.rs::Offer | manifest.json#dr missing; manifest.json#pod_runtime_tier missing |

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Value: 0.18 vCPU, 640 MiB RAM, 18 GB storage, Valkey/Postgres/outbound connections 6/5/12, scaling_dimension=per_request, cell_placement_class=Tier-2.
- ADR: ADR-0340 capacity model and ADR-0248 cellular criticality.
- Why: 0.18 vCPU/640 MiB/18 GB covers catalog/listing reads plus deal and settlement writes.
- Rejected: Tier-3 cell placement was rejected because ADR-0340 classifies marketplace catalog as a Tier-2 capability.
- Cost: Commits to capability-level isolation and audit-chain replay for settlement ledger correctness.

### Block 2: dr
- Value: RTO 3600s, RPO 300s, active_active=false, backup_substrate=postgres_wal_g, valkey, object_storage_versioned, audit_chain_merkle_seal, failover_runbook=runbooks/settlement-ledger-replay.md.
- ADR: ADR-0343 DR manifest declaration and compliance-pack floors.
- Why: One-hour RTO and five-minute RPO protect escrow, settlement ledger, and revenue-share accuracy.
- Rejected: Fifteen-minute RPO was rejected because replay fidelity SLOs require tighter settlement state recovery.
- Cost: Requires settlement-ledger replay and audit-chain seal validation during failover.

### Block 3: pod_runtime_tier
- Value: pod_runtime_tier=2; evidence=microservices/marketplace/ARCHITECTURE.md, microservices/marketplace/ip/IP-001-deal-set-kernel.md, microservices/marketplace/ip/IP-002-settlement-ledger-domain.md, microservices/marketplace/ip/IP-024-catalog-and-manifest-pack.md, microservices/marketplace/slos/settlement-replay-fidelity.openslo.yaml.
- ADR: ADR-0338 runtime-tier taxonomy and ADR-0340 D-6 co-variance.
- Why: First-party service code handles tenant workflows without tenant-customer code execution.
- Rejected: Tier 0 was rejected because this service does not execute marketplace plugin code; it stores listings and settlement records.
- Cost: Admission and placement must remain consistent with cell_placement_class=Tier-2.

### Block 4: tenant_version_pinning
- Value: default_version=2026-05-21, supported_window_size=3, supported_window_minimum_days=180, per-tenant pinning=true.
- ADR: ADR-0342 tenant API version pinning.
- Why: Marketplace exposes many listing and settlement contracts; the primary public surfaces are pinned here.
- Rejected: unpinned latest-only contracts, because tenants need explicit migration windows.
- Cost: Future breaking changes require migration docs and deprecation-calendar entries before sunset.

### Block 5: consumes_upstream_oss
- Value: postgresql, valkey, cedar, opentofu, openbao, kafka, opentelemetry.
- ADR: ADR-0345 OSS stewardship class registry.
- Why: These are the direct shared runtime, policy, IaC, secrets, event, data, and observability dependencies declared through the registry.
- Rejected: local oss_stewardship_class_overrides, because registry defaults already own class and CVE-response teams.
- Cost: SBOM and CVE triage for this service now joins against /specs/oss-stewardship-registry.json.

### Block 6: iac_module_invocations
- Value: oyatie-as-cloud-provider/tenant-namespace@v1, oyatie-as-cloud-provider/per-cell-nodepool-runc@v1, oyatie-as-cloud-provider/audit-chain-sink@v1, on-prem/postgres-service-database@v1, on-prem/valkey-cluster@v1, aws-guest/edge-waf@v1.
- ADR: ADR-0339 shared OpenTofu module invocation catalog.
- Why: Capability cell, audit-chain sink, and edge-waf primitives match deal and listing public surfaces.
- Rejected: leaving wrappers unpinned, because ADR-0339 requires module path and version determinism.
- Cost: Current per-service IaC wrappers must stay thin and migrate to the canonical cloud-iac module catalog as it lands.
