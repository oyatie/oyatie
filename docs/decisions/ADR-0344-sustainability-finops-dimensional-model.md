---
id: ADR-0344
title: Sustainability + finops dimensional model (per-call CO2-grams + watt-hours + USD-cost emitted alongside every audit row; finops-portal dimensional rollup per tenant/product/capability/provider/cell/compliance-pack)
status: Accepted
planning_impact: true
date: 2026-05-21
owner_team:
  - council-architecture
  - council-legal
  - ops-finops
  - ops-sre-reliability
  - axis-audit-chain
  - axis-observability
  - ops-compliance
owners:
  - council-architecture
  - council-legal
  - ops-finops
  - ops-sre-reliability
  - axis-audit-chain
  - axis-observability
  - ops-compliance
supersedes: []
superseded_by: []
amends:
  - ADR-0174-finops-cost-attribution-chargeback.md (cost-attribution tag block widened: every audit row carries cost_usd_minor_units + co2_grams + watt_hours + provider + region in addition to the existing tenant_id + cell_id + microservice + plane + environment + cost_center + sustainability_class tags; chargeback formula gains the carbon-cost and energy-cost factor; anomaly thresholds gain carbon-spike + watt-hours-creep)
  - ADR-0263-observability-emission-contract.md (mandatory event envelope extended: every audit-chain row emits cost_usd_minor_units + co2_grams + watt_hours + provider + region computed at HLC-timestamped emission time; D-11 per-tenant cost attribution emission section is extended with the dimensional rollup contract in §D below)
  - ADR-0028-cloud-microservice-architecture.md (the cloud-* microservice substrate gains a per-call carbon + energy + cost attribution responsibility; oya-cloud-finops-* lanes pick up the new dimensional rollup)
  - ADR-0212-buildability-doctrine.md (per-microservice manifest gains a top-level `sustainability_emission_model` block declaring how the microservice computes per-call CO2-grams / watt-hours from its workload signal: pod_runtime_tier per ADR-0338 + cpu-seconds + memory-byte-seconds + storage-byte-hours + network-bytes + per-region carbon intensity)
  - ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md (substance-bar discipline applies: per-microservice sustainability_emission_model authoring is bespoke per microservice; template-stamping forbidden)
  - ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md (Wave 15Y-Sustainability-FinOps added as a sub-wave that authors the per-microservice sustainability_emission_model declarations + the finops-portal dimensional rollup + the carbon-aware-scheduling Cedar context extension)
related_adrs:
  - ADR-0028-cloud-microservice-architecture.md
  - ADR-0099-data-class-registry.md
  - ADR-0108-sunset-lifecycle-automation.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0174-finops-cost-attribution-chargeback.md
  - ADR-0181-cosign-signed-artifacts-and-modules.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0215-multi-context-platform.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0246-mls-rfc-9420-e2ee-personal-messenger.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0249-multi-category-marketplace-doctrine.md
  - ADR-0250-build-ahead-of-certification.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0252-time-coordination-distributed-consistency.md
  - ADR-0253-network-topology-edge-service-mesh.md
  - ADR-0254-deployment-model-spectrum.md
  - ADR-0255-intelligence-as-two-layer-ai-substrate.md
  - ADR-0263-observability-emission-contract.md
  - ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md
  - ADR-0324-anti-script-authoring-doctrine.md
  - ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md
  - ADR-0329-tier-system-retired-replaced-by-tenant-class.md
  - ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components.md
  - ADR-0331-cross-microservice-tenant-class-adoption-template.md
  - ADR-0335-intelligence-microservice-consolidation.md
  - ADR-0336-valkey-not-redis-substrate.md
  - ADR-0337-iceberg-canonical-olap-write-path.md
  - ADR-0338-pod-runtime-tier-0-3.md
  - ADR-0339-shared-iac-module-library.md
  - ADR-0340-capacity-model-per-microservice.md
  - ADR-0341-cellular-promotion-gates-explicit.md
  - ADR-0342-api-versioning-hybrid-date-public-semver-sdk.md
  - ADR-0343-dr-rto-rpo-matrix-per-microservice-per-compliance-pack.md
related_specs:
  - /specs/audit-event-schema.json
  - /specs/audit-event-class-registry.json
  - /specs/finops-dimensional-model.json
  - /specs/finops-cost-attribution.json
  - /specs/master-plan-sequencing.json
  - /specs/microservices/manifest-schema.json
  - /specs/microservices/finops-portal.json
  - /specs/microservices/observability.json
  - /specs/microservices/audit-chain.json
  - /specs/markdown-retirement-policy.json
related_memory:
  - feedback_six_candidate_adrs_2026_05_21
  - feedback_quality_performance_scalability_bar
  - feedback_no_silent_regression
  - feedback_multi_context_provider_agnostic_2026_05_20
  - feedback_oci_always_free_maximization_2026_05_20
  - feedback_clean_architecture_requirements
  - feedback_microservice_ownership_coherence_2026_05_20
  - feedback_rust_strict_only_no_python_2026_05_20
  - feedback_bominal_inheritance_precedence
  - feedback_docs_substance_not_scaffold_2026_05_20
  - feedback_drift_too_big_2026_05_20
  - feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20
companion_docs:
  - docs/standards/dependency-policy.md
  - docs/standards/finops-dimensional-rollup.md
  - docs/standards/carbon-accounting-methodology.md
  - microservices/finops-portal/ARCHITECTURE.md
  - microservices/audit-chain/ARCHITECTURE.md
  - microservices/observability/ARCHITECTURE.md
  - microservices/cloud-finops/ARCHITECTURE.md
  - tools/hooks/_canonical-primitives.md
inbound_citations:
  - /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_six_candidate_adrs_2026_05_21.md
doc_class: Architecture-Decision-Record
shape: Decision
authority_tier: 1
line_floor: 600
bespoke_authoring_requirement: documentation-rigor-1.1-plus-ADR-0322
enforcement_status: advisory-until-per-microservice-sustainability-emission-model-declared
enforced_by:
  - oya-check-audit-row-carbon-fields (new lane; advisory until crate lands; planned to refuse audit-chain emission paths that do not populate cost_usd_minor_units + co2_grams + watt_hours + provider + region; promoted to BLOCKER after Wave 15Y-Sustainability-FinOps lands)
  - oya-check-sustainability-emission-model (new lane; advisory until crate lands; planned to refuse µservice manifests that lack sustainability_emission_model block once a workload is emitted)
  - oya-check-finops-portal-dimensional-rollup (new lane; advisory until crate lands; planned to refuse finops-portal dashboards / reports that do not expose all six canonical dimensional axes — tenant / product / capability / provider / cell / compliance_pack)
  - oya-check-carbon-intensity-provider-binding (new lane; advisory until crate lands; planned to refuse microservice runtime that resolves region carbon intensity from a non-canonical provider; electricityMaps API is canonical, with documented fallback grid-average per ADR-0263)
  - oya-check-regulatory-sustainability-report-emission (new lane; advisory until crate lands; planned to refuse regulator-class quarterly audit-chain reports — CSRD / SB-253 / SEC-climate-disclosure — that lack carbon + energy + cost totals across the dimensional axes)
  - oya-governance-sustainability-tag-allowlist (existing ADR-0174 sustainability_class lane; preserved unchanged)
  - oya-governance-carbon-aware-scheduling (new lane; refuses Cedar fragments that gate workload placement on carbon intensity without referencing this ADR + ADR-0243)
purpose: >
  Establish the canonical sustainability + finops dimensional model for
  Oyatie: every audit-chain row emitted under ADR-0263 MUST carry, at
  emission time and HLC-timestamped per ADR-0252, the five-field
  sustainability-and-finops tuple { cost_usd_minor_units, co2_grams,
  watt_hours, provider, region } computed from the µservice's declared
  sustainability_emission_model (per-microservice manifest field). The
  finops-portal microservice owns the canonical dimensional rollup
  surface, exposing per-tenant, per-product, per-capability, per-provider,
  per-cell, and per-compliance-pack aggregations of cost + carbon +
  energy with HLC-timestamped commit ordering. Region carbon intensity
  is resolved at emission time from the electricityMaps API (canonical
  provider) with a documented fallback to provider-published
  grid-average carbon intensity. The model serves five named driving
  use cases — CSRD (EU Corporate Sustainability Reporting Directive,
  first reports 2025-2026 for large companies), SB-253 (California
  Climate Corporate Data Accountability Act, first reports 2026), SEC
  climate disclosure rule (US public companies), per-tenant cost +
  carbon transparency (in-product reporting per ADR-0174), provider
  routing decisions (cost + carbon as Cedar context inputs at the
  Istio Ambient ext_authz gate per ADR-0243), and carbon-aware
  scheduling (workloads delayed to low-carbon hours when the SLO error
  budget per ADR-0341 + the compliance pack RTO/RPO floor per ADR-0343
  permits the deferral). The ADR joins ADR-0174 (sustainability tag),
  ADR-0252 (HLC for accurate emission ordering), and ADR-0263
  (observability emission contract). Out of scope: actual carbon
  intensity provider integration (electricityMaps SDK binding) and
  per-microservice emission instrumentation; both sequenced as
  Wave 15Y-Sustainability-FinOps follow-on under ADR-0328 batch
  discipline. This ADR authors the contract; downstream waves
  instrument the µservices.
---

> **Disposition light-edit (2026-08-06):** Sustainability + FinOps dimensions

# ADR-0344: Sustainability + finops dimensional model (per-call CO2-grams + watt-hours + USD-cost emitted alongside every audit row; finops-portal dimensional rollup per tenant/product/capability/provider/cell/compliance-pack)

## Status

Proposed on 2026-05-21.

This ADR is the canonical sustainability-and-finops-substrate-shape decision establishing five new fields on every audit-chain row emitted under ADR-0263 — `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region` — and establishing the finops-portal microservice as the owner of the canonical dimensional rollup surface across six axes (tenant, product, capability, provider, cell, compliance-pack).

It runs in coordination with the in-flight 2026-05-21 realignment effort: ADR-0337 (Iceberg canonical OLAP), ADR-0338 (Pod runtime tier 0..3), ADR-0339 (shared IaC module library), ADR-0340 (per-microservice capacity model), ADR-0341 (cellular promotion gates), ADR-0342 (API versioning hybrid), and ADR-0343 (DR + RTO/RPO matrix per microservice per compliance pack) are sibling decisions from the same `/idea-refine` + interview sessions captured in `feedback_idea_refine_decisions_2026_05_21` and `feedback_six_candidate_adrs_2026_05_21`. This ADR is the fifth of the second six-candidate batch (ADR-0340 through ADR-0345).

