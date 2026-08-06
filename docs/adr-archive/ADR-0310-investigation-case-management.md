---
id: ADR-0310
status: Superseded
date: 2026-05-20
owners:
  - council-architecture
  - council-security
  - council-privacy
  - council-product
  - council-legal
  - council-data
  - council-trust-and-safety
  - council-investigation
  - ops-sre-reliability
  - ops-security
  - ops-trust-and-safety
  - ops-compliance
  - ops-investigation
  - axis-detection
  - axis-investigation
  - axis-case-management
  - axis-audit-chain
  - axis-ops-dashboard
supersedes: []
amends: []
superseded_by: [ADR-703]
related:
  - ADR-0028-audit-chain-merkle-sealed.md
  - ADR-0099-data-class-registry.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0132-no-grouping-microservice-rule.md
  - ADR-0140-cedar-policy-enforcement.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0250-build-ahead-of-certification-doctrine.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0258-api-versioning-semver-policy.md
  - ADR-0263-observability-emission-contract.md
  - ADR-0276-backup-portability-gdpr-art-20.md
  - ADR-0293-governance-meta-trust-root.md
  - ADR-0294-cedar-fragment-soak-anomaly-rollback.md
  - ADR-0295-bootstrap-ci-spiffe-kill-switch.md
  - ADR-0296-library-first-credential-sidecar.md
  - ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md
  - ADR-0298-emergency-services-critical-path-exemption.md
  - ADR-0307-detection-substrate-streaming-batch.md
  - ADR-0308-ml-model-lifecycle-ai-act-compliance.md
  - ADR-0309-detection-fairness-audit-civil-rights.md
related_specs:
  - /specs/microservices/detection.json
  - /specs/microservices/ops-dashboard-control-center.json
  - /specs/investigation-case-schema.json
  - /specs/investigation-case-lifecycle-schema.json
  - /specs/investigation-evidence-schema.json
  - /specs/investigation-chain-of-custody-schema.json
  - /specs/investigation-regulator-surface-schema.json
related_memory:
  - feedback_quality_performance_scalability_bar
  - feedback_clean_architecture_requirements
  - feedback_no_silent_regression
  - feedback_autonomous_implementation_artifacts
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_cedar_as_universal_gate
  - feedback_compliance_pack_primitive
  - feedback_naming_justification
  - feedback_substrate_vs_product_layering
  - feedback_build_ahead_of_certification
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-foundational-doctrine
keystone_position: drmp-investigation-case-management
purpose: >
  Establish the Investigation Case-Management substrate as a
  cross-cutting workflow primitive for the Detection Substrate (per
  ADR-0307) + ML Model Lifecycle (per ADR-0308) + Fairness Audit
  (per ADR-0309) + Compliance Pack overlay (per ADR-0251). Codifies
  the canonical case-management workflow from documentation-rigor.md
  §3.2.6 substrate-primitive #7:
    detection signal → triage → investigation → escalation/dismissal
                    → feedback → model retrain
  Integrates with Cedar (per ADR-0243; Cedar-gated PII access per
  case); cross-references microservices/ops-dashboard-control-center/
  panels for analyst UI; chain-of-custody preserved per ADR-0028
  audit-chain Merkle-sealed doctrine; per-pack regulator-facing
  surface (e.g., FedRAMP continuous-monitoring, EU AI Act Art. 73
  serious-incident reporting, DSA Art. 17 statement-of-reasons, NY
  AEDT Local Law 144 (2023) public bias audit, GDPR Art. 22 +
  Art. 86 appeal adjudication); per-tenant investigation case
  ownership with cross-tenant-isolation Cedar gating. Without this
  substrate, detection signals route nowhere, appeals route nowhere,
  regulator-facing surfaces are ad-hoc, and chain-of-custody is
  unenforceable — incompatible with HIPAA 60d breach-notification,
  GDPR 72h breach-notification, NIS2 24h/72h/1mo cadence, KR-PIPA
  72h, EU AI Act Art. 73 24h, DSA Art. 17, NY DFS 23-NYCRR-500 72h,
  18 USC §2258A NCMEC reporting, FinCEN SAR thresholds, OFAC
  sanctions reporting, FBI IC3 reporting.
enforcement_status: advisory-until-2026-10-15-blocker-thereafter
enforced_by:
  - cloud-ci/Rust gate packet investigation-case-substrate-emission-coherence
  - cloud-ci/Rust gate packet investigation-cedar-pii-gating-coherence
  - cloud-ci/Rust gate packet investigation-chain-of-custody-merkle-sealed
  - cloud-ci/Rust gate packet investigation-per-tenant-case-ownership
  - cloud-ci/Rust gate packet investigation-per-pack-regulator-surface
  - cloud-ci/Rust gate packet investigation-appeal-mechanism-sla
  - cloud-ci/Rust gate packet investigation-analyst-label-feedback-loop
  - cloud-ci/Rust gate packet investigation-triage-priority-cadence
  - cloud-ci/Rust gate packet investigation-escalation-routing-coherence
naming_justifications:
  - name: oya-shared-investigation-case
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.investigation-case
    justification: >
      Per ADR-0105 13-layer canonical enum row 5 (shared-substrate),
      the crate that exposes the canonical case lifecycle (triage →
      investigation → escalation/dismissal → feedback) across every
      detection-emitting + adverse-action-driving µservice belongs
      at the shared layer. Single-concern per ADR-0131 + ADR-0132;
      not bundled under "trust-safety-suite" name.
  - name: oya-shared-investigation-triage
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.investigation-triage
    justification: >
      Per-signal triage scoring + per-priority queue routing. Single-
      concern; separate from the case crate so triage SLA can evolve
      independently (e.g., per-pack regulator floor tightening).
  - name: oya-shared-investigation-evidence
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.investigation-evidence
    justification: >
      Per-case evidence collection + chain-of-custody preservation
      per ADR-0028 audit-chain Merkle-sealed doctrine. Single-concern;
      evidence handling has a specific cryptographic shape that
      benefits from its own crate boundary.
  - name: oya-shared-investigation-chain-of-custody
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.investigation-chain-of-custody
    justification: >
      Per-evidence chain-of-custody tracking; every access logged;
      Merkle-sealed audit chain (per ADR-0028); legal-grade
      preservation for court-admissible evidence. Single-concern.
  - name: oya-shared-investigation-regulator-surface
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.investigation-regulator-surface
    justification: >
      Per-pack regulator-facing surface emission (HIPAA, GDPR, NIS2,
      DSA Art. 17, EU AI Act Art. 73, NCMEC CyberTipline, FinCEN
      SAR, OFAC). Single-concern; per-regulator wire-format adapter.
  - name: oya-shared-investigation-appeal-adjudication
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.investigation-appeal-adjudication
    justification: >
      Per GDPR Art. 22 + EU AI Act Art. 86 + ECOA Reg B + NY AEDT
      2023 appeal adjudication workflow; routes appeals to human
      reviewers with per-pack SLA. Single-concern; companion to
      oya-shared-ml-appeal-mechanism per ADR-0308.
  - name: oya-shared-investigation-analyst-label-feedback
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.investigation-analyst-label-feedback
    justification: >
      Per-case analyst label routes back to detection substrate's
      feature store for model retraining per ADR-0307 §D-7 + ADR-0308
      drift-triggered retraining. Single-concern; bidirectional bridge.
  - name: oya-governance-investigation-case-substrate-emission
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.investigation-case-substrate-emission
    justification: >
      CI fitness lane per ADR-0212; verifies every detection-emitting
      µservice routes signals into the case substrate.
  - name: oya-governance-investigation-cedar-pii-gating
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.investigation-cedar-pii-gating
    justification: >
      CI fitness lane per ADR-0212; verifies per-case PII access is
      Cedar-gated (per ADR-0243).
  - name: oya-governance-investigation-chain-of-custody-merkle
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.investigation-chain-of-custody-merkle
    justification: >
      CI fitness lane per ADR-0212; verifies chain-of-custody is
      Merkle-sealed per ADR-0028.
  - name: oya-governance-investigation-appeal-sla
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.investigation-appeal-sla
    justification: >
      CI fitness lane per ADR-0212; verifies per-pack appeal SLA
      met (GDPR 1mo, ECOA 30d, KR-FSC 60d, NY AEDT 10 business days
      candidate notice).
  - name: InvestigationCaseOpened
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Investigation.CaseOpened
    justification: >
      Audit-event-class emitted when a case opens (detection signal
      arrived OR appeal filed OR regulator inquiry received).
      Registered per ADR-0263.
  - name: InvestigationCaseTriagePriorityAssigned
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Investigation.CaseTriagePriorityAssigned
    justification: >
      Emitted when triage scorer assigns priority (P0..P4).
      Registered per ADR-0263.
  - name: InvestigationCaseInvestigatorAssigned
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Investigation.CaseInvestigatorAssigned
    justification: >
      Emitted when human investigator is assigned (per per-pack
      expertise + per-tenant Cedar gate).
  - name: InvestigationEvidenceAdded
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Investigation.EvidenceAdded
    justification: >
      Emitted per evidence-item add; carries chain-of-custody
      Merkle root.
  - name: InvestigationPIIAccessed
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Investigation.PIIAccessed
    justification: >
      Emitted per PII access; Cedar-permit-required (per ADR-0243).
      Records investigator + case + PII data-class + reason-of-access.
  - name: InvestigationCaseEscalated
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Investigation.CaseEscalated
    justification: >
      Emitted on escalation (per-pack regulator-facing surface +
      ombudsman + law-enforcement).
  - name: InvestigationCaseDismissed
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Investigation.CaseDismissed
    justification: >
      Emitted on dismissal; carries dismissal-reason for analyst
      label feedback.
  - name: InvestigationCaseAdjudicated
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Investigation.CaseAdjudicated
    justification: >
      Emitted on appeal adjudication; verdict + reasoning routed to
      affected party.
  - name: InvestigationLabelFedBack
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Investigation.LabelFedBack
    justification: >
      Emitted when analyst label routed back to detection substrate's
      feature store for model retraining.
  - name: InvestigationRegulatorEmissionDelivered
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Investigation.RegulatorEmissionDelivered
    justification: >
      Emitted on per-pack regulator-facing emission (HIPAA breach,
      GDPR 72h, NIS2, DSA Art. 17, EU AI Act Art. 73, NCMEC, FinCEN,
      OFAC).
  - name: investigation-case-schema.json
    layer: N/A (JSON Schema spec)
    bnf_segments: spec.investigation-case-schema
    justification: >
      JSON Schema declaring the case shape (id, tenant_id, priority,
      lifecycle_state, evidence[], investigator_id, regulator_emissions[],
      appeal_id, model_id, signal_id). Per the §3.2.2 consistency
      invariant.
  - name: investigation-chain-of-custody-schema.json
    layer: N/A (JSON Schema spec)
    bnf_segments: spec.investigation-chain-of-custody-schema
    justification: >
      JSON Schema declaring per-evidence chain-of-custody shape;
      Merkle-sealed (per ADR-0028).
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0310: Investigation Case-Management Substrate

