---
id: ADR-0301
status: Accepted
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
  - axis-survivor-safety
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
related_specs:
  - /specs/platform-architecture.json
  - /specs/microservices/identity.json
  - /specs/microservices/policy-engine.json
  - /specs/microservices/audit-chain.json
  - /specs/microservices/observability.json
  - /specs/survivor-safety-shelter-mode.json
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
keystone_position: critical-path-doctrine-cluster-row-8-survivor-safety-domestic-abuse-mode
purpose: >
  Codify the Survivor-Safety + Domestic-Abuse Mode doctrine — a
  per-account silent shelter mode, hide-from-shared-device option,
  per-tenant escape-plan, alert-when-checked-by-other-party,
  no-SMS-MFA-fallback (predictable to a shared-device abuser),
  survivor-unilateral-lockout-power-over-abuser-on-shared-credential,
  and audit-trail-visible-only-to-survivor that closes row 8 of
  the 30-row critical-path matrix in documentation-rigor.md §3.2.5.
  The bar is: a domestic-violence survivor (with stalkerware risk,
  controlling abuser sharing device or account credentials, financial
  abuse via shared accounts, post-separation digital harassment)
  retains exclusive operational control over their account without
  alerting the abuser, without leaving forensic traces visible on
  the shared device, and with the ability to unilaterally lock out
  the abuser even if the abuser holds shared credentials. Per
  documentation-rigor.md §3.2.5 row 8 — the safety/security/policy
  invariant: survivor has unilateral power to lock out abuser even
  if abuser holds shared credentials; audit visible only to
  survivor; the §3.2.5 anti-pattern call-out "Domestic-violence
  shelter mode shares audit trail with tenant admin → No. Survivor
  controls their audit visibility; tenant admin cannot stalk."
enforcement_status: advisory-until-2026-08-15-blocker-thereafter
enforced_by:
  - cloud-ci/Rust gate packet survivor-safety-shelter-mode-cedar-fragment-present
  - cloud-ci/Rust gate packet survivor-safety-audit-visibility-survivor-only
  - cloud-ci/Rust gate packet survivor-safety-no-sms-mfa-fallback
  - cloud-ci/Rust gate packet survivor-safety-shared-device-detection
  - cloud-ci/Rust gate packet survivor-safety-escape-plan-template-present
  - cloud-ci/Rust gate packet survivor-safety-unilateral-lockout-power
  - cloud-ci/Rust gate packet survivor-safety-stalkerware-pattern-detection
naming_justifications:
  - name: oya-shared-survivor-safety
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.survivor-safety
    justification: >
      Per ADR-0105 13-layer canonical enum row 5 (shared-substrate),
      the crate that exposes the silent-shelter-mode orchestrator,
      shared-device detector, escape-plan template engine,
      alert-when-checked-by-other-party emitter, no-SMS-MFA-fallback
      enforcer, unilateral-lockout-power router, and survivor-only
      audit-visibility filter belongs at the shared layer. Naming
      `oya-shared-survivor-safety` keeps the single-concern flat
      layout per ADR-0131 + ADR-0132. Drop-in companion to the
      keystone-bundle 2026-05-20 critical-path doctrine cluster.
  - name: oya-governance-survivor-safety
    layer: N/A (foundry-fitness aggregate CI lane)
    bnf_segments: oya.foundry-fitness.survivor-safety
    justification: >
      Aggregate CI fitness lane per ADR-0212 buildability doctrine;
      rolls up per-µservice declaration of the silent-shelter-mode,
      the shared-device detection, the no-SMS-MFA-fallback, the
      escape-plan template, the unilateral-lockout-power, the
      stalkerware-pattern detection, and the survivor-only audit-
      visibility filter.
  - name: oya-governance-survivor-safety-audit-visibility
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.survivor-safety-audit-visibility
    justification: >
      Per-µservice child lane verifying audit-trail rows tagged with
      the survivor-safety-shelter-mode flag are visible ONLY to the
      survivor + per-pack ombudsman + emergency-services principals
      (per ADR-0298) — never to tenant-admin or abuser-account.
  - name: oya-governance-survivor-safety-no-sms-mfa-fallback
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.survivor-safety-no-sms-mfa-fallback
    justification: >
      Per-µservice child lane verifying SMS-OTP fallback is DISABLED
      for accounts with the survivor-safety-shelter-mode flag set —
      SMS is predictable to a shared-device abuser per §3.2.5 row 8.
  - name: oya-governance-survivor-safety-shared-device-detection
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.survivor-safety-shared-device-detection
    justification: >
      Per-µservice child lane verifying shared-device detection (per
      device-fingerprint heterogeneity + per-session usage pattern)
      is wired and emits the SharedDeviceDetected event to the
      survivor's verified secondary channel.
  - name: oya-governance-survivor-safety-stalkerware-pattern-detection
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.survivor-safety-stalkerware-pattern-detection
    justification: >
      Per-µservice child lane verifying stalkerware-pattern detection
      (per known stalkerware signatures + behavioural anomaly +
      remote-access-trojan pattern) emits alerts on the survivor's
      verified secondary channel.
  - name: X-Oya-Shelter-Mode
    layer: N/A (HTTP request header naming)
    bnf_segments: X-Oya.Shelter-Mode
    justification: >
      Custom HTTP request header carrying the per-session shelter-
      mode flag (boolean) for accounts with the survivor-safety-
      shelter-mode active. The header is set by the per-cell
      attestation-verifier when the per-session principal carries
      the survivor-safety scope. Namespace prefix `X-Oya-` reserves
      the platform's header surface.
  - name: X-Oya-Survivor-Channel
    layer: N/A (HTTP request header naming)
    bnf_segments: X-Oya.Survivor-Channel
    justification: >
      Custom HTTP request header identifying the survivor's verified
      secondary channel (enum: BACKUP_EMAIL, TRUSTED_CONTACT,
      OMBUDSMAN_PORTAL, EMERGENCY_SERVICES_LIFELINE) — used to route
      alerts that the survivor's primary channel may be compromised
      by a shared-device abuser.
  - name: SurvivorSafetyShelterModeActivated
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: SurvivorSafety.ShelterModeActivated
    justification: >
      Audit-event-class emitted when an account enters silent shelter
      mode; visible ONLY to the survivor + per-pack ombudsman.
      Registered in ADR-0263 central registry per §3.2.2 consistency
      invariant. Distinct from AccountRecoveryGranted (ADR-0299)
      because shelter mode is a per-account state, not a recovery
      transition.
  - name: SurvivorSafetyShelterModeDeactivated
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: SurvivorSafety.ShelterModeDeactivated
    justification: >
      Audit-event-class emitted when an account exits silent shelter
      mode (survivor-initiated deactivation, ombudsman-confirmed).
  - name: SurvivorSafetyAbuserLockedOut
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: SurvivorSafety.AbuserLockedOut
    justification: >
      Audit-event-class emitted when the survivor exercises
      unilateral-lockout-power against a shared-credential abuser
      principal. Visible to survivor + per-pack ombudsman + (where
      applicable) emergency-services principals.
  - name: SurvivorSafetySharedDeviceDetected
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: SurvivorSafety.SharedDeviceDetected
    justification: >
      Audit-event-class emitted when shared-device detection fires
      (per device-fingerprint heterogeneity + per-session usage
      pattern signaling distinct concurrent users).
  - name: SurvivorSafetyStalkerwarePatternDetected
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: SurvivorSafety.StalkerwarePatternDetected
    justification: >
      Audit-event-class emitted when stalkerware-pattern detection
      fires (per known stalkerware signatures + behavioural anomaly
      + remote-access-trojan pattern).
  - name: SurvivorSafetyAlertEmittedToSecondaryChannel
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: SurvivorSafety.AlertEmittedToSecondaryChannel
    justification: >
      Audit-event-class emitted when an alert (account checked by
      other party, shared-device-detected, stalkerware-pattern-
      detected, escape-plan-trigger) is sent to the survivor's
      verified secondary channel.
  - name: SurvivorSafetyEscapePlanTriggered
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: SurvivorSafety.EscapePlanTriggered
    justification: >
      Audit-event-class emitted when the per-tenant escape-plan
      template fires (survivor-initiated panic-button OR auto-
      triggered by per-pack regulator threshold).
  - name: policy/survivor-safety.cedar
    layer: N/A (per-µservice Cedar fragment file)
    bnf_segments: policy.survivor-safety
    justification: >
      Canonical filename for the per-µservice survivor-safety Cedar
      fragment under the µservice's `policy/` directory per ADR-0246
      + ADR-0243 fragment-lifecycle conventions; single-concern
      naming keeps the policy directory's contract-by-name
      invariant.
  - name: spec/survivor-safety-shelter-mode.json
    layer: N/A (canonical shelter-mode state spec)
    bnf_segments: specs.survivor-safety-shelter-mode
    justification: >
      Canonical machine-readable spec for the survivor-safety
      shelter-mode state machine; declares states, transitions,
      preconditions, alert-emission obligations, escape-plan
      template structure.
  - name: SHELTER_MODE_ACTIVE
    layer: N/A (Account.lifecycle_state enum value per ADR-0244)
    bnf_segments: lifecycle_state.SHELTER_MODE_ACTIVE
    justification: >
      Account lifecycle_state enum extension per ADR-0244 §D-3;
      identifies accounts under active shelter mode. Distinct from
      ACTIVE / SUSPENDED / DELETED so per-pack ombudsman can audit
      shelter-mode population per pack.
  - name: shared-credential-abuser
    layer: N/A (Principal.role enum value per ADR-0244)
    bnf_segments: role.shared-credential-abuser
    justification: >
      Principal.role enum extension per ADR-0244 §D-3; identifies
      a principal that the survivor has flagged as a shared-
      credential abuser. The principal is locked out per §D-7
      below.
