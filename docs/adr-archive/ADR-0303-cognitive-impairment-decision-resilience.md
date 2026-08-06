---
id: ADR-0303
status: Superseded
date: 2026-05-20
owners:
  - council-architecture
  - council-security
  - council-privacy
  - council-product
  - council-legal
  - council-accessibility
  - ops-sre-reliability
  - ops-trust-and-safety
  - ops-compliance
  - axis-identity
  - axis-policy-engine
  - axis-audit-chain
  - axis-consent
  - axis-tenancy
  - axis-notifications
supersedes: []
amends: []
superseded_by: [ADR-700]
related:
  - ADR-0028-audit-chain-merkle-sealed.md
  - ADR-0099-data-class-registry.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0140-cedar-policy-enforcement.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0188-passkey-webauthn-as-canonical-auth.md
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
  - ADR-0263-observability-emission-contract.md
  - ADR-0272-cookie-consent-per-purpose.md
  - ADR-0276-backup-portability-gdpr-art-20.md
  - ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md
  - ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md
  - ADR-0298-emergency-services-bypass-doctrine.md
  - ADR-0299-account-recovery-resilience.md
  - ADR-0300-whistleblower-press-freedom-anonymity.md
  - ADR-0301-survivor-safety-domestic-abuse-mode.md
  - ADR-0302-deceased-user-inheritance-doctrine.md
  - ADR-0304-cross-jurisdiction-conflict-resolution.md
  - ADR-0305-delegated-agent-authority-chain.md
  - ADR-0306-disaster-mode-cell-resilience.md
related_specs:
  - /specs/platform-architecture.json
  - /specs/microservices/identity.json
  - /specs/microservices/payments.json
  - /specs/microservices/policy-engine.json
  - /specs/microservices/audit-chain.json
  - /specs/microservices/notifications.json
  - /specs/microservices/tenancy.json
  - /specs/cognitive-impairment-controls.json
  - /specs/decision-resilience-schema.json
  - /specs/compliance-pack-schema.json
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
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-foundational-doctrine
keystone_position: critical-path-cluster-cognitive-impairment-decision-resilience
purpose: >
  Establish the Cognitive-Impairment and Decision-Resilience doctrine
  — a substrate-level primitive that introduces cooling-off periods on
  large transfers, trusted-contact alerts (per FINRA Rule 4512),
  cool-down on rapid sequential mutations, per-jurisdiction
  guardianship-law overlays, and informational-only nudges that
  respect autonomy and never block. The bar is: a user in a
  compromised cognitive state (elder financial-abuse target,
  intoxication, post-trauma, dementia, medication-impaired,
  high-stress acute-grief) is protected from irreversible
  consequential decisions by friction that respects autonomy. Per
  documentation-rigor.md §3.2.5 rows 4 + 20.
enforcement_status: advisory-until-2026-09-15-blocker-thereafter
enforced_by:
  - cloud-ci/Rust gate packet cognitive-impairment-cooling-off-coverage
  - cloud-ci/Rust gate packet cognitive-impairment-trusted-contact-binding
  - cloud-ci/Rust gate packet cognitive-impairment-mutation-cool-down
  - cloud-ci/Rust gate packet cognitive-impairment-guardianship-overlay
  - cloud-ci/Rust gate packet cognitive-impairment-nudge-non-blocking
  - cloud-ci/Rust gate packet cognitive-impairment-audit-emission
naming_justifications:
  - name: oya-shared-decision-resilience
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.decision-resilience
    justification: >
      Per ADR-0105 13-layer canonical enum row 5 (shared-substrate),
      the crate that exposes the cooling-off-timer trait + trusted-
      contact-notifier trait + mutation-cadence-tracker trait + per-
      jurisdiction guardianship-overlay registry belongs at the shared
      layer. Naming `oya-shared-decision-resilience` keeps the
      single-concern flat layout per ADR-0131 and avoids any "suite"
      packaging per ADR-0132.
  - name: oya-governance-cooling-off-coverage
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.cooling-off-coverage
    justification: >
      CI fitness lane per ADR-0212 buildability doctrine; verifies
      every µservice with consequential-mutation surface declares a
      cooling-off configuration in `policy/decision-resilience.cedar`
      and `iac/<env>-cooling-off.yaml`. Lane naming follows the
      canonical `oya-governance-<concern>` shape consistent with
      ADR-0297 sibling lanes.
  - name: oya-governance-trusted-contact-binding
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.trusted-contact-binding
    justification: >
      CI fitness lane per ADR-0212; verifies tenants with the
      `audience_type = SENIOR_PROTECTED` or
      `audience_type = HIGH_VALUE_FINANCIAL` opt-in have a binding
      trusted-contact attestation chain per FINRA Rule 4512 §C.
  - name: oya-governance-mutation-cool-down
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.mutation-cool-down
    justification: >
      CI fitness lane per ADR-0212; verifies rapid-sequential-mutation
      windows are declared per µservice + per data class per
      ADR-0099, and that the per-pack cool-down floor (e.g., HIPAA
      delete cool-down) is observed.
  - name: oya-governance-guardianship-overlay
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.guardianship-overlay
    justification: >
      CI fitness lane per ADR-0212; verifies per-jurisdiction
      guardianship-law packs are wired (US-adult-guardianship,
      EU-MentalCapacityAct, KR-Adult-Guardianship-Civil-Act-§9,
      UK-Mental-Capacity-Act-2005).
  - name: oya-governance-decision-resilience
    layer: N/A (foundry-fitness aggregate CI lane)
    bnf_segments: oya.foundry-fitness.decision-resilience
    justification: >
      Aggregate fitness lane per ADR-0212 that rolls up the four
      child lanes into a single advisory/BLOCKER gate per the
      keystone-bundle 2026-05-20 promotion-gate model.
  - name: X-Oya-Cooling-Off-Until
    layer: N/A (HTTP response header naming)
    bnf_segments: X-Oya.Cooling-Off-Until
    justification: >
      Custom HTTP response header returned on
      `409 Conflict — CoolingOffActive` carrying the RFC 3339 timestamp
      at which the mutation may be retried. Namespace prefix `X-Oya-`
      keeps platform headers in their reserved namespace; avoids
      collision with retry-after semantics (Retry-After is volumetric;
      Cooling-Off-Until is decision-resilience).
  - name: X-Oya-Trusted-Contact-Notified
    layer: N/A (HTTP response header naming)
    bnf_segments: X-Oya.Trusted-Contact-Notified
    justification: >
      Custom HTTP response header carrying the redacted
      trusted-contact-id (per-tenant pseudonymous) and the
      notification dispatch status; returned on consequential mutations
      that triggered FINRA Rule 4512 §C notification.
  - name: CoolingOffTriggered
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: DecisionResilience.CoolingOffTriggered
    justification: >
      Audit-event-class emitted whenever a cooling-off window opens on
      a consequential mutation. Registered in ADR-0263 central
      registry to satisfy §3.2.2 consistency invariant.
  - name: TrustedContactAlertSent
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: DecisionResilience.TrustedContactAlertSent
    justification: >
      Audit-event-class emitted whenever a trusted-contact notification
      is dispatched per FINRA Rule 4512 §C or per-jurisdiction
      equivalent. Registered per ADR-0263.
  - name: RapidMutationCoolDownTriggered
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: DecisionResilience.RapidMutationCoolDownTriggered
    justification: >
      Audit-event-class emitted when the rapid-sequential-mutation
      detector enforces a cool-down. Registered per ADR-0263.
  - name: GuardianshipOverlayApplied
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: DecisionResilience.GuardianshipOverlayApplied
    justification: >
      Audit-event-class emitted when a per-jurisdiction guardianship
      overlay is consulted on a consequential mutation. Registered
      per ADR-0263.
  - name: policy/decision-resilience.cedar
    layer: N/A (per-µservice Cedar fragment file)
    bnf_segments: policy.decision-resilience
    justification: >
      Canonical filename for the per-µservice decision-resilience
      Cedar fragment under the µservice's `policy/` directory per
      ADR-0246 + ADR-0243 fragment-lifecycle conventions; single-
      concern naming keeps the policy directory's contract-by-name
      invariant.
  - name: iac/<env>-cooling-off.yaml
    layer: N/A (per-µservice IaC manifest)
    bnf_segments: iac.<env>.cooling-off
    justification: >
      Canonical filename for per-µservice + per-env cooling-off IaC
      manifest; declares per-data-class cooling-off windows in the
      IaC layer paired with the Cedar fragment defence-in-depth.
  - name: SENIOR_PROTECTED
    layer: N/A (Tenant.audience_type enum value per ADR-0244)
    bnf_segments: audience_type.SENIOR_PROTECTED
    justification: >
      Tenant.audience_type enum extension per ADR-0244 §D-3; identifies
      tenants serving users ≥65 (per FINRA Rule 2165 specified-adult
      definition) where elder-financial-abuse defences receive
      elevated sensitivity and trusted-contact opt-in becomes a
      default surface.
  - name: HIGH_VALUE_FINANCIAL
    layer: N/A (Tenant.audience_type enum value per ADR-0244)
    bnf_segments: audience_type.HIGH_VALUE_FINANCIAL
    justification: >
      Tenant.audience_type enum extension per ADR-0244 §D-3; identifies
      tenants conducting high-value financial mutations (≥USD 10,000
      transfer single-shot; ≥USD 25,000 daily aggregate per FATF
      Recommendation 10 threshold) where cooling-off + trusted-contact
      defaults activate.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0303: Cognitive-Impairment and Decision-Resilience Doctrine

## Status

Proposed — 2026-05-20.

Bundled with the keystone-bundle 2026-05-20 foundational doctrine
synthesis as the **critical-path-cluster-cognitive-impairment-decision-
resilience** keystone, closing the gap identified in
`docs/standards/documentation-rigor.md` §3.2.5 rows 4 + 20 of the
critical-path edge-case coverage matrix. The standard already
codifies the row-level handling requirements (cooling-off period on
large transfers; trusted-contact alerts per FINRA Rule 4512; cool-down
on rapid sequential mutations; per-jurisdiction guardianship-law
overlay; respects autonomy; never blocks; informational nudges only);
this ADR is the binding ADR the standard's rows 4 + 20 cite.

Enforcement is `advisory-until-2026-09-15-blocker-thereafter`. The
doctrine accepts in text immediately; the CI lanes that enforce it
promote to BLOCKER on 2026-09-15 to give per-µservice rollout
sequenced by impact-severity (per §F migration) time to land.
Until 2026-09-15, validators emit findings without failing CI; post-
2026-09-15, the lanes block merge.

## Date

2026-05-20.

## Context

