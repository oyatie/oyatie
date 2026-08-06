---
id: ADR-0298
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
  - axis-edge
  - axis-network
  - axis-identity
  - axis-policy-engine
  - axis-audit-chain
  - axis-tenancy
  - axis-emergency-services
supersedes: []
amends: []
superseded_by: [ADR-709]
related:
  - ADR-0028-audit-chain-merkle-sealed.md
  - ADR-0044-service-mesh-and-mtls.md
  - ADR-0099-data-class-registry.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0140-cedar-policy-enforcement.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0148-service-mesh-cilium-ambient-layered.md
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
related_specs:
  - /specs/platform-architecture.json
  - /specs/microservices/edge-gateway.json
  - /specs/microservices/identity.json
  - /specs/microservices/policy-engine.json
  - /specs/microservices/audit-chain.json
  - /specs/microservices/observability.json
  - /specs/emergency-services-bypass.json
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
keystone_position: critical-path-doctrine-cluster-row-1-emergency-services-bypass
purpose: >
  Codify the Emergency-Services Bypass — a life-safety hard rule that
  authenticated emergency-services principals (NENA i3 PSAPs, carrier-
  pinned SIP gateways, NCMEC CyberTipline, crisis-line federated
  identity, J-ALERT issuers, EU 112 ESInets, KR 119, AU/NZ Triple
  Zero, UK 999, HHS 988 988-Suicide-and-Crisis-Lifeline) are
  cryptographically attested at the Tier-0 edge, bypass all abuse-
  defence gates from ADR-0297, are NEVER rate-limit-throttled, and
  emit a separate audit-event-class family with non-repudiable
  attestation chain so post-hoc forensic review can detect forgery
  without slowing the life-safety path. The bar is: a real 911 / 112
  / 119 / J-ALERT call reaches its destination in ≤200ms p99 from
  edge ingress to PSAP dispatch even under saturation; a forged
  emergency claim is detected at audit-time, never at gate-time
  (cryptographic attestation + revocation, not friction). Per
  documentation-rigor.md §3.2.3 LIFE-SAFETY HARD RULE and §3.2.5 row
  1 of the 30-row critical-path matrix.
enforcement_status: advisory-until-2026-08-15-blocker-thereafter
enforced_by:
  - cloud-ci/Rust gate packet emergency-services-bypass-attestation-chain
  - cloud-ci/Rust gate packet emergency-services-bypass-cedar-fragment-present
  - cloud-ci/Rust gate packet emergency-services-bypass-audit-event-class
  - cloud-ci/Rust gate packet emergency-services-bypass-latency-budget
  - cloud-ci/Rust gate packet emergency-services-bypass-revocation-window
  - cloud-ci/Rust gate packet emergency-services-bypass-per-pack-registry
  - cloud-ci/Rust gate packet emergency-services-bypass-no-rate-limit-floor
naming_justifications:
  - name: oya-shared-emergency-services-bypass
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.emergency-services-bypass
    justification: >
      Per ADR-0105 13-layer canonical enum row 5 (shared-substrate),
      the crate that exposes the attestation-verifier trait, the
      emergency-principal Cedar resolver, the bypass-aware bot-score
      forwarder, and the audit-event emitter for emergency-services
      flows belongs at the shared layer. Naming
      `oya-shared-emergency-services-bypass` keeps the single-concern
      flat layout per ADR-0131 and avoids any suite packaging per
      ADR-0132. Drop-in companion to `oya-shared-abuse-defence` from
      ADR-0297; the two are co-located but each is independently
      vendored to permit shelf rotation and per-pack jurisdictional
      override without dragging the abuse-defence baseline.
  - name: oya-governance-emergency-services-bypass
    layer: N/A (foundry-fitness aggregate CI lane)
    bnf_segments: oya.foundry-fitness.emergency-services-bypass
    justification: >
      Aggregate CI fitness lane per ADR-0212 buildability doctrine;
      rolls up per-µservice declaration of the emergency-services-
      bypass attestation chain, Cedar fragment presence, audit-event
      registration, latency-budget headroom, revocation-window
      currency, per-pack jurisdictional registry, and the no-rate-
      limit-floor invariant. Lane naming follows the canonical
      `oya-governance-<concern>` shape consistent with sibling
      lanes (per documentation-rigor.md §3.2.3 + ADR-0212 §G).
  - name: oya-governance-emergency-services-attestation-chain
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.emergency-services-attestation-chain
    justification: >
      Per-µservice child lane verifying every emergency-services
      ingress (SIP, HTTP, SMTP, AsyncAPI, gRPC) wires the canonical
      attestation chain (SPIFFE for workload identity + X.509 for
      carrier-pinned SIP + NENA i3 SIP-Identity for PSAP + eIDAS
      Art. 24 qualified certificate for EU + NCMEC HMAC for
      CyberTipline + crisis-line federated SSO for HHS 988).
  - name: oya-governance-emergency-services-latency-budget
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.emergency-services-latency-budget
    justification: >
      Per-µservice child lane verifying the published latency budget
      (≤200ms p99 edge-to-PSAP-dispatch) is exercised in every
      µservice that participates in the emergency-services flow.
  - name: oya-governance-emergency-services-no-rate-limit-floor
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.emergency-services-no-rate-limit-floor
    justification: >
      Per-µservice child lane verifying there is NO rate-limit applied
      to an attested emergency-services principal. The mass-casualty
      surge invariant (§3.2.5 row 22) demands an elevated floor for
      emergency-services even at platform saturation.
  - name: oya-governance-emergency-services-revocation-window
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.emergency-services-revocation-window
    justification: >
      Per-µservice child lane verifying the per-pack revocation window
      (≤60s for a confirmed forged emergency principal) is honored
      across the attestation chain so a compromised PSAP credential
      can be revoked without taking down the legitimate PSAPs in the
      same pack.
  - name: X-Oya-Emergency-Attestation
    layer: N/A (HTTP request header naming)
    bnf_segments: X-Oya.Emergency-Attestation
    justification: >
      Custom HTTP request header carrying the verified attestation
      bundle (base64-encoded JWT carrying the principal identifier,
      pack, jurisdiction, attestation source, expiry, revocation
      pointer); namespace prefix `X-Oya-` reserves the platform's
      header surface and avoids collision with NENA i3
      `Geolocation-Routing` headers or carrier-specific `P-Asserted-
      Identity` SIP headers.
  - name: X-Oya-Emergency-Pack
    layer: N/A (HTTP request header naming)
    bnf_segments: X-Oya.Emergency-Pack
    justification: >
      Custom HTTP request header carrying the active compliance-pack
      id for the emergency-services principal (e.g., `pack-us-fcc-9-1-1`,
      `pack-eu-eecc-112`, `pack-kr-ncmpda-119`, `pack-jp-j-alert`).
      Used by the Cedar gate to route per-pack policy overlays.
  - name: EmergencyServiceBypassGranted
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: EmergencyService.BypassGranted
    justification: >
      Audit-event-class emitted whenever the Tier-0 edge or per-
      µservice Cedar gate permits a request on emergency-services-
      bypass grounds; registered in ADR-0263 central registry to
      satisfy the §3.2.2 consistency invariant. Distinct from
      AbuseDefenceBotBlocked (ADR-0297) so forensic review can isolate
      emergency-services flow without conflating abuse-defence noise.
  - name: AbuseDefenceEmergencyServiceBypass
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: AbuseDefence.EmergencyServiceBypass
    justification: >
      Companion audit-event-class emitted whenever an abuse-defence
      control (rate-limit, bot-score, CAPTCHA) would have fired but
      was deferred by the emergency-services-bypass; captured for
      forensic review (the defence still observed the signal even
      though it did not gate). Distinct from EmergencyServiceBypass-
      Granted because the former records what the abuse-defence layer
      observed, while the latter records what the bypass layer
      decided.
  - name: EmergencyServiceForgeryDetected
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: EmergencyService.ForgeryDetected
    justification: >
      Audit-event-class emitted when post-hoc forensic review (or
      live attestation-revocation polling) flags a previously-
      permitted emergency-services request as forged or revoked.
      Triggers the revocation cascade per §D-7.
  - name: EmergencyServiceRateLimitElevation
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: EmergencyService.RateLimitElevation
    justification: >
      Audit-event-class emitted when the per-cell rate-limit floor
      is elevated for emergency-services traffic during mass-
      casualty surge (per §3.2.5 row 22 + §D-6 below); records when
      the elevation began, the surge threshold tripped, the cells
      affected, and the recovery point.
  - name: policy/emergency-services-bypass.cedar
    layer: N/A (per-µservice Cedar fragment file)
    bnf_segments: policy.emergency-services-bypass
    justification: >
      Canonical filename for the per-µservice emergency-services-
      bypass Cedar fragment under the µservice's `policy/` directory
      per ADR-0246 + ADR-0243 fragment-lifecycle conventions; single-
      concern naming keeps the policy directory's contract-by-name
      invariant. Independent of `policy/abuse-defence.cedar` so the
      two fragments can promote on separate soak windows.
  - name: iac/<env>-emergency-services-ingress.yaml
    layer: N/A (per-µservice IaC manifest)
    bnf_segments: iac.<env>.emergency-services-ingress
    justification: >
      Canonical filename for per-µservice + per-env emergency-
      services ingress IaC manifest; expresses the Tier-0 edge
      attestation-verifier configuration, the per-pack overlay
      enable/disable, the latency-budget telemetry hook, and the
      revocation-window polling cadence.
  - name: EMERGENCY_SERVICES
    layer: N/A (Tenant.audience_type enum value per ADR-0244)
    bnf_segments: audience_type.EMERGENCY_SERVICES
    justification: >
      Tenant.audience_type enum extension per ADR-0244 §D-3;
      identifies tenants whose principals serve as emergency-services
      ingress (e.g., the `oyatie.emergency.us.psap.*` family, the
      `oyatie.emergency.eu.esinet.*` family, the `oyatie.emergency.
      kr.119.*` family, the `oyatie.emergency.jp.j-alert.*` family,
      etc.). Distinct from FRIENDLY_CRAWLER_PARTNER (ADR-0297) and
      HIGH_RISK_USER (ADR-0292) so per-pack bypass policies do not
      collide.
  - name: oyatie.emergency.{country}.{service}.{principal}
    layer: N/A (SPIFFE-URI principal namespace per ADR-0295)
    bnf_segments: oyatie.emergency.{country}.{service}.{principal}
    justification: >
      Canonical SPIFFE-URI principal namespace for emergency-services
      ingress; reserves `oyatie.emergency.*` under the platform's
      principal tree per ADR-0242 + ADR-0295. Examples:
      `oyatie.emergency.us.psap.king-county-9-1-1`,
      `oyatie.emergency.eu.esinet.frankfurt-112`,
      `oyatie.emergency.kr.119.seoul-119`,
      `oyatie.emergency.jp.j-alert.cabinet-secretariat`,
      `oyatie.emergency.us.ncmec.cybertipline`,
      `oyatie.emergency.us.hhs.988-suicide-and-crisis-lifeline`.
  - name: spec/emergency-services-registry.json
    layer: N/A (canonical registry spec)
    bnf_segments: specs.emergency-services-registry
    justification: >
      Canonical machine-readable registry of all emergency-services
      principals (one row per principal) carrying: principal slug,
      compliance pack, jurisdiction, attestation source, attestation
      key material reference (OpenBao path), revocation-window
      target, per-pack regulatory anchor (statute citation), per-
      µservice consumer list. Registry rotation per ADR-0258 SemVer
      policy.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0298: Emergency-Services Bypass — Life-Safety Hard Rule