---

# ADR-0301: Survivor-Safety + Domestic-Abuse Mode

## Status

Proposed — 2026-05-20.

Bundled with the keystone-bundle 2026-05-20 foundational doctrine
synthesis as the **critical-path-doctrine-cluster-row-8-survivor-
safety-domestic-abuse-mode** keystone. Closes row 8 of the 30-row
critical-path matrix in `docs/standards/documentation-rigor.md`
§3.2.5.

Enforcement is `advisory-until-2026-08-15-blocker-thereafter`. The
doctrine accepts in text immediately; the seven CI lanes that
enforce it promote to BLOCKER on 2026-08-15 to give per-tenant
verified-secondary-channel onboarding, per-pack ombudsman
provisioning (cross-ref ADR-0299), per-µservice shared-device-
detector wiring (per device-fingerprint heterogeneity), per-µservice
stalkerware-pattern-detector wiring (per known stalkerware
signatures + behavioural anomaly), and per-µservice escape-plan
template authoring time to land. Until 2026-08-15, validators emit
findings without failing CI; post-2026-08-15 the lanes block
merge.

## Date

2026-05-20.

## Context

### §A. Why survivor-safety is a substrate primitive, not a per-µservice afterthought

Survivor-safety is a critical-path edge case (per documentation-
rigor.md §3.2.5 row 8) because the standard auth-defence pattern
— SMS-OTP fallback, shared-device-tolerant session, per-tenant
unified audit-trail, account-recovery via the same channel as the
adversary holds — actively HARMS the survivor. A survivor with
a controlling abuser sharing the same device (or sharing the
account credential, or having installed stalkerware) faces a
fundamentally different threat model from the generic-internet-
user threat model.

The pattern across mature survivor-safety substrates is unambiguous:

- **Apple Safety Check.** Apple's Safety Check (rolled out iOS
  16, refined in iOS 17 + iOS 18) provides a one-tap reset of
  shared-credential access: revokes Family Sharing, revokes
  iCloud sharing, revokes Find My access, revokes Photo Sharing,
  revokes shared Apple ID payment methods. Per Apple's published
  documentation (`support.apple.com/en-us/HT213014` Safety Check),
  the surface is designed for domestic-violence survivors to
  unilaterally lock out a shared-credential abuser. The surface
  is mode-aware: it presents an emergency exit when the user
  rapidly closes the surface (e.g., the abuser walks into the
  room). The reset is silent — the abuser is not notified of
  the access revocation. Per the Apple Personal Safety User
  Guide (`support.apple.com/en-us/guide/personal-safety/welcome/
  web`), Apple operates a dedicated personal-safety surface
  across iOS / macOS / iCloud / Find My / Family Sharing /
  Messages.
- **Google Privacy Checkup + Stalkerware Protection.** Google's
  Privacy Checkup (`myaccount.google.com/privacycheckup`)
  provides per-app sharing audit + per-device active-session
  list + sharing-revocation. Per Google's published documentation
  on stalkerware (`support.google.com/accounts/answer/6010255`),
  the Android substrate detects and warns the user on
  installations of known stalkerware (per the Coalition Against
  Stalkerware shared signature database; Google is a founding
  Coalition member).
- **Coalition Against Stalkerware.** The Coalition Against
  Stalkerware (`stopstalkerware.org`; founded 2019 by Kaspersky
  + Avira + EFF + Operation Safe Escape + National Network to End
  Domestic Violence + others) publishes the shared stalkerware-
  signature database used by every major anti-virus vendor
  (Kaspersky, Avira, AVG, ESET, Norton, McAfee, F-Secure, Trend
  Micro). Per the Coalition's published 2024 report, ~31,000
  unique stalkerware variants were detected across consumer
  devices in 2024 — up ~58% from 2020.
- **Signal Disappearing Messages + Lock Screen + Registration
  PIN.** Signal's domestic-violence-survivor features (per
  `support.signal.org/hc/en-us/categories/360002917252` Privacy
  & Security): per-conversation disappearing messages, per-chat
  screen lock, registration PIN required for account recovery
  (cannot be reset via phone-only — protects against SIM-swap-
  by-abuser), per-conversation hidden from app drawer.
- **Whisper Systems / Open Whisper Systems (now Signal) "Censored
  Account" feature.** Signal's safety-net feature suppresses
  notification preview on screen when the screen is in a
  "censored" mode — protects against abuser glimpsing notifications.
- **WhatsApp Disappearing Messages + Hide Chat.** WhatsApp's
  disappearing messages (per `faq.whatsapp.com/673193694148717`)
  + chat-hide-from-list (per `faq.whatsapp.com/433755712763408`)
  provide per-chat survivor-safety controls.
- **Apple Personal Safety User Guide.** Apple published a
  dedicated 75-page Personal Safety User Guide in 2022
  (`support.apple.com/en-us/guide/personal-safety/welcome/web`)
  with sections on: leave a relationship safely, protect against
  stalkerware, get help from the National Domestic Violence
  Hotline + Apple's emergency resources. The guide is the most
  thorough survivor-safety substrate published by a hyperscaler.
- **National Network to End Domestic Violence (NNEDV) Safety
  Net Project.** NNEDV's Safety Net (operational since 2002)
  publishes the technology-safety standard for the domestic-
  violence-services field. Per the Safety Net "Tech Safety App"
  and the Safety Net Best Practice Guidelines (annual; current
  revision 2024), the canonical survivor-safety primitives are:
  silent shelter mode + audit visible only to survivor + no
  SMS-MFA fallback + escape-plan template + unilateral lockout
  power + stalkerware detection. ADR-0301 inherits this template
  1:1.
- **Refuge Tech Safety (UK).** Refuge (the UK's largest domestic-
  abuse charity) operates a dedicated Tech Safety team that
  publishes per-platform survivor-safety guidance (per `refuge.
  org.uk/tech-safety`). Refuge's per-platform recommendations
  guide oyatie's per-pack overlay design.
- **Coinbase Survivor-Safety in financial accounts.** Coinbase's
  domestic-financial-abuse protections (per Coinbase's published
  Trust & Safety documentation) include: shared-credential
  detection on withdrawal patterns, per-account anti-coercion
  cool-down on high-value transfers, per-account "are you safe"
  prompts on anomalous behaviour. Financial coercion is a
  specific domestic-violence pattern; the same primitives apply
  to oyatie's payments + cloud-billing surfaces.

The corollary: **every internet-facing surface oyatie ships MUST
inherit survivor-safety from the substrate, not author it per-
µservice.** A µservice that authors its own shared-device
detector, its own audit-visibility filter, its own escape-plan
template is duplicating substrate primitives that the shared crate
already serves. That duplication is a `feedback_no_silent_
regression` violation (every µservice's survivor-safety drifts
independently); it is a `feedback_quality_performance_scalability_
bar` violation (one survivor's safety depends on the substrate-
wide detection signal, not per-µservice signals); and it is a
`feedback_autonomous_implementation_artifacts` violation (intern-
buildable means the doc surface is one substrate, not 46
µservice-private implementations).

The ADR-0301 survivor-safety doctrine closes this gap.

### §A.1. The row 8 mandate from §3.2.5 — verbatim

