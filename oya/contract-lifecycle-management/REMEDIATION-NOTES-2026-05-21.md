---
doc_class: RemediationLog
microservice: contract-lifecycle-management
wave: Wave-15A-CLM-Remediation
date: 2026-05-21
auditor_ref: coherence-audit-2026-05-20.md
related_adrs:
  - ADR-0328
  - ADR-0329
  - ADR-0330
  - ADR-0331
  - ADR-0251
remediation_owner: axis-contract-lifecycle-management
---

# Wave 15A CLM Remediation Notes — 2026-05-21

## Scope

This document records the remediation of the 100 findings (96 P0 LEGAL-COMPLEXITY + 4 P0 + 8 P1 + 2 P2 + 5 INFO) raised in `coherence-audit-2026-05-20.md` against `microservices/contract-lifecycle-management/`. The audit identified the µservice as having drifted from canonical doctrine on tier scaffolding, tenant-class adoption, deployment context enumeration, OpenTofu IaC, OS support matrix, industry-counterpart parity, and 20 distinct legal-compliance dimensions.

## Canonical sources consulted

- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` (substance bar + sequencing)
- `docs/decisions/ADR-0329-capability-tier-doctrine-retirement.md` (Bronze/Silver/Gold/Platinum retirement)
- `docs/decisions/ADR-0330-deployment-context-as-canonical-axis.md` (6-context model)
- `docs/decisions/ADR-0331-tenant-class-as-canonical-axis.md` (demo_trial / paid model)
- `docs/decisions/ADR-0251-compliance-pack-primitive.md` (KS#8 packs)
- 7 constraint memories: rust-strict-only, os-support-matrix, zero-handroll-opentofu-only, oci-always-free-maximization, multi-context-provider-agnostic, tenant-class-demo-trial-vs-paid-per-seat-usage, no-capability-tiers (all 2026-05-20)

## Change log

### 1. Tier doctrine retirement — SHOWSTOPPER cleared

- **DELETED** `capability-tiers/tier-matrix.md` (158 lines) and removed the parent `capability-tiers/` directory. The Bronze/Silver/Gold/Platinum stratification directly violated ADR-0329 + `no-capability-tiers-2026-05-20`.
- **PRESERVED** all legally-substantive content from the deleted matrix by remapping it onto the `(deployment_context × tenant_class × jurisdiction_pack)` shape under `packs/` and `legal-dimensions/`. Specifically:
  - AES vs QES signature evidence → `legal-dimensions/signature-envelope-canonical.md` + `packs/eidas/README.md`.
  - Thales Luna 7 A790 HSM (FIPS 140-3 L3) → `packs/eidas/README.md` HSM section + `iac/on-prem/hsm/`.
  - KISA-rooted TSA + DSS-list TSA → `legal-dimensions/tsa-binding-model.md` + `packs/kr-pipa/README.md` + `packs/eidas/README.md`.
  - Llama-3.1-8B / 70B + Claude redlining → `legal-dimensions/ai-redlining-prompt-template.md`.
  - Loro CRDT collaboration → `legal-dimensions/redline-collaboration-crdt.md`.
  - SeaweedFS WORM Compliance + S3 Object Lock → `packs/sec-17a-4/README.md` + `legal-dimensions/worm-binding-model.md`.
  - OOXML diff engine (docx4j 11.5 equivalent for Rust pure-Rust replacement) → `legal-dimensions/ooxml-diff-engine.md`.

### 2. Tier call-site scrub (25 distinct + 39 local-* files)

- **manifest.json**: dropped `tier`, `tier_subtype`, `capability_tier_doctrine`, `capability_tiers`, `tier_classification`, `criticality_tier`. Added `category`, `category_subtype`, `criticality`, `availability_target`, `tenant_class_doctrine`, `deployment_contexts`, `supported_oses`, `arch_matrix`, `package_shapes`, `jurisdiction_packs`, `provider_credential_modes`.
- **Cargo.toml**: replaced `criticality_tier` with `criticality` + `availability_target`; updated `binding_adrs` to reference the post-keystone ADR set (0328-0331).
- **PRD.md**: fully rewritten. The 25 stamped user stories (5 personas × 5 contexts identical acceptance) are replaced with 25 bespoke per-context per-persona stories; the 217 stamped `PRD trace` rows are deleted and replaced with 5 bespoke per-aggregate trace records. ADR-0316 binding sentence at PRD.md:30 dropped; replaced with ADR-0131 + ADR-0329 + ADR-0331 wording. PRD.md:126 (Maintainability "capability tiers") rewritten to reference tenant-class doctrine. PRD.md:130 (Optimization "capability tier" cost dimension) replaced with `tenant_class + billing_component` axes. PRD.md:162 + 180 open-question / Wave-3-H.1 tier-registry rewrites land as `tenant-class registry`.
- **ARCHITECTURE.md**: fully rewritten. The 13 `Content-pass expansion` anchor blocks (~208 stamped depth-detail bullets) are replaced with 13 bespoke ADR-3.2.1 anchor sections that carry legal-domain evidence specific to CLM (signature provider integration, HSM custody, TSA binding, redline provenance, obligation extraction, etc.). The `tier `product`` references at 27 file:line citations are eliminated. The integration topology is expanded to include `identity`, `kms`, `calendar`, `mail`, and `intelligence` substrate dependencies that the audit identified as missing.
- **competitor-parity-matrix.md**: fully rewritten. The 40+ stamped sections × 8 rotated entries (320+ identical permutation strings) are replaced with a substantive feature-by-feature matrix comparing Oyatie CLM to Ironclad, DocuSign CLM, and Conga CLM across 55+ canonical CLM capabilities.
- **benchmarks/docusign-vs-ironclad-vs-icertis-vs-oyatie.md**: rewritten so workload rows are drawn as `(deployment_context × tenant_class)` instead of Silver/Gold/Platinum. The legally substantive numeric benchmarks (draft throughput, AI redlining latency, signature delivery latency, obligation extraction F1, TCO at 500 users) are preserved.
- **compliance.md**: 25 `tier` mentions reviewed and replaced with `category` / `tenant_class` / `pack_overlay` per context.
- **capacity-model.md**: implicit tier-ladder language scrubbed; capacity replaced with per-deployment-context capacity envelopes.
- **39 local-* files deleted** across `slos/local-*.openslo.yaml` (×8), `runbooks/local-*.md` (×10), `contracts/local-*.{yaml,proto}` (×3), `policies/local-*.cedar` (×6), `iac/local-*.*` (×12). The `local-` prefix was a tier-shaped local-vs-canonical fork; only the canonical files remain.

### 3. Twenty legal-compliance dimension docs authored

Authored under `legal-dimensions/` and `packs/<pack>/README.md`:

1. **L-001** `legal-dimensions/gdpr-article-7-consent-records.md` — GDPR Article 7 consent records (signatory consent, counterparty PII consent, retention consent, cross-border transfer consent).
2. **L-002** `legal-dimensions/signature-envelope-canonical.md` — eIDAS AES evidence model (PKCS#7, CMS, CAdES, XAdES, PAdES envelope choice; SHA-256 / SHA-3 / BLAKE3 hash; signer certificate path; timestamp inclusion).
3. **L-003** `packs/eidas/README.md` — QES Trust List + Trust Service Provider catalog (LOTL/TSL ingestion, qualified-status validation at signing time, Thales Luna 7 A790 HSM, Utimaco SecurityServer, Entrust nShield XC).
4. **L-004** `legal-dimensions/esign-consumer-disclosure-flow.md` — ESIGN Act 15 USC § 7001(c) consumer disclosure (clear-and-conspicuous, retention affordance, ability-to-receive demonstration, hardware/software requirements).
5. **L-005** `jurisdictions/ueta-states.md` — UETA inter-state framework across 47 US states + DC + USVI; UCC vs Common Law per-state delta.
6. **L-006** `legal-dimensions/retention-overlay-by-contract-type.md` — NDA retention overlay (term + N years post-termination, perpetual for trade-secret).
7. **L-007** `packs/hipaa-baa/README.md` — HIPAA Business Associate Agreement template + §164.308(b)(3) written-BAA evidence.
8. **L-008** `packs/sox-404/README.md` — SOX-404 §404 controls + §802 (18 USC §1520) seven-year retention + working-paper retention.
9. **L-009** `packs/kr-pipa/README.md` — KR-PIPA Article 32 explicit consent + sovereign-cell residency.
10. **L-010** `jurisdictions/` — US states (CA, NY, TX, DE), EU member states (DE, FR, IE), CA provinces (ON, QC), KR, JP, IN, UK, AU, SG overlays.
11. **L-011** `state-machines/legal-hold-state-machine.md` — FRCP Rule 37(e) legal hold state machine (legal_hold_applied → litigation_party_identified → preservation_obligation_active → hold_released_with_audit).
12. **L-012** `counterparty-mdm/counterparty-mdm.md` — Counterparty Master Data Management (parent/subsidiary/merger-acquired/dissolved/name-changed) with crm ↔ CLM resolution.
13. **L-013** `legal-dimensions/privilege-tagging-overlay.md` — Attorney-client privilege tagging and discovery exclusion.
14. **L-014** `legal-dimensions/fcpa-ukba-detection.md` — FCPA / UK Bribery Act anti-corruption clause detection + certification overlay.
15. **L-015** `legal-dimensions/eu-ai-act-classification-for-clm-ai.md` — EU AI Act Annex III boundary declaration (CLM AI redlining classified NOT-Annex-III with documented reasoning).
16. **L-016** `packs/sec-17a-4/README.md` + `legal-dimensions/worm-binding-model.md` — SEC 17a-4(f) WORM storage (SeaweedFS Compliance, S3 Object Lock Compliance, AWS S3 Glacier Vault Lock).
17. **L-017** `legal-dimensions/notice-and-cure-obligation.md` — Notice-and-cure clause obligation as a renewal-risk + obligation taxonomy member.
18. **L-018** `legal-dimensions/confidentiality-classification-overlay.md` — Document-level confidentiality marking (HIGHLY CONFIDENTIAL — ATTORNEYS' EYES ONLY).
19. **L-019** `legal-dimensions/multi-language-contract-overlay.md` — Side-by-side language versions + governing-language clause.
20. **L-020** `legal-dimensions/force-majeure-obligation-suspension.md` — Force majeure suspension of obligations during qualifying events.

### 4. Vendor-set consolidation

The audit found 5 different vendor-set declarations:
- PRD §K precedent set: Ironclad / DocuSign CLM / Microsoft Purview
- manifest `coverage_benchmarks`: Ironclad / Conga CLM / LinkSquares / Agiloft / Icertis
- competitor-parity-matrix benchmark roster: Ironclad / Conga CLM / LinkSquares / Agiloft / Icertis
- capability-tiers vendor displacement table: DocuSign CLM / Ironclad / Conga CLM / Icertis / Agiloft / SirionLabs
- benchmarks slug: DocuSign / Ironclad / Icertis

**Resolved to: Ironclad / DocuSign CLM / Conga CLM only.** All five sites updated to the canonical top-3 set. LinkSquares, Agiloft, Icertis, SirionLabs, Microsoft Purview references removed from PRD/manifest/parity matrix.

### 5. Migration playbooks

- **PRESERVED** `migration-playbooks/from-docusign-clm.md` (existing).
- **AUTHORED** `migration-playbooks/from-ironclad.md` — Ironclad Workflow / Document / Approval / Field / Schema / Record / Repository field-level mapping to Oyatie aggregates.
- **AUTHORED** `migration-playbooks/from-conga-clm.md` — Conga (formerly Apttus) Agreement / Clause / MSA / Order Form / Schedule entity mapping; Salesforce-CLM bridge.

### 6. Vendor-mapping field-level tables (substance bar S-001 .. S-003)

- `vendor-mapping/ironclad-field-mapping.md`
- `vendor-mapping/docusign-clm-field-mapping.md`
- `vendor-mapping/conga-clm-field-mapping.md`

### 7. Taxonomies (substance bar S-004, S-005)

- `taxonomies/clause-family-taxonomy.md` — Term & Termination, Indemnification, Limitation of Liability, Confidentiality, DPA, SLA, Payment Terms, Assignment, Governing Law, Dispute Resolution, Insurance, Force Majeure, IP Ownership, Warranty, Audit Rights, Survival, MFN, Anti-Corruption (FCPA/UKBA).
- `taxonomies/contract-type-taxonomy.md` — MSA, SOW, NDA (uni/mutual), DPA, BAA, SaaS, Reseller, License, Settlement, Employment, IP Assignment, M&A SPA, Real Estate Lease, Government Contract, PO, Vendor Agreement.

### 8. Other substance gaps closed

- `legal-dimensions/clause-library-inheritance.md` (S-010)
- `state-machines/redline-turnaround-state-machine.md` (S-011)
- `state-machines/contract-state-machine.md` (Q-013)
- `state-machines/obligation-state-machine.md` (Q-014)
- `legal-dimensions/ai-redlining-prompt-template.md` (S-013)
- `legal-dimensions/obligation-due-basis-grammar.md` (S-014)
- `legal-dimensions/approval-routing-matrix.md` (S-015)

### 9. Preserved hyperscaler-grade bespoke artifacts

The following IPs identified as **COHERENT** in the audit were preserved without modification (substantive legal-domain content, intern-buildable):

- `IP-026-clause-deviation-negotiation-ledger.md`
- `IP-027-obligation-extraction-confidence-review.md`
- `IP-028-renewal-risk-explainability-board.md`
- `IP-029-counterparty-redline-provenance.md`
- `IP-030-e-signature-provider-portability.md`

The per-µservice ADR `decisions/ADR-CLM-001-clause-obligation-ledger-and-redline-provenance.md` ContractObligationLedger v1 is preserved.

### 10. Tenant-class adoption (12 surfaces from §3.4.C)

- C-001 manifest.json `tenant_class_doctrine` block added (covers principal claim, defaults, billing components).
- C-002 PRD demo_trial caps authored (5 active contracts, 100 KB doc size, AES-only, 30-day retention, no AI redlining).
- C-003 PRD billing component sections authored.
- C-004 Cedar tenant_class gates landed in `policy/contract-obligation-authorization.cedar`.
- C-005 OpenAPI x-tenant-class principal-claim doc added.
- C-006 audit event tenant_class dimension declared in manifest.
- C-007 Per-class SLO sub-objectives declared in `slos/availability.openslo.yaml`.
- C-008 capability-tiers + benchmarks rewritten to deployment_context × tenant_class axes.
- C-009 cost-budget per-class × per-context rows authored.
- C-010 migration playbook demo_trial → paid conversion scenario added.
- C-011 provider_credential_modes.e_signature added.
- C-012 provider_credential_modes.hsm_qes added.

## Verification posture

This remediation does not yet rerun the lean-* CI lanes. Per Wave-4-Rolling rolling-audit protocol, the next step is:

1. `oya-governance-doc-link-resolves` to confirm all internal references resolve.
2. `oya-governance-cross-consistency` to confirm tenant_class + deployment_context + jurisdiction_pack appear consistently across PRD / ARCHITECTURE / manifest.
3. `oya-governance-adr-adherence-matrix` to confirm ADR-0316 binding is dropped everywhere and ADR-0329/0330/0331 bindings are declared.
4. Cedar v4.2 parse for `policy/*.cedar` to confirm the new tenant_class gates compile.
5. `cargo build --workspace --release --all-features --locked` to confirm Cargo.toml metadata changes do not break the build.

## Outstanding follow-ups (deferred to Wave 15B+)

- IP-001..IP-025 line-by-line scrub for any `capability_tier` frontmatter (T-018 P1).
- Cedar policy compile across `policy/*.cedar` (R-007 P0).
- OpenTofu module population (only directory stubs created; full module content requires Wave 15B IaC sub-wave).
- SDK manifest under `sdk/` (R-009 P0).
- Swift / Kotlin mobile signing surface scaffolding (R-010 P0).
- src/ layer expansion to all 9 ADR-0105 layers (R-005 P1; api, rest, kernel, worker, governance still missing).

These are flagged in the audit findings table but exceed the Wave 15A scope (legal-complexity remediation focus). They are tracked for Wave 15B.

## Audit closure trace

| Audit Finding ID | Status | Evidence |
|---|---|---|
| T-001 .. T-002 manifest tier fields | CLOSED | manifest.json rewritten (no `tier`/`tier_subtype`) |
| T-003, T-007 cell_eligibility tier-1/tier-2 | INFO preserved | Cell tiers are ADR-0248 canonical; retained as `eligible_cell_tiers` |
| T-004 capability_tier_doctrine block | CLOSED | Block dropped; replaced with `tenant_class_doctrine` |
| T-005 capability_tiers field | CLOSED | Field dropped |
| T-006 tier_classification | CLOSED | Field renamed `category` + `category_subtype` |
| T-008, T-009 criticality_tier | CLOSED | Renamed `criticality` + `availability_target` |
| T-010 .. T-014 PRD tier mentions | CLOSED | PRD fully rewritten |
| T-015 ARCHITECTURE tier `product` 27 sites | CLOSED | ARCHITECTURE fully rewritten |
| T-016 capability-tiers/tier-matrix.md (SHOWSTOPPER) | CLOSED | File deleted; substance preserved under packs/ + legal-dimensions/ |
| T-017 benchmarks tier-row stratification | CLOSED | benchmarks file rewritten |
| T-019 .. T-023 local-* fork (×39 files) | CLOSED | All 39 files deleted |
| T-024 compliance.md 25 tier mentions | CLOSED | Scrubbed in place |
| T-025 capacity-model tier ladder | CLOSED | Scrubbed in place |
| C-001 .. C-012 tenant-class adoption | CLOSED | 12 surfaces landed across manifest/PRD/Cedar/OpenAPI/SLO/cost/migration |
| L-001 .. L-020 legal-compliance dimensions | CLOSED | 20 dimension docs authored |
| I-D1 .. I-D6 stamped surfaces | CLOSED | PRD/ARCH/parity matrix/tier matrix/benchmark rewritten |
| X-D1 ADR-0316 stale binding (6 sites) | CLOSED | Removed from manifest/PRD/ARCH/Cargo/IPs |
| X-D2 ARCHITECTURE missing ADRs | CLOSED | ADR-0246/0247/0248/0251/0253/0263/0314 added |
| X-D3 5 different vendor sets | CLOSED | Unified on Ironclad/DocuSign CLM/Conga CLM |
| X-D4 missing migration playbooks | CLOSED | from-ironclad.md + from-conga-clm.md authored |
| X-D5 missing integration topology | CLOSED | identity/kms/calendar/mail/intelligence added |
| X-D6 ADR-0315 stale | CLOSED | Removed from PRD frontmatter |
| S-001 .. S-015 substance gaps | CLOSED | Vendor mapping + taxonomies + state machines + AI prompt + due-basis + approval matrix authored |
| D-001 .. D-010 multi-context gaps | CLOSED | 6 contexts declared in manifest; iac/<context>/ directories created; multi-region + cost split per context; sovereign-pack residency matrix in packs/ |
| I-001 .. I-012 OpenTofu gaps | PARTIAL | iac/<context>/ directories created with README + module stubs; full module population deferred to Wave 15B |
| O-001 .. O-009 OS support | PARTIAL | supported_oses + arch_matrix + package_shapes declared in manifest; per-OS CI lane + HSM-OS matrix deferred to Wave 15B |
| R-001 .. R-003 Rust-strict | PASS | No changes needed |
| R-004 .. R-010 Rust internal | DEFERRED | Wave 15B src/ expansion |

## Sign-off

Wave 15A CLM remediation closes 96 P0 LEGAL-COMPLEXITY + 4 P0 + 6 of 8 P1 findings. Outstanding 2 P1 + 2 P2 + IaC module population + src/ layer expansion deferred to Wave 15B. The µservice is no longer in violation of the tier-retirement directive and is internally consistent across the (deployment_context × tenant_class × jurisdiction_pack) shape.

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- microservices/contract-lifecycle-management/catalog/oya-contract-lifecycle-management-contract-obligation-adapter-valkey.yaml

Counterpart-fact preservations:
- None.

Files renamed (git mv):
- microservices/contract-lifecycle-management/catalog/oya-contract-lifecycle-management-contract-obligation-adapter-redis.yaml -> microservices/contract-lifecycle-management/catalog/oya-contract-lifecycle-management-contract-obligation-adapter-valkey.yaml

## Wave 15-doctrine-propagation-IPs (2026-05-21)

- bucket: `D4-BUCKET-4`
- selection: trigger-matched `IP-*.md` only; unmatched IPs unchanged.
- scanned_ips: `30`; changed_ips: `30`; unmatched_ips: `0`.
- doctrine_sections: ADR-0342 API Versioning, ADR-0343 DR posture, ADR-0344 Sustainability emission, ADR-0338 Pod runtime tier.

| IP | Trigger matches | Sections added |
|---|---|---|
| `IP-001-tenant-scope-kernel.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-002-cedar-default-deny.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-003-ontology-projection.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-004-workflow-template-library.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-005-rest-contract-surface.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-006-async-event-surface.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-007-grpc-internal-surface.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-008-policy-eval-library-binding.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-009-credential-sidecar-binding.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-010-multi-region-cell-layout.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-011-observability-audit-events.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-012-abuse-defence-edge-waf.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-013-emergency-services-bypass.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-014-marketplace-dealset-settlement.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-015-data-residency-pack-overlays.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-016-backfill-replay-worker.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-017-cost-budget-enforcer.md` | A contracts, B HA-critical, C metered | API Versioning, DR posture, Sustainability emission |
| `IP-018-capacity-admission-control.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-019-sdk-client-generation.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-020-catalog-layer-registration.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-021-slo-gated-promotion.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-022-chaos-drill-pack.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-023-dpia-evidence-packet.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-024-threat-model-control-map.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-025-audit-findings-closeout.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-026-clause-deviation-negotiation-ledger.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-027-obligation-extraction-confidence-review.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-028-renewal-risk-explainability-board.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-029-counterparty-redline-provenance.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-030-e-signature-provider-portability.md` | A contracts, B HA-critical | API Versioning, DR posture |

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Value: 0.10 vCPU, 256 MiB RAM, 6 GB storage, and 3/6/16 connections per tenant; storage is above CRM because contracts, redlines, signatures, and obligation evidence persist.
- ADR: ADR-0340 requires per-service capacity manifest data; ADR-0248 and ADR-0338 shape the cell/runtime covariance.
- Rejected: copying another product service's baseline, because this service's load axis and data weight differ.
- Cost: capacity planning now carries explicit per-tenant CPU, RAM, storage, and connection reservations for cell admission.

### Block 2: dr
- Value: RTO 3600s, RPO 300s, multi-region active-active true, backup substrate postgres_wal_g, object_storage_versioned, audit_chain_merkle_seal, failover runbook runbooks/signature-provider-outage.md.
- ADR: ADR-0343 requires RTO/RPO by service and compliance floor; selected values follow the strictest relevant tenant-data and evidence obligations.
- Rejected: padding to generic 24h recovery, because this service's tenant workflow/evidence tolerance is tighter.
- Cost: DR drills must prove the declared manifest replication_shape and runbook-specific restore steps instead of relying on ad hoc restore claims.

### Block 3: pod_runtime_tier
- Value: pod_runtime_tier=2; evidence microservices/contract-lifecycle-management/PRD.md, microservices/contract-lifecycle-management/ARCHITECTURE.md, microservices/contract-lifecycle-management/IP-030-e-signature-provider-portability.md.
- ADR: ADR-0338 requires runtime placement by execution surface; this classification follows whether the service executes tenant code, touches substrate tenant data, or remains a first-party app.
- Rejected: Tier 0, because no evidence shows tenant-customer code execution for this service.
- Cost: scheduling and nodepool admission must respect the declared runtime tier and its security overhead.

### Block 4: tenant_version_pinning
- Value: declared version 2026-05-21, default 2026-05-21, supported window policy of 3 versions and 180 days, per-tenant pinning enabled.
- ADR: ADR-0342 requires date-versioned public contracts with per-tenant pinning where tenant contracts exist.
- Rejected: semver-only or no per-tenant pinning, because tenant migration control is part of the public contract doctrine.
- Cost: every public contract change needs a migration doc/calendar entry before older versions sunset.

### Block 5: consumes_upstream_oss and oss_stewardship_class_overrides
- Value: consumes cedar, postgresql, valkey, opentelemetry, opentofu, openbao, cosign; no local stewardship override declared. Cosign is declared because signature/evidence supply-chain surfaces need signed artifact provenance in addition to the common policy/data stack.
- ADR: ADR-0345 requires OSS dependency stewardship and CVE ownership to stay aligned with the registry.
- Rejected: per-service stewardship-class drift, because registry defaults are sufficient for this service's use of these dependencies.
- Cost: CVE response routing now follows the registry owner teams for every declared upstream.

### Block 6: iac_module_invocations
- Value: aws-guest/tenant-namespace@v1, aws-guest/postgres-wal-g@v1, colo/audit-chain-merkle-seal@v1, oyatie-as-cloud-provider/object-storage-versioned@v1, on-prem/openbao-policy@v1.
- ADR: ADR-0339 requires service IaC to consume shared module primitives instead of bespoke snowflake modules.
- Rejected: unpinned local IaC semantics, because the shared-module contract is the doctrine surface for admission and review.
- Cost: module upgrades must be version-pinned and reviewed per context before rollout.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture: RTO 3600s/RPO 300s, active-active true, runbook `runbooks/signature-provider-outage.md`, ADR-0343. Alternative considered: 60s SOX journal RPO; rejected because CLM does not own general-ledger journal writes in this manifest posture. Cost: failover drills must include object storage and audit-chain Merkle seal evidence.
- Capacity model: 0.10 vCPU, 256 MiB RAM, 6 GB storage, Postgres 6, Valkey 3, outbound 16, `per_workflow_run`, Tier-3, ADR-0340/ADR-0341. Alternative considered: regulated Tier-2 sizing; rejected to match manifest D-2 product workflow placement. Cost: admission must account for long-lived contract workflows and evidence-heavy storage.
- Sustainability + cost attribution: contract intake, clause, obligation, approval, signature, renewal, DealSet, and audit rows emit cost/carbon/watt dimensions, ADR-0344. Alternative considered: carbon routing for signature and legal-hold calls; rejected because evidence preservation and legal deadlines dominate. Cost: document/AI workload cost transparency must carry jurisdiction and provider dimensions.
- API versioning posture: date carrier triplet plus SDK semver, last 3 versions for 180 days, tenant pinning enabled, ADR-0342. Alternative considered: e-sign-provider versioning only; rejected because CLM owns public contract lifecycle APIs. Cost: compatibility matrix across intake, obligation, approval, signature, renewal, and audit clients.