## Status

Proposed — 2026-05-20.

Bundled with the keystone-bundle 2026-05-20 foundational doctrine
synthesis as the **critical-path-doctrine-cluster-row-1-emergency-
services-bypass** keystone. Closes row 1 of the 30-row critical-path
matrix in `docs/standards/documentation-rigor.md` §3.2.5 and the
LIFE-SAFETY HARD RULE in §3.2.3 (the highest-priority hardrule).

Enforcement is `advisory-until-2026-08-15-blocker-thereafter`. The
doctrine accepts in text immediately; the seven CI lanes that
enforce it promote to BLOCKER on 2026-08-15 to give per-pack
attestation-source onboarding (carrier-pinned SIP gateways, NENA i3
PSAPs, EU eIDAS Art. 24 qualified-certificate issuers, KR-119
attestation-issuer, JP J-ALERT key-distribution-center, AU/NZ Triple
Zero, UK 999/112 carrier-pinned SIP, NCMEC CyberTipline HMAC, HHS
988-Suicide-and-Crisis-Lifeline federated SSO) time to land. Until
2026-08-15, validators emit findings without failing CI; post-
2026-08-15 the lanes block merge.

## Date

2026-05-20.

## Context

### §A. Why emergency-services bypass is a substrate primitive, not a per-µservice afterthought

Emergency-services traffic is the **single highest-priority class of
request** any internet-facing platform can carry. A failed 911 call,
a delayed 112 dispatch, a throttled J-ALERT broadcast, a forged
NCMEC CyberTipline submission, a blocked HHS 988-Suicide-and-Crisis-
Lifeline reach-out — each has direct, irreversible human-safety
consequence. The bar is qualitatively different from every other
abuse-defence policy: where ADR-0297 trades off latency, conversion,
and accessibility against abuse-pressure-relief, ADR-0298 trades off
**nothing**. The life-safety path is the path. Abuse-defence
controls observe, audit, and detect — they never gate.

Mature hyperscaler platforms and telecom carriers treat the
emergency-services bypass as a *first-class substrate primitive* —
wired at the planetary edge, in every internet-facing µservice's
request path, and as Cedar policy composing with every other gate
in a strict "bypass-then-audit" rather than "evaluate-then-permit"
order. The pattern is unambiguous across the named industry
references:

- **Apple iOS Emergency SOS via Satellite.** Apple's iOS 16+
  Emergency SOS via Satellite (launched November 2022, expanded to
  16 markets by 2025) bypasses cellular-tower authentication,
  bypasses iCloud account locking, bypasses every Apple-server
  authentication gate, and routes directly to relay-and-PSAP
  partners (Globalstar + Garmin Response). Per Apple's published
  white paper (`support.apple.com/en-us/102099`), the satellite
  relay carries device-side encrypted attestation that the relay
  verifies; the PSAP receives the attested location + status without
  any Apple-side authentication challenge. The primitive is wired
  at the iOS substrate, not at every app.
