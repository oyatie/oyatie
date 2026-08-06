---
id: ADR-0309
status: Accepted
date: 2026-05-20
owners:
  - council-architecture
  - council-security
  - council-privacy
  - council-product
  - council-legal
  - council-data
  - council-ml
  - council-civil-rights
  - ops-trust-and-safety
  - ops-compliance
  - ops-ml-platform
  - axis-detection
  - axis-fairness
  - axis-jurisdiction-overlay
  - axis-investigation
supersedes: []
amends: []
superseded_by: []
related:
  - ADR-0028-audit-chain-merkle-sealed.md
  - ADR-0063-doc-coverage-enforced.md
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
  - ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md
  - ADR-0293-governance-meta-trust-root.md
  - ADR-0294-cedar-fragment-soak-anomaly-rollback.md
  - ADR-0295-bootstrap-ci-spiffe-kill-switch.md
  - ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md
  - ADR-0298-emergency-services-critical-path-exemption.md
  - ADR-0307-detection-substrate-streaming-batch.md
  - ADR-0308-ml-model-lifecycle-ai-act-compliance.md
  - ADR-0310-investigation-case-management.md
related_specs:
  - /specs/microservices/detection.json
  - /specs/fairness-audit-schema.json
  - /specs/protected-class-registry.json
  - /specs/per-jurisdiction-model-variant-schema.json
  - /specs/proxy-feature-prohibition-list.json
  - /specs/disparate-impact-test-schema.json
  - /specs/civil-rights-pack-overlay-schema.json
related_memory:
  - feedback_quality_performance_scalability_bar
  - feedback_clean_architecture_requirements
  - feedback_no_silent_regression
  - feedback_autonomous_implementation_artifacts
  - feedback_canonical_base_localization
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_cedar_as_universal_gate
  - feedback_compliance_pack_primitive
  - feedback_naming_justification
  - feedback_substrate_vs_product_layering
  - feedback_build_ahead_of_certification
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-foundational-doctrine
keystone_position: drmp-detection-fairness-civil-rights
purpose: >
  Establish the Detection Fairness + Civil-Rights Compliance baseline
  for the Detection Substrate (per ADR-0307) and the ML Model
  Lifecycle (per ADR-0308). Codifies the five fairness invariants
  from documentation-rigor.md §3.2.6 + the per-jurisdiction model
  variants required by EU AI Act + ECOA + Fair Housing Act + NY
  AEDT Local Law 144 (2023) + HUD's disparate-impact rule + KR
  Financial Consumer Protection Act Art. 30 + JP APPI + UK AADC.
  The five invariants:
    1. No proxy discrimination (zip code → race, name → ethnicity,
       language → national origin)
    2. Per-class TPR/FPR equity within ±2pp baseline; wider gaps
       require explicit ADR justification + regulator notification
    3. Disparate-impact testing per 4/5ths rule (29 CFR §1607.4)
    4. Explainability floor (ECOA Reg B + GDPR Art. 13/22 + EU AI
       Act Art. 13)
    5. Per-jurisdiction model variants (US ECOA + NY AEDT + state-AG;
       EU AI Act Art. 5 social-scoring forbidden; KR FCPA Art. 30;
       JP APPI; UK AADC)
  Without these invariants, the detection substrate ships an EU AI
  Act + ECOA + Fair Housing + NY AEDT + KR-FSC violation. Build-
  ahead-of-certification (per ADR-0250) requires day-one compliance
  — the invariants MUST be in place before any detection model
  serves production traffic.
enforcement_status: advisory-until-2026-09-15-blocker-thereafter
enforced_by:
  - cloud-ci/Rust gate packet detection-fairness-proxy-feature-prohibition
  - cloud-ci/Rust gate packet detection-fairness-per-class-tpr-fpr-equity
  - cloud-ci/Rust gate packet detection-fairness-disparate-impact-4-5ths-rule
  - cloud-ci/Rust gate packet detection-fairness-explainability-floor
  - cloud-ci/Rust gate packet detection-fairness-per-jurisdiction-variant-coverage
  - cloud-ci/Rust gate packet detection-fairness-eu-ai-act-art-5-social-scoring-forbidden
  - cloud-ci/Rust gate packet detection-fairness-kr-fcpa-art-30-financial-protected-class-forbidden
  - cloud-ci/Rust gate packet detection-fairness-quarterly-cadence-public-notice
naming_justifications:
  - name: oya-shared-fairness-audit
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.fairness-audit
    justification: >
      Per ADR-0105 13-layer canonical enum row 5 (shared-substrate),
      the crate that exposes the per-class TPR/FPR computation +
      disparate-impact test + per-jurisdiction protected-class
      registry across every model-serving µservice belongs at the
      shared layer. Single-concern per ADR-0131; not bundled with
      validation (oya-shared-ml-validation per ADR-0308) so fairness-
      specific tests can SemVer-evolve independently.
  - name: oya-shared-proxy-feature-detector
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.proxy-feature-detector
    justification: >
      Detects features that proxy protected classes (zip code →
      race, name → ethnicity, language preference → national origin,
      device → socio-economic class). Single-concern per ADR-0131;
      maintained as separate crate because the proxy-detection
      methodology (correlation analysis + counterfactual generation
      + mutual-information score) evolves on a different cadence
      than the per-class TPR/FPR computation.
  - name: oya-shared-disparate-impact-tester
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.disparate-impact-tester
    justification: >
      Implements the 4/5ths rule (29 CFR §1607.4) + EU + KR + JP
      regulator-equivalent floors. Single-concern per ADR-0131;
      separate from fairness-audit because disparate-impact has
      a specific legal definition + per-jurisdiction interpretation
      that benefits from its own crate boundary.
  - name: oya-shared-explainability-floor
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.explainability-floor
    justification: >
      Implements the ECOA Reg B specific-reasons surface + GDPR Art.
      13/22 meaningful-explanation + EU AI Act Art. 13 transparency
      floor. Wraps SHAP / LIME + per-feature human-readable
      translation. Single-concern.
  - name: oya-shared-per-jurisdiction-model-variant
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.per-jurisdiction-model-variant
    justification: >
      Implements per-jurisdiction model overlays (US ECOA + NY AEDT;
      EU AI Act feature-mask; KR FCPA Art. 30; JP APPI; UK AADC).
      Single-concern; per-jurisdiction overlay shape is a stable
      primitive that downstream µservices depend on, deserves its
      own crate.
  - name: oya-shared-protected-class-registry
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.protected-class-registry
    justification: >
      Per-jurisdiction enumeration of protected classes (US: race,
      gender, age, religion, national origin, disability, veteran;
      EU Charter Art. 21: 14 classes; KR Equality Act; JP Equal
      Employment Opportunity Act; UK Equality Act 2010). Single-
      concern; registry per ADR-0105.
  - name: oya-governance-detection-fairness-audit
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.detection-fairness-audit
    justification: >
      CI fitness lane per ADR-0212; verifies quarterly fairness re-
      audit emitted per model per jurisdiction within ±15-day window
      per quarter.
  - name: oya-governance-detection-fairness-proxy-feature-prohibition
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.detection-fairness-proxy-feature-prohibition
    justification: >
      CI fitness lane per ADR-0212; verifies no feature on the
      proxy-feature-prohibition-list enters training data without
      explicit ADR-0309 amendment.
  - name: oya-governance-detection-fairness-explainability-floor
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.detection-fairness-explainability-floor
    justification: >
      CI fitness lane per ADR-0212; verifies every adverse-action
      decision carries top-5 feature-importance + appeal-link.
  - name: oya-governance-detection-fairness-per-jurisdiction-variant-coverage
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.detection-fairness-per-jurisdiction-variant-coverage
    justification: >
      CI fitness lane per ADR-0212; verifies per-model per-
      jurisdiction variant exists for every model serving a tenant
      in that jurisdiction.
  - name: FairnessAuditPassed
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Fairness.AuditPassed
    justification: >
      Emitted when quarterly fairness audit passes thresholds.
      Registered per ADR-0263.
  - name: FairnessAuditFailed
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Fairness.AuditFailed
    justification: >
      Emitted when audit detects gap > ±2pp on any protected class;
      triggers retraining queue + per-pack regulator notification.
  - name: ProxyFeatureDetected
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Fairness.ProxyFeatureDetected
    justification: >
      Emitted when correlation analysis detects a feature is acting
      as a proxy for a protected class (e.g., zip code → race
      correlation > 0.7); routed to per-µservice owner + Council-
      Civil-Rights review.
  - name: DisparateImpactDetected
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Fairness.DisparateImpactDetected
    justification: >
      Emitted when 4/5ths rule fails (impact ratio < 0.8) for any
      protected class.
  - name: ExplanationProvided
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Fairness.ExplanationProvided
    justification: >
      Emitted when adverse-action decision is paired with specific-
      reasons explanation per ECOA Reg B / GDPR Art. 22 / EU AI Act
      Art. 13. Per-pack required.
  - name: PerJurisdictionVariantApplied
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Fairness.PerJurisdictionVariantApplied
    justification: >
      Emitted per inference identifying which jurisdiction-specific
      model variant was used (e.g., us-ecoa, eu-ai-act-art-5, kr-fcpa).
  - name: SocialScoringRefused
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Fairness.SocialScoringRefused
    justification: >
      Emitted when an inference is refused due to EU AI Act Art. 5
      social-scoring prohibition. Required regulator-facing audit
      trail.
  - name: protected-class-registry.json
    layer: N/A (JSON Schema spec)
    bnf_segments: spec.protected-class-registry
    justification: >
      JSON Schema declaring per-jurisdiction protected-class
      enumeration; per the §3.2.2 consistency invariant, registry
      is the single source of truth across every model card +
      fairness audit.
  - name: proxy-feature-prohibition-list.json
    layer: N/A (JSON Schema spec)
    bnf_segments: spec.proxy-feature-prohibition-list
    justification: >
      JSON Schema declaring the forbidden-without-explicit-justification
      list of proxy features (zip code, name, language, device).
      Adding a feature to the prohibition list is an ADR amendment.