### §A. Why decision-resilience is a substrate primitive, not a µservice afterthought

Modern hyperscaler-class fintech, healthcare, and consumer platforms
treat cognitive-state-aware decision resilience as a *first-class
substrate primitive* — wired into the consequential-mutation path,
in every µservice that issues an irreversible side-effect, and
composing with the Cedar gate that authorizes the mutation. The
pattern is unambiguous across the named industry references:

- **Stripe** ships radar-rules, idempotency keys, and refund-window
  primitives as Tier-0 control plane offerings. Per the Stripe API
  documentation (2024-2025 "Radar" + "Adaptive Acceptance" series),
  Stripe's risk model evaluates ~250 signals per transaction including
  behavioural deviation; transactions that score above the platform's
  cognitive-deviation threshold trigger a cool-down with merchant +
  cardholder notification. The primitive is operated as a substrate
  with operational SLAs, not per-merchant code.
- **Apple Cash + Apple Wallet** ships "Tap to Cash" with a built-in
  72-hour reversal-window for transactions over the per-account
  protected threshold (per the Apple Cash Terms 2024 and the Family
  Sharing financial-abuse-defence patterns), trusted-contact
  notification for the "Family Protected User" cohort, and a default
  cooling-off on tier-elevated transactions. Apple's iCloud Family
  Sharing for senior protected users is the canonical pattern that
  ADR-0303's `audience_type = SENIOR_PROTECTED` codifies.
- **Charles Schwab + Fidelity + Vanguard** all implement FINRA Rule
  4512 trusted-contact + Rule 2165 specified-adult temporary holds as
  the substrate primitive for the U.S. retail brokerage industry. Per
  the FINRA 2024 Report on Senior Investor Protection, member firms
  reported ~5,300 Rule 2165 holds in 2023 — preventing ~USD 220M in
  documented elder financial abuse. The primitive is operated as
  substrate by every Tier-1 retail broker.
