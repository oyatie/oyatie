---
id: ADR-0299
status: Rejected
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
  - axis-account-recovery
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
related_specs:
  - /specs/platform-architecture.json
  - /specs/microservices/identity.json
  - /specs/microservices/policy-engine.json
  - /specs/microservices/audit-chain.json
  - /specs/microservices/observability.json
  - /specs/account-recovery-flow.json
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
keystone_position: critical-path-doctrine-cluster-rows-2-24-account-recovery-and-hijack-recovery
purpose: >
  Codify the Account-Recovery + Hijack-Recovery Resilience doctrine —
  a multi-factor recovery path (passkey backup + recovery code +
  delegated trusted contact) that closes the locked-out-legitimate-
  user critical path (§3.2.5 row 2) without creating an exploitable
  shortcut for hijackers (§3.2.5 row 24). The bar is: a legitimate
  user who has lost their primary credential (device wiped, SIM
  swapped, OAuth-token-leaked, phishing-victim, hardware-key-lost)
  reaches a verifiable recovered state in ≤24h p95 / ≤72h p99 with
  cryptographic non-repudiation, while an adversary attempting the
  same path is blocked by per-account cooling-period + step-up auth
  + behavioural-baseline anomaly detection + per-tenant SIM-swap
  detection + trusted-contact second-channel verification. Recovery
  is auditable + tenant-policy-bounded; recovery is never permanent
  lockout without ombudsman path. Per documentation-rigor.md §3.2.5
  row 2 (account-recovery) + row 24 (account-hijack-victim-recovery).
enforcement_status: advisory-until-2026-08-15-blocker-thereafter
enforced_by:
  - cloud-ci/Rust gate packet account-recovery-multi-factor-required
  - cloud-ci/Rust gate packet account-recovery-cooling-period-enforced
  - cloud-ci/Rust gate packet account-recovery-cedar-fragment-present
  - cloud-ci/Rust gate packet account-recovery-audit-event-class
  - cloud-ci/Rust gate packet account-recovery-no-permanent-lockout-without-ombudsman
  - cloud-ci/Rust gate packet account-recovery-72h-cooldown-post-recovery
  - cloud-ci/Rust gate packet account-recovery-sim-swap-detection-wired
naming_justifications:
  - name: oya-shared-account-recovery
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.account-recovery
    justification: >
      Per ADR-0105 13-layer canonical enum row 5 (shared-substrate),
      the crate that exposes the multi-factor recovery orchestrator,
      cooling-period state machine, SIM-swap detector, trusted-
      contact second-channel verifier, and ombudsman escalation
      router belongs at the shared layer. Naming `oya-shared-
      account-recovery` keeps the single-concern flat layout per
      ADR-0131 + ADR-0132. Drop-in companion to `oya-shared-
      abuse-defence` (ADR-0297) and `oya-shared-emergency-services-
      bypass` (ADR-0298); the three substrate primitives co-locate
      but are independently versioned.
  - name: oya-governance-account-recovery
    layer: N/A (foundry-fitness aggregate CI lane)
    bnf_segments: oya.foundry-fitness.account-recovery
    justification: >
      Aggregate CI fitness lane per ADR-0212 buildability doctrine;
      rolls up per-µservice declaration of the multi-factor recovery
      orchestrator, the cooling-period state machine, the SIM-swap
      detection, the trusted-contact verifier, the ombudsman path,
      and the 72h post-recovery cool-down. Lane naming follows the
      canonical `oya-governance-<concern>` shape consistent
      with sibling lanes.
  - name: oya-governance-account-recovery-multi-factor-required
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.account-recovery-multi-factor-required
    justification: >
      Per-µservice child lane verifying every account-recovery
      surface requires ≥2 of (passkey-backup, recovery-code, delegated-
      trusted-contact, ombudsman) before granting recovered state.
      Single-factor recovery is REVISE.
  - name: oya-governance-account-recovery-cooling-period
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.account-recovery-cooling-period
    justification: >
      Per-µservice child lane verifying every account-recovery
      surface honors the per-pack cooling-period (≥30 minutes for
      consumer-tier, ≥24h for high-value-mutations, ≥72h for
      post-recovery high-value-mutations).
  - name: oya-governance-account-recovery-sim-swap-detection
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.account-recovery-sim-swap-detection
    justification: >
      Per-µservice child lane verifying SIM-swap detection (per-
      tenant telco-signal integration) is wired on every recovery
      path that touches the phone-number factor.
  - name: oya-governance-account-recovery-ombudsman-path
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.account-recovery-ombudsman-path
    justification: >
      Per-µservice child lane verifying the ombudsman escalation
      path is wired — no permanent lockout is permitted without an
      ombudsman escalation route per ADR-0247 break-glass pattern.
  - name: X-Oya-Recovery-State
    layer: N/A (HTTP request header naming)
    bnf_segments: X-Oya.Recovery-State
    justification: >
      Custom HTTP request header carrying the per-session recovery-
      state machine cursor (enum: NORMAL, RECOVERY_INITIATED,
      RECOVERY_FACTOR_1_VERIFIED, RECOVERY_FACTOR_2_VERIFIED,
      RECOVERY_GRANTED_72H_COOLDOWN, RECOVERY_OMBUDSMAN_ESCALATED).
      Namespace prefix `X-Oya-` reserves the platform's header
      surface and avoids collision with existing identity headers.
  - name: AccountRecoveryInitiated
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: AccountRecovery.Initiated
    justification: >
      Audit-event-class emitted when an account-recovery flow
      starts; registered in ADR-0263 central registry per §3.2.2
      consistency invariant.
  - name: AccountRecoveryFactorVerified
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: AccountRecovery.FactorVerified
    justification: >
      Audit-event-class emitted on each factor verification in the
      multi-factor recovery flow.
  - name: AccountRecoveryGranted
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: AccountRecovery.Granted
    justification: >
      Audit-event-class emitted when the recovery flow completes
      and the principal returns to authenticated state.
  - name: AccountRecoveryDenied
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: AccountRecovery.Denied
    justification: >
      Audit-event-class emitted when the recovery flow is denied
      (e.g., insufficient factors verified, cooling-period violation,
      SIM-swap detection fired, behavioural-baseline anomaly fired).
  - name: AccountRecoveryOmbudsmanEscalated
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: AccountRecovery.OmbudsmanEscalated
    justification: >
      Audit-event-class emitted when the recovery flow escalates
      to the per-pack ombudsman path (e.g., after N failed
      automated recovery attempts, or when the user explicitly
      requests human review).
  - name: AccountHijackDetected
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: AccountHijack.Detected
    justification: >
      Audit-event-class emitted when behavioural-baseline anomaly,
      SIM-swap detection, geo-impossibility, or device-change-after-
      auth fires on an active session; distinct from
      AccountRecoveryDenied since the hijack-detection path is the
      pre-recovery inverse.
  - name: SimSwapDetected
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: SimSwap.Detected
    justification: >
      Audit-event-class emitted on SIM-swap detection via per-
      tenant telco signal (e.g., T-Mobile / Verizon / AT&T / EE /
      Vodafone / DT / NTT / SKT / KT / LGU+ Number Verification
      API per GSMA Mobile Connect).
  - name: policy/account-recovery.cedar
    layer: N/A (per-µservice Cedar fragment file)
    bnf_segments: policy.account-recovery
    justification: >
      Canonical filename for the per-µservice account-recovery
      Cedar fragment under the µservice's `policy/` directory per
      ADR-0246 + ADR-0243 fragment-lifecycle conventions; single-
      concern naming keeps the policy directory's contract-by-name
      invariant.
  - name: spec/account-recovery-flow.json
    layer: N/A (canonical state-machine spec)
    bnf_segments: specs.account-recovery-flow
    justification: >
      Canonical machine-readable state-machine spec for the
      account-recovery flow; declares states, transitions,
      preconditions, postconditions, audit-emission obligations,
      cooling-period bounds, ombudsman-escalation triggers.
  - name: oyatie.recovery.delegate.trusted-contact
    layer: N/A (SPIFFE-URI principal namespace per ADR-0295)
    bnf_segments: oyatie.recovery.delegate.trusted-contact
    justification: >
      Canonical SPIFFE-URI principal namespace for delegated-
      trusted-contact recovery agents (Apple-class legacy-contact +
      Google-class trusted-contact + per-tenant custom delegate);
      reserves `oyatie.recovery.delegate.*` under the platform's
      principal tree per ADR-0242 + ADR-0295.
---

# ADR-0299: Account-Recovery + Hijack-Recovery Resilience

## Status

Proposed — 2026-05-20.

Bundled with the keystone-bundle 2026-05-20 foundational doctrine
synthesis as the **critical-path-doctrine-cluster-rows-2-and-24-
account-recovery-and-hijack-recovery** keystone. Closes rows 2 +
24 of the 30-row critical-path matrix in `docs/standards/
documentation-rigor.md` §3.2.5.