Per documentation-rigor.md §3.2.5 row 8 — "Domestic violence /
abuse survivor":

> Stalkerware risk; controlling abuser may share device. Silent
> shelter mode (incognito session); hide-from-shared-device option;
> per-tenant escape-plan + alert-when-checked-by-other-party; no
> SMS-MFA fallback (predictable). Safety/security/policy invariant:
> Survivor has unilateral power to lock out abuser even if abuser
> holds shared credentials; audit visible only to survivor.

Plus the anti-pattern call-out from documentation-rigor.md §3.2.5
"Forbidden anti-patterns":

> "Domestic-violence shelter mode shares audit trail with tenant
> admin" → No. Survivor controls their audit visibility; tenant
> admin cannot stalk.

The two clauses are load-bearing:
- **"Survivor has unilateral power to lock out abuser even if
  abuser holds shared credentials."** The survivor's authority over
  their account supersedes the abuser's possession of shared
  credentials. The substrate MUST provide the lockout primitive
  without requiring the survivor to prove identity to the abuser-
  accessible recovery flow (ADR-0299 §B Decision 1).
- **"Audit visible only to survivor."** The tenant-admin cannot
  see the survivor's audit-trail. This is novel — it inverts the
  default tenant-admin authority over their tenant's audit data.
  Per ADR-0301 §B Decision 5 below, the survivor-tagged audit rows
  are encrypted under the survivor's per-account DEK + visible
  only via the survivor's per-account decryption path.

### §A.2. Why the keystone bundle 2026-05-20 requires this as a substrate-level primitive

The keystone bundle's foundational ADRs intersect survivor-safety
as follows:

- **ADR-0242 (oyatie-is-a-tenant).** Survivor-safety applies to
  oyatie platform admins identically to tenant users — no carve-
  out. A platform admin in a domestic-abuse situation has the
  same shelter-mode primitive available.
- **ADR-0243 (Cedar universal gate).** Survivor-safety is a Cedar
  policy decision. The Cedar fragment FORBIDs tenant-admin access
  to survivor-tagged audit rows + FORBIDs SMS-MFA fallback on
  shelter-mode-active accounts.
- **ADR-0244 (tenant scoping primitive).** This ADR adds the
  `SHELTER_MODE_ACTIVE` Account.lifecycle_state enum value + the
  `shared-credential-abuser` Principal.role enum value per ADR-
  0244 §D-3.
- **ADR-0246 + amendment (policy-engine library-first).** Every
  µservice's library-first Cedar evaluator carries the survivor-
  safety Cedar fragment.
- **ADR-0247 (self-modification / break-glass).** Per-pack
  ombudsman path (cross-ref ADR-0299 §D-7) provides the human-
  reviewer SLA for survivor-initiated unilateral-lockout
  confirmation.
- **ADR-0248 (Amazon-shape cellular architecture).** Per-tenant
  shelter-mode state is partitioned per-cell; cross-cell shelter-
  mode preserves the survivor's exclusive audit-trail visibility.
- **ADR-0251 (compliance packs).** Each pack adds per-jurisdiction
  domestic-violence-survivor protections (e.g., US Violence
  Against Women Act (VAWA) 2022 reauthorization; UK Domestic
  Abuse Act 2021; EU Council of Europe Convention on preventing
  and combating violence against women — Istanbul Convention;
  KR Domestic Violence Crime Punishment Act; JP Spousal Violence
  Prevention Act 2001 + 2024 revision; AU Family Law Act 1975 +
  state-level domestic-violence-protection orders).
- **ADR-0253 (HTTP/3 + QUIC default + ECH + PQC).** Survivor-
  safety traffic uses ECH-enabled TLS (without ECH, the inner
  SNI could reveal the shelter-mode surface to a passive observer
  on the shared network).
- **ADR-0263 (observability emission contract).** Survivor-safety
  emits seven dedicated audit-event classes (per the naming-
  justifications); the rows are encrypted under the survivor's
  per-account DEK so tenant-admin cannot read them.
- **ADR-0272 (cookie consent per-purpose).** Shelter-mode sessions
  use ephemeral storage (no persistent cookies that the abuser
  could inspect on the shared device).
- **ADR-0273 (per-tenant DKIM/SPF/DMARC).** Alert emails to the
  survivor's verified secondary channel use per-tenant DKIM with
  metadata-minimized content (no shelter-mode-revealing subject
  lines).
- **ADR-0276 (backup portability GDPR Art. 20).** Survivor-tagged
  audit data is portable per ADR-0276 BUT the tenant-admin cannot
  initiate the export on behalf of the survivor; only the survivor
  can export.
- **ADR-0280 (substrate-of-substrate).** Survivor-safety depends
  on `oya-shared-cedar-evaluator` + `oya-shared-audit-emit` +
  `oya-shared-anonymity-substrate` (ADR-0300 — the survivor uses
  pseudonymity scope to bypass shared-device behavioural-fingerprint
  correlation).
- **ADR-0284 (platform-owner name indirection).** Namespace
  `oyatie.survivor-safety.*` parameterized.
