---
plan_id: REGCLOUD-001
kanban_task: t_44b3d436
title: Compliance-Selective Cell Placement Architecture
status: Draft
date: 2026-06-30
lane: cloud-compliance/placement
scope: planning-spec-only
owner: council-architecture + ops-compliance + axis-cell + axis-tenancy
blast_radius: no product-code mutation; no new CLI; no oya/cloud reorg
related_adrs:
  - ADR-0240
  - ADR-0241
  - ADR-0243
  - ADR-0244
  - ADR-0248
  - ADR-0251
  - ADR-0343
source_specs:
  - specs/tenant-model.json
  - specs/compliance-pack-schema.json
  - specs/compliance-pack-floors.json
  - specs/platform-architecture.json
  - docs/architecture/diagrams/cell-routing-shuffle-sharding.md
  - docs/architecture/diagrams/compliance-pack-overlay-precedence.md
---

# Compliance-Selective Cell Placement Architecture

## 0. Goal and Non-Goals

Goal: define a placement architecture where compliance-heavy controls are applied only to the tenant sub-scopes, data classes, workloads, and regulatory packs that require them, while non-regulated workloads remain on lean standard cells.

This plan is a planning/spec artifact. It does not mutate product code, does not add a CLI surface, and does not introduce a new cloud/oya reorg axis. It composes existing doctrine:

- ADR-0248: cells are the universal blast-radius primitive; Tier 3 cells host tenant workloads; hot paths remain intra-cell; shuffle sharding draws from an eligible cell pool.
- ADR-0251: compliance packs are signed bundles; cells declare certification levels; tenant pack activation requires cell pinning and emits audit evidence.
- ADR-0240: a cell lives on exactly one provider; sovereign data classes stay within the provider set declared by the regional/sovereign overlay.
- `specs/tenant-model.json`: tenant rows carry `home_cell`, `dr_cell`, `jurisdiction`, `sovereign_cloud_pack`, `kyc_status`, `compliance_packs`, `provider_credential_mode`, `policy_evaluation_mode`, `ontology_read_mode`, `freshness_floor`, and `business_continuity_dr_tier`.
- `specs/compliance-pack-schema.json`: packs declare `cell_eligibility.requires_certification`, `forbidden_cells`, BYOK requirements, data-class extensions, retention, consent, cross-tenant rules, jurisdiction overlays, and regulator references.
- `specs/compliance-pack-floors.json`: effective DR posture is tightened by applicable compliance-pack floors.

Non-goals:

- Do not claim any cell is certified unless a pack/cell evidence packet proves it.
- Do not move an entire tenant to a high-cost compliant cell just because one workload or data class is regulated.
- Do not make product services duplicate legal interpretation; pack policy remains the legal/control vessel.
- Do not allow standard cells to become implicit fallbacks for regulated data.

## 1. Architectural Principle: Compliance Islands, Lean Mainland

The default tenant footprint stays in a standard Tier 3 data-plane cell shard. Regulated slices are carved into compliance islands: tenant sub-scopes, data products, or workflow execution envelopes with their own eligible cell pool and audit evidence.

A parent tenant may therefore have:

- `tenant-acme`: standard business/product workloads in standard cells.
- `tenant-acme.payments-cde`: cardholder-data environment in PCI-certified cells.
- `tenant-acme.kyc`: KYB/KYC artifacts and sanctions/PEP evidence in a compliance cell.
- `tenant-acme.identity-sensitive`: identity proofing and high-assurance credential artifacts in a compliant identity envelope.
- `tenant-acme.eu-ai-high-risk`: EU AI Act high-risk execution and evidence in EU-sovereign compliant cells.

The tenant sub-scope pattern uses the existing `tenant_id` / `parent_tenant_id` model. It avoids whole-tenant over-placement while preserving Cedar, audit, FinOps, and DSAR rollup to the parent tenant.

## 2. Cell Classes

