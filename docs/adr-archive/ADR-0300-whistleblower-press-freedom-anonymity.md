---
id: ADR-0300
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
  - axis-anonymity
  - axis-whistleblower-channel
supersedes: []
amends: []
superseded_by: [ADR-707]
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
related_specs:
  - /specs/platform-architecture.json
  - /specs/microservices/identity.json
  - /specs/microservices/policy-engine.json
  - /specs/microservices/audit-chain.json
  - /specs/microservices/observability.json
  - /specs/whistleblower-submission-channel.json
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
keystone_position: critical-path-doctrine-cluster-rows-6-7-16-21-whistleblower-press-anonymity
purpose: >
  Codify the Whistleblower + Press-Freedom + Anonymity doctrine —
  a per-tenant SecureDrop-class anonymous-submission surface,
  Tor-friendly ingress, metadata-minimization mode, pseudonymity-
  class principal scope, chain-of-custody to ombudsman, and per-
  jurisdiction reporter-privilege overlay that closes rows 6, 7,
  16, and 21 of the 30-row critical-path matrix in documentation-
  rigor.md §3.2.5. The bar is: a whistleblower / journalist
  source / activist in authoritarian jurisdiction / pseudonymous
  legitimate user can reach the platform's submission surface
  WITHOUT linking their identity to their submission, WITHOUT
  exposing their IP / device fingerprint / behavioural pattern
  to the receiving tenant or to any oyatie operator beyond the
  per-pack ombudsman, and WITHOUT triggering any abuse-defence
  control that would deanonymize them. Anonymity is preserved
  end-to-end via sealed-sender envelope encryption (per Signal
  sealed-sender + Apple Private Relay + Tor onion-routing
  precedents); metadata is minimized via per-submission scope
  isolation; per-jurisdiction reporter-privilege (SOX 806, EU
  Whistleblower Directive 2019/1937, KR Anti-Corruption Act,
  US Dodd-Frank §922; First Amendment + Art-19 + KR Press Freedom)
  is honored per the active compliance pack overlay.
enforcement_status: advisory-until-2026-08-15-blocker-thereafter
enforced_by:
  - cloud-ci/Rust gate packet whistleblower-channel-sealed-sender-envelope
  - cloud-ci/Rust gate packet whistleblower-channel-tor-friendly-ingress
  - cloud-ci/Rust gate packet whistleblower-channel-metadata-minimization
  - cloud-ci/Rust gate packet whistleblower-channel-pseudonymity-scope
  - cloud-ci/Rust gate packet whistleblower-channel-cedar-fragment-present
  - cloud-ci/Rust gate packet whistleblower-channel-audit-event-class
  - cloud-ci/Rust gate packet whistleblower-channel-per-pack-reporter-privilege
naming_justifications:
  - name: oya-shared-whistleblower-channel
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.whistleblower-channel
    justification: >
      Per ADR-0105 13-layer canonical enum row 5 (shared-substrate),
      the crate that exposes the sealed-sender envelope encryption,
      the Tor-friendly ingress adapter, the metadata-minimization
      filter, the pseudonymity-class principal resolver, and the
      ombudsman chain-of-custody router belongs at the shared
      layer. Naming `oya-shared-whistleblower-channel` keeps the
      single-concern flat layout per ADR-0131 + ADR-0132. Drop-in
      companion to `oya-shared-abuse-defence` (ADR-0297),
      `oya-shared-emergency-services-bypass` (ADR-0298), and
      `oya-shared-account-recovery` (ADR-0299).
  - name: oya-shared-anonymity-substrate
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.anonymity-substrate
    justification: >
      Per ADR-0105 13-layer canonical enum row 5; the crate that
      exposes the pseudonymity-class principal resolver, sealed-
      sender envelope cryptography, metadata-minimization filter,
      and Tor-friendly ingress adapter, factored separately from
      `oya-shared-whistleblower-channel` so the anonymity primitive
      can serve other surfaces (e.g., per-tenant pseudonymous
      community accounts per row 21, per-tenant minor-safety self-
      report path per ADR-0292 + §3.2.5 row 9) without dragging
      the whistleblower-specific submission protocol.
  - name: oya-governance-whistleblower-channel
    layer: N/A (foundry-fitness aggregate CI lane)
    bnf_segments: oya.foundry-fitness.whistleblower-channel
    justification: >
      Aggregate CI fitness lane per ADR-0212 buildability doctrine;
      rolls up per-µservice declaration of the sealed-sender
      envelope, the Tor-friendly ingress, the metadata-minimization
      filter, the pseudonymity-class scope, the chain-of-custody
      to ombudsman, and the per-pack reporter-privilege overlay.
  - name: oya-governance-anonymity-substrate
    layer: N/A (foundry-fitness aggregate CI lane)
    bnf_segments: oya.foundry-fitness.anonymity-substrate
    justification: >
      Aggregate CI fitness lane per ADR-0212; verifies every
      pseudonymity-class surface wires the anonymity substrate
      and never leaks identifying metadata to non-pseudonymity-
      authorized observers.
  - name: oya-governance-whistleblower-channel-tor-friendly-ingress
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.whistleblower-channel-tor-friendly-ingress
    justification: >
      Per-µservice child lane verifying the whistleblower-channel
      surface accepts Tor onion-service v3 ingress + does NOT
      gate-block on exit-node IP + emits no IP-correlation signal
      to abuse-defence.
  - name: oya-governance-whistleblower-channel-metadata-minimization
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.whistleblower-channel-metadata-minimization
    justification: >
      Per-µservice child lane verifying the metadata-minimization
      filter strips IP, user-agent, accept-language, accept-
      encoding, TLS JA4/JA4+ fingerprint, HTTP/2-3 frame pattern,
      and behavioural-fingerprint signals from the submission
      payload + audit-event.
  - name: oya-governance-whistleblower-channel-pseudonymity-scope
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.whistleblower-channel-pseudonymity-scope
    justification: >
      Per-µservice child lane verifying the pseudonymity-class
      principal scope is enforced (the receiving tenant cannot
      enumerate submitters across submissions; the audit trail
      is sealed except to the per-pack ombudsman).
  - name: X-Oya-Anonymity-Class
    layer: N/A (HTTP request header naming)
    bnf_segments: X-Oya.Anonymity-Class
    justification: >
      Custom HTTP request header carrying the anonymity-class
      identifier (enum: NORMAL, PSEUDONYM, ANONYMOUS_SEALED_
      SENDER, TOR_ORIGIN, JOURNALIST_SOURCE, WHISTLEBLOWER).
      Namespace prefix `X-Oya-` reserves the platform's header
      surface and avoids collision with Tor's `X-Tor-Exit-Node`
      or Signal's `X-Sealed-Sender`.
  - name: X-Oya-Submission-Channel
    layer: N/A (HTTP request header naming)
    bnf_segments: X-Oya.Submission-Channel
    justification: >
      Custom HTTP request header identifying the submission
      channel (enum: WHISTLEBLOWER, JOURNALIST_SOURCE,
      PSEUDONYM_USER, ACTIVIST_DISSIDENT, ANONYMOUS_FEEDBACK);
      paired with X-Oya-Anonymity-Class.
  - name: WhistleblowerSubmissionReceived
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Whistleblower.SubmissionReceived
    justification: >
      Audit-event-class emitted when a whistleblower submission
      is received and sealed in the chain-of-custody. The event
      contains only the per-tenant + per-pack scope + sealed-
      hash; NO submitter-identifying fields. Registered in
      ADR-0263 central registry per §3.2.2 consistency invariant.
  - name: WhistleblowerSubmissionRoutedToOmbudsman
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Whistleblower.SubmissionRoutedToOmbudsman
    justification: >
      Audit-event-class emitted when the submission is routed to
      the per-pack ombudsman. NO submitter-identifying fields;
      only the per-pack scope + the ombudsman case-id.
  - name: PseudonymitySessionEstablished
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Pseudonymity.SessionEstablished
    justification: >
      Audit-event-class emitted on establishment of a pseudonymity-
      class session. Distinct from AccountRecoveryGranted (ADR-
      0299) since pseudonymity is a per-session identity rather
      than a long-term account.
  - name: PseudonymityIdentityIsolationViolationDetected
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Pseudonymity.IdentityIsolationViolationDetected
    justification: >
      Audit-event-class emitted when a cross-submission identity
      correlation is detected (e.g., a tenant attempting to enumerate
      pseudonymous submitters via metadata cross-reference).
      Triggers per-pack regulator notification per §D-9.
  - name: JournalistSourceAccessGranted
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: JournalistSource.AccessGranted
    justification: >
      Audit-event-class emitted when a journalist's source enters
      the platform's source-protection surface under the per-tenant
      `publisher-source-protection` pack overlay.
  - name: ReporterPrivilegeAsserted
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: ReporterPrivilege.Asserted
    justification: >
      Audit-event-class emitted when the per-jurisdiction reporter-
      privilege (US shield laws, EU Whistleblower Directive
      2019/1937, KR Press Freedom Act) is asserted by the platform
      against a discovery / disclosure request.
  - name: policy/whistleblower-channel.cedar
    layer: N/A (per-µservice Cedar fragment file)
    bnf_segments: policy.whistleblower-channel
    justification: >
      Canonical filename for the per-µservice whistleblower-channel
      Cedar fragment under the µservice's `policy/` directory per
      ADR-0246 + ADR-0243 fragment-lifecycle conventions; single-
      concern naming keeps the policy directory's contract-by-name
      invariant.
  - name: policy/anonymity-substrate.cedar
    layer: N/A (per-µservice Cedar fragment file)
    bnf_segments: policy.anonymity-substrate
    justification: >
      Canonical filename for the per-µservice anonymity-substrate
      Cedar fragment; companion to whistleblower-channel.cedar,
      separately versioned so the anonymity primitive promotes
      independently of the whistleblower-specific submission
      protocol.
  - name: spec/whistleblower-submission-channel.json
    layer: N/A (canonical submission-channel spec)
    bnf_segments: specs.whistleblower-submission-channel
    justification: >
      Canonical machine-readable spec for the whistleblower
      submission channel; declares submission shape, sealed-sender
      envelope cryptography, metadata-minimization filter,
      pseudonymity-class scope, chain-of-custody to ombudsman.
  - name: oyatie.pseudonym.session.<hash>
    layer: N/A (SPIFFE-URI principal namespace per ADR-0295)
    bnf_segments: oyatie.pseudonym.session.<hash>
    justification: >
      Canonical SPIFFE-URI principal namespace for pseudonymity-
      class session principals. The `<hash>` is a per-session
      random 256-bit identifier; the principal is not linkable
      across sessions even within the same physical user.
  - name: oyatie.whistleblower.submission.<sealed-hash>
    layer: N/A (SPIFFE-URI principal namespace per ADR-0295)
    bnf_segments: oyatie.whistleblower.submission.<sealed-hash>
    justification: >
      Canonical SPIFFE-URI principal namespace for whistleblower
      submission identifiers. The `<sealed-hash>` is the per-
      submission sealed-sender envelope hash; per-pack ombudsman
      can correlate within a per-tenant scope but cross-tenant
      correlation is cryptographically prevented.
  - name: HIGH_RISK_USER
    layer: N/A (Tenant.audience_type enum value per ADR-0244)
    bnf_segments: audience_type.HIGH_RISK_USER
    justification: >
      Tenant.audience_type enum extension per ADR-0244 §D-3;
      identifies tenants serving HIGH_RISK_USER audience-types
      (journalist, activist, security researcher, dissident);
      enables stricter anonymity guarantees on the per-tenant
      surface.
  - name: publisher-source-protection
    layer: N/A (per-pack overlay slug per ADR-0251)
    bnf_segments: publisher-source-protection
    justification: >
      Per-tenant pack overlay slug per ADR-0251; enables the
      SecureDrop-class submission surface for publisher tenants
      (newspapers, investigative-journalism non-profits, academic
      research with confidential informants).
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0300: Whistleblower + Press-Freedom + Anonymity Doctrine

