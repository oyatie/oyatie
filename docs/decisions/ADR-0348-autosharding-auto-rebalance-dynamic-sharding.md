---
id: ADR-0348
title: Autosharding + auto-rebalance + dynamic sharding (cellular topology MUST support three control-plane-driven automation modes for tenant→cell/shard placement, hot-cell rebalancing, and within-cell hot-split + cold-merge shard count adjustment; manifest-declared per-µservice; cell-orchestrator (within tenancy + observability) executes; honors residency + compliance packs; emits audit-chain per ADR-0263; reversible)
status: Accepted
planning_impact: true
date: 2026-05-21
owner_team:
  - council-architecture
  - ops-sre-reliability
  - axis-tenancy
  - axis-observability
  - axis-cloud-iac
owners:
  - council-architecture
  - ops-sre-reliability
  - axis-tenancy
  - axis-observability
  - axis-cloud-iac
supersedes: []
superseded_by: []
amended_by: [ADR-0351]
amends:
  - ADR-0248-amazon-shape-cellular-architecture.md (the cellular topology baseline from ADR-0248 declared Tier 0..4 + shuffle sharding as the cell-shape primitive; this ADR fills the within-cell + across-cell TENANT-LEVEL automation contract that ADR-0248 deferred to a follow-on doctrine — specifically: tenant→cell/shard placement is now CONTROL-PLANE-DRIVEN by default rather than operator-driven; hot-cell rebalancing is now automatic by default rather than manual; shard count within a cell is now dynamic by default rather than static)
  - ADR-0341-cellular-promotion-gates-explicit-tier-criteria.md (ADR-0341 specified CELL-LEVEL promotion-gate automation — promotion of a cell between Tier 0..4 based on declared criteria; this ADR layers TENANT-LEVEL + SHARD-LEVEL automation underneath the cell-level promotion gates; auto-rebalance triggers when a cell's promotion criteria are breached due to load skew rather than capacity declaration drift; dynamic sharding executes within the cell that fails the load-skew threshold)
  - ADR-0333-cell-microservice-retired-pattern-not-service.md (ADR-0333 retired the cell µservice as a first-class deliverable; cellular shape is absorbed into tenancy=assignment + cloud-iac=provisioning+registry + observability=health/blast-radius + oya-shuffle-sharding crate=algorithm + api-gateway=routing + audit-chain=cell-scoped audit; THIS ADR clarifies that the "cell-orchestrator" responsibility for the three automation modes lives within tenancy + observability — NOT as a revived cell µservice; the orchestrator is a logical responsibility composed across the two owning µservices, not a new µservice)
  - ADR-0340-capacity-model-per-microservice-manifest.md (ADR-0340 declared the capacity_model per-µservice manifest block as the input to capacity-planning gates; this ADR consumes capacity_model as one of the inputs to the autosharding placement algorithm — the capacity-model {storage_per_tenant, cpu_per_tenant, memory_per_tenant, network_per_tenant} fields are read by the autosharding control plane to compute tenant→cell/shard placement)
related_adrs:
  - ADR-0110-changeset-state-machine.md
  - ADR-0111-merge-queue-projected-state-fix-at-any-stage.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0132-no-grouping-policy-and-flat-microservice-layout.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0158-multi-region-active-active.md
  - ADR-0181-cosign-signed-artifacts-and-modules.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0241-dr-business-continuity-portfolio-policy.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0250-build-ahead-of-certification.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0252-hlc-default-truetime-tier.md
  - ADR-0253-http3-quic-default-protocol.md
  - ADR-0254-kubernetes-everywhere-pods-cloud-hypervisor.md
  - ADR-0263-observability-emission-contract.md
  - ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md
  - ADR-0324-anti-script-authoring-doctrine.md
  - ADR-0327-realignment-wave-promotion-gate.md
  - ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md
  - ADR-0333-cell-microservice-retired-pattern-not-service.md
  - ADR-0335-intelligence-microservice-consolidation.md
  - ADR-0338-pod-runtime-tier-declaration.md
  - ADR-0340-capacity-model-per-microservice-manifest.md
  - ADR-0341-cellular-promotion-gates-explicit-tier-criteria.md
  - ADR-0342-api-versioning-hybrid-date-public-semver-sdk.md
  - ADR-0343-dr-rto-rpo-matrix-per-microservice-per-compliance-pack.md
  - ADR-0344-sustainability-finops-dimensional-model.md
  - ADR-0345-oss-stewardship-class-policy-and-cve-response-sla.md
  - ADR-0346-product-readiness-checklist.md
  - ADR-0347-governance-fitness-bulk-rename.md
related_specs:
  - /specs/master-plan-sequencing.json
  - /specs/microservices/manifest-schema.json
  - /specs/microservices/cell.json
  - /specs/root-hub-pointers.json
related_memory:
  - feedback_autosharding_dynamic_rebalance_2026_05_21
  - feedback_amazon_shape_cellular_architecture
  - feedback_quality_performance_scalability_bar
  - feedback_no_silent_regression
  - feedback_clean_architecture_requirements
  - feedback_microservice_ownership_coherence_2026_05_20
  - feedback_verify_deliverables_not_just_line_count_2026_05_20
  - feedback_automate_everything
  - feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20
  - feedback_canonical_base_localization
  - feedback_drift_too_big_2026_05_20
companion_docs:
  - tools/hooks/_canonical-primitives.md
  - docs/standards/dependency-policy.md
  - microservices/tenancy/PRD.md
  - microservices/observability/PRD.md
inbound_citations:
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0248-amazon-shape-cellular-architecture.md
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0341-cellular-promotion-gates-explicit-tier-criteria.md
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0333-cell-microservice-retired-pattern-not-service.md
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0340-capacity-model-per-microservice-manifest.md
doc_class: Architecture-Decision-Record
shape: Decision
authority_tier: 1
line_floor: 600
bespoke_authoring_requirement: documentation-rigor-1.1-plus-ADR-0322
enforcement_status: advisory-until-wave-15-zd-doctrine-pr-lands
enforced_by:
  - oya-governance-sharding-automation-coverage (new lane; refuses any µservice manifest.json that lacks a complete `sharding_automation` block with autosharding + auto_rebalance + dynamic_sharding sub-blocks declared per the D-1 schema; allowlist for µservices on the EXEMPT_FROM_CELLULAR list at .omc/state/cellular-exemption-allowlist-2026-05-21.json — e.g., static-only edge surfaces, no-tenant-state µservices)
  - oya-governance-autosharding-manual-mode-refusal (new lane; refuses any manifest.json that declares the sharding_automation.autosharding field set to the value manual; the canonical autosharding mode is control_plane_driven; a manual-mode exception requires an ADR-amendment to this ADR enumerating the surface justifying the exception)
  - oya-governance-auto-rebalance-residency-honored (new lane; greps every manifest declaring sharding_automation.auto_rebalance.enabled true and refuses if the same manifest also declares honors_residency false OR omits the field; cross-jurisdiction rebalance without Cedar permit is refused at admission time per ADR-0243)
  - oya-governance-dynamic-sharding-threshold-coverage (new lane; refuses any manifest declaring sharding_automation.dynamic_sharding.enabled true that omits ANY of the four canonical thresholds (hot_split_threshold_p99_ms, hot_split_utilization_threshold_percent, cold_merge_utilization_threshold_percent, cold_merge_minimum_quiet_hours); default-fill is REJECTED to force per-µservice declaration of load characteristics)
  - oya-governance-audit-chain-emit-on-automation-events (new lane; greps every manifest declaring auto_rebalance.enabled true OR dynamic_sharding.enabled true and refuses if the same manifest omits audit_chain_emit true on the corresponding sub-block; every automation event MUST emit per ADR-0263 observability-emission-contract)
  - oya-governance-tenant-migration-reversibility (new lane; refuses any µservice IP authoring under {oya,cloud}/<service>/ (legacy microservices/ removal-candidate) IPs/IP-*-auto-rebalance-*.md that lacks an explicit `rollback_path` section enumerating how an automation-event-driven tenant migration is reversed via the audit-chain trail)
purpose: >
  Declare that cellular topology MUST support three control-plane-driven
  automation modes underneath the cell-level promotion gates already
  doctrined in ADR-0341: (1) AUTOSHARDING — tenant→cell/shard placement is
  computed by the control plane automatically, with no human operator
  picking placement; inputs are capacity_model (ADR-0340) + compliance_pack
  constraints (ADR-0251) + ResidencyClass + cell_placement_class (Tier 0..4
  per ADR-0248) + the shuffle-sharding algorithm in the oya-shuffle-sharding
  crate (ADR-0333). (2) AUTO-REBALANCE — when cell load skews beyond
  promotion-gate criteria, the cell-orchestrator (a logical responsibility
  composed across tenancy + observability per ADR-0333) automatically
  migrates tenants from hot cells to cooler cells; tenant migration honors
  residency + compliance pack constraints; cross-jurisdiction migration
  requires an explicit Cedar permit per ADR-0243; migration is observable,
  reversible, and audit-chain-emit per ADR-0263. (3) DYNAMIC SHARDING —
  shard count within a cell adjusts based on load: HOT-SPLIT when shard p99
  latency exceeds SLO OR capacity utilization exceeds 80% (defaults; per-
  µservice override), COLD-MERGE when adjacent shards both run below 20%
  utilization for more than 24 hours (defaults; per-µservice override);
  both operations are atomic + audit-emit. Every µservice manifest.json
  gains a `sharding_automation` block declaring per-automation-mode
  configuration. Six new CI lanes enforce manifest coverage, autosharding-
  mode discipline, residency-honoring, threshold completeness, audit-chain
  emission, and IP-level reversibility documentation. The actual cell-
  orchestrator implementation in tenancy + observability is queued as Wave
  15-ZD (separate executor PR sequenced under ADR-0328 batch discipline);
  this ADR is DOCTRINE-ONLY. Sunset: 30 days post-Wave-15-ZD completion
  the new lanes promote to BLOCKER. Out of scope: actual cell-orchestrator
  Rust crate implementation (deferred to Wave 15-ZD); pipeline-level
  rebalance for batch/streaming workloads (separate ADR if needed);
  cross-pack tenant migration with PHI (refused by E.3 + the compliance-
  pack constraints per ADR-0251); Bominal sibling ADR (Bominal authors
  independently per feedback_bominal_inheritance_precedence).
---

> **Disposition light-edit (2026-08-06):** Context re-triage Accept: Autosharding / cellular topology — hyperscaler

# ADR-0348: Autosharding + auto-rebalance + dynamic sharding (cellular topology MUST support three control-plane-driven automation modes for tenant→cell/shard placement, hot-cell rebalancing, and within-cell hot-split + cold-merge shard count adjustment; manifest-declared per-µservice; cell-orchestrator (within tenancy + observability) executes; honors residency + compliance packs; emits audit-chain per ADR-0263; reversible)

## Status

Proposed on 2026-05-21. **Amendment 2026-05-21 per ADR-0351**: the rebalancing workflow lives in the dedicated `cell-rebalancer` µservice (NOT "composed across tenancy + observability" as originally stated in the rationale). The `sharding_automation` manifest field declarations apply to `cell-rebalancer` for the rebalance modes (auto_rebalance + dynamic_sharding hot-split + cold-merge). See ADR-0351 D-2 for the cell-rebalancer bounded context and D-3 for the related cell-lifecycle µservice.

This ADR is the canonical doctrine decision binding cellular topology to three control-plane-driven automation modes — autosharding, auto-rebalance, dynamic sharding — that layer underneath the cell-level promotion gates already doctrined in ADR-0341. Without these three modes, every load skew or growth event would require manual operator intervention; hyperscaler-grade horizontal scalability per `feedback_quality_performance_scalability_bar` requires control-plane-driven automation across the tenant + shard axis as well as the cell axis.

It runs in coordination with the 2026-05-21 realignment-wave authoring session: ADR-0340 (capacity model per-µservice manifest), ADR-0341 (cellular promotion gates explicit per-tier), ADR-0342 (API versioning hybrid date + semver), ADR-0343 (DR + RTO/RPO matrix per microservice per compliance pack), ADR-0344 (sustainability + finops dimensional model), ADR-0345 (OSS stewardship class policy + CVE-response SLA), ADR-0346 (product readiness checklist), ADR-0347 (foundry-fitness to governance bulk rename), and this ADR are sibling decisions from the same authoring session. This ADR closes the within-cell + across-cell TENANT-LEVEL automation backlog that ADR-0248 cellular topology baseline and ADR-0341 cellular promotion gates deferred to a follow-on doctrine.

It directly amends ADR-0248 (cellular topology baseline) by declaring that tenant→cell/shard placement is control-plane-driven by default rather than operator-driven; hot-cell rebalancing is automatic by default; shard count within a cell is dynamic by default. It directly amends ADR-0341 (cellular promotion gates) by layering tenant-level + shard-level automation underneath the cell-level promotion gates; auto-rebalance triggers when a cell's promotion criteria are breached due to load skew rather than capacity declaration drift. It directly amends ADR-0333 (cell µservice retired — pattern not service) by clarifying that the cell-orchestrator responsibility lives within tenancy + observability — NOT as a revived cell µservice. **(Superseded by ADR-0351 per the line-145 amendment: cross-cell rebalancing is owned by the cell-rebalancer µservice, cell lifecycle by cell-lifecycle, and within-cell sharding by the oya-shuffle-sharding crate — NOT "within tenancy + observability". The "tenancy + observability" wording below is the original 2026-05-21 rationale, retained as historical record.)** It directly amends ADR-0340 (capacity_model per-µservice manifest) by consuming the capacity_model block as one of the inputs to the autosharding placement algorithm.

Enforcement transitions from `advisory-until-wave-15-zd-doctrine-pr-lands` to `BLOCKER` per the lane sequence in §E below: at landing of this ADR's doctrine PR, the six new lanes (`oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, `oya-governance-tenant-migration-reversibility`) promote from REPORT-ONLY to BLOCKER 30 days post-Wave-15-ZD completion for new authoring. Wave 15-ZD is the actual cell-orchestrator implementation sub-wave; the doctrine PR (this ADR + manifest-schema amendment + lane scaffolds) lands first.

The decision does not author the actual cell-orchestrator Rust crate implementation; the doctrine PR + manifest-schema admission + lane scaffolds land first, then Wave 15-ZD implements. The decision does not change the existing oya-shuffle-sharding crate (ADR-0333); the crate continues to provide the shuffle-sharding algorithm consumed by the autosharding control plane. The decision does not retire any prior ADR. The decision does not introduce a new µservice; the cell-orchestrator is a logical responsibility composed across tenancy + observability per ADR-0333 absorption (ownership later moved to the cell-rebalancer + cell-lifecycle µservices per ADR-0351 — see the line-145 amendment; this clause is the original rationale, retained as historical record). The decision does not relax compliance pack constraints (ADR-0251) or residency invariants; cross-jurisdiction migration continues to require a Cedar permit per ADR-0243.

## Date

2026-05-21.

## Context

### A.1 Named pressure: hyperscaler horizontal scalability requires within-cell + across-cell automation

The hyperscaler-grade bar declared by `feedback_quality_performance_scalability_bar` requires horizontal scalability without manual operator intervention. Existing ADR-0248 cellular topology baseline + ADR-0341 cellular promotion gates handle CELL-LEVEL automation: cells are provisioned per Tier 0..4 declarations; cells promote across tiers per declared promotion criteria; cell shuffle-sharding is provided by the oya-shuffle-sharding crate per ADR-0333. None of these mechanisms automate the TENANT-LEVEL placement decision (which cell + which shard a tenant lives on) or the SHARD-LEVEL load distribution decision (how many shards a cell carries, and when to split or merge them).

Without tenant-level autosharding, every new tenant onboarding requires an operator to pick a target cell + shard. At Oyatie's projected tenant density (~10,000+ paid tenants across ~77 µservices × ~10-100 cells per µservice per region × ~3-5 regions per pack), operator-driven placement is structurally impossible. The hyperscaler precedent (AWS account placement, Google project placement, Azure subscription placement) is uniformly control-plane-driven; the Oyatie equivalent must be control-plane-driven by default.

Without across-cell auto-rebalance, every load skew (e.g., a single tenant onboarding generates 10x the projected load due to a viral product launch; a cell's hardware degrades partially; a region capacity expansion adds new cells with zero tenants) requires an operator to manually identify the skew, pick which tenants to migrate, schedule the migration, and execute it. Hyperscaler precedent (Amazon's S3 SHIELD migrator, Google's Spanner re-sharder, Azure's Cosmos partition mover) is uniformly automatic — operators set thresholds; the control plane executes.

Without within-cell dynamic sharding, every shard saturation (e.g., a cell's primary shard p99 latency drifts past SLO due to data growth; a cell's quiet shards stack up after tenants migrate away) requires an operator to manually plan + execute shard splits / merges. Hyperscaler precedent (Spanner automatic re-sharding, DynamoDB automatic adaptive capacity, Cosmos automatic re-partitioning) is uniformly automatic.

The Oyatie cellular topology MUST match the hyperscaler precedent on all three axes: tenant-placement automation (autosharding), cross-cell rebalance automation (auto-rebalance), within-cell shard count automation (dynamic sharding).

### A.2 Named pressure: existing per-µservice declarations are incomplete

Existing µservice manifests at `microservices/<name>/manifest.json` already carry partial cellular declarations per ADR-0248 + ADR-0341:
- `cell_placement_class` (Tier 0..4) declares the cell tier the µservice lives on.
- `cellular` block declares cell-level promotion criteria (per ADR-0341).
- `capacity_model` block (per ADR-0340) declares per-tenant resource consumption.

But no manifest field declares the tenant-placement automation mode, the auto-rebalance configuration, or the dynamic sharding thresholds. The control plane therefore has no canonical configuration source for these three modes. Each operator team currently defaults to operator-driven placement out of necessity; the absence of the manifest field is itself the constraint forcing manual ops.

This ADR adds a `sharding_automation` manifest block declaring per-automation-mode configuration; the manifest schema admits the block; the six new CI lanes enforce coverage + correctness.

### A.3 Named pressure: residency + compliance packs constrain auto-rebalance migration

Tenant migration during auto-rebalance is not unconstrained. ADR-0244 (tenant scoping primitive) declares tenant context is universal; every row, audit, cost carries tenant context. ADR-0251 (compliance-pack-cell-certification-levels) declares that compliance packs (HIPAA / GDPR / SOC2 / CSAP / PCI / EU-AI-Act) constrain which cells a tenant can live on. ADR-0240 (sovereign-cloud-per-regional-pack) declares that sovereign cells (e.g., CSAP KR cells; EU GDPR-strict cells) are residency-scoped — tenants in a sovereign pack cannot migrate to a non-sovereign cell. ADR-0243 (Cedar as universal gate) declares that every gate is a Cedar eval.

Auto-rebalance therefore must honor residency + compliance packs as hard constraints. The control plane consumes the tenant's ResidencyClass + active compliance packs as filters on the candidate-target-cell set; if no candidate exists, auto-rebalance refuses (operator escalation; not a silent failure). Cross-jurisdiction migration requires an explicit Cedar permit per ADR-0243 (e.g., a tenant migrates from a US-East cell to a US-West cell within the same residency domain — permitted; from a US cell to an EU cell — refused absent a tenant-issued cross-jurisdiction permit). PHI cross-pack migration is refused absent a BYOK + ADR-0251 §D-10 cross-pack permission.

The six new CI lanes include `oya-governance-auto-rebalance-residency-honored` which refuses any manifest declaring `auto_rebalance.enabled: true` with `honors_residency: false`.

### A.4 Named pressure: cell-orchestrator responsibility composition per ADR-0333

ADR-0333 retired the cell µservice as a first-class deliverable. Cellular shape is a PATTERN — not a service — absorbed into:
- **tenancy**: tenant→cell/shard assignment registry; the source of truth for which tenant lives on which cell + which shard.
- **cloud-iac**: cell provisioning + registry (OpenTofu modules; cell catalog).
- **observability**: cell health, load, blast-radius telemetry; the source of truth for load-skew detection.
- **oya-shuffle-sharding crate**: shuffle-sharding algorithm.
- **api-gateway**: cell-aware routing; tenant→cell lookup at request time.
- **audit-chain**: cell-scoped audit emission per ADR-0263.

The "cell-orchestrator" responsibility for executing the three automation modes therefore is NOT a revived cell µservice. It is a LOGICAL RESPONSIBILITY composed across tenancy (placement registry) + observability (load-skew detection) + cloud-iac (cell provisioning if new cells are needed) + oya-shuffle-sharding (algorithm) + audit-chain (emission). This ADR's manifest declarations are read by the tenancy + observability µservices at the orchestrator-implementation layer; the orchestration logic itself is implemented as a coordinator subsystem inside tenancy + observability, not as a new µservice.

This composition is consistent with `feedback_microservice_ownership_coherence_2026_05_20` (one team owns one µservice end-to-end) by assigning the orchestrator implementation to the joint tenancy + observability owning teams via the cross-µservice coordinator pattern established for similar substrate-level responsibilities in ADR-0145 (inter-microservice communication reform) and ADR-0263 (observability emission contract).

### A.5 Named pressure: dynamic sharding thresholds must be per-µservice declarable

Hot-split + cold-merge thresholds are not universal. A workflow-engine µservice with bursty CPU-bound workloads has a different sensible hot-split threshold than a object-storage µservice with sustained IOPS load. A messenger µservice with E2EE-encrypted writes has a different cold-merge minimum quiet hours threshold than a marketplace µservice with infrequent listing-update writes.

This ADR therefore declares DEFAULT thresholds (hot-split: p99 > 50ms OR utilization > 80%; cold-merge: utilization < 20% for > 24h) but REQUIRES each µservice to explicitly declare them in `sharding_automation.dynamic_sharding`. The `oya-governance-dynamic-sharding-threshold-coverage` lane refuses any manifest declaring `dynamic_sharding.enabled: true` that omits any of the four threshold fields; default-fill is REJECTED to force per-µservice declaration of load characteristics. The defaults exist as starting points for new µservice authoring; they must be reviewed + signed off per-µservice before manifest acceptance.

### A.6 Named pressure: every automation event must be observable, reversible, audit-emit

Tenant migrations + shard splits + shard merges are state-mutating cellular operations. Without observability, operators cannot diagnose why a tenant ended up on a different cell after auto-rebalance. Without reversibility, a botched migration cannot be undone without operator manual intervention contradicting the automation goal. Without audit-chain emission per ADR-0263, compliance auditors cannot verify that the migration honored residency + compliance pack constraints.

This ADR declares all three properties as MANDATORY for every automation event:
- **Observability**: every event emits a structured event to observability per ADR-0263.
- **Reversibility**: every event records pre-state + post-state + transition rationale in the audit chain; the inverse operation is enumerable from the audit-chain row.
- **Audit-chain emission**: every event emits per ADR-0263 observability emission contract; emission includes tenant_id + cell_source + cell_target (for auto-rebalance) OR shard_source + shard_targets (for hot-split) OR shard_sources + shard_target (for cold-merge) + residency_check_result + compliance_pack_check_result + cedar_permit_id (if cross-jurisdiction).

The `oya-governance-audit-chain-emit-on-automation-events` lane refuses any manifest declaring auto_rebalance.enabled OR dynamic_sharding.enabled that omits `audit_chain_emit: true` on the corresponding sub-block. The `oya-governance-tenant-migration-reversibility` lane refuses IP authoring without an explicit `rollback_path` section.

### A.7 Named pressure: doctrine before implementation per ADR-0328 batch discipline

ADR-0328 (substance-bar-as-canonical-sequence-and-batch-discipline) declares that doctrine ADRs land before implementation sub-waves. The cell-orchestrator implementation in tenancy + observability is a non-trivial multi-µservice change (thousands of lines of Rust; new orchestrator subsystems in two µservices; new manifest field admission in `specs/microservices/manifest-schema.json`; new CI lanes). Implementing without a doctrine ADR would create scattered surface-by-surface decisions that drift over time per `feedback_drift_too_big_2026_05_20`.

This ADR is therefore DOCTRINE-ONLY. It declares the canonical shape; it admits the manifest field; it scaffolds the six new CI lanes as REPORT-ONLY. The implementation sub-wave (Wave 15-ZD) is sequenced as a follow-on under ADR-0328 batch discipline. At Wave 15-ZD completion, the lanes promote to BLOCKER per the 30-day sunset window.

### A.8 Named pressure: counterpart precedent — AWS Cell SHIELD, Google Spanner, Azure Cosmos automation

Counterpart-precedent calibration (M1 facet per multispectrum-review v2.4.0):

- **AWS Cell SHIELD migrator** (S3 cross-cell tenant migration). AWS S3's cellular topology has an automated cross-cell migrator that re-balances tenants across cells based on load + capacity + residency constraints. Operator declares thresholds; control plane executes. The Oyatie auto-rebalance shape matches.
- **Google Spanner automatic re-sharding** (within-database shard count adjustment). Spanner's split + merge logic is fully automatic; thresholds are declared per-database; control plane executes hot-split + cold-merge atomically with multi-region paxos + audit emit. The Oyatie dynamic sharding shape matches.
- **DynamoDB Adaptive Capacity** (per-partition capacity rebalance). DynamoDB's adaptive capacity automatically moves hot-partition capacity to under-utilized partitions without operator intervention. The Oyatie dynamic-sharding hot-split shape parallels.
- **Azure Cosmos automatic re-partitioning** (per-container partition count adjustment). Cosmos automatically splits + merges physical partitions based on logical partition load + storage growth; thresholds are declared per-container. The Oyatie dynamic-sharding shape matches.
- **Snowflake automatic clustering** (per-table clustering key re-organization). Snowflake automatically re-clusters tables based on declared clustering keys; operator sets the key; control plane executes. The Oyatie autosharding shape parallels (tenant→cell placement is a clustering decision).

The Oyatie three-mode shape is uniformly hyperscaler-typical. The doctrine bar is set.

### A.9 Anchors this ADR binds

- Anchor 1: `feedback_autosharding_dynamic_rebalance_2026_05_21` — the source memory file declaring the three-mode doctrine.
- Anchor 2: ADR-0248 (cellular topology baseline) — Tier 0..4 + shuffle sharding as the cell-shape primitive; THIS ADR fills the within-cell + across-cell TENANT-LEVEL automation contract.
- Anchor 3: ADR-0341 (cellular promotion gates explicit per-tier) — CELL-LEVEL automation; THIS ADR layers TENANT + SHARD-level automation underneath.
- Anchor 4: ADR-0333 (cell µservice retired — pattern not service) — cell-orchestrator is a logical responsibility composed across tenancy + observability; NOT a new µservice.
- Anchor 5: ADR-0340 (capacity_model per-µservice manifest) — capacity_model is one of the inputs to autosharding placement algorithm.
- Anchor 6: ADR-0251 (compliance-pack-cell-certification-levels) — compliance packs constrain auto-rebalance migration candidate cells.
- Anchor 7: ADR-0244 (tenant-as-universal-scoping-primitive) — tenant context is universal; every automation event carries tenant_id.
- Anchor 8: ADR-0243 (Cedar as universal gate) — cross-jurisdiction migration requires an explicit Cedar permit.
- Anchor 9: ADR-0240 (sovereign-cloud-per-regional-pack) — sovereign cells are residency-scoped; auto-rebalance refuses if no candidate cell satisfies residency.
- Anchor 10: ADR-0263 (observability emission contract) — every automation event emits per the canonical observability contract.
- Anchor 11: `feedback_quality_performance_scalability_bar` — hyperscaler-grade horizontal scalability without manual operator intervention.
- Anchor 12: `feedback_amazon_shape_cellular_architecture` — AWS cell topology Tiers 0..4; shuffle sharding; Cloud Hypervisor.
- Anchor 13: `feedback_microservice_ownership_coherence_2026_05_20` — cell-orchestrator implementation joint-owned by tenancy + observability teams via the cross-µservice coordinator pattern.
- Anchor 14: `feedback_automate_everything` — control-plane-driven automation; no operator-driven placement / rebalance / shard adjustment.
- Anchor 15: `feedback_no_silent_regression` — every automation event is observable + reversible + audit-emit; no silent state mutation.
- Anchor 16: ADR-0322 (substance bar) — doctrine ADR substance is the per-mode mechanics + per-lane enforcement, not template-stamped prose.
- Anchor 17: ADR-0324 (anti-script authoring) — manifest field admission is mechanical; the doctrine ADR is bespoke.
- Anchor 18: ADR-0328 (substance bar canonical sequence + batch discipline) — Wave 15-ZD sequenced after doctrine ADR Acceptance.
- Anchor 19: ADR-0247 (self-modification doctrine) — Foundry runs as oyatie.foundry.* principals under Cedar; auto-rebalance principals declared.
- Anchor 20: `feedback_canonical_base_localization` — canonical-base autosharding behavior + per-pack overlays for compliance-pack-specific candidate-cell filtering.

### A.10 What this ADR does not assert

- **A.10.1** Does not author the cell-orchestrator Rust crate or the tenancy + observability orchestrator subsystem implementations. All implementation is sequenced as Wave 15-ZD under ADR-0328 batch discipline.
- **A.10.2** Does not retire any prior ADR. ADR-0248 + ADR-0341 + ADR-0333 + ADR-0340 are amended (declared above), not retired.
- **A.10.3** Does not introduce a new µservice. The cell-orchestrator responsibility is composed across existing tenancy + observability + cloud-iac + audit-chain µservices per ADR-0333.
- **A.10.4** Does not relax compliance pack constraints (ADR-0251). Auto-rebalance honors compliance packs; PHI cross-pack migration requires ADR-0251 §D-10 BYOK + explicit permit.
- **A.10.5** Does not relax residency invariants. Sovereign cells (per ADR-0240) cannot have tenants auto-rebalanced out of the sovereign domain absent an explicit Cedar permit per ADR-0243.
- **A.10.6** Does not declare an exception clause for any µservice that wants to opt out of autosharding. The canonical autosharding mode is `"control_plane_driven"`. Manual mode is refused by E.2. An ADR amendment to this ADR is required for any exception, and the exception must enumerate the surface justification.
- **A.10.7** Does not change the oya-shuffle-sharding crate (ADR-0333). The crate continues to provide the shuffle-sharding algorithm consumed by the autosharding control plane.
- **A.10.8** Does not change the cell_placement_class field declared in ADR-0248. The Tier 0..4 declaration remains a separate manifest field from `sharding_automation`.
- **A.10.9** Does not change capacity_model schema (ADR-0340). capacity_model is READ by the autosharding control plane; the schema is unchanged.
- **A.10.10** Does not change cellular promotion gates (ADR-0341). Cell-level promotion gates remain a separate manifest block from `sharding_automation`; auto-rebalance triggers when a cell breaches promotion-gate criteria due to load skew, but the gates themselves are unchanged.
- **A.10.11** Does not introduce pipeline-level rebalance for batch/streaming workloads. That is a separate question explicitly OUT OF SCOPE; a follow-on ADR is authored if pipeline-rebalance automation is required.
- **A.10.12** Does not change the Bominal parallel corpus. Bominal authors its sibling ADR independently per `feedback_bominal_inheritance_precedence`.

## Decision

### B.1 Decision statement

Cellular topology MUST support three control-plane-driven automation modes underneath the cell-level promotion gates already doctrined in ADR-0341:

1. **Autosharding** — tenant→cell/shard placement is computed automatically by the control plane; no human operator picks placement; inputs are capacity_model (ADR-0340) + compliance_pack constraints (ADR-0251) + ResidencyClass + cell_placement_class (ADR-0248 Tier 0..4) + the shuffle-sharding algorithm in the oya-shuffle-sharding crate (ADR-0333). The canonical autosharding mode is `"control_plane_driven"`; the `"manual"` mode is refused by E.2.

2. **Auto-rebalance** — when cell load skews beyond promotion-gate criteria (ADR-0341), the cell-orchestrator (logical responsibility composed across tenancy + observability per ADR-0333) automatically migrates tenants from hot cells to cooler cells. Tenant migration honors residency + compliance pack constraints; cross-jurisdiction migration requires an explicit Cedar permit per ADR-0243; migration is observable, reversible, and audit-chain-emit per ADR-0263.

3. **Dynamic sharding** — shard count within a cell adjusts based on load:
   - **Hot-split**: when shard p99 latency exceeds SLO OR capacity utilization exceeds 80% (defaults; per-µservice override required), control plane splits the shard into 2 sub-shards + redistributes hash-ring assignments. Atomic + audit-emit.
   - **Cold-merge**: when adjacent shards both run below 20% utilization for more than 24 hours (defaults; per-µservice override required), control plane merges them into 1 shard + redistributes assignments. Atomic + audit-emit.

Every µservice manifest.json gains a `sharding_automation` block declaring per-automation-mode configuration. Six new CI lanes enforce manifest coverage, autosharding-mode discipline, residency-honoring, threshold completeness, audit-chain emission, and IP-level reversibility documentation. The actual cell-orchestrator implementation in tenancy + observability is queued as Wave 15-ZD (separate executor PR sequenced under ADR-0328 batch discipline); this ADR is DOCTRINE-ONLY.

### B.2 Numbered decision clauses

B2.001. The per-µservice manifest.json gains a `sharding_automation` block declared in D-1 below. Every µservice manifest MUST carry the block, with the exception of µservices on the EXEMPT_FROM_CELLULAR allowlist at `.omc/state/cellular-exemption-allowlist-2026-05-21.json` (e.g., static-only edge surfaces, no-tenant-state µservices).

B2.002. The canonical autosharding mode is `"control_plane_driven"`. The `"manual"` mode is refused by E.2 (`oya-governance-autosharding-manual-mode-refusal`). An ADR amendment to this ADR is required for any exception, and the exception must enumerate the surface justification.

B2.003. The autosharding control plane consumes the following inputs in computing tenant→cell/shard placement:
- (a) capacity_model per ADR-0340 (storage_per_tenant + cpu_per_tenant + memory_per_tenant + network_per_tenant);
- (b) compliance_pack constraints per ADR-0251 (allowed cells for the tenant's active packs);
- (c) ResidencyClass per ADR-0240 (sovereign-domain filter);
- (d) cell_placement_class per ADR-0248 (Tier 0..4 filter; tenant goes to a cell of matching tier);
- (e) shuffle-sharding algorithm per ADR-0333 (oya-shuffle-sharding crate; canonical shuffle-shard primitive).

B2.004. Auto-rebalance is triggered by load-skew detection from observability per ADR-0263. The default trigger threshold is `load_skew_threshold_percent: 30` (i.e., when a cell's load deviates from the per-µservice mean by more than 30%, the orchestrator considers rebalance). The threshold is per-µservice declarable in the `sharding_automation.auto_rebalance.trigger_load_skew_threshold_percent` manifest field.

B2.005. Auto-rebalance candidate-target-cell selection filters by (a) residency (ResidencyClass match required); (b) compliance packs (active packs match required); (c) cell_placement_class (Tier 0..4 match required); (d) capacity headroom (target cell has > 30% headroom after migration); (e) Cedar permit (cross-jurisdiction migration requires explicit permit per ADR-0243).

B2.006. Auto-rebalance migration is atomic per tenant: the migration transitions tenant_id from cell_source to cell_target in a single ACID transaction in the tenancy assignment registry; routing through api-gateway switches to the target cell at the same transaction boundary; audit-chain emission per ADR-0263 records the transition with cell_source + cell_target + residency_check_result + compliance_pack_check_result + cedar_permit_id (if applicable).

B2.007. Auto-rebalance migration is reversible: the audit-chain row enables the inverse operation; rollback re-applies the inverse transition with a new audit-chain row marking it as a rollback of the prior automation event.

B2.008. Dynamic sharding hot-split threshold defaults are `hot_split_threshold_p99_ms: 50` and `hot_split_utilization_threshold_percent: 80`. Both thresholds are per-µservice declarable; default-fill is REJECTED by E.4 (`oya-governance-dynamic-sharding-threshold-coverage`) to force explicit per-µservice declaration.

B2.009. Dynamic sharding cold-merge threshold defaults are `cold_merge_utilization_threshold_percent: 20` and `cold_merge_minimum_quiet_hours: 24`. Both thresholds are per-µservice declarable; default-fill is REJECTED by E.4.

B2.010. Hot-split execution: control plane splits the shard into 2 sub-shards using the oya-shuffle-sharding crate's split primitive; hash-ring assignments are atomically redistributed; tenants on the split shard are re-mapped to one of the 2 sub-shards based on shuffle-sharding hash; audit-chain emit per ADR-0263.

B2.011. Cold-merge execution: control plane verifies both adjacent shards have run below the merge threshold for the minimum quiet hours; merges them into 1 shard using the oya-shuffle-sharding crate's merge primitive; hash-ring assignments are atomically redistributed; audit-chain emit per ADR-0263.

B2.012. Both hot-split + cold-merge are atomic at the cell level: the cell's shard count transitions in a single ACID transaction in the cell registry (cloud-iac shard catalog); routing through api-gateway switches to the new shard topology at the same transaction boundary.

B2.013. Both hot-split + cold-merge are reversible: hot-split → cold-merge of the two sub-shards; cold-merge → hot-split of the merged shard. The inverse operation is enumerable from the audit-chain row.

B2.014. Every automation event (auto-rebalance migration, hot-split, cold-merge) MUST emit an audit-chain row per ADR-0263. The emission includes event_type + cell_id + shard_id (if shard-level) + tenant_id (if tenant-level) + pre_state + post_state + residency_check_result + compliance_pack_check_result + cedar_permit_id (if applicable) + initiated_by (always "control_plane:cell-orchestrator").

B2.015. The cell-orchestrator is a logical responsibility composed across tenancy + observability + cloud-iac + audit-chain per ADR-0333. It is NOT a new µservice. Implementation is via the cross-µservice coordinator pattern; the orchestrator subsystem lives inside tenancy (placement registry) + observability (load-skew detection) jointly, calling cloud-iac (provisioning if new cells needed) + audit-chain (emission).

B2.016. The `sharding_automation` manifest block schema is admitted to `specs/microservices/manifest-schema.json` in the doctrine PR landing this ADR. The schema declares the three sub-blocks (autosharding, auto_rebalance, dynamic_sharding) and their fields per D-1.

B2.017. The six new CI lanes (E.1..E.6) start REPORT-ONLY at this ADR's Acceptance.

B2.018. The six new CI lanes promote from REPORT-ONLY to BLOCKER 30 days post-Wave-15-ZD completion for new authoring.

B2.019. Five Rejected Alternatives are recorded in §F below: (i) operator-driven placement (status quo); (ii) auto-rebalance via consistent hashing without shuffle sharding; (iii) static shard count per cell (no dynamic sharding); (iv) cross-jurisdiction migration without Cedar permit; (v) silent (non-audit-chain) automation events.

B2.020. The Bominal parallel corpus authors its sibling autosharding doctrine ADR independently per `feedback_bominal_inheritance_precedence`. No Oyatie-side enforcement applies to Bominal.

B2.021. The cell_placement_class field declared in ADR-0248 is preserved verbatim. The Tier 0..4 declaration remains a separate manifest field from `sharding_automation`. The autosharding control plane reads cell_placement_class as one of its inputs (B2.003.d).

B2.022. capacity_model schema (ADR-0340) is preserved verbatim. The autosharding control plane reads capacity_model as one of its inputs (B2.003.a).

B2.023. Cellular promotion gates (ADR-0341) are preserved verbatim. Cell-level promotion remains a separate manifest block from `sharding_automation`. Auto-rebalance triggers WHEN a cell breaches promotion-gate criteria due to load skew, but the gates themselves are unchanged.

B2.024. The oya-shuffle-sharding crate (ADR-0333) is preserved verbatim. The crate gains documentation for its `split` + `merge` primitive APIs consumed by dynamic sharding; the algorithm is unchanged.

B2.025. The compliance pack constraints (ADR-0251) are preserved verbatim. PHI cross-pack migration is refused by E.3 + the compliance-pack-aware Cedar policies authored under ADR-0251.

B2.026. Cross-jurisdiction migration requires an explicit Cedar permit per ADR-0243. The permit is issued by the tenant (or tenant's authorized representative) via an explicit Cedar policy fragment; the permit is consumed by the auto-rebalance candidate-cell selector; the permit ID is recorded in the audit-chain row per ADR-0263.

B2.027. The orchestrator principal is `oyatie.foundry.cell-orchestrator` per ADR-0247 (self-modification doctrine). All automation events run under this principal's Cedar context; the principal is authorized via standard Cedar policies; no special-case carve-out.

B2.028. Per `feedback_no_silent_regression`, every automation event MUST be observable + reversible + audit-emit. Silent state mutation is refused at the lane level by E.5 + at the IP-authoring level by E.6.

B2.029. Per `feedback_canonical_base_localization`, the canonical-base autosharding behavior is provider-agnostic; per-pack overlays (e.g., HIPAA-pack-specific candidate-cell filtering; CSAP-pack-specific sovereign-domain filtering) are declared in the per-pack policy adapters. The canonical base is preserved across packs.

B2.030. The doctrine PR landing this ADR carries (a) the ADR file; (b) the manifest-schema admission edit; (c) the six new CI lane scaffolds as REPORT-ONLY GitHub Actions workflows + lane records in registry/quality/lanes.yaml; (d) the master-plan Wave 15-ZD sub-wave entry; (e) the cellular-exemption-allowlist file scaffold. Implementation (the cell-orchestrator Rust crate or coordinator subsystem) is NOT in the doctrine PR.

B2.031. Multispectrum review v2.4.0 applies to this ADR per ADR-0322 §D-2. Review evidence at `evidence/debate/ADR-0348/<facet>.md` after this ADR lands in a review-track PR.

B2.032. The ADR is announced in the realignment-wave findings aggregation and in the next ADR-0327 promotion gate report.

B2.033. The ADR's enforcement and sunset run in coordination with Wave 15-ZD landing.

B2.034. The 30-day sunset window starts on Wave-15-ZD-completion (not on this ADR's Acceptance). Until Wave 15-ZD lands, the six new lanes are REPORT-ONLY. After Wave 15-ZD lands, the 30-day window begins; at day 30, the lanes promote to BLOCKER for new authoring.

### B.3 What this decision does not do

- This ADR does not author the cell-orchestrator implementation; Wave 15-ZD does.
- This ADR does not retire any prior ADR or change µservice ownership.
- This ADR does not change the oya-shuffle-sharding crate; only its documentation gains the split + merge API description.
- This ADR does not relax compliance pack or residency constraints.
- This ADR does not introduce a new µservice; the cell-orchestrator is composed across tenancy + observability.
- This ADR does not enable cross-jurisdiction migration without an explicit Cedar permit.

## Consequences

### C.1 Positive consequences

- **Hyperscaler-grade automation matched.** Operator-driven placement / rebalance / shard adjustment is eliminated; the three modes match AWS Cell SHIELD + Spanner + DynamoDB + Cosmos + Snowflake precedent.
- **Tenant onboarding scales.** Tenant→cell placement is computed automatically; ~10,000+ paid tenants × ~77 µservices × ~10-100 cells × ~3-5 regions per pack is structurally feasible.
- **Load skews resolved without manual ops.** Auto-rebalance moves tenants automatically; operator sets thresholds; control plane executes.
- **Shard saturation resolved without manual ops.** Hot-split + cold-merge run automatically; operator sets thresholds; control plane executes.
- **Compliance + residency invariants preserved.** Auto-rebalance honors compliance packs + residency; cross-jurisdiction migration requires explicit Cedar permit.
- **Every automation event auditable.** Audit-chain emit per ADR-0263 records every event; compliance auditors can verify residency + pack honoring on every migration.
- **Reversibility preserved.** Every automation event is reversible via the audit-chain row; rollback is operator-initiated or control-plane-initiated.
- **Cellular ownership coherent.** Cell-orchestrator is a logical responsibility composed across tenancy + observability per ADR-0333; no new µservice introduced.
- **Manifest-driven discipline.** Per-µservice declaration of automation modes + thresholds; per-µservice CI enforcement; no silent operator-team-by-team default drift.
- **Doctrine-first cadence.** Doctrine PR lands before implementation; per ADR-0328 batch discipline; no implementation-driven drift.

### C.2 Negative consequences

- **Doctrine + lane scaffolding cost.** The doctrine PR carries the ADR + manifest-schema admission + six lane scaffolds + master-plan sub-wave entry + exemption allowlist file. Estimated ~3,500 lines (ADR is ~2,500 lines + schema delta ~80 lines + lane scaffolds ~900 lines + sub-wave entry ~100 lines + allowlist ~50 lines). Mitigation: the ADR is doctrine-only; the implementation cost lands in Wave 15-ZD.
- **Wave 15-ZD implementation cost.** Implementation in tenancy + observability is a non-trivial multi-µservice change (estimated ~15,000-25,000 lines of Rust across the orchestrator subsystems + integration tests + per-µservice manifest declaration updates). Mitigation: Wave 15-ZD is sequenced under ADR-0328 batch discipline; the orchestrator is composed across existing µservices, not a new µservice.
- **Per-µservice manifest declaration churn.** All ~77 µservices need a `sharding_automation` block (minus exempt µservices). Mitigation: declaration is bounded to ~12-line block per manifest; per-µservice authoring under ADR-0322 substance bar; the EXEMPT_FROM_CELLULAR allowlist captures static-only / no-tenant-state surfaces.
- **Default threshold rejection.** E.4 rejects manifest declarations that omit dynamic-sharding thresholds; per-µservice declaration is required even though defaults exist. Mitigation: the defaults exist as documented starting points; per-µservice review is intentional to capture per-workload load characteristics.
- **Cross-jurisdiction Cedar permit overhead.** Cross-jurisdiction migration requires explicit tenant-issued Cedar permits; tenants need to author + sign permits. Mitigation: the permit cost reflects the actual sovereignty boundary cost; without it, sovereign cells are not actually sovereign.
- **Coordinator pattern complexity.** Cell-orchestrator implementation as a cross-µservice coordinator (tenancy + observability) is a more complex shape than a dedicated orchestrator µservice. Mitigation: the coordinator pattern is established (ADR-0145 + ADR-0263 use the pattern); the alternative (revive cell µservice) is refused by ADR-0333; the composition is intentional.

### C.3 Neutral consequences

- **oya-shuffle-sharding crate unchanged.** The crate continues to provide the shuffle-sharding algorithm; only its API documentation gains the split + merge primitive description.
- **cell_placement_class field unchanged.** ADR-0248 Tier 0..4 declaration remains a separate manifest field.
- **capacity_model schema unchanged.** ADR-0340 schema preserved; the field is READ by the autosharding control plane.
- **Cellular promotion gates unchanged.** ADR-0341 cell-level promotion gates preserved verbatim; auto-rebalance triggers underneath the gates.
- **Compliance pack constraints unchanged.** ADR-0251 packs constrain auto-rebalance candidates; the constraints themselves are unchanged.
- **Cedar policy engine unchanged.** ADR-0243 Cedar engine is consumed for cross-jurisdiction permits; the engine itself is unchanged.
- **Audit-chain emission contract unchanged.** ADR-0263 emission contract is consumed for every automation event; the contract itself is unchanged.

### C.4 Engineering-rigor dimensions

| Dimension | Requirement created by this ADR | Acceptance signal |
|---|---|---|
| Automation | Three modes (autosharding + auto-rebalance + dynamic-sharding) control-plane-driven | Manifest declarations green across non-exempt µservices |
| Manifest discipline | `sharding_automation` block in every non-exempt manifest | E.1 lane green corpus-wide |
| Mode discipline | `autosharding: "control_plane_driven"` (manual refused) | E.2 lane green corpus-wide |
| Residency honoring | `auto_rebalance.honors_residency: true` mandatory if enabled | E.3 lane green corpus-wide |
| Threshold completeness | Four dynamic-sharding thresholds declared per-µservice | E.4 lane green corpus-wide |
| Observability | Every automation event audit-chain-emit per ADR-0263 | E.5 lane green corpus-wide |
| Reversibility | IP authoring carries `rollback_path` section | E.6 lane green for new IP authoring |
| Compliance | PHI cross-pack migration refused absent BYOK + ADR-0251 §D-10 permit | Compliance pack-aware Cedar policies green |
| Jurisdiction | Cross-jurisdiction migration requires explicit Cedar permit per ADR-0243 | Cedar policy evaluation green |
| Cellular ownership | Cell-orchestrator composed across tenancy + observability (NOT new µservice) | ADR-0333 absorption preserved |
| Hyperscaler alignment | Three modes match AWS Cell SHIELD + Spanner + DynamoDB + Cosmos + Snowflake precedent | M1 facet evidence green |
| Substance-bar | Per-mode mechanics + per-lane enforcement (not template-stamped prose) | M2 facet evidence green; ADR-0322 lane green |
| Anti-script | Doctrine ADR bespoke; manifest field admission mechanical | ADR-0324 lane green |
| Doctrine-first | Doctrine PR lands before Wave 15-ZD implementation | Wave 15-ZD sub-wave entry sequenced after this ADR Acceptance |
| Bominal inheritance | Bominal sibling ADR authored independently | Bominal corpus carries sibling autosharding ADR |

### C.5 Hyperscaler-grade rigor application

**Named precedent.** AWS S3 Cell SHIELD migrator + DynamoDB Adaptive Capacity bundle automatic tenant + capacity rebalance across cellular topology with declared thresholds; control plane executes; operator does not. Google Spanner automatic re-sharding bundles split + merge logic with declared thresholds; control plane executes atomically with multi-region paxos + audit emit. Azure Cosmos automatic re-partitioning bundles partition count adjustment with declared logical-partition load thresholds. Snowflake automatic clustering bundles clustering key re-organization with declared keys. Each precedent confirms the three-mode shape is hyperscaler-typical for cellular topology automation.

**Failure-mode tree.** Failure modes:
(1) Manifest declares `autosharding: "manual"` → E.2 refuses; reviewer authors mode discipline correction.
(2) Manifest declares auto_rebalance.enabled without honors_residency → E.3 refuses; reviewer authors residency-honoring field.
(3) Manifest declares dynamic_sharding.enabled with missing thresholds → E.4 refuses; reviewer authors per-µservice threshold declaration.
(4) Manifest declares auto_rebalance.enabled OR dynamic_sharding.enabled with missing audit_chain_emit → E.5 refuses; reviewer authors emission declaration.
(5) IP authoring for auto-rebalance lacks rollback_path section → E.6 refuses; reviewer authors rollback path.
(6) Cross-jurisdiction migration attempted without Cedar permit → Cedar policy evaluation refuses at runtime; the auto-rebalance candidate-cell selector excludes the cross-jurisdiction cell.
(7) PHI cross-pack migration attempted without ADR-0251 §D-10 permit → compliance pack-aware Cedar policy refuses at runtime.
(8) Hot-split splits a shard during a tenant-active transaction → atomicity invariant: the split transactions on the new shard topology after the transaction commits; in-flight transactions on the old topology drain to completion or roll back per ADR-0252 HLC semantics.
(9) Cold-merge merges shards while a tenant is in the middle of a write → atomicity invariant: the merge atomically redirects writes to the merged shard after a brief read-only window.
(10) Auto-rebalance migration interrupts a tenant's session → migration is transparent at the api-gateway layer; the session re-routes to the new cell at the same transaction boundary; session state is migrated as part of the atomic transition.

**Capacity math.** Per-µservice automation event rate: ~1-10 auto-rebalance events per cell per quarter (estimated based on hyperscaler precedent — Spanner re-sharding rate at ~1 per database per quarter); ~1-100 hot-split events per cell per year (estimated based on DynamoDB partition split rate at ~5 per table per year); ~1-10 cold-merge events per cell per year. Aggregate audit-chain emission rate per ~77 µservices × ~10-100 cells × ~3-5 regions ≈ ~3,000-150,000 events/year. Audit-chain capacity per ADR-0263 is sized for ~10^9 events/year; the automation event rate is well within capacity. Cell-orchestrator implementation runtime cost: ~5-10ms placement decision per autosharding event; ~30s atomic transition per auto-rebalance migration; ~5-30s atomic transition per hot-split / cold-merge. Per `feedback_quality_performance_scalability_bar`, the runtime envelope is bounded.

**Observability hooks.** Automation-aware metrics:
- `autosharding_placement_decisions_total` — count of tenant→cell/shard placement decisions per µservice.
- `auto_rebalance_migrations_total` — count of tenant migrations per µservice + per cell-source + per cell-target.
- `dynamic_sharding_hot_splits_total` — count of hot-split events per µservice + per cell.
- `dynamic_sharding_cold_merges_total` — count of cold-merge events per µservice + per cell.
- `auto_rebalance_residency_check_refused_total` — count of auto-rebalance candidates refused due to residency mismatch.
- `auto_rebalance_compliance_pack_check_refused_total` — count of auto-rebalance candidates refused due to compliance pack mismatch.
- `auto_rebalance_cedar_permit_required_total` — count of auto-rebalance attempts that required cross-jurisdiction Cedar permits.
- `automation_event_audit_chain_emit_total` — count of audit-chain emissions per automation event type.

**Rollback path.** Per-event rollback: audit-chain row enables inverse operation per B2.007 (auto-rebalance) / B2.013 (hot-split / cold-merge). Per-doctrine rollback: revert the doctrine PR commit; the manifest-schema admission is rolled back; the six lane scaffolds are removed; the master-plan Wave 15-ZD entry is removed; per-µservice manifest declarations are removed via per-µservice rollback IPs. The doctrine rollback is git-revert-clean because the doctrine PR is mechanical scaffold + ADR text; no in-memory state mutation.

**Multi-region awareness.** The three modes operate per-region within each pack. Cross-region tenant migration (e.g., moving a tenant from a US region cell to an APAC region cell) requires cross-region Cedar permit per ADR-0158 (multi-region-active-active); the auto-rebalance candidate-cell selector treats cross-region migration as cross-jurisdiction by default.

**Sovereign-cell awareness.** Sovereign cells (per ADR-0240 sovereign-cloud-per-regional-pack) are treated as residency-scoped: tenants in a sovereign pack cannot auto-rebalance out of the sovereign domain absent an explicit tenant-issued Cedar permit. CSAP KR cells + EU GDPR-strict cells + IL5 cells + PCI cells are all sovereign-scoped; auto-rebalance honors per-cell residency declarations.

**Versioning + deprecation.** Per ADR-0108 sunset discipline. The doctrine PR is the canonical ADR-0348 v1.0.0; the six new lanes start REPORT-ONLY; promote to BLOCKER 30 days post-Wave-15-ZD completion. The `sharding_automation` manifest block is admitted v1.0.0 in the doctrine PR; future schema evolution (e.g., adding cross-region rebalance fields) is authored as ADR amendments with explicit version bump per ADR-0108.

## D. Detailed mechanics — eight automation surfaces (D-1..D-8)

The autosharding + auto-rebalance + dynamic-sharding doctrine touches eight surfaces in the corpus. Subsections D-1 through D-8 enumerate each surface. Numbering is normative.

### D-1: Per-µservice manifest.json `sharding_automation` block

D-1.1. Every non-exempt µservice manifest.json gains a `sharding_automation` block of the following canonical shape (declared in `specs/microservices/manifest-schema.json` in the doctrine PR):

```json
{
  "sharding_automation": {
    "autosharding": "control_plane_driven",
    "auto_rebalance": {
      "enabled": true,
      "trigger_load_skew_threshold_percent": 30,
      "honors_residency": true,
      "honors_compliance_packs": true,
      "audit_chain_emit": true
    },
    "dynamic_sharding": {
      "enabled": true,
      "hot_split_threshold_p99_ms": 50,
      "hot_split_utilization_threshold_percent": 80,
      "cold_merge_utilization_threshold_percent": 20,
      "cold_merge_minimum_quiet_hours": 24,
      "audit_chain_emit": true
    }
  }
}
```

D-1.2. The `autosharding` field is a string enum: `{"control_plane_driven", "manual"}`. The canonical value is `"control_plane_driven"`. The `"manual"` value is refused by E.2.

D-1.3. The `auto_rebalance` sub-block carries:
- `enabled` (boolean; mandatory)
- `trigger_load_skew_threshold_percent` (integer 1..100; default 30; per-µservice override)
- `honors_residency` (boolean; MUST be true if enabled; refused by E.3 otherwise)
- `honors_compliance_packs` (boolean; MUST be true if enabled)
- `audit_chain_emit` (boolean; MUST be true if enabled; refused by E.5 otherwise)

D-1.4. The `dynamic_sharding` sub-block carries:
- `enabled` (boolean; mandatory)
- `hot_split_threshold_p99_ms` (integer; per-µservice declarable; default-fill rejected by E.4 — explicit declaration required if enabled)
- `hot_split_utilization_threshold_percent` (integer 1..100; per-µservice declarable; default-fill rejected)
- `cold_merge_utilization_threshold_percent` (integer 1..100; per-µservice declarable; default-fill rejected)
- `cold_merge_minimum_quiet_hours` (integer; per-µservice declarable; default-fill rejected)
- `audit_chain_emit` (boolean; MUST be true if enabled; refused by E.5 otherwise)

D-1.5. The block schema is admitted to `specs/microservices/manifest-schema.json` in the doctrine PR. The manifest-schema admission is the only schema delta in the doctrine PR.

D-1.6. The `.omc/state/cellular-exemption-allowlist-2026-05-21.json` file enumerates µservices on the EXEMPT_FROM_CELLULAR list (e.g., static-only edge surfaces, no-tenant-state µservices). Exempt µservices may omit the `sharding_automation` block.

D-1.7. The discovery enumeration at authoring time of this ADR estimates ~77 µservices in the corpus; exempt subset is ~5-10 (e.g., docs-site, marketing-site, status-page edge surfaces). Per-µservice manifest declaration churn is ~65-70 manifests.

### D-2: Cell-orchestrator responsibility composition across tenancy + observability + cloud-iac + audit-chain

D-2.1. The cell-orchestrator is a LOGICAL RESPONSIBILITY composed across:
- **tenancy**: tenant→cell/shard assignment registry; placement decision recording; reversibility audit-trail.
- **observability**: load-skew detection; metric-driven threshold evaluation; rebalance trigger emission.
- **cloud-iac**: cell provisioning if new cells are needed; shard catalog updates for hot-split / cold-merge.
- **oya-shuffle-sharding crate**: shuffle-sharding algorithm; split + merge primitive APIs.
- **audit-chain**: cell-scoped audit emission per ADR-0263 for every automation event.
- **api-gateway**: cell-aware routing; tenant→cell lookup at request time; routing transitions during auto-rebalance.

D-2.2. Implementation is via the cross-µservice coordinator pattern established in ADR-0145 (inter-microservice communication reform) + ADR-0263 (observability emission contract). The orchestrator subsystem lives inside tenancy (primary owner; placement registry) + observability (secondary owner; load-skew detection) jointly.

D-2.3. The orchestrator principal is `oyatie.foundry.cell-orchestrator` per ADR-0247 (self-modification doctrine). All automation events run under this principal's Cedar context.

D-2.4. The orchestrator does NOT need a new µservice. ADR-0333 (cell µservice retired — pattern not service) is preserved.

D-2.5. Implementation is queued as Wave 15-ZD (separate executor PR sequenced under ADR-0328 batch discipline). The doctrine PR is the doctrine + manifest-schema admission + lane scaffolds.

### D-3: Autosharding inputs + algorithm

D-3.1. Autosharding placement algorithm consumes the following inputs:
- **capacity_model** per ADR-0340 (storage_per_tenant + cpu_per_tenant + memory_per_tenant + network_per_tenant) — input to capacity-headroom filter on candidate cells.
- **compliance_pack constraints** per ADR-0251 — input to compliance-pack-allowed cells filter.
- **ResidencyClass** per ADR-0240 — input to residency-domain filter.
- **cell_placement_class** per ADR-0248 (Tier 0..4) — input to tier-match filter; tenant goes to a cell of matching tier.
- **shuffle-sharding algorithm** per ADR-0333 (oya-shuffle-sharding crate) — input to canonical shuffle-shard primitive.

D-3.2. Algorithm steps (canonical):
1. Filter candidate cells by Tier match (cell_placement_class).
2. Filter candidate cells by ResidencyClass match.
3. Filter candidate cells by active compliance packs.
4. Filter candidate cells by capacity headroom (consumed via capacity_model × current tenant load).
5. Apply shuffle-sharding hash on tenant_id; select target cell from the remaining candidate set.
6. Within the selected cell, apply shuffle-sharding hash on tenant_id + cell_id; select target shard from the cell's shard set.
7. Record placement decision in tenancy assignment registry; emit audit-chain row per ADR-0263.

D-3.3. The algorithm is implemented in the oya-shuffle-sharding crate's `place_tenant` API; the API is consumed by the orchestrator subsystem in tenancy + observability.

D-3.4. The algorithm is deterministic per tenant_id + candidate cell set + shard set; the same inputs produce the same placement decision; the algorithm is reproducible for audit purposes.

### D-4: Auto-rebalance trigger + candidate selection + migration execution

D-4.1. Auto-rebalance trigger: observability detects cell load skew when a cell's load deviates from the per-µservice mean by more than `trigger_load_skew_threshold_percent` (default 30; per-µservice override). The trigger emits an event to the orchestrator subsystem.

D-4.2. Candidate-target-cell selection (per B2.005):
1. Filter by ResidencyClass match (mandatory; refused absent Cedar permit if cross-jurisdiction).
2. Filter by active compliance packs match (mandatory; refused absent ADR-0251 §D-10 BYOK if cross-pack PHI).
3. Filter by cell_placement_class match (Tier 0..4 match required).
4. Filter by capacity headroom (target cell has > 30% headroom after migration).
5. Filter by Cedar permit if cross-jurisdiction.

D-4.3. Migration execution (atomic per tenant per B2.006):
1. Tenancy assignment registry: transition tenant_id from cell_source to cell_target in a single ACID transaction.
2. api-gateway routing: switch routing to the target cell at the same transaction boundary.
3. Session state migration: if tenant has active session state on cell_source, migrate the state to cell_target within the atomic transition.
4. Audit-chain emission: record the transition per ADR-0263 (cell_source + cell_target + residency_check_result + compliance_pack_check_result + cedar_permit_id if applicable + initiated_by "control_plane:cell-orchestrator").

D-4.4. Reversibility (per B2.007): the audit-chain row enables the inverse operation; rollback re-applies the inverse transition with a new audit-chain row marking it as a rollback of the prior automation event.

D-4.5. Refusal cases: if no candidate cell satisfies the filters, auto-rebalance refuses + escalates to operator. The escalation emits an observability event + a notification per ADR-0263.

### D-5: Dynamic sharding hot-split + cold-merge execution

D-5.1. Hot-split trigger: observability detects shard p99 latency exceeding `hot_split_threshold_p99_ms` (default 50) OR shard utilization exceeding `hot_split_utilization_threshold_percent` (default 80). Per-µservice declarable; default-fill rejected by E.4.

D-5.2. Hot-split execution (atomic per cell per B2.012):
1. oya-shuffle-sharding crate: split the shard into 2 sub-shards using the `split` primitive.
2. Cell registry (cloud-iac shard catalog): atomically update the cell's shard count from N to N+1.
3. Hash-ring assignments: atomically redistribute tenants on the split shard to one of the 2 sub-shards based on shuffle-sharding hash.
4. api-gateway routing: switch routing to the new shard topology at the same transaction boundary.
5. In-flight transactions: drain to completion on the old topology or roll back per ADR-0252 HLC semantics; new transactions hit the new topology.
6. Audit-chain emission: record the split per ADR-0263.

D-5.3. Cold-merge trigger: observability detects two adjacent shards both running below `cold_merge_utilization_threshold_percent` (default 20) for more than `cold_merge_minimum_quiet_hours` (default 24). Per-µservice declarable; default-fill rejected by E.4.

D-5.4. Cold-merge execution (atomic per cell per B2.012):
1. Verify both adjacent shards have run below the merge threshold for the minimum quiet hours.
2. oya-shuffle-sharding crate: merge them into 1 shard using the `merge` primitive.
3. Cell registry: atomically update the cell's shard count from N to N-1.
4. Hash-ring assignments: atomically redistribute tenants to the merged shard.
5. api-gateway routing: switch routing to the new shard topology at the same transaction boundary.
6. Brief read-only window: writes pause for the atomicity boundary; reads continue.
7. Audit-chain emission: record the merge per ADR-0263.

D-5.5. Reversibility (per B2.013): hot-split → cold-merge of the two sub-shards; cold-merge → hot-split of the merged shard. The inverse operation is enumerable from the audit-chain row.

### D-6: Audit-chain emission per ADR-0263 for every automation event

D-6.1. Every automation event (auto-rebalance migration, hot-split, cold-merge) emits an audit-chain row per ADR-0263 observability emission contract.

D-6.2. Emission schema (canonical):
```json
{
  "event_type": "auto_rebalance_migration" | "dynamic_sharding_hot_split" | "dynamic_sharding_cold_merge",
  "microservice": "<microservice_name>",
  "cell_id": "<cell_id>",
  "shard_id": "<shard_id>",
  "tenant_id": "<tenant_id>",
  "pre_state": { ... },
  "post_state": { ... },
  "residency_check_result": "match" | "refused" | "cross_jurisdiction_with_permit",
  "compliance_pack_check_result": "match" | "refused" | "cross_pack_with_permit",
  "cedar_permit_id": "<permit_id_if_applicable>",
  "initiated_by": "control_plane:cell-orchestrator",
  "timestamp": "<HLC_timestamp>",
  "audit_chain_seal": "<seal_hash>"
}
```

D-6.3. The audit-chain emission is enforced at the lane level by E.5 (`oya-governance-audit-chain-emit-on-automation-events`): any manifest declaring auto_rebalance.enabled OR dynamic_sharding.enabled MUST declare `audit_chain_emit: true` on the corresponding sub-block; otherwise the lane refuses.

D-6.4. The audit-chain emission is consumed by compliance auditors per ADR-0251 audit pipeline: every migration must be reconstructable from the audit-chain trail; every residency check + compliance pack check + cedar permit must be verifiable from the audit-chain row.

D-6.5. The audit-chain seal hash is per ADR-0263 emission contract; the seal is verified by the audit-chain µservice; tamper detection is per ADR-0263 invariants.

### D-7: Tenant migration reversibility — IP authoring `rollback_path` section

D-7.1. Every µservice IP authoring under `{oya,cloud}/<service>/ (legacy microservices/ removal-candidate) IPs/IP-*-auto-rebalance-*.md` MUST carry an explicit `rollback_path` section.

D-7.2. The `rollback_path` section enumerates:
- (a) How the automation-event-driven tenant migration is reversed via the audit-chain trail.
- (b) Which operator role is authorized to initiate the rollback (typically: ops-sre-reliability for auto-rebalance; ops-platform for hot-split / cold-merge).
- (c) The Cedar policy fragment authorizing the rollback principal.
- (d) The audit-chain emission contract for the rollback event (rollback is itself an automation event; rollback emits per ADR-0263).
- (e) Edge cases: in-flight transaction handling during rollback; session state restoration; cross-jurisdiction rollback (requires new Cedar permit).

D-7.3. The `rollback_path` section is enforced by E.6 (`oya-governance-tenant-migration-reversibility`). New IP authoring without the section is refused.

D-7.4. Existing IPs authored before this ADR's Acceptance are NOT retroactively required to carry the section; the lane applies prospectively to new IP authoring.

### D-8: New CI lane scaffolds in the doctrine PR

D-8.1. The doctrine PR landing this ADR carries scaffolds for the six new CI lanes (E.1..E.6). Each scaffold is:
- (a) A GitHub Actions workflow file at `.github/workflows/<lane-name>.yml`.
- (b) A lane record in `registry/quality/lanes.yaml`.
- (c) A catalog record at `registry/catalog/<lane-name>.yaml`.
- (d) A Rust check-family crate scaffold at `crates/<lane-name>-*/` (kernel + adapter; the actual check logic is implemented in Wave 15-ZD).

D-8.2. The scaffolds start REPORT-ONLY at this ADR's Acceptance. They promote to BLOCKER 30 days post-Wave-15-ZD completion.

D-8.3. Per `feedback_no_silent_regression`, the scaffolds emit a warning on detected violations during the REPORT-ONLY phase; reviewers can see the violations + author corrections before the BLOCKER promotion.

D-8.4. The scaffolds are consumed by `tools/hooks/_canonical-primitives.md` Lifecycle Skill Map per the canonical-primitives cheat sheet pattern.

## E. Enforcement-by-lanes

E.1 `oya-governance-sharding-automation-coverage` (new) — refuses any non-exempt µservice manifest.json that lacks a complete `sharding_automation` block declared per the D-1 schema. The lane reads the EXEMPT_FROM_CELLULAR allowlist at `.omc/state/cellular-exemption-allowlist-2026-05-21.json`; exempt µservices may omit the block. REPORT-ONLY at this ADR's Acceptance; promotes to BLOCKER 30 days post-Wave-15-ZD-completion for new authoring.

E.2 `oya-governance-autosharding-manual-mode-refusal` (new) — refuses any manifest.json that declares `sharding_automation.autosharding: "manual"`. The canonical autosharding mode is `"control_plane_driven"`. A manual-mode exception requires an ADR-amendment to this ADR enumerating the surface justifying the exception. REPORT-ONLY at this ADR's Acceptance; promotes to BLOCKER 30 days post-Wave-15-ZD-completion for new authoring.

E.3 `oya-governance-auto-rebalance-residency-honored` (new) — greps every manifest declaring `sharding_automation.auto_rebalance.enabled: true` and refuses if the same manifest also declares `honors_residency: false` OR omits the field. Cross-jurisdiction rebalance without Cedar permit is refused at admission time per ADR-0243. REPORT-ONLY at this ADR's Acceptance; promotes to BLOCKER 30 days post-Wave-15-ZD-completion for new authoring.

E.4 `oya-governance-dynamic-sharding-threshold-coverage` (new) — refuses any manifest declaring `sharding_automation.dynamic_sharding.enabled: true` that omits ANY of the four canonical thresholds: `hot_split_threshold_p99_ms`, `hot_split_utilization_threshold_percent`, `cold_merge_utilization_threshold_percent`, `cold_merge_minimum_quiet_hours`. Default-fill is REJECTED to force per-µservice declaration of load characteristics. REPORT-ONLY at this ADR's Acceptance; promotes to BLOCKER 30 days post-Wave-15-ZD-completion for new authoring.

E.5 `oya-governance-audit-chain-emit-on-automation-events` (new) — greps every manifest declaring `auto_rebalance.enabled: true` OR `dynamic_sharding.enabled: true` and refuses if the same manifest omits `audit_chain_emit: true` on the corresponding sub-block. Every automation event MUST emit per ADR-0263 observability-emission-contract. REPORT-ONLY at this ADR's Acceptance; promotes to BLOCKER 30 days post-Wave-15-ZD-completion for new authoring.

E.6 `oya-governance-tenant-migration-reversibility` (new) — refuses any µservice IP authoring under `{oya,cloud}/<service>/ (legacy microservices/ removal-candidate) IPs/IP-*-auto-rebalance-*.md` that lacks an explicit `rollback_path` section enumerating how an automation-event-driven tenant migration is reversed via the audit-chain trail. The lane applies prospectively to new IP authoring; existing IPs are not retroactively required. REPORT-ONLY at this ADR's Acceptance; promotes to BLOCKER 30 days post-Wave-15-ZD-completion for new authoring.

E.7 `oya-governance-cell-orchestrator-no-new-microservice` (informational; not enforced as a blocker) — verifies that the corpus does not introduce a new `cell-orchestrator` µservice directory under `microservices/`. The cell-orchestrator responsibility is a logical composition across tenancy + observability + cloud-iac + audit-chain per ADR-0333; introducing a new µservice would revive cell µservice ownership shape that ADR-0333 retired. REPORT-ONLY indefinitely.

## F. Alternatives Rejected

F.1 **Operator-driven placement (status quo).** Continue requiring operator picks for tenant→cell/shard placement; rebalance via operator-initiated migration; shard count adjustment via operator-initiated split / merge. Rejected because: (a) hyperscaler-grade horizontal scalability per `feedback_quality_performance_scalability_bar` requires control-plane-driven automation; (b) ~10,000+ paid tenants × ~77 µservices × ~10-100 cells × ~3-5 regions per pack is structurally impossible for operator-driven placement; (c) hyperscaler precedent (AWS Cell SHIELD + Spanner + DynamoDB + Cosmos + Snowflake) is uniformly automatic; (d) operator-driven placement causes drift in capacity utilization + tenant distribution per `feedback_drift_too_big_2026_05_20`.

F.2 **Auto-rebalance via consistent hashing without shuffle sharding.** Replace shuffle-sharding (oya-shuffle-sharding crate per ADR-0333) with consistent hashing for tenant→cell placement. Rejected because: (a) consistent hashing does not provide the blast-radius isolation that shuffle sharding does (a hot tenant on a consistent-hash cell affects all neighbor tenants on the same hash slot; shuffle sharding limits per-tenant cell intersection); (b) ADR-0333 declared shuffle sharding as the canonical algorithm via the oya-shuffle-sharding crate; (c) AWS Cell SHIELD + similar hyperscaler cellular topologies use shuffle sharding for blast-radius isolation; (d) the algorithm choice is settled per ADR-0333; this ADR consumes the canonical primitive.

F.3 **Static shard count per cell (no dynamic sharding).** Declare shard count per cell at provisioning time; never adjust. Rejected because: (a) tenant load is non-stationary; shard saturation is structural and requires manual intervention without dynamic sharding; (b) hyperscaler precedent (Spanner automatic re-sharding + DynamoDB adaptive capacity + Cosmos automatic re-partitioning) is uniformly dynamic; (c) cold-merge enables cost reclamation as tenants migrate away; without it, cells carry permanently underused capacity; (d) the dynamic sharding shape is hyperscaler-typical.

F.4 **Cross-jurisdiction migration without Cedar permit.** Allow auto-rebalance to migrate tenants across jurisdictions automatically based on residency-policy heuristics. Rejected because: (a) ADR-0243 declares every gate is a Cedar eval; cross-jurisdiction migration is a gate; the gate must be Cedar-evaluated; (b) ADR-0240 declares sovereign cells are residency-scoped; cross-jurisdiction migration without explicit tenant authorization violates the sovereignty contract; (c) GDPR + CSAP + similar regulatory frameworks require explicit tenant authorization for cross-jurisdiction data movement; (d) automatic cross-jurisdiction migration without Cedar permit creates compliance liability; (e) the Cedar permit overhead reflects the actual sovereignty boundary cost.

F.5 **Silent (non-audit-chain) automation events.** Allow auto-rebalance + hot-split + cold-merge to execute without audit-chain emission, treating them as internal control-plane operations not visible to tenants. Rejected because: (a) `feedback_no_silent_regression` declares silent state mutation is refused; (b) ADR-0263 declares every state-mutating operation emits per the canonical observability-emission-contract; (c) compliance auditors per ADR-0251 require every migration to be reconstructable from the audit-chain trail; (d) reversibility requires audit-chain rows enabling inverse operations; (e) tenants must be able to query their own migration history for transparency.

F.6 **Cell-orchestrator as a new µservice.** Introduce a new `microservices/cell-orchestrator/` µservice to own the three automation modes. Rejected because: (a) ADR-0333 retired the cell µservice as a first-class deliverable; reviving a cell-orchestrator µservice partially undoes the retirement; (b) the cellular shape is a PATTERN composed across existing µservices per ADR-0333; the orchestrator responsibility is consistent with the pattern; (c) `feedback_microservice_ownership_coherence_2026_05_20` requires one team owns one µservice end-to-end; the orchestrator responsibility is consistent with the cross-µservice coordinator pattern (ADR-0145 + ADR-0263) which has joint-team ownership precedent; (d) the composition reduces the µservice count + the integration test surface.

F.7 **Per-tenant manual override of placement.** Allow individual tenants to manually pick their cell + shard at onboarding time; override the control-plane decision. Rejected because: (a) tenant-level placement override defeats the purpose of autosharding; (b) hyperscaler precedent (AWS account placement, Google project placement) does not expose cell + shard as tenant-facing concepts; (c) tenants are abstracted from cellular topology by design; (d) operator-escalation paths exist for genuinely-needed placement overrides via ADR-0247 self-modification doctrine principals; manual override is the exception not the rule.

F.8 **Implement before doctrine (Wave 15-ZD authored before ADR).** Author the cell-orchestrator implementation in tenancy + observability without a preceding doctrine ADR. Rejected because: (a) ADR-0328 declares doctrine ADRs land before implementation sub-waves; (b) implementing without doctrine creates scattered surface-by-surface decisions that drift per `feedback_drift_too_big_2026_05_20`; (c) the manifest field admission + lane scaffolds need a doctrine anchor; (d) per `feedback_verify_deliverables_not_just_line_count_2026_05_20`, implementation without doctrine cannot be reviewed for ADR-adherence + architectural coherence + hyperscaler-grade quality.

## G. Multispectrum Review v2.4.0

Per ADR-0322 §D-2 and ADR-0328 §D-4, this ADR is subject to multispectrum-review v2.4.0 evaluation across the F-family critique facets, M-family meta facets, and A-family own-policy-adherence facets. Evidence files land at `evidence/debate/ADR-0348/<facet>.md` after this ADR is opened in a review-track PR.

The expected critique surface:

- **F1 (correctness).** Is the autosharding algorithm in D-3.2 correct? Does the order of filters preserve the intended semantics (Tier match → residency match → compliance pack match → capacity headroom → shuffle hash)? Does the shuffle-sharding hash select the same target cell across multiple invocations with the same inputs?
- **F2 (architecture).** Is the cell-orchestrator composition across tenancy + observability + cloud-iac + audit-chain correct per ADR-0333? Does the cross-µservice coordinator pattern preserve µservice ownership boundaries per ADR-0145?
- **F3 (security).** Does the Cedar permit requirement for cross-jurisdiction migration correctly prevent unauthorized cross-jurisdiction data movement per ADR-0243 + GDPR + CSAP? Is the orchestrator principal (`oyatie.foundry.cell-orchestrator`) correctly authorized per ADR-0247 self-modification doctrine?
- **F4 (performance).** Are the per-event runtime envelopes (5-10ms placement decision; 30s atomic migration; 5-30s atomic split / merge) bounded for hyperscaler-scale tenant counts? Does the audit-chain emission rate (3,000-150,000 events/year aggregate) fit within ADR-0263 audit-chain capacity?
- **F5 (operability).** Are the operator escalation paths for refusal cases (no candidate cell satisfies filters; cross-jurisdiction permit absent) clearly defined? Is the rollback path enumerable from the audit-chain trail?
- **F6 (compliance).** Does the audit-chain emission contract per ADR-0263 enable compliance auditor reconstruction per ADR-0251? Does the per-pack candidate-cell filtering correctly enforce HIPAA / GDPR / SOC2 / CSAP / PCI / EU-AI-Act constraints?
- **F7 (cost).** Is the doctrine PR cost (~3,500 lines) bounded? Is the Wave 15-ZD implementation cost (~15,000-25,000 lines) bounded? Is the per-µservice manifest declaration churn (~65-70 manifests) bounded?
- **F8 (testability).** How is autosharding tested? Auto-rebalance? Dynamic sharding? Are the integration tests in Wave 15-ZD sufficient to verify the three modes work end-to-end across tenancy + observability + cloud-iac + audit-chain?
- **F9 (failure modes).** Is the failure-mode tree in C.5 complete? Are the atomicity invariants for hot-split / cold-merge / auto-rebalance sound per ADR-0252 HLC semantics?
- **M1 (counterpart-precedent calibration).** Are AWS Cell SHIELD + Spanner + DynamoDB + Cosmos + Snowflake the right precedents? Is the three-mode shape (autosharding + auto-rebalance + dynamic-sharding) hyperscaler-typical?
- **M2 (substance bar).** Is the per-mode mechanics + per-lane enforcement the right substance shape (vs template-stamped per-mode prose)?
- **A1..A7 (own-policy-adherence).** Does this ADR adhere to naming BNF v4 (lane prefixes conform to v4 BNF + ADR-0347 canonical prefix discipline), documentation rigor 1.1, structural placement under `docs/decisions/`, architectural boundaries (cell-orchestrator composed across tenancy + observability per ADR-0333), dependency policy (no new dependencies introduced; consumes oya-shuffle-sharding + ADR-0263 audit-chain + ADR-0243 Cedar), schema (manifest-schema admission of `sharding_automation` block), and algorithmic invariants (shuffle-sharding hash determinism; atomicity per ACID + ADR-0252 HLC)?

## H. Enforcement + Sunset

H.1 **Enforcement transition.** From ADR Acceptance, the six new lanes (§E.1..E.6) start REPORT-ONLY. They promote per the schedule:

- E.1 (`oya-governance-sharding-automation-coverage`) promotes to BLOCKER 30 days post-Wave-15-ZD-completion for new authoring.
- E.2 (`oya-governance-autosharding-manual-mode-refusal`) promotes to BLOCKER 30 days post-Wave-15-ZD-completion for new authoring.
- E.3 (`oya-governance-auto-rebalance-residency-honored`) promotes to BLOCKER 30 days post-Wave-15-ZD-completion for new authoring.
- E.4 (`oya-governance-dynamic-sharding-threshold-coverage`) promotes to BLOCKER 30 days post-Wave-15-ZD-completion for new authoring.
- E.5 (`oya-governance-audit-chain-emit-on-automation-events`) promotes to BLOCKER 30 days post-Wave-15-ZD-completion for new authoring.
- E.6 (`oya-governance-tenant-migration-reversibility`) promotes to BLOCKER 30 days post-Wave-15-ZD-completion for new IP authoring.
- E.7 (`oya-governance-cell-orchestrator-no-new-microservice`) remains informational indefinitely.

H.2 **Sunset window.** The 30-day post-Wave-15-ZD sunset window is the window for new authoring to update to the new prefix. After day 30, new authoring under the legacy prefix is refused outside the historical-context allowlist.

H.3 **Wave 15-ZD sub-wave.** Wave 15-ZD (queued in `/specs/master-plan-sequencing.json#realignment_wave_sequence.waves_15_plus.sub_waves`) is the cell-orchestrator implementation sub-wave. It implements the orchestrator subsystem in tenancy + observability + cloud-iac + audit-chain per D-2 composition. Sub-wave dispatch follows ADR-0328 batch discipline as a multi-µservice coordinator-pattern landing.

H.4 **Exemption allowlist maintenance.** The EXEMPT_FROM_CELLULAR allowlist at `.omc/state/cellular-exemption-allowlist-2026-05-21.json` is maintained over time as new edge µservices are scaffolded. New exemption additions require an ADR amendment to this ADR.

H.5 **Exception clause.** None for the canonical autosharding mode (manual mode is refused by E.2 absent ADR amendment). None for the audit-chain emission requirement (silent automation events are refused by E.5 absent ADR amendment). None for the Cedar permit requirement on cross-jurisdiction migration (refused by Cedar policy evaluation per ADR-0243).

H.6 **Sunset of prior doctrine.** None. This ADR is additive to the cellular topology doctrine in ADR-0248 + ADR-0341 + ADR-0333 + ADR-0340; no prior doctrine is retired.

H.7 **Bominal inheritance window.** Bominal parallel corpus authors its sibling autosharding doctrine ADR independently per `feedback_bominal_inheritance_precedence`. No Oyatie-side enforcement applies to Bominal.

H.8 **Quarterly review.** Council-architecture + ops-sre-reliability + axis-tenancy + axis-observability conduct a quarterly review of:
- Per-µservice threshold declarations (are the per-µservice hot-split / cold-merge thresholds calibrated to observed load characteristics?)
- Automation event rates (are they within the projected envelopes? are runaway migration rates indicating a misconfigured threshold?)
- Exemption allowlist additions (are new edge µservices appropriately exempt or should they declare the block?)
- Failure-mode incidents (have any of the failure modes in C.5 been triggered? do they require ADR amendment?)

## I. Cross-references

I.1 Memory anchors:

- `feedback_autosharding_dynamic_rebalance_2026_05_21` — source memory file declaring the three-mode doctrine.
- `feedback_amazon_shape_cellular_architecture` — AWS cell topology Tier 0..4; shuffle sharding; Cloud Hypervisor.
- `feedback_quality_performance_scalability_bar` — hyperscaler-grade horizontal scalability without manual operator intervention.
- `feedback_no_silent_regression` — every automation event is observable + reversible + audit-emit.
- `feedback_clean_architecture_requirements` — cell-orchestrator composed across tenancy + observability per ADR-0333; consistent with port-in-kernel + cross-product refusal.
- `feedback_microservice_ownership_coherence_2026_05_20` — orchestrator joint-owned by tenancy + observability teams via the cross-µservice coordinator pattern.
- `feedback_verify_deliverables_not_just_line_count_2026_05_20` — verification via lane-coverage + per-µservice manifest declaration, not via line count.
- `feedback_automate_everything` — control-plane-driven automation; no operator-driven placement / rebalance / shard adjustment.
- `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20` — autosharding applies uniformly across demo_trial + paid tenant classes; same automation quality bar.
- `feedback_canonical_base_localization` — canonical-base autosharding behavior + per-pack overlays for compliance-pack-specific candidate-cell filtering.
- `feedback_drift_too_big_2026_05_20` — operator-driven placement causes drift; control-plane-driven automation prevents drift.

I.2 ADR anchors:

- ADR-0110 (changeset state machine) — doctrine PR's changeset state transitions through the standard sequence.
- ADR-0111 (merge queue projected state) — doctrine PR enters the merge queue.
- ADR-0131 (per-microservice flat layout) — preserved verbatim.
- ADR-0132 (no-grouping policy + governance prefix) — preserved verbatim; six new lanes carry the `oya-governance-*` canonical prefix per ADR-0347.
- ADR-0145 (inter-microservice communication reform) — cross-µservice coordinator pattern consumed for cell-orchestrator composition.
- ADR-0150 (Cedar policy engine) — Cedar gates for cross-jurisdiction migration permits.
- ADR-0158 (multi-region active-active) — cross-region migration treated as cross-jurisdiction by default.
- ADR-0181 (cosign signed artifacts) — preserved verbatim.
- ADR-0211 (in-house tech stack preference) — preserved verbatim.
- ADR-0212 (buildability doctrine) — manifest declaration is the canonical declaration surface.
- ADR-0240 (sovereign-cloud per regional pack) — sovereign cells are residency-scoped; auto-rebalance honors per-cell residency declarations.
- ADR-0241 (DR business continuity portfolio policy) — preserved verbatim; tenant migration honors DR floor.
- ADR-0242 (oyatie-is-a-tenant doctrine) — preserved verbatim.
- ADR-0243 (Cedar as universal gate) — cross-jurisdiction migration requires explicit Cedar permit.
- ADR-0244 (tenant scoping universal primitive) — tenant context is universal; every automation event carries tenant_id.
- ADR-0245 (substrate vs product layering) — preserved verbatim.
- ADR-0247 (self-modification doctrine) — orchestrator principal is `oyatie.foundry.cell-orchestrator`.
- ADR-0248 (cellular topology baseline) — amended; tenant→cell/shard placement is control-plane-driven; hot-cell rebalancing is automatic; shard count is dynamic.
- ADR-0250 (build ahead of certification) — preserved verbatim.
- ADR-0251 (compliance-pack-cell-certification-levels) — compliance packs constrain auto-rebalance candidate cells.
- ADR-0252 (HLC default + TrueTime tier) — atomicity invariants for hot-split / cold-merge / auto-rebalance based on HLC semantics.
- ADR-0253 (HTTP/3 + QUIC default) — preserved verbatim.
- ADR-0254 (K8s + Cloud Hypervisor + Kata) — preserved verbatim.
- ADR-0263 (observability emission contract) — every automation event emits per the canonical contract.
- ADR-0322 (substance bar) — doctrine ADR substance per-mode mechanics + per-lane enforcement.
- ADR-0324 (anti-script authoring) — manifest field admission mechanical; doctrine ADR bespoke.
- ADR-0327 (realignment wave promotion gate) — Wave 15-ZD sequenced per the realignment promotion gate.
- ADR-0328 (substance bar canonical sequence + batch discipline) — doctrine PR before Wave 15-ZD implementation.
- ADR-0333 (cell µservice retired — pattern not service) — amended; cell-orchestrator is a logical responsibility composed across tenancy + observability + cloud-iac + audit-chain.
- ADR-0335 (foundry µservice retired) — preserved verbatim.
- ADR-0338 (pod runtime tier declaration) — tier integration: cell tier (ADR-0248) co-varies with pod runtime tier (ADR-0338); auto-rebalance honors pod runtime tier match.
- ADR-0340 (capacity_model per-µservice manifest) — amended; capacity_model is one of the inputs to autosharding.
- ADR-0341 (cellular promotion gates) — amended; auto-rebalance triggers underneath cell-level promotion gates.
- ADR-0342..ADR-0347 — sibling realignment-wave ADRs.

I.3 Spec anchors:

- `/specs/master-plan-sequencing.json` — adds Wave 15-ZD sub-wave entry under `realignment_wave_sequence.waves_15_plus.sub_wave_landings`.
- `/specs/microservices/manifest-schema.json` — admits the `sharding_automation` block schema.
- `/specs/microservices/cell.json` — preserved verbatim; cell catalog continues per ADR-0248.
- `/specs/root-hub-pointers.json` — preserved verbatim.

I.4 Companion-doc anchors:

- `tools/hooks/_canonical-primitives.md` — Lifecycle Skill Map gains a Sharding Automation section under the canonical-primitives cheat sheet pattern.
- `docs/standards/dependency-policy.md` — preserved verbatim (no new dependencies introduced).
- `microservices/tenancy/PRD.md` — gains a cross-reference to this ADR's cell-orchestrator composition section (Wave 15-ZD authoring).
- `microservices/observability/PRD.md` — gains a cross-reference to this ADR's load-skew-detection responsibility (Wave 15-ZD authoring).
- `.omc/state/cellular-exemption-allowlist-2026-05-21.json` — exemption allowlist; authored under this ADR's required-artifact contract; maintained over time as new edge µservices are scaffolded.

## J. Completion Report

<!--
adr: ADR-0348
status: Proposed
date: 2026-05-21
session: 2026-05-21 realignment-wave authoring (sibling to ADR-0340..ADR-0347; doctrine fills within-cell + across-cell TENANT-LEVEL automation backlog deferred from ADR-0248 + ADR-0341)
sibling_adrs: ADR-0340 (capacity model), ADR-0341 (cellular promotion gates), ADR-0342 (API versioning hybrid), ADR-0343 (DR matrix), ADR-0344 (sustainability + finops), ADR-0345 (OSS stewardship class), ADR-0346 (product readiness checklist), ADR-0347 (foundry-fitness → governance rename)
authority_source: feedback_autosharding_dynamic_rebalance_2026_05_21 + ADR-0248 + ADR-0341 + ADR-0333 + ADR-0340
three_automation_modes: autosharding (control_plane_driven) + auto_rebalance (residency + compliance pack honored; Cedar permit for cross-jurisdiction) + dynamic_sharding (hot_split + cold_merge; per-µservice thresholds)
manifest_block_path: microservices/<name>/manifest.json#sharding_automation
manifest_schema_admission_target: specs/microservices/manifest-schema.json
exemption_allowlist_path: .omc/state/cellular-exemption-allowlist-2026-05-21.json
cell_orchestrator_owners: tenancy (placement registry) + observability (load-skew detection) + cloud-iac (cell provisioning + shard catalog) + audit-chain (audit emission) + oya-shuffle-sharding crate (algorithm) + api-gateway (cell-aware routing)
cell_orchestrator_no_new_microservice: ADR-0333 preserved; logical composition across existing µservices
orchestrator_principal: oyatie.foundry.cell-orchestrator (ADR-0247)
audit_chain_emission_contract: ADR-0263 (every automation event emits)
cross_jurisdiction_permit_contract: ADR-0243 (Cedar permit required)
compliance_pack_constraint_contract: ADR-0251 (compliance packs filter candidate cells; PHI cross-pack requires §D-10 BYOK)
residency_contract: ADR-0240 (sovereign cells residency-scoped)
new_lanes: 6 + 1 informational (oya-governance-sharding-automation-coverage, oya-governance-autosharding-manual-mode-refusal, oya-governance-auto-rebalance-residency-honored, oya-governance-dynamic-sharding-threshold-coverage, oya-governance-audit-chain-emit-on-automation-events, oya-governance-tenant-migration-reversibility; informational oya-governance-cell-orchestrator-no-new-microservice)
manifest_field_canonical_shape: sharding_automation.{autosharding, auto_rebalance.{enabled, trigger_load_skew_threshold_percent, honors_residency, honors_compliance_packs, audit_chain_emit}, dynamic_sharding.{enabled, hot_split_threshold_p99_ms, hot_split_utilization_threshold_percent, cold_merge_utilization_threshold_percent, cold_merge_minimum_quiet_hours, audit_chain_emit}}
default_thresholds: hot_split_p99_ms=50, hot_split_utilization=80%, cold_merge_utilization=20%, cold_merge_quiet_hours=24, auto_rebalance_load_skew=30%
default_thresholds_require_per_microservice_declaration: true (E.4 rejects default-fill)
default_autosharding_mode: control_plane_driven (manual rejected by E.2)
sunset_window: 30 days post-Wave-15-ZD-completion for new authoring (E.1..E.6); informational indefinitely for E.7
wave_queue: Wave 15-ZD added to /specs/master-plan-sequencing.json#realignment_wave_sequence.waves_15_plus.sub_waves; cell-orchestrator implementation; sequenced under ADR-0328 batch discipline as multi-µservice coordinator-pattern landing
amendments:
  - ADR-0248 (cellular topology baseline; control-plane-driven tenant→cell/shard placement; automatic hot-cell rebalancing; dynamic shard count)
  - ADR-0341 (cellular promotion gates; tenant-level + shard-level automation layered underneath cell-level promotion gates)
  - ADR-0333 (cell-orchestrator is a logical responsibility composed across tenancy + observability + cloud-iac + audit-chain; NOT a new µservice)
  - ADR-0340 (capacity_model consumed as input to autosharding placement algorithm)
out_of_scope: actual cell-orchestrator Rust crate implementation (deferred to Wave 15-ZD); pipeline-level rebalance for batch/streaming workloads (separate ADR if needed); cross-pack tenant migration with PHI (refused by E.3 + ADR-0251 compliance-pack-aware Cedar policies); Bominal sibling ADR (Bominal authors independently per feedback_bominal_inheritance_precedence)
hyperscaler_precedents: AWS S3 Cell SHIELD migrator + DynamoDB Adaptive Capacity (auto-rebalance + capacity); Google Spanner automatic re-sharding (hot-split + cold-merge); Azure Cosmos automatic re-partitioning (dynamic sharding); Snowflake automatic clustering (autosharding placement)
commits: ADR + specs/microservices/manifest-schema.json sharding_automation admission + .omc/state/cellular-exemption-allowlist-2026-05-21.json scaffold + six lane scaffolds (registry/quality/lanes.yaml + registry/catalog/<lane>.yaml + .github/workflows/<lane>.yml + crates/<lane>-* scaffold) + specs/master-plan-sequencing.json Wave 15-ZD sub-wave entry + tools/hooks/_canonical-primitives.md Sharding Automation section
-->