| Class | Purpose | Default certification posture | Typical tenants/workloads | Cost posture | Placement rule |
|---|---|---|---|---|---|
| `standard-general` | Lean default Tier 3 data-plane cells for non-regulated and baseline SaaS workloads. | General baseline such as SOC 2 / ISO 27001 controls, per-cell audit, TLS, per-tenant encryption, static-stability cache. | Consumer/B2B product data without CDE, PHI, sovereign-only, public-sector, or high-risk AI obligations. | Lowest steady-state cost; shared capacity; normal DR tier. | Eligible when no active pack/data class/workload requires a stricter certification, provider, DR floor, or freshness mode. |
| `standard-privacy` | Standard cells that can host baseline privacy obligations where the general certification matrix permits them. | General-purpose plus jurisdiction overlays such as GDPR/CCPA/CPRA when no sovereign-provider or special-category elevation is required. | Ordinary personal data, consent state projections, DSAR-ready product data. | Slightly higher evidence/audit storage cost, still standard cell economics. | Eligible only if residency and pack rules allow general-purpose placement. |
| `compliant-pack-bound` | Tier 3 cells carrying one or more compliance certifications from ADR-0251. | Cell certification set includes pack requirements such as `pci-certified`, `hipaa-certified`, `eu-sovereign`, `kr-csap`, `fedramp-high`, `finance-sox`, or equivalent current matrix labels. | CDE, PHI, regulated public-sector data, KR/EU/US-Gov sovereign workloads, high-risk AI, SOX/GLBA financial controls. | Higher substrate, evidence, audit, personnel, key-management, and DR costs. | Required when `compliance_pack.cell_eligibility.requires_certification` or data-class/provider constraints demand it. |
| `compliant-dedicated` | Dedicated or reserved-capacity compliant cells for one tenant/sub-scope. | Same as pack-bound plus physical/logical tenant pinning and explicit capacity reservation. | Large enterprise, regulator-mandated isolation, extreme CDE/PHI/FedRAMP/IL workloads. | Highest cost; reserved-capacity pricing. | Use only when pack, contract, regulator, or capacity evidence requires dedicated cell pinning. |
| `service-evidence` | Service cells for audit aggregation, compliance evidence export, regulator portals, analytics aggregation, and ops consoles. | Service-cell certifications match the evidence they aggregate; they do not host product hot-path data. | Regulator evidence packets, audit-chain rollups, compliance dashboards. | Cost attributed to pack/evidence overhead, not product serving. | Receives async evidence only; never becomes a product data store. |

Tier 2 control-plane cells remain authoritative for tenancy, cell registry, policy/pack registry, identity authority, and compliance certification metadata. Tier 3 compliant cells consume signed snapshots and continue serving within static-stability bounds; they do not call Tier 2 on the request hot path.

## 3. Placement Inputs and Hard Constraints

A placement decision consumes these inputs:

1. Tenant axes:
   - `tenant_id`, `parent_tenant_id`, `audience_type`.
   - `home_cell`, `dr_cell`, and any sub-scope cell binding.
   - `jurisdiction.primary` and `jurisdiction.data_residency_allowed`.
   - `sovereign_cloud_pack`.
   - `kyc_status` and merchant/payment capability flags.
   - `compliance_packs`.
   - `provider_credential_mode`, `policy_evaluation_mode`, `ontology_read_mode`, `freshness_floor`, and `business_continuity_dr_tier`.
2. Pack axes:
   - `cell_eligibility.requires_certification` and `forbidden_cells`.
   - `provider_byok_required` and `encryption_byok_required`.
   - `data_class_extensions` and retention/consent/cross-tenant rules.
   - `jurisdiction_overlay` and sovereign provider restrictions.
   - DR floors from `specs/compliance-pack-floors.json`.
3. Workload axes:
   - Data classes touched by the operation.
   - Whether the workload is CDE, KYC/KYB, identity proofing, PHI, public-sector, high-risk AI, financial reporting, or ordinary product data.
   - Whether the action is hot-path request serving, async workflow, evidence export, analytics aggregation, or cross-tenant/cross-cell coordination.
   - Declared cell placement class / capacity model where present.
4. Cell axes:
   - Cell tier, provider, region, AZ, capacity headroom, health, certification levels, certification expiration, pack evidence freshness, HSM/KMS posture, and PUE/cost tags.