## Status

Proposed — 2026-05-20.

Bundled with the keystone-bundle 2026-05-20 foundational doctrine
synthesis as the **critical-path-doctrine-cluster-rows-6-7-16-21-
whistleblower-press-anonymity** keystone. Closes rows 6, 7, 16,
and 21 of the 30-row critical-path matrix in `docs/standards/
documentation-rigor.md` §3.2.5.

Enforcement is `advisory-until-2026-08-15-blocker-thereafter`. The
doctrine accepts in text immediately; the seven CI lanes that
enforce it promote to BLOCKER on 2026-08-15 to give per-pack
reporter-privilege overlay onboarding (US shield-law statutes per
state, EU Whistleblower Directive 2019/1937 transposition per
member state, KR Anti-Corruption Act, JP Whistleblower Protection
Act 2004, UK Public Interest Disclosure Act 1998, AU Public
Interest Disclosure Act 2013), per-tenant SecureDrop-class
surface provisioning, and per-µservice metadata-minimization
filter wiring time to land. Until 2026-08-15, validators emit
findings without failing CI; post-2026-08-15 the lanes block
merge.

## Date

2026-05-20.

## Context

### §A. Why anonymity is a substrate primitive, not a per-tenant afterthought

Anonymity-preserving submission surfaces are a critical-path edge
case (per documentation-rigor.md §3.2.5 rows 6, 7, 16, 21) because
the standard auth-defence + abuse-defence pattern — which depends
on linking every request to a principal, observing behavioural
fingerprints, rate-limiting per-IP — is fundamentally hostile to
the four critical-path classes:

- **Row 6 (Whistleblower + ethics report).** SOX 806, EU
  Whistleblower Directive 2019/1937, KR Anti-Corruption Act, US
  Dodd-Frank §922 each mandate anonymous-submission channels
  where the receiving organization CANNOT learn the submitter's
  identity. A platform that links the submission to the caller's
  identity violates the per-pack regulator's protection requirement.
- **Row 7 (Press freedom / journalist source).** First Amendment
  shield laws (varying by US state; ~40 states have shield laws),
  Article 19 of the Universal Declaration of Human Rights, KR
  Press Freedom Act, EU Charter of Fundamental Rights Art. 11
  each protect journalist sources from forced disclosure. A
  platform that retains IP-correlation or metadata that could
  reveal the source under court order violates the per-jurisdiction
  reporter-privilege.
- **Row 16 (Activist / dissident in authoritarian jurisdiction).**
  Users in countries with state-level surveillance (e.g., the
  Reporters Without Borders Press Freedom Index lowest-tier)
  face direct safety risk from any platform-side metadata that
  could be subpoenaed by their government. Tor-friendly ingress
  + metadata-minimization is the safety floor.
- **Row 21 (Pseudonymous + privacy-by-default users).** Users
  who legitimately need pseudonymity (LGBTQ+ rights activists,
  hate-target groups, abuse-survivors per ADR-0301, security
  researchers, healthcare-stigma sufferers) face direct safety
  risk from any platform-side identity-correlation that could
  link their pseudonym to their legal identity.

The pattern across mature anonymity-preserving substrates is
unambiguous:

- **SecureDrop (Freedom of the Press Foundation).** SecureDrop
  (open source since 2013, deployed by ~70 news organizations
  including The New York Times, The Washington Post, The Guardian,
  Forbes, BBC, Le Monde, Süddeutsche Zeitung, ProPublica per the
  Freedom of the Press Foundation public registry) is the
  industry-canonical anonymous-submission surface for journalist
  sources. Per the SecureDrop architecture (`docs.securedrop.org`),
  the surface: (i) accepts only Tor onion-service v3 ingress, (ii)
  generates a per-source codename rather than an account, (iii)
  encrypts every submission with the publisher's PGP key at the
  ingress edge, (iv) strips all metadata from documents (Office
  documents, PDFs, images), (v) routes the submission through an
  air-gapped journalist workstation. The platform-side never sees
  the source's identity.
- **Signal Sealed Sender.** Signal's sealed-sender envelope
  encryption (published 2018 per `signal.org/blog/sealed-sender/`)
  hides the sender's identity from Signal's own server. Per the
  protocol, the sender's identity is encrypted with the recipient's
  long-term key + a per-message ephemeral key; only the recipient
  can learn the sender. The server forwards messages without
  knowing who they are from. This is the canonical sealed-sender
  pattern oyatie inherits.
- **Apple Private Relay.** Apple's Private Relay (launched 2021
  in iCloud+ plans per `support.apple.com/en-us/HT212614`) is
  a two-hop relay where the first hop (Apple) sees the IP but not
  the destination + the second hop (a partner CDN like Cloudflare
  or Akamai) sees the destination but not the IP. Per Apple's
  published architecture, no single party can correlate IP-to-
  destination. Private Relay is the canonical two-hop anonymity
  pattern.
- **Tor + onion services v3.** The Tor Project's onion-service
  v3 protocol (rolled out in Tor 0.3.2+ per `2017-12-12 Tor blog`)
  provides hidden-service ingress where the client connects via
  3-hop relay path + the server is reachable only via the onion
  address (256-bit identifier). Per the Tor design specification,
  the server never learns the client's IP + the client never
  learns the server's IP. Tor onion-service v3 is the canonical
  anonymity-preserving ingress oyatie supports.
- **Cloudflare Onion Routing for Tor.** Cloudflare's per-domain
  onion-service support (rolled out 2018 per `blog.cloudflare.
  com/cloudflare-onion-service/`) provides automatic Tor onion-
  service publication for any Cloudflare-protected zone. This is
  the canonical pattern for adding Tor-friendly ingress to a
  conventional internet-facing surface — the platform serves both
  clearnet + onion paths.
- **Mullvad VPN no-log + multihop.** Mullvad (operating since
  2009; ~750k subscribers per 2024 published figures) ships
  account-anonymous payment + no-account-tied-IP-log + multihop
  routing as the canonical no-log VPN substrate. Per Mullvad's
  published infrastructure-audit reports (annual since 2019),
  the substrate retains no per-user identifying data.
- **ProtonMail anonymous-signup.** ProtonMail (operated by Proton
  AG, ~110M users per 2025 published figures) ships Tor-friendly
  signup + anonymous-payment options + per-account end-to-end
  encryption + the Swiss-jurisdiction reporter-privilege overlay.
- **OnionShare anonymous file transfer.** OnionShare (open source
  since 2014, integrated into Tails OS) ships per-recipient
  ephemeral onion-services for anonymous file-and-message
  transfer. Per the OnionShare architecture, the recipient
  receives only the onion-address + an optional decryption
  passphrase.

The corollary: **every internet-facing surface oyatie ships that
serves a HIGH_RISK_USER audience-type or a publisher-source-
protection pack overlay MUST inherit anonymity from the
substrate, not author it per-µservice.** A µservice that authors
its own anonymous-submission flow, its own metadata-minimization
filter, its own pseudonymity scope is duplicating substrate
primitives that the shared crate already serves. That duplication
is a `feedback_no_silent_regression` violation (every µservice's
anonymity drifts independently); it is a
`feedback_quality_performance_scalability_bar` violation (per-
µservice flows cannot share the cross-µservice metadata-leakage
detection); and it is a `feedback_autonomous_implementation_
artifacts` violation (intern-buildable means the doc surface is
one substrate, not 46 µservice-private implementations).