Enforcement is `advisory-until-2026-08-15-blocker-thereafter`. The
doctrine accepts in text immediately; the seven CI lanes that
enforce it promote to BLOCKER on 2026-08-15 to give per-tenant
SIM-swap detector wiring (per-carrier Number Verification API
onboarding), per-pack ombudsman path provisioning, and per-µservice
recovery surface refactoring time to land. Until 2026-08-15,
validators emit findings without failing CI; post-2026-08-15 the
lanes block merge.

## Date

2026-05-20.

## Context

### §A. Why account-recovery is a substrate primitive, not a per-µservice afterthought

Account-recovery is a critical-path edge case (per documentation-
rigor.md §3.2.5 row 2) precisely because the standard auth-defence
pattern — phishing-resistant WebAuthn passkeys per ADR-0188, SIM-
swap detection, behavioural-baseline anomaly detection, HIBP
credential check — also creates the most common failure mode for
legitimate users: device wiped, hardware key lost, SIM swapped (by
an attacker OR by the user), OAuth-token leaked, recovery-email
inaccessible, recovery-phone-number transferred. Without a
deliberate recovery path, every hardening control becomes a
trapped-legitimate-user lockout.

The pattern across mature hyperscaler platforms is unambiguous:

- **Apple iCloud Account Recovery + Legacy Contact.** Apple's
  account recovery (rolled out 2019, refined in iOS 16+ with the
  Legacy Contact program) provides multi-factor recovery via (i)
  recovery code, (ii) recovery contact, (iii) device-PIN +
  device-passcode, (iv) ombudsman path via Apple Support with
  identity verification. Per Apple's published documentation
  (`support.apple.com/en-us/HT212513` Legacy Contact, `support.
  apple.com/en-us/HT204921` Account Recovery), the recovery flow
  has cooling-periods (24h-72h depending on factors verified) +
  post-recovery 72h cool-down on high-value mutations (purchases,
  device removal). Apple does not implement permanent lockout
  without ombudsman path. The substrate carries the recovery
  primitive; every Apple service consumes it via the iCloud
  account-recovery API.