## Status

Proposed — 2026-05-20.

Bundled with the keystone-bundle 2026-05-20 foundational doctrine
synthesis as the **drmp-investigation-case-management** keystone,
closing the gap identified in `docs/standards/documentation-rigor.md`
§3.2.6 substrate-primitive #7 (investigation case-management;
binding ADR called out as ADR-0310). This ADR is the binding ADR
row 52 of the §3.2.1 ADR-adherence matrix cites.

Enforcement is `advisory-until-2026-10-15-blocker-thereafter`. The
case-management workflow accepts in text immediately; the CI lanes
that gate per-µservice substrate emission + Cedar PII gating +
chain-of-custody Merkle-sealing + per-pack appeal SLA promote to
BLOCKER on 2026-10-15 to give the human-reviewer recruitment + SLA
tooling rollout per §F a month after ADR-0307/0308/0309 GA.

## Date

2026-05-20.

## Context

### §A. Why investigation case-management is a substrate primitive, not a per-µservice afterthought

Mature trust-and-safety + fraud-investigation + regulator-response
organizations treat case-management as a *first-class substrate
primitive* — wired centrally so every detection signal, every
appeal, every regulator inquiry, every escalation routes through
a unified workflow with consistent chain-of-custody, consistent
PII gating, and consistent per-pack regulator surface. The pattern
is unambiguous across the named industry references:

- **Splunk Enterprise Security + Splunk SOAR.** Splunk's security
  operations platform (splunk.com/en_us/products/enterprise-security)
  + SOAR (Security Orchestration, Automation, and Response) — per-
  signal case automation with built-in chain-of-custody + Cedar-
  class permit engine + per-pack regulator-template emission.
  ~PB/day telemetry ingest at hyperscaler scale.
- **IBM QRadar SIEM + IBM SOAR (formerly Resilient).** Industry
  reference for case-management workflow; per-case evidence
  collection, per-investigator role-based access, per-jurisdiction
  regulator playbooks (HIPAA, GDPR, NIS2 templates ship out-of-box).
- **Palo Alto Cortex XSOAR (formerly Demisto).** Per the 2024
  Cortex XSOAR product disclosure, ~5,000 enterprise customers
  use the case-management substrate; per-pack playbooks (HIPAA-
  breach, GDPR-72h-notification, CSAM-reporting) are first-class
  primitives.
- **Google Chronicle SOAR (formerly Siemplify).** Per the 2024
  Chronicle product overview, ~10⁹ events/day ingest with case
  routing + analyst-label feedback → ML retraining loop.
- **Salesforce Service Cloud + Salesforce Trust & Safety.** Per
  Salesforce's published architecture, Service Cloud underpins
  the trust-and-safety case-management for most major SaaS
  platforms (Twitter/X pre-2022, Slack, GitHub, etc).
- **YouTube / Meta / TikTok trust-and-safety case-management.** Per
  their published transparency reports, each ships per-region
  human-reviewer pools (~10k-20k per platform) routing through
  centralized case-management infrastructure; per-pack regulator
  emission (DSA Art. 17, NCMEC CyberTipline, GIFCT).
- **Stripe Trust & Safety investigation flow.** Per Stripe's
  published architecture (stripe.com/docs/disputes + 2024 Stripe
  Sessions keynote), per-dispute case-management with chain-of-
  custody + per-jurisdiction regulator workflow (KR-FSS chargeback,
  EU PSD2 SCA exceptions, US Reg E disputes).
- **Toss riskOps case-management.** Per the 2024 Toss Tech
  Conference keynote, case-management is a centralized substrate
  consumed by Toss Pay + Toss Bank + Toss Securities + Toss
  Insurance — single-substrate shape matching Splunk + Stripe.
- **AWS Security Hub + Amazon Detective + AWS Audit Manager.** AWS's
  case-management primitive ships chain-of-custody + per-pack
  regulator emission (FedRAMP continuous monitoring, HIPAA,
  PCI-DSS, ISO 27001).

The corollary: **every detection signal, every appeal, every
adverse-action notice, every regulator inquiry in oyatie MUST route
through the substrate case-management, not per-µservice ad-hoc
queues.** A µservice authoring its own case workflow drifts from
the chain-of-custody invariant (per ADR-0028 audit-chain Merkle-
sealed), drifts from the Cedar PII gate invariant (per ADR-0243),
drifts from the per-pack regulator emission cadence (per ADR-0251).
The substrate shape closes the gap.

### §A.1. The case lifecycle — six canonical phases

Per documentation-rigor.md §3.2.6 substrate-primitive #7:

```
            ┌─────────────────────────────────┐
            │ 1. SIGNAL / APPEAL / INQUIRY   │
            │  - detection signal arrives    │
            │  - user appeal filed           │
            │  - regulator inquiry received  │
            │  - ombudsman referral          │
            └────────────────┬────────────────┘
                             │
                             ▼
            ┌─────────────────────────────────┐
            │ 2. TRIAGE                       │
            │  - composite-scorer priority    │
            │  - per-pack regulator floor     │
            │  - emergency-services exempt    │
            │  - DV-survivor-shelter mode     │
            └────────────────┬────────────────┘
                             │
                             ▼
            ┌─────────────────────────────────┐
            │ 3. INVESTIGATION                │
            │  - investigator assigned        │
            │    (per pack expertise + Cedar) │
            │  - evidence collected           │
            │  - chain-of-custody Merkle-     │
            │    sealed                       │
            │  - PII access Cedar-gated       │
            └────────────────┬────────────────┘
                             │
                ┌────────────┼────────────┐
                ▼            ▼            ▼
       ┌──────────┐ ┌──────────────┐ ┌──────────────┐
       │ 4A. ESCAL│ │ 4B. DISMISS  │ │ 4C. ADJUDIC. │
       │ - per-   │ │ - reasoning  │ │ - per appeal │
       │   pack   │ │ - feedback   │ │ - per-pack   │
       │   reg.   │ │   to FS      │ │   SLA met    │
       │ - law    │ │              │ │ - reverse /  │
       │   enf.   │ │              │ │   uphold /   │
       │ - ombud. │ │              │ │   partial    │
       └────┬─────┘ └──────┬───────┘ └──────┬───────┘
            │              │                │
            └──────────────┼────────────────┘
                           │
                           ▼
            ┌─────────────────────────────────┐
            │ 5. FEEDBACK                     │
            │  - analyst label → feature      │
            │    store (per ADR-0307 §D-7)    │
            │  - per-model retraining trigger │
            │    (per ADR-0308 §B.4 drift)    │
            └────────────────┬────────────────┘
                             │
                             ▼
            ┌─────────────────────────────────┐
            │ 6. RETENTION + AUDIT TRAIL      │
            │  - per-pack retention cadence   │
            │  - Merkle-sealed (ADR-0028)     │
            │  - regulator-facing emission    │
            │    (per ADR-0251)               │
            └─────────────────────────────────┘
```

### §A.2. Why the keystone bundle 2026-05-20 requires this as a substrate µservice integration

The keystone bundle's foundational ADRs intersect case-management
as follows:

- **ADR-0028 (audit-chain Merkle-sealed).** Every case action +
  every evidence-add + every PII-access emits a Merkle-sealed audit
  row; chain-of-custody invariant satisfied by ADR-0028's
  cryptographic seal.
- **ADR-0099 (data-class registry).** Per-evidence data-class tag
  (PII / PSEUDONYMOUS / AGGREGATE / DERIVED / NON-PII) determines
  Cedar-gate strictness on access.
- **ADR-0242 (oyatie-is-a-tenant).** Oyatie internal investigations
  (e.g., insider-risk cases on oyatie employees) go through the
  same substrate as tenant-tenant investigations; no carve-out.
- **ADR-0243 (Cedar universal gate).** Every per-case PII access
  is Cedar-evaluated; no bypass paths. Per-case PII surface
  enumerated in `policy/investigation-pii-access.cedar`.
- **ADR-0244 (tenant scoping).** Per-case `tenant_id` enforced;
  cross-tenant case access forbidden by Cedar default-deny.
- **ADR-0245 (substrate vs product).** Case-management is a
  substrate µservice consumed by every detection-emitting product;
  the analyst UI lives in `microservices/ops-dashboard-control-center/`
  (a separate µservice; case-management substrate emits to
  ops-dashboard panels).
- **ADR-0248 (cellular architecture).** Tier-2 control-plane cells
  host case-management runtime; Tier-3 data-plane cells host
  evidence storage (per-pack residency honored).
- **ADR-0251 (compliance packs).** Per-pack regulator-facing
  emission cadence (HIPAA 60d, GDPR 72h, NIS2 24h/72h/1mo,
  KR-PIPA 72h, EU AI Act Art. 73 24h, DSA Art. 17, NY DFS 72h)
  surface as per-pack workflow templates.
- **ADR-0263 (observability emission contract).** Every case
  lifecycle transition emits an audit-event-class (10 new classes
  registered per §B.7).
- **ADR-0297 (abuse-defence baseline).** Detection signals from
  abuse-defence (anti-bot + anti-spoof + anti-scrape) route into
  case-management for human-review queue.
- **ADR-0298 (emergency-services critical-path exemption).**
  Emergency-services traffic never blocks but always audited;
  audit-and-investigate routes through case-management.
- **ADR-0307 (detection substrate).** Detection signals route to
  case-management; analyst labels route back per the feedback
  bridge.
- **ADR-0308 (ML model lifecycle).** Appeal mechanism routes per-
  ADR-0308 §B.8 appeals into case-management's adjudication
  workflow.
- **ADR-0309 (fairness audit).** Adverse-action notices emitted per
  ADR-0309 invariant 4 include an appeal-link routing to case-
  management.

### §A.3. The chain-of-custody invariant — court-admissible evidence

Per ADR-0028 audit-chain Merkle-sealed doctrine:

- Every evidence-add to a case produces a per-evidence Merkle leaf
  containing: evidence-content-hash, investigator-id (signer),
  timestamp, case-id, evidence-data-class
- Per-case Merkle root rolls up all evidence + lifecycle transitions
- Per-day platform-wide Merkle root publishes to public-witness
  (per ADR-0028 §D-3 Bitcoin / Sigstore Rekor / Certificate
  Transparency log)
- Chain-of-custody verifiable by anyone with case + evidence pair
  + Merkle proof
- Court-admissible per FRE Rule 901 (authentication of evidence)
  + EU Council Regulation 910/2014 (eIDAS qualified electronic
  signatures)

### §A.4. Per-pack regulator-facing surface roster

Per ADR-0251 compliance-pack overlay shape, case-management emits
to:

| Regulator | Trigger | Cadence | Wire format |
|---|---|---|---|
| **HIPAA Privacy Rule** | PHI breach detection | 60 days | OCR breach-portal submission |
| **GDPR Art. 33-34** | Personal-data breach | 72h supervisory authority + affected-data-subjects | EU EDPB national-DPA portal per Member State |
| **NIS2 Directive (2022/2555)** | Significant incident | 24h initial + 72h full + 1mo final | National CSIRT + EU CSIRT-Network |
| **EU AI Act Art. 73** | Serious AI-system incident | 24h | National competent authority + EU AI Office |
| **DSA Art. 17** | Content moderation decision | Statement-of-reasons per decision; ~real-time | EU DSA Transparency Database |
| **DSA Art. 22 (trusted flagger)** | Trusted-flagger referral | Per-flagger SLA | Trusted-flagger API |
| **KR-PIPA Art. 39** | Personal-data breach | 72h | KCPC (Korea Personal Information Protection Committee) |
| **KR-FSS** | Financial fraud breach | 24h fraud freeze cadence | FSS portal |
| **NY DFS Cybersecurity Regulation 23-NYCRR-500** | Cybersecurity event | 72h | DFS portal |
| **NCMEC CyberTipline** | CSAM detection | Mandatory; per 18 USC §2258A | NCMEC API |
| **GIFCT** | Terrorism content | Per-platform agreement; ~real-time | GIFCT hash-sharing API |
| **FinCEN SAR** | Suspicious financial activity | Per BSA 30/60/90 day cadences | FinCEN BSA E-Filing |
| **OFAC** | Sanctions match | Per OFAC 10-day reporting | OFAC reporting portal |
| **CFPB** | Adverse-action consumer-credit dispute | Per ECOA Reg B 30d + CFPB inquiry cadence | CFPB consumer-complaint portal |
| **FTC** | Consumer-protection violation | Per FTC inquiry cadence | FTC complaint portal |
| **FCA (UK)** | Financial-services regulatory breach | Per FCA inquiry cadence | FCA portal |
| **JP-FIU + JP-METI** | Financial intelligence / export control | Per-jurisdiction cadence | per-agency portal |
| **State-AG (US 50 states)** | State-level consumer protection | Per state-AG inquiry | Per-state portal |
| **California Privacy Protection Agency** | CCPA + CPRA enforcement | Per CPPA inquiry cadence | CPPA portal |
| **Colorado AG (CO AI Act, 2026-02-01)** | High-risk AI consumer disclosure | Per CO inquiry cadence | CO AG portal |
| **NY DCWP (NY AEDT Local Law 144 2023)** | AEDT bias audit + public notice | Annual + per-inquiry | DCWP portal |
| **DOJ Civil Rights Division** | ECOA + Fair Housing Act enforcement | Per DOJ inquiry cadence | DOJ portal |
| **HUD** | Fair Housing Act enforcement | Per HUD inquiry cadence | HUD portal |
| **EEOC** | Title VII enforcement | Per EEOC inquiry cadence | EEOC portal |
| **FBI IC3** | Cyber-crime reporting | Per FBI cadence | IC3 portal |
| **FedRAMP (when US-Gov pack active)** | Continuous monitoring per FedRAMP-High | Monthly + per-incident | FedRAMP secure portal |
| **National DPAs (28 EU MS + UK ICO)** | GDPR enforcement | Per-DPA inquiry cadence | Per-DPA portal |
| **Australian Privacy Commissioner (OAIC)** | Australian Privacy Act 1988 | 30 days for serious breach | OAIC portal |
| **Privacy Commissioner of Canada** | PIPEDA enforcement | Per OPC inquiry cadence | OPC portal |

### §A.5. What this ADR explicitly does NOT do

ADR-0310 is the **investigation case-management substrate**. It
does NOT:

1. **Author the detection substrate.** That lives in ADR-0307.
2. **Author the ML model lifecycle.** That lives in ADR-0308.
3. **Author the fairness invariants + per-jurisdiction model
   variants.** That lives in ADR-0309.
4. **Author the analyst UI.** That lives in
   `microservices/ops-dashboard-control-center/`.
5. **Author per-pack regulator notification cadence.** That lives
   in ADR-0251 + per-pack compliance.md.
6. **Author the user-facing appeal surface.** Per-µservice
   `compliance.md §appeal-surface` documents the per-µservice
   appeal route + entrypoint UI (which lives in product µservices,
   not in case-management substrate).

## Decision

### §B. Investigation case-management substrate

Establish `microservices/detection/case-management/` (subdirectory
of detection µservice — case-management is a closely-coupled
companion primitive to detection substrate; same µservice flat
layout per ADR-0131) exposing seven substrate primitives:

1. **Case lifecycle** — six-phase canonical workflow per §A.1
2. **Triage scorer** — per-signal priority assignment
3. **Investigator assignment** — per-pack expertise + per-tenant
   Cedar gate
4. **Evidence collection** — per-evidence chain-of-custody Merkle-
   sealed (per ADR-0028)
5. **Cedar-gated PII access** — per-case PII surface Cedar-
   evaluated (per ADR-0243)
6. **Per-pack regulator-facing surface emission** — per regulator
   roster (§A.4)
7. **Analyst-label feedback bridge** — bidirectional bridge to
   detection substrate's feature store (per ADR-0307 §D-7)

### §B.1. Case lifecycle — six-phase state machine

Per `investigation-case-lifecycle-schema.json`:

```yaml
states:
  - OPENED            # case opened (signal arrived OR appeal filed OR regulator inquiry)
  - TRIAGED           # priority assigned
  - INVESTIGATING     # investigator assigned + active investigation
  - ESCALATED         # routed to per-pack regulator / law enforcement / ombudsman
  - DISMISSED         # investigator dismissed; reason logged
  - ADJUDICATED       # appeal adjudicated; verdict assigned
  - FED_BACK          # analyst label sent to feature store
  - CLOSED            # final state; retention per pack

transitions:
  - from: OPENED
    to: TRIAGED
    actor: triage-scorer
    sla: 5min for P0, 30min for P1, 1h for P2, 4h for P3, 24h for P4
  - from: TRIAGED
    to: INVESTIGATING
    actor: case-router
    sla: per-pack expertise routing
  - from: INVESTIGATING
    to: ESCALATED
    actor: investigator
    permit: cedar://investigation::escalate
  - from: INVESTIGATING
    to: DISMISSED
    actor: investigator
    permit: cedar::investigation::dismiss
  - from: INVESTIGATING
    to: ADJUDICATED
    actor: investigator (appeal-adjudicator role)
    permit: cedar::investigation::adjudicate
    sla: per-pack appeal SLA (GDPR 1mo, ECOA 30d, KR-FSC 60d)
  - from: ADJUDICATED
    to: FED_BACK
    actor: feedback-bridge
  - from: DISMISSED
    to: FED_BACK
    actor: feedback-bridge
  - from: ESCALATED
    to: CLOSED
    actor: post-escalation-controller
    sla: per-regulator response cadence
  - from: FED_BACK
    to: CLOSED
    actor: retention-controller
    sla: per-pack retention
```

### §B.2. Triage scorer — per-priority assignment

Per `oya-shared-investigation-triage`. Composite priority based on:

| Factor | Weight | Source |
|---|---|---|
| Detection composite-scorer risk score | 30% | ADR-0307 §D-5 |
| Per-pack regulator floor (e.g., HIPAA breach → P0) | 30% | ADR-0251 |
| Affected-entity count (1 user vs 1000+ users) | 20% | per-µservice event |
| Critical-path exemption (per ADR-0298) | -100% (forces P0 audit-only) | ADR-0298 |
| DV-survivor-shelter mode (per §3.2.5 row 8) | per-case route shelter-team | per-µservice flag |
| Emergency-services tag (per ADR-0298) | bypass-all-block | per-µservice flag |
| Minor-protection (per ADR-0292) | priority elevation | ADR-0292 |
| Per-jurisdiction overlay | per-pack tightening | ADR-0309 |

Priority enum (P0-P4):

- **P0 (critical):** SEV1 cascade, EU AI Act Art. 73 serious
  incident, HIPAA breach, NCMEC CSAM detection. Triage SLA ≤5min;
  human reviewer SLA ≤1h.
- **P1 (high):** SEV2, GDPR Art. 33 breach, NIS2 significant
  incident. Triage ≤30min; reviewer ≤4h.
- **P2 (medium):** Standard appeals, fraud-detection medium-
  confidence. Triage ≤1h; reviewer ≤24h.
- **P3 (low):** Low-confidence appeals, retrospective audits.
  Triage ≤4h; reviewer ≤72h.
- **P4 (audit-only):** Emergency-services bypass-audited, critical-
  path-exempt cases, fairness-audit retrospective findings.
  Audit-only; no human review unless threshold exceeded.

Audit event: `InvestigationCaseTriagePriorityAssigned`.

### §B.3. Investigator assignment — per-pack expertise + Cedar gate

Per `oya-shared-investigation-case`. Investigator routing:

```
function route_investigator(case) {
  let candidates = investigator_pool.filter(|inv| {
    inv.has_expertise(case.family)
    && inv.has_jurisdiction(case.jurisdiction)
    && inv.has_pack_certifications(case.compliance_packs)
    && cedar_eval(
         principal: inv.principal,
         action: "investigation::access_case",
         resource: case,
         context: { case_priority, case_data_classes }
       ).is_permit()
  });
  let assigned = candidates.min_by(|inv| inv.current_case_load);
  case.investigator_id = assigned.id;
  audit.emit("InvestigationCaseInvestigatorAssigned", { case_id, inv_id: assigned.id });
  return assigned;
}
```

**Per-pack expertise roster:**

- **HIPAA-pack:** HIPAA-certified investigators (per pack-certified roster)
- **PCI-DSS:** QSA-equivalent or PCI-certified investigators
- **CSAM:** NCMEC-trained reviewers (FBI Innocent Images program-aligned)
- **Terrorism:** GIFCT-trained reviewers
- **AML:** ACAMS-certified (Association of Certified Anti-Money
  Laundering Specialists) investigators
- **KYC:** ACFE-certified (Association of Certified Fraud
  Examiners) investigators
- **Civil rights (ECOA/Fair Housing/NY AEDT):** civil-rights-
  trained reviewers
- **EU GDPR:** GDPR-trained DPO-certified reviewers
- **EU AI Act:** EU AI Act-trained reviewers
- **Korean (KR-PIPA + KR-FSC):** KR-jurisdiction-trained reviewers
- **Japanese (JP APPI):** JP-jurisdiction-trained reviewers

**Per-tenant Cedar gate:**

Per ADR-0243, every investigator-access to a case is Cedar-gated.
Per-tenant case-access requires the investigator's principal to
hold the `investigation::access::<tenant_id>` permit. Cross-tenant
investigators (e.g., oyatie ombudsman investigating cross-tenant
abuse rings) require the `investigation::access::cross-tenant`
permit which is rarely granted.

