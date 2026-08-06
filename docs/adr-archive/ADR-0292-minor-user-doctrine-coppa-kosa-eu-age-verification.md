---
id: ADR-0292
status: Superseded
date: 2026-05-20
owners:
  - council-architecture
  - council-privacy
  - council-security
  - council-legal
  - council-product
  - ops-compliance
  - ops-trust-and-safety
  - axis-policy-engine
  - axis-audit-chain
  - axis-tenancy
  - axis-identity
  - axis-consent
  - axis-pack-registry
supersedes: []
amends:
  - ADR-0007-cedar-authorization-policy-and-persona-tier.md (introduces minor-user persona tier + age-gated obligations as Cedar context attributes)
  - ADR-0099-data-class-registry.md (adds MINOR_PII data class + per-jurisdiction subtypes)
  - ADR-0218-tenant-granular-control-surface.md (introduces tenant-level minor-user policy override surface)
  - ADR-0251-compliance-pack-cell-certification-levels.md (registers MINOR-USER-2024 as the umbrella minor-user pack)
superseded_by: [ADR-709]
amended_by: [ADR-0350]
related:
  - ADR-0002-tenant-and-identity-kernel.md
  - ADR-0003-audit-chain-and-evidence-emission.md
  - ADR-0007-cedar-authorization-policy-and-persona-tier.md
  - ADR-0008-data-use-boundary.md
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0010-regional-pack-architecture.md
  - ADR-0011-cross-microservice-contract-registry.md
  - ADR-0064-canonical-base-and-localization-packs.md
  - ADR-0099-data-class-registry.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0140-cedar-policy-enforcement.md
  - ADR-0144-eu-ai-act-graduated-risk-tier-model.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
  - ADR-0188-passkey-webauthn-as-canonical-auth.md
  - ADR-0218-tenant-granular-control-surface.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0249-multi-category-marketplace-doctrine.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
related_specs:
  - /specs/compliance-pack-schema.json
  - /specs/minor-user-doctrine.json
  - /specs/microservices/identity.json
  - /specs/microservices/consent.json
  - /specs/microservices/policy-engine.json
  - /specs/microservices/audit-chain.json
  - /specs/microservices/tenancy.json
  - /specs/microservices/age-assurance.json
  - /specs/microservices/payments.json
  - /specs/microservices/messenger.json
  - /specs/microservices/mail.json
  - /specs/microservices/marketplace.json
  - /specs/data-class-registry.json
  - /specs/cedar-fragment-schema.json
related_memory:
  - feedback_canonical_base_localization
  - feedback_quality_performance_scalability_bar
  - feedback_no_silent_regression
  - feedback_autonomous_implementation_artifacts
  - feedback_doc_coverage_enforced
  - feedback_clean_architecture_requirements
  - feedback_workflow_objectgraph_adapter_layer
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-b2c-tier-1-lockdown
keystone_position: 3-of-9
purpose: >
  Establish the Minor User Doctrine as a Tier-1 lockdown precondition for
  shipping any business-to-consumer (B2C) surface. Bind COPPA (US,
  under-13), KOSA (US 2024, under-17), EU age-verification guidance
  (2024-2025 enforcement), UK Age Appropriate Design Code (AADC), KR
  Youth Protection Revision Act 2024, and JP Act on Provision of Healthy
  Environment for Young People to a single canonical pack
  (MINOR-USER-2024) with per-jurisdiction age thresholds, age-assurance
  flows, parental-consent workflows, default-restricted feature surfaces,
  age-down protection, algorithm transparency, marketplace + payments
  restrictions, and per-µservice minor-user-aware UX semantics. The pack
  composes through ADR-0251 onto cells; tenant overrides flow through
  ADR-0218; Cedar gates flow through ADR-0243; audit-chain emissions
  flow through ADR-0246; consent records flow through the Consent
  µservice. No B2C surface ships without this doctrine; the platform
  refuses minor-affecting traffic when the doctrine is not pinned.
enforcement_status: blocker-before-any-b2c-tenant-onboarding
enforced_by:
  - cloud-ci/Rust gate packet minor-user-pack-pinning
  - cloud-ci/Rust gate packet age-threshold-per-jurisdiction-coherence
  - cloud-ci/Rust gate packet age-assurance-provider-binding
  - cloud-ci/Rust gate packet parental-consent-workflow-coverage
  - cloud-ci/Rust gate packet minor-default-privacy-maximal
  - cloud-ci/Rust gate packet age-down-edit-refusal
  - cloud-ci/Rust gate packet age-of-majority-migration-flow
  - cloud-ci/Rust gate packet algorithm-transparency-for-minors
  - cloud-ci/Rust gate packet marketplace-minor-purchase-gate
  - cloud-ci/Rust gate packet per-microservice-minor-ux-binding
  - cloud-ci/Rust gate packet audit-emission-on-minor-policy-decision
  - cloud-ci/Rust gate packet tenant-override-bounds-check
---

# ADR-0292: Minor User Doctrine — COPPA + KOSA + EU Age Verification

## Status

Proposed — 2026-05-20.

Bundled with the 9-ADR B2C Tier-1 lockdown keystone set. Lands as a
single multispectrum-reviewed PR. Partial acceptance is rejected because
the B2C surface cannot ship absent any one of: minor-user doctrine (this
ADR), consumer-grade identity, trust-and-safety reporting, payments and
refunds, accessibility, content moderation, creator economy and payouts,
dark-patterns refusal, and content-rating or region locks. Minor-user is the keystone
that **cannot be retrofitted** — a single under-age signup that is not
governed correctly is a regulator-facing incident with statutory
penalties starting at USD 50,120 per child per FTC COPPA Rule (2024
inflation-adjusted) and uncapped KOSA + EU + KR exposure.

Enforcement status is `blocker-before-any-b2c-tenant-onboarding`. The
doctrine becomes BLOCKER at the tenant-admission gate the moment any
tenant whose `tenant.facing ∈ { B2C, B2B2C, B2C2B }` is requested. Until
the pack registry substrate ships (per ADR-0251 §D-1 cadence), the
validators emit findings without failing the platform CI; the
tenant-admission gate itself, however, is BLOCKER from day one because
no B2C tenant should be admitted before the pack is pinned.

The doctrine is accepted in text now; the CI lanes that enforce it move
to platform-wide BLOCKER status only after:

1. `microservices/age-assurance/` is promoted to peer substrate
   µservice (per ADR-0246) with at least one production provider
   (Yoti, Onfido, Persona, or KR PASS) bound per jurisdiction.
2. `microservices/consent/` provides the verifiable-parental-consent
   workflow templates (per COPPA Rule §312.5(b)) including
   knowledge-based authentication, government-ID match, credit-card
   verification (USD 0.50+ refundable transaction), video-conference
   consent, and signed-form upload — at least three methods per
   jurisdiction.
3. The `MINOR-USER-2024` compliance pack (per ADR-0251) is authored,
   signed by the oyatie compliance-office Ed25519 key, and published at
   `pack/MINOR-USER-2024/`.
4. Per-jurisdiction age-threshold matrix is published in
   `/specs/minor-user-doctrine.json` and the policy-engine ingests it
   as Cedar context attributes.
5. Per-µservice minor-user-aware UX bindings are authored for every
   µservice in the B2C bill of materials (per §D-14).
6. The age-of-majority migration workflow is drilled end-to-end at
   least once per `ops-trust-and-safety` runbook.

Until those six bootstrap items land, the platform refuses to admit any
B2C tenant; the validators emit findings without failing the broader
platform CI lanes. Post-bootstrap, the lanes promote to BLOCKER across
the portfolio.

## Date

2026-05-20.

## Context

### The Tier-1 lockdown framing

The masterplan distinguishes between **Tier-1 lockdowns** (cannot ship
without; statutory penalty exposure) and **Tier-2 hardenings**
(should ship without; reputational exposure only). Minor-user doctrine
is the canonical Tier-1 lockdown: in every jurisdiction where the
platform aspires to operate B2C, the absence of a correct minor-user
governance substrate makes the platform structurally illegal.

The cost-of-failure is asymmetric:

| Regime | Per-violation statutory ceiling | Practical-floor enforcement | Forcing function |
|---|---|---|---|
| **COPPA (US, under-13)** | USD 50,120 per child per violation (FTC 2024 inflation-adjusted) | FTC actions on TikTok (USD 5.7M, 2019), YouTube (USD 170M, 2019), Epic Games (USD 275M, 2022), Microsoft (USD 20M, 2023), Amazon Alexa (USD 25M, 2023) | Statutory minimum + parens patriae state actions |
| **KOSA (US 2024)** | Variable; AG enforcement + state private rights of action | Pre-enforcement; first cases expected 2026-2027 | "Duty of care" for design choices affecting minors under 17 |
| **EU GDPR Article 8** | EUR 20M or 4% global turnover | Meta EUR 405M (Instagram for minors, 2022); TikTok EUR 345M (2023) | Member-state DPA + EDPB coordination |
| **EU DSA (very large online platforms)** | 6% global turnover | Pre-enforcement; first cases active 2024-2025 | Article 28 (minor protection) + Article 35 (risk mitigation) |
| **UK AADC (Age Appropriate Design Code, 2021)** | UK GDPR penalties (GBP 17.5M or 4% global turnover) | ICO actions on TikTok (GBP 12.7M, 2023); Discord under investigation; Snap under investigation | "Standards of age-appropriate design" — 15 mandatory standards |
| **KR Youth Protection Revision Act 2024 (청소년 보호법 개정)** | KRW 30M base + criminal liability of officers | KCC + MOGEF coordinated enforcement; PASS-based age verification compulsory | 셧다운제 (shutdown) repealed 2022 but consent + age-verification preserved |
| **JP 青少年が安全に安心してインターネットを利用できる環境の整備等に関する法律 (Act on Provision of Healthy Environment for Young People, 2008, 2024 amendments)** | Administrative orders + reputational | METI + Cabinet Office coordinated; filtering obligation for ISPs and platforms | Filtering + parental control obligations |
| **AU eSafety + Online Safety Act 2021** | AUD 782,500 per violation | eSafety Commissioner enforcement; age-assurance trial 2024-2025 | Coming into harder force 2026 |
| **CA AADC (California SB 1394 + AB 1949 + AB 2273)** | USD 7,500 per affected child (intentional) | First enforcement 2024-2025 | Mirrors UK AADC structurally |

The aggregate per-child statutory ceiling under a coordinated
multi-jurisdiction enforcement campaign (FTC + UK ICO + EU DPA + KR
KCC + JP METI all opening files on the same incident) exceeds USD 1M
per misclassified child, before reputational damage and forced-divest
risk.

### What "ad-hoc per-µservice minor handling" costs

Pre-doctrine, the temptation is to handle minor-affecting code paths
on a µservice-by-µservice basis. The cost of this approach is
super-linear in three axes:

1. **Definition drift.** Each µservice independently decides what
   "minor" means. Messenger uses 13 (COPPA-anchored); Marketplace uses
   18 (chargeback-anchored); Identity uses 14 (KR-anchored); Mail uses
   16 (EU-default-anchored). A single user whose age is 15 receives
   inconsistent treatment across the surface. A regulator inspection
   immediately surfaces the inconsistency.
2. **Consent fragmentation.** Each µservice independently captures
   parental consent without a shared consent ledger. Verifiable
   parental consent obtained for Messenger does not flow to Marketplace,
   forcing the parent to re-consent. Worse, a withdrawal of consent in
   the Consent µservice does not propagate; the µservices retain stale
   consent.
3. **Audit incoherence.** Each µservice independently emits audit
   events on minor-affecting decisions. The audit-chain (per ADR-0003)
   is per-µservice; without a doctrine-level emission convention, a
   regulator request for "all events that touched user X between
   dates A and B while X was a minor" requires N independent queries
   with N inconsistent schemas.