5. Evidence axes:
   - Signed pack version, cell certification evidence, provider certifications, BAA/DPA/SCC references, DPIA references, audit-chain receipts, and DR drill receipts.

Hard constraints are evaluated before cost or utilization scoring:

- Residency and sovereign-provider constraints are hard denies.
- Required cell certifications are hard constraints.
- `forbidden_cells` and forbidden provider egress are hard denies.
- KYC/KYB and pack-install prerequisites are hard gates; the system must block or route to onboarding rather than silently elevating placement.
- DR floors tighten the effective RTO/RPO; a candidate cell pair that cannot meet the stricter floor is ineligible.
- Stale certification, stale topology telemetry, or stale pack registry snapshots block new placements and migrations.
- Cross-cell hot-path tenant data reads remain forbidden.

## 4. Placement Algorithm

Placement is deterministic and audit-emitting:

1. Classify the workload envelope.
   - `standard`: no active pack/data-class/workload trigger beyond general baseline.
   - `standard-privacy`: ordinary personal data under baseline privacy overlays and general certification.
   - `compliant-pack-bound`: one or more packs/data classes require elevated certification or provider controls.
   - `compliant-dedicated`: the pack, customer contract, regulator, or capacity model requires dedicated cell pinning.
   - `service-evidence`: async evidence/audit/analytics/regulator export only.
2. Compute the required control vector.
   - Union baseline controls, jurisdiction overlays, active pack fragments, data-class restrictions, workload kind, DR floor, BYOK requirements, and freshness requirements.
   - Resolve conflicts by stricter-rule-wins and deny-wins.
3. Decide scope of elevation.
   - If only a workload/data class is regulated, create or reuse a tenant sub-scope for that workload/data class.
   - If every workload under the tenant is regulated by the same pack set, place the root tenant itself in the compliant pool.
   - If packs are incompatible for co-residency, split into separate sub-scopes and separate cell bindings.
4. Resolve the eligible cell pool.
   - Start from Tier 3 cells in the required region/provider/jurisdiction pack.
   - Intersect with cell certifications required by the active pack set.
   - Remove forbidden cells/providers and cells with expired/expiring certification inside the migration window.
   - Filter by capacity headroom, health, HSM/KMS posture, DR-pair availability, and telemetry freshness.
5. Score eligible cells.
   - Cost, PUE, capacity headroom, latency, read-replica freshness, and operational load are tie-breakers only after hard constraints pass.
   - Cost must never override compliance eligibility.
6. Shuffle-shard inside the eligible pool.
   - Use ADR-0248's deterministic tenant-seeded shuffle within the eligible pool.
   - Select `home_cell`, `dr_cell`, and read replicas from the same compliant/standard class unless the pack explicitly allows mixed-class read replicas.
7. Emit placement evidence.
   - Emit a placement decision row with tenant/sub-scope, pack set, required control vector, eligible pool hash, chosen home/DR/read cells, cost class, freshness mode, and blocker/exception state.
8. Publish cell bindings by snapshot.
   - Tier 2 remains authoritative; Tier 3 cells pull signed snapshots and serve from cache inside static-stability limits.

## 5. Workload Placement Matrix