**Investigator load balancing:**

Per-investigator current case load capped at:

- P0: max 2 concurrent
- P1: max 5 concurrent
- P2: max 10 concurrent
- P3: max 20 concurrent
- P4: max 50 concurrent (audit-only; low touch)

### §B.4. Evidence collection — chain-of-custody Merkle-sealed

Per `oya-shared-investigation-evidence` + `oya-shared-investigation-chain-of-custody`.

#### §B.4.1. Per-evidence-add mechanics

```
function add_evidence(case_id, evidence_data, investigator) {
  // Verify Cedar permit for the data-class
  cedar_eval(
    principal: investigator.principal,
    action: "investigation::evidence::add",
    resource: case_id,
    context: { evidence_data_class: classify(evidence_data) }
  ).require_permit();
  // Hash the evidence
  let evidence_hash = blake3(evidence_data);
  // Build Merkle leaf
  let leaf = {
    case_id,
    evidence_hash,
    investigator_id: investigator.id,
    timestamp: now(),
    data_class: classify(evidence_data),
  };
  let leaf_hash = blake3(canonical_json(leaf));
  // Persist + emit
  case_evidence_store.append(case_id, leaf);
  audit_chain.append_to_per_case_tree(case_id, leaf_hash);
  audit_chain.append_to_per_day_tree(blake3(case_id || leaf_hash));
  audit.emit("InvestigationEvidenceAdded", { case_id, leaf_hash, investigator_id });
  return leaf_hash;
}
```

#### §B.4.2. Per-day public-witness emission

Per ADR-0028 §D-3:

```
function emit_daily_merkle_root() {
  let day_root = audit_chain.compute_daily_root();
  // Emit to Sigstore Rekor for public-witness
  sigstore_rekor.append(day_root);
  // Optional: emit to Bitcoin OP_RETURN (per ADR-0028 §D-3 option)
  // Optional: emit to Certificate Transparency log (per ADR-0028 §D-3 option)
  audit.emit("AuditChainDailyRootEmitted", { day_root });
}
```

#### §B.4.3. Court-admissible verification

Given a case-id + evidence-hash + Merkle proof:

```
function verify_chain_of_custody(case_id, evidence_hash, merkle_proof) {
  let case_root = audit_chain.fetch_case_root(case_id);
  assert merkle_verify(evidence_hash, merkle_proof, case_root);
  let day_root = audit_chain.fetch_day_root_containing(case_root);
  assert merkle_verify(case_root, day_root.case_proof, day_root);
  let public_witness = sigstore_rekor.fetch(day_root);
  assert public_witness.is_some();  // public-witness confirms not tampered
  return Verified {
    case_id, evidence_hash, day_root, sigstore_rekor_uri: public_witness.uri
  };
}
```

Per FRE Rule 901 (US) + EU Council Regulation 910/2014 (eIDAS
qualified electronic signatures) + KR Electronic Signature Act +
JP Electronic Signatures Act, this chain-of-custody satisfies the
court-admissibility floor.

### §B.5. Cedar-gated PII access

Per ADR-0243. Per-case PII surface enumerated in
`policy/investigation-pii-access.cedar`:

```cedar
// Default-deny baseline (always present)
forbid(
  principal,
  action,
  resource
) when {
  resource is Case
};

// Per-pack-investigator-can-access-tenant-PII
permit(
  principal in InvestigatorPool,
  action == Action::"investigation::pii::access",
  resource is Case
) when {
  // Investigator has expertise for the case family
  principal.expertise_family.contains(resource.family)
  // Investigator has jurisdiction
  && principal.jurisdiction.contains(resource.jurisdiction)
  // Investigator has pack certification
  && principal.pack_certifications.containsAll(resource.compliance_packs)
  // Investigator has tenant access (per ADR-0244)
  && principal.tenant_access_permits.contains(resource.tenant_id)
  // Investigator has data-class permit
  && principal.data_class_permits.contains(resource.data_class)
  // Case is in INVESTIGATING state
  && resource.lifecycle_state == "INVESTIGATING"
};

// Forbid post-departure access (per ADR-0307 §A.1 row 7 insider risk)
forbid(
  principal,
  action == Action::"investigation::pii::access",
  resource
) when {
  principal.termination_date != null
  && now() > principal.termination_date.add(hours: 48)
};

// Forbid cross-tenant PII access (per ADR-0244)
forbid(
  principal,
  action == Action::"investigation::pii::access",
  resource
) when {
  resource.tenant_id != principal.tenant_access_permits[0]
  && !principal.has_cross_tenant_permit
};
```

Audit event: `InvestigationPIIAccessed`.

### §B.6. Per-pack regulator-facing surface emission

Per `oya-shared-investigation-regulator-surface`.

#### §B.6.1. Per-pack template registry

Per `microservices/detection/case-management/regulator-templates/`:

```yaml
hipaa-breach-notification:
  trigger: HIPAA_PHI_breach_detected
  sla_hours: 1440  # 60 days
  wire_format: ocr_breach_portal_v2
  required_fields:
    - covered_entity_name
    - breach_description
    - affected_individuals_count
    - phi_types_involved
    - mitigation_steps
gdpr-72h-notification:
  trigger: GDPR_personal_data_breach
  sla_hours: 72
  wire_format: edpb_national_dpa_portal_v1
  required_fields:
    - controller_dpo_contact
    - breach_nature
    - data_subjects_count
    - data_categories
    - measures_taken
nis2-three-stage:
  trigger: significant_incident
  cadences:
    - hours: 24
      stage: early_warning
    - hours: 72
      stage: incident_notification
    - hours: 720  # 1 month
      stage: final_report
  wire_format: national_csirt_eu_csirt_network
eu-ai-act-art-73:
  trigger: serious_ai_incident
  sla_hours: 24
  wire_format: national_competent_authority_eu_ai_office
  required_fields:
    - ai_system_id
    - incident_description
    - affected_users
    - fundamental_rights_impact
    - mitigation_steps
dsa-art-17-statement-of-reasons:
  trigger: content_moderation_decision
  sla_hours: 1  # ~real-time
  wire_format: eu_dsa_transparency_database
  required_fields:
    - content_id
    - decision
    - reason_category
    - automated_flag
    - per_user_notification
ncmec-cybertipline:
  trigger: CSAM_detection
  sla_hours: 1  # mandatory; per 18 USC §2258A
  wire_format: ncmec_api
  required_fields:
    - content_hash
    - upload_user_id
    - upload_timestamp
    - photodna_match_score
    - per_user_account_status
fincen-sar:
  trigger: suspicious_activity_threshold
  sla_hours: 720  # 30 days
  wire_format: fincen_bsa_efile
  required_fields:
    - sar_subject
    - transaction_details
    - suspicious_indicator
    - prior_sar_history
ofac-sanctions-match:
  trigger: sanctions_match
  sla_hours: 240  # 10 days
  wire_format: ofac_reporting_portal
  required_fields:
    - sanctions_list_match
    - transaction_details
    - blocked_property_value
```

#### §B.6.2. Per-pack template fan-out

```
function emit_regulator_surface(case) {
  for pack in case.compliance_packs {
    let templates = regulator_template_registry.fetch_by_pack(pack);
    for template in templates {
      if template.trigger.matches(case) {
        let payload = fill_template(template, case);
        let surface = regulator_surface_adapter[template.wire_format];
        surface.deliver(payload, template.sla_hours);
        audit.emit("InvestigationRegulatorEmissionDelivered", {
          case_id: case.id,
          template: template.id,
          sla_hours: template.sla_hours,
          delivery_status: surface.last_delivery_status,
        });
      }
    }
  }
}
```

### §B.7. Audit-event-class taxonomy (10 new classes)

Per ADR-0263 emission contract, the case-management substrate
emits 10 new audit-event-classes:

| Class | Cardinality budget | Trace span shape | Retention |
|---|---|---|---|
| `InvestigationCaseOpened` | ~10³-10⁵/day | Parent: case-opener | per-pack retention (HIPAA 6yr, GDPR varies, NCMEC permanent) |
| `InvestigationCaseTriagePriorityAssigned` | ~10³-10⁵/day | Parent: triage-scorer | per-pack |
| `InvestigationCaseInvestigatorAssigned` | ~10³-10⁵/day | Parent: case-router | per-pack |
| `InvestigationEvidenceAdded` | ~10⁴-10⁶/day | Parent: evidence-collector; Child: chain-of-custody-Merkle-leaf | per-pack (10yr+ for EU AI Act high-risk) |
| `InvestigationPIIAccessed` | ~10⁵/day | Parent: investigator-PII-API | per-pack |
| `InvestigationCaseEscalated` | ~10²/day | Parent: escalation-controller | per-pack |
| `InvestigationCaseDismissed` | ~10³/day | Parent: investigator | per-pack |
| `InvestigationCaseAdjudicated` | ~10³-10⁴/day | Parent: appeal-adjudicator | per-pack |
| `InvestigationLabelFedBack` | ~10⁴/day | Parent: feedback-bridge | per-pack |
| `InvestigationRegulatorEmissionDelivered` | ~10-10³/day | Parent: regulator-surface-adapter | per-pack |

## §C. Consequences

The 6 engineering-rigor dimensions per documentation-rigor.md §1.2:

### §C.1. Maintainability dimension

The case-management substrate's maintainability surface is
concentrated in single-concern crates under `crates/`:

- `oya-shared-investigation-case` — case lifecycle + state machine
- `oya-shared-investigation-triage` — priority scorer
- `oya-shared-investigation-evidence` — evidence collection
- `oya-shared-investigation-chain-of-custody` — Merkle-sealed
  chain-of-custody
- `oya-shared-investigation-regulator-surface` — per-pack regulator
  template emission
- `oya-shared-investigation-appeal-adjudication` — appeal workflow
- `oya-shared-investigation-analyst-label-feedback` — feedback
  bridge

Per-pack regulator templates live under
`microservices/detection/case-management/regulator-templates/<pack>.yaml`.

Versioning policy: every crate SemVer per ADR-0258; the
investigation-case-schema.json + investigation-case-lifecycle-schema.json
+ investigation-chain-of-custody-schema.json are versioned via
`_meta.schema_version`; breaking changes require ADR amendment +
60-day deprecation. Per-pack regulator templates evolve
independently (per-template version + per-template effective-date).

Per-config-flag rationale: ~25 per-tenant config flags (per-pack
regulator template selection, per-pack SLA override, per-tenant
investigator pool, per-pack PII gating strictness, per-jurisdiction
escalation routing). Each flag has a documented default + per-pack
override behavior.

Reverse dependencies: detection µservice (all 8 families), ml-
lifecycle (appeal-flow), fairness-audit (regulator-surface),
ops-dashboard-control-center (UI), every adverse-action-driving
µservice (payments, identity, marketplace, social, community,
messenger, mail).

### §C.2. Observability dimension