The ADR-0300 whistleblower-press-anonymity doctrine closes this
gap.

### §A.1. The four critical-path rows from §3.2.5

This ADR addresses four paired rows of the 30-row critical-path
matrix in documentation-rigor.md §3.2.5:

**Row 6 — Whistleblower + ethics report.** "SOX 806, EU
Whistleblower Directive (2019/1937), KR Anti-Corruption Act, US
Dodd-Frank §922." Per the standard's mandate, the special handling
MUST include: anonymous-submission surface; never tied to caller
identity by µservice; per-pack overlay for jurisdiction; chain-of-
custody to ombudsman. The safety/security/policy invariant:
anonymity preserved E2E; tenant admin CANNOT see submitter;
metadata-minimized + sealed-sender.

**Row 7 — Press freedom / journalist source.** "First-Amendment /
Article-19 / KR-Press-Freedom protections." Per the standard's
mandate, the special handling MUST include: per-tenant
SecureDrop-class option; Tor-friendly ingress; no IP-log + no
IP-correlation; per-pack overlay (e.g., publisher tenant + tenant-
pack `publisher-source-protection`). The safety/security/policy
invariant: no metadata retained beyond minimum-required; per-
jurisdiction reporter-privilege honored.

**Row 16 — Activist / dissident in authoritarian jurisdiction.**
"High-risk users in countries with state-level surveillance." Per
the standard's mandate, the special handling MUST include: Tor-
friendly ingress; metadata-minimization mode; per-tenant `audience_
type = HIGH_RISK_USER` overlay; no cross-border data export unless
tenant-opted. The safety/security/policy invariant: E2EE preserved;
metadata minimal; per-pack `pack-cn-pipl` + per-tenant override
permitted within regulator floor.

**Row 21 — Pseudonymous + privacy-by-default users.** "Users who
legitimately need pseudonymity (gay rights activists, hate-target
groups, victims)." Per the standard's mandate, the special handling
MUST include: per-tenant pseudonymity-preservation; KYB tier
separated from public identity; cross-reference protection. The
safety/security/policy invariant: pseudonymity-class principal
scope; audit-trail accessible only to authorized tier; per-
jurisdiction legal-name-required compliance.

The four rows compose: a whistleblower (row 6) IS a pseudonymous
user (row 21) WHO IS POTENTIALLY in an authoritarian jurisdiction
(row 16) WHO might be a journalist source (row 7). The doctrine
treats them as four orthogonal but composable scopes of the same
anonymity substrate.

### §A.2. Why the keystone bundle 2026-05-20 requires this as a substrate-level primitive

The keystone bundle's foundational ADRs intersect anonymity as
follows:

- **ADR-0242 (oyatie-is-a-tenant).** Pseudonymity-class principals
  occupy the `oyatie.pseudonym.session.*` namespace under the
  platform's tenant tree. No carve-out from the tenant model — the
  pseudonymity scope is a per-tenant policy expressed as a Cedar
  fragment.
- **ADR-0243 (Cedar universal gate).** Anonymity is a Cedar
  policy decision. The pseudonymity-class scope is enforced via
  Cedar's principal type + resource type + action type composition;
  the policy FORBIDS cross-pseudonym correlation.
- **ADR-0244 (tenant scoping primitive).** This ADR adds the
  HIGH_RISK_USER `audience_type` enum value per ADR-0244 §D-3 +
  the `publisher-source-protection` per-tenant pack overlay slug.
- **ADR-0246 + amendment (policy-engine library-first).** Every
  µservice's library-first Cedar evaluator carries the anonymity-
  substrate Cedar fragment.
- **ADR-0247 (self-modification / break-glass).** Per-pack
  ombudsman chain-of-custody inherits the ADR-0247 break-glass
  pattern (post-hoc audit-and-justify + cryptographically sealed
  per ADR-0028).
- **ADR-0248 (Amazon-shape cellular architecture).** Per-tenant
  pseudonymity state is partitioned per-cell; cross-cell
  correlation is forbidden by Cedar policy.
- **ADR-0251 (compliance packs).** Each pack adds per-pack
  reporter-privilege overlay (pack-sox-806-us, pack-eu-whistleblower-
  directive-2019-1937, pack-kr-anti-corruption-act, pack-us-dodd-
  frank-922, pack-us-shield-law-<state>, pack-jp-whistleblower-
  protection-act-2004, pack-uk-pida-1998, pack-au-pida-2013).
- **ADR-0252 (HLC + TrueTime).** Submission timestamps use HLC
  for causality without revealing precise wall-clock (per ADR-
  0252 HLC default mode); precise wall-clock is stripped from
  metadata-minimization filter.
- **ADR-0253 (HTTP/3 + QUIC default + ECH + PQC).** ECH (Encrypted
  Client Hello) is critical here — without ECH, the SNI reveals
  the destination domain to passive observers; with ECH, the
  inner SNI is encrypted. PQC hybrid KEX defends against harvest-
  now-decrypt-later attackers (state-level surveillance, per row
  16). Tor onion-service v3 ingress runs over QUIC + TLS 1.3
  with ECH + PQC where available.
- **ADR-0263 (observability emission contract).** Anonymity-class
  events emit dedicated audit-event classes with NO submitter-
  identifying fields. The six new classes per the naming-
  justifications.
- **ADR-0272 (cookie consent per-purpose).** Anonymity-class
  sessions DO NOT set persistent cookies; per-session ephemeral
  state only.
- **ADR-0273 (per-tenant DKIM/SPF/DMARC).** Per-tenant DKIM-
  signed acknowledgement email is sent ONLY if the submitter
  opts-in to receive acknowledgement; the email content is
  metadata-minimized (no per-submission identifiers).
- **ADR-0276 (backup portability GDPR Art. 20).** Anonymity-class
  submissions are NOT subject to GDPR Art. 20 backup-portability
  by the receiving tenant — the data is by definition not the
  tenant's data subject's data; portability is the submitter's
  prerogative (and they hold the per-session sealed-sender key).
- **ADR-0280 (substrate-of-substrate).** The anonymity substrate
  depends on `oya-shared-cedar-evaluator` + `oya-shared-audit-emit`
  + `oya-shared-emergency-services-bypass` (for emergency
  escalation paths from within an anonymous submission).
- **ADR-0284 (platform-owner name indirection).** Namespace
  `oyatie.pseudonym.*` and `oyatie.whistleblower.*` parameterized
  via the platform-owner indirection.
- **ADR-0292 (minor user doctrine).** A minor whistleblower self-
  reporting safety concern (per §3.2.5 row 9) uses the anonymity
  substrate; parental control surface cannot suppress.
- **ADR-0293 (meta-trust-root).** Per-session sealed-sender keys
  rooted at the meta-trust-root.
- **ADR-0294 (Cedar fragment soak).** ≥60s soak window respected.
- **ADR-0295 (bootstrap CI SPIFFE + kill-switch).** SPIFFE
  identity for the anonymity-substrate µservice + per-cell kill-
  switch.
- **ADR-0296 (library-first credential sidecar).** Per-session
  sealed-sender keys held in sidecar with ≤60s OpenBao TTL.
- **ADR-0297 (abuse-defence baseline).** The abuse-defence layer
  is OBSERVATION-ONLY on anonymity-class traffic. NO behavioural-
  fingerprint forwarding; NO IP-correlation across sessions; NO
  CAPTCHA on submission path (CAPTCHA reveals the submitter to
  the CAPTCHA provider).
- **ADR-0298 (emergency-services bypass).** Emergency-services
  takes precedence: a whistleblower who reveals an imminent threat
  to life-safety can transition mid-submission to the emergency-
  services bypass.
- **ADR-0299 (account-recovery resilience).** Pseudonymity-class
  sessions are not subject to account-recovery (they are per-
  session ephemeral); the per-session sealed-sender key is the
  recovery primitive (held by the submitter).

### §A.3. The failure modes the anonymity substrate must defend against

Per documentation-rigor.md §1.1 rigor sub-test "failure-mode tree":

**FM-1: Receiving tenant attempts to enumerate submitters across
submissions.** Mitigation: per-submission pseudonymity-class
principal scope; cross-submission correlation FORBIDDEN by Cedar
fragment §D-4. Detection: `PseudonymityIdentityIsolationViolation
Detected` event fires on attempted cross-correlation.

**FM-2: State-level adversary subpoenas oyatie for the submitter's
identity.** Mitigation: oyatie operationally CANNOT produce the
identity because (i) the sealed-sender envelope is encrypted with
the recipient's key (Signal protocol per §D-1), (ii) the metadata-
minimization filter strips IP / fingerprint / behavioural pattern
before the audit-event is written, (iii) the per-pack reporter-
privilege overlay enables the platform's legal team to assert
reporter-privilege per the per-jurisdiction statute.

**FM-3: Abuse-defence behavioural-fingerprint forwarding leaks
identity across anonymity-class sessions.** Mitigation: abuse-
defence layer is observation-only on anonymity-class traffic per
§D-7. JA4/JA4+ TLS fingerprints + HTTP/2-3 frame patterns are
hashed-with-pepper before forwarding so cross-session correlation
is cryptographically prevented.

**FM-4: Tor exit-node IP is gate-blocked by abuse-defence.**
Mitigation: per the §D-6 Tor-friendly ingress invariant, Tor exit
nodes are NEVER gate-blocked on anonymity-class surfaces. Bot-
score forwarding is suppressed on anonymity-class traffic.

**FM-5: ECH downgrade to non-ECH leaks SNI.** Mitigation: per
ADR-0253 §D-10 PQC + ECH advertisement; anonymity-class surfaces
publish HTTPS RR with ECH config; clients without ECH support
fall through to non-ECH but the surface logs the fallback for
operator-side review (does NOT log the SNI itself).