- **Android Emergency Location Service (ELS) + Android Earthquake
  Alerts.** Google's ELS (deployed 2016 globally, integrated into
  Android 4.1+ since 2018) bypasses GPS-permission gating and
  forwards device location directly to PSAPs on detected emergency
  calls. The Android Earthquake Alerts system (deployed 2020 in
  California, 2024 globally) bypasses notification-channel-mute and
  do-not-disturb to deliver shake-alert seconds before P-wave
  arrival. Per Google's published architecture (`developer.android.
  com/guide/topics/connectivity/location-emergency`), the bypass is
  wired into Android Location Manager itself — not at every app.
- **WhatsApp / Signal emergency-call carry-through.** WhatsApp's
  emergency-call integration (rolled out 2023 in select markets per
  Meta's published platform documentation) and Signal's E.164-fallback
  to native dialer on 911-pattern detection (Signal's open-source
  implementation, signalapp/Signal-Android) bypass app-level rate-
  limit + abuse-defence to ensure emergency calls always succeed.
- **Cloudflare planetary edge for emergency-services tenants.**
  Cloudflare publishes a `Project Galileo` (launched 2014; ~3k
  protected sites by 2024) + `Athenian Project` (launched 2017;
  ~400 protected election sites by 2024) program providing
  emergency-bypass + DDoS-absorption + abuse-defence-bypass to
  protected emergency-services + critical-democratic-infrastructure
  customers. Per Cloudflare's published architecture (`blog.
  cloudflare.com/galileo` + `blog.cloudflare.com/athenian`), the
  bypass is configured at the edge — every Cloudflare POP carries
  the bypass logic.
- **AWS GovCloud + IL5/IL6 emergency-services tier.** AWS's GovCloud
  (US) + Top Secret-East/West regions (deployed for IC + DOD
  customers since 2017) ship dedicated emergency-services tier with
  guaranteed capacity reservations + bypass of all consumer-tier
  abuse-defence. Per AWS's published FedRAMP-High SSP documentation
  (publicly available since 2020), the tier carries a separate
  control-plane with attestation-bound principals.
- **NENA i3 PSAP federation.** NENA's i3 standard (NENA-STA-010.3a,
  current revision 2024) defines how PSAPs (Public Safety Answering
  Points) federate location-routing, identity-attestation, and
  call-handover across the US Next-Generation 911 (NG911) substrate.
  Per the NENA i3 SIP-Identity profile, every PSAP-originated
  identity carries a carrier-signed `Identity` header (RFC 8224,
  Authenticated Identity Management in the Session Initiation
  Protocol — STIR/SHAKEN) plus a NENA-specific extension for
  emergency-call routing. Every internet-facing platform serving
  emergency-services principals MUST verify this chain.
- **EU EECC Article 109 + ESInets.** The EU Electronic Communications
  Code (Directive (EU) 2018/1972) Article 109 mandates that all
  member states operate ESInets (Emergency Services Internet
  Protocol Networks) with the same federated identity-attestation
  + location-routing semantics as US NENA i3, plus eIDAS Art. 24
  qualified-certificate-issuer attestation for cross-border PSAP
  federation.
- **KR-119 (Korean fire and emergency).** Korea's 119 service
  (operated by the National Fire Agency) integrates with the
  Korean Real-Name-Verification substrate per KR-NCMPDA (National
  Counter-Terrorism Manual for Public Disaster Authority); every
  internet-facing platform with KR-CSAP pack active MUST recognize
  the per-pack attestation source.
- **JP J-ALERT (Japan All-Hazards Early Warning).** Japan's
  J-ALERT system (operated by the Cabinet Secretariat) is the
  national all-hazards early-warning broadcast (earthquake, tsunami,
  ballistic missile, large-volcano, NBC); J-ALERT carries a
  cryptographic key-distribution-center attestation that every
  Japan-resident emergency-services-receiving µservice MUST verify.
- **AU/NZ Triple Zero (000).** Australia's Triple Zero (operated by
  Telstra under contract from the Department of Home Affairs) and
  New Zealand's 111 (operated by Spark / Vodafone NZ) integrate
  with the per-pack carrier-pinned SIP attestation; the AU-NZ-PAC
  pack overlay carries the per-pack key material.
- **UK 999 + 112.** The UK's 999 + 112 service (operated by BT 999
  under contract from Ofcom) carries the UK-specific eIDAS Art. 24
  qualified-certificate attestation (post-Brexit, the UK retained
  eIDAS-compatible qualified trust services per the UK eIDAS
  Regulation 2016/EU910 transposition).
- **NCMEC CyberTipline.** The National Center for Missing &
  Exploited Children (US) operates the CyberTipline under 18 USC
  §2258A — every US-resident platform handling user-generated
  content MUST forward CSAM detections to NCMEC under this statute.
  NCMEC submissions are HMAC-attested + replay-window-bound.
- **HHS 988-Suicide-and-Crisis-Lifeline.** The US Health and Human
  Services 988 Suicide and Crisis Lifeline (launched July 2022,
  replacing the older National Suicide Prevention Lifeline)
  integrates with crisis-line federated SSO. Every B2C consumer
  surface that exposes a "Get Help" button SHOULD route to 988 if
  US-resident.

The corollary: **every internet-facing surface oyatie ships MUST
inherit the emergency-services bypass from the substrate, not
author it per-µservice.** A µservice that authors its own attestation
logic, its own per-pack carrier-pinned SIP verifier, its own NCMEC
HMAC verifier is duplicating substrate primitives that the Tier-0
edge already serves. That duplication is a
`feedback_no_silent_regression` violation (every µservice's bypass
drifts independently); it is a
`feedback_quality_performance_scalability_bar` violation (one
forged-PSAP credential in one µservice would let the attacker pivot
across the entire platform); and it is a
`feedback_autonomous_implementation_artifacts` violation (intern-
buildable means the doc surface is one substrate, not 46
µservice-private implementations).

The ADR-0298 emergency-services bypass closes this gap.

### §A.1. The life-safety hardrule from documentation-rigor.md §3.2.3

Per documentation-rigor.md §3.2.3, the emergency-services bypass is
the **highest-priority hardrule** in the abuse-defence taxonomy —
above every other anti-bot, anti-spoof, anti-scrape control. The
LIFE-SAFETY HARD RULE reads (verbatim from the standard):

> No anti-bot, no rate-limit, no CAPTCHA, no abuse-defence control
> shall delay or block a request originating from an attested
> emergency-services principal. The bar is: a real 911 / 112 / 119
> call reaches its destination in ≤200ms p99 from edge ingress to
> PSAP dispatch even under saturation. Detection of forgery happens
> at audit-time (cryptographic attestation + revocation), never at
> gate-time (friction would kill people).

The "even under saturation" clause is load-bearing. Mass-casualty
incidents (§3.2.5 row 22) drive 10×-100× the normal emergency-
services traffic at the exact moment the rest of the platform may
be saturated by the same incident. The bypass MUST hold at peak.

The "cryptographic attestation + revocation, not friction" clause
is equally load-bearing. The bypass cannot be a friction-free
free-pass to any IP claiming PSAP identity. The bypass is gated
on a per-pack cryptographic attestation chain (§D-1 below). A
forged emergency claim is detected at audit-time by attestation
verification (signed by the per-pack issuer, traceable to the
attestation-key revocation list, freshness-window bounded), and
revocation cascades through the audit chain in ≤60s — fast enough
to prevent sustained abuse, slow enough that legitimate PSAPs are
never throttled by overly-aggressive revocation.

### §A.2. Why the keystone bundle 2026-05-20 requires this as a Tier-0 edge primitive

The keystone bundle's foundational ADRs intersect emergency-services
bypass as follows:

- **ADR-0242 (oyatie-is-a-tenant).** Emergency-services principals
  ARE tenants under the `oyatie.emergency.*` namespace (per the
  naming-justification for the SPIFFE-URI principal namespace).
  No carve-out from the tenant model — the bypass is a per-tenant
  policy expressed as a Cedar fragment, not a hard-coded special
  case.
- **ADR-0243 (Cedar universal gate).** The emergency-services
  bypass IS a Cedar policy decision. Cedar evaluates the bypass
  first (in the policy chain) so that downstream policies see the
  bypass-flag attribute on the request context and skip their
  abuse-defence gating logic.
- **ADR-0244 (tenant scoping primitive).** The bypass adds a new
  `audience_type` enum value (`EMERGENCY_SERVICES`) to Tenant
  records per ADR-0244 §D-3. Per-row tenant scoping continues to
  hold; the bypass adjusts policy, not data isolation.
- **ADR-0246 + amendment (policy-engine library-first).** Every
  µservice's library-first Cedar evaluator carries the emergency-
  services-bypass Cedar fragment; no µservice can opt out of
  the bypass.
- **ADR-0248 (Amazon-shape cellular architecture).** Tier-0 edge
  cells host the emergency-services attestation verifier; Tier-1/
  2 cells inherit the attested principal via the verified `X-Oya-
  Emergency-Attestation` header; Tier-3 (offline / air-gapped)
  cells receive emergency-services traffic only via the per-pack
  out-of-band SIP gateway and the verification path is locally
  rooted.
- **ADR-0251 (compliance packs).** Each pack adds per-jurisdiction
  emergency-services principals (US-FCC-911 pack adds NENA i3 PSAPs;
  EU-EECC-112 pack adds ESInets; KR-NCMPDA-119 pack adds 119;
  JP-J-ALERT pack adds the Cabinet Secretariat issuer; AU-NZ-PAC
  adds Triple Zero + 111; UK-OFCOM-999 adds BT 999; HIPAA-US adds
  988-Suicide-and-Crisis-Lifeline; PCI-US adds banking-fraud-hotline
  per FinCEN guidance).
- **ADR-0253 (HTTP/3 + QUIC default + ECH + PQC).** Emergency-
  services traffic preferentially uses HTTP/3 + QUIC for the
  lowest tail-latency under saturation; emergency-services traffic
  is exempted from PQC negotiation friction (a PSAP client running
  legacy carrier-pinned SIP MUST still succeed even without PQC
  hybrid).
- **ADR-0263 (observability emission contract).** Emergency-services
  traffic emits dedicated audit-event classes (`EmergencyService
  BypassGranted`, `EmergencyServiceForgeryDetected`, `Emergency
  ServiceRateLimitElevation`, plus the abuse-defence companion
  `AbuseDefenceEmergencyServiceBypass`) so forensic review can
  isolate emergency-services flow.
- **ADR-0292 (minor user doctrine).** A minor reaching for 988
  Suicide and Crisis Lifeline cannot be parental-control-suppressed
  per §3.2.5 row 9; the bypass overrides the parental-control
  surface for any 988 / NCMEC self-report path.
- **ADR-0295 (bootstrap CI SPIFFE + kill-switch).** Workload
  identity for the emergency-services attestation-verifier µservice
  itself uses SPIFFE; the verifier carries an out-of-band
  attestation root per the per-pack key-distribution-center.
- **ADR-0296 (library-first credential sidecar).** The per-pack
  attestation verification keys are held in the sidecar per the
  ≤60s OpenBao TTL invariant; rotation cadence per pack is ≤30
  days for the production tier, ≤7 days for the most sensitive
  packs (J-ALERT, US-FCC-911).
- **ADR-0297 (abuse-defence baseline).** The emergency-services
  bypass is wired BEFORE the abuse-defence Cedar fragment in the
  policy chain so the abuse-defence layer observes (and emits
  `AbuseDefenceEmergencyServiceBypass`) but never gates.

### §A.3. The failure modes the bypass must defend against

Three failure-mode trees motivate the design (per documentation-
rigor.md §1.1 rigor sub-test "failure-mode tree"):

**Failure mode 1: forged emergency-services claim from a hostile
actor.** An adversary impersonates a PSAP and claims emergency-
services-bypass to flood a tenant surface. The bypass MUST detect
forgery cryptographically — not via friction (no CAPTCHA, no rate-
limit). The detection happens at attestation-verification time
(every request) plus at audit-time (post-hoc forensic review +
revocation polling). A forged claim that passes the first-pass
attestation verification (e.g., because the attacker stole a key
material) MUST be detected at audit-time and the revocation
cascade MUST suspend the attacker's pseudo-principal in ≤60s
across every cell.

**Failure mode 2: mass-casualty surge of legitimate emergency-
services traffic.** A magnitude-7+ earthquake, a cross-state wildfire,
a Marathon-bombing-class event drives 10×-100× the baseline
emergency-services traffic at the moment the rest of the platform
is saturated by social-media traffic about the event. The bypass
MUST hold at the surge peak: emergency-services traffic gets an
elevated rate-limit floor (per §D-6 below), other tiers degrade
gracefully, and the per-cell DR-pair failover (per ADR-0241)
preserves emergency-services availability even if a cell goes hot.

**Failure mode 3: regional outage hits the cell hosting the
attestation verifier.** The emergency-services attestation verifier
is a substrate µservice; if its hosting cell fails, emergency-
services traffic cannot be attested. The bypass MUST fail-safe to
the per-pack out-of-band SIP gateway (the carrier-pinned
attestation chain is rooted at the carrier, not the platform) so
a regional outage at oyatie does not prevent a 911 call from
reaching a PSAP. The attestation cache (§D-3 below) is replicated
per cell so warm-failover is possible without revalidating against
the central issuer.

### §A.4. Performance budget — the ≤200ms p99 invariant

The latency budget for the bypass is **≤200ms p99 edge-to-PSAP-
dispatch**. This is derived from:

- **PSAP human-response budget.** PSAPs target a call-answer time
  of ≤10s p95 per NENA i3; the platform's contribution is the
  edge-to-PSAP-dispatch hop. Allocating ≤200ms p99 from the platform
  hop preserves ~9.8s of PSAP-side budget.
- **Public-safety telecom precedent.** Per FCC Public Notice DA
  18-1106 (Public Safety and Homeland Security Bureau, 2018),
  carrier 911 call-setup is budgeted at ≤2s p95 from origination
  to PSAP-side ring; the platform's contribution at the carrier-
  agnostic internet hop is a fraction of this.
- **Tier-0 edge SLO.** Per ADR-0248 §D-1, Tier-0 edge serves
  ≤50ms p99 for connection acceptance + TLS handshake; the
  remaining ≤150ms p99 covers attestation verification (≤30ms p99
  per §D-3 cache + ≤120ms p99 cold attestation-key fetch).

The budget is exercised in CI by `oya-governance-emergency-
services-latency-budget` (per the naming-justification above).
Cell-level SLO emission per §D-6 surfaces deviation.

### §A.5. The cross-jurisdiction conflict and how this ADR resolves it

Per §3.2.5 row 23, cross-jurisdiction conflicts are resolved by
higher-restriction floor wins. For emergency-services, this rule
inverts: **lower-friction-floor wins for life-safety**. A request
that is emergency-services-attested under any active pack MUST
bypass abuse-defence, even if another active pack would have
imposed friction on a non-emergency-services request. The pack
overlay is union-of-bypass-grants, not intersection.

Specifically:

- A US-resident user submitting a 911 emergency through a tenant
  that has both US-FCC-911 pack + EU-EECC-112 pack active is bypassed
  on the US-FCC-911 attestation chain (the EU-EECC-112 attestation
  is not required).
- A tenant operating in both US + KR territories (HIPAA-US + KR-CSAP
  + KR-NCMPDA-119 packs active) honors 988-Suicide-and-Crisis-
  Lifeline (HIPAA-US pack) AND 119 (KR-NCMPDA-119 pack) independently;
  the user can reach either via the per-pack attestation chain.
- A non-emergency-services request that crosses jurisdictions
  continues to obey the higher-restriction-floor-wins rule from
  §3.2.5 row 23.

This conflict-resolution rule is encoded in §D-4 below.

## Decision

### §B. Decision summary

**Decision 1: Three-layer Tier-0 edge bypass primitive.** Every
internet-facing surface (Edge Gateway, API Gateway, AsyncAPI
broker, SMTP MTA, SIP gateway, WebRTC gateway) routes through the
canonical attestation-verifier at the Tier-0 edge. Verification
results are forwarded as the `X-Oya-Emergency-Attestation` +
`X-Oya-Emergency-Pack` headers (per the naming-justifications
above) plus the verified SPIFFE workload identity. The verifier
is `oya-shared-emergency-services-bypass` — a single shared crate
per the naming-justification, vendored into every µservice.

**Decision 2: Cedar-policy-first composition.** The emergency-
services-bypass Cedar fragment is the FIRST fragment in every
µservice's policy chain (before the abuse-defence fragment from
ADR-0297, before per-µservice business-logic fragments). The
fragment grants `permit` with `request.is_emergency_services ==
true && request.emergency_attestation_verified == true`, plus
the per-pack overlay grant from §D-4.

**Decision 3: Distinct audit-event-class family.** Four new audit-
event classes (`EmergencyServiceBypassGranted`,
`AbuseDefenceEmergencyServiceBypass`, `EmergencyServiceForgery
Detected`, `EmergencyServiceRateLimitElevation`) are registered in
the ADR-0263 central registry. Every emergency-services request
emits at least the first; abuse-defence-deferred emits the second;
post-hoc forgery detection emits the third; mass-casualty surge
emits the fourth.

**Decision 4: ≤200ms p99 edge-to-PSAP-dispatch latency budget.**
Per §A.4 above. Exercised in CI by `oya-governance-emergency-
services-latency-budget`.

**Decision 5: ≤60s revocation cascade.** A confirmed forged
emergency-services principal is revoked across every cell in ≤60s
via the per-pack revocation list polling. Exercised in CI by
`oya-governance-emergency-services-revocation-window`.

**Decision 6: No rate-limit invariant.** An attested emergency-
services principal is NEVER rate-limited at the edge, per-µservice,
or per-Cedar-policy gate. Cells implement an elevated rate-limit
floor per §D-6 to absorb mass-casualty surge without dropping
emergency-services traffic. Exercised in CI by `oya-foundry-
fitness-emergency-services-no-rate-limit-floor`.

**Decision 7: Per-pack attestation registry.** The canonical
machine-readable registry `spec/emergency-services-registry.json`
(per the naming-justification) lists every emergency-services
principal with its compliance pack, jurisdiction, attestation
source, attestation key reference, revocation-window target, and
regulatory anchor. The registry is per-pack-overlay-extensible —
each pack adds its principals — and is enforced via `oya-foundry-
fitness-emergency-services-per-pack-registry`.

**Decision 8: Bidirectional attestation for outbound emergency
flow.** When the platform initiates an outbound emergency-services
request (e.g., a tenant µservice forwarding a CSAM detection to
NCMEC CyberTipline, or routing a 988 reach-out via the federated
crisis-line SSO), the platform's own credential carries a SPIFFE
SVID that the destination MUST verify. The bypass works in both
directions.

## Consequences

### §C. Consequences across all 6 engineering-rigor dimensions

Per documentation-rigor.md §1.2 engineering-rigor dimensions matrix,
this ADR addresses all six:

#### §C.1. Maintainability

- **Module boundaries.** The bypass logic is encapsulated in
  `oya-shared-emergency-services-bypass` (single crate, single
  concern per ADR-0131 + ADR-0132). The Cedar fragment is in a
  single file per µservice (`policy/emergency-services-bypass.cedar`).
  The IaC manifest is in a single file per env (`iac/<env>-
  emergency-services-ingress.yaml`). No "scattered logic" across
  every µservice; the bypass is the shared crate plus one Cedar
  fragment plus one IaC file per pack.
- **Versioning policy.** Per ADR-0258 SemVer policy: the Cedar
  fragment carries a per-fragment SemVer (`policy_version: 1.0.0`);
  the shared crate carries Cargo SemVer; the IaC manifest carries
  a per-env version pin. Any MAJOR bump triggers a soak window per
  ADR-0294 (≥60s) and a sunset window per ADR-0258 (≥90 days for
  deprecation, ≥6 months for removal).
- **Deprecation cadence.** Per-pack attestation sources may be
  deprecated when the regulator publishes a new attestation source
  (e.g., NENA i3 published NENA-STA-010.3a updated 2024; the prior
  NENA-STA-010.3 attestation source is in a 2-year sunset). The
  shared crate retains the deprecated source under a feature flag
  for the sunset window.
- **Reverse-dependencies enumerated.** Every µservice that consumes
  the bypass is declared in `manifest.json:reverse_consumers_of_
  emergency_services_bypass`. The shared-crate maintainer can
  enumerate downstream consumers in CI via `cargo tree --invert`.
- **What is hard-coded vs configurable.** Hard-coded: the
  audit-event-class slugs, the SPIFFE-URI principal namespace
  shape, the latency-budget invariant. Configurable: per-pack
  attestation sources, per-pack revocation-window, per-cell
  elevated rate-limit floor (within the no-limit-on-attested
  invariant), per-tenant `audience_type=EMERGENCY_SERVICES` opt-in.

#### §C.2. Observability

- **Emitted metrics.** Per ADR-0263 cardinality budget — see §D-6
  for the full metric roster. Top-level metrics include:
  - `emergency_services_bypass_granted_total` (counter, dimensions:
    pack, principal_namespace, µservice, cell, cardinality budget
    ≤2k series total).
  - `emergency_services_attestation_verification_latency_seconds`
    (histogram, dimensions: pack, attestation_source, cell, le
    buckets `1ms 5ms 10ms 25ms 50ms 100ms 200ms 500ms`, cardinality
    ≤500 series).
  - `emergency_services_forgery_detected_total` (counter,
    dimensions: pack, principal_namespace, detection_path
    (attestation-fail, audit-time, revocation-polling), cardinality
    ≤1k series).
  - `emergency_services_rate_limit_elevation_total` (counter,
    dimensions: cell, pack, elevation_class (mass-casualty,
    regional-surge, scheduled-drill), cardinality ≤300 series).
- **Trace span shape.** Every emergency-services request carries a
  top-level trace span `emergency_services_bypass.verify` with
  child spans `emergency_services_bypass.attestation_fetch`,
  `emergency_services_bypass.cedar_evaluate`, `emergency_services_
  bypass.audit_emit`. Parent-child rules: parent span MUST be the
  ingress request span; child spans MUST link to the parent via
  W3C Trace Context.
- **Logs.** Every bypass decision emits a JSON-structured log line
  at INFO level with: tenant_id, principal_slug, pack, attestation_
  source, attestation_verified_at, latency_ms, audit_event_id.
  Retention class: ≥7 years for emergency-services log retention
  (per FCC + FTC + EU EECC retention requirements). Retention is
  pack-extensible.
- **Audit events.** Per ADR-0263 emission contract — the four
  classes from Decision 3 above. Audit chain Merkle-sealed per
  ADR-0028.
- **SLO floor.** P99 edge-to-PSAP-dispatch latency ≤200ms. P99.9
  ≤500ms. Error budget: 0.001% — every dropped emergency-services
  request is a SEV0 incident.
- **Dashboards.** `dashboards/emergency-services-bypass.json` (per
  µservice) + `dashboards/emergency-services-platform-wide.json`
  (substrate-level, served from the observability µservice). The
  platform-wide dashboard correlates with the per-cell `dashboards/
  cell-tier-0-traffic.json`.

#### §C.3. Scalability

- **Capacity math.** Baseline emergency-services traffic across
  the active packs is ~10k req/sec sustained (derived: US 911 ~240M
  calls/yr ≈ 7.6 req/sec average + EU 112 ~150M calls/yr ≈ 4.8 req/
  sec + KR 119 ~10M calls/yr ≈ 0.3 req/sec + JP J-ALERT ~rare-but-
  spikey ≤500k devices receiving in single event + AU/NZ Triple
  Zero ~4M calls/yr + UK 999 ~30M calls/yr ≈ 1 req/sec + NCMEC
  CyberTipline ~30M reports/yr ≈ 1 req/sec + HHS 988 ~5M outreach/
  yr ≈ 0.16 req/sec, with the platform-attested share being a
  small fraction of these underlying carrier-level numbers, scaled
  up by abuse-attempt-floor + the mass-casualty surge factor of
  ~100×).
- **Mass-casualty surge.** A mass-casualty surge drives ≤1M req/sec
  peak across the active packs (100× baseline) per §3.2.5 row 22.
  The cell-tier-0 edge MUST absorb this without dropping any
  attested request. Per ADR-0248 §D-1, cell-tier-0 capacity
  ceiling is sized at 10M req/sec per cell — the surge headroom is
  ≥10×. Cell-tier-0 horizontal scale-out is achieved via shuffle
  sharding per ADR-0248 §D-2 (a single cell-tier-0 instance is
  sharded across ~16 sub-cells, each handling ~625k req/sec).
- **Bottleneck identification.** The single bottleneck is the
  per-pack attestation-key fetch on a cold cache miss (≤120ms p99
  cold per §A.4). The mitigation is the per-cell attestation cache
  (§D-3) with a 24-hour TTL + asynchronous revocation polling.
  Warm-cache hit rate ≥99.9% under normal operation; cold-cache
  miss rate ≤0.1%.
- **Horizontal scale-out path.** Adding cells per ADR-0248 §D-3
  scales the bypass horizontally. The shared crate is stateless
  beyond its per-cell cache; cells can be added without coordinated
  cache invalidation.

#### §C.4. Performance

- **P50/P95/P99 latency targets.** P50 ≤25ms; P95 ≤80ms; P99
  ≤200ms; P99.9 ≤500ms. All measured edge-ingress-to-PSAP-dispatch.
  Error bars: ±10ms p99 over a 24-hour rolling window.
- **Modeling note.** The latency budget is dominated by attestation
  verification (≤30ms p99 warm + ≤120ms p99 cold) + Cedar evaluation
  (≤5ms p99 per ADR-0246 amendment library-first dispatch) + audit
  emission (≤10ms p99 per ADR-0263 emission contract). The
  remainder (≤35ms) covers the request-path overhead.
- **Per-region budget split.** Per ADR-0240 sovereign-cloud-per-
  regional-pack, the budget is honored per region: US (FCC, NENA)
  targets ≤200ms p99; EU (EECC, eIDAS) targets ≤200ms p99; KR
  (NCMPDA, K-CSAP) targets ≤250ms p99 (slightly relaxed due to
  Real-Name-Verification handshake); JP (J-ALERT) targets ≤200ms
  p99; AU/NZ targets ≤250ms p99 (extended due to satellite-link
  fallback); UK targets ≤200ms p99.
- **Tail-latency mitigation.** Per ADR-0297 §D-2 hedging-and-take-
  first pattern applied to attestation-key fetch under cold cache:
  fire the fetch to both the per-cell cache + the per-pack OpenBao
  source simultaneously, take the first response. Circuit-breaker
  on per-pack attestation-key fetch with a 5-second open-circuit
  fall-through to the local attestation cache. Cold-start budget:
  the verifier carries a pre-warmed cache populated at cell-startup
  via per-pack pre-fetch.
- **Cold-vs-warm path latency separation.** Warm: ≤30ms p99. Cold:
  ≤120ms p99. The cold path is exercised only on cell-startup +
  cache-TTL-expiry; the warm path serves ≥99.9% of requests.

#### §C.5. Optimization

- **Per-call cost model.** CPU ≤2ms (attestation verification:
  Ed25519 + ChaCha20-Poly1305 + Merkle-path check); RAM ≤512KB
  (per-call attestation context); IOPS ≤0.001 per request (cache
  hit) / ≤1 per request (cache miss); $/M-requests ≤$0.50 at the
  shared-crate boundary (excluding per-pack attestation-source
  egress; per-pack costs are per ADR-0240 sovereign-cloud-per-
  regional-pack).
- **Lazy vs eager.** Cedar evaluation is eager (the bypass MUST
  resolve before any other Cedar fragment evaluates). Attestation-
  key fetch is eager-with-pre-warm (per-cell startup + per-pack
  pre-fetch). Audit emission is lazy-with-flush (per ADR-0263
  emission contract, audit-event flush cadence is ≤1s p99).
- **Cache-invalidation policy.** Per-cell attestation cache TTL is
  24 hours per default. Revocation polling cadence is ≤60s (per
  §D-7 below). On revocation, the cache entry is purged
  immediately and the Cedar fragment re-evaluates.
- **Profiling evidence link.** `tools/profiling/emergency-services-
  bypass-baseline.json` — produced from a hyperscaler-grade load
  test simulating mass-casualty surge.

#### §C.6. Code quality

- **Required test classes.** Unit tests (Cedar fragment, attestation
  verifier, audit emitter), property tests (attestation chain
  invariants), fuzz tests (attestation parser hardening), load
  tests (mass-casualty surge), e2e tests (full path from edge
  ingress to PSAP dispatch under a synthetic PSAP).
- **Coverage floor.** ≥90% line, ≥80% branch for `oya-shared-
  emergency-services-bypass` (above the standard ≥85% line / ≥75%
  branch floor due to life-safety stakes).
- **Lint passes.** `oya-check-cedar-fragment-soak` (per ADR-0294)
  + `oya-check-spiffe-uri-conformance` (per ADR-0295) +
  `oya-check-audit-event-class-registration` (per ADR-0263) +
  `oya-check-naming-justification-block` (per
  `feedback_naming_justification`).
- **Type-strictness.** Rust `deny(warnings) + deny(missing_docs) +
  deny(unsafe_op_in_unsafe_fn)`; TypeScript surface (admin UI for
  per-pack registry editor) `strict + noUncheckedIndexedAccess`.
- **SemVer + ABI policy.** Per ADR-0258. The shared crate's
  `oya_shared_emergency_services_bypass::EmergencyAttestation`
  trait is a public-stable surface; ABI changes require an ADR.

## Detailed mechanics

### §D. Detailed mechanics

#### §D-1. Attestation chain — per-pack roster

Every emergency-services principal carries an attestation chain
that the Tier-0 edge verifies. The chain shape is per-pack:

| # | Pack | Principal namespace | Attestation source | Attestation primitive | Revocation target |
|---:|---|---|---|---|---|
| 1 | `pack-us-fcc-9-1-1` | `oyatie.emergency.us.psap.<county>-9-1-1` | NENA i3 SIP-Identity (RFC 8224 STIR/SHAKEN) + per-PSAP X.509 cert pinned to per-pack root CA | SIP `Identity` header signed by carrier-pinned Ed25519 key; X.509 cert chain rooted at NENA federated CA | ≤60s polling against NENA-published OCSP responder + CRL distribution point |
| 2 | `pack-eu-eecc-112` | `oyatie.emergency.eu.esinet.<member-state>-112` | eIDAS Art. 24 qualified-trust-service certificate + ETSI EN 319 412 (Certificate Profiles for Trust Services) | Qualified certificate signed by per-member-state ETSI-accredited QTSP | ≤60s polling against per-member-state EUSIG OCSP responder |
| 3 | `pack-kr-ncmpda-119` | `oyatie.emergency.kr.119.<region>-119` | KR-NCMPDA per-pack key-distribution-center attestation + KR-NIA Real-Name-Verification | NIA-issued attestation token signed by per-region 119 key | ≤60s polling against KR-NCMPDA-published revocation feed |
| 4 | `pack-jp-j-alert` | `oyatie.emergency.jp.j-alert.cabinet-secretariat` | JP Cabinet Secretariat key-distribution-center + per-prefecture relay key | JP Cabinet Secretariat-issued attestation token signed by per-prefecture Ed25519 key | ≤30s polling against JP CS-published revocation feed (faster cadence for J-ALERT) |
| 5 | `pack-au-nz-pac` | `oyatie.emergency.au.tripletzero.000` / `oyatie.emergency.nz.111` | AU Department of Home Affairs carrier-pinned SIP + NZ MBIE per-carrier SIP | Per-carrier Ed25519 key pinned to per-pack root CA | ≤60s polling against AU DHA + NZ MBIE OCSP responders |
| 6 | `pack-uk-ofcom-999` | `oyatie.emergency.uk.999.bt-999` | UK eIDAS-compatible qualified-trust-service per UK eIDAS Regulation 2016 | UK-QTSP issued qualified certificate signed by Ed25519 | ≤60s polling against UK-QTSP OCSP responder |
| 7 | `pack-us-ncmec-cybertipline` | `oyatie.emergency.us.ncmec.cybertipline` | NCMEC HMAC-shared-secret per-platform credential (per 18 USC §2258A operations) | HMAC-SHA-256 signed with per-platform shared secret + replay-window ≤5min | Out-of-band rotation cadence (≤30 days) |
| 8 | `pack-us-hhs-988` | `oyatie.emergency.us.hhs.988-suicide-and-crisis-lifeline` | HHS 988 federated SSO (OIDC + SAML 2.0) with per-platform client credential | OIDC ID token signed by HHS 988 key + JWKS rotation cadence ≤30 days | ≤60s polling against HHS 988 JWKS endpoint |

The shared crate `oya-shared-emergency-services-bypass` carries the
verifier for each attestation primitive. The per-pack registry
file `spec/emergency-services-registry.json` declares the active
roster and binds each principal to its attestation primitive +
revocation target.

#### §D-2. Tier-0 edge verifier — implementation footprint

The Tier-0 edge verifier is wired into `microservices/edge-gateway/`
+ `microservices/api-gateway/` + `microservices/asyncapi-broker/`
+ `microservices/sip-gateway/` + `microservices/smtp-mta/`.
Verification order at the edge:

1. **Receive request.** Extract the candidate attestation bundle
   from one of: `X-Oya-Emergency-Attestation` header (HTTP),
   `Identity` header (SIP per RFC 8224), `Authentication-Results`
   header (SMTP), AsyncAPI message `_emergency_attestation` field,
   or gRPC metadata `oya-emergency-attestation`.
2. **Identify the candidate pack.** Inspect the attestation bundle's
   `iss` (issuer) claim or the SIP `Identity` URI to map to a pack
   in the registry.
3. **Verify the attestation primitive.** Per the per-pack row in
   §D-1: verify the signature, the certificate chain, the
   replay-window, the freshness, the issuer-binding.
4. **Cache the verification result.** Cache key: SHA-256 of the
   attestation bundle. Cache value: `{verified: true, pack:
   "...", principal_namespace: "oyatie.emergency.<pack-suffix>",
   expires_at: <epoch>, attestation_source: "<source>"}`. TTL:
   24 hours OR attestation expiry (whichever earlier).
5. **Forward verified headers.** Set `X-Oya-Emergency-
   Attestation: verified=true; pack=<pack>; principal=<namespace>;
   expires_at=<epoch>` and `X-Oya-Emergency-Pack: <pack>`.
6. **Emit `EmergencyServiceBypassGranted` audit event.** Per
   ADR-0263 emission contract.

#### §D-3. Per-cell attestation cache

The per-cell attestation cache holds verified attestation bundles
+ per-pack issuer keys. Cache shape:

- **Backend.** In-memory (cell-local) with optional Redis-cluster
  L2 (per-cell) for ≤5 cells. The L2 cache is per-cell, not cross-
  cell; cross-cell consistency is achieved via revocation polling,
  not cache replication.
- **Sizing.** ≤10MB per cell. Holds ~50k cached attestation bundles
  + per-pack issuer-key set (~100KB per pack × 8 packs = ~800KB).
- **Eviction.** LRU + TTL. TTL is min(attestation_expiry, 24 hours,
  revocation_window).
- **Pre-warming.** At cell startup, the per-pack issuer keys are
  pre-fetched from the central OpenBao path. The first ~100
  attestation bundles received are subject to the cold-cache
  budget; the rest are warm.

#### §D-4. Cedar fragment — `policy/emergency-services-bypass.cedar`

The canonical Cedar fragment for the emergency-services bypass is
authored once in `policy/emergency-services-bypass.cedar` and
deployed to every internet-facing µservice. The fragment is FIRST
in the Cedar fragment chain (before the abuse-defence fragment
from ADR-0297, before per-µservice business-logic fragments).

```cedar
// policy/emergency-services-bypass.cedar
// Per ADR-0298 emergency-services bypass life-safety hard rule.
// Soak window: per ADR-0294 (≥60s before promotion).
// Maintainer: ops-trust-and-safety + axis-emergency-services.

