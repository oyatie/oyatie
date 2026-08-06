---
id: ADR-0302
status: Superseded
date: 2026-05-20
owners:
  - council-architecture
  - council-security
  - council-privacy
  - council-product
  - council-legal
  - ops-sre-reliability
  - ops-security
  - ops-trust-and-safety
  - ops-compliance
  - axis-identity
  - axis-policy-engine
  - axis-audit-chain
  - axis-tenancy
  - axis-deceased-user-inheritance
supersedes: []
amends: []
superseded_by: []
related:
  - ADR-0028-audit-chain-merkle-sealed.md
  - ADR-0044-service-mesh-and-mtls.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0140-cedar-policy-enforcement.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0188-passkey-webauthn-as-canonical-auth.md
  - ADR-0211-in-house-tech-stack-preference.md
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
  - ADR-0253-network-topology-edge-service-mesh.md
  - ADR-0258-api-versioning-semver-policy.md
  - ADR-0263-observability-emission-contract.md
  - ADR-0272-cookie-consent-per-purpose.md
  - ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md
  - ADR-0276-backup-portability-gdpr-art-20.md
  - ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md
  - ADR-0293-governance-meta-trust-root.md
  - ADR-0295-bootstrap-ci-spiffe-kill-switch.md
  - ADR-0296-library-first-credential-sidecar.md
  - ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md
  - ADR-0298-emergency-services-bypass-life-safety.md
  - ADR-0299-account-recovery-resilience.md
  - ADR-0300-whistleblower-press-freedom-anonymity.md
  - ADR-0301-survivor-safety-domestic-abuse-mode.md
related_specs:
  - /specs/platform-architecture.json
  - /specs/microservices/identity.json
  - /specs/microservices/policy-engine.json
  - /specs/microservices/audit-chain.json
  - /specs/microservices/observability.json
  - /specs/deceased-user-inheritance.json
  - /specs/cedar-fragment-schema.json
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
keystone_position: critical-path-doctrine-cluster-row-10-deceased-user-inheritance
purpose: >
  Codify the Deceased-User Inheritance Doctrine — a Legacy-Contact
  (Apple-class) + Inactive-Account-Manager (Google-class) + legal-
  representative court-order ingress + per-jurisdiction inheritance
  overlay surface that closes row 10 of the 30-row critical-path
  matrix in documentation-rigor.md §3.2.5. The bar is: a deceased
  user's pre-mortem wishes are honored exactly (whatever the
  survivor designated as Legacy-Contact OR Inactive-Account-Manager
  OR no-disclosure-and-delete-on-confirmation); beneficiaries are
  authenticated via per-tenant legacy-contact attestation OR per-
  pack legal-representative court-order; heirs cannot be unilaterally
  locked out by the tenant; per-jurisdiction inheritance law
  (US Revised Uniform Fiduciary Access to Digital Assets Act
  (RUFADAA) per state, EU Digital Heritage per member-state, KR
  Inheritance Act + Bukhumeolban (Civil Code), JP Civil Code +
  Family Court orders, UK Inheritance and Trustees' Powers Act
  2014, AU Succession Acts per state) is honored via per-pack
  overlay. The DSAR-cascade per ADR-0276 honors the deceased-user
  wish (export to legacy-contact OR delete OR archive-pseudonymously).
  Per documentation-rigor.md §3.2.5 row 10 — the safety/security/
  policy invariant: DSAR-cascade per ADR-0276 honors deceased-user
  wish; legal-rep access requires court order; tenant cannot
  unilaterally lock out heirs.
enforcement_status: advisory-until-2026-08-15-blocker-thereafter
enforced_by:
  - cloud-ci/Rust gate packet deceased-user-legacy-contact-attestation-chain
  - cloud-ci/Rust gate packet deceased-user-inactive-account-manager-fired
  - cloud-ci/Rust gate packet deceased-user-legal-rep-court-order-required
  - cloud-ci/Rust gate packet deceased-user-pre-mortem-wish-honored
  - cloud-ci/Rust gate packet deceased-user-dsar-cascade-per-adr-0276
  - cloud-ci/Rust gate packet deceased-user-per-pack-inheritance-overlay
  - cloud-ci/Rust gate packet deceased-user-cedar-fragment-present
naming_justifications:
  - name: oya-shared-deceased-user-inheritance
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.deceased-user-inheritance
    justification: >
      Per ADR-0105 13-layer canonical enum row 5 (shared-substrate),
      the crate that exposes the legacy-contact attestation
      verifier, the inactive-account-manager state machine, the
      legal-representative court-order ingress, the per-jurisdiction
      inheritance overlay router, the pre-mortem wish state-machine,
      and the DSAR-cascade per ADR-0276 belongs at the shared layer.
      Naming `oya-shared-deceased-user-inheritance` keeps the
      single-concern flat layout per ADR-0131 + ADR-0132.
  - name: oya-governance-deceased-user-inheritance
    layer: N/A (foundry-fitness aggregate CI lane)
    bnf_segments: oya.foundry-fitness.deceased-user-inheritance
    justification: >
      Aggregate CI fitness lane per ADR-0212 buildability doctrine;
      rolls up per-µservice declaration of the legacy-contact
      attestation chain, the inactive-account-manager firing, the
      legal-representative court-order ingress, the pre-mortem wish
      enforcement, the DSAR-cascade per ADR-0276, the per-pack
      inheritance overlay, and the Cedar fragment.
  - name: oya-governance-deceased-user-legacy-contact-attestation
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.deceased-user-legacy-contact-attestation
    justification: >
      Per-µservice child lane verifying the Apple-class legacy-
      contact attestation chain is wired (per-account designated
      Legacy-Contact + per-pack death-certificate attestation OR
      per-tenant operational alternative for jurisdictions without
      Apple-class chain).
  - name: oya-governance-deceased-user-inactive-account-manager
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.deceased-user-inactive-account-manager
    justification: >
      Per-µservice child lane verifying the Google-class Inactive-
      Account-Manager (IAM) state machine is wired (per-account
      designated inactivity threshold + per-account designated
      beneficiary list + per-account designated action on firing).
  - name: oya-governance-deceased-user-legal-rep-court-order
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.deceased-user-legal-rep-court-order
    justification: >
      Per-µservice child lane verifying the per-jurisdiction legal-
      representative court-order ingress is wired and emits the
      DeceasedUserLegalRepAccessGranted audit event with court-
      order attestation.
  - name: oya-governance-deceased-user-pre-mortem-wish-honored
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.deceased-user-pre-mortem-wish-honored
    justification: >
      Per-µservice child lane verifying the pre-mortem wish state
      machine honors the deceased-user's pre-recorded preference
      (export-to-legacy-contact, delete, archive-pseudonymously, or
      per-pack default) exactly.
  - name: oya-governance-deceased-user-per-pack-inheritance-overlay
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.deceased-user-per-pack-inheritance-overlay
    justification: >
      Per-µservice child lane verifying the per-jurisdiction
      inheritance overlay (US RUFADAA + EU + KR + JP + UK + AU)
      is present + Cedar-fragment-encoded.
  - name: X-Oya-Account-Lifecycle-State
    layer: N/A (HTTP request header naming)
    bnf_segments: X-Oya.Account-Lifecycle-State
    justification: >
      Custom HTTP request header carrying the account lifecycle
      state (extended enum: ACTIVE, SUSPENDED, SHELTER_MODE_ACTIVE,
      DECEASED_CONFIRMED, DECEASED_PRE_MORTEM_PENDING, INACTIVE_
      ACCOUNT_MANAGER_FIRED, LEGAL_REP_COURT_ORDER_GRANTED,
      MEMORIALIZED). Used to route requests per ADR-0244 §D-3.
  - name: X-Oya-Inheritance-Path
    layer: N/A (HTTP request header naming)
    bnf_segments: X-Oya.Inheritance-Path
    justification: >
      Custom HTTP request header identifying the active inheritance
      path (enum: LEGACY_CONTACT, INACTIVE_ACCOUNT_MANAGER, LEGAL_
      REP_COURT_ORDER, PRE_MORTEM_DELETE_ON_CONFIRMATION,
      PRE_MORTEM_ARCHIVE_PSEUDONYMOUSLY, PRE_MORTEM_MEMORIALIZE).
  - name: DeceasedUserDeathReported
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: DeceasedUser.DeathReported
    justification: >
      Audit-event-class emitted when death is reported to the
      platform (by legacy-contact, by legal-rep, by per-pack
      regulator). Registered in ADR-0263 central registry per
      §3.2.2 consistency invariant.
  - name: DeceasedUserLegacyContactAttestationVerified
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: DeceasedUser.LegacyContactAttestationVerified
    justification: >
      Audit-event-class emitted when the per-account designated
      Legacy-Contact's attestation chain is verified + the per-
      pack death-certificate attestation passes.
  - name: DeceasedUserInactiveAccountManagerFired
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: DeceasedUser.InactiveAccountManagerFired
    justification: >
      Audit-event-class emitted when the per-account Inactive-
      Account-Manager threshold trips + the designated action is
      taken (notify beneficiaries, export data, delete, archive).
  - name: DeceasedUserLegalRepAccessGranted
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: DeceasedUser.LegalRepAccessGranted
    justification: >
      Audit-event-class emitted when a legal-representative is
      granted access pursuant to a court order with per-pack
      jurisdictional verification.
  - name: DeceasedUserPreMortemWishHonored
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: DeceasedUser.PreMortemWishHonored
    justification: >
      Audit-event-class emitted when the deceased-user's pre-mortem
      wish (export, delete, archive, memorialize) is executed.
  - name: DeceasedUserDsarCascadeExecuted
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: DeceasedUser.DsarCascadeExecuted
    justification: >
      Audit-event-class emitted when the DSAR-cascade per ADR-
      0276 (GDPR Art. 20 portability + per-pack inheritance overlay)
      executes on the deceased user's data.
  - name: DeceasedUserMemorialized
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: DeceasedUser.Memorialized
    justification: >
      Audit-event-class emitted when an account transitions to
      memorialized state (Facebook-class memorialization where
      profile is preserved but locked).
  - name: policy/deceased-user-inheritance.cedar
    layer: N/A (per-µservice Cedar fragment file)
    bnf_segments: policy.deceased-user-inheritance
    justification: >
      Canonical filename for the per-µservice deceased-user-
      inheritance Cedar fragment under the µservice's `policy/`
      directory per ADR-0246 + ADR-0243 fragment-lifecycle
      conventions; single-concern naming keeps the policy
      directory's contract-by-name invariant.
  - name: spec/deceased-user-inheritance.json
    layer: N/A (canonical inheritance state spec)
    bnf_segments: specs.deceased-user-inheritance
    justification: >
      Canonical machine-readable spec for the deceased-user
      inheritance state machine + the per-jurisdiction overlay
      roster + the pre-mortem wish format.
  - name: DECEASED_CONFIRMED
    layer: N/A (Account.lifecycle_state enum value per ADR-0244)
    bnf_segments: lifecycle_state.DECEASED_CONFIRMED
    justification: >
      Account.lifecycle_state enum extension per ADR-0244 §D-3;
      identifies accounts where the death has been reported AND
      attested via the per-account legacy-contact chain OR per-
      pack legal-rep court-order.
  - name: DECEASED_PRE_MORTEM_PENDING
    layer: N/A (Account.lifecycle_state enum value per ADR-0244)
    bnf_segments: lifecycle_state.DECEASED_PRE_MORTEM_PENDING
    justification: >
      Account.lifecycle_state enum extension; identifies accounts
      where the death has been reported but attestation is pending
      (within the per-pack pending window).
  - name: INACTIVE_ACCOUNT_MANAGER_FIRED
    layer: N/A (Account.lifecycle_state enum value per ADR-0244)
    bnf_segments: lifecycle_state.INACTIVE_ACCOUNT_MANAGER_FIRED
    justification: >
      Account.lifecycle_state enum extension; identifies accounts
      where the Google-class Inactive-Account-Manager threshold
      has tripped.
  - name: LEGAL_REP_COURT_ORDER_GRANTED
    layer: N/A (Account.lifecycle_state enum value per ADR-0244)
    bnf_segments: lifecycle_state.LEGAL_REP_COURT_ORDER_GRANTED
    justification: >
      Account.lifecycle_state enum extension; identifies accounts
      where a legal-rep court-order has been granted per ADR-0247
      break-glass + 2-member quorum.
  - name: MEMORIALIZED
    layer: N/A (Account.lifecycle_state enum value per ADR-0244)
    bnf_segments: lifecycle_state.MEMORIALIZED
    justification: >
      Account.lifecycle_state enum extension; identifies accounts
      that have transitioned to memorialized state per the deceased
      user's pre-mortem wish.
  - name: legacy-contact
    layer: N/A (Principal.role enum value per ADR-0244)
    bnf_segments: role.legacy-contact
    justification: >
      Principal.role enum extension per ADR-0244 §D-3; identifies
      the per-account designated Legacy-Contact who can be
      attested as the legitimate inheritor.
  - name: legal-representative
    layer: N/A (Principal.role enum value per ADR-0244)
    bnf_segments: role.legal-representative
    justification: >
      Principal.role enum extension; identifies the per-pack legal
      representative with court-order-attested access.