- **ADR-0292 (minor user doctrine).** A minor survivor (per the
  §3.2.5 row 9 anti-pattern call-out: "Crisis-hotline minors
  require parental consent → No. Child safety > parental control
  on mandatory-reporting paths") bypasses parental control for
  the shelter-mode primitive.
- **ADR-0293 (meta-trust-root).** Survivor's per-account DEK
  rooted at the meta-trust-root.
- **ADR-0294 (Cedar fragment soak).** ≥60s soak window respected.
- **ADR-0295 (bootstrap CI SPIFFE + kill-switch).** SPIFFE
  identity for the survivor-safety µservice + per-cell kill-
  switch.
- **ADR-0296 (library-first credential sidecar).** Survivor's
  per-account DEK held in sidecar with ≤60s OpenBao TTL.
- **ADR-0297 (abuse-defence baseline).** Abuse-defence is
  observation-only on shelter-mode-active traffic (no behavioural-
  fingerprint forwarding that could leak the shelter-mode-active
  state to a tenant-admin or to the abuser).
- **ADR-0298 (emergency-services bypass).** An imminent-threat
  trigger transitions the survivor to the emergency-services
  bypass (911 / 112 / 119 / 988 etc.); shelter-mode preserves
  the bypass.
- **ADR-0299 (account-recovery resilience).** Account-recovery
  for a shelter-mode-active account requires F4-class ombudsman
  path; SMS-OTP fallback is DISABLED; the abuser cannot piggy-
  back via shared-device.
- **ADR-0300 (whistleblower-press-anonymity).** Pseudonymity-
  class sessions are available to the survivor for cross-tenant
  access (e.g., the survivor seeking help on a public-victim-
  advocacy tenant without revealing identity).

### §A.3. The failure modes the substrate must defend against

Per documentation-rigor.md §1.1 rigor sub-test "failure-mode tree":

**FM-1: Abuser holds shared device + intercepts SMS-OTP for
recovery.** Mitigation: SMS-OTP fallback DISABLED for shelter-
mode-active accounts (§D-3). Recovery uses F4-class ombudsman
path (cross-ref ADR-0299 §D-7).

**FM-2: Abuser sees notification preview on lock screen.**
Mitigation: shelter-mode-active accounts use per-µservice
notification-preview-suppression (notification content shown only
after device unlock per the survivor's authentication).

**FM-3: Abuser sees tenant-admin-side audit-trail of the
survivor's actions.** Mitigation: survivor-tagged audit rows
encrypted under survivor's per-account DEK (§D-5); tenant-admin
sees only the row count + sealed-hash; the row content is
visible only via survivor's authenticated decryption path.

**FM-4: Abuser logs into shared-credential account from another
device and checks the survivor's recent activity.** Mitigation:
shelter-mode-active accounts emit `SurvivorSafetyAlertEmittedTo
SecondaryChannel` when a session from a non-survivor-attested
device authenticates; the survivor receives an out-of-band alert
via verified secondary channel.

**FM-5: Abuser installs stalkerware on the shared device.**
Mitigation: stalkerware-pattern detector (§D-6) per the Coalition
Against Stalkerware shared signature database + behavioural
anomaly detection.

**FM-6: Abuser threatens the survivor into surrendering the
device.** Mitigation: per-tenant escape-plan template (§D-4)
provides a panic-button surface that the survivor can invoke
(silently, with one tap, with a duress code).

**FM-7: Tenant-admin is compromised or coerced by the abuser
(e.g., the abuser is the tenant-admin in a small-business
scenario).** Mitigation: per-pack ombudsman path (cross-ref
ADR-0247 break-glass + ADR-0299 §D-7) provides cross-tenant
escalation; survivor can reach platform-level ombudsman even
when tenant-admin is compromised.

**FM-8: Abuser performs SIM-swap to gain control of the
survivor's phone-number factor.** Mitigation: SIM-swap detection
(cross-ref ADR-0299 §D-3) + per-tenant telco signal integration;
shelter-mode-active account auto-disables phone-number factor.

**FM-9: Abuser uses financial coercion via shared payment-
method.** Mitigation: 72h post-shelter-mode cool-down on payouts
+ payment-method changes (cross-ref ADR-0299 §D-6); per-tenant
unilateral-lockout-power on shared-credential-abuser principal.

**FM-10: Abuser is a co-parent + uses shared-account custody
dispute as cover.** Mitigation: per-pack family-court order
integration (cross-ref §3.2.5 row 11 — Custody / shared-account
dispute, future ADR per the §3.2.5 ADR roster); no parent can
unilaterally lock the other out WITHOUT court order, but the
survivor's safety-mode primitive does not require court order
(child-best-interest precedes parental-rights per §3.2.5 row 11
invariant).

**FM-11: Abuser is a minor (e.g., teen relationship abuse).**
Mitigation: minor survivor self-report path per ADR-0292 §3.2.5
row 9 — bypasses parental control + routes to per-pack mandatory
reporter.

**FM-12: Tenant-admin attempts to enumerate shelter-mode-active
accounts within the tenant.** Mitigation: Cedar fragment §D-2
FORBID + detection event emission to survivor's secondary
channel + per-pack ombudsman alert.

## Decision

### §B. Decision summary

**Decision 1: Silent shelter mode.** Per-account `SHELTER_MODE_
ACTIVE` lifecycle_state per ADR-0244 §D-3. Activation is silent
(no notification to other shared-credential principals; no per-
tenant-admin alert). The shelter-mode state is the foundational
primitive on which all other decisions compose.

**Decision 2: Hide-from-shared-device option.** Per-account
notification-preview-suppression + per-app-drawer-hide. When the
device is in a "shared mode" (per shared-device-detector §D-6),
the app surfaces a survivor-controllable "censored" appearance
(no preview text, optional alternative app name + icon).

**Decision 3: Per-tenant escape-plan template + alert-when-checked
-by-other-party.** Per-tenant escape-plan template defines: the
survivor's verified secondary channel; the per-tenant emergency-
contact roster; the per-tenant local-domestic-violence-services
referral (per the National Domestic Violence Hotline 1-800-799-
SAFE in US; 0808 2000 247 in UK; 1366 in KR; 0570-0-55210 in JP;
1800 RESPECT 1800 737 732 in AU; 04-9116 in EU 116 116 child-
helpline-extension); the per-tenant panic-button surface; the
per-tenant duress-code recognition; the per-tenant alert
recipients for "checked-by-other-party" events.

**Decision 4: No SMS-MFA fallback for shelter-mode-active
accounts.** Per §D-3, SMS-OTP fallback is DISABLED for shelter-
mode-active accounts. Recovery uses F1 (passkey backup), F4
(ombudsman path), or F5 (per-pack jurisdictional override) only.
Phone-number factor (F6 per ADR-0299 §B Decision 1) is auto-
disabled on shelter-mode activation.

**Decision 5: Audit-trail visible only to survivor.** Survivor-
tagged audit rows are encrypted under the survivor's per-account
DEK + per-row HMAC. Tenant-admin sees the sealed-hash + the
per-row count (for billing + per-pack regulator reporting
purposes) but cannot decrypt the row content. The survivor
decrypts via their authenticated session.

**Decision 6: Survivor unilateral-lockout-power over abuser.**
The survivor can invoke a one-tap surface to lock out a shared-
credential-abuser principal. The lockout is immediate (≤5s);
the abuser's active sessions are revoked; the abuser's principal
is downgraded to `shared-credential-abuser` role per ADR-0244
§D-3; the abuser's access to the shared resources is suspended.

**Decision 7: Stalkerware-pattern detection.** Per-µservice
stalkerware-pattern detector (per the Coalition Against
Stalkerware shared signature database + behavioural anomaly +
remote-access-trojan pattern) emits alerts to the survivor's
verified secondary channel. The alerts are metadata-minimized
(per ADR-0300 §D-3 — they don't reveal the stalkerware-detected
state on the shared device).

## Consequences

### §C. Consequences across all 6 engineering-rigor dimensions

Per documentation-rigor.md §1.2 engineering-rigor dimensions
matrix:

#### §C.1. Maintainability

- **Module boundaries.** Survivor-safety logic encapsulated in
  `oya-shared-survivor-safety` (single concern). The Cedar
  fragment is one file per µservice. The IaC manifest is one
  file per env. No scattered logic.
- **Versioning policy.** Per ADR-0258 SemVer. Cedar fragment
  per-fragment SemVer; shared crate Cargo SemVer; per-pack
  overlay SemVer.
- **Deprecation cadence.** SMS-OTP fallback is DEPRECATED for
  shelter-mode-active accounts (and SHOULD be deprecated globally
  per WebAuthn passkey roll-forward). The shared crate retains
  backward-compat for tenants that still allow SMS-OTP on non-
  shelter-mode accounts, but with a per-tenant deprecation warning.
- **Reverse-dependencies enumerated.** Every µservice that
  consumes survivor-safety declared in `manifest.json:reverse_
  consumers_of_survivor_safety`.
- **What is hard-coded vs configurable.** Hard-coded: audit-
  event-class slugs, the FORBID tenant-admin access to survivor-
  tagged audit rows, the SMS-MFA-disabled invariant. Configurable:
  per-pack escape-plan template, per-pack emergency-contact
  roster, per-pack regulator-notify cadence, per-tenant alert-
  recipient roster (within the survivor's verified secondary
  channel set).

#### §C.2. Observability

- **Metrics.** Per ADR-0263 cardinality budget:
  - `survivor_safety_shelter_mode_activated_total` (counter,
    dimensions: tenant_pack, audience_type, cardinality ≤2k
    series — NO survivor-identifying dimensions).
  - `survivor_safety_shelter_mode_deactivated_total` (counter,
    cardinality ≤2k series).
  - `survivor_safety_abuser_locked_out_total` (counter,
    dimensions: tenant_pack, cardinality ≤1k series).
  - `survivor_safety_shared_device_detected_total` (counter,
    cardinality ≤500 series).
  - `survivor_safety_stalkerware_pattern_detected_total`
    (counter, dimensions: stalkerware_signature_id, cardinality
    ≤1k series).
  - `survivor_safety_alert_emitted_total` (counter, dimensions:
    channel_class (BACKUP_EMAIL / TRUSTED_CONTACT /
    OMBUDSMAN_PORTAL / EMERGENCY_SERVICES_LIFELINE), cardinality
    ≤500 series).
  - `survivor_safety_escape_plan_triggered_total` (counter,
    dimensions: trigger_class (survivor-initiated, auto-
    triggered, duress-code), cardinality ≤500 series).
- **Trace span shape.** Every shelter-mode flow carries
  `survivor_safety.flow` parent span with child spans. NO
  survivor-identifying fields in span attributes (the trace is
  per-account but the per-account identifier is the survivor's
  per-account DEK-encrypted hash).
- **Logs.** JSON-structured log lines at INFO with: tenant_id,
  per-account-sealed-hash, event_class, channel_class. NO
  survivor PII; NO abuser PII visible to tenant-admin.
- **Audit events.** Per ADR-0263 — seven new classes from §B.
  Audit chain Merkle-sealed per ADR-0028; per-row HMAC under
  survivor's per-account DEK so only the survivor + per-pack
  ombudsman can decrypt.
- **SLO floor.** P95 shelter-mode-activation-to-completion ≤5
  seconds; P99 ≤30 seconds. P95 unilateral-lockout-power-to-
  effective ≤5 seconds (per §B Decision 6).
- **Dashboards.** `dashboards/survivor-safety-shelter-mode.json`
  (per µservice; aggregate metrics only, no per-survivor data)
  + `dashboards/survivor-safety-platform-wide.json` (substrate-
  level).

#### §C.3. Scalability

- **Capacity math.** Baseline shelter-mode-active population:
  estimated ~1% of consumer accounts (~per the National Domestic
  Violence Hotline US estimate of ~10M survivors annually × the
  oyatie consumer-tier active base × 12-month-rolling window).
  At ~100M active consumers, ~1M shelter-mode-active accounts.
  Throughput: ~100 shelter-mode-state-transitions / sec at
  baseline; ~1k / sec at peak (e.g., during awareness-month
  October).
- **Bottleneck identification.** Per-pack ombudsman queue is
  the bottleneck (cross-ref ADR-0299 §C.3); shelter-mode
  activation does not require ombudsman (survivor self-initiated);
  the bottleneck only applies on F4-class recovery from a
  shelter-mode-active state.
- **Horizontal scale-out path.** Per-tenant shelter-mode state
  partitioned per-cell per ADR-0248; adding cells scales
  horizontally.

#### §C.4. Performance

- **P50/P95/P99 latency.** P50 shelter-mode-activation ≤1s;
  P95 ≤5s; P99 ≤30s. P50 unilateral-lockout-effective ≤2s;
  P95 ≤5s; P99 ≤10s. P50 audit-row-decrypt-for-survivor ≤100ms;
  P95 ≤500ms; P99 ≤2s.
- **Modeling note.** Latency dominated by per-account DEK
  fetch from OpenBao (≤30ms p99) + cross-cell session-revocation
  propagation (≤2s p99 per ADR-0248 §D-N).
- **Per-region budget split.** Per ADR-0240: US ≤5s P95; EU
  ≤5s P95 (Istanbul Convention-pack tenants); KR ≤5s P95; JP
  ≤5s P95; AU ≤5s P95; UK ≤5s P95.
- **Tail-latency mitigation.** Per-cell active-session-cache
  pre-warms shelter-mode-active accounts so the lockout
  propagation is sub-second.
- **Cold-vs-warm.** Warm: per-account DEK + active-session
  cache hit. Cold: per-account DEK fetch + active-session
  enumeration (≤500ms p99 cold).

#### §C.5. Optimization

- **Per-call cost model.** CPU ≤5ms per shelter-mode-activation
  (Cedar evaluation + per-account DEK fetch + audit emission);
  RAM ≤128KB per shelter-mode state; IOPS ≤0.1 per shelter-mode
  state transition; $/M-shelter-mode-state-transitions ≤$0.50.
- **Lazy vs eager.** Per-account DEK fetch is eager (cached at
  shelter-mode-activation). Audit emission is lazy-with-flush
  (≤1s p99 per ADR-0263). Stalkerware-pattern detection is
  eager-streaming (every session emits a check; results are
  aggregated per-cell).
- **Cache-invalidation policy.** Per-account DEK cache TTL is
  ≤60s per ADR-0296; per-tenant escape-plan template cache TTL
  is 60 minutes (refresh on per-pack regulator update).
- **Profiling evidence link.** `tools/profiling/survivor-safety-
  baseline.json`.

#### §C.6. Code quality

- **Required test classes.** Unit (Cedar fragment, escape-plan
  template, stalkerware-pattern detector), property (audit-
  visibility-survivor-only invariants), fuzz (stalkerware-
  signature parser), load (mass shelter-mode-activation event),
  e2e (full flow with synthetic abuser).
- **Coverage floor.** ≥90% line, ≥80% branch (above standard
  ≥85%/≥75% due to safety stakes).
- **Lint passes.** `oya-check-cedar-fragment-soak`, `oya-check-
  survivor-only-audit-visibility-invariant` (new lint for this
  ADR), `oya-check-sms-mfa-disabled-on-shelter-mode-invariant`
  (new lint), `oya-check-spiffe-uri-conformance`, `oya-check-
  audit-event-class-registration`, `oya-check-naming-justification
  -block`.
- **Type-strictness.** Rust `deny(warnings) + deny(missing_docs)
  + deny(unsafe_op_in_unsafe_fn)`; TypeScript surface (survivor-
  facing app) `strict + noUncheckedIndexedAccess`.
- **SemVer + ABI policy.** Per ADR-0258. The shared crate's
  `ShelterMode`, `UnilateralLockoutPower`, `EscapePlanTemplate`,
  `StalkerwarePatternDetector` traits are public-stable.

## Detailed mechanics

### §D. Detailed mechanics

#### §D-1. Silent shelter mode — state machine

```
       ┌──────────────────────┐
       │   NORMAL             │◄──────────┐
       └──────┬───────────────┘           │
              │                            │
              │ user-initiated activation  │ user-initiated
              │ (or auto-triggered by      │ deactivation
              │  per-pack threshold)       │ (after ≥7-day
              ▼                            │  minimum, with
       ┌──────────────────────┐            │  ombudsman confirm)
       │   SHELTER_MODE_       ├────────────┘
       │   ACTIVE              │
       │   - SMS-MFA disabled │
       │   - Audit-visibility │
       │     survivor-only    │
       │   - Notification-    │
       │     preview-suppress │
       │   - Shared-device    │
       │     detector active  │
       │   - Stalkerware-     │
       │     pattern detector │
       │     active           │
       └──────┬───────────────┘
              │
              │ unilateral lockout
              │ invoked
              ▼
       ┌──────────────────────┐
       │   SHELTER_MODE_      │
       │   ACTIVE +            │
       │   ABUSER_LOCKED_OUT  │
       │   (abuser principal  │
       │    downgraded;       │
       │    sessions revoked) │
       └──────────────────────┘
```

Silent shelter mode activation:
1. The survivor opens the per-tenant survivor-safety surface
   (or invokes the panic-button per §D-4).
2. The survivor authenticates via passkey (F1 per ADR-0299; SMS
   is NOT permitted).
3. The survivor confirms the shelter-mode activation + designates
   the abuser principal(s) + designates the verified secondary
   channel.
4. The platform sets `SHELTER_MODE_ACTIVE` lifecycle_state per
   ADR-0244 + tags the survivor's per-account DEK as shelter-
   mode-active.
5. Audit emission: `SurvivorSafetyShelterModeActivated` event.
6. NO notification to abuser; NO notification to tenant-admin
   (unless tenant-admin has the survivor's authenticated
   delegation OR the per-pack regulator mandates tenant-admin
   awareness for VAWA enforcement).

#### §D-2. Cedar fragment — `policy/survivor-safety.cedar`

```cedar
// policy/survivor-safety.cedar
// Per ADR-0301 survivor-safety + domestic-abuse mode.
// Soak window: per ADR-0294 (≥60s before promotion).

// Permit silent shelter-mode activation by the survivor.
permit(
  principal,
  action == Action::"activate_shelter_mode",
  resource
)
when {
  principal == resource.account.primary_principal &&
  context.factor_class != "SMS_OTP" &&
  context.factor_class != "F6_phone_with_sim_swap_detection"
};

// FORBID tenant-admin access to survivor-tagged audit rows.
forbid(
  principal in PrincipalGroup::"tenant_admin",
  action in [
    Action::"read_audit_row",
    Action::"decrypt_audit_row",
    Action::"export_audit_row"
  ],
  resource
)
when {
  resource.audit_row has shelter_mode_tag &&
  resource.audit_row.shelter_mode_tag == true
};

// FORBID SMS-OTP factor on shelter-mode-active accounts.
forbid(
  principal,
  action == Action::"verify_recovery_factor",
  resource
)
when {
  resource.account.lifecycle_state == "SHELTER_MODE_ACTIVE" &&
  context.factor_class in ["SMS_OTP", "F6_phone_with_sim_swap_detection"]
};

// Permit survivor-invoked unilateral lockout of shared-credential
// abuser principal.
permit(
  principal,
  action == Action::"unilateral_lockout_abuser",
  resource
)
when {
  principal == resource.account.primary_principal &&
  resource.target_principal.role == "shared-credential-abuser"
};

// FORBID cross-account enumeration of shelter-mode-active accounts
// by tenant-admin.
forbid(
  principal in PrincipalGroup::"tenant_admin",
  action in [
    Action::"enumerate_shelter_mode_accounts",
    Action::"query_shelter_mode_state",
    Action::"export_shelter_mode_population"
  ],
  resource
)
when {
  resource.account.lifecycle_state == "SHELTER_MODE_ACTIVE"
};

// Permit emergency-services bypass to override shelter-mode
// (cross-ref ADR-0298).
permit(
  principal in PrincipalGroup::"emergency_services",
  action,
  resource
)
when {
  context.emergency_attestation_verified == true
};
```

#### §D-3. SMS-MFA disabled invariant

For every account where `lifecycle_state == "SHELTER_MODE_ACTIVE"`,
the auth-µservice MUST refuse SMS-OTP factor verification and the
recovery flow (cross-ref ADR-0299) MUST refuse F6 phone-number-
with-sim-swap-detection. The Cedar fragment §D-2 enforces this.

Per-µservice implementation:
- At session-establishment: `if account.lifecycle_state ==
  SHELTER_MODE_ACTIVE: factor_roster.remove(SMS_OTP)`.
- At recovery-flow-initiation: same removal.
- Surface UX: SMS-OTP factor surface is hidden + replaced with a
  passkey-backup or ombudsman-path advisory.

#### §D-4. Per-tenant escape-plan template

Per-tenant escape-plan template (authored by tenant + reviewed
by per-pack ombudsman) declares:

```yaml
apiVersion: oyatie.io/v1
kind: SurvivorSafetyEscapePlan
metadata:
  tenant_id: <tenant-id>
  pack_overlay: <pack-slug>
spec:
  verified_secondary_channels:
    - backup_email: <hash-of-email>
    - trusted_contact: <hash-of-contact-principal>
    - ombudsman_portal: <per-pack-ombudsman-url>
    - emergency_services_lifeline: <per-pack-988-or-equivalent>
  emergency_contacts:
    - name: National Domestic Violence Hotline (US)
      phone: "1-800-799-SAFE (1-800-799-7233)"
      url: "thehotline.org"
    - name: Refuge (UK)
      phone: "0808 2000 247"
      url: "refuge.org.uk"
    - name: Korean Women's Hotline (KR)
      phone: "1366"
      url: "hotline.or.kr"
    - name: Spousal Violence Counseling Center (JP)
      phone: "0570-0-55210"
      url: "gender.go.jp/policy/no_violence"
    - name: 1800 RESPECT (AU)
      phone: "1800 737 732"
      url: "1800respect.org.au"
    - name: EU 116 116 (child helpline) + 116 006 (adult-victim)
      phone: "116 006"
      url: "116006.eu"
  panic_button:
    activation_method: triple_tap_power_button OR per_tenant_widget
    on_activation:
      - silent_alert_to_secondary_channel
      - trigger_emergency_services_bypass_if_within_threshold
      - log_to_shelter_mode_audit_with_duress_flag
  duress_codes:
    - code_type: alternative_pin
      on_recognition:
        - present_decoy_app_state (looks like normal app)
        - silent_alert_to_secondary_channel
  alert_recipients_for_account_checked_by_other_party:
    - backup_email
    - trusted_contact
```

Per-pack overlay extension extends the template (e.g.,
`packs/pack-vawa-us-2022/escape-plan-overlay.yaml` adds VAWA-
mandated US-specific shelter-locator + FCC SafeConnections-Act
requirements; `packs/pack-istanbul-convention-eu/escape-plan-
overlay.yaml` adds Council of Europe Istanbul Convention member-
state shelter-locator).

#### §D-5. Audit-visibility-survivor-only — per-row DEK encryption

Survivor-tagged audit rows are encrypted under the survivor's
per-account DEK. The flow:

1. **Audit-row emission.** When the platform emits an audit-row
   tied to a shelter-mode-active account, the row is tagged with
   `shelter_mode_tag: true` + encrypted under the survivor's
   per-account DEK (XChaCha20-Poly1305 with per-row nonce).
2. **Per-tenant-admin view.** The tenant-admin sees only the row
   count + sealed-hash per ADR-0263 emission contract; the row
   content is opaque.
3. **Per-pack-ombudsman view.** The per-pack ombudsman holds a
   key-rotation-bound share of the survivor's DEK (per ADR-0247
   break-glass + 2-member quorum); ombudsman can decrypt with
   the survivor's authenticated consent OR per-pack mandatory-
   reporter-statute basis.
4. **Survivor view.** The survivor's authenticated session
   decrypts the row via their per-account DEK in the sidecar
   (per ADR-0296 ≤60s OpenBao TTL).
5. **Per-pack regulator view.** Where the per-pack regulator
   mandates audit-row disclosure (e.g., VAWA enforcement, court
   order), the disclosure is gated by the §D-9 below per-
   jurisdiction reporter-privilege legal-layer assertion.

#### §D-6. Shared-device + stalkerware-pattern detector

**Shared-device detector.** Per-µservice detector flags a session
as "shared device" when:
- Per-device fingerprint (TLS JA4 + HTTP/2-3 frame pattern + OS
  fingerprint + screen resolution + timezone) shows heterogeneity
  across the past N=20 sessions (different users use the same
  device).
- Per-session usage pattern shows shared-device markers (e.g.,
  rapid auth-state-switch between two principals on the same
  device fingerprint, in a short time window).
- Per-tenant configuration declares the device as shared (e.g.,
  family-account tier).

On shared-device detected, the µservice emits `SurvivorSafety
SharedDeviceDetected` to the survivor's verified secondary
channel (per ADR-0273 metadata-minimized DKIM-signed email).

**Stalkerware-pattern detector.** Per-µservice detector flags a
session as "stalkerware-pattern" when:
- The Coalition Against Stalkerware shared-signature database
  matches (per `stopstalkerware.org/research/data-sharing`).
- Behavioural anomaly suggests remote-control (e.g., synthetic-
  cursor pattern, headless-browser fingerprint, automation-tool
  signature like Selenium / Playwright / Puppeteer driving the
  session).
- Per-session API-call pattern shows known stalkerware C&C
  signatures (e.g., periodic exfil pattern, screenshot-API abuse).

On stalkerware-pattern detected, the µservice emits `SurvivorSafety
StalkerwarePatternDetected` to the survivor's verified secondary
channel + the per-pack ombudsman.

#### §D-7. Survivor unilateral-lockout-power

The survivor invokes the unilateral-lockout-power surface (e.g.,
a one-tap "Lock out shared-credential abuser" button in the
survivor-safety app).

1. The platform authenticates the survivor via passkey (F1; F4-
   class ombudsman if no passkey).
2. The survivor designates the abuser principal(s).
3. The platform:
   - Downgrades the designated principal's role to
     `shared-credential-abuser` per ADR-0244 §D-3.
   - Revokes the abuser's active sessions (cross-µservice via
     audit-emit per ADR-0263).
   - Rotates the survivor's per-account DEK (so any pre-lockout
     audit-row remains decryptable by survivor but the abuser
     cannot piggy-back via shared-credential).
   - Locks shared-resource access (cross-tenant resources require
     per-pack court-order escalation per §3.2.5 row 11).