The doctrine pre-empts all three failure modes by establishing the
minor-user persona as **a first-class concern in the policy engine,
the consent ledger, the audit chain, and every µservice's UX
contract** — not a per-µservice afterthought.

### What every named hyperscaler / B2C reference actually does

The pattern across mature B2C platforms is unambiguous: minor-user
governance is centrally specified and pack-versioned, applied
uniformly across product surfaces, and audited continuously.

- **Apple Family Sharing + Screen Time + Ask to Buy (iOS 16+, 2023+ refresh).**
  Apple binds minor status at the Apple ID level; all purchases route
  through "Ask to Buy"; Screen Time + Communication Safety enforce
  per-feature defaults; Ad tracking disabled by default; Sign In With
  Apple's "Hide My Email" defaults to on. Apple ships **one minor
  policy**, not seventeen.
- **Google Family Link + YouTube Kids (post-2019 COPPA settlement).**
  Google binds minor status at the Google Account level; all
  Google-product surfaces (YouTube, Photos, Gmail, Chrome, Maps, Play,
  Search) consume the same minor-flag and obey the same default-
  restrictions matrix. The 2019 USD 170M YouTube settlement led to
  the YouTube-for-Kids surface being a separate product that **does
  not target ads** (the FTC consent decree's central remedy).
- **Microsoft Family Safety + Xbox Family Settings (post-2023 FTC settlement).**
  Microsoft binds minor status at the Microsoft Account level; the
  USD 20M FTC settlement (2023) over Xbox Live data retention drove a
  centralized minor-consent system. Xbox + Outlook.com + Teams +
  Minecraft all consume the same flag.
- **Meta Instagram Teen Accounts (2024 rollout).** Meta launched "Teen
  Accounts" in September 2024 with default-private profiles, strict
  message restrictions (no DMs from non-followers), sleep-mode 22:00-
  07:00, and sensitive-content limits. Parental supervision is
  required to alter any default; defaults are restored on every
  update.
- **TikTok For You restrictions for minors (post-EU DSA + KOSA preparation).**
  TikTok defaults 13-15-year-old accounts to private; restricts DMs
  to non-followers; defaults push notifications off after 21:00; and
  in EU (per DSA Article 28) offers a non-profiled feed option.
- **Roblox Account Age + Parental Controls.** Roblox segments accounts
  by age band (<13, 13-17, 18+) and applies different chat filtering,
  experience-eligibility, and purchase ceilings per band. Verifiable
  parental consent gates the <13 surface.
- **Discord Teen Safety Assist (2024).** Discord defaults teen-account
  DMs to off for non-friends; image scanning on; explicit content
  filter on; under regulatory pressure from UK Ofcom.
- **Snap Family Center (2022).** Snap requires parents to set up a
  Family Center to gain visibility into teen Snapchat usage; map
  visibility restricted; My AI restricted for under-18s; ads
  personalization off by default for minors.

The shared pattern: **minor status is centrally bound at the identity
layer and consumed uniformly across product surfaces, with the most
restrictive defaults applied first and parental consent required to
loosen any default.** The platform offers minor governance as a
foundational service, not a per-product feature.

### What this doctrine does

Establishes the **Minor User Doctrine** as the platform's first-class
primitive for "minor-aware behavior." The doctrine binds:

- Per-jurisdiction age thresholds at signup (per D-1).
- Age-assurance methods and providers (per D-2 + D-7).
- Verifiable parental consent flow (per D-3).
- Default-restricted feature surface (per D-4 + D-9).
- Tenant-level override surface (per D-5).
- Compliance pack composition (per D-6).
- Per-jurisdiction provider binding (per D-7).
- Age-of-majority migration (per D-8).
- Maximal-restriction defaults (per D-9).
- Age-down protection (per D-10).
- Algorithm transparency for minors (per D-11).
- Marketplace + payments restrictions (per D-12).
- Audit-chain emissions on every minor-affecting decision (per D-13).
- Per-µservice minor-user-aware UX bindings (per D-14).

The unit of minor-user composition is the unit of:

- Regulator response (one minor-user incident → one packet to FTC + UK
  ICO + relevant EU DPA + KR KCC + JP METI in parallel).
- Drift control (doctrine version → all-or-nothing activation).
- DPIA (Data Protection Impact Assessment) for minor-affecting features.
- Consent-record retention (per jurisdiction's minimum retention).
- Tenant onboarding refusal (B2C tenant cannot be admitted without
  pinning the doctrine).
- Audit emission (one event class: `minor_policy_decision_v1`).

### Why now (2026-05-20)

Six forcing functions converge in 2024-2026:

- **KOSA Senate passage (2024-07-30, 91-3).** The Kids Online Safety
  Act passed the US Senate with broad bipartisan support. The
  companion House bill (KOSPA) is in markup. Pre-enforcement
  preparation is the rational stance for any 2026-onward B2C platform;
  retrofit after enforcement begins is dramatically more expensive.
- **EU DSA Article 28 enforcement (active 2024-).** Very Large Online
  Platforms (VLOPs) must mitigate systemic risk to minors. First
  enforcement actions opened against Meta, TikTok, X, AliExpress in
  2024.
- **UK AADC enforcement maturation (2023-2025).** UK ICO concluded
  TikTok (GBP 12.7M, 2023) and opened investigations into Discord,
  Snap, Imgur. The AADC's 15 standards are now treated as enforceable
  baseline.
- **EU age-verification guidance (EDPB + Commission, 2024-2025).**
  The European Commission issued guidance on protection of minors
  under DSA Article 28; EDPB guidelines 02/2024 on consent of children
  + DSA-compatible age-assurance pilot launched 2024.
- **KR Youth Protection Revision Act 2024.** Re-enacted core youth-
  protection obligations after the 2022 셧다운제 (shutdown) repeal;
  consent + PASS-based age verification compulsory for over-the-top
  platforms targeting users under 14.
- **JP 2024 amendments to the Act on Provision of Healthy Environment
  for Young People.** Expanded filtering obligation; "internet user
  policy" obligation on platforms with substantial JP minor users.

These six converging forcing functions make the pre-2024 ad-hoc
posture untenable for a B2C platform launching in 2026.

### What this is NOT

- This is NOT a substitute for actually obtaining COPPA Safe Harbor
  certification (kidSAFE Seal, Privo, TRUSTe Children's Privacy
  Certification, ESRB Privacy Certified). The doctrine defines the
  substrate; the platform team still has to enroll, audit, and maintain
  Safe Harbor membership where commercially advantageous.
- This is NOT an ADR that closes per-jurisdiction interpretive
  questions. Each jurisdiction's pack overlay carries its own author
  + reviewer + signer who is responsible for the legal interpretation
  in that jurisdiction.
- This is NOT a marketing claim about "child safety completeness."
  Operating any B2C platform with minor users carries irreducible
  residual risk; the doctrine establishes the structural defenses but
  does not eliminate operational risk.
- This is NOT a substitute for Trust-and-Safety operational headcount.
  The doctrine establishes the platform's structural defenses; the
  ops-trust-and-safety team operates the runbooks, handles regulator
  correspondence, responds to parent-initiated DSARs + erasure
  requests, and drives incident response.
- This is NOT permission to operate in jurisdictions where the
  platform has not procured a per-jurisdiction age-assurance provider
  or has not contracted with a local DPO/representative as required.

## Decision

### D-1. Per-jurisdiction age thresholds (canonical)

Define the per-jurisdiction age threshold matrix canonically at
`/specs/minor-user-doctrine.json` and ingest it as Cedar context
attributes for evaluation.

| Jurisdiction | "Child" threshold (under) | "Teen" upper bound (under) | "Adult" age | Authority |
|---|---|---|---|---|
| **US (federal, COPPA)** | 13 | n/a | 13 | 15 USC §6501-6506; 16 CFR Part 312 (2024 amendments) |
| **US (federal, KOSA when in force)** | 13 (COPPA-anchored) | 17 | 17 | KOSA S.1409 (2024 Senate-passed) |
| **CA (state, AADC)** | 13 (COPPA-anchored) | 17 | 18 | CA Civ. Code §1798.99.28 et seq.; AB 2273; SB 1394 |
| **EU/EEA (GDPR Article 8, default)** | 16 | n/a | 16 | Reg. (EU) 2016/679 Art. 8(1) |
| **DE / NL / PL / RO / HU / LU / SI / SK / FI / EE / IE (EU Art. 8(1) member-state override)** | 16 | n/a | 16 | Member-state law |
| **AT / BG / CY / FR / IT / LV / LT / MT** | 13-15 | n/a | 13-15 | Member-state law (varies) |
| **BE / CZ / DK / GR / HR / PT / SE / ES** | 13 | n/a | 13 | Member-state law (lowest permissible under GDPR Art. 8(1)) |
| **UK (UK GDPR + AADC)** | 13 | 17 | 18 | UK GDPR Art. 8; ICO AADC (15 standards) |
| **KR (Youth Protection Revision Act 2024 + PIPA Art. 22-2)** | 14 | 18 (청소년, "youth") | 19 (만 19세) | 청소년 보호법; 개인정보 보호법 §22-2; 정보통신망법 §31 |
| **JP (Act on Provision of Healthy Environment for Young People)** | 18 (青少年) | 18 | 18 (per Act) / 20 (per Civil Code historic; 18 since 2022 reform) | 青少年が安全に安心してインターネットを利用できる環境の整備等に関する法律 |
| **AU (Privacy Act + eSafety)** | 13 (Privacy Act 2024 reforms anchor) | 18 (eSafety) | 18 | Privacy Act 1988 (2024 reforms); Online Safety Act 2021 |
| **CA-can (Canada PIPEDA + Bill C-27 CPPA)** | 13 (sensitive-data anchor) | 18 | 18 (provincial age of majority varies 18-19) | PIPEDA; CPPA |
| **CA-quebec (Quebec Law 25)** | 14 | 18 | 18 | Loi 25 Art. 4.1 |
| **BR (LGPD Art. 14)** | 12 (criança) | 18 (adolescente) | 18 | LGPD Art. 14 |
| **SG (PDPA + Children's Personal Data Advisory)** | 13 | 18 | 21 (statutory) / 18 (PDPC guideline) | PDPA; PDPC Advisory 2017 (under review) |
| **IN (DPDPA 2023)** | 18 (per DPDPA §9) | n/a | 18 | DPDPA 2023 §9 |
| **AE (UAE Federal Decree-Law 45/2021)** | 21 | n/a | 21 | Federal Decree-Law 45/2021 |
| **KSA (PDPL 2023)** | 18 | n/a | 18 | KSA PDPL |
| **All other jurisdictions (default)** | 16 (EU-default-as-conservative-default) | 18 | 18 | Doctrine fallback |

Each tenant pins the doctrine version. Each user record carries
`birth_date` (encrypted at rest, residency-pinned to the user's home
cell per ADR-0009 + ADR-0010), `claimed_jurisdiction`,
`verified_jurisdiction` (after KYC where applicable),
`age_assurance_method`, `age_assurance_provider`,
`age_assurance_completed_at`, `age_assurance_re-assured_at`,
`age_of_majority_at` (derived; the date on which the user crosses
that jurisdiction's adult threshold).

Cedar context surfaces:

```cedar
// Pseudocode; canonical fragment in
//   pack/MINOR-USER-2024/cedar/01-age-context.cedar
context.user.is_minor = (
  context.now < context.user.age_of_majority_at
);
context.user.is_child = (
  age_in_years(context.user.birth_date, context.now)
    < jurisdiction_table[context.user.verified_jurisdiction].child_threshold
);
context.user.is_teen = (
  context.user.is_minor && !context.user.is_child
);
```

All policy decisions referencing `context.user.is_minor`,
`context.user.is_child`, or `context.user.is_teen` MUST emit an audit
event of class `minor_policy_decision_v1` (per D-13).

### D-2. Age assurance at signup

Define `age_assurance_method` as a tagged union of:

1. **`self_declared` (lowest assurance).** User enters birth date at
   signup. Permitted only where local law allows; defaults to refused
   in EU/UK/KR for users in the under-X bands without supplementary
   assurance.
2. **`estimation_facial` (medium assurance).** Provider performs
   facial-age-estimation (e.g., Yoti Age Estimation, AgeChecked).
   Output: estimated age in years with confidence interval. Permitted
   for "obviously adult / obviously minor" gating where the legal
   threshold lies outside the confidence band.
3. **`estimation_behavioral` (medium assurance).** Inference from
   behavioral telemetry over a probation window. Permitted only as a
   secondary signal to flag re-assurance; not permitted as primary
   age determination.
4. **`document_verified` (high assurance).** Government-ID verification
   (Yoti, Onfido, Persona, IDology, Jumio). Output: verified birth
   date.
5. **`carrier_verified` (high assurance, KR + JP).** Carrier-bound
   identity verification (KR PASS, JP MNP/carrier-account binding).
6. **`open_banking` (high assurance, UK + EU + AU).** Open Banking
   identity check (Truid, Bud, Plaid Identity Verification).
7. **`parental_account_link` (assurance derived from parent).** Minor
   account is created as a sub-account of a verified parent account
   (Apple Family Sharing model, Google Family Link model). Parent
   asserts the minor's birth date; assurance derives from the parent.
8. **`school_district_assertion` (B2B education path).** When a B2B
   education tenant onboards classes, the school district asserts the
   age band per student under a contractual agreement.

The age-assurance method required at signup is the **highest assurance
that is provider-available AND legally compliant in the user's
declared jurisdiction**, subject to D-7's per-jurisdiction provider
binding. A degraded provider does not lower the legal floor; if no
provider can satisfy the legal floor, signup is refused with an
`E_MINOR_ASSURANCE_PROVIDER_UNAVAILABLE` error.

### D-3. Parental consent flow

Define the parental-consent flow as a state machine in
`microservices/consent/` with the following terminal states:

- `consent_granted` — verifiable parental consent obtained per COPPA
  Rule §312.5(b) or equivalent in the user's jurisdiction.
- `consent_denied` — parent affirmatively denied; account is refused;
  signup wizard offers a "try again with a different parent" path
  capped at 3 attempts within 14 days.
- `consent_pending` — verification in flight; the account is locked
  with read-only access to a "parent-pending" stub UI until the
  consent state settles.
- `consent_revoked` — parent has revoked previously-granted consent;
  the µservice cascade tears down the minor account per the revocation
  workflow (delete-vs-retain branches per local law).

Permitted verification methods (per COPPA Rule §312.5(b) and
jurisdictional equivalents):

- **Knowledge-based authentication (KBA).** Parent answers
  questions from a credit-header data source.
- **Government-ID match.** Parent uploads a government ID; provider
  matches name + DOB against external data.
- **Credit-card verification.** Parent enters a payment instrument;
  USD 0.50 (or equivalent) refundable transaction is processed.
- **Video consent.** Parent joins a video call with a trust-and-safety
  agent and presents ID; the call is recorded and retained per pack's
  retention rule.
- **Signed-form upload.** Parent downloads, signs, and uploads a
  consent form; T&S manually reviews.
- **Email-plus.** "Email-plus-additional-step" (per COPPA Rule
  §312.5(b)(2)(ii)); permitted only for internal-use disclosures,
  insufficient for third-party-sharing or public-disclosure consent.
- **KR PASS-based parental verification.** Parent verifies via PASS
  for KR jurisdictions; mandatory per KR PIPA §22-2 implementation
  decree for under-14 accounts.
- **JP carrier-bound parental verification.** Parent's carrier
  account binds the minor account; the carrier asserts adulthood.

For each method, the Consent µservice records:

- `consent_method`, `consent_provider`, `consent_request_id`,
  `consent_granted_at`, `consent_granted_by` (parent's verified
  identity), `consent_granted_for` (minor's user_id),
  `consent_evidence_ref` (encrypted blob ref, retention-pinned).

The consent record is **immutable**; revocation creates a successor
record with `replaces`, never edits the prior record. This is enforced
at the storage layer (append-only outbox per ADR-0005; lineage by
`replaces` edge in the consent ledger).

### D-4. Under-age user feature restrictions

The default-restricted feature surface for minors is:

| Feature | <13 (COPPA) | 13-15 (teen, default) | 16-17 (teen, EU+UK+US-KOSA) | 18+ (adult) |
|---|---|---|---|---|
| **Targeted advertising** | REFUSED (zero personalization, zero behavioral signals) | REFUSED | REFUSED until 18 (per KOSA + DSA Article 28); contextual-only ads permitted | Permitted (subject to consent + dark-patterns refusal doctrine) |
| **Behavioral profiling / "For You"-style algorithmic feed** | REFUSED | OPT-IN ONLY; non-profiled feed default | OPT-IN ONLY; non-profiled feed default | Default ON |
| **Direct messages from non-contacts** | REFUSED | REFUSED by default; opt-in unlocks "contacts only" then "contacts of contacts" | REFUSED by default; opt-in unlocks one level at a time | Permitted (subject to T&S restrictions) |
| **Public profiles by default** | REFUSED (no public profile permissible) | Private by default; opt-in to public requires parental consent | Private by default; opt-in to public permitted but loud-default-warn | Public by default permitted |
| **In-app purchases / virtual currency** | REFUSED | REFUSED without parental approval per transaction (Ask to Buy model) | REFUSED without parental approval per transaction OR per-monthly-cap parental pre-authorization | Permitted (subject to spending-limits doctrine) |
| **Marketplace listings (sell side)** | REFUSED | REFUSED | REFUSED until 18 | Permitted |
| **Marketplace purchases (buy side)** | REFUSED | REFUSED without parental approval per transaction | REFUSED without parental approval per transaction OR pre-auth ceiling | Permitted |
| **Creator economy / payouts** | REFUSED | REFUSED | REFUSED (per US labor law minor protections + tax withholding) | Permitted |
| **Sensitive-content surfaces** (NSFW, gambling, alcohol/tobacco, dating) | REFUSED | REFUSED | REFUSED | Permitted subject to region locks |
| **Location sharing (precise)** | REFUSED | OPT-IN ONLY; expires after 24 hours by default | OPT-IN ONLY; expires after 24 hours by default | Permitted |
| **Push notifications during sleep hours** (22:00-07:00 local) | REFUSED (sleep mode ON, immutable) | OPT-IN ONLY to disable sleep mode | OPT-IN ONLY to disable sleep mode | Default OFF disabled; user toggles |
| **Live streaming** | REFUSED | REFUSED | OPT-IN ONLY with parental approval | Permitted |
| **AI features (image generation, chat, voice clone)** | REFUSED | OPT-IN with restricted prompt-space (per ADR-0144 EU AI Act tier) | OPT-IN with restricted prompt-space | Permitted |
| **External-sender warnings on Mail** | ON (immutable, loud) | ON (immutable, loud) | ON (loud, dismissible per-thread) | Default ON, dismissible globally |
| **Group invite acceptance (Messenger groups, Workspace groups)** | REFUSED for adult-group joins | REFUSED for adult-group joins; teen-only groups permitted | REFUSED for adult-group joins; mixed-group joins require parental consent | Permitted |

The matrix is canonical at `/specs/minor-user-doctrine.json`. The
policy engine enforces it as a Cedar fragment bundle in
`pack/MINOR-USER-2024/cedar/`. Each µservice consumes the bundle and
binds the UX-level enforcement.

### D-5. Per-tenant override

Tenants may override the default minor-user policy within bounds.
Overrides flow through ADR-0218's tenant-granular control surface and
are bounds-checked by the policy engine.

Permissible overrides:

- **Stricter than default.** A B2B education tenant operating in a
  K-12 district may set "no DMs at all, even between contacts" or
  "no AI features at any age." Stricter overrides are always
  permitted.
- **Looser than default within statutory floor.** A B2B health-research
  tenant operating under a separate consent regime (e.g., IRB-approved
  research consent for adolescents 13-17 with assent + parent permission)
  may unlock specific features that are otherwise default-refused —
  but only when the override is signed by a council-privacy-bound
  exception ticket and the override is recorded in the consent ledger.
- **Loosening cap.** No override may loosen below the statutory floor
  in any user's verified_jurisdiction. The policy engine enforces this
  as a deny-wins evaluation: tenant policy that conflicts with the
  jurisdiction's statutory floor is rejected at admission time, not at
  evaluation time.

The override surface is a per-µservice + per-tenant + per-feature
matrix recorded at `microservices/tenancy/` and re-evaluated on every
tenant configuration change. Audit emission is mandatory on every
override grant + revocation + bounds-check failure.

### D-6. Compliance pack `MINOR-USER-2024`

Register `MINOR-USER-2024` as the umbrella compliance pack per
ADR-0251. The pack carries:

- `pack_id`: `MINOR-USER-2024`
- `version`: `1.0.0` at publication
- `regulation`: union of COPPA, KOSA, EU GDPR Art. 8, EU DSA Art. 28,
  UK AADC, KR Youth Protection Revision Act 2024, JP Act on Provision
  of Healthy Environment for Young People, CA AADC, AU Privacy Act
  2024 reforms, Quebec Law 25, IN DPDPA §9, BR LGPD Art. 14.
- `signed_by`: oyatie compliance-office Ed25519 key
  `pack-signing/compliance-office/2026`
- `effective_at`: `2026-06-01` (post-bootstrap target)
- `cedar_fragments`: 14 fragments covering age-context, child gate,
  teen gate, parental-consent gate, targeted-ads refusal,
  behavioral-profiling refusal, public-profile refusal, marketplace
  refusal, creator-payout refusal, sensitive-content refusal,
  location-sharing refusal, sleep-mode enforcement, algorithm-
  transparency emission, age-down refusal.
- `audit_chain_requirements`: emit `minor_policy_decision_v1` on every
  policy decision touching `context.user.is_minor`; emit
  `parental_consent_v1` on consent state transitions; emit
  `age_assurance_v1` on assurance method outcomes; emit
  `age_of_majority_migration_v1` on migration workflow events.
- `data_class_extensions`: registers `MINOR_PII` data class with
  per-jurisdiction subtypes (`MINOR_PII_US`, `MINOR_PII_EU`,
  `MINOR_PII_UK`, `MINOR_PII_KR`, `MINOR_PII_JP`, `MINOR_PII_BR`, etc.).
- `cell_eligibility`: all cells with `general` certification level can
  host MINOR-USER-2024 traffic; cells with `hipaa-certified` or
  `eu-sovereign` certification levels overlay their stricter rules.
- `retention_rules`: minor PII retention defaults to "longest of
  (consent-revocation event, age-of-majority + statute-of-limitations
  window per jurisdiction)" — minimum 7 years in US, 5 years in EU,
  3 years in KR, per jurisdictional limitation periods.
- `consent_requirements`: verifiable parental consent per D-3 for any
  data collection covered by the pack.
- `cross_tenant_rules`: cross-tenant minor PII flow REFUSED unless the
  flow is part of an explicit B2B education contract with bilateral
  pack pinning.
- `jurisdiction_overlay`: per-jurisdiction overlay matrix per D-1 + D-7.
- `dpia_template`: a 14-section DPIA template covering minor-affecting
  data flows.
- `breach_notification_workflow`: minor-affecting breach workflow is
  the **fastest** path across all applicable jurisdictions — KR PIPA's
  24-hour notification, then GDPR's 72-hour, then HIPAA's 60-day.
- `regulator_evidence_cadence`: annual transparency report;
  per-incident regulator packet on FTC + UK ICO + EU DPA-of-lead-
  supervisor + KR KCC + JP METI in parallel.
- `agreement_template_refs`: Children's Online Privacy Statement
  template; Family Account Agreement template; B2B education DPA
  template; KR 부모 동의서 (parental consent form) template; UK AADC
  Children's Code self-assessment template.

### D-7. Per-jurisdiction age-verification provider

Bind providers per jurisdiction. Each tenant cell selects one
provider per band, with a designated fallback. The provider matrix
is canonical at `/specs/minor-user-doctrine.json`:

| Jurisdiction | Primary provider | Fallback provider | Tertiary provider |
|---|---|---|---|
| **US** | Persona Identity Verification | Onfido | Jumio |
| **EU/EEA** | Yoti | Onfido | iDenfy |
| **UK** | Yoti | Onfido | Veriff |
| **KR** | NICE PASS (NICE Information Service) | KCB (Korea Credit Bureau) | KFTC (open-banking-bound) |
| **JP** | NTT DOCOMO d-account binding | LINE Verify | TRUSTDOCK |
| **AU** | Yoti | Onfido | IDVerse |
| **BR** | Idwall | Onfido | Truora |
| **IN** | Aadhaar-based eKYC (UIDAI-licensed providers: HyperVerge, Signzy, Karza) | Onfido | Persona |
| **CA-can** | Onfido | Persona | Trulioo |
| **SG** | MyInfo (SingPass-bound) | Onfido | Yoti |
| **MX** | Mati (Metamap) | Veriff | Truora |
| **AE / KSA** | UAE Pass (UAE) / Absher (KSA) | IDology | Persona |

Provider contracts include a Data Processing Addendum (DPA), a Sub-
processor Agreement, and a Specific Annex on minor-data handling.
Provider attestations are renewed annually; the renewal cadence is
tracked in the audit-chain.

Provider failure or contract lapse triggers `pack_degradation_v1`
audit emission and, after a 30-day grace, refusal of new signups in
that jurisdiction (existing accounts continue under cached assurance;
re-assurance windows extend by the grace period).

### D-8. Account migration at age-of-majority

Define the age-of-majority migration workflow as a state machine in
`microservices/identity/` + `microservices/consent/`.

Trigger: `now >= user.age_of_majority_at` is evaluated daily by a
scheduled job; on transition, the workflow opens.

Workflow steps:

1. **T-30 days notice.** Send the user a notification on every surface
   (in-app banner, email to the verified parent contact, optional SMS):
   "You will become an adult on [date]. Here's what changes."
2. **T-7 days notice.** Repeat notice with detailed feature-change
   matrix. Surface a "preview adult features" preview without
   activating them.
3. **T-0 (age of majority).** Account state machine transitions
   `account.minor_status → minor_status_transitioning`.
   - The user is presented an interstitial wizard: "You're now an
     adult on this platform. Review your privacy defaults."
   - Defaults remain at the maximal-restriction setting; the wizard
     surfaces opt-in toggles for each feature.
   - Parental supervision links are preserved as **visibility-only**
     by default; the user may revoke parental supervision in the
     wizard with a confirm-and-confirm-again flow.
   - The user must provide an adult-grade identity assurance at T-0
     OR within 30 days (per D-2 method 4 or higher); failure to
     re-assure triggers account suspension at T+30 with a 90-day
     reactivation window.
4. **T+0 to T+30.** Account is in "transitioning" state; minor
   defaults still apply; the user can opt-in feature-by-feature.
   Audit emission on every opt-in.
5. **T+30.** Account state machine transitions
   `account.minor_status → adult`. Parental consent records become
   historical; new consent records for adult-only data collection
   replace them. Minor-PII retention timer starts ticking down toward
   eventual erasure per the pack's retention rule.

The migration is **irreversible** by design. If the user later
contests their age claim (e.g., admits they had falsified their
birth date during signup), the account is treated as a fresh signup
under the corrected age and the platform may pursue the false-claim
under its T&S terms.

### D-9. Default privacy: maximal-restriction for minors

UK AADC's 15 standards and KOSA's "duty of care" both center on a
single principle: **the strictest privacy setting is the default for
minors, and any loosening is parent-mediated.**

Apply the principle uniformly across the µservice surface:

- **Profile visibility:** private by default; followers approve;
  no search-engine indexing.
- **Friend/contact discovery:** disabled by default; opt-in via
  parent-mediated approval.
- **Geolocation:** disabled by default; precise location never
  collected for minors; coarse location only with documented purpose.
- **Behavioral tracking:** zero behavioral-profiling cookies / SDKs /
  trackers for the minor surface; contextual targeting only;
  third-party trackers BLOCKED at the edge for minor-flagged
  sessions.
- **Algorithmic feed:** non-profiled chronological feed is the
  default; "For You"-style feed is OPT-IN with parental approval for
  under-16.
- **Search history retention:** ephemeral by default (24-hour rolling
  window); permanent history is opt-in.
- **AI features:** disabled by default; OPT-IN with restricted prompt
  space (per ADR-0144) and content-rating gates.
- **Communication:** DMs from non-contacts disabled; group invites
  require manual approval; nudity-detection on incoming images ON
  with no off-switch.
- **Ads:** non-personalized, contextual-only; per-session frequency
  cap; no retargeting; no cross-app tracking; no use of minor-PII
  for ad-lookalike modeling.

Defaults are restored on every minor-affecting update (the "loud
default" pattern per UK AADC Standard 7). The user is notified when
a default has been restored; the parent is notified if the user is
under 13.

### D-10. Age-down protection

The age-down attack: a user signs up at age 12 (correctly classified
as a child); the user later edits their birth date to age 18 to
unlock adult features. The platform MUST refuse silent age-down
edits.

Enforce the following protocol:

- **The `birth_date` field is write-once at signup.** Subsequent
  edit attempts route through a dedicated `age_correction_request`
  workflow.
- **Direction matters.** An edit that **decreases** the user's age
  (e.g., from 18 to 12) routes to T&S for manual review (it might be
  a fraud signal or a corrected misrepresentation).
- **An edit that increases the user's age** (e.g., from 12 to 18)
  REQUIRES a high-assurance identity verification per D-2 method 4 or
  higher AND a cooling-off period of 14 days AND notification to the
  account's parental-consent contact.
- **A cross-jurisdiction edit** (changing `verified_jurisdiction`)
  is treated as a re-signup: the user re-attests, the new
  jurisdiction's age threshold applies, and the user may end up
  re-classified as a minor under the new jurisdiction even if they
  were an adult under the old one (e.g., a 19-year-old US user moving
  to UAE jurisdiction where adulthood is 21 reverts to teen status).
- **Audit emission** on every age-correction request, regardless of
  outcome.

The platform's UX surfaces the policy clearly during the edit
attempt: "Changing your birth date can affect what features are
available. Some changes require additional verification."

### D-11. Algorithm transparency for minors

Per KOSA's "transparency requirements" and EU DSA Article 27 +
Article 28, the platform owes minors (and their parents) explicit
transparency about algorithmic curation.

Requirements:

- **Disclosure surface.** Every minor account has access to an
  "Algorithm Settings" surface that lists every algorithmic curation
  decision that affects them: feed ranking, recommendations, ads
  selection, notification timing, content moderation outcomes.
- **Non-profiled alternative.** For every algorithmic feed surface,
  the minor account has a one-click toggle to "Show chronologically"
  (or equivalent non-profiled alternative). The toggle persists
  across sessions and is the default for under-16.
- **Explanation per recommendation.** Every recommendation surface
  carries a "Why am I seeing this?" affordance. For minors, the
  explanation is presented in plain-language (per UK AADC Standard 4
  "Transparency") at the appropriate reading level.
- **Opt-out from training.** Minor-account interaction data is by
  default **NOT** used for model training (per ADR-0144 + EU AI
  Act). The parent may opt-in for under-16; the user may opt-in for
  16-17. Audit emission on opt-in / opt-out.
- **Risk-rating disclosure.** Per DSA Article 35, the systemic risks
  identified in the platform's annual risk-assessment-for-minors
  report are surfaced in plain-language to minor users.
- **Per-recommendation provenance.** When the platform serves a
  recommended item to a minor, the audit chain records:
  `(user_id, item_id, model_id, ranker_score, features_used,
   timestamp)`. On parent-initiated DSAR, the platform produces
  this lineage.

### D-12. Marketplace + payments restrictions

Per ADR-0249 (multi-category marketplace) plus the payments and refunds
doctrine, the marketplace surface for minors is constrained:

- **No marketplace listings (sell side).** Minors cannot list goods
  or services for sale. This protects against minor-labor exploitation
  and tax-withholding violations.
- **No marketplace purchases without parental approval.** Every
  purchase by a minor account routes through the parental-approval
  workflow (the "Ask to Buy" model). The parent receives a push
  notification + email; the parent approves or denies; the platform
  holds funds in escrow until resolution; default expiry is 48 hours.
- **Pre-authorization ceiling option.** The parent may set a monthly
  ceiling (e.g., USD 25 / EUR 25 / KRW 30,000) under which
  purchases auto-approve. The ceiling is recorded with the consent;
  changes are audit-emitted.
- **No virtual-currency purchases beyond per-session cap.** Minors
  cannot purchase platform virtual currency in transactions exceeding
  USD 10 per session and USD 50 per month without parental approval
  per transaction.
- **No subscriptions auto-renewal without per-renewal approval.**
  Subscription products billed to a minor account require parental
  approval at every renewal point (per EU consumer-protection law +
  UK CMA dark-patterns enforcement).
- **No gambling / sweepstakes / loot-box mechanics.** Per UK
  Gambling Commission interpretation and EU consumer-protection,
  loot-box mechanics targeting minors are REFUSED.
- **Mandatory price-transparency.** All prices to minors are
  displayed inclusive of taxes and fees in the local currency, with
  the parental-approval flow's USD-equivalent shown for
  cross-border purchases.
- **Refund-friendly defaults.** Per CMA + FTC + EU consumer-rights
  jurisprudence on minor-initiated transactions, refund requests
  from a parent on behalf of a minor are processed with a default
  approval bias (subject to T&S anti-abuse heuristics).

### D-13. Audit-chain emission for every minor-affecting policy decision

Every policy decision whose Cedar evaluation touches
`context.user.is_minor`, `context.user.is_child`,
`context.user.is_teen`, or any minor-PII data class MUST emit an
audit event of class `minor_policy_decision_v1`.

Event schema (canonical at `/specs/audit-events/minor-policy-decision-v1.json`):

```json
{
  "event_class": "minor_policy_decision_v1",
  "event_id": "<ulid>",
  "tenant_id": "<tenant>",
  "user_id_hash": "<hmac-pseudonymous>",
  "verified_jurisdiction": "<iso-3166-1-alpha-2-or-supranational>",
  "age_band": "<child|teen|adult>",
  "decision_principal": "<service-principal>",
  "decision_resource": "<resource-id>",
  "decision_action": "<action>",
  "decision_outcome": "<permit|deny>",
  "policy_pack_id": "MINOR-USER-2024",
  "policy_pack_version": "<semver>",
  "cedar_fragment_id": "<fragment-id>",
  "cedar_principals_matched": [...],
  "cedar_resources_matched": [...],
  "tenant_override_applied": "<override-id-or-null>",
  "request_id": "<request-id>",
  "trace_id": "<otel-trace-id>",
  "occurred_at": "<rfc3339-utc>",
  "cell_id": "<cell-id>",
  "model_lineage_ref": "<if-algorithm-decision-then-model-id-else-null>"
}
```

Storage:

- Hot store: real-time outbox per ADR-0005, replicated to the
  audit-chain substrate per ADR-0003 + ADR-0246.
- Cold store: retention per pack's `retention_rules`. For minor-
  affecting events, retention is **age-of-majority + statute-of-
  limitations of the operative jurisdiction**, never less than 7
  years.
- Query surface: regulator-facing query API at `microservices/audit-
  chain/`, gated by Cedar policy that authorizes regulators (per
  ADR-0243) without exposing other tenants' data.

### D-14. Per-µservice minor-user-aware UX

Each µservice in the B2C bill of materials MUST bind a minor-user-
aware UX contract enforced at the gateway + at the UI layer.
Canonical bindings:

- **`microservices/identity/`**: at signup, route by jurisdiction to
  the appropriate age-assurance provider; refuse signup if no
  provider is bound; bind `birth_date` write-once; surface the
  age-of-majority migration wizard at T-30/T-7/T-0/T+30.
- **`microservices/consent/`**: hold the verifiable-parental-consent
  state machine; expose parent-facing dashboards for active consents
  + revocation + audit history.
- **`microservices/messenger/`**: enforce minor DM defaults (no DMs
  from non-contacts; group invite requires manual approval; nudity-
  detection ON; "external sender" warning on cross-tenant DMs);
  enforce sleep-mode 22:00-07:00; loud-default-warn on parental
  loosening.
- **`microservices/mail/`**: enforce minor mail defaults (external-
  sender warnings immutable; phishing detection ON; attachment
  scanning ON; sensitive-attachment refusal ON); disable bulk mail
  send.
- **`microservices/recordings/` (calls / meetings)**: enforce minor
  recording defaults (calls with adults outside contacts REFUSED;
  recording disabled when minor is on the call without parental
  approval).
- **`microservices/feed/` (algorithmic feed)**: enforce
  chronological-feed default; surface the "Why am I seeing this?"
  affordance; emit `minor_policy_decision_v1` on every rank decision.
- **`microservices/search/`**: enforce SafeSearch ON immutable for
  under-13; ON dismissible-per-session for teen; minor search-history
  ephemeral by default.
- **`microservices/marketplace/`**: enforce minor purchase + listing
  refusal per D-12; route purchases through parental-approval
  workflow.
- **`microservices/payments/`**: enforce minor payment refusal
  except via parental-approval workflow; refund-friendly defaults.
- **`microservices/ads/`**: refuse targeted ads to minors; serve
  contextual-only; per-session frequency cap; emit per-impression
  `minor_policy_decision_v1`.
- **`microservices/recommendations/`**: refuse behavioral-profiling
  for under-16; surface "Why?" affordance; non-profiled alternative.
- **`microservices/safety/`**: provide minor-specific reporting flow;
  expedited T&S review SLA for minor-affecting reports; coordinate
  with NCMEC (US), IWF (UK), CEOP (UK), KOC (KR) per jurisdiction.
- **`microservices/ai-substrate/`**: enforce minor AI defaults
  (disabled; restricted prompt-space; no minor data in training per
  D-11); emit per-inference `minor_policy_decision_v1` when minor
  context.
- **`microservices/calendar/`**: refuse minor cross-tenant invites
  without manual approval; enforce "do not show free/busy to
  non-contacts" default.
- **`microservices/storage/` (cloud-storage)**: enforce minor shares
  default to "private link"; revoke on parental request; SafeSearch
  on previews.
- **`microservices/observability/`**: per-µservice SLO for
  `minor_policy_decision_v1` emission completeness ≥ 99.99%; alert
  on shortfall.

Each binding is an enforcement contract recorded at
`microservices/<svc>/policies/minor-user-binding.yaml` and validated
at CI by `cloud-ci/Rust gate packet per-microservice-minor-ux-binding`.

## Alternatives Considered

### Alternative A: ship B2C without minor-user doctrine and "deal with it later"

**Rejected.** This is the option modeled by several pre-2018 startups
that subsequently paid USD 5-275M in settlements (TikTok, YouTube,
Epic Games, Microsoft Xbox, Amazon Alexa, Meta). The per-incident
cost dramatically exceeds the up-front substrate cost. The platform's
multi-jurisdiction footprint compounds the exposure: a single
mishandled minor under simultaneous FTC + UK ICO + EU DPA + KR KCC
+ JP METI scrutiny exceeds USD 1M before reputational damage.

### Alternative B: ship B2C with a single global "minor = under-18" threshold

**Rejected.** Over-restriction breaks B2B education tenants (K-12
districts onboarding 13-year-olds with school-district assertion).
Over-restriction also leaks legal jeopardy in KR (PIPA's threshold
is 14, not 18; collecting "minor-status" data on 14-17-year-olds
without parental consent under a self-declared "minor = under-18"
banner exceeds the regulator's expectation and may trigger DPA
review). Per-jurisdiction thresholds are statutorily required and
must be enforced.

### Alternative C: outsource age assurance entirely to a single global provider

**Rejected.** No single provider covers all jurisdictions
satisfactorily. KR mandates PASS-based verification; JP requires
carrier-bound or LINE-bound identity; IN requires Aadhaar-bound
eKYC; AE requires UAE Pass. A single global provider would either
fail to support these or impose unacceptable single-point-of-failure
risk. The doctrine binds a per-jurisdiction primary + fallback
+ tertiary, with degradation paths.

### Alternative D: rely on each µservice to implement minor handling individually

**Rejected.** This is the pre-doctrine state. The cost is super-linear
in µservices × jurisdictions × features. Definition drift, consent
fragmentation, and audit incoherence cumulatively make regulator
response impossible at scale. The doctrine centralizes the substrate
and binds µservices to a single canonical policy bundle.

### Alternative E: refuse to operate where minors are present (adults-only platform)

**Rejected.** This is the OnlyFans / Pornhub model. Even strict
adults-only platforms must operate minor-user doctrine for the false-
claim case (a minor falsifies their birth date); the doctrine is not
optional even for adults-only positioning. Moreover, the platform's
multi-product surface includes products (Messenger, Calendar, Mail,
Workspace) where minor-user is an explicit B2C use case.

### Alternative F: build minor-user doctrine as a Tier-2 hardening (post-launch retrofit)

**Rejected.** The cost of retrofit is dramatic. Every µservice would
need to re-instrument for minor-aware logging, re-design defaults to
restore them on minor-flag flip, and re-bind to the consent ledger.
Worse, the retrofit window between B2C launch and doctrine completion
is the highest-exposure window: it's exactly when a single
mishandled minor triggers a settlement that funds the prosecutor's
office for the next decade. Retrofit is rejected.

### Alternative G: rely on platform-level age estimation only (no document verification)

**Rejected.** Age estimation has documented confidence-band issues
(±2-3 years at the 13-18 boundary, the most regulatorily-sensitive
range). Regulators (UK ICO, EU DPA, FTC) have explicitly stated that
estimation-only is insufficient where the legal threshold lies within
the confidence band. Document verification is required for the
highest-stakes assurance.

### Alternative H: implement minor-user doctrine but skip the per-µservice UX bindings (D-14)

**Rejected.** The Cedar bundle without UX binding produces inconsistent
user-facing behavior: the policy might deny a feature but the UI
still surfaces it. UI surfacing a feature that the policy denies is
itself a deceptive-design (dark-pattern) violation under FTC Section
5 + EU UCPD + UK CMA enforcement. UX bindings are non-optional.

## Consequences

### Positive

1. **Single source of truth.** Minor-user policy is bundled, signed,
   versioned, and consumed uniformly across every µservice. Definition
   drift is eliminated.
2. **Tenant-admission gate.** B2C tenants cannot be admitted without
   the doctrine pinned; the platform refuses pre-doctrine B2C traffic.
3. **Regulator-ready evidence.** Audit-chain emission on every minor-
   affecting decision produces a queryable evidence corpus for FTC +
   UK ICO + EU DPA + KR KCC + JP METI inspections.
4. **Per-jurisdiction provider binding.** Each jurisdiction gets the
   regulator-acceptable assurance provider; degradation paths exist
   for provider failure.
5. **Age-down protection.** A user cannot silently edit themselves
   from 12 to 18 to unlock adult features.
6. **Age-of-majority migration.** Users transition smoothly without
   manual intervention; defaults remain restrictive at the transition
   moment and opt-in surfaces are presented.
7. **Algorithm transparency.** Minor users + parents have explicit
   visibility into algorithmic curation; the "Why am I seeing this?"
   affordance is mandatory.
8. **Marketplace + payments safe defaults.** Minors cannot incur
   purchase liability without parental approval; refund-friendly
   defaults reduce parental dispute volume.
9. **B2B education path preserved.** Stricter overrides + school-
   district assertion accommodate K-12 tenants without weakening the
   B2C floor.
10. **Multi-product cohesion.** Identity + Consent + Messenger + Mail
    + Marketplace + Payments + Ads + Recommendations + AI substrate
    + Calendar + Storage all consume the same policy bundle.

### Negative

1. **Signup friction.** Per-jurisdiction age assurance adds steps to
   signup (esp. document verification for under-X bands). Expected
   conversion drop in the 13-17 band: 8-18% across the cohort.
2. **Provider cost.** Per-verification fees range USD 0.50-3.00 per
   verification; aggregate annual cost at 1M B2C signups is USD
   500K-3M. Per-jurisdiction provider contracts add legal + ops
   overhead.
3. **Operational headcount.** Trust-and-safety operational headcount
   grows: parental-consent verification (video-consent path); manual
   age-correction review; parent-initiated DSAR + revocation
   processing. Expected initial headcount: 6-12 FTE depending on B2C
   volume.
4. **Migration workflow complexity.** Age-of-majority migration is a
   substantive engineering effort: a state machine per user, an
   interstitial wizard, T-30/T-7/T-0/T+30 notifications across
   channels, an opt-in flow for each newly-available feature.
5. **B2C UX surface expansion.** Every minor-affecting feature needs
   a "Why?" affordance + non-profiled alternative + parental-approval
   path. The B2C UX surface area roughly doubles.
6. **Audit volume.** `minor_policy_decision_v1` emission on every
   minor-affecting decision can generate 100-1000x the audit volume
   of a non-minor decision (ads serve every page view; feed ranks
   on every refresh). Audit-chain storage cost grows materially.
7. **Cross-product policy alignment.** Maintaining the canonical
   policy across 15+ µservices requires per-µservice ownership and a
   cadence of policy-engine review. Drift mitigation through CI
   validators (D-14) is necessary.
8. **Regulator-response cadence.** Annual transparency report + per-
   incident regulator packet impose ongoing legal-team load.
9. **Provider-lock risk.** Per-jurisdiction provider binding creates
   dependency on the provider's continued availability + price
   stability. Mitigation: primary + fallback + tertiary per
   jurisdiction.
10. **Tenant onboarding friction.** B2C tenants experience admission-
    gate refusal until the doctrine is pinned; tenant-side education
    is required to set expectations.

### Neutral / informational

1. The doctrine intentionally does not extend to non-B2C surfaces
   (substrate-only tenants like `oyatie-cloud-internal` do not consume
   the doctrine). The doctrine activates only on B2C-flagged tenants.
2. The doctrine version (`1.0.0` at publication) is expected to
   increment frequently in the first 24 months as jurisdictions
   publish enforcement guidance. Pack-version bumps follow ADR-0251's
   pack-versioning rules.
3. The doctrine composes with ADR-0144 (EU AI Act tiers); minor-user
   + AI-substrate interactions are bounded by the more-restrictive of
   the two regimes.
4. The doctrine composes with ADR-0240 (sovereign-cloud) per pack
   overlay; minor PII in KR routes through KR sovereign cells; minor
   PII in EU routes through EU sovereign cells; etc.
5. The doctrine composes with ADR-0241 (DR / BC) per pack overlay;
   minor-PII recovery objectives are bound to the strictest
   jurisdiction's minimum.

## Implementation Surface

### Per-microservice changes

The following µservices receive new code paths:

- `microservices/identity/` — new `birth_date` field (write-once);
  new `age_assurance_*` fields; signup flow routing to provider;
  age-of-majority migration workflow.
- `microservices/consent/` — new `verifiable_parental_consent`
  state machine; new consent-method handlers (KBA, government-ID,
  credit-card, video, signed-form, PASS, carrier); revocation cascade.
- `microservices/age-assurance/` — new µservice; provider adapters
  (Yoti, Onfido, Persona, PASS, etc.); jurisdiction-routing logic;
  fallback/degradation handling.
- `microservices/policy-engine/` — load `pack/MINOR-USER-2024/`
  Cedar fragments; ingest jurisdiction matrix as context; emit
  `minor_policy_decision_v1` on every minor-touched evaluation.
- `microservices/audit-chain/` — new event class
  `minor_policy_decision_v1` registered; storage retention pinned
  per pack rule; regulator-query API surface.
- `microservices/tenancy/` — tenant-admission gate enforces pack
  pinning; tenant-override surface enforces bounds-check.
- `microservices/messenger/`, `microservices/mail/`,
  `microservices/recordings/`, `microservices/feed/`,
  `microservices/search/`, `microservices/marketplace/`,
  `microservices/payments/`, `microservices/ads/`,
  `microservices/recommendations/`, `microservices/safety/`,
  `microservices/ai-substrate/`, `microservices/calendar/`,
  `microservices/storage/` — bind per-µservice minor-user UX contracts
  per §D-14.
- `microservices/observability/` — SLO per `minor_policy_decision_v1`
  emission completeness; alerting on shortfall.

### New crates

Following the canonical-base + per-pack-overlay pattern per ADR-0064:

- `crates/oya-minor-user-doctrine-domain/` — domain model (Age band,
  Jurisdiction, Provider, Consent state, Migration state, Override).
- `crates/oya-minor-user-doctrine-kernel/` — pure business logic
  (age-band classification, jurisdiction routing, consent state
  transitions, age-down detection).
- `crates/oya-minor-user-doctrine-port/` — port traits (AgeAssurance,
  ParentalConsent, AuditEmitter, TenantOverrideEval).
- `crates/oya-minor-user-doctrine-adapter-yoti/` — Yoti provider
  adapter.
- `crates/oya-minor-user-doctrine-adapter-onfido/` — Onfido adapter.
- `crates/oya-minor-user-doctrine-adapter-persona/` — Persona adapter.
- `crates/oya-minor-user-doctrine-adapter-nicepass/` — KR NICE PASS
  adapter.
- `crates/oya-minor-user-doctrine-adapter-kcb/` — KR KCB adapter.
- `crates/oya-minor-user-doctrine-adapter-aadhaar/` — IN Aadhaar
  adapter (UIDAI-licensed providers).
- `crates/oya-minor-user-doctrine-adapter-uaepass/` — UAE Pass
  adapter.
- `crates/oya-minor-user-doctrine-adapter-mati/` — BR Mati adapter.
- `crates/oya-minor-user-doctrine-app/` — application orchestration
  (signup flow, parental-consent workflow, migration workflow).
- `crates/oya-minor-user-doctrine-api/` — gateway-facing API surface.
- `crates/oya-minor-user-doctrine-localization-kr/` — KR pack
  overlay (PIPA + Youth Protection Revision Act 2024 specifics).
- `crates/oya-minor-user-doctrine-localization-eu/` — EU pack
  overlay (GDPR Art. 8 + DSA Art. 28).
- `crates/oya-minor-user-doctrine-localization-uk/` — UK pack
  overlay (AADC 15 standards).
- `crates/oya-minor-user-doctrine-localization-jp/` — JP pack
  overlay (Act on Provision of Healthy Environment for Young
  People).
- `crates/oya-minor-user-doctrine-localization-au/` — AU pack
  overlay (Privacy Act 2024 reforms + eSafety).
- `crates/oya-minor-user-doctrine-localization-br/` — BR pack
  overlay (LGPD Art. 14).
- `crates/oya-minor-user-doctrine-localization-in/` — IN pack
  overlay (DPDPA §9).
- `crates/oya-minor-user-doctrine-localization-ca/` — CA-can pack
  overlay (PIPEDA + Quebec Law 25).

### New specs

- `/specs/minor-user-doctrine.json` — age threshold matrix +
  provider matrix + feature restriction matrix.
- `/specs/microservices/age-assurance.json` — age-assurance
  µservice spec.
- `/specs/audit-events/minor-policy-decision-v1.json` — audit event
  schema.
- `/specs/audit-events/parental-consent-v1.json` — consent event
  schema.
- `/specs/audit-events/age-assurance-v1.json` — assurance event
  schema.
- `/specs/audit-events/age-of-majority-migration-v1.json` — migration
  event schema.
- `pack/MINOR-USER-2024/manifest.json` — pack manifest per
  ADR-0251.
- `pack/MINOR-USER-2024/cedar/01-age-context.cedar` — age context
  fragment.
- `pack/MINOR-USER-2024/cedar/02-child-gate.cedar`
- `pack/MINOR-USER-2024/cedar/03-teen-gate.cedar`
- `pack/MINOR-USER-2024/cedar/04-parental-consent-gate.cedar`
- `pack/MINOR-USER-2024/cedar/05-targeted-ads-refusal.cedar`
- `pack/MINOR-USER-2024/cedar/06-behavioral-profiling-refusal.cedar`
- `pack/MINOR-USER-2024/cedar/07-public-profile-refusal.cedar`
- `pack/MINOR-USER-2024/cedar/08-marketplace-refusal.cedar`
- `pack/MINOR-USER-2024/cedar/09-creator-payout-refusal.cedar`
- `pack/MINOR-USER-2024/cedar/10-sensitive-content-refusal.cedar`
- `pack/MINOR-USER-2024/cedar/11-location-sharing-refusal.cedar`
- `pack/MINOR-USER-2024/cedar/12-sleep-mode-enforcement.cedar`
- `pack/MINOR-USER-2024/cedar/13-algorithm-transparency-emission.cedar`
- `pack/MINOR-USER-2024/cedar/14-age-down-refusal.cedar`
- `pack/MINOR-USER-2024/dpia/template.md`
- `pack/MINOR-USER-2024/breach-workflow/notification-template.md`
- `pack/MINOR-USER-2024/agreements/childrens-online-privacy-statement.md`
- `pack/MINOR-USER-2024/agreements/family-account-agreement.md`
- `pack/MINOR-USER-2024/agreements/b2b-education-dpa.md`

### New CI lanes

- `cloud-ci/Rust gate packet minor-user-pack-pinning` — verifies B2C tenants
  pin `MINOR-USER-2024@<version>`.
- `cloud-ci/Rust gate packet age-threshold-per-jurisdiction-coherence` —
  verifies the jurisdiction matrix is well-formed and matches the
  Cedar fragment context.
- `cloud-ci/Rust gate packet age-assurance-provider-binding` — verifies each
  jurisdiction has primary + fallback + tertiary providers bound and
  the contracts are current.
- `cloud-ci/Rust gate packet parental-consent-workflow-coverage` — verifies
  the consent state machine handles all transitions per D-3.
- `cloud-ci/Rust gate packet minor-default-privacy-maximal` — verifies the
  default-restriction matrix per D-4 is enforced at each µservice.
- `cloud-ci/Rust gate packet age-down-edit-refusal` — verifies the age-down
  refusal logic per D-10 is enforced at identity.
- `cloud-ci/Rust gate packet age-of-majority-migration-flow` — verifies the
  migration workflow per D-8 is end-to-end traversable.
- `cloud-ci/Rust gate packet algorithm-transparency-for-minors` — verifies
  the "Why am I seeing this?" + non-profiled alternative are bound
  per recommendation surface.
- `cloud-ci/Rust gate packet marketplace-minor-purchase-gate` — verifies
  marketplace purchases by minors route through the parental-approval
  workflow per D-12.
- `cloud-ci/Rust gate packet per-microservice-minor-ux-binding` — verifies
  each µservice in the B2C BOM has a `minor-user-binding.yaml`.
- `cloud-ci/Rust gate packet audit-emission-on-minor-policy-decision` —
  verifies the emission completeness per D-13 SLO.
- `cloud-ci/Rust gate packet tenant-override-bounds-check` — verifies tenant
  overrides do not violate the statutory floor.

### Migration of existing surfaces

Existing surfaces with B2C exposure (none yet at 2026-05-20, per
the pre-B2C posture) will be onboarded to the doctrine before any
B2C tenant is admitted. The onboarding sequence per µservice:

1. Author the per-µservice `minor-user-binding.yaml`.
2. Plumb the Cedar context through to the µservice gateway.
3. Bind audit emission to `minor_policy_decision_v1`.
4. Add the µservice to the B2C BOM in
   `/specs/platform-architecture.json`.
5. Pass `cloud-ci/Rust gate packet per-microservice-minor-ux-binding`.
6. Pass an end-to-end T&S drill: a synthetic 12-year-old account
   signs up; the consent flow runs; the migration workflow runs at
   the synthetic age-of-majority timestamp; the audit chain is
   queried as a regulator-simulated request.

### Rollout sequencing

- **Phase 1 (2026-Q3): doctrine + pack publication.** Publish
  MINOR-USER-2024 v1.0.0. Author per-jurisdiction provider contracts
  (KR PASS, Yoti EU+UK, Persona US, Onfido fallback). Author the
  age-assurance µservice in dev. CI lanes emit findings as advisory.
- **Phase 2 (2026-Q4): consent + migration in dev.** Implement
  verifiable-parental-consent workflow. Implement age-of-majority
  migration workflow. T&S team drills end-to-end.
- **Phase 3 (2027-Q1): per-µservice UX bindings.** Each µservice in
  the B2C BOM authors and lands its `minor-user-binding.yaml`.
- **Phase 4 (2027-Q2): bootstrap items 1-6 complete; CI lanes
  promote to BLOCKER.** First B2C tenant admission becomes possible
  with full doctrine enforcement.
- **Phase 5 (2027-Q3): first B2C tenant onboarding.** A bounded pilot
  tenant onboards with explicit T&S co-monitoring.

## Verification

### Doctrine-level

- **V-1:** `cloud-ci/Rust gate packet minor-user-pack-pinning` returns clean
  on every B2C tenant.
- **V-2:** `/specs/minor-user-doctrine.json` is well-formed JSON,
  schema-validated, and includes a row per jurisdiction in the matrix.
- **V-3:** `pack/MINOR-USER-2024/manifest.json` is signed by the
  oyatie compliance-office Ed25519 key; cosign verification passes.
- **V-4:** All 14 Cedar fragments are present and Cedar-validated.

### Per-jurisdiction

- **V-5:** For each jurisdiction in §D-1, at least one age-assurance
  provider is bound and the contract is current (per `oya gate
  validate age-assurance-provider-binding`).
- **V-6:** The Cedar context attribute `verified_jurisdiction`
  receives the correct value for synthetic test users in each
  jurisdiction.
- **V-7:** The per-jurisdiction child/teen/adult thresholds in the
  matrix match the legal authorities cited in the references.

### Per-µservice

- **V-8:** Every µservice in the B2C BOM has a
  `minor-user-binding.yaml` (per `cloud-ci/Rust gate packet per-microservice-
  minor-ux-binding`).
- **V-9:** Each binding declares the per-feature minor defaults per
  §D-4 and the audit-emission classes the µservice will produce.
- **V-10:** End-to-end synthetic test: a 12-year-old account in each
  jurisdiction signs up, hits the parental-consent workflow, and
  receives the correct default-restricted feature set.

### Audit-chain

- **V-11:** `minor_policy_decision_v1` emission completeness ≥ 99.99%
  per the SLO (per `microservices/observability/`).
- **V-12:** Per-request lineage from gateway → policy-engine → audit-
  chain is preserved by `trace_id`.
- **V-13:** Regulator-query API returns events matching `user_id_hash
  + date_range` with provenance lineage.

### Migration

- **V-14:** T-30 / T-7 / T-0 / T+30 notifications fire for synthetic
  test users at the age-of-majority transition.
- **V-15:** Default privacy settings remain at maximal-restriction
  at T-0; opt-in is required for each feature loosening.
- **V-16:** Age-down attempt (synthetic 18-to-12 edit) is REFUSED;
  audit emission records the refusal.
- **V-17:** Age-up attempt (synthetic 12-to-18 edit) requires high-
  assurance identity + cooling-off + parental notification per §D-10.

### Tenant override

- **V-18:** Stricter override (B2B education tenant) is ACCEPTED;
  audit emission records the acceptance.
- **V-19:** Looser override that crosses the statutory floor is
  REJECTED at the admission gate; audit emission records the rejection.

### Algorithm transparency

- **V-20:** Every recommendation surface for minors carries the
  "Why am I seeing this?" affordance.
- **V-21:** Every algorithmic feed for under-16 defaults to
  chronological; the toggle persists across sessions.
- **V-22:** Per-recommendation provenance is queryable via DSAR.

### Marketplace + payments

- **V-23:** Synthetic minor account cannot list a product; the
  attempt is REFUSED with `E_MINOR_LISTING_REFUSED`.
- **V-24:** Synthetic minor account purchase routes through
  parental-approval workflow; default 48-hour expiry holds.
- **V-25:** Parental refund request on minor-initiated transaction
  is approved within the default-bias window.

### Pack composition

- **V-26:** Cells with `general` certification level can host
  MINOR-USER-2024 traffic; cells without cannot.
- **V-27:** Cross-tenant minor PII flow is REFUSED unless explicit
  bilateral pack pinning + B2B education contract.

### Drill

- **V-28:** Annual T&S minor-user drill: simulated multi-jurisdiction
  regulator inquiry (FTC + UK ICO + EU DPA + KR KCC) is answered
  within the drilled SLA using audit-chain query.

## References

### COPPA (US, under-13)

- **15 USC §6501-6506.** Children's Online Privacy Protection Act,
  Public Law 105-277 (1998). Codified at 15 USC §§6501-6506.
- **16 CFR Part 312.** FTC's Children's Online Privacy Protection Rule.
- **2024 amendments to the COPPA Rule.** FTC Notice of Proposed
  Rulemaking 2024-01-11; final rule pending as of 2026-05-20;
  inflation-adjusted civil penalty raised to USD 50,120 per
  violation per child.
- **FTC enforcement history (selected):** YouTube USD 170M (2019);
  TikTok / Musical.ly USD 5.7M (2019); Epic Games USD 275M (2022,
  COPPA + dark-patterns); Microsoft Xbox USD 20M (2023); Amazon
  Alexa USD 25M (2023).

### KOSA (US 2024, under-17)

- **S.1409 — Kids Online Safety Act (118th Congress).** Senate-passed
  91-3 on 2024-07-30. Companion bill H.R.7891 (KOSPA).
- **Duty of care** for design choices reasonably likely to cause harm
  to minors under 17 (anxiety, depression, eating disorders,
  substance abuse, suicidal behavior, sexual exploitation, online
  bullying, predatory marketing).
- **Required disclosures:** annual transparency report, independent
  audit, NIST-coordinated systemic-risk assessment.

### EU age verification (2024-2025 enforcement)

- **Reg. (EU) 2016/679 (GDPR) Article 8.** Conditions applicable to
  child's consent in relation to information society services.
- **EDPB Guidelines 02/2024 on consent of children.** Published
  2024-Q2; codifies the "informed and freely given" + verifiable-
  parental-consent expectations.
- **Reg. (EU) 2022/2065 (DSA) Article 28.** Protection of minors
  online; Very Large Online Platforms (VLOPs) must take appropriate
  and proportionate measures.
- **Reg. (EU) 2022/2065 (DSA) Article 35.** Mitigation of risks
  identified in risk-assessment for minors.
- **EU Commission DSA Article 28 enforcement actions:** Meta, TikTok,
  X, AliExpress (2024-2025).
- **EU age-verification pilot.** Commission + Member-State pilot
  launched 2024; first-phase deliverables 2025.

### UK Age Appropriate Design Code (AADC)

- **UK ICO Children's Code (Age Appropriate Design Code).** Statutory
  code under Section 123 of the Data Protection Act 2018. Effective
  2021-09-02.
- **15 standards.** Best interests of the child; data protection
  impact assessments; age-appropriate application; transparency;
  detrimental use of data; policies and community standards;
  default settings; data minimisation; data sharing; geolocation;
  parental controls; profiling; nudge techniques; connected toys
  and devices; online tools.
- **UK ICO enforcement history:** TikTok GBP 12.7M (2023);
  Imgur + Discord + Snap under investigation (2024-2025).

### KR Youth Protection Revision Act 2024

- **청소년 보호법** (Youth Protection Act), 1997, latest revision 2024.
- **셧다운제 (shutdown) repealed 2022** but core youth-protection
  obligations preserved.
- **PIPA §22-2** (Article 22-2 of the Personal Information Protection
  Act) — special provisions for personal information of children
  under 14; mandatory parental consent.
- **정보통신망법 §31** (Act on Promotion of Information and
  Communications Network Utilization) — youth protection obligations
  on information communication service providers.
- **KCC + MOGEF + KISA + Privacy Commission of Korea** — coordinated
  enforcement.
- **PASS** — mobile-carrier-bound identity service operated by KCB +
  NICE + SCI; mandatory for age-verification on most KR platforms.

### JP Act on Provision of Healthy Environment for Young People

- **青少年が安全に安心してインターネットを利用できる環境の整備等に関する法律** (Act
  No. 79 of 2008), as amended 2024.
- **Filtering obligation** on ISPs + smartphone manufacturers + app
  stores serving minors.
- **Internet user policy** obligation on platforms with substantial
  JP minor users.
- **METI + Cabinet Office** — coordinated enforcement.
- **2022 Civil Code amendment** lowered Japanese age of majority
  from 20 to 18 (effective 2022-04-01).

### Other authorities

- **CA AADC** — SB 1394 (2024); AB 1949 (2024); AB 2273 (2022).
- **AU Privacy Act 1988** + **Privacy Act Reform Act 2024** + **Online
  Safety Act 2021** + eSafety Commissioner age-assurance trial 2024-
  2025.
- **CA-can PIPEDA** + **Bill C-27 (CPPA)** 2024.
- **Quebec Law 25** — Loi modernisant des dispositions législatives en
  matière de protection des renseignements personnels, 2021-Q3,
  phased through 2024.
- **BR LGPD** — Lei No. 13.709, 2018; Article 14 special provisions
  for children and adolescents.
- **IN DPDPA** — Digital Personal Data Protection Act 2023; §9
  processing of personal data of children.
- **SG PDPA** — Personal Data Protection Act 2012, latest revisions
  2020-2023; PDPC Children's Personal Data Advisory 2017 (under
  review 2024-2025).
- **AE Federal Decree-Law 45/2021.**
- **KSA PDPL** — Personal Data Protection Law (2023-09).

### Industry references

- **kidSAFE Seal Program** — FTC-approved COPPA Safe Harbor.
- **Privo COPPA Safe Harbor.**
- **TRUSTe Children's Privacy Certification (TrustArc).**
- **ESRB Privacy Certified.**
- **5Rights Foundation** — UK + EU advocacy on digital rights of
  children; AADC originator.
- **NCMEC** — National Center for Missing & Exploited Children (US);
  CyberTipline.
- **IWF** — Internet Watch Foundation (UK).
- **CEOP** — Child Exploitation and Online Protection Command (UK).
- **KOC** — Korea Online Coordination (KR); coordinates with KCC +
  MOGEF.

### Hyperscaler / B2C reference patterns

- **Apple Family Sharing + Screen Time + Ask to Buy** (iOS 16+
  refresh, 2023+).
- **Google Family Link + YouTube Kids** (post-2019 FTC settlement).
- **Microsoft Family Safety + Xbox Family Settings** (post-2023 FTC
  settlement).
- **Meta Instagram Teen Accounts** (2024-09 rollout).
- **TikTok For You restrictions for minors** (post-DSA + KOSA
  preparation).
- **Roblox Account Age + Parental Controls.**
- **Discord Teen Safety Assist** (2024).
- **Snap Family Center** (2022).

### Internal references

- **ADR-0002** — Tenant + Identity Kernel.
- **ADR-0003** — Audit Chain + Evidence Emission.
- **ADR-0007** — Cedar Authorization Policy + Persona Tier.
- **ADR-0099** — Data Class Registry.
- **ADR-0144** — EU AI Act graduated risk tier model.
- **ADR-0218** — Tenant-Granular Control Surface.
- **ADR-0242** — Oyatie is a Tenant Doctrine.
- **ADR-0243** — Cedar as Universal Gate.
- **ADR-0251** — Compliance Pack + Cell Certification Levels.

## Appendix A — Pattern attribution

The Minor User Doctrine is a synthesis of:

- **FTC's COPPA Rule structural pattern** — verifiable-parental-consent
  as the gating primitive, per-method enumeration (KBA, government-
  ID, credit-card, video, signed-form, email-plus).
- **UK ICO's AADC structural pattern** — 15-standards enumeration,
  loud-default-warn, "best interests of the child" framing.
- **EU EDPB's age-verification structural pattern** — risk-based
  assurance, per-Member-State threshold delegation, EDPB-coordinated
  guidance.
- **Apple's Family Sharing structural pattern** — minor status bound
  at identity, Ask-to-Buy gating, Screen Time defaults, Communication
  Safety scanning.
- **Google's Family Link structural pattern** — minor flag consumed
  uniformly across Google product surfaces, per-product UI surfacing.
- **Meta's Teen Accounts structural pattern** — defaults restored on
  every update, parental supervision opt-in, sleep-mode 22:00-07:00.
- **TikTok's post-DSA structural pattern** — non-profiled alternative,
  "Why am I seeing this?" affordance, per-jurisdiction band defaults.
- **KR PASS structural pattern** — carrier-bound identity verification,
  PIPA §22-2 parental consent.
- **JP carrier-bound parental verification structural pattern** —
  carrier-account binding for minor accounts.

The synthesis innovation is the **single canonical pack** combining
all jurisdictions into a uniformly-applied default-restricted
substrate, with per-jurisdiction overlays only where the
jurisdiction's statutory floor is stricter.

## Appendix B — Worked example: 12-year-old Carol signs up

This worked example walks through every doctrine surface that
activates when a 12-year-old user named Carol attempts to sign up for
a B2C oyatie tenant from a US/CA jurisdiction.

### B.1. Signup attempt

Carol opens the B2C signup flow at `https://signup.<tenant>.app/`.

The gateway resolves the request → routes to
`microservices/identity/signup` → identifies the tenant as B2C-facing
→ enforces pack-pinning (per `cloud-ci/Rust gate packet minor-user-pack-
pinning`, ENFORCED) → loads
`pack/MINOR-USER-2024@1.0.0/`.

The signup wizard surfaces an age-attestation step:

> "Please enter your birth date. We use this to provide age-appropriate
> features. (Required by law.)"

Carol enters `2014-03-15`.

The wizard computes `age_in_years(2014-03-15, 2026-05-20) = 12` →
classifies as `is_child = true` under US jurisdiction's COPPA
threshold (13) → activates the COPPA workflow.

### B.2. Age assurance (estimation tier)

The wizard invokes the age-assurance µservice, which routes to the
US-jurisdiction primary provider (Persona Identity Verification).
Persona surfaces a facial-age-estimation flow:

- Carol opts in; takes a brief selfie video; the estimation returns
  `estimated_age = 11.5, confidence_band = [10, 14], confidence = 0.84`.
- The confidence band spans the COPPA threshold (13), so the result
  is treated as inconclusive at the assurance level required for
  proceeding without parental consent.

The wizard transitions to the parental-consent path. Audit emission:

```json
{
  "event_class": "age_assurance_v1",
  "event_id": "01HXXX...",
  "tenant_id": "tenant-acme-b2c",
  "user_id_hash": "<hmac-pseudonymous-of-carol>",
  "verified_jurisdiction": "US-CA",
  "age_band": "child",
  "method": "estimation_facial",
  "provider": "persona",
  "outcome": "inconclusive_due_to_confidence_band",
  "estimated_age": 11.5,
  "confidence_band_low": 10,
  "confidence_band_high": 14,
  "occurred_at": "2026-05-20T14:32:11Z",
  "request_id": "req-...",
  "trace_id": "trace-..."
}
```

### B.3. Parental consent invitation

The wizard prompts Carol:

> "It looks like you might be 13 or younger. To use our service, a
> parent or guardian needs to give consent. We'll send them an
> invitation. What's your parent's email address?"

Carol enters `parent.carol@example.com`.

The wizard:

- Locks Carol's account in `account_state = consent_pending`.
- Sends an invitation email to `parent.carol@example.com` with a
  unique consent-request link.
- Emits `parental_consent_v1` with `state = pending`.

Carol sees a stub UI:

> "We've sent your parent an email. Once they give consent, your
> account will be ready. We'll let you know!"

### B.4. Parent consent flow

Carol's parent, Alex, receives the email. Alex clicks the link → is
routed to `microservices/consent/parental` → presented with the
list of consent verification methods available for US-CA
jurisdiction:

1. Knowledge-based authentication (KBA).
2. Government-ID match.
3. Credit-card verification (USD 0.50 refundable).
4. Video consent (schedule a 5-minute call with a T&S agent).
5. Signed-form upload.

Alex picks (3) credit-card verification.

The Consent µservice:

- Routes to the payments substrate to process a USD 0.50 refundable
  authorization.
- The authorization succeeds; the funds are immediately released back
  to Alex's card; the verified card-holder name + billing address
  match Alex's claimed identity.
- The Consent µservice records:
  - `consent_method = credit_card_verification`
  - `consent_provider = payments-substrate`
  - `consent_request_id = req-consent-...`
  - `consent_granted_at = 2026-05-20T14:47:33Z`
  - `consent_granted_by = <hmac-pseudonymous-of-alex>`
  - `consent_granted_for = <hmac-pseudonymous-of-carol>`
  - `consent_evidence_ref = <encrypted-blob-ref>`

Audit emission:

```json
{
  "event_class": "parental_consent_v1",
  "event_id": "01HXXX...",
  "state": "consent_granted",
  "method": "credit_card_verification",
  "provider": "payments-substrate",
  "consent_granted_at": "2026-05-20T14:47:33Z",
  "consent_for_user_hash": "<hmac-carol>",
  "consent_by_user_hash": "<hmac-alex>",
  "tenant_id": "tenant-acme-b2c",
  "verified_jurisdiction": "US-CA",
  "pack_id": "MINOR-USER-2024",
  "pack_version": "1.0.0",
  "occurred_at": "2026-05-20T14:47:33Z",
  "request_id": "req-consent-...",
  "trace_id": "trace-..."
}
```

The account state machine transitions Carol's account from
`consent_pending` → `consent_granted`. Carol receives a notification:
"Your parent gave consent! You can use the service now."

### B.5. Default-restricted feature set

Carol logs in. The policy engine evaluates every gateway request
under the Cedar context with `context.user.is_child = true`. The
following defaults apply (per D-4 + D-9):

- **Targeted ads:** REFUSED. Carol sees only contextual ads selected
  by the page content, not by Carol's profile.
- **Behavioral profiling / "For You" feed:** REFUSED. Carol sees a
  chronological feed.
- **DMs from non-contacts:** REFUSED. Carol can only DM users in her
  contacts. The platform asks Alex (parent) to approve each contact
  Carol adds.
- **Public profile:** REFUSED. Carol's profile is private; no search-
  engine indexing.
- **In-app purchases:** REFUSED without parental approval per
  transaction.
- **Marketplace listings:** REFUSED.
- **Marketplace purchases:** REFUSED without parental approval per
  transaction.
- **Creator economy / payouts:** REFUSED.
- **Sensitive-content surfaces:** REFUSED.
- **Location sharing:** REFUSED. No precise location collected.
- **Sleep mode:** ON (immutable for child). Push notifications
  silenced 22:00-07:00 local.
- **Live streaming:** REFUSED.
- **AI features:** REFUSED for under-13 (per ADR-0144 + D-11).
- **External-sender warnings (Mail):** ON (immutable, loud).
- **Group invites:** REFUSED for adult-group joins; teen-only or
  age-band-matched groups permitted.

Every one of these evaluations emits a `minor_policy_decision_v1`
audit event. The audit-chain receives a high-volume stream; the
SLO target is ≥ 99.99% completeness (per V-11).

### B.6. Parental dashboard

Alex (parent) logs into the parent dashboard at
`https://family.<tenant>.app/`. The dashboard surfaces:

- Carol's account status (active; child-band).
- Active consents (one: signup; granted 2026-05-20).
- Active contacts list (Alex can approve / remove).
- Recent activity summary (rolled up; not per-message).
- Sleep-mode setting (ON; not user-toggleable).
- Loosening toggles (each requires Alex's confirmation):
  - "Allow DMs from contacts of contacts" — OFF.
  - "Allow public profile" — OFF.
  - "Allow AI features (when Carol turns 13)" — pre-set OFF.
- Spending controls (default ceiling: USD 0; raise to USD 10/USD 25/
  USD 50 per month with per-transaction approval).
- Revoke consent button (triggers the revocation workflow).

Audit emission on every dashboard interaction:
`minor_policy_decision_v1` for view; `parental_consent_v1` for
state transitions.

### B.7. Time passes; Carol approaches 13

`2027-03-15`: Carol turns 13.

In US-CA jurisdiction, under COPPA, Carol crosses the
`child → teen` band but is still a minor for KOSA + CA-AADC
purposes (under 17). The doctrine bands Carol as `is_teen = true` with
parental consent still required for many feature loosenings.

Carol's account state machine does NOT trigger the age-of-majority
migration workflow at 13 (US age of majority is 18). It triggers
band-transition handling:

- The platform sends Carol + Alex a notification: "Carol is 13 today.
  Here's what changes."
- Some defaults loosen slightly (e.g., AI features become parent-
  opt-in-able; previously they were hard-refused).
- The `is_child = true` Cedar attribute flips to `is_child = false,
  is_teen = true`. The is_minor attribute remains true.

Carol's overall experience remains highly-restricted; the band-
transition is mostly invisible.

### B.8. 2032-03-15: Carol turns 18

Now Carol is approaching the US age of majority (18).

`2032-02-13` (T-30): The platform sends Carol + Alex notifications.

> "Carol, in 30 days you'll become an adult on this platform. Here's
> what changes."

Carol can preview the adult features but cannot yet activate them.
Alex receives a parallel notification: "Carol becomes an adult in 30
days. Here's how parental supervision will change."

`2032-03-08` (T-7): Repeat notice.

`2032-03-15` (T-0): Account state transitions to
`minor_status_transitioning`. Carol is greeted with an interstitial
wizard:

> "Welcome to adulthood on the platform! Let's review your privacy
> defaults."

Carol must complete an adult-grade identity assurance (per D-2 method
4: government-ID document verification). Carol uploads her
government ID; Persona verifies the document; the wizard records
`age_assurance_method = document_verified`,
`age_assurance_completed_at = 2032-03-15T...`.

The wizard surfaces opt-in toggles for each feature loosening:

- "Show targeted ads" — Carol opts out.
- "Show 'For You' feed" — Carol opts in.
- "Make profile public" — Carol opts in.
- "Allow DMs from non-contacts" — Carol opts in.
- "Enable AI features" — Carol opts in.
- ...

Each opt-in emits `minor_policy_decision_v1` with the explicit
consent_basis.

`2032-04-14` (T+30): Account state transitions
`account.minor_status → adult`. Parental supervision links remain
visible-only; Carol can revoke them in the dashboard. Minor-PII
retention timer starts ticking down; per the pack's retention rule,
Carol's child-band data is retained for the longer of (age-of-majority
+ statute-of-limitations) — for US-CA that's 18 + 7 years = 25 years
old, i.e., 2039-03-15.

Audit emission:

```json
{
  "event_class": "age_of_majority_migration_v1",
  "event_id": "01HXXX...",
  "state": "adult",
  "user_id_hash": "<hmac-carol>",
  "verified_jurisdiction": "US-CA",
  "transitioned_at": "2032-04-14T...",
  "tenant_id": "tenant-acme-b2c",
  "pack_id": "MINOR-USER-2024",
  "pack_version": "1.4.7",
  "trace_id": "trace-..."
}
```

### B.9. Counterfactual: Carol tries to age-up at 14

Counterfactual: at age 14, Carol attempts to edit her birth date from
`2014-03-15` to `2008-03-15` (claiming she's 18).

The identity µservice's age-down protection (per D-10) routes the
request through the `age_correction_request` workflow:

- Direction = age-up (claiming older). REQUIRES high-assurance
  identity verification per D-2 method 4 or higher AND a 14-day
  cooling-off period AND notification to the parental contact (Alex).
- The wizard requires Carol to upload a government ID; Persona
  attempts to verify against the claimed `2008-03-15` DOB.
- The government ID actually shows Carol's true DOB
  (`2014-03-15`). Verification FAILS.
- Audit emission:

```json
{
  "event_class": "minor_policy_decision_v1",
  "event_id": "01HXXX...",
  "tenant_id": "tenant-acme-b2c",
  "user_id_hash": "<hmac-carol>",
  "verified_jurisdiction": "US-CA",
  "age_band": "teen",
  "decision_principal": "identity-microservice",
  "decision_resource": "user-carol/birth-date",
  "decision_action": "edit",
  "decision_outcome": "deny",
  "policy_pack_id": "MINOR-USER-2024",
  "policy_pack_version": "1.4.7",
  "cedar_fragment_id": "14-age-down-refusal",
  "tenant_override_applied": null,
  "reason": "high_assurance_verification_failed",
  "occurred_at": "2028-08-12T...",
  "trace_id": "trace-..."
}
```

The platform's T&S surface receives the failed verification as a
fraud-signal candidate; Alex receives a parental notification.

### B.10. Regulator inquiry

In 2033, the FTC opens a sectoral inquiry into platforms hosting
US-CA minors. The FTC requests:

> "Produce all audit events for users in jurisdiction US-CA where the
> user was a minor between 2026-05-20 and 2031-12-31, including
> policy decisions affecting targeted ads, behavioral profiling,
> public profile defaults, and parental consent state."

The platform's audit-chain µservice (per ADR-0003 + D-13) returns:

- Per-user audit-event sequences keyed by `user_id_hash`.
- Per-pack-decision provenance: `policy_pack_id`,
  `policy_pack_version`, `cedar_fragment_id`,
  `tenant_override_applied`.
- Per-decision lineage: `trace_id` connecting gateway → policy-engine
  → µservice → audit-chain.

The FTC receives a queryable, signed evidence packet. The platform
demonstrates compliance with COPPA + KOSA + CA-AADC across the
inquiry window. The inquiry closes without enforcement action.

### B.11. Lessons

The worked example demonstrates:

- **Single source of truth.** Carol's minor status flows uniformly
  across signup, login, parental consent, feature gating, marketplace,
  ads, and migration.
- **Centralized audit emission.** Every minor-affecting decision
  emits `minor_policy_decision_v1`; the regulator-facing query is
  trivial.
- **Default-restricted privacy.** Carol's defaults are maximal-
  restriction; Alex's loosening is per-feature and audit-emitted.
- **Age-down protection.** Carol cannot silently age-up at 14.
- **Age-of-majority migration.** At 18, Carol opts-in feature-by-
  feature; defaults remain restrictive at T-0.
- **Regulator-readiness.** The 2033 FTC inquiry is answered by
  audit-chain query without an emergency forensics project.

This is the doctrine working as designed. Every B2C tenant onboarded
to the platform receives the same substrate; every minor receives
the same default-restrictive treatment; every parent receives the
same dashboard surface. The doctrine is the platform's first-class
defense.

---

End of ADR-0292.