---

# ADR-0302: Deceased-User Inheritance Doctrine

## Status

Proposed — 2026-05-20.

Bundled with the keystone-bundle 2026-05-20 foundational doctrine
synthesis as the **critical-path-doctrine-cluster-row-10-deceased-
user-inheritance** keystone. Closes row 10 of the 30-row critical-
path matrix in `docs/standards/documentation-rigor.md` §3.2.5.

Enforcement is `advisory-until-2026-08-15-blocker-thereafter`. The
doctrine accepts in text immediately; the seven CI lanes that
enforce it promote to BLOCKER on 2026-08-15 to give per-pack
inheritance overlay onboarding (US RUFADAA per state, EU Digital
Heritage per member-state, KR Civil Code + Inheritance Act, JP
Civil Code + Family Court order, UK Inheritance and Trustees'
Powers Act 2014, AU Succession Acts per state), per-tenant legacy-
contact attestation chain provisioning, and per-µservice DSAR-
cascade wiring time to land. Until 2026-08-15, validators emit
findings without failing CI; post-2026-08-15 the lanes block
merge.

## Date

2026-05-20.

## Context

### §A. Why deceased-user inheritance is a substrate primitive, not a per-µservice afterthought

Deceased-user inheritance is a critical-path edge case (per
documentation-rigor.md §3.2.5 row 10) because the standard
account-lifecycle pattern — long-lived sessions, password-protected
account, no special handling of inactivity — silently fails when
the legitimate user dies. Without a substrate-level inheritance
primitive, the deceased user's data either (a) becomes inaccessible
to legitimate heirs (locked behind the deceased's credentials),
(b) is unilaterally exfiltrated by anyone who can guess credentials
(no per-pack verification), (c) is unilaterally deleted by the
tenant in a privacy-overreach move (ignoring the deceased's pre-
mortem wishes), or (d) lingers indefinitely as a dormant tail
risk (data-leakage risk + post-mortem identity-theft risk).

The pattern across mature hyperscaler platforms is unambiguous:

- **Apple Legacy Contact.** Apple's Legacy Contact program (rolled
  out iOS 15.2 + iCloud, refined in iOS 16+ per `support.apple.
  com/en-us/HT212513`) provides a per-account pre-mortem designation
  of trusted contacts who can request access after the user's
  death. Per Apple's published architecture: (i) the user pre-
  designates ≤5 Legacy Contacts in iCloud settings, (ii) each
  Legacy Contact receives an access-key fragment, (iii) after the
  user's death, the Legacy Contact submits a death-certificate
  (notarized; per-pack jurisdictional verification) + their access-
  key fragment, (iv) Apple grants access to the deceased's iCloud
  data (photos, notes, files, calendar, but NOT iCloud Keychain
  passwords, NOT Apple Pay, NOT in-app subscriptions). The Legacy
  Contact has 3 years of access; after 3 years the data is deleted.
  This is the canonical Legacy-Contact pattern oyatie inherits.
- **Google Inactive Account Manager (IAM).** Google's Inactive
  Account Manager (rolled out 2013 per `myaccount.google.com/
  inactive`) provides a per-account pre-mortem designation of
  the inactivity threshold + the designated beneficiary list +
  the action on firing. Per Google's published documentation:
  (i) the user designates an inactivity threshold (3-18 months),
  (ii) Google sends multiple advance-warning emails + SMS, (iii)
  if the user does not respond, Google fires the IAM, (iv) the
  designated beneficiaries (≤10) receive an access-link to a per-
  beneficiary subset of the user's data, (v) optionally, the
  account is permanently deleted on firing. This is the canonical
  Inactive-Account-Manager pattern oyatie inherits.
- **Facebook / Meta Memorialization + Legacy Contact.** Facebook's
  memorialization (rolled out 2009, refined 2015 with Legacy
  Contact per `facebook.com/help/1568013990080948`) provides per-
  account post-mortem designation of: (i) memorialize the profile
  (preserve content + lock new posts), (ii) delete the account,
  (iii) Legacy Contact who can manage the memorialized profile
  (post pinned tribute, accept new friend requests, update
  profile photo). Memorialization is the canonical "preserve but
  lock" pattern.
- **Microsoft Next of Kin Access.** Microsoft's Next of Kin
  process (per `support.microsoft.com/en-us/account-billing/
  access-outlook-com-onedrive-and-other-microsoft-services-when-
  someone-has-died-ebbd2860-917e-4b39-9913-212362da6b2f`) provides
  per-pack legal-representative court-order ingress for
  beneficiaries seeking access to a deceased user's Microsoft
  account data. The process requires (i) death certificate, (ii)
  legal-rep-court-order OR per-pack jurisdictional inheritance
  documentation, (iii) per-jurisdiction verification by Microsoft
  Legal. The data is provided as a one-time export (no ongoing
  access).
- **Twitter / X Memorialization.** Twitter's memorialization (per
  `help.x.com/en/forms/account-access/deactivate-or-close-account/
  request-deactivation-deceased-user`) supports per-account
  deactivation by an immediate-family member + death certificate.
  X does not provide Legacy-Contact-style access — the data is
  either deactivated OR (separately, with court order) exported.
- **iCloud Digital Legacy Program.** Apple's broader Digital
  Legacy Program (`support.apple.com/en-us/HT212361`) extends the
  Legacy Contact to include the access-key generation + the per-
  jurisdiction death-certificate verification process. The program
  is operational in ~30 jurisdictions as of 2026, with per-pack
  jurisdictional overlays.