**FM-6: Per-session sealed-sender key is compromised.**
Mitigation: per-session keys are ephemeral (≤24h TTL); a
compromised key reveals only the one session. Per-pack ombudsman
rotation cadence ≤30 days.

**FM-7: Receiving tenant ombudsman becomes the adversary
(insider risk).** Mitigation: per-pack 2-member quorum per
ADR-0247 break-glass; per-action MFA; UEBA monitoring on ombudsman
behaviour; cross-tenant escalation path if the per-tenant
ombudsman is suspected compromised.

**FM-8: Receiving tenant correlates the submission with public
contextual data (e.g., the submission references a unique event;
the tenant correlates the timing).** Mitigation: timing
minimization via HLC + per-session jitter; per-tenant operational
controls (e.g., the publisher tenant's ombudsman trains
journalists in source-protection); per-jurisdiction reporter-
privilege as the legal-layer defence.

**FM-9: Tenant attempts to use anonymous submissions as an
exfiltration channel.** Mitigation: per-pack policy authorizes
the anonymous-submission surface ONLY where the tenant has a
legitimate use case (publisher tenant + `publisher-source-
protection` pack overlay; corporate tenant + `pack-sox-806-us`
or `pack-eu-whistleblower-directive-2019-1937`); per-pack
ombudsman cross-tenant audit.

**FM-10: Submitter inadvertently de-anonymizes themselves via
content (e.g., embedded EXIF metadata in image, or naming a
unique role they hold).** Mitigation: metadata-minimization
filter strips embedded metadata from documents; per-submission
"sanitize before send" UX advisory; per-pack ombudsman trained
to redact submitter-identifying content before sharing with the
tenant.

## Decision

### §B. Decision summary

**Decision 1: Per-tenant anonymous-submission surface via sealed-
sender envelope.** Every tenant with the `publisher-source-
protection` pack overlay OR a per-pack reporter-privilege overlay
gets a per-tenant SecureDrop-class submission surface. The surface:
(i) accepts only Tor onion-service v3 ingress OR clearnet ingress
with metadata-minimization + ECH-enabled TLS, (ii) generates a
per-session pseudonymity-class principal per the `oyatie.
pseudonym.session.<hash>` namespace, (iii) encrypts every
submission with the per-tenant recipient PGP key + per-session
ephemeral key (Signal sealed-sender protocol per §D-1), (iv)
strips all metadata from documents (EXIF, Office document
properties, PDF metadata, image steganography hashes), (v) routes
the submission through the per-pack ombudsman with a per-tenant
chain-of-custody seal.

**Decision 2: Tor-friendly ingress + metadata-minimization mode.**
Every internet-facing surface that serves a HIGH_RISK_USER
audience-type publishes a per-zone Tor onion-service v3 address
(via the Cloudflare onion-routing pattern). On anonymity-class
traffic, the metadata-minimization filter strips: IP (the per-
session principal has no IP attribute); user-agent; accept-
language; accept-encoding; TLS JA4/JA4+ fingerprint (hashed-with-
pepper before forwarding); HTTP/2-3 frame pattern; behavioural-
fingerprint signals.

**Decision 3: Pseudonymity-class principal scope.** A per-session
pseudonymity-class principal is generated per `oyatie.pseudonym.
session.<hash>` where `<hash>` is a 256-bit random identifier.
The principal is not linkable across sessions even within the
same physical user. Cedar fragment §D-4 enforces this scope.

**Decision 4: Chain-of-custody to ombudsman.** Every submission
emits `WhistleblowerSubmissionReceived` (per ADR-0263) with NO
submitter-identifying fields. The submission is routed through
the per-pack ombudsman; the ombudsman case-id is emitted via
`WhistleblowerSubmissionRoutedToOmbudsman`. Audit chain Merkle-
sealed per ADR-0028.

**Decision 5: Per-pack reporter-privilege overlay.** Each
applicable pack adds an overlay under `packs/<pack-slug>/policy/
whistleblower-channel-overlay.cedar` that declares: per-pack
reporter-privilege assertion, per-pack mandatory regulator
escalation, per-pack ombudsman SLA, per-pack jurisdictional
override (e.g., KR Anti-Corruption Act mandates Real-Name-
Verification ONLY if the submitter opts to receive
acknowledgement; SOX 806 mandates ≤30-day acknowledgement to
submitter via per-tenant verified channel).

**Decision 6: Abuse-defence is observation-only on anonymity-
class traffic.** Per the §3.2.5 row 7 anti-pattern call-out:
"Whistleblower submissions go through normal auth → No.
Submission must be anonymous; binding to caller identity is the
safety violation." Per the §D-7 below, the abuse-defence layer
suppresses bot-score forwarding + IP-correlation + CAPTCHA on
anonymity-class traffic.

**Decision 7: Per-jurisdiction reporter-privilege legal-layer
assertion.** When the platform receives a discovery / disclosure
request (subpoena, FOIA, GDPR Art. 15 DSAR from a third party,
court order) for an anonymity-class submission, the platform's
legal team asserts the per-jurisdiction reporter-privilege per
the active pack overlay. The `ReporterPrivilegeAsserted` event
fires; per-pack regulator-notify workflow runs per ADR-0251.

## Consequences

### §C. Consequences across all 6 engineering-rigor dimensions

Per documentation-rigor.md §1.2 engineering-rigor dimensions
matrix:

#### §C.1. Maintainability

- **Module boundaries.** Anonymity primitives in `oya-shared-
  anonymity-substrate` (single concern). Whistleblower-specific
  submission protocol in `oya-shared-whistleblower-channel`
  (single concern). The two crates are independently versioned
  so per-tenant pseudonymity (row 21) does not require the full
  whistleblower-channel.
- **Versioning policy.** Per ADR-0258 SemVer. Cedar fragments
  per-fragment SemVer; shared crates Cargo SemVer; per-pack
  overlay SemVer.
- **Deprecation cadence.** Tor onion-service v3 is the current
  generation; the prior v2 protocol is retired (per Tor Project
  decision 2021). Future Tor protocol revisions handled via
  ≥6-month sunset per ADR-0258.
- **Reverse-dependencies enumerated.** Every µservice consuming
  anonymity declared in `manifest.json:reverse_consumers_of_
  anonymity_substrate`.
- **What is hard-coded vs configurable.** Hard-coded: audit-
  event-class slugs, the FORBID cross-pseudonym correlation
  policy, the metadata-minimization filter roster, the no-IP-
  attribute on pseudonymity principals invariant. Configurable:
  per-tenant Tor onion-address, per-pack reporter-privilege
  overlay roster, per-pack ombudsman SLA.

#### §C.2. Observability

- **Metrics.** Per ADR-0263 cardinality budget:
  - `whistleblower_submission_received_total` (counter,
    dimensions: tenant_pack, ombudsman_pack, channel_class,
    cardinality ≤3k series — NO submitter-identifying
    dimensions).
  - `whistleblower_submission_routed_to_ombudsman_total`
    (counter, dimensions: tenant_pack, ombudsman_pack,
    cardinality ≤2k series).
  - `pseudonymity_session_established_total` (counter,
    dimensions: tenant_pack, audience_type, cardinality ≤2k
    series).
  - `pseudonymity_identity_isolation_violation_detected_total`
    (counter, dimensions: violation_class, tenant_pack,
    cardinality ≤500 series — fires on attempted cross-correlation
    by tenant).
  - `journalist_source_access_granted_total` (counter,
    dimensions: publisher_tenant_pack, cardinality ≤500 series).
  - `reporter_privilege_asserted_total` (counter, dimensions:
    per_pack_jurisdiction, assertion_class (subpoena, foia,
    dsar, court_order), cardinality ≤500 series).
  - `whistleblower_submission_latency_seconds` (histogram,
    dimensions: tenant_pack, buckets `1s 5s 30s 5min 30min 24h
    72h`, cardinality ≤500 series).
- **Trace span shape.** Every submission carries `whistleblower_
  channel.submission` parent span with child spans `whistleblower_
  channel.sealed_envelope_encrypt`, `whistleblower_channel.metadata_
  strip`, `whistleblower_channel.ombudsman_route`. NO submitter-
  identifying fields in span attributes.
- **Logs.** JSON-structured log lines at INFO level — NO submitter-
  identifying fields. Retention: per-pack (e.g., SOX 806 mandates
  ≥7 years retention; EU Whistleblower Directive 2019/1937 leaves
  retention to member-state transposition; GDPR data-minimization
  pushes toward minimal retention).
- **Audit events.** Per ADR-0263 — six new classes from the
  naming-justifications. Audit chain Merkle-sealed per ADR-0028;
  per-pack ombudsman holds the decryption key; non-ombudsman
  operators see only the sealed-hash.
- **SLO floor.** P95 submission-to-ombudsman-routing ≤30 seconds;
  P99 ≤5 minutes. Per-pack ombudsman SLA varies (SOX 806: ≤30
  days; EU Whistleblower Directive: ≤7 days; KR Anti-Corruption
  Act: ≤60 days).
- **Dashboards.** `dashboards/whistleblower-channel.json` per
  µservice + `dashboards/anonymity-substrate.json` substrate-
  level.

#### §C.3. Scalability

- **Capacity math.** Baseline whistleblower-channel traffic:
  ~100 submissions/sec sustained across all packs (derived:
  ~70 publisher tenants on SecureDrop-class average ~10 sub/day
  + ~1000 corporate tenants on SOX 806 / EU Whistleblower
  Directive average ~5 sub/day each = ~5500 sub/day = ~0.06/sec;
  scale up for the 100× public-investigation surge factor and
  the per-cell shard width).
