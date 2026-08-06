---
id: ADR-0343
title: DR + RTO/RPO matrix per-µservice + per-compliance-pack (effective tenant RTO/RPO = max(µservice declared, all-applicable-pack floors))
status: Rejected
planning_impact: true
date: 2026-05-21
owner_team:
  - council-architecture
  - council-security
  - council-legal
  - ops-sre-reliability
  - ops-compliance
  - axis-observability
owners:
  - council-architecture
  - council-security
  - council-legal
  - ops-sre-reliability
  - ops-compliance
  - axis-observability
supersedes: []
superseded_by: []
amends:
  - ADR-0028-cloud-microservice-architecture.md (per-µservice DR declaration becomes a first-class manifest block, supplementing the existing operational-readiness expectations)
  - ADR-0158-multi-region-active-active.md (per-µservice `multi_region_active_active` declaration becomes a manifest-level commitment with floor enforcement)
  - ADR-0212-buildability-doctrine.md (manifest schema gains a top-level `dr` block; manifest-coverage and buildability gates take the field into the canonical schema)
  - ADR-0241-dr-business-continuity-portfolio-policy.md (the prior four-tier T1..T4 portfolio scheme is preserved as the µservice-declared shape; this ADR adds the per-compliance-pack FLOOR overlay so effective tenant RTO/RPO = max(µservice declared, all applicable pack floors); the manifest `dr` block supersedes the prior `dr_tier` ergonomics by carrying both numeric targets and replication/backup substrate declarations)
  - ADR-0251-compliance-pack-cell-certification-levels.md (compliance packs gain a per-pack RTO/RPO floor that participates in tenant-effective resolution; pack-activation gate validates floor satisfaction)
  - ADR-0263-observability-emission-contract.md (DR-state observability emits the new `dr.rto_p99_seconds_observed` and `dr.rpo_p99_seconds_observed` SLIs per µservice; pack floor satisfaction is a labelled dashboard dimension)
related_adrs:
  - ADR-0028-cloud-microservice-architecture.md
  - ADR-0044-inter-cell-mesh-tunnel.md
  - ADR-0099-data-class-registry.md
  - ADR-0108-deprecation-and-sunset-discipline.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0152-rpo-rto-five-tier-model.md
  - ADR-0158-multi-region-active-active.md
  - ADR-0181-cosign-signed-artifacts-and-modules.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
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
  - ADR-0252-time-coordination-distributed-consistency.md
  - ADR-0253-network-topology-edge-service-mesh.md
  - ADR-0254-deployment-model-spectrum.md
  - ADR-0255-intelligence-as-two-layer-ai-substrate.md
  - ADR-0263-observability-emission-contract.md
  - ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md
  - ADR-0324-anti-script-authoring-doctrine.md
  - ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md
  - ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components.md
  - ADR-0331-cross-microservice-tenant-class-adoption-template.md
  - ADR-0336-valkey-not-redis-substrate.md
  - ADR-0337-iceberg-canonical-olap-write-path.md
  - ADR-0338-pod-runtime-tier-0-to-3.md
  - ADR-0339-shared-iac-module-library.md
  - ADR-0340-capacity-model-per-microservice-manifest.md
  - ADR-0341-cellular-promotion-gates-per-tier.md
  - ADR-0342-api-versioning-hybrid-date-and-semver.md
related_specs:
  - /specs/master-plan-sequencing.json
  - /specs/microservices/manifest-schema.json
  - /specs/compliance-pack-floors.json
  - /specs/compliance-pack-schema.json
  - /specs/dr-business-continuity.json
  - /specs/markdown-retirement-policy.json
related_memory:
  - feedback_six_candidate_adrs_2026_05_21
  - feedback_quality_performance_scalability_bar
  - feedback_no_silent_regression
  - feedback_build_ahead_of_certification
  - feedback_compliance_pack_primitive
  - feedback_amazon_shape_cellular_architecture
  - feedback_tenant_scoping_primitive
  - feedback_bominal_inheritance_precedence
  - feedback_microservice_ownership_coherence_2026_05_20
  - feedback_clean_architecture_requirements
  - feedback_docs_substance_not_scaffold_2026_05_20
  - feedback_drift_too_big_2026_05_20
companion_docs:
  - docs/standards/dr-portfolio.md
  - docs/standards/compliance-pack-floors-catalog.md
  - docs/runbooks/dr-failover.md
  - tools/hooks/_canonical-primitives.md
inbound_citations:
  - /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_six_candidate_adrs_2026_05_21.md
doc_class: Architecture-Decision-Record
shape: Decision
authority_tier: 1
line_floor: 600
bespoke_authoring_requirement: documentation-rigor-1.1-plus-ADR-0322
enforcement_status: advisory-until-manifest-dr-block-lands
enforced_by:
  - oya-check-dr-manifest-block (new lane; advisory until crate lands; planned to refuse missing or malformed `dr` block in per-µservice manifest)
  - oya-check-dr-pack-floor-satisfaction (new lane; advisory until crate lands; planned to compute effective RTO/RPO per tenant-pack combination and refuse µservice declarations that fail to satisfy any applicable pack floor on cells they may serve)
  - oya-check-dr-multi-region-required (new lane; advisory until crate lands; planned to refuse `multi_region_active_active=false` when µservice serves a pack that mandates multi-region — HIPAA, KR-PIPA via CSAP, EU AI Act high-risk)
  - oya-check-dr-runbook-presence (new lane; advisory until crate lands; planned to refuse a `failover_runbook` pointer that does not resolve to an existing markdown file in the µservice's `runbooks/` directory)
  - oya-check-dr-backup-substrate-allowlist (new lane; advisory until crate lands; planned to refuse backup substrate identifiers outside the canonical allowlist defined in §D-5)
  - oya-check-dr-drill-evidence-fresh (new lane; advisory until crate lands; planned to refuse µservices whose `last_drill_evidence_id` resolves to an audit row older than the pack-mandated drill cadence)
  - oya-check-dr-pack-floor-table-coverage (new lane; advisory until crate lands; planned to refuse `/specs/compliance-pack-floors.json` versions that omit a pack referenced by any active tenant)
  - oya-governance-dr-auditor-dashboard-presence (new lane; refuses missing per-pack auditor dashboard manifest entries that enumerate which µservices serve the pack and report floor satisfaction)
purpose: >
  Establish the canonical disaster-recovery (DR) and business-continuity
  RTO/RPO matrix at two coupled layers: (1) per-µservice declared targets
  captured in the µservice's `manifest.json#dr` block (numeric
  `rto_p99_seconds` + `rpo_p99_seconds` + `multi_region_active_active`
  + `backup_substrate` + `failover_runbook`) and (2) per-compliance-pack
  RTO/RPO floors captured in the new spec `/specs/compliance-pack-floors.json`
  consumed by the pack-activation gate. Effective tenant RTO/RPO is the
  pointwise MAXIMUM of the µservice declared target and every applicable
  pack floor (i.e., effective is more stringent than either input alone).
  Eight initial pack floors are codified (HIPAA, PCI-DSS, SOC2 Type II,
  EU AI Act Annex III high-risk, CSAP-Korea, ISO 27001, SOX 404, KR PIPA),
  with explicit semantics for SOX-per-process and KR-PIPA-per-data-class.
  Per-µservice manifest field updates are out of scope for this ADR;
  the corpus-wide manifest sub-wave is sequenced separately under ADR-0328.
  Author the ADR, queue the sub-wave in `/specs/master-plan-sequencing.json`,
  extend `/specs/microservices/manifest-schema.json` with the `dr` block,
  and create the new spec file `/specs/compliance-pack-floors.json`
  containing the per-pack floor table consumed by the pack-activation gate.
  Per-pack auditor dashboard surface (which µservices serving that pack
  meet floor) lands as a downstream observability lane under ADR-0263.
---

# ADR-0343: DR + RTO/RPO matrix per-µservice + per-compliance-pack (effective tenant RTO/RPO = max(µservice declared, all-applicable-pack floors))

## Status

Proposed on 2026-05-21.

This ADR is the canonical DR substrate decision establishing the two-layer RTO/RPO matrix that governs every Oyatie µservice and every tenant-activated compliance pack. The two layers — per-µservice declared (`manifest.json#dr`) and per-compliance-pack floor (`/specs/compliance-pack-floors.json`) — combine pointwise via MAXIMUM (more-stringent-wins) to produce the effective tenant DR contract. The combination is deterministic, machine-evaluable, and auditor-visible.

It runs in coordination with the in-flight 2026-05-21 realignment effort: ADR-0340 (capacity model per µservice manifest), ADR-0341 (cellular promotion gates per tier), ADR-0342 (API versioning hybrid model), ADR-0344 (sustainability + finops dimensional model), and ADR-0345 (talent + OSS contribution policy) are sibling decisions from the same `/idea-refine` session captured in `feedback_six_candidate_adrs_2026_05_21.md`; this ADR is the fourth in that hexad.

It directly amends ADR-0241 (DR business continuity portfolio) so that the prior T1..T4 tier shorthand is preserved at the µservice declaration layer while the new per-pack floor layer is added beneath the effective resolution. It directly amends ADR-0251 (compliance pack cell certification levels) so that each pack ships a per-pack `dr_floor` field consumed by the pack-activation gate. It directly amends ADR-0212 (buildability doctrine) so that the per-µservice manifest schema includes a top-level `dr` block as a canonical declaration target. It is binding on every µservice that declares a workload and on every compliance pack that is or becomes tenant-activatable.

Enforcement transitions from `advisory-until-manifest-dr-block-lands` to `BLOCKER` per the lane sequence in §E below: at landing of the Wave 15-DR-Matrix sub-wave (which carries the per-µservice manifest declaration into the corpus), the `oya-check-dr-manifest-block` and `oya-check-dr-pack-floor-satisfaction` lanes promote to BLOCKER per the §G sunset schedule.

DR-001 first-slice review-visible artifacts are `scripts/tests/OWNERS`, `scripts/tests/dr_001_rto_rpo_matrix_slice_check.py`, and `specs/fixtures/dr-rto-rpo-matrix/dr-001-dashboard-manifest.fixture.json`. They are contract/fixture/local-bridge evidence only: they may demonstrate the manifest `dr` block, compliance-pack floor table, effective RTO/RPO resolution, failover-runbook pointer, drill-evidence freshness, and dashboard row shape, but they do not claim runtime DR execution, pack-activation runtime enforcement, auditor acceptance, tenant workload readiness, or production readiness. The successor enforcement path remains the native Rust/Buck2 cloud-ci DR gate family listed in `enforced_by`.

This ADR does not delete the four-tier T1..T4 shorthand used in ADR-0241 / `/specs/dr-business-continuity.json`. The shorthand is preserved as the per-µservice ergonomic tier-name selector; the numeric values inside the manifest `dr` block remain the source of truth. The shorthand and the numeric block are reconciled at admission per §D-8.

This ADR does not change the cellular tier classification from ADR-0248. The cellular tier governs blast-radius scope per cell; the DR matrix governs RTO/RPO targets per µservice and per pack. The two axes co-vary but are independent decision surfaces.