- **Last Will + Estate-planning platforms (LegalZoom, Trust &
  Will).** These platforms publish per-jurisdiction digital-asset
  estate-planning templates that pre-designate how online
  accounts are handled. Per the National Conference of
  Commissioners on Uniform State Laws (NCCUSL), the Revised
  Uniform Fiduciary Access to Digital Assets Act (RUFADAA;
  adopted by ~47 US states as of 2026) is the canonical per-
  state inheritance overlay for digital assets.
- **Coinbase Inheritance Beneficiary.** Coinbase's inheritance
  beneficiary program (per `help.coinbase.com/en/coinbase/managing-
  my-account/identity-verification/inheritance-beneficiary`)
  provides per-account beneficiary designation + per-pack
  jurisdictional verification + per-asset-class transfer
  (USD-equivalent cash-out OR per-asset transfer to beneficiary
  Coinbase account). Crypto inheritance is the highest-stakes
  digital-asset inheritance scenario.

The corollary: **every internet-facing surface oyatie ships MUST
inherit deceased-user inheritance from the substrate, not author
it per-µservice.** A µservice that authors its own legacy-contact
attestation, its own inactive-account-manager state machine, its
own legal-rep court-order ingress is duplicating substrate
primitives that the shared crate already serves. That duplication
is a `feedback_no_silent_regression` violation (every µservice's
inheritance drifts independently); it is a
`feedback_quality_performance_scalability_bar` violation (per-
µservice flows cannot share the cross-µservice DSAR-cascade per
ADR-0276); and it is a `feedback_autonomous_implementation_
artifacts` violation (intern-buildable means the doc surface is
one substrate, not 46 µservice-private implementations).

The ADR-0302 deceased-user inheritance doctrine closes this gap.

### §A.1. The row 10 mandate from §3.2.5 — verbatim

Per documentation-rigor.md §3.2.5 row 10 — "Deceased-user account":

> Beneficiary access; legal-rep access; per-jurisdiction
> inheritance law. Legacy-contact (Apple-class) + legal-rep court-
> order ingress + per-jurisdiction inheritance overlay; explicit
> per-tenant pre-mortem wish honored. Safety/security/policy
> invariant: DSAR-cascade per ADR-0276 honors deceased-user wish;
> legal-rep access requires court order; tenant cannot
> unilaterally lock out heirs.

Three load-bearing clauses:
- **"Legacy-contact (Apple-class) + legal-rep court-order ingress
  + per-jurisdiction inheritance overlay."** Three distinct paths
  must be wired: (i) per-account Legacy-Contact pre-designation,
  (ii) Google-class Inactive-Account-Manager firing, (iii) legal-
  rep court-order ingress (Microsoft-class). Per-jurisdiction
  overlay binds each path to the local inheritance statute.
- **"Explicit per-tenant pre-mortem wish honored."** The deceased
  user's pre-mortem wish (export, delete, archive-pseudonymously,
  memorialize) takes precedence over both Legacy-Contact and
  legal-rep court-order — the wish is the user's exercise of
  pre-mortem autonomy and the substrate honors it exactly.
- **"Tenant cannot unilaterally lock out heirs."** The tenant
  cannot delete or restrict access against the deceased user's
  pre-mortem wish AND the legitimate heirs' inheritance claim.
  The substrate provides cross-tenant ombudsman escalation when
  tenant policy conflicts with inheritance.

### §A.2. Why the keystone bundle 2026-05-20 requires this as a substrate-level primitive

The keystone bundle's foundational ADRs intersect deceased-user
inheritance as follows:

- **ADR-0242 (oyatie-is-a-tenant).** Deceased-user inheritance
  applies to oyatie platform admins identically (a deceased
  platform admin's account is subject to the inheritance flow).
- **ADR-0243 (Cedar universal gate).** Inheritance is a Cedar
  policy decision. The Cedar fragment FORBIDs tenant unilateral
  delete OR restriction of a deceased-user account where heirs
  have inheritance claim.
- **ADR-0244 (tenant scoping primitive).** This ADR adds five
  Account.lifecycle_state enum values (`DECEASED_CONFIRMED`,
  `DECEASED_PRE_MORTEM_PENDING`, `INACTIVE_ACCOUNT_MANAGER_FIRED`,
  `LEGAL_REP_COURT_ORDER_GRANTED`, `MEMORIALIZED`) + two
  Principal.role enum values (`legacy-contact`, `legal-
  representative`) per ADR-0244 §D-3.
- **ADR-0246 + amendment (policy-engine library-first).** Every
  µservice's library-first Cedar evaluator carries the deceased-
  user-inheritance Cedar fragment.
- **ADR-0247 (self-modification / break-glass).** Per-pack legal-
  rep court-order ingress inherits the ADR-0247 break-glass
  pattern (2-member quorum + post-hoc audit-and-justify +
  cryptographically sealed per ADR-0028).
- **ADR-0248 (Amazon-shape cellular architecture).** Per-tenant
  inheritance state is partitioned per-cell; cross-cell
  inheritance is achieved via the per-tenant home-cell + DR-cell
  binding.
- **ADR-0251 (compliance packs).** Each pack adds per-jurisdiction
  inheritance overlay (US RUFADAA-<state>, EU-Digital-Heritage-
  <member-state>, KR-Civil-Code-Inheritance, JP-Civil-Code-
  Inheritance, UK-Inheritance-and-Trustees-Powers-Act-2014, AU-
  Succession-Act-<state>).
- **ADR-0252 (HLC + TrueTime).** Inheritance timestamp resolution
  uses HLC default (per ADR-0252) for the per-pack inactivity
  threshold timing.
- **ADR-0253 (HTTP/3 + QUIC default + ECH + PQC).** Inheritance
  data transfer (DSAR cascade) uses ECH-enabled TLS + PQC hybrid
  KEX where available (the inheritance data is long-term
  confidentiality-critical).
- **ADR-0263 (observability emission contract).** Inheritance
  flow emits seven dedicated audit-event classes; per-pack regulator
  notification per ADR-0251.
- **ADR-0272 (cookie consent per-purpose).** Legacy-Contact's
  post-mortem access surface uses purpose-scoped cookies (no
  pre-mortem consent record carries forward).
- **ADR-0273 (per-tenant DKIM/SPF/DMARC).** Pre-mortem advance-
  warning emails (Inactive-Account-Manager) use per-tenant DKIM-
  signed delivery per ADR-0273.
- **ADR-0276 (backup portability GDPR Art. 20).** The DSAR-cascade
  per ADR-0276 is THE primitive that honors the deceased-user wish
  — the cascade exports the data to the Legacy-Contact OR the
  legal-rep with court-order OR deletes per the wish.
- **ADR-0280 (substrate-of-substrate).** Inheritance depends on
  `oya-shared-cedar-evaluator` + `oya-shared-audit-emit` +
  `oya-shared-account-recovery` (ADR-0299) for the legacy-contact
  attestation chain + `oya-shared-anonymity-substrate` (ADR-0300)
  for the archive-pseudonymously wish path.
- **ADR-0284 (platform-owner name indirection).** Namespace
  `oyatie.deceased.*` and `oyatie.legacy-contact.*` parameterized
  via the platform-owner indirection.
- **ADR-0292 (minor user doctrine).** A deceased minor's account
  routes through the parental-rights overlay (parents' inheritance
  claim per per-jurisdiction law); per-pack regulator notification
  per ADR-0292 + ADR-0251.
- **ADR-0293 (meta-trust-root).** Legacy-Contact attestation keys
  rooted at the meta-trust-root.
- **ADR-0294 (Cedar fragment soak).** ≥60s soak window respected.
- **ADR-0295 (bootstrap CI SPIFFE + kill-switch).** SPIFFE identity
  for the inheritance-µservice + per-cell kill-switch.
- **ADR-0296 (library-first credential sidecar).** Legacy-Contact
  attestation keys held in sidecar with ≤60s OpenBao TTL.
- **ADR-0297 (abuse-defence baseline).** Abuse-defence observation-
  only on the inheritance ingress surface (legacy-contact
  submissions of death certificates must not be gate-blocked by
  bot-mgmt false-positives during family grief).
- **ADR-0298 (emergency-services bypass).** An imminent-threat
  reported via the inheritance surface (e.g., during legacy-
  contact onboarding, the heir reports a suicide risk for another
  family member) transitions to the bypass.
- **ADR-0299 (account-recovery resilience).** Legacy-Contact
  attestation uses the F3-class delegated-trusted-contact
  pattern + F4-class ombudsman path for legal-rep court-order.
- **ADR-0300 (whistleblower-press-anonymity).** Archive-
  pseudonymously wish-path uses the pseudonymity substrate from
  ADR-0300; deceased-user data archived under a pseudonymity-
  class principal scope.
- **ADR-0301 (survivor-safety-domestic-abuse-mode).** A deceased
  survivor's account preserves the SHELTER_MODE_ACTIVE state
  through the inheritance flow; the shared-credential-abuser
  designation prevents the abuser from inheriting.

### §A.3. The failure modes the substrate must defend against

Per documentation-rigor.md §1.1 rigor sub-test "failure-mode tree":

**FM-1: Death is unreported to the platform.** Mitigation: Google-
class Inactive-Account-Manager fires on inactivity threshold (per
§D-3 below) even without explicit death notification.

**FM-2: False death claim from a hostile actor.** Mitigation:
death-certificate attestation chain (notarized + per-pack
jurisdictional verification) + per-pack ombudsman 2-member quorum
+ per-account designated Legacy-Contact pre-attestation. A
hostile-actor claim without these chains is denied at gate-time.

**FM-3: Legitimate Legacy-Contact cannot prove identity.**
Mitigation: per-account access-key fragment pre-designation (Apple-
class); per-pack jurisdictional alternative (e.g., per-pack
notarial-deed); per-pack ombudsman escalation.

**FM-4: Conflicting inheritance claims from multiple heirs.**
Mitigation: per-pack legal-rep court-order ingress; per-pack
ombudsman escalation; per-pack jurisdictional inheritance overlay
adjudication (per-jurisdiction statute determines distribution).

**FM-5: Deceased-user's pre-mortem wish conflicts with heirs'
inheritance claim.** Mitigation: pre-mortem wish takes precedence
EXCEPT where per-pack statute mandates inheritance (e.g., KR
Inheritance Act mandates forced-heirship of ≥50% to spouse +
children regardless of will; EU member-state forced-heirship
varies). Per-pack Cedar fragment resolves the conflict per the
applicable statute.

**FM-6: Tenant attempts to unilaterally delete the deceased-user
account against heirs' inheritance claim.** Mitigation: Cedar
fragment §D-2 FORBID + per-pack ombudsman escalation + per-
jurisdiction inheritance claim hold.

**FM-7: Legal-rep court-order is forged.** Mitigation: per-pack
ombudsman 2-member quorum + per-pack jurisdictional verification
(per-jurisdiction court records check + per-jurisdiction notary
verification) + per-pack regulator notification.

**FM-8: Inactive-Account-Manager fires falsely (user is alive
but inactive due to extended travel, hospitalization, military
deployment).** Mitigation: pre-fire advance-warning emails (≥3
warnings over ≥30 days per §D-3) + per-account verified secondary
channel notification + opt-in extension; per-pack regulator
mandate may extend the threshold (e.g., military-deployment
extension per US Servicemembers Civil Relief Act).

**FM-9: Deceased-user has SHELTER_MODE_ACTIVE (ADR-0301) status.**
Mitigation: shelter-mode is preserved through inheritance flow;
the designated shared-credential-abuser cannot inherit; per-pack
ombudsman ensures inheritance honors the survivor's pre-mortem
wishes.

**FM-10: Cross-pack jurisdictional conflict on inheritance.**
Mitigation: per ADR-0299 §3.2.5 row 23 conflict-resolution rule
+ per-pack ombudsman cross-jurisdictional adjudication; in
inheritance specifically, the deceased's domicile pack typically
governs (per the Hague Convention on the Law Applicable to
Succession to the Estates of Deceased Persons + per-jurisdiction
private international law).

**FM-11: Account contains the deceased's whistleblower
submission (ADR-0300).** Mitigation: per ADR-0300 §A.1 + §D-1
sealed-sender envelope, the submission's confidentiality survives
death; the legacy-contact does NOT see the submission content;
per-pack ombudsman holds the sealed-key share + maintains the
chain-of-custody.

**FM-12: Account contains a minor's data (deceased was a minor
OR account held minor's content per ADR-0292).** Mitigation:
per-pack minor-protection overlay (cross-ref ADR-0292) + per-
jurisdiction parental-rights / guardianship resolution +
per-pack ombudsman.

## Decision

### §B. Decision summary

**Decision 1: Per-account Legacy-Contact pre-designation (Apple-
class).** Every account can pre-designate ≤5 Legacy-Contacts in
the account preferences. Each Legacy-Contact:
- Receives an access-key fragment at designation time (Shamir-
  shared per ADR-0247 break-glass + per-Legacy-Contact threshold
  of 2-of-N).
- After the user's death, submits the death certificate (per-
  pack jurisdictional verification) + their access-key fragment.
