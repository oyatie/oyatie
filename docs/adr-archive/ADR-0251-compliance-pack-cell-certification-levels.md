---
id: ADR-0251
status: Superseded
planning_impact: true
date: 2026-05-20
owners:
  - council-architecture
  - council-privacy
  - council-security
  - council-legal
  - ops-compliance
  - ops-sre-reliability
  - ops-dr-capacity
  - axis-policy-engine
  - axis-audit-chain
  - axis-cell
  - axis-tenancy
  - axis-identity
supersedes: []
amends:
  - ADR-0099-data-class-registry.md (extends data-class taxonomy with per-pack class extensions)
  - ADR-0144-eu-ai-act-graduated-risk-tier-model.md (positions EU AI Act tiers inside the EU-AI-Act compliance pack)
  - ADR-0150-cedar-policy-engine.md (introduces signed-fragment-bundle scope `pack/<pack-id>/`)
superseded_by: [ADR-708]
amended_by: [ADR-0329]
related:
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0010-regional-pack-architecture.md
  - ADR-0028-cloud-microservice-architecture.md
  - ADR-0049-cross-region-replication-and-residency.md
  - ADR-0064-canonical-base-and-localization-packs.md
  - ADR-0099-data-class-registry.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0132-no-grouping-forward-policy.md
  - ADR-0140-cedar-policy-enforcement.md
  - ADR-0144-eu-ai-act-graduated-risk-tier-model.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0174-finops-cost-tag-sustainability.md
  - ADR-0176-brown-out-degradation-signal.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
  - ADR-0188-passkey-webauthn-as-canonical-auth.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0218-tenant-granular-control-surface.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0241-dr-business-continuity-portfolio-policy.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0249-multi-category-marketplace-doctrine.md
  - ADR-0250-build-ahead-of-certification-doctrine.md
  - ADR-0255-intelligence-as-two-layer-ai-substrate.md
  - ADR-0263-observability-emission-contract.md
related_specs:
  - /specs/platform-architecture.json
  - /specs/microservices/policy-engine.json
  - /specs/microservices/governance.json
  - /specs/microservices/audit-chain.json
  - /specs/microservices/tenancy.json
  - /specs/microservices/cell.json
  - /specs/compliance-pack-schema.json
  - specs/derived-compliance-cell-placement-contract.json
  - /specs/cell-certification-level-matrix.json
  - /specs/data-class-registry.json
  - /specs/cedar-fragment-schema.json
related_memory:
  - feedback_compliance_pack_first_class
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_cedar_as_universal_gate
  - feedback_canonical_base_localization
  - feedback_quality_performance_scalability_bar
  - feedback_no_silent_regression
  - feedback_autonomous_implementation_artifacts
  - feedback_automate_everything
  - feedback_doc_coverage_enforced
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-foundational-doctrine
keystone_position: 10-of-14
purpose: >
  Establish Compliance Pack as a first-class versioned signed bundle
  that wraps a single regulation (HIPAA, PCI DSS, FedRAMP, EU GDPR,
  KR-PIPA, KR-FSS, DoD IL5/6, FERPA, FDA 21 CFR Part 11, EU AI Act,
  EU NIS2, EU DSA, JP APPI, SG PDPA, AU Privacy Act, KSA NDMO/SDAIA,
  etc.). Cells declare a SET of certifications (certification levels)
  enumerating which packs they can host. Tenants install packs onto
  themselves; the policy-engine aggregates installed packs at evaluation
  time, deny-wins. Cross-pack traffic is Cedar-gated. The mechanism is
  the unit of regulator response, the unit of evidence emission, and
  the unit of pack-versioned drift.
enforcement_status: advisory-until-pack-registry-substrate-lands
enforced_by:
  - oya gate validate compliance-pack-schema
  - oya gate validate compliance-pack-signature
  - oya gate validate cell-certification-coherence
  - oya gate validate tenant-pack-cell-pinning
  - oya gate validate cross-pack-traffic-cedar-gated
  - oya gate validate baa-dpa-coverage
  - oya gate validate breach-notification-workflow-coverage
  - oya gate validate consent-record-coverage
  - oya gate validate dpia-template-coverage
  - oya gate validate auditor-evidence-emission
---

# ADR-0251: Compliance Pack + Cell Certification Levels

## Status

Proposed — 2026-05-20.

Bundled with the 14-ADR foundational keystone set (ADR-0242 through
ADR-0255 inclusive). Lands as a single multispectrum-reviewed PR.
Partial acceptance is rejected because the keystones are mutually
reinforcing and depend on each other's primitives (Cedar fragments
from ADR-0243, tenant primitive from ADR-0242 + ADR-0244, cell
primitive from ADR-0248, audit-chain substrate from ADR-0246).

Enforcement status is `advisory-until-pack-registry-substrate-lands`.
The doctrine is accepted in text now; the CI lanes that enforce it
move to BLOCKER status only after:

1. `microservices/policy-engine/` is promoted to peer substrate
   µservice (per ADR-0246).
2. `microservices/governance/` provides a fragment-bundle registry
   (Postgres + Citus on `(pack_id, version)` + cosign-attested
   immutable blob in SeaweedFS).
3. At least three baseline packs (`SOC-2-T2-v2024`, `EU-GDPR-2018-
   baseline-v2024`, `KR-PIPA-2023-amendment`) are authored, signed by
   the oyatie compliance-office Ed25519 key, and published.
4. At least three cell certification level definitions (`general`,
   `eu-sovereign`, `hipaa-certified`) are published in
   `/specs/cell-certification-level-matrix.json`.
5. The breach-notification-substrate (per §D-8) provisions per-
   jurisdiction workflow templates and has been drilled at least once
   per `ops-compliance` runbook.

Until those five bootstrap items land, validators emit findings without
failing CI. Post-bootstrap, the lanes promote to BLOCKER.

## Date

2026-05-20.

## Context

### The regulation explosion

Compliance regulations governing modern multi-tenant platforms have
proliferated faster than any platform-team headcount can absorb ad hoc.
At the date of this ADR (2026-05-20), the portfolio is responsible —
either today or under its declared expansion plan — for tenants subject
to at least the following regulatory regimes:

| Regulation / Framework | Effective | Jurisdiction | Scope summary |
|---|---|---|---|
| **HIPAA + HITECH** (45 CFR §164) | 1996 / 2009 | US (federal) | Protected Health Information (PHI), business-associate liability, breach notification |
| **PCI DSS 4.0.1** (PCI SSC, 2024) | 2024-04 | Global card networks | Cardholder data (PAN, CVV, expiry), QSA-assessed |
| **FedRAMP Moderate** (FedRAMP PMO, baseline 2024) | 2024 | US Federal Government | Federal Information Systems Moderate impact |
| **FedRAMP High** (FedRAMP PMO, baseline 2024) | 2024 | US Federal Government | Federal Information Systems High impact |
| **DoD IL4** (DISA SRG v1r5) | 2022 | US Department of Defense | Controlled Unclassified Information |
| **DoD IL5** (DISA SRG v1r5) | 2022 | US Department of Defense | Controlled Unclassified Information + Mission-Critical |
| **DoD IL6** (DISA SRG v1r5) | 2022 | US Department of Defense | Classified up to SECRET |
| **FERPA** (20 USC §1232g) | 1974, amended 2024 | US (federal) | Student education records |
| **FDA 21 CFR Part 11** (FDA, 2024 revisions) | 1997, revised 2024 | US (federal) | Electronic records + e-signatures for FDA-regulated industries |
| **SOX (Sarbanes-Oxley) §404** | 2002 | US (federal) | Public company financial controls |
| **GLBA (Gramm-Leach-Bliley) Safeguards Rule** (16 CFR §314) | 2003, revised 2024 | US (federal) | Financial institution customer data |
| **CCPA / CPRA** (Cal. Civ. Code §1798.100) | 2020 / 2023 | California | Consumer privacy rights |
| **State breach notification** (50 states) | varies | US states | Per-state breach notification timing + scope |
| **EU GDPR** (Reg. (EU) 2016/679) | 2018-05-25 | EU/EEA | Personal data, lawful basis, DSAR, DPIA |
| **UK GDPR + DPA 2018** (UK Parliament) | 2018, post-Brexit retention | UK | Personal data |
| **EU AI Act** (Reg. (EU) 2024/1689) | 2024-08-01 effective; phased application 2025-2027 | EU/EEA | AI-system classification + obligations (per ADR-0144) |
| **EU NIS2** (Dir. (EU) 2022/2555) | Transposition by 2024-10-17 | EU/EEA | Critical-entity cybersecurity |
| **EU DSA** (Reg. (EU) 2022/2065) | Effective 2024-02-17 | EU/EEA | Digital Services Act — content moderation, transparency |
| **EU DMA** (Reg. (EU) 2022/1925) | Effective 2024-03-06 | EU/EEA | Digital Markets Act — gatekeeper obligations |
| **EU Data Act** (Reg. (EU) 2023/2854) | Effective 2025-09-12 | EU/EEA | Data-sharing and switching obligations |
| **Schrems II / EU-US Data Privacy Framework** | 2023-07 adequacy | EU↔US | Transatlantic data transfer |
| **KR PIPA** (개인정보 보호법) | 2011, 2023-09-15 amendment | KR | Personal information, DSAR, DPO |
| **KR 의료법 + 의료정보보호** | 2024 amendment | KR | Healthcare data |
| **KR-FSS / 금융위원회 cloud guideline** (전자금융감독규정) | 2024 | KR financial supervisory | Cloud + payment institution obligations |
| **KR 전자금융거래법** (Electronic Financial Transactions Act) | 1999, 2024 amendment | KR | Payment + 결제대행업 (PG) |
| **KR ISMS-P** (한국인터넷진흥원) | 2024 | KR (information security mgmt) | KISA certification |
| **KR CSAP** (Cloud Security Assurance Program v3.1) | 2024 | KR public sector cloud | Cloud provider certification |
| **JP APPI** (個人情報の保護に関する法律) | 2022-04 amendment | JP | Personal information |
| **JP METI Cloud Security Mark** | 2024 | JP government cloud | JP government cloud certification |
| **SG PDPA** (Personal Data Protection Act) | 2012, 2020 amendment | Singapore | Personal data |
| **AU Privacy Act 1988 + 2024 reforms** (Privacy Act Reform Act 2024) | 1988, 2024 reforms phased | Australia | Personal information |
| **AU Notifiable Data Breaches Scheme** (Pt IIIC Privacy Act) | 2018-02 | Australia | Breach notification within 30 days |
| **KSA PDPL** (Personal Data Protection Law) | 2023-09 | Saudi Arabia | Personal data |
| **KSA NDMO** (National Data Management Office) | 2024 framework | Saudi Arabia | Sovereign data management |
| **KSA SDAIA Cloud Framework** | 2023 | Saudi Arabia | Cloud computing controls |
| **UAE Data Protection Law (Federal Decree-Law No. 45/2021)** | 2023-01 | UAE | Personal data |
| **Brazil LGPD** (Lei Geral de Proteção de Dados) | 2020-09 | Brazil | Personal data |
| **Canada PIPEDA + Bill C-27 (CPPA)** | 2000, 2024 | Canada | Personal information |
| **Canada Quebec Law 25** | 2022-2024 phased | Quebec | Stricter privacy |
| **India DPDPA** (Digital Personal Data Protection Act 2023) | 2023, rules 2024 | India | Personal data |
| **ISO 27001:2022** | 2022-10 | International standard | Information security management |
| **ISO 27701:2019** | 2019 | International standard | Privacy information management |
| **ISO 22301:2019** | 2019 | International standard | Business continuity management |
| **ISO 42001:2023** | 2023-12 | International standard | AI management system |
| **SOC 2 Type II** (AICPA TSC 2017, 2024 revisions) | 2024 | International | Trust services criteria |
| **CSA STAR** (Cloud Security Alliance) | 2024 | International | Cloud assurance |

That is **38+ distinct regulatory regimes** the platform must serve.

### What "handle each ad hoc" costs

The current state-of-portfolio (pre-keystone) handles compliance
case-by-case:

1. **HIPAA-touching code paths** are tagged by hand in
   `microservices/messenger/` and `microservices/recordings/`, with
   ad-hoc tests and ad-hoc audit emissions. No single source of truth
   declares "this µservice handles PHI."
2. **EU AI Act tier evaluation** lives in
   `crates/oya-check-eu-ai-act-annex-iii-refusal` (per ADR-0144),
   currently a stand-alone validator with no relationship to other
   compliance regimes.
3. **GDPR DSAR cascade** is alluded to in ADR-0242 §D-4 but the cascade
   enumeration is bespoke per µservice — there is no shared abstraction
   for "the set of regulations whose erasure obligations include this
   data class."
4. **PCI DSS encryption + segmentation** would be authored as a
   separate body of Cedar fragments + Kyverno admission policies +
   network-policy templates if/when a tenant brings card data; no
   primitive ties them together.
5. **Sovereign-cloud overlay (ADR-0240)** binds data classes to
   providers but does not bind data classes to regulation-specific
   obligations (the overlay says "PII_KR must live on Naver/KT" but
   not "PII_KR is subject to KR-PIPA Article 22 consent rules").
6. **DR tier (ADR-0241)** declares per-µservice RTO/RPO but does not
   bind those declarations to regulatory minimums (HIPAA §164.308(a)(7)
   Contingency Plan requires demonstrated recovery; FedRAMP requires
   documented RPO ≤ 4h for High).
7. **Breach notification** is currently absent from the portfolio. The
   first time a regulator-reportable event occurs, the team would build
   the notification workflow under deadline pressure. GDPR Article 33
   requires notification within 72 hours; HIPAA §164.404 within 60
   days; KR-PIPA Article 34 within 24 hours; California Civ. Code
   §1798.82 "in the most expedient time possible."

Without a unifying primitive, **the cost of adding the N+1th regulation
grows superlinearly** because each new regulation requires touching
every µservice it applies to, every data class it covers, every Cedar
fragment, every audit stream, every retention rule.

### What every named hyperscaler reference actually does

The pattern across mature platforms is unambiguous: compliance is
packaged.

- **AWS Audit Manager (2020, GA 2024 enhancements).** AWS packages
  compliance as "frameworks": HIPAA Audit Controls, PCI DSS Audit
  Controls, FedRAMP Moderate/High, SOC 2, GDPR readiness, NIST
  800-53/171, etc. Each framework is a bundle of controls + evidence-
  collection rules + assessment workflows. Tenants subscribe to
  frameworks; AWS auto-collects evidence.
- **Microsoft Purview Compliance Manager (2021 GA, 2024 expansion).**
  Microsoft packages compliance as "assessment templates" wrapping
  Microsoft 365 + Azure controls. Each template targets a regulation
  (HIPAA, NIST 800-171, EU GDPR, FedRAMP, CMMC, NHS DSPT, etc.).
  Customers install templates; controls auto-evaluate.
- **Google Cloud Assured Workloads (2020, FedRAMP-High 2022, IL5 2024).**
  Google packages compliance as "compliance regimes" applied at the
  project level. Each regime locks the project's region set, key
  management, personnel access (US-persons-only for ITAR), and audit
  emission. Regimes include FedRAMP Moderate/High, IL2/IL4/IL5, IRS
  1075, HIPAA, EU Sovereign Controls, JP-2 (JP government).
- **Oracle Cloud Compliance Documents** packages compliance per region
  per regulation.
- **Salesforce HealthCloud + GovCloud + Financial Services Cloud.**
  Salesforce packages industry-specific compliance bundles as separate
  cloud SKUs.
- **Snowflake Healthcare + Financial Data Cloud.** Snowflake packages
  per-industry compliance.
- **Cloudflare Compliance Bundles** (HIPAA BAA available, ISO 27001
  certified, FedRAMP Moderate authorized) — each is a discrete bundle
  with its own onboarding flow.
- **Databricks Compliance Security Profile.** Toggles HIPAA, PCI DSS,
  FedRAMP-Moderate, IRAP, HITRUST as discrete profile activations on a
  workspace.

The shared pattern: **regulation as bundle, not as scattered controls.**
The platform offers regulation-as-product to its tenants. Activation is
explicit; controls are pre-bundled and pre-audited; evidence emission
is automatic.

### What this keystone does

Establishes **Compliance Pack** as the platform's first-class primitive
for "regulation as a bundle." Cells declare which packs they can host
(certification levels). Tenants install packs onto themselves. The
Cedar policy engine (per ADR-0243) aggregates installed packs at
evaluation time. Audit chain emits per-pack evidence. Cross-pack
traffic is Cedar-gated.

The unit of pack composition is the unit of:

- Regulator response (one pack → one regulator-evidence packet)
- Drift control (pack version → all-or-nothing activation)
- DPIA (Data Protection Impact Assessment) coverage
- Breach-notification workflow (per pack's jurisdiction)
- BAA / DPA agreement scope (per pack)
- Tenant onboarding for regulated industries (pack-install workflow)
- Sunset on regulation update (pack version replaces; old packs
  archived)

### Why now (2026-05-20)

Five forcing functions converge:

- **ADR-0144 (EU AI Act) was scoped narrowly.** ADR-0144 introduced
  graduated tiers for AI systems but did not bind these tiers to a
  broader compliance primitive. As more AI surfaces ship, the lack of
  primitive becomes architecturally costly.
- **ADR-0240 (sovereign cloud) and ADR-0241 (DR/BC) both reference
  per-pack overlays** but did not define what a pack contains beyond
  its sovereign-cloud-overlay block. The pack as a primitive is left
  half-specified.
- **The 2024 regulatory wave** (NIS2 effective 2024-10-17; DSA
  effective 2024-02-17; EU AI Act effective 2024-08-01; KR-PIPA 2023
  amendment in force; CA CPRA 2023; AU Privacy Act 2024 reforms;
  Quebec Law 25 phased through 2024) makes "ad-hoc per regulation"
  untenable.
- **The autonomous-masterplan goal**
  (`feedback_autonomous_implementation_artifacts`) requires that
  tenant onboarding (including for regulated industries) be a
  deterministic workflow. Without packs, regulated-tenant onboarding
  is bespoke and human-mediated.
- **ADR-0243 (Cedar as universal gate)** introduced `pack/<pack-id>/`
  as a Cedar fragment scope but did not specify the lifecycle, signing,
  composition, or evidence-emission semantics. This keystone fills the
  specification gap.

### What this is NOT

- This is NOT a substitute for actually achieving certifications. A
  HIPAA Compliance Pack defines the substrate; the platform team still
  has to pass HIPAA audits, sign BAAs with HHS-eligible counterparties,
  and maintain ongoing controls. The pack is the substrate, not the
  certificate.
- This is NOT an ADR that closes the per-regulation interpretation
  question. Each pack carries its own author + reviewer + signer who
  is responsible for the legal interpretation. The pack primitive is
  the **vessel**; the **content** is per-regulation legal work.
- This is NOT a marketing claim about regulatory completeness. Packs
  ship over time; each pack's published version represents the team's
  current best-effort interpretation, subject to regulator clarification.
- This is NOT a substitute for Data Protection Officer (DPO) /
  Compliance Officer / Legal Counsel sign-off. Pack publication
  requires explicit human signoff per §D-2.

## Decision

### D-1. Compliance Pack schema (canonical)

A Compliance Pack is a versioned, signed bundle described by the
following JSON Schema (canonical at `/specs/compliance-pack-schema.json`):

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://specs.oyatie/compliance-pack-schema.json",
  "title": "Compliance Pack",
  "type": "object",
  "required": [
    "pack_id", "version", "regulation", "signed_by", "effective_at",
    "cedar_fragments", "audit_chain_requirements", "data_class_extensions",
    "cell_eligibility", "retention_rules", "consent_requirements",
    "cross_tenant_rules", "jurisdiction_overlay", "dpia_template",
    "breach_notification_workflow", "regulator_evidence_cadence",
    "agreement_template_refs"
  ],
  "properties": {
    "pack_id": {
      "type": "string",
      "pattern": "^[A-Z][A-Z0-9-]*-[A-Z0-9.-]+$",
      "description": "Canonical pack identifier. Examples: HIPAA-2024, PCI-DSS-L1-v4.0.1, FedRAMP-Moderate-v5, EU-GDPR-2018-baseline, EU-AI-ACT-2024, KR-PIPA-2023-amendment, JP-APPI-2022-amendment, AU-PRIVACY-1988-2024-reforms, KSA-PDPL-2023, BR-LGPD-2020, IN-DPDPA-2023, ISO-27001-2022, ISO-22301-2019, SOC2-T2-2024, DoD-IL5-SRGv1r5, FERPA-2024, FDA-21CFR-PART11-2024, SOX-404-2024."
    },
    "version": {
      "type": "string",
      "pattern": "^[0-9]+\\.[0-9]+\\.[0-9]+(-[a-z0-9.-]+)?$",
      "description": "Semantic version of the pack. Increments on any normative change. Major-version bump on backward-incompatible reinterpretation."
    },
    "regulation": {
      "type": "object",
      "required": ["short_name", "full_citation", "effective_date", "regulator_authority", "jurisdiction"],
      "properties": {
        "short_name": {"type": "string"},
        "full_citation": {"type": "string"},
        "effective_date": {"type": "string", "format": "date"},
        "last_amended": {"type": "string", "format": "date"},
        "regulator_authority": {"type": "string"},
        "regulator_contact": {"type": "string"},
        "jurisdiction": {
          "type": "array",
          "items": {"type": "string", "description": "ISO 3166-1 alpha-2 or alpha-3, or supranational code (EU, EEA, GCC)"}
        }
      }
    },
    "signed_by": {
      "type": "object",
      "required": ["signer_key_id", "signature", "signature_algorithm", "attestation_blob_ref"],
      "properties": {
        "signer_key_id": {"type": "string", "description": "Ed25519 key fingerprint of oyatie-compliance-office signing key"},
        "signer_role": {"type": "string", "enum": ["oyatie-compliance-office", "pack-author-delegate"]},
        "signature": {"type": "string", "description": "Ed25519 signature over the canonicalized pack content (RFC 8785 JCS)"},
        "signature_algorithm": {"type": "string", "const": "Ed25519"},
        "attestation_blob_ref": {"type": "string", "description": "Sigstore Rekor entry ID + cosign attestation reference"},
        "co_signers": {
          "type": "array",
          "description": "Optional additional signers (DPO, external legal counsel, third-party auditor)",
          "items": {
            "type": "object",
            "properties": {
              "role": {"type": "string"},
              "key_id": {"type": "string"},
              "signature": {"type": "string"}
            }
          }
        }
      }
    },
    "effective_at": {"type": "string", "format": "date-time"},
    "sunset_at": {"type": "string", "format": "date-time"},
    "supersedes_pack_versions": {
      "type": "array",
      "items": {"type": "string"}
    },
    "cedar_fragments": {
      "type": "array",
      "description": "Cedar fragment references under scope pack/<pack-id>/",
      "items": {
        "type": "object",
        "required": ["fragment_id", "fragment_path", "version", "fragment_hash", "applies_to_actions"],
        "properties": {
          "fragment_id": {"type": "string"},
          "fragment_path": {"type": "string"},
          "version": {"type": "string"},
          "fragment_hash": {"type": "string", "description": "SHA-256 of the fragment file"},
          "applies_to_actions": {"type": "array", "items": {"type": "string"}},
          "applies_to_data_classes": {"type": "array", "items": {"type": "string"}},
          "applies_to_resources": {"type": "array", "items": {"type": "string"}},
          "default_deny_companion": {"type": "string"}
        }
      }
    },
    "audit_chain_requirements": {
      "type": "object",
      "required": ["stream_class", "retention_minimum", "schema_extension_refs", "tamper_evidence_level"],
      "properties": {
        "stream_class": {
          "type": "string",
          "description": "Audit-chain stream class to emit pack-specific events into"
        },
        "retention_minimum": {
          "type": "object",
          "properties": {
            "default_retention_years": {"type": "number"},
            "legal_hold_supersedes": {"type": "boolean"},
            "cold_storage_tier_after_days": {"type": "number"}
          }
        },
        "schema_extension_refs": {
          "type": "array",
          "items": {"type": "string"}
        },
        "tamper_evidence_level": {
          "type": "string",
          "enum": ["merkle-sealed-per-period", "merkle-sealed-per-event", "rfc-3161-timestamped"]
        },
        "required_event_classes": {
          "type": "array",
          "items": {"type": "string"}
        }
      }
    },
    "data_class_extensions": {
      "type": "array",
      "description": "Data classes that this pack contributes to the data-class registry (per ADR-0099)",
      "items": {
        "type": "object",
        "required": ["data_class_id", "data_class_name", "regulatory_category"],
        "properties": {
          "data_class_id": {"type": "string"},
          "data_class_name": {"type": "string"},
          "regulatory_category": {"type": "string"},
          "encryption_required_at_rest": {"type": "boolean"},
          "encryption_required_in_transit": {"type": "boolean"},
          "tokenization_required": {"type": "boolean"},
          "minimum_key_strength": {"type": "string"},
          "key_management_requirements": {"type": "string"},
          "permitted_processing_purposes": {"type": "array", "items": {"type": "string"}},
          "forbidden_processing_purposes": {"type": "array", "items": {"type": "string"}}
        }
      }
    },
    "cell_eligibility": {
      "type": "object",
      "required": ["minimum_certification_level_set", "permitted_certification_levels"],
      "properties": {
        "minimum_certification_level_set": {
          "type": "array",
          "items": {"type": "string"}
        },
        "permitted_certification_levels": {
          "type": "array",
          "items": {"type": "string"}
        },
        "forbidden_co_pack_certifications": {
          "type": "array",
          "items": {"type": "string"}
        },
        "provider_restrictions": {
          "type": "object",
          "properties": {
            "permitted_providers": {"type": "array", "items": {"type": "string"}},
            "forbidden_providers": {"type": "array", "items": {"type": "string"}}
          }
        }
      }
    },
    "retention_rules": {
      "type": "object",
      "required": ["per_data_class_retention", "legal_hold_supersedes"],
      "properties": {
        "per_data_class_retention": {
          "type": "object",
          "additionalProperties": {
            "type": "object",
            "properties": {
              "minimum_years": {"type": "number"},
              "maximum_years": {"type": "number"},
              "cold_storage_tier_after_days": {"type": "number"},
              "deletion_method": {"type": "string", "enum": ["hard-delete", "pseudonymize", "tombstone", "crypto-shred"]}
            }
          }
        },
        "legal_hold_supersedes": {"type": "boolean"},
        "subject_initiated_erasure_overrides_default": {"type": "boolean"}
      }
    },
    "consent_requirements": {
      "type": "object",
      "properties": {
        "consent_required_per_purpose": {"type": "boolean"},
        "consent_minimum_age": {"type": "number"},
        "guardian_consent_age_threshold": {"type": "number"},
        "consent_grant_workflow_ref": {"type": "string"},
        "consent_revoke_workflow_ref": {"type": "string"},
        "consent_audit_class": {"type": "string"},
        "withdrawal_must_match_grant_ease": {"type": "boolean", "description": "GDPR Article 7(3) — withdrawal as easy as grant"}
      }
    },
    "cross_tenant_rules": {
      "type": "object",
      "properties": {
        "cross_tenant_permitted_by_default": {"type": "boolean"},
        "cross_pack_traffic_default": {
          "type": "string",
          "enum": ["forbidden", "case-by-case-cedar-permit", "permitted-with-agreement"]
        },
        "required_agreements_for_cross_tenant": {
          "type": "array",
          "items": {"type": "string"}
        }
      }
    },
    "jurisdiction_overlay": {
      "type": "array",
      "description": "Country-level or state-level overlays where this pack manifests differently",
      "items": {
        "type": "object",
        "properties": {
          "jurisdiction_code": {"type": "string"},
          "overlay_fragment_refs": {"type": "array", "items": {"type": "string"}}
        }
      }
    },
    "dpia_template": {
      "type": "object",
      "required": ["template_ref", "applicability_criteria", "review_cadence_days"],
      "properties": {
        "template_ref": {"type": "string"},
        "applicability_criteria": {"type": "string"},
        "review_cadence_days": {"type": "number"},
        "required_signers": {"type": "array", "items": {"type": "string"}}
      }
    },
    "breach_notification_workflow": {
      "type": "object",
      "description": "Per-jurisdiction breach notification workflow declaration. For EU NIS2, the three-stage cadence (nis2_cadence_24h / nis2_cadence_72h / nis2_cadence_1mo) MUST be declared on eu-sovereign cert-level packs. Cross-reference: Directive (EU) 2022/2555 Article 23.",
      "required": ["workflow_ref", "regulator_notification_deadline_hours", "subject_notification_deadline_hours"],
      "properties": {
        "workflow_ref": {"type": "string"},
        "regulator_notification_deadline_hours": {"type": "number"},
        "regulator_notification_endpoint": {"type": "string"},
        "subject_notification_deadline_hours": {"type": "number"},
        "subject_notification_required_above_severity": {"type": "string"},
        "post_mortem_required": {"type": "boolean"},
        "nis2_three_stage_cadence": {
          "type": "object",
          "description": "EU NIS2 Article 23 three-stage incident notification cadence. Required on all packs bound to the eu-sovereign certification level. Cross-reference: Directive (EU) 2022/2555 Article 23(4)(a)–(c).",
          "properties": {
            "nis2_cadence_24h": {
              "type": "object",
              "description": "Stage 1 — Early Warning: notify the competent authority or CSIRT within 24 hours of becoming aware of a significant incident. Minimum-information notification: incident type, suspected origin if known, whether cross-border impact is possible. Does NOT require root cause or full scope at this stage.",
              "required": ["deadline_hours", "required_fields"],
              "properties": {
                "deadline_hours": {"type": "number", "const": 24, "description": "Maximum hours from awareness to early-warning notification per NIS2 Article 23(4)(a)."},
                "required_fields": {
                  "type": "array",
                  "description": "Minimum information required in the early-warning notification.",
                  "items": {"type": "string"},
                  "default": ["incident_type", "suspected_origin_if_known", "cross_border_impact_possible"]
                },
                "workflow_stage_id": {"type": "string", "description": "Workflow Engine stage ID for this notification step.", "examples": ["nis2-early-warning-24h"]},
                "audit_event_class": {"type": "string", "const": "Nis2EarlyWarning24h", "description": "Audit chain event class emitted when this stage fires."}
              }
            },
            "nis2_cadence_72h": {
              "type": "object",
              "description": "Stage 2 — Incident Notification: full incident notification to competent authority or CSIRT within 72 hours of awareness. Replaces or supersedes the early-warning. Required fields: severity assessment, indicators of compromise (IoCs), scope (systems, data classes, affected Member States), mitigations underway. Cross-reference: NIS2 Article 23(4)(b).",
              "required": ["deadline_hours", "required_fields"],
              "properties": {
                "deadline_hours": {"type": "number", "const": 72, "description": "Maximum hours from awareness to full incident notification per NIS2 Article 23(4)(b)."},
                "required_fields": {
                  "type": "array",
                  "description": "Full information required in the incident notification.",
                  "items": {"type": "string"},
                  "default": ["severity_assessment", "indicators_of_compromise", "affected_systems_scope", "affected_data_classes", "affected_member_states", "mitigations_underway", "cross_border_impact_assessment"]
                },
                "workflow_stage_id": {"type": "string", "description": "Workflow Engine stage ID for this notification step.", "examples": ["nis2-incident-notification-72h"]},
                "audit_event_class": {"type": "string", "const": "Nis2IncidentNotification72h", "description": "Audit chain event class emitted when this stage fires."}
              }
            },
            "nis2_cadence_1mo": {
              "type": "object",
              "description": "Stage 3 — Final Report: detailed final report to competent authority within 1 month of the incident notification (i.e., ≤1 month after awareness). Required fields: root cause analysis, full impact assessment (systems, data, individuals, Member States), lessons learned, remediation steps completed or planned. Cross-reference: NIS2 Article 23(4)(c).",
              "required": ["deadline_days", "required_fields"],
              "properties": {
                "deadline_days": {"type": "number", "const": 30, "description": "Maximum days from awareness to final report per NIS2 Article 23(4)(c)."},
                "required_fields": {
                  "type": "array",
                  "description": "Content required in the final report.",
                  "items": {"type": "string"},
                  "default": ["root_cause_analysis", "full_impact_assessment", "affected_individuals_count", "affected_member_states", "lessons_learned", "remediation_steps_completed", "remediation_steps_planned", "recurrence_prevention_measures"]
                },
                "workflow_stage_id": {"type": "string", "description": "Workflow Engine stage ID for this notification step.", "examples": ["nis2-final-report-1mo"]},
                "audit_event_class": {"type": "string", "const": "Nis2FinalReport1Mo", "description": "Audit chain event class emitted when this stage fires."}
              }
            }
          },
          "required": ["nis2_cadence_24h", "nis2_cadence_72h", "nis2_cadence_1mo"]
        }
      }
    },
    "regulator_evidence_cadence": {
      "type": "object",
      "description": "Per-pack regulator evidence cadence. The cadence enum is extended with EU DSA-specific values per Regulation (EU) 2022/2065 Articles 24 and 28. eu-sovereign cert-level packs MUST declare the appropriate DSA cadence and bind the regulator_audit_workflow_id.",
      "properties": {
        "cadence": {
          "type": "string",
          "enum": [
            "monthly",
            "quarterly",
            "semi-annual",
            "annual",
            "ad-hoc",
            "dsa_transparency_report_semi_annual",
            "dsa_transparency_report_quarterly_vlop",
            "dsa_minor_risk_mitigation_assessment_annual"
          ],
          "description": "Evidence emission cadence. New DSA-specific values: 'dsa_transparency_report_semi_annual' — semi-annual public transparency report per EU DSA Article 24 (applies to online platforms with ≥45M EU monthly active users); 'dsa_transparency_report_quarterly_vlop' — quarterly transparency report per DSA Article 24(2) as applicable to Very Large Online Platforms (VLOPs) and Very Large Online Search Engines (VLOSEs) designated by the European Commission; 'dsa_minor_risk_mitigation_assessment_annual' — annual risk assessment and mitigation report specifically addressing risks to minors per EU DSA Article 28, covering recommender systems, advertising targeting, and age-appropriate design measures. Cross-references: Regulation (EU) 2022/2065 Articles 24, 28."
        },
        "evidence_class": {"type": "string"},
        "regulator_pull_endpoint": {"type": "string"},
        "regulator_audit_workflow_id": {
          "type": "string",
          "description": "Workflow Engine workflow ID that orchestrates periodic regulator evidence packaging and submission. Required for eu-sovereign cert-level packs that declare DSA cadence values. Pattern: wf-regulator-audit-<pack-stem>.",
          "pattern": "^wf-[a-z0-9][a-z0-9-]*$",
          "examples": [
            "wf-regulator-audit-eu-dsa-transparency",
            "wf-regulator-audit-eu-dsa-minor-risk",
            "wf-regulator-audit-eu-nis2-annual"
          ]
        }
      }
    },
    "agreement_template_refs": {
      "type": "object",
      "description": "Per-pack contractual agreements required",
      "properties": {
        "baa_template_ref": {"type": "string", "description": "Business Associate Agreement (HIPAA)"},
        "dpa_template_ref": {"type": "string", "description": "Data Processing Agreement (GDPR Article 28)"},
        "scc_template_ref": {"type": "string", "description": "Standard Contractual Clauses (cross-border EU transfer)"},
        "subprocessor_disclosure_template_ref": {"type": "string"},
        "agreement_lifecycle_workflow_ref": {"type": "string"}
      }
    },
    "ai_act_tier_binding": {
      "type": "object",
      "description": "Only set for EU-AI-ACT-* packs; references ADR-0144 tier model",
      "properties": {
        "tier_id": {"type": "string"},
        "obligations": {"type": "array", "items": {"type": "string"}}
      }
    },
    "kyb_required": {"type": "boolean", "description": "Know-Your-Business verification required at install"},
    "kyc_required": {"type": "boolean", "description": "Know-Your-Customer required at install (e.g., PCI DSS)"}
  }
}
```

### D-2. Pack lifecycle

A Compliance Pack progresses through a controlled lifecycle, every
transition emitting to the audit chain:

```
[Authored] -> [Multispectrum-Reviewed] -> [Signed] -> [Published]
   -> [Tenant-Installs] -> [Cedar-Aggregates-At-Evaluation]
   -> [Audit-Chain-Emits-Per-Pack] -> [Sunset-On-Regulation-Update]
   -> [Tombstoned-After-Archive-Retention]
```

**Stage 1: Authored.** Pack drafted in `microservices/governance/
packs/<pack-id>/v<version>/` directory structure:

```
microservices/governance/packs/<pack-id>/v<version>/
  pack.yaml                       # canonical pack content matching D-1 schema
  cedar/
    <fragment>.cedar              # signed-by-pack-owner-key fragments
  audit-schema/
    <event-class>.json            # JSON Schema for pack-specific audit events
  dpia-template.md                # human-readable DPIA template
  breach-workflow.yaml            # Workflow Engine durable workflow definition
  agreements/
    baa-template.md
    dpa-template.md
  jurisdiction-overlays/
    <jurisdiction-code>/
      overlay.cedar
  test-fixtures/
    permit-scenarios.yaml         # expected-permit cases
    deny-scenarios.yaml           # expected-deny cases
  CHANGELOG.md
  REGULATORY-MAPPING.md           # cite each Article/Section satisfied