- **Mass-event surge.** A whistleblower-led public investigation
  (e.g., a Snowden-class disclosure) drives 100×-1000× the
  baseline for ≤30 days. The cell-tier-0 edge MUST absorb without
  dropping submissions.
- **Bottleneck identification.** The single bottleneck is the
  per-pack ombudsman SLA (human review). Per-tenant ombudsman
  queue sized for ≤7-day SLA on EU Whistleblower Directive packs.
- **Horizontal scale-out path.** Per-tenant pseudonymity state
  is partitioned per-cell; submission throughput scales
  horizontally with cell-tier-0 + cell-tier-1 + cell-tier-2.

#### §C.4. Performance

- **P50/P95/P99 latency.** P50 submission-to-ombudsman-routing
  ≤5s; P95 ≤30s; P99 ≤5min. P50 sealed-envelope-encrypt ≤50ms;
  P95 ≤200ms; P99 ≤500ms.
- **Modeling note.** Latency dominated by sealed-envelope-encrypt
  (Ed25519 + XChaCha20-Poly1305 per Signal sealed-sender) and
  per-pack ombudsman queue depth. Tor relay adds 3-hop latency
  (~600ms p99 per the Tor metrics).
- **Per-region budget split.** Per ADR-0240: US ≤30s P95; EU
  ≤30s P95; KR ≤30s P95; CN ≤60s P95 (slower due to per-pack
  PIPL handshake); other packs ≤30s P95.
- **Tail-latency mitigation.** Per-pack ombudsman queue auto-
  routes to secondary reviewer on SLA-risk.
- **Cold-vs-warm.** Warm: per-session sealed-sender key cached.
  Cold: per-session key generation (~10ms p99) + per-tenant
  recipient PGP key fetch from OpenBao (~30ms p99).

#### §C.5. Optimization

- **Per-call cost model.** CPU ≤10ms per sealed-envelope-encrypt
  (Ed25519 sign + XChaCha20-Poly1305 encrypt); RAM ≤2MB per
  submission (envelope buffer + per-document metadata-strip
  buffer); IOPS ≤1 per submission; $/M-submissions ≤$5.00.
- **Lazy vs eager.** Per-session key generation is eager (at
  session-start). Per-tenant recipient PGP key fetch is eager-
  with-pre-warm (active publisher tenants pre-warmed at cell-
  startup). Metadata strip is eager (every submission). Per-pack
  ombudsman queue write is async-with-flush (queue write within
  ≤1s p99 per ADR-0263).
- **Cache-invalidation policy.** Per-session sealed-sender keys
  ephemeral (≤24h TTL); per-tenant recipient PGP keys ≤24h TTL
  with rotation on per-pack regulator update.
- **Profiling evidence link.** `tools/profiling/whistleblower-
  channel-baseline.json`.

#### §C.6. Code quality

- **Required test classes.** Unit (Cedar fragment, sealed-sender
  envelope, metadata-strip filter), property (pseudonymity-class
  isolation invariants — random fuzzing of cross-session
  correlation attempts), fuzz (envelope-parser hardening,
  metadata-stripper hardening against malformed documents),
  load (mass-submission surge), e2e (full flow including Tor
  ingress against a synthetic publisher).
- **Coverage floor.** ≥90% line, ≥80% branch for both shared
  crates (above standard ≥85%/≥75% due to anonymity stakes).
- **Lint passes.** `oya-check-cedar-fragment-soak`,
  `oya-check-pseudonymity-isolation-invariant` (new lint for
  this ADR), `oya-check-metadata-leak-invariant` (new lint),
  `oya-check-spiffe-uri-conformance`, `oya-check-audit-event-
  class-registration`, `oya-check-naming-justification-block`.
- **Type-strictness.** Rust `deny(warnings) + deny(missing_docs)
  + deny(unsafe_op_in_unsafe_fn)`; TypeScript surface (per-
  tenant whistleblower-channel admin UI) `strict + noUncheckedIndexed
  Access`.
- **SemVer + ABI policy.** Per ADR-0258. The shared crates'
  `SealedSenderEnvelope`, `PseudonymitySession`, `MetadataStripFilter`
  traits are public-stable.

## Detailed mechanics

### §D. Detailed mechanics

#### §D-1. Sealed-sender envelope protocol — Signal-class

Per Signal's sealed-sender protocol (`signal.org/blog/sealed-sender/`):

1. **Per-tenant recipient long-term key.** Each tenant with the
   `publisher-source-protection` pack overlay registers a per-tenant
   Ed25519 long-term public key + an X25519 long-term public key
   in the platform's per-tenant key registry. The per-tenant
   recipient's private key is held in the tenant's own per-tenant
   HSM (the platform NEVER holds the private key).
2. **Per-session ephemeral key.** At submission time, the platform
   generates a per-session ephemeral X25519 keypair.
3. **Sealed envelope.** The submission payload is encrypted under
   a per-session XChaCha20-Poly1305 symmetric key. The symmetric
   key is encrypted under the per-tenant recipient's long-term
   X25519 public key (X25519-DH between recipient long-term + per-
   session ephemeral). The submission timestamp is HLC-clocked
   (per ADR-0252) + per-session-jittered.
4. **Sealed sender identifier.** The per-session pseudonymity-
   class principal (`oyatie.pseudonym.session.<hash>` where
   `<hash>` is the SHA-256 of the per-session ephemeral public
   key) is included in the envelope; the platform sees this hash
   only.
5. **Per-session reply path (optional).** If the submitter opts
   to receive acknowledgement, a per-session reply-path public
   key is included in the envelope. The platform stores this
   public key + a per-session reply-channel URL. The submitter
   later polls the URL with the corresponding private key to
   retrieve the acknowledgement; the platform does NOT push
   notifications.

#### §D-2. Per-tenant Tor onion-service v3 publication

Per the Cloudflare onion-routing pattern (`blog.cloudflare.com/
cloudflare-onion-service/`):

1. **Per-zone onion-address.** Each tenant zone (e.g.,
   `submissions.<tenant-domain>.com`) is paired with a per-zone
   Tor onion-service v3 address (e.g., `<56-char-base32>.onion`).
2. **Per-zone HTTPS RR.** The zone publishes an HTTPS Resource
   Record with the onion-address in the `alpn` + `port` fields
   per the Tor onion-service-v3 specification.
3. **Per-zone alternative-service header.** Clearnet responses
   include the `Alt-Svc: h2=":443"; h3=":443"` + `Tor-Onion: <56-
   char-base32>.onion` headers so Tor-aware clients can transition
   to the onion-service path.
4. **Per-cell Tor relay infrastructure.** The platform operates
   per-cell Tor relays (3-hop per Tor protocol) + per-cell exit
   nodes for outbound flows. Per-cell relays are SPIFFE-attested
   per ADR-0295.

#### §D-3. Metadata-minimization filter

The metadata-minimization filter strips every identifying
attribute from the submission ingress envelope:

| # | Attribute | Action | Justification |
|---:|---|---|---|
| 1 | Client IP | Stripped — pseudonymity-class principal has no IP attribute | IP-correlation leaks identity per row 16 |
| 2 | User-Agent | Stripped | UA fingerprint leaks identity |
| 3 | Accept-Language | Stripped | Language preference correlates with jurisdiction |
| 4 | Accept-Encoding | Stripped | Encoding preference correlates with client software |
| 5 | TLS JA4/JA4+ fingerprint | Hashed-with-pepper (per-cell rotating pepper) | Forwarded for abuse-defence observation only |
| 6 | HTTP/2-3 frame pattern | Stripped | Frame pattern fingerprints client |
| 7 | TCP TTL + window size | Stripped at Tier-0 edge | OS fingerprint |
| 8 | Behavioural-fingerprint (cursor trajectory, scroll cadence, dwell time) | Stripped | Behavioural fingerprint identifies user |
| 9 | EXIF metadata (images) | Stripped | Camera serial number + location + timestamp leaks identity |
| 10 | Office document properties | Stripped | Author + last-modified-by + revision-history leaks identity |
| 11 | PDF metadata | Stripped | Producer + author + title leaks identity |
| 12 | Image steganography hash | Watermark-stripped via per-image entropy normalization | Per-pixel-bit-twiddling can encode identifier |
| 13 | Cookies + LocalStorage | Stripped (anonymity-class sessions never set persistent state) | Persistent state defeats per-session pseudonymity |
| 14 | TLS exporter binding | Stripped before forwarding | TLS exporter ties to per-connection identity |
| 15 | HLC timestamp precision | Quantized to per-session jitter (±30s) | Precise timestamp correlates across submissions |

The filter is `oya-shared-anonymity-substrate::MetadataStripFilter`
(per the public surface in §E.1).

#### §D-4. Cedar fragment — `policy/anonymity-substrate.cedar` + `policy/whistleblower-channel.cedar`

Two fragments, separately versioned:

**`policy/anonymity-substrate.cedar`** (per ADR-0299 §D-4 + this
ADR):

```cedar
// policy/anonymity-substrate.cedar
// Per ADR-0300 anonymity-substrate.
// Soak window: per ADR-0294 (≥60s before promotion).

// Permit pseudonymity-class session establishment.
permit(
  principal,
  action == Action::"establish_pseudonymity_session",
  resource
)
when {
  resource.tenant.pseudonymity_enabled == true &&
  resource.tenant.audience_type in [
    "HIGH_RISK_USER",
    "ANONYMOUS_FEEDBACK"
  ]
};

// FORBID cross-pseudonym correlation.
forbid(
  principal in PrincipalGroup::"tenant_admin",
  action in [
    Action::"correlate_pseudonyms",
    Action::"enumerate_pseudonyms",
    Action::"resolve_pseudonym_to_legal_identity"
  ],
  resource
)
when {
  resource.principal.namespace like "oyatie.pseudonym.*"
};

// FORBID metadata-revealing actions on pseudonymity-class
// principals.
forbid(
  principal,
  action in [
    Action::"resolve_pseudonym_ip",
    Action::"resolve_pseudonym_fingerprint",
    Action::"resolve_pseudonym_user_agent"
  ],
  resource
)
when {
  resource.principal.namespace like "oyatie.pseudonym.*"
};
```