---

# ADR-0309: Detection Fairness + Civil-Rights Compliance Baseline

## Status

Proposed — 2026-05-20.

Bundled with the keystone-bundle 2026-05-20 foundational doctrine
synthesis as the **drmp-detection-fairness-civil-rights** keystone,
closing the gap identified in `docs/standards/documentation-rigor.md`
§3.2.6 detection-fairness invariants (five invariants enumerated;
binding ADR called out as ADR-0309). This ADR is the binding ADR
row 51 of the §3.2.1 ADR-adherence matrix cites.

Enforcement is `advisory-until-2026-09-15-blocker-thereafter`. The
invariants accept in text immediately; the CI lanes that gate per-
class equity + proxy-feature prohibition + disparate-impact + per-
jurisdiction variant coverage promote to BLOCKER on 2026-09-15 to
coincide with the ADR-0307 detection substrate GA promotion (no
detection substrate goes GA without fairness invariants enforced).

## Date

2026-05-20.

## Context

### §A. Why fairness is a substrate-level commitment, not a per-model afterthought

Mature ML organizations treat fairness as a *first-class substrate
primitive* — wired centrally so every model serving production
traffic inherits a uniform fairness floor. The pattern is
unambiguous across the named industry references:

- **Microsoft Fairlearn (Azure ML Responsible AI dashboard).**
  Microsoft Research's open-source library (github.com/fairlearn/fairlearn,
  MIT license) implements ~12 fairness metrics + ~8 bias-mitigation
  algorithms; Azure ML's Responsible AI dashboard ships Fairlearn
  as the standard fairness toolkit. Per Microsoft's 2024 Responsible
  AI Transparency Report, every Azure ML production model passes
  Fairlearn-gated validation.
- **IBM AI Fairness 360 (AIF360).** github.com/Trusted-AI/AIF360
  (Apache 2.0) — ~70 fairness metrics + ~10 bias-mitigation
  algorithms; deployed across IBM's Watson AI portfolio + cross-
  industry adoption.
- **Google Responsible AI Practices.** ai.google/responsibility —
  publishes per-product fairness reports; Vertex AI Workbench's
  Responsible AI tab ships fairness + interpretability + counterfactual
  analysis.
- **Stripe Radar.** Per Stripe's 2024 ECOA-compliance disclosure,
  Radar runs quarterly fairness audit on every credit-impact
  model; per-class TPR/FPR equity maintained within ±2pp; specific-
  reasons explainability surfaced on every adverse-action notice.
- **Meta Civil Rights Audit.** Meta's 2020 Civil Rights Audit
  (about.fb.com/civilrightsaudit) established a per-product fairness
  review framework + Independent Civil Rights Officer role; per the
  2024 Civil Rights Annual Report, ~30 product domains audited
  annually.
- **Apple Fairness, Bias, and Transparency in AI.**
  apple.com/legal/privacy/aifairness — Apple's per-model fairness
  audit + explainability surface for high-impact decisions.
- **Anthropic Constitutional AI + bias evaluations.** Anthropic
  publishes per-Claude bias evaluations across protected classes
  (anthropic.com/news).
- **NIST Special Publication 1270** — "Towards a Standard for
  Identifying and Managing Bias in Artificial Intelligence" — sets
  the US federal floor for AI bias management.

The corollary: **every detection signal that drives an adverse
action MUST pass through the fairness substrate, not per-model
ad-hoc checks.** A model authored outside the substrate skips per-
class equity, skips proxy-detection, skips disparate-impact tests,
skips per-jurisdiction overlay — and ships an EU AI Act Art. 5
(prohibited practices) + Art. 10 (data quality) + Art. 13
(transparency) violation. The substrate shape closes the gap.

### §A.1. The five fairness invariants — regulatory anchors

Per documentation-rigor.md §3.2.6 detection-fairness invariants:

**Invariant 1 — No proxy discrimination.** Features that proxy
protected classes (zip code → race, name → ethnicity, language
preference → national origin) MUST be flagged and either excluded
or explicitly justified. Regulatory anchors:

- **Fair Housing Act + HUD's 2013 Disparate-Impact Rule** (24 CFR
  §100.500) — proxies for race/national-origin in housing decisions
  are unlawful; HUD's 2020 amendment was reversed in 2023 restoring
  the 2013 standard
- **ECOA + Reg B** (12 CFR §1002) — proxies for race/gender/age
  in credit decisions are unlawful
- **EU AI Act Art. 10** (data governance) — training data MUST be
  free from biases that lead to discrimination
- **GDPR Art. 9** — special category data + proxies for special
  category data trigger heightened protection
- **CFPB Circular 2022-03** — adverse-action notices for AI-driven
  credit decisions MUST include specific reasons (not just
  "complex algorithm")

**Invariant 2 — Per-class TPR/FPR equity within ±2pp baseline.**
True-positive rate + false-positive rate per protected class within
±2pp baseline; wider gaps require explicit ADR justification +
regulator notification. Regulatory anchors:

- **EU AI Act Art. 15** (accuracy, robustness, cybersecurity) —
  high-risk AI systems MUST achieve appropriate levels of accuracy
  consistent across all relevant subpopulations
- **NIST SP 1270 §4.2** — bias quantification with TPR/FPR equity
  is the federal-floor metric
- **NIST AI RMF 1.0 MEASURE 2.11** — fairness assessment metrics
  required
- **Microsoft Fairlearn equalized-odds metric** — operationalizes
  TPR/FPR equity

**Invariant 3 — Disparate-impact testing (4/5ths rule).**
Per Federal Uniform Guidelines on Employee Selection Procedures
(29 CFR §1607.4), selection rate for any protected class MUST be
≥80% of selection rate for majority class. Regulatory anchors:

- **29 CFR §1607.4(D)** — the 4/5ths rule (also "80% rule") as the
  federal-floor disparate-impact threshold
- **Griggs v. Duke Power Co. (1971)** — established disparate-
  impact doctrine in US case law
- **HUD's 2013 Disparate-Impact Rule (24 CFR §100.500)** — extends
  4/5ths to housing
- **EU Race Equality Directive 2000/43/EC** + **EU Employment
  Equality Directive 2000/78/EC** — EU-level disparate-impact
  prohibitions
- **KR National Human Rights Commission Act** — KR disparate-impact
  floor
- **JP Act on the Promotion of Equal Opportunity and Treatment** —
  JP disparate-impact floor

**Invariant 4 — Explainability floor.** Every adverse-action signal
(denied transaction, locked account, content removed) carries
human-readable explanation. Regulatory anchors:

- **ECOA Reg B §1002.9** — adverse-action notice MUST include
  specific reasons for denial within 30 days
- **GDPR Art. 13** — data subjects entitled to meaningful information
  about automated decision-making logic
- **GDPR Art. 22(3)** — data subjects entitled to obtain human
  intervention + express opinion + contest the decision
- **EU AI Act Art. 13** — high-risk AI systems MUST be designed for
  transparency
- **EU AI Act Art. 86** — right to meaningful explanation
- **NY AEDT Local Law 144 (2023)** — public audit-results notice
  + candidate notification
- **CFPB Circular 2022-03** — specific reasons required for AI-
  driven credit decisions

**Invariant 5 — Per-jurisdiction model variants.** Global model +
per-pack overlay enforced at evaluation time. Regulatory anchors:

- **EU AI Act Art. 5** — prohibited practices (social scoring,
  emotional recognition in workplace/education, untargeted face-
  recognition scraping for biometric databases) MUST be refused
- **KR Financial Consumer Protection Act Art. 30** — protected-class
  proxies in financial-decision ML are forbidden
- **NY AEDT Local Law 144 (2023)** — annual bias audit for
  AI-driven employment decision tools in NYC
- **California Civil Rights Department Automated-Decision-System
  regulation** — proposed 2024; per-California overlay required
- **California AB 2013 (2024)** — generative AI training-data
  transparency
- **Colorado AI Act (SB 24-205, 2026-02-01)** — high-risk AI
  documentation + consumer disclosure
- **Illinois BIPA** — biometric consent layer
- **JP APPI** — JP overlay
- **UK AADC** — minor-protection layer

### §A.2. The 14 EU Charter Art. 21 protected classes — EU baseline

EU Charter of Fundamental Rights Art. 21 (Non-discrimination, 2000):

> Any discrimination based on any ground such as sex, race,
> colour, ethnic or social origin, genetic features, language,
> religion or belief, political or any other opinion, membership
> of a national minority, property, birth, disability, age or
> sexual orientation shall be prohibited.

These 14 classes are the EU baseline. Per ADR-0240 sovereign-cloud
overlay, EU-cell deployments MUST audit per-class TPR/FPR equity
across all 14 classes; per-class data MUST be collected within
GDPR's special-category-data restrictions.