```

**Stage 2: Multispectrum review (v2.4.0).** Pack drafts undergo
fan-out per facet:

- **F1 (correctness):** Cedar fragments do what the pack content claims.
- **F5 (security):** No privilege escalation; default-deny coverage
  complete.
- **F6 (performance):** Per-pack evaluation cost within budget.
- **F7 (supply chain):** Pack signature chain verifies.
- **F11 (regulatory-compliance):** Regulatory mapping in
  `REGULATORY-MAPPING.md` cites each cited Article/Section accurately.
- **A1 (own-policy-adherence-naming):** Pack ID follows the
  `^[A-Z][A-Z0-9-]*-[A-Z0-9.-]+$` BNF.
- **A2 (own-policy-adherence-documentation):** All required artifacts
  per the directory structure present (per
  `feedback_doc_coverage_enforced`).
- **A4 (own-policy-adherence-architecture):** Pack respects the
  baseline/pack/overlay/tenant layering per ADR-0243 §D-4.
- **A6 (own-policy-adherence-schema):** Audit-schema extensions
  match Ontology Object Type definitions (per the Ontology IP-002
  conventions).

**Stage 3: Sign.** A reviewed pack is signed by the **oyatie-compliance-
office Ed25519 key** (the canonical "pack-owner" key — held in HSM by
`oyatie.security.signing` per ADR-0242 §D-7 audit-stream + ADR-0243
§D-5 chain-of-trust pattern). Co-signers may also sign:

- **DPO (Data Protection Officer)** for GDPR + KR-PIPA + JP APPI packs.
- **CISO** for FedRAMP / DoD IL packs.
- **External counsel** for FedRAMP / NIS2 / state-law packs.
- **Third-party auditor attestation** (e.g., AICPA SOC 2 reporting
  CPA firm) for SOC 2 packs.

Signature is Ed25519 over RFC 8785 (JCS) canonicalized pack content,
plus a Sigstore Rekor transparency-log entry + cosign attestation. The
attestation references the pack hash + signing-key fingerprint.

**Stage 4: Publish.** Signed pack is published to the pack registry:

- Postgres + Citus shard on `(pack_id, version)` (catalogue).
- SeaweedFS immutable blob storage (binary blobs of pack content +
  agreements + DPIA templates).
- Audit-chain event `CompliancePackPublished` emitted with
  `{pack_id, version, signed_by_key_id, attestation_blob_ref,
  effective_at}`.

**Stage 5: Tenant installs.** Tenant invokes the pack-install workflow
(per §D-3). On successful install, audit emits
`CompliancePackInstalled` event on the tenant's audit stream.

**Stage 6: Cedar evaluates with pack overlay.** Every Cedar evaluation
for the tenant aggregates baseline + jurisdiction-overlay + installed-
pack fragments + tenant-fragments per ADR-0243 §D-4. Per-pack
fragments load at evaluation time.

**Stage 7: Audit emission per pack.** Every state-changing action under
a tenant emits to the pack's declared `audit_chain_requirements.stream_class`
in addition to the tenant's baseline stream. Per-pack stream retention
applies independently of baseline retention.

**Stage 8: Sunset on regulation update.** When the regulation issues a
material amendment (e.g., PCI DSS 4.0.1 → 4.1; KR-PIPA Q3-2026
amendment), a new pack version is drafted. The previous version sunsets
per its `sunset_at`. Tenants must migrate (per §D-3 install-version-
migration workflow).

**Stage 9: Tombstone.** After the regulator-required archive retention
expires (typically 6-7 years past sunset), the pack version is
tombstoned. Audit emissions retained per their own retention rules.

### D-3. Tenant pack installation

A tenant installs packs by setting `tenant.compliance_packs[]` in its
tenant configuration (per the tenant model from ADR-0242 §D-7 and
ADR-0244):

```yaml
tenant_id: "tenant-acme-healthcare-inc"
compliance_packs:
  - pack_id: "HIPAA-2024"
    version: "1.2.3"
    installed_at: "2026-04-15T14:00:00Z"
    installed_by: "tenant.acme-healthcare-inc.admin.<id>"
    dpia_signed_ref: "dpia-blob-ref-7821"
    baa_signed_ref: "baa-blob-ref-9132"
    kyb_verified_at: "2026-04-10T09:30:00Z"
    installation_evidence_id: "audit-event-7e1f3a"
  - pack_id: "SOC2-T2-2024"
    version: "2.0.1"
    installed_at: "2026-04-15T14:00:00Z"
    installed_by: "tenant.acme-healthcare-inc.admin.<id>"
    auditor_attestation_ref: "soc2-attestation-blob-5512"
```

The tenant pack-install workflow:

1. **Eligibility check.** Tenant's `audience_type` (per ADR-0244)
   compatible with pack. (E.g., individual consumer cannot install
   DoD IL5.)
2. **KYB / KYC verification.** If pack requires (`kyb_required: true`),
   tenant submits business verification documents; oyatie identity-
   verification substrate (per §Implementation surface) confirms.
3. **Jurisdiction match.** Tenant's `jurisdiction.primary` (or any of
   `data_residency_allowed`) intersects pack's
   `regulation.jurisdiction[]`.
4. **DPIA signed.** Tenant DPO completes the pack's DPIA template;
   blob stored; reference recorded.
5. **Per-pack agreements signed.** BAA (HIPAA), DPA (GDPR), SCCs
   (cross-border EU transfer), etc., signed via the Workflow Engine
   durable saga (per §D-7).
6. **Cell pinning check.** Tenant's `home_cell` and `dr_cell` certify
   for the pack per §D-5.
7. **Cedar fragment activation.** Pack's Cedar fragments load into the
   tenant's effective policy.
8. **Audit emission.** `CompliancePackInstalled` event emitted with
   full installation evidence.
9. **Onboarding workflow.** Pack-specific onboarding steps (e.g.,
   PCI DSS QSA scope confirmation; HIPAA staff training acknowledgement;
   FedRAMP authorization-package update) run via Workflow Engine
   durable workflow.

Pack uninstallation requires:

- All data classes contributed by the pack are either erased or
  reclassified.
- No legal-hold blocks erasure.
- Tenant DPO + oyatie council-legal countersign.
- Subject notification (where applicable per GDPR Article 13/14 / KR-
  PIPA Article 21).
- `CompliancePackUninstalled` event emitted with full evidence.

### D-4. Cell certification level matrix

Cells (per ADR-0009 + ADR-0248 Amazon-shape cellular architecture)
declare a **SET** of certification levels they have achieved. A
certification level is a named bundle of substrate prerequisites that
qualify the cell to host one or more compliance packs.

The canonical matrix at `/specs/cell-certification-level-matrix.json`:

| Certification Level | Description | Prerequisite Controls | Eligible Packs | Reference Standard |
|---|---|---|---|---|
| **`general`** | Baseline cell; non-regulated tenants | SOC 2 Type II controls baseline; ISO 27001:2022 ISMS; per-tenant encryption at rest; TLS 1.3+ in transit; per-cell audit chain; quarterly DR drill (T2 minimum per ADR-0241) | SOC2-T2-2024, ISO-27001-2022, ISO-22301-2019, ISO-27701-2019 (privacy mgmt baseline), CSA-STAR-2024 baseline | AICPA TSC 2024; ISO 27001:2022; CSA STAR Level 2 |
| **`pci-certified`** | + PCI DSS Level 1 certified | Network segmentation per PCI DSS Requirement 1; cardholder-data-environment (CDE) isolation; tokenization gateway; QSA-assessed annually; quarterly ASV scans; file-integrity monitoring; PCI-DSS-specific incident response | All of `general` PLUS PCI-DSS-L1-v4.0.1 | PCI DSS 4.0.1 (PCI SSC 2024) |
| **`hipaa-certified`** | + HIPAA / HITECH ready; BAA infrastructure; breach notification machinery | All of `general`; PHI data class extensions; BAA workflow per §D-7; breach notification machinery per §D-8 (HIPAA §164.404 60-day rule); HIPAA Security Rule §164.308/.310/.312 administrative + physical + technical safeguards; HHS OCR audit evidence emission | HIPAA-2024 | HIPAA 45 CFR §164; HITECH 2009; HHS OCR audit protocol 2024 |
| **`hipaa-pci-certified`** | + Both HIPAA and PCI (e.g., for health-finance tenants) | Both above; segmentation between PHI and CHD environments | HIPAA-2024, PCI-DSS-L1-v4.0.1 | Combined |
| **`fedramp-moderate`** | + FedRAMP Moderate authorized; GovCloud isolation | All of `general`; NIST 800-53 Rev. 5 Moderate baseline controls; FedRAMP-eligible substrate (AWS GovCloud, Azure Government, Google Assured Workloads FedRAMP Moderate); US-persons-only personnel for control-plane access; FIPS 140-2 Level 2 cryptographic modules; continuous monitoring per NIST 800-137 | FedRAMP-Moderate-v5, NIST-800-53-MOD-Rev5, FedRAMP-Moderate-2024 | FedRAMP PMO 2024 baseline; NIST SP 800-53 Rev. 5 |
| **`fedramp-high`** | + FedRAMP High authorized; air-gap-capable | All of `fedramp-moderate`; NIST 800-53 Rev. 5 High baseline; air-gap-capable substrate; FIPS 140-2 Level 3 cryptographic modules; CONUS-only personnel | FedRAMP-High-v5 | FedRAMP PMO 2024 High baseline |
| **`il4`** | + DoD IL4 ATO; Controlled Unclassified Information (CUI) | All of `fedramp-moderate`; CMMC Level 2 controls per CMMC 2.0; CJCSI 6510.01 CUI handling; DoD-CIO-approved cryptography | DoD-IL4-SRGv1r5, CMMC-L2 | DISA SRG v1r5 IL4; CMMC 2.0 (DoD CIO 2024) |
| **`il5`** | + DoD IL5; classified-network adjacency | All of `il4`; SECRET-network-adjacent infrastructure; CMMC Level 3; STIG-compliant configurations across substrate; DISA-managed key encryption | DoD-IL5-SRGv1r5, CMMC-L3 | DISA SRG v1r5 IL5 |
| **`il6`** | + DoD IL6; SECRET classification | All of `il5`; SECRET-classified network; cleared personnel (US-persons + secret-clearance); NSA Type 1 cryptographic modules; air-gap operations | DoD-IL6-SRGv1r5 | DISA SRG v1r5 IL6 |
| **`kr-fss-financial`** | + KR Financial Supervisory Service compliance; 결제대행업 PG | All of `general`; 전자금융감독규정 (FSS) cloud guideline 2024 controls; 결제대행업 (PG payment-gateway) license-holder substrate; KR-resident-only operations personnel for sensitive controls; KISA ISMS-P certification | KR-FSS-2024, KR-EFTA-2024-amendment, KR-ISMS-P-2024 | 금융위원회 전자금융감독규정 2024 |
| **`kr-csap`** | + KR CSAP certified | All of `general`; CSAP v3.1 controls; Naver Cloud / KT Cloud substrate; KR-resident data; KR-resident personnel for control-plane access | KR-CSAP-v3.1 | KISA CSAP v3.1 |
| **`eu-sovereign`** | + GAIA-X-aligned; EU data sovereignty | All of `general`; GAIA-X self-description; OVH / T-Systems / Scaleway substrate (or EU-headquartered provider verified by GAIA-X); EU-resident-only personnel for control-plane access; EU-resident data; Schrems-II-aligned cross-border transfer controls (or none); EU GDPR DPA infrastructure | EU-GDPR-2018-baseline, EU-NIS2-2022, EU-DSA-2022, EU-AI-ACT-2024, EU-DATA-ACT-2023, EU-DMA-2022 | GAIA-X 2024; EU GDPR; EU NIS2 transposition 2024-10-17 |
| **`healthcare-sovereign`** | + Per-country health-data sovereignty | All of `general`; per-country health-data substrate; combined HIPAA (US tenants) + EU GDPR Article 9 (special-category data) + KR 의료법 (KR tenants) | HIPAA-2024, EU-GDPR-2018-baseline, KR-MEDICAL-LAW-2024, JP-APPI-2022-amendment-medical | HIPAA + GDPR Article 9 + KR 의료법 + JP APPI medical |
| **`ksa-government`** | + KSA government-cloud | All of `general`; SDAIA Cloud Computing Framework v1.0; NDMO controls 2024; STC Cloud or Mobily Cloud substrate; KSA-resident data; KSA-resident personnel | KSA-PDPL-2023, KSA-NDMO-2024, KSA-SDAIA-CCF-v1.0 | SDAIA 2024; NDMO 2024 |
| **`jp-government`** | + JP government-cloud | All of `general`; METI Cloud Security Mark; Sakura Internet / KDDI Cloud substrate; JP-resident data; JP-resident personnel | JP-APPI-2022-amendment, JP-METI-CSM-2024 | METI Cloud Security Mark 2024 |
| **`au-government`** | + AU government-cloud | All of `general`; IRAP-assessed substrate; AU-resident data; AU-resident personnel | AU-PRIVACY-1988-2024-reforms, AU-NDBS, AU-IRAP-PROTECTED | AU IRAP 2024 |
| **`fda-regulated`** | + FDA 21 CFR Part 11 e-records + e-signatures | All of `general`; immutable audit trail; cryptographic e-signature substrate; system validation per GAMP 5 | FDA-21CFR-PART11-2024 | FDA 21 CFR Part 11 (2024 revision) |
| **`student-records-ferpa`** | + FERPA student-record handling | All of `general`; education-record data class; per-school district BAA-equivalent; parental-consent workflow | FERPA-2024 | FERPA (20 USC §1232g) |
| **`finance-sox`** | + SOX §404 financial controls; public-company reporting | All of `general`; SOX-404 ICFR controls; segregation of duties enforced via Cedar; audit-chain SOX retention 7 years | SOX-404-2024, GLBA-SAFEGUARDS-2024 | SOX 2002 §404; GLBA Safeguards Rule (16 CFR §314) 2024 |
| **`cn-pipl-eligible`** | + China PIPL data-plane; mainland-China-only data residency | All of `general`; data-plane infrastructure physically located in mainland China only (no cross-border data egress by default); CAC security assessment completed for the cell per PIPL Article 40 + CAC Measures on Security Assessment for Cross-Border Data Transfer 2022; CAC-approved Key Management Service (KMS) — Alibaba Cloud KMS / Tencent Cloud KMS / Huawei Cloud DEW in a mainland-China region; mainland-China-resident operations staff with PRC citizenship for control-plane access to the cell; CN-PIPL-2021 pack installed | CN-PIPL-2021 | PIPL Article 40 (data localization for CIIOs + processors above CAC volume threshold); CAC Security Assessment Measures 2022; MIIT guidelines; hyperscaler precedents: AWS China (Sinnet/NWCD), Alibaba Cloud, Tencent Cloud, Microsoft Azure China (21Vianet) — all operate separate legal entities with mainland-China KMS and residency-only data plane |

A cell may carry multiple certification levels (e.g., a cell certified
for both `hipaa-certified` AND `eu-sovereign` hosts a HIPAA-installed
EU-resident tenant). Pack co-residency is permitted iff the union of
all packs the cell hosts is in the cell's certifications set and no
pack's `cell_eligibility.forbidden_co_pack_certifications[]` lists a
co-resident certification.

The cell certification declaration:

```yaml
cell_id: "cell-eu-west-3-a-hipaa-001"
certification_levels:
  - "general"
  - "hipaa-certified"
  - "eu-sovereign"
certification_evidence:
  general:
    soc2_type_ii_attestation_ref: "soc2-2026q1-attestation-blob-1234"
    iso_27001_certificate_ref: "iso-27001-cert-blob-5678"
    iso_22301_certificate_ref: "iso-22301-cert-blob-9012"
    last_drill_evidence_ref: "drill-evidence-2026-04-30"
  hipaa-certified:
    hipaa_security_rule_attestation_ref: "hipaa-2026q1-attestation-blob-3456"
    last_breach_drill_ref: "breach-drill-2026-04-15"
  eu-sovereign:
    gaia_x_self_description_ref: "gaia-x-self-desc-blob-7890"
    eu_personnel_attestation_ref: "eu-personnel-2026-attestation-blob-2345"
substrate_provider: "ovh"
substrate_region: "eu-gra"
```

### D-5. Tenant → cell pinning rule

A tenant with installed packs `P = {P_1, P_2, ..., P_n}` must live in
a cell whose certifications set `C_cell` satisfies:

```
∀p ∈ P: p.cell_eligibility.minimum_certification_level_set ⊆ C_cell
∀p ∈ P: C_cell ⊆ p.cell_eligibility.permitted_certification_levels
∀p_i, p_j ∈ P, i ≠ j:
  p_i.cell_eligibility.forbidden_co_pack_certifications ∩ p_j.minimum_certification_level_set = ∅
