---
id: ADR-0307
status: Superseded
date: 2026-05-20
owners:
  - council-architecture
  - council-security
  - council-privacy
  - council-product
  - council-legal
  - council-data
  - ops-sre-reliability
  - ops-security
  - ops-trust-and-safety
  - ops-compliance
  - ops-data-platform
  - axis-detection
  - axis-feature-store
  - axis-rules-engine
  - axis-graph-store
  - axis-audit-chain
  - axis-observability
  - axis-investigation
supersedes: []
amends: []
superseded_by: [ADR-0701]
related:
  - ADR-0028-audit-chain-merkle-sealed.md
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
  - ADR-0308-ml-model-lifecycle-ai-act-compliance.md
  - ADR-0309-detection-fairness-audit-civil-rights.md
  - ADR-0310-investigation-case-management.md
related_specs:
  - /specs/platform-architecture.json
  - /specs/microservices/detection.json
  - /specs/microservices/payments.json
  - /specs/microservices/identity.json
  - /specs/microservices/audit-chain.json
  - /specs/microservices/observability.json
  - /specs/detection-rule-schema.json
  - /specs/detection-feature-schema.json
  - /specs/detection-signal-schema.json
  - /specs/detection-family-registry.json
related_memory:
  - feedback_quality_performance_scalability_bar
  - feedback_clean_architecture_requirements
  - feedback_no_silent_regression
  - feedback_autonomous_implementation_artifacts
  - feedback_canonical_base_localization
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_cedar_as_universal_gate
  - feedback_amazon_shape_cellular_architecture
  - feedback_compliance_pack_primitive
  - feedback_naming_justification
  - feedback_substrate_vs_product_layering
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-foundational-doctrine
keystone_position: drmp-detection-substrate
purpose: >
  Establish the Detection Substrate (streaming + batch) as a substrate-level
  µservice (microservices/detection/) covering all eight detection families
  defined in documentation-rigor.md §3.2.6.A: payment fraud, account-takeover
  (ATO), synthetic identity, AML+sanctions, content abuse (CSAM + terrorism +
  NCII + copyright + misinformation), fake reviews + engagement manipulation,
  insider risk, and policy violation. Codifies the eight substrate primitives
  (streaming pipeline, batch pipeline, feature store, rules engine, composite
  scorer, graph store + community detection, investigation integration, sandbox
  + replay) with hyperscaler precedent named on each primitive (Stripe Radar,
  Adyen RevenueProtect, Toss riskOps, AWS GuardDuty, Google Chronicle, NCMEC
  PhotoDNA, GIFCT hash-matching, Apache Flink, Apache Spark, Feast, Tecton,
  Vertex AI Feature Store, ClickHouse, Trino, Polars, Apache AGE, Neo4j). The
  detection substrate is the D in DRMP (Detection → Risk → Mitigation →
  Prevention); without it, every µservice authors its own detection rules and
  the platform drifts into a fragmented, non-auditable, regulator-unfriendly
  posture incompatible with EU AI Act Art. 13 transparency, GDPR Art. 22
  automated-decision rights, ECOA Reg B adverse-action explainability, NY AEDT
  Local Law 144 (2023) bias-audit cadence, ISO/IEC 42001:2023 AI management,
  NIST AI RMF 1.0, and 18 USC §2258A NCMEC reporting obligations.
enforcement_status: advisory-until-2026-09-15-blocker-thereafter
enforced_by:
  - cloud-ci/Rust gate packet detection-substrate-streaming-pipeline-coherence
  - cloud-ci/Rust gate packet detection-substrate-batch-pipeline-coherence
  - cloud-ci/Rust gate packet detection-substrate-feature-store-coherence
  - cloud-ci/Rust gate packet detection-substrate-rules-engine-coherence
  - cloud-ci/Rust gate packet detection-substrate-composite-scorer-coherence
  - cloud-ci/Rust gate packet detection-substrate-graph-store-coherence
  - cloud-ci/Rust gate packet detection-substrate-sandbox-replay-coherence
  - cloud-ci/Rust gate packet detection-substrate-emission-per-microservice
  - cloud-ci/Rust gate packet detection-substrate-rule-lifecycle-soak
  - cloud-ci/Rust gate packet detection-family-coverage-eight-of-eight
naming_justifications:
  - name: microservices/detection
    layer: layer_5_shared_substrate
    bnf_segments: microservices.detection
    justification: >
      Per ADR-0131 per-microservice flat layout + ADR-0132 no-grouping rule, the
      detection capability MUST ship as a single-concern flat µservice under
      microservices/detection/ — not bundled under a "fraud-suite" or
      "risk-suite" name. The µservice serves all eight detection families
      (§3.2.6.A) from a single substrate; family-specific BCs live inside
      src/ as bounded contexts, not as separate µservices. Per ADR-0245
      substrate vs product, detection is a substrate µservice consumed by
      every internet-facing product (payments, identity, marketplace, social,
      community, messenger, mail). Naming uses the canonical microservices/
      prefix per ADR-0131.
  - name: oya-shared-detection-streaming
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.detection-streaming
    justification: >
      Per ADR-0105 13-layer canonical enum row 5 (shared-substrate), the crate
      that exposes the Apache Flink job builder + per-family rule executor +
      per-entity scorer dispatch trait belongs at the shared layer. Single-
      concern naming per ADR-0131; never bundle the batch path into this
      crate (those are oya-shared-detection-batch). Two crates allow
      independent SemVer cadence per ADR-0258.
  - name: oya-shared-detection-batch
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.detection-batch
    justification: >
      Companion to oya-shared-detection-streaming; the Spark/Polars/Trino
      backfill + scheduled-batch + retrospective-detection path. Separate
      crate so the batch path can SemVer-evolve independently of the
      streaming path (e.g., Spark major version upgrades land here without
      forcing a Flink version bump).
  - name: oya-shared-feature-store
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.feature-store
    justification: >
      Per-entity feature computation + per-tenant feature isolation + Feast/
      Tecton/Vertex-AI-Feature-Store-compatible API. Single-concern; not
      bundled under a "ml-suite" name. Per ADR-0244 tenant scoping, every
      feature row carries tenant_id; per ADR-0099 data-class registry,
      every feature declares its data-class (PII / pseudonymous / aggregate)
      so the feature store can enforce per-pack restrictions (HIPAA-pack
      tenants → PHI features never enter the store).
  - name: oya-shared-detection-rules-engine
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.detection-rules-engine
    justification: >
      Sigma-rule-class declarative DSL evaluator + per-rule lifecycle
      (Proposed → Soaking → Active → Sunset) mirroring Cedar fragment
      lifecycle per ADR-0294. Single-concern; the engine is generic over
      rule body shape so all eight families share one runtime.
  - name: oya-shared-detection-composite-scorer
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.detection-composite-scorer
    justification: >
      Combines per-family signals into a unified per-entity risk score;
      LIME/SHAP-style feature-importance available for explainability per
      EU AI Act Art. 13 + GDPR Art. 22 + ECOA Reg B. Single-concern: scoring
      is one primitive — the scorer does not own the rules or the model
      lifecycle (those are separate ADRs 0307 §D + ADR-0308).
  - name: oya-shared-graph-store
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.graph-store
    justification: >
      Apache AGE (Postgres+graph extension) or Neo4j-compatible link-
      analysis runtime; powers fraud-ring detection, mule-account graphs,
      synthetic-identity clusters, click-farm topology. Single-concern;
      not bundled with the rules engine — graph algorithms (Louvain, label
      propagation, PageRank) have a different SemVer cadence than rules.
  - name: oya-shared-detection-sandbox-replay
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.detection-sandbox-replay
    justification: >
      Replay harness: any new rule or new model can be back-tested against
      historical audit-stream before promotion to Active. Single-concern;
      separate from the streaming runtime because the replay harness reads
      ClickHouse cold tier, not Kafka hot tier — different I/O shape, so
      different crate.
  - name: oya-governance-detection-substrate-emission
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.detection-substrate-emission
    justification: >
      CI fitness lane per ADR-0212 buildability doctrine; verifies every
      µservice emits the audit-event-classes declared in its
      compliance.md §detection-substrate-binding. Lane naming follows the
      canonical oya-governance-<concern> shape consistent with sibling
      lanes (per documentation-rigor.md §3.2.6.I + ADR-0212 §G).
  - name: oya-governance-detection-rule-lifecycle-soak
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.detection-rule-lifecycle-soak
    justification: >
      CI fitness lane per ADR-0212; verifies every new detection rule
      passes ≥7-day Soaking before promotion to Active, mirroring the
      ≥60s Cedar fragment soak per ADR-0294 (detection rules use a longer
      soak window because they fire on streaming data continuously and
      their false-positive rate cannot be measured in seconds).
  - name: oya-governance-detection-family-coverage
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.detection-family-coverage
    justification: >
      CI fitness lane per ADR-0212; verifies the substrate offers active
      rules + models for all eight families enumerated in §3.2.6.A. A
      detection substrate that lacks any family is a substrate gap and
      MUST fail this lane.
  - name: DetectionSignalEmitted
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Detection.SignalEmitted
    justification: >
      Audit-event-class emitted whenever the streaming or batch pipeline
      emits a signal to the investigation queue or to the mitigation
      substrate (per ADR-0263 registry). Single class for all eight
      families; family is a tag on the event payload, not a separate
      class — keeps the registry's cardinality bounded.
  - name: DetectionRulePromoted
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Detection.RulePromoted
    justification: >
      Emitted when a rule transitions Soaking → Active. Mirrors
      CedarFragmentPromoted per ADR-0294. Registered in the central
      registry to satisfy the §3.2.2 consistency invariant.
  - name: DetectionRuleSunset
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Detection.RuleSunset
    justification: >
      Emitted when a rule transitions Active → Sunset (retired). Pair to
      DetectionRulePromoted; registered per ADR-0263.
  - name: DetectionModelDeployed
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Detection.ModelDeployed
    justification: >
      Emitted when an ML model is deployed (champion or challenger).
      Lifecycle event coordinated with ADR-0308 ML lifecycle.
  - name: DetectionModelRolledBack
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Detection.ModelRolledBack
    justification: >
      Emitted on rollback per ADR-0294 anomaly-rollback semantics applied
      to ML models. Registered per ADR-0263.
  - name: DetectionDriftAlertTriggered
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Detection.DriftAlertTriggered
    justification: >
      Emitted when feature / label / concept drift crosses threshold per
      ADR-0308 drift-detection rules. Registered per ADR-0263.
  - name: DetectionSignalConsumed
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Detection.SignalConsumed
    justification: >
      Emitted by downstream µservices when they act on a detection signal
      (cool-down, step-up auth, freeze, etc.). Pairs with
      DetectionSignalEmitted to form an end-to-end traceability chain.
  - name: detection-rule-schema.json
    layer: N/A (JSON Schema spec)
    bnf_segments: spec.detection-rule-schema
    justification: >
      JSON Schema declaring the Sigma-rule-class detection rule shape;
      every rule under microservices/detection/rules/ MUST validate
      against this schema. Per the §3.2.2 consistency invariant.
  - name: detection-family-registry.json
    layer: N/A (JSON Schema spec)
    bnf_segments: spec.detection-family-registry
    justification: >
      JSON Schema declaring the eight-family enum (payment_fraud, ato,
      synthetic_identity, aml_sanctions, content_abuse, engagement_manipulation,
      insider_risk, policy_violation). Closed enum cap; new families require
      ADR amendment to ADR-0307. Per documentation-rigor.md §3.2.6.A.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0307: Detection Substrate — Streaming + Batch (DRMP "D" Layer)

## Status

Proposed — 2026-05-20.

Bundled with the keystone-bundle 2026-05-20 foundational doctrine
synthesis as the **drmp-detection-substrate** keystone, closing the
gap identified in `docs/standards/documentation-rigor.md` §3.2.6.A
(eight detection families enumerated; substrate primitives named;
binding ADR called out as ADR-0307). This ADR is the binding ADR
row 49 of the §3.2.1 ADR-adherence matrix cites.

Enforcement is `advisory-until-2026-09-15-blocker-thereafter`. The
doctrine accepts in text immediately; the CI lanes that enforce
per-µservice detection-substrate emission promote to BLOCKER on
2026-09-15 to give every contributing µservice (payments, identity,
marketplace, social, community, messenger, mail, ops-dashboard-control-center,
audit-chain, observability, intelligence) time to wire their
emission contract per §E.