### §A.3. The US 7-class baseline — ECOA + Fair Housing + ADA

US federal civil-rights laws enumerate ≥7 protected classes
relevant to oyatie's detection substrate:

1. **Race + color** (Title VII, ECOA, Fair Housing Act)
2. **Sex/gender + sexual orientation + gender identity** (Title VII
   per Bostock v. Clayton County 2020; ECOA + Reg B post-2024)
3. **National origin + ancestry** (Title VII, ECOA, Fair Housing)
4. **Religion** (Title VII, Fair Housing)
5. **Age** (ADEA for ≥40; ECOA across age spectrum)
6. **Disability** (ADA, Section 504, Fair Housing FHA-amended)
7. **Genetic information** (GINA, ACA)
8. **Marital status** (ECOA Reg B)
9. **Receipt of public assistance** (ECOA Reg B)
10. **Veteran status** (USERRA, VEVRAA)
11. **Familial status** (Fair Housing)

### §A.4. KR + JP + UK + AU baselines

- **KR (Korean baseline).** Per KR Constitution Art. 11 (equality)
  + Equality Act (proposed) + Financial Consumer Protection Act
  Art. 30 + KR Equal Employment Opportunity Act + KR Disabilities
  Act: gender, age, religion, national origin, disability, marital
  status, family status. KR-specific: Real-Name Verification regime
  affects synthetic-identity detection.
- **JP (Japanese baseline).** Per Equal Employment Opportunity Act
  + Disabilities Discrimination Act + APPI: gender, age, nationality,
  disability. JP-specific: no comprehensive anti-discrimination law
  comparable to EU/US; sector-specific protections.
- **UK (UK baseline).** Per Equality Act 2010: 9 protected
  characteristics (age, disability, gender reassignment, marriage/
  civil partnership, pregnancy/maternity, race, religion or belief,
  sex, sexual orientation). UK AADC (2020) adds minor-protection
  layer.
- **AU (Australian baseline).** Per Sex Discrimination Act + Age
  Discrimination Act + Disability Discrimination Act + Racial
  Discrimination Act: race, sex, age, disability, marital status,
  pregnancy, sexual orientation, gender identity.

### §A.5. What this ADR explicitly does NOT do

ADR-0309 is the **fairness + civil-rights compliance baseline**. It
does NOT:

1. **Author the detection substrate runtime.** That lives in ADR-0307.
2. **Author the ML model lifecycle.** That lives in ADR-0308.
3. **Author the investigation case-management workflow.** That lives
   in ADR-0310.
4. **Author per-pack regulator notification cadence.** That lives in
   ADR-0251.
5. **Author the minor-protection doctrine.** That lives in ADR-0292.
6. **Replace per-µservice business logic.** Fairness invariants are
   substrate-level; per-µservice business logic implements them but
   does not author them.

## Decision

### §B. Five fairness invariants — substrate-enforced

Establish the canonical five fairness invariants as substrate-level
gates enforced before any ML model serves production traffic in
the detection substrate (per ADR-0307) or any other product-facing
AI surface.

### §B.1. Invariant 1 — No proxy discrimination

Features that proxy protected classes MUST be flagged + either
excluded or explicitly justified.

**Proxy-feature prohibition list** (per `proxy-feature-prohibition-list.json`):

| Feature | Proxies for | Default action | Override |
|---|---|---|---|
| `zip_code_5digit` | Race + national origin (US, post-Shelby County) | EXCLUDE from training | Explicit ADR amendment |
| `zip_code_3digit` | Race + national origin | EXCLUDE from training | Explicit ADR amendment |
| `first_name` | Ethnicity + national origin + gender | EXCLUDE from training | Explicit ADR amendment |
| `last_name` | Ethnicity + national origin + race | EXCLUDE from training | Explicit ADR amendment |
| `device_model` | Socio-economic class | FLAG; allow with monitoring | Per-jurisdiction override |
| `browser_language` | National origin | EXCLUDE from training | Explicit ADR amendment |
| `device_locale` | National origin | EXCLUDE from training | Explicit ADR amendment |
| `IP_geo_country` | National origin | FLAG; allow only for sanctions screening per OFAC | Per-rule justification |
| `payment_method_country` | National origin | FLAG; allow only for AML per FinCEN | Per-rule justification |
| `email_domain` | Provider preference → socio-economic | FLAG; allow with monitoring | Per-jurisdiction override |
| `phone_number_country_code` | National origin | FLAG; allow only for AML | Per-rule justification |
| `account_age_days` | Indirect age proxy if ≤16 | FLAG per minor-protection (ADR-0292) | Per-µservice review |
| `government_id_type` | National origin + immigration status | FLAG; allow only for KYC | Per-pack KYC obligation |
| `address_text` | Race + national origin + socio-economic | FLAG; tokenize before training | Per-pack tokenization |
| `surname_match_ethnicity_index` | Ethnicity | EXCLUDE; never train on this | (no override) |
| `voice_print_features` | Gender + age + accent | FLAG; allow only for ID liveness | Per-rule justification |
| `face_print_features` | Race + gender + age + ethnicity | FLAG; allow only for ID liveness + per-pack BIPA consent | Per-pack BIPA consent |

**Proxy detection methodology** (per `oya-shared-proxy-feature-detector`):

1. **Mutual-information score.** Compute MI between each candidate
   feature + each protected-class label (where label collected
   per consented self-report). MI > 0.3 → flagged as proxy.
2. **Correlation analysis.** Pearson correlation > 0.5 OR Spearman
   correlation > 0.5 → flagged.
3. **Counterfactual generation.** For tabular features, generate
   counterfactual examples flipping protected class; if feature
   distribution shifts significantly, flagged.
4. **Adversarial classifier.** Train a classifier predicting
   protected class from the feature alone; if accuracy > 70% +
   baseline ~50%, feature is a proxy.

**Per-jurisdiction override.** Per `oya-shared-per-jurisdiction-model-variant`,
some features that are proxies in one jurisdiction are required by
regulator in another (e.g., national origin for OFAC sanctions
screening, IP_geo_country for AML). The override is per-rule, not
per-model, and emits `ProxyFeatureDetected` audit event with
per-rule justification.

Audit event: `ProxyFeatureDetected` (per ADR-0263 registry).

### §B.2. Invariant 2 — Per-class TPR/FPR equity within ±2pp baseline

Per protected class per jurisdiction, true-positive rate + false-
positive rate within ±2pp of cross-class baseline. Wider gaps
require explicit ADR justification + regulator notification.

**Computation.** Per `oya-shared-fairness-audit` quarterly batch:

```
for protected_class in jurisdiction.protected_classes() {
  for class_value in protected_class.values() {
    let mask = held_out.where(protected_class == class_value);
    let tpr[class_value] = mask.tpr();
    let fpr[class_value] = mask.fpr();
  }
  let max_tpr_gap = max(tpr.values()) - min(tpr.values());
  let max_fpr_gap = max(fpr.values()) - min(fpr.values());
  assert max_tpr_gap <= 0.02;
  assert max_fpr_gap <= 0.02;
}
```

**Threshold rationale.** ±2pp is the substrate baseline; per-
jurisdiction overlay may tighten (e.g., HUD's disparate-impact
rule effectively demands tighter than ±2pp for housing). Wider
gaps:

- Gap > 2pp: ADR amendment required documenting why; per-pack
  regulator notification
- Gap > 5pp: per EU AI Act Art. 73 serious-incident report (24h
  cadence per ADR-0308 §D-7.2)
- Gap > 10pp: model rollback per ADR-0294 anomaly-rollback + ADR-0308

**Held-out set requirements.** Per ADR-0308 §B.2:

- ≥10⁴ samples per protected class
- Sourced from same Iceberg snapshot as training
- Stratified by jurisdiction

**Per-jurisdiction protected-class enumeration.** Per `protected-class-registry.json`:

```yaml
US:
  classes:
    - race
    - color
    - sex_gender_identity
    - sexual_orientation
    - national_origin
    - religion
    - age
    - disability
    - genetic_information
    - marital_status
    - veteran_status
    - familial_status
EU:
  classes:
    # 14 classes per Charter Art. 21
    - sex
    - race
    - color
    - ethnic_or_social_origin
    - genetic_features
    - language
    - religion_or_belief
    - political_opinion
    - national_minority_membership
    - property
    - birth
    - disability
    - age
    - sexual_orientation
KR:
  classes:
    - gender
    - age
    - religion
    - national_origin
    - disability
    - marital_status
    - family_status
JP:
  classes:
    - gender
    - age
    - nationality
    - disability
UK:
  classes:
    # 9 protected characteristics per Equality Act 2010
    - age
    - disability
    - gender_reassignment
    - marriage_or_civil_partnership
    - pregnancy_or_maternity
    - race
    - religion_or_belief
    - sex
    - sexual_orientation
AU:
  classes:
    - race
    - sex
    - age
    - disability
    - marital_status
    - pregnancy
    - sexual_orientation
    - gender_identity
```

Audit events: `FairnessAuditPassed`, `FairnessAuditFailed`.

### §B.3. Invariant 3 — Disparate-impact testing (4/5ths rule)

Per Federal Uniform Guidelines on Employee Selection Procedures
(29 CFR §1607.4(D)): selection rate for any protected class MUST
be ≥80% of selection rate for majority class.