- Receives the per-Legacy-Contact subset of the deceased's data
  per the deceased's pre-mortem designation.

**Decision 2: Per-account Inactive-Account-Manager pre-designation
(Google-class).** Every account can pre-designate the inactivity
threshold (3-18 months default; per-pack regulator-mandated
extensions for military deployment / extended hospitalization),
the designated beneficiary list (≤10), and the action on firing
(notify + grant access, delete, archive-pseudonymously, memorialize).

**Decision 3: Per-pack legal-rep court-order ingress (Microsoft-
class).** Per-pack legal-rep can submit a court-order with the
per-pack jurisdictional verification chain. The ingress requires:
- Court-order document attestation (per-pack notarial verification).
- Death certificate attestation.
- Per-pack ombudsman 2-member quorum per ADR-0247.
- Per-jurisdictional inheritance overlay verification.
- Per-pack regulator notification per ADR-0251.

**Decision 4: Pre-mortem wish takes precedence (within per-pack
statutory bound).** The deceased's pre-mortem wish (recorded via
the account preferences + per-pack jurisdictional confirmation)
takes precedence over inheritance flow, EXCEPT where per-pack
statute mandates inheritance (e.g., KR forced-heirship). Per-pack
Cedar fragment §D-2 resolves the conflict per the active statute.

**Decision 5: DSAR-cascade per ADR-0276.** Per ADR-0276 (GDPR
Art. 20 backup portability), the inheritance flow executes the
DSAR-cascade — the per-account data is exported in the canonical
portable format (per ADR-0276 §D-N) and delivered to the Legacy-
Contact OR legal-rep OR deleted per the deceased's wish.

**Decision 6: Tenant cannot unilaterally lock out heirs.** Per
the §3.2.5 row 10 invariant — tenant policy CANNOT delete or
restrict access against the deceased's pre-mortem wish AND the
legitimate heirs' inheritance claim. Cedar fragment §D-2 FORBID
+ cross-tenant ombudsman escalation.

**Decision 7: Per-jurisdiction inheritance overlay.** Each per-
pack overlay declares: applicable inheritance statute, forced-
heirship requirements, court-order verification chain, per-
jurisdiction notary requirements, per-jurisdiction regulator
notification, per-jurisdiction data-residency for inheritance
data transfer.

**Decision 8: Memorialization optional + per-account opt-in.**
The Facebook-class memorialization is available as a pre-mortem
wish (preserve profile + lock new posts + designate memorial
Legacy-Contact who can manage the memorialized profile). Per-
tenant policy may default-enable memorialization for community
surfaces (e.g., social-network tenants).

## Consequences

### §C. Consequences across all 6 engineering-rigor dimensions

Per documentation-rigor.md §1.2 engineering-rigor dimensions
matrix:

#### §C.1. Maintainability

- **Module boundaries.** Inheritance logic encapsulated in
  `oya-shared-deceased-user-inheritance` (single concern). Cedar
  fragment in one file per µservice. IaC manifest in one file
  per env. No scattered logic.
- **Versioning policy.** Per ADR-0258 SemVer. Cedar fragment
  per-fragment SemVer; shared crate Cargo SemVer; per-pack
  overlay SemVer (per-jurisdiction statute updates trigger
  MINOR bumps).
- **Deprecation cadence.** Per-pack overlay updates as
  jurisdictions revise inheritance statutes (e.g., EU member-
  state Digital Heritage transposition; US per-state RUFADAA
  adoption). ≥6-month sunset per ADR-0258 for any per-pack
  overlay change.
- **Reverse-dependencies enumerated.** Every µservice that
  consumes inheritance declared in `manifest.json:reverse_
  consumers_of_deceased_user_inheritance`.
- **What is hard-coded vs configurable.** Hard-coded: audit-
  event-class slugs, the FORBID tenant unilateral lockout of
  heirs, the pre-mortem wish precedence (within per-pack statute).
  Configurable: per-account Legacy-Contact roster (≤5), per-
  account Inactive-Account-Manager threshold (3-18 months),
  per-account pre-mortem wish (export / delete / archive /
  memorialize), per-pack legal-rep court-order verification chain.

#### §C.2. Observability

- **Metrics.** Per ADR-0263 cardinality budget:
  - `deceased_user_death_reported_total` (counter, dimensions:
    tenant_pack, ingress_path (legacy_contact, inactive_account_
    manager, legal_rep), cardinality ≤3k series).
  - `deceased_user_legacy_contact_attestation_verified_total`
    (counter, dimensions: tenant_pack, cardinality ≤2k series).
  - `deceased_user_inactive_account_manager_fired_total`
    (counter, dimensions: tenant_pack, threshold_months,
    cardinality ≤2k series).
  - `deceased_user_legal_rep_access_granted_total` (counter,
    dimensions: tenant_pack, jurisdiction, cardinality ≤1k
    series).
  - `deceased_user_pre_mortem_wish_honored_total` (counter,
    dimensions: wish_class (export, delete, archive, memorialize),
    cardinality ≤500 series).
  - `deceased_user_dsar_cascade_executed_total` (counter,
    dimensions: tenant_pack, cardinality ≤2k series).
  - `deceased_user_memorialized_total` (counter, dimensions:
    tenant_pack, cardinality ≤1k series).
- **Trace span shape.** Every inheritance flow carries
  `deceased_user_inheritance.flow` parent span with child spans
  `deceased_user_inheritance.legacy_contact_attestation`,
  `deceased_user_inheritance.dsar_cascade`, `deceased_user_
  inheritance.audit_emit`.
- **Logs.** JSON-structured log lines at INFO. Per-pack regulator
  retention floor (e.g., US RUFADAA: retain inheritance audit
  rows ≥7 years; EU Digital Heritage: retain per member-state
  statute).