This ADR does not change tenant_class behavior from ADR-0330. demo_trial and paid tenants share the same per-µservice DR declaration; pack-activation is the demo_trial-vs-paid divergence point (demo_trial tenants typically activate only the `OPEN-INTERNAL-BASELINE` pack, paid tenants activate any pack from their entitlement scope).

This ADR does not change Cedar evaluation from ADR-0243. DR decisions are admission-time and observability-time concerns; Cedar gates remain authorization. The pack-activation gate is the Cedar fragment that enforces floor satisfaction at admission, not authorization.

## Date

2026-05-21.

## Context

### A.1 Named pressure: ADR-0241 portfolio is one-dimensional; compliance reality is two-dimensional

ADR-0241 established the canonical four-tier DR portfolio: T1 (< 5 min RTO, 0 RPO, active-active multi-AZ cross-region warm), T2 (60 min RTO, 60 s RPO, active-passive cross-region continuous), T3 (240 min RTO, 900 s RPO, backup-restore cross-region warm), T4 (1440 min RTO, 3600 s RPO, backup-restore cold). The portfolio is declared per µservice via `manifest.json#dr_tier`. It is sufficient when a µservice has a single DR contract that holds across every tenant context the µservice serves.

The two-dimensional reality is: a µservice's effective DR contract for a tenant depends on the compliance packs the tenant has activated. A µservice declared at T3 (240 min RTO) is correct for a tenant in the `OPEN-INTERNAL-BASELINE` pack. The same µservice, serving a tenant in the `HIPAA-2024` pack, is non-compliant — HIPAA's contingency-plan controls (§164.308(a)(7), §164.310(a)(2)(i)) impose a tight RTO/RPO discipline for systems handling PHI. The µservice cannot satisfy HIPAA at T3 without elevating its effective contract.

The naïve fix — declaring the µservice at T1 to cover every pack — is wrong on two axes. First, T1 imposes 1-2 orders-of-magnitude cost vs T3 (active-active multi-region warm vs backup-restore warm), and a µservice that serves predominantly non-HIPAA tenants is paying for a contract its baseline tenants do not need. Second, T1 is still not strict enough for the EU AI Act high-risk regime, which mandates RTO ≤ 30 min and RPO ≤ 5 min for systems whose unavailability creates a fundamental-rights impact; the four-tier portfolio doesn't even contain a tier that strict.

The two-dimensional layering this ADR introduces resolves both axes: a µservice declares its baseline DR contract once (`manifest.json#dr`), and each compliance pack declares its floor once (`/specs/compliance-pack-floors.json`). The effective contract per tenant is computed as the pointwise MAX across the µservice declaration and every applicable pack floor. The µservice does not pay the T1 cost for non-HIPAA tenants; it pays only when an activated pack pushes the floor above its declared target. The strictness of the EU AI Act high-risk pack is captured as a pack floor without polluting the µservice's baseline contract.

### A.2 Named pressure: ADR-0251 packs lack an explicit DR floor

ADR-0251 (compliance pack cell certification levels) and `/specs/compliance-pack-schema.json` codify per-pack Cedar fragments, audit-chain requirements, data-class extensions, cell-eligibility constraints, retention rules, consent requirements, cross-tenant rules, jurisdiction overlay, DPIA template references, and breach-notification workflow references. The schema does not currently encode the pack's RTO/RPO floor. As a result, the per-pack RTO/RPO posture is **implicit** — it lives in regulator citations (HIPAA Contingency Plan §164.308(a)(7); PCI-DSS v4.0 §12.10.1 incident response timing; SOC2 Type II Common Criteria CC7.5 "system recovery"; EU AI Act Article 17 "quality management system continuity"; CSAP-Korea §3.5.1 BCP/DR; ISO 27001 Annex A.5.30 "ICT readiness for business continuity"; SOX 404 ICFR §404 audit-evidence availability; KR PIPA Article 29 "safeguards for personal information").

Implicit floors fail in three ways. First, auditors cannot machine-evaluate floor satisfaction; they must hand-walk every µservice that touches the pack's data class to verify its DR contract is strict enough. Second, the platform cannot refuse admission at pack-activation time when a µservice's declared contract is below the pack's mandated floor; the breach surface is detected post-facto. Third, the floor is restated in prose across regulator-specific documentation rather than being a single machine-readable artifact, which guarantees inconsistency over time as regulators update their requirements.

This ADR resolves all three by codifying the per-pack floor in `/specs/compliance-pack-floors.json`, signed and versioned, consumed by the pack-activation gate at admission. The floor table is auditable on its own (a regulator can review the table without inspecting any µservice) and combinable with µservice declarations on demand (the platform computes effective contract per tenant via the §D-4 algorithm).

### A.3 Named pressure: per-data-class regimes (SOX 404, KR PIPA) need per-data-class floor refinement

Two of the eight initial packs have per-data-class or per-process semantics that resist a single per-pack floor:

- **SOX 404.** Sarbanes-Oxley §404 (Management Assessment of Internal Controls) imposes audit-evidence-availability requirements per-process. A SOX-in-scope general ledger journal-entry process has a sharper RTO/RPO than a SOX-in-scope vendor-master-data process, which in turn is sharper than a SOX-in-scope quarterly close working-paper archive. The pack's overall floor is the strictest of the in-scope processes; per-process refinement allows µservices to satisfy each process's specific floor without over-paying.
- **KR PIPA.** Personal Information Protection Act §29 imposes safeguards per personal-information data class. PIPA's `PI_KR_RESIDENT_REGISTRATION_NUMBER` (RRN, 주민등록번호) handling has a tighter availability + tamper-resistance posture than `PI_KR_PIPA` general personal information (because RRN compromise has criminal-class regulator response under §59). Per-data-class refinement allows µservices that touch only general personal information to declare a more lenient contract than µservices that touch RRN.

This ADR encodes per-data-class floor refinement for SOX 404 and KR PIPA as `process_floors` and `data_class_floors` sub-tables under those two pack entries (see §D-3.7 and §D-3.8). Other packs (HIPAA, PCI-DSS, SOC2, EU AI Act, CSAP, ISO 27001) have a single per-pack floor.

### A.4 Named pressure: multi-region-required is binary and pack-driven

HIPAA Contingency Plan §164.308(a)(7)(ii)(B) requires emergency-mode operation with a documented plan for sustaining critical business processes during major-region failure. The customary AWS / Azure / GCP interpretation is multi-region active-active or active-passive with ≤ 1h RTO; EU AI Act Annex III high-risk imposes the same; CSAP-Korea §3.5 ICT-BCP imposes a multi-region-or-cold-site-with-tested-failover posture.

PCI-DSS v4.0 does **not** impose multi-region (it imposes RTO/RPO via business-continuity policy under §12.10 but does not mandate multi-region topology); SOC2 Type II Common Criteria CC7.5 does **not** impose multi-region (system recovery is the audit objective, not the topology). ISO 27001 Annex A.5.30 does **not** impose multi-region (ICT readiness is the audit objective, not the topology). SOX 404 does **not** impose multi-region. KR PIPA does **not** impose multi-region as such; CSAP-Korea (which often combines with KR PIPA in Korean enterprise contexts) does.

The two-tier multi-region rule extracted from these regulators:

- **multi-region MANDATORY**: HIPAA, EU AI Act Annex III high-risk, CSAP-Korea.
- **multi-region OPTIONAL**: PCI-DSS, SOC2 Type II, ISO 27001, SOX 404, KR PIPA.

This ADR encodes multi-region as a pack-floor field (`multi_region_required` boolean) that defaults to `false` for packs that don't mandate it and is `true` for packs that do. The effective tenant `multi_region_active_active` setting is the OR (logical inclusive OR) of the µservice's declared `multi_region_active_active` flag and every applicable pack's `multi_region_required` flag. Once any applicable pack requires multi-region, the effective setting is `true` regardless of the µservice's declaration.

### A.5 Named pressure: failover-runbook presence is the operational expression of DR

A DR contract without an exercised runbook is paper. Per ADR-0241 §D.6 and `/specs/dr-business-continuity.json#manifest_block.last_drill_evidence_id`, every µservice is expected to ship a `failover_runbook` markdown file and a `last_drill_evidence_id` pointing to an audit row that demonstrates the runbook was exercised within the tier-mandated drill cadence (quarterly for T1/T2, semi-annual for T3/T4).

This ADR carries the runbook pointer into the new `manifest.json#dr` block (`failover_runbook` field, required, MUST resolve to a markdown file path inside the µservice's `runbooks/` directory). The pack floor adds an orthogonal cadence constraint: HIPAA pack mandates quarterly drill cadence; EU AI Act high-risk mandates quarterly; SOC2 Type II mandates annual; PCI-DSS v4.0 §12.10.2 mandates annual incident-response test; CSAP-Korea mandates semi-annual; KR PIPA does not mandate drill cadence specifically (it is inherited from the µservice's declared tier).

The effective drill cadence is the SHORTEST cadence across the µservice's tier-default and every applicable pack's `drill_cadence_required` (i.e., the most-frequent cadence wins).

### A.6 Named pressure: backup substrate allowlist constrains the durable persistence surface

ADR-0336 (Valkey not Redis substrate) and ADR-0337 (Iceberg canonical OLAP write path) establish the canonical persistence substrate for in-memory KV and OLAP table format respectively. PostgreSQL with WAL-G continuous archive is the canonical OLTP substrate. Object storage with versioning (S3 + object-lock; OCI Object Storage with retention; MinIO on-prem with versioning) is the canonical immutable-snapshot substrate. SeaweedFS is the canonical large-scale object substrate per ADR-0196.

A µservice's `backup_substrate` field declares which of these substrates the µservice depends on for DR-time recovery. The allowlist (planned to be enforced by `oya-check-dr-backup-substrate-allowlist`, advisory until the crate lands per §D-5) is:

- `valkey` (in-memory KV; backup via Valkey AOF + RDB to object storage)
- `valkey_cluster` (in-memory KV cluster; backup via per-shard AOF + RDB)
- `postgres_wal_g` (OLTP; continuous WAL archive to object storage via WAL-G or equivalent)
- `iceberg_snapshot` (OLAP table format; per-snapshot rollback per ADR-0337)
- `object_storage_versioned` (object storage with versioning + object-lock for tamper-evidence)
- `seaweedfs_replicated` (SeaweedFS with cross-cell replication per ADR-0196)
- `milvus_snapshot` (vector store snapshots to object storage)
- `clickhouse_iceberg_layered` (ClickHouse compute layered on Iceberg per ADR-0337)
- `openbao_seal_unseal` (OpenBao secrets with multi-region seal/unseal)
- `audit_chain_merkle_seal` (audit-chain Merkle seal to object storage; sovereign-pack tamper-evidence)

Any substrate identifier outside the allowlist is refused by the lane. The allowlist is versioned alongside this ADR and evolves under ADR-0108 sunset discipline.

### A.7 Named pressure: per-pack auditor dashboard demands floor-satisfaction visibility per µservice

Per `feedback_quality_performance_scalability_bar`, Oyatie operates at hyperscaler-grade rigor. Hyperscaler precedent for DR visibility:

- **AWS.** AWS publishes per-service RTO/RPO contracts in the Service Health Dashboard and per-account DR posture in AWS Resilience Hub assessments. Regulator-mapped views (HIPAA Eligible Services, PCI-DSS in-scope services) are surfaced via AWS Audit Manager.
- **Google Cloud.** GCP publishes per-service availability contracts via Cloud Status Dashboard and per-product DR posture via Cloud Operations. Compliance views are surfaced via the Compliance Reports Manager.
- **Microsoft Azure.** Azure publishes per-service SLA / DR contracts via Service Health and per-subscription DR posture via Site Recovery. Compliance views are surfaced via Microsoft Purview Compliance Manager.

The Oyatie equivalent is the per-pack auditor dashboard: for each tenant-activatable pack, a dashboard surfaces every µservice that may serve that pack and shows (a) the µservice's declared `manifest.json#dr.rto_p99_seconds` / `rpo_p99_seconds`, (b) the pack's `dr_floor.rto_p99_seconds` / `rpo_p99_seconds`, (c) the effective max, (d) the µservice's most-recent drill evidence, and (e) the floor-satisfaction status (green = effective satisfies floor; red = floor cannot be satisfied with current declaration).

The dashboard manifest is `microservices/compliance/dashboards/per-pack-dr-floor-satisfaction.yaml` and is auto-generated from `/specs/compliance-pack-floors.json` × per-µservice manifests. The presence of the dashboard manifest is enforced by `oya-governance-dr-auditor-dashboard-presence`.

### A.8 Anchors this ADR binds

- Anchor 1: the user directive of 2026-05-21 captured in `feedback_six_candidate_adrs_2026_05_21.md` ADR-0343 section — "Per-µservice manifest.json declares dr.rto_p99_seconds / dr.rpo_p99_seconds / dr.multi_region_active_active / dr.backup_substrate / dr.failover_runbook; per compliance pack sets the FLOOR; effective tenant RTO/RPO = max(µservice declared, all-applicable-pack floors); HIPAA RTO ≤ 1h RPO ≤ 5min multi-region required; PCI-DSS RTO ≤ 24h RPO ≤ 1h; SOC2 RTO ≤ 4h RPO ≤ 15min; EU AI Act high-risk RTO ≤ 30min RPO ≤ 5min; CSAP RTO ≤ 1h RPO ≤ 15min; ISO 27001 RTO ≤ 4h RPO ≤ 1h; SOX 404 per-process; KR PIPA per-data-class; auditor view per-pack dashboard shows which µservices meet floor".
- Anchor 2: ADR-0241 (DR business continuity portfolio). The four-tier T1..T4 portfolio is preserved as the µservice-declared shape; this ADR adds the per-pack floor overlay.
- Anchor 3: ADR-0251 (compliance pack cell certification levels). Each pack gains a `dr_floor` field consumed by the pack-activation gate.
- Anchor 4: ADR-0212 (buildability doctrine). The per-µservice manifest schema gains a top-level `dr` block.
- Anchor 5: ADR-0263 (observability emission contract). DR-state observability emits `dr.rto_p99_seconds_observed`, `dr.rpo_p99_seconds_observed`, `dr.failover_runbook_executed_at`, `dr.last_drill_evidence_id` SLIs per µservice.
- Anchor 6: ADR-0158 (multi-region active-active). The `multi_region_active_active` declaration becomes a manifest-level commitment with floor enforcement.
- Anchor 7: ADR-0244 (tenant scoping). Effective DR is resolved per (tenant, µservice) pair using the tenant's activated pack set.
- Anchor 8: ADR-0250 (build ahead of certification). The DR posture for every pack is established before tenant onboarding, not retrofitted.
- Anchor 9: ADR-0247 (self-modification doctrine). `oyatie.foundry.*` workflow execution lands on substrate µservices that participate in their own DR contracts; recursive coverage applies.
- Anchor 10: ADR-0248 (cellular architecture). DR primitives live inside the cell boundary; cross-cell failover uses the inter-cell mesh tunnel per ADR-0044.
- Anchor 11: ADR-0150 (Cedar policy engine). The pack-activation gate is a Cedar fragment per pack that checks DR floor satisfaction at admission.
- Anchor 12: `feedback_quality_performance_scalability_bar`. Hyperscaler-grade rigor demands per-pack DR-floor satisfaction is machine-evaluable and auditor-visible.

### A.9 What this ADR does not assert

- **A.9.1** Does not author the per-µservice manifest `dr` block content. The corpus-wide manifest sub-wave is sequenced separately as `15W-DR-Matrix-declaration` under ADR-0328 batch discipline.
- **A.9.2** Does not author the per-pack auditor dashboard wiring. The dashboard lands as a downstream observability lane under ADR-0263.
- **A.9.3** Does not retire the four-tier T1..T4 shorthand. The shorthand is preserved at the µservice declaration layer for ergonomics; the numeric fields inside the `dr` block remain the source of truth.
- **A.9.4** Does not change the cellular tier classification from ADR-0248. The cellular tier governs blast-radius; the DR matrix governs RTO/RPO.
- **A.9.5** Does not change the data-class registry from ADR-0099. The per-data-class refinement under KR PIPA references data-class identifiers from the canonical taxonomy.
- **A.9.6** Does not introduce new compliance packs. The eight initial floors codify packs that already exist in `/specs/compliance-pack-schema.json` examples.
- **A.9.7** Does not change pack-activation semantics from ADR-0251. The pack-activation gate gains a floor-check step; the activation flow is preserved.
- **A.9.8** Does not change tenant_class semantics from ADR-0330. demo_trial and paid tenants resolve effective DR via the same algorithm; demo_trial tenants typically activate fewer packs.
- **A.9.9** Does not change incident-response classification from ADR-0338. Pod runtime tier and DR tier are independent axes.
- **A.9.10** Does not constrain which cells host DR-tested workloads. Cell placement is a separate decision under ADR-0240 + ADR-0248.

## Decision

### B.1 Decision statement