- **Google Account Recovery + Trusted Contact.** Google's account
  recovery (rolled out 2016, refined with Advanced Protection in
  2017 and Trusted Contact in 2018) provides multi-factor recovery
  via (i) recovery email, (ii) recovery phone, (iii) trusted
  contact, (iv) device-PIN, (v) account-creation-date + recently-
  accessed-services + recent-passwords. The cooling-period is
  variable (7 days default, 30 days for Advanced Protection
  accounts). Per Google's published documentation (`support.google.
  com/accounts/answer/7682439` Account Recovery), the recovery
  flow is multi-factor + cooling-period + post-recovery
  restrictions on high-value mutations.
- **Microsoft Account Recovery + Recovery Code + Trusted Device.**
  Microsoft's recovery (rolled out 2014, refined with security
  keys 2019) provides multi-factor recovery via (i) recovery
  email, (ii) recovery phone, (iii) trusted device, (iv) backup
  code (one-time use), (v) per-pack regulator-floor for enterprise
  accounts. Per Microsoft's published documentation, the recovery
  flow has 30-day cooling-period + post-recovery restrictions.
- **Stripe Account Recovery.** Stripe's recovery (per `support.
  stripe.com/topics/account-recovery`) provides multi-factor
  recovery via (i) backup code, (ii) account-owner email
  verification, (iii) ID-based ombudsman path. Stripe is
  particularly disciplined about the 72h post-recovery cooling-
  period on payouts + payment-method changes (the high-value
  mutation class for Stripe accounts).
- **GitHub Account Recovery.** GitHub's recovery (per `docs.
  github.com/en/authentication/securing-your-account-with-two-
  factor-authentication-2fa/recovering-your-account-if-you-lose-
  your-2fa-credentials`) provides multi-factor recovery via (i)
  recovery codes, (ii) personal access token, (iii) verified
  email, (iv) ombudsman path. GitHub enforces post-recovery
  cooling-period on SSH key changes + access-token rotation.
- **Coinbase Account Recovery.** Coinbase's recovery (the highest-
  stakes consumer-facing account-recovery in finance) provides
  multi-factor recovery via (i) recovery phrase, (ii) ID-based
  ombudsman, (iii) trusted-contact second-channel verification,
  (iv) 72h cool-down on every withdrawal post-recovery. Coinbase
  also publishes their SIM-swap detection (per `blog.coinbase.com/
  preventing-sim-swap-attacks`) integrated with US carrier
  Number Verification APIs.

The corollary: **every internet-facing surface oyatie ships MUST
inherit the account-recovery primitive from the substrate, not
author it per-µservice.** A µservice that authors its own
recovery flow, its own cooling-period state machine, its own SIM-
swap detector, its own trusted-contact router is duplicating
substrate primitives that the shared crate already serves. That
duplication is a `feedback_no_silent_regression` violation (every
µservice's recovery flow drifts independently), a
`feedback_quality_performance_scalability_bar` violation (per-
µservice flows cannot share the cross-µservice behavioural-baseline
anomaly detection), and a `feedback_autonomous_implementation_
artifacts` violation (intern-buildable means the doc surface is
one substrate, not 46 µservice-private implementations).

The ADR-0299 account-recovery resilience closes this gap.

### §A.1. The two paired critical-path rows from §3.2.5

The two rows the ADR addresses:

**Row 2 — Account recovery / lockout.** "Locked-out legitimate
users; phishing-resistance must not become user-resistance." Per
the standard's mandate, the special handling MUST include: multi-
factor recovery (passkey backup + recovery code + delegated
trusted contact); cool-down + step-up; never permanent lockout
without ombudsman path. The safety/security/policy invariant:
phishing-resistant ≠ user-hostile; recovery is auditable + tenant-
policy-bounded.

**Row 24 — Account-hijack victim recovery.** "Stolen credentials,
SIM-swap, OAuth-token-theft." Per the standard's mandate, the
special handling MUST include: phishing-resistant passkey +
hardware-key recovery; per-tenant SIM-swap detection (telco
signal); 72h cool-down on high-value mutations post-recovery. The
safety/security/policy invariant: recovery is auditable + non-
repudiable; legitimate user gets back fast; hijacker cannot piggy-
back; per-pack timing honored.

The two rows are paired because they are inverses of the same
flow. Row 2 is the legitimate-user-needs-recovery case; row 24
is the hijack-victim case. Both share the same multi-factor
recovery primitive but differ in the post-recovery enforcement:
row 2 takes the user to a recovered state with normal subsequent
operation; row 24 takes the user to a recovered state PLUS a
72h cool-down on high-value mutations + active session
re-authentication of every active session attached to the prior
credential.

### §A.2. Why the keystone bundle 2026-05-20 requires this as a substrate-level primitive

The keystone bundle's foundational ADRs intersect account-recovery
as follows:

- **ADR-0242 (oyatie-is-a-tenant).** Account-recovery for an
  oyatie platform admin uses the same shared crate as account-
  recovery for any tenant user; no special platform-side flow.
- **ADR-0243 (Cedar universal gate).** The recovery flow is a
  Cedar policy decision; the policy fragment governs which
  factors are sufficient + which cooling-periods apply + when
  the ombudsman path triggers.
- **ADR-0244 (tenant scoping primitive).** Per-tenant `audience_
  type` informs recovery requirements (HIGH_RISK_USER may require
  ombudsman path for any recovery; SECURITY_RESEARCHER may have
  expedited recovery; MINOR (per ADR-0292) routes through
  parental-control surface OR self-report safety path).
- **ADR-0246 + amendment (policy-engine library-first).** Every
  µservice's library-first Cedar evaluator carries the account-
  recovery Cedar fragment.
- **ADR-0247 (self-modification doctrine + break-glass).** The
  ombudsman escalation path inherits the ADR-0247 break-glass
  pattern (post-hoc audit-and-justify + cryptographically sealed
  per ADR-0028).
- **ADR-0248 (Amazon-shape cellular architecture).** Recovery
  state is per-tenant + per-cell; cross-cell recovery is
  achieved via the per-tenant home-cell + DR-cell binding.
- **ADR-0251 (compliance packs).** Each pack adds per-pack
  recovery requirements (HIPAA: 72h cool-down on PHI access post-
  recovery; PCI: 72h cool-down on payment-method changes post-
  recovery; GDPR: data-subject right-to-recovery per Art. 12
  facilitation; KR-PIPA: Real-Name-Verification recovery option;
  COPPA: parental verification on minor-account recovery; SOX
  806: whistleblower-specific ombudsman path per ADR-0300).
- **ADR-0253 (HTTP/3 + QUIC default + ECH + PQC).** Recovery
  traffic uses the same transport tier; PQC-hybrid KEX is
  preferentially negotiated to protect long-term confidentiality
  of recovery state.
- **ADR-0263 (observability emission contract).** Recovery
  flow emits dedicated audit-event classes (six new ones per
  the naming-justifications: AccountRecoveryInitiated,
  AccountRecoveryFactorVerified, AccountRecoveryGranted,
  AccountRecoveryDenied, AccountRecoveryOmbudsmanEscalated,
  AccountHijackDetected, SimSwapDetected).
- **ADR-0272 (cookie consent per-purpose).** Recovery flow
  consent is implicit (legitimate-interest per GDPR Art.
  6(1)(f) + data-subject-request per Art. 12); however, the
  recovery cookie is purpose-scoped to recovery only.
- **ADR-0273 (per-tenant DKIM/SPF/DMARC).** Recovery email
  carries per-tenant DKIM + SPF + DMARC for deliverability +
  anti-spoof.
- **ADR-0276 (backup portability GDPR Art. 20).** Recovery
  state is portable per ADR-0276; a user can export the recovery
  state and restore to another tenant via per-pack-permitted
  cross-tenant migration.
- **ADR-0292 (minor user doctrine).** Minor account recovery
  routes through the parental-control surface OR (for safety
  reports) the self-report path that bypasses parental control.
- **ADR-0293 (meta-trust-root).** Recovery-cryptography signing
  keys rooted at the meta-trust-root.
- **ADR-0294 (Cedar fragment soak).** The account-recovery Cedar
  fragment respects the ≥60s soak window.
- **ADR-0295 (bootstrap CI SPIFFE + kill-switch).** SPIFFE
  identity for the recovery-µservice + per-cell kill-switch on
  the recovery path under sustained hijack-attempt-pressure.
- **ADR-0296 (library-first credential sidecar).** Recovery
  secrets (recovery codes, backup passkey keys, trusted-contact
  delegation tokens) held in the sidecar with ≤60s OpenBao TTL.
- **ADR-0297 (abuse-defence baseline).** The abuse-defence layer
  observes recovery attempts but does NOT gate-block legitimate
  recovery (per the §3.2.5 row 2 anti-pattern call-out — "We'll
  just add a CAPTCHA on the recovery path → No"). Abuse-defence
  emits `AbuseDefenceAccountRecoveryObserved` (companion event)
  but the gating is performed by the recovery Cedar fragment.
- **ADR-0298 (emergency-services bypass).** Emergency-services
  principals never enter the recovery flow; the bypass takes
  precedence.

### §A.3. The failure modes the recovery flow must defend against

Per documentation-rigor.md §1.1 rigor sub-test "failure-mode tree",
the explicit failure modes:

**FM-1: Legitimate user lost all factors.** Device wiped, hardware
key lost, SIM-swap-victim, recovery-email-inaccessible. Mitigation:
ombudsman path with ID-based verification + per-tenant per-pack
human-reviewer SLA.

**FM-2: Hijacker has stolen primary credential and is racing user
to recovery.** Mitigation: SIM-swap detection (per §D-3 below) +
behavioural-baseline anomaly detection + geo-impossibility soft-
block + 72h cool-down on high-value mutations post-recovery so
even a successful hijacker cannot exfiltrate fast.

**FM-3: Hijacker has stolen primary credential AND has performed
SIM-swap to gain control of the phone-number factor.** Mitigation:
the multi-factor requirement means SIM-swap-only doesn't suffice;
the recovery flow REQUIRES a second non-phone factor (passkey-
backup OR recovery-code OR delegated-trusted-contact OR ombudsman).

**FM-4: Trusted contact is also compromised (e.g., shared device,
spouse-abuser scenario per §3.2.5 row 8).** Mitigation: trusted
contact verification routes through a separate channel (the
delegate's own primary credential, not a shared device); domestic-
violence shelter mode per ADR-0301 takes precedence.

**FM-5: Per-tenant policy too aggressive (every legitimate
recovery routed to ombudsman, ombudsman queue overflows).**
Mitigation: per-pack default recovery thresholds tuned for ≤5%
ombudsman-escalation rate on legitimate-recovery flows; tenant
policy can tighten (toward ombudsman) but not relax (away from
ombudsman); ombudsman SLA per pack documented.

**FM-6: Per-tenant policy too lax (every recovery succeeds without
sufficient factors).** Mitigation: pack-floor enforcement (HIPAA
pack mandates ≥3 factors; PCI mandates ≥3 factors + 72h cool-down;
GDPR mandates audit-and-non-repudiation but not factor count;
KR-PIPA mandates Real-Name-Verification on factor-2).

**FM-7: Recovery state is exfiltrated from a compromised cell.**
Mitigation: per-cell recovery state is encrypted with per-tenant
DEK per ADR-0251; the per-tenant DEK is held in OpenBao with
≤60s TTL per ADR-0296; exfiltration of cold-storage recovery
state alone does not reveal the key material.

**FM-8: Recovery cookie / token is replayed by an attacker.**
Mitigation: recovery tokens are bound to TLS exporter (RFC 8473
token-binding where supported) + audience-bound HMAC + replay-
window ≤5min + idempotency-key required.

**FM-9: Recovery flow as denial-of-service vector.** Mitigation:
per-account recovery-attempt-velocity caps (≤5 attempts per 24h
per account) + per-IP recovery-flow rate-limit + abuse-defence
companion event emission (no gate-block, observation only).

**FM-10: Ombudsman queue compromised by insider.** Mitigation:
per-pack ombudsman 2-member quorum per ADR-0247 break-glass +
per-action MFA + UEBA monitoring on ombudsman behaviour.

## Decision

### §B. Decision summary

**Decision 1: Multi-factor recovery is the canonical path.** Every
account-recovery flow requires ≥2 factors verified before
granting recovered state. The canonical factor set:
- F1: Passkey backup (per ADR-0188 WebAuthn passkey-backup
  protocol).
- F2: Recovery code (one-time-use, generated at account-creation
  + at recovery-state-change).
- F3: Delegated trusted contact (per the
  `oyatie.recovery.delegate.trusted-contact` SPIFFE namespace; the
  delegate authenticates with their own primary credential and
  vouches for the recovering user).
- F4: Ombudsman path (per-pack human-reviewer escalation; ID-
  based verification; per-pack 2-member quorum per ADR-0247).
- F5: Per-pack jurisdictional override (e.g., KR-PIPA Real-Name-
  Verification; EU eIDAS qualified electronic signature).
- F6: Phone-number-with-SIM-swap-detection (relegated to a
  "weakening factor" that satisfies only F1 of the ≥2 requirement
  due to SIM-swap risk; cannot satisfy ≥2 alone).

**Decision 2: Per-pack cooling-period state machine.** Recovery
state transitions through `NORMAL → RECOVERY_INITIATED →
RECOVERY_FACTOR_1_VERIFIED → RECOVERY_FACTOR_2_VERIFIED →
RECOVERY_GRANTED_72H_COOLDOWN → NORMAL`. The cooling-period
between factor verifications respects per-pack timing (≥30
minutes for consumer-tier; ≥24h for high-value-mutations; ≥72h
post-recovery on high-value-mutations). Per-tenant policy may
tighten (longer cooling-period) but never relax below the pack-
floor.

**Decision 3: Per-tenant SIM-swap detection wired through telco
Number Verification APIs.** Every recovery flow that touches the
phone-number factor MUST query the per-tenant SIM-swap detector
(GSMA Mobile Number Verification API + per-carrier
direct integration). On SIM-swap detected within ≤14 days of
phone-number-factor use, the recovery flow falls back to F2+ (the
phone-number factor is invalidated for this recovery).

**Decision 4: 72h post-recovery cool-down on high-value mutations.**
Per-tenant per-pack high-value-mutations roster (e.g., payouts,
payment-method changes, principal-permission changes, data-export,
device-trust-changes). On `RECOVERY_GRANTED_72H_COOLDOWN`, these
mutations are blocked for 72h with a soft-block message and an
opt-in "verify with second factor" override.

**Decision 5: Never permanent lockout without ombudsman path.**
A failed recovery flow CANNOT result in permanent lockout. After
N=5 failed factor-verification attempts within 24h, the flow auto-
routes to ombudsman with a 24h-72h human-reviewer SLA per pack.
The user can also explicitly request ombudsman escalation at any
time.

**Decision 6: Auditable + non-repudiable.** Every recovery flow
emits ≥1 of the seven new audit-event classes per ADR-0263. Audit
chain Merkle-sealed per ADR-0028. Non-repudiation: every factor
verification is signed by the per-factor signing key in the
sidecar per ADR-0296.

**Decision 7: Abuse-defence layer is observation-only on recovery.**
Per the §3.2.5 row 2 anti-pattern call-out, abuse-defence MUST
NOT block legitimate recovery. The abuse-defence layer emits a
companion observation event (`AbuseDefenceAccountRecoveryObserved`)
but does NOT gate.

## Consequences

### §C. Consequences across all 6 engineering-rigor dimensions

Per documentation-rigor.md §1.2 engineering-rigor dimensions matrix:

#### §C.1. Maintainability

- **Module boundaries.** Recovery logic encapsulated in
  `oya-shared-account-recovery` (single crate, single concern per
  ADR-0131 + ADR-0132). The Cedar fragment is one file per
  µservice. The IaC manifest is one file per env. No scattered
  logic.
- **Versioning policy.** Per ADR-0258 SemVer. Cedar fragment per-
  fragment SemVer; shared crate Cargo SemVer; per-tenant policy
  schema SemVer (additions are MINOR; removals are MAJOR with
  ≥90-day sunset).
- **Deprecation cadence.** SMS-OTP factor is in active sunset by
  the doctrine (per Decision 1, phone-number factor satisfies
  only F1-class). New factors (e.g., per-pack jurisdiction-specific
  attestation) added with ≥90-day per-pack onboarding window.
- **Reverse-dependencies enumerated.** Every µservice that consumes
  the recovery flow declared in `manifest.json:reverse_consumers_
  of_account_recovery`.
- **What is hard-coded vs configurable.** Hard-coded: audit-event-
  class slugs, factor-count floor (≥2), recovery-attempt-velocity
  cap (≤5 per 24h). Configurable: per-pack factor-floor (HIPAA ≥3,
  PCI ≥3), per-tenant cooling-period (must be ≥ pack-floor), per-
  tenant high-value-mutations roster, per-tenant trusted-contact
  delegation roster.

#### §C.2. Observability

- **Metrics.** Per ADR-0263 cardinality budget:
  - `account_recovery_initiated_total` (counter, dimensions:
    tenant_pack, audience_type, µservice, cell, cardinality
    ≤5k series).
  - `account_recovery_factor_verified_total` (counter, dimensions:
    factor_class (F1/F2/F3/F4/F5/F6), tenant_pack, cell,
    cardinality ≤500 series).
  - `account_recovery_granted_total` (counter, dimensions:
    factors_count, tenant_pack, cell, cardinality ≤2k series).
  - `account_recovery_denied_total` (counter, dimensions:
    denial_reason (insufficient_factors, cooling_period_violation,
    sim_swap_detected, anomaly_detected, attempt_velocity_cap_
    exceeded), tenant_pack, cell, cardinality ≤3k series).
  - `account_recovery_ombudsman_escalated_total` (counter,
    dimensions: escalation_reason, tenant_pack, cell, cardinality
    ≤1k series).
  - `account_recovery_latency_seconds` (histogram, dimensions:
    factors_count, tenant_pack, cell, buckets `1s 5s 30s 5min
    30min 24h 72h`, cardinality ≤2k series).
  - `sim_swap_detected_total` (counter, dimensions: carrier,
    detection_window_days, tenant_pack, cardinality ≤500 series).
  - `account_hijack_detected_total` (counter, dimensions:
    detection_path (behavioural, geo-impossibility, device-
    change-after-auth, sim-swap), tenant_pack, cardinality ≤2k
    series).
- **Trace span shape.** Every recovery flow carries `account_
  recovery.flow` parent span with child spans `account_recovery.
  factor_verify`, `account_recovery.cedar_evaluate`, `account_
  recovery.audit_emit`, `account_recovery.cooling_period_check`.
- **Logs.** JSON-structured log lines at INFO with: tenant_id,
  principal_id, recovery_flow_id, factor_class, state_transition,
  factors_count, latency_ms, audit_event_id. Retention ≥7 years
  per HIPAA + GDPR + SOX retention floors.
- **Audit events.** Per ADR-0263 — seven new classes from §B
  Decision 6. Audit chain Merkle-sealed per ADR-0028.
- **SLO floor.** P95 ≤24h end-to-end recovery latency; P99 ≤72h.
  ombudsman path SLA: ≤72h human-reviewer response per pack;
  ≤7d total resolution per pack.
- **Dashboards.** `dashboards/account-recovery-flow.json` per
  µservice + `dashboards/account-recovery-platform-wide.json`
  substrate-level.

#### §C.3. Scalability

- **Capacity math.** Baseline recovery traffic across tenants:
  ~1k recovery-flows/sec sustained (derived: ~100M active users
  × ~0.1% recovery-rate-per-year ÷ ~3M seconds-per-year). Mass-
  recovery event (e.g., a per-tenant bulk credential rotation):
  ~10× spike = ~10k/sec for ≤1h.
- **Bottleneck identification.** Ombudsman path is the bottleneck
  (≤72h human-reviewer SLA caps throughput). Mitigation: per-pack
  ombudsman queue sized for ≤5% escalation rate; tenant policy
  cannot push escalation rate >20% without per-pack regulator
  notification.
- **Horizontal scale-out path.** Per-tenant recovery state is
  partitioned per-cell per ADR-0248; adding cells scales recovery
  horizontally. Shared-crate stateless beyond per-cell cache.

#### §C.4. Performance

- **P50/P95/P99 latency.** P50 ≤30 minutes (multi-factor verify
  in real-time); P95 ≤24h (includes per-factor cooling-periods);
  P99 ≤72h (ombudsman path).
- **Modeling note.** Latency dominated by cooling-period (24h
  minimum on high-value-mutations + 30-minute multi-factor
  cooling). Cryptographic verification ≤100ms p99. Ombudsman
  human-review ≤72h p99 per pack.
- **Per-region budget split.** Per ADR-0240 sovereign-cloud-per-
  regional-pack: US targets ≤24h P95; EU ≤24h P95 (faster due to
  GDPR Art. 12 facilitation); KR ≤48h P95 (slower due to Real-
  Name-Verification handshake); other packs ≤24h P95.
- **Tail-latency mitigation.** Ombudsman escalation auto-routes
  to per-pack secondary reviewer if primary reviewer SLA at risk.
- **Cold-vs-warm.** Warm: cached recovery state + sidecar key
  material. Cold: full recovery-state hydration from per-cell
  recovery-state-store + per-tenant DEK fetch ≤30ms p99 from
  OpenBao.

#### §C.5. Optimization

- **Per-call cost model.** CPU ≤5ms per factor verification
  (cryptographic primitives); RAM ≤256KB per recovery flow; IOPS
  ≤0.1 per factor verification; $/M-recovery-flows ≤$2.00 at the
  shared-crate boundary (excludes per-pack regulator-floor cost +
  ombudsman human-reviewer cost).
- **Lazy vs eager.** Cedar evaluation is eager (recovery permits
  resolve before any other Cedar fragment). Per-tenant policy
  hydration is lazy-with-pre-warm (tenants with active recovery
  flows are pre-warmed).
- **Cache-invalidation policy.** Per-tenant policy cache TTL is
  5 minutes; per-pack ombudsman roster cache TTL is 60 minutes;
  invalidation on per-pack regulator update.
- **Profiling evidence link.** `tools/profiling/account-recovery-
  baseline.json`.

#### §C.6. Code quality

- **Required test classes.** Unit (Cedar fragment, factor
  verifier), property (state-machine invariants), fuzz (factor-
  bundle parser), load (mass-recovery event), e2e (full flow with
  ombudsman path stub).
- **Coverage floor.** ≥90% line, ≥80% branch for
  `oya-shared-account-recovery` (above standard ≥85%/≥75% due to
  hijack-recovery stakes).
- **Lint passes.** `oya-check-cedar-fragment-soak`,
  `oya-check-state-machine-invariants` (new lint for this ADR),
  `oya-check-spiffe-uri-conformance`, `oya-check-audit-event-
  class-registration`, `oya-check-naming-justification-block`.
- **Type-strictness.** Rust `deny(warnings) + deny(missing_docs)`;
  TypeScript surface (recovery UI) `strict + noUncheckedIndexed
  Access`.
- **SemVer + ABI policy.** Per ADR-0258. The shared crate's
  `RecoveryFlow` + `RecoveryFactor` traits are public-stable.

## Detailed mechanics

### §D. Detailed mechanics

#### §D-1. State machine — RECOVERY_INITIATED → RECOVERY_GRANTED

Recovery state transitions:

```
                                    ┌──────────────┐
                                    │   NORMAL     │◄────────┐
                                    └──────┬───────┘         │
                                           │                 │ 72h elapses
                                           │ user requests   │
                                           ▼ recovery        │
                              ┌────────────────────────────┐ │
                              │   RECOVERY_INITIATED       │ │
                              │   (≤24h to F1 verify)      │ │
                              └────────┬───────────────────┘ │
                                       │ F1 verified         │
                                       ▼                     │
                              ┌────────────────────────────┐ │
                              │  RECOVERY_FACTOR_1_         │ │
                              │  VERIFIED                  │ │
                              │  (≥30min cooling +         │ │
                              │   ≤24h to F2 verify)       │ │
                              └────────┬───────────────────┘ │
                                       │ F2 verified         │
                                       ▼                     │
                              ┌────────────────────────────┐ │
                              │  RECOVERY_FACTOR_2_         │ │
                              │  VERIFIED                  │ │
                              │  (anomaly-check pass +     │ │
                              │   sim-swap-check pass +    │ │
                              │   per-pack approve)        │ │
                              └────────┬───────────────────┘ │
                                       │                     │
                                       ▼                     │
                              ┌────────────────────────────┐ │
                              │  RECOVERY_GRANTED_72H_      │ │
                              │  COOLDOWN                   ├─┘
                              │  (high-value-mutations     │
                              │   blocked for 72h)         │
                              └────────────────────────────┘
                                       │
                                       │ on any of:
                                       │   N=5 failures,
                                       │   user-requested,
                                       │   anomaly-fired,
                                       │   sim-swap-detected
                                       ▼
                              ┌────────────────────────────┐
                              │  RECOVERY_OMBUDSMAN_        │
                              │  ESCALATED                  │
                              │  (per-pack human-reviewer  │
                              │   ≤72h SLA)                │
                              └────────────────────────────┘