It directly amends ADR-0174 (FinOps cost attribution + chargeback) by widening the cost-attribution tag block to include the per-call carbon and energy fields and by adding the carbon-cost factor to the chargeback formula. It directly amends ADR-0263 (observability emission contract) by extending the mandatory event envelope with the five sustainability-and-finops fields and by binding the §D-11 per-tenant cost attribution emission section to the dimensional rollup contract in §D below. It is binding on every µservice that emits audit-chain rows.

Enforcement transitions from `advisory-until-per-microservice-sustainability-emission-model-declared` to `BLOCKER` per the lane sequence in §E below: at landing of Wave 15Y-Sustainability-FinOps (which authors the per-microservice sustainability_emission_model blocks and the finops-portal dimensional rollup surface), the `oya-check-audit-row-carbon-fields` lane promotes to BLOCKER for new emission paths; per-µservice migration of legacy emission paths follows the µservice's canonical-build phase order under ADR-0328.

The decision does not change the canonical audit-chain commit protocol per ADR-0263. The decision does not change the existing ADR-0174 sustainability_class tag (`pue-gte-1-2` / `pue-1-2-to-1-1` / `pue-lt-1-1`); that tag is preserved verbatim as a per-cell PUE-class marker and is orthogonal to the per-call carbon emission introduced here. The decision does not change which microservice owns which capability. The decision does not change tenant_class semantics per ADR-0330. The decision does not change Cedar evaluation surface per ADR-0243; carbon-aware scheduling adds Cedar context inputs (cost_per_hour + co2_grams_per_hour + region carbon intensity tier) but does not change the gate shape. The decision does not retire any existing observability metric or audit-event class.

## Date

2026-05-21.

## Context

### A.1 Named pressure: regulatory wall arriving 2025-2026 across three major jurisdictions

Three named regulatory pressures land within the same 18-month window and converge on the same demand: per-call, per-tenant, per-product, per-region carbon and energy disclosure with auditor-grade evidence.

**CSRD — EU Corporate Sustainability Reporting Directive.** The CSRD entered into force on 2023-01-05 and applies in waves: large EU-listed companies file first reports for fiscal year 2024 (filed in 2025); large EU non-listed companies file for FY 2025 (filed in 2026); listed SMEs file for FY 2026 (filed in 2027). The European Sustainability Reporting Standards (ESRS) under the CSRD require Scope 1, Scope 2, and Scope 3 emissions disclosure with double materiality assessment and third-party limited-assurance audit. The Scope 3 disclosure requirement (purchased goods and services, including cloud services) is the binding requirement for Oyatie tenants and for Oyatie itself: an Oyatie tenant operating in the EU MUST be able to attribute its Oyatie consumption to a Scope 3 emissions line, which requires Oyatie to publish per-tenant per-period emissions evidence. Without per-call attribution at emission time, this is reconstruction-after-the-fact, which fails the limited-assurance audit posture.

**SB-253 — California Climate Corporate Data Accountability Act.** Signed into California law on 2023-10-07. Applies to entities doing business in California with annual revenue greater than 1 billion USD. Requires Scope 1 + Scope 2 disclosure starting fiscal year 2025 (filed 2026) and Scope 3 disclosure starting fiscal year 2026 (filed 2027). Third-party assurance ramps from limited to reasonable assurance by 2030. The disclosure must be in line with the Greenhouse Gas Protocol Corporate Accounting and Reporting Standard. Per the GHG Protocol Scope 3 Standard Category 1 (Purchased goods and services), cloud-service consumption is an in-scope Scope 3 emissions line — the same posture as CSRD.

**SEC climate disclosure rule.** The U.S. Securities and Exchange Commission adopted the "Enhancement and Standardization of Climate-Related Disclosures for Investors" rule on 2024-03-06. The rule requires registrants to disclose material climate-related risks, governance, GHG emissions (Scope 1 + Scope 2 for large accelerated filers and accelerated filers, Scope 3 if material and used in target-setting), and financial-statement footnote impacts. The rule is in litigation hold as of 2024-04-04 (stayed pending Eighth Circuit consolidated review), but the substantive disclosure expectation already shapes public-company reporting practice for tenants of Oyatie that are SEC-reporting companies. Oyatie tenants in this class need per-tenant carbon evidence at fiscal-year close.

The three regimes share a common shape: per-tenant, per-period, per-region carbon evidence with auditor-grade traceability. The shape is not reconstructable from cloud-provider bills alone — bills do not carry per-tenant attribution inside a multi-tenant SaaS, and bills do not carry HLC-ordered emission timestamps, and bills do not carry per-call dimensional context. The shape is only constructable by emitting carbon + energy + cost at the audit-chain emission moment, on the same HLC clock as the audit row itself, with the same dimensional tags.

### A.2 Named pressure: counterpart precedent in hyperscaler sustainability dashboards

Hyperscalers have published per-tenant sustainability dashboards over the 2021-2025 window. The counterpart shape is consistent across vendors:

- **AWS Customer Carbon Footprint Tool.** Launched 2022-03-01. Provides per-AWS-account monthly Scope 1, Scope 2 (market-based and location-based), and Scope 3 (limited) emissions estimates with 90-day lag. Coverage: EC2, S3, RDS, EBS, Lambda, ECS, EKS. Methodology: aggregates account-level resource consumption against AWS-published grid-average carbon intensity per region.
- **Google Cloud Carbon Footprint.** Launched 2021-10-12. Provides per-billing-account monthly gross location-based and market-based emissions. Coverage: all of Google Cloud. Methodology: aggregates consumption against Google-published per-region per-product carbon intensity. Provides hourly carbon intensity feeds via the Carbon Free Energy Hourly Match (CFEHM) product.
- **Microsoft Azure Emissions Impact Dashboard.** Launched 2020-11-12 (originally as the Microsoft Sustainability Calculator). Provides per-subscription monthly Scope 1, Scope 2 (location-based and market-based), and Scope 3 emissions. Coverage: all of Azure. Methodology: aggregates against Microsoft-published per-region carbon intensity.
- **Oracle Cloud Sustainability Dashboard.** Launched 2023-09-18. Provides per-tenancy monthly emissions estimates with per-region grid-average factors. Coverage: OCI. Important for Oyatie because the OCI guest-on-oci context is a first-class deployment per `feedback_oci_always_free_maximization_2026_05_20`.
- **Snowflake Carbon Insights.** Launched 2023-06-26. Provides per-Snowflake-account monthly emissions estimates broken down by workload type (warehouse compute, storage, transfer). Methodology: aggregates Snowflake-internal consumption metrics against per-region grid-average carbon intensity.
- **Salesforce Sustainability Cloud (now Net Zero Cloud).** Launched 2021-10-13 as a tenant-facing analytics surface for Scope 1/2/3 emissions. The relevance here is the consumer-facing per-tenant dashboard pattern that Oyatie's finops-portal must match.

The counterpart shape is: per-tenant aggregation, per-period (typically monthly), per-region carbon intensity, per-product (or per-service) breakdown, dashboard surface plus regulator-export-ready evidence file. The hyperscaler precedent landed on 30-90 day lag because per-call attribution was retrofitted onto pre-existing billing systems; Oyatie builds carbon + cost at emission time (`feedback_build_ahead_of_certification`, ADR-0250) and avoids the retrofit. Oyatie's finops-portal can therefore expose **per-call, per-tenant, near-real-time** rollups instead of the 90-day-lag rollups that hyperscalers ship.

### A.3 Named pressure: per-call attribution is not optional under Scope 3 with limited assurance

The GHG Protocol Scope 3 Standard requires reporters to disclose the methodology underpinning each line item. Under limited-assurance audit (CSRD baseline, SB-253 baseline 2026-2029, SEC if reasonable assurance is reached), the auditor evaluates the methodology and tests the underlying evidence chain. Per-month roll-ups derived from end-of-month cloud bills produce a methodology in the "spend-based" category of the GHG Protocol Spend-Based vs Activity-Based methods. Spend-based methods carry the highest uncertainty grade.

Activity-based methods require per-activity evidence: kWh consumed, hours used, vCPU-seconds executed, GB-hours stored, GB transferred. Under multi-tenant SaaS the activity must be attributed to the tenant that triggered it. Without per-call attribution at emission time, the activity-based method is unauditable for the SaaS-tenant tier; the auditor cannot trace the kWh back to the tenant whose request produced it.

Oyatie's audit-chain per ADR-0263 already emits per-call audit rows with HLC-ordered timestamps + tenant_id + cell_id + sub_scope_path + microservice + plane. Adding cost_usd_minor_units + co2_grams + watt_hours + provider + region to the same row is the cheapest possible incremental cost to produce activity-based, per-tenant, auditor-grade evidence. The marginal storage cost is bounded at ~40 bytes per audit row (five additional fields). The marginal compute cost is bounded at ~one carbon-intensity lookup per emission, cached per (region, hour) tuple to amortize across thousands of calls.

### A.4 Named pressure: per-tenant cost transparency is already required by ADR-0174

ADR-0174 (FinOps cost attribution + chargeback) established per-tenant cost attribution as a first-class concern: every cloud resource carries the tag block { tenant_id, cell_id, microservice, plane, environment, cost_center, sustainability_class }, and `oya-cloud-billing-domain::chargeback::compute_period()` aggregates labelled spend + capability invocation + audit-chain emission + storage + credits into a per-tenant chargeback. This ADR does not replace ADR-0174; it widens ADR-0174's per-resource tag block into a per-call audit row with two added dimensions (carbon, energy) plus two added scope dimensions (provider, region). The widening is additive; ADR-0174's tag block at provisioning time is preserved verbatim.

The reason for widening to per-call rather than continuing to operate at per-resource resolution is that per-resource attribution misses the SaaS-internal call cost: a single AWS RDS instance serves N tenants simultaneously; the per-resource cost cannot be split across tenants without per-call evidence. ADR-0174 §D anticipated this by listing capability invocation as a chargeback-formula component; this ADR specifies the emission shape that makes that component computable.

### A.5 Named pressure: provider routing is a Cedar decision per ADR-0243

ADR-0243 (Cedar as universal gate) established that every gate is a Cedar evaluation. Provider routing (which provider — oyatie-own / aws / oci / on-prem — serves a request) is a gate. Today the gate decides on cost + capacity + region + compliance-pack constraints; this ADR adds carbon intensity as a Cedar context input. The Cedar fragment shape is preserved; only the context shape grows.

The Cedar context input is the per-region carbon intensity in grams-CO2-equivalent-per-kWh, resolved from the canonical electricityMaps API at gate-evaluation time (with the cached fallback per A.7). The Cedar policy can then express routing rules like:

```
permit (
    principal,
    action == Action::"RouteWorkload",
    resource
) when {
    context.workload.slo_error_budget_remaining > 0.5
    && context.providers.lowest_carbon_intensity_g_co2e_per_kwh < context.workload.carbon_budget_g_co2e_per_kwh
};
```