- **Chase + Bank of America + Wells Fargo** ship cooling-off windows
  on first-time wire transfers (72 hours typical), step-up
  authentication on high-value transfers, and trusted-contact
  programs (Bank of America "Trusted Contact Program" 2022; Chase
  "Family Protected Beneficiary" 2023; Wells Fargo "Elder Client
  Initiative" 2020). Per the American Bankers Association 2024
  Survey on Senior Banking, ~78% of U.S. banks now operate
  cooling-off windows as a substrate primitive — not as per-product
  feature.
- **Microsoft Family Safety + Google Family Link** both ship
  cognitive-state-aware purchase confirmations: when a child or
  protected family member attempts an irreversible mutation
  (in-app purchase, app installation, content unlock), the
  primary-account parent receives a notification with a cooling-off
  window before the mutation commits. Per the Google Family Link
  Terms 2024 + Microsoft Family Safety documentation, the pattern is
  substrate across the consumer device ecosystem.
- **Korean Adult Guardianship Civil Act §9** (성년후견인 제도) +
  **Japan Adult Guardianship Law (成年後見制度)** + **EU Mental
  Capacity Acts** (EN-MCA 2005, IE-Assisted-Decision-Making 2015,
  DE-BetreuungsRecht §1814 BGB) + **U.S. UPC §5-303** all impose
  jurisdiction-specific guardianship overlays on consequential
  decisions where the user has been adjudicated to lack capacity. A
  legitimate substrate platform must honor the per-jurisdiction
  overlay; failing to do so creates legal exposure under the
  applicable jurisdictional regime and harms the user the law is
  designed to protect.

The corollary: **every internet-facing surface oyatie ships MUST
inherit decision-resilience from the substrate, not author it per-
µservice.** A µservice that authors its own cooling-off-timer logic,
its own trusted-contact notification, its own rapid-mutation cool-down,
its own guardianship overlay is duplicating substrate primitives that
the shared `oya-shared-decision-resilience` crate already serves. That
duplication is a `feedback_no_silent_regression` violation (every
µservice's friction drifts independently); it is a
`feedback_quality_performance_scalability_bar` violation (the substrate
sees signal across every µservice's mutation cadence that a single
µservice cannot); and it is a `feedback_autonomous_implementation_
artifacts` violation (intern-buildable means one substrate, not 46
µservice-private implementations).

The ADR-0303 decision-resilience doctrine closes this gap.

### §A.1. The cognitive-impairment landscape 2026 — the user the substrate protects

The 2026 cognitive-impairment landscape spans far more than the
historical narrow "elder financial abuse" cohort:

- **Elder financial abuse.** Per the AARP 2024 BankSafe Report,
  ~USD 28.3 billion in U.S. annual losses from elder financial
  exploitation; ~88% are perpetrated by someone the victim knows;
  the median case loss is ~USD 39,200. The cognitive-state at the
  point of mutation is impaired by trust-grooming, social isolation,
  mild cognitive decline (MCI), early-stage dementia, or active
  pharmacological side-effects. The defence: cooling-off +
  trusted-contact + per-jurisdiction guardianship overlay.
- **Intoxication-state decision-making.** Per the U.S. National
  Institute on Alcohol Abuse and Alcoholism (NIAAA 2023), ~15.1
  million U.S. adults have alcohol-use disorder; per the SAMHSA
  2023 National Survey on Drug Use, ~46.3 million U.S. adults
  reported past-month binge drinking. Decisions made in this state
  on financial / social / professional mutations carry
  cognitive-deviation signal. The defence: rapid-sequential-mutation
  cool-down on behavioural-deviation signal (typing-cadence anomaly,
  session-time anomaly, mutation-velocity anomaly).
- **Post-trauma / acute-grief decision-making.** Per the U.S.
  Bureau of Labor Statistics 2024, ~2.8 million U.S. deaths per
  year produce ~14 million bereaved-survivor accounts. Per APA
  2024 grief research, the 6-month-post-loss period shows elevated
  impulsive-decision frequency; ~22% of bereaved survivors report
  later regret on a major financial mutation made in the first 60
  days. The defence: optional self-imposed cooling-off (per-tenant
  opt-in via the `BEREAVEMENT_PROTECTED` audience-type).
- **Medication-impaired cognitive state.** Per the FDA 2023 Drug
  Safety Communications, ~80 commonly prescribed medications
  including sedative-hypnotics, anti-anxiety, anti-depressant, and
  pain-management drugs carry cognitive-impairment warnings. The
  per-tenant accessibility opt-in for `COGNITIVE_PROTECTED` users
  surfaces the cooling-off + trusted-contact defaults regardless of
  age or audience type.
- **Dementia and MCI.** Per the Alzheimer's Association 2024 Facts &
  Figures, ~6.9 million U.S. adults aged ≥65 live with Alzheimer's
  dementia; the prevalence rises to ~33% above age 85. Mild Cognitive
  Impairment (MCI) affects ~12-18% of adults ≥60. The substrate's
  per-jurisdiction guardianship overlay activates when a court-adjudicated
  guardian is in place.
- **High-stress acute decision pressure.** Per FINRA's 2024 Investor
  Behaviour Report, retail investors reported 4× higher trading
  velocity during the March 2020 COVID panic; ~30% reported later
  regret on at least one trade. The substrate's rapid-mutation
  cool-down adds friction (informational, never blocking) when the
  user's mutation cadence deviates ≥3σ from their 30-day baseline.

The substrate baseline MUST be sized to this 2026 landscape — not
the 2002 landscape that earlier elder-financial-abuse protections
were designed against. The bar is not "warn the user once at sign-up";
the bar is "operate adaptive friction across continuously-evolving
cognitive-state-deviation signal, in informational and never-blocking
form, that respects autonomy across every consequential-mutation
surface."

### §A.2. Why the keystone bundle 2026-05-20 requires this as a substrate primitive

The keystone bundle's foundational ADRs intersect decision-resilience
as follows:

- **ADR-0242 (oyatie-is-a-tenant).** The platform's own surfaces are
  subject to the same decision-resilience defences as any tenant
  surface. A platform admin attempting to mutate a critical
  configuration during atypical hours receives the same cool-down +
  informational nudge as any other principal. No carve-outs.
- **ADR-0243 (Cedar universal gate).** Every decision-resilience
  decision is composable as a Cedar fragment. The cooling-off-active
  predicate, the trusted-contact-attestation predicate, the
  rapid-mutation cool-down predicate, the guardianship-overlay
  predicate all enter Cedar evaluation as principal/resource/action
  context attributes. No bypass paths.
- **ADR-0244 (tenant scoping primitive).** Trusted-contact identity
  is scoped to the tenant; cross-tenant trusted-contact attestation
  is blocked. The audience-type enum extends with
  `SENIOR_PROTECTED`, `HIGH_VALUE_FINANCIAL`, `BEREAVEMENT_PROTECTED`,
  `COGNITIVE_PROTECTED`.
- **ADR-0248 (Amazon-shape cellular architecture).** Decision-
  resilience state (active cooling-off windows, trusted-contact
  attestation cache, rapid-mutation cadence history) is cell-local.
  Cross-cell state mirror happens only via the audit-chain (write-once,
  append-only) for audit integrity.
- **ADR-0251 (compliance packs).** Different jurisdiction packs
  (`pack-us-finra`, `pack-eu-mca`, `pack-kr-adult-guardianship`,
  `pack-jp-adult-guardianship`, `pack-uk-mca`) extend the baseline
  with jurisdiction-specific guardianship overlays + cooling-off
  floor values. Higher-restriction pack wins per ADR-0304.
- **ADR-0263 (observability emission contract).** Every decision-
  resilience event emits an audit-event-class (per the registry);
  every cooling-off trigger, every trusted-contact notification,
  every rapid-mutation cool-down becomes a row in the audit chain.
- **ADR-0292 (minor user doctrine).** Decision-resilience defaults
  are elevated for minor-targeted surfaces; COPPA + KOSA + AADC
  packs compose with `audience_type = MINOR_PII` to require
  parental-trusted-contact for consequential mutations under
  age-of-majority.
- **ADR-0297 (abuse-defence baseline).** Decision-resilience is
  layered with bot-defence. A request that passes bot-defence
  (the request is human) but deviates from the user's cognitive-state
  baseline triggers decision-resilience (informational nudge +
  cooling-off + trusted-contact). The two systems compose.
- **ADR-0298 (emergency-services bypass).** Decision-resilience
  NEVER applies to emergency-services paths. A user dialing 911,
  initiating an EHR break-glass, or filing a domestic-violence
  shelter request never experiences cooling-off friction. The
  exemption is enumerated in §D-4 Cedar fragment.
- **ADR-0299 (account-recovery resilience).** Decision-resilience
  composes with account-recovery to provide post-recovery cooling-off
  (72h on high-value mutations following a recovery event) per
  the row 24 hijack-recovery handling.

The bundle cannot land without the decision-resilience baseline
articulated explicitly. The promotion gate for the 2026-05-20 bundle
is: *the substrate MUST protect users in compromised cognitive states
from irreversible consequential decisions, while respecting autonomy
and never blocking.* This ADR is the binding articulation.

### §A.3. What this ADR explicitly does NOT do

- This ADR does not specify the per-µservice cool-down threshold per
  data class — each µservice's `iac/<env>-cooling-off.yaml` declares
  its concrete values layered atop the substrate baseline (the
  substrate provides defaults; the µservice tunes).
- This ADR does not specify the per-tenant trusted-contact UI; the
  tenant control surface for trusted-contact registration is the
  responsibility of the `microservices/tenancy/` PRD per ADR-0244 +
  ADR-0218.
- This ADR does not redefine Cedar fragment authoring conventions —
  that is ADR-0243 + ADR-0294's scope. This ADR declares the *content*
  of `policy/decision-resilience.cedar` but the *lifecycle* is
  ADR-0294 (≥60s soak + signed publication + rollback).
- This ADR does not specify the audit-event-class registry shape —
  that is ADR-0263's scope. This ADR adds four event classes to the
  registry.
- This ADR does not displace per-pack regulator floors (e.g., FINRA's
  hold duration ≤ 25 business days for Rule 2165); the substrate
  honors per-pack floors and never relaxes them. ADR-0304 resolves
  multi-pack conflicts.
- This ADR does not adjudicate capacity. Capacity adjudication is a
  jurisdictional-court matter; the substrate consumes per-tenant
  guardianship attestations + per-jurisdiction overlay, not the
  capacity decision itself.

## Decision

### §B. Four orthogonal primitives composed at three layers

The decision-resilience baseline is **four orthogonal primitives**
(cooling-off, trusted-contact, mutation-cadence, guardianship-overlay)
composed at **three layers** (Tier-0 substrate shared crate,
per-µservice gate, Cedar policy fragment). The 4×3 matrix produces
twelve cells; each cell has a defined primitive. The matrix is
defence-in-depth: **no single cell gates a request alone**; the
twelve cells compose via Cedar fragment evaluation and informational-
nudge layering.

```
                       Tier-0 shared           Per-µservice            Cedar policy
                       -------------           -------------           -------------
Cooling-off            Timer registry +        Per-route cool-down +   forbid when
                       window-vault             scheduled-commit        cooling_off_until
                                                                        > now()
Trusted-contact        Attestation cache +     Per-mutation alert +    forbid (when
                       FINRA Rule 4512          notification fan-out    audience_type
                                                                        ∈ SENIOR_PROTECTED
                                                                        ∧ amount ≥ threshold
                                                                        ∧ ¬contact_acked)
Mutation-cadence       3σ deviation detector + Per-route velocity      forbid when
                       30-day rolling baseline   cap + cool-down       mutation_cadence_z
                                                                        > 3
Guardianship-overlay   Per-jurisdiction        Per-mutation overlay    forbid when
                       overlay registry +       check + co-sign         guardian_required
                       legal-cite cache         workflow                ∧ ¬guardian_sig
```

The four primitives are **orthogonal** — they defend against
different cognitive-state-deviation modes:

- **Cooling-off** addresses irreversibility risk. A user who clicks
  "send $50,000" should have a window to reverse the decision before
  it commits. The default windows are: 60 minutes on first-time
  large-amount mutations; 72 hours on tier-elevated mutations
  (≥USD 10,000 per FATF Recommendation 10); 7 days on
  guardianship-protected accounts.
- **Trusted-contact** addresses social-isolation risk. An elder
  client cannot be reached by a scammer without their FINRA Rule
  4512 trusted-contact also being notified. The trusted-contact is
  not a co-signer (autonomy preserved); they are an informed third
  party who can intervene if the mutation is fraudulent.
- **Mutation-cadence** addresses behavioural-deviation risk. A user
  whose baseline mutation cadence is 2 transfers per month suddenly
  initiating 15 transfers in 30 minutes is exhibiting a deviation
  signal. The cool-down is informational (the user is informed +
  asked to wait); it never blocks.
- **Guardianship-overlay** addresses adjudicated-capacity risk. A
  user with a court-adjudicated guardian per the applicable
  jurisdiction has the guardian's co-sign requirement enforced per
  the per-jurisdiction overlay.

A single user may benefit from all four primitives composing (e.g.,
a 78-year-old elder client with an adjudicated co-guardian, a FINRA
Rule 4512 trusted contact, an active 72h cooling-off on a $25k
transfer initiated in atypical late-night hours).

The three layers are **complementary** — they catch different
adversary tactics:

- **Tier-0 shared crate** centralizes the timer registry, the
  attestation cache, the mutation-cadence detector, and the per-
  jurisdiction overlay registry. The per-µservice path imports the
  crate; the substrate operates the state.
- **Per-µservice gate** sees the µservice-local context (route,
  data class, resource value, user tier). The µservice contributes
  context that the substrate cannot see alone.
- **Cedar policy fragment** composes the substrate + µservice +
  per-tenant + per-pack signals into a single permit/forbid decision.
  Cedar's evaluation is deterministic + signed + audited per
  ADR-0243 + ADR-0263.

### §B.1. The autonomy invariant — informational nudges only, never blocks

The decision-resilience baseline categorically rejects paternalistic
blocking. The autonomy invariant:

> A user's autonomy is the final authority on their own consequential
> decisions. The substrate's role is to provide friction, information,
> and time — never to deny.

In practice, this means:

- **Cooling-off windows are user-overridable** within the regulator
  floor. A user may elect to wait the default 60 minutes or override
  to "commit now" via an explicit acknowledgment workflow. The
  override is audited (`CoolingOffOverridden` event class).
  Exception: per-pack regulator floors (FINRA Rule 2165 ≤ 25
  business days hold) cannot be overridden by the user — these are
  regulator-mandated holds.
- **Trusted-contact alerts are informational.** The trusted contact
  is notified; they are not asked to consent. The user proceeds
  regardless of whether the contact responds. Exception: per the
  per-jurisdiction guardianship overlay where the guardian's signature
  IS legally required (e.g., adjudicated incapacity).
- **Rapid-mutation cool-down is a delay, not a refusal.** A user
  exceeding 3σ mutation cadence is informed of the deviation +
  shown an explicit "wait 5 minutes" prompt + given the option to
  proceed anyway with elevated audit. The cool-down does NOT cap
  total mutations per session — only the velocity.
- **Guardianship overlay requires explicit court adjudication.** The
  substrate does not adjudicate capacity; it consumes per-tenant
  attestation that a guardian is in place. Without explicit
  guardianship attestation, no overlay applies.

This principle is canonical across hyperscalers: per Apple's "iCloud
Family Sharing for Seniors" 2024 documentation, the protective surface
is informational + recommended-but-not-required; per Charles Schwab's
FINRA Rule 4512 implementation, the trusted contact "may NOT direct
trading or transfers" — the contact is informational; per Google
Family Link's purchase-confirmation pattern, the parent's notification
is informational unless explicit-control-mode is enabled.

The principle: **friction respects autonomy. Information replaces
gates. Cooling-off replaces refusal.**

### §B.2. The non-discrimination invariant — opt-in protections, never imposed

The protective surface for `SENIOR_PROTECTED`, `BEREAVEMENT_PROTECTED`,
`COGNITIVE_PROTECTED`, and `HIGH_VALUE_FINANCIAL` audience-types is
**opt-in**, not assigned by inference. The substrate categorically
rejects the failure mode of imposing senior-protected status by age
inference, intoxication-protected status by behavioural inference, or
dementia-protected status by accessibility-tool detection.

- **Age does not assign SENIOR_PROTECTED.** A user ≥65 receives the
  same default protective surface as any adult. They may opt-in to
  SENIOR_PROTECTED via the tenancy substrate's control surface.
  Inferring "this is a senior" from session signals (e.g., slow
  cursor) is forbidden — flagged by the `oya-governance-no-age-
  inference` lane.
- **Intoxication detection is not assignment.** The mutation-cadence
  detector signals "deviation from your baseline" — it does NOT
  assert "you are intoxicated." The framing matters; the user is
  informed of the deviation, not labeled.
- **Accessibility profile does not assign cognitive-protection.** A
  user with screen-reader profile receives the WCAG 2.2 AAA target
  on critical paths per ADR-0303 + ADR-0292 + the §3.2.5 row 12
  disability-accommodation handling — but cognitive-protection
  defaults activate only via explicit opt-in.

This principle ensures the decision-resilience baseline does NOT
become a covert age-discrimination surface, ableist-inference surface,
or paternalism vector.

## §C. Consequences

### §C.1. Maintainability dimension

The decision-resilience baseline is the **substrate** that ≥30
consequential-mutation-emitting µservices inherit (payments,
ontology, tenancy, identity, governance, cloud-iac, marketplace,
billing, intelligence, finops-portal, compliance, foundry, mail,
notes, social, comms-email, connect, workflow-studio, ops-dashboard-
control-center, and the long tail of internal-mutation surfaces). The
maintainability invariants:

- **Per-µservice declaration is configuration, not code.** Each
  µservice declares its decision-resilience posture in
  `ARCHITECTURE.md §decision-resilience` + `iac/<env>-cooling-off.yaml`
  + `policy/decision-resilience.cedar`. The actual primitive
  implementation lives in the shared crate
  `oya-shared-decision-resilience`.
- **Per-tenant tuning is configuration, not code.** Audience-type
  sensitivity (SENIOR_PROTECTED, BEREAVEMENT_PROTECTED,
  COGNITIVE_PROTECTED, HIGH_VALUE_FINANCIAL, MINOR_PII) is set via
  the tenancy substrate's control surface per ADR-0244. No code
  change required to retune.
- **Versioning policy.** The Cedar fragment `policy/decision-
  resilience.cedar` follows ADR-0294 Cedar fragment lifecycle (≥60s
  soak + signed publication + rollback). The IaC manifest follows
  the µservice's ADR-0258 SemVer policy. Per-jurisdiction overlays
  follow ADR-0251 compliance-pack version lifecycle.
- **Deprecation cadence.** Per-jurisdiction overlays update on the
  jurisdiction's legal cadence (typically every 5-7 years per
  major mental-capacity-law revision). The substrate exposes an
  overlay-registry API; updates are pull-request driven by the
  legal-council axis.
- **Single-concern crate.** The shared crate is single-concern per
  ADR-0131. It does not absorb related-but-distinct concerns
  (account-recovery is ADR-0299; survivor-safety is ADR-0301;
  inheritance is ADR-0302). The crate has one purpose: protect
  consequential decisions against compromised cognitive state.
- **Tests as inheritance proof.** Every µservice that emits a
  consequential mutation MUST ship contract tests demonstrating
  inheritance from `oya-shared-decision-resilience`. The
  `oya-governance-decision-resilience` lane verifies the
  per-µservice contract test exists and passes; missing tests
  trigger BLOCKER from 2026-09-15.
- **Documentation density.** Each µservice's PRD MUST cite which
  cooling-off windows apply, which audience-types receive
  trusted-contact defaults, which per-jurisdiction overlays are
  relevant. The PRD section is verified by the
  `oya-governance-doc-coverage` lane.

### §C.2. Observability dimension

Per ADR-0263 observability emission contract, the decision-resilience
baseline emits:

- **Audit-event-classes (registered in ADR-0263 registry):**
  - `CoolingOffTriggered` — emitted whenever a cooling-off window
    opens on a consequential mutation. Carries: principal_id,
    tenant_id, request_id, route, data_class, amount_redacted,
    window_seconds, audience_type, applicable_pack_ids[].
  - `CoolingOffOverridden` — emitted whenever a user explicitly
    overrides a cooling-off window. Carries: principal_id, tenant_id,
    request_id, original_window_seconds, override_acknowledgment_id.
  - `CoolingOffElapsed` — emitted whenever a cooling-off window
    elapses and the mutation commits. Carries the original
    `CoolingOffTriggered` event_id for correlation.
  - `TrustedContactAlertSent` — emitted whenever a trusted-contact
    notification is dispatched per FINRA Rule 4512 §C or per-
    jurisdiction equivalent. Carries: principal_id, tenant_id,
    contact_attestation_id (pseudonymous), notification_channel
    (email/SMS/push), per-pack legal-cite.
  - `TrustedContactAlertAcked` — emitted whenever the trusted contact
    acknowledges receipt. Optional event (the contact is not required
    to acknowledge).
  - `RapidMutationCoolDownTriggered` — emitted when the rapid-
    sequential-mutation detector enforces a cool-down. Carries:
    principal_id, tenant_id, mutation_velocity_z_score,
    baseline_window_days, cool_down_seconds.
  - `GuardianshipOverlayApplied` — emitted when a per-jurisdiction
    guardianship overlay is consulted on a consequential mutation.
    Carries: principal_id, tenant_id, jurisdiction_pack_id,
    guardian_attestation_id (pseudonymous), legal_cite.
  - `GuardianshipOverlayBypassed` — emitted only on emergency-services
    paths per ADR-0298 where the overlay would have applied but is
    bypassed by the life-safety exemption.
  - `DecisionResilienceUserAccommodation` — emitted whenever the
    cognitive-protected audience type receives the enhanced surface.
- **Metrics (per ADR-0263 cardinality budget):**
  - `oya_decision_resilience_cooling_off_active_gauge` — count of
    active cooling-off windows. Dimensions: tenant_id_bucket,
    µservice, audience_type, window_class_id.
  - `oya_decision_resilience_cooling_off_triggered_counter` — total
    cooling-offs triggered. Dimensions: same.
  - `oya_decision_resilience_cooling_off_overridden_counter` — total
    user-overrides. Dimensions: same.
  - `oya_decision_resilience_trusted_contact_notify_counter` —
    notifications dispatched. Dimensions: tenant_id_bucket,
    audience_type, notification_channel.
  - `oya_decision_resilience_mutation_cadence_z_histogram` —
    distribution of cadence z-scores. Dimensions: tenant_id_bucket,
    µservice, audience_type.
  - `oya_decision_resilience_guardianship_overlay_apply_counter` —
    overlays applied. Dimensions: jurisdiction_pack_id, audience_type.
- **Dashboards:** every consequential-mutation-emitting µservice
  MUST ship a `dashboards/decision-resilience.json` Grafana dashboard
  with the canonical 9-panel layout (cooling-off active gauge;
  trigger rate per route; user-override rate; trusted-contact
  fan-out latency p99; mutation-cadence z heatmap; guardianship
  overlay apply rate; per-pack-cite breakdown; emergency-services
  bypass count; audience-type distribution). Dashboard naming
  follows ADR-0263.

### §C.3. Scalability dimension

The decision-resilience baseline scales horizontally:

- **Cell-local state.** Active cooling-off windows, mutation-cadence
  baselines, and trusted-contact attestation caches are cell-local
  per ADR-0248. Cross-cell coordination happens only at audit-chain
  write (write-once, append-only) — not at decision time.
- **Per-tenant sharding.** Per-tenant decision-resilience state is
  sharded by tenant_id_bucket per the canonical shuffle-shard
  topology. The state-per-tenant is O(active_mutations + 30-day
  cadence baseline + trusted-contact attestation cache size); bounded.
- **Vendor primitive headroom.** The trusted-contact notification
  fan-out reuses ADR-0273 per-tenant DKIM/SPF/DMARC + ADR-0257 SMS
  + future push-notification substrate ADR. No new vendor surface;
  reuse of substrate primitives.
- **Hot-path performance.** The cooling-off check is a cell-local
  Cedar evaluation with O(1) lookup against the active-windows
  registry. Target p99 latency ≤ 5 ms per the SLO. The 30-day
  cadence baseline is precomputed offline (per-tenant batch); the
  hot-path consumes the precomputed z-score lookup.
- **Burst capacity.** During mass-casualty events (per ADR-0306
  disaster-mode), the substrate's mutation-cadence cool-down
  relaxes per the disaster-mode override; emergency-services
  surfaces never gate.
- **Multi-region coherence.** Per-pack data-residency floors (per
  ADR-0240 + ADR-0304) prevent cross-region mirror of trusted-contact
  state in residency-restricted packs. The cell-local design
  satisfies this constraint naturally.

### §C.4. Performance dimension

The decision-resilience hot-path SLOs:

- **Cooling-off check latency.** p50 ≤ 1 ms; p99 ≤ 5 ms; p99.9 ≤
  20 ms. Measured at the per-µservice gate; reported per ADR-0263.
- **Trusted-contact notification dispatch latency.** p50 ≤ 200 ms;
  p99 ≤ 2 s; p99.9 ≤ 10 s. Measured from `TrustedContactAlertSent`
  emission to ADR-0257 SMS / ADR-0273 email / a future push substrate ADR
  successful handoff to vendor.
- **Mutation-cadence z-score lookup latency.** p99 ≤ 5 ms; the
  baseline is precomputed offline per-tenant + per-µservice; the
  hot-path looks up the precomputed z-score.
- **Guardianship overlay apply latency.** p99 ≤ 10 ms; the per-
  jurisdiction overlay registry is in-memory cell-local; the
  lookup is O(1) by jurisdiction_pack_id.
- **CPU budget per consequential mutation.** ≤ 50 μs CPU including
  all four primitive evaluations. The substrate respects the
  90-μs total Cedar evaluation budget per ADR-0243.
- **Memory budget.** ≤ 500 bytes per active cooling-off window;
  ≤ 200 bytes per cell-local trusted-contact attestation entry;
  ≤ 5 KB per 30-day mutation cadence baseline. Per-cell aggregate
  ≤ 50 MB per million active tenants — bounded.
- **Garbage collection.** Cooling-off windows are TTL'd (default
  72 hours); cadence baselines are LRU'd (30-day window); trusted-
  contact attestation cache TTL'd at the per-pack legal-retention
  ceiling (FINRA Rule 4512 §C: ≥ 6 years post-account-closure).