```

#### §D-2. Multi-factor verification — per-factor protocol

**F1: Passkey backup.** Per ADR-0188 WebAuthn passkey-backup
protocol. The user holds a backup passkey on a hardware key (e.g.,
YubiKey 5C, Solokey, Titan Security Key). Verification: WebAuthn
ceremony with `userVerification: required`. Latency ≤500ms p99
warm; ≤2s p99 cold (first-time challenge issuance).

**F2: Recovery code.** Single-use recovery codes generated at
account-creation (set of 10) + at recovery-state-change (regenerated
on every successful recovery). Codes are 128-bit random, base32-
encoded, displayed once at generation. Verification: HMAC-SHA-256
match against the per-account code-set held in OpenBao with ≤60s
TTL per ADR-0296. Latency ≤100ms p99.

**F3: Delegated trusted contact.** Per the `oyatie.recovery.delegate.
trusted-contact` SPIFFE namespace. The user pre-designates ≤3
trusted contacts at account-setup. Verification: the delegate
authenticates with their own primary credential + signs a
delegation token + presents it within ≤24h. The delegate's
primary credential is verified per the delegate's own account
state. Latency: ≤24h p99 (depends on the delegate's responsiveness).

**F4: Ombudsman path.** Per-pack ombudsman 2-member quorum per
ADR-0247 break-glass. The user submits ID-based verification
materials (government-issued ID + selfie liveness check + per-pack
jurisdictional override). Verification: per-pack human-reviewer
SLA ≤72h. Latency: ≤72h p99.

**F5: Per-pack jurisdictional override.** Pack-specific factor
(e.g., KR-PIPA: Real-Name-Verification via KR-NIA per-region
identity-provider; eIDAS: qualified electronic signature via
EU-QTSP). Verification: per-pack handshake. Latency: ≤24h p99.

**F6: Phone-number-with-SIM-swap-detection (weakening factor).**
Phone-number factor only satisfies F1 of the ≥2 floor. Verification:
SMS OTP + SIM-swap-detection check via per-tenant telco Number
Verification API. If SIM-swap detected within ≤14 days of
recovery flow, this factor is invalidated.

#### §D-3. SIM-swap detection — per-tenant telco signal integration

The SIM-swap detector queries the per-carrier Number Verification
API per GSMA Mobile Connect. Supported carriers (as of 2026-05-20):

- **US:** T-Mobile (TruID), Verizon (BlueIQ), AT&T (Authenticator),
  US Cellular, Boost (via T-Mobile API).
- **EU:** Deutsche Telekom (T-Number), Vodafone (Mobile Connect),
  Orange (Mobile Connect), Telefonica (Mobile Connect), TIM
  (Mobile Connect), KPN (Mobile Connect).
- **KR:** SK Telecom (TID), KT (KT Mobile Connect), LG U+ (U+
  Mobile Connect).
- **JP:** NTT Docomo (D-Authenticator), KDDI (au ID), SoftBank
  (SBM Mobile Connect).
- **AU/NZ:** Telstra (Mobile Connect), Optus (Mobile Connect),
  Spark NZ (Mobile Connect), Vodafone NZ (Mobile Connect).
- **UK:** EE (Mobile Connect), Vodafone UK (Mobile Connect), O2
  (Mobile Connect), Three UK (Mobile Connect).

Per-carrier API returns: `last_sim_swap_at` (timestamp), `last_
phone_number_change_at` (timestamp), `account_age` (days), `is_
prepaid` (boolean). The detector flags SIM-swap if `last_sim_
swap_at` within ≤14 days OR account-age <30 days.

#### §D-4. Cedar fragment — `policy/account-recovery.cedar`

The canonical Cedar fragment for account-recovery is FIRST in the
recovery policy chain (after the emergency-services-bypass from
ADR-0298 but before the abuse-defence fragment from ADR-0297 on
the recovery-specific surface).

```cedar
// policy/account-recovery.cedar
// Per ADR-0299 account-recovery resilience.
// Soak window: per ADR-0294 (≥60s before promotion).