The Cedar gate evaluates carbon as one input among many; the same gate continues to evaluate cost, capacity, region, compliance pack, and tenant class. The shape is unchanged.

### A.6 Named pressure: carbon-aware scheduling is a documented hyperscaler practice

Carbon-aware scheduling — deferring non-urgent workloads to hours of low grid carbon intensity — is published practice at Google (Carbon-Intelligent Computing platform, 2020-04-22), Microsoft (Sustainable Software Engineering principles, 2022 onward; carbon-aware-windows-services SDK), and AWS (Customer Carbon Footprint Tool methodology guidance, 2022 onward).

The Oyatie carbon-aware scheduling surface uses the same primitive: the workload declares a maximum acceptable carbon cost per execution; the scheduler defers the workload to the next low-carbon window provided that the workload's SLO error budget per ADR-0341 + the applicable compliance-pack RTO/RPO floor per ADR-0343 permit the deferral. The Cedar fragment that authorizes the deferral references this ADR + ADR-0243 + ADR-0341 + ADR-0343.

Carbon-aware scheduling is an opt-in surface per workload, not a default. The default scheduling policy is latency-optimal placement. Workloads opt in to carbon-aware scheduling by declaring `carbon_aware: true` in their workflow definition (per workflow-studio + workflow-engine per ADR-0328 microservices) or by a tenant-class-scoped setting (`tenant_class == demo_trial` may default to carbon-aware where compatible).

### A.7 Named pressure: carbon intensity provider availability + caching