permit(
  principal,
  action,
  resource
)
when {
  // The request context carries the verified attestation bundle.
  context has emergency_attestation_verified &&
  context.emergency_attestation_verified == true &&
  context has emergency_pack &&
  // The pack must be one of the registered packs.
  context.emergency_pack in [
    "pack-us-fcc-9-1-1",
    "pack-eu-eecc-112",
    "pack-kr-ncmpda-119",
    "pack-jp-j-alert",
    "pack-au-nz-pac",
    "pack-uk-ofcom-999",
    "pack-us-ncmec-cybertipline",
    "pack-us-hhs-988"
  ] &&
  // The principal namespace must match the pack's reserved namespace.
  principal.namespace like "oyatie.emergency.*" &&
  // The attestation must not be expired.
  context.emergency_attestation_expires_at > context.now &&
  // The pack must be active on the destination tenant.
  resource.tenant.active_compliance_packs.contains(context.emergency_pack)
};

// Defence-in-depth FORBID: even if a downstream fragment grants
// access, NEVER permit a non-attested principal to act under an
// emergency-services namespace.
forbid(
  principal,
  action,
  resource
)
when {
  principal.namespace like "oyatie.emergency.*" &&
  !(context has emergency_attestation_verified &&
    context.emergency_attestation_verified == true)
};
```

The fragment is published with a per-fragment soak window per
ADR-0294 (≥60s before promotion). Promotion is gated by
`oya-governance-emergency-services-bypass-cedar-fragment-present`
+ multispectrum review v2.4.0 per ADR-0243 §D-8 facets F1, F2,
F3, F4, F5, F6, F7, F8, F9, M1, M2, A1, A2, A3.

#### §D-5. Per-pack overlay extension

Each pack contributes its own per-pack overlay fragment under
`packs/<pack-slug>/policy/emergency-services-overlay.cedar`. The
overlay adds per-pack principals, per-pack actions, per-pack
resource constraints. The overlay MUST NOT relax the §D-4
default-deny baseline; it can only add additional permits within
the bypass scope.

Example per-pack overlay (`packs/pack-us-fcc-9-1-1/policy/
emergency-services-overlay.cedar`):

```cedar
// Per ADR-0298 + pack-us-fcc-9-1-1 emergency-services overlay.
permit(
  principal in PrincipalGroup::"oyatie_emergency_us_psap",
  action in [Action::"dispatch_emergency",
             Action::"locate_caller",
             Action::"transfer_call"],
  resource in ResourceGroup::"us_psap_resources"
)
when {
  context.emergency_pack == "pack-us-fcc-9-1-1" &&
  context.emergency_attestation_source == "nena-i3-sip-identity"
};
```

#### §D-6. Per-cell-tier variants — observability + rate-limit elevation

The bypass is wired into all four cell tiers per ADR-0248:

**Tier-0 (planetary edge).** WAF + bot-mgmt observe the attested
request (and emit `AbuseDefenceEmergencyServiceBypass`) but do not
gate. Rate-limit floor for attested traffic: NO LIMIT. Per-cell
metric: `cell_tier_0_emergency_services_bypass_granted_per_second`.

**Tier-1 (regional).** API Gateway + AsyncAPI broker re-verify the
forwarded `X-Oya-Emergency-Attestation` header (defence-in-depth
against a compromised Tier-0). Rate-limit floor: 10× the
non-emergency-services baseline.

**Tier-2 (per-tenant).** Per-µservice Cedar evaluation runs the
emergency-services bypass fragment first. Rate-limit floor: NO
LIMIT.

**Tier-3 (offline / air-gapped).** Emergency-services traffic
arrives only via the per-pack out-of-band SIP gateway. The
attestation chain is locally rooted in the per-cell SPIFFE root.

**Mass-casualty surge elevation.** When a cell-tier-0 detects a
≥10× surge in attested emergency-services traffic, it elevates
the cell-tier-1 rate-limit floor proportionally and emits
`EmergencyServiceRateLimitElevation`. The elevation is auto-
triggered by the surge detector; recovery is auto-triggered when
the surge subsides for ≥10 minutes.

Per-cell-tier metrics emit per ADR-0263 cardinality budget:
≤2k series for the `emergency_services_bypass_granted_total`
counter (4 packs × 4 tiers × ~125 cells = ~2k). Other metrics
follow the same budgeting.

#### §D-7. Revocation cascade — ≤60s confirmation-to-suspension

When a per-pack issuer revokes an attestation key (e.g., NENA i3
publishes an OCSP revocation, eIDAS QTSP publishes a CRL update,
KR-NCMPDA publishes a revocation feed entry), the cascade is:

1. **Polling cadence.** Per-cell revocation poller polls per-pack
   revocation sources at ≤60s cadence (≤30s for J-ALERT).
2. **Cache purge.** On revocation hit, the per-cell attestation
   cache purges every entry signed by the revoked key.
3. **Per-cell broadcast.** The cell broadcasts the revocation to
   sibling cells via the per-pack `revocation-broadcast` AsyncAPI
   channel (single-direction, no acknowledgement; idempotent).
4. **Audit emission.** Each cell emits `EmergencyServiceForgery
   Detected` for every previously-permitted request that would
   have been denied under the new revocation.
5. **Cross-cell convergence.** All cells converge on the revocation
   in ≤60s p99 (≤30s p99 for J-ALERT).

#### §D-8. Forgery detection at audit-time

Beyond the cache-purge cascade, the audit chain runs a separate
forgery-detection pipeline. Per ADR-0263 emission contract, every
`EmergencyServiceBypassGranted` event is Merkle-sealed per ADR-0028
+ written to the audit chain. The audit pipeline:

1. **Replay verification.** Every ≤24h, the audit pipeline replays
   the past 24 hours of `EmergencyServiceBypassGranted` events
   against the current revocation list.
2. **Forgery alert.** Any event signed by a now-revoked key emits
   `EmergencyServiceForgeryDetected` + alerts ops-trust-and-safety.
3. **Tenant notification.** Per ADR-0263 §C.6 affected-party
   notification: the tenant whose resource was accessed under the
   forged emergency-services request is notified within ≤24h.
4. **Per-pack regulator notification.** Per ADR-0251 + per-pack
   `breach_notification_workflow_id`: the per-pack regulator is
   notified per their statute. For US-FCC-911, this is the FCC
   Public Safety and Homeland Security Bureau; for EU-EECC-112,
   the per-member-state competent authority; for KR-NCMPDA-119,
   the KR Ministry of the Interior and Safety; etc.

#### §D-9. Bidirectional attestation — outbound flow

When the platform initiates an outbound emergency-services flow
(e.g., a tenant µservice forwards a CSAM detection to NCMEC
CyberTipline per 18 USC §2258A, or routes a 988 outreach via the
federated crisis-line SSO), the platform's own credential carries
a SPIFFE SVID that the destination MUST verify.

Per-pack outbound paths:

- **NCMEC CyberTipline submission.** HMAC signed with the per-
  platform shared secret + replay-window ≤5min + idempotency-key
  required. The platform-side principal is `oyatie.emergency.us.
  ncmec.cybertipline-submitter`.
- **988 federated SSO outreach.** OIDC client-credentials grant
  with the HHS 988 issuer; per-tenant client-id with the platform's
  per-tenant SPIFFE SVID as the binding.
- **EU EECC ESInet relay.** mTLS with the platform's qualified
  certificate per eIDAS Art. 24.

#### §D-10. Per-pack data-residency interaction

Emergency-services data (PII of the caller, location, dispatch
state, transcript) is subject to per-pack data-residency per
ADR-0240 + ADR-0251. The per-pack overlay declares the data-
residency:

- US-FCC-911 + US-NCMEC + US-HHS-988: US-only (per FedRAMP-High
  + HIPAA).
- EU-EECC-112: EU-only (per GDPR Art. 6(1)(d) vital-interests +
  Art. 9(2)(c) protection-of-vital-interests + sovereign-cell EU
  pack overlay per ADR-0240).
- KR-NCMPDA-119: KR-only (per K-CSAP + PIPA Art. 17 +
  cross-border-transfer prohibition).
- JP-J-ALERT: JP-only (per Cabinet Secretariat operational
  requirement).

Per ADR-0240, the per-pack data-residency hard-stop prevents the
emergency-services data from being replicated to a non-pack-
authorized cell, even under cross-region DR-pair failover.

#### §D-11. Per-µservice ARCHITECTURE.md §emergency-services-bypass section

Every internet-facing µservice's ARCHITECTURE.md SHALL include a
§emergency-services-bypass section with:

1. **Surface inventory.** Which ingress paths could carry emergency-
   services traffic (e.g., REST `/v1/emergency/dispatch`,
   AsyncAPI `emergency.dispatch.v1`, SIP `sip:911@<psap>.oyatie.
   com`).
2. **Attestation source map.** For each pack the µservice serves,
   which attestation source is wired (e.g., `pack-us-fcc-9-1-1
   -> nena-i3-sip-identity`).
3. **Cedar fragment reference.** Cite `policy/emergency-services-
   bypass.cedar` + the per-pack overlays.
4. **Audit-event-class emission.** Cite the four audit-event
   classes emitted (`EmergencyServiceBypassGranted`,
   `AbuseDefenceEmergencyServiceBypass`,
   `EmergencyServiceForgeryDetected`,
   `EmergencyServiceRateLimitElevation`).
5. **Latency-budget compliance.** Cite the per-µservice latency-
   budget hook into the CI lane.
6. **Revocation-window compliance.** Cite the per-pack revocation
   polling cadence.
7. **No-rate-limit invariant.** Affirm the no-rate-limit-on-
   attested invariant.

#### §D-12. Multispectrum review v2.4.0 wiring

Per ADR-0243 §D-8 + the multispectrum-review-v2-4-0 doctrine, the
emergency-services bypass is reviewed across facets:

- F1 (security): attestation chain integrity + revocation cascade.
- F2 (privacy): per-pack data-residency + PII-of-the-caller
  isolation.
- F3 (reliability): mass-casualty surge + regional outage failover.
- F4 (performance): ≤200ms p99 latency budget.
- F5 (cost): per-call cost model + per-cell cache sizing.
- F6 (operability): per-cell observability + audit-event-class
  emission.
- F7 (compliance): per-pack regulatory anchor + per-statute citation.
- F8 (user safety): the life-safety hardrule + no-friction floor.
- F9 (accessibility): bypass works across the disability-
  accommodations critical path (§3.2.5 row 12).
- M1 (meta-policy): Cedar fragment chain ordering + soak window.
- M2 (meta-architecture): the bypass is a substrate primitive,
  not a per-µservice afterthought.
- A1 (own-naming): the SPIFFE-URI principal namespace + audit-
  event-class slugs conform to BNF v4.1.
- A2 (own-documentation): ARCHITECTURE.md §emergency-services-
  bypass section is present in every internet-facing µservice.
- A3 (own-structure): the shared crate is single-concern + flat
  layout per ADR-0131 + ADR-0132.

#### §D-13. Failure-mode tree — explicit walk-through

Per documentation-rigor.md §1.1 rigor sub-test "failure-mode tree",
the explicit failure modes are:

**FM-1: Forged emergency-services claim from a hostile actor.**
Mitigation: cryptographic attestation chain (§D-1) + ≤60s
revocation cascade (§D-7) + audit-time forgery detection (§D-8).
Detection latency: ≤1 request (attestation verification fails
at gate-time) OR ≤24h (audit-time replay verification). Never
gate-blocking on suspicion.

**FM-2: Mass-casualty surge of legitimate emergency-services
traffic.** Mitigation: cell-tier-0 capacity ceiling of 10M req/sec
(≥10× headroom over 1M req/sec peak) + elevated rate-limit floor
on cell-tier-1 (§D-6) + DR-pair failover per ADR-0241. Per-cell
metrics surface the surge in real-time.

**FM-3: Regional outage hits the cell hosting the attestation
verifier.** Mitigation: per-cell attestation cache (§D-3) +
warm-failover via cross-cell broadcast (§D-7) + per-pack out-of-
band SIP gateway fallback for cell-tier-3.

**FM-4: Per-pack issuer key compromise.** Mitigation: per-pack
revocation cascade (§D-7) + audit-time forgery detection (§D-8)
+ per-pack regulator notification (§D-10) + per-pack key rotation
(≤30 days for production, ≤7 days for J-ALERT).

**FM-5: Audit-chain compromise.** Mitigation: Merkle-sealed audit
chain per ADR-0028 + per-µservice signing key in sidecar per
ADR-0296 + per-pack regulator-reachable audit-chain replication.

**FM-6: Cross-pack overlay collision.** Mitigation: the Cedar
fragment §D-4 default-deny baseline + per-pack overlay constraint
(§D-5) + cross-pack conflict resolution rule from §A.5 (union-
of-bypass-grants for life-safety).

**FM-7: Latency budget breach.** Mitigation: per-cell SLO emission
(§D-6) + CI lane gating (§B Decision 4) + per-cell pre-warm
strategy (§D-3) + hedging on cold cache miss (§C.4).

**FM-8: Bidirectional outbound flow failure.** Mitigation: per-
pack outbound credential rotation (§D-9) + per-µservice circuit-
breaker on outbound emergency-services flow + escalation runbook
to ops-trust-and-safety.

## Implementation footprint

### §E. Implementation footprint — files, crates, schemas

#### §E.1. New crate: `crates/oya-shared-emergency-services-bypass/`

```text
crates/oya-shared-emergency-services-bypass/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs              // Public surface (traits + types)
│   ├── attestation/
│   │   ├── mod.rs
│   │   ├── nena_i3.rs      // NENA i3 SIP-Identity verifier
│   │   ├── eidas.rs        // eIDAS Art. 24 qualified-certificate verifier
│   │   ├── kr_ncmpda.rs    // KR-NCMPDA per-region attestation verifier
│   │   ├── jp_j_alert.rs   // JP J-ALERT key-distribution-center verifier
│   │   ├── au_nz_pac.rs    // AU/NZ Triple Zero + 111 verifier
│   │   ├── uk_ofcom.rs     // UK OFCOM 999 verifier
│   │   ├── ncmec_hmac.rs   // NCMEC CyberTipline HMAC verifier
│   │   └── hhs_988.rs      // HHS 988 OIDC verifier
│   ├── cache/
│   │   ├── mod.rs
│   │   ├── in_memory.rs    // Cell-local cache
│   │   └── redis_l2.rs     // Optional Redis L2 cache
│   ├── revocation/
│   │   ├── mod.rs
│   │   ├── poller.rs       // Per-pack revocation polling
│   │   └── broadcast.rs    // Cross-cell revocation broadcast
│   ├── audit.rs            // Audit event emission (per ADR-0263)
│   ├── cedar.rs            // Cedar fragment loader + evaluator
│   ├── metrics.rs          // Metric emission (per ADR-0263)
│   ├── traces.rs           // Trace span emission
│   └── error.rs            // Error types
├── tests/
│   ├── attestation_chain_integrity.rs
│   ├── mass_casualty_surge.rs
│   ├── revocation_cascade.rs
│   ├── cross_cell_convergence.rs
│   └── forgery_detection.rs
└── benches/
    ├── attestation_verification_latency.rs
    └── cache_lookup_latency.rs