Per ADR-0263 emission contract, the case-management substrate
emits 10 new audit-event-classes (per §B.7).

Metrics (Prometheus + OpenTelemetry per ADR-0263):

- `investigation_case_opened_total{tenant_id, family, priority}` — counter
- `investigation_case_triage_latency_seconds{priority, p50|p95|p99}` — histogram; SLA per §B.2
- `investigation_case_investigator_assigned_latency_seconds{priority, p50|p95|p99}` — histogram
- `investigation_case_lifecycle_state{tenant_id, lifecycle_state}` — gauge
- `investigation_evidence_added_total{case_id, data_class}` — counter
- `investigation_chain_of_custody_merkle_verified_total{case_id}` — counter
- `investigation_pii_accessed_total{tenant_id, investigator_id, data_class}` — counter
- `investigation_appeal_filed_total{tenant_id, family, jurisdiction}` — counter
- `investigation_appeal_adjudication_latency_seconds{pack, p50|p95|p99}` — histogram; SLA per pack
- `investigation_regulator_emission_delivered_total{regulator, pack, template}` — counter
- `investigation_regulator_emission_sla_breach_total{regulator, pack}` — counter (should be ~0)
- `investigation_analyst_label_fed_back_total{model_id, family}` — counter

Dashboards (Grafana, stored in `microservices/detection/dashboards/`):

1. `investigation-overview.json` — case volume + priority distribution + lifecycle state distribution
2. `investigation-triage-sla.json` — triage SLA tracking per priority
3. `investigation-investigator-load.json` — per-investigator case load + capacity
4. `investigation-chain-of-custody.json` — evidence-add rate + Merkle-verification rate
5. `investigation-pii-access-audit.json` — per-investigator PII access audit (Cedar permit decisions)
6. `investigation-appeal-adjudication.json` — appeal volume + SLA per pack + verdict distribution
7. `investigation-regulator-emission.json` — per-regulator delivery rate + SLA compliance
8. `investigation-analyst-label-feedback-loop.json` — analyst-label-volume + drift-trigger-rate

SLO floor (per `microservices/detection/slos/*.openslo.yaml`):

- Triage SLA per priority (P0 ≤5min, P1 ≤30min, P2 ≤1h, P3 ≤4h, P4 ≤24h); 99.9% monthly
- Investigator assignment SLA per priority; 99.5% monthly
- Per-pack appeal adjudication SLA (GDPR 1mo, ECOA 30d, KR-FSC 60d); 99.9% monthly
- EU AI Act Art. 73 24h regulator emission; 100% (BLOCKER)
- GDPR Art. 33 72h regulator emission; 100% (BLOCKER)
- NIS2 24h/72h/1mo cadence; 100% (BLOCKER)
- HIPAA 60d breach notification; 100% (BLOCKER)
- NCMEC CyberTipline 1h emission; 100% (BLOCKER)
- Chain-of-custody Merkle-seal integrity; 100% (BLOCKER)
- Per-day public-witness emission to Sigstore Rekor; 99.99% monthly

### §C.3. Scalability dimension

Capacity math per documentation-rigor.md §1.1 item 3:

**Case-opening rate.** At platform GA: ~10³-10⁵ cases/day = ~10-1000
cases/sec sustained. Per-case ingestion ≤10ms via Kafka topic
`audit.investigation.case.opened`; per-cell sized for ~100-1000
opening/sec.

**Triage scorer.** Per-case scorer runtime ≤100ms (LightGBM-class
priority model); per-cell sustains ~10⁴/sec.

**Investigator assignment.** Per-case routing ≤500ms (includes
Cedar permit eval + investigator-pool query); per-cell sustains
~10⁴/sec.

**Evidence collection.** Per-evidence add ≤200ms (includes Cedar
permit + hash + Merkle-leaf append). Per-cell sustains ~10⁴/sec.

**Cedar-gated PII access.** Per-access ≤100ms (Cedar permit eval
+ per-pack overlay + audit emit). Per-cell sustains ~10⁵/sec
(higher than other primitives because PII access fires per analyst
keystroke during investigation).

**Per-pack regulator surface emission.** Per-emission ≤5s (includes
template fill + wire-format adapter + delivery). Per-cell sustains
~10²-10³/sec.

**Analyst-label feedback.** Per-label ≤500ms (includes feature-
store write + retraining-queue check). Per-cell sustains ~10⁴/sec.

10× and 100× scale-out path: per-primitive horizontal scale-out
via stateless replicas + Kafka partition increase + Postgres
read-replica fan-out + Redis Cluster shard increase. Bottleneck at
100×: chain-of-custody Merkle-tree write rate (one Merkle-root
per second per cell is the comfortable bound; at 100× ingest,
multiple per-case trees may merge into a higher-level batch tree).

### §C.4. Performance dimension

| Primitive | P50 | P95 | P99 | Tail mitigation |
|---|---|---|---|---|
| Case opening | 10ms | 30ms | 80ms | Async Kafka send + ack |
| Triage scoring | 30ms | 100ms | 300ms | LightGBM batch inference |
| Investigator assignment | 100ms | 500ms | 1s | Per-investigator-pool query cache |
| Evidence add | 50ms | 200ms | 500ms | Async Merkle-leaf append + batched daily root |
| Cedar permit eval (PII access) | 5ms | 30ms | 100ms | Per ADR-0246 library-first; in-process eval |
| Per-pack regulator surface delivery | 1s | 3s | 5s | Async + retry-with-backoff |
| Appeal adjudication SLA (per pack) | 7d | 21d | 30d | Per-pack human-reviewer queue |
| Analyst-label feedback | 200ms | 500ms | 1s | Async write to feature-store |

Per-region budget split: per ADR-0240 sovereign-cloud overlay, EU
cells run independent case-management from US cells; per-pack
residency honored.

Cold-start budget: case-management substrate cold-start ≤30s per
Tier-2 cell (Cedar engine warm + investigator-pool registry warm
+ regulator-template registry warm).

### §C.5. Optimization dimension

Per-stage cost model:

- Case opening + triage: ~$0.00005 per case × 10⁵/day = $5/day per
  cell = ~$1,800/month per cell
- Investigator assignment: ~$0.0002 per case × 10⁵/day = $20/day
- Evidence add: ~$0.00001 per evidence × ~10⁶/day = $10/day
- Cedar PII permit eval: ~$0.0000001 per eval × ~10⁵/day = trivial
- Per-pack regulator delivery: ~$0.01 per delivery × ~10³/day = $10/day
- Human-reviewer adjudication: $50-200/hour × per-pack-investigator-pool
- Chain-of-custody Merkle batch: ~$5/day per cell (Sigstore Rekor write)
- Per-pack retention storage: per-pack residency × storage tier;
  HIPAA 6yr requires ~$X/month per-tenant; GDPR varies per request;
  NCMEC permanent retention requires specialty surface

Lazy vs eager trade-offs:

- **Eager** for triage scoring (priority SLA depends on it)
- **Eager** for Cedar PII permit eval (cannot afford to deny PII
  access after access happened)
- **Eager** for chain-of-custody Merkle leaf write (legal-grade)
- **Lazy** for per-pack regulator surface emission (template fill
  on case-state-transition, not on every event)
- **Lazy** for analyst-label feedback to feature store (batched
  every 10min to amortize feature-store write cost)
- **Cached** for investigator-pool registry (Redis-cached; per-pack
  invalidation on roster update)
- **Cached** for regulator-template registry (Redis-cached; per-
  template invalidation on template publish)

### §C.6. Code quality dimension

Per documentation-rigor.md §1.2:

- **Test classes:** unit (per-lifecycle-transition, per-template-
  fill, per-Cedar-permit), property-based (chain-of-custody Merkle-
  proof correctness; lifecycle state-machine reachability), fuzz
  (case-schema validator + Cedar fragment parser), load (per-pack
  regulator emission at SLA), e2e (full case lifecycle for
  synthetic-fraud + synthetic-appeal scenarios)
- **Coverage floor:** ≥85% line, ≥75% branch
- **Lint passes:** `cargo clippy -- -D warnings`,
  `oya-check-investigation-case-schema-conformance`,
  `oya-check-investigation-cedar-fragment-soak`,
  `oya-check-naming-bnf-v4`, `oya-check-layer-enum-conformance`,
  `oya-governance-investigation-case-substrate-emission`
- **Type-strictness:** Rust `deny(warnings)` + `deny(unsafe_code)`
- **SemVer + ABI policy:** per ADR-0258

## §D. Detailed mechanics

### §D-1. Case lifecycle state-machine mechanics

#### §D-1.1. Transition guards

Every state transition is guarded by Cedar permit + per-pack
overlay:

```
function transition(case, target_state, actor) {
  cedar_eval(
    principal: actor.principal,
    action: format!("investigation::transition::{}", target_state),
    resource: case,
    context: { current_state: case.lifecycle_state, target_state }
  ).require_permit();
  // Per-pack overlay
  for pack in case.compliance_packs {
    let pack_rules = pack_rules_registry.fetch(pack);
    pack_rules.validate_transition(case, target_state)?;
  }
  let prev_state = case.lifecycle_state;
  case.lifecycle_state = target_state;
  case.transition_history.push({ from: prev_state, to: target_state, actor: actor.id, timestamp: now() });
  audit.emit(format!("InvestigationCase{}", title_case(target_state)), { case_id: case.id, from: prev_state });
}
```

#### §D-1.2. Per-pack lifecycle constraints

- **HIPAA-pack:** PHI evidence MUST be encrypted-at-rest with
  per-tenant DEK + KEK (per ADR-0099 + ADR-0244); cross-tenant
  evidence-add forbidden.
- **PCI-DSS-pack:** PAN evidence MUST be tokenized before storage;
  per-pack PCI-certified investigator access required.
- **GDPR-pack:** Per-data-subject access surface (per GDPR Art. 15);
  per-data-subject erasure handled by case-retention policy.
- **EU-AI-Act-high-risk:** Per Art. 18 10-year retention; per-case
  + per-evidence retained.
- **NCMEC-pack:** CSAM evidence routes to NCMEC CyberTipline per
  18 USC §2258A; per-investigator FBI-Innocent-Images-program
  authorization required.

### §D-2. Triage scorer mechanics

#### §D-2.1. Priority computation

```
function compute_priority(case) {
  let base = case.detection_signal.composite_score * 0.3;
  let pack_floor = max(case.compliance_packs.map(|p| pack_registry.priority_floor(p))) * 0.3;
  let affected_factor = min(1.0, case.affected_entity_count / 1000) * 0.2;
  // Critical-path exemption forces P0 audit-only
  if case.critical_path_exemption {
    return P0_AUDIT_ONLY;
  }
  // DV-survivor-shelter mode routes shelter team
  if case.shelter_mode {
    return SHELTER_TEAM_PRIORITY;
  }
  // Minor-protection elevates per ADR-0292
  let minor_factor = case.minor_protection ? 0.2 : 0.0;
  let composite = base + pack_floor + affected_factor + minor_factor;
  return priority_thresholds.map(composite);
}
```