## Date

2026-05-20.

## Context

### §A. Why detection is a substrate primitive, not a per-µservice afterthought

Mature hyperscaler platforms treat detection (fraud, abuse,
policy-violation, AML, insider risk) as a *first-class substrate
primitive* — wired centrally so signals from every surface compose
into a unified per-entity (user, tenant, transaction, content) risk
score, fed by a unified feature store, evaluated by a unified rules
engine, and explained by a unified composite scorer. The pattern is
unambiguous across the named industry references:

- **Stripe Radar.** Stripe's payment-fraud detection runs centrally
  across every Stripe-processed charge — not per-merchant. Per the
  Stripe Radar product docs (stripe.com/docs/radar) and the 2024
  Stripe Annual Letter, Radar trains on signal from ~135 countries +
  ~1.4M businesses + the entire Stripe-processed corpus. A single
  merchant cannot replicate Radar with their own ML — Radar's
  precision comes from the cross-merchant signal Stripe sees that
  no individual merchant sees. Substrate scale is the precision
  driver.
- **Adyen RevenueProtect.** Adyen's equivalent runs at network
  scale across the Adyen-processed corpus (~$1T annualized 2024 per
  Adyen 2024 annual report). RevenueProtect emits risk score + 3DS
  challenge decision + chargeback-likelihood as a substrate service,
  not a per-merchant integration. Per Adyen's published architecture
  (adyen.com/blog/risk), RevenueProtect runs as a Flink-class
  streaming runtime fed by Kafka + ClickHouse OLAP backfill.
- **Toss riskOps.** KR-domestic fintech Toss runs detection
  substrate across all Toss-mediated transactions (~5B/yr per Toss
  2024 disclosure); per the Toss 2024 Tech Conference keynote,
  riskOps is a single substrate that all 100+ Toss-product surfaces
  (Toss Pay, Toss Bank, Toss Securities, Toss Insurance) consume.
  The single-substrate shape matches Stripe + Adyen.
- **AWS GuardDuty + Amazon Detective + AWS Macie.** AWS's detection
  substrate covers cloud-workload threat (GuardDuty), investigation
  correlation (Detective), and data-classification + sensitive-data
  detection (Macie). Per AWS documentation (aws.amazon.com/guardduty,
  /detective, /macie), all three run as managed substrate services
  — customers consume detection signals, they don't author detection
  rules per-account.
- **Google Chronicle (now Google Security Operations).** Google's
  unified detection + investigation runtime; ingests ~PB/day of
  telemetry; runs YARA-L rules (Sigma-rule-class declarative DSL)
  + ML models centrally. Per the Chronicle product overview
  (cloud.google.com/chronicle-security-operations), customers
  benefit from cross-customer signal aggregated at Google scale.
- **NCMEC PhotoDNA + GIFCT hash-matching.** Content-abuse detection
  for CSAM (PhotoDNA, Microsoft Research 2009, donated to NCMEC) +
  terrorism (GIFCT, Global Internet Forum to Counter Terrorism,
  founded 2017 by Microsoft + Facebook + Google + Twitter) runs as
  a *cross-platform substrate*: every platform contributes hashes,
  every platform queries the unified hash database. The substrate
  shape is so essential to the threat model that the major platforms
  built a cross-company substrate.

The corollary: **every detection-emitting and detection-consuming
surface oyatie ships MUST go through the substrate, not author
per-µservice detection rules.** A payments µservice that authors its
own fraud heuristics drifts from the identity µservice's ATO signals,
drifts from the marketplace µservice's engagement-manipulation
signals, drifts from the audit-chain's policy-violation signals.
The cross-family signal that makes Radar precise is *lost* if every
µservice runs its own model. The substrate shape captures it.