4. Audit emission: `SurvivorSafetyAbuserLockedOut`.
5. NO notification to abuser (silent lockout); the abuser learns
   only on next attempted login.

#### §D-8. Per-pack regulator notification — survivor-controlled

The survivor optionally consents to per-pack regulator
notification. Where the per-pack regulator MANDATES notification
(e.g., VAWA enforcement on confirmed serious-injury cases per
HHS reporting requirements; KR Domestic Violence Crime Punishment
Act Article 4 mandatory-reporter; JP Spousal Violence Prevention
Act Article 6 protective order), the notification proceeds per
ADR-0251 per-pack `breach_notification_workflow_id`.

The notification is metadata-minimized: it carries the per-pack
case-id + the per-pack jurisdiction + the per-pack regulator
ID; it does NOT carry the survivor's identity unless the
survivor explicitly consents OR the per-pack statute mandates
identity disclosure.

#### §D-9. Per-jurisdiction reporter-privilege + DSAR + court-order interactions

When the platform receives a discovery / disclosure request for a
shelter-mode-active account:
- The per-pack overlay (cross-ref ADR-0300 §D-5) declares the
  applicable reporter-privilege scope.
- The platform's legal team asserts the per-pack survivor-safety
  protection per the active statute (e.g., VAWA confidentiality
  per 42 USC §13975; UK Domestic Abuse Act 2021 sec. 65; KR DV
  Crime Punishment Act Article 18; JP Spousal Violence Prevention
  Act Article 23).