#### §D-2.2. Per-pack priority floor

```yaml
hipaa-breach: P0
gdpr-art-33-breach: P0
nis2-significant-incident: P0
eu-ai-act-art-73-serious-incident: P0
ncmec-csam: P0
fincen-sar-threshold: P1
ofac-sanctions-match: P1
ecoa-adverse-action: P2
dsa-art-17-content-moderation: P2
ny-aedt-bias-audit-finding: P2
```

### §D-3. Investigator assignment mechanics

#### §D-3.1. Per-investigator profile

```yaml
investigator_id: inv-12345
principal: "oyatie::investigator::trust-safety::us-region"
jurisdiction:
  - US
  - NY
  - CA
expertise_family:
  - payment_fraud
  - ato
  - aml_sanctions
pack_certifications:
  - HIPAA
  - PCI-DSS
  - GDPR
  - EU-AI-Act
  - CSAM-NCMEC
  - terrorism-GIFCT
  - AML-ACAMS
  - civil-rights-ECOA-Fair-Housing-NY-AEDT
tenant_access_permits:
  - tenant-001
  - tenant-002
  - tenant-003
data_class_permits:
  - PII
  - PSEUDONYMOUS
  - AGGREGATE
  - DERIVED
current_case_load:
  P0: 1
  P1: 3
  P2: 7
  P3: 12
  P4: 25
termination_date: null
language_proficiency:
  - en
  - ko
  - ja
```

#### §D-3.2. Per-case investigator pool query

```
function query_investigator_pool(case) {
  let candidates = investigator_pool.filter(|inv| {
    inv.expertise_family.contains(case.family)
    && inv.jurisdiction.contains(case.jurisdiction)
    && inv.pack_certifications.containsAll(case.compliance_packs)
    && inv.tenant_access_permits.contains(case.tenant_id)
    && inv.data_class_permits.containsAll(case.data_classes)
    && inv.current_case_load[case.priority] < max_load[case.priority]
    && inv.termination_date == null
    // Language preference if user-facing
    && (case.affected_party_language == null
        || inv.language_proficiency.contains(case.affected_party_language))
  });
  return candidates;
}
```

### §D-4. Evidence collection mechanics

#### §D-4.1. Evidence types

Per `investigation-evidence-schema.json`:

```yaml
evidence_types:
  - audit_event_reference            # ref to ADR-0263 audit-event
  - feature_store_snapshot            # snapshot of features at time of decision
  - model_card_reference              # ref to model card per ADR-0308
  - cedar_evaluation_log              # Cedar decision log
  - cilium_network_policy_violation   # network-policy violation per ADR-0145
  - user_statement                    # user-provided narrative
  - investigator_note                 # investigator narrative
  - external_document                 # uploaded by user via appeal
  - external_api_response             # NCMEC PhotoDNA, GIFCT match, OFAC match
  - regulator_correspondence          # incoming regulator inquiry/order
  - chain_of_custody_proof            # Merkle proof
```

#### §D-4.2. Per-evidence data-class classification

```
function classify(evidence_data) {
  if evidence_data.contains_pii() {
    return DataClass::PII;
  }
  if evidence_data.is_pseudonymized() {
    return DataClass::PSEUDONYMOUS;
  }
  if evidence_data.is_aggregate() {
    return DataClass::AGGREGATE;
  }
  return DataClass::DERIVED;
}
```

#### §D-4.3. Merkle leaf format

Per `investigation-chain-of-custody-schema.json`:

```json
{
  "schema_version": "1.0",
  "case_id": "case-abc123",
  "evidence_hash": "blake3:0x1234...",
  "evidence_type": "audit_event_reference",
  "data_class": "PII",
  "investigator_id": "inv-12345",
  "investigator_principal_signature": "ed25519:...",
  "timestamp": "2026-05-20T10:30:00Z",
  "previous_leaf_hash": "blake3:0xabcd...",
  "leaf_hash": "blake3:0x5678..."
}
```

#### §D-4.4. Per-case Merkle tree construction

Per ADR-0028:

- Per-case append-only Merkle tree; leaves are evidence + lifecycle
  transitions
- Per-case root computed on every transition
- Per-day platform root rolls up all per-case roots
- Per-day root published to Sigstore Rekor (per ADR-0028 §D-3)

### §D-5. Cedar PII gating mechanics

#### §D-5.1. Per-investigation Cedar fragment

Per `policy/investigation-pii-access.cedar`:

```cedar
// See §B.5 above for the full fragment

// Per-purpose access permit
permit(
  principal in InvestigatorPool,
  action == Action::"investigation::pii::access",
  resource is Case
) when {
  ...
  // Per-investigator must declare purpose
  && context.access_purpose in [
      "case_investigation",
      "appeal_adjudication",
      "regulator_inquiry_response",
      "law_enforcement_warrant_response"
    ]
  // Per-data-subject access doesn't bypass per-pack consent
  && (
    context.data_subject.consents.contains(context.access_purpose)
    || pack_compels_investigation(resource.compliance_packs)
  )
};

// Pack-compels-investigation policy
// HIPAA Art. permits PHI access for compliance with law
// GDPR Art. 6(1)(c) permits processing for legal obligation
// GDPR Art. 6(1)(f) permits processing for legitimate interests
// NCMEC reporting per 18 USC §2258A
function pack_compels_investigation(packs) {
  return packs.contains("HIPAA") && context.access_purpose == "law_enforcement_warrant_response"
      || packs.contains("GDPR") && context.access_purpose in ["regulator_inquiry_response", "law_enforcement_warrant_response"]
      || packs.contains("CSAM-NCMEC") && context.access_purpose == "case_investigation"
      || packs.contains("AML-ACAMS") && context.access_purpose == "regulator_inquiry_response";
}
```

#### §D-5.2. Per-access audit trail

Every PII access emits `InvestigationPIIAccessed`:

```json
{
  "case_id": "case-abc123",
  "investigator_id": "inv-12345",
  "data_class": "PII",
  "data_subject_id": "user-xyz789",
  "access_purpose": "case_investigation",
  "cedar_permit_id": "permit-investigation-pii-access-v1.2",
  "timestamp": "2026-05-20T10:35:00Z"
}
```

### §D-6. Appeal adjudication mechanics

#### §D-6.1. Per-appeal-filing workflow

Per ADR-0308 §B.8:

```
POST /v1/investigation/appeal
Body: {
  decision_id: string,           # links to ADR-0307 signal or ADR-0308 model decision
  affected_party_id: string,
  appeal_reason: string,
  requested_outcome: "reverse" | "explain" | "modify",
  evidence: [...]
}
```

Routes to case-management; opens new case with `appeal_id` link
to decision.

#### §D-6.2. Per-pack appeal SLA

```yaml
GDPR: 1 month  # Art. 12(3); extendable +2 months on complexity
ECOA-Reg-B: 30 days  # 12 CFR §1002.9
KR-FCPA-Art-30: 60 days  # KR-FSC cadence
NY-AEDT-Local-Law-144-2023: 10 business days candidate notice
EU-AI-Act-Art-86: 30 days
DSA-Art-17: ~real-time statement-of-reasons; 6 months substantive
HUD-Fair-Housing: 100 days
EEOC-Title-VII: 180 days
default: 30 days
```

#### §D-6.3. Adjudication verdict

```
function adjudicate(appeal_case) {
  let adjudicator = appeal_case.investigator;  // appeal-adjudicator role
  let verdict = adjudicator.adjudicate(appeal_case);  // human decision
  case.verdict = verdict;
  case.verdict_reasoning = adjudicator.reasoning;
  case.lifecycle_state = "ADJUDICATED";
  audit.emit("InvestigationCaseAdjudicated", { ... });
  if verdict == "reverse" {
    // Reverse the original decision (per ADR-0308 §D-8.4)
    original_decision_engine.reverse(appeal_case.decision_id);
    // Update feature store with analyst label
    feature_store.write_analyst_label(appeal_case.decision_id, "reverse");
    retraining_queue.enqueue_label_feedback(appeal_case.affected_party_id);
  }
  notify_affected_party(appeal_case);
}
```

### §D-7. Per-pack regulator-emission mechanics

#### §D-7.1. Template fill

```
function fill_template(template, case) {
  let payload = {};
  for field in template.required_fields {
    payload[field] = field_resolver.resolve(case, field);
  }
  return payload;
}
```

#### §D-7.2. Wire-format adapter

Per `oya-shared-investigation-regulator-surface` per-format adapter:

- `ocr_breach_portal_v2` — HIPAA OCR Breach Notification Portal
- `edpb_national_dpa_portal_v1` — GDPR per-Member-State DPA portal
- `national_csirt_eu_csirt_network` — NIS2 CSIRT-Network
- `national_competent_authority_eu_ai_office` — EU AI Act Art. 73
- `eu_dsa_transparency_database` — DSA Art. 17
- `ncmec_api` — NCMEC CyberTipline API
- `fincen_bsa_efile` — FinCEN SAR e-filing
- `ofac_reporting_portal` — OFAC sanctions reporting

#### §D-7.3. SLA-tracker

```
function track_sla(emission) {
  let deadline = emission.created_at + duration(emission.template.sla_hours);
  if now() > deadline && emission.delivery_status != "delivered" {
    audit.emit("InvestigationRegulatorEmissionSLABreach", {
      emission_id: emission.id, deadline, current_status: emission.delivery_status
    });
    sev1_alert(emission);
  }
}
```

### §D-8. Analyst-label feedback bridge mechanics

#### §D-8.1. Per-case label routing

```
function emit_analyst_label(case) {
  let label = {
    decision_id: case.decision_id,
    affected_party_id: case.affected_party_id,
    model_id: case.model_id,
    family: case.family,
    analyst_verdict: case.verdict,  // reverse / uphold / partial
    investigator_id: case.investigator_id,
    reasoning: case.verdict_reasoning,
    timestamp: now(),
  };
  feature_store.write_analyst_label(label);
  audit.emit("InvestigationLabelFedBack", label);
  // Check if retraining threshold crossed
  let label_count_since_last_retrain = feature_store.count_labels_since_last_retrain(case.model_id);
  if label_count_since_last_retrain >= 1000 {
    retraining_queue.enqueue(case.model_id);
  }
}
```

#### §D-8.2. Per-family label batching

Labels batched per family every 10min to amortize feature-store
write cost.

### §D-9. Per-tenant case ownership mechanics

#### §D-9.1. Tenant-scoped case query

```
function query_tenant_cases(tenant_id, principal, query) {
  cedar_eval(
    principal,
    action: "investigation::cases::query",
    resource: tenant_id
  ).require_permit();
  return case_store.query(
    tenant_id: tenant_id,
    filter: query,
    cedar_principal: principal  // additional per-row Cedar filter applied
  );
}
```