This duplication-of-detection-logic is a `feedback_no_silent_regression`
violation (every µservice's detection drifts independently); it is a
`feedback_quality_performance_scalability_bar` violation (the substrate
sees signal across every µservice's traffic that a single µservice
cannot); it is a `feedback_clean_architecture_requirements` violation
(detection logic in every µservice = no inward-only flow + no
single-concern). The ADR-0307 detection substrate baseline closes
the gap.

### §A.1. The 2026 threat landscape — what the substrate detects

The eight detection families enumerated in §3.2.6.A are the canonical
threat surface oyatie's substrate covers. Each family is sized by
its 2026 baseline:

1. **Payment fraud.** Card-not-present fraud globally ~$48B/yr (Nilson
   Report 2024); friendly-fraud chargeback rate ~0.3-1.2% across
   e-commerce (LexisNexis True Cost of Fraud 2024); BIN-attack burst
   rate ~10⁴-10⁶ probes/sec per attacker via residential proxies.
   Detection: per-PSP risk score (Radar / RevenueProtect / Toss
   riskOps) + in-house composite scorer + graph-based fraud-ring
   detection (mule networks, refund-fraud rings, gift-card-laundering
   rings).
2. **Account-takeover (ATO).** Credential-stuffing campaigns hit
   ~10⁵-10⁶ login attempts per attacker per hour via OpenBullet 2 +
   Sentry MBA + Storm-1152-class infrastructure (Microsoft Threat
   Intelligence 2024 disclosure). SIM-swap fraud sees ~$72M/yr loss
   in the US alone (FBI IC3 2024 report). Detection: behavioural
   fingerprint drift (TLS JA4 + browser/device + cursor cadence) +
   geo-impossibility (~3% of legitimate users have impossible
   geo-deltas due to VPNs; the substrate baseline filters those) +
   device-change-after-auth + password-reset-velocity.
3. **Synthetic identity.** Manufactured KYC + deepfake selfies + AI-
   generated docs. FinCEN estimated synthetic-identity fraud at ~$20B/
   yr US-domestic (FinCEN 2024 Annual Trends Report). Detection: live-
   ness check (NIST iBeta Level 2 compliant) + face-match to passkey-
   enrolled device + KYB graph correlation (cross-business directorship
   overlap, shared address signal, shared phone signal) + Bureau-data
   cross-check (Experian / Equifax / TransUnion in US; equivalent
   Bureau wiring per jurisdiction).
4. **AML + sanctions.** Money-laundering structuring + sanctions
   evasion (OFAC / EU / UN / KR-MOFA / JP-METI). FinCEN SAR (Suspicious
   Activity Report) threshold $5,000 for some structuring patterns,
   $10,000 for CTR (Currency Transaction Report). Sanctions list churn
   ~weekly OFAC SDN updates; EU sanctions ~weekly to fortnightly.
   Detection: transaction-graph analysis (Louvain community detection
   for layering patterns) + sanctions-list match (fuzzy name match
   + DOB + address) + PEP enrichment (Refinitiv World-Check + LexisNexis
   Bridger + in-house enrichment) + suspicious-activity threshold.
5. **Content abuse.** Six sub-classes: CSAM (NCMEC PhotoDNA + Apple's
   NeuralHash + Microsoft PhotoDNA Cloud Service + Google CSAI Match);
   terrorism (GIFCT hash database, ~5M hashes 2024); NCII (StopNCII.org
   hash database, ~500k hashes 2024); copyright (Content ID / Audible
   Magic / Pex / proprietary perceptual-hash); misinformation
   (per-claim fact-check from Snopes / PolitiFact / Maldita / Full
   Fact + per-source authority signal); hate speech / harassment
   (per-category Cedar policy + per-pack DSA Art 16+17 + EU-CSAM-Reg).
   Detection: NCMEC PhotoDNA pre-upload check + GIFCT hash match +
   ML classifier (CLIP / per-modality vision-language model fine-
   tuned per family) + human-review queue with case-management
   integration (cross-ref ADR-0310) + per-pack DSA Article 16+17
   refusal predicates.
6. **Fake reviews + engagement manipulation.** Review-bombing,
   paid-review fraud, fake follows, click-farm activity, view-count
   manipulation. Per the FTC's 2024 Rule on Use of Consumer Reviews
   and Testimonials (16 CFR Part 465, finalized August 2024,
   effective October 2024), fake reviews are explicitly prohibited
   with civil penalties up to $51,744 per violation. Detection:
   graph community-detection (Louvain, Label Propagation) on
   review-author-product tripartite graph + behavioural pattern
   detection (typing cadence, edit history, IP fingerprint
   clustering) + temporal clustering (review burst within window).
7. **Insider risk.** Tenant-admin exfiltration, employee data access
   pattern anomaly, JIT-access abuse, post-departure access. Per
   Verizon's 2024 DBIR, insider breaches account for ~28% of
   incidents. Detection: UEBA (User and Entity Behaviour Analytics)
   per §3.2.4 Domain 8 + per-employee baseline + sensitive-resource
   access pattern + post-departure access (no access should occur
   ≥48h after termination per HR-Cedar-binding).
8. **Policy violation.** Cedar permit forge, audit-row tamper attempt,
   cross-tenant access attempt, sidecar credential exfil, sanctions-
   bypass attempt. Detection: Cedar evaluation anomaly (per-fragment
   permit/forbid rate drift) + audit-chain tamper detection (Merkle-
   seal verification per ADR-0028) + cross-tenant flow detection
   (Cilium NetworkPolicy violation telemetry) + sidecar exit anomaly
   (per ADR-0296 sidecar attestation).

The substrate baseline MUST be sized to cover all eight families
from the substrate, not 1-2 families with the rest "later".

### §A.2. Why the keystone bundle 2026-05-20 requires this as a substrate µservice

The keystone bundle's foundational ADRs intersect detection as follows:

- **ADR-0242 (oyatie-is-a-tenant).** The platform's own surfaces are
  subject to the same detection substrate — oyatie internal employees
  hitting admin surfaces trigger insider-risk signals just like any
  other tenant; no internal-vs-external carve-outs.
- **ADR-0243 (Cedar universal gate).** Detection signals become inputs
  to Cedar evaluation — every action's Cedar permit checks the
  current per-entity risk score. The substrate provides the score;
  Cedar gates the action.
- **ADR-0244 (tenant scoping).** Every detection signal carries
  tenant_id; every feature in the feature store carries tenant_id;
  per-tenant pack overlays (HIPAA → PHI features never enter the
  store) enforced at write time.
- **ADR-0245 (substrate vs product).** Detection is a substrate
  µservice consumed by every internet-facing product. Not a product
  itself — it has no end-user surface; surface lives in
  `microservices/ops-dashboard-control-center/` (per ADR-0310
  investigation case-management).
- **ADR-0248 (cellular architecture).** Detection runs in Tier-2
  (control plane) cells primarily; Tier-3 data cells host the
  feature store + ClickHouse cold tier; Tier-0 edge cells emit
  signals to detection but do not host detection runtime themselves.
- **ADR-0251 (compliance packs).** Per-pack overlays modify which
  features are computable (HIPAA forbids PHI features), which rules
  are active (EU-CSAM-Reg adds mandatory CSAM rule, GDPR + Art 22
  forbids fully-automated adverse-action without human-in-loop), and
  which mitigation flows are triggered (per-pack regulator
  notification cadence per ADR-0251 §nis2_three_stage_cadence).
- **ADR-0263 (observability emission contract).** Every detection
  signal is an audit-event-class. The registry registers seven new
  classes (DetectionSignalEmitted, DetectionRulePromoted,
  DetectionRuleSunset, DetectionModelDeployed, DetectionModelRolledBack,
  DetectionDriftAlertTriggered, DetectionSignalConsumed).
- **ADR-0294 (Cedar fragment soak).** Detection rule lifecycle
  mirrors Cedar fragment lifecycle (Proposed → Soaking → Active →
  Sunset); soak window for detection rules is ≥7 days (not ≥60s)
  because false-positive rate cannot be measured in seconds on
  streaming data.
- **ADR-0297 (abuse-defence baseline).** Abuse-defence layer L0-L8
  feeds detection at L6 (observability + detection); detection
  signals feed back into Cedar at L3 + per-action Cedar gates.
- **ADR-0298 (emergency-services exemption).** Detection MUST NOT
  block emergency-services traffic; audit-and-investigate, never
  block.

### §A.3. What this ADR explicitly does NOT do

ADR-0307 is the **detection substrate** — the D in DRMP. It does
NOT:

1. **Author ML model lifecycle.** Model training, validation, A/B
   testing, drift detection, fairness re-audit, model versioning,
   rollback, appeal mechanism — those live in ADR-0308.
2. **Author fairness audit cadence + per-jurisdiction overlay.**
   Those live in ADR-0309.
3. **Author investigation case-management workflow.** Triage →
   investigation → escalation → feedback lives in ADR-0310.
4. **Author per-family rule bodies.** Specific rule grammar (e.g.,
   "if velocity_24h > 10 AND new_device THEN signal=ATO") lives in
   per-family rule files under microservices/detection/rules/
   authored by Wave-3-D agents per the rule-authoring runbook.
5. **Author the consumer surface.** End-user appeal surface +
   tenant-admin investigation dashboard + regulator-facing report
   surface lives in microservices/ops-dashboard-control-center/
   per ADR-0310.
6. **Replace abuse-defence at the edge.** Edge-layer bot/spoof/
   scrape defence per ADR-0297 still runs at Tier-0; detection
   substrate consumes its signals + adds graph + ML + per-family
   correlation on top.

## Decision

### §B. Detection substrate as a single-concern flat µservice

Establish `microservices/detection/` as a substrate-tier flat
µservice (per ADR-0131 per-microservice flat layout + ADR-0132
no-grouping rule) exposing eight substrate primitives:

1. **Streaming pipeline (Apache Flink-class).** Consumes audit
   events from Kafka per ADR-0263; per-family rules + ML models
   score in-flight; signals emitted to the investigation queue.
   Implementation: Apache Flink 1.20 LTS (released 2024-08, LTS
   support through 2027) hosted on K8s per ADR-0254 deployment
   model + Tier-2 control-plane cells per ADR-0248. Alternative
   implementations supported: Apache Beam (Beam Sql), Kafka Streams,
   Materialize, RisingWave — but the canonical reference is Flink.
2. **Batch pipeline (Apache Spark-class).** Scheduled jobs over the
   audit-event lake (ClickHouse cold tier per
   cloud-iac/clickhouse-cluster-iac.yaml + per ADR-0099 data-class
   registry) for retrospective detection. Implementation: Apache
   Spark 3.5 + Polars (Rust-native columnar) for sub-TB workloads
   + ClickHouse for OLAP + Trino for federated query across Spark
   + ClickHouse + Postgres + Iceberg.
3. **Feature store.** Per-entity feature computation + per-tenant
   feature isolation + Feast/Tecton/Vertex-AI-Feature-Store
   compatible API. Implementation: Feast 0.40+ as the open-source
   canonical reference; Tecton + Vertex AI Feature Store supported
   as alternative deployments for Tier-3+ cells (cloud-managed)
   per ADR-0240 sovereign-cloud overlay. Online + offline split:
   online (Redis-class low-latency) for streaming scorers; offline
   (Parquet + Iceberg) for batch training.
4. **Rules engine (Sigma-rule-class).** Declarative rule DSL +
   per-rule lifecycle (Proposed → Soaking → Active → Sunset)
   mirroring Cedar fragment lifecycle per ADR-0294. Implementation:
   Rust-native rule evaluator over Apache Arrow record batches;
   rules compile to a tree-walking interpreter + JIT to LLVM for
   hot rules. Rule grammar inspired by SigmaHQ
   (github.com/SigmaHQ/sigma) — declarative YAML with selection +
   condition + timeframe semantics.
5. **Composite scorer.** Combines per-family signals into unified
   per-entity risk score; LIME/SHAP explainable per EU AI Act
   Art. 13 + GDPR Art. 22 + ECOA Reg B. Implementation: gradient-
   boosted-trees baseline (LightGBM / XGBoost) for explainability
   + optional neural-network secondary scorer for high-dim feature
   spaces (per family) — but adverse-action decisions MUST be
   explainable, so neural scorer outputs feed into a final LightGBM
   ensemble whose feature importance is reported on appeal.
6. **Graph store + community detection.** Apache AGE (Postgres
   extension; canonical for substrate-tier deployments) or Neo4j
   (alternative for Tier-3+ cloud-managed) with Louvain + Label
   Propagation + PageRank + connected components algorithms exposed
   via Cypher-compatible interface.
7. **Investigation case-management integration.** Detection signals
   route to investigation queue per ADR-0310; bidirectional —
   detection emits signal, investigation feeds analyst labels back
   to feature store for model retraining.
8. **Sandbox + replay.** Any new rule or new model can be back-tested
   against historical audit-stream from ClickHouse cold tier before
   promotion to Active. Sandbox runs the same Flink topology
   parameterized by replay-time-window; promotion gate requires
   ≥7-day soak in shadow mode (rule fires but no mitigation
   triggered) + ≤3% false-positive rate against analyst-labeled
   ground truth.

### §B.1. Eight detection families — substrate coverage matrix

Per §3.2.6.A, the substrate MUST cover all eight families with
named hyperscaler precedent on each. The coverage matrix:

| # | Family | Streaming detector | Batch detector | Feature-store features | Graph algorithm | Hyperscaler precedent |
|---:|---|---|---|---|---|---|
| 1 | Payment fraud | Flink job `payment_fraud_streaming` consuming `PaymentChargeAttempted` + `PaymentChargeSucceeded` + `PaymentChargeDeclined`; LightGBM scorer per-transaction | Spark batch `payment_fraud_backfill` runs daily over 90-day window; recomputes per-merchant baseline + per-BIN velocity | `feature.payment.velocity_24h`, `feature.payment.bin_attack_score`, `feature.payment.refund_rate_30d`, `feature.payment.geo_distance_from_baseline`, `feature.payment.device_change_velocity` | Louvain on merchant-BIN-card tripartite for refund-fraud rings | Stripe Radar; Adyen RevenueProtect; Toss riskOps |
| 2 | Account-takeover (ATO) | Flink job `ato_streaming` consuming `IdentitySignInAttempted` + `IdentitySignInSucceeded`; behavioural-fingerprint-drift scorer per-session | Spark batch `ato_backfill` daily over 90-day window; recomputes per-user baseline | `feature.identity.tls_fingerprint_drift`, `feature.identity.geo_impossibility_score`, `feature.identity.device_change_velocity`, `feature.identity.password_reset_velocity`, `feature.identity.hibp_match` | Connected-components on session-device-IP graph for credential-stuffing burst | Microsoft Entra Identity Protection; Okta ThreatInsight; Google reCAPTCHA Enterprise account-defender |
| 3 | Synthetic identity | Flink job `synth_identity_streaming` consuming `IdentityKYCAttempted` + `KYBAttempted`; liveness-check score + KYB-graph-correlation scorer | Spark batch `synth_identity_backfill` weekly; recomputes per-corp directorship overlap graph | `feature.kyc.liveness_score`, `feature.kyc.face_match_score`, `feature.kyb.directorship_overlap`, `feature.kyb.shared_address_signal`, `feature.kyb.shared_phone_signal` | Louvain on corp-director-address graph for mule-corp clusters | FinCEN synthetic-identity detection guidance; Persona; Onfido; iProov; SumSub |
| 4 | AML + sanctions | Flink job `aml_streaming` consuming `PaymentChargeSucceeded` + `PaymentPayoutAttempted`; threshold-velocity scorer + sanctions-list match | Spark batch `aml_backfill` daily; runs transaction-graph community detection for layering patterns | `feature.aml.transaction_velocity_24h`, `feature.aml.transaction_velocity_30d`, `feature.aml.sanctions_match_score`, `feature.aml.pep_score`, `feature.aml.high_risk_corridor_score` | Louvain + Label Propagation on transaction-graph for layering rings | FinCEN AML model risk management; OFAC sanctions screening; LexisNexis Bridger; Refinitiv World-Check; ComplyAdvantage |
| 5 | Content abuse | Flink job `content_abuse_streaming` consuming `MessengerMessageSent` + `MailDeliveryAttempted` + `CommunityPostCreated` + `MarketplaceListingCreated`; per-modality classifier + hash-match | Spark batch `content_abuse_backfill` daily; recomputes per-corpus hash database refresh + per-creator score | `feature.content.photodna_match`, `feature.content.gifct_match`, `feature.content.stopncii_match`, `feature.content.classifier_score`, `feature.content.per_creator_violation_count` | Connected-components on creator-content-victim graph for serial-offender detection | NCMEC PhotoDNA; GIFCT hash database; StopNCII.org; Apple NeuralHash; Google CSAI Match; Hive Moderation; ActiveFence; Spectrum Labs |
| 6 | Fake reviews / engagement manipulation | Flink job `engagement_manipulation_streaming` consuming `MarketplaceReviewSubmitted` + `SocialFollowCreated` + `CommunityVoteCast`; temporal-burst detector | Spark batch `engagement_manipulation_backfill` daily; runs Louvain on review-author-product graph | `feature.engagement.review_burst_score`, `feature.engagement.author_velocity`, `feature.engagement.review_text_template_match`, `feature.engagement.purchase_verification_present` | Louvain on review-author-product graph for paid-review rings | Amazon's review-fraud detection (per Amazon 2024 transparency report); TripAdvisor's review-fraud team; Yelp's recommendation software |
| 7 | Insider risk | Flink job `insider_risk_streaming` consuming all `ops.admin.*` + JIT-access-grant events; per-employee baseline drift scorer | Spark batch `insider_risk_backfill` daily; recomputes per-employee baseline + per-role peer cohort | `feature.insider.access_pattern_drift`, `feature.insider.off_hours_access_count`, `feature.insider.sensitive_resource_burst`, `feature.insider.post_departure_access_count`, `feature.insider.peer_cohort_z_score` | Connected-components on employee-resource access graph | Exabeam UEBA; Securonix; Microsoft Sentinel UEBA; Splunk UBA; ObserveIT (Proofpoint); Code42 Incydr |
| 8 | Policy violation | Flink job `policy_violation_streaming` consuming `CedarEvaluationCompleted` + audit-row Merkle-seal verification + Cilium NetworkPolicy violation events; per-fragment permit/forbid drift scorer | Spark batch `policy_violation_backfill` daily; recomputes Cedar fragment baseline; detects audit-chain anomalies | `feature.policy.cedar_forbid_rate_drift`, `feature.policy.audit_chain_seal_failures`, `feature.policy.cross_tenant_l3_attempts`, `feature.policy.sidecar_exit_anomalies` | Cross-tenant flow graph + Cedar fragment dependency graph | Open Policy Agent decision logging; Cedar v4 evaluation telemetry; AWS GuardDuty cloud-policy-violation; Google Chronicle policy-event correlation |

Every µservice MUST contribute signals to at least one family (per
its surface area); MUST consume at least one family's signals where
its actions intersect risk (e.g., payments consumes ATO signal from
identity and applies cool-down on high-value transaction). The
per-µservice obligation matrix lives in §3.2.6.H and is per-µservice
documented in `compliance.md §detection-substrate-binding`.

### §B.2. The accessibility floor — no detection-blocks-on-default-path

Per `feedback_no_silent_regression` + ADR-0298 emergency-services
exemption + §3.2.6 UX-floor + ADR-0297 §B.2 accessibility floor:

1. Detection signals NEVER auto-trigger mitigation on default user
   path. Signals route to risk score → Cedar gate → optionally
   triggers cool-down / step-up / freeze based on Cedar policy +
   tenant-admin-tuned thresholds. The substrate emits signals;
   Cedar decides actions.
2. Emergency-services exemption (per ADR-0298 §B) — emergency-
   services traffic bypasses detection-flag → still gets through.
   Audit-and-investigate, never block. Detection records the
   anomaly but does not gate the action.
3. Critical-path exemption per §3.2.5 (healthcare-acute-care, crisis-
   line, financial-emergency, DV-survivor-shelter-mode) — same
   bypass with audit trail.
4. Appeal mechanism mandatory for adverse action per GDPR Art. 22 +
   EU AI Act Art. 86 + ECOA Reg B + NY AEDT 2023 (cross-ref
   ADR-0309 + ADR-0310). Detection substrate emits
   `DetectionSignalEmitted` with appeal-mechanism-link populated
   from the per-pack appeal routing table.

## §C. Consequences

The 6 engineering-rigor dimensions per documentation-rigor.md §1.2:

### §C.1. Maintainability dimension

The detection substrate's maintainability surface is concentrated in
single-concern crates (oya-shared-detection-streaming, -batch,
-feature-store, -rules-engine, -composite-scorer, -graph-store,
-sandbox-replay) under microservices/detection/src/. Per-family
rule files are flat-organized under microservices/detection/rules/
{payment_fraud, ato, synthetic_identity, aml_sanctions, content_abuse,
engagement_manipulation, insider_risk, policy_violation}/*.yaml
where each rule file matches the detection-rule-schema.json shape.

Versioning policy: every crate ships SemVer per ADR-0258; the
detection-rule-schema.json is versioned via `_meta.schema_version`
and breaking changes require ADR amendment + 60-day deprecation
cadence (per the markdown-retirement-policy). Feature store schema
breakage requires explicit ADR + a re-train of all consuming
models (because feature semantics change can invalidate model
behavior).

Per-config-flag rationale: the substrate ships ~50 per-tenant
config flags (per-family rule activation, per-family threshold,
per-tenant feature store overlay, per-pack regulator override). Each
flag has a documented default + per-pack override behavior + per-
tenant migration path. No flag is "tribal knowledge" — every flag
is in the per-µservice manifest.json's `detection_config_flags` map
and audited daily by `oya-governance-detection-config-flag-coherence`.

Reverse dependencies: payments + identity + marketplace + social +
community + messenger + mail + ops-dashboard-control-center +
intelligence + audit-chain consume detection signals. The reverse
dependency list lives in `microservices/detection/manifest.json:reverse_dependencies`
and is maintained by the per-µservice migration playbook (§F.2).

### §C.2. Observability dimension

Per ADR-0263 emission contract, the detection substrate emits seven
new audit-event-classes registered in the central registry:

| Class | Cardinality budget | Trace span shape | Retention |
|---|---|---|---|
| `DetectionSignalEmitted` | ~10⁹/day at platform GA | Parent: streaming-pipeline-job; Child: per-family-evaluator | 7-year cold (audit-chain) + 90-day hot (ClickHouse) |
| `DetectionRulePromoted` | ~10²/day | Parent: rule-publish-API; Child: soak-verification + sandbox-replay | 7-year cold |
| `DetectionRuleSunset` | ~10/day | Parent: rule-sunset-API | 7-year cold |
| `DetectionModelDeployed` | ~10/day | Parent: model-deploy-API | 7-year cold (per EU AI Act Art. 18 model documentation retention) |
| `DetectionModelRolledBack` | ~1/day at platform GA (rare) | Parent: model-rollback-API | 7-year cold + EU AI Act Art. 73 serious-incident report |
| `DetectionDriftAlertTriggered` | ~10³/day | Parent: drift-detection-batch | 90-day hot + 7-year cold |
| `DetectionSignalConsumed` | ~10⁹/day | Parent: consuming-µservice-action; Child: cedar-eval | 90-day hot |

Metrics (Prometheus + OpenTelemetry per ADR-0263):

- `detection_signal_emitted_total{family, tenant_id, severity}` — counter; cardinality budget tenant_id × family × severity = ~10⁶
- `detection_signal_latency_seconds{family, p50|p95|p99}` — histogram; per-family P50 ≤ 200ms, P95 ≤ 1s, P99 ≤ 5s
- `detection_rule_evaluations_total{rule_id, lifecycle_state, outcome}` — counter
- `detection_model_inference_latency_seconds{model_id, p50|p95|p99}` — histogram; P50 ≤ 50ms, P99 ≤ 500ms
- `detection_feature_store_read_latency_seconds{store, p50|p95|p99}` — histogram; P99 ≤ 50ms online tier
- `detection_drift_score{model_id, drift_type}` — gauge; per-feature/label/concept drift
- `detection_appeal_filed_total{tenant_id, family}` — counter; for GDPR Art. 22 + AI Act Art. 86 SLA dashboards
- `detection_false_positive_rate{family, jurisdiction, protected_class}` — gauge; quarterly fairness audit dashboard

Dashboards (Grafana, stored in microservices/detection/dashboards/):

1. `detection-substrate-overview.json` — top-level per-family signal volume + latency + FPR + appeal volume
2. `detection-streaming-pipeline.json` — Flink job-level metrics (checkpoint latency, watermark drift, throughput)
3. `detection-batch-pipeline.json` — Spark job duration + stage metrics + backfill lag
4. `detection-feature-store.json` — per-feature freshness + read latency + cache hit rate
5. `detection-rules-engine.json` — per-rule lifecycle state + evaluation count + outcome distribution
6. `detection-composite-scorer.json` — per-entity risk-score distribution + LIME/SHAP feature-importance heatmap
7. `detection-graph-store.json` — graph size + Louvain run duration + community count per family
8. `detection-sandbox-replay.json` — replay-test pass-rate + soak-window time-remaining per pending rule/model
9. `detection-fairness-quarterly.json` — per-class TPR/FPR + disparate-impact ratio per protected class per jurisdiction (cross-ref ADR-0309)
10. `detection-appeal-mechanism.json` — appeal SLA tracking per GDPR Art. 22 + EU AI Act Art. 86

SLO floor (per `microservices/detection/slos/*.openslo.yaml`):

- Streaming-signal-latency P99 ≤ 5s (per-family); 99.9% monthly availability
- Batch-backfill completion ≤24h per daily job; 99.5% monthly availability
- Feature-store online read P99 ≤ 50ms; 99.95% monthly availability
- Composite-scorer inference P99 ≤ 500ms; 99.9% monthly availability
- Rule promotion soak window ≥7 days enforced; 100% (BLOCKER lane)
- Appeal SLA per pack honored; 99.5% monthly (per ADR-0309)

### §C.3. Scalability dimension

Capacity math (Little's Law + percentile arithmetic + queue theory
steady-state per documentation-rigor.md §1.1 item 3):

**Streaming pipeline.** Audit-event ingest at platform GA targets
~10⁹ events/day = ~11,600 events/sec sustained, ~5×10⁴ events/sec
peak (5x burst factor). Flink topology sizing: per-event processing
cost ~200µs CPU + ~10KB shuffle bandwidth; per task-manager at
8-core / 32GB heap handles ~4×10⁴ events/sec at 50% headroom. At
peak 5×10⁴ events/sec, ~2 task-managers required for baseline +
~2 task-managers for headroom = ~4 task-managers per Tier-2 cell
× ~6 cells (per ADR-0248 cellular topology) = ~24 task-managers
platform-wide. Horizontal scale-out path: add task-managers; Flink
re-shards via key-group rescaling without job restart (Flink 1.20
LTS feature). Bottleneck: Kafka broker shard count must keep pace
— each Flink subtask consumes 1 Kafka partition, so platform-wide
Kafka partitions ≥ task-managers × parallelism. Budgeted at
platform GA: 256 partitions per topic.

**Batch pipeline.** Daily Spark backfill over 90-day audit-event
window ~9×10¹⁰ events × 1KB = ~90TB total; per-job runtime budget
≤24h. Spark cluster sizing: 32 executor × 8-core × 32GB heap =
256 cores × 1TB heap; at ~30MB/sec/core sustained Spark throughput,
~7.7GB/sec aggregate ≈ ~28TB/hour ≈ ~14h for 90TB (well under 24h
budget with 30% headroom). Horizontal scale-out: more executors;
Spark dynamic-allocation scales workers per stage.

**Feature store.** Online tier (Redis-class) sized for ~10⁸ entities
× ~200 features × 8 bytes = ~160GB working set; sharded across ~16
Redis nodes per Tier-2 cell. Read QPS budget ~10⁵/sec per cell;
Redis sustains ~10⁵-10⁶ ops/sec per node. Offline tier (Parquet +
Iceberg on S3-class object storage) sized for ~1PB at platform GA;
columnar compression yields ~4x compression so ~250TB on-disk.

**Composite scorer.** LightGBM inference at ~1ms/inference per
LightGBM 4.x benchmarks (one-tree-ensemble of ~200 trees);
batched inference at ~10µs/inference. At ~10⁵/sec inference QPS
× 1ms = ~100 cores × headroom = ~150 cores per cell; horizontal
scale-out via stateless replicas behind L7 load balancer.

**Graph store.** Apache AGE on Postgres sized for ~10⁹ edges +
~10⁸ vertices at platform GA; Louvain community detection at
~O(E log V) ≈ ~10¹⁰ ops ≈ ~hours per run; runs weekly per family;
horizontal scale-out via per-family graph sharding (each family's
graph is independent so per-family Postgres+AGE cluster).

10× and 100× scale-out path: every primitive scales horizontally
without architectural change. The bottlenecks at 100× are: (a)
Kafka partition count → bumping from 256 to 25,600 (Kafka supports
this); (b) Feast online tier → bumping from Redis to ScyllaDB at
100× working set (10's-of-TB); (c) Graph store → bumping from AGE
to Neo4j Aura Enterprise or per-family sharding into per-tenant
graph isolation; (d) ClickHouse cold tier → bumping from per-cell
to per-region clusters with per-pack residency overlay.

### §C.4. Performance dimension

Per-primitive P50/P95/P99 targets (per documentation-rigor.md §1.2
Performance dimension):

| Primitive | P50 | P95 | P99 | Tail mitigation |
|---|---|---|---|---|
| Streaming signal emission | 100ms | 800ms | 3s | Hedging on Flink subtask hot-spots; per-partition timeout |
| Batch backfill (per daily job) | 12h | 18h | 22h | Spark dynamic-allocation + speculative execution |
| Feature store online read | 5ms | 20ms | 40ms | Read-replica fan-out-and-take-first; per-shard hedging |
| Feature store offline read (Parquet+Iceberg) | 1s | 5s | 10s | Per-partition predicate-pushdown; columnar projection-pushdown |
| Composite scorer LightGBM inference | 1ms | 5ms | 20ms | Batched inference; vectorized score path |
| Graph Louvain community detection | 1h | 4h | 8h | Per-family parallel run; checkpoint-and-resume |
| Sandbox replay (per 90-day window) | 6h | 12h | 18h | Spark dynamic-allocation; replay-time-window-partitioning |

Per-region budget split: each Tier-2 cell sized for its regional
audit-event volume; per ADR-0240 sovereign-cloud overlay, EU-
sovereign cells run independent Flink + Spark clusters from US +
KR cells. Cross-region replication for the feature store online
tier uses Postgres logical replication (active-passive) with
≤60s lag floor.

Cold-start budget: detection substrate µservice cold-start ≤30s
per Tier-2 cell (Flink JobManager startup + Spark Driver pool warm
+ Feast online tier warm). Cold-start is rare (planned restarts
during minor version upgrades; the Flink savepoint mechanism
ensures state-preservation across upgrades).

### §C.5. Optimization dimension

Per-call cost model (per documentation-rigor.md §1.2 Optimization):

- Streaming signal: ~200µs CPU + ~10KB shuffle + ~2KB feature-store
  read = ~50µCPU-cents + ~$0.0001/M-events at platform-GA cost
- Batch backfill: ~$15-25 per daily 90TB job (Spark on K8s + S3
  storage tier costs at platform-GA pricing)
- Feature-store online read: ~10µs CPU + ~1KB Redis network = ~$0.000001
  per read; at 10⁵/sec QPS = ~$0.1/sec = ~$8,640/day per cell;
  per platform with 6 cells = ~$52,000/day = ~$1.6M/month at platform
  GA. Optimization: per-feature TTL + LRU eviction; offline-only
  features (never cache); cache-warming on signal-likely cohort.
- Composite scorer inference: ~$0.00002 per inference; at 10⁵/sec
  QPS = $1.7/sec = ~$148,000/month per cell.
- Graph Louvain: ~$50-100 per weekly per-family run.

Lazy vs eager trade-offs:

- **Lazy (compute-on-demand)** for offline-tier feature store reads
  (rare access pattern); for graph queries (Louvain runs scheduled,
  not on every action).
- **Eager (compute-and-store)** for streaming feature aggregations
  (velocity_24h, velocity_30d) — eager because the alternative is
  90-day-window-scan-on-every-event which violates the P99 budget.
- **Cached** for sanctions-list match results (per-name match
  cached for 24h; OFAC SDN updates trigger cache invalidation
  per sanction-list-refresh event).

Cache-invalidation policy: feature store online tier honors per-
entity TTL (per-family default 60s for velocity features, 5min for
demographic features, 24h for sanctions match); invalidation on
upstream µservice's audit-event-class arrival (e.g., new
PaymentChargeSucceeded → invalidate per-user velocity_24h feature).

Cold-vs-warm path latency: cold (first scoring per user) ≈ 50ms
(includes feature-store fill from offline tier); warm (subsequent
scoring) ≈ 5ms (online tier hit). Substrate budget for cold path
is 100ms P95; warm path is 20ms P95.

Profiling evidence: per microservices/detection/perf/*.bench.rs
benchmarks gated by `oya-governance-perf-regression`.

### §C.6. Code quality dimension

Per documentation-rigor.md §1.2 Code quality dimension:

- **Test classes:** unit (per-rule, per-feature, per-scorer; ~10⁴
  test cases), property-based (proptest crate; ~50 properties on
  rule grammar + feature semantics), fuzz (cargo-fuzz on rule
  parser + audit-event ingestor; ~24h continuous fuzzing per
  release), load (k6 + Locust for streaming-pipeline P99; ~10⁵/sec
  sustained for 1h per release), e2e (full DRMP loop with synthetic
  fraud-ring scenario; ~50 scenarios per family).
- **Coverage floor:** ≥85% line, ≥75% branch per `cargo-llvm-cov`
  + `cargo-tarpaulin`. Per-family rule coverage ≥90% (rule logic
  is the substrate's core, higher bar).
- **Lint passes:** `cargo clippy -- -D warnings`, `oya-check-cedar-fragment-soak`,
  `oya-check-naming-bnf-v4`, `oya-check-layer-enum-conformance`,
  `oya-check-data-class-registry-binding`, `oya-check-detection-rule-schema`,
  `oya-check-detection-feature-data-class-tag`,
  `oya-governance-detection-substrate-emission`.
- **Type-strictness:** Rust `deny(warnings)` + `deny(unsafe_code)`
  at crate root for all oya-shared-detection-* crates (per
  ADR-0211 in-house tech stack preference). Cross-crate APIs
  exposed via traits with `#[non_exhaustive]` to allow additive
  evolution without SemVer-major bumps.
- **SemVer + ABI policy:** per ADR-0258; major bumps require ADR
  amendment + 60-day deprecation; minor bumps additive-only; patch
  bumps bug-fix-only. ABI policy: stable across patch + minor;
  breaking across major.

## §D. Detailed mechanics

### §D-1. Streaming pipeline — Apache Flink topology

#### §D-1.1. Topology shape

The streaming pipeline is one Flink job per detection family
(eight jobs total), each consuming a subset of Kafka topics
matching the audit-event-classes that family scores. Topology:

```
Kafka source (per-family topic subscription)
    └─> Per-event deserialization (per-event-schema-registry lookup)
        └─> Per-entity keyBy (tenant_id × entity_id, where entity
            is user / transaction / content / employee / Cedar-principal)
            └─> Stateful enrichment (feature-store online read)
                └─> Per-rule evaluation (rules-engine evaluator)
                    └─> Per-family ML scorer (LightGBM via JNI)
                        └─> Composite scorer aggregator
                            └─> Signal emitter (Kafka sink to investigation queue)
                                └─> Audit-event emitter (DetectionSignalEmitted)
```

Per-family Flink job parameters:

- Parallelism: 16 (Tier-2 cell baseline); 64 (Tier-2 cell peak via
  Flink reactive mode); per-cell scaled by audit-event ingest rate.
- Checkpoint interval: 60s (per Flink 1.20 LTS default for state-
  backed jobs); checkpoint backend: RocksDB on S3-class object
  storage.
- State TTL: per-feature TTL honored (velocity_24h state retained
  24h; velocity_30d retained 30d).
- Watermark strategy: bounded-out-of-orderness 5s; allowed lateness
  10s; late events route to side output for batch backfill.

#### §D-1.2. Per-family Flink jobs (listing)

Eight jobs, each in `microservices/detection/src/streaming/`:

1. `streaming/payment_fraud_streaming.rs` — Kafka topics:
   `audit.payment.charge_attempted`, `audit.payment.charge_succeeded`,
   `audit.payment.charge_declined`, `audit.payment.refund_requested`,
   `audit.payment.payout_attempted`, `audit.payment.dispute_filed`
2. `streaming/ato_streaming.rs` — Kafka topics:
   `audit.identity.sign_in_attempted`, `audit.identity.sign_in_succeeded`,
   `audit.identity.sign_in_failed`, `audit.identity.password_reset_requested`,
   `audit.identity.mfa_challenge_completed`,
   `audit.identity.session_created`, `audit.identity.device_registered`
3. `streaming/synth_identity_streaming.rs` — Kafka topics:
   `audit.identity.kyc_attempted`, `audit.identity.kyc_completed`,
   `audit.identity.kyb_attempted`, `audit.identity.kyb_completed`
4. `streaming/aml_streaming.rs` — Kafka topics:
   `audit.payment.charge_succeeded`, `audit.payment.payout_attempted`,
   `audit.payment.transfer_initiated`
5. `streaming/content_abuse_streaming.rs` — Kafka topics:
   `audit.messenger.message_sent`, `audit.mail.delivery_attempted`,
   `audit.community.post_created`, `audit.marketplace.listing_created`,
   `audit.social.post_created`, `audit.shorts.video_uploaded`
6. `streaming/engagement_manipulation_streaming.rs` — Kafka topics:
   `audit.marketplace.review_submitted`, `audit.social.follow_created`,
   `audit.community.vote_cast`, `audit.community.upvote_cast`,
   `audit.shorts.like_recorded`
7. `streaming/insider_risk_streaming.rs` — Kafka topics:
   `audit.ops.admin_action_performed`, `audit.ops.jit_access_granted`,
   `audit.ops.jit_access_revoked`, `audit.hr.employee_terminated`,
   `audit.identity.sensitive_resource_accessed`
8. `streaming/policy_violation_streaming.rs` — Kafka topics:
   `audit.cedar.evaluation_completed`, `audit.cedar.fragment_published`,
   `audit.cedar.fragment_sunset`, `audit.audit_chain.seal_verified`,
   `audit.audit_chain.seal_failed`, `audit.network.policy_violation`,
   `audit.sidecar.attestation_completed`, `audit.sidecar.exit_anomaly`

#### §D-1.3. Per-job state shape

State backend RocksDB on S3-class storage. Per-job state:

- ValueState per-entity-key for current feature snapshot (read-
  through to feature store online tier)
- ListState per-entity-key for recent-event window (last N events
  for streaming aggregation)
- MapState per-rule-id for per-rule firing-window (de-dup against
  rule re-firing inside short window)
- ReducingState per-tenant for aggregate signals (per-tenant
  velocity, per-tenant burst count)

State TTL per Flink 1.20 LTS native TTL feature; per-feature TTL
configured at job-submit time.

#### §D-1.4. Failure modes — Flink-level

Per documentation-rigor.md §1.1 item 2 failure-mode tree:

1. **JobManager crash.** Flink HA via Zookeeper (one ZK quorum per
   cell); JobManager restart ≤30s; checkpoint resume from RocksDB
   ≤60s.
2. **TaskManager crash.** Flink Task slot reassignment ≤60s;
   checkpoint state reloaded ≤2min.
3. **Kafka partition outage.** Per-partition consumer pauses;
   Flink watermark stalls; late events route to side output for
   batch backfill (no signal loss).
4. **Feature-store outage.** Per-feature fallback to last-known
   value with staleness flag; alerts at staleness > 5min; signal
   emitted with `feature_staleness=stale` for downstream
   confidence-discount.
5. **Composite-scorer model outage.** Per-family fallback to
   previous champion model; alerts at fallback-active > 5min;
   never falls back to "no scoring" — fail-safe default is the
   most-recent known-good model.
6. **Kafka producer outage (sink).** Buffer at Flink sink with
   backpressure; alerts at buffer-depth > 1M events; downstream
   investigation queue eventually-consistent.

### §D-2. Batch pipeline — Apache Spark

#### §D-2.1. Job-set shape

Daily + weekly + monthly Spark jobs over the ClickHouse cold-tier
audit-event lake + Parquet + Iceberg-backed historical store.
Eight per-family daily jobs + per-family weekly graph recomputation
+ monthly model-retraining trigger.

```
ClickHouse cold tier (90d retention) + Iceberg lake (7y retention)
    └─> Spark Driver (per-family)
        └─> Spark Executors (32× per cell)
            └─> Stage 1: extract per-family audit events for the window
                └─> Stage 2: per-entity feature recomputation
                    └─> Stage 3: graph construction (per-family)
                        └─> Stage 4: graph algorithm (Louvain / connected-components)
                            └─> Stage 5: per-rule batch evaluation
                                └─> Stage 6: per-family composite re-scoring
                                    └─> Stage 7: write to feature store offline tier
                                        └─> Stage 8: emit DetectionSignalEmitted batch
```

#### §D-2.2. Job listing

Daily jobs (one per family, run nightly per cell):

1. `batch/payment_fraud_backfill.scala` — daily 90-day window
2. `batch/ato_backfill.scala` — daily 90-day window
3. `batch/synth_identity_backfill.scala` — daily 90-day window
4. `batch/aml_backfill.scala` — daily 90-day window
5. `batch/content_abuse_backfill.scala` — daily 30-day window
   (hashes refresh sub-daily)
6. `batch/engagement_manipulation_backfill.scala` — daily 30-day
   window + weekly Louvain
7. `batch/insider_risk_backfill.scala` — daily 90-day window per-
   employee baseline
8. `batch/policy_violation_backfill.scala` — daily 30-day window
   + weekly Cedar fragment baseline recomputation

Weekly jobs (graph-recomputation, one per family):

- `batch/payment_fraud_graph_recompute.scala` — Louvain over
  merchant-BIN-card tripartite
- `batch/aml_graph_recompute.scala` — Louvain + Label Propagation
  over transaction graph
- `batch/synth_identity_graph_recompute.scala` — Louvain over
  corp-director-address graph
- `batch/engagement_manipulation_graph_recompute.scala` — Louvain
  over review-author-product graph
- `batch/insider_risk_graph_recompute.scala` — connected-
  components over employee-resource access graph

Monthly jobs (model-retraining trigger; full retraining lives in
ADR-0308):

- `batch/model_retrain_trigger.scala` — per-family drift check
  triggers ADR-0308's training pipeline if drift threshold crossed
  in any of the last 30 daily runs

#### §D-2.3. Polars + ClickHouse + Trino alternative path

For sub-TB batch workloads (insider-risk per-employee baseline,
synthetic-identity per-corp directorship), Polars (Rust-native
columnar) provides ~5-10x faster runtime than Spark on a single
node. The substrate routes per-job execution to Polars when the
input-size estimator < 1TB; Spark otherwise. Trino provides
federated query across Spark + ClickHouse + Postgres + Iceberg
for ad-hoc investigation queries (cross-ref ADR-0310).

### §D-3. Feature store — per-tenant isolation + online/offline split

#### §D-3.1. Architecture

Feast 0.40+ as canonical reference; per-entity feature definitions
under `microservices/detection/features/` matching the
detection-feature-schema.json. Online tier Redis-class (Redis 7 +
Redis Cluster mode); offline tier Parquet + Iceberg on S3-class
object storage. Per ADR-0240 sovereign-cloud overlay, Tier-3+ cells
may substitute Tecton (US managed) or Vertex AI Feature Store (GCP
managed) — but the canonical reference is Feast.

#### §D-3.2. Per-feature definition

Every feature declares:

- `feature_id` — globally unique slug (e.g., `feature.payment.velocity_24h`)
- `entity` — `user` / `tenant` / `transaction` / `content` / `employee` / `principal`
- `value_type` — int64 / float32 / string / boolean / bytes / vector<float32>
- `data_class` — per ADR-0099 data-class registry (PII / PSEUDONYMOUS / AGGREGATE / DERIVED / NON-PII)
- `online_ttl` — seconds (e.g., 86400 for velocity_24h)
- `offline_retention_days` — int (e.g., 2555 for 7-year-audit)
- `compliance_pack_overlay` — per-pack feature-availability table (HIPAA → NULL for PHI-derived features; GDPR-EU → pseudonymized; KR-PIPA → pseudonymized; CCPA → opt-out-honoring; COPPA → never-available for <13 users)
- `binding_adr` — `ADR-0307` (the binding ADR for this feature definition format)

#### §D-3.3. Per-tenant feature isolation

Per ADR-0244 tenant scoping invariant, every feature row in the
online + offline tiers carries `tenant_id` as the primary scope key.
Cross-tenant feature read is forbidden by Cedar gate (per
ADR-0243). Per-pack overlay enforced at write time — HIPAA-pack
tenants cannot write PHI-derived features to the offline tier (the
substrate refuses with `DataClassPackOverlayViolation` audit event).

#### §D-3.4. Online tier sizing + Redis topology

Per-cell Redis Cluster (6-shard × 3-replica = 18 nodes per cell);
~160GB working set per cell at platform-GA load. Per-feature TTL
enforced via Redis EXPIRE; per-tenant key prefix `tenant:{tenant_id}:`
to enable per-tenant key-scan + per-tenant eviction (per pack
removal request per GDPR Art. 17 + KR-PIPA).

#### §D-3.5. Offline tier — Iceberg on S3-class object storage

Parquet files with per-feature column projection; Iceberg manifest
metadata for time-travel queries (per EU AI Act Art. 18 model
training reproducibility requirement: training data MUST be
reproducible from the model card's training-data-snapshot reference).
Per-tenant Iceberg namespace; cross-tenant scan forbidden.

### §D-4. Rules engine — Sigma-rule-class declarative DSL

#### §D-4.1. Rule grammar

Per the detection-rule-schema.json, rules are YAML with:

```yaml
id: <slug>                # globally unique rule id
title: <human-readable>
description: <human-readable>
family: <one of 8>        # payment_fraud / ato / etc
status: Proposed | Soaking | Active | Sunset
lifecycle:
  proposed_at: <RFC3339>
  soaking_at: <RFC3339>
  soak_window_seconds: 604800   # ≥7d for detection rules
  active_at: <RFC3339?>
  sunset_at: <RFC3339?>
soak_metrics:
  false_positive_rate_ceiling: 0.03    # ≤3% to promote
  analyst_label_ground_truth_size: 1000  # ≥1000 to promote
selection:
  audit_event_class: <regex on event-class name>
  filters:
    - field: <feature_id or event_field>
      op: gt | lt | eq | regex | in | contains
      value: <constant or feature reference>
condition: <boolean expression over selections>
timeframe: <duration; e.g., 5m, 1h, 24h>
output:
  signal_severity: low | medium | high | critical
  signal_payload:
    family: <slug>
    features_contributing_most: <list of feature_id>
binding_adr: ADR-0307
naming_justification: <text>
```

#### §D-4.2. Rule lifecycle — Proposed → Soaking → Active → Sunset

Mirrors Cedar fragment lifecycle per ADR-0294. Differences:

- **Soak window:** ≥7 days for detection rules (vs ≥60s for Cedar
  fragments). Detection rules need many false-positive-rate
  measurement cycles before promotion; Cedar fragments are deterministic
  permits and can promote after a short soak.
- **Soak metrics:** detection rules require analyst-labeled ground
  truth of ≥1000 events + false-positive-rate ≤3% to promote.
- **Sunset:** detection rules sunset when superseded by ML model
  or by amended rule; sunset retention ≥90d for audit (per ADR-0263).

#### §D-4.3. Rule evaluator — Rust-native + Arrow-aware

Per microservices/detection/src/rules/. Evaluator:

- Compiles each rule's `condition` into an AST + tree-walking
  interpreter
- For hot rules (>10⁴ evaluations/sec sustained), JITs to LLVM
  IR via cranelift or inkwell
- Operates over Apache Arrow record batches for vectorized
  evaluation
- Per-rule rate limits + soak-window enforcement at evaluation
  time (Proposed rules emit signal with `shadow_mode=true`; Active
  rules emit with `shadow_mode=false`)

### §D-5. Composite scorer — LIME/SHAP explainable

#### §D-5.1. Architecture

Per microservices/detection/src/scorer/. Per-family LightGBM
ensemble (~200 trees × ~12 depth per tree) trained on feature-store
offline tier; inference at ~1ms latency P50. SHAP TreeExplainer
runs per inference to compute feature-importance values; LIME
runs as fallback for non-tree-based models.

#### §D-5.2. Per-jurisdiction model variants

Per ADR-0309 fairness invariants, the composite scorer runs a global
model + per-jurisdiction overlay (e.g., EU AI Act forbids social-
scoring features; KR Financial Consumer Protection Act forbids
certain protected-class proxies). Overlay enforced at evaluation
time via per-jurisdiction feature-mask config.

#### §D-5.3. Explainability output

Every signal carries:

- `top_5_features_contributing` — list of (feature_id, SHAP-value, direction)
- `model_id`, `model_version`, `model_card_url`
- `appeal_mechanism_link` — per-pack appeal route per ADR-0309 + ADR-0310

Per GDPR Art. 22 + EU AI Act Art. 13 + ECOA Reg B + NY AEDT 2023
adverse-action notice requirements.

### §D-6. Graph store + community detection

#### §D-6.1. Apache AGE (Postgres+graph extension) — canonical

Per-family Postgres+AGE cluster under
`microservices/detection/graph/`. Cypher-compatible interface;
Louvain community detection + Label Propagation + PageRank +
connected-components algorithms exposed as stored procedures.

#### §D-6.2. Neo4j alternative for Tier-3+ cells

Per ADR-0240 sovereign-cloud overlay, Tier-3+ cells may substitute
Neo4j Aura Enterprise (US managed) or in-region Neo4j Bloom for
EU + KR + JP deployments. The substrate's graph-store trait is
implementation-agnostic; per-cell config selects backend.

#### §D-6.3. Per-family graph schema

- **Payment fraud:** merchant-BIN-card-transaction quadripartite;
  edges: `(merchant)-[:ACCEPTED]->(card)`, `(card)-[:HAS_BIN]->(BIN)`,
  `(transaction)-[:USES]->(card)`, `(transaction)-[:CHARGED]->(merchant)`.
  Louvain detects refund-fraud rings; PageRank scores high-volume
  fraud-adjacent merchants.
- **AML:** transaction-graph; edges: `(account)-[:SENDS]->(account)`,
  `(account)-[:OWNED_BY]->(entity)`, `(entity)-[:LOCATED_IN]->(jurisdiction)`.
  Louvain detects layering rings; Label Propagation detects mule-
  account communities.
- **Synthetic identity:** corp-director-address-phone quadripartite;
  edges: `(corp)-[:HAS_DIRECTOR]->(person)`, `(corp)-[:AT_ADDRESS]->(address)`,
  `(corp)-[:HAS_PHONE]->(phone)`. Louvain detects mule-corp clusters;
  connected-components detect shared-address shell entities.
- **Engagement manipulation:** review-author-product tripartite;
  edges: `(author)-[:REVIEWED]->(product)`, `(author)-[:PURCHASED]->(product)`,
  `(author)-[:LOCATED_IN]->(IP)`. Louvain detects paid-review rings;
  connected-components detect click-farm IP clusters.
- **Insider risk:** employee-resource access graph; edges:
  `(employee)-[:ACCESSED]->(resource)`, `(employee)-[:MEMBER_OF]->(role)`,
  `(role)-[:GRANTED]->(resource)`. Connected-components detect
  over-privileged-role + sensitive-resource-cluster patterns.

### §D-7. Investigation case-management integration

Detection signals route to investigation queue per ADR-0310;
bidirectional — detection emits signal, investigation feeds analyst
labels back to the feature store for model retraining. Per
microservices/detection/src/investigation_bridge/.

The bridge:

- Subscribes to investigation case-management's
  `InvestigationCaseAdjudicated` events (per ADR-0310)
- Writes analyst-labels back to feature-store offline tier as
  `feature.investigation.analyst_label_per_signal`
- Triggers model-retrain on label-volume threshold (per family,
  ≥1000 new labels triggers retrain queue per ADR-0308)

### §D-8. Sandbox + replay

Any new rule or new model can be back-tested against historical
audit-stream from ClickHouse cold tier before promotion to Active.
Sandbox runs the same Flink topology parameterized by replay-time-
window:

- Sandbox topology reads from ClickHouse cold tier instead of Kafka
  (uses Flink's BoundedSource API with bounded-watermark)
- Per-event timestamps from the historical stream drive watermark
  progression
- Sandbox emits signals to a `sandbox-signals` Kafka topic (not
  routed to investigation queue) for offline analysis
- Promotion gate: ≥7-day soak + ≤3% false-positive rate + sandbox-
  replay-pass on the last 90d historical stream

### §D-9. Per-family interactions with abuse-defence + emergency-services

Per §3.2.6 critical-path invariants:

- **Emergency-services bypass (per ADR-0298).** Detection MUST NOT
  block emergency-services traffic. Per ADR-0298 §B, emergency-
  services tenants are tagged with `tenant.audience_type=EMERGENCY_SERVICE`;
  per-family rules MUST include a `where audience_type != EMERGENCY_SERVICE`
  predicate or equivalent bypass. Detection still emits signal +
  audits + investigates but does not gate the action.
- **DV-survivor-shelter-mode (per §3.2.5 row 8).** Detection MUST
  NOT notify the abuser-party of survivor-party actions. Per-signal
  notification routing respects shelter-mode flag.
- **Per-pack regulator floor preserved.** Mitigation cadence honors
  per-pack regulator timing (no faster, no slower where regulator
  has explicit floor). Detection emits signal at substrate cadence;
  mitigation downstream respects pack floor.

## §E. Implementation footprint

### §E.1. New µservice — microservices/detection/

Single-concern flat µservice per ADR-0131 + ADR-0132. Directory
tree:

```
microservices/detection/
├── manifest.json
├── PRD.md
├── PHASE-01-streaming-pipeline.md
├── PHASE-02-batch-pipeline.md
├── PHASE-03-feature-store.md
├── PHASE-04-rules-engine.md
├── PHASE-05-composite-scorer.md
├── PHASE-06-graph-store.md
├── PHASE-07-investigation-bridge.md
├── PHASE-08-sandbox-replay.md
├── threat-model.md
├── dpia.md
├── ARCHITECTURE.md
├── README.md
├── CHANGELOG.md
├── capacity-model.md
├── cost-budget.md
├── failure-modes.md
├── multi-region.md
├── incident-response.md
├── backfill-replay.md
├── compliance.md
├── competitor-parity-matrix.md
├── sdk-plan.md
├── policy/
│   ├── detection-default-deny.cedar
│   ├── detection-substrate.cedar
│   ├── detection-rule-publish.cedar
│   ├── detection-rule-sunset.cedar
│   ├── detection-feature-write.cedar
│   ├── detection-signal-consume.cedar
│   ├── data-residency.md
│   ├── auditor-scope.cedar
│   └── ci-scope.cedar
├── runbooks/
│   ├── detection-rule-publish.md
│   ├── detection-rule-sunset.md
│   ├── detection-rule-soak-failure.md
│   ├── detection-model-rollback.md
│   ├── detection-drift-alert.md
│   ├── detection-feature-store-failure.md
│   ├── detection-graph-store-rebuild.md
│   ├── detection-sandbox-replay-debug.md
│   └── detection-fairness-incident.md
├── contracts/
│   ├── openapi-v1.yaml
│   ├── asyncapi-v1.yaml
│   ├── detection-substrate-v1.proto
│   └── metric-naming-convention.md
├── capabilities/
│   ├── streaming-pipeline.capability.yaml
│   ├── batch-pipeline.capability.yaml
│   ├── feature-store.capability.yaml
│   ├── rules-engine.capability.yaml
│   ├── composite-scorer.capability.yaml
│   ├── graph-store.capability.yaml
│   ├── sandbox-replay.capability.yaml
│   └── investigation-bridge.capability.yaml
├── dashboards/
│   ├── detection-substrate-overview.json
│   ├── detection-streaming-pipeline.json
│   ├── detection-batch-pipeline.json
│   ├── detection-feature-store.json
│   ├── detection-rules-engine.json
│   ├── detection-composite-scorer.json
│   ├── detection-graph-store.json
│   ├── detection-sandbox-replay.json
│   ├── detection-fairness-quarterly.json
│   └── detection-appeal-mechanism.json
├── slos/
│   ├── streaming-signal-latency.openslo.yaml
│   ├── batch-backfill-completion.openslo.yaml
│   ├── feature-store-online-read.openslo.yaml
│   ├── composite-scorer-inference.openslo.yaml
│   ├── rule-promotion-soak-window.openslo.yaml
│   └── appeal-sla.openslo.yaml
├── IP-001-streaming-pipeline-scaffolding.md
├── IP-002-batch-pipeline-scaffolding.md
├── IP-003-feature-store-feast-integration.md
├── IP-004-rules-engine-evaluator.md
├── IP-005-rules-engine-grammar.md
├── IP-006-composite-scorer-lightgbm.md
├── IP-007-composite-scorer-shap-explainability.md
├── IP-008-graph-store-age-integration.md
├── IP-009-graph-store-louvain.md
├── IP-010-sandbox-replay-harness.md
├── IP-011-investigation-bridge.md
├── IP-012-per-family-rules-payment-fraud.md
├── IP-013-per-family-rules-ato.md
├── IP-014-per-family-rules-synth-identity.md
├── IP-015-per-family-rules-aml.md
├── IP-016-per-family-rules-content-abuse.md
├── IP-017-per-family-rules-engagement-manipulation.md
├── IP-018-per-family-rules-insider-risk.md
├── IP-019-per-family-rules-policy-violation.md
├── IP-020-feature-definitions-per-family.md
├── catalog/
│   ├── oya-shared-detection-streaming.catalog.yaml
│   ├── oya-shared-detection-batch.catalog.yaml
│   ├── oya-shared-feature-store.catalog.yaml
│   ├── oya-shared-detection-rules-engine.catalog.yaml
│   ├── oya-shared-detection-composite-scorer.catalog.yaml
│   ├── oya-shared-graph-store.catalog.yaml
│   ├── oya-shared-detection-sandbox-replay.catalog.yaml
│   ├── oya-shared-detection-investigation-bridge.catalog.yaml
│   ├── microservice-detection-streaming-pipeline.catalog.yaml
│   ├── microservice-detection-batch-pipeline.catalog.yaml
│   └── microservice-detection-substrate.catalog.yaml
├── iac/
│   ├── dev-detection-flink-cluster.tf
│   ├── stg-detection-flink-cluster.tf
│   ├── prod-detection-flink-cluster.tf
│   ├── dev-detection-spark-cluster.tf
│   ├── prod-detection-spark-cluster.tf
│   ├── dev-detection-feast-redis-cluster.tf
│   ├── prod-detection-feast-redis-cluster.tf
│   ├── dev-detection-iceberg-s3-bucket.tf
│   ├── prod-detection-iceberg-s3-bucket.tf
│   ├── dev-detection-age-postgres-cluster.tf
│   ├── prod-detection-age-postgres-cluster.tf
│   ├── dev-detection-network-policy.yaml
│   ├── prod-detection-network-policy.yaml
│   ├── dev-detection-openbao-policy.yaml
│   └── prod-detection-openbao-policy.yaml
├── src/
│   ├── streaming/
│   │   ├── mod.rs
│   │   ├── payment_fraud_streaming.rs
│   │   ├── ato_streaming.rs
│   │   ├── synth_identity_streaming.rs
│   │   ├── aml_streaming.rs
│   │   ├── content_abuse_streaming.rs
│   │   ├── engagement_manipulation_streaming.rs
│   │   ├── insider_risk_streaming.rs
│   │   └── policy_violation_streaming.rs
│   ├── batch/
│   │   ├── mod.rs
│   │   ├── payment_fraud_backfill.scala
│   │   ├── ato_backfill.scala
│   │   ├── synth_identity_backfill.scala
│   │   ├── aml_backfill.scala
│   │   ├── content_abuse_backfill.scala
│   │   ├── engagement_manipulation_backfill.scala
│   │   ├── insider_risk_backfill.scala
│   │   ├── policy_violation_backfill.scala
│   │   └── graph_recompute/
│   ├── feature_store/
│   │   ├── mod.rs
│   │   ├── feast_bridge.rs
│   │   ├── online_tier_redis.rs
│   │   └── offline_tier_iceberg.rs
│   ├── rules/
│   │   ├── mod.rs
│   │   ├── parser.rs
│   │   ├── evaluator.rs
│   │   ├── jit.rs
│   │   └── lifecycle.rs
│   ├── scorer/
│   │   ├── mod.rs
│   │   ├── lightgbm_inference.rs
│   │   ├── shap_explainer.rs
│   │   └── per_jurisdiction_overlay.rs
│   ├── graph/
│   │   ├── mod.rs
│   │   ├── age_bridge.rs
│   │   ├── louvain.rs
│   │   └── connected_components.rs
│   ├── sandbox_replay/
│   │   ├── mod.rs
│   │   └── replay_harness.rs
│   ├── investigation_bridge/
│   │   ├── mod.rs
│   │   └── label_feedback.rs
│   └── lib.rs
├── rules/
│   ├── payment_fraud/
│   ├── ato/
│   ├── synth_identity/
│   ├── aml_sanctions/
│   ├── content_abuse/
│   ├── engagement_manipulation/
│   ├── insider_risk/
│   └── policy_violation/
├── features/
│   ├── payment.yaml
│   ├── identity.yaml
│   ├── kyc.yaml
│   ├── kyb.yaml
│   ├── aml.yaml
│   ├── content.yaml
│   ├── engagement.yaml
│   ├── insider.yaml
│   └── policy.yaml
├── AUDIT-FINDINGS-2026-05-20.json
└── scorecards/
    └── overrides.json
```

### §E.2. New crates (per layer-5 shared-substrate)

Per ADR-0105 13-layer canonical enum row 5:

1. `crates/oya-shared-detection-streaming/` — Flink job builder + per-family streaming-evaluator trait + per-rule streaming runtime
2. `crates/oya-shared-detection-batch/` — Spark/Polars job builder + per-family batch-evaluator trait
3. `crates/oya-shared-feature-store/` — Feast bridge + online/offline tier traits + per-tenant isolation
4. `crates/oya-shared-detection-rules-engine/` — Sigma-rule-class evaluator + JIT path
5. `crates/oya-shared-detection-composite-scorer/` — LightGBM inference + SHAP explainer
6. `crates/oya-shared-graph-store/` — Apache AGE + Neo4j abstraction + Louvain/connected-components/PageRank
7. `crates/oya-shared-detection-sandbox-replay/` — replay harness over ClickHouse cold tier
8. `crates/oya-shared-detection-investigation-bridge/` — bidirectional bridge to ADR-0310 case-management

### §E.3. New JSON Schemas

Under `/specs/`:

1. `detection-rule-schema.json` — Sigma-rule-class shape (per §D-4.1)
2. `detection-feature-schema.json` — feature definition shape (per §D-3.2)
3. `detection-signal-schema.json` — signal payload shape (per §D-1.1)
4. `detection-family-registry.json` — closed enum of 8 families

### §E.4. New runbooks (per microservices/detection/runbooks/)

9 runbooks listed in the tree above; each per §2 runbook rigor
(Trigger / Pre-checks / Procedure / Verification / Rollback /
Post-incident / References).

### §E.5. New CI lanes

- `oya-governance-detection-substrate-emission` — verifies every µservice emits declared events
- `oya-governance-detection-rule-lifecycle-soak` — verifies ≥7-day soak before promotion
- `oya-governance-detection-family-coverage` — verifies all 8 families have active rules + models
- `oya-governance-detection-feature-data-class-coherence` — verifies feature data-class matches ADR-0099 registry
- `oya-governance-detection-baseline` — aggregate lane

### §E.6. Per-µservice extensions

Every contributing µservice updates:

- `compliance.md §detection-substrate-binding` — per §3.2.6.H + §B.1 above
- `ARCHITECTURE.md §detection-integration` — which family/families it serves + which signals it consumes
- `manifest.json:detection_family_contributions` — array of family enums
- `manifest.json:detection_signal_consumption` — array of family enums + consumption action

### §E.7. Vendor selection rationale

#### §E.7.1. Streaming: Apache Flink 1.20 LTS

Selected because:
- Open-source + Apache 2.0 license (no vendor lock-in per
  ADR-0211 in-house tech stack preference)
- 1.20 is LTS through 2027; long-support window matches the
  substrate's lifecycle
- Native exactly-once semantics via 2-phase commit; required for
  audit-event-class consistency with ADR-0263
- Watermark + windowing + state-management built-in; matches the
  per-entity keyBy + stateful enrichment pattern
- ~10-100x faster than Kafka Streams on stateful aggregations per
  the 2024 nexmark-benchmark public results

Alternatives evaluated:
- **Kafka Streams.** Simpler operational model but stateful
  aggregations 10x slower; rejected for per-family scorers
- **Materialize.** PostgreSQL-compatible streaming SQL; rejected
  for vendor concentration (single-vendor commercial)
- **RisingWave.** Open-source streaming SQL; rejected for newer
  project (not LTS); revisit Year 3+

#### §E.7.2. Batch: Apache Spark 3.5 + Polars + ClickHouse + Trino

Selected because:
- Spark 3.5 is the de-facto batch-processing substrate (Stripe,
  Adyen, Toss all use Spark for fraud backfill per their published
  conference talks)
- Polars (Rust-native columnar) for sub-TB workloads; ~5-10x
  faster than Spark on single-node per Polars-vs-Spark benchmarks
- ClickHouse for OLAP audit-event lake (per existing
  cloud-iac/clickhouse-cluster-iac.yaml)
- Trino for federated query (AWS Athena equivalent; Stripe + Airbnb
  + Pinterest all run Trino at scale)

#### §E.7.3. Feature store: Feast 0.40+ (open-source canonical)

Selected because:
- Open-source + Apache 2.0 license
- Tecton + Vertex AI Feature Store are Feast-compatible (Feast
  is the open standard); per-cell deployment chooses backend
- Native online + offline split matching the §D-3 architecture
- Per-entity feature definitions match the §D-3.2 shape

Alternatives evaluated:
- **Tecton.** Managed commercial; supported as alternative for
  Tier-3+ cloud-managed cells per ADR-0240
- **Vertex AI Feature Store.** GCP-managed; supported for GCP cells
- **Hopsworks.** Open-source; rejected for smaller community vs Feast

#### §E.7.4. Graph store: Apache AGE (canonical) + Neo4j (alternative)

Selected because:
- Apache AGE runs as Postgres extension (substrate already runs
  Postgres per ADR-0145; one fewer external dependency)
- Cypher-compatible interface; standard graph query language
- Louvain + Label Propagation + PageRank exposed via stored procedures
- Per-cell deployment uses AGE; Tier-3+ cells may use Neo4j Aura
  Enterprise for managed offering

#### §E.7.5. Rules engine: Rust-native Sigma-rule-class evaluator

Selected because:
- In-house implementation per ADR-0211 (Rust-native, no Java/JVM
  dependency); avoids commercial SIEM lock-in
- Sigma rule grammar (github.com/SigmaHQ/sigma) is the de-facto
  open declarative-DSL for detection rules; ecosystem-friendly
- Native Apache Arrow integration for vectorized evaluation
- LLVM JIT path via cranelift for hot rules

## §F. Migration

### §F.1. Wave-3-D rollout sequencing

Per the keystone bundle 2026-05-20 Wave-3-D backlog (per
documentation-rigor.md §3.2.6.I), the detection substrate buildout
sequences:

1. **2026-05-20 to 2026-06-15.** ADR-0307 + ADR-0308 + ADR-0309 +
   ADR-0310 authored + accepted. Wave-3-D backlog populated with
   IP-001 through IP-020 atomic-PR-sized implementation plans.
2. **2026-06-15 to 2026-08-15.** Substrate µservice scaffold (per
   §E.1 directory tree) + initial crate skeletons + per-family
   Flink job stubs + Feast integration + Cedar policy fragment +
   IaC manifests + runbooks + dashboards + SLO declarations.
   `microservices/detection/` clears PR-143 ~70-artifact baseline
   floor.
3. **2026-08-15 to 2026-09-15.** Per-family rule authoring (4-6
   rules per family in Soaking state); Apache AGE deployment;
   composite scorer baseline (LightGBM per family); sandbox-replay
   harness operational.
4. **2026-09-15.** CI lanes promote to BLOCKER. Every contributing
   µservice MUST emit declared events; every Active rule MUST have
   passed ≥7d soak; all 8 families MUST have active coverage.
5. **2026-09-15 to 2026-12-15.** Per-µservice integration —
   payments + identity + marketplace + social + community + messenger
   + mail + ops-dashboard-control-center + intelligence + audit-chain
   each adds `compliance.md §detection-substrate-binding` +
   audit-event emission contract.
6. **2026-12-15 onwards.** Ongoing per-rule + per-model promotion;
   quarterly fairness audit (per ADR-0309); monthly model retrain
   trigger (per ADR-0308).

### §F.2. Per-µservice migration playbook

For each consuming µservice:

1. **Audit current detection logic.** Find any in-µservice
   detection rules / heuristics / ML models. Document them in
   `MIGRATION-2026-XX-detection-substrate.md`.
2. **Map to substrate families.** For each in-µservice rule,
   identify which of the 8 substrate families it belongs to.
3. **Author equivalent substrate rule.** Convert the rule to
   Sigma-rule-class YAML under microservices/detection/rules/<family>/;
   submit through Proposed → Soaking state.
4. **Run dual-detection during soak.** µservice runs both old
   in-µservice detection + new substrate detection in shadow mode
   for ≥7d.
5. **Compare false-positive + true-positive rates.** Substrate FPR
   must be within ±20% of old FPR (no degradation); TPR within
   ±5pp.
6. **Promote substrate rule + sunset in-µservice rule.** Soak
   passes; substrate rule promotes to Active; in-µservice rule
   sunsets per ADR-0294 deprecation cadence.
7. **Update compliance.md + ARCHITECTURE.md + manifest.json.**
   Per §E.6.
8. **Lane green.** `oya-governance-detection-substrate-emission`
   reports per-µservice green.

### §F.3. Per-cell rollout pattern

- Tier-0 edge cells: emit detection-signal-relevant events; no
  detection runtime
- Tier-1 bootstrap cell: emit only; no detection runtime
- Tier-2 control plane cells: full detection runtime (Flink +
  Spark + Feast + AGE + LightGBM)
- Tier-3 data plane cells: feature store offline tier + ClickHouse
  cold tier; no streaming runtime

### §F.4. What is NOT migrated

- Edge-layer bot/spoof/scrape defence (per ADR-0297) stays at edge;
  detection substrate consumes its signals but does not replace it
- Cedar policy gating (per ADR-0243) stays as a separate substrate;
  detection signals feed Cedar decisions
- Audit-chain Merkle-seal verification (per ADR-0028) stays at
  audit-chain µservice; detection consumes failure-events but does
  not replace verification

### §F.5. Rollback path

Per ADR-0294 anomaly-rollback applied to detection substrate:

1. Per-rule rollback: rule transitions Active → Sunset via
   `detection-rule-sunset.md` runbook; sunset retention ≥90d
2. Per-model rollback: model transitions champion → previous
   champion via `detection-model-rollback.md` runbook; per ADR-0308
3. Per-substrate-µservice rollback: Flink savepoint + Spark
   checkpoint allow restart from last-known-good state; per
   incident-response.md procedure
4. Emergency kill-switch: per ADR-0295 kill-switch invariant,
   the detection substrate has an emergency-bypass mode (`DETECTION_BYPASS=1`
   env flag) that disables signal-emission while preserving audit
   logging. Used only on confirmed substrate failure mode.

## §G. References

### §G.1. Hyperscaler precedents

- **Stripe Radar** — stripe.com/docs/radar; Stripe Annual Letter 2024; Stripe Sessions 2024 keynote "Radar at scale"
- **Adyen RevenueProtect** — adyen.com/blog/risk; Adyen Annual Report 2024 (€1T+ processed)
- **Toss riskOps** — Toss 2024 Tech Conference keynote (Korean fintech)
- **AWS GuardDuty** — aws.amazon.com/guardduty
- **AWS Macie** — aws.amazon.com/macie
- **Amazon Detective** — aws.amazon.com/detective
- **Google Chronicle (Google Security Operations)** — cloud.google.com/chronicle-security-operations
- **NCMEC PhotoDNA** — missingkids.org/photodna; Microsoft Research 2009 PhotoDNA paper
- **GIFCT Hash Sharing Database** — gifct.org (5M hashes 2024)
- **StopNCII.org** — stopncii.org (500k+ hashes 2024)
- **Apple NeuralHash** — apple.com/child-safety (deprecated Aug 2022 but precedent for on-device CSAM detection)
- **Google CSAI Match** — protectingchildren.google
- **Exabeam UEBA** — exabeam.com
- **Securonix SIEM** — securonix.com
- **Microsoft Sentinel UEBA** — microsoft.com/en-us/security/business/siem-and-xdr/microsoft-sentinel
- **Splunk UBA** — splunk.com/en_us/products/user-behavior-analytics
- **LightGBM** — github.com/microsoft/LightGBM (Microsoft Research 2017+)
- **XGBoost** — github.com/dmlc/xgboost
- **Apache Flink 1.20 LTS** — flink.apache.org (released 2024-08)
- **Apache Spark 3.5** — spark.apache.org
- **Polars** — pola.rs (Rust-native columnar)
- **ClickHouse** — clickhouse.com
- **Trino** — trino.io (Presto fork; Stripe/Airbnb/Pinterest at scale)
- **Feast** — feast.dev (open-source feature store)
- **Tecton** — tecton.ai (commercial managed feature store)
- **Vertex AI Feature Store** — cloud.google.com/vertex-ai/docs/featurestore
- **Apache AGE** — age.apache.org (Postgres graph extension)
- **Neo4j** — neo4j.com
- **Apache Arrow** — arrow.apache.org
- **Sigma Rules** — github.com/SigmaHQ/sigma (declarative detection DSL)
- **Hive Moderation** — thehive.ai (content-moderation)
- **ActiveFence** — activefence.com (trust-and-safety)
- **Spectrum Labs** — spectrumlabs.ai (content-moderation)

### §G.2. Standards + RFCs

- **NIST AI Risk Management Framework 1.0** — nist.gov/itl/ai-risk-management-framework
- **ISO/IEC 42001:2023** — AI management systems (released Dec 2023)
- **NIST iBeta Level 2** — biometric-presentation-attack-detection conformance
- **Apache Iceberg spec** — iceberg.apache.org/spec
- **Apache Arrow Columnar Format spec** — arrow.apache.org/docs/format
- **Open Telemetry semantic conventions** — opentelemetry.io/docs/specs/semconv
- **Sigma Rule Format** — github.com/SigmaHQ/sigma/wiki/Specification

### §G.3. Legal + compliance

- **EU AI Act (Regulation 2024/1689)** — Article 13 (transparency), Article 14 (human oversight), Article 18 (record-keeping), Article 27 (fundamental-rights impact assessment), Article 73 (serious-incident reporting), Article 86 (right-to-meaningful-explanation)
- **GDPR (Regulation 2016/679)** — Article 13/14 (right to be informed), Article 17 (right to erasure), Article 21 (right to object), Article 22 (automated-decision-making rights), Article 35 (DPIA)
- **NY AEDT Local Law 144 (2023)** — NYC bias-audit-and-public-notice for automated employment decision tools
- **ECOA + Regulation B** — 12 CFR §1002.9 adverse-action notice with specific reasons
- **Fair Housing Act + HUD's disparate-impact rule** — 24 CFR §100.500
- **Federal Uniform Guidelines on Employee Selection Procedures** — 29 CFR §1607.4 (4/5ths rule)
- **18 USC §2258A** — NCMEC reporting obligations
- **FinCEN BSA + SAR thresholds** — 31 CFR §1010, §1020-1029
- **OFAC sanctions** — Treasury OFAC SDN List
- **EU sanctions (Council Decision 2014/145, ongoing)**
- **UN sanctions (UN Security Council Consolidated List)**
- **KR-MOFA sanctions list**
- **JP-METI End User List**
- **NIS2 (EU Directive 2022/2555)** — incident-reporting 24h/72h/1mo cadence
- **NY DFS Cybersecurity Regulation 23-NYCRR-500** — 72h breach notification
- **KR-PIPA (Personal Information Protection Act)** — 72h breach notification
- **HIPAA** — 60d Privacy Rule breach notification
- **CCPA + CPRA** — California Consumer Privacy Act + Privacy Rights Act
- **DSA (Regulation 2022/2065)** — Article 16 notice-and-action; Article 17 statement-of-reasons; Article 22 trusted-flagger; Article 27 transparency
- **EU CSAM Regulation (proposal 2022/0155)** — pending; mandates CSAM detection
- **FTC Rule on Use of Consumer Reviews and Testimonials (16 CFR Part 465, 2024)** — civil penalty $51,744 per violation
- **KR Financial Consumer Protection Act Art. 30** — protected-class restrictions in financial-decision ML

### §G.4. Internal portfolio ADRs

- **ADR-0028** — audit-chain Merkle-sealed (the audit-stream the substrate consumes)
- **ADR-0099** — data-class registry (feature data-class tagging)
- **ADR-0105** — 13-layer canonical enum (layer 5 shared-substrate)
- **ADR-0130** — agentic SLO-gated promotion (per-µservice SLO)
- **ADR-0131** — per-microservice flat layout
- **ADR-0132** — no-grouping microservice rule (single-concern)
- **ADR-0140** — Cedar policy enforcement
- **ADR-0145** — inter-microservice communication reform (direct gRPC)
- **ADR-0212** — buildability doctrine (CI fitness lanes)
- **ADR-0240** — sovereign-cloud per-regional pack overlay
- **ADR-0242** — oyatie-is-a-tenant doctrine
- **ADR-0243** — Cedar as universal gate
- **ADR-0244** — tenant as universal scoping primitive
- **ADR-0245** — substrate vs product layering
- **ADR-0246** — policy-engine substrate promotion (library-first)
- **ADR-0248** — Amazon-shape cellular architecture
- **ADR-0250** — build-ahead-of-certification doctrine
- **ADR-0251** — compliance-pack cell certification levels
- **ADR-0252** — HLC default; TrueTime tier
- **ADR-0253** — HTTP/3 + QUIC + ECH + PQC network topology
- **ADR-0254** — deployment-model spectrum (K8s + Cloud Hypervisor + Kata)
- **ADR-0255** — intelligence two-layer substrate (AI Substrate + Consumer Brand Surface)
- **ADR-0258** — API versioning SemVer policy
- **ADR-0263** — observability emission contract
- **ADR-0276** — backup portability GDPR Art. 20
- **ADR-0280** — substrate-of-substrate dependency DAG
- **ADR-0293** — Foundry meta-trust-root
- **ADR-0294** — Cedar fragment soak + anomaly-rollback
- **ADR-0295** — bootstrap CI SPIFFE + kill-switch
- **ADR-0296** — library-first credential sidecar
- **ADR-0297** — abuse-defence baseline (anti-bot + anti-spoof + anti-scrape)
- **ADR-0298** — emergency-services critical-path exemption
- **ADR-0308** — ML model lifecycle (this bundle)
- **ADR-0309** — detection fairness audit + civil-rights compliance (this bundle)
- **ADR-0310** — investigation case-management (this bundle)

### §G.5. Standards docs

- `docs/standards/documentation-rigor.md` §3.2.6 — DRMP baseline
- `docs/standards/doc-style.md` — Diátaxis + RFC-2119
- `docs/STANDARDS-AND-TEMPLATES.md` — catalog
- `docs/templates/adr-template-v2.md` — ADR template
- `docs/standards/event-schema-versioning-canonical.md`
- `docs/standards/fintech-compliance.md` (KR-FSS / PCI-DSS / ECOA / etc)

### §G.6. Auto-memory feedback (related)

- `feedback_quality_performance_scalability_bar` — hyperscaler-grade rigor floor
- `feedback_clean_architecture_requirements` — inward-only + single-concern
- `feedback_no_silent_regression` — public-contract protection
- `feedback_autonomous_implementation_artifacts` — intern-buildable substrate
- `feedback_oyatie_is_a_tenant_doctrine` — substrate applies to oyatie's own surfaces
- `feedback_cedar_as_universal_gate` — Cedar evaluates detection-augmented context
- `feedback_amazon_shape_cellular_architecture` — Tier-2 hosts detection
- `feedback_substrate_vs_product_layering` — detection is substrate
- `feedback_compliance_pack_primitive` — per-pack feature/rule/model overlay
- `feedback_naming_justification` — every primitive justified per v4 BNF + 12-layer-enum

## §H. Change log

- **2026-05-20** — Initial draft authored as part of keystone-bundle 2026-05-20 Wave-3-D detection-cluster batch (ADR-0307..0310). Bundled with ADR-0308 (ML lifecycle), ADR-0309 (fairness audit), ADR-0310 (investigation case-management) as the **drmp-detection-cluster** keystone batch. Cross-references documentation-rigor.md §3.2.6.A eight families + §3.2.6 substrate primitives. Enforcement advisory-until-2026-09-15-blocker-thereafter.