// Permit recovery transition NORMAL → RECOVERY_INITIATED on
// authenticated user request.
permit(
  principal,
  action == Action::"initiate_recovery",
  resource
)
when {
  resource.tenant.recovery_enabled == true &&
  // Velocity cap: ≤5 attempts per 24h per account.
  context.recovery_attempts_24h <= 5
};

// Permit factor verification on factor-verifier success +
// cooling-period satisfied.
permit(
  principal,
  action == Action::"verify_recovery_factor",
  resource
)
when {
  context.factor_class in [
    "F1_passkey_backup",
    "F2_recovery_code",
    "F3_delegated_trusted_contact",
    "F4_ombudsman_path",
    "F5_per_pack_jurisdiction",
    "F6_phone_with_sim_swap_detection"
  ] &&
  context.cooling_period_satisfied == true &&
  // F6 alone never satisfies ≥2 requirement.
  (context.factor_class != "F6_phone_with_sim_swap_detection" ||
   context.factors_verified_count >= 1)
};

// Permit transition to RECOVERY_GRANTED on factors_verified >= 2 +
// per-pack-floor satisfied.
permit(
  principal,
  action == Action::"grant_recovery",
  resource
)
when {
  context.factors_verified_count >= 2 &&
  context.factors_verified_count >= resource.tenant.pack_factor_floor &&
  context.anomaly_check_passed == true &&
  context.sim_swap_check_passed == true
};

// Defence-in-depth FORBID: never grant recovery on a session with
// active hijack signal.
forbid(
  principal,
  action == Action::"grant_recovery",
  resource
)
when {
  context has hijack_signal_active &&
  context.hijack_signal_active == true
};

// Defence-in-depth FORBID: never grant recovery if sim-swap
// detected within 14 days.
forbid(
  principal,
  action == Action::"grant_recovery",
  resource
)
when {
  context has sim_swap_within_14d &&
  context.sim_swap_within_14d == true &&
  context.factors_verified_count < 3
};