**Computation.** Per `oya-shared-disparate-impact-tester`:

```
let selection_rates = {};
for class_value in protected_class.values() {
  let mask = held_out.where(protected_class == class_value);
  selection_rates[class_value] = mask.predicted_positive.count() / mask.count();
}
let majority_class_rate = selection_rates[protected_class.majority_value()];
for class_value in protected_class.values() {
  let ratio = selection_rates[class_value] / majority_class_rate;
  assert ratio >= 0.8;
}
```

**Per-jurisdiction equivalents.**

- US: 4/5ths rule (29 CFR §1607.4(D))
- EU: Race Equality Directive 2000/43/EC + Employment Equality
  Directive 2000/78/EC — "disparate impact" prohibition without
  specific numeric threshold; EU AI Act Art. 10 demands "appropriate
  measures to detect, prevent, and mitigate possible biases"
- KR: KR National Human Rights Commission Act + Equality Act
  (proposed) — disparate-impact prohibition
- JP: Act on the Promotion of Equal Opportunity and Treatment +
  Disability Discrimination Act — disparate-impact prohibition
- UK: Equality Act 2010 §19 — indirect discrimination prohibition

Audit event: `DisparateImpactDetected`.

**Adverse-outcome trigger.** Failure of 4/5ths rule:

- Triggers retraining queue per ADR-0308 §B.5
- Per-pack regulator notification per ADR-0251
- For high-risk models (per EU AI Act Annex III), 24h Art. 73
  serious-incident report

### §B.4. Invariant 4 — Explainability floor

Every adverse-action signal (denied transaction, locked account,
content removed) carries human-readable explanation. Per
`oya-shared-explainability-floor`.

**Specific-reasons surface.** Per ECOA Reg B + GDPR Art. 13/22 +
EU AI Act Art. 13:

```
function adverse_action_notice(decision) {
  let top_5 = shap_explainer.top_features(decision.prediction);
  let translated = translate_to_human_readable(top_5);
  let appeal_link = build_appeal_link(decision.decision_id);
  return {
    decision_id: decision.decision_id,
    decision: decision.action,
    specific_reasons: translated,  // ECOA Reg B §1002.9
    explanation_logic: "logistic-gradient-boosted-trees over verified-feature-set; see /transparency/model_id/{model_card_url}",
    appeal_link: appeal_link,
    appeal_sla: per_pack_sla(decision.affected_party.compliance_packs),
    binding_adrs: ["ADR-0307", "ADR-0308", "ADR-0309", "ADR-0310"],
  };
}
```

**Human-readable feature translation.** Per
`microservices/detection/src/explainability/human_translation.rs`:

- `feature.payment.velocity_24h` → "The number of charges on this
  card in the last 24 hours is unusually high (top 1% of
  legitimate charges)"
- `feature.identity.tls_fingerprint_drift` → "The device fingerprint
  on this sign-in differs from your previous sign-ins"
- `feature.aml.sanctions_match_score` → "The recipient name matched
  a sanctions list entry with high confidence"
- `feature.content.photodna_match` → "The image matched a known
  CSAM hash from NCMEC's PhotoDNA database"

Feature translations are reviewed for plain-language clarity (per
EU AI Act Art. 13 "in a clear and comprehensive manner" + GDPR
Art. 12 "intelligible and easily accessible form").

**Explanation logic surface.** Per EU AI Act Art. 13(3)(a) "the
characteristics, capabilities and limitations of the AI system" +
Art. 13(3)(b)(ii) "an indication of the changes in performance":

- Per-model card URL pointing to `/transparency/<model_id>/<version>/`
- Model card per Google Model Card template (ADR-0308 §B.6)
- Per-version SHAP/LIME explainability surface

**Refusal of unexplainable models.** Per ADR-0309 + EU AI Act Art.
13, models without explainability surface (e.g., pure neural networks
without SHAP/LIME wrappers) MUST NOT serve adverse-action decisions.
Per `oya-governance-detection-fairness-explainability-floor`
CI lane.

Audit event: `ExplanationProvided`.

### §B.5. Invariant 5 — Per-jurisdiction model variants

Global model + per-pack overlay enforced at evaluation time. Per
`oya-shared-per-jurisdiction-model-variant`.

**Per-jurisdiction overlay shape.** Per `per-jurisdiction-model-variant-schema.json`:

```yaml
model_id: payment_fraud_v3.2.1
base_model_uri: "iceberg://prod/models/payment_fraud_v3.2.1"
variants:
  - jurisdiction: US
    feature_mask: us-ecoa-reg-b-mask
    threshold_overlay: us-default
    appeal_sla_days: 30
    public_notice_required: true  # NY AEDT for employment-decision
    forbidden_features:
      - first_name
      - last_name
      - zip_code_5digit
      - browser_language
  - jurisdiction: EU
    feature_mask: eu-ai-act-art-5-mask
    threshold_overlay: eu-default
    appeal_sla_days: 30
    public_notice_required: false  # default; per AI Act Art. 13 transparency
    forbidden_features:
      - first_name
      - last_name
      - genetic_features
      - sexual_orientation_inference
    refused_actions:
      - social_scoring
      - emotion_recognition_workplace_education
      - untargeted_face_recognition_scraping
  - jurisdiction: KR
    feature_mask: kr-fcpa-art-30-mask
    threshold_overlay: kr-fsc-default
    appeal_sla_days: 60
    public_notice_required: false
    forbidden_features:
      - first_name
      - last_name
      - real_name_verification_alternative  # KR-specific
  - jurisdiction: JP
    feature_mask: jp-appi-mask
    threshold_overlay: jp-default
    appeal_sla_days: 30
    public_notice_required: false
  - jurisdiction: UK
    feature_mask: uk-equality-act-mask
    threshold_overlay: uk-default
    appeal_sla_days: 30
    public_notice_required: false
  - jurisdiction: AU
    feature_mask: au-privacy-act-mask
    threshold_overlay: au-default
    appeal_sla_days: 30
    public_notice_required: false
  - jurisdiction: CA
    feature_mask: ca-ab-2013-mask
    threshold_overlay: ca-default
    appeal_sla_days: 30
    public_notice_required: true  # CA-specific transparency
  - jurisdiction: NY
    feature_mask: ny-aedt-mask
    threshold_overlay: ny-default
    appeal_sla_days: 30
    public_notice_required: true  # NY AEDT Local Law 144 (2023)
  - jurisdiction: CO
    feature_mask: co-ai-act-mask
    threshold_overlay: co-default
    appeal_sla_days: 30
    public_notice_required: true  # CO AI Act effective 2026-02-01
```

**EU AI Act Art. 5 — prohibited practices refusal.** Per
`oya-shared-per-jurisdiction-model-variant`:

```
function evaluate(model_id, features, jurisdiction) {
  let variant = per_jurisdiction_variant_registry.fetch(model_id, jurisdiction);
  if features.action_class in variant.refused_actions {
    audit.emit("SocialScoringRefused", { model_id, action_class, jurisdiction });
    return Refuse {
      reason: "EU AI Act Art. 5 prohibited practice",
      action_class: features.action_class,
    };
  }
  let masked_features = apply_feature_mask(features, variant.feature_mask);
  return base_model.predict(masked_features);
}
```

**KR FCPA Art. 30 — protected-class restrictions.** Per the
2021-03-25 effective Financial Consumer Protection Act Art. 30,
financial-decision ML in KR cannot use:

- Customer's nationality, race, ethnicity (except as required by
  regulator for sanctions screening)
- Customer's gender (except as KR-specific Real-Name Verification
  requires)
- Customer's marital status, family structure
- Customer's religion, political affiliation
- Customer's disability status

The kr-fcpa-art-30-mask enforces this at inference time.

**NY AEDT Local Law 144 (2023) — annual bias audit + public
notice.** Per the 2023-07-05 effective Local Law 144, AI-driven
employment decision tools serving NYC candidates require:

- Annual bias audit by independent auditor
- Public posting of audit results
- Candidate notification of AI use ≥10 business days before
  assessment

The ny-aedt-mask enforces public-notice cadence + the
`oya-governance-detection-fairness-quarterly-cadence-public-notice`
CI lane gates emission.

Audit event: `PerJurisdictionVariantApplied`.

## §C. Consequences

The 6 engineering-rigor dimensions per documentation-rigor.md §1.2:

### §C.1. Maintainability dimension

The fairness substrate's maintainability surface is concentrated
in single-concern crates (oya-shared-fairness-audit,
-proxy-feature-detector, -disparate-impact-tester, -explainability-floor,
-per-jurisdiction-model-variant, -protected-class-registry) under
`crates/`. Per-jurisdiction overlay configurations live in
`/specs/per-jurisdiction-model-variant-schema.json` instances under
`microservices/detection/per-jurisdiction-variants/`.

Versioning policy: every crate SemVer per ADR-0258; the
protected-class-registry.json + proxy-feature-prohibition-list.json
+ per-jurisdiction-model-variant-schema.json are versioned via
`_meta.schema_version`; breaking changes require ADR amendment +
60-day deprecation; additive (new jurisdiction added) is minor;
breaking (jurisdiction removed) is major.