```

The pinning rule applies **transitively for all packs**: a tenant with
both HIPAA-2024 and EU-GDPR-2018-baseline must live in a cell certified
for both `hipaa-certified` and `eu-sovereign`. The
`microservices/tenancy/` admission gate refuses tenant placement that
would violate the rule.

DR cell pinning follows the same rule: `tenant.dr_cell` must also
satisfy the pack requirements. Cross-cell failover (per ADR-0241 +
ADR-0249) cannot route tenants to cells outside their pack-certified
set.

Tenants with mismatched pack/cell pinning are quarantined: their
operations halt at the Cedar gate (forbid all actions) until the
operator resolves the mismatch (migrate the tenant or upgrade the
cell's certification levels).

### D-6. Cross-pack traffic rules

When two tenants attempt to interact across pack boundaries, the
following rules apply. The matrix is evaluated by the Cedar policy
engine at every cross-tenant call:

| Caller pack profile | Callee pack profile | Default decision | Override mechanism | Audit emission |
|---|---|---|---|---|
| **General ↔ General** | General ↔ General | PERMITTED (subject to per-tenant Cedar fragments + per-action authorization) | Tenant-fragment can `forbid` further | `CrossTenantCallPermitted` |
| **Regulated ↔ Regulated, same pack set** | e.g., HIPAA tenant A ↔ HIPAA tenant B | PERMITTED iff per-tenant BAA/DPA present + per-action Cedar permit | BAA/DPA agreement-lifecycle workflow per §D-7 | `CrossRegulatedCallPermitted` + `AgreementReferenceAttached` |
| **Regulated ↔ Regulated, different pack set** | e.g., HIPAA tenant ↔ PCI tenant | FORBIDDEN by default | Case-by-case Cedar `permit` requires: (1) both packs' `cross_tenant_rules.required_agreements_for_cross_tenant` satisfied; (2) data-class compatibility check (no PHI flows to PCI-only resources); (3) cross-tenant DPIA signed for the specific interaction; (4) explicit oyatie council-legal review | `CrossPackDifferentRegulationsCallPermitted` (after explicit override) OR `CrossPackDifferentRegulationsCallForbidden` |
| **Regulated ↔ General** | e.g., HIPAA tenant calls into a General SaaS tenant | FORBIDDEN by default; PHI / PCI-CHD / classified data MUST NOT leak | Cedar permit requires data-class downgrading (e.g., de-identified) + the General tenant agreeing to behave as if subject to the regulated tenant's rules for the duration of the call. Most commonly: a de-identified data export from regulated tenant to general tenant; the de-identification engine (per §D-9) produces an artifact that no longer carries the regulated class. | `RegulatedToGeneralCallPermitted` (de-identified) OR `RegulatedToGeneralCallForbidden` |
| **General ↔ Regulated** | General tenant calls into a regulated tenant (e.g., to read a public schedule from a healthcare tenant) | PERMITTED iff regulated tenant's exposed surface is non-PHI; FORBIDDEN if surface touches regulated data | Cedar fragment on regulated tenant's resources controls; data-class enforcement ensures protected data is not visible | `GeneralToRegulatedCallPermitted` (non-regulated-data surface) OR `GeneralToRegulatedCallForbidden` |
| **EU-Sovereign ↔ Non-EU-Sovereign** | Any cross-jurisdiction with EU on one side | FORBIDDEN by default for PII data classes (Schrems II) | Permitted via SCC + adequacy decision (EU-US DPF 2023 adequacy decision); per-call audit emission with the SCC reference | `CrossBorderEUTransferPermitted` (with SCC ref) OR `CrossBorderEUTransferForbidden` |
| **DoD IL5/IL6 ↔ Any** | Classified ↔ unclassified | FORBIDDEN unconditionally (cannot egress classified network) | NO override (classified data does not transit unclassified substrate) | `ClassifiedCallForbidden` |
| **Pack tenant ↔ oyatie (platform-owner)** | Any pack tenant ↔ `oyatie.*` | Subject to same rules as any cross-tenant; oyatie's audience_type is `PLATFORM_OWNER` per ADR-0242 §D-7; treated as a regulated tenant for purposes of egress (the platform team's data classes count) | Same agreement-signing requirements per pack | Same audit events |

The cross-pack matrix is encoded in Cedar fragments at
`microservices/policy-engine/fragments/baseline/cross-pack-matrix.cedar`
(baseline scope; signed by org-baseline-key per ADR-0243 §D-5).

### D-7. BAA / DPA agreement infrastructure

Each pack declares one or more agreement templates in its
`agreement_template_refs`. Examples:

- **BAA (Business Associate Agreement)** for HIPAA-2024 — per HHS OCR
  guidance, BAA must include: permitted uses; safeguards; subcontractor
  flow-through; breach notification; term + termination; HHS OCR
  cooperation.
- **DPA (Data Processing Agreement)** for EU-GDPR-2018-baseline — per
  GDPR Article 28 — controller/processor relationship; instructions;
  confidentiality; security measures; subprocessor consent; subject
  rights assistance; audit cooperation; termination data return/erasure.
- **SCC (Standard Contractual Clauses)** for cross-border EU transfer
  per EU Commission Decision (EU) 2021/914.
- **Subprocessor disclosure list** per GDPR Article 28(2-4).
- **Tax-residency attestation** for sales/VAT compliance per
  jurisdiction.

The agreement lifecycle is managed by a **durable Workflow Engine
saga** at `microservices/governance/workflows/agreement_lifecycle.yaml`:

```yaml
workflow_id: agreement_lifecycle
states:
  - DRAFTED
  - SUBMITTED_FOR_TENANT_REVIEW
  - TENANT_COUNTERSIGNED
  - SUBMITTED_FOR_OYATIE_REVIEW
  - OYATIE_COUNTERSIGNED
  - ACTIVE
  - SUBPROCESSOR_DISCLOSURE_PENDING
  - SUBPROCESSOR_DISCLOSURE_NOTIFIED
  - RENEWAL_PENDING
  - RENEWED
  - TERMINATING
  - TERMINATED
transitions:
  DRAFTED -> SUBMITTED_FOR_TENANT_REVIEW: on(adminPublishesDraft)
  SUBMITTED_FOR_TENANT_REVIEW -> TENANT_COUNTERSIGNED: on(tenantSignatureReceived)
  TENANT_COUNTERSIGNED -> SUBMITTED_FOR_OYATIE_REVIEW: on(allSignaturesValidated)
  SUBMITTED_FOR_OYATIE_REVIEW -> OYATIE_COUNTERSIGNED: on(oyatieCounselSigns)
  OYATIE_COUNTERSIGNED -> ACTIVE: on(allSignaturesValid)
  ACTIVE -> RENEWAL_PENDING: on(scheduleEvent: 90daysBeforeTerm)
  RENEWAL_PENDING -> RENEWED: on(renewalCompleted) -> ACTIVE
  ACTIVE -> TERMINATING: on(eitherPartyInitiatesTermination)
  TERMINATING -> TERMINATED: on(dataReturnedOrErasedPerAgreement)
audit_emissions:
  CompliancePackAgreementLifecycleTransition:
    on_every_transition: true
expiration_actions:
  on_expiration:
    - pause_pack_activation_for_tenant
    - notify_tenant_admin
    - escalate_to_council_legal_after_grace_period_days(14)
```

Expiration: each agreement carries a term (typically 2-3 years for
BAA; 3-5 years for DPA; aligned with tenant subscription term). Auto-
renewal is scheduled by Workflow Engine 90 days before term; tenants
can also re-execute at any time. Expired agreements pause the
associated pack's activation for the tenant until renewed.

Subprocessor flow-through: when oyatie's subprocessor list changes
(e.g., new substrate provider added per ADR-0240), affected tenants
receive notice per their DPAs' subprocessor-notification clause
(GDPR Article 28(2)) within the disclosure cadence (typically 30
days advance for material changes).

### D-8. Breach notification machinery

Each pack declares a `breach_notification_workflow` with per-jurisdiction
deadlines. The substrate
`microservices/governance/workflows/breach_notification/` provides per-
jurisdiction workflow templates. Key deadlines and shapes:

| Regulation | Regulator deadline | Subject deadline | Severity threshold for subject notice |
|---|---|---|---|
| **HIPAA Breach Notification Rule (45 CFR §164.404)** | HHS OCR within 60 days of discovery; smaller breaches (< 500 individuals) reported annually | Affected individuals within 60 days; substitute notice if contact info unavailable | Breach affecting unsecured PHI (per §164.402 definition) |
| **GDPR Article 33 (regulator) + Article 34 (subject)** | Supervisory Authority within 72 hours of becoming aware | Subjects "without undue delay" when high-risk to rights and freedoms | High risk to data subjects |
| **KR-PIPA Article 34** | 개인정보보호위원회 (KCS) within 24 hours; details within 72 hours | Affected subjects within 24 hours | Any breach |
| **CA Civ. Code §1798.82 + §1798.150 (CCPA/CPRA)** | CA AG notification ≥ 500 affected | "In the most expedient time possible" | Any breach |
| **State breach laws (50 US states)** | Per-state — most require notification 30-60 days | Per-state | Varies |
| **EU NIS2 Art. 23** | CSIRT / competent authority within 24 hours (early warning); 72 hours (incident notification); 1 month (final report) | Service users | "Significant impact" per Art. 23 |
| **AU Notifiable Data Breaches Scheme** | OAIC within 30 days | Affected individuals within 30 days | Likely to result in serious harm |
| **JP APPI Article 26** | PPC (Personal Information Protection Commission) — prompt, with details within reasonable time | Subjects | Any breach with significant risk |
| **SG PDPA §26B (2020 amendment)** | PDPC within 3 calendar days of assessment; subjects "as soon as practicable" | Subjects | Significant scale OR significant harm |
| **Brazil LGPD Article 48** | ANPD "in a reasonable period" (rule-making finalized 2024) | Affected subjects | Risk or harm |
| **Canada PIPEDA + CPPA** | Office of Privacy Commissioner — "as soon as feasible"; record kept regardless | Affected individuals where risk of significant harm | Risk of significant harm |
| **India DPDPA Section 8(6)** | Data Protection Board — "as soon as possible" | Data Principals (subjects) | Personal data breach (broadly) |

The per-pack breach workflow steps:

1. **Detect.** Incident-detection signal from
   `microservices/observability/` SIEM (per the upcoming security
   substrate ADR) or manual report via `oyatie.security.incident-
   response`. Detection triggers per-pack workflow instances for every
   pack potentially affected.
2. **Triage.** Within 1 hour of detection, the on-call SecOps
   `oyatie.security.incident-response` principal triages: confirmed
   breach vs false positive; preliminary scope.
3. **Scope.** Within 4-6 hours, full scope analysis: which data classes
   touched; which tenants affected; which jurisdictions implicated.
   The DSAR cascade engine (per ADR-0242 §D-4 Appendix B + the
   Ontology IP-013) enumerates affected records.
4. **Decide notification.** Workflow consults the per-pack breach
   workflow per the matrix above. Each pack/jurisdiction independently
   determines whether and when to notify.
5. **Notify.** Workflow Engine durable workflow constructs per-
   regulator notice from template (per `breach_notification/templates/
   <pack-id>/regulator-notice.md.tmpl`); per-subject notice from
   template (`subject-notice.md.tmpl`); transmits via the regulator's
   declared endpoint (e.g., HHS OCR portal; ICO online form; KCS
   online form; OAIC portal); emits notifications to subjects (via
   `microservices/mail/` + `microservices/messenger/` + optional SMS
   per the subject's contact preference).
6. **Remediate.** Workflow tracks remediation actions; each remediation
   step audits its completion.
7. **Post-mortem.** Within 30 days, multispectrum-reviewed post-
   mortem published (internal to oyatie + DPO + counsel) per the
   pack's `breach_notification_workflow.post_mortem_required` flag.
8. **Regulator filing.** Final report (e.g., HIPAA annual aggregate
   for sub-500 breaches; NIS2 final report at 1 month) filed via the
   per-pack regulator-filing template.

Every step emits to the audit chain with class `BreachNotificationStep`
under the pack's audit stream + the affected tenant's audit stream.

The breach-notification-substrate is a NEW substrate µservice
provisioned at
`microservices/breach-notification/` per ADR-0249 substrate doctrine.

### D-9. De-identification engine substrate

A de-identification engine substrate
`microservices/de-identification/` provides primitives required by
multiple packs:

- **Tokenization.** Replace direct identifiers (SSN, MRN, PAN, etc.)
  with non-reversible tokens; tokens stored in a separate tokenization
  vault with stricter access controls. Used by HIPAA, PCI DSS,
  KR-PIPA pseudonymization.
- **k-anonymity** (k ≥ 5 default for HIPAA Safe Harbor; configurable
  per pack). Generalization or suppression of quasi-identifiers.
- **l-diversity** for sensitive attributes (l ≥ 3 default).
- **t-closeness** for distribution-preserving anonymization.
- **Differential privacy** (ε-DP; ε ≤ 1 default for statistical
  release). Noise injection per the Laplace or Gaussian mechanism.
- **HIPAA Safe Harbor de-identification** (45 CFR §164.514(b)(2)) —
  18 identifier categories removed.
- **HIPAA Expert Determination** (45 CFR §164.514(b)(1)) — statistical
  expert sign-off on the de-identification method.
- **GDPR pseudonymization** (Article 4(5)) — replace identifiers with
  artificial identifiers; key separately managed.
- **Format-preserving encryption** for PCI DSS PAN tokenization (per
  PCI SSC Tokenization Guidelines 2024).

The engine exposes a Cedar-gated API; data classes that require
de-identification before egress invoke the engine.

### D-10. Encryption substrate

An encryption substrate (the existing `microservices/cloud-secrets/`
plus a new µservice or extension at
`microservices/encryption-substrate/`) provides:

- **Per-data-class KMS keys.** Each data class has its own key
  hierarchy (or root); rotation per pack's key-management requirements.
- **encryption-BYOK (Bring Your Own Encryption Key).** Per-tenant
  encryption-root key import/control: tenants supply a KMS root key
  (or HSM partition) used to wrap the per-data-class key hierarchy
  for their tenant. This is the **encryption-BYOK** concern owned
  by this ADR (§D-10) and surfaced by the `byok_enabled` BOOLEAN
  column on `tenants` per ADR-0244 §D-3. encryption-BYOK is the
  same code path for every pack; the pack declares whether
  encryption-BYOK is permitted, required, or forbidden.

  **Disambiguation (per the corpus-rigor-audit-2026-05-21 finding
  that 7 IPs conflated the two acronym senses):** encryption-BYOK
  is disjoint from **provider-BYOK** (the LLM / provider
  API-key provider-BYOK governed by ADR-0255 §D-4 + the
  `provider_credential_mode` enum on `tenants`). A tenant may opt
  into one, both, or neither; they share no code path, no key
  hierarchy, and no audit-event class. Cross-reference each
  consumer of the acronym to one of the two anchors:
  - **encryption-BYOK** → this ADR (ADR-0251 §D-10) +
    `byok_enabled` column.
  - **provider-BYOK** → ADR-0255 §D-4 +
    `provider_credential_mode` column.
- **HYOK (Hold Your Own Key).** External KMS reference; key never
  leaves customer infrastructure. Required for IL5/IL6 + some EU-
  sovereign deployments.
- **FIPS 140-2 / 140-3 modules.** FIPS 140-2 Level 2 for FedRAMP
  Moderate; Level 3 for FedRAMP High + IL4; Level 3 + NSA Type 1 for
  IL5+. The cell's substrate provider determines available FIPS
  level; cell certification level binds.
- **HSM-rooted.** AWS CloudHSM, Azure Dedicated HSM, on-prem
  Thales/Entrust HSM for IL5+ and EU-sovereign.
- **Post-quantum hybrid** (planned for Year 2 of platform operations).
  ML-KEM-768 (FIPS 203) + classical X25519 hybrid for transport
  encryption; ML-DSA (FIPS 204) for signatures; SLH-DSA (FIPS 205)
  for stateful signatures. NIST PQC standards finalized August 2024.
- **Crypto-shred** (key destruction) as a data-deletion mechanism.
  When the per-data-class key is destroyed, the underlying ciphertext
  becomes mathematically inaccessible; this satisfies GDPR Article
  17 erasure for tombstoned audit-chain Merkle entries that cannot
  be physically deleted (per ADR-0242 §D-4 Appendix B step 4).

The encryption substrate emits an `EncryptionOperation` audit event
class for every encrypt / decrypt / sign / verify / rotate / destroy
operation.

### D-11. Consent management substrate

A consent management substrate
`microservices/consent/` provides:

- **Per-tenant per-purpose consent records.** Each subject (data
  subject) has consent records per (tenant_id, purpose_id, lawful_basis).
- **GDPR Article 7 conformance.** Withdrawal as easy as grant; granular
  per-purpose; auditable; freely given; specific; informed; unambiguous;
  records demonstrate consent.
- **KR-PIPA Article 22 conformance.** Per-purpose consent; minor-
  consent (age < 14 requires guardian); separate consent for marketing.
- **EU AI Act consent.** AI-system-specific transparency consent per
  Article 50 (AI-generated content disclosure) and tier-based per
  ADR-0144.
- **HIPAA authorization** (45 CFR §164.508) for uses + disclosures
  beyond TPO (Treatment, Payment, Operations).
- **Lifecycle.** Grant → active → revoke → revoked. Revocation does
  not retroactively invalidate processing performed before revocation
  but stops further processing. Audit trail preserved.
- **Cookie + tracking consent** for EU ePrivacy Directive + cookie law
  (Directive 2002/58/EC + Member State transpositions).

The substrate is Cedar-evaluated: any action requiring consent
queries the consent state via Cedar context attribute
`subject.consent[purpose_id]`. Missing consent yields `Forbid` with
human-readable reason.

### D-12. Per-pack DPIA template

Each pack carries a DPIA (Data Protection Impact Assessment) template
that the installing tenant must complete:

- **GDPR Article 35.** DPIA required for high-risk processing (large-
  scale special category, public-area systematic monitoring, etc.).
- **HIPAA Risk Analysis** (45 CFR §164.308(a)(1)(ii)(A)). Equivalent.
- **EU AI Act High-Risk DPIA / FRIA** (Fundamental Rights Impact
  Assessment per Article 27 for public-authority deployers).
- **KR-PIPA Privacy Impact Assessment** (개인정보 영향평가) per Article
  33.

The template structure:

```markdown
# DPIA Template — <pack-id> v<version>

## 1. Processing description
   - Purpose
   - Lawful basis
   - Data subjects + numbers
   - Data categories
   - Recipients
   - Retention period
   - Cross-border transfers

## 2. Necessity + proportionality assessment

## 3. Risk identification
   - Per-risk: likelihood × severity × affected rights

## 4. Mitigation measures
   - Technical
   - Organizational

## 5. Residual risk

## 6. Stakeholder consultation evidence

## 7. DPO opinion (if required)

## 8. Tenant DPO signature

## 9. oyatie council-privacy review signature (where applicable)