| Workload/resource | Default location | Elevation trigger | Required cell class | Data crossing rule |
|---|---|---|---|---|
| Ordinary product records | Parent tenant standard shard. | Special data class, sovereign pack, high-risk workflow, or tenant pack activation. | `standard-general` or `standard-privacy`. | No cross-cell hot-path reads; async rollup only. |
| Consumer/B2B personal data | Standard privacy-capable shard if residency permits. | Special category, child/minor, health, biometric, jurisdiction-specific sovereign data, or pack requiring certification. | `standard-privacy` or matching `compliant-pack-bound`. | Jurisdiction overlay and Cedar pack fragments decide. |
| CDE / PAN / CVV / payment auth data | Never in ordinary product cells. Product cells receive tokens and payment state projections only. | Any cardholder data or PCI-DSS pack. | PCI-certified compliant sub-scope; often `tenant.<id>.payments-cde`. | Tokenization boundary is the only standard-cell interface; raw CDE does not leave PCI cells. |
| KYC/KYB documents, sanctions, PEP, source-of-funds | Identity/compliance sub-scope; standard cells receive status/projection only. | Merchant/payment/settlement capability, enhanced due diligence, KR-FSS/GLBA/SOX/marketplace facilitator flows. | Financial/compliance certified sub-scope; higher KYC can require dedicated controls. | Product cells consume verified/pending/rejected claims, not raw documents. |
| Identity auth runtime: sessions, passkeys, JWKS cache | Tier 2 identity authority plus cell-local Tier 3 caches in every serving cell. | High-assurance identity proofing, regulated personnel constraints, pack-required network evaluation or HSM posture. | Authority in Tier 2; sensitive proofing artifacts in compliant identity sub-scope; cell-local caches in all eligible serving cells. | Caches may validate tokens locally; new identity proofing or key ceremonies require fresh authority. |
| Workload identity and signing roots | Tier 2 authority, per-cell SPIRE/HSM projection. | Pack requires FIPS/HSM/personnel/sovereign-key residency. | Matching compliant control/data cells for the tenant sub-scope. | Signing keys never fall back to a non-certified cell. |
| PHI / healthcare data | Health sub-scope. | HIPAA, healthcare-sovereign, country medical-law packs. | `hipaa-certified` / `healthcare-sovereign` as applicable. | Standard cells receive minimum-necessary projections only. |
| High-risk AI prompts, outputs, evaluations | Standard AI substrate for ordinary prompts; compliant AI sub-scope for regulated data. | EU AI Act high-risk, PHI/CDE in prompt context, sovereign LLM provider restrictions, provider-BYOK-required pack. | Matching compliant pack-bound cell; provider routing by Cedar. | Prompt logs and embeddings stay in eligible cell/provider; product cells receive redacted result projections. |
| SOX/GLBA financial controls | Standard finance workflows for non-public/non-regulated bookkeeping. | SOX-404, GLBA, public-company reporting, payment settlement. | `finance-sox` or financial compliant sub-scope. | Audit/event retention follows stricter pack floor. |
| Audit-chain events | Cell-local audit shard plus async Tier 2/audit-aggregator rollup. | Pack-specific evidence cadence, regulator portal, legal hold. | Source cell class plus `service-evidence` cells for aggregation. | Evidence rows may cross cells asynchronously; protected payloads do not. |
| Analytics / dashboards | Aggregate in service cells from redacted/aggregated feeds. | Regulated cohorts, minimum cohort thresholds, pack-specific analytics restrictions. | `service-evidence` / analytics service cells with matching pack constraints. | No direct hot-path product data reads from analytics cells. |
| Cross-tenant workflow | Async durable workflow, Cedar-gated. | Either tenant side has stricter pack/data class. | Execute in the stricter side or split into projections. | Every cross-cell coordination permit is audited. |

## 6. Per-Pack Control Elevation

Pack activation does not equal whole-platform elevation. Each pack contributes control deltas to the required control vector. Examples:

| Pack/control family | Elevation applied | Scope of elevation |
|---|---|---|
| PCI-DSS / CDE | PCI-certified cells, CDE segmentation, tokenization, ASV/vuln evidence, cardholder-data audit class, stricter network policy, CDE-only key material. | CDE sub-scope and tokenization boundary; ordinary commerce UX can stay standard with tokens. |
| HIPAA / PHI | HIPAA-certified cells, BAA inventory, minimum-necessary access, PHI audit streams, breach workflow, HIPAA DR floor. | PHI sub-scope and health workflows; support/product cells receive minimum-necessary projections. |
| FedRAMP / DoD IL | US government provider/personnel restrictions, FIPS/HSM posture, continuous monitoring evidence, stricter control-plane access. | Government tenant/sub-scope; no fallback to commercial standard cells. |
| KR-CSAP / KR-FSS / KR-PIPA | KR sovereign providers, KR-resident data/personnel where required, CSAP/KISA evidence, KR-specific breach/consent flows. | KR sovereign sub-scope; non-sovereign data may stay standard only if overlay allows fallback. |
| EU-GDPR / EU-NIS2 / EU-DSA / EU AI Act | EU-resident provider constraints where sovereign pack requires, DPA/SCC evidence, NIS2/DSA cadence, high-risk AI oversight/logging. | EU data/high-risk AI sub-scopes; ordinary global product data can stay standard if lawful basis and transfer controls permit. |
| SOX / GLBA / financial controls | Segregation-of-duties Cedar fragments, 7-year retention for finance processes, immutable approval evidence, stricter RPO for journals. | Finance/control sub-scope; non-finance product workflows stay standard. |
| ISO/SOC baseline | Baseline evidence, access review, backup drill, vulnerability scan, deploy receipt. | Standard cells and all compliant cells. |