- **Audit events.** Per ADR-0263 — seven new classes from §B.
  Audit chain Merkle-sealed per ADR-0028.
- **SLO floor.** P95 legacy-contact-attestation ≤5 days
  (jurisdictional verification + per-pack ombudsman + DSAR
  cascade); P99 ≤30 days. P95 inactive-account-manager firing
  on threshold ≤1 day; P99 ≤7 days.
- **Dashboards.** `dashboards/deceased-user-inheritance.json`
  per µservice + `dashboards/deceased-user-platform-wide.json`
  substrate-level.

#### §C.3. Scalability

- **Capacity math.** Baseline death-rate: ~0.8% per year for the
  consumer-tier population. At ~100M active consumers, ~800k
  deaths per year = ~92 deaths per hour. Inactive-Account-
  Manager firing: ~5× the death rate (legitimate inactivity)
  = ~500 firings per hour.
- **Bottleneck identification.** Per-pack ombudsman + per-
  jurisdiction notarial verification is the bottleneck (≤5 day
  P95 SLA per §C.2). DSAR cascade execution scales horizontally
  per ADR-0276 §D-N.
- **Horizontal scale-out path.** Per-tenant inheritance state
  partitioned per-cell; adding cells scales horizontally.

#### §C.4. Performance

- **P50/P95/P99 latency.** P50 legacy-contact-attestation ≤2
  days; P95 ≤5 days; P99 ≤30 days. P50 inactive-account-manager-
  firing ≤1 hour after threshold; P95 ≤1 day; P99 ≤7 days. P50
  legal-rep-court-order ≤7 days; P95 ≤30 days; P99 ≤90 days.
  P50 DSAR-cascade-execution ≤1 hour; P95 ≤24 hours; P99 ≤7
  days.
- **Modeling note.** Latency dominated by per-pack ombudsman
  human-review SLA + per-jurisdiction notarial verification +
  per-jurisdiction court-order processing.
- **Per-region budget split.** Per ADR-0240: US ≤5 days P95 (per-
  state RUFADAA + per-state probate court); EU ≤7 days P95
  (per-member-state Digital Heritage transposition); KR ≤5 days
  P95; JP ≤7 days P95; AU ≤7 days P95; UK ≤5 days P95.
- **Tail-latency mitigation.** Per-pack secondary ombudsman
  reviewer auto-routing on SLA risk.

#### §C.5. Optimization

- **Per-call cost model.** CPU ≤10ms per attestation verification
  (Shamir-shared access-key reconstitution + per-pack
  jurisdictional verification handshake); RAM ≤512KB per
  inheritance flow; IOPS ≤1 per flow; $/M-flows ≤$10.00
  (excludes per-pack ombudsman human-reviewer cost + per-
  jurisdictional notarial cost).
- **Lazy vs eager.** Cedar evaluation eager. Inactive-Account-
  Manager threshold scanning is batch (per-cell daily). DSAR-
  cascade execution is queue-flushed per ADR-0276 §D-N.
- **Cache-invalidation policy.** Per-account Legacy-Contact
  roster cache TTL is 60 minutes; per-pack overlay cache TTL
  is 24 hours (refresh on per-jurisdiction statute update).
- **Profiling evidence link.** `tools/profiling/deceased-user-
  inheritance-baseline.json`.

#### §C.6. Code quality

- **Required test classes.** Unit (Cedar fragment, Shamir-share
  reconstitution, per-pack jurisdictional verifier), property
  (pre-mortem-wish precedence invariants), fuzz (death-
  certificate parser), load (mass-death event, e.g., natural-
  disaster surge), e2e (full flow including legacy-contact,
  inactive-account-manager, legal-rep paths).
- **Coverage floor.** ≥90% line, ≥80% branch (above standard
  ≥85%/≥75% due to inheritance stakes).
- **Lint passes.** `oya-check-cedar-fragment-soak`, `oya-check-
  pre-mortem-wish-precedence-invariant` (new lint),
  `oya-check-tenant-unilateral-lockout-forbid-invariant` (new
  lint), `oya-check-spiffe-uri-conformance`, `oya-check-audit-
  event-class-registration`, `oya-check-naming-justification-
  block`.
- **Type-strictness.** Rust `deny(warnings) + deny(missing_docs)
  + deny(unsafe_op_in_unsafe_fn)`; TypeScript surface (Legacy-
  Contact admin UI) `strict + noUncheckedIndexedAccess`.
- **SemVer + ABI policy.** Per ADR-0258. The shared crate's
  `LegacyContact`, `InactiveAccountManager`, `LegalRepCourtOrder`,
  `PreMortemWish`, `InheritanceDsarCascade` traits are public-
  stable.

## Detailed mechanics

### §D. Detailed mechanics

#### §D-1. Legacy-Contact (Apple-class) — per-account pre-designation flow