## 10. Review cadence + next review date
```

Templates live in
`microservices/governance/packs/<pack-id>/v<version>/dpia-template.md`.

### D-13. Audit chain per-pack stream

Each pack declares an audit-chain stream class per its
`audit_chain_requirements.stream_class`. Streams are
Merkle-sealed per period (period typically 1 hour for active streams;
24 hours for low-volume). Per-pack retention rules:

| Pack | Audit retention minimum | Cold storage tier after | Notes |
|---|---|---|---|
| HIPAA-2024 | 6 years from creation OR from last action on the record (whichever later) | 90 days | HIPAA §164.316(b)(2)(i) |
| SOX-404-2024 | 7 years | 90 days | SOX §103(a)(2)(A) |
| KR-FSS-2024 | 3 years (default); 5 years for transaction records | 90 days | KR 전자금융감독규정 |
| GDPR | Subject to DSAR + erasure; default retention per purpose | n/a (per purpose) | DPA + tenant config |
| PCI DSS | 1 year minimum (most logs); 3 months immediately searchable | 90 days | PCI DSS Req 10.5.1 (4.0.1) |
| FedRAMP Moderate/High | 3 years | 90 days | NIST 800-53 AU-11 |
| EU AI Act high-risk system logs | 6 months minimum; up to retention required by Member State | 90 days | EU AI Act Art. 12 |
| FERPA | Retain per institution; minimum until student records purged | n/a | FERPA + state law overlay |
| FDA 21 CFR Part 11 | Per the predicate rule (typically duration of regulated activity + minimum retention) | n/a | 21 CFR Part 11 + predicate rules |
| ISO 27001:2022 | 3 years (typical audit scope) | 90 days | ISO 27001 audit cycle |
| Default (no pack) | 1 year (general operational) | 30 days | oyatie baseline |

Cold-storage tiering per ADR-0241 + ADR-0249 (DR substrate). Cold-
tiered audit retrievable within 24 hours.

### D-14. Pack composition semantics

When a tenant installs multiple packs `P_1, P_2, ..., P_n`:

- **Cedar fragments union.** Effective policy is the union of all
  baseline + overlay + pack + tenant fragments per ADR-0243 §D-4.
  Any permit in any layer permits; any forbid in any layer forbids
  (deny wins).
- **Data class extensions union.** Effective data-class registry
  for the tenant is baseline + union of all packs' `data_class_
  extensions[]`. The same data instance may carry multiple
  classes (e.g., a Berlin patient's record is both `PHI` from
  HIPAA-2024 AND `EU_PERSONAL_DATA_ARTICLE_9` from EU-GDPR-2018-
  baseline). Encryption + retention + access rules apply for each
  class.
- **Retention rules take MAX.** Per data class, the effective
  retention is the maximum of the per-class retention rules across
  all installed packs. (E.g., if HIPAA says 6 years and SOX says 7
  years for an audit row touched by both, retention is 7 years.)
  Cold-storage tier is the maximum of the tier thresholds.
- **Cross-tenant rules take MOST RESTRICTIVE.** If pack P_1's
  `cross_tenant_rules.cross_pack_traffic_default = "forbidden"` and
  pack P_2's is `"case-by-case-cedar-permit"`, the effective rule is
  the more restrictive (forbidden).
- **Consent requirements compose.** Each purpose requiring consent
  under any pack requires consent. Pack-specific consent flows compose.
- **Breach notification deadlines compose.** All applicable
  jurisdictions' deadlines apply; the tightest deadline drives the
  workflow (typically KR-PIPA's 24 hours if any KR data subject
  affected).

### D-15. Certification level inheritance (cell certifications as a SET)

Cell certification is a **set, not a single level.** Inheritance is
implicit through the set membership rules in §D-4. Concretely:

- A `hipaa-certified` cell is also a `general` cell (because the
  certification matrix specifies `hipaa-certified` as "All of `general`
  PLUS HIPAA-specific"). The cell declares both `general` AND `hipaa-
  certified` in its certifications set.
- A `fedramp-high` cell declares `general`, `fedramp-moderate`, and
  `fedramp-high`.
- An `il6` cell declares `general`, `fedramp-moderate`, `fedramp-high`,
  `il4`, `il5`, AND `il6`.
- A cell certified for `hipaa-certified` AND `eu-sovereign` declares
  `general`, `hipaa-certified`, `eu-sovereign` (and any chains they
  imply).

Cell-certification-coherence is CI-checked by
`oya gate validate cell-certification-coherence`: if a cell declares
`fedramp-high` but not `fedramp-moderate` or `general`, the check
fails.

Mutually-exclusive certifications: `il6` is mutually exclusive with
non-classified certifications because IL6 requires a SECRET-classified
substrate that cannot also host unclassified workloads safely.
`forbidden_co_pack_certifications[]` on each pack encodes the
exclusions.

### D-16. Auditor evidence package emission

Each pack declares a `regulator_evidence_cadence`. The platform emits
auto-generated evidence packages on cadence:

- **Audit-chain class `ComplianceEvidence`** events emitted per pack
  per cadence period.
- **Evidence package contents** (per pack, per period):
  - Per-tenant compliance state (active packs, agreement status,
    DPIA reviews).
  - Per-tenant audit-chain extract for the period (filtered to pack-
    relevant events).
  - Per-cell certification evidence (last attestation, drill receipts).
  - Per-data-class operational counts (records touched, accesses,
    egresses, denials).
  - Per-Cedar-fragment evaluation count + decision distribution.
  - Per-incident records (with notification-deadline conformance
    proof).
  - Pack-version + signing-key chain proof (verifiable via Sigstore).
- **Regulator pull endpoint.** Each pack declares an endpoint
  (`regulator_evidence_cadence.regulator_pull_endpoint`) that allows
  the regulator (authorized via Cedar) to fetch the package. For
  regulators without a digital intake, the package is exported as a
  signed PDF + JSON bundle via the `microservices/governance/`
  evidence-export workflow.

Evidence packages are immutable; once emitted, the period's package
is sealed by Merkle root and stored in the audit-chain.

## Alternatives considered

### Alt-1. Per-regulation bespoke code

Continue handling each regulation as ad-hoc code: HIPAA-touching paths
tagged by hand; EU AI Act tiers in their own validator; GDPR DSAR
hand-stitched per µservice; no shared abstraction.

**Pros:**

- Zero migration cost — current portfolio shape.
- Each regulation's nuances captured in the team that owns the
  µservice; deep regulatory knowledge embedded.

**Cons:**

- **Superlinear cost growth.** Each new regulation requires touching
  every µservice it might apply to. With 38+ regulations on the
  horizon, the cost is unbounded.
- **Drift between regulations.** Common substrate (DSAR cascade,
  breach notification, consent management, encryption) is reimplemented
  per regulation, drifting in subtle ways. Tamper-detection coverage
  uneven (same problem as ADR-0242 audience-as-µservice-scope).
- **No regulator-evidence-emission cadence.** Each regulator gets
  bespoke evidence packets, with inconsistent contents and schedules.
- **Onboarding a regulated tenant is human-mediated.** Sales engineer
  + compliance officer + DPO meet, design the deployment, hand-craft
  config, hand-craft policy. Not autonomous-implementation-compatible.
- **Pack version control impossible.** When PCI DSS 4.0.1 → 4.1
  arrives, there's no obvious "migrate all PCI tenants" workflow.
- **Cross-pack rules undefined.** When a tenant subject to both HIPAA
  and PCI tries to interact with a tenant subject to only HIPAA, no
  unified rule decides the outcome.
- **Contradicts every named hyperscaler reference.** AWS Audit
  Manager, Microsoft Purview Compliance Manager, Google Assured
  Workloads, Oracle, Salesforce HealthCloud, Databricks all
  package compliance.

**Rejected** because the cost is unbounded and every industry
reference disagrees.

### Alt-2. One-size-fits-all compliance bundle

Define a single "fully-compliant" bundle that satisfies all known
regulations (the strictest union). Apply to all tenants.

**Pros:**

- One bundle to maintain — no per-pack drift.
- Simplest configuration; no tenant has to decide which pack.

**Cons:**

- **Most restrictive rule applies everywhere.** A general-SaaS tenant
  is subjected to HIPAA breach-notification timing + FedRAMP personnel
  controls + DoD IL5 air-gap requirements. Overhead crushes the use
  case.
- **Impossible to satisfy all regulations simultaneously.** Regulations
  conflict (e.g., GDPR Article 17 erasure vs SOX retention; FedRAMP
  US-persons-only personnel vs EU-sovereign EU-personnel-only). A
  single bundle cannot satisfy mutually-exclusive requirements.
- **Tenants in unregulated industries pay for irrelevant overhead.**
- **Cell-pinning becomes degenerate.** Every tenant must live in a
  cell that satisfies everything; only IL5+ cells qualify; cost
  prohibitive.
- **No regulator-evidence specificity.** Per-regulator evidence
  packages can't be cleanly separated.

**Rejected** because regulations conflict and one bundle cannot
satisfy them simultaneously.

### Alt-3. Compliance as documentation (no enforcement)

Maintain compliance documentation (per pack) but do not enforce
controls programmatically. Trust that engineers will read the docs
and behave accordingly.

**Pros:**

- Zero engineering cost.
- Documentation already exists per regulation in industry literature.

**Cons:**

- **Doctrine of "compliance theater"** — auditors increasingly reject
  documentation-only compliance. SOC 2 Type II requires evidence of
  operation; FedRAMP requires continuous monitoring; CSAP requires
  technical control demonstration; HHS OCR random audits require
  evidence trail; EU regulators (per ICO 2024 enforcement guidance)
  expect technical controls.
- **Drift inevitable.** Engineers under deadline pressure cut
  corners; documented rules diverge from runtime behavior.
- **Audit-chain gap.** No per-pack evidence emission; regulators
  can't pull packages.
- **DSAR cascade still bespoke.** No primitive for "the union of
  data classes touched by the subject across all packs."
- **Cross-pack rules unenforced.** PHI can leak to general tenants
  if no Cedar gate stops it.

**Rejected** because regulators and auditors require enforcement
evidence, not just documentation.

### Alt-4. Outsource compliance (third-party SaaS)

Use a third-party compliance-management platform (Drata, Vanta,
Secureframe, Tugboat Logic, etc.) for compliance enforcement.

**Pros:**

- Vendor handles per-regulation interpretation.
- Faster onboarding for SOC 2 / HIPAA / PCI / ISO 27001 (their primary
  audience).
- Pre-built integrations with cloud providers.

**Cons:**

- **Violates ADR-0211 in-house tech stack preference.** Compliance
  is central to the platform's value proposition; outsourcing it
  cedes strategic ground.
- **Vendor lock-in.** Compliance state stored in vendor's system;
  audit-chain dependencies external.
- **Vendor SLA risk.** Compliance state availability tied to vendor
  uptime; outages during audit windows cause regulator escalation.
- **Doesn't cover the long tail** (DoD IL5/IL6, KR-FSS, KSA NDMO,
  JP government, etc.). Vendors target SOC 2 + HIPAA + PCI + GDPR
  primarily; specialty regimes underserved.
- **Cross-tenant rules + cross-pack rules can't be enforced** by an
  external compliance tool — only by the platform's own gates.
- **Per-tenant overlay impossible** because the tenant model is
  internal.
- **Hyperscaler-reference disagreement.** AWS, Microsoft, Google,
  Oracle, Salesforce, Databricks all build their own compliance
  primitives.

**Rejected** because compliance is platform-central and outsourcing
it doesn't scale to the full regulatory surface.

### Alt-5. Compliance Pack primitive (CHOSEN)

The selected alternative, fully specified in §Decision.

**Pros:**

- **First-class primitive.** Compliance is the unit of regulator
  response, drift control, DPIA, breach notification, BAA/DPA,
  onboarding, sunset.
- **Cedar-evaluated.** Packs are Cedar fragment bundles per ADR-0243
  §D-2 + §D-4. Composition is uniform with other policy.
- **Auditor-evidence emission is automatic.** Per-pack regulator-pull
  endpoint; per-cadence evidence packages.
- **Cross-pack rules enforceable.** Matrix per §D-6 evaluated at
  every cross-tenant call.
- **Cell pinning enforceable.** Tenant + pack + cell coherence
  CI-checked + admission-time-checked.
- **Pack versioning enables migration.** When a regulation updates,
  pack version bumps; tenants migrate via workflow.
- **Hyperscaler-grade.** Matches AWS Audit Manager, Microsoft Purview,
  Google Assured Workloads, Salesforce, Databricks, Oracle patterns.
- **In-house** per ADR-0211; cedes nothing strategically.
- **Autonomous-implementation-compatible.** Regulated-tenant onboarding
  is a deterministic workflow.

**Cons:**

- **Pack authoring requires regulatory expertise.** Mitigation: each
  pack has an explicit authoring team + DPO + counsel; multispectrum
  review F11 facet validates regulatory interpretation; external
  legal counsel countersigns where required.
- **Pack lifecycle is non-trivial.** Mitigation: lifecycle is fully
  specified in §D-2; sunset + tombstone clean.
- **One-time substrate cost.** New substrates: breach-notification,
  de-identification, consent, encryption-substrate enhancements,
  identity-verification (KYB). Mitigation: each substrate is bounded
  in scope; many already exist as fragments.

**Accepted** as the foundational keystone for compliance.

## Consequences

### Positive

1. **Regulator response is unified.** Per-pack evidence packages,
   per-pack notification workflows, per-pack DPIA templates,
   per-pack BAA/DPA agreements. Regulator interactions are
   deterministic rather than bespoke.
2. **Drift control is automatic.** Pack version + signing chain
   prevents un-attributed change. Multispectrum review catches drift
   before publication.
3. **Cross-pack rules enforceable at every call.** PHI cannot leak
   to PCI-only tenants because the Cedar fragment denies it.
4. **Onboarding regulated tenants is autonomous.** The pack-install
   workflow runs end-to-end without human-mediated translation.
5. **Per-pack drift detection.** Pack-version diff + Cedar fragment
   diff visible in the audit chain.
6. **Hyperscaler-shape.** Matches AWS / Microsoft / Google /
   Salesforce / Databricks compliance-bundle patterns.
7. **Per-tenant overlay clean.** Tenants install packs; effective
   policy aggregates packs; tenant-fragments restricted to
   additionally restrict.
8. **Layered jurisdictional overlays.** A tenant operating in both
   EU and KR installs EU-GDPR-2018-baseline + KR-PIPA-2023-amendment;
   per-jurisdiction overlays apply per the call's jurisdictional
   context.
9. **De-identification + encryption + consent unified.** Substrate
   primitives serve multiple packs; engineering investment compounds.
10. **EU AI Act tier model (ADR-0144) integrates cleanly.** EU-AI-
    ACT-2024 pack carries the tier model; per-deployment-context
    tier evaluation occurs inside the pack's Cedar fragments.

### Negative

1. **Pack-authoring complexity.** Each pack requires regulatory
   expertise + DPO + counsel + multispectrum review. Mitigation:
   pack-authoring runbook + per-pack authoring team + amortization
   across all tenants subject to the pack.
2. **Cell-certification operational discipline.** Each cell must
   maintain its declared certification levels (annual attestations,
   ongoing controls). Mitigation: per-cell certification monitor +
   pre-expiry escalation + ops-compliance runbook.
3. **Pack-version-migration disruption.** When a pack version
   sunsets, tenants must migrate within a grace period. Mitigation:
   migration workflow with explicit cutover window + opt-in early
   migration.
4. **Audit-chain volume growth.** Per-pack streams + per-evaluation
   audit emission grow the audit-chain corpus. Mitigation: cold-
   storage tiering per ADR-0249; per-pack retention rules.
5. **Cross-pack interaction complexity for tenants with many packs.**
   Cedar policy evaluation cost grows with installed-pack count.
   Mitigation: per-cell evaluator cache (per ADR-0243 §D-6);
   evaluation budget remains < 1ms p99.

### Operational

1. **New CI lanes:**
   - `oya-check-compliance-pack-schema` — verifies pack content
     conforms to D-1 JSON Schema.
   - `oya-check-compliance-pack-signature` — verifies Ed25519
     signature + Sigstore Rekor entry + cosign attestation.
   - `oya-check-cell-certification-coherence` — verifies cell
     certifications form a valid set (no mutually-exclusive pairs;
     no missing-prerequisite chains).
   - `oya-check-tenant-pack-cell-pinning` — verifies tenant +
     installed packs + home_cell + dr_cell coherence per §D-5.
   - `oya-check-cross-pack-traffic-cedar-gated` — verifies every
     cross-pack call path has a Cedar fragment per §D-6.
   - `oya-check-baa-dpa-coverage` — verifies every installed pack
     with `agreement_template_refs` has signed agreements.
   - `oya-check-breach-notification-workflow-coverage` — verifies
     every pack with a `breach_notification_workflow` has the
     workflow deployed in Workflow Engine.
   - `oya-check-consent-record-coverage` — verifies every pack with
     `consent_requirements` has consent records for tenant's
     subjects.
   - `oya-check-dpia-template-coverage` — verifies every installed
     pack has a signed DPIA on file.
   - `oya-check-auditor-evidence-emission` — verifies per-pack
     `ComplianceEvidence` events emit per cadence.

2. **New µservice surfaces:**
   - `microservices/breach-notification/` (NEW substrate).
   - `microservices/de-identification/` (NEW substrate).
   - `microservices/consent/` (NEW substrate).
   - `microservices/identity-verification/` (NEW substrate; provides
     KYB / KYC).
   - `microservices/encryption-substrate/` (enhancement to the
     existing `microservices/cloud-secrets/`; possibly factored as
     a peer substrate µservice per ADR-0245).
   - `microservices/governance/packs/` (pack registry).

3. **New specs:**
   - `/specs/compliance-pack-schema.json` (canonical schema).
   - `/specs/cell-certification-level-matrix.json` (canonical matrix).
   - `/specs/microservices/governance.json` (pack registry surfaces).
   - `/specs/microservices/breach-notification.json`.
   - `/specs/microservices/de-identification.json`.
   - `/specs/microservices/consent.json`.

4. **Pack registry shape:**
   - Postgres + Citus shard on `(pack_id, version)`.
   - SeaweedFS immutable blob storage for pack content + agreements
     + DPIA templates.
   - cosign-attested + Sigstore Rekor-logged.

5. **Workflow Engine workflows:**
   - `pack_install`
   - `pack_uninstall`
   - `pack_version_migration`
   - `agreement_lifecycle` (per §D-7)
   - `breach_notification` (per §D-8, per-jurisdiction templates)
   - `dpia_review` (cadence per pack)
   - `regulator_evidence_emission` (cadence per pack)
   - `cell_certification_renewal` (annual per certification level)
   - `subprocessor_disclosure_notification`

6. **Observability:**
   - Per-pack dashboard: active tenants, evidence-package cadence
     adherence, agreement expiration risk, breach-workflow drill
     status, Cedar fragment evaluation counts.
   - Per-cell certification status dashboard: per-certification-level
     expiration calendar.
   - Per-tenant compliance posture dashboard: installed packs,
     DPIA review status, BAA/DPA status, consent coverage.

7. **Tenancy admin console:**
   - Pack catalog: browse + install + uninstall.
   - DPIA editor + signing.
   - Agreement viewer + e-signing.
   - Compliance posture display.

### Sustainability

- Per-pack audit-chain volume grows with active tenants. Cold-
  storage tiering (per ADR-0249) bounds active storage. Carbon
  impact per tenant per pack is tracked via FinOps + sustainability
  tag (per ADR-0174).
- De-identification + encryption operations consume compute;
  budget added to cell-level compute capacity planning per
  ADR-0241/0249 portfolios.

### Compliance — how compliance machinery itself is compliant

A meta-consequence: the compliance machinery itself is subject to
audit. The pack registry, signing infrastructure, audit-chain emission,
DPIA review workflows, breach-notification workflows are subject to:

- **SOC 2 Type II coverage** because the platform claims trust
  service criteria including CC6 (logical + physical access),
  CC7 (system operations), CC8 (change management).
- **ISO 27001:2022 coverage** because the platform claims ISO
  27001; the compliance machinery is part of the ISMS.
- **ISO 42001:2023** for the AI-system aspects of compliance pack
  authoring + multispectrum review (which is AI-mediated per
  ADR-0144 + ADR-0220 retired + ADR-0255).
- **FedRAMP continuous monitoring** for the pack registry +
  signing-key chain.
- **HIPAA Security Rule** for the breach-notification substrate
  itself (because it processes ePHI during a real breach).
- **Internal audit by oyatie's external auditor** (CPA firm) on the
  pack-signing process.
- **Multispectrum review F11 (regulatory compliance facet)** on
  every pack publication.

The platform is "auditing the auditor": the compliance machinery
must itself withstand the same audit pressure it applies to tenant
deployments. This is the dogfooded-compliance pattern from
ADR-0242 §D-4 generalized to the compliance-tooling itself.

## Implementation surface

### COMPLIANCE-001 contract-slice machine-readable contract

The COMPLIANCE-001 review slice adds only schema/fixture/test-contract
shape for SOC 2 evidence, CMP consent, cell-certification state, and
portability-export metadata. The authoritative governed-surface list and
claim boundary for this review slice live in
`/specs/compliance-001-contract-slice.json`; this ADR remains rationale
and source context only, not the operative reachability contract.

This ADR intentionally justifies the PR-local contract-slice artifacts
that make that machine-readable contract executable without granting
production, runtime, certification, auditor-acceptance, tenant-activation,
or portability-service authority:

- `specs/compliance-001-contract-slice.json`
- `specs/fixtures/compliance-pack/compliance-001-soc2-cmp-portability.fixture.json`
- `specs/fixtures/compliance-pack/compliance-001-portability-export-manifest.fixture.json`
- `ci/facade/contract-slice-conformance/contract-slice-policy.json` (owned-Rust/Buck2 gate policy entries; retires the former repo-local Python validator)

The following artifacts are required for this keystone to be considered
implemented:

| Artifact | Status |
|---|---|
| `/specs/compliance-pack-schema.json` | NEW |
| `/specs/cell-certification-level-matrix.json` | NEW |
| `/specs/microservices/governance.json` (pack-registry surfaces) | NEW |
| `/specs/microservices/breach-notification.json` | NEW |
| `/specs/microservices/de-identification.json` | NEW |
| `/specs/microservices/consent.json` | NEW |
| `/specs/microservices/identity-verification.json` | NEW |
| `/specs/microservices/encryption-substrate.json` | NEW (or extension to cloud-secrets) |
| `microservices/governance/packs/` directory + tooling | NEW |
| `microservices/governance/packs/HIPAA-2024/v1.0.0/` (bootstrap pack) | NEW |
| `microservices/governance/packs/SOC2-T2-2024/v1.0.0/` (bootstrap pack) | NEW |
| `microservices/governance/packs/EU-GDPR-2018-baseline/v2024.0.0/` (bootstrap pack) | NEW |
| `microservices/governance/packs/PCI-DSS-L1-v4.0.1/v1.0.0/` (bootstrap pack) | NEW |
| `microservices/governance/packs/FedRAMP-Moderate-v5/v1.0.0/` (bootstrap pack) | NEW |
| `microservices/governance/packs/EU-AI-ACT-2024/v1.0.0/` (bootstrap pack, integrates ADR-0144 tier model) | NEW |
| `microservices/governance/packs/KR-PIPA-2023-amendment/v1.0.0/` (bootstrap pack) | NEW |
| `microservices/governance/packs/EU-NIS2-2022/v1.0.0/` (bootstrap pack) | NEW |
| `microservices/governance/packs/EU-DSA-2022/v1.0.0/` (bootstrap pack) | NEW |
| `microservices/breach-notification/` µservice | NEW |
| `microservices/de-identification/` µservice | NEW |
| `microservices/consent/` µservice | NEW |
| `microservices/identity-verification/` µservice | NEW |
| `microservices/encryption-substrate/` µservice (or extension to cloud-secrets) | NEW |
| `microservices/governance/workflows/pack_install.yaml` | NEW |
| `microservices/governance/workflows/pack_uninstall.yaml` | NEW |
| `microservices/governance/workflows/pack_version_migration.yaml` | NEW |
| `microservices/governance/workflows/agreement_lifecycle.yaml` | NEW |
| `microservices/governance/workflows/breach_notification/*.yaml` (per-jurisdiction templates) | NEW |
| `microservices/governance/workflows/dpia_review.yaml` | NEW |
| `microservices/governance/workflows/regulator_evidence_emission.yaml` | NEW |
| `microservices/governance/workflows/cell_certification_renewal.yaml` | NEW |
| `microservices/governance/workflows/subprocessor_disclosure_notification.yaml` | NEW |
| `microservices/cloud-iac/ARCHITECTURE.md#cell-provisioning` | UPDATE — certification level admission metadata owner after ADR-0333 |
| `microservices/tenancy/src/pack_install.rs` | NEW |
| `microservices/tenancy/src/pack_cell_pinning_validator.rs` | NEW |
| `microservices/policy-engine/fragments/baseline/cross-pack-matrix.cedar` | NEW |
| `microservices/policy-engine/fragments/baseline/cell-certification-coherence.cedar` | NEW |
| `microservices/policy-engine/fragments/baseline/pack-install-eligibility.cedar` | NEW |
| `microservices/audit-chain/src/per_pack_stream_provisioner.rs` | NEW |
| `microservices/observability/dashboards/compliance-packs.md` | NEW |
| `microservices/observability/dashboards/cell-certification-status.md` | NEW |
| `microservices/observability/dashboards/per-tenant-compliance-posture.md` | NEW |
| `crates/oya-shared-de-identification-client/` | NEW |
| `crates/oya-shared-consent-client/` | NEW |
| `crates/oya-shared-breach-notification-client/` | NEW |
| `tools/oya-check-compliance-pack-schema/` | NEW |
| `tools/oya-check-compliance-pack-signature/` | NEW |
| `tools/oya-check-cell-certification-coherence/` | NEW |
| `tools/oya-check-tenant-pack-cell-pinning/` | NEW |
| `tools/oya-check-cross-pack-traffic-cedar-gated/` | NEW |
| `tools/oya-check-baa-dpa-coverage/` | NEW |
| `tools/oya-check-breach-notification-workflow-coverage/` | NEW |
| `tools/oya-check-consent-record-coverage/` | NEW |
| `tools/oya-check-dpia-template-coverage/` | NEW |
| `tools/oya-check-auditor-evidence-emission/` | NEW |
| `docs/standards/compliance-pack-authoring.md` | NEW |
| `docs/standards/cell-certification-level-matrix.md` | NEW |
| `docs/runbooks/pack-publication-ceremony.md` | NEW |
| `docs/runbooks/pack-version-migration.md` | NEW |
| `docs/runbooks/breach-notification-response.md` | NEW |
| `docs/runbooks/regulator-evidence-pull.md` | NEW |
| `docs/runbooks/cell-certification-renewal.md` | NEW |

## Verification

- [ ] `/specs/compliance-pack-schema.json` exists and validates the bootstrap packs.
- [ ] `/specs/cell-certification-level-matrix.json` exists with all certification levels from §D-4.
- [ ] `microservices/governance/packs/SOC2-T2-2024/v1.0.0/` exists and signed by oyatie-compliance-office Ed25519 key.
- [ ] `microservices/governance/packs/EU-GDPR-2018-baseline/v2024.0.0/` exists, signed, with DPIA template + DPA template.
- [ ] `microservices/governance/packs/HIPAA-2024/v1.0.0/` exists, signed, with DPIA + BAA + breach-workflow.
- [ ] At least one cell registered with `general` certification level; one with `eu-sovereign`; one with `hipaa-certified`.
- [ ] `oya gate validate compliance-pack-schema` reports 100% pass on bootstrap packs.
- [ ] `oya gate validate compliance-pack-signature` reports 100% pass on bootstrap packs.
- [ ] `oya gate validate cell-certification-coherence` reports 100% pass on registered cells.
- [ ] `oya gate validate tenant-pack-cell-pinning` reports 100% pass on test-tenant + bootstrap-packs scenarios.
- [ ] `oya gate validate cross-pack-traffic-cedar-gated` reports 100% Cedar coverage on the cross-pack matrix from §D-6.
- [ ] `oya gate validate baa-dpa-coverage` reports 100% on test-tenant installations.
- [ ] `oya gate validate breach-notification-workflow-coverage` reports 100% on installed packs.
- [ ] `oya gate validate consent-record-coverage` reports 100% on tenant + purpose pairs.
- [ ] `oya gate validate dpia-template-coverage` reports 100% on installed packs.
- [ ] `oya gate validate auditor-evidence-emission` reports cadence adherence per pack.
- [ ] Per-pack `ComplianceEvidence` audit event emitted at the declared cadence.
- [ ] Pack install workflow runs end-to-end for a test tenant (HIPAA-2024 install + DPIA + BAA + cell pinning + cedar activation).
- [ ] Pack uninstall workflow runs end-to-end (with data-class erasure + agreement-termination).
- [ ] Pack version migration drill (SOC2-T2-2024 v1.0.0 → v1.0.1) completes without service disruption.
- [ ] Breach notification drill (simulated HIPAA breach) completes per HIPAA §164.404 60-day deadline.
- [ ] Breach notification drill (simulated GDPR breach) completes per Article 33 72-hour deadline.
- [ ] Breach notification drill (simulated KR-PIPA breach) completes per Article 34 24-hour deadline.
- [ ] Cross-pack-deny drill: PHI-touching action from HIPAA tenant to PCI-only tenant denied at Cedar gate.
- [ ] Cell-certification-renewal workflow runs ahead of certification-expiry deadline.
- [ ] De-identification engine emits `EncryptionOperation` + `DeIdentificationOperation` audit events.
- [ ] Consent grant + revoke workflow emits `ConsentLifecycleEvent` events.
- [ ] Encryption substrate provides per-data-class KMS keys + encryption-BYOK + (planned Year 2) post-quantum hybrid.
- [ ] FedRAMP Moderate baseline controls — pack bootstrap demonstrated against AWS GovCloud test cell.
- [ ] EU AI Act pack v1.0.0 — tier evaluation per (archetype, deployment_context) tuple per ADR-0144 §Decision.
- [ ] DSAR cascade (per ADR-0242 §D-4 Appendix B) integrates with pack-aware data-class enumeration.
- [ ] HSM root-key + intermediate-key ceremony for pack-owner-key (oyatie-compliance-office) drilled at least once before BLOCKER promotion.
- [ ] Multispectrum review F11 facet (regulatory-compliance) trained reviewers + pilot review on a bootstrap pack.
- [ ] Tenancy admin console: pack catalog browser + install/uninstall workflow + compliance posture view.

## References

### Primary regulatory sources

- **HIPAA / HITECH:**
  - 45 CFR §164 Subpart C — Security Rule.
  - 45 CFR §164 Subpart D — Breach Notification Rule (§164.400-414).
  - 45 CFR §164 Subpart E — Privacy Rule.
  - 45 CFR §164.508 — Uses and disclosures requiring authorization.
  - 45 CFR §164.514(b) — De-identification (Safe Harbor + Expert
    Determination).
  - HITECH Act 2009 — Subtitle D (Privacy).
  - HHS OCR Audit Protocol 2024 update.
  - HHS OCR Breach Portal (ocrportal.hhs.gov).

- **PCI DSS:**
  - PCI DSS 4.0.1 (PCI SSC, June 2024).
  - PCI SSC Tokenization Product Security Guidelines 2024.
  - PCI SSC P2PE v3.1 Standard.
  - PCI Forensic Investigators (PFI) Program Guide 2024.

- **FedRAMP:**
  - FedRAMP Baseline Moderate Rev. 5 (2024).
  - FedRAMP Baseline High Rev. 5 (2024).
  - FedRAMP Continuous Monitoring Strategy Guide 2024.
  - FedRAMP Authorization Process 2024.
  - FedRAMP Authorization Boundary Guidance 2024.
  - FedRAMP 3PAO Assessment Process 2024.

- **DoD IL:**
  - DoD Cloud Computing Security Requirements Guide (SRG) v1r5 (DISA,
    2022 + 2024 updates).
  - DoD Cloud Computing SRG Sections 5.2-5.6 (IL2, IL4, IL5, IL6).
  - CMMC 2.0 (DoD CIO, 2024).
  - CJCSI 6510.01F — Information Assurance and Support to Computer
    Network Defense.
  - DoD 8500.01E — Information Assurance.

- **EU Regulations:**
  - Regulation (EU) 2016/679 — GDPR (effective 2018-05-25).
  - Regulation (EU) 2024/1689 — EU AI Act (effective 2024-08-01;
    phased application).
  - Directive (EU) 2022/2555 — NIS2 (transposition deadline
    2024-10-17).
  - Regulation (EU) 2022/2065 — DSA (effective 2024-02-17).
  - Regulation (EU) 2022/1925 — DMA (effective 2024-03-06).
  - Regulation (EU) 2023/2854 — Data Act (effective 2025-09-12).
  - Regulation (EU) 2024/1183 — Electronic Identity (eIDAS 2).
  - Directive 2002/58/EC — ePrivacy Directive.
  - Commission Decision (EU) 2021/914 — Standard Contractual Clauses.
  - Commission Implementing Decision (EU) 2023/1795 — EU-US Data
    Privacy Framework adequacy (Schrems II resolution).
  - EU AI Act Article 5 (prohibited practices).
  - EU AI Act Article 6 + Annex III (high-risk classification).
  - EU AI Act Article 9-15 (high-risk obligations).
  - EU AI Act Article 27 (FRIA — Fundamental Rights Impact
    Assessment).
  - EU AI Act Article 50 (transparency).
  - EU AI Act Article 52-54 (GPAI obligations).
  - EU AI Act Article 60 (post-market monitoring).
  - GDPR Article 7 — Consent conditions.
  - GDPR Article 17 — Right to erasure.
  - GDPR Article 22 — Automated individual decision-making.
  - GDPR Article 28 — Processor.
  - GDPR Article 32 — Security of processing.
  - GDPR Article 33 — Notification to supervisory authority.
  - GDPR Article 34 — Communication to data subjects.
  - GDPR Article 35 — DPIA.
  - GDPR Article 44-49 — Cross-border transfers.

- **UK + post-Brexit:**
  - UK GDPR (retained EU law, post-Brexit).
  - Data Protection Act 2018 (UK Parliament).
  - ICO Audit and Investigations 2024 Guidance.

- **Korea:**
  - 개인정보 보호법 (Personal Information Protection Act, 2011,
    2023-09-15 amendment).
  - 개인정보보호위원회 (Personal Information Protection Commission)
    enforcement notices 2024.
  - 의료법 (Medical Service Act, 2024 amendment).
  - 전자금융감독규정 (Financial Supervisory Service e-Financial
    Supervisory Regulation 2024).
  - 전자금융거래법 (Electronic Financial Transactions Act, 2024
    amendment).
  - 정보통신망법 (Act on Promotion of Information and Communications
    Network Utilization).
  - KISA ISMS-P (Information Security Management System +
    Personal Information) 2024.
  - KISA CSAP v3.1 (Cloud Security Assurance Program).
  - KCS (개인정보보호위원회) Breach Notification Guidelines 2024.

- **Japan:**
  - 個人情報の保護に関する法律 (APPI — Act on the Protection of
    Personal Information, 2022-04 amendment).
  - PPC (Personal Information Protection Commission) Guidelines 2024.
  - METI (Ministry of Economy, Trade, Industry) Cloud Security Mark
    2024.

- **Singapore:**
  - PDPA (Personal Data Protection Act 2012, 2020 amendment).
  - PDPC (Personal Data Protection Commission) Advisory Guidelines
    2024.
  - PDPC Guide on Active Enforcement 2024.

- **Australia:**
  - Privacy Act 1988 (Commonwealth) + 2024 reforms (Privacy Act
    Reform Act 2024).
  - Notifiable Data Breaches Scheme (Pt IIIC Privacy Act).
  - OAIC Guide to Securing Personal Information 2024.
  - Australian Privacy Principles (APP) 2024.

- **Middle East:**
  - KSA Personal Data Protection Law (Royal Decree M/19 of 9/2/1443
    Hijri; English translation 2023).
  - KSA SDAIA Cloud Computing Framework v1.0 (2023).
  - KSA NDMO (National Data Management Office) 2024 Framework.
  - UAE Federal Decree-Law No. 45/2021 (Personal Data Protection
    Law, effective 2023-01-02).

- **Other major jurisdictions:**
  - Brazil Lei Geral de Proteção de Dados (Law No. 13,709/2018; rules
    finalized 2024 by ANPD).
  - Canada PIPEDA (Personal Information Protection and Electronic
    Documents Act) + Bill C-27 / CPPA (Consumer Privacy Protection
    Act, advancing 2024).
  - Quebec Law 25 (Act to modernize legislative provisions as regards
    the protection of personal information, phased through 2024).
  - India Digital Personal Data Protection Act 2023 (rules 2024).
  - California Civil Code §1798.100-199.100 (CCPA + CPRA).
  - California Civil Code §1798.82 (Breach Notification).
  - 50-state US breach notification compendium (NAAG 2024 reference).
  - FERPA (20 USC §1232g + 34 CFR Part 99) 2024 amendments.
  - FDA 21 CFR Part 11 (2024 revision) — Electronic Records;
    Electronic Signatures.
  - SOX (Sarbanes-Oxley Act of 2002) §404 — Management Assessment of
    Internal Controls.
  - GLBA Safeguards Rule (16 CFR §314, 2023 revision).

- **International standards:**
  - ISO/IEC 27001:2022 — Information security management systems.
  - ISO/IEC 27701:2019 — Privacy information management.
  - ISO/IEC 27018:2019 — Cloud PII processor.
  - ISO 22301:2019 — Business continuity management.
  - ISO/IEC 42001:2023 — AI management system.
  - NIST SP 800-53 Rev. 5 (December 2020 + 2024 errata).
  - NIST SP 800-171 Rev. 3 (2024).
  - NIST AI Risk Management Framework 1.0 (NIST AI 100-1, January
    2023).
  - NIST FIPS 140-2 (2001) + FIPS 140-3 (2019, transition 2024).
  - NIST FIPS 203 (ML-KEM) — August 2024.
  - NIST FIPS 204 (ML-DSA) — August 2024.
  - NIST FIPS 205 (SLH-DSA) — August 2024.

### Industry sources

- **AWS Audit Manager** documentation (aws.amazon.com/audit-manager)
  + 2024 enhancements. Framework catalog (HIPAA, PCI DSS, FedRAMP
  Moderate/High, SOC 2, NIST 800-53, NIST 800-171, GDPR, ISO 27001,
  GxP, ENISA Cybersecurity Certification, AWS Foundational Technical
  Review).
- **Microsoft Purview Compliance Manager** documentation
  (learn.microsoft.com/en-us/purview/compliance-manager) + 2024
  expansion. Assessment templates catalog.
- **Google Cloud Assured Workloads** documentation
  (cloud.google.com/assured-workloads/docs) + 2024 expansion.
  Compliance regimes catalog (FedRAMP Moderate/High, IL2/IL4/IL5,
  IRS 1075, ITAR, HIPAA, EU Sovereign Controls, JP-2).
- **Oracle Cloud Compliance Documents**.
- **Salesforce HealthCloud, GovCloud, Financial Services Cloud**
  product documentation.
- **Snowflake Healthcare Data Cloud + Financial Services Data Cloud**
  documentation.
- **Cloudflare Compliance** (cloudflare.com/trust-hub).
- **Databricks Compliance Security Profile** documentation
  (docs.databricks.com/security/privacy/cmp-overview).
- **Stripe Compliance Documentation** (stripe.com/docs/security)
  — PCI DSS Level 1, SOC 1/2/3, ISO 27001, HIPAA-eligible.
- **AWS Builder's Library — "Designing for highly available systems"** + "Static stability using Availability Zones."
- **AWS re:Inforce 2024 — Audit Manager + Verified Permissions + GuardDuty integration.**
- **Microsoft Build 2024 — Purview Compliance Manager keynote.**
- **Google Cloud Next 2024 — Assured Workloads expansion.**
- **PCI SSC 2024 Community Meeting — Tokenization + scoping guidance.**
- **HIPAA Summit 2024 — HHS OCR enforcement trends.**
- **HIMSS 2024 — Healthcare cloud compliance trends.**

### Standards bodies + working groups

- **PCI Security Standards Council** (pcisecuritystandards.org).
- **HHS OCR** (hhs.gov/hipaa).
- **NIST Information Technology Laboratory** (nist.gov/itl).
- **FedRAMP PMO** (fedramp.gov).
- **DISA** (disa.mil).
- **EDPB (European Data Protection Board)** — guidelines.
- **ICO** (ico.org.uk) — UK.
- **CNIL** (cnil.fr) — France.
- **BfDI** (bfdi.bund.de) — Germany.
- **개인정보보호위원회** (pipc.go.kr) — Korea.
- **PPC** (ppc.go.jp) — Japan.
- **PDPC** (pdpc.gov.sg) — Singapore.
- **OAIC** (oaic.gov.au) — Australia.
- **SDAIA** (sdaia.gov.sa) — Saudi Arabia.
- **ANPD** (gov.br/anpd) — Brazil.
- **OPC** (priv.gc.ca) — Canada.
- **AICPA** — Trust Service Criteria 2017 + 2024 revisions.
- **CSA (Cloud Security Alliance)** STAR registry.
- **ISO Technical Committees TC 22 (security) + TC 42 (AI).**

### Cryptographic + signing infrastructure

- **Sigstore Rekor** transparency log (rekor.sigstore.dev).
- **cosign** signature tool (github.com/sigstore/cosign).
- **NIST FIPS 203 (ML-KEM)** — Post-Quantum Key Encapsulation
  Mechanism Standard, August 2024.
- **NIST FIPS 204 (ML-DSA)** — Post-Quantum Module-Lattice-based
  Digital Signature Standard, August 2024.
- **NIST FIPS 205 (SLH-DSA)** — Stateless Hash-Based Digital
  Signature Standard, August 2024.
- **RFC 8785** — JSON Canonicalization Scheme (JCS).
- **RFC 8032** — Edwards-Curve Digital Signature Algorithm (Ed25519).
- **RFC 3161** — Internet X.509 Time-Stamp Protocol (TSP).

### De-identification literature

- **Sweeney, "k-anonymity: A model for protecting privacy" (IJUFKS
  2002).**
- **Machanavajjhala et al., "L-diversity: Privacy beyond
  k-anonymity" (TKDD 2007).**
- **Li, Li, Venkatasubramanian, "t-Closeness: Privacy beyond
  k-anonymity and l-diversity" (ICDE 2007).**
- **Dwork, "Differential Privacy" (ICALP 2006).**
- **Dwork, McSherry, Nissim, Smith, "Calibrating Noise to
  Sensitivity in Private Data Analysis" (TCC 2006).**
- **NIST Special Publication 800-188 — De-Identification of
  Personal Information.**
- **HHS Guidance on De-identification of Protected Health Information
  per HIPAA Privacy Rule (2012 + 2024 updates).**

### Internal portfolio ADRs

- **ADR-0009** — Cell architecture per-tenant per-region.
- **ADR-0010** — Regional pack architecture.
- **ADR-0028** — Cloud microservice architecture.
- **ADR-0049** — Cross-region replication + residency.
- **ADR-0064** — Canonical base + localization packs.
- **ADR-0099** — Data class registry.
- **ADR-0105** — Thirteen-layer canonical enum.
- **ADR-0128** — Hyperscaler architecture invariants.
- **ADR-0131** — Per-microservice flat layout.
- **ADR-0132** — No-grouping forward policy.
- **ADR-0140** — Cedar policy enforcement (retired; superseded
  fragments live in policy-engine).
- **ADR-0144** — EU AI Act graduated-risk tier model.
- **ADR-0145** — Inter-microservice communication reform.
- **ADR-0150** — Cedar policy engine.
- **ADR-0174** — FinOps cost tag + sustainability.
- **ADR-0176** — Brown-out + degradation signal.
- **ADR-0183** — Cedar app authz + Kyverno admission.
- **ADR-0188** — Passkey / WebAuthn as canonical auth.
- **ADR-0211** — In-house Rust-primary tech stack.
- **ADR-0212** — Buildability doctrine.
- **ADR-0218** — Tenant granular control surface.
- **ADR-0240** — Sovereign cloud per regional pack.
- **ADR-0241** — DR + business-continuity portfolio policy.
- **ADR-0242** — `oyatie`-is-a-tenant doctrine (keystone #1).
- **ADR-0243** — Cedar as universal gate (keystone #2).
- **ADR-0244** — Tenant as universal scoping primitive (keystone #3).
- **ADR-0245** — Substrate vs Product layering (keystone #4).
- **ADR-0246** — Policy-engine substrate promotion (keystone #5).
- **ADR-0247** — Self-hosting / self-modification doctrine (keystone #6).
- **ADR-0248** — Amazon-shape cellular architecture (keystone #7).
- **ADR-0249** — DR substrate doctrine (keystone #8).
- **ADR-0250** — Data residency + jurisdiction model (keystone #9).
- **ADR-0252** — Identity + authentication substrate (keystone #11
  — companion).
- **ADR-0253** — Audit chain substrate doctrine (keystone #12 —
  companion).
- **ADR-0254** — Observability substrate doctrine (keystone #13 —
  companion).
- **ADR-0255** — Intelligence substrate rewrite (keystone #14 —
  companion).

### Auto-memory feedback

- `feedback_compliance_pack_first_class` — NEW; captures this
  keystone for future agent context.
- `feedback_oyatie_is_a_tenant_doctrine` — applies; oyatie tenant
  also installs packs (e.g., SOC 2 Type II, ISO 27001, internal
  GDPR for EU contributors).
- `feedback_cedar_as_universal_gate` — applies; packs are Cedar
  fragment bundles.
- `feedback_canonical_base_localization` — applies; packs are the
  unit of localization in the compliance domain.
- `feedback_quality_performance_scalability_bar` — reinforced;
  hyperscaler-grade.
- `feedback_no_silent_regression` — reinforced; pack versioning +
  signature + sunset prevents silent change.
- `feedback_autonomous_implementation_artifacts` — reinforced;
  regulated-tenant onboarding becomes an autonomous workflow.
- `feedback_automate_everything` — reinforced; pack workflows are
  mechanically driven.
- `feedback_doc_coverage_enforced` — reinforced; per-pack doc
  surface is planned to be enforced by `oya-check-compliance-pack-schema`.

---

## Appendix A: Hyperscaler-pattern attribution matrix

Per the audit pattern established in the foundational keystone bundle,
every architectural decision in this ADR is attributed to a named
hyperscaler pattern + source + anti-pattern avoided.

| Decision section | Hyperscaler pattern (named) | Source citation | Anti-pattern avoided |
|---|---|---|---|
| D-1 (Compliance Pack schema) | "Compliance-as-Packaged-Bundle" | AWS Audit Manager framework catalog; Microsoft Purview assessment templates; Google Assured Workloads compliance regimes; Salesforce HealthCloud / GovCloud SKUs; Databricks Compliance Security Profile | "Ad-Hoc Per-Regulation Implementation" — N+1 regulation cost scales superlinearly |
| D-2 (pack lifecycle: author → review → sign → publish → activate → audit → sunset → tombstone) | "Signed Policy Bundle Lifecycle with Transparency Log" | Sigstore Rekor + cosign; AWS Verified Permissions policy store; AWS IAM Policy versioning + history | "Imperative Policy Patching" — pack content changes without provenance |
| D-3 (tenant pack installation: eligibility + KYB/KYC + jurisdiction + DPIA + agreements + cell pinning + Cedar activation + audit + onboarding) | "Tenant-Installs-Compliance-Regime" | Google Assured Workloads regime activation; AWS Control Tower compliance guardrail installation; Microsoft Purview tenant assessment activation | "Implicit Compliance Inheritance" — tenant assumed-compliant without explicit opt-in + verification |
| D-4 (cell certification level matrix) | "Cell-Certification-as-Discrete-Levels" | AWS regions partitioned (Commercial vs GovCloud vs ISO vs China vs Top Secret); Google Cloud regional compliance designations; Azure compliance offerings per region; AWS Outposts compliance binding | "Single-Tier Substrate" — one substrate must satisfy all regulations |
| D-5 (tenant → cell pinning rule) | "Mandatory-Compliance-Pinning" | AWS Organizations + Control Tower account-to-OU pinning; Azure Subscription compliance binding; Google Cloud Project assured-workloads binding | "Drift via Tenant Movement" — tenant migrates to incompatible cell silently |
| D-6 (cross-pack traffic Cedar-gated; FULL matrix) | "Cross-Tenant Policy Gate" | AWS Verified Permissions cross-account evaluation; AWS Resource Access Manager (RAM) cross-account share gating; Azure RBAC cross-tenant guest access; Google IAM cross-project deny | "Implicit Cross-Tenant Trust" — data flows freely across compliance domains |
| D-7 (BAA/DPA agreement lifecycle saga) | "Durable-Workflow-Driven Compliance-Agreement Lifecycle" | AWS Artifact agreement automation; DocuSign + Adobe Sign integration; Stripe onboarding workflow | "Manual-Email-PDF Agreement Lifecycle" — agreements lost in inboxes |
| D-8 (breach notification machinery per-jurisdiction workflow) | "Per-Jurisdiction Breach-Notification Workflow" | AWS GuardDuty + Detective + Security Hub integrated incident response; Microsoft Sentinel + Compliance Manager breach playbook; Atlassian + PagerDuty + ServiceNow incident response | "First-Breach Scramble" — workflow built under deadline pressure with errors |
| D-9 (de-identification engine substrate: tokenization + k-anonymity + l-diversity + t-closeness + differential privacy + HIPAA Safe Harbor + GDPR pseudonymization) | "Shared De-Identification Substrate" | AWS Glue DataBrew PII transforms; Google Cloud DLP API; Microsoft Presidio; Privitar (acquired by Informatica 2023) | "Per-Use-Case De-ID Implementation" — quality varies; HIPAA Safe Harbor implemented incorrectly |
| D-10 (encryption substrate per-data-class + encryption-BYOK + HYOK + FIPS 140-2/3 + HSM-rooted + PQ-hybrid) | "Hierarchical-Key-Management Substrate" | AWS KMS + CloudHSM + per-service-key hierarchy; Google Cloud KMS + Cloud HSM + External Key Manager; Azure Key Vault Managed HSM | "Per-Service KMS" — keys reimplemented per service inconsistently |
| D-11 (consent management substrate per-tenant per-purpose) | "Per-Purpose Consent Substrate" | OneTrust Consent Management; TrustArc Consent Manager; Cookiebot (Usercentrics) | "Boolean Consent Field" — consent collapsed to one column; granularity lost |
| D-12 (per-pack DPIA template) | "Per-Regulation DPIA Template" | ICO DPIA template (UK); CNIL PIA tool (France); HHS HIPAA Risk Analysis tool; EU AI Act FRIA template | "Free-Form DPIA Document" — DPIA inconsistent across deployments |
| D-13 (audit chain per-pack stream with per-jurisdiction retention) | "Per-Stream Audit-Chain with Per-Pack Retention" | AWS CloudTrail Lake per-event-data-store retention; Google Cloud Audit Logs retention buckets; Microsoft Sentinel retention tiers | "Single Audit Stream" — retention overshooting cost; under-shooting compliance |
| D-14 (pack composition semantics: deny wins, retention takes MAX, cross-tenant rules take MOST RESTRICTIVE) | "Compositional Policy Semantics" | AWS SCP + IAM policy intersection (deny wins); GCP Org Policy + Project Policy union; Cedar fragment composition | "Per-Pack Re-Implementation of Composition" — composition rules drift per pack |
| D-15 (certification level inheritance) | "Hierarchical Certification Inheritance" | FedRAMP High ⊇ FedRAMP Moderate ⊇ FedRAMP Low; CMMC Level 3 ⊇ Level 2 ⊇ Level 1; ISO 27001 implies ISO 27002 controls | "Flat Certification Catalog" — certifications listed independently; prerequisites not enforced |
| D-16 (auditor evidence package per-pack per-cadence emission) | "Auto-Emit Auditor Evidence Package" | AWS Audit Manager evidence collection + assessment report; Microsoft Purview Compliance Manager scorecard export; Google Cloud Assured Workloads evidence | "Manual Audit-Evidence Compilation" — quarterly engineering scramble |

---

## Appendix B: Worked example — tenant installs HIPAA + SOC 2 T2 packs

To illustrate the keystone end-to-end, here is a worked example.

**Scenario.** A US-based healthcare-tech tenant, "Acme Healthcare
Inc." (`tenant-acme-healthcare-inc`), incorporated in Delaware,
operating in California + New York + Texas, needs to install:

1. `HIPAA-2024 v1.0.0` (Privacy Rule + Security Rule + Breach
   Notification Rule + HITECH).
2. `SOC2-T2-2024 v1.0.0` (baseline trust service criteria, customer
   confidence + sales evidence).

**Step 0: Pre-conditions.**

- `tenant-acme-healthcare-inc` is registered per ADR-0244 with
  `audience_type: B2B-tenant`, `jurisdiction.primary: US-DE`,
  `data_residency_allowed: ["US"]`.
- `oyatie-compliance-office` has published both packs to the
  registry (signed by Ed25519 + cosign-attested + Sigstore-logged).
- Two cells qualify: `cell-us-east-1-a-hipaa-001` and
  `cell-us-west-2-a-hipaa-001`, both with certifications
  `{general, hipaa-certified}`.

**Step 1: Pack catalog browse.**

Acme's tenant admin (`tenant.acme-healthcare-inc.admin.alice`,
authenticated via passkey/WebAuthn per ADR-0188) browses the pack
catalog in the tenancy admin console:

```
Available Packs (jurisdiction: US):
  [ ] HIPAA-2024 v1.0.0 (regulator: HHS OCR)
  [x] SOC2-T2-2024 v1.0.0 (auditor: AICPA-aligned CPA firm)
  [ ] FERPA-2024 v1.0.0 (regulator: US Dept. of Education)
  [ ] FedRAMP-Moderate-v5 v1.0.0 (regulator: FedRAMP PMO)
  [ ] EU-GDPR-2018-baseline v2024.0.0 (regulator: per-EU-MS supervisory authority)
  [ ] PCI-DSS-L1-v4.0.1 v1.0.0 (assessor: QSA)
  ...
```

Alice selects HIPAA-2024 + SOC2-T2-2024 and clicks "Install."

**Step 2: Eligibility check.**

The pack-install workflow runs:

- HIPAA-2024 eligibility: `audience_type: B2B-tenant` — OK. (Individual
  consumers cannot install HIPAA; healthcare-providing businesses
  can.)
- HIPAA-2024 jurisdiction: `regulation.jurisdiction: [US]`; tenant
  `jurisdiction.primary: US-DE` — OK.
- HIPAA-2024 KYB required (true): tenant submits business
  verification documents (state corporate registration, EIN, healthcare-
  business attestation). `microservices/identity-verification/`
  validates within 1-3 business days.
- HIPAA-2024 cell-eligibility: `minimum_certification_level_set: [general,
  hipaa-certified]`. Tenant's current home_cell?
  - If currently in `cell-us-east-1-a-general-001`
    (certifications `{general}` only), the install workflow rejects
    with: "Cell migration required. Available HIPAA-certified cells:
    cell-us-east-1-a-hipaa-001, cell-us-west-2-a-hipaa-001."
  - Alice selects `cell-us-east-1-a-hipaa-001`. Tenant migration
    workflow runs (data copy + verification + cutover) before pack
    activation.
- SOC2-T2-2024 eligibility: `audience_type: any non-consumer` — OK.
  Cell-eligibility: `minimum_certification_level_set: [general]` —
  satisfied by `cell-us-east-1-a-hipaa-001` (which is also a
  `general` cell per §D-15 inheritance).

**Step 3: DPIA + risk analysis.**

The pack-install workflow opens the DPIA template per HIPAA-2024
v1.0.0:

```markdown
# DPIA — HIPAA-2024 v1.0.0 for tenant-acme-healthcare-inc

## 1. Processing description
   - Purpose: [Alice fills in: "Patient appointment management, EHR
     storage, telehealth"]
   - Lawful basis: HIPAA TPO (Treatment, Payment, Operations)
   - Data subjects: ~50,000 patients
   - Data categories: PHI (Demographics, Medical History, Diagnoses,
     Prescriptions, Provider Notes, Insurance, Billing)
   - Recipients: [list of business associates with BAAs]
   - Retention: 6 years per §164.316(b)(2)(i); longer per state law
   - Cross-border transfers: None planned
...
```

Alice + Acme's DPO complete + sign the DPIA. The signed blob is
stored at `dpia-blob-ref-7821`.

**Step 4: Per-pack agreement signing.**

The agreement-lifecycle saga (per §D-7) runs:

1. **BAA template** (HIPAA-2024 v1.0.0's `agreement_template_refs.baa_template_ref`)
   pulled from the pack registry.
2. State: `DRAFTED`. Workflow loads the template + interpolates
   tenant-specific fields.
3. Transition: `DRAFTED → SUBMITTED_FOR_TENANT_REVIEW`. Alice
   reviews + e-signs via the admin console.
4. State: `TENANT_COUNTERSIGNED`. Workflow validates signature.
5. Transition: `TENANT_COUNTERSIGNED → SUBMITTED_FOR_OYATIE_REVIEW`.
   Oyatie council-legal reviews + e-signs.
6. State: `OYATIE_COUNTERSIGNED → ACTIVE`. BAA blob stored at
   `baa-blob-ref-9132`. Term 3 years; auto-renewal scheduled at
   T-90 days.

Audit-chain emits `CompliancePackAgreementLifecycleTransition` events
at each transition (4 events for this BAA).

**Step 5: Cell pinning + Cedar activation.**

- `tenant-acme-healthcare-inc.home_cell = cell-us-east-1-a-hipaa-001`.
- `tenant-acme-healthcare-inc.dr_cell = cell-us-west-2-a-hipaa-001`
  (selected per ADR-0241 T2 RTO requirements + HIPAA Contingency
  Plan §164.308(a)(7)).
- HIPAA-2024 v1.0.0 Cedar fragments load into the tenant's effective
  policy:
  - `pack/HIPAA-2024/v1.0.0/phi-access-permits.cedar`
  - `pack/HIPAA-2024/v1.0.0/phi-default-deny.cedar`
  - `pack/HIPAA-2024/v1.0.0/baa-coverage-requirement.cedar`
  - `pack/HIPAA-2024/v1.0.0/audit-stream-selection.cedar`
  - `pack/HIPAA-2024/v1.0.0/cross-tenant-phi-default-forbid.cedar`
  - ... etc.
- SOC2-T2-2024 v1.0.0 Cedar fragments similarly load.
- `microservices/cloud-iac/ARCHITECTURE.md#cell-provisioning` confirms
  the cell can host both packs.

**Step 6: Audit emission.**

`CompliancePackInstalled` event emitted on the tenant's audit stream:

```json
{
  "event_class": "CompliancePackInstalled",
  "tenant_id": "tenant-acme-healthcare-inc",
  "pack_id": "HIPAA-2024",
  "pack_version": "1.0.0",
  "pack_signing_attestation_blob_ref": "rekor-12345...",
  "installed_at": "2026-04-15T14:00:00Z",
  "installed_by": "tenant.acme-healthcare-inc.admin.alice",
  "dpia_signed_ref": "dpia-blob-ref-7821",
  "baa_signed_ref": "baa-blob-ref-9132",
  "kyb_verified_at": "2026-04-10T09:30:00Z",
  "home_cell": "cell-us-east-1-a-hipaa-001",
  "dr_cell": "cell-us-west-2-a-hipaa-001",
  "evidence_id": "audit-event-7e1f3a"
}
```

A second event for `SOC2-T2-2024` installation. Both events Merkle-
sealed into the hourly period.

**Step 7: Onboarding workflow.**

HIPAA-2024's pack-specific onboarding workflow runs:

- HIPAA Workforce Training acknowledgement (oyatie-side: all `oyatie.*`
  principals with PHI access have current annual training).
- HIPAA Security Officer designation (Acme designates).
- HIPAA Privacy Officer designation (Acme designates).
- Annual Risk Analysis schedule established (per §164.308(a)(1)(ii)(A)).
- Quarterly DR drill schedule established (per §164.308(a)(7) +
  ADR-0241 T2 cadence).
- Annual SOC 2 audit schedule (for SOC2-T2-2024) established.

**Step 8: Cedar evaluation walkthrough — a PHI write action.**

Alice's clinician colleague `tenant.acme-healthcare-inc.user.bob`
(authenticated via passkey, role `clinician`) attempts to write a
patient note for patient `patient-12345`:

Request:

```
EvaluationRequest {
  principal: tenant.acme-healthcare-inc.user.bob,
  action: PatientRecord::Action::WriteNote,
  resource: PatientRecord::id/patient-12345,
  context: {
    data_class: ["PHI"],
    purpose: "treatment",
    consent: { ... },
    request_origin_cell: "cell-us-east-1-a-hipaa-001",
    home_cell_of_subject: "cell-us-east-1-a-hipaa-001"
  },
  tenant_id: "tenant-acme-healthcare-inc",
  evaluation_id: <uuid>
}
```

Aggregated effective policy:
- baseline fragments
- overlay/us-de/* fragments
- pack/HIPAA-2024/v1.0.0/* fragments
- pack/SOC2-T2-2024/v1.0.0/* fragments
- tenant/tenant-acme-healthcare-inc/* fragments (if any)

Per HIPAA-2024's `phi-access-permits.cedar`:

```cedar
permit (
  principal,
  action == PatientRecord::Action::WriteNote,
  resource is PatientRecord
)
when {
  principal in Role::"clinician"
  && principal.tenant_id == resource.tenant_id  // no cross-tenant PHI by default
  && context.purpose in ["treatment", "payment", "operations"]
  && context.data_class.contains("PHI")
  && context.request_origin_cell.cell_certifications.contains("hipaa-certified")
  && principal.has_current_hipaa_workforce_training
};
```

Per SOC2-T2-2024 + baseline: complementary permit fragments. Per
each pack's default-deny: no other permit catches.

Decision: `Permit`. Applied fragments:
`[baseline/general-permits.cedar:v3,
overlay/us-de/baseline-overlay.cedar:v1,
pack/HIPAA-2024/v1.0.0/phi-access-permits.cedar:v1,
pack/SOC2-T2-2024/v1.0.0/general-controls.cedar:v1]`.

Audit chain emits `CedarEvaluation` (per ADR-0243 §D-7) AND a
HIPAA-specific `PhiAccessAuditEvent` (per HIPAA-2024's
`audit_chain_requirements.required_event_classes`) AND a SOC 2 control
evidence event. The PhiAccessAuditEvent goes into the HIPAA-pack
stream with 6-year retention; the SOC 2 event goes into the SOC2-pack
stream with 1-year + 90-day-cold retention.

**Step 9: Cross-pack-deny walkthrough — a cross-tenant PHI export
attempt.**

Bob attempts to share `patient-12345` records with
`tenant-acme-marketing-llc` (a marketing affiliate, has installed
only `SOC2-T2-2024` — NOT HIPAA-2024):

Request:

```
EvaluationRequest {
  principal: tenant.acme-healthcare-inc.user.bob,
  action: PatientRecord::Action::ShareCrossTenant,
  resource: PatientRecord::id/patient-12345,
  context: {
    data_class: ["PHI"],
    target_tenant: "tenant-acme-marketing-llc",
    target_tenant_packs: ["SOC2-T2-2024"]
  },
  ...
}
```

Per §D-6 cross-pack matrix: caller pack profile `{HIPAA-2024, SOC2-T2-2024}`;
callee pack profile `{SOC2-T2-2024}`. Profile relationship: "Regulated
↔ General" → FORBIDDEN by default (PHI cannot leak to a general tenant).

Per HIPAA-2024's `cross-tenant-phi-default-forbid.cedar`:

```cedar
forbid (
  principal,
  action == PatientRecord::Action::ShareCrossTenant,
  resource is PatientRecord
)
when {
  context.data_class.contains("PHI")
  && !context.target_tenant_packs.contains("HIPAA-2024")
};
```

Decision: `Forbid { reason: "PHI cannot egress to a tenant lacking
HIPAA-2024 pack. Target tenant tenant-acme-marketing-llc has packs
[SOC2-T2-2024]. To enable, the target tenant must install HIPAA-2024
plus a Business Associate Agreement must be in force between Acme
Healthcare and Acme Marketing." }`.

Audit chain emits `RegulatedToGeneralCallForbidden` AND a HIPAA-
specific `PhiExportAttemptDenied` event.

The user receives the human-readable reason; counsel review may
follow. The deny is permanent until the target tenant onboards
HIPAA + signs a BAA.

**Step 10: Quarterly regulator evidence emission.**

At 2026-06-30T23:59Z, the per-pack regulator-evidence-emission
workflow runs for HIPAA-2024:

1. Workflow enumerates Acme's audit-stream entries for Q2 2026.
2. Generates per-control evidence per HHS OCR Audit Protocol 2024.
3. Computes operational counts (PHI accesses, denials, breaches).
4. Signs the package with `oyatie-compliance-office` Ed25519 key.
5. Emits `ComplianceEvidence` event for HIPAA-2024-Q2-2026 with
   the package blob reference.
6. HHS OCR audit pull (if any) authenticated via Cedar + presents
   the package.

Similar for SOC2-T2-2024 — annual emission cadence.

**Why this works:** the entire flow above is mechanically driven.
The pack primitive carries the regulatory specifics; the tenant
+ cell + Cedar + audit substrate carries the platform-uniform
machinery. Onboarding a HIPAA tenant requires no bespoke engineering;
it's the same workflow for any tenant installing any pack. The
cross-pack matrix prevents PHI leakage by construction.

Under the prior ad-hoc model, the equivalent install would have
required:

- Per-µservice code changes to tag PHI paths.
- Hand-authored Cedar fragments (no shared HIPAA pack to reuse).
- One-off BAA template + manual email-PDF agreement workflow.
- One-off DPIA Word document.
- One-off breach-notification workflow (built during the first
  breach, under deadline pressure).
- One-off audit-stream class.
- One-off regulator-evidence packet, compiled manually each quarter.

The compliance pack primitive closes that variance by construction.

---

## Appendix C: Per-pack Cedar fragment naming convention

Per ADR-0243 §D-2 + ADR-0099 + BNF v4.1 naming conventions:

```
microservices/policy-engine/fragments/pack/<pack-id>/v<version>/<action-family>-<purpose>.cedar
```

Where:
- `<pack-id>` is the canonical pack ID (e.g., `HIPAA-2024`, `PCI-DSS-L1-v4.0.1`).
- `<version>` is the pack version.
- `<action-family>` is the action category (e.g., `phi-access`, `chd-access`,
  `consent`, `dsar`, `audit-emission`, `cross-tenant`, `cell-pinning`).
- `<purpose>` is the fragment's specific purpose (e.g., `permits`,
  `default-deny`, `forbid`, `overlay-<jurisdiction>`).

Examples:
- `pack/HIPAA-2024/v1.0.0/phi-access-permits.cedar`
- `pack/HIPAA-2024/v1.0.0/phi-access-default-deny.cedar`
- `pack/HIPAA-2024/v1.0.0/cross-tenant-phi-default-forbid.cedar`
- `pack/PCI-DSS-L1-v4.0.1/v1.0.0/chd-access-permits.cedar`
- `pack/EU-GDPR-2018-baseline/v2024.0.0/dsar-cascade-permits.cedar`
- `pack/EU-GDPR-2018-baseline/v2024.0.0/overlay-de/lkda-permits.cedar`
  (Germany Landesdatenschutzbeauftragte overlay)
- `pack/EU-AI-ACT-2024/v1.0.0/high-risk-tier-obligations.cedar`
- `pack/KR-PIPA-2023-amendment/v1.0.0/article-22-consent-permits.cedar`

Fragment IDs are unique within the namespace; A1 (own-policy-
adherence-naming) facet enforces the convention.

---

## Appendix D: Pack publication ceremony — runbook outline

The pack-publication ceremony for a new pack version:

1. **T-30 days:** Pack draft authored by pack-author team.
2. **T-21 days:** Multispectrum review v2.4.0 fan-out begins (F1 +
   F5 + F6 + F7 + F11 + A1 + A2 + A4 + A6 facets).
3. **T-14 days:** External legal counsel review (where required by
   pack scope, e.g., FedRAMP / NIS2 / state-law packs).
4. **T-7 days:** DPO + CISO + council-legal sign-off; pack hash
   computed.
5. **T-3 days:** HSM ceremony at oyatie-compliance-office (2-of-3
   key holders + DPO witness):
   - Verify pack hash matches latest reviewed version.
   - Sign with oyatie-compliance-office Ed25519 key.
   - Submit to Sigstore Rekor; receive transparency-log entry.
   - Generate cosign attestation.
6. **T-0:** Pack published to registry; `effective_at` set; tenants
   notified if affected.
7. **T+30 days:** Pack-installation rollout monitoring; pack-author
   team on-call for tenant questions.
8. **T+90 days:** Pack stability review; minor-version increments
   permitted for non-normative fixes; major-version on next
   regulatory amendment.

Sunset ceremony (when a prior version sunsets):

1. **T-180 days:** Sunset announced; affected tenants notified to
   plan migration.
2. **T-90 days:** Migration workflow available; tenants opt-in to
   migrate.
3. **T-30 days:** Migration mandatory; tenants on the old version
   receive escalating reminders.
4. **T-7 days:** Final reminder; cell-admission gate begins
   refusing the old version on new actions.
5. **T-0:** Old version sunsets; tenants on the old version are
   quarantined (Cedar forbids actions) until migration completes.
6. **T+30 days:** Tombstone scheduled per archive retention.

---

## Appendix E: Mapping each pack to its required substrate µservices

The substrates each pack relies on:

| Pack | Required substrate µservices |
|---|---|
| SOC2-T2-2024 | audit-chain, observability, policy-engine, identity, tenancy |
| ISO-27001-2022 | audit-chain, policy-engine, identity, observability, encryption-substrate |
| ISO-22301-2019 | observability, cloud-iac, dr (per ADR-0249) |
| ISO-42001-2023 | observability, audit-chain, intelligence (per ADR-0255) |
| HIPAA-2024 | audit-chain, breach-notification, consent, de-identification, encryption-substrate, identity-verification |
| PCI-DSS-L1-v4.0.1 | audit-chain, breach-notification, encryption-substrate (FIPS 140-3), de-identification (tokenization), identity-verification (KYC) |
| FedRAMP-Moderate-v5 | audit-chain, observability, encryption-substrate (FIPS 140-2 L2), identity, policy-engine, cloud-iac (AWS GovCloud) |
| FedRAMP-High-v5 | + encryption-substrate (FIPS 140-2 L3), + air-gap-capable substrate |
| DoD-IL5-SRGv1r5 | + encryption-substrate (FIPS 140-2 L3 + NSA Type 1), + classified-adjacent substrate |
| DoD-IL6-SRGv1r5 | + SECRET-classified substrate, + cleared-personnel-management |
| EU-GDPR-2018-baseline | consent, breach-notification, audit-chain, de-identification, identity-verification (KYB) |
| EU-NIS2-2022 | breach-notification (24h regulator), observability (SIEM-grade) |
| EU-DSA-2022 | content-moderation (planned in upcoming ADR), audit-chain (transparency reports) |
| EU-AI-ACT-2024 | intelligence (per ADR-0255), audit-chain, consent (per Article 50/52), policy-engine (tier evaluation) |
| KR-PIPA-2023-amendment | consent (per Article 22), breach-notification (24h regulator), de-identification |
| KR-FSS-2024 | identity-verification (PG license-holder), audit-chain (5y transaction retention) |
| JP-APPI-2022-amendment | consent, breach-notification, audit-chain |
| SG-PDPA-2020 | consent, breach-notification |
| AU-PRIVACY-1988-2024-reforms | consent, breach-notification (30d) |
| KSA-PDPL-2023 | consent, audit-chain, encryption-substrate |
| FERPA-2024 | consent (parental), audit-chain |
| FDA-21CFR-PART11-2024 | audit-chain (immutable), identity (e-signature), encryption-substrate |
| SOX-404-2024 | audit-chain (7y retention), identity (segregation of duties) |

Substrates are reused across packs — engineering investment compounds.

---

## Naming justification

Every name introduced or ratified by this ADR is validated against BNF v4.1
(`oya-<microservice>[-<bc-tokens>]-<layer>`) and the ADR-0105 13-value canonical
layer enum.

| Name | Layer (ADR-0105) | BNF v4.1 segments | Justification |
|------|-----------------|-------------------|---------------|
| `oya-shared-de-identification-client` | `sdk` (client = SDK layer) | `oya` · `shared` · `de-identification` · `client` | Shared client SDK for de-identification substrate per §D-9; `client` maps to `sdk` layer |
| `oya-shared-consent-client` | `sdk` (client = SDK layer) | `oya` · `shared` · `consent` · `client` | Shared client SDK for consent management substrate per §D-11; `client` maps to `sdk` layer |
| `oya-shared-breach-notification-client` | `sdk` (client = SDK layer) | `oya` · `shared` · `breach-notification` · `client` | Shared client SDK for breach-notification machinery per §D-8; `client` maps to `sdk` layer |
| `oya-check-compliance-pack-schema` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `compliance-pack-schema` | Fitness-check; verifies pack content schema per §D-1; `oya-check-*` flat namespace |
| `oya-check-compliance-pack-signature` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `compliance-pack-signature` | Fitness-check; verifies Ed25519 cosign signature on every published pack; `oya-check-*` flat namespace |
| `oya-check-cell-certification-coherence` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `cell-certification-coherence` | Fitness-check; verifies cell certification matrix per §D-4; `oya-check-*` flat namespace |
| `oya-check-tenant-pack-cell-pinning` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `tenant-pack-cell-pinning` | Fitness-check; verifies tenant + pack → cell pinning rule per §D-5; `oya-check-*` flat namespace |
| `oya-check-cross-pack-traffic-cedar-gated` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `cross-pack-traffic-cedar-gated` | Fitness-check; verifies every cross-pack traffic path has Cedar permit per §D-6; `oya-check-*` flat namespace |
| `oya-check-baa-dpa-coverage` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `baa-dpa-coverage` | Fitness-check; verifies every HIPAA/GDPR pack has BAA/DPA infrastructure per §D-7; `oya-check-*` flat namespace |
| `oya-check-breach-notification-workflow-coverage` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `breach-notification-workflow-coverage` | Fitness-check; verifies breach-notification workflow exists for every applicable pack per §D-8; `oya-check-*` flat namespace |
| `oya-check-consent-record-coverage` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `consent-record-coverage` | Fitness-check; verifies consent records present for every pack with consent obligations per §D-11; `oya-check-*` flat namespace |
| `oya-check-dpia-template-coverage` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `dpia-template-coverage` | Fitness-check; verifies DPIA template populated for every installed pack per §D-12; `oya-check-*` flat namespace |
| `oya-check-auditor-evidence-emission` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `auditor-evidence-emission` | Fitness-check; verifies per-pack auditor evidence emission per §D-16; `oya-check-*` flat namespace |

---

*End of ADR-0251.*