Every Oyatie µservice that produces a workload declares a top-level `dr` block in its `microservices/<name>/manifest.json` carrying five required fields: `rto_p99_seconds` (integer ≥ 0), `rpo_p99_seconds` (integer ≥ 0), `multi_region_active_active` (boolean), `backup_substrate` (array of allowlisted substrate identifiers per §D-5), and `failover_runbook` (string path to a markdown file resolvable inside the µservice's `runbooks/` directory). Two optional fields are permitted: `dr_tier` (the T1..T4 ergonomic selector from ADR-0241; reconciled to the numeric fields per §D-8) and `last_drill_evidence_id` (per ADR-0241 / `/specs/dr-business-continuity.json#manifest_block`).

Every tenant-activatable compliance pack declares a `dr_floor` sub-block in `/specs/compliance-pack-floors.json` carrying four required fields: `rto_p99_seconds` (integer ≥ 0), `rpo_p99_seconds` (integer ≥ 0), `multi_region_required` (boolean), `drill_cadence_required` (string enum from `/specs/dr-business-continuity.json#tiers[*].drill_cadence`). Two optional refinement sub-tables are permitted: `process_floors` (for SOX 404) and `data_class_floors` (for KR PIPA).

The effective tenant DR contract per (tenant, µservice) pair is computed by the algorithm in §D-4:

- Let `M.rto = manifest.dr.rto_p99_seconds`, `M.rpo = manifest.dr.rpo_p99_seconds`, `M.mr = manifest.dr.multi_region_active_active`.
- Let `P_i.rto`, `P_i.rpo`, `P_i.mr_required`, `P_i.drill` denote the floor and cadence for each pack `P_i` activated by the tenant and applicable to the µservice (i.e., pack's `applies_to_microservices` intersection is non-empty per `/specs/compliance-pack-schema.json#cedar_fragments[*].applies_to_microservices`).
- Effective `rto_p99_seconds = min(M.rto, min over i of P_i.rto)` — the **MIN** (because RTO floors are upper bounds — a stricter floor is a smaller number).
- Effective `rpo_p99_seconds = min(M.rpo, min over i of P_i.rpo)` — the **MIN** (same reason).
- Effective `multi_region_active_active = M.mr OR (OR over i of P_i.mr_required)`.
- Effective `drill_cadence = shortest cadence among µservice's tier-default and all P_i.drill`.

The "max of µservice declared, all-applicable-pack floors" phrasing in the authority memory refers to the stringency comparison (more-stringent wins); the numeric realization is MIN over upper-bound seconds and OR over the multi-region boolean. See §D-4 for the precise algorithm and worked examples.

The eight initial pack floors (§D-3) cover HIPAA, PCI-DSS, SOC2 Type II, EU AI Act Annex III high-risk, CSAP-Korea, ISO 27001, SOX 404 (with per-process refinement), and KR PIPA (with per-data-class refinement).

### B.2 Numbered decision clauses

B2.001. `microservices/<name>/manifest.json` declares a top-level `dr` block.

B2.002. The `dr` block requires `rto_p99_seconds`, `rpo_p99_seconds`, `multi_region_active_active`, `backup_substrate`, `failover_runbook`.

B2.003. The `dr` block accepts the optional fields `dr_tier ∈ {T1, T2, T3, T4}` and `last_drill_evidence_id`.

B2.004. `rto_p99_seconds` is the µservice's declared p99 recovery-time-objective in whole seconds; integer ≥ 0.

B2.005. `rpo_p99_seconds` is the µservice's declared p99 recovery-point-objective in whole seconds; integer ≥ 0; 0 means "no data loss tolerated" (active-active or synchronous replication).

B2.006. `multi_region_active_active` is the µservice's declared multi-region active-active topology flag; true ⇒ active-active across ≥ 2 regions; false ⇒ active-passive, single-region, or backup-restore.

B2.007. `backup_substrate` is an array of substrate identifiers drawn from the allowlist in §D-5; minimum cardinality 1.

B2.008. `failover_runbook` is a string path relative to the µservice root pointing to a markdown file in the `runbooks/` directory; MUST resolve at admission.

B2.009. When `dr_tier` is declared, the numeric fields MUST satisfy the tier's numeric floor per §D-8 reconciliation table; a mismatch is refused by `oya-check-dr-manifest-block`.

B2.010. When `last_drill_evidence_id` is declared, it MUST resolve to an audit row in the audit-chain whose age is within the pack-mandated cadence per §D-6.

B2.011. `/specs/compliance-pack-floors.json` is the canonical machine-readable per-pack floor table.

B2.012. Each pack entry in the floor table declares `pack_id` (matching `/specs/compliance-pack-schema.json#pack_id`), `version`, and `dr_floor`.

B2.013. The `dr_floor` block requires `rto_p99_seconds`, `rpo_p99_seconds`, `multi_region_required`, `drill_cadence_required`.

B2.014. The eight initial packs in the floor table are HIPAA-2024, PCI-DSS-L1-v4, SOC2-T2, EU-AI-ACT-2024-HIGH-RISK, KR-CSAP-v3.1, ISO27001-2022, SOX-404, KR-PIPA-2023-amendment per §D-3.

B2.015. HIPAA-2024 floor is RTO ≤ 3600s (1h), RPO ≤ 300s (5min), multi-region REQUIRED, drill quarterly.

B2.016. PCI-DSS-L1-v4 floor is RTO ≤ 86400s (24h), RPO ≤ 3600s (1h), multi-region NOT required, drill annual (per PCI-DSS §12.10.2).

B2.017. SOC2-T2 floor is RTO ≤ 14400s (4h), RPO ≤ 900s (15min), multi-region NOT required, drill annual.

B2.018. EU-AI-ACT-2024-HIGH-RISK floor is RTO ≤ 1800s (30min), RPO ≤ 300s (5min), multi-region REQUIRED, drill quarterly.

B2.019. KR-CSAP-v3.1 floor is RTO ≤ 3600s (1h), RPO ≤ 900s (15min), multi-region REQUIRED, drill semi-annual.

B2.020. ISO27001-2022 floor is RTO ≤ 14400s (4h), RPO ≤ 3600s (1h), multi-region NOT required, drill annual.

B2.021. SOX-404 floor declares `process_floors` per in-scope SOX process; default fallback floor is RTO ≤ 14400s (4h), RPO ≤ 3600s (1h), multi-region NOT required, drill annual.

B2.022. KR-PIPA-2023-amendment floor declares `data_class_floors` per personal-information data class; default fallback floor is RTO ≤ 14400s (4h), RPO ≤ 900s (15min), multi-region NOT required, drill semi-annual; RRN data class refinement is RTO ≤ 3600s (1h), RPO ≤ 300s (5min), multi-region REQUIRED, drill quarterly.

B2.023. The effective tenant DR contract per (tenant, µservice) is computed per §D-4 algorithm.

B2.024. Effective `rto_p99_seconds` and `rpo_p99_seconds` take the **MIN** across the µservice declaration and applicable pack floors (more-stringent wins; smaller seconds wins).

B2.025. Effective `multi_region_active_active` takes the **OR** across the µservice declaration and applicable pack `multi_region_required` flags.

B2.026. Effective `drill_cadence` takes the SHORTEST cadence (most-frequent wins) across the µservice tier-default cadence and applicable pack `drill_cadence_required`.

B2.027. The pack-activation gate refuses tenant pack activation when the (tenant, µservice) pair fails effective-floor resolution (i.e., the µservice's declared `rto_p99_seconds > pack.dr_floor.rto_p99_seconds` or analogous RPO mismatch or multi-region mismatch).

B2.028. The pack-activation gate is implemented as a Cedar fragment named `pack-<pack-id>-dr-floor-satisfaction` and signed per ADR-0181 cosign discipline.

B2.029. The pack-activation gate Cedar fragment is loaded under scope `pack/<pack-id>` per ADR-0251 fragment loading and consumed at tenant pack-activation admission.

B2.030. The auditor dashboard manifest at `microservices/compliance/dashboards/per-pack-dr-floor-satisfaction.yaml` is auto-generated from `/specs/compliance-pack-floors.json` × per-µservice manifest declarations; the dashboard MUST be regenerated on every manifest change.

B2.031. The dashboard surfaces, per pack, the list of µservices serving the pack with their declared / pack-floor / effective DR contract triple and a green/red floor-satisfaction status.

B2.032. The dashboard MUST be queryable by auditors at admission, during drill review, and during incident retrospective.

B2.033. The CI lane `oya-check-dr-manifest-block` validates: (a) `dr` block presence; (b) required fields well-typed and within numeric bounds; (c) `backup_substrate` entries within the allowlist; (d) `failover_runbook` path resolves to an existing markdown file; (e) optional `dr_tier` reconciles with numeric fields per §D-8.

B2.034. The CI lane `oya-check-dr-pack-floor-satisfaction` computes effective per (tenant-class, µservice) and refuses any pair whose effective resolution would fail floor satisfaction for any pack activatable by that tenant-class.

B2.035. The CI lane `oya-check-dr-multi-region-required` refuses `multi_region_active_active=false` for any µservice that serves a pack with `multi_region_required=true` per pack `applies_to_microservices`.

B2.036. The CI lane `oya-check-dr-runbook-presence` refuses `failover_runbook` pointer that fails to resolve to an existing markdown file.

B2.037. The CI lane `oya-check-dr-backup-substrate-allowlist` refuses substrate identifiers outside the §D-5 allowlist.

B2.038. The CI lane `oya-check-dr-drill-evidence-fresh` refuses µservices whose `last_drill_evidence_id` audit row is older than the effective drill cadence.

B2.039. The CI lane `oya-check-dr-pack-floor-table-coverage` refuses `/specs/compliance-pack-floors.json` versions that omit a pack referenced by any active tenant per `/specs/compliance-pack-schema.json` registry.

B2.040. The CI lane `oya-governance-dr-auditor-dashboard-presence` refuses missing or stale auditor dashboard entries.

B2.041. The new CI lanes (§E) are REPORT-ONLY at landing and promote to BLOCKER per the §G sunset schedule.

B2.042. The corpus-wide per-µservice manifest update is the `15W-DR-Matrix-declaration` sub-wave queued under ADR-0328 batch discipline.

B2.043. The per-pack auditor dashboard wiring is sequenced as a downstream observability lane under ADR-0263; not authored by this ADR.

B2.044. The pack floor table at `/specs/compliance-pack-floors.json` is authored by this ADR (§D-3 below) with the eight initial pack floors.

B2.045. The `dr_tier` shorthand from ADR-0241 is preserved at the µservice declaration layer; reconciliation per §D-8.

B2.046. The four-tier T1..T4 portfolio numeric defaults from `/specs/dr-business-continuity.json#tiers` remain canonical for tier-shorthand resolution and are referenced verbatim by §D-8.

B2.047. The `replication_shape` enumeration from `/specs/dr-business-continuity.json` is preserved as an optional `replication_shape` field inside the `dr` block; not required.

B2.048. ADR-0241 §C.2 "Mixed-mode is forbidden" rule (µservices may not declare two tiers per environment) is preserved verbatim.

B2.049. ADR-0241 §C.3 "Pack overlay can elevate, never relax" rule is restated here as the MIN-over-stringency algorithm in §D-4.

B2.050. Multispectrum review v2.4.0 applies to this ADR per ADR-0322 §D-2. The review evidence is at `evidence/debate/ADR-0343/` after this ADR lands in a review-track PR.

B2.051. The 30-day sunset window starts on Acceptance. The eight new lanes (§E) promote from REPORT-ONLY to BLOCKER at day 30; per-µservice migration of existing `dr_tier`-only declarations to the full `dr` block follows the per-µservice canonical-build phase order under ADR-0328.

B2.052. The ADR is final on Acceptance. No exception clause is provided. A µservice that cannot meet the pack floor MUST decline tenant pack activation at the pack-activation gate.

B2.053. The ADR's enforcement and sunset run in coordination with ADR-0341 (cellular promotion gates) — promotion to Tier 0 / Tier 1 / Tier 2 cells per ADR-0341 requires DR floor satisfaction per this ADR for every pack the cell hosts.

B2.054. The realignment_wave_sequence in `specs/master-plan-sequencing.json` adds the new sub-wave `15W-DR-Matrix-declaration` queued for dispatch after this ADR lands; the sub-wave is per-µservice bespoke authoring under ADR-0322 substance-bar + ADR-0324 anti-template discipline.

B2.055. The canonical-primitives cheat sheet at `tools/hooks/_canonical-primitives.md` adds a DR matrix section naming this ADR, the manifest block, and the per-pack floor spec.

### B.3 What this decision does not do

- This ADR does not author per-µservice manifest updates; the corpus-wide declaration sub-wave handles that.
- This ADR does not author the auditor dashboard wiring; that lands under ADR-0263.
- This ADR does not change the cellular tier topology from ADR-0248.
- This ADR does not change the four-tier T1..T4 ergonomic shorthand from ADR-0241; that shorthand is preserved and reconciled to the numeric fields per §D-8.
- This ADR does not introduce new compliance packs; the eight initial packs already exist in `/specs/compliance-pack-schema.json` examples.

## Consequences

### C.1 Positive consequences

- **Two-dimensional DR matrix.** µservices declare baseline DR contract once; packs declare floor once; effective contract per tenant is the deterministic combination. The 77 × 8 = 616 (µservice, pack) pair surface collapses to 77 + 8 declarations + 1 algorithm.
- **Pack-activation gate enforces floor at admission.** Tenant pack activation that would push effective DR below the µservice's capacity is refused at admission, not detected post-facto.
- **Auditor dashboard per pack.** Regulators see per-pack µservice coverage with green / red floor-satisfaction status at a glance. Audit-cycle time drops from per-µservice hand-walks to a single dashboard query.
- **Build-ahead-of-certification.** Per ADR-0250, the DR posture for every pack is established before tenant onboarding; this ADR makes the floor machine-readable so the build-ahead bar is testable.
- **Cost discipline.** µservices declare their actual baseline DR contract; they pay T1 cost only when an activated pack pushes them above their baseline. The 1-2 OOM cost difference between T3 and T1 is preserved for non-HIPAA tenants.
- **Per-process and per-data-class refinement.** SOX 404 and KR PIPA escape the single-floor-per-pack straitjacket; µservices that touch only general data classes pay general-floor cost, not RRN-floor or critical-process-floor cost.
- **Multi-region binary captured as pack floor.** The multi-region requirement is encoded once per pack (HIPAA / EU AI Act / CSAP require; PCI / SOC2 / ISO / SOX / PIPA optional) rather than restated per µservice.
- **Backup substrate allowlist.** The 10-substrate allowlist constrains the durable persistence surface to the canonical Oyatie substrates (Valkey, PostgreSQL+WAL-G, Iceberg, object storage, SeaweedFS, Milvus, ClickHouse on Iceberg, OpenBao, audit-chain Merkle). Drift to unsupported substrates is refused at admission.
- **Runbook presence enforced.** A µservice cannot ship a DR contract without a runbook; runbook absence is a P0 admission refusal.
- **Drill cadence freshness.** `last_drill_evidence_id` is checked against the effective cadence; stale drill evidence is refused at admission.
- **Hyperscaler-grade rigor.** AWS Audit Manager / GCP Compliance Reports Manager / Azure Purview Compliance Manager precedent is matched. Auditor visibility is parity with hyperscaler offerings.
- **Substrate-vs-product layering preserved.** Substrate µservices (cloud-iam, cloud-kms, audit-chain) declare strict baseline DR; product µservices (crm, marketing-automation, drive) declare their actual baseline; the pack overlay handles the rest.

### C.2 Negative consequences

- **Per-µservice manifest sub-wave required.** ~77 µservices need a manifest update declaring the full `dr` block. The update is per-µservice bespoke (substance-bar applies). Estimated ~4 codex batches under ADR-0328 batch discipline.
- **Pack-activation gate authoring + rollout.** The eight pack-floor Cedar fragments must be authored, signed per ADR-0181, deployed per cell, and soaked as REPORT-ONLY before promoting to BLOCKER.
- **Auditor dashboard authoring.** The per-pack dashboard manifest + auto-generation tooling must be authored under ADR-0263 downstream lane; estimated ~2 codex batches.
- **Cross-team coordination.** DR matrix authoring involves council-architecture + council-security + council-legal + ops-sre-reliability + ops-compliance + axis-observability; each pack floor publication is a six-axis review.
- **Cost spike on tier-mismatch.** A µservice declared at T3 that serves a HIPAA tenant must elevate to ≤ 1h RTO + ≤ 5min RPO + multi-region for that tenant; cell sizing, IaC modules, and backup substrate may need upgrades. The upgrade cost is real but bounded by the pack-activation gate (refusal at admission means no surprise).
- **Pack-floor table evolution.** Regulator updates (HIPAA modernization, PCI-DSS v5, ISO 27001 next revision) require coordinated floor-table version bumps. Each version follows semver per `/specs/compliance-pack-floors.json#packs[*].version`.
- **Per-data-class / per-process refinement complexity.** SOX 404 and KR PIPA refinement sub-tables require auditor sign-off; the auditor's signoff is part of the pack publication flow under ADR-0251.

### C.3 Neutral consequences

- **Service mesh unchanged.** Direct gRPC over HTTP/3 + mTLS via ADR-0145 + ADR-0253 continues regardless of DR matrix.
- **Cedar authorization unchanged.** Cedar evaluates application-layer authorization at request time; DR matrix is admission-time + observability-time.
- **Observability emission preserved.** Per ADR-0263 the new `dr.*` labels are additive.
- **Tenant_class behavior preserved.** demo_trial and paid tenants resolve effective DR via the same algorithm.
- **Cellular tier numbering preserved.** ADR-0248 Tier 0..Tier 4 governs blast-radius; DR matrix governs RTO/RPO. The two axes are orthogonal.
- **Pod runtime tier preserved.** ADR-0338 Tier 0..Tier 3 governs pod isolation; DR matrix governs availability. The two axes are orthogonal.
- **Compliance pack activation flow preserved.** ADR-0251 pack-activation flow gains a floor-check step; the flow shape is unchanged.

### C.4 Engineering-rigor dimensions

| Dimension | Requirement created by this ADR | Acceptance signal |
|---|---|---|
| Maintainability | Single `dr` block per µservice; single per-pack floor entry; one algorithm to resolve effective | Every µservice manifest declares the `dr` block; floor table covers every active pack; `oya-check-dr-pack-floor-satisfaction` green |
| Availability posture | µservice-declared baseline + pack-floor overlay = effective DR | Per-tenant effective triple computable at admission; pack-activation gate refuses on floor violation |
| Compliance | Per-pack floor encoded in machine-readable spec; auditor dashboard per pack | `/specs/compliance-pack-floors.json` ships with 8 packs; dashboard renders per-pack µservice coverage |
| Performance | Floor algorithm is O(packs × µservices) at admission; cached per (tenant, µservice) | Pack-activation gate adds < 50ms p95 to admission |
| Observability | New SLIs `dr.rto_p99_seconds_observed`, `dr.rpo_p99_seconds_observed`, `dr.failover_runbook_executed_at`, `dr.last_drill_evidence_id` | Dashboards segment by pack; drill freshness audited weekly |
| Auditability | Per-pack dashboard surfaces declared/floor/effective per µservice | Regulator query time < 5 min for any pack |
| Cost | µservices pay declared cost; pack overlay forces elevation only when needed | Per-µservice cell sizing reflects declared baseline; per-tenant elevation reflected in finops attribution |
| Hyperscaler alignment | AWS Audit Manager / GCP Compliance Reports / Azure Purview precedent matched | Auditor flow parity with hyperscaler offerings |

### C.5 Hyperscaler-grade rigor application

**Named precedent.**

- **AWS Audit Manager.** Per-control + per-service evidence collection; HIPAA Eligible Services list maps services to HIPAA scope; Resilience Hub assesses per-application DR posture against business-continuity policy.
- **Google Cloud Compliance Reports Manager.** Per-product compliance certifications; Cloud Operations dashboards surface per-service DR contracts; Assured Workloads enforces FedRAMP / HIPAA / PCI / IL5 perimeter.
- **Microsoft Purview Compliance Manager.** Per-control assessment templates (HIPAA, PCI, SOC 2, ISO 27001, FedRAMP); Site Recovery surfaces per-VM DR contracts; Service Health surfaces per-service SLA.
- **HashiCorp Sentinel.** Versioned policy bundles with explicit framework reference; per-policy enforcement at admission.

Every hyperscaler precedent operates a per-pack DR / availability posture with machine-readable controls and auditor-visible dashboards. This ADR adopts the same shape for Oyatie.

**Failure-mode tree.**

(1) µservice declares `dr` block missing a required field → `oya-check-dr-manifest-block` refuses (REPORT-ONLY at landing, BLOCKER post-soak).
(2) µservice declares `multi_region_active_active=false` but serves a pack requiring multi-region → `oya-check-dr-multi-region-required` refuses.
(3) µservice declares `backup_substrate` containing an unallowlisted identifier → `oya-check-dr-backup-substrate-allowlist` refuses.
(4) µservice declares `failover_runbook` path that doesn't resolve → `oya-check-dr-runbook-presence` refuses.
(5) µservice declares `last_drill_evidence_id` audit row older than effective cadence → `oya-check-dr-drill-evidence-fresh` refuses.
(6) Tenant activates a pack whose floor exceeds µservice baseline → pack-activation gate refuses at admission (Cedar fragment `pack-<id>-dr-floor-satisfaction`).
(7) Pack floor table omits a pack referenced by an active tenant → `oya-check-dr-pack-floor-table-coverage` refuses on table publication.
(8) Auditor dashboard manifest missing → `oya-governance-dr-auditor-dashboard-presence` refuses.
(9) `dr_tier` shorthand mismatches numeric fields → `oya-check-dr-manifest-block` refuses on reconciliation.
(10) Effective drill cadence not satisfied at observed cadence → observability emits `dr.drill_cadence_violation` alert; pack-activation gate refuses on next admission.

**Capacity math.** 8 packs × 77 µservices = 616 (pack, µservice) admission combinations. Pack-activation gate caches resolution per (tenant, µservice) pair; cache invalidation on manifest change or pack-floor change. Per-admission overhead < 50ms p95.

**Observability hooks.** Per ADR-0263 the new SLIs and labels are emitted:

- `dr.rto_p99_seconds_declared` (gauge; per µservice)
- `dr.rpo_p99_seconds_declared` (gauge; per µservice)
- `dr.multi_region_active_active` (gauge; per µservice; 0/1)
- `dr.rto_p99_seconds_observed` (histogram; per failover event)
- `dr.rpo_p99_seconds_observed` (histogram; per failover event)
- `dr.failover_runbook_executed_at` (timestamp; per drill)
- `dr.pack_floor_satisfaction` (gauge per (pack, µservice); 0=floor unmet, 1=floor met)
- `dr.drill_evidence_age_seconds` (gauge per µservice)

Audit-chain emits `dr.failover.initiated`, `dr.failover.completed`, `dr.drill.executed`, `dr.pack_activation.floor_check.passed`, `dr.pack_activation.floor_check.failed` event classes.

**Rollback path.** Per-µservice rollback: a misdeclared `dr` block lands a corrective manifest update; the pack-activation gate re-resolves on next admission. Pack-floor rollback: the floor table publishes a prior signed version per ADR-0181; consumers re-resolve. Cross-µservice rollback (e.g., abandon the two-layer matrix entirely) requires a new ADR superseding this one.

**Multi-region awareness.** The `multi_region_active_active` flag is the per-µservice multi-region declaration; the `multi_region_required` pack-floor field is the per-pack multi-region constraint. Effective multi-region is the OR of the two. Cross-region IaC modules per ADR-0339 materialize the cell topology; per-cell observability emits per-region SLIs.

**Sovereign-cell awareness.** Sovereign packs (CSAP-Korea, FedRAMP, IL5/6) participate in the floor table with sovereign-region constraints. Sovereign-cell placement is governed by ADR-0240 + ADR-0251 `cell_eligibility`; this ADR adds the DR layer on top.

**Versioning + deprecation.** This ADR is versioned per ADR-0108. Pack-floor table evolution follows semver per pack entry. Regulator updates require coordinated floor-table version bumps; old floor versions retain for at least one quarterly cycle to support pinned tenants.

## D. Detailed mechanics — ten enforcement surfaces

The DR matrix mechanism touches ten enforcement surfaces. Each subsection D-1 through D-10 enumerates one surface. Numbering is normative.

### D-1: Manifest `dr` block declaration

D-1.1. Every µservice's `microservices/<name>/manifest.json` MUST declare a top-level `dr` block whose value is an object satisfying the schema in `/specs/microservices/manifest-schema.json#properties.dr` (updated by this ADR's downstream schema update).

D-1.2. The block REQUIRES five fields: `rto_p99_seconds`, `rpo_p99_seconds`, `multi_region_active_active`, `backup_substrate`, `failover_runbook`.

D-1.3. The block ACCEPTS two optional fields: `dr_tier ∈ {T1, T2, T3, T4}` and `last_drill_evidence_id`.

D-1.4. The block MAY accept an optional `replication_shape` field whose value is drawn from `/specs/dr-business-continuity.json#tiers[*].replication_shape`.

D-1.5. Reference example for a Tier 1 substrate µservice (cloud-kms):

```json
{
  "name": "cloud-kms",
  "dr": {
    "rto_p99_seconds": 300,
    "rpo_p99_seconds": 30,
    "multi_region_active_active": true,
    "backup_substrate": ["valkey_cluster", "postgres_wal_g", "object_storage_versioned", "openbao_seal_unseal"],
    "failover_runbook": "runbooks/dr-failover.md",
    "dr_tier": "T1",
    "last_drill_evidence_id": "audit:dr.drill:2026-04-15T09:30:00Z:cloud-kms",
    "replication_shape": "active-active-multi-az-cross-region-warm"
  }
}
```

D-1.6. Reference example for a Tier 2 first-party application µservice (crm):

```json
{
  "name": "crm",
  "dr": {
    "rto_p99_seconds": 3600,
    "rpo_p99_seconds": 60,
    "multi_region_active_active": false,
    "backup_substrate": ["postgres_wal_g", "object_storage_versioned"],
    "failover_runbook": "runbooks/dr-failover.md",
    "dr_tier": "T2",
    "last_drill_evidence_id": "audit:dr.drill:2026-04-01T14:00:00Z:crm",
    "replication_shape": "active-passive-cross-region-continuous"
  }
}
```

D-1.7. Reference example for a Tier 3 long-tail µservice (analytics):

```json
{
  "name": "analytics",
  "dr": {
    "rto_p99_seconds": 14400,
    "rpo_p99_seconds": 900,
    "multi_region_active_active": false,
    "backup_substrate": ["iceberg_snapshot", "object_storage_versioned"],
    "failover_runbook": "runbooks/dr-failover.md",
    "dr_tier": "T3"
  }
}
```

D-1.8. CI lane `oya-check-dr-manifest-block` step 1 parses the manifest, validates `dr` block presence and well-typedness.

D-1.9. CI lane step 2 validates `backup_substrate` array members are drawn from the §D-5 allowlist.

D-1.10. CI lane step 3 validates `failover_runbook` path resolves to an existing markdown file inside `microservices/<name>/runbooks/`.

D-1.11. CI lane step 4 reconciles `dr_tier` (if present) with the numeric fields per §D-8 table; mismatch is refused.

### D-2: Compliance pack floor table — `/specs/compliance-pack-floors.json`

D-2.1. The canonical machine-readable per-pack DR floor table lives at `/specs/compliance-pack-floors.json`.

D-2.2. Each entry declares `pack_id` (matching `/specs/compliance-pack-schema.json#pack_id`), `version` (semver), `dr_floor` (the floor sub-block), and `_meta` (citations + effective_at + sunset_at + signed_by).

D-2.3. The `dr_floor` sub-block REQUIRES `rto_p99_seconds`, `rpo_p99_seconds`, `multi_region_required`, `drill_cadence_required`.

D-2.4. The `dr_floor` sub-block ACCEPTS optional refinement sub-tables `process_floors` (for SOX 404) and `data_class_floors` (for KR PIPA).

D-2.5. The table is signed per ADR-0181 cosign discipline by the compliance-office signing key (`compliance-office:dr-pack-floors:v<N>`).

D-2.6. Table version bumps follow semver: MAJOR for floor-stringency increase (smaller seconds = stricter); MINOR for additive refinement (new `process_floors` row, new `data_class_floors` row); PATCH for editorial-only updates (citation polish).

D-2.7. Table evolution under regulator updates (e.g., HIPAA modernization) follows ADR-0108 sunset discipline; prior table versions retain for at least one quarterly cycle.

D-2.8. The table is the source of truth for pack-floor data consumed by the pack-activation gate Cedar fragments and the auditor dashboard.

### D-3: Eight initial pack floors

The eight initial entries in `/specs/compliance-pack-floors.json` are codified here. Each pack's floor is anchored to specific regulator citations.

#### D-3.1 HIPAA-2024

- **Floor**: RTO ≤ 3600s (1h), RPO ≤ 300s (5min), multi-region REQUIRED, drill quarterly.
- **Citations**: 45 CFR §164.308(a)(7) (Contingency Plan); §164.310(a)(2)(i) (Facility Security Plan); §164.312(a)(2)(ii) (Emergency Access Procedure); §164.308(a)(7)(ii)(D) (Testing and Revision Procedures — quarterly testing customary baseline for BAA-eligible providers).
- **Rationale**: HIPAA Contingency Plan requires emergency-mode operation; the customary AWS / Azure / GCP HIPAA-Eligible Services interpretation is ≤ 1h RTO with multi-region active-active or active-passive. PHI-touching µservices on this floor preserve patient care continuity during major-region failure.

#### D-3.2 PCI-DSS-L1-v4

- **Floor**: RTO ≤ 86400s (24h), RPO ≤ 3600s (1h), multi-region NOT required, drill annual.
- **Citations**: PCI DSS v4.0 §12.10 (Incident Response Plan); §12.10.1 (Documented business continuity); §12.10.2 (Annual testing of incident response procedures).
- **Rationale**: PCI-DSS focuses on cardholder-data segmentation and incident response timing, not on multi-region topology. The 24h RTO + 1h RPO reflects the regulator's tolerance for backup-restore within a documented business-continuity policy. Per-region active-active is not mandated; tenants requiring tighter RTO/RPO compose PCI with SOC2 or ISO floors.

#### D-3.3 SOC2-T2

- **Floor**: RTO ≤ 14400s (4h), RPO ≤ 900s (15min), multi-region NOT required, drill annual.
- **Citations**: SOC 2 Trust Services Criteria CC7.5 (System recovery); CC9.2 (Vendor and business partner risk management — DR provisions). AICPA SOC 2 Type II audit guidance — system recovery objectives + tested annually.
- **Rationale**: SOC 2 Type II demands tested system recovery without prescribing a specific RTO/RPO. The 4h / 15min floor reflects the customary baseline for SaaS providers seeking SOC 2 Type II audit reports. Multi-region is not required; many SOC 2-attested SaaS providers operate single-region active-passive with documented runbooks.

#### D-3.4 EU-AI-ACT-2024-HIGH-RISK

- **Floor**: RTO ≤ 1800s (30min), RPO ≤ 300s (5min), multi-region REQUIRED, drill quarterly.
- **Citations**: EU AI Act Regulation (EU) 2024/1689 Article 17 (Quality management system — continuity obligations); Article 19 (Automatically generated logs — retention through system lifecycle); Article 26 (Obligations of deployers — sustained availability); Article 27 (Fundamental rights impact assessment — operational continuity).
- **Rationale**: High-risk AI systems under EU AI Act Annex III (biometric identification, critical infrastructure, education, employment, essential services, law enforcement, migration, justice + democratic process) face fundamental-rights impact assessment. Unavailability of these systems creates a fundamental-rights gap. The 30min RTO + 5min RPO + multi-region floor reflects the regulator's expectation of continuous operation.

#### D-3.5 KR-CSAP-v3.1

- **Floor**: RTO ≤ 3600s (1h), RPO ≤ 900s (15min), multi-region REQUIRED, drill semi-annual.
- **Citations**: Korea CSAP (Cloud Security Assurance Program) §3.5.1 (BCP/DR); §3.5.2 (Continuity testing — semi-annual minimum for CSAP-certified cloud services); KISA notice 2024-1 CSAP v3.1 update.
- **Rationale**: CSAP-Korea is the canonical Korean public-sector cloud security certification. The 1h RTO + 15min RPO + multi-region (within KR sovereign region) reflects KISA's expectation. The drill cadence is semi-annual customary baseline.

#### D-3.6 ISO27001-2022

- **Floor**: RTO ≤ 14400s (4h), RPO ≤ 3600s (1h), multi-region NOT required, drill annual.
- **Citations**: ISO/IEC 27001:2022 Annex A.5.30 (ICT readiness for business continuity); ISO/IEC 22301:2019 (Business continuity management — referenced normatively); ISO/IEC 27002:2022 §5.30 (Control implementation guidance).
- **Rationale**: ISO 27001 demands ICT readiness for business continuity; ISO 22301 provides the BCM framework. The 4h / 1h floor reflects the customary baseline for ISO 27001-certified information security management systems.

#### D-3.7 SOX-404 — per-process refinement

- **Default floor**: RTO ≤ 14400s (4h), RPO ≤ 3600s (1h), multi-region NOT required, drill annual.
- **Process floors** (representative; the auditor sponsoring SOX scope publishes the in-scope list per quarter):
  - `general_ledger_journal_entry`: RTO ≤ 14400s (4h), RPO ≤ 60s (1min) — close cycle continuity.
  - `revenue_recognition`: RTO ≤ 28800s (8h), RPO ≤ 900s (15min).
  - `vendor_master_data`: RTO ≤ 86400s (24h), RPO ≤ 3600s (1h).
  - `quarterly_close_working_papers`: RTO ≤ 86400s (24h), RPO ≤ 3600s (1h); RETENTION 7-year minimum.
- **Citations**: Sarbanes-Oxley Act §404 (Management Assessment of Internal Controls); PCAOB AS 2201 (An Audit of Internal Control Over Financial Reporting); COSO Internal Control — Integrated Framework (2013).
- **Rationale**: SOX 404 is process-oriented; financial-process availability is the audit objective. Per-process refinement allows µservices to satisfy each process's specific floor without over-paying. The per-process list is owned by the auditor sponsoring SOX scope.

#### D-3.8 KR-PIPA-2023-amendment — per-data-class refinement

- **Default floor**: RTO ≤ 14400s (4h), RPO ≤ 900s (15min), multi-region NOT required, drill semi-annual.
- **Data-class floors**:
  - `PI_KR_RESIDENT_REGISTRATION_NUMBER` (RRN, 주민등록번호): RTO ≤ 3600s (1h), RPO ≤ 300s (5min), multi-region REQUIRED (within KR sovereign region), drill quarterly.
  - `PI_KR_SENSITIVE`: RTO ≤ 7200s (2h), RPO ≤ 600s (10min), multi-region NOT required, drill semi-annual.
  - `PI_KR_PIPA` (general personal information): default floor applies.
- **Citations**: 개인정보보호법 (Personal Information Protection Act) §29 (Safeguards); §24-2 (RRN handling restrictions); §59 (Criminal penalties for serious leakage); PIPC notice 2023-05 (technical and managerial safeguards).
- **Rationale**: PIPA imposes safeguards per personal-information data class. RRN compromise has criminal-class regulator response under §59; the floor for RRN-touching µservices is tighter than general PI.

### D-4: Effective tenant DR resolution algorithm

D-4.1. The effective tenant DR contract per (tenant, µservice) pair is computed by the algorithm below. The algorithm is invoked at pack-activation admission and at every effective-DR query (auditor dashboard, observability dashboard, finops cost attribution).

D-4.2. **Inputs**:

- µservice declared block: `M = manifest.json#dr` with `M.rto`, `M.rpo`, `M.mr`, `M.backup`, `M.runbook`, `M.cadence_default` (derived from `M.dr_tier`).
- Tenant's activated pack set: `T.packs = {P_1, P_2, ..., P_n}`.
- For each pack `P_i` whose `applies_to_microservices` includes the µservice name: `P_i.rto`, `P_i.rpo`, `P_i.mr_required`, `P_i.drill_required`.
- For SOX-404 packs: per-process floor lookup keyed by the in-scope process declared by the µservice.
- For KR-PIPA packs: per-data-class floor lookup keyed by `manifest.json#data_classes_processed` intersection with PIPA data classes.

D-4.3. **Outputs**:

- `effective.rto = min(M.rto, min over applicable P_i.rto)`
- `effective.rpo = min(M.rpo, min over applicable P_i.rpo)`
- `effective.mr = M.mr OR (OR over applicable P_i.mr_required)`
- `effective.cadence = shortest cadence among M.cadence_default and all applicable P_i.drill_required`

D-4.4. The MIN is the realization of "more-stringent-wins". RTO/RPO are upper bounds; smaller seconds = stricter; the MIN selects the strictest. The user-memory phrasing "max of µservice declared, all-applicable-pack floors" refers to the stringency partial order, not numeric max — a stricter floor is "max stringency."

D-4.5. The pack-activation gate refuses tenant pack activation IF effective resolution proves the µservice's declared `M.rto > P_i.rto` (or analogous RPO mismatch or multi-region mismatch) for ANY applicable pack `P_i` in the activation set. Stated equivalently: the gate requires `M.rto ≤ P_i.rto AND M.rpo ≤ P_i.rpo AND (P_i.mr_required → M.mr)` for every applicable pack `P_i`.

D-4.6. **Worked example 1**: tenant activates `OPEN-INTERNAL-BASELINE` only; µservice `crm` declares (3600s, 60s, false). Effective = (3600s, 60s, false). Pack-activation gate: PASS.

D-4.7. **Worked example 2**: tenant activates `HIPAA-2024`; µservice `crm` declares (3600s, 60s, false). HIPAA floor = (3600s, 300s, true). Pack-activation gate: REFUSE — `crm` declares `multi_region_active_active=false` but HIPAA `multi_region_required=true`. Resolution: `crm` must elevate to `multi_region_active_active=true` to serve HIPAA tenants.

D-4.8. **Worked example 3**: tenant activates `EU-AI-ACT-2024-HIGH-RISK` + `EU-GDPR-2018-baseline`; µservice `intelligence` declares (300s, 30s, true). EU AI Act floor = (1800s, 300s, true). GDPR has no DR floor (per `/specs/compliance-pack-floors.json`). Effective = (min(300, 1800), min(30, 300), true OR true) = (300s, 30s, true). Pack-activation gate: PASS — `intelligence` baseline is stricter than EU AI Act floor.

D-4.9. **Worked example 4**: tenant activates `SOX-404`; µservice `cloud-billing` declares (14400s, 900s, false) and `manifest.json#sox_in_scope_processes = ["general_ledger_journal_entry"]`. SOX general-ledger process floor = (14400s, 60s, false). Effective = (14400s, min(900, 60), false) = (14400s, 60s, false). Pack-activation gate: REFUSE — `cloud-billing` declares `rpo_p99_seconds=900` but general-ledger floor requires `rpo_p99_seconds ≤ 60`. Resolution: `cloud-billing` must tighten its RPO for general-ledger workflows or scope general-ledger to a different µservice.

D-4.10. **Worked example 5**: tenant activates `KR-PIPA-2023-amendment`; µservice `identity` declares (3600s, 300s, true) and `manifest.json#data_classes_processed` includes `PI_KR_RESIDENT_REGISTRATION_NUMBER`. PIPA RRN data-class floor = (3600s, 300s, true). Effective = (3600s, 300s, true). Pack-activation gate: PASS — `identity` declared meets the RRN floor exactly.

### D-5: Backup substrate allowlist

D-5.1. The canonical backup substrate allowlist (planned to be enforced by `oya-check-dr-backup-substrate-allowlist`, advisory until the crate lands):

- `valkey` (in-memory KV; backup via Valkey AOF + RDB to object_storage_versioned)
- `valkey_cluster` (in-memory KV cluster; per-shard AOF + RDB)
- `postgres_wal_g` (OLTP; continuous WAL archive to object storage)
- `iceberg_snapshot` (OLAP table format; per-snapshot rollback per ADR-0337)
- `object_storage_versioned` (object storage with versioning + object-lock)
- `seaweedfs_replicated` (SeaweedFS with cross-cell replication per ADR-0196)
- `milvus_snapshot` (vector store snapshots to object storage)
- `clickhouse_iceberg_layered` (ClickHouse compute layered on Iceberg per ADR-0337)
- `openbao_seal_unseal` (OpenBao secrets with multi-region seal/unseal)
- `audit_chain_merkle_seal` (audit-chain Merkle seal to object storage; tamper-evident)

D-5.2. Allowlist additions follow ADR-0108 sunset discipline and require a substrate-decision ADR (e.g., a new ADR adopting a new canonical substrate).

D-5.3. Allowlist removals follow ADR-0108 sunset discipline; the removed substrate retains at least one quarterly cycle of consumer drainage.

D-5.4. The allowlist is encoded in `/specs/compliance-pack-floors.json#_meta.backup_substrate_allowlist` for machine consumption and is also enumerated in `tools/hooks/_canonical-primitives.md`.

### D-6: Drill cadence freshness check

D-6.1. The CI lane `oya-check-dr-drill-evidence-fresh` parses `manifest.json#dr.last_drill_evidence_id` and resolves the audit-chain row.

D-6.2. The lane computes the row's age (now - row_emitted_at) and compares it to the µservice's effective drill cadence.

D-6.3. Effective drill cadence is the SHORTEST cadence among the µservice's tier-default cadence (per ADR-0241 §C.1 table) and every applicable pack's `drill_cadence_required`.

D-6.4. Cadence-to-seconds mapping for freshness comparison:

- `quarterly-plus-ad-hoc` → 90 days
- `quarterly` → 90 days
- `semi-annual` → 180 days
- `annual-tabletop` → 365 days
- `annual` → 365 days

D-6.5. Age > cadence-seconds refuses the manifest at admission.

D-6.6. Pack-activation gate also checks freshness at activation time; activation is refused for a (tenant, µservice) pair whose `last_drill_evidence_id` is stale relative to the activated pack's cadence requirement.

### D-7: Multi-region required overlay

D-7.1. Each pack entry in `/specs/compliance-pack-floors.json` declares `multi_region_required` as a boolean.

D-7.2. The CI lane `oya-check-dr-multi-region-required` cross-walks every pack's `applies_to_microservices` set and refuses any µservice that declares `multi_region_active_active=false` while serving a pack with `multi_region_required=true`.

D-7.3. Effective tenant `multi_region_active_active = M.mr OR (OR over applicable P_i.mr_required)`. Once any applicable pack requires multi-region, the effective is `true`.

D-7.4. Multi-region implementation is governed by ADR-0158 (multi-region active-active) and the per-cell IaC modules from ADR-0339 (`oyatie-as-cloud-provider/cell-zone`, `aws-guest/vpc` with cross-region peering, `oci-guest/drg` Dynamic Routing Gateway).

D-7.5. Sovereign packs (CSAP-Korea, FedRAMP, IL5/6) interpret "multi-region" as within the sovereign perimeter (CSAP within KR; FedRAMP within US authorized regions; IL5 within DoD-authorized regions); cross-perimeter multi-region is forbidden per ADR-0240.

### D-8: `dr_tier` shorthand reconciliation

D-8.1. The ADR-0241 four-tier T1..T4 shorthand is preserved at the µservice declaration layer. When `manifest.json#dr.dr_tier` is declared, the numeric fields MUST satisfy the tier's numeric defaults from `/specs/dr-business-continuity.json#tiers`:

- T1: `rto_p99_seconds ≤ 300` (5 min), `rpo_p99_seconds = 0`, `multi_region_active_active = true`.
- T2: `rto_p99_seconds ≤ 3600` (60 min), `rpo_p99_seconds ≤ 60` (1 min), `multi_region_active_active` MAY be true or false (per `replication_shape = active-passive-cross-region-continuous`).
- T3: `rto_p99_seconds ≤ 14400` (240 min), `rpo_p99_seconds ≤ 900` (15 min), `multi_region_active_active` typically false.
- T4: `rto_p99_seconds ≤ 86400` (1440 min), `rpo_p99_seconds ≤ 3600` (1 h), `multi_region_active_active` typically false.

D-8.2. A µservice that declares T1 but populates the numeric fields at T2 values is refused by `oya-check-dr-manifest-block` (mismatch).

D-8.3. A µservice MAY declare numeric fields tighter than the tier shorthand (e.g., T2 µservice declaring `rpo_p99_seconds=0` and `multi_region_active_active=true`); this is permitted and reconciles cleanly.

D-8.4. Omitting `dr_tier` and declaring only numeric fields is permitted; the tier is inferred for ergonomic display purposes but is not authoritative.

D-8.5. The ADR-0241 §C.2 "mixed-mode is forbidden" rule (µservices may not declare two tiers per environment) is preserved verbatim; the inference does not split the µservice across tiers.

### D-9: Auditor dashboard

D-9.1. The per-pack auditor dashboard manifest lives at `microservices/compliance/dashboards/per-pack-dr-floor-satisfaction.yaml`.

D-9.2. The manifest is auto-generated from `/specs/compliance-pack-floors.json` × per-µservice manifest declarations. Regeneration triggers: manifest change, floor-table version bump.

D-9.3. The dashboard surfaces, per pack, the following table:

| µservice | declared (RTO, RPO, mr) | pack floor (RTO, RPO, mr_required) | effective (RTO, RPO, mr) | last drill | floor satisfaction |
|---|---|---|---|---|---|

D-9.4. The `floor satisfaction` column displays green when the µservice's declared meets the floor or its baseline is strict enough that the effective resolution succeeds; red when the µservice cannot satisfy the floor and pack-activation would refuse.

D-9.5. The dashboard is queryable by auditors via the observability portal per ADR-0263 + the compliance evidence pipeline.

D-9.6. The CI lane `oya-governance-dr-auditor-dashboard-presence` refuses missing or stale auditor dashboard entries.

D-9.7. The dashboard authoring is sequenced as a downstream observability lane under ADR-0263; this ADR queues the lane but does not author the dashboard.

### D-10: Pack-activation gate Cedar fragment

D-10.1. Each pack `P` in `/specs/compliance-pack-floors.json` ships a companion Cedar fragment `pack-<pack-id>-dr-floor-satisfaction` under `/specs/cedar-fragment-schema.json`.

D-10.2. The fragment evaluates at tenant pack-activation admission. It receives the (tenant, µservice) pair, the µservice's `manifest.json#dr` block, the pack's `dr_floor`, and emits a `forbid` decision when floor satisfaction fails.

D-10.3. Fragment skeleton (illustrative; the real fragment is signed per ADR-0181 and lives under the pack bundle per ADR-0251):

```cedar
// pack-hipaa-2024-dr-floor-satisfaction.cedar
forbid (
  principal,
  action == Action::"activate_pack",
  resource is Pack::"HIPAA-2024"
)
when {
  context.tenant.active_microservices.any(ms,
    ms.manifest.dr.rto_p99_seconds > 3600 ||
    ms.manifest.dr.rpo_p99_seconds > 300 ||
    !ms.manifest.dr.multi_region_active_active
  )
};
```

D-10.4. The fragment is loaded under scope `pack/<pack-id>` per ADR-0251 fragment loading.

D-10.5. The fragment is signed via cosign per ADR-0181 (`compliance-office:dr-pack-floors:v<N>`) and the attestation is referenced in `/specs/compliance-pack-floors.json#packs[*]._meta.cedar_fragment_attestation`.

D-10.6. Per-process and per-data-class refinement (SOX-404, KR-PIPA) materializes as additional Cedar `when` clauses keyed by `ms.manifest.sox_in_scope_processes` or `ms.manifest.data_classes_processed` intersections.

D-10.7. Fragment authoring lands as part of the per-pack publication flow under ADR-0251; this ADR queues the fragment but does not author them inline (the eight fragments live in the per-pack bundle directories).

## E. Enforcement lanes — eight new lanes

The eight new CI / governance lanes added by this ADR:

E-1. **`oya-check-dr-manifest-block`** — REPORT-ONLY at landing; BLOCKER after Wave 15W-DR-Matrix-declaration lands. Refuses missing or malformed `dr` block in per-µservice manifest.

E-2. **`oya-check-dr-pack-floor-satisfaction`** — REPORT-ONLY at landing; BLOCKER after the floor table publishes signed v1.0.0. Computes effective RTO/RPO per (tenant-class, µservice) and refuses µservice declarations that fail floor satisfaction for any applicable pack.

E-3. **`oya-check-dr-multi-region-required`** — REPORT-ONLY at landing; BLOCKER at day 30. Refuses `multi_region_active_active=false` for µservices serving a pack with `multi_region_required=true`.

E-4. **`oya-check-dr-runbook-presence`** — REPORT-ONLY at landing; BLOCKER at day 30. Refuses `failover_runbook` pointer that does not resolve to an existing markdown file inside `microservices/<name>/runbooks/`.

E-5. **`oya-check-dr-backup-substrate-allowlist`** — REPORT-ONLY at landing; BLOCKER at day 30. Refuses substrate identifiers outside the §D-5 allowlist.

E-6. **`oya-check-dr-drill-evidence-fresh`** — REPORT-ONLY at landing; BLOCKER at day 60 (giving µservices time to schedule drills). Refuses µservices whose `last_drill_evidence_id` audit row is older than the effective drill cadence.

E-7. **`oya-check-dr-pack-floor-table-coverage`** — REPORT-ONLY at landing; BLOCKER on next floor-table version publication. Refuses table versions that omit a pack referenced by any active tenant.

E-8. **`oya-governance-dr-auditor-dashboard-presence`** — REPORT-ONLY at landing; BLOCKER after the dashboard auto-generation tooling lands. Refuses missing per-pack auditor dashboard manifest entries.

## F. Rejected alternatives

### F.1 Alt-1: keep ADR-0241 four-tier shorthand only; no per-pack floor

**Rejected.** Forces every µservice to declare its strictest possible DR contract upfront to cover every possible pack a tenant might activate. Imposes T1 cost on µservices that serve predominantly non-HIPAA / non-EU-AI-Act tenants. Auditor cannot see per-pack µservice coverage; pack-activation cannot refuse at admission. The two-dimensional reality of "µservice baseline + pack overlay" is the right shape.

### F.2 Alt-2: per-tenant RTO/RPO declaration (per-tenant override of µservice baseline)

**Rejected.** Two reasons. First, tenant-specific RTO/RPO is not a tenant-controlled parameter — it is regulator-driven and pack-driven. A tenant cannot lower its effective floor below the pack mandate (the regulator does not allow it); a tenant cannot raise its floor above the µservice's capacity (the µservice cannot serve a tighter contract than it can engineer). Second, per-tenant DR breaks the µservice's cell-sizing math (per ADR-0009 + ADR-0248) — cells size for the strictest tenant they host; if every tenant had its own floor, cell sizing would be O(tenant) instead of O(pack). The per-pack overlay is the right granularity.

### F.3 Alt-3: a single RTO/RPO floor across all packs (HIPAA-class for everyone)

**Rejected.** Mirrors Alt-1 but more extreme. Forces every pack tenant to incur HIPAA-class cost regardless of regulator mandate. Imposes 1-2 orders-of-magnitude unnecessary cost on PCI / SOC2 / ISO 27001 tenants. Loses per-pack regulator alignment — auditors cannot see why a µservice meets HIPAA when PCI is the pack actually activated. Pack-specific floors are the right shape.

### F.4 Alt-4: declare DR posture in a separate file `microservices/<name>/dr-posture.yaml` instead of in `manifest.json`

**Rejected.** Per ADR-0212 buildability doctrine, the per-µservice manifest is the single canonical declaration target for all per-µservice metadata. Splitting DR posture into a separate file fragments the declaration surface and complicates the `oya-check-dr-*` lane implementations (each lane would need to load two files instead of one). The unified manifest is the right shape.

### F.5 Alt-5: encode pack floors directly inside `/specs/compliance-pack-schema.json` instead of a new `/specs/compliance-pack-floors.json`

**Rejected.** Two reasons. First, `/specs/compliance-pack-schema.json` describes the schema (structure + constraints) for pack bundles; the per-pack DR floor table is data, not schema. Mixing schema and data complicates versioning (a floor table version bump should not bump the schema version). Second, the floor table is consumed by the pack-activation gate as a separate machine-readable artifact; consumers want a focused file (`compliance-pack-floors.json`) rather than the entire schema file. The split is the right shape and matches the precedent of `/specs/dr-business-continuity.json` being a separate file from the manifest schema.

## G. Sunset schedule

G-1. **Day 0 (Acceptance)**: ADR-0343 lands as Proposed; `/specs/compliance-pack-floors.json` lands at v1.0.0 with eight initial floors; manifest-schema gains the `dr` block; master-plan-sequencing.json queues `15W-DR-Matrix-declaration` sub-wave.

G-2. **Day 0 — Day 30**: Eight new CI lanes are REPORT-ONLY. Per-µservice manifest updates land in Wave 15W-DR-Matrix-declaration. Per-pack Cedar fragments are authored and signed.

G-3. **Day 30**: `oya-check-dr-manifest-block`, `oya-check-dr-multi-region-required`, `oya-check-dr-runbook-presence`, `oya-check-dr-backup-substrate-allowlist`, and `oya-check-dr-pack-floor-table-coverage` promote to BLOCKER.

G-4. **Day 60**: `oya-check-dr-drill-evidence-fresh` promotes to BLOCKER (giving µservices a 60-day window to schedule drills).

G-5. **Day 90**: `oya-check-dr-pack-floor-satisfaction` and `oya-governance-dr-auditor-dashboard-presence` promote to BLOCKER. Pack-activation gate Cedar fragments are fully deployed across all cells.

G-6. **Quarterly review** (every 90 days post-Acceptance): council-architecture + council-security + council-legal walk the pack-floor table for regulator updates (HIPAA modernization, PCI-DSS v5, ISO 27001 next revision); coordinated floor-table version bump under ADR-0108.

## H. Acceptance criteria

H-1. `/specs/compliance-pack-floors.json` lands at v1.0.0 with eight initial pack floors signed via cosign.

H-2. `/specs/microservices/manifest-schema.json` updated with the top-level `dr` block schema; `oya-check-dr-manifest-block` lane validates the schema.

H-3. `/specs/master-plan-sequencing.json` updated with `15W-DR-Matrix-declaration` sub-wave queued under `waves_15_plus.sub_waves`.

H-4. `tools/hooks/_canonical-primitives.md` updated with DR matrix section naming this ADR.

H-5. Eight new CI lanes (E-1 through E-8) are stubbed and report REPORT-ONLY at landing.

H-6. Eight pack-floor Cedar fragments authored and signed; cosign attestations recorded in the floor table.

H-7. Auditor dashboard manifest generation tooling stubbed (full implementation deferred to ADR-0263 downstream lane).

H-8. Multispectrum review v2.4.0 evidence pack lands at `evidence/debate/ADR-0343/`.

H-9. Wave 15W-DR-Matrix-declaration sub-wave dispatched under ADR-0328 batch discipline.

H-10. ADR-0341 (cellular promotion gates) consumes this ADR's effective-DR algorithm at promotion-gate evaluation.

## I. Related work

- ADR-0028 (cloud-microservice architecture): operational-readiness expectations.
- ADR-0099 (data-class registry): canonical data-class taxonomy for KR-PIPA refinement.
- ADR-0108 (deprecation and sunset discipline): floor-table evolution.
- ADR-0128 (hyperscaler architecture invariants): hyperscaler-grade rigor baseline.
- ADR-0145 (inter-µservice communication reform): direct gRPC over HTTP/3 + 3 invariants.
- ADR-0150 (Cedar policy engine): pack-activation gate substrate.
- ADR-0152 (RPO/RTO five-tier model): predecessor of ADR-0241 four-tier portfolio; tiering history.
- ADR-0158 (multi-region active-active): canonical multi-region topology decision.
- ADR-0181 (cosign-signed artifacts and modules): floor-table + Cedar fragment signing.
- ADR-0183 (policy-engine separation): Cedar vs Kyverno.
- ADR-0211 (in-house tech stack preference): substrate allowlist constraints.
- ADR-0212 (buildability doctrine): manifest as canonical declaration.
- ADR-0240 (sovereign-cloud per regional pack): sovereign-perimeter multi-region.
- ADR-0241 (DR business continuity portfolio): predecessor; this ADR overlays the per-pack floor.
- ADR-0242 (oyatie-is-a-tenant): oyatie itself activates packs and resolves effective DR.
- ADR-0243 (Cedar as universal gate): pack-activation gate is a Cedar fragment.
- ADR-0244 (tenant as universal scoping primitive): per (tenant, µservice) resolution.
- ADR-0245 (substrate-vs-product layering): substrate µservices declare strict baseline; product µservices declare actual baseline.
- ADR-0247 (self-modification doctrine): foundry workflow execution under DR matrix.
- ADR-0248 (cellular architecture): cell-internal DR primitives.
- ADR-0250 (build-ahead-of-certification): floor established before tenant onboarding.
- ADR-0251 (compliance pack cell certification levels): pack-activation flow integration.
- ADR-0252 (HLC default + TrueTime tier): per-µservice causality affecting RPO semantics.
- ADR-0253 (network topology edge service mesh): cross-region failover transport.
- ADR-0254 (deployment-model-spectrum): K8s + Cloud Hypervisor + Kata DR substrate.
- ADR-0255 (intelligence as two-layer AI substrate): EU AI Act high-risk affected systems.
- ADR-0263 (observability emission contract): DR SLI emission + auditor dashboard.
- ADR-0322 (substance-bar as doctrine and CI enforcement): substance-bar baseline for this ADR.
- ADR-0324 (anti-script authoring doctrine): bespoke authoring requirement.
- ADR-0328 (substance-bar as canonical sequence and batch discipline): Wave 15W-DR-Matrix-declaration sub-wave under this batch discipline.
- ADR-0330 (tenant-class demo_trial vs paid + composable billing components): demo_trial activates fewer packs; paid activates per entitlement.
- ADR-0331 (cross-microservice tenant-class adoption template): per-context DR module wiring.
- ADR-0336 (Valkey not Redis substrate): backup substrate allowlist baseline.
- ADR-0337 (Iceberg canonical OLAP write path): iceberg_snapshot substrate.
- ADR-0338 (pod runtime tier 0..3): orthogonal axis to DR matrix.
- ADR-0339 (shared IaC module library): multi-region IaC primitives.
- ADR-0340 (capacity model per µservice manifest): sibling decision from 2026-05-21 hexad.
- ADR-0341 (cellular promotion gates per tier): consumes effective-DR algorithm at promotion.
- ADR-0342 (API versioning hybrid model): sibling decision.

## J. Authority + provenance

This ADR is authored under the realignment effort's authority chain:

- Primary directive: `feedback_six_candidate_adrs_2026_05_21.md` ADR-0343 section.
- ADR-0328 substance-bar canonical sequence and batch discipline.
- ADR-0322 substance-bar as doctrine and CI enforcement.
- ADR-0324 anti-script authoring doctrine.
- Multispectrum review v2.4.0 per ADR-0322 §D-2 (evidence at `evidence/debate/ADR-0343/` post-PR).
- Authority chain per CLAUDE.md root hub.

Bound by `feedback_quality_performance_scalability_bar` (hyperscaler-grade rigor) and `feedback_no_silent_regression` (no silent changes to substrate contracts). Hyperscaler precedent: AWS Audit Manager + Resilience Hub; GCP Compliance Reports Manager + Assured Workloads; Microsoft Purview Compliance Manager + Site Recovery.