The canonical carbon intensity provider for this ADR is **electricityMaps** (https://electricitymaps.com/api). electricityMaps publishes near-real-time and historical per-region grid carbon intensity (grams-CO2-equivalent-per-kWh, both marginal and average) across 150+ regions covering all jurisdictions Oyatie operates in. Their API is the most-cited industry source for region-level carbon intensity and is consumed by the Google Cloud Region Picker, the Microsoft Green Software Foundation's `carbon-aware-sdk`, and AWS's published guidance for the Customer Carbon Footprint Tool.

The provider integration is in-scope for a follow-on wave (out-of-scope for this ADR — see §A.10). What this ADR specifies is the caching + fallback model:

- **Hot cache.** electricityMaps responses are cached per (region, hour) tuple in Valkey per ADR-0336. Cache TTL: 1 hour. Cache miss policy: synchronous fetch.
- **Cold fallback.** If electricityMaps is unreachable, the µservice falls back to a provider-published grid-average per region. Each provider (AWS, OCI, Google Cloud, Azure, Oracle, on-prem self-published) publishes static per-region grid-average factors at known URLs; Oyatie's deployment IaC ships these factors as Kubernetes ConfigMaps refreshed quarterly.
- **Sovereign fallback.** Sovereign cells per ADR-0240 may pin to a sovereign-published per-region carbon intensity source (e.g., the European Environment Agency for EU regions; the U.S. Environmental Protection Agency eGRID for U.S. regions; the Korea Ministry of Environment for Korean regions). The sovereign source overrides electricityMaps when the sovereign pack mandates it.

The fallback chain MUST be documented per emitting µservice. Audit rows carry a `carbon_intensity_source` field (enum: `electricitymaps` / `provider_grid_avg` / `sovereign_published` / `cache_hit` / `cache_miss_fallback`) so auditors can trace methodology.

### A.8 Anchors this ADR binds

- Anchor 1: the user directive of 2026-05-21 captured in `feedback_six_candidate_adrs_2026_05_21.md` Decision 5 — "every audit-chain row carries CO2-grams + watt-hours + USD-cost computed at emission time using region-specific carbon intensity from electricityMaps API. Per-tenant + per-product + per-capability dimensional rollup in finops-portal."
- Anchor 2: ADR-0263 (observability emission contract). The mandatory event envelope already carries tenant_id + cell_id + audit_id + schema_version + source_microservice; the envelope is extended additively.
- Anchor 3: ADR-0174 (FinOps cost attribution + chargeback). The cost-attribution tag block is extended; the chargeback formula is widened with a carbon-cost factor.
- Anchor 4: ADR-0252 (HLC for accurate emission ordering). Carbon + energy + cost are emitted at the same HLC tick as the audit row; ordering invariants from ADR-0252 propagate cleanly.
- Anchor 5: ADR-0243 (Cedar as universal gate). Provider routing gates gain carbon as a context input.
- Anchor 6: ADR-0244 (tenant scoping universal primitive). tenant_id is already required on every audit row; this ADR adds the product + capability + provider + region dimensions, not replacing tenant_id.
- Anchor 7: ADR-0248 (Amazon-shape cellular architecture). The cell_id dimension is preserved; the rollup adds compliance_pack as a new dimension via the per-cell compliance pack mapping per ADR-0251.
- Anchor 8: ADR-0250 (build ahead of certification). CSRD / SB-253 / SEC are certification regimes; building per-call attribution day one is the canonical posture.
- Anchor 9: ADR-0251 (compliance pack primitive). The per-compliance-pack dimensional axis is enabled by the per-cell compliance pack tagging already established by this ADR.
- Anchor 10: ADR-0252 (HLC + TrueTime tier). HLC is the canonical timestamp; TrueTime upgrade per ADR-0252 transparently applies to carbon + energy + cost emissions.
- Anchor 11: ADR-0255 (intelligence two-layer). Intelligence transport surface carries provider-routing decisions; this ADR adds carbon as a routing input.
- Anchor 12: ADR-0263 (observability emission contract). §D-11 per-tenant cost attribution emission is extended.
- Anchor 13: ADR-0322 (substance-bar). Per-microservice sustainability_emission_model declarations are bespoke; template-stamping is forbidden.
- Anchor 14: ADR-0338 (pod runtime tier 0..3). Pod runtime tier feeds into the energy model — Kata + Cloud Hypervisor pods carry the ~30-40 percent density-tax overhead, which translates directly into a per-pod kWh baseline.
- Anchor 15: ADR-0340 (capacity model). The per-microservice capacity_model block already declares baseline_cpu_per_tenant + baseline_ram_per_tenant + storage_per_tenant + connections_per_tenant; the sustainability_emission_model in this ADR consumes the same primitives to compute watt_hours.

### A.9 Inherited constraints

- **ADR-0099 data-class registry.** Carbon-emission audit rows are themselves `INTERNAL_ONLY` data class unless the tenant has opted into per-tenant sustainability reporting, in which case the row's `cost_usd_minor_units` + `co2_grams` + `watt_hours` summary becomes `TENANT_VISIBLE` for that tenant.
- **ADR-0150 + ADR-0243 Cedar.** Carbon-as-context is a Cedar context input only; it does not change Cedar evaluation shape.
- **ADR-0183 Kyverno admission.** Carbon-aware scheduling decisions per workload are Cedar (authorization-time); pod-runtime-tier admission per ADR-0338 is Kyverno (admission-time); the two are independent.
- **ADR-0212 buildability doctrine.** Per-microservice manifest gains the sustainability_emission_model field; the buildability scaffolder ensures the field is declared on µservice creation.
- **ADR-0244 tenant scoping.** tenant_id remains a mandatory envelope field on every audit row; this ADR adds dimensions, not replacing the scoping primitive.
- **ADR-0251 compliance pack primitive.** Per-cell compliance pack determines the per-compliance-pack rollup axis.
- **ADR-0254 deployment-model-spectrum + ADR-0338 pod runtime tier.** Kata + Cloud Hypervisor density tax feeds the per-pod kWh baseline.
- **ADR-0263 mandatory event envelope.** The envelope is extended additively with the five new fields.
- **ADR-0322 substance-bar.** Per-µservice sustainability_emission_model authoring is bespoke.

### A.10 What this ADR does not assert

- **A.10.1** Does not author the electricityMaps API integration. That work is sequenced as part of Wave 15Y-Sustainability-FinOps under a follow-on IP at `microservices/observability/IPs/IP-electricitymaps-binding.md` plus per-microservice instrumentation IPs at `microservices/<name>/IPs/IP-sustainability-emission-model.md`.
- **A.10.2** Does not author the per-microservice sustainability_emission_model blocks. That work is sequenced per-µservice under ADR-0328 canonical-build phase order; each µservice files a per-µservice IP per A.10.1.
- **A.10.3** Does not introduce a new sustainability microservice. The finops-portal microservice already exists per ADR-0028 cloud-microservice-architecture; this ADR widens its scope to own the dimensional rollup surface.
- **A.10.4** Does not retire the per-resource sustainability_class tag from ADR-0174. The PUE-class tag remains a per-cell marker and is orthogonal to the per-call carbon emission introduced here.
- **A.10.5** Does not change the canonical audit-chain commit protocol per ADR-0263. The envelope is extended; the protocol is unchanged.
- **A.10.6** Does not change tenant_class semantics per ADR-0330. demo_trial and paid tenants both receive per-call attribution; the rollup respects per-tenant visibility (per ADR-0099).
- **A.10.7** Does not introduce mandatory carbon-aware scheduling. Carbon-aware scheduling is opt-in per workload.
- **A.10.8** Does not retire any existing observability metric or audit-event class. The new fields are additive on the envelope.
- **A.10.9** Does not change ClickHouse OLAP compute (per ADR-0337); the finops-portal rollup runs ClickHouse queries against Iceberg tables containing audit rows.
- **A.10.10** Does not change the OpenTofu IaC posture; the electricityMaps client + the Valkey hot cache deploy via existing shared IaC modules per ADR-0339.
- **A.10.11** Does not introduce a new Cedar entity type for carbon-aware scheduling beyond what ADR-0243 + ADR-0150 already permit; the Cedar context input shape is `context.workload.carbon_budget_g_co2e_per_kwh` and `context.region.carbon_intensity_g_co2e_per_kwh`.
- **A.10.12** Does not change the reactive vs proactive scheduling stance; both reactive (drop a request now) and proactive (defer to a low-carbon window) are supported via the same Cedar fragment shape.

## Decision

### B.1 Decision statement

Every audit-chain row emitted under ADR-0263 MUST carry five additional envelope fields — `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region` — computed at emission time on the same HLC tick as the audit row itself per ADR-0252. The values are derived by the emitting µservice from its declared `sustainability_emission_model` manifest block, which maps the µservice's workload signal (pod_runtime_tier per ADR-0338 + cpu-seconds + memory-byte-seconds + storage-byte-hours + network-bytes consumed per call) to watt_hours via a per-pod-runtime-tier power model, then to co2_grams via region carbon intensity resolved from the electricityMaps API (canonical) or the documented fallback chain per §D-7.

The finops-portal microservice owns the canonical dimensional rollup surface, exposing per-tenant, per-product, per-capability, per-provider, per-cell, and per-compliance-pack aggregations of cost + carbon + energy. The rollup is computed by ClickHouse queries layered on Iceberg tables containing the audit rows (per ADR-0337). The rollup surface includes both an internal dashboard (operator-facing) and a regulator-export evidence file format (CSRD ESRS / SB-253 / SEC-climate-disclosure schemas).

Provider routing gates per ADR-0243 gain region carbon intensity as a Cedar context input. Carbon-aware scheduling is opt-in per workload; when opted in, the scheduler defers the workload to a low-carbon window only when the workload's SLO error budget per ADR-0341 + the applicable compliance-pack RTO/RPO floor per ADR-0343 permit the deferral.

The contract is binding on every microservice that emits audit-chain rows. Enforcement transitions from `advisory-until-per-microservice-sustainability-emission-model-declared` to `BLOCKER` per the §E lane schedule.

### B.2 Numbered decision clauses

B2.001. Every audit-chain row MUST carry the five new envelope fields: `cost_usd_minor_units` (signed 64-bit integer; USD cents or jurisdiction equivalent), `co2_grams` (unsigned 64-bit integer; grams of CO2-equivalent), `watt_hours` (unsigned 64-bit integer; whole watt-hours), `provider` (enum: `oyatie_own` / `aws` / `oci` / `gcp` / `azure` / `on_prem` / `colo` / `sovereign`), `region` (string; provider-canonical region identifier — e.g., `aws:us-east-1`, `oci:ap-seoul-1`, `on_prem:dc-iad-1`).

B2.002. The five new fields are emitted at the same HLC tick as the audit row per ADR-0252. No separate timestamp; no per-field-staggered emission.

B2.003. Cost computation MUST use the provider's published pricing pinned at emission time. The pinned price is cached per (provider, region, sku, hour) tuple in Valkey per ADR-0336.

B2.004. Carbon computation MUST use region carbon intensity resolved from the electricityMaps API at emission time per §D-7. The intensity is cached per (region, hour) tuple in Valkey.

B2.005. Energy computation MUST use a per-pod-runtime-tier power model per §D-6. Tier 0 + Tier 1 (Kata + Cloud Hypervisor per ADR-0338) carries a higher per-pod baseline than Tier 2 (runc) or Tier 3 (runc-edge). Storage + network components apply uniformly across tiers.

B2.006. The five new fields are mandatory; emission paths that do not populate them are refused by the `oya-check-audit-row-carbon-fields` lane per §E.

B2.007. Per-microservice `manifest.json` MUST declare a top-level `sustainability_emission_model` block per §D-2 once the µservice emits a workload.

B2.008. The sustainability_emission_model block is bespoke per µservice per the substance-bar discipline of ADR-0322; template-stamping is forbidden.

B2.009. finops-portal owns the canonical dimensional rollup surface per §D-3. The six canonical axes are: tenant, product, capability, provider, cell, compliance_pack.

B2.010. finops-portal MUST expose all six axes via the operator dashboard + the tenant self-service dashboard + the regulator-export evidence file format per §D-4.

B2.011. The regulator-export evidence file format MUST match CSRD ESRS E1 (climate change) tabular schema + SB-253 disclosure schema + SEC climate disclosure rule taxonomy per §D-5. Three separate export formats; each derives from the same underlying rollup.

B2.012. Per-tenant rollup visibility follows ADR-0099 data-class rules. Per-tenant cost + carbon + energy is `TENANT_VISIBLE` for the tenant that produced the activity. Aggregate-across-tenants rollups are `INTERNAL_ONLY` unless explicitly published in a regulator filing.

B2.013. Carbon-intensity provider integration MUST use electricityMaps API as canonical per `feedback_six_candidate_adrs_2026_05_21`. The fallback chain per A.7 + §D-7 is documented; emission paths carry `carbon_intensity_source` enum.

B2.014. The Valkey hot cache TTL is 1 hour per (region, hour) tuple. Cache miss is synchronous fetch from electricityMaps; circuit-breaker on the fetch path per ADR-0263 emission self-cost rule.

B2.015. The cold fallback to provider-published grid-average per region MUST be activated when electricityMaps is unreachable for more than 3 consecutive cache misses. The cold-fallback grid averages ship as Kubernetes ConfigMaps refreshed quarterly via the IaC pipeline per ADR-0339.

B2.016. Sovereign cells per ADR-0240 MAY pin to a sovereign-published per-region carbon intensity source overriding electricityMaps. The override is recorded in the compliance pack per ADR-0251.

B2.017. Provider routing Cedar context inputs gain `context.region.carbon_intensity_g_co2e_per_kwh` and `context.workload.carbon_budget_g_co2e_per_kwh` per §D-9.

B2.018. Carbon-aware scheduling is opt-in per workload per `carbon_aware: true` workflow flag. The scheduler defers only when SLO error budget per ADR-0341 + compliance pack RTO/RPO per ADR-0343 permit.

B2.019. The carbon-aware scheduler may NOT defer Tier 0 (tenant-customer-untrusted-code) or Tier 1 (substrate-touching-tenant-data-plane) workloads per ADR-0338; only Tier 2 + Tier 3 are eligible.

B2.020. The carbon-aware scheduler may NOT defer regulator-mandated workloads (compliance pack `eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`) whose pack mandates real-time execution.

B2.021. The carbon-aware scheduler audit row carries a `carbon_aware_decision` field recording {permit / defer / defer_rejected} with the deferral reason.

B2.022. The chargeback formula per ADR-0174 is widened: `chargeback_period = labelled_spend + capability_invocation + audit_chain_emission + storage + credits + carbon_cost`, where `carbon_cost` is an optional tenant-class-scoped factor (default 0 USD, opt-in for tenants that have purchased the carbon-cost-passthrough product).

B2.023. The anomaly thresholds per ADR-0174 are widened: a `carbon_spike` threshold (tenant `co2_grams > 3× MAD over rolling 14-day baseline AND co2_grams > 10000g/hr`) and a `watt_hours_creep` threshold (tenant `watt_hours > 7-day rolling baseline by 25% sustained >= 24h`) are added.

B2.024. The new fields are versioned per the `oyatie/log/v1` schema version per ADR-0263. The version bumps to `oyatie/log/v2` on this ADR's Acceptance; emission paths declare `schema_version: "oyatie/log/v2"`.

B2.025. The schema version bump is BLOCKER for new authoring at Acceptance; existing emission paths migrate per ADR-0263 §D-13 deferred-retrofit pattern within the 30-day sunset window from §G.

B2.026. The finops-portal dashboard surface MUST be hosted at `finops.oyatie.<tenant-domain>` per `feedback_multi_context_provider_agnostic_2026_05_20` and MUST follow ADR-0341 cellular promotion gates for tier classification.

B2.027. The tenant-visible portion of the dashboard MUST follow the per-tenant Cedar gate per ADR-0243 + ADR-0244; cross-tenant data exposure is refused at request time.

B2.028. The regulator-export evidence file format MUST be signed via cosign per ADR-0181 at evidence-pack-generation time. The signed evidence pack is the auditor-facing artifact.

B2.029. The regulator-export evidence pack MUST be HLC-ordered and HLC-sealed per ADR-0252; out-of-order rows in the evidence pack are refused by the regulator-export validator.

B2.030. The dimensional rollup runs on ClickHouse 26.3 LTS layered on Iceberg tables per ADR-0337. The rollup query is owned by `microservices/finops-portal/src/rollup/`.

B2.031. The rollup's freshness budget is 5 minutes p99 per (tenant, hour) tuple. Stale-by-more-than-5-minutes rollups emit a `rollup.staleness.exceeded` audit event.

B2.032. The rollup's recursive emission cost (the cost of emitting audit rows for the rollup query itself) is bounded per ADR-0263 D-11 recursive-cost rule: rollup emission MAY NOT emit further audit rows for its own resource consumption beyond the per-query summary row.

B2.033. New µservices created after this ADR is Accepted MUST declare sustainability_emission_model from the first authoring step. New microservice scaffolding per ADR-0212 buildability scaffolder emits the block as a substance-bar-required field.

B2.034. Existing emission paths that do not yet populate the five new fields remain compilable until each µservice's migration bucket lands. The lanes per §E are advisory until Wave 15Y-Sustainability-FinOps ships the per-µservice declarations + the rollup surface; the lanes promote to BLOCKER per-µservice as each migration bucket lands.

B2.035. The Wave 15Y-Sustainability-FinOps sub-wave authors: (i) per-microservice `sustainability_emission_model` blocks for every active µservice; (ii) the electricityMaps integration at `microservices/observability/src/carbon_intensity/`; (iii) the Valkey hot-cache wiring; (iv) the cold-fallback ConfigMap deployment per IaC; (v) the finops-portal dimensional rollup surface; (vi) the three regulator-export evidence file formats; (vii) the carbon-aware-scheduler Cedar fragment library; (viii) the seven new CI lanes' implementations; (ix) per-µservice REMEDIATION-NOTES under `microservices/<name>/remediation-notes/2026-05-21-sustainability-emission-model.md`.

B2.036. Three Rejected Alternatives are recorded in §F below: (i) end-of-month roll-up from cloud bills (spend-based methodology — fails limited-assurance audit posture); (ii) per-resource attribution without per-call (cannot split shared-resource cost across tenants); (iii) carbon as a separate emission stream not co-located with audit (breaks HLC ordering and double the storage cost).

B2.037. Multispectrum review v2.4.0 applies to this ADR per ADR-0322 §D-2. Review evidence at `evidence/debate/ADR-0344/` after this ADR lands in a review-track PR.

B2.038. The 30-day sunset window starts on Acceptance. The seven new lanes (§E) promote from REPORT-ONLY to BLOCKER for new authoring at day 30; per-µservice migration of existing emission paths is sequenced under ADR-0328 and may extend the per-µservice-BLOCKER promotion until each migration bucket lands.

B2.039. The ADR is final on Acceptance. No exception clause is provided for any µservice's emission path after Wave 15Y-Sustainability-FinOps lands the per-µservice declarations.

B2.040. The ADR is announced in the realignment-wave findings aggregation, in the next ADR-0327 promotion gate report, and in the regulatory-readiness operator runbook.

## Consequences

### C.1 Positive consequences

- **CSRD / SB-253 / SEC ready day one.** Per-call attribution with HLC ordering + tenant scope + per-region carbon + per-product / per-capability rollup matches the GHG Protocol Scope 3 activity-based methodology under limited-assurance audit. Oyatie tenants in regulated jurisdictions can file evidence on demand; Oyatie itself is positioned for its own Scope 3 reporting.
- **Per-tenant in-product reporting.** Tenants see their cost + carbon + energy live in finops-portal at near-real-time freshness (5-minute p99 rollup). This is a differentiator versus hyperscaler-90-day-lag dashboards.
- **Cedar-gated provider routing on cost + carbon.** Every Cedar provider-routing gate evaluates carbon intensity alongside cost; the platform routes workloads to lower-carbon regions when SLO + compliance pack permit, without a separate scheduler.
- **Carbon-aware scheduling at workload granularity.** Workloads opt in to defer to low-carbon windows; the scheduler honors SLO + compliance-pack constraints. Demo_trial tenants on OCI Always Free may default to carbon-aware (free, time-shiftable) without product or capability degradation.
- **Audit-chain reuse.** The five new fields live on the existing audit row; no new emission stream; no doubled storage; no extra HLC ordering surface.
- **Hyperscaler-grade rigor.** AWS Customer Carbon Footprint Tool, Google Cloud Carbon Footprint, Microsoft Azure Emissions Impact Dashboard, Oracle Cloud Sustainability Dashboard, Snowflake Carbon Insights, Salesforce Net Zero Cloud have all converged on per-tenant, per-product, per-region carbon reporting. Oyatie's posture matches and exceeds (per-call vs per-month) the hyperscaler baseline.
- **Build ahead of certification (ADR-0250).** Carbon emission lands day one for every µservice that emits audit rows; no retrofit; no methodology debt.
- **Substance-bar restoration (ADR-0322).** Per-µservice sustainability_emission_model is bespoke; template-stamping is forbidden; every µservice's emission model reflects its actual workload signal.
- **Multi-context coverage.** The same emission contract applies to oyatie-public-cloud, guest-on-aws, guest-on-oci, on-prem, colo, and oyatie-as-cloud-provider. The provider field captures the deployment context; the region field captures the geographic region.
- **Sovereign-pack-friendly.** Sovereign cells may pin to sovereign-published carbon intensity sources; the override is auditable.
- **FinOps + carbon converge.** Cost-anomaly thresholds gain carbon equivalents (carbon-spike + watt-hours-creep); the chargeback formula gains an optional carbon-cost factor.
- **Cellular-tier visibility.** The compliance_pack rollup axis surfaces per-pack emissions; HIPAA-pinned cells, PCI-pinned cells, EU AI Act high-risk cells each get their own emission view.
- **OCI Always Free alignment.** Demo_trial workloads on OCI Always Free (per `feedback_oci_always_free_maximization_2026_05_20`) carry effectively-zero direct cost — but the carbon emission is still attributed accurately, exposing the true environmental footprint of demo_trial usage even when USD-cost is zero.

### C.2 Negative consequences

- **Wave 15Y-Sustainability-FinOps authoring cost.** ~77 µservices × per-µservice sustainability_emission_model block ≈ ~77 bespoke blocks. Plus the electricityMaps integration. Plus the finops-portal dimensional rollup. Plus the three regulator-export evidence formats. Plus the carbon-aware-scheduler Cedar library. Estimated 4-6 codex batches under ADR-0328 batch discipline.
- **Per-emission compute cost.** Each audit row emission now does an electricityMaps lookup (cached per region+hour) + a per-pod-runtime-tier power model evaluation + a provider price lookup. With the Valkey hot cache, the marginal cost per emission is ~10-50 µs CPU + ~one Valkey GET. At ~10^6 audit rows/sec at peak, this is ~10-50 CPU-seconds/sec of additional work in the audit-chain emission path — about 1-5 percent of the audit-chain CPU budget per ADR-0263.
- **Per-row storage cost.** Five additional fields, ~40 bytes per row. At ~10^6 rows/sec at peak × 86400 sec/day ≈ ~3.5 TB/day additional storage at full corpus scale. Iceberg compression + audit retention shortening compensates.
- **electricityMaps dependency.** A new external dependency on https://electricitymaps.com/api. The fallback chain mitigates outage risk but introduces a third-party vendor in the supply chain.
- **Cedar context input expansion.** Provider routing gates evaluate one more context input; gate evaluation time increases marginally (~5 µs).
- **Carbon-aware scheduler complexity.** Workloads opt in; the scheduler honors SLO + compliance-pack constraints; the decision matrix expands by one axis. Operator dashboards add a carbon-aware-decision panel.
- **Per-tenant data-class shift.** Per-tenant cost + carbon + energy summary moves from `INTERNAL_ONLY` to `TENANT_VISIBLE` for the producing tenant; Cedar gates the cross-tenant boundary.
- **Regulator-export evidence file format authoring.** Three separate formats (CSRD ESRS E1, SB-253, SEC climate disclosure) authored once but maintained over time as regulations evolve.

### C.3 Neutral consequences

- **Audit-chain commit protocol unchanged.** ADR-0263's commit protocol is preserved; the envelope is extended additively.
- **Tenant_class semantics unchanged.** demo_trial and paid both receive per-call attribution; visibility differs by per-tenant configuration.
- **Cellular topology unchanged.** ADR-0248 cell shape preserved.
- **Pod runtime tier unchanged.** ADR-0338 tier model preserved; this ADR consumes the tier as a power-model input.
- **Cedar entity types unchanged.** No new entity type beyond the context input shape.
- **OpenTofu IaC posture unchanged.** electricityMaps client + Valkey cache deploy via existing shared IaC modules per ADR-0339.
- **Iceberg + ClickHouse OLAP posture unchanged.** Per ADR-0337; the rollup queries Iceberg-canonical tables.

### C.4 Engineering-rigor dimensions

| Dimension | Requirement created by this ADR | Acceptance signal |
|---|---|---|
| Maintainability | Five envelope fields universally added; per-µservice manifest declares emission model | Every µservice manifest declares sustainability_emission_model; oya-check-audit-row-carbon-fields green |
| Observability | Per-call carbon + energy + cost on every audit row | Sampled audit rows carry the five fields; finops-portal rollup is fresh ≤5min p99 |
| Compliance | CSRD + SB-253 + SEC ready day one | Per-period regulator-export evidence packs generated; cosign-signed; HLC-sealed |
| Performance | <5 percent CPU budget overhead on audit-chain emission | Per-emission cost ≤ 50 µs CPU + 1 Valkey GET; gate CPU envelope per ADR-0263 preserved |
| Capacity | Per-µservice power model bounded by ADR-0340 capacity model + ADR-0338 pod runtime tier | Per-tenant watt_hours within ±20 percent of independently-measured baseline |
| Resilience | electricityMaps fallback chain documented; cache TTL bounded | electricityMaps outage drill: cold-fallback activates within 3 misses; audit emission unaffected |
| Cost | Per-emission compute + storage cost bounded | Per-row storage cost ~40 bytes; per-emission compute cost ~10-50 µs |
| Carbon | Activity-based methodology; per-region intensity; per-call evidence | Auditor-grade evidence pack generated quarterly |
| Cellular | Per-cell + per-compliance-pack rollup | Per-cell + per-pack rollup queryable in finops-portal |
| Multi-context | Provider field captures aws / oci / on-prem / colo / oyatie_own / sovereign | All six provider values populated in production rollup |

### C.5 Hyperscaler-grade rigor application

**Named precedent.** AWS Customer Carbon Footprint Tool (2022-03-01) provides per-account monthly Scope 1/2/3 with 90-day lag. Google Cloud Carbon Footprint (2021-10-12) provides per-billing-account monthly emissions with hourly intensity feeds via CFEHM. Microsoft Azure Emissions Impact Dashboard (2020-11-12) provides per-subscription monthly emissions. Oracle Cloud Sustainability Dashboard (2023-09-18) provides per-tenancy monthly emissions. Snowflake Carbon Insights (2023-06-26) provides per-account monthly emissions broken down by workload type. Salesforce Net Zero Cloud (2021-10-13) provides tenant-facing Scope 1/2/3 reporting surface. The hyperscaler convergence on per-tenant, per-region, per-product carbon reporting is unambiguous; Oyatie's per-call, near-real-time posture exceeds the hyperscaler-monthly baseline.

**Failure-mode tree.** (1) electricityMaps unreachable → cold-fallback to provider grid-average within 3 misses; audit emission unaffected. (2) Valkey cache miss → synchronous fetch; circuit-breaker per ADR-0263. (3) emission path forgets to populate the five fields → CI lane refuses at landing; promotion to BLOCKER post-soak. (4) per-µservice manifest missing sustainability_emission_model → `oya-check-sustainability-emission-model` lane refuses at landing. (5) finops-portal rollup stale > 5 min → `rollup.staleness.exceeded` audit event; observability alert. (6) Cedar carbon-aware-scheduling fragment defers a Tier 0/1 workload → refused by `oya-governance-carbon-aware-scheduling` lane (Tier 0/1 ineligible per B2.019). (7) regulator-export evidence pack out-of-order → refused by regulator-export validator. (8) cross-tenant data exposure attempt → Cedar tenant-scope gate refuses per ADR-0244.

**Capacity math.** Per emission: ~10-50 µs CPU + 1 Valkey GET. At peak ~10^6 rows/sec corpus-wide: ~10-50 CPU-sec/sec ≈ 1-5 percent of audit-chain CPU envelope. Per-row storage: ~40 bytes; at peak ≈ ~3.5 TB/day additional, ~1.3 PB/year, ~390 TB after Iceberg compression. finops-portal rollup: ClickHouse query against Iceberg, 5-min p99 freshness, indexable by all six dimensional axes.

**Observability hooks.** Audit row gains five fields + `carbon_intensity_source` enum. Cardinality multiplier bounded: tenant_id is already universal; cell_id already universal; provider has 7 enum values; region has ~50 distinct values; carbon_intensity_source has 5 enum values. New cardinality per (tenant, hour) tuple: 7 × 50 × 5 = 1,750 distinct combinations — small relative to existing tenant_id × source_microservice cardinality.

**Rollback path.** Per-µservice rollback: revert manifest to pre-sustainability_emission_model state; audit emission falls back to four-field envelope. finops-portal rollback: serve last-good rollup snapshot. Cell-level rollback: disable carbon-aware scheduler Cedar fragment; latency-optimal scheduling resumes. Aggregate corpus rollback: not provided; rollback is per-µservice or per-feature-flag.

**Multi-region awareness.** Each region's cells emit per-region carbon intensity. Cross-region workloads emit one audit row per region segment. Sovereign cells per ADR-0240 pin to sovereign-published intensity overriding electricityMaps.

**Sovereign-cell awareness.** Sovereign HIPAA / GDPR-strict / CSAP / PCI cells emit carbon + energy per the sovereign-published intensity; the compliance pack records the override.

**Versioning + deprecation.** Per ADR-0108 sunset discipline. Schema version bumps to `oyatie/log/v2` at Acceptance. Per-microservice migration extends per ADR-0328 canonical-build phase order. Schema version `oyatie/log/v1` deprecates 12 months post-Acceptance.

## D. Detailed mechanics — eleven adoption surfaces

The sustainability + finops dimensional model touches eleven adoption surfaces in audit-chain itself, in finops-portal, in every emitting µservice, and in the carbon-aware-scheduling Cedar surface. Subsections D-1 through D-11 enumerate each surface. Numbering is normative.

### D-1: Audit row envelope extension — five new fields

D-1.1. Every audit-chain row emitted under ADR-0263 carries the following five additional envelope fields:

- `cost_usd_minor_units` (int64). Per-call USD cost in minor units (USD cents or jurisdiction equivalent). Computed from provider-published pricing pinned at emission time. Cached per (provider, region, sku, hour) tuple in Valkey per ADR-0336.
- `co2_grams` (uint64). Per-call CO2-equivalent emissions in grams. Computed as `watt_hours * region_carbon_intensity_g_co2e_per_kwh / 1000`. The intensity is resolved from electricityMaps API (canonical) with the documented fallback chain per D-7.
- `watt_hours` (uint64). Per-call energy consumption in whole watt-hours. Computed by the per-pod-runtime-tier power model per D-6 from the µservice's declared sustainability_emission_model and the observed workload signal (cpu-seconds + memory-byte-seconds + storage-byte-hours + network-bytes).
- `provider` (enum string). One of `oyatie_own` / `aws` / `oci` / `gcp` / `azure` / `on_prem` / `colo` / `sovereign`. Captures the deployment provider serving the call.
- `region` (string). Provider-canonical region identifier. Examples: `aws:us-east-1`, `oci:ap-seoul-1`, `gcp:us-central1`, `azure:eastus`, `on_prem:dc-iad-1`, `sovereign:kr-isms-p:gw-1`.

D-1.2. A sixth derived field `carbon_intensity_source` (enum) records methodology provenance: `electricitymaps` / `provider_grid_avg` / `sovereign_published` / `cache_hit` / `cache_miss_fallback`. The derived field supports auditor methodology review.

D-1.3. The five new fields are emitted at the same HLC tick as the audit row per ADR-0252. No separate timestamp. No per-field-staggered emission.

D-1.4. The schema version field on the audit envelope bumps to `oyatie/log/v2` at Acceptance per B2.024.

D-1.5. The mandatory_envelope_fields array in `specs/audit-event-class-registry.json` is widened to include the five new fields plus `carbon_intensity_source`. The mandatory_envelope_fields minItems constraint bumps from 10 to 16.

D-1.6. Per-class payload schemas in `specs/audit-event-class-registry.json` are amended additively to allow the new fields under the existing per-class payload contracts.

D-1.7. Example audit row (Cedar decision class):

```json
{
  "tenant_id": "tnt_2026_kr_acme",
  "audit_id": "01J5...",
  "schema_version": "oyatie/log/v2",
  "source_microservice": "intelligence",
  "cell_id": "cel_kr_isms_p_seoul_1",
  "jurisdiction_code": "KR-ISMS-P",
  "trace_id": "...",
  "span_id": "...",
  "event_id": "...",
  "sub_scope_path": "intelligence/agent-runtime/dispatch",
  "hlc_timestamp": "2026-05-21T14:35:21.0014/lc=42",
  "cost_usd_minor_units": 1547,
  "co2_grams": 23,
  "watt_hours": 88,
  "provider": "oci",
  "region": "oci:ap-seoul-1",
  "carbon_intensity_source": "electricitymaps",
  "...class-specific-payload": "..."
}
```

### D-2: Per-microservice sustainability_emission_model manifest block

D-2.1. Every µservice that emits a workload (i.e., produces audit-chain rows from a runtime path) MUST declare a top-level `sustainability_emission_model` block in `microservices/<name>/manifest.json`.

D-2.2. The block declares the per-pod-runtime-tier power model coefficients + the workload-signal-to-watt-hours mapping + the provider price lookup binding. Schema:

```json
"sustainability_emission_model": {
  "pod_runtime_tier_ref": "self.pod_runtime_tier",
  "power_model": {
    "cpu_watts_per_vcpu_second": 0.5,
    "memory_watts_per_gib_second": 0.0025,
    "storage_watts_per_gib_hour": 0.0008,
    "network_watts_per_gib": 0.06,
    "tier_overhead_factor": {
      "tier_0_kata_clh": 1.4,
      "tier_1_kata_clh": 1.4,
      "tier_2_runc": 1.0,
      "tier_3_runc_edge": 1.05
    }
  },
  "price_model": {
    "source": "provider_sku_pricing",
    "binding": "microservices/cloud-billing/pricing-pinned/",
    "pin_window_hours": 1
  },
  "workload_signal_source": "observability/per-call-resource-accounting",
  "emission_path_tests": "tests/sustainability/emission_model.rs",
  "validation_baseline": {
    "expected_watt_hours_per_request_p50_mwh": 12,
    "expected_co2_grams_per_request_p50_at_grid_400gco2_per_kwh": 5,
    "tolerance_pct": 20
  }
}
```

D-2.3. The block is bespoke per µservice per ADR-0322 substance-bar; template-stamping refused by `oya-check-sustainability-emission-model` lane.

D-2.4. The power_model coefficients are calibrated per µservice. The default values shown in D-2.2 are illustrative; each µservice's IP at `microservices/<name>/IPs/IP-sustainability-emission-model.md` carries the calibration methodology + measurement evidence.

D-2.5. Per-pod-runtime-tier tier_overhead_factor reflects the ~30-40 percent density-tax overhead of Kata + Cloud Hypervisor per ADR-0338. Tier 0 + Tier 1 ≈ 1.4x; Tier 2 = 1.0x baseline; Tier 3 ≈ 1.05x (edge-tuned hardware adds slight kernel-bypass cost).

D-2.6. The workload_signal_source binds the µservice to the canonical per-call resource accounting feed from observability per ADR-0263. The feed publishes cpu-seconds + memory-byte-seconds + storage-byte-hours + network-bytes per audit event.

D-2.7. The emission_path_tests path declares a test target validating that the µservice's emission path actually populates the five new fields with values within the validation_baseline tolerance.

### D-3: finops-portal dimensional rollup surface — six canonical axes

D-3.1. The finops-portal µservice owns the canonical dimensional rollup surface. Six canonical axes:

1. **tenant** (tenant_id; per ADR-0244).
2. **product** (product enum: `core` / `messenger` / `mail` / `community` / `intelligence` / `workflow` / `ontology` / `marketplace` / `crm` / `erp-*` / etc.; product enum is canonical in `docs/standards/product-enum.md` and mirrored in `specs/product-enum.json`).
3. **capability** (capability registry row per `registry/capabilities/`; e.g., `messenger.send_message`, `intelligence.agent_dispatch`, `cloud-data.postgres.query`).
4. **provider** (the per-row provider enum from D-1.1).
5. **cell** (cell_id; per ADR-0248).
6. **compliance_pack** (per-cell compliance pack mapping per ADR-0251; e.g., `hipaa-em`, `gdpr-strict`, `pci-dss-l1`, `eu-ai-act-annex-iii`, `soc2-type-2`, `kr-isms-p`).

D-3.2. The rollup is computed by ClickHouse queries layered on Iceberg tables containing audit rows per ADR-0337. Query owner: `microservices/finops-portal/src/rollup/`.

D-3.3. The rollup query MUST surface, per axis, totals for: `cost_usd_minor_units_sum`, `co2_grams_sum`, `watt_hours_sum`, and `event_count`.

D-3.4. Pre-aggregated materialized views in ClickHouse iceberg engine handle the high-frequency rollup queries (per-tenant per-minute, per-tenant per-hour, per-tenant per-day).

D-3.5. The rollup freshness budget is 5 minutes p99 per (tenant, hour) tuple per B2.031. Stale rollups emit `rollup.staleness.exceeded`.

D-3.6. Cross-axis combinations are first-class queryable: e.g., `co2_grams_sum where tenant_id = T and product = P and compliance_pack = HIPAA-EM and hour = H`. The Iceberg-engine partition strategy is `(tenant_id, hour, cell_id)`; secondary indexes on (product, capability, provider, region, compliance_pack).

D-3.7. The rollup table retention is 7 years per HIPAA / PCI / CSRD / SOC2 audit floor; per ADR-0099 data-class retention rules.

### D-4: finops-portal dashboard + tenant self-service surfaces

D-4.1. finops-portal exposes three surfaces:

1. **Operator dashboard.** Internal-only. Aggregate across-tenants rollup. Used by ops-finops + ops-sre-reliability + council-architecture for capacity + cost + carbon analysis.
2. **Tenant self-service dashboard.** Tenant-visible. Per-tenant rollup with the six axes filtered by tenant_id. Used by tenants for in-product cost + carbon transparency.
3. **Regulator-export evidence pack.** Auditor-facing. Per-period (typically quarterly) signed evidence pack covering the tenant's full activity history with HLC-ordered audit rows + per-period rollups + methodology provenance. Three export formats (D-5).

D-4.2. The tenant self-service dashboard renders the six axes as filter dimensions. Default view: per-period (selectable month / quarter / year) per-product cost + carbon + energy with per-day time series.

D-4.3. The dashboard exposes a "What-if" surface allowing tenants to model the impact of carbon-aware scheduling (counterfactual: how much carbon could be saved by enabling carbon-aware on this workload?).

D-4.4. The dashboard exposes a "Provider routing" surface showing per-region carbon intensity at the current hour, and per-call breakdown of which provider served each capability invocation.

D-4.5. The tenant self-service dashboard is gated by Cedar per ADR-0243 + ADR-0244; cross-tenant exposure is refused at request time.

D-4.6. The operator dashboard is gated by Cedar per ADR-0243; only principals in `ops-finops` / `ops-sre-reliability` / `council-architecture` / `council-legal` / `ops-compliance` see across-tenants data.

### D-5: Regulator-export evidence pack — three formats

D-5.1. **CSRD ESRS E1 export.** Tabular schema matching the European Sustainability Reporting Standards E1 (Climate change). Required columns per E1: Scope 1 emissions, Scope 2 (location-based + market-based), Scope 3 categories (Category 1 — purchased goods and services breakdown; Category 11 — use of sold products). Per-period: annual. Filed: by 30 April of the year following the reporting period. Format: XBRL with ESEF inline taxonomy per EU Commission delegated regulation 2023/2772.

D-5.2. **SB-253 export.** Tabular schema matching the GHG Protocol Corporate Accounting and Reporting Standard. Required: Scope 1 + Scope 2 (year 1 onward); Scope 3 (year 2 onward). Per-period: annual. Filed: with the California Air Resources Board. Format: CARB-published CSV schema with disclosure-protocol metadata.

D-5.3. **SEC climate disclosure export.** Tabular schema matching the SEC final rule "Enhancement and Standardization of Climate-Related Disclosures for Investors". Required: Scope 1 + Scope 2 (large accelerated + accelerated filers); Scope 3 (if material and used in target-setting). Per-period: fiscal-year. Filed: with SEC filings (10-K). Format: XBRL with SEC taxonomy.

D-5.4. Each export pack is generated by a dedicated `microservices/finops-portal/src/regulator_export/<format>/` module. The three modules share the dimensional rollup data source.

D-5.5. Each export pack is signed via cosign per ADR-0181 + B2.028 at evidence-pack-generation time. The cosign attestation digest is recorded in the audit-chain class `RegulatorExportEvidencePackSigned`.

D-5.6. Each export pack is HLC-ordered per ADR-0252 + B2.029. Out-of-order rows are refused.

D-5.7. The auditor-facing pack format includes both the aggregated rollup and a sample of underlying audit rows (per HIPAA-like minimum-necessary disclosure principle) for methodology verification.

### D-6: Per-pod-runtime-tier power model

D-6.1. Per ADR-0338, every pod declares pod_runtime_tier ∈ {0, 1, 2, 3}. The power model uses tier as a multiplier on the baseline per-resource watts.

D-6.2. Baseline per-resource watts (illustrative defaults; per-µservice calibration in D-2):

- cpu_watts_per_vcpu_second: 0.5 W (representative of x86 server-class @ ~2-3 GHz under sustained load).
- memory_watts_per_gib_second: 0.0025 W/GiB·s (representative of DDR4/DDR5 server DIMM idle+access).
- storage_watts_per_gib_hour: 0.0008 W/GiB·h (representative of NVMe SSD active+idle blended).
- network_watts_per_gib: 0.06 W·h/GiB (representative of switching + NIC + transit blended).

D-6.3. tier_overhead_factor per D-2.5: Tier 0 + Tier 1 = 1.4x; Tier 2 = 1.0x; Tier 3 = 1.05x.

D-6.4. The per-call watt_hours formula:

```
watt_hours = (
  cpu_seconds * cpu_watts_per_vcpu_second
  + memory_byte_seconds / (1024^3) * memory_watts_per_gib_second
  + storage_byte_hours / (1024^3) * storage_watts_per_gib_hour
  + network_bytes / (1024^3) * network_watts_per_gib
) * tier_overhead_factor / 3600
```

D-6.5. The formula is computed in the audit emission path. Inputs come from the canonical per-call resource accounting feed per D-2.6. The formula is amortizable: per-call inputs are deltas since the last accounting checkpoint, normalized to the call's duration.

D-6.6. PUE (Power Usage Effectiveness) multiplier from the per-cell sustainability_class tag per ADR-0174 is applied at rollup time, not per-call. Per-cell PUE class:
- `pue-gte-1-2`: multiplier 1.25 (legacy on-prem datacenter).
- `pue-1-2-to-1-1`: multiplier 1.15 (modern hyperscaler datacenter).
- `pue-lt-1-1`: multiplier 1.05 (cutting-edge hyperscaler datacenter; some sovereign cells).

D-6.7. The model is validated quarterly against independent measurement (PDU-level kWh metering on representative on-prem cells; provider-published per-product power factors for AWS / OCI / GCP / Azure cells). Validation results are recorded in `evidence/sustainability/quarterly-power-model-validation/<period>.json`.

### D-7: electricityMaps API binding + fallback chain

D-7.1. The canonical region carbon intensity source is electricityMaps API (https://electricitymaps.com/api). Endpoint: `GET /v3/carbon-intensity/{zone}` (zone = electricityMaps zone identifier mapped from Oyatie's region string per a per-provider mapping table at `microservices/observability/src/carbon_intensity/zone_mapping.json`).

D-7.2. The response is the marginal carbon intensity in grams-CO2-equivalent-per-kWh + the average carbon intensity. Oyatie uses the marginal value for carbon-aware scheduling decisions (per Google CFEHM precedent) and the average value for cost-attribution audit emission (per GHG Protocol guidance).

D-7.3. Cache: Valkey per ADR-0336. Key: `carbon_intensity:{region}:{hour}`. TTL: 1 hour.

D-7.4. Cache miss policy: synchronous fetch. Fetch budget: 200 ms p99. Circuit breaker: after 3 consecutive failures, open the breaker for 5 minutes and fall back to provider grid average.

D-7.5. Provider grid average fallback. Per-provider, per-region static factor published by the provider (AWS Customer Carbon Footprint Tool methodology document; Google Cloud Sustainability data; Microsoft Azure per-region factors; OCI Sustainability Dashboard methodology). Factors refreshed quarterly via Kubernetes ConfigMap deployed by IaC per ADR-0339.

D-7.6. Sovereign fallback. Sovereign cells per ADR-0240 may pin to sovereign-published per-region intensity overriding electricityMaps:
- EU regions: European Environment Agency (EEA) per-country emission factors.
- US regions: EPA eGRID per-eGRID-subregion factors.
- Korea regions: Korea Ministry of Environment per-region factors.
- Japan regions: Japan Ministry of Environment per-region factors.
- Sovereign packs (CSAP / KISA / NDMO / FedRAMP-High) may override at the per-cell level.

D-7.7. The fallback chain is recorded per-call in `carbon_intensity_source` (enum: `electricitymaps` / `provider_grid_avg` / `sovereign_published` / `cache_hit` / `cache_miss_fallback`).

D-7.8. The electricityMaps API client lives at `microservices/observability/src/carbon_intensity/electricitymaps_client.rs` (Rust per `feedback_rust_strict_only_no_python_2026_05_20`). The client is owned by axis-observability.

### D-8: Provider price binding

D-8.1. Provider-published pricing is pinned at emission time. The pricing source is the per-provider pricing API (AWS Pricing API; OCI Cost Management API; GCP Cloud Billing Catalog API; Azure Retail Prices API; Oyatie internal pricing for `oyatie_own` / `on_prem` / `colo`).

D-8.2. The pricing client lives at `microservices/cloud-billing/src/pricing/<provider>_client.rs`. The client is owned by axis-cloud-billing.

D-8.3. Cache: Valkey. Key: `price:{provider}:{region}:{sku}:{hour}`. TTL: 1 hour.

D-8.4. SKU resolution per µservice. Each µservice's sustainability_emission_model declares its per-call SKU mapping (e.g., `microservices/intelligence/manifest.json` maps `intelligence.agent_dispatch` to SKU `compute-tier-1-vcpu-hour` × cpu-seconds + `network-egress-tier-1-gib` × network-bytes).

D-8.5. The cost computation is `cost_usd_minor_units = sum(sku_unit_count * sku_unit_price_per_unit)` over the call's SKUs.

D-8.6. The price pin is preserved for 7 years per audit retention. Historical price reconstruction for audit purposes is supported via the Iceberg snapshot per ADR-0337.

### D-9: Cedar context inputs for provider routing + carbon-aware scheduling

D-9.1. Per ADR-0243, every gate is a Cedar evaluation. Provider routing gates gain two new context inputs:

- `context.region.carbon_intensity_g_co2e_per_kwh` (number). Current marginal intensity for the region under consideration.
- `context.workload.carbon_budget_g_co2e_per_kwh` (number). Per-workload budget declared in the workflow definition (per workflow-studio per ADR-0328).

D-9.2. Example Cedar fragment for cost + carbon provider routing:

```cedar
permit (
  principal == ServiceAccount::"workflow-engine",
  action == Action::"RouteCallToProvider",
  resource is Capability
) when {
  context.workload.slo_error_budget_remaining > 0.5
  && context.providers.lowest_cost_usd_per_call <= context.workload.cost_budget_usd_per_call
  && (
    context.providers.lowest_carbon_intensity_g_co2e_per_kwh <= context.workload.carbon_budget_g_co2e_per_kwh
    || context.workload.carbon_aware == false
  )
};
```

D-9.3. Carbon-aware scheduling fragment example (defer to low-carbon window):

```cedar
permit (
  principal == ServiceAccount::"workflow-engine",
  action == Action::"DeferUntilLowCarbonWindow",
  resource is WorkflowRun
) when {
  resource.carbon_aware == true
  && resource.pod_runtime_tier in [2, 3]  // Tier 0/1 ineligible per ADR-0344 B2.019
  && resource.compliance_pack != "eu-ai-act-annex-iii"  // realtime-mandated packs ineligible per B2.020
  && resource.compliance_pack != "hipaa-em-incident-response"
  && resource.compliance_pack != "pci-dss-realtime-fraud-detection"
  && resource.slo_error_budget_remaining > 0.5
  && context.region.carbon_intensity_g_co2e_per_kwh > resource.carbon_budget_g_co2e_per_kwh
  && context.next_low_carbon_window_starts_within_seconds < resource.compliance_pack_rto_seconds
};
```

D-9.4. The carbon-aware-scheduler Cedar fragment library lives at `microservices/governance/cedar/sustainability/`. Authoring is bespoke per workflow per ADR-0322.

D-9.5. The fragment's audit row includes `carbon_aware_decision` ∈ {permit / defer / defer_rejected} with the deferral reason captured in the per-class payload.

### D-10: ADR-0174 chargeback formula + anomaly thresholds widening

D-10.1. The chargeback formula per ADR-0174 §D widens:

```
chargeback_period_total =
    labelled_spend
  + capability_invocation
  + audit_chain_emission
  + storage
  + credits
  + carbon_cost
```

D-10.2. `carbon_cost` is an optional tenant-class-scoped factor:
- Default (all tenants): 0 USD.
- Opt-in (paid tenants that purchase the carbon-cost-passthrough product): `co2_grams_sum_in_period * carbon_cost_per_gram_usd`, where `carbon_cost_per_gram_usd` is per-jurisdiction-published (EU ETS price; California Cap-and-Trade; voluntary market via Verra or Gold Standard).
- demo_trial tenants: 0 USD (no opt-in).

D-10.3. Anomaly thresholds widening:
- `carbon_spike`: tenant `co2_grams > 3 × MAD over rolling 14-day baseline AND co2_grams > 10000g/hr`; severity SEV-2.
- `watt_hours_creep`: tenant `watt_hours > 7-day rolling baseline by 25% sustained >= 24h`; severity SEV-3.
- Existing thresholds (cost_spike / cost_creep / tenant_budget_headroom / tenant_budget_exhausted / provider_cost_deviation) preserved verbatim per ADR-0174.

D-10.4. The anomaly thresholds emit audit events per ADR-0174 emission contract; the new thresholds emit `carbon.spike.detected` and `energy.watt_hours.creep.detected`.

### D-11: Wave 15Y-Sustainability-FinOps sub-wave sequencing

D-11.1. Wave 15Y-Sustainability-FinOps is added to `specs/master-plan-sequencing.json#realignment_wave_sequence.waves_15_plus.sub_waves` as the eighth queued sub-wave (after 15S-Pod-Runtime-Tier-declaration). Status: `queued`. queued_at: 2026-05-21. ADR: ADR-0344. Sunset window: 30 days post-ADR-0344-Acceptance OR Wave 15Y-Sustainability-FinOps landing, whichever later.

D-11.2. Sub-wave scope per B2.035: per-microservice sustainability_emission_model blocks; electricityMaps integration; Valkey hot-cache wiring; cold-fallback ConfigMap deployment; finops-portal dimensional rollup surface; three regulator-export evidence file formats; carbon-aware-scheduler Cedar fragment library; seven new CI lanes' implementations; per-µservice REMEDIATION-NOTES.

D-11.3. Sub-wave dispatch_mode: codex-bucket fan-out for the per-µservice manifest authoring + Claude orchestrator for the finops-portal dimensional rollup and the regulator-export evidence file formats + per-microservice bespoke authoring under ADR-0322 substance-bar + ADR-0324 anti-template discipline.

D-11.4. Sub-wave depends_on: ADR-0344 Acceptance; ADR-0337 (Iceberg, for the rollup data source) — already queued; ADR-0338 (pod-runtime-tier, for the per-pod power model) — already queued; ADR-0340 (capacity model, for the per-resource baseline) — co-authored.

D-11.5. amends_microservice_artifacts: every µservice manifest (sustainability_emission_model declared); `microservices/finops-portal/` (dimensional rollup surface + dashboards + regulator-export modules); `microservices/observability/src/carbon_intensity/` (electricityMaps client + zone mapping); `microservices/cloud-billing/src/pricing/` (per-provider pricing clients); `microservices/governance/cedar/sustainability/` (carbon-aware Cedar fragment library); `specs/audit-event-schema.json` (envelope extension); `specs/audit-event-class-registry.json` (mandatory_envelope_fields widening); `specs/finops-dimensional-model.json` (rollup contract); `tools/hooks/_canonical-primitives.md` (Sustainability section added).

## E. CI lanes — seven new gates

E.1. **oya-check-audit-row-carbon-fields** — refuses audit-chain emission paths that do not populate `cost_usd_minor_units` + `co2_grams` + `watt_hours` + `provider` + `region`. Stage: REPORT-ONLY at landing; BLOCKER at day 30 post-Acceptance for new authoring; per-µservice BLOCKER promotion as each µservice's migration bucket lands per ADR-0328.

E.2. **oya-check-sustainability-emission-model** — refuses µservice manifests that lack `sustainability_emission_model` block once the µservice emits a workload. Refuses template-stamped blocks (substance-bar per ADR-0322). REPORT-ONLY at landing; BLOCKER at day 30.

E.3. **oya-check-finops-portal-dimensional-rollup** — refuses finops-portal dashboards / regulator-export modules that do not surface all six canonical axes (tenant / product / capability / provider / cell / compliance_pack). REPORT-ONLY at landing; BLOCKER at day 30.

E.4. **oya-check-carbon-intensity-provider-binding** — refuses µservice runtime that resolves region carbon intensity from a non-canonical provider. electricityMaps is canonical; the documented fallback chain per D-7 is permitted. REPORT-ONLY at landing; BLOCKER at day 30.

E.5. **oya-check-regulatory-sustainability-report-emission** — refuses regulator-class quarterly audit-chain reports (CSRD / SB-253 / SEC-climate-disclosure) that lack carbon + energy + cost totals across the dimensional axes. REPORT-ONLY at landing; BLOCKER at day 30.

E.6. **oya-governance-sustainability-tag-allowlist** — existing ADR-0174 lane preserved verbatim. Validates the per-cell `sustainability_class` tag (`pue-gte-1-2` / `pue-1-2-to-1-1` / `pue-lt-1-1`) at provision time.

E.7. **oya-governance-carbon-aware-scheduling** — refuses Cedar fragments that gate workload placement on carbon intensity without referencing this ADR + ADR-0243. Refuses fragments that defer Tier 0 / Tier 1 workloads per B2.019. Refuses fragments that defer realtime-mandated compliance-pack workloads per B2.020. REPORT-ONLY at landing; BLOCKER at day 30.

## F. Rejected alternatives

### F.1 Rejected: end-of-month roll-up from cloud bills (spend-based methodology)

Rationale rejected: spend-based methodology is the highest-uncertainty GHG Protocol class. Limited-assurance audit cannot trace per-tenant attribution from end-of-month bills under multi-tenant SaaS. CSRD + SB-253 + SEC limited assurance expects activity-based methodology. Spend-based reconstruction also produces 30-90 day lag, which fails the per-tenant in-product reporting use case. Hyperscaler precedent (AWS / Google / Azure / OCI / Snowflake / Salesforce) has already converged on per-tenant aggregation — Oyatie's posture must be at least as fine-grained.

### F.2 Rejected: per-resource attribution without per-call

Rationale rejected: per-resource attribution (the ADR-0174 baseline) cannot split shared-resource cost across tenants. A single AWS RDS instance serving N tenants under multi-tenant SaaS attributes 100 percent of the resource cost to the µservice but cannot split per-tenant. Without per-call evidence, the per-tenant chargeback per ADR-0174's capability_invocation factor is computable only by approximation. Per-call attribution is the cleanest evidence chain for both audit and chargeback.

### F.3 Rejected: carbon as a separate emission stream not co-located with audit

Rationale rejected: a separate emission stream doubles the storage cost, doubles the HLC ordering surface, and creates a methodology gap between the audit row's HLC tick and the carbon row's HLC tick. Auditor methodology review demands single-source evidence. The five new fields share the audit row's HLC tick by construction; the methodology gap is zero.

### F.4 Rejected: marginal-only carbon intensity vs average-only

Rationale rejected: marginal intensity is the policy-relevant metric for carbon-aware scheduling (per Google CFEHM precedent); average intensity is the GHG-Protocol-correct metric for emission attribution (per GHG Protocol Scope 3 Standard guidance). The ADR uses average for emission attribution (D-7.2) and marginal for scheduling decisions (D-9). Either-only would fail one of the use cases.

### F.5 Rejected: sustainability microservice as a new separate µservice

Rationale rejected: a separate sustainability µservice would duplicate finops-portal's tenant-facing dashboard surface + audit-chain's emission surface + observability's metric surface. The ADR instead widens finops-portal's scope to own the dimensional rollup. ADR-0245 substrate-vs-product layering forbids the duplication.

## G. Sunset schedule

G.1. ADR Acceptance (day 0): seven new lanes REPORT-ONLY; schema version `oyatie/log/v2` declared; Wave 15Y-Sustainability-FinOps queued in `specs/master-plan-sequencing.json`.

G.2. Day 1-29: per-µservice authors sustainability_emission_model block; new emission paths land with the five new fields populated; finops-portal dimensional rollup authored; electricityMaps integration authored; three regulator-export evidence file formats authored; carbon-aware-scheduler Cedar fragment library authored.

G.3. Day 30: lanes promote to BLOCKER for new authoring. Existing emission paths that have not yet migrated remain advisory until each µservice's migration bucket lands per ADR-0328.

G.4. Day 30+: per-µservice migration sequenced per ADR-0328 canonical-build phase order. Each µservice's migration bucket promotes the lanes to BLOCKER for that µservice at landing.

G.5. Day 365: schema version `oyatie/log/v1` deprecates. All emission paths MUST be on `oyatie/log/v2`. Any remaining `oyatie/log/v1` emission path is refused at admission.

G.6. Day 365+: Wave 15Y-Sustainability-FinOps is considered closed; new µservices use the schema from the first authoring step.

## H. Acceptance signal

H.1. Every µservice manifest declares `sustainability_emission_model` (or is documented as workload-free).

H.2. Every audit-chain row from a workload-emitting µservice populates the five new envelope fields + the `carbon_intensity_source` derived field.

H.3. finops-portal dimensional rollup surface live; six axes queryable; 5-min p99 freshness.

H.4. Three regulator-export evidence file formats (CSRD ESRS E1 / SB-253 / SEC climate disclosure) generate valid signed packs on demand.

H.5. electricityMaps integration live; Valkey hot cache; cold-fallback ConfigMap; sovereign overrides per per-cell compliance pack.

H.6. Cedar context inputs (region carbon intensity + workload carbon budget) flowing to provider-routing gates and carbon-aware-scheduling gates.

H.7. Seven new CI lanes BLOCKER for new authoring at day 30.

H.8. ADR-0174 chargeback formula widened with `carbon_cost`; anomaly thresholds widened with `carbon_spike` + `watt_hours_creep`.

H.9. Schema version bumped to `oyatie/log/v2`; `oyatie/log/v1` deprecates day 365.

H.10. Multispectrum review v2.4.0 evidence at `evidence/debate/ADR-0344/` complete; review-track PR landed; ADR-0344 Status moves from Proposed to Accepted.

## I. Open questions deferred to Wave 15Y-Sustainability-FinOps

I.1. **Per-µservice power model calibration methodology.** Each µservice's IP at `microservices/<name>/IPs/IP-sustainability-emission-model.md` records its calibration methodology. The default coefficients in D-2.2 are illustrative; per-µservice IPs document the actual measurement evidence.

I.2. **Tenant carbon-cost-passthrough product packaging.** ADR-0174 chargeback formula gains `carbon_cost` as an opt-in factor. The product packaging (pricing, opt-in flow, regulatory-disclosure-grade carbon-cost-per-gram source) is a follow-on product-team decision.

I.3. **Sovereign-published intensity authority pinning per compliance pack.** D-7.6 lists EU EEA + US EPA eGRID + KR + JP environment ministries. Per ADR-0251 compliance pack updates, sovereign-pack authors record per-pack pinning at pack authoring time.

I.4. **Carbon-aware scheduling SLO trade study.** Deferring workloads to low-carbon windows trades carbon savings against latency. The trade study per workload class is a follow-on operational decision; the ADR specifies that the trade is per-workload-opt-in, not platform-default.

I.5. **Validation baseline tolerance per µservice.** D-2.7 declares ±20 percent default tolerance. Per-µservice IPs may tighten the tolerance based on workload characterization.

I.6. **Quarterly power-model validation cadence.** D-6.7 declares quarterly cadence; the exact cadence + sample size + evidence pack format is in `microservices/finops-portal/IPs/IP-quarterly-power-model-validation.md`.