**`policy/whistleblower-channel.cedar`** (per this ADR):

```cedar
// policy/whistleblower-channel.cedar
// Per ADR-0300 whistleblower channel.
// Soak window: per ADR-0294 (≥60s before promotion).

// Permit whistleblower submission receipt.
permit(
  principal,
  action == Action::"submit_whistleblower",
  resource
)
when {
  resource.tenant.whistleblower_channel_enabled == true &&
  resource.tenant.active_compliance_packs.containsAny([
    "publisher-source-protection",
    "pack-sox-806-us",
    "pack-eu-whistleblower-directive-2019-1937",
    "pack-kr-anti-corruption-act",
    "pack-us-dodd-frank-922",
    "pack-jp-whistleblower-protection-act-2004",
    "pack-uk-pida-1998",
    "pack-au-pida-2013"
  ]) &&
  principal.namespace like "oyatie.pseudonym.session.*" &&
  context has sealed_sender_envelope_present &&
  context.sealed_sender_envelope_present == true &&
  context has metadata_minimization_applied &&
  context.metadata_minimization_applied == true
};

// FORBID tenant-admin access to submitter identity.
forbid(
  principal in PrincipalGroup::"tenant_admin",
  action in [
    Action::"read_submitter_identity",
    Action::"correlate_submitter_across_submissions",
    Action::"export_submitter_metadata"
  ],
  resource
)
when {
  resource.submission.principal.namespace like
    "oyatie.whistleblower.submission.*"
};

// Permit ombudsman access to sealed payload (via per-pack
// 2-member quorum per ADR-0247 break-glass).
permit(
  principal in PrincipalGroup::"ombudsman_quorum",
  action == Action::"unseal_whistleblower_submission",
  resource
)
when {
  context.quorum_members_count >= 2 &&
  context.quorum_members_distinct == true
};

// Permit per-jurisdiction reporter-privilege assertion.
permit(
  principal in PrincipalGroup::"council_legal",
  action == Action::"assert_reporter_privilege",
  resource
)
when {
  context.disclosure_request_class in [
    "subpoena",
    "foia",
    "dsar_third_party",
    "court_order"
  ]
};
```

#### §D-5. Per-pack reporter-privilege overlay roster

Each pack adds an overlay at `packs/<pack-slug>/policy/
whistleblower-channel-overlay.cedar`. The active roster:

| # | Pack slug | Statute | Reporter-privilege scope | Mandatory escalation cadence |
|---:|---|---|---|---|
| 1 | `publisher-source-protection` | First Amendment + per-state US shield laws + Art. 19 UDHR + EU CFR Art. 11 + KR Press Freedom Act | Journalist source identity | Per-tenant ombudsman; no regulator escalation unless court order |
| 2 | `pack-sox-806-us` | Sarbanes-Oxley Act §806 (18 USC §1514A) | Corporate whistleblower in publicly-traded company | ≤30-day acknowledgement; ≤60-day investigation per SEC OWB |
| 3 | `pack-eu-whistleblower-directive-2019-1937` | Directive (EU) 2019/1937 on the protection of persons who report breaches of Union law | EU whistleblower (50+ employee tenants mandatory) | ≤7-day acknowledgement; ≤3-month follow-up per Art. 9 |
| 4 | `pack-kr-anti-corruption-act` | Korea Anti-Corruption and Civil Rights Commission Act | KR public-sector + listed-corporation whistleblower | ≤60-day investigation per ACRC |
| 5 | `pack-us-dodd-frank-922` | Dodd-Frank Wall Street Reform Act §922 (15 USC §78u-6) | SEC + CFTC whistleblower (financial services tenants) | ≤10% award to whistleblower; per-tenant attorney-client privilege scope |
| 6 | `pack-jp-whistleblower-protection-act-2004` | Whistleblower Protection Act (Japan, 2004; revised 2020) | JP public-sector + private-sector whistleblower | ≤20-day acknowledgement per CAA |
| 7 | `pack-uk-pida-1998` | Public Interest Disclosure Act 1998 | UK whistleblower | Per-tenant ombudsman; no statutory cadence |
| 8 | `pack-au-pida-2013` | Public Interest Disclosure Act 2013 (Cth) | AU Commonwealth public sector | ≤90-day investigation per Commonwealth Ombudsman |
| 9 | `pack-us-shield-law-<state>` | Per-state US shield laws (~40 states) | Journalist source identity | Per-state jurisdictional override |

#### §D-6. Tor-friendly ingress invariant

The Tor-friendly ingress invariant: anonymity-class surfaces MUST
accept Tor onion-service v3 ingress AND MUST NOT gate-block on
Tor exit-node IP. The invariant is encoded in:

- **Cedar fragment §D-4.** The pseudonymity-class principal has
  no IP attribute; downstream policies cannot reference IP for
  these principals.
- **Tier-0 edge configuration.** The WAF rule "block Tor exit
  nodes" is explicitly DISABLED for anonymity-class surfaces.
- **Abuse-defence layer.** Bot-score forwarding is suppressed
  (the `X-Oya-Bot-Score` header is stripped on anonymity-class
  surfaces).
- **Audit emission.** Tor-origin flag is logged via the per-
  session pseudonymity principal but not via per-IP audit.
- **Per-cell Tor relay deployment.** The platform operates
  per-cell Tor relays per §D-2 step 4.

#### §D-7. Abuse-defence observation-only on anonymity-class traffic

Per ADR-0297 §D-7 (the row 7 anti-pattern call-out), the abuse-
defence layer is observation-only on anonymity-class traffic:

| Control | Anonymity-class behavior |
|---|---|
| Edge rate-limiting | Per-session quota (not per-IP); ≤100 submissions/session-hour |
| Behavioural fingerprinting | Hashed-with-pepper before forwarding; never gates |
| Bot-management with ML scoring | Suppressed; X-Oya-Bot-Score not forwarded |
| CAPTCHA-on-suspicion | NEVER presented on anonymity-class surfaces |
| Device attestation | NEVER required on anonymity-class surfaces |
| Stolen-credential check | N/A (no persistent credential) |
| Per-action quota gates | Per-session quota; cross-session correlation forbidden |
| Honeypot routes | Active (detects scraping of pseudonymity-class endpoints) |
| Email anti-spoof | N/A (per-session reply-path via per-session key) |
| Domain anti-spoof / cert pinning | Strict TLS 1.3 + ECH + PQC where supported |
| Identity anti-spoof | N/A (sealed-sender envelope is the identity primitive) |
| Session anti-spoof | Per-session HMAC bound to per-session ephemeral key |
| Payload anti-spoof | Sealed-sender envelope signed by per-session key |
| Audit-trail anti-spoof | Per-pack ombudsman signing key per ADR-0296 |
| Webhook anti-spoof | N/A |
| Caller anti-spoof (workload identity) | SPIFFE SVID for inter-µservice; anonymity-class principal is the session SVID |

#### §D-8. Per-pack ombudsman chain-of-custody

Per-pack ombudsman receives the sealed submission and unseals via
the per-pack 2-member quorum per ADR-0247:

1. **Sealed submission arrival.** The submission arrives in the
   per-pack ombudsman queue with the sealed envelope + the
   chain-of-custody hash.
2. **Per-pack ombudsman roster.** Each pack has a per-pack
   ombudsman roster (e.g., `packs/pack-sox-806-us/ombudsman/
   reviewers.yaml`) with on-call rotation.
3. **2-member quorum.** Two distinct ombudsman members
   authenticate per ADR-0247; both unseal the envelope via the
   per-pack key.
4. **Chain-of-custody seal.** Every action on the unsealed
   payload is signed by both ombudsman members + cryptographically
   sealed per ADR-0028.
5. **Per-pack regulator notification.** Per ADR-0251 + per-pack
   `breach_notification_workflow_id` where applicable (e.g., SOX
   806 mandates SEC OWB escalation on material allegations).
6. **Submitter acknowledgement (if opted-in).** Per the per-
   session reply-path: the ombudsman posts an acknowledgement
   to the per-session reply-channel URL using the per-session
   reply-path public key; the submitter polls + decrypts with
   their per-session private key.

#### §D-9. Cross-tenant identity-isolation enforcement

Per the `PseudonymityIdentityIsolationViolationDetected` event:

1. **Detection.** The platform's pseudonymity-isolation detector
   monitors tenant-admin actions for cross-pseudonym correlation
   attempts (e.g., joining audit-stream rows by per-session hash,
   exporting metadata across submissions, querying the tenant
   for pseudonym-to-IP mapping).
2. **Per-action FORBID.** The Cedar fragment §D-4 FORBIDs these
   actions at the policy gate.
3. **Detection alert.** On attempt, the event fires + alerts
   ops-trust-and-safety + the per-pack ombudsman.
4. **Per-pack regulator notification.** Per ADR-0251 +
   `breach_notification_workflow_id` for the affected pack.
5. **Tenant policy review.** Recurring violations trigger per-
   pack tenant policy review per ADR-0294 Cedar fragment
   emergency rollback runbook.

#### §D-10. Per-jurisdiction reporter-privilege legal assertion

When the platform receives a discovery / disclosure request for
an anonymity-class submission:

1. **Assertion trigger.** The legal team receives the request
   via per-pack legal workflow (per ADR-0251 +
   `disclosure_request_workflow_id`).