- `ReporterPrivilegeAsserted` event (per ADR-0300) fires + the
  per-pack regulator-notify workflow runs.

#### §D-10. Auto-trigger thresholds — when to suggest shelter-mode to user

The platform DOES NOT auto-activate shelter-mode (auto-activation
without survivor consent would be invasive). However, the platform
MAY suggest shelter-mode activation when signals cross a
threshold:
- Stalkerware-pattern detected on shared-device.
- Account-checked-by-other-party event fires ≥3 times in 24h
  from a non-survivor-attested device.
- Per-tenant emergency-contact reaches out via verified secondary
  channel.
- Per-pack regulator-mandated suggestion (e.g., VAWA confidentiality-
  protection-zone tenants suggest shelter-mode at account-setup).

The suggestion is delivered via the survivor's verified secondary
channel — NOT via the shared-device primary channel.

#### §D-11. Per-µservice ARCHITECTURE.md §survivor-safety section

Every µservice serving consumer accounts SHALL include in
ARCHITECTURE.md:
1. **Surface inventory.** Which surfaces serve survivor-safety
   (e.g., REST `/v1/survivor-safety/activate-shelter-mode`,
   `/v1/survivor-safety/unilateral-lockout`, AsyncAPI
   `survivor-safety.alert.v1`).