### §C.5. Optimization dimension

The substrate provides optimizations that per-µservice implementations
would miss:

- **Cross-µservice cadence correlation.** A user making 3 mutations
  in payments + 5 in marketplace + 4 in finops-portal within 10
  minutes is exhibiting cadence deviation that no single µservice
  can see. The substrate aggregates across the user's mutations
  globally.
- **Per-tenant baseline drift detection.** The 30-day cadence
  baseline auto-detects drift (e.g., a user whose new normal is
  10× their previous baseline because they got a promotion). The
  substrate avoids false-positives by recomputing the baseline
  weekly.
- **Trusted-contact deduplication.** A single trusted-contact may
  cover multiple users in a tenant (e.g., a family member who is
  trusted-contact for both parents). The substrate deduplicates
  attestations to avoid notification fatigue.
- **Per-pack legal-cite caching.** The applicable legal-cite
  string (e.g., "FINRA Rule 4512 §C") is precomputed per-pack +
  cached in the notification template; the hot-path inserts the
  string without lookup.
- **Notification channel selection.** The substrate selects the
  optimal notification channel per the user's accessibility
  profile (push for default; SMS for low-bandwidth; email for
  audit; voice for visually-impaired). Selection logic is in the
  substrate, not duplicated per µservice.
- **Cooling-off override-fatigue mitigation.** A user who overrides
  cooling-off ≥ 5 times in 30 days receives an elevated friction
  surface (the override path requires step-up auth per ADR-0188);
  the substrate auto-detects override-fatigue exploitation.

### §C.6. Code quality dimension

The substrate enforces code quality through canonical patterns:

- **Single ingress trait.** Per-µservice integration uses one trait
  `DecisionResilienceGate::check_or_cool_off()`; no µservice
  authors its own cooling-off logic. CI lane
  `oya-governance-decision-resilience-no-private-impl` blocks
  any µservice that re-implements cooling-off / trusted-contact /
  mutation-cadence locally.
- **No `#[cfg(test)]` bypass paths.** The substrate's Cedar gate
  evaluates in test as in prod. CI lane
  `oya-governance-no-test-bypass` blocks any code path with
  test-only short-circuits in decision-resilience gates.
- **Mandatory documentation block.** Every µservice that emits a
  consequential mutation MUST include a
  `compliance.md §decision-resilience-edge-cases` section per the
  §3.2.5 row-coverage requirement. CI lane
  `oya-governance-critical-path-coverage` verifies presence
  + binds to ADR-0303 cite.
- **Deterministic test fixtures.** The shared crate ships test
  fixtures for each of the four primitives (cooling-off-window-
  fixture, trusted-contact-attestation-fixture, mutation-cadence-
  fixture, guardianship-overlay-fixture). Per-µservice contract
  tests reuse these fixtures.
- **No magic numbers.** All thresholds (cooling-off windows,
  z-score thresholds, override-fatigue limits) are declared in
  `iac/<env>-cooling-off.yaml` per ADR-0254 deployment-model-
  spectrum + bound to ADR-0258 SemVer for changes.
- **Audit-event-class registration enforcement.** New event classes
  added by µservices in this domain are blocked from compilation
  unless registered in the central ADR-0263 registry.