Per-config-flag rationale: ~40 per-tenant config flags (per-
jurisdiction overlay selection, per-pack regulator threshold
override, per-protected-class equity tolerance, per-feature
prohibition override, per-jurisdiction appeal SLA). Each flag has
a documented default + per-pack override behavior. Audited daily
by `oya-governance-fairness-config-flag-coherence`.

Reverse dependencies: every ML-serving µservice depends on the
fairness substrate. Initial dependents: detection (all 8 families),
intelligence, ops-dashboard-control-center, marketplace, social,
shorts, workflow-studio.

### §C.2. Observability dimension

Per ADR-0263 emission contract, the fairness substrate emits 7
new audit-event-classes registered in the central registry:

| Class | Cardinality budget | Trace span shape | Retention |
|---|---|---|---|
| `FairnessAuditPassed` | ~4/quarter × ~30 models × ~10 jurisdictions = ~1200/yr | Parent: fairness-audit-batch; Child: per-metric-evaluation | 10-year cold (EU AI Act Art. 18) |
| `FairnessAuditFailed` | ~10/quarter | Parent: fairness-audit-batch | 10-year cold + regulator-facing surface |
| `ProxyFeatureDetected` | ~10/quarter | Parent: proxy-feature-detector-batch | 10-year cold |
| `DisparateImpactDetected` | ~10/quarter | Parent: disparate-impact-tester-batch | 10-year cold |
| `ExplanationProvided` | ~10⁵/day (every adverse action) | Parent: adverse-action-controller | 90-day hot + 7-year cold |
| `PerJurisdictionVariantApplied` | ~10⁹/day (every inference) | Parent: per-jurisdiction-variant-evaluator | 30-day hot |
| `SocialScoringRefused` | ~0/day expected; <10/quarter | Parent: per-jurisdiction-variant-evaluator | 10-year cold + EU regulator surface |

Metrics (Prometheus + OpenTelemetry per ADR-0263):

- `fairness_tpr_per_class{model_id, protected_class, class_value, jurisdiction}` — gauge
- `fairness_fpr_per_class{model_id, protected_class, class_value, jurisdiction}` — gauge
- `fairness_disparate_impact_ratio{model_id, protected_class, jurisdiction}` — gauge; alert at < 0.8
- `fairness_proxy_feature_correlation{feature_id, protected_class, jurisdiction}` — gauge
- `fairness_explanation_latency_seconds{model_id, p50|p95|p99}` — histogram; P99 ≤ 500ms
- `fairness_explanation_present_total{model_id, family}` — counter; SLA: 100% adverse-action coverage
- `fairness_audit_passed_total{model_id, jurisdiction, quarter}` — counter
- `fairness_audit_failed_total{model_id, jurisdiction, quarter}` — counter
- `fairness_per_jurisdiction_variant_applied_total{model_id, jurisdiction}` — counter
- `fairness_social_scoring_refused_total{jurisdiction, action_class}` — counter

Dashboards (Grafana):

1. `fairness-audit-overview.json` — per-model per-jurisdiction quarterly audit results
2. `fairness-per-class-equity.json` — per-class TPR/FPR over time
3. `fairness-disparate-impact-ratio.json` — 4/5ths rule compliance
4. `fairness-proxy-feature-detection.json` — proxy-detection events + correlation scores
5. `fairness-explainability-coverage.json` — per-model adverse-action explanation coverage
6. `fairness-per-jurisdiction-variant.json` — per-jurisdiction variant usage + refusal events
7. `fairness-ny-aedt-public-notice.json` — NY AEDT compliance
8. `fairness-eu-ai-act-art-5-refusal.json` — EU prohibited-practice refusal log

SLO floor (per `microservices/detection/slos/*.openslo.yaml`):

- Quarterly fairness audit emission within ±15d window; 100% (BLOCKER)
- Per-class TPR/FPR equity ≤ ±2pp; 99% (BLOCKER on exceedance)
- 4/5ths rule disparate-impact ratio ≥ 0.8; 99% (BLOCKER on exceedance)
- Adverse-action explanation present; 100% (BLOCKER)
- NY AEDT public notice posted quarterly; 100% (BLOCKER)
- EU AI Act Art. 5 prohibited-practice refusal; 100% (BLOCKER)
- KR FCPA Art. 30 financial protected-class proxies excluded; 100% (BLOCKER)

### §C.3. Scalability dimension

Capacity math per documentation-rigor.md §1.1 item 3:

**Fairness audit batch.** Per-quarterly per-model per-jurisdiction;
~30 models × ~10 jurisdictions × 4 quarters = ~1200 jobs/year ≈
~3/day average ≈ ~10-15/day peak. Per-job runtime ≤6h on Spark
cluster; cluster capacity sized for ≥15 parallel jobs.

**Proxy-feature detection batch.** Per-quarterly per-feature × ~500
features × 4 quarters = ~2000 jobs/year. Per-job runtime ≤2h.

**Disparate-impact testing.** Co-runs with fairness audit; no
additional compute.

**Explainability surface (online).** ~10⁵/day adverse-action
explanations at platform GA × ~50ms per SHAP-explainer-run = ~83min/
day CPU = trivial compared to inference cost.

**Per-jurisdiction variant evaluation (online).** Per ADR-0307 §C.3,
amortized into inference; ~10ns per-jurisdiction-mask-lookup.

10× and 100× scale-out path: per-jurisdiction variant evaluation
is the only online primitive; scales horizontally via stateless
evaluator replicas. Quarterly batches scale via additional Spark
executor capacity.

### §C.4. Performance dimension

| Primitive | P50 | P95 | P99 | Tail mitigation |
|---|---|---|---|---|
| Quarterly fairness audit (per model per jurisdiction) | 3h | 5h | 6h | Per-jurisdiction parallel runs |
| Proxy-feature detection (per quarter) | 1h | 1.5h | 2h | Per-feature parallel |
| Adverse-action explanation generation | 10ms | 100ms | 500ms | Pre-warmed SHAP TreeExplainer |
| Per-jurisdiction variant evaluation (online) | 0.5µs | 2µs | 10µs | In-memory mask cache |
| Adverse-action notice emission | 50ms | 200ms | 500ms | Async notify + audit-emit |

Per-region budget split: per ADR-0240 sovereign-cloud overlay, EU
cells run independent quarterly audits from US cells; per-pack
residency honored.

Cold-start budget: per-jurisdiction variant cold-start ≤1s
(in-memory mask cache warm).

### §C.5. Optimization dimension

Per-stage cost model:

- Quarterly fairness audit: ~$100-300 per job × ~1200 jobs/year =
  ~$120k-$360k/year platform-wide
- Proxy-feature detection: ~$50-100 per quarter per feature × ~500
  features × 4 quarters = ~$100k-$200k/year
- Online explainability: ~$0.0001 per explanation × 10⁵/day =
  ~$30/month per cell
- Per-jurisdiction variant evaluation: amortized; ~$0 incremental

Lazy vs eager trade-offs:

- **Eager** for quarterly fairness audit (regulator-mandated cadence)
- **Eager** for adverse-action explanation (required at notice time)
- **Cached** for per-jurisdiction variant mask (in-memory cache;
  invalidated on variant update)
- **Lazy** for ad-hoc fairness queries (on-demand investigation per
  ADR-0310)

Cold-vs-warm path latency: cold (first variant lookup after deploy)
≈ 5ms (mask load from spec); warm ≈ 0.5µs.

### §C.6. Code quality dimension

Per documentation-rigor.md §1.2:

- **Test classes:** unit (per-fairness-metric, per-proxy-detector,
  per-variant-evaluator), property-based (proptest on equity
  invariants), fuzz (proxy-feature-prohibition-list parser), load
  (variant evaluation at ~10⁹/day), e2e (full quarterly-audit cycle
  for synthetic-fraud-detection model)
- **Coverage floor:** ≥85% line, ≥75% branch
- **Lint passes:** `cargo clippy -- -D warnings`,
  `oya-check-protected-class-registry-conformance`,
  `oya-check-proxy-feature-prohibition-list-conformance`,
  `oya-check-naming-bnf-v4`, `oya-check-layer-enum-conformance`,
  `oya-governance-detection-fairness-audit`
- **Type-strictness:** Rust `deny(warnings)` + `deny(unsafe_code)`
- **SemVer + ABI policy:** per ADR-0258

## §D. Detailed mechanics

### §D-1. Proxy-feature detection methodology

#### §D-1.1. Mutual-information score

Per `oya-shared-proxy-feature-detector`:

```
function compute_mutual_information(feature, protected_class_label) {
  let joint = histogram_2d(feature, protected_class_label);
  let p_feature = histogram(feature).normalize();
  let p_class = histogram(protected_class_label).normalize();
  let mi = 0.0;
  for (f_val, c_val) in joint.cells() {
    let p_joint = joint[f_val, c_val] / total;
    if p_joint > 0 {
      mi += p_joint * log(p_joint / (p_feature[f_val] * p_class[c_val]));
    }
  }
  return mi;
}
```

Threshold: MI > 0.3 → flagged as proxy.

Sourced from where the protected-class label is collected: per-
tenant pack overlay declares which classes are collectable + with
what consent mechanism (e.g., HMDA for US mortgage; EU AI Act Art.
10(5) allows special-category-data processing for bias detection
with safeguards).

#### §D-1.2. Counterfactual generation

For tabular features, generate counterfactual examples where
protected class is flipped (other features held constant via
nearest-neighbor sampling):