2. **Audit-visibility-survivor-only enforcement.** Cite the per-
   row DEK encryption + Cedar fragment FORBID.
3. **SMS-MFA-disabled invariant.** Cite the per-account state-
   machine enforcement.
4. **Shared-device + stalkerware-pattern detector configuration.**
5. **Per-tenant escape-plan template reference.**
6. **Per-pack overlay reference.**
7. **Audit-event-class emission.** Cite the seven new classes.
8. **Survivor unilateral-lockout-power wiring.**

#### §D-12. Multispectrum review v2.4.0 wiring

Per ADR-0243 §D-8: F1 (security: abuser cannot piggy-back even
holding shared credentials), F2 (privacy: tenant-admin cannot
stalk survivor), F3 (reliability: lockout propagation cross-
cell ≤5s), F4 (performance: ≤5s activation), F5 (cost: per-
account DEK), F6 (operability: per-pack runbook), F7
(compliance: per-pack DV-statute coverage), F8 (user safety:
THE keystone), F9 (accessibility: WCAG 2.2 AAA on survivor-
facing surface; per-tenant duress-code recognition), M1 (meta-
policy: Cedar chain ordering), M2 (meta-architecture: substrate
primitive), A1-A7 (own-policy adherence).

#### §D-13. Failure-mode tree — explicit walk-through

(FM-1..FM-12 from §A.3 are addressed as follows.)

- **FM-1 SMS-OTP intercept:** §D-3 SMS-MFA disabled.
- **FM-2 lock-screen preview:** §D-2 notification-preview
  suppression + per-app-icon-alternative.
- **FM-3 tenant-admin audit:** §D-5 per-row DEK encryption +
  Cedar FORBID.
- **FM-4 abuser checks survivor activity:** Account-checked-by-
  other-party alert to verified secondary channel.
- **FM-5 stalkerware on shared device:** §D-6 stalkerware-pattern
  detector.
- **FM-6 abuser-threatened device surrender:** §D-4 panic-button
  + duress-code.
- **FM-7 tenant-admin compromised:** §D-8 per-pack ombudsman
  cross-tenant escalation.
- **FM-8 abuser SIM-swap:** ADR-0299 §D-3 SIM-swap detection +
  per-tenant telco signal; shelter-mode-active auto-disables
  phone-number factor.
- **FM-9 financial coercion:** §D-7 unilateral lockout + ADR-
  0299 §D-6 72h cool-down.
- **FM-10 co-parent custody dispute:** §3.2.5 row 11 (future
  ADR per the roster); child-best-interest precedence.
- **FM-11 minor abuser:** ADR-0292 §3.2.5 row 9 self-report
  path.
- **FM-12 tenant-admin enumeration:** §D-2 Cedar FORBID + per-
  pack ombudsman alert.

## Implementation footprint

### §E. Implementation footprint

#### §E.1. New crate: `crates/oya-shared-survivor-safety/`

```text
crates/oya-shared-survivor-safety/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs
│   ├── shelter_mode/
│   │   ├── mod.rs
│   │   ├── state_machine.rs
│   │   └── activation.rs
│   ├── audit_visibility/
│   │   ├── mod.rs
│   │   ├── per_row_dek_encryption.rs
│   │   └── survivor_only_decryption.rs
│   ├── escape_plan/
│   │   ├── mod.rs
│   │   ├── template.rs
│   │   ├── panic_button.rs
│   │   └── duress_code.rs
│   ├── shared_device_detector.rs
│   ├── stalkerware_pattern_detector.rs
│   ├── unilateral_lockout_power.rs
│   ├── alert_emission/
│   │   ├── mod.rs
│   │   ├── secondary_channel.rs
│   │   └── per_channel_router.rs
│   ├── per_pack_overlay/
│   │   ├── mod.rs
│   │   ├── vawa_us_2022.rs
│   │   ├── istanbul_convention_eu.rs
│   │   ├── kr_dv_punishment_act.rs
│   │   ├── jp_spousal_violence_prevention.rs
│   │   ├── au_family_law_act.rs
│   │   ├── uk_domestic_abuse_act_2021.rs
│   │   └── uk_pida_1998.rs
│   ├── audit.rs
│   ├── cedar.rs
│   ├── metrics.rs
│   ├── traces.rs
│   └── error.rs
└── tests/
    ├── shelter_mode_activation_silent.rs
    ├── audit_visibility_survivor_only.rs
    ├── sms_mfa_disabled_invariant.rs
    ├── shared_device_detection.rs
    ├── stalkerware_pattern_detection.rs
    ├── unilateral_lockout_propagation.rs
    └── per_pack_overlay_application.rs
```

Public surface:

```rust
pub trait ShelterMode: Send + Sync {
    fn activate(&self, account: &AccountId, designation:
        &AbuserDesignation, secondary_channel: &VerifiedChannel)
        -> Result<ShelterModeReceipt, ShelterModeError>;
    fn deactivate(&self, account: &AccountId,
        ombudsman_confirmation: &OmbudsmanConfirmation)
        -> Result<(), ShelterModeError>;
}

pub trait UnilateralLockoutPower: Send + Sync {
    fn lock_out_abuser(&self, account: &AccountId,
        abuser_principal: &PrincipalId)
        -> Result<LockoutReceipt, LockoutError>;
}

pub trait EscapePlanTemplate: Send + Sync {
    fn render(&self, tenant_id: &TenantId, pack: &CompliancePackId)
        -> Result<RenderedTemplate, TemplateError>;
    fn trigger_panic_button(&self, account: &AccountId)
        -> Result<PanicButtonReceipt, PanicButtonError>;
}

pub trait StalkerwarePatternDetector: Send + Sync {
    fn check(&self, session: &SessionContext)
        -> Result<StalkerwareCheckResult, DetectorError>;
}

pub trait AuditVisibilityFilter: Send + Sync {
    fn emit_row(&self, account: &AccountId, row: &AuditRow)
        -> Result<EncryptedAuditRow, AuditError>;
    fn decrypt_row(&self, account: &AccountId, encrypted_row: &EncryptedAuditRow,
        survivor_session: &SurvivorSession)
        -> Result<AuditRow, AuditError>;
}
```

#### §E.2. Cedar fragment: `microservices/<ms>/policy/survivor-safety.cedar`

Per §D-2.

#### §E.3. IaC manifest: `microservices/<ms>/iac/<env>-survivor-safety.yaml`

```yaml
apiVersion: oyatie.io/v1
kind: SurvivorSafety
metadata:
  name: <ms>-survivor-safety
  microservice: <ms>
  env: <env>
spec:
  enabled: true
  shelter_mode_enabled: true
  audit_visibility_filter_enabled: true
  sms_mfa_disabled_on_shelter_mode: true
  shared_device_detector_enabled: true
  stalkerware_pattern_detector_enabled: true
  unilateral_lockout_power_enabled: true
  per_pack_overlays:
    - pack-vawa-us-2022
    - pack-istanbul-convention-eu
    - pack-kr-dv-punishment-act
    - pack-jp-spousal-violence-prevention
    - pack-au-family-law-act
    - pack-uk-domestic-abuse-act-2021
  observability:
    metrics: true
    traces: true
    audit_events:
      - SurvivorSafetyShelterModeActivated
      - SurvivorSafetyShelterModeDeactivated
      - SurvivorSafetyAbuserLockedOut
      - SurvivorSafetySharedDeviceDetected
      - SurvivorSafetyStalkerwarePatternDetected
      - SurvivorSafetyAlertEmittedToSecondaryChannel
      - SurvivorSafetyEscapePlanTriggered
```

#### §E.4. Spec: `specs/survivor-safety-shelter-mode.json`

JSON Schema for shelter-mode state machine per documentation-
rigor.md §2 spec rigor.

#### §E.5. CI lanes per §B Decision

