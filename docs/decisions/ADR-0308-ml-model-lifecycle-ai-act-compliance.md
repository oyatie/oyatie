---
id: ADR-0308
status: Rejected
date: 2026-05-20
owners:
  - council-architecture
  - council-security
  - council-privacy
  - council-product
  - council-legal
  - council-data
  - council-ml
  - ops-sre-reliability
  - ops-security
  - ops-trust-and-safety
  - ops-compliance
  - ops-ml-platform
  - axis-detection
  - axis-ml-lifecycle
  - axis-fairness
  - axis-investigation
  - axis-feature-store
supersedes: []
amends: []
superseded_by: []
related:
  - ADR-0028-audit-chain-merkle-sealed.md
  - ADR-0063-doc-coverage-enforced.md
  - ADR-0099-data-class-registry.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0130-agentic-slo-gated-promotion.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0132-no-grouping-microservice-rule.md
  - ADR-0140-cedar-policy-enforcement.md
  - ADR-0145-inter-microservice-communication-reform.md
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
  - ADR-0252-hlc-default-truetime-tier.md
  - ADR-0253-network-topology-edge-service-mesh.md
  - ADR-0254-deployment-model-spectrum.md
  - ADR-0255-intelligence-two-layer-substrate.md
  - ADR-0258-api-versioning-semver-policy.md
  - ADR-0263-observability-emission-contract.md
  - ADR-0276-backup-portability-gdpr-art-20.md
  - ADR-0280-substrate-of-substrate-dependency.md
  - ADR-0293-governance-meta-trust-root.md
  - ADR-0294-cedar-fragment-soak-anomaly-rollback.md
  - ADR-0295-bootstrap-ci-spiffe-kill-switch.md
  - ADR-0296-library-first-credential-sidecar.md
  - ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md
  - ADR-0298-emergency-services-critical-path-exemption.md
  - ADR-0307-detection-substrate-streaming-batch.md
  - ADR-0309-detection-fairness-audit-civil-rights.md
  - ADR-0310-investigation-case-management.md
related_specs:
  - /specs/microservices/detection.json
  - /specs/ml-model-card-schema.json
  - /specs/ml-model-lifecycle-schema.json
  - /specs/ml-drift-detection-schema.json
  - /specs/ml-ab-test-schema.json
  - /specs/ml-rollback-schema.json
  - /specs/ml-appeal-schema.json
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
keystone_position: drmp-ml-model-lifecycle
purpose: >
  Establish the ML Model Lifecycle for the Detection Substrate per
  EU AI Act (Regulation 2024/1689) + NIST AI Risk Management Framework
  1.0 + ISO/IEC 42001:2023 AI management systems. Codifies the
  eight-stage lifecycle defined in documentation-rigor.md §3.2.6.E:
  training (per-tenant data residency + cross-tenant training consent),
  validation (bias audit + fair-lending), A/B testing (champion-
  challenger, shadow-then-canary-then-full), drift detection (feature
  + label + concept drift; Arize / Fiddler / WhyLabs / Evidently AI),
  fairness re-audit (quarterly cadence per protected class),
  model versioning (SemVer per ADR-0258 + Google Model Card template),
  rollback (per ADR-0294 anomaly-rollback + EU AI Act Art. 73 24h
  serious-incident reporting), appeal mechanism (GDPR Art. 22 +
  EU AI Act Art. 86 + NY AEDT Local Law 144 (2023) + ECOA Reg B).
  The lifecycle is the regulator-facing surface — without it, oyatie's
  detection substrate (ADR-0307) cannot ship under EU AI Act high-
  risk classification (Annex III: biometric ID, credit scoring,
  fraud detection, content moderation, employment ranking all fall
  inside high-risk). Build-ahead-of-certification per ADR-0250
  requires day-one compliance — ML lifecycle MUST be in place before
  detection substrate goes GA.
enforcement_status: advisory-until-2026-09-15-blocker-thereafter
enforced_by:
  - cloud-ci/Rust gate packet ml-model-card-present-per-model
  - cloud-ci/Rust gate packet ml-validation-bias-audit-present
  - cloud-ci/Rust gate packet ml-ab-test-champion-challenger-coherence
  - cloud-ci/Rust gate packet ml-drift-detection-daily-cadence
  - cloud-ci/Rust gate packet ml-fairness-quarterly-cadence
  - cloud-ci/Rust gate packet ml-model-semver-conformance
  - cloud-ci/Rust gate packet ml-rollback-runbook-present
  - cloud-ci/Rust gate packet ml-appeal-mechanism-coverage
  - cloud-ci/Rust gate packet ml-eu-ai-act-art73-24h-incident-reporting
  - cloud-ci/Rust gate packet ml-ny-aedt-bias-audit-public-notice
naming_justifications:
  - name: oya-shared-ml-lifecycle
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.ml-lifecycle
    justification: >
      Per ADR-0105 13-layer canonical enum row 5 (shared-substrate),
      the crate that exposes the ML lifecycle trait (train +
      validate + AB-test + drift-detect + fairness-re-audit + version
      + rollback + appeal) across every model-serving substrate
      µservice belongs at the shared layer. Single-concern naming
      per ADR-0131 + ADR-0132; not bundled under "ml-suite".
  - name: oya-shared-ml-training
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.ml-training
    justification: >
      Per-tenant-residency-aware training pipeline crate; consumes
      feature store offline tier (per ADR-0307 §D-3.5); enforces
      ADR-0244 tenant scoping at training-data-fetch time; per
      ADR-0099 data-class registry, refuses to train on PII-classed
      features without explicit per-pack consent. Single-concern.
  - name: oya-shared-ml-validation
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.ml-validation
    justification: >
      Bias-audit + fair-lending validation crate; runs on every
      pre-promotion model; IBM AI Fairness 360 + Microsoft Fairlearn
      + Google What-If Tool compatible. Single-concern; not bundled
      with training crate so validation can SemVer-evolve independently
      (e.g., new fairness metrics added as minor bumps without
      forcing training-crate major bump).
  - name: oya-shared-ml-ab-test
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.ml-ab-test
    justification: >
      Champion-challenger A/B testing crate; shadow-mode → canary →
      full rollout pattern; per-jurisdiction overlay. Single-concern;
      Statsig / Optimizely / LaunchDarkly Experimentation are the
      hyperscaler precedents.
  - name: oya-shared-ml-drift-detection
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.ml-drift-detection
    justification: >
      Daily-cadence drift detection crate (feature drift + label
      drift + concept drift); Arize AI / Fiddler / WhyLabs / Evidently
      AI are the hyperscaler precedents. Single-concern; alerts to
      DetectionDriftAlertTriggered audit event per ADR-0307 §C.2.
  - name: oya-shared-ml-fairness-audit
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.ml-fairness-audit
    justification: >
      Quarterly fairness re-audit crate per protected class per
      jurisdiction (US: ECOA + Fair Housing; EU: AI Act Annex III +
      Charter Art. 21; KR: Financial Consumer Protection Act Art. 30;
      JP: APPI). Single-concern; companion to oya-shared-ml-validation
      (validation runs pre-promotion; fairness-audit runs quarterly
      against in-production model).
  - name: oya-shared-ml-versioning
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.ml-versioning
    justification: >
      Per-model SemVer per ADR-0258 + Google Model Card template;
      MLflow / Weights & Biases / Hugging Face Hub compatible.
      Single-concern; per-version reproducibility-from-Iceberg-
      training-snapshot enforced.
  - name: oya-shared-ml-rollback
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.ml-rollback
    justification: >
      Per-model rollback crate per ADR-0294 anomaly-rollback applied
      to ML models; emits DetectionModelRolledBack audit event per
      ADR-0307 §C.2 + ADR-0263. Single-concern; integrates with
      EU AI Act Art. 73 24h serious-incident reporting.
  - name: oya-shared-ml-appeal-mechanism
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.ml-appeal-mechanism
    justification: >
      Per GDPR Art. 22 + EU AI Act Art. 86 + ECOA Reg B + NY AEDT
      Local Law 144 (2023) right-to-meaningful-explanation + human-
      reviewer SLA. Single-concern; integrates with ADR-0310
      investigation case-management for human-reviewer routing.
  - name: oya-governance-ml-model-card-present
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.ml-model-card-present
    justification: >
      CI fitness lane per ADR-0212; verifies every deployed ML
      model has a model card matching ml-model-card-schema.json
      (per Google Model Card template).
  - name: oya-governance-ml-fairness-quarterly-cadence
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.ml-fairness-quarterly-cadence
    justification: >
      CI fitness lane per ADR-0212; verifies quarterly fairness
      re-audit emitted within ±15-day window per quarter per
      protected-class per jurisdiction. Cross-references ADR-0309.
  - name: oya-governance-ml-drift-detection-daily
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.ml-drift-detection-daily
    justification: >
      CI fitness lane per ADR-0212; verifies daily-cadence drift
      detection runs + alerts emitted; ≤24h staleness on any
      production model.
  - name: oya-governance-ml-appeal-mechanism-coverage
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.ml-appeal-mechanism-coverage
    justification: >
      CI fitness lane per ADR-0212; verifies every adverse-action
      surface has appeal-mechanism-link populated in
      DetectionSignalEmitted payload per ADR-0307 §D-5.3.
  - name: MLModelTrainingCompleted
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: MLModel.TrainingCompleted
    justification: >
      Emitted when a model finishes training; carries model_id,
      version, training-data-snapshot-uri, training-time-window,
      hyperparameters. Registered per ADR-0263.
  - name: MLModelValidationPassed
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: MLModel.ValidationPassed
    justification: >
      Emitted when validation (bias audit + fair-lending +
      held-out-set evaluation) passes thresholds; required pre-promotion.
  - name: MLModelValidationFailed
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: MLModel.ValidationFailed
    justification: >
      Emitted when validation fails; carries per-metric breakdown
      for analyst diagnosis. Required for audit trail per EU AI Act
      Art. 18.
  - name: MLModelABTestStarted
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: MLModel.ABTestStarted
    justification: >
      Emitted on champion-challenger A/B start; carries
      challenger_model_id + champion_model_id + traffic-split
      schedule + per-jurisdiction overlay.
  - name: MLModelABTestCompleted
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: MLModel.ABTestCompleted
    justification: >
      Emitted on A/B test completion (challenger-wins or
      challenger-loses); carries per-metric comparison + statistical-
      significance + adverse-action-rate per protected class.
  - name: MLModelDriftDetected
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: MLModel.DriftDetected
    justification: >
      Emitted by daily drift-detection job; carries drift_type
      (feature / label / concept), magnitude, affected_feature_ids.
  - name: MLModelFairnessReportEmitted
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: MLModel.FairnessReportEmitted
    justification: >
      Quarterly fairness re-audit report emission per ADR-0309;
      surfaced to regulator-facing dashboard per ADR-0310.
  - name: MLModelAppealFiled
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: MLModel.AppealFiled
    justification: >
      Emitted when an adverse-action-affected party files an appeal;
      starts the per-pack human-reviewer SLA clock per GDPR Art. 22 +
      EU AI Act Art. 86.
  - name: MLModelAppealAdjudicated
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: MLModel.AppealAdjudicated
    justification: >
      Emitted when human reviewer adjudicates an appeal; carries
      verdict (uphold / reverse / partial), reviewer_id,
      reason-narrative. Surfaces to transparency report per DSA Art. 17.
  - name: MLModelSeriousIncidentReported
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: MLModel.SeriousIncidentReported
    justification: >
      Emitted within 24h of a "serious incident" per EU AI Act
      Art. 73 (incident affecting fundamental rights of natural
      persons); routed to regulator-facing surface per ADR-0310.
  - name: ml-model-card-schema.json
    layer: N/A (JSON Schema spec)
    bnf_segments: spec.ml-model-card-schema
    justification: >
      JSON Schema declaring the Google Model Card template shape
      per arxiv.org/abs/1810.03993; every deployed ML model MUST
      validate against this schema per the §3.2.2 consistency
      invariant.