- **Property-based test coverage.** The shared crate ships
  property-based tests (proptest crate) for the cooling-off timer
  monotonicity, the cadence z-score determinism, the guardianship
  overlay precedence, and the trusted-contact deduplication.
  Coverage minimum 85% per ADR-0212 buildability doctrine.

## §D. Detailed mechanics

### §D-1. Cooling-off timer — full mechanics

The cooling-off timer is the substrate primitive that addresses
irreversibility risk. The mechanics:

**Trigger conditions (when the timer opens):**

- **First-time large-amount mutations.** A mutation whose amount
  exceeds the per-tenant + per-µservice threshold AND the user has
  no prior mutation of equivalent or higher amount in the trailing
  90 days. Default threshold: USD 5,000 single-shot; configurable
  per-tenant.
- **Tier-elevated mutations.** A mutation whose amount exceeds the
  FATF Recommendation 10 threshold (USD 10,000 single-shot for
  CTR-reporting jurisdictions; equivalent local thresholds for
  non-USD packs). Always triggers cooling-off regardless of user's
  prior history.
- **First-time recipient.** A transfer to a recipient who is not
  in the user's 90-day prior-recipient set. Cooling-off applies
  regardless of amount.
- **Atypical-hours mutations.** A mutation initiated outside the
  user's typical-hours pattern (e.g., between 02:00-05:00 local
  time when the user's prior 30-day mutations are concentrated
  09:00-18:00). Cooling-off applies on amount-threshold-passing
  mutations only.
- **Geo-impossibility-resolved mutations.** A mutation that follows
  a recently-resolved geo-impossibility signal (e.g., the user's
  prior session was Seoul, the current session is Manila; if
  re-authenticated, cooling-off applies on the next consequential
  mutation).
- **Post-recovery mutations.** A mutation in the 72h post-account-
  recovery window per ADR-0299. Cooling-off applies on amount-
  threshold-passing mutations.
- **Audience-type-tier mutations.** Tenants with
  `audience_type = SENIOR_PROTECTED` or
  `audience_type = COGNITIVE_PROTECTED` apply cooling-off at the
  per-pack regulator floor independent of user-specific history.

**Window durations (default substrate values; per-µservice override
permitted up to per-pack ceiling):**

| Trigger | Default window | Per-pack ceiling |
|---|---:|---|
| First-time large-amount | 60 minutes | FINRA: ≤ 25 business days |
| Tier-elevated (FATF threshold) | 72 hours | FATF: no ceiling |
| First-time recipient | 24 hours | per-pack jurisdiction |
| Atypical-hours | 4 hours | per-pack jurisdiction |
| Geo-impossibility-resolved | 24 hours | per-pack jurisdiction |
| Post-recovery (ADR-0299) | 72 hours | per-pack jurisdiction |
| SENIOR_PROTECTED tier | 72 hours | FINRA Rule 2165: ≤ 25 bd |
| COGNITIVE_PROTECTED tier | 72 hours | per-jurisdiction MCA |
| Guardianship-protected | 7 days | per-jurisdiction MCA |

**State machine:**

```
   ┌─────────────────────────────────────────────────────────┐
   │                                                         │
   │   ┌──────────┐ mutation_triggered  ┌───────────────────┐│
   │   │ Idle      │ ──────────────────▶│ CoolingOffActive   ││
   │   │           │                     │  - timer_ms        ││
   │   │           │ ◀──────────────────┤   - amount_redacted  │
   │   │           │   timer_elapsed     │   - audience_type   │
   │   │           │   OR user_override  │   - pack_cite       │
   │   │           │                     └─────────┬──────────┘
   │   │           │                               │
   │   │           │ ◀─── user_cancel  ────────────┘
   │   │           │
   │   │           │ ◀── trusted_contact_alert_sent (parallel)
   │   │           │ ◀── trusted_contact_ack         (optional)
   │   └──────────┘
   │
   │ Emergency-services path (ADR-0298) bypasses entirely.
   │ Per-pack regulator floors cannot be user-overridden.
   └─────────────────────────────────────────────────────────┘
```

**User actions (during the active window):**

- **Cancel:** User cancels the mutation; window closes; emits
  `CoolingOffElapsed{state=cancelled}`.
- **Wait:** User waits for the window to elapse; mutation commits
  at the end of the window; emits `CoolingOffElapsed{state=committed}`.
- **Override (where permitted):** User explicitly acknowledges the
  override workflow + completes step-up auth per ADR-0188; window
  closes; mutation commits; emits `CoolingOffOverridden`.
- **Trusted-contact intervention:** Trusted contact contacts the
  user out-of-band; user cancels per their own decision; emits
  `CoolingOffElapsed{state=cancelled, contact_intervention=true}`.

**Override workflow:**

The override path is intentionally high-friction to discourage
casual override:

1. User clicks "Cancel cooling-off and proceed now."
2. Substrate presents the override-acknowledgment: "Cooling-off
   protects you from irreversible decisions. Continuing now
   bypasses this protection. Cancel anyway?"
3. User clicks "Continue anyway."
4. Substrate presents step-up auth per ADR-0188 (passkey re-auth
   required; SMS fallback explicitly forbidden per ADR-0297
   anti-spoof).
5. User completes passkey re-auth.
6. Substrate audits the override + emits `CoolingOffOverridden`.
7. Substrate checks override-fatigue (≥ 5 overrides in trailing
   30 days). If exceeded, the override path is locked + the user
   is escalated to the tenant's customer-success workflow for
   manual review.
8. Mutation commits.

**Idempotency:**

The cooling-off timer is keyed by `(tenant_id, principal_id, request_idempotency_key)`.
A retry of the same mutation within the active window does NOT
re-trigger; it returns the existing `Retry-After` + `X-Oya-Cooling-Off-Until`
headers. This satisfies the ADR-0258 idempotency invariant.

### §D-2. Trusted-contact — full mechanics (FINRA Rule 4512 + per-jurisdiction equivalents)

The trusted-contact primitive is the substrate's social-isolation
defence. The mechanics:

**FINRA Rule 4512 §C compliance:**

FINRA Rule 4512(a)(1)(F) (effective 2018-02-05) requires broker-dealers
to make reasonable efforts to obtain the name + contact information
of a trusted contact person upon opening an account. The contact's
role is informational; they receive notifications about possible
financial exploitation, suspected diminished capacity, or unauthorized
activity but cannot direct the account.

The substrate's compliance:

- **Opt-in workflow.** Tenants with `audience_type = SENIOR_PROTECTED`
  or `audience_type = HIGH_VALUE_FINANCIAL` present trusted-contact
  registration as a default surface at account opening; the user is
  not required to designate one (per FINRA Rule 4512 the user may
  decline). The substrate's tenancy substrate per ADR-0244 implements
  the registration UI.
- **Attestation chain.** The trusted-contact attestation includes:
  the contact's name (pseudonymized per ADR-0273 PII handling);
  contact email + phone (encrypted at rest per ADR-0251 §D-10);
  the relationship (family / professional / advisor); the user's
  explicit consent (audited per ADR-0263 `TrustedContactRegistered`
  event class).
- **Notification dispatch.** When a cooling-off triggers on a
  qualifying mutation, the substrate dispatches notification to the
  trusted contact via the user's preferred channel (email per ADR-
  0273; SMS per ADR-0257; voice/push per a future notification ADR where opt-in). The
  notification carries: a redacted summary of the mutation type
  (e.g., "a wire transfer of approximately $25,000"), the cooling-
  off window expiry, the user's contact info, and the regulator
  cite ("This notification is sent per FINRA Rule 4512 §C").
- **Notification timing.** The notification is dispatched within
  60 seconds of the cooling-off trigger. The trusted contact may
  acknowledge or not; acknowledgment is optional.
- **Contact rights.** The trusted contact receives the notification;
  they cannot direct the account; they may contact the user
  out-of-band. The substrate provides no surface for the contact
  to login or act.

**Per-jurisdiction equivalents:**

| Jurisdiction | Equivalent | Substrate handling |
|---|---|---|
| US-Federal | FINRA Rule 4512 §C (broker-dealer); FINRA Rule 2165 (specified-adult hold up to 25 bd) | Default for HIGH_VALUE_FINANCIAL + SENIOR_PROTECTED |
| US-State | NASAA Model Act (45+ states adopted); CA-SB-496; FL-§ 415.111 | Compose per state-pack overlay |
| EU | GDPR + national MCA equivalents (DE-BetreuungsRecht, FR-Code-civil-art-440, ES-Ley-de-jurisdicción-voluntaria) | Compose per EU member-state pack |
| UK | Mental Capacity Act 2005 + Care Act 2014 | `pack-uk-mca` overlay |
| KR | 성년후견인 제도 (Korean Civil Act §9-12); FSC senior-investor-protection 2022 | `pack-kr-adult-guardianship` overlay |
| JP | 成年後見制度 (Japan Civil Code 7-21); FSA-elder-protection-guidelines | `pack-jp-adult-guardianship` overlay |
| CA | Adult Protection Acts (provincial variation); BC-AdultGuardianship Act | `pack-ca-prov-overlay` |
| AU | Aged Care Act 1997; per-state mental-capacity statutes (NSW-MentalHealthAct-§85) | `pack-au-aged-care` overlay |

**Cross-tenant trusted-contact blocking:**

A trusted-contact attestation is scoped to its registering tenant.
Cross-tenant attestation is blocked: a user cannot use their tenant
A trusted-contact as their tenant B trusted-contact unless they
re-register in tenant B. This satisfies the ADR-0244 tenant-scoping
invariant and prevents cross-tenant correlation of the contact's
identity.

**De-registration + change:**

- The user may de-register or change their trusted contact at any
  time via the tenancy substrate surface.
- A de-registration during an active cooling-off does NOT cancel
  the in-flight notification; the contact will receive the alert.
- A re-registration following de-registration requires the same
  attestation workflow as initial registration (consent +
  audit-trail).

**Notification template (canonical):**

```
Subject: Trusted-Contact Alert from {tenant_name}

This is an automated notification per FINRA Rule 4512 §C (or equivalent
jurisdictional rule).

You are listed as the Trusted Contact for {user_first_name_redacted}.

The user has initiated a {mutation_type} of approximately
{amount_redacted_bucket} on {date_iso_8601}.

A cooling-off window of {window_duration_human} is in effect. The
mutation will not commit until the window elapses or the user takes
explicit action.

You are NOT being asked to approve, reject, or direct this mutation.
You are informed so you may contact the user if you have concerns
that this mutation may not be in the user's interest.

To contact the user: {user_contact_redacted}.

If you have concerns about possible financial exploitation, please
contact {tenant_customer_success} or call {tenant_fraud_hotline}.

You may opt-out of these notifications by clicking: {opt_out_link}.

This notification carries no obligation; it is informational only.

Sent per FINRA Rule 4512 §C / equivalent per jurisdiction.
```