#### §D-9.2. Cross-tenant case visibility (rare)

Per ADR-0244, cross-tenant case visibility forbidden by default;
explicit `investigation::access::cross-tenant` permit required for
oyatie ombudsman investigating cross-tenant abuse rings.

### §D-10. Critical-path + emergency-services + DV-survivor invariants

Per ADR-0298 + §3.2.5:

1. **Emergency-services exemption.** Per ADR-0298 §B, emergency-
   services traffic NEVER blocked; case opened in audit-only mode;
   no investigator assigned unless threshold crossed.
2. **Critical-path exemption.** Per §3.2.5 healthcare-acute-care +
   crisis-line + financial-emergency cases: audit-only.
3. **DV-survivor-shelter-mode.** Per §3.2.5 row 8: shelter team
   routing; abuser-party never notified of case existence; investigator
   pool restricted to shelter-trained reviewers.

## §E. Implementation footprint

### §E.1. New µservice extension — microservices/detection/case-management/

Per ADR-0131 flat layout; case-management is a closely-coupled
companion primitive to detection substrate, lives under detection
µservice (not a separate µservice).

Directory tree:

```
microservices/detection/case-management/
├── PHASE-09-case-management-lifecycle.md
├── PHASE-10-case-management-regulator-surface.md
├── PHASE-11-case-management-appeal-workflow.md
├── policy/
│   ├── investigation-default-deny.cedar
│   ├── investigation-pii-access.cedar
│   ├── investigation-evidence-add.cedar
│   ├── investigation-case-transition.cedar
│   ├── investigation-cross-tenant.cedar
│   └── investigation-regulator-surface.cedar
├── regulator-templates/
│   ├── hipaa-breach-notification.yaml
│   ├── gdpr-72h-notification.yaml
│   ├── nis2-three-stage.yaml
│   ├── eu-ai-act-art-73.yaml
│   ├── dsa-art-17-statement-of-reasons.yaml
│   ├── ncmec-cybertipline.yaml
│   ├── gifct-hash-sharing.yaml
│   ├── fincen-sar.yaml
│   ├── ofac-sanctions-match.yaml
│   ├── kr-pipa-72h.yaml
│   ├── kr-fss-24h.yaml
│   ├── ny-dfs-72h.yaml
│   ├── cfpb-adverse-action.yaml
│   ├── ftc-consumer-protection.yaml
│   ├── ny-aedt-public-notice.yaml
│   ├── co-ai-act.yaml
│   ├── doj-civil-rights.yaml
│   ├── hud-fair-housing.yaml
│   ├── eeoc-title-vii.yaml
│   ├── fbi-ic3.yaml
│   ├── fedramp-continuous-monitoring.yaml
│   └── ...
├── runbooks/
│   ├── case-opened-p0-incident.md
│   ├── case-triage-sla-breach.md
│   ├── case-investigator-pool-exhaustion.md
│   ├── case-chain-of-custody-failure.md
│   ├── case-cedar-pii-permit-denial.md
│   ├── case-regulator-emission-sla-breach.md
│   ├── case-appeal-sla-breach.md
│   ├── case-eu-ai-act-art-73-24h-incident.md
│   ├── case-gdpr-72h-incident.md
│   ├── case-ncmec-csam-incident.md
│   └── case-dv-survivor-shelter-routing.md
├── src/
│   ├── case_lifecycle/
│   │   ├── mod.rs
│   │   ├── state_machine.rs
│   │   └── transitions.rs
│   ├── triage/
│   │   ├── mod.rs
│   │   └── priority_scorer.rs
│   ├── investigator_pool/
│   │   ├── mod.rs
│   │   ├── per_pack_expertise.rs
│   │   └── cedar_filter.rs
│   ├── evidence/
│   │   ├── mod.rs
│   │   ├── classifier.rs
│   │   └── chain_of_custody.rs
│   ├── pii_access/
│   │   ├── mod.rs
│   │   └── cedar_gate.rs
│   ├── regulator_surface/
│   │   ├── mod.rs
│   │   ├── template_registry.rs
│   │   ├── per_pack_fan_out.rs
│   │   └── wire_format_adapters/
│   ├── appeal_adjudication/
│   │   ├── mod.rs
│   │   └── per_pack_sla.rs
│   └── feedback_bridge/
│       ├── mod.rs
│       └── label_router.rs
├── catalog/
│   ├── oya-shared-investigation-case.catalog.yaml
│   ├── oya-shared-investigation-triage.catalog.yaml
│   ├── oya-shared-investigation-evidence.catalog.yaml
│   ├── oya-shared-investigation-chain-of-custody.catalog.yaml
│   ├── oya-shared-investigation-regulator-surface.catalog.yaml
│   ├── oya-shared-investigation-appeal-adjudication.catalog.yaml
│   ├── oya-shared-investigation-analyst-label-feedback.catalog.yaml
│   └── microservice-detection-case-management.catalog.yaml
├── iac/
│   ├── dev-investigation-postgres-cluster.tf
│   ├── prod-investigation-postgres-cluster.tf
│   ├── dev-investigation-merkle-tree-storage.tf
│   ├── prod-investigation-merkle-tree-storage.tf
│   ├── dev-investigation-network-policy.yaml
│   ├── prod-investigation-network-policy.yaml
│   └── prod-investigation-sigstore-rekor-config.yaml
├── slos/
│   ├── case-triage-sla.openslo.yaml
│   ├── case-investigator-assignment-sla.openslo.yaml
│   ├── case-appeal-adjudication-sla.openslo.yaml
│   ├── regulator-emission-sla.openslo.yaml
│   └── chain-of-custody-merkle-integrity.openslo.yaml
└── ...
```

### §E.2. New crates (per layer-5 shared-substrate)

Per ADR-0105 13-layer canonical enum row 5:

1. `crates/oya-shared-investigation-case/` — case lifecycle + state machine
2. `crates/oya-shared-investigation-triage/` — priority scorer + per-pack floor
3. `crates/oya-shared-investigation-evidence/` — evidence collection + data-class classifier
4. `crates/oya-shared-investigation-chain-of-custody/` — Merkle-sealed chain-of-custody per ADR-0028
5. `crates/oya-shared-investigation-regulator-surface/` — per-pack template registry + wire-format adapters
6. `crates/oya-shared-investigation-appeal-adjudication/` — appeal workflow + per-pack SLA
7. `crates/oya-shared-investigation-analyst-label-feedback/` — bidirectional feedback bridge to feature store

### §E.3. New JSON Schemas

Under `/specs/`:

1. `investigation-case-schema.json` — case shape
2. `investigation-case-lifecycle-schema.json` — state machine
3. `investigation-evidence-schema.json` — evidence types
4. `investigation-chain-of-custody-schema.json` — Merkle leaf shape
5. `investigation-regulator-surface-schema.json` — per-pack template shape

### §E.4. New runbooks

11 new runbooks (listed above); each per §2 runbook rigor.

### §E.5. New CI lanes

- `oya-governance-investigation-case-substrate-emission` — verifies every detection-emitting µservice routes signals into case substrate
- `oya-governance-investigation-cedar-pii-gating` — verifies per-case PII access is Cedar-gated
- `oya-governance-investigation-chain-of-custody-merkle` — verifies chain-of-custody Merkle-sealed
- `oya-governance-investigation-per-tenant-case-ownership` — verifies tenant scoping
- `oya-governance-investigation-per-pack-regulator-surface` — verifies per-pack regulator templates wired
- `oya-governance-investigation-appeal-sla` — verifies per-pack appeal SLA met
- `oya-governance-investigation-analyst-label-feedback-loop` — verifies feedback bridge active
- `oya-governance-investigation-triage-priority-cadence` — verifies P0..P4 SLA met
- `oya-governance-investigation-escalation-routing-coherence` — verifies escalation per-pack
- Aggregate: `oya-governance-investigation-case-management`

### §E.6. Per-µservice extensions (consumers)

Every µservice serving any detection-emitting or adverse-action-
driving surface updates:

- `compliance.md §investigation-binding` — per row 52 of §3.2.1
  ADR-adherence matrix
- `manifest.json:investigation_case_signal_routing` — Kafka topic
  + signal-class
- `manifest.json:investigation_appeal_surface_uri` — per-µservice
  appeal surface

### §E.7. Per-µservice ops-dashboard-control-center panels

Per cross-reference, `microservices/ops-dashboard-control-center/`
ships investigation panels:

- Per-investigator case-load view
- Per-tenant case queue
- Per-priority case queue
- Per-pack regulator emission dashboard
- Appeal adjudication SLA dashboard
- Chain-of-custody Merkle verification panel
- Per-case PII access audit panel

### §E.8. Vendor selection rationale

#### §E.8.1. Case-management runtime: in-house Rust + Postgres

Selected because:
- Per ADR-0211 in-house tech stack preference
- Postgres + Apache AGE already deployed for detection graph store
  (per ADR-0307 §D-6)
- Per-pack regulator templates are oyatie-domain-specific (not
  available out-of-box from commercial SOAR)

#### §E.8.2. Chain-of-custody public-witness: Sigstore Rekor

Selected because:
- Open-source + transparency log standard
- Per ADR-0028 §D-3 public-witness option
- ~250M+ entries logged per Rekor 2024 disclosure; scale-proven
- Free-tier; oyatie pays for self-hosted Rekor mirror at Tier-2 cells

#### §E.8.3. Per-pack regulator wire-format adapters

Per-regulator commercial / open-source adapter:
- NCMEC CyberTipline: official NCMEC API per 18 USC §2258A
- FinCEN BSA E-Filing: official FinCEN API
- OFAC reporting portal: official OFAC API
- HIPAA OCR Breach Portal: official OCR API
- GDPR DPA per-Member-State portals: per-DPA APIs
- DSA Transparency Database: official EU DSA API
- EU AI Act Art. 73: official EU AI Office API (post-2026)

## §F. Migration

### §F.1. Wave-3-D rollout sequencing

1. **2026-05-20 to 2026-06-15.** ADR-0310 + companion ADRs accepted.
2. **2026-06-15 to 2026-08-15.** Substrate scaffold + crate skeletons
   + Cedar policy fragments + regulator-template registry stubbed;
   per-µservice integration points stubbed.
3. **2026-08-15 to 2026-09-15.** Per-pack regulator templates
   authored; investigator pool recruitment + per-pack expertise
   training; appeal-mechanism wired for ADR-0308 + ADR-0309 surfaces.
4. **2026-09-15 to 2026-10-15.** Chain-of-custody Merkle-tree
   integration with ADR-0028 audit-chain; Sigstore Rekor mirror
   deployed; per-pack regulator emission soak.
5. **2026-10-15.** CI lanes promote to BLOCKER.
6. **2026-10-15 onwards.** Continuous: per-case investigation,
   appeal adjudication, regulator emission, analyst-label feedback
   to detection substrate.