```

The crate exposes the public trait:

```rust
// crates/oya-shared-emergency-services-bypass/src/lib.rs
pub trait EmergencyAttestation: Send + Sync {
    /// Verify the candidate attestation bundle against the per-pack
    /// attestation source.
    fn verify(
        &self,
        bundle: &AttestationBundle,
        pack: &CompliancePackId,
    ) -> Result<VerifiedAttestation, AttestationError>;

    /// Poll the per-pack revocation source.
    fn poll_revocation(
        &self,
        pack: &CompliancePackId,
    ) -> Result<RevocationDelta, AttestationError>;

    /// Emit the audit event for a granted bypass.
    fn emit_audit(
        &self,
        attestation: &VerifiedAttestation,
        request: &EmergencyRequest,
    ) -> Result<(), AttestationError>;
}
```

#### §E.2. Cedar fragment: `microservices/<ms>/policy/emergency-services-bypass.cedar`

Per §D-4 above. Deployed to every internet-facing µservice.

#### §E.3. IaC manifest: `microservices/<ms>/iac/<env>-emergency-services-ingress.yaml`

Per-µservice + per-env IaC manifest. Schema:

```yaml
apiVersion: oyatie.io/v1
kind: EmergencyServicesIngress
metadata:
  name: <ms>-emergency-services-ingress
  microservice: <ms>
  env: <env>