2. **Per-pack overlay lookup.** The overlay per §D-5 declares
   the applicable per-jurisdiction reporter-privilege scope.
3. **`ReporterPrivilegeAsserted` event.** The platform emits the
   audit event with: assertion class (subpoena / FOIA / DSAR /
   court-order), per-pack jurisdiction, statute citation,
   internal counsel-id.
4. **Operational refusal.** The platform's legal team formally
   refuses the disclosure citing the per-jurisdiction statute.
5. **Per-pack regulator notification.** Per ADR-0251 + per-pack
   workflow (e.g., EU Whistleblower Directive transposition's
   member-state competent authority notification).
6. **Transparency report.** Per ADR-0251 §D-N: the per-pack
   transparency report includes the request count + assertion
   count per quarter without revealing per-request details.

#### §D-11. Per-µservice ARCHITECTURE.md §whistleblower-channel section

Every µservice serving HIGH_RISK_USER or publisher-source-
protection audience-types SHALL include in ARCHITECTURE.md:

1. **Submission surface inventory.** Which surfaces serve
   whistleblower / journalist-source / activist / pseudonymous
   submissions (e.g., REST `/v1/submissions/anonymous`, Tor
   onion-service path).
2. **Per-pack roster.** Which packs are enabled for this µservice's
   tenants.
3. **Cedar fragment reference.** Cite `policy/anonymity-substrate.
   cedar` + `policy/whistleblower-channel.cedar` + per-pack
   overlays.
4. **Audit-event-class emission.** Cite the six audit-event
   classes.
5. **Metadata-minimization filter configuration.** Cite the §D-3
   roster + any per-tenant extension.
6. **Tor-friendly ingress configuration.** Cite the per-zone
   onion-address + per-cell Tor relay deployment.
7. **Sealed-sender envelope configuration.** Cite the per-tenant
   recipient key registry.
8. **Per-pack ombudsman roster reference.**

#### §D-12. Multispectrum review v2.4.0 wiring

Per ADR-0243 §D-8: F1 (security: cross-pseudonym correlation
forbidden), F2 (privacy: end-to-end anonymity to the receiving
tenant), F3 (reliability: per-pack ombudsman queue does not
saturate), F4 (performance: ≤30s submission-to-routing), F5
(cost: per-submission Ed25519 + XChaCha20-Poly1305 cost), F6
(operability: per-pack runbook), F7 (compliance: per-pack
reporter-privilege overlay), F8 (user safety: submitter
identity preserved against tenant + against state-level
adversary), F9 (accessibility: per WCAG 2.2 AAA on submission
surface — screen-reader-accessible, non-CAPTCHA, alternative
text on document upload), M1 (meta-policy: Cedar chain ordering),
M2 (meta-architecture: substrate primitive), A1-A7 (own-policy
adherence).

#### §D-13. Failure-mode tree — explicit walk-through

Per documentation-rigor.md §1.1 rigor sub-test, the explicit
failure modes (cross-referencing FM-1..FM-10 from §A.3) are
addressed:

- **FM-1 receiving tenant cross-correlation:** Cedar fragment
  §D-4 FORBID + detection event §D-9.
- **FM-2 state-level subpoena:** Sealed-sender envelope §D-1 +
  per-jurisdiction reporter-privilege §D-10.
- **FM-3 abuse-defence fingerprint forwarding:** Observation-
  only §D-7 + hashed-with-pepper §D-3 row 5.
- **FM-4 Tor exit-node gate-block:** Tor-friendly invariant
  §D-6.
- **FM-5 ECH downgrade:** ECH-enabled TLS per ADR-0253 §D-10;
  per-µservice ECH configuration in §D-11 step 7.
- **FM-6 per-session key compromise:** Per-session ephemeral
  ≤24h TTL §D-1; per-pack rotation ≤30 days.
- **FM-7 ombudsman insider:** Per-pack 2-member quorum §D-8 +
  UEBA monitoring.
- **FM-8 contextual correlation:** Timing minimization §D-3 row
  15 + per-tenant operational training.
- **FM-9 tenant exfiltration via anonymous submissions:** Per-
  pack policy authorization §B Decision 1.
- **FM-10 submitter self-de-anonymization:** Metadata-strip
  filter §D-3 + per-pack ombudsman redaction §D-8.

## Implementation footprint

### §E. Implementation footprint

#### §E.1. New crates

```text
crates/oya-shared-anonymity-substrate/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs                       // Public surface (PseudonymitySession, MetadataStripFilter)
│   ├── pseudonymity/
│   │   ├── mod.rs
│   │   ├── session.rs               // Per-session pseudonymity principal
│   │   └── isolation_invariant.rs   // Cross-pseudonym correlation detector
│   ├── metadata_minimization/
│   │   ├── mod.rs
│   │   ├── http_headers.rs
│   │   ├── tls_fingerprint.rs
│   │   ├── document_metadata.rs
│   │   ├── exif_strip.rs
│   │   ├── pdf_strip.rs
│   │   ├── office_strip.rs
│   │   ├── image_steganography_strip.rs
│   │   └── hlc_jitter.rs
│   ├── tor_ingress/
│   │   ├── mod.rs
│   │   ├── onion_service_v3.rs
│   │   └── per_cell_relay.rs
│   ├── audit.rs
│   ├── cedar.rs
│   ├── metrics.rs
│   ├── traces.rs
│   └── error.rs
└── tests/
    ├── pseudonymity_isolation_property.rs
    ├── metadata_strip_completeness.rs
    └── tor_onion_v3_ingress.rs

crates/oya-shared-whistleblower-channel/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs                       // Public surface (SealedSenderEnvelope, OmbudsmanChainOfCustody)
│   ├── sealed_sender/
│   │   ├── mod.rs
│   │   ├── envelope.rs              // Ed25519 + XChaCha20-Poly1305 envelope
│   │   ├── per_session_key.rs
│   │   └── per_tenant_key_registry.rs
│   ├── ombudsman/
│   │   ├── mod.rs
│   │   ├── chain_of_custody.rs
│   │   ├── quorum.rs                // Per-pack 2-member quorum per ADR-0247
│   │   └── per_pack_router.rs
│   ├── reply_path/
│   │   ├── mod.rs
│   │   └── per_session_reply_channel.rs
│   ├── per_pack_overlay/
│   │   ├── mod.rs
│   │   ├── publisher_source_protection.rs
│   │   ├── sox_806_us.rs
│   │   ├── eu_whistleblower_directive.rs
│   │   ├── kr_anti_corruption_act.rs
│   │   ├── us_dodd_frank_922.rs
│   │   ├── jp_whistleblower_protection.rs
│   │   ├── uk_pida_1998.rs
│   │   ├── au_pida_2013.rs
│   │   └── us_shield_law_per_state.rs
│   ├── reporter_privilege.rs
│   ├── audit.rs
│   ├── cedar.rs
│   ├── metrics.rs
│   ├── traces.rs
│   └── error.rs
└── tests/
    ├── sealed_sender_envelope_roundtrip.rs
    ├── ombudsman_quorum_unseal.rs
    ├── per_pack_overlay_application.rs
    ├── reporter_privilege_assertion.rs
    └── cross_tenant_isolation.rs
```

Public surface:

```rust
// oya-shared-anonymity-substrate
pub trait PseudonymitySession: Send + Sync {
    fn establish(&self, tenant: &TenantId, audience_type: &AudienceType)
        -> Result<PseudonymitySessionHandle, AnonymityError>;
    fn close(&self, handle: PseudonymitySessionHandle)
        -> Result<(), AnonymityError>;
}

pub trait MetadataStripFilter: Send + Sync {
    fn strip(&self, request: &mut Request) -> Result<(), StripError>;
    fn strip_document(&self, doc: &mut Document)
        -> Result<(), StripError>;
}

// oya-shared-whistleblower-channel
pub trait SealedSenderEnvelope: Send + Sync {
    fn seal(&self, payload: &[u8], recipient_key: &TenantRecipientKey)
        -> Result<SealedEnvelope, EnvelopeError>;
    fn unseal_via_quorum(&self, envelope: &SealedEnvelope,
                          quorum: &OmbudsmanQuorum)
        -> Result<Vec<u8>, EnvelopeError>;
}

pub trait OmbudsmanChainOfCustody: Send + Sync {
    fn record_action(&self, action: &OmbudsmanAction)
        -> Result<ChainOfCustodyHash, ChainOfCustodyError>;
    fn assert_reporter_privilege(&self,
                                   disclosure_request: &DisclosureRequest)
        -> Result<PrivilegeAssertion, ChainOfCustodyError>;
}
```

#### §E.2. Cedar fragments

`microservices/<ms>/policy/anonymity-substrate.cedar` +
`microservices/<ms>/policy/whistleblower-channel.cedar` per §D-4.

#### §E.3. IaC manifests

`microservices/<ms>/iac/<env>-whistleblower-channel.yaml` declares:

```yaml
apiVersion: oyatie.io/v1
kind: WhistleblowerChannel
metadata:
  name: <ms>-whistleblower-channel
  microservice: <ms>
  env: <env>
spec:
  per_tenant_packs:
    - publisher-source-protection
    - pack-sox-806-us
    - pack-eu-whistleblower-directive-2019-1937
    - pack-kr-anti-corruption-act
    - pack-us-dodd-frank-922
    - pack-jp-whistleblower-protection-act-2004
    - pack-uk-pida-1998
    - pack-au-pida-2013
  per_zone_onion_address: <56-char-base32>.onion
  metadata_minimization_enabled: true
  sealed_sender_envelope_enabled: true
  abuse_defence_mode: observation_only
  observability:
    metrics: true
    traces: true
    audit_events:
      - WhistleblowerSubmissionReceived
      - WhistleblowerSubmissionRoutedToOmbudsman
      - PseudonymitySessionEstablished
      - PseudonymityIdentityIsolationViolationDetected
      - JournalistSourceAccessGranted
      - ReporterPrivilegeAsserted
```