### §F.2. Per-µservice migration playbook

For each detection-emitting + adverse-action-driving µservice:

1. **Audit current case-handling.** Inventory in-µservice case
   queues / appeal mechanisms / regulator emissions.
2. **Route signals to substrate.** Per Kafka topic
   `audit.investigation.case.opened`.
3. **Wire appeal surface.** Per-µservice appeal surface routes to
   substrate appeal-adjudication.
4. **Wire regulator emission.** Per-pack templates evaluated for
   per-µservice events.
5. **Update compliance.md.** Per row 52.

### §F.3. Per-cell rollout pattern

- Tier-0 edge cells: emit case-opening events; no case-management runtime
- Tier-1 bootstrap cell: emit only
- Tier-2 control plane cells: full case-management runtime
- Tier-3 data plane cells: per-tenant evidence storage (per-pack residency)

### §F.4. What is NOT migrated

- Detection runtime (ADR-0307) — separate substrate
- ML model lifecycle (ADR-0308) — separate substrate
- Fairness invariants (ADR-0309) — separate substrate
- Cedar policy evaluation (ADR-0243) — separate substrate
- Audit-chain Merkle-seal (ADR-0028) — separate substrate

### §F.5. Rollback path

Per ADR-0294 anomaly-rollback applied to case-management:

1. Per-template rollback: per-pack template versions retained;
   per-template rollback via runbook
2. Per-substrate-runtime rollback: Postgres restore-to-snapshot +
   Kafka offset rewind
3. Per-case lifecycle rollback: NOT permitted (state machine is
   monotonic); errors handled via new case opened referencing
   prior

Emergency: per-pack `REGULATOR_EMISSION_BYPASS=1` env flag NOT
permitted (regulator emission is mandatory).

## §G. References

### §G.1. Hyperscaler precedents

- **Splunk Enterprise Security + Splunk SOAR** — splunk.com/en_us/products/enterprise-security
- **IBM QRadar SIEM + IBM SOAR (formerly Resilient)** — ibm.com/products/qradar-soar
- **Palo Alto Cortex XSOAR (formerly Demisto)** — paloaltonetworks.com/cortex/cortex-xsoar
- **Google Chronicle SOAR (formerly Siemplify)** — cloud.google.com/chronicle-security-operations
- **Salesforce Service Cloud + Salesforce Trust & Safety** — salesforce.com
- **YouTube Trust & Safety + Meta Trust & Safety + TikTok Trust & Safety transparency reports**
- **Stripe Trust & Safety + Stripe Disputes** — stripe.com/docs/disputes
- **Toss riskOps case-management** — Toss 2024 Tech Conference
- **AWS Security Hub + Amazon Detective + AWS Audit Manager** — aws.amazon.com/security-hub
- **Microsoft Sentinel + Microsoft Defender XDR** — microsoft.com/security
- **NCMEC PhotoDNA + NCMEC CyberTipline** — missingkids.org
- **GIFCT hash-sharing infrastructure** — gifct.org
- **FinCEN BSA E-Filing** — bsaefiling.fincen.treas.gov
- **OFAC reporting portal** — sanctionssearch.ofac.treas.gov
- **HHS OCR Breach Notification Portal** — hhs.gov/hipaa
- **EU DSA Transparency Database** — transparency.dsa.ec.europa.eu
- **EU AI Office post-market monitoring infrastructure** — digital-strategy.ec.europa.eu/en/policies/european-ai-office
- **Sigstore Rekor transparency log** — sigstore.dev
- **Bitcoin OP_RETURN public-witness** — bitcoin.org
- **Certificate Transparency log infrastructure** — certificate.transparency.dev

### §G.2. Standards + RFCs

- **NIST AI Risk Management Framework 1.0** — nist.gov/itl/ai-risk-management-framework
- **ISO/IEC 42001:2023** — AI management systems
- **ISO/IEC 27001 + ISO/IEC 27002** — information security management
- **ISO/IEC 27035** — information security incident management
- **NIST SP 800-61** — Computer Security Incident Handling Guide
- **Federal Rules of Evidence (FRE) Rule 901** — authentication of evidence
- **EU Council Regulation 910/2014 (eIDAS)** — qualified electronic signatures
- **KR Electronic Signature Act**
- **JP Electronic Signatures Act**
- **W3C Verifiable Credentials Data Model 1.1** — w3.org/TR/vc-data-model

### §G.3. Legal + compliance

- **HIPAA Privacy Rule + Breach Notification Rule** — 45 CFR §164.400-414
- **GDPR (Regulation 2016/679)** — Articles 22, 33-34 (breach notification), 12 (timing), 86 (right to meaningful explanation)
- **NIS2 Directive 2022/2555** — incident reporting
- **EU AI Act (Regulation 2024/1689)** — Articles 17, 27, 73, 86; Annex III
- **EU Digital Services Act (Regulation 2022/2065)** — Articles 16, 17 (statement-of-reasons), 22 (trusted flagger), 27 (transparency)
- **EU Race Equality Directive 2000/43/EC** + **EU Employment Equality Directive 2000/78/EC**
- **EU Charter of Fundamental Rights Art. 21** — non-discrimination
- **KR-PIPA (Personal Information Protection Act)** — Art. 39 breach notification
- **KR-FSC + KR-FSS** — financial regulator surfaces
- **KR Financial Consumer Protection Act Art. 30** — 시행 2021-03-25
- **JP APPI** — Personal Information Protection Act
- **JP-FIU + JP-METI** — financial intelligence + export control
- **18 USC §2258A** — NCMEC reporting obligations
- **FinCEN BSA + SAR thresholds** — 31 CFR §1010, §1020-1029
- **OFAC sanctions reporting** — 31 CFR §501-598
- **ECOA + Regulation B (12 CFR §1002)** — adverse-action notice
- **CFPB Circular 2022-03** — specific reasons for AI-driven credit decisions
- **Fair Housing Act + HUD disparate-impact rule** — 24 CFR §100.500
- **NY DFS Cybersecurity Regulation 23-NYCRR-500** — 72h breach notification
- **NY AEDT Local Law 144 (2023)** — annual bias audit + public notice
- **CCPA + CPRA** — California Consumer Privacy Act + Privacy Rights Act
- **Colorado AI Act (SB 24-205, 2024)** — high-risk AI documentation; effective 2026-02-01
- **California Civil Rights Department ADS regulation (proposed 2024)**
- **California AB 2013 + AB 3030 + AB 2655 (2024)**
- **Illinois BIPA (740 ILCS 14)**
- **Texas CUBI**
- **Washington My Health My Data Act (RCW 19.373)**
- **Utah AI Disclosure Bill (SB 149, 2024)**
- **DSA Art. 17** — statement-of-reasons
- **CSAM Reg (EU proposal 2022/0155)** — pending
- **FedRAMP-High continuous-monitoring framework**
- **NIST SP 800-53 Rev 5** — security and privacy controls
- **HUD Fair Housing Act** — 42 USC §3601-3619
- **EEOC Title VII** — 42 USC §2000e
- **FBI IC3** — Internet Crime Complaint Center
- **DOJ Civil Rights Division** — ECOA + Fair Housing Act enforcement
- **Australian Privacy Act 1988 + 2024 amendments**
- **Privacy Commissioner of Canada PIPEDA**

### §G.4. Internal portfolio ADRs

- **ADR-0028** — audit-chain Merkle-sealed
- **ADR-0099** — data-class registry
- **ADR-0105** — 13-layer canonical enum
- **ADR-0131** — per-microservice flat layout
- **ADR-0132** — no-grouping microservice rule
- **ADR-0140** — Cedar policy enforcement
- **ADR-0212** — buildability doctrine
- **ADR-0240** — sovereign-cloud per-regional pack
- **ADR-0242** — oyatie-is-a-tenant doctrine
- **ADR-0243** — Cedar as universal gate
- **ADR-0244** — tenant as universal scoping primitive
- **ADR-0245** — substrate vs product layering
- **ADR-0246** — policy-engine substrate promotion
- **ADR-0248** — Amazon-shape cellular architecture
- **ADR-0250** — build-ahead-of-certification doctrine
- **ADR-0251** — compliance-pack cell certification levels
- **ADR-0258** — API versioning SemVer policy
- **ADR-0263** — observability emission contract
- **ADR-0276** — backup portability GDPR Art. 20
- **ADR-0293** — Foundry meta-trust-root
- **ADR-0294** — Cedar fragment soak + anomaly-rollback
- **ADR-0295** — bootstrap CI SPIFFE + kill-switch
- **ADR-0296** — library-first credential sidecar
- **ADR-0297** — abuse-defence baseline
- **ADR-0298** — emergency-services critical-path exemption
- **ADR-0307** — detection substrate (this bundle)
- **ADR-0308** — ML model lifecycle (this bundle)
- **ADR-0309** — detection fairness + civil-rights compliance (this bundle)

### §G.5. Standards docs

- `docs/standards/documentation-rigor.md` §3.2.6 — DRMP baseline
- `docs/standards/fintech-compliance.md` — KR-FSS / PCI-DSS / ECOA / AML
- `docs/standards/doc-style.md` — Diátaxis + RFC-2119
- `docs/STANDARDS-AND-TEMPLATES.md` — catalog
- `docs/standards/event-schema-versioning-canonical.md`

### §G.6. Auto-memory feedback (related)

- `feedback_quality_performance_scalability_bar` — hyperscaler rigor
- `feedback_clean_architecture_requirements` — inward-only + single-concern
- `feedback_no_silent_regression` — public-contract protection
- `feedback_autonomous_implementation_artifacts` — intern-buildable case-management
- `feedback_oyatie_is_a_tenant_doctrine` — substrate applies to oyatie internal investigations
- `feedback_cedar_as_universal_gate` — Cedar gates per-case PII access
- `feedback_compliance_pack_primitive` — per-pack regulator templates
- `feedback_substrate_vs_product_layering` — case-management is substrate
- `feedback_build_ahead_of_certification` — day-one regulator emission cadence
- `feedback_naming_justification` — every primitive justified per v4 BNF + 13-layer-enum

## §H. Change log

- **2026-05-20** — Initial draft authored as part of keystone-bundle 2026-05-20 Wave-3-D detection-cluster batch (ADR-0307..0310). Bundled with ADR-0307 (detection substrate), ADR-0308 (ML lifecycle), ADR-0309 (fairness audit) as the **drmp-detection-cluster** keystone batch. Covers triage → investigation → escalation/dismissal → feedback workflow + Cedar-gated PII access + chain-of-custody Merkle-sealed (ADR-0028) + per-pack regulator-facing surface emission (HIPAA, GDPR, NIS2, DSA, EU AI Act, NCMEC, FinCEN, OFAC, KR-PIPA, NY AEDT, etc.) + per-tenant investigation case ownership. Enforcement advisory-until-2026-10-15-blocker-thereafter.