---

# ADR-0308: ML Model Lifecycle — EU AI Act + NIST AI RMF + ISO/IEC 42001 Compliance

## Status

Proposed — 2026-05-20.

Bundled with the keystone-bundle 2026-05-20 foundational doctrine
synthesis as the **drmp-ml-model-lifecycle** keystone, closing the
gap identified in `docs/standards/documentation-rigor.md` §3.2.6.E
(eight-stage lifecycle table; binding ADR called out as ADR-0308).
This ADR is the binding ADR row 50 of the §3.2.1 ADR-adherence
matrix cites.

Enforcement is `advisory-until-2026-09-15-blocker-thereafter`. The
ML lifecycle accepts in text immediately; the CI lanes that gate
per-model card-present + validation-passed + drift-detection-daily
+ fairness-quarterly + appeal-coverage promote to BLOCKER on
2026-09-15 to coincide with the ADR-0307 detection substrate GA
promotion (no detection substrate goes GA without a complete ML
lifecycle running underneath).

## Date

2026-05-20.

## Context

### §A. Why ML model lifecycle is a substrate-level commitment, not a per-model afterthought

Mature ML organizations treat model lifecycle as a *first-class
substrate primitive* — not as a per-team or per-model afterthought.
The pattern is unambiguous across the named industry references:

- **Google Model Cards.** Google's foundational 2018 paper "Model
  Cards for Model Reporting" (arxiv.org/abs/1810.03993, Mitchell
  et al., FAccT 2019) established the model-card template; Google's
  Vertex AI Model Registry + Vertex AI Workbench enforce model
  cards as a deployment gate. Per Google's published responsible-AI
  practices (ai.google/responsibility), model cards are mandatory
  for any model serving production traffic at Google.
- **Microsoft Datasheets for Datasets.** Microsoft Research 2018
  paper (Gebru et al., FAccT 2019) extended the model-card concept
  to training data; Azure Machine Learning Responsible AI dashboard
  ships datasheets for datasets alongside model cards.
- **Meta AI System Cards.** Meta's 2022 "System Cards" approach
  documents not just the model but the end-to-end system (feature
  pipeline + model + downstream actions). Per Meta's published
  system-card library (ai.meta.com/tools/system-cards), every
  consumer-facing AI system at Meta has a system card.
- **OpenAI Model + System Cards.** OpenAI publishes GPT-4o + GPT-4.5
  + o1 + o3 system cards (openai.com/research) with capability
  evaluations + safety mitigations + jailbreak coverage. The
  per-model card is a regulator-facing surface (per EU AI Act
  Art. 53 GPAI model documentation requirements).
- **Anthropic Model Reports.** Anthropic publishes Claude model
  reports (anthropic.com/news) covering training methodology,
  evaluation benchmarks, Constitutional AI specifics, and red-team
  results.
- **Stripe Radar Model Lifecycle.** Per the 2024 Stripe Sessions
  keynote, Radar runs continuous champion-challenger A/B against
  thousands of model variants; production model rollback cadence
  ≤15min on detected drift; quarterly fairness audit cadence
  enforced for credit-scoring sub-models per US ECOA + Reg B.
- **Microsoft Fairlearn.** github.com/fairlearn/fairlearn —
  open-source fairness assessment library; per-class TPR/FPR
  computation + disparate-impact ratio + equalized-odds metric;
  used by Microsoft Azure ML Responsible AI dashboard.
- **IBM AI Fairness 360.** github.com/Trusted-AI/AIF360 —
  open-source bias mitigation; ~70 fairness metrics + ~10 bias
  mitigation algorithms.
- **Google What-If Tool.** github.com/PAIR-code/what-if-tool —
  interactive ML investigation for fairness analysis.
- **Arize AI / Fiddler AI / WhyLabs / Evidently AI.** The model-
  observability vendor stack — per their 2024 product docs, all
  four ship feature/label/concept drift detection with daily
  cadence as the operating baseline.

The corollary: **every ML model serving production traffic in oyatie
MUST live inside the substrate-managed lifecycle, not a per-µservice
or per-team workflow.** A µservice authoring a model outside the
substrate skips model-card gating, skips bias audit, skips drift
detection, skips appeal mechanism — and ships an EU AI Act
violation. The substrate shape closes the gap by hoisting the
lifecycle to a shared layer; per-µservice models register with the
substrate (per §E.1).

### §A.1. EU AI Act + NIST AI RMF + ISO/IEC 42001 — the regulator surface

The keystone bundle 2026-05-20 commits to **build-ahead-of-
certification** (per ADR-0250); the detection substrate (ADR-0307)
covers fraud, biometric ID (KYC liveness + face-match), credit
scoring (payments + KYB), content moderation (CSAM + terrorism +
hate speech), employment ranking (insider risk + JIT-access
attestation). All of these fall inside EU AI Act Annex III's
**high-risk** classification:

- **Annex III §1 (biometric identification + categorization)** —
  synthetic-identity detection's face-match + liveness components
- **Annex III §3 (employment, workers management, access to
  self-employment)** — insider-risk detection's employee scoring
- **Annex III §5(b) (creditworthiness + credit scoring)** — payments
  + AML detection's per-user risk score
- **Annex III §5(c) (life and health insurance, pricing)** — when
  the platform serves insurance products
- **Annex III §6 (law enforcement)** — when the platform serves
  government tenants per ADR-0251 FedRAMP-High pack
- **Annex III §7 (migration, asylum, border control)** — when the
  platform serves government tenants
- **Annex III §8 (administration of justice, democratic processes)**
  — content-abuse + misinformation detection

High-risk classification triggers (per EU AI Act Articles
6 + 8 + 9-15):

- **Article 9 (risk management system)** — mandatory pre-deployment
  risk assessment + post-deployment monitoring
- **Article 10 (data + data governance)** — training-data quality +
  bias mitigation + per-tenant residency
- **Article 11 (technical documentation)** — model card + system
  card + lifecycle log
- **Article 12 (record-keeping + logs)** — audit-event-class emission
  per ADR-0263 (model-lifecycle events registered §G.1)
- **Article 13 (transparency)** — explainability floor per ADR-0309
- **Article 14 (human oversight)** — human-in-loop for adverse
  actions per ADR-0310
- **Article 15 (accuracy + robustness + cybersecurity)** — per-class
  TPR/FPR + adversarial-robustness benchmarks
- **Article 17 (quality management system)** — ISO/IEC 42001:2023
  certified
- **Article 18 (documentation retention)** — 10-year model+data
  retention; per ADR-0028 audit-chain Merkle-sealed + ADR-0276
  backup-portability
- **Article 27 (fundamental rights impact assessment)** — per-model
  FRIA emitted to regulator-facing surface
- **Article 73 (serious incident reporting)** — 24h to authorities
  on any serious incident
- **Article 86 (right to meaningful explanation)** — appeal
  mechanism per §B-8

**NIST AI Risk Management Framework 1.0** (nist.gov/itl/ai-risk-management-framework,
released 2023-01-26) requires the four-function framework: **Govern**
(governance structures), **Map** (context + risks), **Measure**
(risks + impacts), **Manage** (risk-response). The ADR-0308
lifecycle satisfies all four:

- Govern: model-lifecycle policy + per-model owners + per-
  jurisdiction overlay
- Map: training-data context + risk-tier classification + fairness-
  audit scope
- Measure: validation metrics + drift detection + fairness re-audit
  + appeal-rate
- Manage: rollback runbook + champion-challenger A/B + serious-
  incident-reporting workflow

**ISO/IEC 42001:2023** (iso.org/standard/81230.html, released
2023-12) is the international AI management system standard;
oyatie targets ISO/IEC 42001 certification per ADR-0250 day-one
certification commitment. The §B lifecycle satisfies 42001's
core controls (Annex A.5 through A.10).

### §A.2. NY AEDT Local Law 144 (2023) + ECOA Reg B + state-AG laws

US state-level + federal civil-rights laws layer onto the EU AI Act
foundation:

- **NY AEDT Local Law 144 (2023)** (rules.cityofnewyork.us/wp-content/uploads/2023/04/DCWP-NOA-for-Automated-Employment-Decision-Tools-2.pdf,
  effective 2023-07-05) — requires annual bias audit + public
  audit-results notice + candidate notification of AI use for any
  AI-driven employment decision tool. Applies to insider-risk +
  employee-ranking detection.
- **ECOA + Regulation B** (12 CFR §1002, codified 1974, amended
  through 2024) — adverse-action notice with **specific reasons**
  for credit denial within 30 days; applies to payments + AML
  detection's credit-impact decisions.
- **Fair Housing Act (FHA) + HUD's disparate-impact rule** (42 USC
  §3601-3619 + 24 CFR §100.500, finalized 2013, reaffirmed 2023) —
  prohibits housing decisions with disparate impact regardless of
  intent; applies if oyatie serves housing-tenants (e.g., property-
  management workflows).