// Mandatory ombudsman escalation on N=5 failures or anomaly.
permit(
  principal,
  action == Action::"escalate_to_ombudsman",
  resource
)
when {
  context.recovery_attempts_24h >= 5 ||
  context.anomaly_signal_active == true ||
  context has user_requested_ombudsman &&
  context.user_requested_ombudsman == true
};
```

#### §D-5. Per-pack overlay extension

Each pack adds an overlay under `packs/<pack-slug>/policy/
account-recovery-overlay.cedar`. Examples:

- **`packs/pack-hipaa-us/policy/account-recovery-overlay.cedar`**:
  Adds `permit … when {context.factors_verified_count >= 3 &&
  context.factor_classes_include("F4_ombudsman_path")}` — HIPAA
  mandates ≥3 factors including ombudsman.
- **`packs/pack-pci-us/policy/account-recovery-overlay.cedar`**:
  Adds 72h cool-down on payment-method changes per PCI-DSS.
- **`packs/pack-kr-pipa/policy/account-recovery-overlay.cedar`**:
  Adds Real-Name-Verification factor as F5 requirement.
- **`packs/pack-eu-gdpr/policy/account-recovery-overlay.cedar`**:
  Adds Art. 12 facilitation requirements (per-pack ombudsman SLA
  ≤72h; audit-and-non-repudiation).
- **`packs/pack-coppa-us/policy/account-recovery-overlay.cedar`**:
  Adds parental-verification factor for minor accounts (cross-ref
  ADR-0292); minor safety-self-report path bypasses parental
  verification per the §3.2.5 row 9 anti-pattern call-out.
- **`packs/pack-sox-806-us/policy/account-recovery-overlay.cedar`**:
  Adds whistleblower-specific ombudsman path per ADR-0300.

#### §D-6. 72h post-recovery cool-down — high-value mutations roster

Per-tenant high-value-mutations roster (CONFIGURABLE within pack-
floor):

- Payouts / withdrawals (PCI-DSS floor: 72h).
- Payment-method addition / removal (PCI-DSS floor: 72h).
- Principal-permission changes (HIPAA floor: 72h).
- Data export (GDPR Art. 20 facilitation, but allow + audit; not
  block on cool-down).
- Device-trust changes (cross-platform: 72h).
- Recovery-factor roster changes (preserve current state during
  cool-down).
- API-key rotation (sensitive but allow + audit; opt-in 72h
  block per tenant).
- Tenant-admin role assignment (HIPAA floor: 72h).

#### §D-7. Ombudsman escalation — per-pack 2-member quorum

Per-pack ombudsman path per ADR-0247 break-glass:

1. **Trigger.** N=5 failed factor-verification attempts within 24h
   OR anomaly-signal-active OR user-explicit-request OR per-pack
   trigger (e.g., HIPAA: any recovery without ID verification).
2. **Per-pack reviewer roster.** Each pack carries a per-pack
   reviewer roster (e.g., `pack-hipaa-us/ombudsman/reviewers.yaml`
   with on-call rotation).
3. **2-member quorum.** Two distinct ombudsman reviewers per
   ADR-0247; both verify the user's identity independently.
4. **ID-based verification.** Government-issued ID + selfie
   liveness check + per-pack jurisdictional override (e.g., KR-
   PIPA Real-Name-Verification).
5. **Per-pack SLA.** ≤72h human-reviewer response per pack.
6. **Audit emission.** `AccountRecoveryOmbudsmanEscalated` event
   per ADR-0263 + cryptographically sealed per ADR-0028.
7. **Per-pack regulator notification.** Per ADR-0251 + per-pack
   `breach_notification_workflow_id`: where the recovery is tied
   to a hijack-confirmed event, the per-pack regulator is notified
   per their statute.

#### §D-8. Behavioural-baseline anomaly detection

Per ADR-0297 §3.2.6.A (detection categories), the recovery flow
consumes the cross-µservice behavioural-baseline anomaly detection
substrate:

- **Geo-impossibility.** Recovery factor verified from a location
  ≥1,000km from the user's last-known-location within ≤2 hours.
- **Device-change-after-auth.** Recovery factor verified from a
  device the user has never previously used.
- **Behavioural-fingerprint drift.** Recovery factor verified
  with a fingerprint that does not match the user's baseline.
- **Velocity anomaly.** Recovery attempts at a rate ≥3σ above
  the baseline for similar accounts in the tenant.

Anomaly signals do NOT block factor verification but DO route
the flow to RECOVERY_OMBUDSMAN_ESCALATED per Cedar §D-4.

#### §D-9. Trusted-contact delegation — per-tenant roster

Each user pre-designates ≤3 trusted contacts via their account
preferences. The roster is stored per-user with per-tenant DEK
encryption per ADR-0251. Each trusted contact:

- Is identified by their own oyatie principal (or external
  verified email + phone if cross-platform).
- Has a per-pack-permitted scope (e.g., HIPAA pack restricts to
  PHI-related recovery; PCI pack restricts to payment-related
  recovery).
- Authenticates with their own primary credential when called
  upon.
- Signs a delegation token (HMAC-SHA-256 with per-pair secret +
  audience binding + ≤24h expiry).
- Receives an audit notification on their account when invoked.

Trusted-contact delegation is OFF by default for HIGH_RISK_USER
audience-type per ADR-0244 (security-researcher, journalist,
activist tenants) — the delegate could be compromised under
state-level surveillance.

#### §D-10. Hijack-detection inverse path

When `AccountHijackDetected` fires (per ADR-0297 §3.2.6.A
detection category 2: ATO), the recovery flow is auto-triggered:

1. **Active-session termination.** All active sessions attached
   to the prior credential are revoked.
2. **Notification.** The user is notified via the per-tenant
   verified secondary channel (e.g., backup email, trusted-
   contact phone).
3. **Auto-route to RECOVERY_INITIATED.** The recovery flow is
   auto-initiated on the user's behalf.
4. **Hijacker session-blocked.** Any session attempting to
   re-authenticate with the prior credential is met with a
   `recovery_required: true` response + the user-side recovery
   flow URL.
5. **72h cool-down enforced.** Post-recovery, the 72h cool-down
   on high-value mutations is automatically enforced (no opt-in
   override permitted).
6. **Per-pack regulator notification.** Per ADR-0251 +
   `pack-pci-us` (60-day chargeback window per KR-FSS, PCI-DSS
   breach notification) or `pack-hipaa-us` (60-day breach
   notification) or `pack-eu-gdpr` (72-hour breach notification
   per GDPR Art. 33).

#### §D-11. Per-µservice ARCHITECTURE.md §account-recovery section

Every µservice handling user accounts SHALL include in
ARCHITECTURE.md:

1. **Recovery surface inventory.** Which surfaces handle recovery
   (e.g., REST `/v1/account/recover`, AsyncAPI
   `account.recovery.initiated.v1`).
2. **Factor roster.** Which factors are enabled for which audience-
   types (e.g., HIGH_RISK_USER: F1+F2+F4 only; CONSUMER: F1+F2+F3+
   F4+F5+F6).
3. **Cedar fragment reference.** Cite `policy/account-recovery.
   cedar` + per-pack overlays.
4. **Audit-event-class emission.** Cite the seven audit-event
   classes.
5. **Cooling-period configuration.** Per-pack + per-tenant cooling-
   period values.
6. **SIM-swap detector configuration.** Which carriers are wired
   for this µservice's tenants.
7. **Ombudsman escalation path.** Per-pack reviewer roster
   reference.

#### §D-12. Multispectrum review v2.4.0 wiring

Per ADR-0243 §D-8: F1 (security: hijacker cannot piggy-back), F2
(privacy: recovery audit visible only to recovering user + their
delegate + ombudsman), F3 (reliability: ombudsman queue does not
saturate), F4 (performance: 24h-72h SLA budget), F5 (cost: per-
factor + per-ombudsman-hour cost), F6 (operability: per-pack
runbook), F7 (compliance: per-pack regulator floor), F8 (user
safety: legitimate user gets back; locked-out user has ombudsman
path), F9 (accessibility: a11y on recovery surface per WCAG 2.2
AAA), M1 (meta-policy: Cedar chain ordering), M2 (meta-
architecture: substrate primitive), A1-A7 (own-policy adherence).

#### §D-13. Failure-mode tree — explicit walk-through

Per documentation-rigor.md §1.1 rigor sub-test "failure-mode tree",
the explicit failure modes (cross-referencing FM-1..FM-10 from
§A.3) are addressed as follows:

- **FM-1 (legitimate user lost all factors).** §D-2 F4-class
  ombudsman path provides the ID-based escalation route. The
  ombudsman SLA per pack is documented in §G.4 companion runbook.
  Per the §3.2.5 row 2 anti-pattern call-out "We'll just add a
  CAPTCHA on the recovery path → No", abuse-defence does NOT
  gate-block the F4-class submission; the path remains observable
  but unblocked.

- **FM-2 (hijacker racing to recovery).** §D-1 cooling-period state
  machine + §D-3 SIM-swap detection + §D-8 behavioural-baseline
  anomaly detection compose to ensure a hijacker cannot complete
  the recovery flow within the survivor's reaction window. §D-6
  72h post-recovery cool-down on high-value mutations limits
  exfiltration even when hijacker briefly succeeds.

- **FM-3 (hijacker + SIM-swap).** §D-1 Decision 1 mandates ≥2
  factors; per Decision 1 row F6, phone-number-with-SIM-swap-
  detection alone NEVER satisfies the ≥2 floor. The recovery
  flow forces a non-phone secondary factor.

- **FM-4 (trusted contact also compromised).** §D-9 trusted-
  contact delegation routes through the delegate's own primary
  credential (not via the shared device); the survivor-safety
  shelter mode per ADR-0301 takes precedence when domestic-
  violence shared-device pattern is detected.

- **FM-5 (per-tenant policy too aggressive).** §F.3 per-pack
  default thresholds tuned for ≤5% ombudsman-escalation rate on
  legitimate-recovery flows; per-pack ombudsman queue sized for
  this load. Tenant policy can tighten (toward ombudsman) but
  not relax (away from ombudsman).

- **FM-6 (per-tenant policy too lax).** §D-5 per-pack overlay
  pack-floor enforcement (HIPAA ≥3, PCI ≥3, KR-PIPA Real-Name-
  Verification on factor-2); tenant cannot drop below the pack
  floor.

- **FM-7 (recovery state exfiltrated from a compromised cell).**
  Per-cell recovery state encrypted with per-tenant DEK per
  ADR-0251; per-tenant DEK held in OpenBao with ≤60s TTL per
  ADR-0296. Cold-storage exfiltration alone does not reveal key
  material.

- **FM-8 (recovery cookie/token replay).** §D-2 F2 recovery code
  is single-use; per-session recovery tokens are bound to TLS
  exporter (RFC 8473 token-binding where supported) + audience-
  bound HMAC + replay-window ≤5min + idempotency-key required.

- **FM-9 (recovery flow as DoS vector).** §D-4 Cedar fragment
  velocity cap (≤5 attempts/24h/account) + per-IP rate-limit (via
  ADR-0297 abuse-defence companion event emission — observation-
  only). The abuse-defence layer never gate-blocks legitimate
  recovery but DOES emit `AbuseDefenceAccountRecoveryObserved`
  for forensic review.

- **FM-10 (ombudsman queue compromised by insider).** §D-7 per-
  pack 2-member quorum per ADR-0247 break-glass + per-action MFA
  + UEBA monitoring on ombudsman behaviour + cross-tenant
  escalation path. Per-pack regulator notification per ADR-0251
  on confirmed ombudsman compromise.

#### §D-14. Cross-pack jurisdictional conflict resolution

Per §3.2.5 row 23 (cross-jurisdiction conflict), when a recovery
flow crosses jurisdictions (e.g., a US-resident user on a tenant
with both HIPAA-US + EU-GDPR + KR-PIPA packs active), the higher-
restriction floor wins for factor count + cooling-period + audit
retention:

- **Factor count.** Maximum of per-pack floors (HIPAA-US: ≥3,
  PCI-US: ≥3, EU-GDPR: ≥2, KR-PIPA: ≥2 + RNV) = ≥3 with KR-RNV
  if KR-PIPA pack also active.
- **Cooling-period.** Maximum of per-pack timing (HIPAA-US: 72h,
  PCI-US: 72h, EU-GDPR: 24h, KR-FSS: 24h) = 72h on cross-pack
  scope.
- **Audit retention.** Maximum of per-pack retention floors
  (HIPAA-US: ≥7yr, EU-GDPR: per Art. 17 erasure with reasonable
  retention, KR-PIPA: ≥3yr, SOX-806: ≥7yr) = ≥7yr on cross-pack
  scope.
- **Ombudsman SLA.** Minimum of per-pack SLA (EU-GDPR Art. 12
  ≤24h pushes toward fastest SLA = ≤24h on cross-pack scope).

Per-pack tenant policy that conflicts with the cross-pack
resolution is routed to per-pack ombudsman for jurisdiction-
specific adjudication before recovery completes.

#### §D-15. Per-µservice integration checklist

Every consumer-account-serving µservice MUST land the following
deliverables before its `oya-governance-account-recovery`
lane promotes to BLOCKER:

1. **ARCHITECTURE.md §account-recovery** populated per §D-11.
2. **`policy/account-recovery.cedar`** Cedar fragment present and
   soak-approved (≥60s per ADR-0294).
3. **`iac/<env>-account-recovery.yaml`** IaC manifest present.
4. **`manifest.json:account_recovery`** block present declaring
   the per-µservice recovery surface inventory + factor roster +
   per-pack overlay roster.
5. **Per-µservice unit + property tests** covering the §D-1 state
   machine + §D-2 factor verification + §D-3 SIM-swap detection
   + §D-7 ombudsman escalation + §D-10 hijack-detection inverse
   path.
6. **Per-µservice load test** exercising the mass-recovery surge
   scenario (per §C.3 capacity math).
7. **Per-µservice dashboard** at `dashboards/account-recovery-
   flow.json` per §C.2.
8. **Per-µservice runbook** at `docs/runbooks/account-recovery-
   on-call.md` per §G.4 + per-µservice on-call escalation
   roster.
9. **Per-µservice audit-event-class registration** per ADR-0263
   §D-N central registry — seven new classes from §B Decision 6.
10. **Per-µservice cross-µservice integration test** with the
    auth-µservice + the identity-µservice + the policy-engine
    µservice + the audit-chain µservice + the observability
    µservice.

## Implementation footprint

### §E. Implementation footprint

#### §E.1. New crate: `crates/oya-shared-account-recovery/`

```text
crates/oya-shared-account-recovery/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs
│   ├── state_machine/
│   │   ├── mod.rs
│   │   ├── transitions.rs
│   │   └── cooling_period.rs
│   ├── factor/
│   │   ├── mod.rs
│   │   ├── passkey_backup.rs       // F1
│   │   ├── recovery_code.rs        // F2
│   │   ├── trusted_contact.rs      // F3
│   │   ├── ombudsman.rs            // F4
│   │   ├── jurisdiction_override.rs // F5
│   │   └── phone_with_sim_swap.rs  // F6
│   ├── sim_swap/
│   │   ├── mod.rs
│   │   ├── carriers_us.rs
│   │   ├── carriers_eu.rs
│   │   ├── carriers_kr.rs
│   │   ├── carriers_jp.rs
│   │   ├── carriers_au_nz.rs
│   │   └── carriers_uk.rs
│   ├── hijack_detection.rs
│   ├── ombudsman_escalation.rs
│   ├── audit.rs
│   ├── cedar.rs
│   ├── metrics.rs
│   ├── traces.rs
│   └── error.rs
├── tests/
│   ├── multi_factor_recovery_e2e.rs
│   ├── sim_swap_detection.rs
│   ├── hijack_recovery_72h_cooldown.rs
│   ├── ombudsman_escalation.rs
│   └── per_pack_overlay_application.rs
└── benches/
    ├── factor_verification_latency.rs
    └── cooling_period_state_transition.rs