Control deltas can only narrow behavior. Tenant overrides may add restrictions but cannot relax pack or jurisdiction obligations. A cell can host a union of packs only when its certification set satisfies every pack and no pack forbids co-residency.

## 7. Shared-Responsibility Provider Evidence

Every compliant placement produces a shared-responsibility evidence packet. The packet is not a marketing certification claim; it is operational proof of the placement decision and provider boundary.

Minimum packet fields:

- Placement identity: tenant/sub-scope, parent tenant, active pack set, data classes, workload envelope, action class.
- Cell identity: home/DR/read cells, provider, region, AZ, cell class, cell certification levels, certification expiration dates.
- Provider responsibility evidence: provider certifications, region/provider residency proof, personnel-residency proof where required, provider contract/BAA/DPA/SCC references, provider outage/failure mode class.
- Oyatie responsibility evidence: pack signature/version, Cedar fragment bundle hash, cell-binding snapshot hash, KMS/HSM partition evidence, SPIFFE/SPIRE trust bundle version, workload isolation mode, DR drill receipt, backup/retention posture, audit-chain stream class.
- Tenant responsibility evidence: pack install request, KYB/KYC verification status, tenant-signed DPIA/BAA/DPA where applicable, BYOK/provider credential mode, tenant-admin approvals.
- Operational evidence: cross-provider deny counts, cross-cell permits, regulator evidence cadence, observability freshness, cost/PUE tags, capacity headroom, migration/rollback state.

Evidence is emitted through audit-chain and consumed by compliance evidence automation, auditor/regulator portals, FinOps, and observability. Manual spreadsheets are not the source of truth.

## 8. Cost Model

Cost is computed after eligibility, never before. The platform minimizes cost by narrowing the elevated scope rather than by weakening controls.

Cost dimensions:

- Base cell cost: compute, memory, storage, network, HSM/KMS, observability, and audit storage for the selected cell class.
- Compliance uplift: pack-specific evidence, extra logging, vulnerability/pen-test cadence, auditor access, personnel constraints, certified provider premiums, legal workflow artifacts.
- DR uplift: effective RTO/RPO uses the stricter of tenant/workload declaration and pack floor. T1/T2/T3/T4 multipliers follow the DR standard (+100%, +20%, +10%, +5% over primary as a planning baseline).
- Provider/PUE dimension: provider cost and PUE tags come from ADR-0240/ADR-0174 surfaces; within an eligible pool, lower cost and lower carbon can win tie-breaks.
- Freshness uplift: stricter freshness modes require more frequent signed snapshots, network evaluation, or attested fallback, increasing control-plane and cache costs.
- Dedicated-capacity uplift: dedicated compliant cells are priced as reserved capacity plus compliance uplift.

Allocation dimensions:

- `tenant_id` and sub-scope.
- `compliance_pack` and pack version.
- `cell_id`, provider, region, and cell class.
- Workload envelope and data class.
- Evidence/audit stream class.

Decision rule:

1. If a workload has no hard compliance trigger, keep it standard.
2. If only a sub-scope is regulated, charge the compliance uplift to that sub-scope/pack, not the parent tenant's whole footprint.
3. If a compliant cell's utilization falls below an economic floor, consolidate only among cells with identical or compatible certification/provider/control vectors.
4. Dedicated cells require explicit customer/regulator/contract/capacity evidence.

