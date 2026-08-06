---
id: ADR-0340
title: Capacity model per microservice manifest (baseline_cpu_per_tenant + baseline_ram_per_tenant + storage_per_tenant + connections_per_tenant + scaling_dimension + cell_placement_class)
status: Superseded
planning_impact: true
date: 2026-05-21
owner_team:
  - council-architecture
  - ops-sre-reliability
  - ops-finops
  - axis-cloud-data
owners:
  - council-architecture
  - ops-sre-reliability
  - ops-finops
  - axis-cloud-data
supersedes: []
superseded_by: [ADR-700]
amends:
  - ADR-0212-buildability-doctrine.md (per-µservice manifest gains the canonical `capacity_model` block; buildability evidence packs reference declared capacity_model values for autoscale + finops projections)
  - ADR-0244-tenant-as-universal-scoping-primitive.md (the baseline_cpu_per_tenant + baseline_ram_per_tenant + storage_per_tenant + connections_per_tenant axes anchor the tenant-scoping primitive in concrete capacity numbers per µservice)
  - ADR-0245-substrate-vs-product-layering.md (substrate µservices declare their per-tenant footprint authoritatively; product µservices declare capacity in the product's natural scaling_dimension; substrate-vs-product layering is preserved by the typed scaling_dimension enum)
  - ADR-0248-amazon-shape-cellular-architecture.md (cell_placement_class ∈ {Tier-0..Tier-4} co-varies with the ADR-0248 cellular criticality numbering; capacity_model.cell_placement_class is the per-µservice declaration anchoring the cell sizing model + shuffle-sharding tiering decision)
  - ADR-0331-cross-microservice-tenant-class-adoption-template.md (capacity_model is the canonical declaration surface that the tenant_class cap-shape contract sizes against; demo_trial and paid cap-shape ceilings reference per-µservice capacity_model rows)
  - ADR-0338-pod-runtime-tier-0-to-3.md (pod_runtime_tier and cell_placement_class are co-declared per µservice; pod_runtime_tier governs admission/isolation, cell_placement_class governs cellular placement; the two axes are distinct but both bind cell capacity planning)
  - ADR-0339-shared-iac-module-library.md (capacity_model values feed shared-library nodepool sizing primitives at `cloud/cloud-iac/modules/<context>/{kata-pool, runc-pool, runc-edge-pool}/`; module inputs accept per-µservice baseline_cpu_per_tenant + baseline_ram_per_tenant as sizing parameters)
related_adrs:
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0044-inter-cell-mesh-tunnel.md
  - ADR-0099-data-class-registry.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0108-deprecation-sunset-discipline.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0152-rpo-rto-per-microservice.md
  - ADR-0174-finops-cost-attribution-tag.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
  - ADR-0197-backup-tier-driven-rpo-rto.md
  - ADR-0198-karpenter-nodepool-selection.md
  - ADR-0199-finops-cost-attribution-namespace.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0215-multi-context-platform.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0263-observability-emission-contract.md
  - ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md
  - ADR-0324-anti-script-authoring-doctrine.md
  - ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md
  - ADR-0329-tier-system-retired-replaced-by-tenant-class.md
  - ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components.md
  - ADR-0331-cross-microservice-tenant-class-adoption-template.md
  - ADR-0333-cell-microservice-retired-pattern-not-service.md
  - ADR-0335-intelligence-microservice-consolidation.md
  - ADR-0336-valkey-not-redis-substrate.md
  - ADR-0337-iceberg-canonical-olap-write-path.md
  - ADR-0338-pod-runtime-tier-0-to-3.md
  - ADR-0339-shared-iac-module-library.md
related_specs:
  - /specs/master-plan-sequencing.json
  - /specs/microservices/manifest-schema.json
  - /specs/microservices/cell.json
  - /specs/platform-architecture.json
  - /specs/markdown-retirement-policy.json
related_memory:
  - feedback_six_candidate_adrs_2026_05_21
  - feedback_idea_refine_decisions_2026_05_21
  - feedback_amazon_shape_cellular_architecture
  - feedback_quality_performance_scalability_bar
  - feedback_no_silent_regression
  - feedback_tenant_scoping_primitive
  - feedback_substrate_vs_product_layering
  - feedback_clean_architecture_requirements
  - feedback_microservice_ownership_coherence_2026_05_20
  - feedback_bominal_inheritance_precedence
  - feedback_docs_substance_not_scaffold_2026_05_20
  - feedback_drift_too_big_2026_05_20
  - feedback_rust_strict_only_no_python_2026_05_20
  - feedback_zero_handroll_opentofu_only_2026_05_20
  - feedback_oci_always_free_maximization_2026_05_20
  - feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20
companion_docs:
  - docs/standards/hyperscaler-best-practices.md
  - docs/standards/dependency-policy.md
  - docs/GLOSSARY.md
  - docs/machine-readable/glossary.json
  - tools/hooks/_canonical-primitives.md
inbound_citations:
  - /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_six_candidate_adrs_2026_05_21.md
doc_class: Architecture-Decision-Record
shape: Decision
authority_tier: 1
line_floor: 600
bespoke_authoring_requirement: documentation-rigor-1.1-plus-ADR-0322
enforcement_status: advisory-until-capacity-model-declaration-lands
enforced_by:
  - oya-check-capacity-model-present (new CI lane; advisory until crate lands; planned to refuse missing or malformed capacity_model blocks in manifest.json; REPORT-ONLY at landing; BLOCKER per-µservice as each capacity_model declaration lands)
  - oya-check-capacity-model-units (new CI lane; advisory until crate lands; planned to refuse non-canonical units; baseline_cpu_per_tenant MUST be vCPU as a decimal; baseline_ram_per_tenant MUST be MiB as an integer; storage_per_tenant MUST be GB as an integer; connections_per_tenant fields MUST be non-negative integers)
  - oya-check-capacity-model-scaling-dimension (new CI lane; advisory until crate lands; planned to refuse scaling_dimension values outside the closed enum {per_user, per_request, per_capability, per_message, per_query, per_workflow_run})
  - oya-check-capacity-model-cell-placement (new CI lane; advisory until crate lands; planned to refuse cell_placement_class values outside {Tier-0, Tier-1, Tier-2, Tier-3, Tier-4} and refuses Tier-0/Tier-1 declarations without an audit-chain seal event reference per ADR-0263)
  - oya-check-capacity-model-tenant-class-deltas (new CI lane; advisory until crate lands; planned to refuse capacity_model declarations that omit demo_trial vs paid delta rows when the µservice serves both tenant classes per ADR-0330 + ADR-0331)
  - oya-check-capacity-model-finops-anchor (new CI lane; advisory until crate lands; planned to refuse capacity_model values that diverge from the per-µservice FinOps cost-center declaration in `finops` section by more than the soft-band tolerance recorded in registry/finops/cost-tag-vocabulary.yaml)
  - oya-check-capacity-model-cellular-tier-coherence (new CI lane; advisory until crate lands; planned to refuse cell_placement_class values that conflict with the µservice's declared pod_runtime_tier per ADR-0338 + ADR-0248 co-variance table in D-6)
purpose: >
  Establish the canonical `capacity_model` block in every µservice's
  `microservices/<name>/manifest.json` as the machine-readable declaration
  of the µservice's per-tenant compute, memory, storage, and connection
  footprint; the natural scaling_dimension the µservice scales along
  (per_user / per_request / per_capability / per_message / per_query /
  per_workflow_run); and the cellular placement class
  (Tier-0..Tier-4 per ADR-0248) that anchors the µservice in the
  Amazon-shape cellular topology. The capacity_model block is the
  single source of truth driving (a) autoscaler input parameters per
  ADR-0198 Karpenter nodepool selection, (b) cell sizing inputs for
  Cell capacity planning per ADR-0009 + ADR-0248, (c) FinOps
  projection inputs per ADR-0174 + ADR-0199 cost-attribution, and
  (d) cellular placement determinism for shuffle-sharding +
  blast-radius reasoning per ADR-0248 + ADR-0333 (cell-as-pattern).
  The block is distinct from ADR-0338 pod_runtime_tier (which governs
  pod admission + isolation primitive) and from ADR-0251 compliance
  pack ceilings (which gate compliance posture). Declare the schema
  fragment in /specs/microservices/manifest-schema.json. Queue the
  corpus-wide per-µservice declaration as a separate sub-wave under
  ADR-0328 batch discipline. Do NOT author the per-µservice manifest
  updates in this ADR; that authoring is a follow-on sub-wave.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


> **Disposition light-edit (2026-08-06):** Context re-triage Accept: Capacity model per µservice manifest

# ADR-0340: Capacity model per microservice manifest (baseline_cpu_per_tenant + baseline_ram_per_tenant + storage_per_tenant + connections_per_tenant + scaling_dimension + cell_placement_class)

## Status

Proposed on 2026-05-21.

This ADR is the canonical capacity-declaration shape decision establishing the `capacity_model` block in every µservice's `microservices/<name>/manifest.json` as the single source of truth for autoscaler input parameters, cell sizing inputs, FinOps projection inputs, and cellular placement determinism. It is the first of six candidate ADRs from the 2026-05-21 `/idea-refine` session captured in `feedback_six_candidate_adrs_2026_05_21.md` (the other five candidates are ADR-0341 cellular promotion gates, ADR-0342 API versioning HYBRID date+semver, ADR-0343 DR matrix per-µservice + per-pack, ADR-0344 sustainability + finops dimensional model, ADR-0345 talent + OSS contribution policy tiers).

It runs in coordination with the in-flight 2026-05-21 realignment effort: ADR-0337 (Iceberg canonical OLAP) + ADR-0338 (Pod runtime tier 0..3) + ADR-0339 (Shared IaC module library) landed earlier in the session; this ADR is the substrate-data-shape decision that ties the prior triplet's capacity inputs together into a single machine-readable surface.

It directly amends ADR-0212 (buildability doctrine) by adding the canonical `capacity_model` block to the per-µservice manifest contract. It directly amends ADR-0244 (tenant as universal scoping primitive) by anchoring the per-tenant axes in concrete numerical declarations. It directly amends ADR-0248 (Amazon-shape cellular architecture) by adding the `cell_placement_class` field that anchors per-µservice placement in the five-tier cellular topology. It directly amends ADR-0331 (cross-µservice tenant_class adoption template) by establishing the manifest surface against which the demo_trial vs paid cap-shape contract is sized. It directly amends ADR-0338 (pod runtime tier 0..3) by clarifying the co-variance table between pod_runtime_tier (admission/isolation) and cell_placement_class (cellular placement). It directly amends ADR-0339 (shared IaC module library) by establishing the input contract that nodepool primitives accept from per-µservice wrappers.

Enforcement transitions from `advisory-until-capacity-model-declaration-lands` to `BLOCKER` per the lane sequence in §E below: at landing of the corpus-wide per-µservice capacity_model declaration sub-wave (queued as `15U-Capacity-Model-declaration` in `/specs/master-plan-sequencing.json#realignment_wave_sequence`), the lanes promote to BLOCKER for new authoring; existing manifests without `capacity_model` blocks remain compilable until their declaration bucket lands under ADR-0328 canonical-build phase order.

CAPACITY-001 adds only local bridge/advisory validation evidence for this Proposed ADR. The bridge implementation and regression test live at `marketplace/facade/dev-cli/src/capacity_model_manifest_gate.rs` and `marketplace/facade/dev-cli/tests/capacity_model_gate_cli.rs`; they validate the declared manifest shape without asserting autoscaler/runtime, cloud deployment, production-ready, hyperscaler-grade, or measured-SLO readiness.

The decision does not delete any existing manifest field. The decision does not change pod_runtime_tier semantics from ADR-0338. The decision does not change the tenant_class composable-billing-components shape from ADR-0330. The decision does not change cellular tier numbering from ADR-0248. The decision does not change the compliance pack model from ADR-0251. The decision does not change which µservice owns which capability; it adds a single declarative surface that every µservice owner declares once.

## Date

2026-05-21.

## Context

### A.1 Named pressure: no canonical per-µservice capacity declaration today

Oyatie has 77 active µservices (47 baseline + 9 ERP + 13 B2B-leader + the in-flight 8 healthcare/marketing splits captured by the realignment effort). Today there is no canonical machine-readable surface that captures, per µservice, the baseline per-tenant compute / memory / storage / connection footprint, the natural scaling dimension the µservice scales along, or the cellular placement class the µservice anchors in.

Without that surface, the following operate by inference:

- **Autoscaler decisions** (ADR-0198 Karpenter nodepool selection) cannot compute target replica counts from declared per-tenant footprint × tenant-count; they fall back to observed-metric-driven scaling, which is reactive rather than predictive.
- **Cell sizing** (ADR-0009 cell architecture + ADR-0248 cellular topology) cannot precompute how many tenants a Tier-2 cell can host before the cell saturates; cell capacity planning falls back to empirical observation per cell.
- **FinOps projections** (ADR-0174 sustainability-tag + ADR-0199 cost-attribution namespace) cannot project per-tenant cost across the µservice catalog without a per-µservice per-tenant footprint declaration; FinOps reporting falls back to post-hoc attribution of observed billing.
- **Cellular placement determinism** (ADR-0248 + ADR-0333 cell-as-pattern) cannot deterministically assign new µservices to the correct cellular tier without a per-µservice declaration of the tier; placement falls back to ad-hoc decision at deployment time.
- **Shuffle sharding** (the `oya-shuffle-sharding` Rust crate per ADR-0333) cannot select the correct shard sub-population without a per-µservice declaration of the cellular tier it belongs to.

The aggregate pressure is that **every cross-cutting capacity-planning surface today is doing per-µservice reverse-inference from runtime telemetry** instead of consulting a declared per-µservice contract. That is the named pressure this ADR resolves.

### A.2 Named pressure: tenant_class cap-shape contract has no anchor today

ADR-0330 (tenant_class demo_trial vs paid composable billing components) + ADR-0331 (cross-µservice tenant_class adoption template) §D-8 require every µservice to express per-tenant-class cap shapes (demo_trial caps + paid caps + any composable-billing-component overlays). The cap-shape contract today has no per-µservice canonical anchor it sizes against.

Example concrete gap: ADR-0331 §D-8.4 requires Always-Free demo_trial primitives to validate `tenant_class == "demo_trial"` and clamp resource sizes inside the OCI Always Free perpetual ceiling (per `feedback_oci_always_free_maximization_2026_05_20`). The "clamp to OCI Always Free ceiling" rule requires knowing the µservice's `baseline_cpu_per_tenant` + `baseline_ram_per_tenant` + `storage_per_tenant` to compute how many demo_trial tenants fit inside the OCI Always Free 2× Ampere A1 ARM 4 OCPU + 24 GiB compute budget. Without those declarations, the clamp logic is per-µservice ad-hoc rather than canonical.

Similar pressure applies for paid tenant_class with per-seat / per-usage / revenue-share composable billing components (per `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20`): the per-usage billing component MUST be sized against a per-µservice declared baseline_cpu_per_tenant × usage-multiplier; without the baseline, the per-usage component has no anchor.

### A.3 Named pressure: ADR-0248 cellular tier numbering has no per-µservice anchor today

ADR-0248 (Amazon-shape cellular architecture) established a five-tier cellular criticality model: Tier-0 = Foundation cells (identity / KMS / audit), Tier-1 = Substrate cells, Tier-2 = Capability cells, Tier-3 = Application cells, Tier-4 = Edge cells. ADR-0338 (Pod runtime tier 0..3) co-varies pod_runtime_tier with the cellular tier numbering (Tier-0 cell hosts Tier-0 + Tier-1 pods; Tier-2 cell hosts Tier-2 pods; etc.) but does NOT establish the per-µservice declaration of `cell_placement_class`.

Today, the µservice's cellular tier is inferred from its functional role (cloud-iam is "obviously" Tier-0 substrate; crm is "obviously" Tier-3 application). The inference is correct in 95% of cases and wrong in 5% — and the 5% (e.g., audit-chain straddles Tier-0 substrate and Tier-2 capability; messenger MLS keys straddle Tier-1 substrate and Tier-3 application; the wasmtime sandbox host in intelligence straddles Tier-1 substrate and Tier-3 application) is where blast-radius reasoning breaks down. A per-µservice canonical declaration eliminates the inference.

### A.4 Named pressure: scaling dimension is implicit today

µservices scale along different natural axes. Messenger scales along `per_message`. Workflow-engine scales along `per_workflow_run`. CRM scales along `per_user` (active CRM seats). API-gateway scales along `per_request`. Vector-store scales along `per_query`. Cloud-IAM scales along `per_capability` (issued capability scope per principal).

Today the scaling axis is implicit in autoscaler configuration (HorizontalPodAutoscaler metric selection) and in cell capacity planning. It is not declared in the manifest. The lack of declaration means:

- A new ops engineer reading the µservice's manifest cannot tell what axis the µservice scales along.
- The FinOps cost-attribution model (ADR-0174 + ADR-0199) cannot rollup costs by scaling dimension across the corpus.
- The cell sizing model cannot project per-cell throughput in the right unit (messages / requests / capabilities / queries / workflow_runs / users) without a per-µservice declaration.

The natural scaling dimension MUST be declared canonically.

### A.5 Named pressure: connections are a hidden capacity dimension

µservices consume substrate connections (Valkey connections per ADR-0336, PostgreSQL connections, outbound HTTP connections). Connection pool exhaustion is a well-known production failure mode at hyperscaler scale (e.g., PostgreSQL max_connections = 1000 limit; Valkey TCP socket per-connection memory cost; outbound TCP ephemeral port exhaustion on egress NAT). Per-µservice connection footprint is a hidden capacity dimension today.

Per-tenant connection footprint matters because:

- A µservice with `connections_per_tenant.valkey = 5` and 200 tenants on a cell consumes 1,000 Valkey connections — at or near the typical Valkey per-instance connection ceiling.
- A µservice with `connections_per_tenant.postgres = 10` and 100 tenants consumes 1,000 PostgreSQL connections — at the PgBouncer transaction-pool ceiling.
- A µservice with `connections_per_tenant.outbound_http = 50` and 100 tenants consumes 5,000 outbound TCP sockets — within the ephemeral port budget of a single egress NAT instance.

Declaring connections_per_tenant per substrate kind is the only way to size pool/proxy/NAT capacity ahead of failure.

### A.6 Named pressure: counterpart precedent at the hyperscalers

- **AWS Service Quotas.** AWS publishes per-service per-tenant limits (S3 buckets per account, EC2 vCPU per region, RDS connections per DB instance). The per-account / per-tier limit is declarative and machine-readable via the service-quotas API.
- **Google Cloud Quotas.** GCP publishes per-project per-resource quotas with declared baseline + per-tier limits. The quota is the canonical capacity-planning input.
- **Azure Subscription Limits.** Azure publishes per-subscription per-resource limits, broken down by SKU.
- **Stripe Rate Limits.** Stripe documents per-account API rate limits + per-resource limits. The per-account limit is the canonical capacity input.
- **Salesforce Multitenancy Limits.** Salesforce documents per-org governor limits (CPU time, query rows, callouts) as the canonical per-tenant capacity declaration.

Every hyperscaler operates a declared per-tenant capacity model surface. Oyatie's per-µservice `capacity_model` block is the in-tree equivalent for the per-µservice catalog.

### A.7 Named pressure: ADR-0338 pod_runtime_tier + ADR-0339 IaC module library compose with capacity_model

ADR-0338 §D-7 (capacity model implications) called out that kata-pool MUST be sized at 1.5x density-scaled vs runc baseline, runc-pool at 1.0x, runc-edge-pool at 1.0x with edge-hardware constraints. The 1.5x scaling factor is meaningless without a per-µservice baseline_cpu_per_tenant + baseline_ram_per_tenant to scale.

ADR-0339 §D-3 (per-µservice wrapper at `iac/<context>/main.tf`) established that wrappers pass tenant-scoped parameters (cluster size, retention, eviction policy, etc.) to shared-library primitives. The "cluster size" parameter is meaningless without a per-µservice declaration of how many tenants × baseline_cpu_per_tenant the cluster MUST absorb.

Both ADRs explicitly cited "per-µservice capacity model" as a missing input surface. This ADR closes that loop.

### A.8 Anchors this ADR binds

- Anchor 1: the user directive of 2026-05-21 captured in `feedback_six_candidate_adrs_2026_05_21.md` ADR-0340 section — "per-µservice manifest.json gets a `capacity_model` block with baseline_cpu_per_tenant, baseline_ram_per_tenant, storage_per_tenant, connections_per_tenant, scaling_dimension, cell_placement_class".
- Anchor 2: ADR-0212 (buildability doctrine) — per-µservice manifest is the canonical surface for substrate-decision declarations.
- Anchor 3: ADR-0244 (tenant as universal scoping primitive) — the per-tenant axes anchor in concrete numerical declarations.
- Anchor 4: ADR-0245 (substrate vs product layering) — substrate µservices declare per-tenant footprint authoritatively.
- Anchor 5: ADR-0248 (Amazon-shape cellular architecture) — cell_placement_class anchors per-µservice placement in the five-tier cellular topology.
- Anchor 6: ADR-0331 (cross-µservice tenant_class adoption template) — capacity_model is the canonical surface against which the demo_trial vs paid cap-shape contract is sized.
- Anchor 7: ADR-0338 (Pod runtime tier 0..3) — pod_runtime_tier (admission/isolation) is distinct from cell_placement_class (cellular placement); the two co-vary via D-6.
- Anchor 8: ADR-0339 (Shared IaC module library) — capacity_model values feed shared-library nodepool sizing primitives.
- Anchor 9: ADR-0174 + ADR-0199 (FinOps cost-attribution) — capacity_model drives per-tenant cost projections.
- Anchor 10: ADR-0198 (Karpenter nodepool selection) — capacity_model drives autoscaler input parameters.
- Anchor 11: ADR-0009 + ADR-0333 (cell architecture + cell-as-pattern) — capacity_model drives cell sizing inputs.
- Anchor 12: `feedback_quality_performance_scalability_bar` — hyperscaler-grade declared-capacity-model precedent (AWS Service Quotas / GCP Quotas / Azure Limits).

### A.9 What this ADR does not assert

- **A.9.1** Does not author per-µservice capacity_model declarations. That authoring is sequenced as the `15U-Capacity-Model-declaration` sub-wave under ADR-0328 batch discipline.
- **A.9.2** Does not change pod_runtime_tier semantics from ADR-0338. pod_runtime_tier remains the admission/isolation axis; cell_placement_class is the cellular-placement axis.
- **A.9.3** Does not change tenant_class semantics from ADR-0330. tenant_class remains a principal claim travelling per-request; capacity_model is a manifest declaration travelling per-µservice.
- **A.9.4** Does not change cellular tier numbering from ADR-0248. The Tier-0 = highest blast-radius / Tier-4 = edge convention is preserved verbatim.
- **A.9.5** Does not change compliance pack activation gating from ADR-0251. Compliance packs remain orthogonal to capacity_model; a pack may impose a stricter capacity floor (e.g., HIPAA-pinned µservices may require Tier-0 cell_placement_class), but the pack does not change the capacity_model schema.
- **A.9.6** Does not introduce a SaaS quota system. The capacity_model declaration lives in-tree in `microservices/<name>/manifest.json` and is consumed by Oyatie-internal tools (autoscaler, cell capacity planner, FinOps portal). There is no external SaaS dependency.
- **A.9.7** Does not retire any existing manifest field. New fields are added; existing fields are preserved.
- **A.9.8** Does not assert a single value for any field across the corpus. Each µservice's owner declares the values bespoke to that µservice's workload shape; the schema enforces typing and the closed-enum constraints, not specific numerical values.
- **A.9.9** Does not relax the substance-bar for per-µservice manifest authoring. Per ADR-0322, the per-µservice capacity_model values are bespoke substance (each µservice owner reasons about its own workload shape), not template-stamped.
- **A.9.10** Does not change which µservice owns which capability. The ownership shape is unchanged; only the declared surface in each µservice's manifest expands.

## Decision

### B.1 Decision statement

Every µservice's `microservices/<name>/manifest.json` MUST declare a top-level `capacity_model` block with the following required fields:

- `baseline_cpu_per_tenant`: decimal vCPU value per active tenant at steady state (e.g., `0.1`).
- `baseline_ram_per_tenant`: integer MiB value per active tenant at steady state (e.g., `256`).
- `storage_per_tenant`: integer GB value per active tenant at steady state (e.g., `1`).
- `connections_per_tenant`: object with three sub-fields (`valkey`, `postgres`, `outbound_http`), each a non-negative integer (e.g., `{ "valkey": 5, "postgres": 10, "outbound_http": 50 }`).
- `scaling_dimension`: closed enum value from `{per_user, per_request, per_capability, per_message, per_query, per_workflow_run}`.
- `cell_placement_class`: closed enum value from `{Tier-0, Tier-1, Tier-2, Tier-3, Tier-4}` (per ADR-0248 cellular criticality numbering — distinct from ADR-0338 pod_runtime_tier).

The `capacity_model` block is the single source of truth driving (a) autoscaler input parameters per ADR-0198 Karpenter nodepool selection, (b) cell sizing inputs for Cell capacity planning per ADR-0009 + ADR-0248, (c) FinOps projection inputs per ADR-0174 + ADR-0199 cost-attribution, and (d) cellular placement determinism for shuffle-sharding + blast-radius reasoning per ADR-0248 + ADR-0333 cell-as-pattern.

The block is REQUIRED for every µservice that produces a workload (i.e., a Helm chart, a Kubernetes Deployment / StatefulSet / Job / CronJob, a podSpec). The block is OPTIONAL for spec-only µservices that produce no workload (rare; mostly definitional µservices).

Per-µservice declarations are sequenced as the `15U-Capacity-Model-declaration` sub-wave under ADR-0328 canonical-build phase order. Existing manifests without `capacity_model` blocks remain compilable until each µservice's declaration bucket lands.

### B.2 Numbered decision clauses

B2.001. `microservices/<name>/manifest.json` declares a top-level `capacity_model` block.

B2.002. `capacity_model.baseline_cpu_per_tenant` is a JSON `number` representing vCPU at steady state per active tenant; minimum 0.001 (one milli-vCPU); maximum 1000.0 (a thousand vCPU; sanity ceiling for substrate µservices).

B2.003. `capacity_model.baseline_ram_per_tenant` is a JSON `integer` representing MiB at steady state per active tenant; minimum 1; maximum 1048576 (1 TiB; sanity ceiling).

B2.004. `capacity_model.storage_per_tenant` is a JSON `integer` representing GB at steady state per active tenant; minimum 0 (zero GB is permitted for stateless µservices); maximum 1048576 (1 PiB; sanity ceiling for data-warehouse-class µservices).

B2.005. `capacity_model.connections_per_tenant` is a JSON `object` with required sub-fields `valkey`, `postgres`, `outbound_http`; each sub-field is a non-negative JSON `integer`; minimum 0; maximum 1024 (sanity ceiling per connection kind per tenant).

B2.006. `capacity_model.scaling_dimension` is a closed enum string from `{per_user, per_request, per_capability, per_message, per_query, per_workflow_run}`. The enum is closed; new values require an ADR amendment.

B2.007. `capacity_model.cell_placement_class` is a closed enum string from `{Tier-0, Tier-1, Tier-2, Tier-3, Tier-4}` per the ADR-0248 cellular criticality numbering convention (Tier-0 = highest blast-radius / most isolated; Tier-4 = edge).

B2.008. `capacity_model.cell_placement_class` is DISTINCT from `pod_runtime_tier` per ADR-0338. pod_runtime_tier governs pod admission + isolation primitive (Kata vs runc vs runc-edge). cell_placement_class governs cellular placement (Foundation cell vs Substrate cell vs Capability cell vs Application cell vs Edge cell). The two co-vary per the table in D-6 but are independently declared.

B2.009. `capacity_model.scaling_dimension` values map to canonical scaling targets:
  - `per_user` = scales linearly with active named-user count (CRM seats, drive users, etc.).
  - `per_request` = scales with API request volume (api-gateway data-plane, application gateway).
  - `per_capability` = scales with issued capability scope count (cloud-iam, cedar policy engine).
  - `per_message` = scales with message volume (messenger, mail, notifications).
  - `per_query` = scales with query volume (vector-store, search, OLAP queries).
  - `per_workflow_run` = scales with workflow execution count (workflow-engine, workflow-studio).

B2.010. `capacity_model.cell_placement_class` values map to ADR-0248 cellular tiers:
  - `Tier-0` = Foundation cells (cloud-iam, cloud-kms, audit-chain primary, payments-pci core).
  - `Tier-1` = Substrate cells (cloud-secrets, observability backbone, intelligence transport, ontology projection).
  - `Tier-2` = Capability cells (workflow-engine, intelligence agent dispatch, marketplace catalog).
  - `Tier-3` = Application cells (crm, marketing-automation, drive, docs, sheets, slides, calendar, social, community).
  - `Tier-4` = Edge cells (api-gateway data-plane, Envoy edge, ztunnel, CDN edge cache).

B2.011. The `capacity_model` block is REQUIRED for every µservice that produces a workload. Spec-only µservices MAY omit the block.

B2.012. The CI lane `oya-check-capacity-model-present` validates declaration presence; REPORT-ONLY at landing; BLOCKER per-µservice as each declaration lands.

B2.013. The CI lane `oya-check-capacity-model-units` validates field types + unit constraints + sanity-ceiling bounds.

B2.014. The CI lane `oya-check-capacity-model-scaling-dimension` validates the closed enum.

B2.015. The CI lane `oya-check-capacity-model-cell-placement` validates the cell_placement_class enum + refuses Tier-0/Tier-1 declarations without an audit-chain seal event reference per ADR-0263.

B2.016. The CI lane `oya-check-capacity-model-tenant-class-deltas` refuses capacity_model declarations that omit demo_trial-vs-paid delta rows when the µservice serves both tenant classes per ADR-0330 + ADR-0331.

B2.017. The CI lane `oya-check-capacity-model-finops-anchor` refuses capacity_model values that diverge from the per-µservice FinOps cost-center declaration by more than the soft-band tolerance.

B2.018. The CI lane `oya-check-capacity-model-cellular-tier-coherence` refuses cell_placement_class values that conflict with the µservice's pod_runtime_tier per ADR-0338 + ADR-0248 co-variance table in D-6.

B2.019. `capacity_model.tenant_class_deltas` is an OPTIONAL sub-object that documents per-tenant-class capacity overrides for µservices serving both demo_trial and paid; when present, it has sub-fields `demo_trial` and `paid`, each a partial copy of the parent capacity_model with the same field-typing rules.

B2.020. `capacity_model.notes` is an OPTIONAL free-text string field documenting the µservice owner's reasoning behind the declared values; recommended for substance-bar evidence per ADR-0322.

B2.021. `capacity_model.compliance_pack_overrides` is an OPTIONAL sub-object indexed by compliance pack ID (`hipaa`, `pci-dss`, `gdpr-strict`, `soc2`, `csap`, `eu-ai-act-annex-iii`) per ADR-0251, providing per-pack overrides for the capacity model when the pack imposes a stricter floor.

B2.022. The OpenTofu shared module library at `cloud/cloud-iac/modules/<context>/<primitive>/` (per ADR-0339) MUST accept `baseline_cpu_per_tenant` + `baseline_ram_per_tenant` + `tenant_count_expected` as input variables for nodepool sizing primitives.

B2.023. Autoscaler configuration (Karpenter NodePool / HorizontalPodAutoscaler per ADR-0198) MUST derive `target_cpu` and `target_memory` from `capacity_model.baseline_cpu_per_tenant` × `tenant_count_observed` + a 1.5x headroom factor.

B2.024. Cell sizing models (per ADR-0009 + ADR-0248) MUST compute `tenants_per_cell_max` = floor(cell_total_cpu / baseline_cpu_per_tenant) for the µservices co-tenanted in that cell.

B2.025. FinOps cost projection (per ADR-0174 + ADR-0199) MUST attribute per-tenant cost using `baseline_cpu_per_tenant` × `cpu_unit_cost` + `baseline_ram_per_tenant` × `ram_unit_cost` + `storage_per_tenant` × `storage_unit_cost` as the floor; observed costs above the floor are attributed to scaling_dimension overage.

B2.026. Shuffle sharding (per ADR-0333 `oya-shuffle-sharding` Rust crate) MUST select shard sub-population using `cell_placement_class` as the tier-bucket selector.

B2.027. The manifest-schema update at `/specs/microservices/manifest-schema.json` adds the `capacity_model` block per the schema fragment in D-1.

B2.028. The corpus-wide per-µservice declaration sub-wave `15U-Capacity-Model-declaration` is queued in `/specs/master-plan-sequencing.json#realignment_wave_sequence.waves_15_plus.sub_waves` immediately upon this ADR's Acceptance.

B2.029. Per-µservice declaration follows the canonical-build phase order under ADR-0328. Phase 0 cloud-* µservices declare first; Phase 4B long-tail B2B SaaS µservices declare last.

B2.030. The CI lane set (E.1 through E.7) is REPORT-ONLY at landing and promotes to BLOCKER per the §H sunset schedule.

B2.031. The Kyverno admission policy `enforce-capacity-model-presence` (new) refuses Helm release admission for µservices whose manifest omits the `capacity_model` block beyond the per-µservice grace window.

B2.032. The OpenSLO targets per ADR-0263 emit per-cell `capacity_saturation_ratio` = (sum over co-tenanted µservices of (tenant_count × baseline_cpu_per_tenant)) / cell_total_cpu; alerting fires at saturation_ratio ≥ 0.8.

B2.033. The observability emission contract per ADR-0263 adds a `capacity_model` label set to every metric: `capacity_model_scaling_dimension`, `capacity_model_cell_placement_class`. Label cardinality is bounded (6 scaling dimensions × 5 cell tiers = 30 distinct label combinations per µservice).

B2.034. Cellular promotion gates per ADR-0341 (queued candidate) consume `capacity_model.cell_placement_class` to validate that a µservice's promotion-from-Tier-2-to-Tier-1 declaration carries the required substrate-class evidence.

B2.035. The DR matrix per ADR-0343 (queued candidate) consumes `capacity_model.storage_per_tenant` to compute the per-µservice RTO/RPO sizing of backup storage.

B2.036. The sustainability + finops dimensional model per ADR-0344 (queued candidate) consumes `capacity_model.scaling_dimension` to compute per-call CO2 grams × scaling-dimension-weighted attribution.

B2.037. The `capacity_model.cell_placement_class` declaration is the canonical anchor that the `oya-shuffle-sharding` Rust crate (per ADR-0333) consults to compute shard sub-population for blast-radius bounding.

B2.038. The 30-day sunset window starts on Acceptance. The seven new lanes (§E) promote from REPORT-ONLY to BLOCKER for new authoring at day 30; per-µservice declaration of existing manifests is sequenced under ADR-0328 and may extend the per-µservice-BLOCKER promotion until each µservice's declaration bucket lands.

B2.039. The ADR is final on Acceptance. No exception clause is provided for any µservice's omission of the `capacity_model` block after the per-µservice declaration sub-wave lands the canonical declarations.

B2.040. The ADR is announced in the realignment-wave findings aggregation and in the next ADR-0327 promotion gate report.

### B.3 What this decision does not do

- This ADR does not author per-µservice capacity_model declarations; the corpus-wide declaration sub-wave handles that.
- This ADR does not author the OpenTofu nodepool sizing-parameter wiring; that work belongs to ADR-0339 + the cloud-iac shared library updates.
- This ADR does not author the autoscaler configuration changes; that work belongs in the per-µservice Helm chart updates under the declaration sub-wave.
- This ADR does not change the cell topology decision-tree from ADR-0248.
- This ADR does not introduce a new tenant_class beyond demo_trial + paid per ADR-0330.

## Consequences

### C.1 Positive consequences

- **Single source of truth for per-µservice capacity.** Every cross-cutting capacity-planning surface (autoscaler, cell sizer, FinOps projector, shuffle-sharder) consults one declared per-µservice contract instead of reverse-inferring from runtime telemetry.
- **Per-tenant cost transparency.** `baseline_cpu_per_tenant` × `cpu_unit_cost` + `baseline_ram_per_tenant` × `ram_unit_cost` + `storage_per_tenant` × `storage_unit_cost` is the canonical per-tenant cost floor; per-tenant in-product reporting becomes deterministic.
- **Cell sizing determinism.** `tenants_per_cell_max` = floor(cell_total_cpu / sum-of-co-tenanted-baseline_cpu_per_tenant) is computable at cell-design time, not at cell-saturation time.
- **Cellular placement determinism.** `cell_placement_class` is the per-µservice declaration anchoring the µservice in the five-tier cellular topology; placement is computed at deployment time, not at ad-hoc decision time.
- **tenant_class cap-shape anchor.** Demo_trial vs paid cap-shape contracts size against `capacity_model.tenant_class_deltas` rather than per-µservice ad-hoc cap definitions.
- **Substance-bar reinforcement.** Each µservice's `capacity_model` values are bespoke (per the µservice owner's workload shape reasoning); template-stamped values fail the substance-bar lane.
- **Hyperscaler-grade rigor.** Oyatie's capacity-declaration posture aligns with AWS Service Quotas / GCP Quotas / Azure Subscription Limits / Stripe Rate Limits / Salesforce Governor Limits.
- **ADR-0338 pod_runtime_tier disambiguation.** This ADR's `cell_placement_class` is explicitly distinct from pod_runtime_tier; the co-variance table in D-6 removes the confusion that the two axes are the same.
- **Future ADR (0341..0345) anchoring.** Cellular promotion gates (ADR-0341), DR matrix (ADR-0343), and sustainability + finops dimensional model (ADR-0344) all consume capacity_model; landing this ADR first unblocks the four downstream candidates.

### C.2 Negative consequences

- **Per-µservice declaration cost.** ~77 µservices × ~30 LOC of capacity_model declaration each ≈ ~2,300 LOC of new manifest authoring under the `15U-Capacity-Model-declaration` sub-wave. The values are bespoke per µservice (substance-bar applies).
- **Per-µservice reasoning burden.** Each µservice owner reasons through baseline_cpu_per_tenant, baseline_ram_per_tenant, storage_per_tenant, connections_per_tenant for their µservice. The reasoning is non-trivial for µservices that have never had to articulate per-tenant footprint.
- **Coupling across µservices and capacity-planning tooling.** Autoscaler config, cell sizer, FinOps portal, shuffle-sharder all consume capacity_model. A capacity_model schema change cascades across every consumer.
- **Closed-enum constraints.** `scaling_dimension` and `cell_placement_class` are closed enums; new values require an ADR amendment. This is intentional (vocabulary-bound surface) but creates friction for novel scaling shapes (e.g., per_token for LLM-bound µservices).
- **CI lane authoring cost.** Seven new lanes (E.1..E.7) plus a Kyverno admission policy require implementation + soak.

### C.3 Neutral consequences

- **Pod runtime tier unchanged.** ADR-0338 pod_runtime_tier remains the admission/isolation axis; this ADR's cell_placement_class is the cellular-placement axis. The two are independently declared.
- **Tenant class unchanged.** ADR-0330 tenant_class remains a principal claim; capacity_model is a manifest declaration. The two axes do not intersect at request time.
- **Compliance pack unchanged.** ADR-0251 compliance packs remain orthogonal to capacity_model; a pack may impose stricter capacity floors via `capacity_model.compliance_pack_overrides`.
- **Cell architecture unchanged.** ADR-0009 + ADR-0248 + ADR-0333 cellular architecture is preserved; this ADR adds the per-µservice declaration that the architecture consumes.
- **OpenTofu module library unchanged structurally.** ADR-0339 shared library accepts capacity_model values as input variables; no structural change to the library.

### C.4 Engineering-rigor dimensions

| Dimension | Requirement created by this ADR | Acceptance signal |
|---|---|---|
| Maintainability | Single manifest block declares per-tenant footprint across 77+ µservices | Every workload-producing µservice manifest declares capacity_model; oya-check-capacity-model-present green |
| Capacity planning | autoscaler / cell sizer / FinOps projector consume capacity_model | Karpenter NodePool target derives from baseline_cpu_per_tenant; tenants_per_cell_max computed at deployment time |
| FinOps | per-tenant cost projection is canonical | finops-portal per-tenant dashboard segments by µservice using baseline_cpu_per_tenant × cpu_unit_cost |
| Cellular placement | cell_placement_class is the per-µservice declaration anchoring the five-tier topology | shuffle-sharder consults cell_placement_class for shard sub-population |
| Substance bar | each capacity_model value is bespoke to the µservice's workload shape | thin-template capacity_model declarations fail substance-bar lane |
| Hyperscaler alignment | AWS Service Quotas / GCP Quotas / Azure Limits precedent matched | per-µservice declared capacity is the canonical surface |
| Observability | per-µservice capacity_model_scaling_dimension + capacity_model_cell_placement_class labels emitted | bounded cardinality 30 combinations × 77 µservices |
| Compliance | per-pack compliance overrides land in capacity_model.compliance_pack_overrides | HIPAA-pinned µservices declare stricter floors |

### C.5 Hyperscaler-grade rigor application

**Named precedent.** AWS Service Quotas (api: service-quotas.amazonaws.com), Google Cloud Quotas (gcloud compute project-info describe / cloudresourcemanager API), Azure Subscription Limits (azure-resource-manager Microsoft.Capacity), Stripe Rate Limits (per-account in dashboard), Salesforce Governor Limits (per-org developer documentation). The declared-capacity-model shape is the canonical hyperscaler pattern for per-tenant footprint at scale.

**Failure-mode tree.** Failure modes:
(1) µservice forgets to declare capacity_model → CI lane REPORT-ONLY at landing, BLOCKER after sunset;
(2) µservice declares wrong scaling_dimension → quarterly tier review (per ADR-0341 candidate) catches via observed scaling-axis mismatch;
(3) µservice declares wrong cell_placement_class → ADR-0341 candidate promotion gate refuses the placement;
(4) baseline_cpu_per_tenant underestimated → autoscaler scales up but cell saturates earlier than projected; capacity_saturation_ratio alert fires;
(5) baseline_cpu_per_tenant overestimated → cell under-utilized; FinOps dashboard surfaces over-provisioning;
(6) connections_per_tenant.postgres × tenant_count exceeds PgBouncer ceiling → pre-flight check during cell sizing catches;
(7) cell_placement_class conflict with pod_runtime_tier → oya-check-capacity-model-cellular-tier-coherence lane refuses;
(8) tenant_class_deltas omitted for dual-class µservices → oya-check-capacity-model-tenant-class-deltas lane refuses.

**Capacity math.** ~77 µservices × ~30 LOC of capacity_model declaration ≈ ~2,300 LOC of bespoke manifest authoring + ~7 CI lane implementations × ~250 LOC each ≈ ~1,750 LOC of lane logic + 1 Kyverno policy ≈ ~150 LOC. Aggregate new authoring under this ADR's downstream sub-wave: ~4,200 LOC over ~3 batches × ~8 codex agents per ADR-0328 batch discipline.

**Observability hooks.** Every µservice's metric emission gains `capacity_model_scaling_dimension` + `capacity_model_cell_placement_class` labels. Cardinality is bounded at 6 × 5 = 30 distinct label combinations per µservice. Per-cell `capacity_saturation_ratio` SLI is emitted per cell per ADR-0263.

**Rollback path.** Per-µservice rollback: the µservice flips its capacity_model values in manifest.json and lands the next deployment; capacity-planning consumers re-resolve at the next reconciliation. Cell-level rollback: cell-sizing model recomputes tenants_per_cell_max with the new values. Cross-µservice rollback (e.g., abandon the capacity_model block entirely) requires a new ADR superseding this one.

**Multi-region awareness.** Each region's cells consume the same capacity_model values; per-region multipliers (e.g., higher network latency for cross-region calls) are captured in observed-metric overhead, not in the declared model.

**Sovereign-cell awareness.** Sovereign cells (per ADR-0240) consume the same capacity_model declaration; per-pack overrides land in `capacity_model.compliance_pack_overrides` for compliance-pack-imposed stricter floors.

**Versioning + deprecation.** This ADR is versioned per ADR-0108. The capacity_model schema is `schema_version: "1.0"` (per manifest-schema.json const). Field additions are minor-version bumps; field removals or enum-value retirements are major-version bumps requiring an amendment ADR.

## D. Detailed mechanics — ten declaration surfaces

The capacity_model block touches ten declaration surfaces in the manifest schema, the per-µservice authoring path, and the cross-cutting capacity-planning tooling. Subsections D-1 through D-10 enumerate each surface. Numbering is normative.

### D-1: Manifest schema fragment

D-1.1. The schema at `/specs/microservices/manifest-schema.json` adds a top-level `capacity_model` property whose shape is:

```json
"capacity_model": {
  "type": "object",
  "description": "ADR-0340 — Per-µservice machine-readable capacity declaration driving autoscaler + cell sizing + FinOps projection + cellular placement determinism.",
  "additionalProperties": false,
  "required": [
    "baseline_cpu_per_tenant",
    "baseline_ram_per_tenant",
    "storage_per_tenant",
    "connections_per_tenant",
    "scaling_dimension",
    "cell_placement_class"
  ],
  "properties": {
    "baseline_cpu_per_tenant": {
      "type": "number",
      "minimum": 0.001,
      "maximum": 1000.0,
      "description": "vCPU at steady state per active tenant."
    },
    "baseline_ram_per_tenant": {
      "type": "integer",
      "minimum": 1,
      "maximum": 1048576,
      "description": "MiB at steady state per active tenant."
    },
    "storage_per_tenant": {
      "type": "integer",
      "minimum": 0,
      "maximum": 1048576,
      "description": "GB at steady state per active tenant."
    },
    "connections_per_tenant": {
      "type": "object",
      "additionalProperties": false,
      "required": ["valkey", "postgres", "outbound_http"],
      "properties": {
        "valkey": {"type": "integer", "minimum": 0, "maximum": 1024},
        "postgres": {"type": "integer", "minimum": 0, "maximum": 1024},
        "outbound_http": {"type": "integer", "minimum": 0, "maximum": 1024}
      }
    },
    "scaling_dimension": {
      "enum": ["per_user", "per_request", "per_capability", "per_message", "per_query", "per_workflow_run"]
    },
    "cell_placement_class": {
      "enum": ["Tier-0", "Tier-1", "Tier-2", "Tier-3", "Tier-4"]
    },
    "tenant_class_deltas": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "demo_trial": {"$ref": "#/properties/capacity_model"},
        "paid": {"$ref": "#/properties/capacity_model"}
      }
    },
    "compliance_pack_overrides": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "hipaa": {"$ref": "#/properties/capacity_model"},
        "pci-dss": {"$ref": "#/properties/capacity_model"},
        "gdpr-strict": {"$ref": "#/properties/capacity_model"},
        "soc2": {"$ref": "#/properties/capacity_model"},
        "csap": {"$ref": "#/properties/capacity_model"},
        "eu-ai-act-annex-iii": {"$ref": "#/properties/capacity_model"}
      }
    },
    "notes": {"type": "string"}
  }
}
```

D-1.2. The schema_version stays at `"1.0"` because the `capacity_model` property is additive (no existing field is removed or retyped). Per ADR-0108, additive schema changes do not bump the major schema_version.

D-1.3. The schema fragment is applied to `/specs/microservices/manifest-schema.json` at this ADR's landing (the schema-fragment edit is in scope for this ADR; per-µservice declarations are out of scope and sequenced as the follow-on sub-wave).

D-1.4. The schema requires the six core fields when the `capacity_model` block is present. The block itself is OPTIONAL at the top-level manifest until the per-µservice declaration sub-wave lands; after the sub-wave lands, the block is REQUIRED for every workload-producing µservice via the CI lane.

D-1.5. The schema does NOT require the optional `tenant_class_deltas` or `compliance_pack_overrides` sub-objects; they are advisory and used only when per-µservice override semantics apply.

### D-2: Per-µservice manifest authoring example

D-2.1. Concrete `capacity_model` example for a Tier-3 application µservice (CRM):

```json
{
  "name": "crm",
  "capacity_model": {
    "baseline_cpu_per_tenant": 0.05,
    "baseline_ram_per_tenant": 128,
    "storage_per_tenant": 5,
    "connections_per_tenant": {
      "valkey": 2,
      "postgres": 5,
      "outbound_http": 20
    },
    "scaling_dimension": "per_user",
    "cell_placement_class": "Tier-3",
    "notes": "Active CRM seat: ~50 milli-vCPU steady-state + 128 MiB RSS + 5 GB pipeline+contact storage; 2 Valkey conn for cache + session; 5 Postgres conn for OLTP; 20 outbound HTTP for marketing-automation / community / mail webhooks."
  }
}
```

D-2.2. Concrete `capacity_model` example for a Tier-0 substrate µservice (cloud-kms):

```json
{
  "name": "cloud-kms",
  "capacity_model": {
    "baseline_cpu_per_tenant": 0.01,
    "baseline_ram_per_tenant": 32,
    "storage_per_tenant": 1,
    "connections_per_tenant": {
      "valkey": 1,
      "postgres": 2,
      "outbound_http": 0
    },
    "scaling_dimension": "per_capability",
    "cell_placement_class": "Tier-0",
    "notes": "KMS key derivation is lightweight per-tenant; bulk of capacity is in HSM-bound substrate not per-tenant amortized; storage is per-tenant key metadata + audit-emission registers; no outbound HTTP (KMS is sealed by design).",
    "compliance_pack_overrides": {
      "hipaa": {
        "baseline_cpu_per_tenant": 0.02,
        "baseline_ram_per_tenant": 64,
        "storage_per_tenant": 2,
        "connections_per_tenant": {"valkey": 1, "postgres": 3, "outbound_http": 0},
        "scaling_dimension": "per_capability",
        "cell_placement_class": "Tier-0",
        "notes": "HIPAA-pinned cells require enhanced audit-chain seal latency budget; baseline doubled."
      }
    }
  }
}
```

D-2.3. Concrete `capacity_model` example for a Tier-2 capability µservice (workflow-engine) with tenant_class deltas:

```json
{
  "name": "workflow-engine",
  "capacity_model": {
    "baseline_cpu_per_tenant": 0.1,
    "baseline_ram_per_tenant": 256,
    "storage_per_tenant": 10,
    "connections_per_tenant": {"valkey": 3, "postgres": 4, "outbound_http": 15},
    "scaling_dimension": "per_workflow_run",
    "cell_placement_class": "Tier-2",
    "tenant_class_deltas": {
      "demo_trial": {
        "baseline_cpu_per_tenant": 0.02,
        "baseline_ram_per_tenant": 64,
        "storage_per_tenant": 1,
        "connections_per_tenant": {"valkey": 1, "postgres": 2, "outbound_http": 5},
        "scaling_dimension": "per_workflow_run",
        "cell_placement_class": "Tier-2",
        "notes": "demo_trial clamped to fit OCI Always Free 2× Ampere A1 ARM 4 OCPU + 24 GiB compute budget per feedback_oci_always_free_maximization_2026_05_20."
      },
      "paid": {
        "baseline_cpu_per_tenant": 0.1,
        "baseline_ram_per_tenant": 256,
        "storage_per_tenant": 10,
        "connections_per_tenant": {"valkey": 3, "postgres": 4, "outbound_http": 15},
        "scaling_dimension": "per_workflow_run",
        "cell_placement_class": "Tier-2"
      }
    }
  }
}
```

D-2.4. Concrete `capacity_model` example for a Tier-4 edge µservice (api-gateway):

```json
{
  "name": "api-gateway",
  "capacity_model": {
    "baseline_cpu_per_tenant": 0.005,
    "baseline_ram_per_tenant": 16,
    "storage_per_tenant": 0,
    "connections_per_tenant": {"valkey": 1, "postgres": 0, "outbound_http": 30},
    "scaling_dimension": "per_request",
    "cell_placement_class": "Tier-4",
    "notes": "Edge data-plane is stateless; minimal per-tenant footprint; high outbound HTTP for upstream µservice routing; storage=0; Postgres=0 (api-gateway does not persist tenant data)."
  }
}
```

D-2.5. The per-µservice owner's substance is in (a) the numerical values chosen, (b) the scaling_dimension and cell_placement_class assignments, (c) the notes field justifying the values, and (d) any tenant_class_deltas or compliance_pack_overrides reasoning. Template-stamped values fail the substance-bar lane per ADR-0322.

### D-3: Scaling dimension closed enum

D-3.1. The `scaling_dimension` enum is closed at six values: `per_user`, `per_request`, `per_capability`, `per_message`, `per_query`, `per_workflow_run`.

D-3.2. `per_user` covers µservices whose load is proportional to active named-user count (CRM seats, drive users, docs/sheets/slides authors, calendar users, video-call participants, learning-management learners).

D-3.3. `per_request` covers µservices whose load is proportional to API request volume (api-gateway data-plane, application gateway, public REST/AsyncAPI/proto3 endpoints).

D-3.4. `per_capability` covers µservices whose load is proportional to issued capability-scope count (cloud-iam, Cedar policy engine evaluations, principal-claim derivation paths).

D-3.5. `per_message` covers µservices whose load is proportional to message volume (messenger / E2EE MLS, mail, notifications, audit-chain emission, observability metric+log emission).

D-3.6. `per_query` covers µservices whose load is proportional to query volume (vector-store, search, OLAP queries via Iceberg+ClickHouse per ADR-0337, knowledge-graph queries).

D-3.7. `per_workflow_run` covers µservices whose load is proportional to workflow execution count (workflow-engine, workflow-studio, agent-runtime dispatches per ADR-0255, ad-hoc batch jobs).

D-3.8. New scaling dimensions (e.g., `per_token` for LLM-bound µservices, `per_byte` for object-storage-bound µservices) require an ADR amendment to expand the enum.

D-3.9. The enum is closed deliberately to bound the vocabulary that cross-cutting tooling (FinOps portal, cell sizer, autoscaler) needs to understand.

D-3.10. The CI lane `oya-check-capacity-model-scaling-dimension` refuses values outside the closed enum.

### D-4: Cell placement class closed enum (per ADR-0248 cellular tier convention)

D-4.1. The `cell_placement_class` enum is closed at five values: `Tier-0`, `Tier-1`, `Tier-2`, `Tier-3`, `Tier-4`. The numbering follows the ADR-0248 cellular criticality convention (Tier-0 = highest blast-radius / most isolated; Tier-4 = edge).

D-4.2. `Tier-0` (Foundation cells) hosts µservices whose compromise unlocks tenant-wide impersonation, key-material exposure, or audit-trail tampering: cloud-iam, cloud-kms, audit-chain primary, payments-pci core.

D-4.3. `Tier-1` (Substrate cells) hosts µservices that provide tenant-data-plane substrate without being Foundation: cloud-secrets, observability backbone, intelligence transport (provider-router + BYOK resolver), ontology projection backbone, consent-graph.

D-4.4. `Tier-2` (Capability cells) hosts µservices that materialize tenant-facing capabilities on top of Foundation + Substrate: workflow-engine, intelligence agent dispatch, marketplace catalog, identity claim issuance (non-Foundation paths), tenancy.

D-4.5. `Tier-3` (Application cells) hosts µservices that materialize tenant-facing application surfaces: crm, marketing-automation, contract-lifecycle-management, itsm, community, social, drive, docs, sheets, slides, calendar, video-call, meet, mail, learning-management, performance-management, the ERP family, the healthcare family.

D-4.6. `Tier-4` (Edge cells) hosts µservices that operate the edge data-plane: api-gateway data-plane, Envoy edge, ztunnel, CDN edge cache, dns-edge.

D-4.7. The Tier-0 = highest blast-radius convention is deliberate per ADR-0248 + ADR-0338 numbering alignment.

D-4.8. The CI lane `oya-check-capacity-model-cell-placement` refuses cell_placement_class values outside the closed enum and refuses Tier-0/Tier-1 declarations without an audit-chain seal event reference per ADR-0263.

D-4.9. The enum is closed deliberately to align with the ADR-0248 cellular tier numbering; new tiers require an ADR amendment to ADR-0248.

D-4.10. The shuffle-sharding crate (`oya-shuffle-sharding` per ADR-0333) consults `cell_placement_class` to select shard sub-population for blast-radius bounding.

### D-5: Connections sub-object — Valkey + Postgres + outbound HTTP

D-5.1. The `connections_per_tenant` sub-object captures the three substrate connection kinds that are bounded by infrastructure ceilings: Valkey, Postgres, outbound HTTP.

D-5.2. `connections_per_tenant.valkey` is the number of TCP connections the µservice opens against Valkey clusters per active tenant. The ceiling reflects Valkey's per-instance connection budget (typical 10,000 connections per Valkey instance; per-tenant footprint × tenant_count MUST stay below 10,000 per Valkey instance per cell).

D-5.3. `connections_per_tenant.postgres` is the number of connections the µservice opens against PostgreSQL per active tenant. The ceiling reflects PostgreSQL's max_connections (default 100; typical PgBouncer transaction-pool ceiling 1,000-10,000); per-tenant footprint × tenant_count MUST stay below the PgBouncer ceiling per cell.

D-5.4. `connections_per_tenant.outbound_http` is the number of outbound HTTP/3 (per ADR-0253) + HTTP/2 + HTTP/1.1 connections the µservice opens to upstream services per active tenant. The ceiling reflects the egress NAT instance's ephemeral port budget (typical 64K ephemeral ports per IP; per-tenant footprint × tenant_count MUST stay below 64K per egress NAT IP).

D-5.5. The connections sub-object is REQUIRED (all three sub-fields present); zero is permitted for µservices that do not use a given substrate (e.g., api-gateway declares postgres = 0).

D-5.6. The CI lane `oya-check-capacity-model-units` validates the sub-object presence + sub-field types + sanity-ceiling bounds.

D-5.7. The cell sizer (per ADR-0009 + ADR-0248) computes per-cell connection saturation from sum(over co-tenanted µservices)(tenant_count × connections_per_tenant.<kind>) and alerts when saturation exceeds 0.8 of the per-kind ceiling.

D-5.8. Future substrate connection kinds (e.g., Kafka per ADR-0050, Iceberg-catalog per ADR-0337, OpenSearch) may be added via an ADR amendment expanding the sub-object schema.

### D-6: Co-variance table — pod_runtime_tier vs cell_placement_class

D-6.1. `pod_runtime_tier` per ADR-0338 and `cell_placement_class` per this ADR are DISTINCT axes that co-vary by the table below. The CI lane `oya-check-capacity-model-cellular-tier-coherence` refuses combinations marked INVALID.

| `pod_runtime_tier` (ADR-0338) | `cell_placement_class` (ADR-0340) | Combination |
|---|---|---|
| Tier 0 (tenant-customer untrusted code; Kata) | Tier-0 (Foundation) | INVALID — tenant-customer code does not run in Foundation cells |
| Tier 0 (tenant-customer untrusted code; Kata) | Tier-1 (Substrate) | INVALID — substrate cells do not host tenant-customer code |
| Tier 0 (tenant-customer untrusted code; Kata) | Tier-2 (Capability) | VALID — workflow-studio tenant-step executor, agent-runtime tenant capability |
| Tier 0 (tenant-customer untrusted code; Kata) | Tier-3 (Application) | VALID — marketplace plugin executor, developer-sdk uploaded modules |
| Tier 0 (tenant-customer untrusted code; Kata) | Tier-4 (Edge) | INVALID — edge cells host stateless proxies, not tenant-customer code |
| Tier 1 (substrate touching tenant data plane; Kata) | Tier-0 (Foundation) | VALID — cloud-iam, cloud-kms, audit-chain primary, payments-pci core |
| Tier 1 (substrate touching tenant data plane; Kata) | Tier-1 (Substrate) | VALID — cloud-secrets, observability backbone, intelligence transport |
| Tier 1 (substrate touching tenant data plane; Kata) | Tier-2 (Capability) | VALID — messenger MLS keys, intelligence transport (when capability-bound) |
| Tier 1 (substrate touching tenant data plane; Kata) | Tier-3 (Application) | INVALID — application cells do not host substrate-touching code; promote the µservice to Tier-2 or split |
| Tier 1 (substrate touching tenant data plane; Kata) | Tier-4 (Edge) | INVALID — edge cells do not host substrate-touching code |
| Tier 2 (first-party application; runc) | Tier-0 (Foundation) | INVALID — Foundation cells host only Tier-0 + Tier-1 pods |
| Tier 2 (first-party application; runc) | Tier-1 (Substrate) | INVALID — Substrate cells host only Tier-1 pods |
| Tier 2 (first-party application; runc) | Tier-2 (Capability) | VALID — workflow-engine, intelligence agent dispatch, marketplace catalog |
| Tier 2 (first-party application; runc) | Tier-3 (Application) | VALID — crm, marketing-automation, drive, docs, sheets, slides, calendar, social, community |
| Tier 2 (first-party application; runc) | Tier-4 (Edge) | INVALID — Edge cells host only Tier-3 pods |
| Tier 3 (edge / static / perf-critical; runc-edge) | Tier-0 (Foundation) | INVALID |
| Tier 3 (edge / static / perf-critical; runc-edge) | Tier-1 (Substrate) | INVALID |
| Tier 3 (edge / static / perf-critical; runc-edge) | Tier-2 (Capability) | INVALID |
| Tier 3 (edge / static / perf-critical; runc-edge) | Tier-3 (Application) | INVALID |
| Tier 3 (edge / static / perf-critical; runc-edge) | Tier-4 (Edge) | VALID — api-gateway data-plane, Envoy edge, ztunnel, CDN edge cache |

D-6.2. The table is normative. The CI lane `oya-check-capacity-model-cellular-tier-coherence` consults the table at PR-time.

D-6.3. The table preserves the ADR-0338 cellular-tier-co-varies-with-runtime-tier guidance from §B2.037 of that ADR; this ADR's contribution is making the table machine-checkable at manifest authoring time.

### D-7: Autoscaler input wiring (Karpenter NodePool + HorizontalPodAutoscaler)

D-7.1. Karpenter NodePool resources (per ADR-0198) consume `capacity_model.baseline_cpu_per_tenant` × `tenant_count_expected` to compute `cpu_request` and `cpu_limit` targets.

D-7.2. HorizontalPodAutoscaler (HPA) `targetCPUUtilizationPercentage` is derived as: `target = floor((baseline_cpu_per_tenant × tenant_count_observed × headroom_factor) / pod_cpu_capacity × 100)` where `headroom_factor = 1.5` (50% headroom per cell sizing convention).

D-7.3. KEDA ScaledObject (per ADR-0198) for non-CPU scaling targets consumes `capacity_model.scaling_dimension` to select the metric trigger:
  - `per_user` → KEDA Prometheus trigger on `active_users_total` metric.
  - `per_request` → KEDA Prometheus trigger on `http_requests_per_second` metric.
  - `per_capability` → KEDA Prometheus trigger on `capability_issuance_rate` metric.
  - `per_message` → KEDA Prometheus trigger on `messages_per_second` metric.
  - `per_query` → KEDA Prometheus trigger on `queries_per_second` metric.
  - `per_workflow_run` → KEDA Prometheus trigger on `workflow_runs_per_second` metric.

D-7.4. The Karpenter NodePool selector consumes `cell_placement_class` to route nodes to the correct cell tier:
  - Tier-0/Tier-1/Tier-2 → kata-pool (per ADR-0338 + ADR-0339 nodepool primitive).
  - Tier-3 → runc-pool (per ADR-0338 + ADR-0339).
  - Tier-4 → runc-edge-pool (per ADR-0338 + ADR-0339).

D-7.5. The per-µservice Helm chart `values.yaml` references `capacity_model.*` fields via OpenTofu interpolation; per-µservice deployment authors do not duplicate the values.

### D-8: FinOps cost projection wiring (per ADR-0174 + ADR-0199)

D-8.1. The FinOps portal (per ADR-0199 cost-attribution namespace) computes per-tenant cost floor as: `cost_floor = baseline_cpu_per_tenant × cpu_unit_cost + baseline_ram_per_tenant × ram_unit_cost / 1024 + storage_per_tenant × storage_unit_cost` where unit costs are sourced from the FinOps cost-tag vocabulary at `registry/finops/cost-tag-vocabulary.yaml`.

D-8.2. The FinOps portal projects per-tenant cost across the µservice catalog by summing the floor across µservices the tenant consumes.

D-8.3. The CI lane `oya-check-capacity-model-finops-anchor` refuses capacity_model values that diverge from the per-µservice `finops.cost_center` declaration by more than the soft-band tolerance (default 20%); divergence beyond the band requires a notes-field justification.

D-8.4. The per-tenant cost projection is exposed in (a) the FinOps portal admin dashboard, (b) the in-product tenant-facing cost transparency UI, and (c) the regulatory cost-disclosure CSRD / SB-253 / SEC climate-disclosure reports per ADR-0344 candidate.

D-8.5. Cost projection per scaling_dimension surfaces in the FinOps portal as dimensional rollups (per-user-cost, per-request-cost, per-capability-cost, per-message-cost, per-query-cost, per-workflow-run-cost).

### D-9: Cell sizing inputs (per ADR-0009 + ADR-0248)

D-9.1. The cell sizing model (per ADR-0009 cell architecture + ADR-0248 cellular topology) consumes `capacity_model.baseline_cpu_per_tenant` + `baseline_ram_per_tenant` + `storage_per_tenant` to compute per-cell tenant capacity.

D-9.2. `tenants_per_cell_max` (for a cell co-tenanting µservices A, B, C, ...) is computed as: `tenants_per_cell_max = floor(cell_total_cpu / sum(over µservices)(baseline_cpu_per_tenant_µ))` and the analogous formulas for RAM, storage, and per-substrate connections.

D-9.3. Per-cell `capacity_saturation_ratio` is computed at runtime as: `saturation = sum(over µservices)(tenant_count × baseline_cpu_per_tenant) / cell_total_cpu`.

D-9.4. The OpenSLO target per ADR-0263 emits `capacity_saturation_ratio` per cell per minute; alerting fires at `saturation >= 0.8` (warning) or `saturation >= 0.95` (page).

D-9.5. The cell sizing model is consumed by (a) capacity-planning dashboards, (b) the new-tenant onboarding admission gate (per ADR-0244 tenant scoping), and (c) the shuffle-sharding crate (per ADR-0333).

D-9.6. Per-cell connection saturation is computed per substrate kind: `valkey_saturation = sum(over µservices)(tenant_count × connections_per_tenant.valkey) / valkey_instance_max_connections` and analogous for postgres, outbound_http.

### D-10: Sub-wave queue + sunset

D-10.1. The corpus-wide per-µservice capacity_model declaration sub-wave is queued in `/specs/master-plan-sequencing.json#realignment_wave_sequence.waves_15_plus.sub_waves` as `15U-Capacity-Model-declaration` immediately upon this ADR's Acceptance.

D-10.2. The sub-wave is sequenced under ADR-0328 canonical-build phase order: Phase 0 cloud-* µservices declare first; Phase 4B long-tail B2B SaaS µservices declare last.

D-10.3. Per-µservice declaration follows ADR-0322 substance-bar discipline + ADR-0324 anti-script authoring (no template-stamped values).

D-10.4. The 30-day sunset window starts on this ADR's Acceptance. The seven new lanes (E.1..E.7) and the Kyverno admission policy promote from REPORT-ONLY to BLOCKER for new authoring at day 30; per-µservice declaration of existing manifests is sequenced under ADR-0328 and may extend the per-µservice-BLOCKER promotion until each µservice's declaration bucket lands.

D-10.5. The sub-wave dispatch mode is codex-bucket fan-out per the ADR-0328 batch discipline; per-µservice bespoke authoring under ADR-0322 substance-bar.

D-10.6. The sub-wave depends on this ADR's Acceptance + the schema-fragment edit at `/specs/microservices/manifest-schema.json` landing.

D-10.7. The sub-wave does not depend on ADR-0341 (cellular promotion gates) or ADR-0343 (DR matrix) or ADR-0344 (sustainability + finops dimensional model); those candidate ADRs depend on this one.

D-10.8. The sub-wave's completion criteria are: (a) every workload-producing µservice declares `capacity_model`; (b) seven lanes promote to BLOCKER; (c) Kyverno policy promotes to enforce mode; (d) cell sizing dashboards consume `capacity_model` declarations; (e) FinOps portal consumes `capacity_model` declarations; (f) autoscaler config references `capacity_model` interpolations.

## E. Enforcement-by-lanes

E.1 **`oya-check-capacity-model-present`** (new CI lane) — validates per-µservice manifest declares `capacity_model` block when the µservice produces a workload. REPORT-ONLY at landing; BLOCKER per-µservice as each declaration lands.

E.2 **`oya-check-capacity-model-units`** (new CI lane) — refuses non-canonical units; baseline_cpu_per_tenant MUST be vCPU as a decimal; baseline_ram_per_tenant MUST be MiB as an integer; storage_per_tenant MUST be GB as an integer; connections_per_tenant sub-fields MUST be non-negative integers within sanity ceilings. REPORT-ONLY at landing; BLOCKER 30 days post-Acceptance.

E.3 **`oya-check-capacity-model-scaling-dimension`** (new CI lane) — refuses scaling_dimension values outside the closed enum {per_user, per_request, per_capability, per_message, per_query, per_workflow_run}. REPORT-ONLY at landing; BLOCKER 30 days post-Acceptance.

E.4 **`oya-check-capacity-model-cell-placement`** (new CI lane) — refuses cell_placement_class values outside {Tier-0, Tier-1, Tier-2, Tier-3, Tier-4} and refuses Tier-0/Tier-1 declarations without an audit-chain seal event reference per ADR-0263. REPORT-ONLY at landing; BLOCKER 30 days post-Acceptance.

E.5 **`oya-check-capacity-model-tenant-class-deltas`** (new CI lane) — refuses capacity_model declarations that omit demo_trial-vs-paid delta rows when the µservice serves both tenant classes per ADR-0330 + ADR-0331. REPORT-ONLY at landing; BLOCKER per-µservice as each declaration lands.

E.6 **`oya-check-capacity-model-finops-anchor`** (new CI lane) — refuses capacity_model values that diverge from the per-µservice FinOps cost-center declaration by more than the soft-band tolerance recorded in registry/finops/cost-tag-vocabulary.yaml. REPORT-ONLY at landing; BLOCKER 60 days post-Acceptance.

E.7 **`oya-check-capacity-model-cellular-tier-coherence`** (new CI lane) — refuses cell_placement_class values that conflict with the µservice's declared pod_runtime_tier per ADR-0338 + ADR-0248 co-variance table in D-6. REPORT-ONLY at landing; BLOCKER 30 days post-Acceptance.

E.8 **Kyverno admission policy `enforce-capacity-model-presence`** (new) — refuses Helm release admission for µservices whose manifest omits the `capacity_model` block beyond the per-µservice grace window. REPORT-ONLY (`validationFailureAction: audit`) at landing; promoted to BLOCKER (`validationFailureAction: enforce`) per §H sunset.

E.9 **`oya-governance-substance-bar`** (existing) — applies the substance bar to per-µservice capacity_model authoring; refuses template-stamped values + zero-substance notes fields.

E.10 **Multispectrum review v2.4.0** (existing) — reviews each ADR + each substantive capacity_model authoring under the F-family + M-family + A-family facets per ADR-0322.

## F. Alternatives Rejected

F.1 **No capacity declaration; rely on runtime telemetry.** Status-quo. Rejected because: every cross-cutting capacity-planning surface today reverse-infers from runtime metrics, which is reactive rather than predictive; new-tenant onboarding cannot pre-commit cell capacity; FinOps projection is post-hoc; shuffle-sharding has no tier anchor; the ADR-0338 + ADR-0339 + ADR-0331 + ADR-0330 surfaces all explicitly cite "missing capacity declaration" as an input gap.

F.2 **Single combined-tier field replacing pod_runtime_tier + cell_placement_class.** Collapse the two axes into one. Rejected because: pod_runtime_tier governs admission/isolation primitive (Kata vs runc vs runc-edge) which is a per-pod runtime decision; cell_placement_class governs cellular placement (Foundation cell vs Substrate cell vs Capability cell vs Application cell vs Edge cell) which is a per-cell topology decision. The two decisions are made by different actors (Kyverno admission vs Karpenter NodePool selector + shuffle-sharder) and at different times (pod admission vs cell deployment). Collapsing them would conflate concerns. The co-variance table in D-6 captures the relationship without collapsing.

F.3 **Open enum for scaling_dimension.** Allow µservice owners to declare arbitrary scaling dimensions. Rejected because: cross-cutting tooling (FinOps portal, cell sizer, autoscaler KEDA triggers) needs a closed vocabulary to map dimensions to canonical metric triggers; an open enum would require per-µservice ad-hoc metric trigger configuration in every cross-cutting tool; future expansion of the enum is explicitly supported via ADR amendment.

F.4 **Per-tenant_class capacity_model only (no top-level baseline).** Force µservice owners to declare separate demo_trial + paid capacity_models with no top-level default. Rejected because: many µservices (substrate µservices like cloud-iam, cloud-kms) serve both tenant classes identically; forcing separate declarations creates duplication; the top-level baseline + optional tenant_class_deltas pattern is the cleaner shape.

F.5 **Float values for storage_per_tenant (e.g., 0.5 GB).** Allow sub-GB precision for storage. Rejected because: storage granularity at the per-tenant level is typically integer-GB anyway (block-storage allocation granularity is 1 GB on AWS EBS, 50 GB on OCI block storage); sub-GB precision is false precision; the integer constraint forces realistic granularity.

F.6 **HCL-DSL declaration in iac/ directories instead of manifest.json.** Move capacity_model into OpenTofu HCL. Rejected because: capacity_model is consumed by autoscaler config, cell sizer, FinOps portal, shuffle-sharder — none of which read HCL; the manifest.json surface is the canonical machine-readable µservice declaration per ADR-0212; HCL is per-deployment-context.

F.7 **External quota service (Stripe-like SaaS).** Source capacity_model from an external SaaS quota system. Rejected because: per-µservice ownership coherence per `feedback_microservice_ownership_coherence_2026_05_20` requires the declaration to live alongside the µservice's other manifest content; external dependency creates supply-chain risk; the in-tree manifest path provides better provenance.

F.8 **Skip cell_placement_class; rely on pod_runtime_tier alone.** Use ADR-0338's pod_runtime_tier as the sole tier declaration. Rejected because: pod_runtime_tier and cell_placement_class capture different concerns (admission vs placement); the ADR-0248 cellular tier numbering is independently required for shuffle-sharding + blast-radius reasoning; conflating the two axes would lose the cellular topology declaration.

F.9 **No connections_per_tenant sub-object.** Skip the connections dimension entirely. Rejected because: connection pool exhaustion is a well-known production failure mode at hyperscaler scale; per-tenant connection footprint × tenant_count is the only way to size pool/proxy/NAT capacity ahead of failure.

F.10 **Cedar-based capacity_model enforcement.** Use Cedar policies to gate capacity_model declarations. Rejected per ADR-0183 (policy-engine separation): Cedar is application-layer authorization; capacity_model declaration enforcement lives at CI lane + Kyverno admission time, not at Cedar request time.

## G. Multispectrum review v2.4.0

Per ADR-0322 multispectrum review v2.4.0 doctrine, this ADR is reviewed across F1-F9 + M1 + M2 + A1-A7 facets. Each facet's verdict is recorded as evidence under `evidence/multispectrum-review/ADR-0340/<facet>.md` at landing time.

- **F1 Correctness.** The six required fields cover the canonical per-tenant capacity dimensions (CPU, RAM, storage, connections, scaling axis, cellular placement); the closed-enum constraints align with downstream tooling vocabulary; the schema fragment is well-typed and additive. PASS.
- **F2 Readability.** The ADR has explicit section structure (A..I), numbered decision clauses (B2.001..B2.040), and per-tier mapping tables (D-6 co-variance table). The reader can locate any rule in O(log N). PASS.
- **F3 Architecture.** The capacity_model block is the per-µservice declaration anchor for autoscaler + cell sizer + FinOps portal + shuffle-sharder; the co-variance table separates pod_runtime_tier (admission) from cell_placement_class (placement); the schema is additive to manifest-schema.json. PASS.
- **F4 Security.** No tenant-confidential information lives in capacity_model; the declaration is per-µservice, not per-tenant; cell_placement_class Tier-0/Tier-1 declarations require audit-chain seal event references. PASS.
- **F5 Performance.** Cell sizing determinism enables predictive scaling rather than reactive; per-tenant cost floor enables pre-commit billing accuracy; per-cell saturation alerting fires at 0.8 (warning) and 0.95 (page) per ADR-0263. PASS.
- **F6 Maintainability.** Single block per µservice; closed-enum constraints bound the vocabulary; field additions are minor-version bumps via ADR amendment. PASS.
- **F7 Observability.** capacity_model_scaling_dimension + capacity_model_cell_placement_class labels additive per ADR-0263; per-cell capacity_saturation_ratio SLI per cell. Bounded cardinality 30 combinations per µservice. PASS.
- **F8 Compliance.** capacity_model.compliance_pack_overrides supports per-pack stricter floors (HIPAA, PCI-DSS, GDPR-strict, SOC2, CSAP, EU-AI-Act Annex III) per ADR-0251. PASS.
- **F9 Cost.** Per-µservice per-tenant cost floor is the canonical FinOps projection input; cost transparency surfaces at admin + tenant-facing + regulatory dimensions. PASS.
- **M1 Authority chain.** ADR-0212 (buildability doctrine) amended; ADR-0244 (tenant scoping) anchored in concrete numbers; ADR-0248 (cellular tier numbering) anchored per µservice; ADR-0331 (tenant_class adoption) anchored; ADR-0338 (pod runtime tier) co-varied; ADR-0339 (IaC module library) consumes capacity_model. PASS.
- **M2 Substance bar.** Each decision clause is bespoke to the capacity_model declaration shape; no template stamping; the ADR is authored under ADR-0322 + ADR-0328 substance-bar discipline; per-µservice declaration substance is each owner's bespoke reasoning. PASS.
- **A1 Naming.** `capacity_model` follows the manifest field-block naming convention; sub-field names (baseline_cpu_per_tenant, baseline_ram_per_tenant, etc.) follow snake_case convention; closed-enum values (per_user, per_request, etc. + Tier-0..Tier-4) follow the BNF v4 grammar. PASS.
- **A2 Documentation.** This ADR + the manifest-schema fragment + the seven CI lane source-of-truth + the Kyverno policy + the canonical-primitives addition satisfy ADR-0063 doc coverage. PASS.
- **A3 Structure.** Ten declaration surfaces (D-1..D-10) follow the ADR-0331 + ADR-0336 + ADR-0338 + ADR-0339 detailed-mechanics pattern. PASS.
- **A4 Architecture.** Schema-additive amendment pattern follows the ADR-0338 manifest-schema-field-addition precedent. PASS.
- **A5 Dependency.** No new external dependencies introduced; the schema fragment is in-tree at /specs/microservices/manifest-schema.json; downstream consumers (autoscaler, cell sizer, FinOps portal, shuffle-sharder) are all in-tree µservices/tools per ADR-0211. PASS.
- **A6 Schema.** `capacity_model` JSON schema fragment in /specs/microservices/manifest-schema.json is well-typed; closed enums + sanity-ceiling bounds + required-fields constraints are explicit. PASS.
- **A7 Algorithm.** Autoscaler input derivation algorithm specified in D-7.2; cell sizing algorithm specified in D-9.2 (tenants_per_cell_max formula); FinOps cost projection algorithm specified in D-8.1 (cost_floor formula). PASS.

Aggregate verdict: PASS (all facets). The multispectrum-review evidence pack is filed at landing time.

## H. Enforcement (CI lanes + Kyverno) + Sunset

H.1 **Enforcement vectors.**
  - Seven CI lanes (E.1..E.7) at PR-time on every manifest change.
  - Kyverno ClusterPolicy `enforce-capacity-model-presence` (E.8) at Helm release admission time in every cell.
  - Substance-bar lane (E.9) at per-µservice capacity_model authoring time.
  - Multispectrum review v2.4.0 (E.10) at each substantive declaration.

H.2 **Sunset schedule.**
  - Day 0 = ADR Acceptance.
  - Day 0..30 = REPORT-ONLY soak. CI lanes and Kyverno policy run in audit mode. Findings produced per-PR + per-admission but not blocking.
  - Day 30 = E.2, E.3, E.4, E.7 promote to BLOCKER for new authoring (post-Acceptance new µservices MUST declare capacity_model).
  - Day 30..60 = E.6 promotes to BLOCKER (FinOps anchor divergence).
  - Per-µservice schedule = E.1, E.5 promote to BLOCKER per-µservice as each µservice's declaration bucket lands (under ADR-0328 canonical-build phase order).
  - At sub-wave completion = E.8 Kyverno policy promotes to enforce mode.

H.3 **Sunset criteria.**
  - The schema fragment at /specs/microservices/manifest-schema.json has landed.
  - The seven CI lanes are implemented + soaked at REPORT-ONLY for at least 30 days without unexpected false-positive denials.
  - The corpus-wide manifest declaration sub-wave `15U-Capacity-Model-declaration` has completed per ADR-0328 batch discipline.
  - The autoscaler config (Karpenter NodePool + HPA + KEDA ScaledObject) consumes capacity_model values per D-7.
  - The cell sizing model (per ADR-0009 + ADR-0248) consumes capacity_model values per D-9.
  - The FinOps portal (per ADR-0174 + ADR-0199) consumes capacity_model values per D-8.
  - The shuffle-sharding crate (per ADR-0333) consumes cell_placement_class per D-4.10.

H.4 **Post-sunset behavior.**
  - New µservices that omit `capacity_model` declaration: PR blocked by `oya-check-capacity-model-present`.
  - capacity_model with non-canonical units: PR blocked by `oya-check-capacity-model-units`.
  - scaling_dimension outside closed enum: PR blocked by `oya-check-capacity-model-scaling-dimension`.
  - cell_placement_class outside closed enum or Tier-0/Tier-1 without audit-chain seal reference: PR blocked by `oya-check-capacity-model-cell-placement`.
  - Dual-class µservice without demo_trial vs paid deltas: PR blocked by `oya-check-capacity-model-tenant-class-deltas`.
  - capacity_model diverging from FinOps cost-center by more than soft-band: PR blocked by `oya-check-capacity-model-finops-anchor`.
  - cell_placement_class conflicting with pod_runtime_tier per D-6 table: PR blocked by `oya-check-capacity-model-cellular-tier-coherence`.
  - Helm release admission for µservice missing capacity_model beyond grace window: refused by Kyverno policy.

H.5 **Quarterly capacity_model review** runs on rolling 90-day cadence per ADR-0341 candidate. Each review produces an evidence pack at `.omc/state/capacity-model-review-<date>.md` documenting declarations vs observed footprint.

H.6 **No waiver mechanism.** capacity_model declaration is required for every workload-producing µservice; there is no "skip capacity_model" path after sunset. Spec-only µservices that produce no workload MAY omit the block.

H.7 **Sunset of prior shape.** The "no canonical per-µservice capacity declaration" status-quo is retired at sunset. Per ADR-0108 sunset discipline, the retirement is recorded in `tools/hooks/_canonical-primitives.md` per the canonical-primitives cheat sheet pattern.

## I. Cross-references

I.1 **Memory anchors.**
  - `feedback_six_candidate_adrs_2026_05_21` — origin memory; ADR-0340 section is the canonical authority source.
  - `feedback_idea_refine_decisions_2026_05_21` — sibling /idea-refine triplet (ADR-0337/0338/0339) that this ADR anchors.
  - `feedback_amazon_shape_cellular_architecture` — ADR-0248 cellular tier numbering convention.
  - `feedback_quality_performance_scalability_bar` — hyperscaler-grade declared-capacity-model precedent.
  - `feedback_no_silent_regression` — substrate-shape change requires ADR + version bump + sunset.
  - `feedback_tenant_scoping_primitive` — capacity_model anchors per-tenant axes in concrete numbers.
  - `feedback_substrate_vs_product_layering` — substrate µservices declare per-tenant footprint authoritatively.
  - `feedback_clean_architecture_requirements` — separation of admission (pod_runtime_tier) from placement (cell_placement_class).
  - `feedback_microservice_ownership_coherence_2026_05_20` — per-µservice owner declares capacity_model alongside other manifest content.
  - `feedback_bominal_inheritance_precedence` — Bominal corpus inherits the same capacity_model pattern.
  - `feedback_docs_substance_not_scaffold_2026_05_20` — per-µservice capacity_model values are bespoke substance.
  - `feedback_drift_too_big_2026_05_20` — reverse-inference-from-telemetry is exactly the drift this ADR prevents.
  - `feedback_rust_strict_only_no_python_2026_05_20` — CI lane implementation is Rust-strict.
  - `feedback_zero_handroll_opentofu_only_2026_05_20` — OpenTofu nodepool primitives consume capacity_model inputs.
  - `feedback_oci_always_free_maximization_2026_05_20` — demo_trial tenant_class_deltas clamp to OCI Always Free ceiling.
  - `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20` — composable-billing-components size against capacity_model baseline.

I.2 **ADR anchors.**
  - ADR-0009 (cell architecture per tenant per region) — cell sizing consumes capacity_model.
  - ADR-0099 (data class registry) — Tier-0/Tier-1 cell_placement_class typically handles regulated data classes.
  - ADR-0108 (deprecation sunset discipline) — sunset schedule applies.
  - ADR-0145 (inter-µservice communication reform) — transport unchanged.
  - ADR-0152 (RPO/RTO per µservice) — capacity_model.storage_per_tenant sizes backup storage; ADR-0343 candidate consumes.
  - ADR-0174 (FinOps cost-attribution tag) — per-tenant cost floor consumed by FinOps portal.
  - ADR-0183 (policy-engine separation cedar app-authz kyverno admission) — Kyverno policy is the admission gate.
  - ADR-0197 (backup tier-driven RPO/RTO) — capacity_model.storage_per_tenant sizes backup.
  - ADR-0198 (Karpenter nodepool selection) — autoscaler input derivation per D-7.
  - ADR-0199 (FinOps cost-attribution namespace) — capacity_model + finops cost-center co-declared.
  - ADR-0211 (in-house tech stack preference) — all consumers are in-tree.
  - ADR-0212 (buildability doctrine) — amended; capacity_model block added to manifest.
  - ADR-0215 (multi-context platform) — capacity_model applies across all five deployment contexts.
  - ADR-0240 (sovereign-cloud per regional pack) — sovereign cells consume same capacity_model.
  - ADR-0242 (oyatie is a tenant doctrine) — oyatie's own tenant footprint declared.
  - ADR-0243 (Cedar as universal gate) — runtime authorization; capacity_model is manifest-time.
  - ADR-0244 (tenant as universal scoping primitive) — amended; capacity_model anchors per-tenant axes.
  - ADR-0245 (substrate vs product layering) — amended; substrate µservices declare per-tenant footprint authoritatively.
  - ADR-0247 (self-modification doctrine) — oyatie.foundry.* principals (now intelligence per ADR-0335) declare capacity_model.
  - ADR-0248 (Amazon-shape cellular architecture) — amended; cell_placement_class anchors per-µservice placement.
  - ADR-0251 (compliance pack cell certification levels) — capacity_model.compliance_pack_overrides supports per-pack stricter floors.
  - ADR-0263 (observability emission contract) — capacity_model_scaling_dimension + capacity_model_cell_placement_class labels additive.
  - ADR-0322 (substance bar as doctrine and CI enforcement) — bespoke per-µservice capacity_model substance required.
  - ADR-0324 (anti-script authoring doctrine) — template-stamped capacity_model values forbidden.
  - ADR-0328 (substance bar as canonical sequence and batch discipline) — per-µservice declaration sub-wave sequenced.
  - ADR-0329 (tier system retired; replaced by tenant_class) — capacity_model uses tenant_class deltas, not retired Bronze/Silver/Gold/Platinum.
  - ADR-0330 (tenant_class demo_trial vs paid composable billing components) — capacity_model.tenant_class_deltas anchors composable billing.
  - ADR-0331 (cross-µservice tenant_class adoption template) — amended; capacity_model is the canonical surface against which cap-shape contract is sized.
  - ADR-0333 (cell µservice retired; pattern not service) — shuffle-sharding consumes cell_placement_class.
  - ADR-0335 (foundry retired; absorbed by intelligence) — intelligence declares capacity_model for its absorbed responsibilities.
  - ADR-0336 (Valkey not Redis substrate) — connections_per_tenant.valkey references Valkey instance ceiling.
  - ADR-0337 (Iceberg canonical OLAP write path) — per_query scaling_dimension covers Iceberg+ClickHouse queries.
  - ADR-0338 (Pod runtime tier 0..3) — amended; co-variance table in D-6.
  - ADR-0339 (Shared IaC module library) — amended; nodepool primitives accept capacity_model inputs.

I.3 **Spec anchors.**
  - `/specs/master-plan-sequencing.json` — adds the `15U-Capacity-Model-declaration` sub-wave + queued ADR-0340 entry.
  - `/specs/microservices/manifest-schema.json` — admits the `capacity_model` block per D-1 schema fragment.
  - `/specs/microservices/cell.json` — per-cell capacity_saturation_ratio SLI emitted.
  - `/specs/platform-architecture.json` — capacity_model declaration recorded as a cross-cutting manifest surface.
  - `/specs/markdown-retirement-policy.json` — informational; this ADR does not retire any prior markdown.

I.4 **Companion-doc anchors.**
  - `docs/standards/hyperscaler-best-practices.md` — AWS Service Quotas / GCP Quotas / Azure Limits / Stripe Rate Limits / Salesforce Governor Limits precedent.
  - `docs/standards/dependency-policy.md` — informational; no new external dependencies introduced.
  - `docs/GLOSSARY.md` — capacity_model + cell_placement_class + scaling_dimension entries added.
  - `docs/machine-readable/glossary.json` — capacity_model machine-readable entries.
  - `tools/hooks/_canonical-primitives.md` — Capacity Model section added.

I.5 **Inbound citations.**
  - `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_six_candidate_adrs_2026_05_21.md` ADR-0340 section.

I.6 **Successor work (out of scope for this ADR).**
  - Per-µservice manifest capacity_model declaration sub-wave (`15U-Capacity-Model-declaration`).
  - Per-cell autoscaler config wiring (Karpenter NodePool + HPA + KEDA ScaledObject interpolations).
  - Per-cell FinOps portal dashboard updates consuming capacity_model.
  - Shuffle-sharding crate update consuming cell_placement_class.
  - Kyverno policy rollout per cell.
  - ADR-0341 (cellular promotion gates) consumes cell_placement_class.
  - ADR-0343 (DR matrix per µservice + per pack) consumes capacity_model.storage_per_tenant.
  - ADR-0344 (sustainability + finops dimensional model) consumes capacity_model.scaling_dimension.

---

<!--
adr: ADR-0340
status: Proposed
date: 2026-05-21
session: 2026-05-21 /idea-refine 6-candidate-ADR batch (ADR 1 of 6)
sibling_adrs: ADR-0341 (Cellular promotion gates), ADR-0342 (API versioning HYBRID), ADR-0343 (DR matrix per µservice + per pack), ADR-0344 (Sustainability + finops dimensional model), ADR-0345 (Talent + OSS contribution policy)
authority_source: feedback_six_candidate_adrs_2026_05_21
canonical_block: capacity_model
canonical_fields: baseline_cpu_per_tenant (vCPU decimal), baseline_ram_per_tenant (MiB integer), storage_per_tenant (GB integer), connections_per_tenant.{valkey,postgres,outbound_http} (integer), scaling_dimension (closed enum: per_user/per_request/per_capability/per_message/per_query/per_workflow_run), cell_placement_class (closed enum: Tier-0..Tier-4)
distinction_pod_runtime_tier_vs_cell_placement_class: pod_runtime_tier (ADR-0338) governs admission/isolation primitive (Kata vs runc vs runc-edge); cell_placement_class (this ADR) governs cellular placement (Foundation/Substrate/Capability/Application/Edge cells). Co-variance table in D-6.
optional_subobjects: tenant_class_deltas (demo_trial + paid), compliance_pack_overrides (hipaa/pci-dss/gdpr-strict/soc2/csap/eu-ai-act-annex-iii), notes
consumers: autoscaler (Karpenter NodePool + HPA + KEDA per ADR-0198); cell sizer (per ADR-0009 + ADR-0248); FinOps portal (per ADR-0174 + ADR-0199); shuffle-sharder (per ADR-0333)
new_lanes: 7 (oya-check-capacity-model-present, -units, -scaling-dimension, -cell-placement, -tenant-class-deltas, -finops-anchor, -cellular-tier-coherence) + 1 Kyverno policy (enforce-capacity-model-presence)
sunset_window: 30 days post-Acceptance for new authoring; per-µservice declaration follows ADR-0328 canonical-build phase order via 15U-Capacity-Model-declaration sub-wave
schema_fragment: /specs/microservices/manifest-schema.json gains capacity_model object property per D-1
sub_wave_added: 15U-Capacity-Model-declaration queued in /specs/master-plan-sequencing.json#realignment_wave_sequence.waves_15_plus.sub_waves
out_of_scope: per-µservice manifest declarations (sequenced as sub-wave); autoscaler config wiring (per-µservice Helm chart updates under sub-wave); FinOps portal dashboard updates (under sub-wave); shuffle-sharding crate update (under sub-wave); Kyverno rollout per cell (under sub-wave)
hyperscaler_precedents: AWS Service Quotas; Google Cloud Quotas; Azure Subscription Limits; Stripe Rate Limits; Salesforce Governor Limits
commits: none required at this ADR's landing beyond the schema fragment edit
-->