```

Public surface:

```rust
pub trait RecoveryFlow: Send + Sync {
    fn initiate(&self, principal: &Principal, context: &Context)
        -> Result<RecoveryFlowId, RecoveryError>;
    fn verify_factor(&self, flow_id: &RecoveryFlowId,
                     factor: &RecoveryFactor)
        -> Result<FactorVerifiedReceipt, RecoveryError>;
    fn grant(&self, flow_id: &RecoveryFlowId)
        -> Result<RecoveryGrantedReceipt, RecoveryError>;
    fn escalate_to_ombudsman(&self, flow_id: &RecoveryFlowId,
                              reason: &EscalationReason)
        -> Result<OmbudsmanCaseId, RecoveryError>;
}

pub trait RecoveryFactor: Send + Sync {
    fn class(&self) -> FactorClass;
    fn verify(&self, principal: &Principal, factor_bundle: &FactorBundle)
        -> Result<FactorVerifiedReceipt, FactorError>;
}
```

#### §E.2. Cedar fragment: `microservices/<ms>/policy/account-recovery.cedar`

Per §D-4 above.

#### §E.3. IaC manifest: `microservices/<ms>/iac/<env>-account-recovery.yaml`

Per-µservice + per-env IaC manifest schema (similar shape to
ADR-0298 §E.3).

#### §E.4. State-machine spec: `specs/account-recovery-flow.json`

JSON Schema:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://oyatie.io/specs/account-recovery-flow.json",
  "title": "Account Recovery Flow State Machine",
  "version": "1.0.0",
  "_meta": {
    "purpose": "Canonical state-machine spec for account-recovery flow.",
    "industry_citations": ["Apple-Legacy-Contact", "Google-Trusted-Contact", "GSMA-Mobile-Connect"],
    "related_adrs": ["ADR-0299"],
    "binding_adr": "ADR-0299",
    "status": "Proposed"
  },
  "type": "object",
  "required": ["states", "transitions", "factor_roster", "cooling_periods"],
  "properties": {
    "states": {
      "type": "array",
      "items": {
        "type": "string",
        "enum": ["NORMAL", "RECOVERY_INITIATED", "RECOVERY_FACTOR_1_VERIFIED", "RECOVERY_FACTOR_2_VERIFIED", "RECOVERY_GRANTED_72H_COOLDOWN", "RECOVERY_OMBUDSMAN_ESCALATED"]
      }
    },
    "transitions": {
      "type": "array",
      "items": { "$ref": "#/$defs/Transition" }
    },
    "factor_roster": {
      "type": "array",
      "items": { "$ref": "#/$defs/Factor" }
    },
    "cooling_periods": { "$ref": "#/$defs/CoolingPeriods" }
  }
}
```