## 9. Freshness Model

Freshness protects correctness without making Tier 3 hot paths depend on Tier 2 availability.

Freshness tiers:

| Freshness tier | Typical mode | Eligible workloads | Behavior when stale |
|---|---|---|---|
| `standard-cache` | `library_first` with normal cell-local caches. | Ordinary standard workloads. | Continue within ADR-0248 static-stability window; new placements and pack activations still require fresh Tier 2 authority. |
| `attested-cache` | `library_first_with_attested_fallback` or equivalent signed snapshot threshold. | Regulated reads/actions where cached decisions are allowed briefly. | If cache age exceeds threshold, require attested network fallback or block the sensitive action. |
| `network-required` | `network_only` for policy/ontology/control operations. | Pack activation, migration, key ceremonies, high-risk KYC/identity proofing, regulator evidence mutation. | If Tier 2/control authority unavailable, block new mutation; existing serving continues only where the pack allows cached operation. |
| `fresh-projection` | `ontology_read_mode = library_first_with_freshness_floor`. | Regulated projections and analytics. | If projection is older than `freshness_floor`, refresh through the eligible compliant path before serving; do not serve stale regulated projection from a standard cell. |

Baseline snapshot cadences inherit ADR-0248 unless a pack tightens them:

- Cedar fragment bundles: frequent signed snapshots for data-plane evaluation.
- Tenant/cell binding snapshots: pulled into Tier 3 caches for static stability.
- Compliance pack fragments: signed pack snapshots cached in eligible cells.
- Identity/JWKS/SPIFFE bundles: cached locally with bounded validity.

New placement, pack activation, downgrade, certification renewal, or migration cutover requires fresh authority even when existing request serving can continue from cached state.

## 10. Migration Gates

Migrations are controlled workflows, not ad hoc cell edits.

### 10.1 Standard to Compliant

1. Trigger: pack activation, new regulated data class, contract/regulator request, certification expiry, workload reclassification, or detected misplacement.
2. Scope: identify whether the root tenant or a sub-scope/data product/workflow envelope must move.
3. Eligibility dry run: compute required control vector, eligible cell pool, home/DR/read candidates, DR floor, cost estimate, and blocker list.
4. Evidence prerequisites: pack signature, KYB/KYC, DPIA/BAA/DPA/SCC, provider evidence, BYOK/HSM posture, and regulator references.
5. Provision: create/reuse compliant sub-scope and allocate home/DR/read cells by shuffle within the eligible pool.
6. Backfill: copy or re-materialize regulated data into the compliant sub-scope; tokenize/redact standard-cell projections.
7. Shadow: run read shadow, policy shadow, audit shadow, and reconciliation samples.
8. Cutover: route writes to compliant `home_cell`; freeze standard-cell writes for the regulated data class.
9. Verify: compare data integrity, Cedar decisions, audit emissions, DR posture, and user-story smoke evidence.
10. Retain/erase: apply retention/legal-hold rules to old standard-cell copies, then delete/tokenize/crypto-shred per pack rules.
11. Emit: `CompliancePlacementMigrated` evidence with rollback window and verifier results.

### 10.2 Compliant to Standard / Downgrade

Downgrade is allowed only after pack uninstallation or data-class reclassification is legally complete:

- All data classes contributed by the pack are erased, reclassified, tokenized, or retained under legal hold in a compliant evidence store.
- Tenant DPO/ops-compliance/legal approvals are recorded when required.
- No open audit, legal hold, breach workflow, regulator inquiry, or retention minimum requires compliant storage.
- Standard target cell eligibility is recomputed from scratch.
- Downgrade emits evidence and preserves old audit streams for required retention.

### 10.3 Certification Expiry or Provider Loss

If a cell certification expires or a provider loses required eligibility:

1. Stop new placements into the affected cell class.
2. Mark affected tenants/sub-scopes as migration-required.
3. Prefer cells with identical certification/provider vectors.
4. If no eligible pool exists, block activation or brown-out regulated workloads; do not fail open to standard cells.
5. Emit provider/certification evidence and customer/regulator notification according to pack cadence.