1. **Pre-designation.** At account setup OR via account preferences,
   the user designates ≤5 Legacy-Contacts. Each Legacy-Contact is
   identified by:
   - oyatie principal (preferred) OR external verified email +
     verified phone (for cross-platform Legacy-Contacts).
   - Per-Legacy-Contact data-subset designation (e.g., "legacy-
     contact-1 receives photos + notes; legacy-contact-2 receives
     financial records").
   - Per-Legacy-Contact access-key fragment (Shamir-shared with
     2-of-N threshold, per ADR-0247 break-glass).
2. **Per-Legacy-Contact access-key issuance.** Each Legacy-Contact
   receives their fragment via verified secondary channel + an
   advisory to store it securely (e.g., printed + stored in a
   safe-deposit box, per Apple's best-practice).
3. **Post-death claim.** Upon the user's death, the Legacy-Contact
   submits:
   - Death certificate (notarized; per-pack jurisdictional
     verification chain).
   - Their access-key fragment.
   - Identity verification (gov-issued ID + selfie liveness).
4. **Per-pack jurisdictional verification.** The per-pack
   verifier (per §D-5 overlay) verifies the death certificate
   against the per-jurisdiction vital-statistics registry (e.g.,
   US per-state vital-records office, EU per-member-state civil
   registry, KR Ministry of Justice family-relation certificate).
5. **Access-key reconstitution.** With 2-of-N fragments, the
   per-account access-key is reconstituted; the Legacy-Contact's
   designated data-subset is decrypted.
6. **DSAR cascade.** Per ADR-0276 §D-N, the per-Legacy-Contact
   data-subset is exported in the canonical portable format +
   delivered to the Legacy-Contact via verified secondary
   channel.
7. **Audit emission.** `DeceasedUserLegacyContactAttestationVerified`
   + `DeceasedUserDsarCascadeExecuted` events emitted per
   ADR-0263.

#### §D-2. Cedar fragment — `policy/deceased-user-inheritance.cedar`

```cedar
// policy/deceased-user-inheritance.cedar
// Per ADR-0302 deceased-user inheritance doctrine.
// Soak window: per ADR-0294 (≥60s before promotion).

// Permit Legacy-Contact access via attested chain.
permit(
  principal,
  action == Action::"access_deceased_user_data",
  resource
)
when {
  principal.role == "legacy-contact" &&
  context has death_certificate_attestation_verified &&
  context.death_certificate_attestation_verified == true &&
  context has legacy_contact_access_key_reconstituted &&
  context.legacy_contact_access_key_reconstituted == true &&
  context has per_pack_jurisdictional_verification_passed &&
  context.per_pack_jurisdictional_verification_passed == true &&
  context.data_subset in
    resource.account.legacy_contact_data_subset_designation[principal.id]
};

// Permit Inactive-Account-Manager firing.
permit(
  principal in PrincipalGroup::"oyatie_inheritance_substrate",
  action == Action::"fire_inactive_account_manager",
  resource
)
when {
  resource.account.lifecycle_state == "ACTIVE" &&
  context.last_activity_at + resource.account.inactive_account_manager_threshold
    < context.now &&
  context.advance_warning_count >= 3 &&
  context.last_advance_warning_at + 30days < context.now
};

// Permit legal-rep access via court order.
permit(
  principal,
  action == Action::"access_deceased_user_data",
  resource
)
when {
  principal.role == "legal-representative" &&
  context has court_order_attestation_verified &&
  context.court_order_attestation_verified == true &&
  context has death_certificate_attestation_verified &&
  context.death_certificate_attestation_verified == true &&
  context has ombudsman_quorum_members_count &&
  context.ombudsman_quorum_members_count >= 2 &&
  context has per_pack_jurisdictional_verification_passed &&
  context.per_pack_jurisdictional_verification_passed == true
};

// FORBID tenant unilateral lockout of heirs.
forbid(
  principal in PrincipalGroup::"tenant_admin",
  action in [
    Action::"delete_deceased_user_account",
    Action::"restrict_heir_access",
    Action::"deny_inheritance_claim"
  ],
  resource
)
when {
  resource.account.lifecycle_state in [
    "DECEASED_CONFIRMED",
    "INACTIVE_ACCOUNT_MANAGER_FIRED",
    "LEGAL_REP_COURT_ORDER_GRANTED"
  ] &&
  context.heir_claim_active == true
};

// Pre-mortem wish precedence (within per-pack statutory bound).
permit(
  principal in PrincipalGroup::"oyatie_inheritance_substrate",
  action == Action::"execute_pre_mortem_wish",
  resource
)
when {
  resource.account.lifecycle_state == "DECEASED_CONFIRMED" &&
  resource.account.pre_mortem_wish in [
    "export_to_legacy_contact",
    "delete",
    "archive_pseudonymously",
    "memorialize"
  ] &&
  // Per-pack statutory bound check.
  (resource.account.pre_mortem_wish != "delete" ||
   !resource.tenant.active_compliance_packs.containsAny([
     "pack-kr-forced-heirship",
     "pack-eu-forced-heirship-de",
     "pack-eu-forced-heirship-fr",
     "pack-eu-forced-heirship-it"
   ]) ||
   context.forced_heirship_satisfied == true)
};

// FORBID inheritance to shared-credential-abuser principals.
forbid(
  principal,
  action == Action::"access_deceased_user_data",
  resource
)
when {
  principal.role == "shared-credential-abuser" &&
  resource.account.lifecycle_state == "SHELTER_MODE_ACTIVE_DECEASED"
};
```

#### §D-3. Inactive-Account-Manager (Google-class) — state machine

```
        ┌──────────────────────────┐
        │   ACTIVE                  │
        └──────┬───────────────────┘
               │ no-activity for
               │ per-account threshold
               ▼
        ┌──────────────────────────┐
        │   INACTIVE_WARNING_       │
        │   STAGE_1 (T+0d)          │
        │   - email warning #1 sent │
        └──────┬───────────────────┘
               │ no-activity for 14d
               ▼
        ┌──────────────────────────┐
        │   INACTIVE_WARNING_       │
        │   STAGE_2 (T+14d)         │
        │   - email warning #2 +    │
        │     verified secondary    │
        │     channel warning sent  │
        └──────┬───────────────────┘
               │ no-activity for 30d
               ▼
        ┌──────────────────────────┐
        │   INACTIVE_WARNING_       │
        │   STAGE_3 (T+30d)         │
        │   - email warning #3 +    │
        │     designated beneficiaries│
        │     warning sent          │
        └──────┬───────────────────┘
               │ no-activity for 7d
               ▼
        ┌──────────────────────────┐
        │   INACTIVE_ACCOUNT_       │
        │   MANAGER_FIRED (T+37d)   │
        │   - designated action     │
        │     executed              │
        │     (notify+grant,        │
        │      delete, archive,     │
        │      memorialize)         │
        └──────────────────────────┘
```

Per-account designated action options:
- **Notify+grant.** Each designated beneficiary receives a per-
  beneficiary subset of the data (per-account designation).
- **Delete.** Account is deleted (within per-pack statutory bound;
  e.g., not deletable if forced-heirship pack active).
- **Archive-pseudonymously.** Account is moved to pseudonymity-
  class principal scope per ADR-0300 §D-3; data is preserved but
  the per-account identifier is replaced with a pseudonym hash.
- **Memorialize.** Per-pack memorialization (per the Facebook-
  class pattern); profile preserved + locked + memorial Legacy-
  Contact designated.

#### §D-4. Legal-rep court-order ingress (Microsoft-class)

1. **Per-pack legal-rep submission.** Per-pack legal-rep submits:
   - Court-order document (signed by per-jurisdiction probate
     court).
   - Death certificate (notarized; per-pack jurisdictional
     verification).
   - Legal-rep identification (per-jurisdiction bar association
     verification OR per-pack notarial verification).
   - Inheritance claim documentation (per-jurisdiction probate-
     court paperwork).
2. **Per-pack ombudsman 2-member quorum.** Per ADR-0247 break-
   glass; two distinct ombudsman members independently verify
   each attestation.
3. **Per-pack jurisdictional verification.** Per the per-pack
   overlay (§D-5):
   - Court-order document verified against per-jurisdiction court
     record system.
   - Death certificate verified against per-jurisdiction vital-
     statistics registry.
   - Legal-rep verified against per-jurisdiction bar association
     OR per-jurisdiction notarial registry.
   - Inheritance claim verified against per-jurisdiction probate
     court.
4. **DSAR cascade.** Per ADR-0276 §D-N; the per-account data is
   exported in canonical portable format + delivered to the legal-
   rep per the court-order specification.
5. **Per-pack regulator notification.** Per ADR-0251 + per-pack
   `breach_notification_workflow_id` (where the inheritance
   triggers regulatory notification, e.g., per RUFADAA US-state
   probate-court reporting).
6. **Audit emission.** `DeceasedUserLegalRepAccessGranted` +
   `DeceasedUserDsarCascadeExecuted` events.

#### §D-5. Per-pack inheritance overlay roster

Each pack adds an overlay at `packs/<pack-slug>/policy/deceased-
user-inheritance-overlay.cedar`. Active roster:

| # | Pack slug | Statute | Forced-heirship | Court-order verification | DSAR-cascade timing |
|---:|---|---|---|---|---|
| 1 | `pack-us-rufadaa-<state>` | Revised Uniform Fiduciary Access to Digital Assets Act (~47 US states; per-state versions) | No (testamentary freedom) | Per-state probate court | ≤30 days after attestation |
| 2 | `pack-eu-digital-heritage-de` | German Civil Code §§1922-2385 (Inheritance Law) + EU Digital Heritage Directive transposition | Yes (Pflichtteil; ≥50% to spouse + children) | Nachlassgericht (probate court) | ≤45 days |
| 3 | `pack-eu-digital-heritage-fr` | French Code Civil Art. 720-724 + EU Digital Heritage | Yes (réserve héréditaire) | Tribunal Judiciaire | ≤30 days |
| 4 | `pack-eu-digital-heritage-it` | Italian Civil Code Art. 456-768 + EU Digital Heritage | Yes (legittima) | Tribunale | ≤45 days |
| 5 | `pack-kr-civil-code-inheritance` | Korean Civil Code Article 1000-1118 + Bukhumeolban (forced-heirship) | Yes (yuryubun; ≥50% to spouse + children) | Family Court | ≤30 days |
| 6 | `pack-jp-civil-code-inheritance` | Japanese Civil Code Article 882-1054 + Family Court order | Yes (iryūbun; ≥50% to legal heirs) | Family Court | ≤30 days |
| 7 | `pack-uk-inheritance-trustees-powers-act-2014` | UK Inheritance and Trustees' Powers Act 2014 + Inheritance (Provision for Family and Dependants) Act 1975 | Partial (Provision for Family Act) | Probate Court | ≤30 days |
| 8 | `pack-au-succession-act-nsw` | Succession Act 2006 (NSW) + per-state succession acts | Partial (Family Provision claim) | Supreme Court | ≤30 days |

Per-pack overlay example (`packs/pack-kr-civil-code-inheritance/
policy/deceased-user-inheritance-overlay.cedar`):

```cedar
// Per ADR-0302 + pack-kr-civil-code-inheritance overlay.
permit(
  principal in PrincipalGroup::"oyatie_inheritance_substrate",
  action == Action::"execute_pre_mortem_wish",
  resource
)
when {
  resource.account.pre_mortem_wish == "delete" &&
  // KR forced-heirship: cannot delete if spouse or children
  // exist + their forced-heirship claim is unsatisfied.
  context.forced_heirship_satisfied == false
};

// FORBID pre-mortem wish "delete" when forced-heirship
// unsatisfied.
forbid(
  principal,
  action == Action::"execute_pre_mortem_wish",
  resource
)
when {
  resource.account.pre_mortem_wish == "delete" &&
  context.forced_heirship_satisfied == false &&
  resource.tenant.active_compliance_packs.contains(
    "pack-kr-civil-code-inheritance"
  )
};
```

#### §D-6. Pre-mortem wish state machine + UX

The deceased's pre-mortem wish is recorded at account setup OR
via account preferences. The state machine:

```
        ┌────────────────────────┐
        │   NO_WISH_RECORDED      │
        │   (default; tenant-     │
        │    policy default       │
        │    applies on death)    │
        └──────┬─────────────────┘
               │ user explicitly
               │ records wish
               ▼
        ┌────────────────────────┐
        │   WISH_RECORDED         │
        │   - export_to_legacy_   │
        │     contact OR          │
        │   - delete OR           │
        │   - archive_pseudonymously OR │
        │   - memorialize         │
        └────────────────────────┘
```

Per-tenant policy default on `NO_WISH_RECORDED` (per-tenant
configuration):
- Consumer-tier default: memorialize (preserve + lock).
- Business-tier default: per-pack regulator default (e.g.,
  HIPAA-US: archive-pseudonymously after 7-year-retention).
- Per-pack regulator override: forced-heirship packs default
  to export_to_legacy_contact + per-pack inheritance claim
  process.

#### §D-7. DSAR-cascade per ADR-0276 + per-pack data-residency

Per ADR-0276 §D-N, the inheritance flow executes the DSAR-cascade:

1. **Trigger.** Death-confirmed (legacy-contact OR inactive-
   account-manager OR legal-rep) + pre-mortem wish resolved.
2. **Per-account data enumeration.** Cross-µservice enumeration of
   the deceased user's data (per ADR-0276 §D-N + per-tenant
   audience-type scope).
3. **Per-pack data-residency.** Per-pack data-residency hard-stop
   prevents export across regulator-forbidden boundaries.
4. **Canonical portable format.** Per ADR-0276 §D-N canonical
   portable format (JSON + per-µservice schema + binary
   attachments).
5. **Delivery.** Per-Legacy-Contact verified secondary channel OR
   per-legal-rep court-order specification OR per-pre-mortem-wish
   destination.
6. **Audit emission.** `DeceasedUserDsarCascadeExecuted` event
   per ADR-0263.

#### §D-8. Per-µservice ARCHITECTURE.md §deceased-user-inheritance section

Every µservice serving consumer / business accounts SHALL include
in ARCHITECTURE.md:
1. **Surface inventory.** Which surfaces serve inheritance (e.g.,
   REST `/v1/deceased-user-inheritance/submit-death-certificate`,
   `/v1/deceased-user-inheritance/inactive-account-manager-fire`,
   `/v1/deceased-user-inheritance/legal-rep-court-order`).
2. **Legacy-Contact attestation chain wiring.** Cite per-pack
   verification chain.
3. **Inactive-Account-Manager configuration.** Cite per-account
   threshold range + per-account designated-action options.
4. **Legal-rep court-order ingress configuration.** Cite per-pack
   ombudsman roster + per-pack jurisdictional verification chain.
5. **Pre-mortem wish state-machine reference.**
6. **DSAR-cascade integration per ADR-0276 reference.**
7. **Cedar fragment reference.** Cite `policy/deceased-user-
   inheritance.cedar` + per-pack overlays.
8. **Audit-event-class emission.** Cite the seven new classes.

#### §D-9. Multispectrum review v2.4.0 wiring

Per ADR-0243 §D-8: F1 (security: false death claim defended via
cryptographic attestation chain), F2 (privacy: per-Legacy-Contact
data-subset scope + per-pack data-residency), F3 (reliability:
per-pack ombudsman queue does not saturate), F4 (performance:
≤5-day P95 SLA), F5 (cost: per-flow + per-ombudsman-hour cost),
F6 (operability: per-pack runbook), F7 (compliance: per-pack
inheritance overlay coverage), F8 (user safety: pre-mortem
wish + heir rights), F9 (accessibility: WCAG 2.2 AAA on Legacy-
Contact admin UI), M1 (meta-policy: Cedar chain ordering), M2
(meta-architecture: substrate primitive), A1-A7 (own-policy
adherence).

#### §D-10. Failure-mode tree — explicit walk-through

(FM-1..FM-12 from §A.3.)

- **FM-1 unreported death:** §D-3 inactive-account-manager fires
  on threshold without explicit death notification.
- **FM-2 false death claim:** §D-1 attestation chain (notarized
  death certificate + per-pack jurisdictional verification +
  ombudsman 2-member quorum).
- **FM-3 Legacy-Contact cannot prove identity:** §D-1 per-pack
  jurisdictional alternative + per-pack ombudsman escalation.
- **FM-4 conflicting heir claims:** §D-4 per-pack legal-rep
  court-order ingress + per-pack jurisdictional adjudication.
- **FM-5 pre-mortem-wish vs forced-heirship conflict:** §D-2
  Cedar fragment per-pack statutory bound + §D-5 per-pack
  overlay (e.g., KR forced-heirship FORBID).
- **FM-6 tenant unilateral lockout of heirs:** §D-2 Cedar
  fragment FORBID + cross-tenant ombudsman escalation.
- **FM-7 forged legal-rep court-order:** §D-4 per-pack ombudsman
  2-member quorum + per-jurisdiction court record verification +
  per-jurisdiction notary verification + per-pack regulator
  notification.
- **FM-8 Inactive-Account-Manager false-fires:** §D-3 ≥3
  advance-warning over ≥30 days + opt-in extension + per-pack
  regulator-mandated extension (e.g., US Servicemembers Civil
  Relief Act).
- **FM-9 deceased survivor's SHELTER_MODE_ACTIVE:** §D-2 Cedar
  FORBID on shared-credential-abuser inheritance + cross-ref
  ADR-0301 §D-7 unilateral lockout preserved through inheritance.
- **FM-10 cross-pack jurisdictional conflict:** §D-5 per-pack
  overlay + Hague Convention domicile-based jurisdiction.
- **FM-11 deceased's whistleblower submission:** Cross-ref
  ADR-0300 §D-1 sealed-sender envelope survives death; per-
  pack ombudsman holds sealed-key share; Legacy-Contact does
  NOT see submission content.
- **FM-12 minor's data:** Cross-ref ADR-0292 + per-pack minor-
  protection overlay + per-jurisdiction parental-rights
  resolution.

## Implementation footprint

### §E. Implementation footprint

#### §E.1. New crate: `crates/oya-shared-deceased-user-inheritance/`

```text
crates/oya-shared-deceased-user-inheritance/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs
│   ├── legacy_contact/
│   │   ├── mod.rs
│   │   ├── pre_designation.rs
│   │   ├── shamir_access_key.rs
│   │   ├── death_certificate_verifier.rs
│   │   └── post_death_claim.rs
│   ├── inactive_account_manager/
│   │   ├── mod.rs
│   │   ├── state_machine.rs
│   │   ├── advance_warning_emitter.rs
│   │   └── designated_action_executor.rs
│   ├── legal_rep_court_order/
│   │   ├── mod.rs
│   │   ├── court_order_verifier.rs
│   │   ├── jurisdiction_verifier.rs
│   │   └── ombudsman_quorum.rs
│   ├── pre_mortem_wish/
│   │   ├── mod.rs
│   │   ├── state_machine.rs
│   │   ├── precedence_resolver.rs
│   │   └── per_pack_statutory_bound.rs
│   ├── dsar_cascade/
│   │   ├── mod.rs
│   │   ├── per_account_data_enumerator.rs
│   │   └── canonical_portable_format.rs
│   ├── memorialization/
│   │   ├── mod.rs
│   │   └── facebook_class_lock.rs
│   ├── per_pack_overlay/
│   │   ├── mod.rs
│   │   ├── us_rufadaa.rs
│   │   ├── eu_digital_heritage_de.rs
│   │   ├── eu_digital_heritage_fr.rs
│   │   ├── eu_digital_heritage_it.rs
│   │   ├── kr_civil_code_inheritance.rs
│   │   ├── jp_civil_code_inheritance.rs
│   │   ├── uk_inheritance_trustees_powers_act_2014.rs
│   │   └── au_succession_act_nsw.rs
│   ├── audit.rs
│   ├── cedar.rs
│   ├── metrics.rs
│   ├── traces.rs
│   └── error.rs
└── tests/
    ├── legacy_contact_attestation_e2e.rs
    ├── inactive_account_manager_threshold_firing.rs
    ├── legal_rep_court_order_quorum.rs
    ├── pre_mortem_wish_precedence.rs
    ├── per_pack_forced_heirship.rs
    ├── tenant_unilateral_lockout_forbid.rs
    └── dsar_cascade_execution.rs
```

Public surface:

```rust
pub trait LegacyContact: Send + Sync {
    fn pre_designate(&self, account: &AccountId,
        legacy_contacts: &[LegacyContactDesignation])
        -> Result<LegacyContactReceipt, LegacyContactError>;
    fn submit_post_death_claim(&self,
        account: &AccountId,
        claim: &PostDeathClaim)
        -> Result<DsarCascadeReceipt, LegacyContactError>;
}

pub trait InactiveAccountManager: Send + Sync {
    fn configure(&self, account: &AccountId,
        threshold: &InactivityThreshold,
        beneficiaries: &[BeneficiaryDesignation],
        action: &DesignatedAction)
        -> Result<InactiveAccountManagerReceipt, IamError>;
    fn fire(&self, account: &AccountId)
        -> Result<IamFiringReceipt, IamError>;
}

pub trait LegalRepCourtOrder: Send + Sync {
    fn submit(&self, court_order: &CourtOrder,
        ombudsman_quorum: &OmbudsmanQuorum)
        -> Result<DsarCascadeReceipt, LegalRepError>;
}

pub trait PreMortemWish: Send + Sync {
    fn record(&self, account: &AccountId, wish: &Wish)
        -> Result<WishReceipt, WishError>;
    fn execute(&self, account: &AccountId)
        -> Result<DsarCascadeReceipt, WishError>;
}

pub trait InheritanceDsarCascade: Send + Sync {
    fn execute(&self, account: &AccountId,
        destination: &CascadeDestination)
        -> Result<DsarCascadeReceipt, CascadeError>;
}
```

#### §E.2. Cedar fragment

`microservices/<ms>/policy/deceased-user-inheritance.cedar` per
§D-2.

#### §E.3. IaC manifest

`microservices/<ms>/iac/<env>-deceased-user-inheritance.yaml`
declares per-tenant pack overlays, per-account threshold ranges,
per-pack ombudsman SLA, observability hooks.

#### §E.4. Spec: `specs/deceased-user-inheritance.json`

JSON Schema per documentation-rigor.md §2 spec rigor.

#### §E.5. CI lanes per §B Decision

```text
.github/workflows/oya-governance-deceased-user-inheritance.yml
.github/workflows/oya-governance-deceased-user-legacy-contact-attestation.yml
.github/workflows/oya-governance-deceased-user-inactive-account-manager.yml
.github/workflows/oya-governance-deceased-user-legal-rep-court-order.yml
.github/workflows/oya-governance-deceased-user-pre-mortem-wish-honored.yml
.github/workflows/oya-governance-deceased-user-per-pack-inheritance-overlay.yml
.github/workflows/oya-governance-deceased-user-cedar-fragment-present.yml
```

## Migration

### §F. Migration plan

#### §F.1. Phase 0 — Doctrine acceptance (2026-05-20 — 2026-05-27)

ADR accepted in text. Shared crate skeleton + Cedar fragment +
per-pack overlay skeleton land. CI lanes promote to advisory.

#### §F.2. Phase 1 — Per-pack overlay onboarding (2026-05-27 — 2026-07-15)

Onboard the 8 packs per §D-5. Per-pack legal-team training + per-
pack ombudsman roster provisioning + per-jurisdiction court-record
+ notarial-registry integration negotiation.

#### §F.3. Phase 2 — Per-account Legacy-Contact + Inactive-Account-Manager UX (2026-06-01 — 2026-07-15)

Per-account preference UX for Legacy-Contact designation + IAM
configuration + pre-mortem-wish recording. Per-account migration
of existing accounts (opt-in onboarding).

#### §F.4. Phase 3 — Per-µservice wiring (2026-07-01 — 2026-08-15)

Every µservice serving consumer / business accounts adds the
§D-8 ARCHITECTURE.md section + Cedar fragment + IaC manifest.
CI lanes promote to BLOCKER on 2026-08-15.

#### §F.5. Phase 4 — BLOCKER promotion (2026-08-15)

The seven CI lanes promote to BLOCKER.

#### §F.6. Rollback path

Per-fragment Cedar rollback within ≤5 minutes per ADR-0294. Per-
pack overlay rollback within ≤15 minutes. Platform-wide rollback
within ≤30 minutes.

Rollback DOES NOT halt in-flight inheritance flows; they continue
under pre-rollback fragment. Per-account Legacy-Contact
pre-designations are unaffected.

## References

### §G. References

#### §G.1. Hyperscaler precedents

- Apple Legacy Contact: `support.apple.com/en-us/HT212513`
- Apple Digital Legacy Program: `support.apple.com/en-us/HT212361`
- Google Inactive Account Manager: `myaccount.google.com/inactive`
- Facebook Memorialization + Legacy Contact: `facebook.com/help/1568013990080948`
- Microsoft Next of Kin Access: `support.microsoft.com/en-us/account-billing/access-outlook-com-onedrive-and-other-microsoft-services-when-someone-has-died-ebbd2860-917e-4b39-9913-212362da6b2f`
- Twitter/X Memorialization: `help.x.com/en/forms/account-access/deactivate-or-close-account/request-deactivation-deceased-user`
- Coinbase Inheritance Beneficiary: `help.coinbase.com/en/coinbase/managing-my-account/identity-verification/inheritance-beneficiary`

#### §G.2. Regulatory anchors

- **US — Revised Uniform Fiduciary Access to Digital Assets Act
  (RUFADAA)** (NCCUSL 2015; adopted by ~47 US states as of 2026).
- **US — Servicemembers Civil Relief Act (50 USC §§3901-4043)**
  (extends inactivity thresholds for deployed military).
- **EU — Digital Heritage** (per-member-state transposition;
  e.g., German Civil Code §§1922-2385).
- **EU — Hague Convention on the Law Applicable to Succession to
  the Estates of Deceased Persons** (1989; private international
  law for cross-jurisdictional inheritance).
- **EU — GDPR Article 17** (right to erasure — death is not
  expressly enumerated; data subject's rights cease at death per
  Recital 27, but member-state law may extend).
- **EU — Germany — Bürgerliches Gesetzbuch (BGB) §1922-2385**
  (Inheritance Law + Pflichtteil forced-heirship).
- **EU — France — Code Civil Art. 720-724** (succession + réserve
  héréditaire forced-heirship).
- **EU — Italy — Codice Civile Art. 456-768** (succession +
  legittima forced-heirship).
- **KR — Civil Code Article 1000-1118 (Inheritance Act)**
  (yuryubun forced-heirship; ≥50% to spouse + children).
- **KR — Real Property Registration Act** (death-certificate
  verification chain).
- **JP — Civil Code Article 882-1054** (inheritance; iryūbun
  forced-heirship).
- **JP — Family Court Act** (Family Court inheritance
  adjudication).
- **UK — Inheritance and Trustees' Powers Act 2014**.
- **UK — Inheritance (Provision for Family and Dependants) Act
  1975** (Provision claim).
- **AU — Succession Act 2006 (NSW)** + per-state succession
  acts (Family Provision claim).

#### §G.3. Keystone bundle 2026-05-20 cross-references

- **ADR-0297** (abuse-defence baseline): observation-only on
  inheritance ingress surface.
- **ADR-0298** (emergency-services bypass): imminent-threat
  reported during inheritance flow transitions to bypass.
- **ADR-0299** (account-recovery resilience): F3-class delegated-
  trusted-contact + F4-class ombudsman for legacy-contact +
  legal-rep paths.
- **ADR-0300** (whistleblower-press-anonymity): pseudonymity
  scope for archive-pseudonymously wish path + deceased's
  whistleblower submissions preserved per sealed-sender envelope.
- **ADR-0301** (survivor-safety): shared-credential-abuser
  cannot inherit; shelter-mode preserved through inheritance.
- **ADR-0242** (oyatie-is-a-tenant): platform admins use same
  inheritance flow.
- **ADR-0243** (Cedar universal gate): inheritance is Cedar
  policy.
- **ADR-0244** (tenant scoping primitive): adds five lifecycle_
  state values + two role values.
- **ADR-0246** (policy-engine library-first): library-first
  Cedar carries the fragment.
- **ADR-0247** (break-glass): per-pack ombudsman 2-member quorum.
- **ADR-0248** (cellular architecture): per-cell partitioning.
- **ADR-0250** (build-ahead-of-certification): built certified-
  shape day one.
- **ADR-0251** (compliance packs): per-pack overlay.
- **ADR-0252** (HLC + TrueTime): HLC for per-pack inactivity
  threshold timing.
- **ADR-0253** (HTTP/3 + QUIC + ECH + PQC): ECH-enabled + PQC
  hybrid KEX for inheritance data transfer.
- **ADR-0263** (observability emission contract): seven new
  audit-event classes.
- **ADR-0272** (cookie consent per-purpose): purpose-scoped
  cookies on Legacy-Contact post-mortem access.
- **ADR-0273** (per-tenant DKIM/SPF/DMARC): per-tenant DKIM-
  signed advance-warning emails.
- **ADR-0276** (backup portability GDPR Art. 20): THE DSAR-
  cascade primitive that executes inheritance data transfer.
- **ADR-0280** (substrate-of-substrate): depends on cedar-
  evaluator + audit-emit + account-recovery + anonymity-substrate.
- **ADR-0284** (platform-owner name indirection): namespace
  parameterized.
- **ADR-0292** (minor user doctrine): deceased minor + parental-
  rights overlay.
- **ADR-0293** (meta-trust-root): Legacy-Contact attestation
  keys rooted.
- **ADR-0294** (Cedar fragment soak): ≥60s soak window.
- **ADR-0295** (bootstrap CI SPIFFE + kill-switch): SPIFFE +
  kill-switch.
- **ADR-0296** (library-first credential sidecar): Legacy-
  Contact keys held in sidecar with ≤60s OpenBao TTL.

#### §G.4. Companion docs

- `docs/standards/documentation-rigor.md` §3.2.5 row 10.
- `docs/runbooks/deceased-user-legacy-contact-attestation.md`.
- `docs/runbooks/deceased-user-inactive-account-manager-firing.md`.
- `docs/runbooks/deceased-user-legal-rep-court-order-ingress.md`.
- `docs/runbooks/deceased-user-pre-mortem-wish-execution.md`.
- `docs/runbooks/deceased-user-dsar-cascade.md`.
- `docs/runbooks/per-pack-inheritance-jurisdictional-verification.md`.

#### §G.5. Cross-back-pointer follow-ups for existing ADRs

- **ADR-0297** (abuse-defence baseline): add §D-N noting
  observation-only on inheritance ingress.
- **ADR-0298** (emergency-services bypass): add §D-N noting
  imminent-threat reported during inheritance flow transitions
  to bypass.
- **ADR-0299** (account-recovery resilience): add §D-N noting
  F3 + F4 factors used in inheritance paths.
- **ADR-0300** (whistleblower-press-anonymity): add §D-N noting
  pseudonymity scope for archive-pseudonymously wish + sealed-
  sender envelope survives death.
- **ADR-0301** (survivor-safety): add §D-N noting shared-
  credential-abuser cannot inherit.
- **ADR-0263** (observability emission contract): register the
  seven new audit-event classes.
- **ADR-0247** (break-glass): cross-reference per-pack ombudsman
  inheritance quorum.
- **ADR-0244** (tenant scoping primitive): cross-reference the
  five new lifecycle_state values + two role values.
- **ADR-0276** (backup portability GDPR Art. 20): cross-reference
  the inheritance DSAR-cascade primitive (the inheritance flow IS
  a DSAR-cascade consumer).
- **ADR-0292** (minor user doctrine): cross-reference deceased-
  minor parental-rights overlay.

## Change log

### §H. Change log

| Date | Author | Change |
|---|---|---|
| 2026-05-20 | council-architecture + axis-deceased-user-inheritance | Initial Proposed status; bundled with the keystone-bundle 2026-05-20 foundational doctrine as the critical-path-doctrine-cluster-row-10 keystone. Authored per documentation-rigor.md §3.2.5 row 10. Cross-references ADR-0297 + ADR-0298 + ADR-0299 + ADR-0300 + ADR-0301 + the entire keystone bundle 2026-05-20. |