spec:
  packs:
    - id: pack-us-fcc-9-1-1
      attestation_source: nena-i3-sip-identity
      attestation_key_ref: ${openbao:secret/<tenant_id>/emergency/us-fcc-911/key}
      revocation_window_seconds: 60
      data_residency: us-only
    # ... per-pack roster
  latency_budget_ms_p99: 200
  no_rate_limit_invariant: true
  cell_tier_variants:
    - tier: 0
      rate_limit: NO_LIMIT
    - tier: 1
      rate_limit_floor_multiplier: 10
    - tier: 2
      rate_limit: NO_LIMIT
    - tier: 3
      attestation_source: per-cell-spiffe-root
  observability:
    metrics: true
    traces: true
    audit_events:
      - EmergencyServiceBypassGranted
      - AbuseDefenceEmergencyServiceBypass
      - EmergencyServiceForgeryDetected
      - EmergencyServiceRateLimitElevation
```

#### §E.4. Registry: `specs/emergency-services-registry.json`

JSON Schema per documentation-rigor.md §2 spec rigor:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://oyatie.io/specs/emergency-services-registry.json",
  "title": "Emergency Services Registry",
  "version": "1.0.0",
  "_meta": {
    "purpose": "Canonical machine-readable registry of all emergency-services principals across active packs.",
    "industry_citations": ["NENA-STA-010.3a", "eIDAS-Art-24", "18-USC-2258A", "HHS-988-federated-SSO"],
    "related_adrs": ["ADR-0298"],
    "binding_adr": "ADR-0298",
    "status": "Proposed",
    "enforcement_status": "advisory-until-2026-08-15-blocker-thereafter"
  },
  "type": "object",
  "required": ["principals"],
  "properties": {
    "principals": {
      "type": "array",
      "items": {
        "$ref": "#/$defs/Principal"
      }
    }
  },
  "$defs": {
    "Principal": {
      "type": "object",
      "required": ["slug", "namespace", "pack", "jurisdiction", "attestation_source", "attestation_key_ref", "revocation_window_seconds", "regulatory_anchor"],
      "properties": {
        "slug": { "type": "string", "description": "Slug-shape principal identifier (e.g., 'king-county-9-1-1').", "examples": ["king-county-9-1-1", "frankfurt-112"] },
        "namespace": { "type": "string", "description": "SPIFFE-URI namespace (e.g., 'oyatie.emergency.us.psap.king-county-9-1-1').", "examples": ["oyatie.emergency.us.psap.king-county-9-1-1"] },
        "pack": { "type": "string", "description": "Compliance pack identifier.", "examples": ["pack-us-fcc-9-1-1"] },
        "jurisdiction": { "type": "string", "description": "ISO 3166-1 alpha-2 jurisdiction code + optional sub-division.", "examples": ["US-WA", "DE-HE", "KR-11"] },
        "attestation_source": { "type": "string", "description": "Per-pack attestation primitive identifier.", "examples": ["nena-i3-sip-identity", "eidas-qualified-certificate", "ncmec-hmac"] },
        "attestation_key_ref": { "type": "string", "description": "OpenBao path to the per-pack attestation key material.", "examples": ["${openbao:secret/oyatie/emergency/us-fcc-911/nena-i3-king-county}"] },
        "revocation_window_seconds": { "type": "integer", "minimum": 1, "maximum": 60, "description": "Maximum allowed revocation window in seconds.", "examples": [60, 30] },
        "regulatory_anchor": { "type": "string", "description": "Statute citation.", "examples": ["47 CFR §9.10", "EU EECC Article 109", "18 USC §2258A"] }
      }
    }
  }
}
```