- **California's Generative AI Training Data Transparency Act** (SB
  942, signed 2024-09) — disclosure of training data sources for
  generative AI; applies to per-jurisdiction overlay if California
  packs activate generative components.
- **Colorado Artificial Intelligence Act** (CO SB 24-205, signed
  2024-05; effective 2026-02-01) — high-risk AI documentation +
  consumer disclosure; oyatie's effective date precedes 2026-02-01.
- **Illinois Biometric Information Privacy Act (BIPA)** (740 ILCS 14,
  2008) — biometric collection consent; applies to synthetic-
  identity detection's face-match.
- **Texas Capture or Use of Biometric Identifier Act (CUBI)** —
  similar to BIPA.
- **Washington My Health My Data Act** (RCW 19.373, effective
  2024-03-31) — health data consent; applies to per-pack HIPAA
  overlays.
- **Utah AI Disclosure Bill (SB 149, 2024)** + **California AB 2013
  (2024) + AB 3030 (2024) + AB 2655 (2024)** — additional state
  AI laws.

### §A.3. KR + JP + EU + UK + AU regulator surfaces

- **KR Financial Consumer Protection Act Art. 30** (시행 2021-03-25)
  — restricts certain protected-class proxies in financial-decision
  ML.
- **KR-PIPA + KR-Credit-Information-Act** — financial-data scope.
- **KR-FSC + KR-FSS** — financial regulator surfaces.
- **JP APPI** (Personal Information Protection Act) — applies to
  per-pack JP overlays.
- **EU GDPR Art. 22** (Regulation 2016/679) — automated-decision
  rights including explanation + appeal.
- **UK Age Appropriate Design Code (AADC, 2020)** — minor-protection
  layer.
- **AU Online Safety Act 2021** + **AU Privacy Act amendments
  (2024)** — Australian regulator surfaces.

### §A.4. What this ADR explicitly does NOT do

ADR-0308 is the **ML model lifecycle**. It does NOT:

1. **Author the detection substrate runtime.** That lives in ADR-0307.
2. **Author the fairness invariants + per-jurisdiction model variants.**
   Those live in ADR-0309 (companion ADR in this batch).
3. **Author the investigation case-management workflow.** That
   lives in ADR-0310.
4. **Author per-model rule bodies or per-feature training-data
   pipelines.** Those live in per-µservice + per-family files under
   `microservices/detection/`.
5. **Replace per-µservice business logic.** The lifecycle covers ML
   models; deterministic rules (per ADR-0307 §D-4) are a separate
   primitive.
6. **Author general-purpose AI (GPAI) model lifecycle.** Per EU AI
   Act Art. 51-55, GPAI models have separate governance; oyatie's
   detection substrate models are not GPAI — they are task-specific
   high-risk per Annex III.

## Decision

### §B. Eight-stage ML model lifecycle

Establish the canonical eight-stage lifecycle for every ML model
serving production traffic in the detection substrate (per ADR-0307)
or any other product-facing AI surface:

```
                       ┌──────────────────────┐
                       │ 1. TRAINING          │
                       │  - per-tenant data   │
                       │    residency         │
                       │  - cross-tenant      │
                       │    consent           │
                       │  - Iceberg snapshot  │
                       └──────────┬───────────┘
                                  │
                                  ▼
                       ┌──────────────────────┐
                       │ 2. VALIDATION        │
                       │  - bias audit        │
                       │  - fair-lending      │
                       │  - held-out set      │
                       │  - per-class TPR/FPR │
                       └──────────┬───────────┘
                                  │
                                  ▼
                       ┌──────────────────────┐
                       │ 3. A/B TESTING       │
                       │  - champion-challeng.│
                       │  - shadow → canary   │
                       │    → full            │
                       │  - per-jurisdiction  │
                       └──────────┬───────────┘
                                  │
                                  ▼
                       ┌──────────────────────┐    ┌─────────────────────┐
                       │ 4. DRIFT DETECTION   │───▶│ 5. FAIRNESS RE-AUDIT│
                       │  - feature/label/    │    │  - quarterly cadence│
                       │    concept drift     │    │  - per-class        │
                       │  - daily cadence     │    │  - per-jurisdiction │
                       └──────────┬───────────┘    └──────────┬──────────┘
                                  │                           │
                                  └─────────┬─────────────────┘
                                            ▼
                       ┌──────────────────────────────────┐
                       │ 6. MODEL VERSIONING              │
                       │  - SemVer per ADR-0258           │
                       │  - Google Model Card             │
                       │  - reproducibility from Iceberg  │
                       └─────────┬────────────────────────┘
                                 │
                                 ▼
                       ┌──────────────────────────────────┐
                       │ 7. ROLLBACK                      │
                       │  - per ADR-0294 anomaly-rollback │
                       │  - EU AI Act Art. 73 24h         │
                       │    serious-incident reporting    │
                       └─────────┬────────────────────────┘
                                 │
                                 ▼
                       ┌──────────────────────────────────┐
                       │ 8. APPEAL MECHANISM              │
                       │  - GDPR Art. 22                  │
                       │  - EU AI Act Art. 86             │
                       │  - NY AEDT 2023                  │
                       │  - ECOA Reg B                    │
                       │  - human reviewer + SLA          │
                       └──────────────────────────────────┘
```

### §B.1. Stage 1 — Training

Per-tenant data residency honored; cross-tenant training requires
explicit per-pack-permitted consent. Training pipeline:

- **Input.** Feature store offline tier (per ADR-0307 §D-3.5); per-
  entity feature batches sampled from Iceberg-backed Parquet.