```text
.github/workflows/oya-governance-survivor-safety.yml
.github/workflows/oya-governance-survivor-safety-audit-visibility.yml
.github/workflows/oya-governance-survivor-safety-no-sms-mfa-fallback.yml
.github/workflows/oya-governance-survivor-safety-shared-device-detection.yml
.github/workflows/oya-governance-survivor-safety-stalkerware-pattern-detection.yml
.github/workflows/oya-governance-survivor-safety-cedar-fragment-present.yml
.github/workflows/oya-governance-survivor-safety-unilateral-lockout-power.yml
```

## Migration

### §F. Migration plan

#### §F.1. Phase 0 — Doctrine acceptance (2026-05-20 — 2026-05-27)

ADR accepted in text. Shared crate skeleton + Cedar fragment +
escape-plan template skeleton land. CI lanes promote to advisory.

#### §F.2. Phase 1 — Per-pack overlay onboarding (2026-05-27 — 2026-07-15)

Onboard the 7 packs per §D-5. Per-pack legal-team training + per-
pack ombudsman roster provisioning + per-pack escape-plan template
authoring.

#### §F.3. Phase 2 — Stalkerware-signature database integration (2026-06-01 — 2026-07-15)

Join the Coalition Against Stalkerware data-sharing agreement +
integrate the shared-signature database into the per-cell
detector.

#### §F.4. Phase 3 — Per-µservice wiring (2026-07-01 — 2026-08-15)

Every µservice serving consumer accounts adds the §D-11
ARCHITECTURE.md section + the Cedar fragment + the IaC manifest.
CI lanes promote to BLOCKER on 2026-08-15.

#### §F.5. Phase 4 — BLOCKER promotion (2026-08-15)

The seven CI lanes promote to BLOCKER.

#### §F.6. Rollback path

Per-fragment Cedar rollback within ≤5 minutes per ADR-0294. Per-
pack overlay rollback within ≤15 minutes. Platform-wide rollback
within ≤30 minutes.

Rollback DOES NOT compromise existing shelter-mode-active accounts;
they continue under the pre-rollback fragment.

## References

### §G. References

#### §G.1. Hyperscaler precedents

- Apple Safety Check: `support.apple.com/en-us/HT213014`
- Apple Personal Safety User Guide: `support.apple.com/en-us/guide/personal-safety/welcome/web`
- Google Privacy Checkup: `myaccount.google.com/privacycheckup`
- Google stalkerware protection: `support.google.com/accounts/answer/6010255`
- Signal Privacy & Security: `support.signal.org/hc/en-us/categories/360002917252`
- WhatsApp Disappearing Messages: `faq.whatsapp.com/673193694148717`
- WhatsApp Hide Chat: `faq.whatsapp.com/433755712763408`
- Coalition Against Stalkerware: `stopstalkerware.org`
- National Network to End Domestic Violence (NNEDV) Safety Net: `nnedv.org/content/safety-net`
- Refuge Tech Safety (UK): `refuge.org.uk/tech-safety`
- Coinbase Trust & Safety domestic-financial-abuse protections (per Coinbase 2024 published Trust & Safety report).

#### §G.2. Regulatory anchors

- **US — Violence Against Women Act (VAWA) Reauthorization Act
  2022** (Public Law 117-103; confidentiality protection per 42
  USC §13975).
- **US — FCC SafeConnections Act 2022** (Public Law 117-223;
  carrier obligations for survivors).
- **US — 42 USC §13975** (VAWA confidentiality of survivor data).
- **EU — Council of Europe Convention on preventing and combating
  violence against women and domestic violence (Istanbul
  Convention)** (2011; EU accession 2023).
- **EU — Directive 2024/1385 on combating violence against women
  and domestic violence** (transposition deadline 2027-06-14).
- **UK — Domestic Abuse Act 2021** (sec. 65 statutory
  confidentiality of survivor data).
- **KR — Domestic Violence Crime Punishment Act** (Article 4
  mandatory-reporter; Article 18 protection of survivor identity).
- **JP — Spousal Violence Prevention Act 2001** (revised 2024;
  Article 6 mandatory-reporter; Article 23 protective order
  confidentiality).
- **AU — Family Law Act 1975** + per-state DV-protection orders
  (Family Violence Act 2008 VIC, etc.).
- **AU — Privacy Act 1988 APP 11** (security of personal info
  including survivor data).

#### §G.3. Keystone bundle 2026-05-20 cross-references

- **ADR-0297** (abuse-defence baseline): observation-only on
  shelter-mode-active traffic.
- **ADR-0298** (emergency-services bypass): takes precedence
  on imminent-threat trigger.
- **ADR-0299** (account-recovery resilience): SMS-MFA disabled;
  F4-class ombudsman path for shelter-mode-active recovery.
- **ADR-0300** (whistleblower-press-anonymity): pseudonymity
  scope available to survivor for cross-tenant help-seeking.
- **ADR-0242** (oyatie-is-a-tenant): platform admins use same
  shelter-mode primitive.
- **ADR-0243** (Cedar universal gate): survivor-safety is
  Cedar policy.
- **ADR-0244** (tenant scoping primitive): adds SHELTER_MODE_
  ACTIVE lifecycle_state + shared-credential-abuser role.
- **ADR-0246** (policy-engine library-first): library-first
  Cedar carries the fragment.
- **ADR-0247** (break-glass): ombudsman 2-member quorum.
- **ADR-0248** (cellular architecture): per-cell partitioning.
- **ADR-0250** (build-ahead-of-certification): built certified-
  shape day one.
- **ADR-0251** (compliance packs): per-pack overlay.
- **ADR-0263** (observability emission contract): seven new
  audit-event classes.
- **ADR-0272** (cookie consent per-purpose): ephemeral storage.
- **ADR-0273** (per-tenant DKIM/SPF/DMARC): metadata-minimized
  alert emails.
- **ADR-0276** (backup portability): survivor-controlled
  portability.
- **ADR-0280** (substrate-of-substrate): depends on cedar-
  evaluator + audit-emit + anonymity-substrate.
- **ADR-0284** (platform-owner name indirection): namespace
  parameterized.
- **ADR-0292** (minor user doctrine): minor survivor self-
  report bypasses parental control.
- **ADR-0293** (meta-trust-root): per-account DEK rooted.
- **ADR-0294** (Cedar fragment soak): ≥60s soak window.
- **ADR-0295** (bootstrap CI SPIFFE + kill-switch): SPIFFE +
  kill-switch.
- **ADR-0296** (library-first credential sidecar): per-account
  DEK ≤60s TTL.

#### §G.4. Companion docs

- `docs/standards/documentation-rigor.md` §3.2.5 row 8.
- `docs/runbooks/survivor-safety-shelter-mode-activation.md`.
- `docs/runbooks/survivor-safety-unilateral-lockout.md`.
- `docs/runbooks/survivor-safety-stalkerware-pattern-response.md`.
- `docs/runbooks/per-pack-ombudsman-survivor-confirmation.md`.

#### §G.5. Cross-back-pointer follow-ups for existing ADRs

- **ADR-0297** (abuse-defence baseline): add §D-N noting
  observation-only on shelter-mode-active traffic.
- **ADR-0298** (emergency-services bypass): add §D-N noting
  imminent-threat trigger transitions shelter-mode-active
  account.
- **ADR-0299** (account-recovery resilience): add §D-N noting
  F6 phone-number factor auto-disabled on shelter-mode-active.
- **ADR-0300** (whistleblower-press-anonymity): add §D-N noting
  pseudonymity scope available to survivor.
- **ADR-0263** (observability emission contract): register the
  seven new audit-event classes.
- **ADR-0247** (break-glass): cross-reference per-pack ombudsman
  survivor-confirmation pattern.
- **ADR-0244** (tenant scoping primitive): cross-reference
  SHELTER_MODE_ACTIVE lifecycle_state + shared-credential-abuser
  role.
- **ADR-0292** (minor user doctrine): cross-reference minor
  survivor self-report path.

## Change log

### §H. Change log

| Date | Author | Change |
|---|---|---|
| 2026-05-20 | council-architecture + axis-survivor-safety | Initial Proposed status; bundled with the keystone-bundle 2026-05-20 foundational doctrine as the critical-path-doctrine-cluster-row-8 keystone. Authored per documentation-rigor.md §3.2.5 row 8. Cross-references ADR-0297 + ADR-0298 + ADR-0299 + ADR-0300 + the entire keystone bundle 2026-05-20. |