#### §E.4. Submission channel spec: `specs/whistleblower-submission-channel.json`

JSON Schema per documentation-rigor.md §2 spec rigor.

#### §E.5. CI lanes per §B Decision

```text
.github/workflows/oya-governance-whistleblower-channel.yml
.github/workflows/oya-governance-anonymity-substrate.yml
.github/workflows/oya-governance-whistleblower-channel-tor-friendly-ingress.yml
.github/workflows/oya-governance-whistleblower-channel-metadata-minimization.yml
.github/workflows/oya-governance-whistleblower-channel-pseudonymity-scope.yml
.github/workflows/oya-governance-whistleblower-channel-per-pack-reporter-privilege.yml
.github/workflows/oya-governance-whistleblower-channel-cedar-fragment-present.yml
```

## Migration

### §F. Migration plan

#### §F.1. Phase 0 — Doctrine acceptance (2026-05-20 — 2026-05-27)

This ADR accepted in text. Shared crate skeleton + Cedar fragment
skeleton + per-pack overlay roster land. CI lanes promote to
advisory.

#### §F.2. Phase 1 — Per-pack reporter-privilege overlay onboarding (2026-05-27 — 2026-07-15)

Onboard each pack per §D-5 (9 packs). Per-pack legal-team
training + per-pack ombudsman roster provisioning.

#### §F.3. Phase 2 — Per-cell Tor relay infrastructure (2026-06-01 — 2026-07-15)

Deploy per-cell Tor relays (3-hop per Tor protocol) + per-cell
exit nodes. SPIFFE-attest per ADR-0295.

#### §F.4. Phase 3 — Per-tenant SecureDrop-class surface provisioning (2026-06-15 — 2026-08-01)

Per-tenant onboarding of `publisher-source-protection` pack +
per-tenant onion-service v3 address publication.

#### §F.5. Phase 4 — Per-µservice wiring (2026-07-01 — 2026-08-15)

Every µservice serving HIGH_RISK_USER / publisher-source-
protection audience-types adds the §D-11 ARCHITECTURE.md section
+ the Cedar fragments + the IaC manifest. CI lanes promote to
BLOCKER on 2026-08-15.

#### §F.6. Phase 5 — BLOCKER promotion (2026-08-15)

The seven CI lanes promote to BLOCKER.

#### §F.7. Rollback path

Per-fragment Cedar rollback within ≤5 minutes per ADR-0294. Per-
pack overlay rollback within ≤15 minutes. Per-tenant onion-service
rollback (≤30 minutes — onion-address republication).

Rollback does NOT compromise existing in-flight submissions; they
continue under the pre-rollback fragment. Per-session sealed-sender
envelopes are unaffected by Cedar policy rollback (the cryptographic
protocol is independent of the policy gate).

## References

### §G. References

#### §G.1. Hyperscaler precedents

- SecureDrop (Freedom of the Press Foundation): `docs.securedrop.org`
- Signal Sealed Sender: `signal.org/blog/sealed-sender/`
- Apple Private Relay: `support.apple.com/en-us/HT212614`
- Tor Project onion service v3 specification: `spec.torproject.org/rend-spec-v3.txt`
- Cloudflare Onion Routing for Tor: `blog.cloudflare.com/cloudflare-onion-service/`
- Mullvad VPN no-log + multihop: `mullvad.net/en/help/privacy-policy`
- ProtonMail anonymous-signup: `proton.me/blog/anonymous-email`
- OnionShare anonymous file transfer: `onionshare.org/`
- Tails OS source-protection: `tails.boum.org`

#### §G.2. Regulatory anchors

- **US — SOX 806** (18 USC §1514A; Sarbanes-Oxley Act §806;
  whistleblower protection for publicly-traded company employees).
- **US — Dodd-Frank §922** (15 USC §78u-6; SEC + CFTC whistleblower
  award + protection).
- **US — First Amendment** + **per-state shield laws** (~40
  states; e.g., NY Shield Law 2024, CA Shield Law).
- **US — FOIA (5 USC §552)** (Freedom of Information Act).
- **EU — Directive (EU) 2019/1937** (Whistleblower Directive;
  protection of persons who report breaches of Union law).
- **EU — Charter of Fundamental Rights Article 11** (freedom of
  expression + media freedom).
- **EU — GDPR Article 15** (data subject access right;
  reporter-privilege exception per Art. 23 derogations).
- **KR — Anti-Corruption and Civil Rights Commission Act**
  (KR ACRC Act; KR whistleblower protection).
- **KR — Press Freedom Act**.
- **JP — Whistleblower Protection Act 2004** (revised 2020;
  Cabinet Office Bureau of Consumer Affairs).
- **UK — Public Interest Disclosure Act 1998** (UK whistleblower
  protection).
- **AU — Public Interest Disclosure Act 2013** (Commonwealth
  public sector whistleblower protection).
- **UN — Universal Declaration of Human Rights Article 19**
  (freedom of opinion and expression).

#### §G.3. Keystone bundle 2026-05-20 cross-references

- **ADR-0297** (abuse-defence baseline): abuse-defence is
  observation-only on anonymity-class traffic per §D-7.
- **ADR-0298** (emergency-services bypass): emergency-services
  takes precedence; a whistleblower revealing an imminent life-
  safety threat transitions to the bypass.
- **ADR-0299** (account-recovery resilience): pseudonymity-class
  sessions are per-session ephemeral; per-session sealed-sender
  key is the recovery primitive.
- **ADR-0242** (oyatie-is-a-tenant): pseudonymity-class principals
  occupy `oyatie.pseudonym.session.*` under platform tenant tree.
- **ADR-0243** (Cedar universal gate): anonymity policies are
  Cedar fragments.
- **ADR-0244** (tenant scoping primitive): HIGH_RISK_USER
  audience-type + publisher-source-protection pack overlay.
- **ADR-0246** (policy-engine library-first): library-first Cedar
  evaluator carries both anonymity-substrate + whistleblower-
  channel fragments.
- **ADR-0247** (self-modification / break-glass): per-pack
  ombudsman 2-member quorum.
- **ADR-0248** (cellular architecture): per-cell Tor relay
  deployment.
- **ADR-0250** (build-ahead-of-certification): built certified-
  shape day one across all nine packs.
- **ADR-0251** (compliance packs): per-pack overlay per §D-5.
- **ADR-0252** (HLC + TrueTime): HLC-clocked + per-session-jittered
  timestamps.
- **ADR-0253** (HTTP/3 + QUIC + ECH + PQC): ECH for inner-SNI
  protection; PQC for harvest-now-decrypt-later defence.
- **ADR-0263** (observability emission contract): six new
  audit-event classes.
- **ADR-0272** (cookie consent per-purpose): anonymity-class
  sessions never set persistent cookies.
- **ADR-0273** (per-tenant DKIM/SPF/DMARC): per-tenant DKIM-
  signed acknowledgement only if submitter opts-in.
- **ADR-0276** (backup portability GDPR Art. 20): anonymity-
  class data NOT subject to tenant-portability.
- **ADR-0280** (substrate-of-substrate): depends on cedar-
  evaluator + audit-emit + emergency-services-bypass.
- **ADR-0284** (platform-owner name indirection): namespace
  parameterized.
- **ADR-0292** (minor user doctrine): minor whistleblower self-
  reporting bypasses parental control.
- **ADR-0293** (meta-trust-root): per-session sealed-sender keys
  rooted at meta-trust-root.
- **ADR-0294** (Cedar fragment soak): ≥60s soak window.
- **ADR-0295** (bootstrap CI SPIFFE + kill-switch): SPIFFE
  identity + per-cell kill-switch.
- **ADR-0296** (library-first credential sidecar): per-session
  sealed-sender keys held in sidecar with ≤60s OpenBao TTL.

#### §G.4. Companion docs

- `docs/standards/documentation-rigor.md` §3.2.5 rows 6, 7, 16,
  21.
- `docs/runbooks/whistleblower-submission-on-call.md`.
- `docs/runbooks/per-pack-ombudsman-quorum-unseal.md`.
- `docs/runbooks/reporter-privilege-assertion.md`.
- `docs/runbooks/pseudonymity-isolation-violation-response.md`.

#### §G.5. Cross-back-pointer follow-ups for existing ADRs

- **ADR-0297** (abuse-defence baseline): add §D-N cross-reference
  noting anonymity-class traffic is observation-only.
- **ADR-0299** (account-recovery resilience): cross-reference
  pseudonymity-class sessions are not subject to standard recovery.
- **ADR-0263** (observability emission contract): register the
  six new audit-event classes.
- **ADR-0247** (break-glass): cross-reference per-pack ombudsman
  2-member quorum pattern.
- **ADR-0292** (minor user doctrine): cross-reference minor
  whistleblower self-report path overriding parental-control.
- **ADR-0244** (tenant scoping primitive): cross-reference the
  HIGH_RISK_USER audience-type + publisher-source-protection
  pack overlay.

## Change log

### §H. Change log

| Date | Author | Change |
|---|---|---|
| 2026-05-20 | council-architecture + axis-anonymity + axis-whistleblower-channel | Initial Proposed status; bundled with the keystone-bundle 2026-05-20 foundational doctrine as the critical-path-doctrine-cluster-rows-6-7-16-21 keystone. Authored per documentation-rigor.md §3.2.5 rows 6, 7, 16, 21. Cross-references ADR-0297 + ADR-0298 + ADR-0299 + the entire keystone bundle 2026-05-20. |