#### §E.5. Per-pack overlay files: `packs/<pack-slug>/policy/emergency-services-overlay.cedar`

One overlay per pack, per §D-5.

#### §E.6. CI lanes: seven lanes per §B Decision

```text
.github/workflows/oya-governance-emergency-services-bypass.yml
.github/workflows/oya-governance-emergency-services-attestation-chain.yml
.github/workflows/oya-governance-emergency-services-latency-budget.yml
.github/workflows/oya-governance-emergency-services-no-rate-limit-floor.yml
.github/workflows/oya-governance-emergency-services-revocation-window.yml
.github/workflows/oya-governance-emergency-services-per-pack-registry.yml
.github/workflows/oya-governance-emergency-services-cedar-fragment-present.yml
```

Each lane invokes the corresponding `cloud-ci/Rust gate packet ...` per the
`enforced_by` block of this ADR's frontmatter.

#### §E.7. Per-µservice manifest.json fields

Per-µservice `manifest.json` adds:

```json
{
  "emergency_services_bypass": {
    "enabled": true,
    "active_packs": ["pack-us-fcc-9-1-1", "pack-eu-eecc-112", "..."],
    "cedar_fragment": "policy/emergency-services-bypass.cedar",
    "iac_manifest": "iac/<env>-emergency-services-ingress.yaml",
    "latency_budget_ms_p99": 200,
    "reverse_consumers_of_emergency_services_bypass": ["..."]
  }
}
```

## Migration

### §F. Migration plan

#### §F.1. Phase 0 — Doctrine acceptance (2026-05-20 — 2026-05-27)