- **Per-tenant filter.** Per ADR-0244, training data filtered to
  the tenants whose `audience_type` + `compliance_packs[]` permit
  cross-tenant training. Default: per-tenant training only (the
  model trained sees only that tenant's data). Cross-tenant training
  requires `provider_credential_mode == byok_required_by_pack` OR
  explicit consent surface per pack.
- **Per-pack data class filter.** Per ADR-0099 data-class registry:
  HIPAA-pack tenants → PHI features excluded; KR-PIPA → pseudonymized;
  GDPR → pseudonymized; CCPA → opt-out-honoring; COPPA → never-trained
  on <13 users.
- **Iceberg training-data snapshot.** Per Iceberg time-travel,
  training-data-snapshot-uri captured + persisted to model card
  for reproducibility (per EU AI Act Art. 18 retention).
- **Hyperparameter logging.** Per MLflow / Weights & Biases-class
  experiment tracker; per-run hyperparameters captured.
- **Compute attestation.** Per ADR-0293 meta-trust-root + ADR-0295
  bootstrap SPIFFE, training compute attested via SPIFFE SVID;
  rejects training jobs running outside attested compute.

Audit event: `MLModelTrainingCompleted` (per ADR-0263).

### §B.2. Stage 2 — Validation

Bias audit per `docs/standards/fintech-compliance.md` + fair-lending
laws (ECOA, KR-Financial-Consumer-Protection-Act); validation
against held-out fairness slices.

- **Held-out set.** Per ADR-0309, held-out fairness slices stratified
  by protected class (race/gender/age US; KR specific classes; EU
  Charter Art. 21 protected classes). Held-out set ≥10⁴ samples per
  protected class; sourced from same Iceberg snapshot as training
  (different time window).
- **Per-class TPR/FPR.** True-positive rate + false-positive rate
  per protected class. Per §C.1.2 of ADR-0309, equity within ±2pp
  baseline; wider gaps require explicit ADR justification + regulator
  notification (under EU AI Act).
- **Disparate-impact test.** 4/5ths rule (Federal Uniform Guidelines
  on Employee Selection Procedures, 29 CFR §1607.4): selection
  rate for protected class ≥ 80% of selection rate for majority
  class. EU + KR + JP regulator equivalent floors.
- **Adversarial-robustness benchmark.** Per EU AI Act Art. 15
  cybersecurity floor; per-model adversarial examples (e.g., FGSM,
  PGD attacks for vision; HotFlip for NLP) tested + per-attack
  accuracy ≥80% baseline.
- **Calibration check.** Per Brier-score / log-loss / reliability
  diagram — predicted probabilities match observed frequencies.
- **Explainability surface.** SHAP TreeExplainer (for tree-ensembles)
  + LIME (for non-tree) — feature-importance reproducible per
  prediction.

Audit event: `MLModelValidationPassed` (or `MLModelValidationFailed`).

Validation gate: model promotes to A/B testing only if all metrics
pass; otherwise routed back to training queue with feedback.

Hyperscaler precedents: IBM AI Fairness 360 (github.com/Trusted-AI/AIF360),
Microsoft Fairlearn (github.com/fairlearn/fairlearn), Google
What-If Tool (github.com/PAIR-code/what-if-tool).

### §B.3. Stage 3 — A/B Testing

Champion-challenger pattern; shadow-mode then canary then full;
rollback per ADR-0294 anomaly-rollback.

- **Champion.** Currently-serving production model.
- **Challenger.** Newly-trained model candidate.
- **Shadow mode.** Challenger receives 100% of traffic but its
  predictions do not drive actions — only logged for comparison
  against champion. Duration: ≥7 days.
- **Canary mode.** Challenger drives actions for a small traffic
  fraction (1% → 5% → 25%) with per-jurisdiction overlay (some
  jurisdictions require slower rollout). Duration: ≥7 days per stage.
- **Full mode.** Challenger drives actions for 100% of traffic;
  champion retained as fallback for rollback.
- **Per-jurisdiction overlay.** Per EU AI Act, EU-cell rollouts may
  require regulator pre-notification; KR-FSS may require pre-
  notification for financial-decision models.

Statistical-significance threshold: per-model P-value < 0.01 on
primary metric (per family: payment-fraud → AUC + per-class FPR;
ATO → AUC + per-class FNR; etc) with sample size ≥10⁵ per cohort.

Adverse-action-rate parity: challenger's adverse-action rate per
protected class within ±2pp of champion (per ADR-0309 fairness
invariant).

Audit events: `MLModelABTestStarted`, `MLModelABTestCompleted`.

Hyperscaler precedents: Statsig (statsig.com), Optimizely
(optimizely.com), LaunchDarkly Experimentation
(launchdarkly.com/product/experimentation).

### §B.4. Stage 4 — Drift Detection

Feature drift + label drift + concept drift detection; daily
cadence; alert on threshold-crossing.

- **Feature drift.** Per-feature distribution shift (Kullback-
  Leibler divergence, Wasserstein distance, Population Stability
  Index (PSI), Kolmogorov-Smirnov test). Alert threshold: PSI > 0.2
  for any feature; ≥3 features with PSI > 0.1.
- **Label drift.** Distribution shift in ground-truth labels (from
  analyst feedback per ADR-0310 + outcome telemetry). Alert
  threshold: per-class label rate shift > 5pp over 7-day window.
- **Concept drift.** Shift in P(label|features) detected via per-
  cohort model error rate. Alert threshold: per-cohort error rate
  > 1.5× baseline for ≥3 consecutive days.
- **Daily cadence.** Drift detection batch job runs daily per
  ADR-0307 §D-2 batch pipeline.
- **Alert routing.** Drift alerts route to per-model owner +
  on-call SRE + per-µservice alerting dashboard. SEV thresholds:
  - SEV3: PSI > 0.2 on any feature → email alert
  - SEV2: PSI > 0.3 OR ≥3 features with PSI > 0.2 → PagerDuty
  - SEV1: concept drift > 2× baseline → PagerDuty + retraining-
    queue auto-trigger + champion-challenger A/B with shadow
    mode bootstrap

Audit event: `MLModelDriftDetected` + `DetectionDriftAlertTriggered`
per ADR-0307 §C.2.

Hyperscaler precedents:
- **Arize AI** (arize.com) — feature/label drift + concept drift
- **Fiddler AI** (fiddler.ai) — drift + explainability + fairness
- **WhyLabs** (whylabs.ai) — open-source whylogs + commercial
  observability
- **Evidently AI** (evidentlyai.com) — open-source library

### §B.5. Stage 5 — Fairness Re-Audit

Quarterly fairness re-audit per protected class. Companion to
Validation (which runs pre-promotion); fairness re-audit runs
quarterly against in-production model.

- **Cadence.** Quarterly within ±15-day window per quarter; CI
  lane `oya-governance-ml-fairness-quarterly-cadence` enforces.
- **Per-class TPR/FPR re-computation.** Per protected class per
  jurisdiction; same metrics as §B.2 Validation; thresholds remain
  ±2pp baseline.
- **Per-jurisdiction overlay.** Per ADR-0309, EU + US + KR + JP
  have different protected-class enumerations; per-jurisdiction
  fairness slice computed.
- **Public notice.** Per NY AEDT Local Law 144 (2023), public
  audit-results notice posted to `/transparency` surface. Per
  EU AI Act Art. 13, accessible to data subjects.
- **Regulator emission.** Per EU AI Act high-risk obligation,
  fairness report routed to regulator-facing surface per ADR-0310.
- **Trigger retraining.** Fairness audit failure (gap > ±2pp)
  triggers retraining queue + per-pack regulator notification
  per ADR-0251 cadence.

Audit event: `MLModelFairnessReportEmitted`.

### §B.6. Stage 6 — Model Versioning

Per-model SemVer per ADR-0258; model card per Google Model Card
template; per-version reproducibility.

- **SemVer.** `major.minor.patch`:
  - Major: training-data-source change OR architecture change OR
    breaking output-schema change
  - Minor: hyperparameter change OR additive feature addition
  - Patch: bug fix OR drift-correction retrain (same architecture
    + same features + same training-data window)
- **Model card.** Per Google Model Card template; matches
  `ml-model-card-schema.json`:
  - Model details (developer, version, type, license, contact)
  - Intended use (primary users, primary surfaces, out-of-scope
    uses)
  - Factors (relevant factors, evaluation factors)
  - Metrics (model performance + decision thresholds + variation
    approaches)
  - Evaluation data (datasets, motivation, preprocessing)
  - Training data (datasets, motivation, preprocessing)
  - Quantitative analyses (unitary results, intersectional results)
  - Ethical considerations
  - Caveats and recommendations
- **Reproducibility.** Per ADR-0028 audit-chain Merkle-sealed +
  per ADR-0276 backup-portability, each model version's training-
  data-snapshot-uri + hyperparameters + training-compute-attestation
  retained for ≥10 years per EU AI Act Art. 18.
- **Model registry.** MLflow / Weights & Biases / Hugging Face Hub
  compatible; per-cell registry deployment; per-tenant tenant_id
  scoping enforced.

Audit event: `MLModelDeployed` per ADR-0307 §C.2.

### §B.7. Stage 7 — Rollback

Per ADR-0294 anomaly-rollback applied to ML models; EU AI Act Art.
73 24h serious-incident reporting.

- **Trigger.** Per ADR-0294 anomaly-rollback semantics: per-pack
  SLA breach + per-metric threshold breach + serious-incident-
  signal arrival.
- **Rollback target.** Previous champion model (kept as fallback
  per §B.3 Full mode).
- **Time-to-effect.** ≤15min from rollback-decision to traffic-
  serving on previous champion.
- **EU AI Act Art. 73.** Within 24h of "serious incident" (any
  incident affecting fundamental rights of natural persons,
  including discrimination + adverse-action affecting >100 users +
  detected bias > ±5pp), report to regulator per per-pack regulator
  notification cadence (per ADR-0251 §nis2_three_stage_cadence
  + per ADR-0251 EU AI Act overlay).
- **Per-pack regulator surface.** Routed per ADR-0310 investigation
  case-management's regulator-facing panel.

Audit event: `MLModelRolledBack` per ADR-0307 §C.2 +
`MLModelSeriousIncidentReported` per this ADR.

Rollback runbook: `microservices/detection/runbooks/detection-model-rollback.md`.

Hyperscaler precedent: Stripe Radar's documented ≤15min model
rollback cadence; Google's continuous-deployment with auto-rollback
on canary failure.

### §B.8. Stage 8 — Appeal Mechanism

Per GDPR Article 22 + EU AI Act Article 86 + state-level (NY AEDT
Local Law 144 (2023); ECOA Reg B; KR Financial Consumer Protection
Act Art. 30) right-to-meaningful-explanation; appeal routes to
human reviewer.

- **Adverse action.** Per ECOA Reg B, "adverse action" = any
  decision denying / restricting / pricing-disadvantaging the
  affected party. Includes: payment decline, account lock, content
  removal, KYC refusal, KYB refusal, employment-decision
  recommendation.
- **Appeal route.** Affected party receives:
  - Adverse-action notice within 30 days (per ECOA Reg B)
  - Specific reasons (per ECOA Reg B): top-5 feature-importance
    values from SHAP/LIME per §B.2 Validation explainability surface
  - Appeal mechanism link (per GDPR Art. 22 + EU AI Act Art. 86)
- **Human reviewer.** Per ADR-0310 case-management workflow;
  appeal routed to human reviewer with per-pack SLA:
  - Default SLA: ≤30 days substantive review
  - EU AI Act high-risk: ≤30 days
  - GDPR Art. 12 generally: 1 month, extendable +2 months on
    complexity
  - NY AEDT: per-public-notice cadence
- **Outcome.** Reviewer verdict: uphold (adverse action stays) /
  reverse (adverse action reversed; per ADR-0307 §D-7 feedback
  loop updates feature store labels) / partial.

Audit events: `MLModelAppealFiled`, `MLModelAppealAdjudicated`.

Per-pack regulator emission: per ADR-0310 regulator-facing surface
emits aggregated appeal stats quarterly (per DSA Art. 17
transparency report; per EU AI Act Art. 86 right-to-explanation
audit).

## §C. Consequences

The 6 engineering-rigor dimensions per documentation-rigor.md §1.2:

### §C.1. Maintainability dimension

The ML lifecycle surface is concentrated in single-concern crates
under `crates/oya-shared-ml-*/` (training, validation, ab-test,
drift-detection, fairness-audit, versioning, rollback, appeal-
mechanism). Per-model artifacts live under
`microservices/detection/models/<family>/<model_id>/` with model
card + training-data-snapshot-uri + hyperparameters + per-version
metadata.

Versioning policy: every crate SemVer per ADR-0258; the
ml-model-card-schema.json is versioned via `_meta.schema_version`;
breaking changes require ADR amendment + 60-day deprecation
cadence. Model card template additions are minor (additive); model
card template restructuring is major (breaking).

Per-config-flag rationale: ~30 per-tenant config flags (per-stage
threshold tuning, per-jurisdiction overlay, per-pack regulator
cadence override, retraining auto-trigger threshold, drift-alert
SEV threshold). Each flag has a documented default + per-pack
override behavior. Audited daily by
`oya-governance-ml-config-flag-coherence`.

Reverse dependencies: every ML-serving µservice depends on the
substrate. Initial dependents at platform GA: detection (all 8
families), intelligence, ops-dashboard-control-center (insider-risk
ranking), marketplace (recommendation), social (feed-ranking),
shorts (content-ranking), workflow-studio (workflow-suggestion).

### §C.2. Observability dimension

Per ADR-0263 emission contract, the ML lifecycle emits 10 new
audit-event-classes registered in the central registry:

| Class | Cardinality budget | Trace span shape | Retention |
|---|---|---|---|
| `MLModelTrainingCompleted` | ~10²/day | Parent: training-job; Child: per-epoch | 10-year cold (EU AI Act Art. 18) |
| `MLModelValidationPassed` | ~10²/day | Parent: validation-job; Child: per-metric-evaluation | 10-year cold |
| `MLModelValidationFailed` | ~10/day | Parent: validation-job; Child: per-metric-evaluation | 10-year cold |
| `MLModelABTestStarted` | ~10/day | Parent: ab-test-controller | 10-year cold |
| `MLModelABTestCompleted` | ~10/day | Parent: ab-test-controller | 10-year cold |
| `MLModelDriftDetected` | ~10²/day | Parent: drift-detection-batch | 10-year cold |
| `MLModelFairnessReportEmitted` | ~4/quarter | Parent: fairness-audit-batch | 10-year cold + regulator-facing surface |
| `MLModelAppealFiled` | ~10³-10⁴/day at platform GA | Parent: appeal-API | 10-year cold |
| `MLModelAppealAdjudicated` | ~10³-10⁴/day at platform GA | Parent: appeal-adjudication-controller | 10-year cold |
| `MLModelSeriousIncidentReported` | ~1/quarter (rare) | Parent: serious-incident-controller; Child: regulator-emission | 10-year cold + EU AI Act regulator surface |

Metrics (Prometheus + OpenTelemetry per ADR-0263):

- `ml_model_training_duration_seconds{model_id, p50|p95|p99}` — histogram
- `ml_model_validation_pass_rate{model_id, family, jurisdiction}` — gauge
- `ml_model_ab_test_win_rate{family, jurisdiction}` — gauge
- `ml_model_drift_score{model_id, drift_type, feature_id}` — gauge
- `ml_model_fairness_disparate_impact_ratio{model_id, protected_class, jurisdiction}` — gauge
- `ml_model_inference_latency_seconds{model_id, p50|p95|p99}` — histogram (cross-ref ADR-0307 §C.4)
- `ml_model_rollback_duration_seconds{model_id, p50|p95|p99}` — histogram; SLA: P99 ≤ 15min
- `ml_model_appeal_filed_total{tenant_id, family, jurisdiction}` — counter
- `ml_model_appeal_adjudication_latency_seconds{family, jurisdiction, p50|p95|p99}` — histogram; SLA per pack
- `ml_model_serious_incident_total{model_id, family, jurisdiction}` — counter (rare; should be ~0)

Dashboards (Grafana, stored in microservices/detection/dashboards/):

1. `ml-lifecycle-overview.json` — per-model lifecycle state + per-stage
   pass-rate
2. `ml-training-pipeline.json` — per-model training duration + GPU
   utilization + dataset-quality metrics
3. `ml-validation-pre-promotion.json` — per-model validation pass
   rate + per-metric breakdown + bias-audit results
4. `ml-ab-test-tracker.json` — per-model A/B test progress +
   challenger-vs-champion lift
5. `ml-drift-detection-daily.json` — per-model drift scores + alert
   history
6. `ml-fairness-quarterly.json` — per-class TPR/FPR + disparate-impact
   ratio per jurisdiction (joint with ADR-0309)
7. `ml-rollback-history.json` — rollback events + time-to-effect +
   serious-incident attribution
8. `ml-appeal-mechanism.json` — appeal volume + adjudication SLA
   per pack + per-jurisdiction
9. `ml-regulator-facing-surface.json` — quarterly fairness reports
   + serious-incident reports + EU AI Act Art. 27 FRIA emissions

SLO floor (per `microservices/detection/slos/*.openslo.yaml`):

- Validation gate pass-or-fail decision ≤30min per model; 99.9% monthly
- A/B test soak window ≥7d per stage enforced; 100% (BLOCKER lane)
- Drift detection daily cadence ≤24h staleness; 99.5% monthly
- Fairness re-audit quarterly ±15d window; 100% (BLOCKER lane)
- Rollback time-to-effect P99 ≤ 15min; 99.9% monthly
- Appeal adjudication SLA per pack; 99.5% monthly
- EU AI Act Art. 73 serious-incident report within 24h; 100% (BLOCKER lane)

### §C.3. Scalability dimension

Per documentation-rigor.md §1.1 item 3 capacity math:

**Training pipeline.** Per-family per-week training jobs (8 families
× ~3-5 retrains/year + ~weekly drift-correction = ~50 jobs/year);
per-job runtime ≤24h on GPU cluster (4× A100 / H100 equivalent);
horizontal scale-out via per-family parallel training.

**Validation pipeline.** Per-model per-validation runtime ≤2h on
CPU + GPU mixed cluster; ~50 validations/year per family × 8 =
~400/year platform-wide.

**A/B testing.** Per-family A/B traffic split via Flink topology
parameter; no additional compute (the topology already runs both
champion + challenger when in shadow + canary mode).

**Drift detection.** Daily per-model batch job ≤4h runtime on
Spark cluster; ~per-model × ~30 models at platform GA × daily =
~30/day batch jobs; well under Spark cluster capacity.

**Fairness re-audit.** Quarterly per-model per-jurisdiction;
~30 models × 5 jurisdictions × 4 quarters = ~600 jobs/year; ~per-
job ≤6h runtime; ~10/week sustained.

**Appeal mechanism.** Per-appeal human-reviewer routing per ADR-0310;
~10³-10⁴/day at platform GA; human-reviewer capacity sized per
ADR-0310 §F (~100-1000 human reviewers contracted per pack).

**Serious-incident reporting.** ~1/quarter expected; per-pack
regulator emission cadence per ADR-0251.

10× and 100× scale-out path: every primitive scales via additional
GPU/CPU executor capacity; the ML lifecycle does not have inherent
bottlenecks. Per-jurisdiction model variants at 100× tenant count
would multiply model count by ~10-100×; storage for 10-year
training-data retention scales with audit-event volume.

### §C.4. Performance dimension

Per documentation-rigor.md §1.2 Performance dimension:

| Stage | P50 | P95 | P99 | Tail mitigation |
|---|---|---|---|---|
| Training (per model) | 6h | 18h | 22h | GPU dynamic-allocation; checkpoint-resume |
| Validation (per model) | 30min | 90min | 2h | Per-metric parallel evaluation |
| A/B test promotion decision (per stage) | 7d | 7d | 7d | Soak window is fixed; not a latency metric |
| Drift detection (per daily run) | 2h | 3h | 4h | Spark dynamic-allocation |
| Fairness re-audit (per quarterly run) | 4h | 5h | 6h | Per-jurisdiction parallel runs |
| Model inference | 1ms | 5ms | 20ms | Per ADR-0307 §C.4 |
| Rollback time-to-effect | 5min | 12min | 15min | Pre-warmed previous champion |
| Appeal-adjudication SLA | 7d | 21d | 30d | Per-pack human-reviewer queue |
| Serious-incident report | 4h | 18h | 23h | 24h hard ceiling per EU AI Act Art. 73 |

### §C.5. Optimization dimension

Per documentation-rigor.md §1.2 Optimization:

Per-stage cost model:

- Training: GPU-hours × per-GPU-hour cost; ~$100-500 per training
  run at platform GA
- Validation: CPU/GPU hours; ~$50-200 per validation
- A/B test: amortized into ADR-0307 streaming runtime cost; no
  extra compute
- Drift detection: per-daily Spark job ~$20-50 per family × 8
  families × 365d = ~$58k-$146k/year
- Fairness re-audit: ~$50-200 per quarterly job × 30 models × 5
  jurisdictions × 4 quarters = ~$30k-120k/year
- Rollback: ~$0 (uses already-deployed previous champion)
- Appeal adjudication: human-reviewer hours per pack; $50-200/hour
  contracted

Lazy vs eager trade-offs:

- **Eager** for daily drift detection (alternative is reactive-only,
  which fails the SLO)
- **Eager** for quarterly fairness re-audit (regulator-mandated)
- **Lazy (on-demand)** for adversarial-robustness benchmarks
  beyond release-gate (e.g., red-team-mode is on-demand)
- **Cached** for SHAP/LIME feature-importance values per prediction
  (cached for 30 days for appeal-window-coverage)

Cold-vs-warm path latency: cold (first inference on new model
deployment) ≈ 100ms (LightGBM model load from disk); warm ≈ 1-5ms.
Pre-warm on deployment via load-test traffic.

### §C.6. Code quality dimension

Per documentation-rigor.md §1.2 Code quality dimension:

- **Test classes:** unit (per-lifecycle-stage), property-based
  (training-data-snapshot reproducibility properties), fuzz
  (model-card validator + rule-parser), load (per-stage scale-out),
  e2e (full eight-stage lifecycle for synthetic-fraud-detection model).
- **Coverage floor:** ≥85% line, ≥75% branch.
- **Lint passes:** `cargo clippy -- -D warnings`,
  `oya-check-cedar-fragment-soak`, `oya-check-ml-model-card-schema`,
  `oya-check-naming-bnf-v4`, `oya-check-layer-enum-conformance`,
  `oya-check-ml-fairness-quarterly-cadence`,
  `oya-governance-ml-appeal-mechanism-coverage`.
- **Type-strictness:** Rust `deny(warnings)` + `deny(unsafe_code)`.
- **SemVer + ABI policy:** per ADR-0258; major bumps require ADR
  amendment.

## §D. Detailed mechanics

### §D-1. Training pipeline mechanics

#### §D-1.1. Per-tenant residency enforcement

Per ADR-0244 tenant scoping, training-data fetch enforces:

```
function fetch_training_batch(model_id, training_window, tenant_filter) {
  let allowed_tenants = cedar_eval(
    principal: model.training_principal,
    action: "ml::fetch_training_batch",
    resource: training_window,
    context: { tenant_filter, model_id, training_window }
  );
  assert allowed_tenants.is_permit();
  let batch = iceberg.scan(
    table: "audit_features",
    predicate: tenant_id IN allowed_tenants
              AND timestamp IN training_window
              AND data_class IN allowed_data_classes(model)
  );
  return batch;
}
```

The Cedar evaluation per ADR-0243 confirms training principal has
permit to read the relevant tenants' feature data.

#### §D-1.2. Per-pack data-class filter

Per ADR-0099 data-class registry:

- HIPAA pack → PHI features excluded (e.g., diagnosis codes, lab
  values)
- GDPR pack → features pseudonymized (k-anonymity k≥5; differential
  privacy ε≤1.0 for aggregate features)
- KR-PIPA pack → features pseudonymized + Real-Name removed
- COPPA pack → no training on <13 user features (tenant manifest
  declares min_age; users below threshold excluded)
- KOSA pack → minor-protection feature subset only

#### §D-1.3. Iceberg snapshot mechanics

Per Apache Iceberg time-travel:

```
let snapshot_id = iceberg.snapshot_for_timestamp(training_window.end);
let model_card.training_data_snapshot_uri = format!(
  "iceberg://{cluster}/{schema}/audit_features?snapshot={snapshot_id}"
);
```

Snapshot retained ≥10 years per EU AI Act Art. 18.

#### §D-1.4. Hyperparameter logging

Per MLflow-class experiment tracker; tracking server deployed per-
cell:

```
mlflow.log_params({
  "learning_rate": 0.05,
  "num_iterations": 200,
  "max_depth": 12,
  "subsample": 0.8,
  ...
});
mlflow.log_metric("auc_validation", auc);
mlflow.log_artifact("model.lgb");
mlflow.log_artifact("model_card.json");
```

#### §D-1.5. Compute attestation

Per ADR-0293 + ADR-0295:

```
let spiffe_svid = workload_identity.get_svid();
let training_job.compute_attestation = {
  spiffe_id: spiffe_svid.spiffe_id,
  cell_id: cluster.cell_id,
  gpu_attestation: nvidia_confidential_computing.attest(),
  kernel_hash: sha256(kernel_binary),
  oci_image_hash: image.digest,
};
audit.emit("MLModelTrainingComputeAttested", training_job.compute_attestation);
```

### §D-2. Validation pipeline mechanics

#### §D-2.1. Held-out set construction

Per-class stratified sampling from Iceberg snapshot; held-out
window = last 7 days of training-window. Held-out set explicitly
excluded from training-data per `iceberg.exclude_window()`.

#### §D-2.2. Per-class TPR/FPR computation

```
for protected_class in jurisdiction.protected_classes() {
  let mask = held_out.where(protected_class_attr == protected_class);
  let tpr = (mask.predicted_positive AND mask.true_positive).count()
            / mask.true_positive.count();
  let fpr = (mask.predicted_positive AND NOT mask.true_positive).count()
            / NOT mask.true_positive.count();
  metrics[protected_class] = { tpr, fpr };
}
let max_tpr_gap = max(metrics.tpr) - min(metrics.tpr);
let max_fpr_gap = max(metrics.fpr) - min(metrics.fpr);
assert max_tpr_gap <= 0.02 AND max_fpr_gap <= 0.02;  // ±2pp baseline
```

If gap > 0.02 → validation fails; routed to training queue with
fairness-feedback to drive next iteration.

#### §D-2.3. Disparate-impact ratio (4/5ths rule)

```
let selection_rates = {};
for protected_class in jurisdiction.protected_classes() {
  let mask = held_out.where(protected_class_attr == protected_class);
  selection_rates[protected_class] = mask.predicted_positive.count() / mask.count();
}
let majority_rate = selection_rates[jurisdiction.majority_class()];
let min_ratio = min(selection_rates) / majority_rate;
assert min_ratio >= 0.8;  // 4/5ths rule
```

If ratio < 0.8 → validation fails.

#### §D-2.4. Adversarial-robustness benchmark

Per attack class (FGSM for vision; HotFlip for NLP; tabular-data
adversarial gradient):

```
for attack in [FGSM, PGD, HotFlip] {
  let adversarial = attack.generate(held_out, epsilon=0.1);
  let acc_under_attack = model.evaluate(adversarial);
  assert acc_under_attack >= 0.8 * baseline_acc;
}
```

#### §D-2.5. Calibration check

Per Brier score + reliability diagram + Expected Calibration Error
(ECE):

```
let predicted = model.predict_proba(held_out);
let ece = expected_calibration_error(predicted, held_out.true_label);
assert ece <= 0.05;  // 5% calibration tolerance
```

### §D-3. A/B testing mechanics

#### §D-3.1. Shadow mode

Both champion + challenger run inference; only champion drives
action. Challenger predictions logged for offline comparison.

```
let champion_pred = champion.predict(features);
let challenger_pred = challenger.predict(features);
audit.emit("MLModelABShadowPrediction", {
  champion_pred, challenger_pred, features.entity_id, timestamp
});
action = champion_pred.action;  // champion drives
```

Duration ≥7 days; sample size ≥10⁵ per cohort.

#### §D-3.2. Canary mode

Per-traffic-fraction split (1% → 5% → 25% per stage; ≥7d per stage):

```
let cohort = hash(features.entity_id) % 100;
if cohort < canary_fraction {
  action = challenger.predict(features).action;  // canary cohort
} else {
  action = champion.predict(features).action;  // baseline
}
audit.emit("MLModelABCanaryPrediction", { cohort, action });
```

#### §D-3.3. Full mode

```
action = challenger.predict(features).action;  // challenger drives
champion_kept_as_fallback = true;  // for rollback per §B.7
```

#### §D-3.4. Promotion decision

Per-metric statistical-significance + adverse-action-rate parity:

```
let p_value = welch_t_test(challenger.auc, champion.auc, n=samples);
assert p_value < 0.01;
for protected_class in jurisdiction.protected_classes() {
  let challenger_aar = challenger.adverse_action_rate(protected_class);
  let champion_aar = champion.adverse_action_rate(protected_class);
  assert abs(challenger_aar - champion_aar) <= 0.02;  // ±2pp
}
```

Failure → A/B test aborted; challenger sent back to training queue.

### §D-4. Drift detection mechanics

#### §D-4.1. Feature drift (PSI)

Population Stability Index:

```
function psi(reference, current) {
  let bins = 10;
  let ref_dist = histogram(reference, bins).normalize();
  let cur_dist = histogram(current, bins).normalize();
  let psi = sum((cur_dist[i] - ref_dist[i]) * log(cur_dist[i] / ref_dist[i]));
  return psi;
}
for feature in model.features {
  let ref = iceberg.read(feature, training_window);
  let cur = iceberg.read(feature, last_7_days);
  let psi_value = psi(ref, cur);
  if psi_value > 0.2 {
    audit.emit("MLModelDriftDetected", { drift_type: "feature", feature, psi_value });
    alert_sev2(model_id, feature, psi_value);
  }
}
```

#### §D-4.2. Label drift

Distribution shift in ground-truth labels (from analyst feedback per
ADR-0310 + outcome telemetry):

```
let ref_labels = iceberg.read(label_col, training_window);
let cur_labels = iceberg.read(label_col, last_7_days);
let shift_per_class = abs(distribution(ref_labels) - distribution(cur_labels));
if max(shift_per_class) > 0.05 {
  audit.emit("MLModelDriftDetected", { drift_type: "label", shift_per_class });
  alert_sev2(model_id);
}
```

#### §D-4.3. Concept drift

Per-cohort error rate over time:

```
let baseline_error = model.evaluate(training_holdout);
for cohort in cohorts {
  let cohort_error = model.evaluate_cohort(cohort, last_7_days);
  if cohort_error > 1.5 * baseline_error for 3 consecutive days {
    audit.emit("MLModelDriftDetected", { drift_type: "concept", cohort, cohort_error });
    alert_sev1(model_id, cohort);
    retraining_queue.enqueue(model_id);
  }
}
```

### §D-5. Fairness re-audit mechanics

#### §D-5.1. Quarterly cadence

Per cron schedule per cell:

```
cron: "0 0 1 */3 *"  // 00:00 on day 1 of every 3rd month
```

Job: `microservices/detection/batch/fairness_quarterly.scala`.

#### §D-5.2. Per-jurisdiction overlay

Per ADR-0309:

- US: race, gender, age, religion, national origin, disability,
  veteran status (per ECOA Reg B + Fair Housing Act + ADA + USERRA)
- EU: per Charter Art. 21 (sex, race, color, ethnic origin, social
  origin, language, religion, political opinion, age, sexual
  orientation, disability, national-minority membership)
- KR: per Constitution + Equality Act (gender, age, religion,
  national origin, disability, marital status)
- JP: per Equal Employment Opportunity Act (gender, age, nationality,
  disability)
- UK: per Equality Act 2010 (9 protected characteristics)
- CA: per California Fair Employment and Housing Act

#### §D-5.3. Public-notice format

Per NY AEDT Local Law 144 (2023):

```yaml
audit_results:
  date: 2026-Q2
  model_id: payment_fraud_v3.2.1
  used_for: "Payment fraud detection"
  most_recent_audit_date: 2026-05-15
  selection_rate_max: 0.92
  selection_rate_min: 0.81
  impact_ratio: 0.88
  scoring_rate_max: 0.94
  scoring_rate_min: 0.85
  scoring_impact_ratio: 0.91
  protected_classes_covered: [race, gender, age]
```

Published to `/transparency` surface; archived in
`microservices/detection/transparency-reports/`.

### §D-6. Model versioning mechanics

#### §D-6.1. SemVer bump rules

Per ADR-0258:

- **Major.** Training-data-source change (e.g., new tenant cohort) OR
  architecture change (e.g., LightGBM → neural-network) OR breaking
  output-schema change (e.g., added new signal-class)
- **Minor.** Hyperparameter change OR additive feature addition (new
  feature added to scorer without removing existing)
- **Patch.** Bug fix OR drift-correction retrain (same architecture
  + same features + same training-data window structure)

#### §D-6.2. Model card schema

Per `ml-model-card-schema.json` (matches Google Model Card template):

```yaml
model_details:
  developer: oyatie-detection-team
  model_id: payment_fraud_v3.2.1
  type: "LightGBM ensemble + SHAP TreeExplainer"
  license: "Internal proprietary"
  contact: "ml-platform@oyatie.com"
  binding_adr: "ADR-0308"
intended_use:
  primary_users: ["substrate consumers per ADR-0307"]
  primary_surfaces: ["payments::charge_attempt", "payments::dispute_filed"]
  out_of_scope: ["non-payment surfaces"]
factors:
  relevant: ["transaction_velocity", "geo_signal", "device_signal"]
  evaluation: ["protected_class_per_jurisdiction"]
metrics:
  primary: ["AUC", "FPR@95%TPR", "per_class_TPR/FPR"]
  decision_threshold: 0.7
evaluation_data:
  datasets: ["iceberg://prod/audit_features?snapshot=abc123"]
  motivation: "Held-out 7-day window from training-window end"
training_data:
  datasets: ["iceberg://prod/audit_features?snapshot=def456"]
  motivation: "90-day window of historical PaymentChargeAttempted events"
  preprocessing: "tenant_id filter + per-pack data-class filter"
quantitative_analyses:
  unitary: { auc: 0.94, fpr_at_95tpr: 0.03 }
  intersectional: { ... per-class TPR/FPR ... }
ethical_considerations: "..."
caveats_and_recommendations: "..."
```

#### §D-6.3. Reproducibility surface

Per EU AI Act Art. 18 + Apache Iceberg time-travel:

```
function reproduce_model(model_id, version) {
  let card = model_registry.fetch_card(model_id, version);
  let snapshot = iceberg.fetch_snapshot(card.training_data.datasets[0]);
  let hyperparams = card.training_hyperparameters;
  let new_model = training_pipeline.train(snapshot, hyperparams);
  assert new_model.equals(card.model_artifact_hash);
  return new_model;
}
```

### §D-7. Rollback mechanics

#### §D-7.1. Trigger conditions

Per ADR-0294 anomaly-rollback + this ADR:

- Per-pack SLA breach (e.g., FPR exceeds per-pack ceiling for ≥1h)
- Per-metric threshold breach (drift SEV1 alert + retraining queue
  trigger)
- Serious-incident signal arrival (per §B.7 + §D-7.2)
- Manual rollback per `detection-model-rollback.md` runbook

#### §D-7.2. EU AI Act Art. 73 serious-incident classification

"Serious incident" per Art. 3(49):

- Incident leading to death or serious harm to health
- Serious irreversible disruption of management or operation of
  critical infrastructure
- Breach of obligations under EU law for protection of fundamental
  rights
- Serious damage to property or environment

In oyatie's detection substrate context:

- Discrimination affecting ≥100 users in any protected class
- Detected fairness gap > ±5pp (vs ±2pp baseline)
- False-positive cascade locking out ≥1000 users in <1h
- Audit-chain tamper attempt confirmed (cross-ref ADR-0028)
- Cross-tenant feature leakage detected

#### §D-7.3. Rollback procedure

Per runbook `microservices/detection/runbooks/detection-model-rollback.md`:

1. Trigger detected → SEV1 alert + on-call notified
2. Per-pack regulator clock starts (24h)
3. Previous champion model traffic-shift initiated
4. Time-to-effect target: ≤15min
5. Challenger model marked for forensic analysis
6. EU AI Act Art. 73 report drafted + emitted within 24h
7. Per-pack regulator notification per ADR-0251 cadence
8. Post-rollback reconciliation per §C.6 mitigation invariants

Audit events emitted: `MLModelRolledBack` + `MLModelSeriousIncidentReported`.

### §D-8. Appeal mechanism mechanics

#### §D-8.1. Adverse-action surface

Per ECOA Reg B + EU AI Act Art. 86:

- Affected party receives adverse-action notice within 30 days
  (ECOA Reg B requirement)
- Notice contains specific reasons (top-5 feature-importance from
  SHAP/LIME per §B.2)
- Notice contains appeal mechanism link

```
function emit_adverse_action_notice(decision, affected_party) {
  let top_5 = shap_explainer.top_features(decision.prediction);
  let appeal_link = build_appeal_link(decision.decision_id, affected_party);
  notify(affected_party, {
    decision_id: decision.decision_id,
    decision: decision.action,
    specific_reasons: top_5,
    appeal_link: appeal_link,
    appeal_sla: per_pack_sla(affected_party.compliance_packs),
  });
  audit.emit("MLModelAppealNoticeEmitted", { ... });
}
```

#### §D-8.2. Appeal filing

Per `microservices/detection/src/appeal/` API surface:

```
POST /v1/ml-appeal
Body: {
  decision_id: string,
  affected_party_id: string,
  appeal_reason: string (free-form),
  requested_outcome: "reverse" | "explain" | "modify",
  evidence: [ ... attachments ... ]
}
```

Routes to investigation case-management per ADR-0310.

Audit event: `MLModelAppealFiled`.

#### §D-8.3. Human-reviewer adjudication

Per ADR-0310 case-management workflow:

- Appeal enters investigation queue
- Human reviewer assigned per per-pack expertise (e.g., financial-
  fraud reviewers for payment-decision appeals)
- Reviewer evaluates: specific-reasons sufficient? feature-importance
  plausible? affected-party-evidence convincing?
- Verdict: uphold / reverse / partial
- Per-pack SLA: ≤30 days substantive review
- Audit event: `MLModelAppealAdjudicated`

#### §D-8.4. Outcome propagation

- **Reverse:** decision reversed; affected entity restored; per-
  µservice rollback (e.g., payment refunded, account unlocked,
  content restored); feature-store label updated with analyst-
  label for retraining

```
function adjudicate_appeal(appeal, verdict) {
  if verdict == "reverse" {
    decision_engine.reverse(appeal.decision_id);
    feature_store.write_analyst_label(appeal.decision_id, "reverse");
    retraining_queue.enqueue_label_feedback(appeal.affected_party_id);
    notify(appeal.affected_party_id, "appeal_upheld");
  }
  audit.emit("MLModelAppealAdjudicated", { appeal_id, verdict });
}
```

- **Uphold:** decision stands; affected party notified of human-
  reviewer's reasoning
- **Partial:** decision modified (e.g., cool-down reduced from 30d
  to 7d); reasoning provided

### §D-9. Per-jurisdiction overlay mechanics

Per-jurisdiction model variants (companion to ADR-0309):

```yaml
model_id: payment_fraud
variants:
  - jurisdiction: US
    base_model: payment_fraud_v3.2.1
    overlay: us-ecoa-reg-b-feature-mask  # excludes proxies for race / gender
    appeal_sla_days: 30  # ECOA Reg B
  - jurisdiction: EU
    base_model: payment_fraud_v3.2.1
    overlay: eu-ai-act-feature-mask  # excludes social-scoring features
    appeal_sla_days: 30  # GDPR Art. 22
  - jurisdiction: KR
    base_model: payment_fraud_v3.2.1
    overlay: kr-fcpa-feature-mask  # KR Financial Consumer Protection Act Art. 30
    appeal_sla_days: 60  # KR-FSS cadence
  - jurisdiction: JP
    base_model: payment_fraud_v3.2.1
    overlay: jp-appi-feature-mask
    appeal_sla_days: 30
```

## §E. Implementation footprint

### §E.1. New crates (per layer-5 shared-substrate)

Per ADR-0105 13-layer canonical enum row 5:

1. `crates/oya-shared-ml-lifecycle/` — orchestration trait over the eight stages
2. `crates/oya-shared-ml-training/` — training pipeline + Iceberg snapshot + per-tenant residency enforcement
3. `crates/oya-shared-ml-validation/` — bias audit + fair-lending + held-out evaluation + calibration check
4. `crates/oya-shared-ml-ab-test/` — champion-challenger controller + per-jurisdiction overlay
5. `crates/oya-shared-ml-drift-detection/` — PSI + label-drift + concept-drift detectors
6. `crates/oya-shared-ml-fairness-audit/` — quarterly fairness re-audit + per-jurisdiction overlay + public-notice emission
7. `crates/oya-shared-ml-versioning/` — SemVer policy + model card emission + Iceberg-backed reproducibility
8. `crates/oya-shared-ml-rollback/` — anomaly-rollback per ADR-0294 + EU AI Act Art. 73 serious-incident workflow
9. `crates/oya-shared-ml-appeal-mechanism/` — appeal-filing API + adjudication routing + per-pack SLA tracking

### §E.2. New JSON Schemas

Under `/specs/`:

1. `ml-model-card-schema.json` — Google Model Card template
2. `ml-model-lifecycle-schema.json` — eight-stage state machine
3. `ml-drift-detection-schema.json` — PSI + drift-types schema
4. `ml-ab-test-schema.json` — champion-challenger config schema
5. `ml-rollback-schema.json` — rollback trigger + outcome schema
6. `ml-appeal-schema.json` — appeal filing + adjudication schema

### §E.3. New µservice extensions

`microservices/detection/` (no new µservice; lifecycle integrated
into ADR-0307 substrate):

```
microservices/detection/
├── models/                  # NEW: per-family model registry
│   ├── payment_fraud/
│   │   ├── v3.2.1/
│   │   │   ├── model_card.json
│   │   │   ├── model_artifact.lgb
│   │   │   ├── training_data_snapshot.uri
│   │   │   ├── hyperparameters.json
│   │   │   └── compute_attestation.json
│   │   └── ...
│   ├── ato/
│   ├── synth_identity/
│   ├── aml/
│   ├── content_abuse/
│   ├── engagement_manipulation/
│   ├── insider_risk/
│   └── policy_violation/
├── transparency-reports/    # NEW: per-quarterly public notices
│   ├── 2026-Q2-payment_fraud.yaml
│   ├── 2026-Q2-ato.yaml
│   └── ...
├── runbooks/
│   ├── ml-training-failure.md             # NEW
│   ├── ml-validation-failure.md           # NEW
│   ├── ml-ab-test-failure.md              # NEW
│   ├── ml-drift-alert-sev1.md             # NEW
│   ├── ml-fairness-audit-failure.md       # NEW
│   ├── ml-rollback-eu-ai-act-art-73.md    # NEW
│   ├── ml-appeal-sla-breach.md            # NEW
│   └── ml-serious-incident-regulator.md   # NEW
├── policy/
│   ├── ml-training-data-access.cedar      # NEW: per-tenant training data Cedar gate
│   ├── ml-model-deployment.cedar          # NEW: per-deployment Cedar gate
│   ├── ml-appeal-adjudication.cedar       # NEW: per-reviewer Cedar gate
│   └── ...
```

### §E.4. New runbooks

8 new runbooks (listed above); each per §2 runbook rigor.

### §E.5. New CI lanes

- `oya-governance-ml-model-card-present` — verifies every deployed model has a model card matching the schema
- `oya-governance-ml-validation-bias-audit-present` — verifies validation includes bias audit + fair-lending
- `oya-governance-ml-ab-test-champion-challenger-coherence` — verifies A/B test follows the shadow → canary → full pattern
- `oya-governance-ml-drift-detection-daily` — verifies daily cadence on every production model
- `oya-governance-ml-fairness-quarterly-cadence` — verifies quarterly fairness re-audit within ±15d window
- `oya-governance-ml-model-semver-conformance` — verifies SemVer bump rules
- `oya-governance-ml-rollback-runbook-present` — verifies per-model rollback runbook exists
- `oya-governance-ml-appeal-mechanism-coverage` — verifies every adverse-action surface has appeal-mechanism-link
- `oya-governance-ml-eu-ai-act-art73-24h-incident-reporting` — verifies 24h cadence honored
- `oya-governance-ml-ny-aedt-bias-audit-public-notice` — verifies public-notice posted per quarter
- Aggregate: `oya-governance-ml-lifecycle`

### §E.6. Per-µservice extensions (consumers)

Every µservice serving any ML model updates:

- `compliance.md §ml-model-lifecycle` — per row 50 of §3.2.1
  ADR-adherence matrix
- `manifest.json:ml_models[]` — array of model_id + version + family
- `manifest.json:ml_appeal_mechanism_link` — per-pack appeal route

### §E.7. Vendor selection rationale

#### §E.7.1. Experiment tracker: MLflow (canonical) + Weights & Biases (alternative)

Selected because:
- MLflow is open-source + Apache 2.0 + ML standard
- W&B supported for Tier-3+ cells per ADR-0240
- Hugging Face Hub for open-model-card-only artifact retention

#### §E.7.2. Drift detection: in-house + Evidently AI (open-source backup)

Selected because:
- In-house PSI / label-drift / concept-drift implementations in
  oya-shared-ml-drift-detection
- Evidently AI as open-source reference for parity checks

#### §E.7.3. Fairness assessment: Fairlearn + AIF360

Selected because:
- Both are open-source + widely-adopted
- Fairlearn is Microsoft-stewarded (MIT license)
- AIF360 is IBM-stewarded (Apache 2.0)
- Both expose similar metrics; oyatie integrates both for parity

#### §E.7.4. Model card template: Google Model Card

Selected because:
- arxiv.org/abs/1810.03993 is the foundational paper
- Google's Vertex AI Model Registry uses this template
- ISO/IEC 42001:2023 references the model-card concept
- EU AI Act Art. 11 technical documentation compatible

## §F. Migration

### §F.1. Wave-3-D rollout sequencing

1. **2026-05-20 to 2026-06-15.** ADR-0308 + companion ADRs accepted.
2. **2026-06-15 to 2026-08-15.** Crate scaffolds + schemas + per-
   µservice integration points stubbed; existing in-µservice models
   audited + cataloged.
3. **2026-08-15 to 2026-09-15.** Per-stage lifecycle wired for each
   existing model; first quarterly fairness re-audit emitted; first
   appeal adjudication SLA tracked.
4. **2026-09-15.** CI lanes promote to BLOCKER; EU AI Act Art. 73
   24h reporting workflow live.
5. **2026-09-15 onwards.** Continuous: daily drift detection,
   quarterly fairness re-audit, per-PR validation gate.

### §F.2. Per-µservice migration playbook

For each µservice serving any ML model:

1. **Audit existing models.** Catalog model_id + version + family +
   training-data-source + serving-surface.
2. **Author model card.** Per `ml-model-card-schema.json`.
3. **Wire validation gate.** Before next deployment, gate via
   `oya-shared-ml-validation`.
4. **Enable daily drift detection.** Per `oya-shared-ml-drift-detection`.
5. **Schedule quarterly fairness re-audit.** Per
   `oya-shared-ml-fairness-audit`.
6. **Author rollback runbook.** Per `microservices/<svc>/runbooks/`.
7. **Wire appeal mechanism.** Per `oya-shared-ml-appeal-mechanism`.
8. **Update compliance.md.** Per row 50 of §3.2.1 ADR-adherence
   matrix.

### §F.3. Per-cell rollout pattern

- Tier-0 edge cells: no ML lifecycle (no models)
- Tier-1 bootstrap cell: no models
- Tier-2 control plane cells: full lifecycle (training + serving)
- Tier-3 data plane cells: serving only (training in Tier-2);
  per-cell model artifact + model card cached

### §F.4. What is NOT migrated

- Heuristic rules (per ADR-0307 §D-4) — not ML; lifecycle does not
  apply
- Cedar policies — separate substrate per ADR-0243
- Pre-existing approved third-party APIs (e.g., NCMEC PhotoDNA,
  GIFCT) — third-party governance; oyatie consumes but does not
  retrain

### §F.5. Rollback path

Per §B.7 + ADR-0294. Emergency kill-switch: per-µservice
`ML_BYPASS=1` env flag disables model-driven actions while
preserving audit logging; used on confirmed substrate failure.

## §G. References

### §G.1. Hyperscaler precedents

- **Google Model Cards for Model Reporting** — arxiv.org/abs/1810.03993 (Mitchell et al., FAccT 2019); ai.google/responsibility
- **Microsoft Datasheets for Datasets** — Gebru et al. (FAccT 2019); azure.microsoft.com/responsible-ai
- **Meta AI System Cards** — ai.meta.com/tools/system-cards
- **OpenAI Model + System Cards** — openai.com/research
- **Anthropic Model Reports** — anthropic.com/news
- **Stripe Radar Lifecycle** — Stripe Sessions 2024 keynote "Radar at Scale"
- **Microsoft Fairlearn** — github.com/fairlearn/fairlearn
- **IBM AI Fairness 360 (AIF360)** — github.com/Trusted-AI/AIF360
- **Google What-If Tool** — github.com/PAIR-code/what-if-tool
- **Arize AI** — arize.com
- **Fiddler AI** — fiddler.ai
- **WhyLabs** — whylabs.ai
- **Evidently AI** — evidentlyai.com
- **MLflow** — mlflow.org
- **Weights & Biases** — wandb.ai
- **Hugging Face Hub** — huggingface.co
- **Statsig** — statsig.com
- **Optimizely** — optimizely.com
- **LaunchDarkly Experimentation** — launchdarkly.com
- **Vertex AI Model Registry** — cloud.google.com/vertex-ai/docs/model-registry
- **AWS SageMaker Model Monitor** — aws.amazon.com/sagemaker/model-monitor
- **Azure Machine Learning Responsible AI dashboard** — learn.microsoft.com/en-us/azure/machine-learning/concept-responsible-ai-dashboard

### §G.2. Standards + RFCs

- **NIST AI Risk Management Framework 1.0** — nist.gov/itl/ai-risk-management-framework
- **ISO/IEC 42001:2023** — AI management systems; iso.org/standard/81230.html
- **ISO/IEC 23894:2023** — AI risk management guidance
- **ISO/IEC TR 24028:2020** — overview of trustworthiness in AI
- **ISO/IEC TS 4213:2022** — assessment of machine-learning classification performance
- **Apache Iceberg spec** — iceberg.apache.org/spec
- **Open Telemetry semantic conventions** — opentelemetry.io/docs/specs/semconv
- **Google Model Card schema** — github.com/tensorflow/model-card-toolkit

### §G.3. Legal + compliance

- **EU AI Act (Regulation 2024/1689)** — Articles 6, 8, 9, 10, 11, 12, 13, 14, 15, 17, 18, 27, 51-55, 53, 73, 86; Annex III §§1, 3, 5(b), 5(c), 6, 7, 8
- **GDPR (Regulation 2016/679)** — Article 22 (automated-decision rights), Article 12 (timing), Article 35 (DPIA)
- **NY AEDT Local Law 144 (2023)** — rules.cityofnewyork.us/wp-content/uploads/2023/04/DCWP-NOA-for-Automated-Employment-Decision-Tools-2.pdf
- **ECOA + Regulation B (12 CFR §1002)** — adverse-action notice with specific reasons
- **Fair Housing Act (42 USC §3601-3619)** + **HUD's disparate-impact rule (24 CFR §100.500)**
- **Federal Uniform Guidelines on Employee Selection Procedures (29 CFR §1607.4)** — 4/5ths rule
- **California Generative AI Training Data Transparency Act (SB 942, 2024)**
- **Colorado Artificial Intelligence Act (SB 24-205, 2024)** — effective 2026-02-01
- **Illinois Biometric Information Privacy Act (BIPA, 740 ILCS 14)**
- **Texas Capture or Use of Biometric Identifier Act (CUBI)**
- **Washington My Health My Data Act (RCW 19.373)**
- **Utah AI Disclosure Bill (SB 149, 2024)**
- **California AB 2013 (2024) + AB 3030 (2024) + AB 2655 (2024)**
- **KR Financial Consumer Protection Act Art. 30** — 시행 2021-03-25
- **KR-PIPA + KR-Credit-Information-Act**
- **JP APPI**
- **UK Age Appropriate Design Code (AADC, 2020)**
- **UK Equality Act 2010**
- **AU Online Safety Act 2021** + **AU Privacy Act amendments (2024)**

### §G.4. Internal portfolio ADRs

- **ADR-0028** — audit-chain Merkle-sealed
- **ADR-0099** — data-class registry
- **ADR-0105** — 13-layer canonical enum
- **ADR-0130** — agentic SLO-gated promotion
- **ADR-0131** — per-microservice flat layout
- **ADR-0132** — no-grouping microservice rule
- **ADR-0140** — Cedar policy enforcement
- **ADR-0145** — inter-microservice communication reform
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
- **ADR-0280** — substrate-of-substrate dependency
- **ADR-0293** — Foundry meta-trust-root
- **ADR-0294** — Cedar fragment soak + anomaly-rollback
- **ADR-0295** — bootstrap CI SPIFFE + kill-switch
- **ADR-0297** — abuse-defence baseline
- **ADR-0298** — emergency-services critical-path exemption
- **ADR-0307** — detection substrate (this bundle)
- **ADR-0309** — detection fairness audit + civil-rights compliance (this bundle)
- **ADR-0310** — investigation case-management (this bundle)

### §G.5. Standards docs

- `docs/standards/documentation-rigor.md` §3.2.6 — DRMP baseline
- `docs/standards/fintech-compliance.md` — KR-FSS / PCI-DSS / ECOA
- `docs/standards/doc-style.md` — Diátaxis + RFC-2119
- `docs/STANDARDS-AND-TEMPLATES.md` — catalog

### §G.6. Auto-memory feedback (related)

- `feedback_quality_performance_scalability_bar` — hyperscaler-grade rigor
- `feedback_clean_architecture_requirements` — inward-only + single-concern
- `feedback_no_silent_regression` — public-contract protection
- `feedback_autonomous_implementation_artifacts` — intern-buildable lifecycle
- `feedback_build_ahead_of_certification` — day-one EU AI Act + ISO/IEC 42001
- `feedback_oyatie_is_a_tenant_doctrine` — lifecycle applies to oyatie's own AI surfaces
- `feedback_cedar_as_universal_gate` — Cedar gates training-data access + appeal adjudication
- `feedback_compliance_pack_primitive` — per-pack overlay across lifecycle
- `feedback_substrate_vs_product_layering` — lifecycle is substrate
- `feedback_naming_justification` — every primitive justified

## §H. Change log

- **2026-05-20** — Initial draft authored as part of keystone-bundle 2026-05-20 Wave-3-D detection-cluster batch (ADR-0307..0310). Bundled with ADR-0307 (detection substrate), ADR-0309 (fairness audit), ADR-0310 (investigation case-management) as the **drmp-detection-cluster** keystone batch. Covers EU AI Act + NIST AI RMF + ISO/IEC 42001 ML lifecycle obligations. Enforcement advisory-until-2026-09-15-blocker-thereafter.