#### §E.5. CI lanes per §B Decision

```text
.github/workflows/oya-governance-account-recovery.yml
.github/workflows/oya-governance-account-recovery-multi-factor-required.yml
.github/workflows/oya-governance-account-recovery-cooling-period.yml
.github/workflows/oya-governance-account-recovery-sim-swap-detection.yml
.github/workflows/oya-governance-account-recovery-ombudsman-path.yml
.github/workflows/oya-governance-account-recovery-72h-cooldown.yml
.github/workflows/oya-governance-account-recovery-cedar-fragment-present.yml
```

## Migration

### §F. Migration plan

#### §F.1. Phase 0 — Doctrine acceptance (2026-05-20 — 2026-05-27)

This ADR accepted in text. Shared crate skeleton + Cedar fragment
skeleton + per-pack overlay roster land. CI lanes promote to
advisory.

#### §F.2. Phase 1 — Per-carrier SIM-swap detection onboarding (2026-05-27 — 2026-07-15)

Onboard the 28 carriers across 6 regions per §D-3. Per-carrier
contract negotiation + Number Verification API integration.

#### §F.3. Phase 2 — Per-pack ombudsman provisioning (2026-06-15 — 2026-07-15)

Provision per-pack ombudsman reviewer roster, on-call rotation,
SLA contracts, ID-based verification tooling. Per-pack runbook
finalization.

#### §F.4. Phase 3 — Per-µservice wiring (2026-07-01 — 2026-08-15)

Every µservice handling accounts adds the §D-11 ARCHITECTURE.md
section + the Cedar fragment + the IaC manifest. CI lanes
promote to BLOCKER on 2026-08-15.

#### §F.5. Phase 4 — BLOCKER promotion (2026-08-15)

The seven CI lanes promote to BLOCKER. Any µservice without wired
recovery flow fails CI.

#### §F.6. Rollback path

Per-fragment Cedar rollback within ≤5 minutes per ADR-0294. Per-
pack overlay rollback within ≤15 minutes. Platform-wide rollback
within ≤30 minutes.

Rollback DOES NOT remove existing in-flight recovery flows;
they continue under the pre-rollback fragment version. The
ombudsman path remains operational regardless of fragment state.

## References

### §G. References

#### §G.1. Hyperscaler precedents

- Apple Account Recovery: `support.apple.com/en-us/HT204921`
- Apple Legacy Contact: `support.apple.com/en-us/HT212513`
- Google Account Recovery: `support.google.com/accounts/answer/7682439`
- Google Trusted Contact: `support.google.com/accounts/answer/7124018`
- Microsoft Account Recovery: `support.microsoft.com/en-us/account-billing/help-with-the-microsoft-account-recovery-form`
- Stripe Account Recovery: `support.stripe.com/topics/account-recovery`
- GitHub 2FA Recovery: `docs.github.com/en/authentication/securing-your-account-with-two-factor-authentication-2fa/recovering-your-account-if-you-lose-your-2fa-credentials`
- Coinbase SIM-Swap Prevention: `blog.coinbase.com/preventing-sim-swap-attacks`

#### §G.2. Regulatory anchors

- **US — HIPAA Security Rule 45 CFR §164.312(d)** (person-or-
  entity authentication) + **§164.308(a)(7)** (contingency plan
  for account recovery).
- **US — PCI-DSS v4.0 Requirement 8** (identify users and
  authenticate access; 72h cool-down on payment-method changes
  post-recovery per PCI-DSS interpretation).
- **US — SOX 806** (whistleblower-specific ombudsman path; cross-
  ref ADR-0300).
- **US — Reg E (15 USC §1693g)** (consumer liability cap for
  unauthorized electronic fund transfers — drives the 72h cool-
  down on payouts post-recovery).
- **EU — GDPR Article 12** (transparent information,
  communication, and modalities for the exercise of the rights
  of the data subject — drives ≤24h ombudsman SLA).
- **EU — GDPR Article 33** (notification of personal data
  breach to supervisory authority — 72h).
- **EU — GDPR Article 34** (communication of personal data
  breach to the data subject).
- **EU — eIDAS Article 24** (qualified-trust-service requirements
  for F5 jurisdictional override).
- **KR — PIPA Article 27** (account-protection requirements) +
  **PIPA Article 39-7** (Real-Name-Verification for F5).
- **KR — FSS Banking Supervision** (60-day chargeback window;
  drives 72h cool-down on payouts post-hijack-recovery).
- **JP — Act on the Protection of Personal Information (APPI)
  Article 23** (data security measures).
- **AU — Privacy Act 1988 APP 11** (security of personal
  information).
- **UK — UK GDPR Article 12** (mirror of EU GDPR Art. 12 post-
  Brexit).
- **GSMA Mobile Connect** (per-carrier Number Verification API
  specification for SIM-swap detection).

#### §G.3. Keystone bundle 2026-05-20 cross-references

- **ADR-0297** (abuse-defence baseline): abuse-defence is
  observation-only on the recovery path; emits the companion
  `AbuseDefenceAccountRecoveryObserved` event but never gates.
- **ADR-0298** (emergency-services bypass): emergency-services
  principals never enter the recovery flow; bypass takes
  precedence.
- **ADR-0242** (oyatie-is-a-tenant): platform admins use the
  same recovery flow as tenant users.
- **ADR-0243** (Cedar universal gate): recovery is a Cedar
  policy decision.
- **ADR-0244** (tenant scoping primitive): per-tenant audience-
  type informs recovery requirements.
- **ADR-0246** (policy-engine library-first): library-first
  Cedar evaluator carries the recovery fragment.
- **ADR-0247** (self-modification / break-glass): ombudsman
  escalation inherits the break-glass pattern.
- **ADR-0248** (cellular architecture): per-tenant recovery
  state is partitioned per-cell.
- **ADR-0250** (build-ahead-of-certification): recovery flow is
  built certified-shape day one.
- **ADR-0251** (compliance packs): per-pack overlay per §D-5.
- **ADR-0263** (observability emission contract): seven new
  audit-event classes.
- **ADR-0276** (backup portability GDPR Art. 20): recovery state
  is portable per ADR-0276.
- **ADR-0280** (substrate-of-substrate): shared crate depends
  on `oya-shared-cedar-evaluator` + `oya-shared-audit-emit` +
  `oya-shared-emergency-services-bypass` (ADR-0298).
- **ADR-0284** (platform-owner name indirection): namespace
  `oyatie.recovery.*` parameterized.
- **ADR-0292** (minor user doctrine): minor account recovery
  routes through parental-control surface or self-report safety
  path.
- **ADR-0293** (meta-trust-root): recovery signing keys rooted
  at the meta-trust-root.
- **ADR-0294** (Cedar fragment soak): ≥60s soak window.
- **ADR-0295** (bootstrap CI SPIFFE + kill-switch): SPIFFE
  identity + per-cell kill-switch on the recovery path.
- **ADR-0296** (library-first credential sidecar): recovery
  secrets held in sidecar with ≤60s OpenBao TTL.

#### §G.4. Companion docs

- `docs/standards/documentation-rigor.md` §3.2.5 rows 2 + 24.
- `docs/runbooks/account-recovery-on-call.md` (per-pack on-call
  escalation).
- `docs/runbooks/account-recovery-ombudsman-escalation.md` (per-
  pack ombudsman workflow).
- `docs/runbooks/account-hijack-detected-response.md` (hijack-
  detection auto-trigger response).
- `docs/runbooks/sim-swap-detection-incident.md` (SIM-swap
  detected runbook).

#### §G.5. Cross-back-pointer follow-ups for existing ADRs

- **ADR-0297** (abuse-defence baseline): add §D-N cross-reference
  noting the recovery path is observation-only for abuse-defence.
- **ADR-0298** (emergency-services bypass): add §D-N cross-
  reference noting that emergency-services principals are not
  subject to recovery flow.
- **ADR-0263** (observability emission contract): register the
  seven new audit-event classes.
- **ADR-0247** (break-glass): cross-reference the ombudsman
  escalation pattern.
- **ADR-0188** (passkey WebAuthn): cross-reference passkey-
  backup factor.
- **ADR-0292** (minor user doctrine): cross-reference parental-
  verification factor on minor-account recovery.

## Change log

### §H. Change log

| Date | Author | Change |
|---|---|---|
| 2026-05-20 | council-architecture + axis-identity + axis-account-recovery | Initial Proposed status; bundled with the keystone-bundle 2026-05-20 foundational doctrine as the critical-path-doctrine-cluster-rows-2-and-24 keystone. Authored per documentation-rigor.md §3.2.5 rows 2 + 24. Cross-references ADR-0297 + ADR-0298 + the entire keystone bundle 2026-05-20. |