```
function counterfactual_distribution_shift(feature, protected_class) {
  let original = sample(held_out);
  let counterfactual = nearest_neighbors_with_flipped_class(original, protected_class);
  let shift = wasserstein_distance(feature(original), feature(counterfactual));
  return shift;
}
```

Threshold: shift > 0.5 → flagged.

#### §D-1.3. Adversarial classifier

Train a classifier predicting protected class from feature alone:

```
function adversarial_classifier_accuracy(feature, protected_class_label) {
  let X = feature.values();
  let y = protected_class_label.values();
  let clf = LogisticRegression().fit(X, y);
  return clf.accuracy(X_holdout, y_holdout);
}
```

Threshold: accuracy > 70% AND baseline ~50% → feature is proxy.

### §D-2. Per-class TPR/FPR equity mechanics

#### §D-2.1. Per-jurisdiction protected-class-label sourcing

Per ADR-0099 data-class registry + per ADR-0244 tenant scoping +
per ADR-0292 minor-protection:

- Self-reported (consented) — preferred; per-tenant opt-in via
  `/account/demographics` surface
- Inferred (counterfactual + adversarial-classifier-trained) —
  fallback for fairness-audit-only; never used for model prediction
- Per-pack restriction — HIPAA-pack tenants cannot expose health-
  related demographic data; COPPA-pack tenants don't collect for
  <13 users

#### §D-2.2. Per-class held-out set construction

```
function build_held_out_set(model_id, jurisdiction, protected_class) {
  let held_out_window = training_window.end - 7d to training_window.end;
  let held_out = iceberg.scan(
    table: "audit_features",
    snapshot: training_data_snapshot,
    predicate: timestamp IN held_out_window
                AND jurisdiction == @jurisdiction
                AND protected_class_label IS NOT NULL
  );
  let stratified = stratified_sample(held_out, protected_class, min_per_class=10_000);
  return stratified;
}
```

#### §D-2.3. Per-class TPR/FPR computation

```
for class_value in protected_class.values() {
  let mask = held_out.where(protected_class_label == class_value);
  let predictions = model.predict(mask.features);
  let tp = (predictions.positive AND mask.true_positive).count();
  let fn = (predictions.negative AND mask.true_positive).count();
  let fp = (predictions.positive AND NOT mask.true_positive).count();
  let tn = (predictions.negative AND NOT mask.true_positive).count();
  let tpr = tp / (tp + fn);
  let fpr = fp / (fp + tn);
  metrics[class_value] = { tpr, fpr };
}
```

#### §D-2.4. Equity assessment

```
let max_tpr_gap = max(metrics.tpr.values()) - min(metrics.tpr.values());
let max_fpr_gap = max(metrics.fpr.values()) - min(metrics.fpr.values());
if max_tpr_gap > 0.02 || max_fpr_gap > 0.02 {
  audit.emit("FairnessAuditFailed", {
    model_id, jurisdiction, protected_class,
    max_tpr_gap, max_fpr_gap, metrics
  });
  retraining_queue.enqueue(model_id, fairness_feedback={ protected_class });
  per_pack_regulator.notify(model_id, jurisdiction, protected_class);
  if max_tpr_gap > 0.05 || max_fpr_gap > 0.05 {
    eu_ai_act_art_73.report_serious_incident(model_id, jurisdiction, protected_class);
  }
  if max_tpr_gap > 0.10 || max_fpr_gap > 0.10 {
    model_rollback.rollback(model_id);
  }
} else {
  audit.emit("FairnessAuditPassed", { model_id, jurisdiction, protected_class, metrics });
}
```

### §D-3. Disparate-impact testing mechanics

#### §D-3.1. 4/5ths rule computation

```
function disparate_impact_ratio(held_out, protected_class) {
  let selection_rates = {};
  for class_value in protected_class.values() {
    let mask = held_out.where(protected_class_label == class_value);
    let positive = (model.predict(mask.features) == "positive").count();
    selection_rates[class_value] = positive / mask.count();
  }
  let majority_rate = selection_rates[protected_class.majority_value()];
  let min_ratio = min(selection_rates.values()) / majority_rate;
  return { ratio: min_ratio, selection_rates };
}
```

#### §D-3.2. Per-jurisdiction overlay

- US: 4/5ths rule applies
- EU: prohibition without specific numeric threshold; oyatie
  applies 4/5ths as a hard floor + tighter equity from Invariant 2
- KR + JP + UK + AU: 4/5ths applied uniformly

#### §D-3.3. Adverse-outcome trigger

```
let di = disparate_impact_ratio(held_out, protected_class);
if di.ratio < 0.8 {
  audit.emit("DisparateImpactDetected", {
    model_id, jurisdiction, protected_class, ratio: di.ratio, selection_rates: di.selection_rates
  });
  retraining_queue.enqueue(model_id, fairness_feedback={ protected_class });
  per_pack_regulator.notify(...);
}
```

### §D-4. Explainability surface mechanics

#### §D-4.1. SHAP TreeExplainer integration

For LightGBM models (the §B.5 ADR-0308 default):

```
function explain(model, features) {
  let explainer = SHAP::TreeExplainer::new(model);
  let shap_values = explainer.shap_values(features);
  let top_5 = top_n_by_abs(shap_values, 5);
  return top_5.map(|(feature_id, shap_value)| {
    SpecificReason {
      feature_id,
      contribution: shap_value,
      direction: shap_value > 0 ? "increased risk" : "decreased risk",
      human_readable: feature_translator.translate(feature_id),
    }
  });
}
```

#### §D-4.2. LIME integration for non-tree models

For neural-network or non-tree models:

```
function explain_lime(model, features) {
  let explainer = LIME::TabularExplainer::new(training_data_summary);
  let explanation = explainer.explain_instance(features, model);
  let top_5 = top_n_by_abs(explanation, 5);
  return top_5;
}
```

#### §D-4.3. Adverse-action notice mechanics

Per ECOA Reg B §1002.9 + GDPR Art. 22:

```
function emit_adverse_action_notice(decision, affected_party, jurisdiction) {
  let specific_reasons = explainability_floor.explain(decision.model, decision.features);
  let appeal_link = appeal_mechanism.build_link(decision.decision_id, affected_party, jurisdiction);
  let notice = AdverseActionNotice {
    decision_id: decision.decision_id,
    decision: decision.action,
    specific_reasons,
    explanation_logic_url: format!("/transparency/{}/{}", decision.model.model_id, decision.model.version),
    appeal_link,
    appeal_sla: per_pack_sla(affected_party.compliance_packs, jurisdiction),
    binding_adrs: ["ADR-0307", "ADR-0308", "ADR-0309", "ADR-0310"],
    affected_party_id: affected_party.id,
  };
  notify_per_pack(notice, affected_party, jurisdiction);
  audit.emit("ExplanationProvided", { ... });
}
```

#### §D-4.4. Per-jurisdiction notice formatting

- US ECOA: notice delivered within 30 days
- EU GDPR Art. 12: 1 month, extendable +2 months on complexity
- NY AEDT: ≥10 business days advance notice for employment-decision
- KR FCPA: per-FSS cadence
- JP APPI: per-APPI guidelines

### §D-5. Per-jurisdiction model variant mechanics

#### §D-5.1. Variant fetching

Per `microservices/detection/per-jurisdiction-variants/`:

```
function fetch_variant(model_id, jurisdiction) {
  let registry = per_jurisdiction_variant_registry.fetch(model_id);
  let variant = registry.variants.find(|v| v.jurisdiction == jurisdiction);
  if variant.is_none() {
    audit.emit("FairnessAuditFailed", {
      reason: "no per-jurisdiction variant",
      model_id, jurisdiction
    });
    return Refuse {
      reason: "Model not authorized for this jurisdiction",
      jurisdiction,
    };
  }
  return variant.unwrap();
}
```

#### §D-5.2. Feature mask application

```
function apply_feature_mask(features, feature_mask) {
  for forbidden_feature in feature_mask.forbidden_features {
    features[forbidden_feature] = NULL;
  }
  for tokenized_feature in feature_mask.tokenized_features {
    features[tokenized_feature] = tokenize(features[tokenized_feature]);
  }
  return features;
}
```

#### §D-5.3. EU AI Act Art. 5 refusal

```
if features.action_class in variant.refused_actions {
  audit.emit("SocialScoringRefused", {
    model_id, action_class: features.action_class, jurisdiction
  });
  per_pack_regulator.notify_prohibited_practice_attempt(...);
  return Refuse {
    reason: "EU AI Act Art. 5 prohibited practice",
    action_class,
  };
}
```

The "refused_actions" list per EU AI Act Art. 5:

- `social_scoring`
- `emotion_recognition_workplace_education`
- `untargeted_face_recognition_scraping`
- `biometric_categorization_inferring_sensitive_attributes`
- `predictive_policing_natural_persons`
- `subliminal_techniques_manipulating_behavior`

#### §D-5.4. KR FCPA Art. 30 enforcement

```
let kr_fcpa_art_30_mask = FeatureMask {
  forbidden_features: [
    "customer_nationality_or_race_inferred",
    "customer_ethnicity_inferred",
    "customer_gender_for_credit_decision",
    "customer_marital_status_for_credit_decision",
    "customer_family_structure",
    "customer_religion",
    "customer_political_affiliation",
    "customer_disability_status_for_credit_decision",
  ],
};
```

#### §D-5.5. NY AEDT public-notice mechanics