### 10.4 Capacity Rebalance

Auto-rebalance may move regulated sub-scopes only among cells with compatible required control vectors. Capacity cannot justify moving regulated data to a leaner class. Cost optimization can consolidate compliant cells only after evidence proves certification, provider, DR, freshness, and pack compatibility.

## 11. Admission and Validation Gates

Future implementation should add or reuse gates in the current governance model; this plan does not add them.

Required gate semantics:

- `pack-cell-certification-coherence`: every active pack's `requires_certification` is present on selected cells.
- `tenant-pack-cell-pinning`: tenant/sub-scope `home_cell` and `dr_cell` match pack/jurisdiction/provider constraints.
- `regulated-subscope-isolation`: CDE/KYC/identity-sensitive data is not co-mingled with standard product data unless the cell class supports the union.
- `standard-cell-no-heavy-controls`: standard cells do not receive pack-heavy resources unless the pack allows general certification.
- `provider-residency-evidence`: selected provider/region appears in the sovereign overlay and has fresh evidence.
- `dr-floor-satisfaction`: effective DR posture satisfies pack floors and tenant/workload declarations.
- `freshness-mode-satisfaction`: regulated actions respect policy/ontology freshness requirements.
- `migration-evidence-complete`: migration cutover, rollback, erase/tokenize, and post-cutover evidence exist.
- `cost-attribution-pack-dimension`: FinOps can allocate compliance uplift by tenant/sub-scope/pack/cell/provider.

## 12. Failure Modes and Required Responses

| Failure mode | Response |
|---|---|
| No eligible compliant cell for requested pack | Block pack activation or workload launch; surface missing certification/provider/capacity evidence. |
| Tenant has CDE in a standard cell | Freeze CDE writes, create incident, migrate to PCI sub-scope, tokenize/delete standard copy, emit evidence. |
| KYC documents leaked into product cells | Freeze affected projections, move documents to compliant KYC sub-scope, leave only status claims in product cells. |
| Identity proofing requires fresh authority but Tier 2 is unavailable | Block new proofing/key ceremony; continue existing token validation only within cache validity. |
| Certification expires on compliant cell | Stop new placements; migrate affected sub-scopes or brown-out regulated functionality. |
| Cost optimizer proposes cheaper non-compliant cell | Reject as ineligible; cost only scores after hard constraints. |
| Freshness stale for regulated projection | Refresh through eligible compliant path or block; never serve stale projection from a standard cell. |
| Incompatible packs on one tenant | Split into sub-scopes or deny co-residency; do not union packs unless cell certification and pack rules permit it. |

## 13. Acceptance Checklist for REGCLOUD-001

- [x] Defines standard and compliant cell classes.
- [x] Defines CDE, KYC/KYB, and identity workload placement.
- [x] Defines per-pack control elevation with strictest-rule-wins.
- [x] Defines shared-responsibility provider evidence.
- [x] Defines cost model that keeps non-regulated workloads lean.
- [x] Defines freshness model for cached, attested, and network-required paths.
- [x] Defines migration gates for standard-to-compliant, compliant-to-standard, certification expiry, provider loss, and capacity rebalance.
- [x] Avoids product-code mutation, new CLI, and oya/cloud reorg debt.

## 14. Downstream Implementation Notes

Likely downstream work, if authorized separately:

1. Promote the placement control vector into a machine-readable spec or schema fragment.
2. Add per-tenant-sub-scope placement examples to tenant-model fixtures.
3. Add pack/cell/provider/freshness validation lanes in the cloud-ci governance substrate.
4. Add FinOps dashboard dimensions for compliance uplift by pack/sub-scope/cell/provider.
5. Add migration runbook templates for CDE, KYC, identity-sensitive, PHI, and sovereign-pack moves.
6. Add regulator evidence packet schemas for placement decisions and provider shared-responsibility evidence.

Those are follow-up tasks. They should be created as separate Kanban cards rather than folded into this planning/spec-only slice.