This ADR accepted in text. The shared crate skeleton, Cedar fragment
skeleton, and per-pack registry skeleton land. CI lanes promote to
advisory.

#### §F.2. Phase 1 — Per-pack attestation source onboarding (2026-05-27 — 2026-07-15)

Onboard each pack in priority order:

1. **pack-us-fcc-9-1-1** (US 911 / NENA i3) — highest volume.
2. **pack-eu-eecc-112** (EU 112 / ESInets) — second-highest volume.
3. **pack-us-ncmec-cybertipline** (NCMEC) — statutory mandate.
4. **pack-us-hhs-988** (988 Suicide and Crisis Lifeline) — minor-
   protection priority.
5. **pack-kr-ncmpda-119** (KR 119) — KR-CSAP pack alignment.
6. **pack-uk-ofcom-999** (UK 999) — Brexit-era eIDAS pre-onboarding.
7. **pack-au-nz-pac** (AU Triple Zero + NZ 111) — partner-carrier
   coordination.
8. **pack-jp-j-alert** (JP J-ALERT) — Cabinet Secretariat liaison.

For each pack: provision attestation key material in OpenBao;
register principals in `specs/emergency-services-registry.json`;
deploy per-pack overlay Cedar fragment; exercise integration tests
against the per-pack issuer.

#### §F.3. Phase 2 — Per-µservice wiring (2026-06-15 — 2026-07-15)

Every internet-facing µservice (per the §A.2 list) adds the §D-11
ARCHITECTURE.md section + the Cedar fragment + the IaC manifest +
the manifest.json fields. CI lanes promote to BLOCKER on 2026-08-15.

#### §F.4. Phase 3 — Production rollout (2026-07-15 — 2026-08-15)

Per-cell rollout via canary cell-tier-1 → cell-tier-0 promotion.
Per-µservice rollout is gated by the seven CI lanes. Mass-casualty
surge load-test against a synthetic PSAP.

#### §F.5. Phase 4 — BLOCKER promotion (2026-08-15)

The seven CI lanes promote to BLOCKER. Any internet-facing µservice
without a wired emergency-services bypass fails CI.

#### §F.6. Rollback path

Per ADR-0294 Cedar fragment emergency rollback runbook: per-fragment
rollback within ≤5 minutes of detection of a high-severity bypass
defect. Per-pack rollback within ≤15 minutes (revert the per-pack
overlay only). Platform-wide rollback within ≤30 minutes
(revert the §D-4 fragment to the prior soak-stable version).

Rollback DOES NOT close emergency-services traffic; it falls back
to the per-pack out-of-band SIP gateway path which is locally rooted
in the per-cell SPIFFE root.

## References

### §G. References

#### §G.1. Hyperscaler precedents

- Apple iOS Emergency SOS via Satellite white paper:
  `support.apple.com/en-us/102099`
- Google Android Emergency Location Service developer documentation:
  `developer.android.com/guide/topics/connectivity/location-emergency`
- Cloudflare Project Galileo (emergency-bypass + DDoS-absorption):
  `blog.cloudflare.com/galileo`
- Cloudflare Athenian Project (election-infrastructure protection):
  `blog.cloudflare.com/athenian`
- AWS GovCloud + Top Secret-East/West FedRAMP-High SSP documentation
  (publicly available since 2020).
- WhatsApp emergency-call carry-through (Meta platform documentation
  2023).
- Signal E.164-fallback to native dialer on 911-pattern detection
  (signalapp/Signal-Android open source).

#### §G.2. Regulatory anchors

- **US — FCC 47 CFR §9** (Emergency Services / 911) + Public Notice
  DA 18-1106 (PSAP call-setup latency budget).
- **US — 18 USC §2258A** (NCMEC CyberTipline mandatory reporting).
- **US — HHS 988 Suicide and Crisis Lifeline** (federated SSO
  operational requirements per HHS 2022).
- **US — FedRAMP-High** (continuous-monitoring requirements for
  emergency-services-eligible cells).
- **EU — Directive (EU) 2018/1972 (EECC) Article 109** (Emergency
  Services / 112 / ESInets).
- **EU — Regulation (EU) 910/2014 (eIDAS) Article 24** (qualified-
  trust-service requirements).
- **EU — ETSI EN 319 412** (Certificate Profiles for Trust Services).
- **KR — National Counter-Terrorism Manual for Public Disaster
  Authority (NCMPDA)** (KR-119 operational requirements).
- **KR — KR-CSAP** + **PIPA Article 17** (cross-border-transfer
  prohibition for emergency-services data).
- **JP — Cabinet Secretariat J-ALERT operational manual** (key-
  distribution-center attestation).
- **AU — Department of Home Affairs Triple Zero (000) operational
  manual** (carrier-pinned SIP attestation).
- **NZ — Ministry of Business, Innovation and Employment (MBIE)
  111 operational manual** (per-carrier SIP attestation).
- **UK — UK eIDAS Regulation 2016/EU910 transposition** + **Ofcom
  999 operational manual** (BT 999 carrier-pinned SIP).
- **NENA — NENA-STA-010.3a** (i3 Standard for Next Generation 911,
  current revision 2024).
- **IETF — RFC 8224 STIR/SHAKEN** (Authenticated Identity Management
  in SIP).
- **IETF — RFC 9116** (security.txt).

#### §G.3. Keystone bundle 2026-05-20 cross-references

- **ADR-0297** (abuse-defence baseline — anti-bot, anti-spoof,
  anti-scrape): the §D-12 multispectrum review wires the bypass
  before the abuse-defence fragment in the Cedar chain. The
  abuse-defence layer observes and audits via
  `AbuseDefenceEmergencyServiceBypass` but never gates an attested
  emergency-services request.
- **ADR-0242** (oyatie-is-a-tenant doctrine): emergency-services
  principals occupy the `oyatie.emergency.*` namespace under the
  platform tenant; no carve-out from the tenant model.
- **ADR-0243** (Cedar universal gate): the bypass is a Cedar policy
  decision; first in the policy chain.
- **ADR-0244** (tenant scoping primitive): the bypass adds the
  `EMERGENCY_SERVICES` enum value to `Tenant.audience_type`.
- **ADR-0246** (policy-engine library-first): the library-first
  Cedar evaluator carries the bypass fragment.
- **ADR-0248** (Amazon-shape cellular architecture): per-cell-tier
  variants per §D-6.
- **ADR-0250** (build-ahead-of-certification): the bypass is built
  certified-shape day one across all eight packs.
- **ADR-0251** (compliance packs): per-pack overlay extension per
  §D-5.
- **ADR-0252** (HLC + TrueTime): attestation freshness window
  resolved against HLC (default) or TrueTime (for the high-
  precision J-ALERT cadence).
- **ADR-0253** (HTTP/3 + QUIC default): emergency-services traffic
  preferentially uses HTTP/3 + QUIC.
- **ADR-0263** (observability emission contract): the four new
  audit-event classes registered.
- **ADR-0272** (cookie consent per-purpose): N/A (emergency-services
  consent is implicit per GDPR Art. 6(1)(d) vital interests).
- **ADR-0273** (per-tenant DKIM/SPF/DMARC): N/A unless the µservice
  sends emergency-services notification email.
- **ADR-0276** (backup portability GDPR Art. 20): emergency-
  services data is portable but bounded by per-pack data-residency
  per ADR-0240.
- **ADR-0280** (substrate-of-substrate dependency): the shared
  crate is at the substrate level; depends on `oya-shared-cedar-
  evaluator` + `oya-shared-audit-emit`.
- **ADR-0284** (platform-owner name indirection): the namespace
  `oyatie.emergency.*` is parameterized via the platform-owner
  indirection.
- **ADR-0292** (minor user doctrine): minor reaching for 988
  Suicide and Crisis Lifeline bypasses the parental-control surface.
- **ADR-0293** (meta-trust-root): the shared crate's signing key
  is rooted at the meta-trust-root.
- **ADR-0294** (Cedar fragment soak): ≥60s soak window respected.
- **ADR-0295** (bootstrap CI SPIFFE + kill-switch): per-cell
  attestation verifier SVID + kill-switch.
- **ADR-0296** (library-first credential sidecar): per-pack
  attestation keys held in sidecar with ≤60s OpenBao TTL.

#### §G.4. Companion docs

- `docs/standards/documentation-rigor.md` §3.2.3 (LIFE-SAFETY HARD
  RULE) + §3.2.5 row 1 (Emergency Services).
- `docs/runbooks/emergency-services-bypass-on-call.md` (per-pack
  on-call escalation).
- `docs/runbooks/emergency-services-revocation-cascade.md` (≤60s
  revocation cascade runbook).
- `docs/runbooks/emergency-services-mass-casualty-surge.md` (mass-
  casualty surge runbook).
- `docs/runbooks/emergency-services-forgery-detection.md` (audit-
  time forgery detection runbook).

#### §G.5. Cross-back-pointer follow-ups for existing ADRs

The following existing ADRs require a cross-back-pointer to ADR-
0298 (filed for the Wave-3-D editorial pass):

- **ADR-0297** (abuse-defence baseline): cross-reference ADR-0298
  in §D-4 Cedar fragment composition note + §G keystone bundle list.
- **ADR-0263** (observability emission contract): register the four
  new audit-event classes in §D-N central registry.
- **ADR-0248** (cellular architecture): cross-reference §D-6 per-
  cell-tier variants.
- **ADR-0244** (tenant scoping primitive): cross-reference the new
  `EMERGENCY_SERVICES` enum value in §D-3 enum.
- **ADR-0292** (minor user doctrine): cross-reference the 988
  Suicide and Crisis Lifeline bypass overriding parental-control
  surface.
- **ADR-0240** (sovereign-cloud-per-regional-pack): cross-reference
  the per-pack data-residency interaction in §D-10.
- **ADR-0251** (compliance packs): cross-reference the per-pack
  emergency-services overlay convention.

## Change log

### §H. Change log

| Date | Author | Change |
|---|---|---|
| 2026-05-20 | council-architecture + axis-emergency-services | Initial Proposed status; bundled with the keystone-bundle 2026-05-20 foundational doctrine as the critical-path-doctrine-cluster-row-1 keystone. Authored per documentation-rigor.md §3.2.3 LIFE-SAFETY HARD RULE + §3.2.5 row 1. Cross-references the entire keystone bundle 2026-05-20 (ADR-0242..0258 + 0263 + 0272-0292 + 0293-0296 + 0297). |