Per Local Law 144 (2023):

```
function ny_aedt_publish_audit_notice(model_id, audit_results) {
  let notice = NyAedtBiasAuditNotice {
    model_id,
    used_for: "employment_decision",
    most_recent_audit_date: now(),
    auditor_name: independent_auditor.name(),
    auditor_certification: independent_auditor.certification_url(),
    selection_rate_max: audit_results.max_selection_rate,
    selection_rate_min: audit_results.min_selection_rate,
    impact_ratio: audit_results.disparate_impact_ratio,
    scoring_rate_max: audit_results.max_scoring_rate,
    scoring_rate_min: audit_results.min_scoring_rate,
    scoring_impact_ratio: audit_results.scoring_impact_ratio,
    protected_classes_covered: audit_results.protected_classes,
  };
  publish_to_transparency_surface(notice);
  archive_to_microservices_detection_transparency_reports(notice);
  audit.emit("FairnessAuditPassed", notice);
}
```

### §D-6. Per-jurisdiction packed-overlay mechanics

Per ADR-0251 compliance-pack overlay shape:

```yaml
compliance_pack: EU-AI-Act
overlay:
  - apply_eu_ai_act_art_5_refusal
  - apply_eu_charter_art_21_14_class_audit
  - apply_eu_ai_act_art_18_10_year_retention
  - apply_gdpr_art_22_human_intervention
  - apply_gdpr_art_13_meaningful_explanation
compliance_pack: US-ECOA-Reg-B
overlay:
  - apply_us_protected_class_registry
  - apply_ecoa_reg_b_specific_reasons
  - apply_4_5ths_rule
  - apply_30_day_adverse_action_notice
compliance_pack: NY-AEDT-Local-Law-144
overlay:
  - apply_ny_aedt_annual_bias_audit
  - apply_ny_aedt_public_notice
  - apply_ny_aedt_candidate_notification
compliance_pack: KR-FCPA-Art-30
overlay:
  - apply_kr_fcpa_art_30_mask
  - apply_kr_fcpa_appeal_sla_60d
compliance_pack: JP-APPI
overlay:
  - apply_jp_appi_mask
  - apply_jp_appi_explainability
compliance_pack: UK-AADC-Equality-Act-2010
overlay:
  - apply_uk_equality_act_9_class_audit
  - apply_uk_aadc_minor_protection
```

### §D-7. Quarterly cadence + public-notice mechanics

#### §D-7.1. Cron schedule

```
cron: "0 0 1 */3 *"  # 00:00 on day 1 of every 3rd month
```

#### §D-7.2. Per-jurisdiction batch fan-out

Per `microservices/detection/batch/fairness_quarterly.scala`:

```scala
val jurisdictions = List("US", "EU", "KR", "JP", "UK", "AU", "CA", "NY", "CO")
val models = model_registry.list_production_models()
jurisdictions.par.foreach { jurisdiction =>
  models.par.foreach { model =>
    val variant = per_jurisdiction_variant_registry.fetch(model.id, jurisdiction)
    if (variant.is_some()) {
      val audit = run_fairness_audit(model, jurisdiction, variant.get())
      if (audit.passed) {
        emit_audit_passed_event(audit)
      } else {
        emit_audit_failed_event(audit)
        retraining_queue.enqueue(model.id, jurisdiction)
        per_pack_regulator.notify(...)
      }
      if (jurisdiction.requires_public_notice()) {
        publish_to_transparency_surface(audit)
      }
    }
  }
}
```

#### §D-7.3. Public-notice transparency surface

Per `microservices/detection/transparency-reports/`:

```
/transparency/
├── per-model/
│   ├── payment_fraud_v3.2.1/
│   │   ├── 2026-Q2-US.yaml
│   │   ├── 2026-Q2-EU.yaml
│   │   ├── 2026-Q2-NY-aedt.yaml
│   │   └── ...
│   └── ...
├── per-jurisdiction/
│   ├── US/
│   ├── EU/
│   ├── NY-aedt/
│   └── ...
└── per-quarter/
    ├── 2026-Q2-overview.yaml
    └── ...
```

## §E. Implementation footprint

### §E.1. New crates (per layer-5 shared-substrate)

Per ADR-0105 13-layer canonical enum row 5:

1. `crates/oya-shared-fairness-audit/` — quarterly fairness audit batch + per-class TPR/FPR + equity assessment
2. `crates/oya-shared-proxy-feature-detector/` — mutual-information + counterfactual + adversarial-classifier proxy detection
3. `crates/oya-shared-disparate-impact-tester/` — 4/5ths rule + per-jurisdiction overlay
4. `crates/oya-shared-explainability-floor/` — SHAP/LIME + per-feature human translation
5. `crates/oya-shared-per-jurisdiction-model-variant/` — variant registry + feature mask + per-jurisdiction overlay
6. `crates/oya-shared-protected-class-registry/` — per-jurisdiction enumeration + per-class collection-consent surface

### §E.2. New JSON Schemas

Under `/specs/`:

1. `protected-class-registry.json` — per-jurisdiction protected-class enumeration
2. `proxy-feature-prohibition-list.json` — forbidden + flagged proxy features
3. `per-jurisdiction-model-variant-schema.json` — variant shape
4. `fairness-audit-schema.json` — audit-result shape
5. `disparate-impact-test-schema.json` — 4/5ths rule test shape
6. `civil-rights-pack-overlay-schema.json` — per-pack overlay shape

### §E.3. New µservice extensions

`microservices/detection/`:

```
microservices/detection/
├── per-jurisdiction-variants/    # NEW: per-model per-jurisdiction
│   ├── payment_fraud_v3.2.1/
│   │   ├── US.yaml
│   │   ├── EU.yaml
│   │   ├── KR.yaml
│   │   ├── JP.yaml
│   │   ├── UK.yaml
│   │   ├── AU.yaml
│   │   ├── CA.yaml
│   │   ├── NY-aedt.yaml
│   │   └── CO.yaml
│   └── ...
├── transparency-reports/          # NEW: per-quarter per-jurisdiction
│   ├── 2026-Q2-payment_fraud-US.yaml
│   ├── 2026-Q2-payment_fraud-EU.yaml
│   ├── 2026-Q2-payment_fraud-NY-aedt.yaml
│   └── ...
├── runbooks/
│   ├── fairness-audit-failure-tpr-fpr-gap.md       # NEW
│   ├── fairness-audit-failure-disparate-impact.md  # NEW
│   ├── fairness-proxy-feature-detected.md          # NEW
│   ├── fairness-eu-ai-act-art-5-refusal.md         # NEW
│   ├── fairness-ny-aedt-public-notice.md           # NEW
│   ├── fairness-explainability-floor-breach.md     # NEW
│   └── fairness-per-jurisdiction-variant-missing.md # NEW
└── policy/
    ├── fairness-feature-mask.cedar                  # NEW: per-jurisdiction Cedar gate
    ├── fairness-protected-class-access.cedar       # NEW: per-class data access Cedar gate
    └── ...
```

### §E.4. New runbooks

7 new runbooks (listed above); each per §2 runbook rigor.

### §E.5. New CI lanes

- `oya-governance-detection-fairness-proxy-feature-prohibition` — proxy-feature list enforced
- `oya-governance-detection-fairness-per-class-tpr-fpr-equity` — ±2pp equity baseline
- `oya-governance-detection-fairness-disparate-impact-4-5ths-rule` — 4/5ths rule
- `oya-governance-detection-fairness-explainability-floor` — adverse-action explanation present
- `oya-governance-detection-fairness-per-jurisdiction-variant-coverage` — per-model per-jurisdiction variant present
- `oya-governance-detection-fairness-eu-ai-act-art-5-social-scoring-forbidden` — Art. 5 refusal wired
- `oya-governance-detection-fairness-kr-fcpa-art-30-financial-protected-class-forbidden` — KR FCPA mask applied
- `oya-governance-detection-fairness-quarterly-cadence-public-notice` — quarterly cadence + NY AEDT public notice
- Aggregate: `oya-governance-detection-fairness`

### §E.6. Per-µservice extensions (consumers)

Every µservice serving any model updates:

- `compliance.md §detection-fairness-audit` — per row 51 of §3.2.1
- `manifest.json:fairness_audit_required_jurisdictions[]` — per-µservice jurisdiction roster
- `manifest.json:per_jurisdiction_variant_registry_uri` — registry path

### §E.7. Vendor selection rationale

#### §E.7.1. Fairness toolkit: Fairlearn + AIF360 (parity)

Both used for parity-check; in-house implementations in
`oya-shared-fairness-audit` per ADR-0211 in-house preference.

#### §E.7.2. Independent auditor for NY AEDT

Per NY AEDT Local Law 144 (2023), annual bias audit MUST be by
independent auditor. Acceptable auditors per NYC DCWP guidance:
ARC AI Audit, ORCAA, Holistic AI, Babl AI, others on DCWP-published
list. Per-pack selection lives in compliance.md.

#### §E.7.3. Per-jurisdiction protected-class data sourcing

- US: self-report under HMDA; ECOA permits collection for fairness
  monitoring with safeguards
- EU: special-category-data under GDPR Art. 9(2)(b) employment +
  Art. 9(2)(j) substantial public interest + EU AI Act Art. 10(5)
  for bias detection
- KR: per KR-PIPA pseudonymization + KR-FSC fairness-monitoring
  authorization