### §D-3. Mutation-cadence detection — full mechanics

The mutation-cadence detector addresses behavioural-deviation risk.
The mechanics:

**Baseline computation:**

- **Window:** 30-day trailing rolling baseline per
  `(tenant_id, principal_id, µservice, mutation_class)`.
- **Aggregation:** Per-day count of mutations of the class. Computed
  offline (batch nightly) per ADR-0263 observability emission.
- **Statistics:** Mean μ + standard deviation σ + median + p95 +
  p99 over the 30-day window.
- **Drift detection:** If the 7-day mean deviates ≥ 2σ from the
  30-day mean, the baseline is auto-recomputed weekly to avoid
  false-positives on legitimate new-normal patterns.

**Z-score computation (hot-path):**

```
z_score = (current_window_count - μ_30day) / σ_30day_smoothed
```

Where `current_window_count` is the count of mutations of the class
in the trailing 10-minute window, and `σ_30day_smoothed` is the
30-day standard deviation with Laplace smoothing (σ + 1 to avoid
division by zero on low-activity users).

**Threshold:**

| z_score | Treatment |
|---:|---|
| ≤ 2.0 | No cool-down; pass-through |
| 2.0 – 3.0 | Informational notice; "your activity is unusual but proceeding"; emits `MutationCadenceElevated` event class |
| 3.0 – 5.0 | Cool-down 5 minutes; user informed + may proceed after wait; emits `RapidMutationCoolDownTriggered` |
| > 5.0 | Cool-down 15 minutes; user informed + recommended to contact support; emits `RapidMutationCoolDownTriggered{tier=high}`; if persists, escalates to tenant customer-success queue |

**False-positive handling:**

- **Legitimate burst.** A user posting one tweet per minute during
  a breaking-news event is NOT cognitively impaired. The mutation-
  class is partitioned: high-frequency-permitted classes (post,
  comment, like, view) have higher thresholds (z > 8.0); low-
  frequency-irreversible classes (transfer, delete, send) have
  the standard z > 3.0.
- **Tenant-specific tuning.** A tenant in the `B2B_TENANT` audience
  type with a high-frequency-API-user cohort may raise the threshold
  per-µservice; the substrate accepts per-tenant overrides within
  the per-pack ceiling.
- **Promotion / role-change drift.** A user whose new role causes
  legitimate 5× cadence increase is auto-detected via the weekly
  baseline-recomputation; false-positives self-resolve within 7
  days.

**Privacy invariant:**

- The 30-day baseline is **pseudonymous** at rest; the baseline
  is keyed by hashed `(tenant_id, principal_id)` pair, never by
  raw PII.
- The baseline is **cell-local** per ADR-0248; not mirrored
  cross-region for residency-restricted packs.
- The z-score computation happens in-process; no external service
  receives the raw mutation history.

### §D-4. Guardianship overlay — per-jurisdiction registry

The guardianship-overlay registry maps per-jurisdiction adult-
guardianship laws to substrate behavior. The mechanics:

**Per-jurisdiction packs:**

```yaml
# Per-pack guardianship overlay (canonical examples)
pack-us-upc:
  legal_cite: "Uniform Probate Code §5-303 (adult guardianship)"
  capacity_adjudication_authority: "state probate court"
  guardian_signature_required_for_mutations:
    - financial_transfer_above_threshold
    - real_property_disposition
    - healthcare_decision_terminal
  threshold_amount_usd: 500
  override_path: "court order modification"
  legal_review_council: "council-legal-us"

pack-uk-mca:
  legal_cite: "Mental Capacity Act 2005 §1-3"
  capacity_adjudication_authority: "Court of Protection"
  guardian_signature_required_for_mutations:
    - financial_transfer
    - care_arrangement_change
    - residence_change
  threshold_amount_gbp: 500
  override_path: "best-interest decision per §4"
  legal_review_council: "council-legal-uk"

pack-kr-adult-guardianship:
  legal_cite: "Korean Civil Act §9-12 + KR Adult Guardianship Civil Act"
  capacity_adjudication_authority: "family court"
  guardian_signature_required_for_mutations:
    - financial_transfer
    - real_property
    - healthcare_decision
  threshold_amount_krw: 500000
  override_path: "court modification of guardianship order"
  legal_review_council: "council-legal-kr"

pack-jp-adult-guardianship:
  legal_cite: "Japan Civil Code Article 7-21"
  capacity_adjudication_authority: "family court"
  guardian_signature_required_for_mutations:
    - financial_transfer
    - real_property
    - healthcare_decision
    - care_facility_admission
  threshold_amount_jpy: 50000
  override_path: "court modification"
  legal_review_council: "council-legal-jp"

pack-eu-mca-de:
  legal_cite: "BGB §1814 (Betreuung)"
  capacity_adjudication_authority: "Betreuungsgericht"
  guardian_signature_required_for_mutations:
    - financial_transfer_above_threshold
    - real_property
    - healthcare_decision
  threshold_amount_eur: 500
  override_path: "court modification"
  legal_review_council: "council-legal-eu-de"
```

**Activation conditions:**

The overlay activates ONLY when:

1. The user has an explicit per-tenant guardianship attestation
   registered (a per-jurisdiction court order or its equivalent
   filed with the tenant + attested in the user's tenant record).
2. The mutation falls within the per-pack
   `guardian_signature_required_for_mutations` enumeration.
3. The mutation exceeds the per-pack threshold (if applicable).

**Overlay enforcement:**

When activated, the substrate:

1. Identifies the per-pack overlay applicable.
2. Looks up the guardian's attestation per the per-tenant guardian
   record.
3. Initiates a co-sign workflow: the user proposes the mutation;
   the guardian must counter-sign via the tenancy substrate's
   co-sign UI.
4. Mutation does NOT commit without guardian counter-signature.
5. Audit emits `GuardianshipOverlayApplied{guardian_attestation_id,
   legal_cite}`.

**Emergency-services bypass:**

Per ADR-0298, emergency-services paths NEVER trigger guardianship
overlay. A user in cardiac arrest cannot wait for guardian
co-signature on a medical-decision break-glass. The bypass emits
`GuardianshipOverlayBypassed` for audit; the action proceeds.

**Cross-jurisdiction conflict:**

When the user resides in jurisdiction A but the resource is held
in jurisdiction B, the higher-restriction pack wins per ADR-0304.
Example: a Korean resident with a U.S. brokerage account; both
KR-§9 + US-UPC apply; the substrate honors both.

### §D-5. Cedar policy fragment — `policy/decision-resilience.cedar`

The Cedar fragment composes the four primitives into a single
permit/forbid decision. The canonical fragment:

```cedar
// policy/decision-resilience.cedar
// Per-µservice Cedar fragment composed per ADR-0303 + ADR-0243
// fragment-lifecycle conventions per ADR-0294.

// Default-deny: consequential mutation refused unless decision-
// resilience predicates satisfied.

forbid (
  principal,
  action in [
    Action::"transfer_funds",
    Action::"delete_account_data",
    Action::"send_irreversible_message",
    Action::"commit_real_property_transaction",
    Action::"sign_legal_document",
    Action::"close_account",
    Action::"modify_beneficiary",
    Action::"export_bulk_data"
  ],
  resource
)
when {
  // Predicate 1: active cooling-off window
  context.cooling_off_until > context.now ||
  // Predicate 2: guardian signature required but not present
  (context.guardianship_overlay_active &&
   !context.guardian_counter_signature_present) ||
  // Predicate 3: mutation cadence z-score above threshold
  context.mutation_cadence_z_score > 3.0 ||
  // Predicate 4: trusted-contact alert pending dispatch
  (context.audience_type in
   ["SENIOR_PROTECTED", "HIGH_VALUE_FINANCIAL"] &&
   context.amount_bucket >= context.tier_threshold &&
   !context.trusted_contact_alert_dispatched)
};

// Emergency-services bypass per ADR-0298
permit (
  principal,
  action in [
    Action::"emergency_services_initiate",
    Action::"crisis_hotline_connect",
    Action::"healthcare_break_glass",
    Action::"shelter_mode_activate"
  ],
  resource
)
when {
  context.emergency_path_attested == true
};

// User override path (where permitted; not for per-pack regulator
// floors like FINRA Rule 2165 holds)
permit (
  principal,
  action == Action::"override_cooling_off",
  resource
)
when {
  context.user_override_acknowledged == true &&
  context.step_up_auth_completed == true &&
  context.per_pack_floor_locked == false &&
  context.override_fatigue_count_30d < 5
};

// Guardianship-protected emergency-services pre-attested bypass
permit (
  principal,
  action == Action::"emergency_proceed_without_guardian_signature",
  resource
)
when {
  context.life_safety_situation_attested == true &&
  context.guardianship_overlay_active == true &&
  context.post_hoc_audit_required == true
};
```

**Cedar evaluation order:**

The substrate evaluates the fragment with the following context
attributes set per-request:

- `context.cooling_off_until` — set to the active window expiry
  RFC 3339 timestamp, or `null` if no active window.
- `context.now` — request-time RFC 3339 timestamp from the
  µservice's clock.
- `context.guardianship_overlay_active` — boolean; set to `true` if
  the user has a per-tenant guardianship attestation matching the
  applicable per-jurisdiction overlay.
- `context.guardian_counter_signature_present` — boolean; set to
  `true` if the guardian has counter-signed this specific request.
- `context.mutation_cadence_z_score` — float; computed per §D-3.
- `context.audience_type` — string enum per ADR-0244.
- `context.amount_bucket` — bucketed amount (privacy-preserving
  per ADR-0263 cardinality budget).
- `context.tier_threshold` — bucketed threshold per the audience-type
  configuration.
- `context.trusted_contact_alert_dispatched` — boolean.
- `context.emergency_path_attested` — boolean (per ADR-0298).
- `context.user_override_acknowledged` — boolean (per the override
  workflow §D-1).
- `context.step_up_auth_completed` — boolean (per ADR-0188).
- `context.per_pack_floor_locked` — boolean; `true` for FINRA Rule
  2165 specified-adult holds.
- `context.override_fatigue_count_30d` — integer.
- `context.life_safety_situation_attested` — boolean (per ADR-0298).
- `context.post_hoc_audit_required` — boolean.

**Fragment versioning + lifecycle:**

Per ADR-0294, fragment changes:

1. Author the change in the µservice's `policy/decision-resilience.cedar`.
2. Sign the change per the µservice's signing key.
3. Submit to the foundry pipeline (per ADR-0116 retired external
   tooling; raw git/gh).
4. Soak ≥ 60 seconds in a dark deployment with audit emission.
5. Promote via the merge queue per ADR-0111.
6. Roll back via `oya policy revert` if any audit-event-class shows
   regression.

### §D-6. Per-cell-tier variants

Per ADR-0248, the substrate spans cell tiers:

- **Tier-0 cells (edge POPs).** Decision-resilience is N/A; the
  edge does not process consequential mutations. Edge enforces
  per ADR-0297 abuse-defence only.
- **Tier-1 cells (regional control planes).** Hosts the substrate's
  shared crate runtime; runs the cooling-off timer registry, the
  trusted-contact attestation cache, the mutation-cadence detector,
  the per-jurisdiction overlay registry.
- **Tier-2 cells (data plane regions).** Hosts µservice instances
  that invoke the substrate at the per-µservice gate; emits
  audit-event-classes via ADR-0263.
- **Tier-3 cells (compliance-isolated).** Same as Tier-2 but with
  enhanced data-residency floor; per-pack overlays applicable.
- **Tier-4 cells (HIPAA-eligible / PCI-isolated / EU-sovereign).**
  Decision-resilience inherits + adds per-pack floor (e.g.,
  HIPAA delete cool-down ≥ 30 days; PCI key-rotation cool-down).

### §D-7. Observability — metrics, dashboards, audit-event-classes

Per ADR-0263 observability emission contract, the substrate emits:

**Audit-event-classes (registered in central registry):**

- `CoolingOffTriggered`
- `CoolingOffOverridden`
- `CoolingOffElapsed`
- `TrustedContactRegistered`
- `TrustedContactAlertSent`
- `TrustedContactAlertAcked`
- `TrustedContactDeregistered`
- `RapidMutationCoolDownTriggered`
- `MutationCadenceElevated`
- `GuardianshipOverlayApplied`
- `GuardianshipOverlayBypassed`
- `DecisionResilienceUserAccommodation`

**Metrics (cardinality budget):**

| Metric | Dimensions | Cardinality bound |
|---|---|---:|
| `oya_decision_resilience_cooling_off_active_gauge` | tenant_bucket, µservice, audience_type, window_class | 5K |
| `oya_decision_resilience_cooling_off_triggered_counter` | same | 5K |
| `oya_decision_resilience_cooling_off_overridden_counter` | same | 5K |
| `oya_decision_resilience_trusted_contact_notify_counter` | tenant_bucket, audience_type, channel | 2K |
| `oya_decision_resilience_mutation_cadence_z_histogram` | tenant_bucket, µservice, audience_type | 5K |
| `oya_decision_resilience_guardianship_overlay_apply_counter` | jurisdiction_pack, audience_type | 500 |
| `oya_decision_resilience_emergency_bypass_counter` | µservice, route | 500 |
| `oya_decision_resilience_override_fatigue_count_gauge` | tenant_bucket, audience_type | 200 |

Aggregate cardinality ≤ 18K per cell — within ADR-0263 ceiling
of 50K per cell.

**Dashboard (canonical 12-panel layout):**

Each consequential-mutation-emitting µservice ships
`dashboards/decision-resilience.json` with panels:

1. Cooling-off active gauge by µservice
2. Cooling-off triggered rate per minute
3. User-override rate (per-pack-non-floor)
4. Trusted-contact notification latency p99
5. Trusted-contact notification fan-out volume
6. Mutation-cadence z-score heatmap (audience_type × hour-of-day)
7. Guardianship overlay applied count per jurisdiction
8. Emergency-services bypass count
9. Override-fatigue alerts (≥5 in 30 days)
10. Per-pack legal-cite distribution
11. Audience-type distribution
12. Cell-tier health (Tier-1 substrate latency)

### §D-8. Per-tenant audience-type tuning

Per ADR-0244 audience-type enum, the substrate's defaults vary:

| Audience type | Cooling-off default | Trusted-contact | Mutation-cadence threshold | Guardianship overlay |
|---|---:|---|---:|---|
| `B2C_CONSUMER` | 60 min on first-time large | optional | z > 3.0 | per-jurisdiction if attested |
| `B2B_TENANT` | per-tenant config; default 30 min | optional | z > 5.0 (high-API) | rarely applicable |
| `SENIOR_PROTECTED` | 72 hours on amount ≥ tier; tier auto-elevates | default surface | z > 2.5 | always applicable per-jurisdiction |
| `HIGH_VALUE_FINANCIAL` | 72 hours on FATF tier; auto | default surface | z > 3.0 | per-jurisdiction if attested |
| `MINOR_PII` (per ADR-0292) | per-COPPA / KOSA / AADC | parental | z > 3.0 | always applicable |
| `BEREAVEMENT_PROTECTED` | 7-day user-elected | optional | z > 2.5 | per-jurisdiction if attested |
| `COGNITIVE_PROTECTED` | 72 hours on consequential | default surface | z > 2.5 | per-jurisdiction if attested |
| `SECURITY_RESEARCHER` (per ADR-0297) | exempt with attestation | n/a | exempt | n/a |
| `FRIENDLY_CRAWLER_PARTNER` (per ADR-0297) | exempt | n/a | exempt | n/a |

### §D-9. Compliance interactions

The decision-resilience baseline composes with regulatory packs:

- **FINRA Rule 4512 §C.** Trusted-contact registration + notification.
- **FINRA Rule 2165.** Specified-adult temporary holds up to 25
  business days; the substrate's cooling-off window may extend to
  this ceiling on suspected exploitation.
- **NASAA Model Act.** State-broker-dealer + state-investment-advisor
  variants; the substrate composes per-state overlay.
- **HIPAA §164.524 + §164.526.** Healthcare delete cool-down ≥ 30
  days for medical records.
- **GDPR Article 17 + Article 21.** EU right-to-erasure cool-down +
  right-to-object cool-down; the substrate honors the 30-day
  consideration window per Article 12(3).
- **KR-PIPA Article 36 + Article 37.** Korean cool-down on personal
  data deletion requests.
- **PCI-DSS Requirement 8.3.** Multi-factor authentication on
  consequential mutations + cool-down on detected anomaly.
- **EU AI Act Article 14 (human oversight).** Cooling-off on AI-
  initiated mutations in regulated AI systems.
- **COPPA + KOSA + EU AADC.** Parental cool-down + guardian co-sign
  on minor mutations per ADR-0292.

## §E. Implementation footprint

### §E.1. New crate

```
oya-shared-decision-resilience/
├── Cargo.toml                          # workspace crate, single-concern per ADR-0131
├── src/
│   ├── lib.rs                          # crate root; exports DecisionResilienceGate trait
│   ├── cooling_off/
│   │   ├── mod.rs                      # cooling-off submodule
│   │   ├── timer.rs                    # timer state machine
│   │   ├── registry.rs                 # cell-local active-window registry
│   │   └── window_class.rs             # WindowClass enum + defaults
│   ├── trusted_contact/
│   │   ├── mod.rs                      # trusted-contact submodule
│   │   ├── attestation.rs              # attestation chain
│   │   ├── notifier.rs                 # FINRA + per-jurisdiction notification dispatch
│   │   └── template.rs                 # canonical notification template
│   ├── mutation_cadence/
│   │   ├── mod.rs                      # mutation-cadence submodule
│   │   ├── baseline.rs                 # 30-day rolling baseline
│   │   ├── z_score.rs                  # hot-path z-score computation
│   │   └── drift.rs                    # weekly drift recomputation
│   ├── guardianship/
│   │   ├── mod.rs                      # guardianship submodule
│   │   ├── overlay_registry.rs         # per-jurisdiction overlay registry
│   │   ├── attestation.rs              # per-tenant guardian attestation
│   │   └── co_sign.rs                  # co-sign workflow
│   ├── cedar_fragment/
│   │   ├── mod.rs                      # Cedar fragment authoring helpers
│   │   ├── context_builder.rs          # request-context builder
│   │   └── evaluator.rs                # invokes ADR-0243 Cedar engine
│   ├── audit/
│   │   ├── mod.rs                      # audit-event-class emission per ADR-0263
│   │   ├── event_class.rs              # enum of event classes
│   │   └── emit.rs                     # OTel + audit-chain emit
│   ├── observability/
│   │   ├── mod.rs                      # metrics + dashboards per ADR-0263
│   │   ├── metrics.rs                  # Prometheus + OTel metric definitions
│   │   └── tracing.rs                  # span attributes
│   ├── tenancy/
│   │   ├── mod.rs                      # tenancy substrate integration per ADR-0244
│   │   └── audience_type.rs            # AudienceType enum extension
│   └── error.rs                        # canonical errors
├── tests/
│   ├── cooling_off_property.rs         # property-based tests
│   ├── trusted_contact_finra.rs        # FINRA Rule 4512 conformance
│   ├── mutation_cadence_drift.rs       # drift handling tests
│   ├── guardianship_per_pack.rs        # per-jurisdiction overlay tests
│   ├── cedar_fragment_evaluation.rs    # Cedar fragment integration
│   └── fixtures/
│       ├── window_fixtures.rs
│       ├── attestation_fixtures.rs
│       └── cadence_fixtures.rs
└── docs/
    ├── README.md
    ├── ARCHITECTURE.md
    ├── usage.md
    └── per-pack-overlay-authoring.md
```

### §E.2. New µservice extensions

Every consequential-mutation-emitting µservice extends with:

```
microservices/<name>/
├── policy/
│   ├── decision-resilience.cedar       # Cedar fragment per §D-5
│   └── decision-resilience-overlays/   # per-pack overlays
│       ├── pack-us-finra.cedar
│       ├── pack-eu-mca.cedar
│       ├── pack-kr-adult-guardianship.cedar
│       ├── pack-jp-adult-guardianship.cedar
│       └── pack-uk-mca.cedar
├── iac/
│   ├── dev-cooling-off.yaml            # per-env cooling-off windows
│   ├── staging-cooling-off.yaml
│   └── prod-cooling-off.yaml
├── docs/
│   ├── ARCHITECTURE.md                 # +§decision-resilience section
│   ├── PRD.md                          # +§decision-resilience-edge-cases
│   ├── compliance.md                   # +§decision-resilience per §3.2.5 rows 4, 20
│   └── runbooks/
│       ├── decision-resilience-cooling-off-stuck.md
│       ├── decision-resilience-trusted-contact-dispatch-failure.md
│       └── decision-resilience-guardianship-overlay-conflict.md
├── tests/
│   └── decision_resilience_contract.rs # contract test per §C.6
├── dashboards/
│   └── decision-resilience.json        # canonical 12-panel dashboard
└── slos/
    ├── cooling-off-latency.openslo.yaml
    ├── trusted-contact-dispatch-latency.openslo.yaml
    └── mutation-cadence-z-lookup-latency.openslo.yaml
```

### §E.3. New runbooks

Three new runbooks per µservice (authored by Wave-3 ops agents per
the runbook template):

- `decision-resilience-cooling-off-stuck.md` — diagnose + resolve a
  cooling-off window that fails to elapse (clock drift, registry
  corruption, audit emission failure).
- `decision-resilience-trusted-contact-dispatch-failure.md` —
  diagnose + resolve trusted-contact notification dispatch failures
  (vendor outage per ADR-0257 SMS / ADR-0273 email / future push substrate ADR).
- `decision-resilience-guardianship-overlay-conflict.md` — diagnose
  + resolve conflicts when multiple per-jurisdiction overlays apply
  (per ADR-0304 higher-restriction-wins).

### §E.4. New CI lanes

- `oya-governance-cooling-off-coverage` — verifies every
  consequential-mutation µservice declares cooling-off windows.
- `oya-governance-trusted-contact-binding` — verifies tenants
  with `SENIOR_PROTECTED` / `HIGH_VALUE_FINANCIAL` audience types
  have a trusted-contact attestation surface.
- `oya-governance-mutation-cool-down` — verifies rapid-
  sequential-mutation cool-down is wired per µservice + per data
  class.
- `oya-governance-guardianship-overlay` — verifies per-
  jurisdiction overlays are wired (≥5 jurisdictions required: US,
  EU, UK, KR, JP).
- `oya-governance-decision-resilience` — aggregate roll-up lane.
- `oya-governance-no-age-inference` — blocks any code that
  assigns `SENIOR_PROTECTED` by age inference rather than opt-in.

### §E.5. Vendor selection rationale

- **SMS dispatch** — Twilio Programmable Messaging (per ADR-0257);
  fallback Vonage; per-tenant config.
- **Email dispatch** — Per-tenant DKIM/SPF/DMARC per ADR-0273;
  AWS SES + SendGrid for high-volume.
- **Push dispatch** — Apple APNs + Google FCM (per a future push substrate ADR).
- **Voice dispatch** (for visually-impaired accommodation) — Twilio
  Voice with TTS via Amazon Polly.
- **Cedar evaluation** — Cedar engine v4.2 LTS per ADR-0243.
- **Audit-chain** — Merkle-sealed per ADR-0028 + ADR-0263.

## §F. Migration

### §F.1. Per-µservice rollout sequenced by impact-severity

| Wave | Cohort | µservices | Window |
|---:|---|---|---|
| 1 | High-financial-impact | payments, billing, finops-portal, marketplace | 2026-05-25 → 2026-06-30 |
| 2 | Healthcare-impact | (pending healthcare µservice roster after scaffold) | 2026-06-30 → 2026-07-31 |
| 3 | High-social-impact | governance, ontology, tenancy, identity | 2026-07-31 → 2026-08-31 |
| 4 | Long-tail consequential | mail, notes, social, comms-email, workflow-studio | 2026-08-31 → 2026-09-15 |
| 5 | Cleanup + audit | all remaining | 2026-09-15 → 2026-09-30 |

### §F.2. Per-µservice migration playbook

For each µservice rolling out:

1. Add `oya-shared-decision-resilience` workspace dependency.
2. Author `policy/decision-resilience.cedar` per the template.
3. Author `iac/<env>-cooling-off.yaml` per the per-data-class window
   defaults.
4. Add `§decision-resilience` section to `ARCHITECTURE.md`.
5. Add `§decision-resilience-edge-cases` section to `PRD.md` and
   `compliance.md` per §3.2.5 row coverage.
6. Add `dashboards/decision-resilience.json` per the canonical panel
   layout.
7. Add SLO files per §E.2.
8. Add contract test per §C.6.
9. Pass `oya-governance-decision-resilience` lane.
10. Soak ≥ 60s in dark deployment per ADR-0294; promote.

### §F.3. Per-cell rollout pattern

Cells receive the substrate baseline + per-pack overlays in order:

1. dev cells receive 2026-05-25 → 2026-06-15.
2. staging cells receive 2026-06-15 → 2026-07-15.
3. prod-non-residency cells receive 2026-07-15 → 2026-08-15.
4. prod-residency-restricted cells (EU-sovereign, KR-pack, JP-pack,
   CN-PIPL-pack) receive 2026-08-15 → 2026-09-15.

### §F.4. What is NOT migrated

- Per-tenant trusted-contact UI is the responsibility of the
  tenancy substrate per ADR-0244; not migrated by this ADR's
  rollout.
- Per-µservice idempotency keys per ADR-0258 are NOT touched.
- Existing emergency-services bypass paths per ADR-0298 are NOT
  modified.

### §F.5. Rollback path

- Cell-tier rollback: `oya policy revert decision-resilience-v1` on
  the affected cell.
- µservice rollback: revert `policy/decision-resilience.cedar` to
  the prior signed version per ADR-0294.
- Soft-disable: set `decision_resilience_enabled = false` in the
  µservice's `iac/<env>-cooling-off.yaml`; active windows complete
  + no new windows open.
- Hard-disable: drop the workspace dependency; reverts to no
  decision-resilience friction; emergency-services bypass remains.

## §G. References

### §G.1. Hyperscaler precedents

- Stripe Radar + Adaptive Acceptance documentation (2024-2025).
- Apple Cash Terms 2024 + Family Sharing senior-protected
  documentation.
- Charles Schwab + Fidelity + Vanguard FINRA Rule 4512 + Rule
  2165 implementation patterns (per FINRA 2024 Senior Investor
  Protection Report).
- Chase + Bank of America + Wells Fargo cooling-off + trusted-
  contact programs (per American Bankers Association 2024 Survey).
- Microsoft Family Safety + Google Family Link purchase-confirmation
  patterns (2024 documentation).
- AARP 2024 BankSafe Report.
- FINRA 2024 Senior Investor Protection Report.

### §G.2. Standards + RFCs

- FINRA Rule 4512(a)(1)(F) — Trusted Contact (2018-02-05).
- FINRA Rule 2165 — Specified-Adult Holds (2018-02-05; amended
  2022-03-17).
- NASAA Model Act on Senior Investor Protection (45+ states
  adopted).
- US Uniform Probate Code §5-303 — Adult Guardianship.
- UK Mental Capacity Act 2005 §1-3.
- EU GDPR Article 12(3), 17, 21 — consideration windows.
- HIPAA §164.524, §164.526 — record amendment cool-downs.
- PCI-DSS Requirement 8.3 — multi-factor + anomaly cool-down.
- EU AI Act Article 14 — human oversight.
- COPPA + KOSA + EU AADC (per ADR-0292).
- RFC 3339 — Date and Time on the Internet (for timestamp format).
- RFC 9457 — Problem Details for HTTP APIs (for cooling-off response).
- RFC 7231 §6.5.8 — 409 Conflict.

### §G.3. Legal + compliance

- Korean Civil Act §9-12 + Korean Adult Guardianship Civil Act.
- Japan Civil Code Article 7-21 + Adult Guardianship Law.
- German BGB §1814 (Betreuung).
- French Code civil art. 440.
- Spanish Ley de jurisdicción voluntaria.
- US-State NASAA Model Act variants (45+ state-pack overlays).
- US-Federal FATF Recommendation 10 (USD 10,000 CTR threshold).
- KR-FSC Senior Investor Protection 2022.
- JP-FSA Elder Investor Protection Guidelines.

### §G.4. Internal portfolio ADRs

- ADR-0028 Audit Chain (Merkle-sealed).
- ADR-0099 Data Class Registry.
- ADR-0105 Thirteen-Layer Canonical Enum.
- ADR-0131 Per-µservice Flat Layout.
- ADR-0140 Cedar Policy Enforcement.
- ADR-0145 Inter-Microservice Communication Reform.
- ADR-0188 Passkey + WebAuthn as Canonical Auth.
- ADR-0212 Buildability Doctrine.
- ADR-0240 Sovereign-Cloud per Regional Pack.
- ADR-0242 Oyatie is a Tenant Doctrine.
- ADR-0243 Cedar as Universal Gate.
- ADR-0244 Tenant as Universal Scoping Primitive.
- ADR-0245 Substrate vs Product Layering.
- ADR-0246 Policy Engine Substrate Promotion.
- ADR-0248 Amazon-Shape Cellular Architecture.
- ADR-0250 Build Ahead of Certification Doctrine.
- ADR-0251 Compliance Pack — Cell Certification Levels.
- ADR-0258 API Versioning + SemVer Policy.
- ADR-0263 Observability Emission Contract.
- ADR-0272 Cookie Consent per Purpose.
- ADR-0276 Backup Portability per GDPR Article 20.
- ADR-0292 Minor User Doctrine.
- ADR-0294 Cedar Fragment Lifecycle.
- ADR-0297 Abuse-Defence Baseline.
- ADR-0298 Emergency-Services Bypass Doctrine.
- ADR-0299 Account-Recovery Resilience.
- ADR-0300 Whistleblower + Press-Freedom Anonymity.
- ADR-0301 Survivor-Safety Domestic-Abuse Mode.
- ADR-0302 Deceased-User Inheritance Doctrine.
- ADR-0304 Cross-Jurisdiction Conflict Resolution.
- ADR-0305 Delegated-Agent Authority Chain.
- ADR-0306 Disaster-Mode + Cell Resilience.

### §G.5. Standards docs

- `docs/standards/documentation-rigor.md` §3.2.5 rows 4 + 20 +
  §3.2.6 DRMP lifecycle.
- `docs/standards/doc-style.md`.
- `docs/templates/adr-template-v2.md`.
- `docs/templates/runbook-template-v2.md`.

### §G.6. Auto-memory feedback (related)

- feedback_quality_performance_scalability_bar
- feedback_clean_architecture_requirements
- feedback_no_silent_regression
- feedback_autonomous_implementation_artifacts
- feedback_canonical_base_localization
- feedback_oyatie_is_a_tenant_doctrine
- feedback_cedar_as_universal_gate
- feedback_compliance_pack_primitive
- feedback_naming_justification

## §H. Change log

- **2026-05-20** — Initial proposal. Bundled with keystone-bundle
  2026-05-20 foundational doctrine synthesis as the critical-path-
  cluster-cognitive-impairment-decision-resilience keystone. Closes
  documentation-rigor.md §3.2.5 rows 4 + 20. Enforcement advisory
  until 2026-09-15, BLOCKER thereafter.

---

End of ADR-0303.