- JP: per APPI consent
- UK: per Equality Act 2010 §158 + Equality Duty
- AU: per Privacy Act consent

## §F. Migration

### §F.1. Wave-3-D rollout sequencing

1. **2026-05-20 to 2026-06-15.** ADR-0309 + companion ADRs accepted.
2. **2026-06-15 to 2026-08-15.** Crate scaffolds + schemas + per-
   jurisdiction variant registry stubbed; existing models audited
   for proxy-feature + per-class TPR/FPR baseline.
3. **2026-08-15 to 2026-09-15.** First quarterly fairness audit
   emitted; NY AEDT public notice posted; EU AI Act Art. 5 refusal
   wired.
4. **2026-09-15.** CI lanes promote to BLOCKER.
5. **2026-09-15 onwards.** Continuous: quarterly fairness audit,
   per-PR proxy-feature check, per-jurisdiction variant maintenance.

### §F.2. Per-µservice migration playbook

1. **Audit current model features for proxies.** Run
   oya-shared-proxy-feature-detector against every production model.
2. **Quarterly fairness audit retrospectively.** Run audit against
   last 4 quarters' models.
3. **Author per-jurisdiction variants.** Per model serving multi-
   jurisdiction tenants.
4. **Wire adverse-action explanation.** Per ECOA Reg B / GDPR Art.
   22 / EU AI Act Art. 13.
5. **Publish NY AEDT public notice.** If employment-decision tool.
6. **Update compliance.md.** Per row 51.

### §F.3. Per-cell rollout pattern

- Tier-0 edge cells: no fairness substrate (no models)
- Tier-1 bootstrap cell: no models
- Tier-2 control plane cells: full fairness substrate
- Tier-3 data plane cells: per-jurisdiction variant lookup + adverse-
  action explanation generation

### §F.4. What is NOT migrated

- Deterministic rules (per ADR-0307 §D-4) — not ML; fairness
  invariants apply per-rule individually but not as a substrate
- Third-party APIs (NCMEC PhotoDNA, GIFCT) — third-party governance
- Cedar policies — separate substrate per ADR-0243

### §F.5. Rollback path

Per ADR-0294 + ADR-0308 §B.7. Emergency: per-jurisdiction variant
removal forces refusal-by-default; `FAIRNESS_BYPASS=1` env flag
NOT permitted — fairness invariants are BLOCKER-by-design; no
bypass.

## §G. References

### §G.1. Hyperscaler precedents

- **Microsoft Fairlearn** — github.com/fairlearn/fairlearn
- **IBM AI Fairness 360 (AIF360)** — github.com/Trusted-AI/AIF360
- **Google Responsible AI Practices** — ai.google/responsibility
- **Google What-If Tool** — github.com/PAIR-code/what-if-tool
- **Meta Civil Rights Audit** — about.fb.com/civilrightsaudit
- **Meta 2024 Civil Rights Annual Report**
- **Apple Fairness, Bias, and Transparency in AI** — apple.com/legal/privacy/aifairness
- **Anthropic Constitutional AI + bias evaluations** — anthropic.com/news
- **Stripe Radar ECOA compliance** — Stripe Sessions 2024 keynote
- **OpenAI bias evaluations** — openai.com/research
- **Vertex AI Workbench Responsible AI tab** — cloud.google.com/vertex-ai
- **AWS SageMaker Clarify** — aws.amazon.com/sagemaker/clarify (fairness)
- **Azure ML Responsible AI dashboard** — learn.microsoft.com/en-us/azure/machine-learning/concept-responsible-ai-dashboard
- **NIST SP 1270** — "Towards a Standard for Identifying and Managing Bias in Artificial Intelligence"

### §G.2. Standards + RFCs

- **NIST AI Risk Management Framework 1.0** — nist.gov/itl/ai-risk-management-framework
- **NIST SP 1270** — Bias in AI standard
- **ISO/IEC 42001:2023** — AI management systems
- **ISO/IEC 24028:2020** — AI trustworthiness overview

### §G.3. Legal + compliance

- **EU AI Act (Regulation 2024/1689)** — Articles 5 (prohibited practices), 6, 8, 9 (risk management), 10 (data + data governance), 11 (technical documentation), 12 (record-keeping), 13 (transparency), 14 (human oversight), 15 (accuracy + robustness), 17 (quality management), 18 (documentation retention), 27 (FRIA), 73 (serious incident), 86 (right to meaningful explanation); Annex III §1, 3, 5(b), 5(c), 6, 7, 8
- **EU Charter of Fundamental Rights Art. 21** — 14 protected classes
- **GDPR (Regulation 2016/679)** — Articles 9 (special category), 13/14 (transparency), 22 (automated decisions), 12 (timing)
- **EU Race Equality Directive 2000/43/EC** + **EU Employment Equality Directive 2000/78/EC**
- **NY AEDT Local Law 144 (2023)** — rules.cityofnewyork.us/wp-content/uploads/2023/04/DCWP-NOA-for-Automated-Employment-Decision-Tools-2.pdf
- **ECOA + Regulation B (12 CFR §1002)** — adverse-action notice with specific reasons
- **Fair Housing Act (42 USC §3601-3619)** + **HUD's disparate-impact rule (24 CFR §100.500)**
- **HUD 2013 Final Rule + 2023 reaffirmation**
- **Federal Uniform Guidelines on Employee Selection Procedures (29 CFR §1607.4)** — 4/5ths rule
- **Title VII of the Civil Rights Act of 1964** — race + sex + national origin + religion
- **ADEA (Age Discrimination in Employment Act)**
- **ADA (Americans with Disabilities Act)**
- **GINA (Genetic Information Nondiscrimination Act)**
- **USERRA (Uniformed Services Employment and Reemployment Rights Act)**
- **Bostock v. Clayton County (2020)** — sexual orientation + gender identity included in Title VII sex discrimination
- **Griggs v. Duke Power Co. (1971)** — disparate-impact doctrine
- **CFPB Circular 2022-03** — specific reasons for AI-driven credit decisions
- **California Civil Rights Department ADS regulation (proposed 2024)**
- **California AB 2013 + AB 3030 + AB 2655 (2024)**
- **Colorado AI Act (SB 24-205, 2024)** — effective 2026-02-01
- **Illinois BIPA (740 ILCS 14)**
- **Texas CUBI**
- **Washington My Health My Data Act (RCW 19.373)**
- **Utah AI Disclosure Bill (SB 149, 2024)**
- **KR Constitution Art. 11** — equality
- **KR Financial Consumer Protection Act Art. 30** — 시행 2021-03-25
- **KR National Human Rights Commission Act**
- **KR Equality Act (proposed)**
- **JP Act on the Promotion of Equal Opportunity and Treatment**
- **JP Disability Discrimination Act**
- **JP Equal Employment Opportunity Act**
- **JP APPI** — Personal Information Protection Act
- **UK Equality Act 2010** — 9 protected characteristics
- **UK Age Appropriate Design Code (AADC, 2020)**
- **AU Sex Discrimination Act**, **AU Age Discrimination Act**, **AU Disability Discrimination Act**, **AU Racial Discrimination Act**, **AU Privacy Act amendments (2024)**

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
- **ADR-0292** — minor user doctrine COPPA + KOSA + EU age verification
- **ADR-0293** — Foundry meta-trust-root
- **ADR-0294** — Cedar fragment soak + anomaly-rollback
- **ADR-0297** — abuse-defence baseline
- **ADR-0298** — emergency-services critical-path exemption
- **ADR-0307** — detection substrate (this bundle)
- **ADR-0308** — ML model lifecycle (this bundle)
- **ADR-0310** — investigation case-management (this bundle)

### §G.5. Standards docs

- `docs/standards/documentation-rigor.md` §3.2.6 — DRMP baseline
- `docs/standards/fintech-compliance.md` — KR-FSS / PCI-DSS / ECOA
- `docs/standards/doc-style.md` — Diátaxis + RFC-2119

### §G.6. Auto-memory feedback (related)

- `feedback_quality_performance_scalability_bar` — hyperscaler rigor
- `feedback_clean_architecture_requirements` — inward-only + single-concern
- `feedback_no_silent_regression` — public-contract protection
- `feedback_autonomous_implementation_artifacts` — intern-buildable fairness
- `feedback_canonical_base_localization` — global base + per-jurisdiction overlay
- `feedback_build_ahead_of_certification` — day-one EU AI Act + ECOA + NY AEDT
- `feedback_oyatie_is_a_tenant_doctrine` — fairness applies to oyatie's own AI
- `feedback_cedar_as_universal_gate` — Cedar gates protected-class access
- `feedback_compliance_pack_primitive` — per-pack overlay
- `feedback_substrate_vs_product_layering` — fairness is substrate
- `feedback_naming_justification` — every primitive justified

## §H. Change log

- **2026-05-20** — Initial draft authored as part of keystone-bundle 2026-05-20 Wave-3-D detection-cluster batch (ADR-0307..0310). Bundled with ADR-0307 (detection substrate), ADR-0308 (ML lifecycle), ADR-0310 (investigation case-management) as the **drmp-detection-cluster** keystone batch. Covers EU AI Act + ECOA + Fair Housing + NY AEDT + 4/5ths rule + KR FCPA Art. 30 + JP APPI + UK AADC fairness invariants. Enforcement advisory-until-2026-09-15-blocker-thereafter.
