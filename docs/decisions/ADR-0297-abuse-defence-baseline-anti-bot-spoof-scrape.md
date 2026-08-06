---
id: ADR-0297
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
  - axis-edge
  - axis-network
  - axis-identity
  - axis-policy-engine
  - axis-audit-chain
  - axis-consent
  - axis-tenancy
supersedes: []
amends: []
superseded_by: []
related:
  - ADR-0028-audit-chain-merkle-sealed.md
  - ADR-0044-service-mesh-and-mtls.md
  - ADR-0099-data-class-registry.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0140-cedar-policy-enforcement.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0148-service-mesh-cilium-ambient-layered.md
  - ADR-0149-api-gateway-vs-service-mesh-separation.md
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
  - ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md
  - ADR-0319-front-middle-back-office-information-barrier.md
related_specs:
  - /specs/platform-architecture.json
  - /specs/microservices/edge-gateway.json
  - /specs/microservices/api-gateway.json
  - /specs/microservices/identity.json
  - /specs/microservices/policy-engine.json
  - /specs/microservices/audit-chain.json
  - /specs/microservices/observability.json
  - /specs/abuse-defence-controls.json
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
keystone_position: promotion-gate-fix-abuse-defence-baseline
purpose: >
  Establish the Abuse-Defence Baseline — three orthogonal control
  families (anti-bot, anti-spoof, anti-scrape) wired at three layers
  (Tier-0 edge, per-µservice, Cedar policy) — as a substrate-level
  primitive for every internet-facing oyatie surface. The bar is:
  a determined adversary with hyperscaler-grade resources (residential
  proxy farms, AI-driven CAPTCHA solvers, distributed credential
  stuffing as a service, generative scraping at scale, reflection
  amplification) cannot succeed via volumetric, credential-stuffing,
  spoofing, or scraping attacks. Defence-in-depth: no single control
  gates; layered scoring + adaptive challenge + Cedar policy
  composition. Codifies the 24-row taxonomy from
  documentation-rigor.md §3.2.3 + the per-cell-tier variants + the
  per-tenant audience-type tuning + the compliance interactions
  (GDPR Article 14 / 21, CCPA Do-Not-Sell, COPPA <13 refusal, KOSA
  minor tier).
enforcement_status: advisory-until-2026-08-15-blocker-thereafter
enforced_by:
  - cloud-ci/Rust gate packet abuse-defence-anti-bot-coverage
  - cloud-ci/Rust gate packet abuse-defence-anti-spoof-coverage
  - cloud-ci/Rust gate packet abuse-defence-anti-scrape-coverage
  - cloud-ci/Rust gate packet abuse-defence-cedar-fragment-present
  - cloud-ci/Rust gate packet abuse-defence-cell-tier-variant-coherence
  - cloud-ci/Rust gate packet abuse-defence-audience-type-tuning
  - cloud-ci/Rust gate packet abuse-defence-compliance-surface-present
naming_justifications:
  - name: oya-shared-abuse-defence
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.abuse-defence
    justification: >
      Per ADR-0105 13-layer canonical enum row 5 (shared-substrate), the
      crate that exposes the Cedar evaluator hook + bot-score forwarding
      trait + spoof-detector trait + scrape-pattern-detector trait
      across every internet-facing µservice belongs at the shared layer.
      Naming `oya-shared-abuse-defence` keeps the single-concern flat
      layout per ADR-0131 and avoids any "suite" packaging per ADR-0132.
  - name: oya-governance-anti-bot-coverage
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.anti-bot-coverage
    justification: >
      CI fitness lane per ADR-0212 buildability doctrine; verifies
      every internet-facing µservice declares the 8-row anti-bot taxonomy
      in its ARCHITECTURE.md §abuse-defence + iac/<env>-edge-waf.yaml.
      Lane naming follows the canonical `oya-governance-<concern>`
      shape consistent with sibling lanes (per documentation-rigor.md
      §3.2.3 + ADR-0212 §G).
  - name: oya-governance-anti-spoof-coverage
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.anti-spoof-coverage
    justification: >
      CI fitness lane per ADR-0212; verifies the 8-row anti-spoof
      taxonomy is wired across DKIM/SPF/DMARC/ARC/BIMI + strict TLS +
      session-token + workload-identity surfaces. Companion to the
      anti-bot lane.
  - name: oya-governance-anti-scrape-coverage
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.anti-scrape-coverage
    justification: >
      CI fitness lane per ADR-0212; verifies the 8-row anti-scrape
      taxonomy is wired across rate-limiting, pattern-anomaly detection,
      robots.txt authority, paid API tier, watermarking, adaptive
      challenge, content rewriting, legal-channel registration.
  - name: oya-governance-abuse-defence
    layer: N/A (foundry-fitness aggregate CI lane)
    bnf_segments: oya.foundry-fitness.abuse-defence
    justification: >
      Aggregate fitness lane per ADR-0212; rolls up the three child
      lanes into a single advisory/BLOCKER gate per the keystone-bundle
      2026-05-20 promotion-gate model.
  - name: X-Oya-Bot-Score
    layer: N/A (HTTP header naming)
    bnf_segments: X-Oya.Bot-Score
    justification: >
      Custom HTTP request header carrying the Tier-0 edge bot-score
      forwarded to downstream µservices; namespace prefix `X-Oya-`
      reserves the platform's header surface and avoids collision with
      Cloudflare's `Cf-Bot-Score`, AWS WAF's `X-Amzn-Bot-Score`, or
      Akamai's `X-Akamai-Bot-Score` precedents.
  - name: AbuseDefenceBotBlocked
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: AbuseDefence.BotBlocked
    justification: >
      Audit-event-class emitted whenever Tier-0 edge or µservice Cedar
      gate forbids a request on bot-score grounds; registered in
      ADR-0263 central registry to satisfy the §3.2.2 consistency
      invariant.
  - name: AbuseDefenceSpoofDetected
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: AbuseDefence.SpoofDetected
    justification: >
      Audit-event-class emitted on detection of any anti-spoof control
      firing (DMARC reject, mTLS mismatch, session-token replay,
      webhook HMAC mismatch). Registered per ADR-0263.
  - name: AbuseDefenceScrapeBlocked
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: AbuseDefence.ScrapeBlocked
    justification: >
      Audit-event-class emitted on pattern-anomaly scrape blocking.
      Registered per ADR-0263.
  - name: policy/abuse-defence.cedar
    layer: N/A (per-µservice Cedar fragment file)
    bnf_segments: policy.abuse-defence
    justification: >
      Canonical filename for the per-µservice abuse-defence Cedar
      fragment under the µservice's `policy/` directory per ADR-0246 +
      ADR-0243 fragment-lifecycle conventions; single-concern naming
      keeps the policy directory's contract-by-name invariant.
  - name: iac/<env>-edge-waf.yaml
    layer: N/A (per-µservice IaC manifest)
    bnf_segments: iac.<env>.edge-waf
    justification: >
      Canonical filename for per-µservice + per-env edge WAF IaC
      manifest; pairs with the Cedar fragment to express the
      defence-in-depth at the IaC layer; env slug per ADR-0254
      deployment-model-spectrum.
  - name: FRIENDLY_CRAWLER_PARTNER
    layer: N/A (Tenant.audience_type enum value per ADR-0244)
    bnf_segments: audience_type.FRIENDLY_CRAWLER_PARTNER
    justification: >
      Tenant.audience_type enum extension per ADR-0244 §D-3; identifies
      accredited search engines, academic researchers, and partner
      data-aggregators that receive bot-score allow-list bypass while
      still being rate-limited and audit-logged.
---

# ADR-0297: Abuse-Defence Baseline — Anti-Bot + Anti-Spoof + Anti-Scrape

## Status

Proposed — 2026-05-20.

Bundled with the keystone-bundle 2026-05-20 foundational doctrine
synthesis as the **promotion-gate-fix-abuse-defence-baseline**
keystone, closing the gap identified in
`docs/standards/documentation-rigor.md` §3.2.3 (row 28 of the
per-µservice ADR-adherence checklist). The standard already
codifies the 24-row taxonomy + the Cedar fragment shape + the
CI lanes + the per-cell-tier variants; this ADR is the binding
ADR the standard's row 28 cites.

Enforcement is `advisory-until-2026-08-15-blocker-thereafter`. The
doctrine accepts in text immediately; the CI lanes that enforce it
promote to BLOCKER on 2026-08-15 to give per-µservice rollout
sequenced by audience exposure (per §F migration) time to land.
Until 2026-08-15, validators emit findings without failing CI;
post-2026-08-15, the lanes block merge.

## Date

2026-05-20.

## Context

### §A. Why abuse-defence is a substrate primitive, not a µservice afterthought

Mature hyperscaler platforms treat abuse-defence as a *first-class
substrate primitive* — wired at the planetary edge, in every
internet-facing µservice's request path, and as Cedar policy
composing with every other gate. The pattern is unambiguous across
the named industry references:

- **Cloudflare** ships Bot Management + Turnstile + WAF + Super Bot
  Fight Mode + Browser Integrity Check + DDoS protection as integrated
  Tier-0 edge primitives that EVERY zone receives by default. Per
  Cloudflare's published architecture (blog.cloudflare.com 2018-2025
  "Bot Management" series), the bot-score is computed at every POP
  before the request enters the customer's origin; the score is
  forwarded as `Cf-Bot-Score` (0-99) for downstream policy. No
  Cloudflare customer wires bot-defence themselves — the substrate
  serves it.
- **AWS Shield + AWS WAF Bot Control** is provisioned as a standalone
  managed control plane offering with separate SKU (`AWSManagedRulesBotControlRuleSet`,
  released 2021; updated 2024 with `Target` + `Common` inspection
  levels). AWS treats this as a *substrate* offering — not as
  application-layer code each AWS customer authors. Per AWS Shield
  Advanced SLA documentation, AWS commits a cost-protection guarantee
  when Shield mis-classifies a legitimate burst as DDoS; the
  primitive is operated as a substrate with operational SLAs, not
  per-tenant code.
- **Google Cloud Armor** ships Adaptive Protection (ML-driven L7
  DDoS + bot defence, generally available 2021) + reCAPTCHA Enterprise
  + WAF + Bot Management as Tier-0 edge primitives served from Google
  Front End (GFE). Per Google Cloud Armor product documentation, the
  ML model is trained on Google-wide signals across every Google
  property; per-tenant tuning is configuration, not code. Same shape
  as Cloudflare: substrate first, µservice never.
- **Akamai Bot Manager** ships at Akamai's ~4,000 POPs as the Tier-0
  edge bot defence; per Akamai's State of the Internet reports
  (Q2 2024 + Q4 2024), Akamai observes ~1.3 billion bot-tagged
  requests per day across its customers — the scale demands a
  substrate primitive, not per-tenant µservice code.

The corollary: **every internet-facing surface oyatie ships MUST
inherit abuse-defence from the substrate, not author it per-µservice.**
A µservice that authors its own rate-limiting, its own bot
detection, its own credential-stuffing detection, its own scraping
defence is duplicating substrate primitives that the Tier-0 edge
already serves. That duplication is a `feedback_no_silent_regression`
violation (every µservice's defence drifts independently); it is a
`feedback_quality_performance_scalability_bar` violation (the substrate
sees signal across every µservice's traffic that a single µservice
cannot); and it is a `feedback_autonomous_implementation_artifacts`
violation (intern-buildable means the doc surface is one substrate,
not 46 µservice-private implementations).

The ADR-0297 abuse-defence baseline closes this gap.

### §A.1. Threat landscape 2026 — the adversary the substrate defends against

The 2026 abuse landscape is qualitatively different from the
2018-2022 landscape that earlier defences were designed against:

- **Residential-proxy networks.** Services like Bright Data, Oxylabs,
  Smartproxy operate ~150M+ residential IP pools rotating per request
  (per their 2024 published pool sizes). Traditional IP-based blocks
  fail because the next request comes from a clean residential ASN
  on a clean subnet. The defence: behavioural fingerprinting (JA4 /
  JA4+ / HTTP/2-3 frame patterns + WebGL fingerprint + cursor
  trajectory) + tenant-bound quota gates that survive IP rotation.
- **AI-driven CAPTCHA solvers.** Generative vision models (Anthropic
  Claude 3.5 Sonnet vision, OpenAI GPT-4o vision, Google Gemini 1.5
  Pro vision; released 2024) solve image-recognition CAPTCHAs (`select
  all images with a bicycle`) at ~85-92% accuracy — above human
  baseline ~83%. Services like CapMonster, 2Captcha, AntiCaptcha
  pipeline LLM-vision against arbitrary CAPTCHA images at ~$0.50 per
  thousand solves. The defence: behavioural CAPTCHAs (cursor
  trajectory analysis, scroll cadence, dwell time) + Turnstile-class
  invisible challenges + device attestation + proof-of-work.
- **Distributed credential stuffing as a service.** Underground forums
  (BreachForums, RaidForums successors) operate "combo list" markets
  pushing 10-100 billion `email:password` pairs per quarter; services
  like Sentry MBA, OpenBullet 2 pipeline these against any login
  endpoint at ~10k req/sec sustained per attacker via residential
  proxies. The defence: HIBP-style stolen-credential check on every
  password submission + step-up auth (WebAuthn passkeys per ADR-0188)
  + per-tenant rate limits + behavioural anomaly detection on the
  auth path.
- **Generative scraping at scale.** LLM-training-corpus collection
  drives bot traffic at unprecedented volume: Common Crawl's CCBot
  alone fetches ~10B pages/month; OpenAI's GPTBot, Anthropic's
  ClaudeBot, Google's Google-Extended, Bytespider, Amazonbot, and
  hundreds of less-disciplined crawlers operate at similar scale.
  Defence-in-depth: robots.txt + Sitemaps as the authoritative
  contract per RFC 9309 + per-bot rate limits + paid-API tier for
  legitimate bulk consumers + per-user watermarking on high-value
  content + DMCA-agent + GDPR Article 14 surface for cease-and-desist.
- **Reflection / amplification + Rapid Reset.** The October 2023
  HTTP/2 Rapid Reset attack (CVE-2023-44487) hit Google, Cloudflare,
  and AWS at ~398 million / 201 million / 155 million RPS
  respectively (per Google + Cloudflare + AWS published disclosures).
  Defence: HTTP/3 + QUIC default per ADR-0253 (Rapid Reset is HTTP/2
  specific); per-connection request caps; Tier-0 edge absorbs
  volumetric attacks before they reach the cell.
- **Spoofed sender + spear-phishing.** DKIM + SPF + DMARC reject-mode
  enforcement closes the easy spoofing path; ARC chains protect the
  forwarder case; BIMI surfaces the visual indicator. Per Google's
  2024 Postmaster requirements (one of the forcing functions of
  ADR-0273), receivers MUST receive `DMARC=pass` to be deliverable;
  the same logic applies in reverse for inbound mail.
- **Workload-identity spoofing across the mesh.** Sibling-µservice
  calls without workload identity are subject to lateral-movement
  abuse if any one µservice is compromised. SPIFFE workload identity
  per ADR-0253 + ADR-0295 closes this.

The substrate baseline MUST be sized to this 2026 landscape — not
the 2018 landscape. Cloudflare's 2024 figures alone (3.5 trillion
mitigated DDoS requests per quarter per their 2024-Q4 DDoS report)
exceed every other comparable substrate's prior decade combined.
The bar is not "block some bots"; the bar is "operate at hyperscaler
density across a continuously-evolving threat model."

### §A.2. Why the keystone bundle 2026-05-20 requires this as a Tier-0 edge primitive

The keystone bundle's foundational ADRs intersect abuse-defence as
follows:

- **ADR-0242 (oyatie-is-a-tenant).** The platform's own surfaces
  (e.g., `app.oyatie.com`, `api.oyatie.com`, `dev.oyatie.com`,
  Workflow Studio, Intelligence Console) are subject to the same
  abuse defences as any tenant surface. No carve-outs for "internal"
  traffic — the keystone retires internal-vs-external as a meaningful
  distinction.
- **ADR-0243 (Cedar universal gate).** Every abuse-defence decision
  is composable as a Cedar fragment. The bot-score, the rate-limit,
  the scrape-pattern detection, the workload-identity gate all enter
  Cedar evaluation as principal/resource/action context attributes.
  No bypass paths; no `if internal then skip`.
- **ADR-0248 (Amazon-shape cellular architecture).** Tier-0 edge
  cells host the bot-management ML model + JA4 fingerprinter + WAF;
  Tier-1/2 cells forward bot-score header; Tier-3 cells are not
  internet-facing and the control family is N/A. The cellular
  topology determines control variant per cell tier.
- **ADR-0251 (compliance packs).** Different audience types (HIGH_RISK
  financial / health, MINOR_PII per ADR-0292, B2C_CONSUMER, B2B_TENANT,
  FRIENDLY_CRAWLER_PARTNER) require different control sensitivity.
  Compliance packs (HIPAA, PCI, GDPR, COPPA, KOSA) extend the
  baseline with additional refusal predicates.
- **ADR-0253 (HTTP/3 + QUIC default + edge POPs + ECH + PQC).** The
  Tier-0 edge IS the abuse-defence layer. ECH protects the inner SNI
  from passive observers (defence against fingerprint-based active
  attacker steering); PQC hybrid KEX defends the long-term
  confidentiality of session keys for harvest-now-decrypt-later
  attackers; HTTP/3 absorbs Rapid Reset.
- **ADR-0263 (observability emission contract).** Every abuse-defence
  decision emits an audit-event-class (per the registry); every
  rate-limit hit, every bot-score forbid, every spoof detection
  becomes a row in the audit chain.
- **ADR-0273 (per-tenant DKIM/SPF/DMARC).** The anti-spoof email
  surface inherits ADR-0273's per-tenant deliverability infrastructure;
  ADR-0297 extends with ARC + BIMI + inbound DMARC enforcement.
- **ADR-0292 (minor user doctrine).** Bot-defence sensitivity is
  elevated for tenants serving minor-targeted surfaces; KOSA's
  "addictive design" prohibition + EU age-verification refuse-on-bot
  + COPPA <13 refusal each compose into abuse-defence Cedar fragments.
- **ADR-0295 (bootstrap CI SPIFFE + kill-switch).** Workload identity
  on every µservice-to-µservice call is the anti-spoof primitive for
  the east-west surface; SPIFFE SVIDs serve as the Cedar principal
  on every call.
- **ADR-0296 (library-first credential sidecar).** Provider
  credentials are held in the sidecar; the µservice never directly
  holds credentials that an abuse path could exfiltrate.

The bundle cannot land without the abuse-defence baseline articulated
explicitly. The promotion gate for the 2026-05-20 bundle is:
*the substrate MUST refuse abuse classes at hyperscaler density.*
This ADR is the binding articulation.

### §A.3. What this ADR explicitly does NOT do

- This ADR does not specify per-µservice WAF rules in detail; each
  µservice's `iac/<env>-edge-waf.yaml` declares its concrete rules
  layered atop the substrate baseline.
- This ADR does not specify per-tenant tuning UI; the tenant control
  surface for abuse-defence sensitivity is the responsibility of the
  `microservices/tenancy/` PRD per ADR-0244 + ADR-0218.
- This ADR does not redefine Cedar fragment authoring conventions —
  that is ADR-0243 + ADR-0294's scope. This ADR declares the
  *content* of `policy/abuse-defence.cedar` but the *lifecycle* is
  ADR-0294 (≥60s soak + signed publication + rollback).
- This ADR does not specify the audit-event-class registry shape —
  that is ADR-0263's scope. This ADR adds three event classes to the
  registry.

## Decision

### §B. Three orthogonal control families wired at three layers

The abuse-defence baseline is **three orthogonal control families**
(anti-bot, anti-spoof, anti-scrape) wired at **three layers**
(Tier-0 edge, per-µservice, Cedar policy). The 3×3 matrix produces
nine cells; each cell has a defined primitive. The matrix is
defence-in-depth: **no single cell gates a request alone**; the
nine cells compose via Cedar fragment evaluation and adaptive
challenge scoring.

```
                  Tier-0 edge          Per-µservice          Cedar policy
                  -----------          -------------         -------------
Anti-bot          Bot-mgmt ML +        Bot-score consume +   forbid when
                  JA4 + WAF +          quota per route +     bot_score > N
                  CAPTCHA              honeypot routes
                                                              forbid when
Anti-spoof        DMARC inbound +      mTLS + session-       caller_svid not
                  TLS strict +         token verify +        in approved set
                  workload-id          webhook HMAC
                                                              forbid when
Anti-scrape       Rate-limit +         Pattern-anomaly +     scrape_pattern
                  robots authority +   watermark +           detected and
                  paid API tier        rewrite               not partner
```

The three families are **orthogonal** — they defend against
different threats:

- **Anti-bot** defends against automated traffic — high-volume
  scripts, residential proxy farms, ML-driven CAPTCHA solvers
  attempting credential stuffing or content extraction.
- **Anti-spoof** defends against identity falsification — spoofed
  sender domains, replayed session tokens, forged webhooks,
  workload-identity impersonation across the mesh.
- **Anti-scrape** defends against bulk content extraction — even
  by legitimate-looking authenticated clients that crawl beyond
  normal user patterns.

A single adversary may attack via all three vectors simultaneously
(e.g., a credential-stuffing bot that spoofs a forwarded `Cookie`
header while scraping the rate-limit headers to time its attacks).
Defence-in-depth requires all three families to compose. No request
that fails any one family proceeds; many requests that pass each
family individually but fail when their signals combine are also
refused (per the §D-4 Cedar fragment composition).

The three layers are **complementary** — they catch different
adversary tactics:

- **Tier-0 edge** catches volumetric + signature-based attacks
  before they reach the cell. Operating at planetary scale (Cloudflare
  ~300 POPs initial, Pingora migration Year 3+ per ADR-0253 §D-2),
  the edge sees signal across every tenant + every request and
  scores accordingly.
- **Per-µservice** catches behaviour that the edge cannot see —
  e.g., a request that authenticates legitimately at the edge but
  then issues a fan-out crawl beyond tenant-tier quota. The µservice
  sees the tenant's request history; the edge sees only the current
  request.
- **Cedar policy** composes the signals from the edge + the µservice
  + the tenant's audience type + the compliance pack into a single
  permit/forbid decision. Cedar's evaluation is deterministic + signed
  + audited per ADR-0243 + ADR-0263.

### §B.1. Defence-in-depth — no single control gates

The baseline rejects the failure mode of single-control gating:

- A request that the bot-mgmt ML model scores at 96 (high bot
  suspicion) is NOT automatically blocked. It is layered with the
  CAPTCHA challenge (Turnstile invisible-pass for ~95% of human
  traffic); only if the CAPTCHA fails AND the bot-score remains
  elevated AND the request would hit a sensitive endpoint AND the
  tenant's audience type is not `FRIENDLY_CRAWLER_PARTNER` does the
  Cedar gate forbid.
- A request from an IP with no prior reputation is NOT automatically
  blocked. The behavioural fingerprint (JA4 + HTTP/2-3 frame pattern
  + WebGL + cursor trajectory) is observed; if all signals match a
  legitimate browser shape, the request proceeds with elevated
  monitoring.
- A request that fails one of the eight anti-bot controls (e.g.,
  device attestation unavailable because the user is on a desktop
  Linux browser with no WebAuthn key) is NOT blocked solely on that
  basis; the other seven controls compose to a score.

This composition is canonical across hyperscalers: per Cloudflare's
"How Cloudflare's Bot Management algorithm reliably identifies
bots" blog (2022 + 2024 update), the score is the output of ~50
features computed simultaneously, no single feature gates. Per
Google Cloud Armor Adaptive Protection documentation, the L7 DDoS
model emits a `confidence` score; the production gate is a Cedar-
equivalent composition of confidence + rate + behavioural-anomaly.

The principle: **layered scoring + adaptive challenge + Cedar policy
composition.** Defence-in-depth.

### §B.2. The accessibility floor — no default-path CAPTCHA

CAPTCHA on the default path is an accessibility failure. Per WCAG
2.2 Success Criterion 1.1.1 + 2.1.1 + 2.4.7, a website that gates
unauthenticated access behind image-recognition CAPTCHAs excludes
users with visual impairments, motor impairments, and cognitive
impairments. The COPPA / KOSA / EU AADC packs explicitly require
accessibility-compliant abuse defence for minor-targeted surfaces.

The baseline therefore declares: **CAPTCHA presented only when
bot-score crosses threshold; never on default path.** Turnstile +
hCaptcha + Cloudflare Challenge all support invisible-mode
challenges that pass ~95% of legitimate traffic without user
interaction; only suspicious traffic sees an interactive challenge,
and the interactive challenge MUST offer audio + arithmetic +
behavioural alternatives per WCAG.

The `microservices/intelligence/` AI-slop detector (per ADR-0255)
provides a fallback challenge path for users who cannot complete
any conventional CAPTCHA: a low-confidence conversational query
(e.g., "name a fruit that is red") whose answer is graded by the
substrate Intelligence layer, with manual review fallback.

## §C. Consequences

### §C.1. Maintainability dimension

The abuse-defence baseline is the **substrate** that 46 internet-
facing µservices inherit. The maintainability invariants:

- **Per-µservice declaration is configuration, not code.** Each
  µservice declares its abuse-defence posture in `ARCHITECTURE.md
  §abuse-defence` + `iac/<env>-edge-waf.yaml` + `policy/abuse-
  defence.cedar`. The actual primitive implementation lives in the
  shared crate `oya-shared-abuse-defence` + the Tier-0 edge
  configuration (Cloudflare zone config in Year 1-2; Pingora
  configuration Year 3+).
- **Per-tenant tuning is configuration, not code.** Tenant-tier
  sensitivity (HIGH_RISK, B2C_CONSUMER, B2B_TENANT,
  FRIENDLY_CRAWLER_PARTNER) is set via the tenancy substrate's
  control surface per ADR-0244. No code change required to retune.
- **Versioning policy.** The Cedar fragment `policy/abuse-defence.cedar`
  follows ADR-0294 Cedar fragment lifecycle (≥60s soak + signed
  publication + rollback). The IaC manifest follows the µservice's
  ADR-0258 SemVer policy.
- **Deprecation cadence.** Vendor primitives have a 12-month
  deprecation cadence; if Cloudflare Bot Management is replaced by
  Pingora-native bot defence (Year 3+ per ADR-0253), the migration
  follows ADR-0258 deprecation cadence + a per-µservice opt-in
  parallel run.

### §C.2. Observability dimension

Per ADR-0263 observability emission contract, the abuse-defence
baseline emits:

- **Audit-event-classes (registered in ADR-0263 registry):**
  - `AbuseDefenceBotBlocked` — emitted whenever Tier-0 edge or
    µservice Cedar gate forbids a request on bot-score grounds.
    Carries: principal_id, tenant_id, request_id, bot_score,
    fingerprint_hash, blocking_rule_id, suggested_remediation.
  - `AbuseDefenceSpoofDetected` — emitted on detection of any
    anti-spoof control firing. Carries: principal_id (claimed),
    tenant_id, control_id (DMARC / SPF / DKIM / mTLS / session-token
    / webhook-HMAC / SPIFFE), expected_vs_actual digest.
  - `AbuseDefenceScrapeBlocked` — emitted on pattern-anomaly scrape
    blocking. Carries: principal_id, tenant_id, scrape_pattern_id
    (BFS / alphabetical / parallel-tab / depth-overflow), refusal_layer
    (edge / µservice / Cedar), tenant_audience_type.
  - `AbuseDefenceChallengeIssued` — emitted whenever a CAPTCHA or
    proof-of-work challenge is issued; carries challenge_type +
    bot_score + suspected_class.
  - `AbuseDefenceChallengeSolved` / `AbuseDefenceChallengeFailed` —
    emitted on challenge outcome.
- **Metrics (per ADR-0263 cardinality budget — see §D-6 for the
  per-metric dimension list):**
  - `oya_abuse_defence_bot_score_histogram` — distribution of bot
    scores across all requests.
  - `oya_abuse_defence_captcha_challenge_rate` — rate of CAPTCHA
    issuance per unit traffic.
  - `oya_abuse_defence_spoof_detected_count` — counter per spoof
    control.
  - `oya_abuse_defence_scrape_pattern_detected_count` — counter per
    scrape pattern.
  - `oya_abuse_defence_request_blocked_count` — counter per layer
    (edge / µservice / Cedar).
- **Dashboards:** per-µservice dashboard at
  `microservices/observability/dashboards/abuse-defence.json` (flag
  for follow-up authoring in §G); aggregate at
  `dashboards/abuse-defence-fleet.json`.
- **SLO floors:** false-positive rate < 0.1% for legitimate-tenant
  traffic; mean-time-to-detect (MTTD) < 30s for new attack
  signature; mean-time-to-mitigate (MTTM) < 5min for substrate-wide
  rule deployment.

### §C.3. Scalability dimension

The substrate scales horizontally with the planetary edge:

- **Tier-0 edge.** Cloudflare ~300 POPs (Year 1-2) → Pingora ~30
  POPs initial, ~300 POPs Year 7 (per ADR-0253 §D-2). Each POP
  handles ~10k-100k req/sec sustained; planetary capacity ~3M-30M
  req/sec sustained.
- **Bot-management ML model.** Inference per request ~50μs at the
  edge (Cloudflare published 2024 figures); scales linearly with POP
  count. Re-training daily on planetary-scale signal.
- **Cedar evaluation.** Per-µservice Cedar evaluator runs in process
  at ≤500μs per evaluation (per ADR-0246 library-first dispatch);
  scales horizontally with µservice replica count.
- **Capacity math (Little's Law):** average concurrent in-flight
  Cedar abuse-defence evaluations = arrival_rate × service_time =
  1M req/sec × 0.5ms = 500 concurrent. Each µservice replica handles
  ~10k concurrent; the substrate at planetary scale needs ~50 replicas
  for Cedar evaluation alone — well within the 1000-replica per-cell
  ceiling per ADR-0248.

### §C.4. Performance dimension

Latency budgets for the abuse-defence layer:

- **Tier-0 edge bot-score computation:** P50 ≤ 0.5ms, P95 ≤ 1ms,
  P99 ≤ 2ms (Cloudflare published 2024 figures; Pingora target Year
  3+ matches).
- **TLS JA4 fingerprint extraction:** P50 ≤ 50μs, P99 ≤ 200μs
  (FoxIO published 2024 figures).
- **Cedar abuse-defence evaluation:** P50 ≤ 200μs, P95 ≤ 500μs,
  P99 ≤ 1ms (per ADR-0246 library-first dispatch).
- **End-to-end abuse-defence overhead:** P50 ≤ 1ms, P95 ≤ 3ms,
  P99 ≤ 5ms (composing the above + the per-µservice quota check).
- **CAPTCHA challenge round-trip (when issued):** P50 ≤ 200ms,
  P95 ≤ 500ms, P99 ≤ 1s. Most requests do not see this path.

Tail-latency mitigation: hedged execution of the bot-management
inference across two POP replicas (Cloudflare published 2024
pattern); circuit-breaker on Cedar evaluation timeout (fail-closed
default per ADR-0243 default-deny baseline; emergency fail-open
permitted only via the kill-switch fragment per ADR-0294 + ADR-0295).
Cold-start budget: zero — the edge POP is long-lived; Cedar
evaluator is library-loaded once per µservice startup.

### §C.5. Optimization dimension

Cost-performance frontier:

- **Per-call cost model.** Cloudflare Bot Management licensed at
  ~$0.0005 per request (2024 figures); Pingora self-host at Year 3+
  drops to ~$0.0001 per request (~80% reduction per ADR-0211
  in-house preference rationale).
- **Lazy vs eager.** Bot-score computation is **eager** (always
  computed for internet-facing surface); CAPTCHA issuance is **lazy**
  (only when score crosses threshold); device-attestation
  verification is **lazy** (only when the surface explicitly requires
  it).
- **Cache-invalidation policy.** Bot-score is computed per-request;
  no caching. CAPTCHA solve token is cached per-session (~30min
  TTL); device-attestation token cached per-device (~24h TTL).
- **Cold-vs-warm path latency separation.** First request from new
  IP sees full bot-score + fingerprint extraction (~5ms); subsequent
  requests in same session see cached score (~0.5ms).
- **Profiling evidence.** See `microservices/observability/dashboards/
  abuse-defence-perf.json` for production-trace breakdown
  (flag for follow-up authoring in §G).

### §C.6. Code quality dimension

Required test classes for the `oya-shared-abuse-defence` crate:

- **Unit tests:** ≥85% line coverage, ≥75% branch coverage. Cover
  bot-score parsing, JA4 fingerprint extraction, Cedar evaluator
  hook, audit-event emission.
- **Property tests:** Cedar fragment evaluation is deterministic;
  same inputs always produce same outputs.
- **Fuzz tests:** Bot-score header parsing fuzz-tested against
  malformed inputs (per ADR-0211 in-house preference for `cargo fuzz`).
- **Integration tests:** End-to-end test of edge → µservice → Cedar
  flow against a recorded adversary corpus.
- **Load tests:** Sustained 100k req/sec against the µservice
  Cedar evaluator; verify P99 < 1ms.
- **e2e tests:** Per-µservice browser-driven test against a recorded
  legitimate-user session + a recorded bot-script session; verify
  legitimate passes + bot blocked.

Lint passes named: `oya-check-naming`, `oya-check-layer-conformance`,
`oya-check-cedar-fragment-format`, `oya-check-audit-event-class-registered`.
Type-strictness: Rust `deny(warnings)` + `deny(unsafe_code)` per
ADR-0211. SemVer + ABI policy per ADR-0258.

## §D. Detailed mechanics

### §D-1. Anti-bot taxonomy — eight controls fully expanded

The eight-row anti-bot taxonomy from `documentation-rigor.md`
§3.2.3, with every row fully mechanically specified:

#### §D-1.1. Edge rate-limiting (per-IP, per-fingerprint, per-tenant, per-route)

**Layer:** Tier-0 edge.

**Mechanism:** Token-bucket per dimension + sliding-window per
dimension; burst caps per route class.

The edge maintains four parallel rate-limit buckets:

| Dimension | Bucket key | Default cap | Override |
|---|---|---|---|
| Per-IP | `ratelimit:ip:<sha256(ip)>` | 100 req/sec sustained, 200 burst | Per-IP allow-list for known bots (see §D-3 paid API tier) |
| Per-fingerprint | `ratelimit:fp:<ja4_hash>` | 50 req/sec sustained, 100 burst | Tighter for known-bot JA4s |
| Per-tenant | `ratelimit:tenant:<tenant_id>` | 10k req/sec sustained (B2B_TENANT), 1k (B2C_CONSUMER) | Per-tenant tier from Tenancy substrate |
| Per-route | `ratelimit:tenant:<tenant_id>:<route>` | Per-route class | `auth:10 req/sec, write:100 req/sec, read:1000 req/sec, admin:5 req/sec` |

**Route class taxonomy:**
- `auth` — POST /v1/auth/*, POST /v1/login, POST /v1/oauth/token,
  POST /v1/passkey/*. Aggressive: 10 req/sec sustained, 20 burst
  per tenant. Per-IP cap 1 req/sec sustained, 5 burst. Justification:
  credential stuffing pressures auth surfaces; legitimate users
  rarely exceed 1 req/sec on these endpoints.
- `write` — POST/PUT/PATCH/DELETE to data endpoints. Moderate: 100
  req/sec sustained, 200 burst per tenant. Per-IP cap 50 req/sec.
- `read` — GET to data endpoints. Permissive: 1000 req/sec sustained
  per tenant. Per-IP cap 200 req/sec. Note that the anti-scrape
  family (§D-3) tightens read further when scrape-pattern detected.
- `admin` — Tenant admin / oyatie staff-admin / billing surfaces.
  Tight: 5 req/sec sustained, 10 burst. Per-IP cap 1 req/sec.

**Algorithm:** Sliding-window log per Cloudflare's published
algorithm (Cloudflare Workers Durable Objects RateLimiter, 2024);
token-bucket for the per-route classes. The sliding window stores
timestamps; an arriving request's timestamp is compared against the
window's count.

**Storage:** Cloudflare Workers KV at the edge (Year 1-2); Pingora
in-process LRU + Redis-backed at-Year-3+. Per-IP bucket TTL = 1 min;
per-tenant TTL = 1 hour; per-fingerprint TTL = 5 min.

**Hyperscaler precedent:**
- AWS WAF Rate-Based Rules (`RateBasedStatement`, max 20M req per
  5min window per rule).
- Cloudflare Rate Limiting (per-zone, per-rule; documented at
  developers.cloudflare.com/waf/rate-limiting-rules/).
- Google Cloud Armor rate-based ban rules (per `cloud.google.com/armor/docs/rate-limiting-overview`).

**Failure modes:**
- **Network partition:** Edge can't reach KV/Redis. Behaviour: fail
  conservative — fall back to in-process per-instance counters with
  10× tighter caps; emit `AbuseDefenceRateLimitFallback` audit row.
- **Byzantine actor:** Attacker rotates IPs faster than per-IP TTL.
  Per-tenant + per-fingerprint buckets catch this; tenant-bound
  quotas survive IP rotation.
- **Regional outage:** Per-POP buckets are local; outage of one POP
  does not affect another POP's counters. Edge GeoDNS routes around.

**Rollback path:** Per-route rate-limit values are configuration
in `iac/<env>-edge-waf.yaml`; rollback is a single revert + push,
with re-distribution to edge POPs in ≤60s. Cedar-fragment-style
soak per ADR-0294 applies.

**Observability emission:** `AbuseDefenceRateLimitHit` audit class;
`oya_abuse_defence_ratelimit_hit_count` counter with dimensions
{layer, dimension, route_class, tenant_id_class}; cardinality
budget per ADR-0263 §D-N (≤10k unique dimension combos per
µservice per day).

#### §D-1.2. Behavioural fingerprinting — TLS JA4 / JA4+ / HTTP frame-pattern

**Layer:** Edge (passive observation).

**Mechanism:** Per FoxIO's TLS JA4 specification (released 2023;
`github.com/FoxIO-LLC/ja4`), every TLS connection is fingerprinted
from the ClientHello's cipher list + extension list + signature
algorithms + ALPN + supported_versions. The fingerprint is a
deterministic hash of 30 fields; collision-free across legitimate
browser fingerprints; trivially separates browser fingerprints from
script/library fingerprints.

**JA4 specification (FoxIO 2023):**
- Format: `t<proto><tls_version><ciphers_count><exts_count>_<sni>_<alpn>_<ciphers_hash>_<exts_hash>`
  - Example: `t13d1516h2_8daaf6152771_b186095e22b6` —
    `t13` = TLS 1.3, `d` = encrypted (vs `i` undeterminate),
    `15` = 15 ciphers, `16` = 16 extensions, etc.
- The full fingerprint hashes via a deterministic-order SHA256.

**JA4+ extensions (FoxIO 2024):**
- `JA4S` — server-side fingerprint (from ServerHello).
- `JA4H` — HTTP/2 fingerprint from frame patterns + header order.
- `JA4L` — latency-based fingerprint (round-trip timing).
- `JA4X` — X.509 certificate fingerprint.
- `JA4T` — TCP fingerprint (window size, MSS, options).

**HTTP/2-3 frame-pattern fingerprint:** Per Akamai's "HTTP/2
fingerprinting" research (2020-2023), the order of HEADERS/SETTINGS/
WINDOW_UPDATE/PRIORITY frames + SETTINGS values + frame sizes
distinguishes browser implementations (Chrome / Firefox / Safari)
from scripted clients (curl, Go net/http, Python requests, Node fetch).

**Passive observation only — never alone gates a request:** The
fingerprint is added to the request context as `X-Oya-JA4` header
+ `X-Oya-JA4H` header (forwarded to µservice); the request proceeds
unless other signals compose with the fingerprint to a forbid
decision via Cedar.

**Hyperscaler precedent:**
- Akamai Bot Manager uses HTTP/2 fingerprinting + TLS fingerprinting
  as core signals (per their 2023 white paper).
- Cloudflare Bot Management documents JA3 + JA4 + HTTP/2 fingerprint
  as features (Cloudflare blog "JA4 and JA4S Now in Cloudflare", 2024).
- Google Cloud Armor uses TLS fingerprinting in Adaptive Protection.

**Failure modes:**
- **Fingerprint collision (legitimate user mis-fingerprinted as
  known-bot):** The signal is composed with bot-score, not gating
  alone. False-positive rate < 0.01% per Cloudflare's published
  figures.
- **Adversary spoofs browser fingerprint:** Per FoxIO's research,
  spoofing JA4 requires modifying the TLS library — possible for
  curl-impersonate (`github.com/lwthiker/curl-impersonate`) but
  expensive to keep up to date as browsers ship new TLS extensions.
  The fingerprint stays current with browser shipping cadence (~6
  weeks); adversaries must re-spoof every shipping.

**Rollback path:** N/A — passive observation only.

**Observability emission:** `oya_abuse_defence_ja4_distribution`
histogram (dimensions: tenant_id_class, route_class).

#### §D-1.3. Bot-management with ML scoring

**Layer:** Edge (Cloudflare Bot Management Year 1-2; Pingora-
native + in-house ML Year 3+).

**Mechanism:** A gradient-boosted decision tree (GBDT) over ~50
features computes a bot-score in [0, 99]. The score is forwarded
to the µservice as request header `X-Oya-Bot-Score: <N>`.

**Feature set (~50 features):**
- TLS JA4 fingerprint (§D-1.2).
- HTTP/2-3 frame pattern (§D-1.2).
- IP reputation (ASN classification, residential vs datacenter,
  TOR/VPN/proxy lists).
- Request rate (per-IP, per-fingerprint, per-tenant — see §D-1.1).
- Request shape (HTTP method, path entropy, header order, header
  values).
- Browser-fingerprint signals (User-Agent string, Accept-Language,
  Accept-Encoding, sec-ch-* hints).
- Behavioural signals (cursor trajectory, scroll cadence, dwell
  time — JavaScript-emitted; only available on browser surfaces).
- Session signals (length of session, request diversity, login state).
- Reputation history (per-fingerprint historical bot score, per-IP
  historical bot score, per-tenant historical bot score).
- Network signals (TCP window size, MSS, IP fragmentation pattern).
- Geo signals (geo IP, distance-from-tenant-cell, sovereign-pack
  conformance).

**Behavioural LSTM (sequence model):** An LSTM model trained on
sequences of (request_time, request_path, request_method, ...)
classifies sequence patterns — e.g., a sequence that looks like
"crawl: alphabetical pagination through /users/A → /users/B → ..."
gets elevated bot-score even if individual requests look benign.

**Model training:** Daily re-training on planetary-scale labeled
corpus (Cloudflare Year 1-2 trains on Cloudflare-wide signal;
oyatie Year 3+ trains on planetary-aggregated oyatie signal with
differential-privacy guarantees per ADR-0099 data-class registry).

**Score thresholding:**
- Score `< 30`: human or legitimate bot. Request proceeds.
- Score `30-60`: ambiguous. Invisible Turnstile challenge issued;
  if passes, request proceeds.
- Score `60-90`: likely bot. Interactive Turnstile challenge; if
  passes, request proceeds with elevated monitoring.
- Score `90-99`: certain bot. Forbid via Cedar unless tenant
  audience type is `FRIENDLY_CRAWLER_PARTNER`. Emit
  `AbuseDefenceBotBlocked`.

**Forwarded header:** `X-Oya-Bot-Score: <N>` (0-99). Downstream
µservice Cedar evaluator consumes the score as
`principal.bot_score`.

**Hyperscaler precedent:**
- Cloudflare Bot Management (since 2017; ML-driven since 2019; ~50
  features per their 2024 blog).
- AWS WAF Bot Control (since 2021; Common + Target inspection levels
  in 2024 update).
- Akamai Bot Manager (since 2014; LSTM-based behavioural model in
  2023 release).
- Google Cloud Armor Adaptive Protection (since 2021; ML-driven
  L7 DDoS).

**Failure modes:**
- **Model drift:** Production traffic shifts and false-positive rate
  rises. Mitigation: daily retraining + per-tenant FP rate SLO
  ≤ 0.1% with auto-alerting at 0.05% breach.
- **Adversarial ML attack:** Attacker poisons the training corpus.
  Mitigation: training corpus is verified-only (signed by
  `oyatie.foundry.ml-training-pipeline`); per-feature outlier
  detection rejects poisoned features before training.
- **Vendor outage (Cloudflare):** Bot Management API unavailable.
  Fallback: forward `X-Oya-Bot-Score: 50` (neutral); revert to
  Cedar evaluation with rate-limiting + JA4 fingerprinting alone;
  emit `AbuseDefenceVendorOutage` audit row.

**Rollback path:** Model rollback via Pingora config flag or
Cloudflare zone setting; revert to previous-day's model in ≤60s
distribution across edge POPs.

**Observability emission:** `oya_abuse_defence_bot_score_histogram`
{tenant_id_class, route_class, audience_type}; `oya_abuse_defence_
bot_score_distribution_percentile` (P50/P95/P99 per dimension).

#### §D-1.4. CAPTCHA-on-suspicion (composition stack)

**Layer:** Edge (Cloudflare Turnstile + hCaptcha + Cloudflare
Challenge).

**Mechanism:** When bot-score crosses threshold (§D-1.3), the edge
issues a challenge from a composition stack:

| Challenge type | When issued | Round-trip | Accessibility |
|---|---|---|---|
| **Cloudflare Turnstile (invisible)** | bot_score 30-60 | Invisible (no UI) | Passes ~95% of legitimate browser traffic without interaction |
| **Cloudflare Turnstile (managed)** | bot_score 60-80 | One click | WCAG-compliant; arithmetic + audio alternatives |
| **hCaptcha (interactive)** | bot_score 80-90 (fallback) | Image-grid | WCAG-compliant; audio alternative; accessibility cookie for assistive-tech users |
| **Cloudflare Challenge (JS proof-of-work)** | bot_score 80-90 (fallback) | 3-5s compute | Headless-browser detection |
| **Intelligence-mediated CSAQ** | Accessibility opt-in | Conversation | LLM-graded conversational query (per §B.2 accessibility floor) |

**Composition:** Per Cloudflare's "Bot Management Challenge"
documentation, the challenge stack is layered — the cheapest
challenge (invisible Turnstile) is tried first; if that fails,
escalate to Turnstile managed; if that fails, escalate to hCaptcha
or JS proof-of-work. The Cedar fragment composes the challenge-
solved attribute into the permit decision.

**Never on default path:** Per §B.2 accessibility floor, CAPTCHA
on the default path is forbidden. The edge presents the challenge
only when bot-score crosses the §D-1.3 threshold.

**Hyperscaler precedent:**
- Cloudflare Turnstile (released 2022 GA; replaces reCAPTCHA at
  Cloudflare scale; invisible-mode passes ~95% legitimate traffic).
- hCaptcha (since 2017; primary accessibility-compliant alternative;
  WCAG 2.1 AA tested).
- Google reCAPTCHA Enterprise v3 (since 2018; risk score 0.0-1.0;
  Google Cloud Armor uses this).

**Failure modes:**
- **Adversary solves Turnstile via AI vision:** Per recent
  Anthropic + OpenAI + Google vision benchmarks, ~85-92% solve
  rate on image-recognition. Mitigation: invisible Turnstile is
  not image-based (browser-environment fingerprint instead);
  managed Turnstile uses behavioural signals over images. AI
  solvers struggle with these.
- **Legitimate user cannot solve CAPTCHA:** Accessibility fallback
  via Intelligence-mediated CSAQ (§B.2).
- **Vendor outage (Cloudflare Turnstile):** Fallback to hCaptcha;
  emit `AbuseDefenceChallengeVendorFailover` audit row.

**Rollback path:** Challenge-issuance threshold is configuration in
`iac/<env>-edge-waf.yaml`; rollback per ADR-0294 ≥60s soak.

**Observability emission:** `AbuseDefenceChallengeIssued` audit class;
`oya_abuse_defence_challenge_issued_count` {challenge_type,
bot_score_band}; `AbuseDefenceChallengeSolved` /
`AbuseDefenceChallengeFailed` audit classes.

#### §D-1.5. Device attestation (App Attest + Play Integrity + WebAuthn)

**Layer:** Edge + native app + web client.

**Mechanism:** Native + web clients prove device authenticity to
the edge via platform attestation APIs.

**iOS — Apple App Attest (since iOS 14, 2020):**
- Native app calls `DCAppAttestService.attestKey` to obtain an
  attestation object signed by Apple's hardware-rooted attestation
  certificate.
- Backend verifies the attestation per Apple's published
  attestation chain validation (Apple's "Validating Apps That
  to Your Server" developer documentation).
- Per-request, native app calls `DCAppAttestService.generateAssertion`
  to sign the request; backend verifies against the previously-
  attested key.

**Android — Google Play Integrity API (since 2022, replacing
SafetyNet):**
- Native app calls `IntegrityManager.requestIntegrityToken` to
  obtain an integrity verdict.
- Backend decrypts the verdict and validates the device-integrity +
  app-integrity + account-integrity claims.
- Per Google Play Integrity API documentation (developer.android.
  com/google/play/integrity).

**Web — WebAuthn Origin-binding (W3C Recommendation, 2021):**
- Web app uses WebAuthn with `authenticatorAttachment: platform`
  to bind a passkey to the device (per ADR-0188 passkey-webauthn-
  as-canonical-auth).
- Origin-binding prevents passkey use on phishing origins; the
  passkey signs the request origin.

**Forwarded header:** `X-Oya-Device-Attestation: <jwt>` (signed
JWT bearing the platform-attestation claims). Downstream Cedar
evaluator consumes as `principal.device_attestation_verified`.

**Hyperscaler precedent:**
- Apple's App Attest is the canonical iOS attestation; documented
  in Apple's "Establishing Your App's Integrity" series.
- Google Play Integrity replaces SafetyNet Attestation API
  (deprecated 2024) for Android attestation.
- WebAuthn is W3C / FIDO Alliance standard; Microsoft + Apple +
  Google + Mozilla all implement.

**Failure modes:**
- **Device lacks attestation (older OS):** Fallback to lower-tier
  trust (request proceeds but elevated bot-score weighting); user
  may be prompted to upgrade OS.
- **App tampered (rooted phone, Magisk module):** Attestation fails;
  Cedar forbids high-risk operations; allows read-only legitimate
  use (e.g., login attempt blocked, public-content view permitted).
- **WebAuthn key lost:** Per ADR-0188 passkey recovery flow;
  recovery passkey enrolled on another device.

**Rollback path:** Attestation verification can be disabled
per-route via `iac/<env>-edge-waf.yaml` for emergency cases (e.g.,
Apple App Attest service outage); emit
`AbuseDefenceAttestationDisabled` audit row.

**Observability emission:**
`oya_abuse_defence_device_attestation_verified_rate`;
`AbuseDefenceAttestationFailed` audit class.

#### §D-1.6. Stolen-credential check (HIBP-style)

**Layer:** Auth path (`microservices/identity/`).

**Mechanism:** On every password submission (login, registration,
password change), the password is checked against the HaveIBeenPwned
(HIBP) k-anonymity API + oyatie's internal credential-stuffing
detector.

**HIBP k-anonymity API (per HIBP 2018 specification):**
- Client/edge computes `sha1(password)` locally.
- Sends first 5 hex chars of SHA1 to HIBP API
  (api.pwnedpasswords.com/range/<prefix>).
- API returns all SHA1 suffixes that share that prefix + their
  observed count in breach corpora.
- Client checks if the local SHA1's suffix matches.
- The password itself never leaves the client/edge; HIBP never sees
  full hash.

**Internal credential-stuffing detector:**
- Tracks observed `email:password_hash` pairs from prior failed
  login attempts.
- Detects patterns where the same `password_hash` is tried across
  many `email` accounts (credential stuffing) or the same `email` is
  tried with many `password_hash` (brute force).
- Tracks observed `email:password_hash` pairs from any login attempt
  that matches a HIBP-flagged combination.

**Action on detection:**
- **HIBP-found password (count ≥ 1):** Pause sign-in; require user
  to change password before completing login. Forbid registration
  with the password. Audit `AbuseDefenceCredentialPwned`.
- **Credential-stuffing pattern (same hash across many emails):**
  Step-up auth required on the matched accounts (passkey + email
  verification + maybe SMS+TOTP). Audit `AbuseDefenceCredentialStuffing`.
- **Brute-force pattern:** Per-account exponential backoff (1s,
  10s, 60s, 300s) + IP-level rate-limit reduction.

**Hyperscaler precedent:**
- Have I Been Pwned (HIBP) is the canonical breach corpus + k-
  anonymity API; cited by NIST SP 800-63B Section 5.1.1 as the
  password-strength reference.
- Google's Password Checkup extension uses k-anonymity similarly
  (Bonneau et al., USENIX Security 2019, "Protecting accounts from
  credential stuffing with password breach alerting").
- Microsoft's Azure AD Identity Protection uses internal credential-
  stuffing detection.

**Failure modes:**
- **HIBP API outage:** Fallback to internal corpus only; emit
  `AbuseDefenceHIBPUnavailable`; tighten per-IP auth-path rate
  limits.
- **False-positive (legitimate user's strong password coincidentally
  matches HIBP suffix):** Very rare; user-friendly remediation flow
  guides password change.
- **K-anonymity bucket size attack:** Adversary inspects HIBP
  response to enumerate likely passwords. Mitigation: HIBP responds
  with ~470 suffixes per 5-char prefix on average; bucket sizes
  large enough for k-anonymity.

**Rollback path:** Stolen-credential check can be downgraded from
"forbid registration" to "warn user" per-tenant via tenancy
substrate's abuse-defence sensitivity control.

**Observability emission:**
`oya_abuse_defence_stolen_credential_check_count`;
`AbuseDefenceCredentialPwned` + `AbuseDefenceCredentialStuffing`
audit classes.

#### §D-1.7. Per-action quota gates (Cedar-evaluated)

**Layer:** µservice (Cedar evaluation).

**Mechanism:** Per ADR-0246 library-first dispatch, every action
on every µservice resource is gated by Cedar. The abuse-defence
fragment adds a quota predicate composed with the bot-score and
the tenant audience type.

**Per-action quota predicates:**
- `principal.bot_score > N` ⇒ scale per-action quota inversely
  (higher bot-score → lower quota).
- `principal.tenant.audience_type == "FRIENDLY_CRAWLER_PARTNER"`
  ⇒ quota allows higher bot-score thresholds.
- `principal.request_rate_per_minute > resource.rate_limit_per_minute`
  ⇒ forbid.
- `principal.tenant.tier == "HIGH_RISK"` ⇒ quotas tighter (financial
  / health surfaces).

**Bot-score composed with quota:**
The substrate maintains tenant-bound quotas — quotas persist across
IP rotation. An adversary cannot escape per-tenant quota by rotating
residential proxies because the quota key is `tenant_id`, not `ip`.

**Cedar fragment shape (excerpt from §D-4 below):**
```cedar
forbid (principal, action, resource) when {
    principal.bot_score > 95
    && !(principal.tenant.audience_type == "FRIENDLY_CRAWLER_PARTNER")
};
```

**Hyperscaler precedent:**
- AWS API Gateway throttling (per-API + per-method) is the AWS
  canonical equivalent.
- Stripe API rate limits are documented per-route at
  stripe.com/docs/rate-limits.
- Cedar policy authorization is canonical per AWS Verified Permissions
  (Cedar v4.2 LTS).

**Failure modes:**
- **Cedar evaluator latency spike:** Per ADR-0246 library-first
  dispatch, evaluation is in-process; circuit-breaker on > 1ms
  evaluation latency fails closed per default-deny.
- **Quota counter inconsistency across replicas:** Quotas are
  eventually consistent; per-replica counter drift bounded by ≤10%.
  Hard quota cap is across-replica aggregate (Redis-backed).

**Rollback path:** Quota values configured per-tenant in tenancy
substrate; rollback via tenancy admin UI + 60s propagation.

**Observability emission:**
`oya_abuse_defence_quota_check_count`;
`AbuseDefenceQuotaExceeded` audit class.

#### §D-1.8. Honeypot routes + canary payloads

**Layer:** µservice.

**Mechanism:** Routes that no legitimate client should hit; canary
payloads (fake API keys, fake user-ids) seeded into surfaces to
detect scrapers that ingest them.

**Honeypot routes:**
- Routes hidden from public OpenAPI spec but discoverable by
  enumeration (e.g., `/v1/admin/_unstable_dashboard`,
  `/v1/internal/debug/dump`).
- Any traffic to a honeypot route → immediate forbid + tenant-
  level alert + bot-score elevated.
- Honeypot routes return plausible-looking but fake responses to
  delay attacker detection.

**Canary payloads:**
- Fake API keys (e.g., `oya_test_canary_DO_NOT_USE_XXXXX`) seeded
  into responses to suspected scraper sessions; any use of the
  canary key in subsequent requests → certain bot detection.
- Fake user-ids / tenant-ids seeded into scrape responses; any
  request referencing the canary id → certain scraper.
- Per-session canary uniqueness (canary tagged with session-id +
  timestamp + per-target watermark) — when canary surfaces in
  leaked corpora, the leak source is identifiable.

**Hyperscaler precedent:**
- AWS's GuardDuty uses honeypot S3 buckets + canary credentials
  to detect leaked AWS access keys (per AWS blog "Continuous
  monitoring with Amazon GuardDuty").
- Thinkst Canary tokens (canarytokens.org) — industry-standard
  canary primitive used by many platforms.
- Cloudflare zone-level honeypot rules detect scraper enumeration.

**Failure modes:**
- **Legitimate developer trips honeypot route:** Tenant-level alert
  reviewed; tenant audience type cross-checked; if developer
  registered via legitimate channel, false-positive remediation flow.
- **Canary token never recovered:** Implies attacker did not parse
  the canary; not a false negative (canary is best-effort detection,
  not a hard gate).

**Rollback path:** Honeypot routes can be removed in subsequent
release; canary tokens have natural expiry (~30 days).

**Observability emission:**
`AbuseDefenceHoneypotHit` audit class with extreme priority alert.
`AbuseDefenceCanaryRecovered` audit class.

### §D-2. Anti-spoof taxonomy — eight controls fully expanded

The eight-row anti-spoof taxonomy:

#### §D-2.1. Email anti-spoof (DKIM + SPF + DMARC + ARC + BIMI)

**Layer:** Domain (DNS) + outbound mail + inbound mail.

**Mechanism (cross-references ADR-0273 per-tenant deliverability):**

**DKIM (DomainKeys Identified Mail, RFC 6376):**
- Per ADR-0273, each tenant gets a tenant-scoped DKIM signing key
  (Ed25519 + RSA-2048 dual-signature); selector
  `<tenant_id>._domainkey.<tenant_domain>`.
- Every outbound mail is DKIM-signed with the tenant's key;
  body-hash + header-hash signed.

**SPF (Sender Policy Framework, RFC 7208):**
- Per-tenant SPF record published at `_spf.<tenant_domain>`;
  declares the oyatie outbound mail edge IPs.
- Receivers verify the envelope sender IP against the SPF record.

**DMARC (Domain-based Message Authentication, Reporting, and
Conformance, RFC 7489):**
- Per-tenant DMARC record published at `_dmarc.<tenant_domain>`.
- Rollout: starts in `p=none` (observation-only) for 30 days
  collecting reports; promotes to `p=quarantine` for 30 days;
  promotes to `p=reject` after.
- `rua=mailto:dmarc-reports@<tenant_domain>` ingested by ADR-0273's
  reporting pipeline.

**ARC (Authenticated Received Chain, RFC 8617):**
- Each forwarder (e.g., mailing list, group alias) adds an
  ARC-Authentication-Results + ARC-Message-Signature + ARC-Seal
  header.
- Receivers use the ARC chain to authenticate the original-sender's
  DKIM result even when SPF would fail due to forwarding.

**BIMI (Brand Indicators for Message Identification, draft RFC):**
- Per-tenant BIMI record published at
  `default._bimi.<tenant_domain>`; declares URL of tenant's
  brand-logo SVG signed by a VMC (Verified Mark Certificate).
- Receivers (Gmail, Yahoo, Apple Mail) display the brand logo
  next to the message when DMARC `p=reject` + VMC verified.

**Inbound DMARC enforcement (oyatie as receiver):**
- Oyatie's inbound mail edge (`microservices/mail/` per ADR-0273)
  enforces DMARC `p=reject` for inbound mail; messages failing DMARC
  + ARC chain are rejected with `550 5.7.26 DMARC fail`.

**Hyperscaler precedent:**
- Google's Postmaster requirements (2024) mandate DMARC `p=reject`
  for senders > 5k messages/day to Gmail users.
- Yahoo's 2024 sender requirements parallel Google's.
- Apple Mail's BIMI display launched 2023.
- AWS SES + SendGrid + Postmark all support per-customer DKIM +
  SPF + DMARC.

**Failure modes:**
- **Tenant DKIM key compromise:** Per ADR-0273 rotation procedure;
  emergency key rotation in ≤1h.
- **DMARC report flood:** Per ADR-0273 reporting pipeline; reports
  rate-limited per-aggregator.
- **Receiver disagrees with DMARC reject (rare false-positive):**
  Per `rua` ingest, oyatie's DMARC report processor identifies the
  receiver + adjusts SPF/DKIM if needed.

**Rollback path:** DMARC promotion from `none` → `quarantine` →
`reject` is gated by 30-day observation; can roll back to previous
level per ADR-0294 ≥60s soak (DNS TTL bounded).

**Observability emission:**
`oya_mail_dmarc_check_count` {result: pass/fail/none};
`AbuseDefenceSpoofDetected` audit class with control_id="DMARC".

#### §D-2.2. Domain anti-spoof / cert pinning (TLS strict)

**Layer:** TLS.

**Mechanism:** Strict TLS 1.3 per ADR-0253 amendment.

**TLS strict profile (per ADR-0253 row 12):**
- TLS 1.3 floor; TLS 1.2 explicitly rejected; SSLv3 + TLS 1.0/1.1
  forbidden absolutely.
- Cipher suites limited to AEAD-only: `TLS_AES_128_GCM_SHA256`,
  `TLS_AES_256_GCM_SHA384`, `TLS_CHACHA20_POLY1305_SHA256`.
- Curve preferences: `X25519` + `secp256r1` + `secp384r1`;
  Ed25519 preferred for new certs.
- HSTS: `max-age=63072000; includeSubDomains; preload` (per
  ADR-0253).
- HSTS preload list: oyatie domains submitted to
  `hstspreload.org`.
- Certificate transparency: `expect-ct` header + CT log monitoring
  via `crt.sh` + `Google Certificate Transparency Reports`.
- OCSP stapling: enabled for all certs; soft-fail fallback per
  RFC 8954.
- No MITM-bypass headers (no `strict-transport-security: max-age=0`
  emergency disable in production).
- No `insecure_skip_verify` anywhere in code; per ADR-0253
  amendment `cloud-ci/Rust gate packet tls13-only` enforces.
- ECH (Encrypted Client Hello) advertised per ADR-0253; configured
  via `iac/<env>-ech-config.yaml`.
- PQC hybrid `X25519MLKEM768` offered in ClientHello per ADR-0253.

**Cert pinning (native apps):**
- Per ADR-0188 + ADR-0253, native iOS + Android apps pin the
  oyatie root CA chain (cosign-attested per ADR-0247 §D-4) in the
  app bundle; runtime cert validation requires a chain rooted in
  the pinned root.
- Pin rotation: 12-month rolling pin set (current + next-pin),
  enabling smooth rotation without app-update.

**DNS-over-HTTPS / DoT for resolver path:**
- Native apps + web clients prefer DoH (DNS-over-HTTPS, RFC 8484)
  to oyatie's recursive resolver or to Cloudflare 1.1.1.1 +
  Google 8.8.8.8 as fallback.

**Hyperscaler precedent:**
- Apple's App Transport Security (ATS) enforces TLS 1.3-strict for
  iOS apps.
- Google's Network Security Configuration enforces strict TLS for
  Android.
- Cloudflare's TLS 1.3-only mode + HSTS preload + ECH (Cloudflare
  rolled out ECH GA 2024).
- AWS s2n-tls is FIPS 140-3 validated + PQC hybrid (per AWS blog
  2024).

**Failure modes:**
- **Cert mis-issuance (root CA breach):** Per ADR-0247 + meta-trust-
  root per ADR-0293, root-key compromise triggers Shamir M-of-N
  reconstitution ceremony.
- **CT log not updated (rare):** OCSP stapling + Cert Transparency
  monitoring catch the gap.
- **HSTS preload not enforced (very old browser):** Browser pre-
  HSTS gets standard TLS protection; no graceful degradation to
  HTTP — HTTP is unconditionally redirected to HTTPS.

**Rollback path:** TLS strict profile cannot be downgraded
without ADR amendment + multispectrum review per ADR-0253.

**Observability emission:**
`oya_tls_handshake_result_count` {result: success/fail, profile:
strict/legacy}; `AbuseDefenceSpoofDetected` with
control_id="TLS_DOWNGRADE".

#### §D-2.3. Identity anti-spoof (step-up auth classes)

**Layer:** Auth (`microservices/identity/`).

**Mechanism:** Per `docs/standards/step-up-auth-classes.md` + ADR-0188
passkey-webauthn-as-canonical-auth.

**Step-up auth classes (per the standards doc):**
- **Class A (no step-up):** Read public content; session cookie
  sufficient.
- **Class B (low step-up):** Read own data; refreshed session
  cookie ≤ 1h old.
- **Class C (medium step-up):** Write own data; passkey within
  ≤ 15min of authentication.
- **Class D (high step-up):** Write sensitive data (financial,
  health, payment-method addition); fresh passkey signature ≤ 1min
  + phishing-resistant MFA.
- **Class E (admin step-up):** Tenant admin operations + delete-
  account-or-data + cross-tenant share; Class D + email confirmation
  link clicked from registered email.
- **Class F (recovery):** Account recovery; requires multiple
  signed devices (M-of-N WebAuthn) + identity-proof per ADR-0188
  recovery flow.

**Phishing-resistance:** WebAuthn passkeys are phishing-resistant
because the passkey signs the request origin; phishing origins
(typo-squatted domains) cannot solicit a valid passkey signature.
TOTP + SMS are NOT phishing-resistant and are downgraded paths
only.

**Forwarded context:** Cedar evaluator receives
`principal.auth_class` (A/B/C/D/E/F) + `principal.auth_age_seconds`
+ `principal.auth_phishing_resistant` (boolean).

**Hyperscaler precedent:**
- Apple's iCloud + ID step-up flow (passkey + device-trust + email
  + SMS combination).
- Google's Advanced Protection Program (passkey-only for high-risk
  accounts).
- Microsoft's Entra Conditional Access (per-action step-up policy).
- Stripe's elevated authentication for sensitive dashboard actions
  (per Stripe Dashboard 2FA + WebAuthn for sensitive operations).

**Failure modes:**
- **User loses primary passkey:** Recovery flow via Class F
  (M-of-N WebAuthn).
- **User on device without WebAuthn (very old browser):** Downgrade
  to TOTP + email (phishing-resistance reduced; high-risk actions
  forbidden).
- **Step-up loop (user keeps being asked to step-up):** Class
  threshold + auth_age tuned per-tenant via tenancy substrate.

**Rollback path:** Step-up policy is Cedar fragment per ADR-0243;
rollback per ADR-0294.

**Observability emission:**
`oya_identity_step_up_class_count`;
`AbuseDefenceSpoofDetected` with control_id="STEP_UP_FAIL".

#### §D-2.4. Session anti-spoof (HMAC + audience + SameSite + TLS exporter)

**Layer:** Auth + transport.

**Mechanism:**

**HMAC-signed session token:**
- Session token = `base64(opaque_session_id) || base64(hmac_sha256(opaque_session_id || audience_binding || exp))`.
- HMAC key per-cell (rotated weekly per ADR-0294); held in OpenBao;
  loaded by `microservices/identity/` at startup.

**Audience binding:**
- Token includes `audience_binding = sha256(tenant_id || expected_route_class || tls_exporter)`.
- Tokens issued for tenant T cannot be replayed against tenant T'.

**SameSite=Strict cookies:**
- Session cookie `Set-Cookie: oya_session=<token>; SameSite=Strict;
  Secure; HttpOnly; Domain=<tenant_subdomain>; Path=/`.
- SameSite=Strict prevents CSRF: cookie not sent on cross-site
  requests.
- Secure ensures cookie is HTTPS-only.
- HttpOnly prevents JavaScript read.

**Rotating session-id on privilege escalation:**
- On step-up auth (Class C → D escalation) or sensitive operation,
  session-id is rotated; old session-id immediately invalidated
  server-side.

**TLS exporter binding (per RFC 8473 token-binding):**
- For browsers + clients that support token-binding (Chrome
  experimental, Edge supported), the session token is cryptographically
  bound to the TLS exporter; replay on different TLS connection
  fails.

**Hyperscaler precedent:**
- Google's GAIA cookie binding + recent SameSite=Strict default.
- AWS console session cookies bound per-region.
- Stripe's session-token rotation per dashboard 2024 update.
- OAuth 2.0 DPoP (Demonstrating Proof of Possession, RFC 9449) — the
  modern token-binding successor.

**Failure modes:**
- **Token replay:** Audience binding mismatch + TLS exporter mismatch
  ⇒ forbid; emit `AbuseDefenceSpoofDetected` with control_id="SESSION_REPLAY".
- **Cookie theft via XSS:** HttpOnly + CSP block JS access; session
  invalidation on suspicious activity.
- **Browser lacks token-binding:** Audience-binding without TLS
  exporter still provides reasonable protection.

**Rollback path:** Session token format is `microservices/identity/`
versioned; downgrade path via per-cell config.

**Observability emission:**
`oya_identity_session_replay_attempted_count`;
`AbuseDefenceSpoofDetected` with control_id="SESSION_REPLAY".

#### §D-2.5. Payload anti-spoof (HMAC + signed JWT for webhooks + machine clients)

**Layer:** API.

**Mechanism:**

**Webhook HMAC signing (per ADR-0273-style):**
- For each tenant, oyatie issues a per-webhook HMAC key
  (`webhook_secret`).
- Outbound webhook to tenant's URL:
  `X-Oya-Signature-Timestamp: <unix_seconds>`
  `X-Oya-Signature: sha256(webhook_secret + ":" + timestamp + ":" + body)`
- Inbound webhook from tenant (e.g., third-party callback to
  oyatie): tenant signs same way; oyatie verifies.
- Replay window: ≤ 5 minutes (timestamp recency); ≥ 5 minutes
  rejected.
- Idempotency: tenant supplies `Idempotency-Key` header; oyatie
  deduplicates.

**Machine client JWT (per ADR-0188 + ADR-0295):**
- Machine clients (API consumers, agents) authenticate via signed
  JWT bound to client identity.
- JWT includes `iss` (issuer), `sub` (client id), `aud` (oyatie
  resource), `iat`, `exp` ≤ 1h, `jti` (one-time use).
- Signed by client's private key; oyatie verifies via client's
  public key registered during onboarding.

**mTLS for machine-to-machine:**
- Per ADR-0253 + ADR-0044, sibling-µservice calls + sensitive
  partner integrations use mTLS; each side presents an X.509 cert
  rooted in oyatie's trust chain (or partner's trust chain pinned
  by oyatie).

**Hyperscaler precedent:**
- Stripe's webhook signature scheme (per stripe.com/docs/webhooks/signatures).
- GitHub webhook HMAC signature (per docs.github.com/en/webhooks/using-webhooks/validating-webhook-deliveries).
- AWS signature v4 for API requests.
- Twilio's webhook signature with X-Twilio-Signature.

**Failure modes:**
- **Webhook key compromise:** Per-tenant rotation flow; tenant
  invalidates and reissues; new webhook URL signed with new key.
- **Replay outside 5-min window:** Cleanly rejected; emit
  `AbuseDefenceSpoofDetected` with control_id="WEBHOOK_REPLAY".
- **Idempotency-key collision:** Deduplication is correct; tenant
  retries with same key produce identical responses.

**Rollback path:** Webhook signature format is `microservices/<ms>/`
versioned; downgrade per ADR-0258 SemVer.

**Observability emission:**
`oya_webhook_signature_check_count`;
`AbuseDefenceSpoofDetected` with control_id="WEBHOOK_HMAC".

#### §D-2.6. Audit-trail anti-spoof (per ADR-0263 + sidecar signing per ADR-0296)

**Layer:** Audit (`microservices/audit-chain/`).

**Mechanism (cross-references ADR-0263 + ADR-0028 Merkle-sealed):**

**Per-µservice signing key in sidecar:**
- Per ADR-0296 library-first credential sidecar, each µservice's
  audit-signing key is held in a sidecar container with ≤60s
  OpenBao TTL.
- Audit events emitted by the µservice are signed by the sidecar
  before reaching audit-chain.
- µservice process itself never holds the signing key directly.

**Audit-chain Merkle-sealed (per ADR-0028):**
- audit-chain assembles signed audit rows into Merkle trees per
  ADR-0028.
- Each tree's root is sealed with the audit-chain Ed25519 key
  (tier 1 HSM per `docs/standards/fips-hsm-substrate-root-signing.md`).
- Trees published to an immutable transparency log (Sigstore-style
  Rekor for oyatie per ADR-0247).

**No in-µservice forgery possible:**
- Compromised µservice cannot forge a sealed audit row because the
  signing key never enters µservice process memory.
- Sidecar isolation per ADR-0296 means even root-on-µservice does
  not expose the signing key.

**Hyperscaler precedent:**
- AWS CloudTrail uses internal-AWS signing for tamper-evidence.
- Google Cloud Audit Logs use internal Google signing.
- Stripe's internal audit chain (per Stripe Engineering's "Sorbet
  in Production" mentions internal tamper-evident logs).
- Sigstore Rekor (canonical transparency log for sigstore artifacts).

**Failure modes:**
- **Sidecar unavailable:** µservice cannot emit signed audit; emit
  un-signed audit with `unsigned: true` marker; alert SEV-2.
- **audit-chain compromised:** Trees no longer verifiable; per
  ADR-0247 rollback to last-good tree + emergency root-key ceremony.
- **Replay of old audit rows:** Per-row timestamps + Merkle position
  bind row to chain position; replay rejected.

**Rollback path:** Audit signing infrastructure is substrate; no
per-µservice rollback path; substrate-level incident response per
runbook `audit-chain-integrity-recovery.md`.

**Observability emission:**
`AbuseDefenceSpoofDetected` with control_id="AUDIT_FORGERY"
(extremely rare).

#### §D-2.7. Webhook anti-spoof (inbound HMAC verification + replay window + idempotency)

**Layer:** Inbound.

**Mechanism (overlaps §D-2.5 — this row focuses on the inbound side):**

For every inbound webhook (third-party callback, partner integration,
oyatie-receiving-from-tenant):

1. **HMAC signature verification:** Per the partner's documented
   signature scheme (Stripe, GitHub, Twilio all publish theirs).
2. **Replay window:** Timestamp must be within ≤5 minutes of server
   time; out-of-window rejected.
3. **Idempotency-key required:** `Idempotency-Key` header on every
   webhook; oyatie deduplicates within 24h window.
4. **Source IP allow-list:** Optional per-tenant allow-list of
   partner IP ranges (e.g., Stripe's published webhook IPs).
5. **TLS strict:** Inbound webhook only over TLS 1.3 per ADR-0253.

**Hyperscaler precedent:**
- Same as §D-2.5; Stripe's webhook documentation is the de-facto
  industry reference.

**Failure modes:**
- **Same as §D-2.5.**

**Rollback path:** Per partner's signature scheme; oyatie's verifier
is versioned per ADR-0258.

**Observability emission:**
`oya_webhook_inbound_signature_check_count` {result, partner_id};
`AbuseDefenceSpoofDetected` with control_id="WEBHOOK_INBOUND".

#### §D-2.8. Caller anti-spoof — SPIFFE workload identity (per ADR-0295)

**Layer:** mTLS / east-west.

**Mechanism (cross-references ADR-0253 + ADR-0295):**

**SPIFFE workload identity:**
- Every µservice runs a SPIRE agent + SPIFFE Workload API per
  ADR-0295.
- Each µservice replica is issued a short-lived SVID (X.509-SVID
  with ≤10min TTL) by the cell's SPIRE server.
- SVID encodes `spiffe://oyatie/<cell_id>/<µservice_id>/<replica_id>`.

**Every µservice-to-µservice call carries SVID:**
- Calling µservice presents SVID as client cert in mTLS handshake.
- Receiving µservice validates SVID via SPIRE Workload API.

**Cedar gate verifies caller identity before action:**
- Cedar evaluator receives `principal.svid` (parsed SPIFFE-ID).
- Per-action permits in the µservice's Cedar fragments allow only
  the approved caller set:
  ```cedar
  permit (
      principal == SVID::"spiffe://oyatie/<cell>/sender-µservice/*",
      action == Action::"call-receiver-method",
      resource
  );
  forbid (
      principal,
      action == Action::"call-receiver-method",
      resource
  ) unless { /* approved caller list */ };
  ```

**Hyperscaler precedent:**
- Google's ALTS (workload identity) is the canonical Google internal
  equivalent (per Verma et al., "Borg, Omega, and Kubernetes," CACM
  2016 + Google's "Cloud Native Authorization with ALTS" 2019 talk).
- AWS IAM Roles for Service Accounts (IRSA) on EKS.
- Azure Workload Identity (federated).
- SPIFFE / SPIRE are the canonical CNCF-graduated standards
  implementation.

**Failure modes:**
- **SVID compromise:** Per ≤10min TTL, exposure window bounded;
  per ADR-0295 kill-switch, compromised SVIDs can be revoked at
  cell level.
- **SPIRE server outage:** Per ADR-0295 + ADR-0253, fallback to
  cached SVIDs for ≤TTL; emit `AbuseDefenceSPIREOutage`.
- **Cross-cell SVID acceptance:** Per ADR-0248 cell-isolation, no
  cross-cell SVID trust by default; opt-in federation per
  approved cross-cell traffic permits.

**Rollback path:** SPIFFE deployment is substrate; rollback per
ADR-0295 runbook.

**Observability emission:**
`oya_spiffe_svid_validation_count`;
`AbuseDefenceSpoofDetected` with control_id="SVID_INVALID".

### §D-3. Anti-scrape taxonomy — eight controls fully expanded

The eight-row anti-scrape taxonomy:

#### §D-3.1. Per-tenant + per-fingerprint rate limiting

**Layer:** Edge.

**Mechanism (overlaps §D-1.1 but with aggressive low caps on read
endpoints for unauthenticated):**

| Surface | Authenticated tenant cap | Unauthenticated cap |
|---|---|---|
| Public read (`/v1/public/*`) | 1k req/sec per tenant | 30 req/sec per IP, 100 req/sec per ASN aggregate |
| Search (`/v1/search`) | 100 req/sec per tenant | 5 req/sec per IP |
| Listing pagination | 50 req/sec per tenant | 1 req/sec per IP (cursor pagination forced) |
| Detail page (`/v1/items/<id>`) | 500 req/sec per tenant | 10 req/sec per IP |

**Cursor pagination forcing (per ADR-0150 cursor-pagination-canonical):**
- All listing endpoints use cursor pagination; offset pagination
  rejected.
- Per-cursor lifetime ≤ 1h; cursor cannot be reused after expiry.
- Per-session cursor scoping: same session can paginate; cross-
  session cursor sharing rejected.

**Tighter caps for higher tenant tiers (paradoxical at first
glance but correct):** A `HIGH_RISK` tenant (financial, health)
has *tighter* per-route caps for unauthenticated read because the
data is sensitive and bulk-extraction risk is highest. A
`B2C_CONSUMER` tenant has medium caps. A `FRIENDLY_CRAWLER_PARTNER`
has permissive caps but logged thoroughly.

**Hyperscaler precedent:**
- Same as §D-1.1 + GitHub's per-route rate limiting documented at
  docs.github.com/en/rest/rate-limit.

**Failure modes:**
- **Same as §D-1.1.**

**Rollback path:** Per-route caps configured per-tenant.

**Observability emission:**
`oya_abuse_defence_scrape_ratelimit_hit_count` {tenant_audience_type,
route_class};
`AbuseDefenceScrapeBlocked` with refusal_layer="EDGE_RATELIMIT".

#### §D-3.2. Pattern-anomaly detection (BFS, alphabetical, parallel-tab signatures)

**Layer:** Edge + µservice.

**Mechanism:**

**Breadth-first crawl detection:**
- Request sequence pattern: `/v1/items/1`, `/v1/items/2`,
  `/v1/items/3`, ... (sequential ID enumeration).
- LSTM model trained on legitimate browsing sessions vs. scraping
  sessions classifies.

**Alphabetical pagination detection:**
- Request sequence: `/v1/users?prefix=A`, `/v1/users?prefix=B`, ...
- Detected via path-pattern entropy across a session.

**High page-depth fan-out:**
- Same session loads >50 items in <60s with no human-pace dwell
  time between.

**Parallel-tab signatures:**
- Same session-id (or fingerprint) issues ≥5 simultaneous unrelated
  requests in <100ms — pattern not characteristic of human browsing
  (which is mostly sequential with parallelism limited by browser
  connection limits).

**Forwarded context:**
- µservice receives `X-Oya-Scrape-Pattern: <BFS|ALPHA|FANOUT|PARALLEL_TAB>`
  header when pattern detected; Cedar consumes as
  `principal.scrape_pattern_detected`.

**Hyperscaler precedent:**
- Akamai Bot Manager LSTM-based behavioural detection (per their
  2023 white paper).
- Cloudflare Bot Management behavioural features (per their 2024
  blog).

**Failure modes:**
- **Legitimate analytics or bulk loader trips pattern:** Tenant
  audience type `FRIENDLY_CRAWLER_PARTNER` exempt; legitimate
  partner bots register via §D-3.4 paid API.
- **Adversary randomizes request order:** Pattern entropy still
  detectable from session-level signals (request rate, dwell time,
  fingerprint).

**Rollback path:** Pattern detection thresholds configurable per
ADR-0294.

**Observability emission:**
`oya_abuse_defence_scrape_pattern_detected_count`;
`AbuseDefenceScrapeBlocked` with refusal_layer="PATTERN_DETECTION".

#### §D-3.3. robots.txt + Sitemaps + crawl-delay authority (RFC 9309)

**Layer:** Edge.

**Mechanism (per RFC 9309 robots.txt specification, September 2022):**

**Per-tenant robots.txt:**
- Each tenant publishes a per-locale `/robots.txt` at the tenant's
  apex domain.
- robots.txt declares per-User-Agent allow/disallow + crawl-delay
  + sitemap location.
- oyatie's edge enforces robots.txt by user-agent matching:
  scrapers ignoring robots.txt have `User-Agent` reputation
  classified as `bot_malicious`; bot-score elevated; rate-limits
  tightened.

**Sitemaps protocol:**
- Each tenant publishes XML sitemap per `sitemaps.org` specification.
- Legitimate search engines (Googlebot, Bingbot, DuckDuckGoBot)
  discover content via sitemap.
- Sitemap URL listed in robots.txt + submitted to Google Search
  Console + Bing Webmaster Tools (per-tenant onboarding).

**Crawl-delay enforcement:**
- robots.txt `Crawl-delay: N` directive enforced; bots exceeding
  rate trip per-User-Agent rate-limit (§D-1.1).
- Per RFC 9309, crawl-delay is advisory; oyatie elevates it to
  enforced by treating violators as `bot_malicious`.

**User-agent-based rejection:**
- Known-bad crawlers (e.g., scraping-as-a-service user-agents
  `ScrapeNinja/`, `Apify/`, `DataForSEO/` when not opted-in via
  paid API) rejected at edge.

**Hyperscaler precedent:**
- Google's robots.txt enforcement (foundational standard).
- Cloudflare's "Verified Bots" allow-list (per Cloudflare's
  documentation).
- Akamai's bot-detection categories include robots.txt compliance.

**Failure modes:**
- **Legitimate crawler with mis-configured user-agent:** Crawler
  Verified Bots program enrollment + manual whitelist on appeal.
- **robots.txt cache stale:** Edge fetches robots.txt with ≤1h
  TTL; tenant updates propagate.

**Rollback path:** Per-tenant robots.txt is authoritative; tenants
control their own crawler policy.

**Observability emission:**
`oya_robots_violation_count`;
`AbuseDefenceScrapeBlocked` with refusal_layer="ROBOTS_VIOLATION".

#### §D-3.4. Paid-API tier for legitimate scrapers

**Layer:** API gateway.

**Mechanism:**

**Per-tenant paid API tier:**
- Tenant offers a paid API surface for legitimate bulk consumers
  (search engines, data aggregators, academic researchers).
- Tenant sets per-tier rate limit + per-tier price + per-tier
  ToS acceptance.
- Paid API consumers authenticate via OAuth2 client credentials
  (per ADR-0188); their requests are tagged `audience_type=FRIENDLY_CRAWLER_PARTNER`.

**ToS-of-service:**
- Bulk consumers accept a Bulk Data Access Terms-of-Service that
  prohibits redistribution + requires attribution + permits oyatie
  audit.

**Cedar permission:**
- `audience_type == "FRIENDLY_CRAWLER_PARTNER"` ⇒ scrape-pattern
  detection still active but Cedar permits the patterns it
  declared in its onboarding spec.

**Pricing model (illustrative):**
- Free tier: 100 req/min per IP.
- Standard paid tier: 10k req/min for $99/month per consumer.
- Premium tier: 100k req/min for $999/month + per-extraction fees.

**Hyperscaler precedent:**
- Google's Programmable Search API (paid bulk access to search
  results).
- Twitter's enterprise API (now X API; tiered access since 2010).
- Reddit's paid API (since 2023 pricing change).
- LinkedIn's official partner API.

**Failure modes:**
- **Paid consumer abuses tier:** Tier downgrade; eventual termination
  per ToS.
- **Paid consumer's clients leak credentials:** Per ADR-0296
  credential isolation; tenant rotates.

**Rollback path:** Paid tier is product surface; not a substrate
rollback.

**Observability emission:**
`oya_paid_api_usage_count` {tier, consumer_id};
`AbuseDefenceScrapeBlocked` rarely emitted for paid consumers
(only on quota exceed).

#### §D-3.5. Content fingerprinting / per-user watermarking

**Layer:** µservice.

**Mechanism:**

**Per-user invisible watermark on high-value content:**
- Text content: zero-width-character watermarks (U+200B, U+200C,
  U+200D, U+FEFF interleaved per user-id bits) embedded in text
  rendered to the user's session.
- Image content: steganographic watermark via least-significant-bit
  manipulation or DCT-domain perturbation (per Watermarking
  Information Encoding Standard).
- Audio content: spread-spectrum audio watermark in inaudible
  frequencies.
- PDF/document content: per-user invisible footer + EXIF-like
  metadata + structural watermarking (kerning perturbations).

**Per-tenant watermark per ADR-0244 tenant scoping:**
- Watermark encodes `tenant_id || user_id || timestamp || session_id`
  via a tenant-key-bound HMAC; only the tenant + oyatie can decode.

**Leak source identification:**
- When watermarked content surfaces in leaked corpora (LLM training
  data, scrape archives), oyatie's leak-detection pipeline
  recovers the watermark + identifies the source user/session.

**Hyperscaler precedent:**
- Apple iTunes (legacy DRM-era): per-user iTunes metadata embedded
  in purchased music files (2007-2009 era).
- Netflix: per-user video-stream watermarks (forensic watermarking
  per their 2018 published patent applications).
- Spotify: per-user audio fingerprints on certain content.
- Google's published research on text-document watermarking
  ("Embedding Watermarks in Text," Sigir 2008).

**Failure modes:**
- **Adversary strips watermark:** Possible but expensive (must
  re-render content); incremental fidelity loss; some watermarks
  (DCT-domain image) survive compression + cropping.
- **Watermark collision:** With ≥64-bit watermark, collision
  probability negligible.
- **Watermark exposure undermines protection:** Watermark key is
  HMAC-bound to tenant; per-user encoding unique; even known watermark
  scheme cannot be forged without tenant key.

**Rollback path:** Watermarking can be disabled per-tenant per
ADR-0294.

**Observability emission:**
`oya_content_watermark_emit_count`;
`AbuseDefenceScrapeBlocked` rarely from this control directly;
`AbuseDefenceWatermarkRecovered` audit class on leak detection.

#### §D-3.6. Adaptive challenge on scrape-pattern

**Layer:** Edge.

**Mechanism:**

**Composition: bot-score + scrape-pattern + tenant-policy → adaptive
challenge.**

When scrape-pattern detected (§D-3.2) AND bot-score elevated AND
tenant policy says "challenge-on-scrape":

1. **CAPTCHA challenge** (per §D-1.4) — Turnstile invisible →
   Turnstile managed → hCaptcha.
2. **JS proof-of-work challenge** — client computes a hash chain
   (~5s CPU work); legitimate user passes; bot pipeline economics
   degrade.
3. **Throttle-then-degrade** — request rate progressively reduced
   (5s delay, 30s delay, 5min delay); attacker eventually gives
   up due to opportunity cost.

**Tenant policy customization:**
- Tenant can choose per-route challenge policy (e.g., "CAPTCHA on
  /v1/search, JS-PoW on /v1/items/*, throttle on /v1/listing").

**Hyperscaler precedent:**
- Cloudflare's "Bot Fight Mode" issues JS-PoW challenges to
  suspected bots.
- Akamai's Bot Manager has adaptive challenge logic.

**Failure modes:**
- **JS-PoW evaded by full-browser headless scraping:** Less common
  due to cost; bot-score elevation still tracks.
- **Throttle accidentally hits legitimate user during traffic
  spike:** Per-tenant SLO + auto-tuning.

**Rollback path:** Adaptive challenge thresholds per-tenant.

**Observability emission:**
`oya_abuse_defence_adaptive_challenge_issued_count`;
`AbuseDefenceScrapeBlocked` refusal_layer="ADAPTIVE_CHALLENGE".

#### §D-3.7. Dynamic content rewriting (CSS class randomisation, structural HTML mutation)

**Layer:** µservice (frontend rendering).

**Mechanism:**

**Per-session CSS class randomisation:**
- React/SSR per-session: CSS class names are post-build randomised
  per-session deployment (`.btn-primary` → `.b-prm-xY7zQ`,
  `.user-card` → `.uc-aBc1F`, etc.).
- Per-session unique; scrapers targeting CSS selectors break on
  next session.

**Structural HTML mutation:**
- Per-session: HTML tag wrappers vary (`<div>` → `<section>`,
  `<span>` → `<small>`), padding `<div>` elements injected per
  session, attribute ordering randomised.
- Semantic API surface (JSON-LD, microdata) remains stable for
  search engines + accessibility tools.

**Semantic-API stable, scrape-surface unstable:**
- Per ADR-0188 + accessibility doctrine, the semantic API surface
  (ARIA labels, JSON-LD) is stable and indexable.
- The visual HTML/CSS surface is unstable for scrapers; legitimate
  search engines consume the semantic surface.

**Hyperscaler precedent:**
- LinkedIn (per Hacker News + Reddit discussions 2018-2024) is
  known for aggressive HTML mutation against scrapers.
- Instagram's web surface mutates frequently (per published research).
- Twitter/X's web surface mutates with each release.

**Failure modes:**
- **Mutation breaks accessibility tools:** Semantic API surface
  remains stable; accessibility tested via automated WCAG checks.
- **Mutation breaks SEO:** Search engines crawl semantic API
  surface; SEO unaffected.

**Rollback path:** Mutation can be downgraded to "stable mode" for
debugging per ADR-0294.

**Observability emission:**
`oya_content_mutation_emit_count`;
no direct `AbuseDefence*` audit emission for this control alone.

#### §D-3.8. Legal-channel registration (Bug Bounty + abuse-report + DMCA + GDPR Article 14)

**Layer:** Out-of-band.

**Mechanism:**

**Public Bug Bounty surface:**
- `bugbounty@oyatie.com` (or per-tenant equivalent for B2C tenants);
  acknowledged within ≤24h.
- HackerOne + Bugcrowd integration for triage.
- Per-class bounty: P0 (substrate compromise) up to $50k; P1
  (data extraction) up to $20k; P2 to $5k; P3 to $1k.
- Coordinated disclosure: ≥90 days from accepted report to public.

**Abuse-report email:**
- `abuse@oyatie.com` for general abuse (scraping that bypasses
  defences, phishing using oyatie infrastructure, spam from oyatie
  IPs).
- Per-tenant `abuse@<tenant_domain>` for tenant-specific abuse.
- SLA: response ≤8h, mitigation ≤48h on confirmed abuse.

**DMCA agent registration (US Copyright Office):**
- Per 17 U.S.C. § 512(c)(2), oyatie registers a DMCA designated
  agent with the US Copyright Office; agent contact published on
  oyatie's legal page.
- Per-tenant DMCA agent registration available for tenants
  hosting copyrighted content.

**GDPR Article 14 + Article 21 right-to-object surface:**
- Per GDPR Article 14 + 21, data subjects have right-to-object to
  processing for direct marketing, profiling, automated decision-
  making.
- Public-facing surface `gdpr@oyatie.com` + per-tenant
  `dpa@<tenant_domain>` to file Article 21 objection.
- Article 14 transparency disclosure: oyatie's processing operations
  documented + accessible without DSAR for most categories.

**CCPA "do not sell" surface:**
- Per CA Civ Code § 1798.135(a), California residents can opt out
  of sale of personal info.
- "Do Not Sell My Personal Information" link on consumer surfaces.

**COPPA + KOSA refusal (per ADR-0292):**
- Bots targeting minor-targeted surfaces face additional refusal;
  any bot targeting `/v1/users?age<13` patterns receives certain-bot
  classification + auto-refusal.

**Hyperscaler precedent:**
- HackerOne + Bugcrowd are the canonical bug-bounty platforms;
  Cloudflare, Google, Apple, Microsoft, Stripe all use them.
- DMCA agent registration is statutory.
- GDPR Article 14 + 21 surfaces are mandatory for EU-operating
  businesses.

**Failure modes:**
- **Bad-faith abuse report (false-positive):** Triage process
  filters; rate-limits per reporter.
- **DMCA abuse (false copyright claims):** Counter-notice procedure
  per 17 U.S.C. § 512(g).

**Rollback path:** N/A — legal compliance is non-rollback.

**Observability emission:**
Audit-event-classes `AbuseReportFiled`, `DMCANoticeReceived`,
`GDPRArticle21ObjectionReceived` per ADR-0263 registry.

### §D-4. Cedar policy fragment — `policy/abuse-defence.cedar`

The canonical Cedar fragment shape per ADR-0243 + ADR-0246 +
ADR-0294 (≥60s soak + signed publication).

**Fragment metadata header:**
```
// SCOPE: per-µservice abuse-defence baseline
// SIGNED BY: org-baseline-key (intermediate, chained to org root
//            per docs/standards/fips-hsm-substrate-root-signing.md)
// VERSION: v1
// EFFECTIVE_AT: <publication-timestamp-after-≥60s-soak>
// SUNSET_AT: null (long-lived; ratified per ADR-0294)
// SCHEMA_REF: /specs/cedar-fragment-schema.json
// BINDING_ADR: ADR-0297
```

**Cedar v4.2 grammar (LTS):**

```cedar
// ============================================================
// §1. Anti-bot family — bot-score forbid
// ============================================================
//
// Forbid when bot-score crosses certain-bot threshold, UNLESS the
// tenant audience type is FRIENDLY_CRAWLER_PARTNER (registered
// partner like Google Search, academic researcher with paid API).
//
forbid (principal, action, resource) when {
    principal.bot_score > 95
    && !(principal.tenant.audience_type == "FRIENDLY_CRAWLER_PARTNER")
};

// ============================================================
// §2. Anti-bot family — rate-limit forbid (per route class)
// ============================================================
//
// Forbid when per-tenant request-rate exceeds resource's per-route
// cap. The cap is read from resource.rate_limit_per_minute set by
// the tenancy substrate per-tenant tier.
//
forbid (principal, action, resource) when {
    principal.request_rate_per_minute > resource.rate_limit_per_minute
};

// ============================================================
// §3. Anti-scrape family — depth + age forbid
// ============================================================
//
// Forbid bulk-read actions when the principal's fingerprint is
// fresh (likely a new session created to evade tenant-bound
// quotas) AND the requested depth exceeds 50.
//
forbid (principal == ?, action in [Action::"Read", Action::"Scrape"], resource) when {
    principal.fingerprint_age_seconds < 30
    && action.depth > 50
};

// ============================================================
// §4. Anti-spoof family — workload-identity mismatch
// ============================================================
//
// Forbid east-west call when caller's SPIFFE-SVID is not in the
// approved set declared by the receiver µservice. SPIFFE-SVID
// comparison is exact-match on the SPIFFE ID after parsing.
//
forbid (principal, action == Action::"east-west-call", resource) when {
    !(principal.svid in resource.approved_caller_svid_set)
};

// ============================================================
// §5. Anti-spoof family — session replay
// ============================================================
//
// Forbid when session-token's audience-binding does not match
// the current request's audience.
//
forbid (principal, action, resource) when {
    principal.session_audience_binding != resource.expected_audience_binding
};

// ============================================================
// §6. Anti-bot family — minor-user surface refusal (per ADR-0292)
// ============================================================
//
// Forbid bot requests against minor-targeted surfaces (KOSA + EU
// AADC requirements); tenant's audience type MINOR_TARGETED gets
// elevated sensitivity (bot-score threshold 70 instead of 95).
//
forbid (principal, action, resource) when {
    principal.bot_score > 70
    && resource.audience_type == "MINOR_TARGETED"
    && !(principal.tenant.audience_type == "FRIENDLY_CRAWLER_PARTNER")
};

// ============================================================
// §7. Friendly crawler partner allow-list
// ============================================================
//
// Explicit permit for FRIENDLY_CRAWLER_PARTNER tenants on
// read-only routes; subject to their tier's rate cap.
//
permit (
    principal in Tenant::"oyatie",
    action in [Action::"Read", Action::"Scrape"],
    resource
) when {
    principal.tenant.audience_type == "FRIENDLY_CRAWLER_PARTNER"
    && principal.request_rate_per_minute <= resource.rate_limit_per_minute_friendly_crawler
};

// ============================================================
// §8. Default-deny baseline (per ADR-0243 §D-3)
// ============================================================
//
// Every action against every resource must be explicitly permitted
// by a positive permit fragment. This fragment provides the
// default-deny floor.
//
forbid (principal, action, resource);
```

**Fragment lifecycle (per ADR-0294):**
1. Author fragment with `oyatie.foundry.fragment-author` workflow.
2. Multispectrum review v2.4.0 per ADR-0243 §D-8 (facets F1, F2,
   F5, F6, F7, A1, A4, A6).
3. Sign by intermediate signing key per
   `docs/standards/fips-hsm-substrate-root-signing.md` Tier 1.
4. Publish to fragment registry; ≥60s soak.
5. Hot-reload to per-cell evaluators.
6. Audit `AbuseDefenceFragmentActivated` row emitted.
7. Rollback path: ActivateFragmentVersion(v_prev) per ADR-0247 §D-7.

### §D-5. Per-cell-tier variants

Per ADR-0248 cellular architecture, the abuse-defence baseline
varies by cell tier:

#### §D-5.1. Tier-0 edge cells (Cloudflare ~300 POPs Year 1-2; Pingora Year 3+)

**Rich controls:**
- Full ML bot-management (~50 features).
- TLS JA4 + JA4+ + HTTP/2-3 frame fingerprint.
- WAF (Cloudflare Managed + Coraza per ADR-0253 §D-2 OSS path).
- DDoS absorption (Cloudflare native + Coraza-equivalent fallback).
- Rate-limiting (per-IP, per-fingerprint, per-tenant, per-route).
- Device attestation forwarding (App Attest, Play Integrity, WebAuthn).
- CAPTCHA composition stack (Turnstile + hCaptcha + Cloudflare
  Challenge).
- ECH advertisement per ADR-0253.
- PQC hybrid offering per ADR-0253.

**Forwarded headers to downstream:**
- `X-Oya-Bot-Score: <0-99>`
- `X-Oya-JA4: <fingerprint>`
- `X-Oya-JA4H: <http-frame-fingerprint>`
- `X-Oya-Device-Attestation: <jwt>`
- `X-Oya-Challenge-Solved: <token>` (when CAPTCHA passed)
- `X-Oya-Scrape-Pattern: <pattern_id>` (when detected)
- `X-Oya-Source-IP: <client-ip-after-CDN>`
- `X-Oya-Source-ASN: <asn>`
- `X-Oya-Source-Country: <iso2>`

#### §D-5.2. Tier-1 bootstrap cell

**Inherited from Tier-0 edge headers.** Bootstrap cell is internal-
only (per ADR-0248 §D-2); not directly internet-facing. Bot-score +
JA4 headers consumed for diagnostic purposes only.

#### §D-5.3. Tier-2 control plane cells

**Use forwarded bot-score header from Tier-0 edge.** Control plane
µservices (identity, tenancy, policy-engine, audit-chain, cell,
workflow-engine, observability, cloud-iac, dev-tools-cell-prod)
are reached via Tier-0 edge → Tier-2; bot-score forwarded;
µservice Cedar evaluator consumes.

**Additional µservice-level controls:**
- Per-action quota gates (§D-1.7).
- Honeypot routes (§D-1.8).
- Audit-trail anti-spoof (§D-2.6).
- SPIFFE workload identity (§D-2.8).

#### §D-5.4. Tier-3 data plane cells

**Same as Tier-2.** Data plane cells host product µservices (mail,
drive, calendar, messenger, workflow-studio, intelligence, etc.);
internet-facing via Tier-0 edge; bot-score forwarded.

**Additional µservice-level controls:**
- Content fingerprinting / per-user watermarking (§D-3.5).
- Dynamic content rewriting (§D-3.7).
- Pattern-anomaly detection (§D-3.2).
- Per-action quota gates (§D-1.7).
- Stolen-credential check on auth path (§D-1.6).

#### §D-5.5. Tier-3 data cells (non-internet-facing)

**Control family N/A.** Some Tier-3 data cells (e.g., per-tenant
postgres replicas, internal audit-chain stores) do not directly
egress to internet; they are reached only via Tier-2 or Tier-3
µservices over the east-west service mesh.

For these cells:
- Anti-bot: N/A (no internet-facing surface).
- Anti-spoof: SPIFFE workload identity per §D-2.8 + audit-trail
  per §D-2.6.
- Anti-scrape: N/A.

#### §D-5.6. Tier-4 reserved (financial-grade + fulfillment-grade)

**Per ADR-0248 §D-16 reserved.** When Tier-4 lands:
- All controls from §D-5.4 PLUS:
- Confidential compute (Cloud Hypervisor + Kata per ADR-0248 §D-14
  + ADR-0254).
- Hardware-backed device attestation required (TPM 2.0 + App Attest
  + Play Integrity); WebAuthn-only paths refused for Class D + E
  step-up.
- Per-transaction watermarking + signed receipts.
- Real-time fraud detection via Intelligence (per ADR-0255).

### §D-6. Observability — metrics, dashboards, audit-event-classes

Per ADR-0263 observability emission contract.

#### §D-6.1. Metrics

| Metric | Type | Dimensions | Cardinality budget |
|---|---|---|---|
| `oya_abuse_defence_bot_score_histogram` | histogram | {tenant_audience_type, route_class} | ≤ 100 per µservice per day |
| `oya_abuse_defence_captcha_challenge_rate` | counter | {challenge_type, bot_score_band} | ≤ 50 per µservice per day |
| `oya_abuse_defence_spoof_detected_count` | counter | {control_id} | ≤ 20 per µservice per day |
| `oya_abuse_defence_scrape_pattern_detected_count` | counter | {scrape_pattern_id, refusal_layer} | ≤ 30 per µservice per day |
| `oya_abuse_defence_request_blocked_count` | counter | {family: bot/spoof/scrape, layer: edge/µservice/cedar} | ≤ 50 per µservice per day |
| `oya_abuse_defence_ratelimit_hit_count` | counter | {dimension: ip/fp/tenant/route, route_class} | ≤ 100 per µservice per day |
| `oya_abuse_defence_quota_check_count` | counter | {result: allow/forbid, action_class} | ≤ 50 per µservice per day |
| `oya_abuse_defence_challenge_issued_count` | counter | {challenge_type, bot_score_band} | ≤ 50 per µservice per day |
| `oya_abuse_defence_ja4_distribution` | histogram | {tenant_audience_type} | ≤ 200 per µservice per day |
| `oya_abuse_defence_device_attestation_verified_rate` | gauge | {platform: ios/android/web} | ≤ 10 per µservice per day |
| `oya_abuse_defence_stolen_credential_check_count` | counter | {result: clean/pwned/stuffing} | ≤ 10 per µservice per day |
| `oya_abuse_defence_session_replay_attempted_count` | counter | {} | ≤ 1 per µservice per day |
| `oya_webhook_signature_check_count` | counter | {result, direction: in/out} | ≤ 5 per µservice per day |
| `oya_spiffe_svid_validation_count` | counter | {result: pass/fail} | ≤ 5 per µservice per day |
| `oya_content_watermark_emit_count` | counter | {content_type} | ≤ 10 per µservice per day |
| `oya_robots_violation_count` | counter | {user_agent_class} | ≤ 50 per µservice per day |
| `oya_paid_api_usage_count` | counter | {tier, consumer_id_class} | ≤ 100 per µservice per day |

#### §D-6.2. Dashboards

Per-µservice dashboard at
`microservices/observability/dashboards/abuse-defence.json` (flag
for follow-up authoring — Grafana JSON shape per
`microservices/observability/dashboards/_TEMPLATE.json`).

Aggregate dashboard at
`microservices/observability/dashboards/abuse-defence-fleet.json`
showing planetary-wide abuse trends.

Dashboard panels:
- Bot-score histogram by tenant audience type.
- CAPTCHA challenge rate (issued vs solved vs failed).
- Spoof detection counter by control_id.
- Scrape pattern detection counter by pattern_id.
- Per-tenant abuse-defence false-positive rate (SLO 0.1%).
- Request-blocked counter by family + layer.
- Top blocked IPs / ASNs / countries / user-agents (90-day window).
- Geographic heatmap of blocked traffic.
- Per-µservice abuse-defence latency overhead (P50/P95/P99).

#### §D-6.3. Audit-event-classes (registered in ADR-0263 registry)

| Audit class | Emission trigger | Severity | Carries |
|---|---|---|---|
| `AbuseDefenceBotBlocked` | Tier-0 edge or µservice Cedar gate forbids on bot-score | INFO | {principal_id, tenant_id, request_id, bot_score, fingerprint_hash, blocking_rule_id, suggested_remediation} |
| `AbuseDefenceSpoofDetected` | Any anti-spoof control fires | WARN | {principal_id_claimed, tenant_id, control_id, expected_vs_actual_digest} |
| `AbuseDefenceScrapeBlocked` | Pattern-anomaly + scrape-rate forbid | INFO | {principal_id, tenant_id, scrape_pattern_id, refusal_layer, tenant_audience_type} |
| `AbuseDefenceChallengeIssued` | CAPTCHA or PoW challenge issued | INFO | {principal_id, challenge_type, bot_score, suspected_class} |
| `AbuseDefenceChallengeSolved` | Challenge succeeded | INFO | {principal_id, challenge_type, time_to_solve_ms} |
| `AbuseDefenceChallengeFailed` | Challenge failed | INFO | {principal_id, challenge_type, attempts} |
| `AbuseDefenceRateLimitHit` | Rate-limit bucket exceeded | INFO | {principal_id, tenant_id, dimension, route_class} |
| `AbuseDefenceRateLimitFallback` | Edge fallback to in-process counters | WARN | {edge_pop_id, fallback_reason} |
| `AbuseDefenceHoneypotHit` | Honeypot route accessed | CRITICAL | {principal_id, route, tenant_id} |
| `AbuseDefenceCanaryRecovered` | Canary token surfaced in scrape | CRITICAL | {canary_id, recovered_in_corpus, source_session_id} |
| `AbuseDefenceQuotaExceeded` | Per-action quota exceeded | INFO | {principal_id, tenant_id, action_class, quota_value} |
| `AbuseDefenceCredentialPwned` | HIBP-detected password used | INFO | {principal_id, tenant_id (email-derived)} |
| `AbuseDefenceCredentialStuffing` | Credential-stuffing pattern detected | WARN | {target_emails_count, stuffing_pattern_signature} |
| `AbuseDefenceAttestationFailed` | Device attestation rejected | INFO | {principal_id, platform, attestation_failure_reason} |
| `AbuseDefenceVendorOutage` | Vendor service (Cloudflare, hCaptcha) unavailable | WARN | {vendor, fallback_taken} |
| `AbuseDefenceFragmentActivated` | Cedar fragment hot-reloaded | INFO | {fragment_version, soak_duration_s} |
| `AbuseDefenceWatermarkRecovered` | Per-user watermark identified in leak | WARN | {watermark_id, recovered_in_corpus} |
| `AbuseDefenceSPIREOutage` | SPIRE server unreachable | WARN | {cell_id, fallback_taken} |

### §D-7. Per-tenant audience-type tuning

Per `Tenant.audience_type` enum (ADR-0244 §D-3 extended):

| audience_type | Bot-mgmt sensitivity | Scrape sensitivity | Step-up class default | Notes |
|---|---|---|---|---|
| `FRIENDLY_CRAWLER_PARTNER` | Permissive (allow bot-score < 95 with logging) | Permissive (allow declared patterns) | Class A | Search engines, academic researchers, paid bulk consumers |
| `B2C_CONSUMER` | Default (forbid bot-score > 95) | Default (rate-limit + pattern detect) | Class B | Consumer brand surfaces (mail, drive, calendar, messenger, social, notes) |
| `B2B_TENANT` | Default (forbid bot-score > 95) | Default | Class B | Enterprise tenants (workflow-studio, intelligence, governance) |
| `HIGH_RISK` | Aggressive (forbid bot-score > 70 + step-up CAPTCHA on bot-score > 30) | Aggressive (forbid bot-score > 50 + tight rate-limits) | Class D | Financial, health, payment-method addition, tenant admin |
| `MINOR_TARGETED` (per ADR-0292) | Aggressive (forbid bot-score > 70) | Aggressive | Class C minimum | Surfaces serving users < 13 (COPPA refusal) or 14-17 (KOSA tier) |
| `INTERNAL_DEV_TOOLS` | Default | Default | Class C | oyatie-internal dev surfaces (dev-tools-cell-N) per ADR-0247 |

Per-tenant adjustment: tenancy substrate provides per-tenant
controls to bump sensitivity. Tenants cannot decrease sensitivity
below their audience-type default (no compliance bypass) but can
increase.

### §D-8. Compliance interactions

#### §D-8.1. GDPR Article 14 + Article 21 right-to-object

Per GDPR Article 14, data controllers must provide transparency
about processing operations. Per Article 21, data subjects have
right-to-object to processing for direct marketing, profiling, and
automated decision-making.

oyatie's abuse-defence baseline implications:
- Bot-management ML scoring is "automated decision-making" under
  GDPR (recital 71); affected data subjects can object per Article 21.
- Object-to-bot-scoring surface: `gdpr@oyatie.com` + per-tenant
  `dpa@<tenant_domain>`.
- On objection: bot-scoring continues but the affected principal
  is not auto-blocked solely on bot-score; manual review required.

Per the GDPR pack registered per ADR-0251.

#### §D-8.2. CCPA "do not sell" surface

Per CA Civ Code § 1798.135(a):
- "Do Not Sell My Personal Information" link on consumer surfaces.
- Bot-mgmt feature data + behavioural signals are within scope of
  "personal info" under California Privacy Rights Act (CPRA, 2023+);
  CCPA opt-out limits sharing of these signals with third-party
  vendors.

Per the CCPA pack per ADR-0251.

#### §D-8.3. COPPA <13 refusal (per ADR-0292)

Per the Children's Online Privacy Protection Act (COPPA):
- Surfaces detecting bot traffic against accounts identified as
  `age < 13` per ADR-0292: REFUSE the request + emit
  `AbuseDefenceMinorRefused`.
- Per ADR-0292 §D-N, "minor-affecting traffic the doctrine is not
  pinned" is refused; the abuse-defence layer enforces.

Per the COPPA / MINOR-USER-2024 pack per ADR-0251 + ADR-0292.

#### §D-8.4. KOSA tier for minor-targeted bot defence

Per the Kids Online Safety Act (KOSA) of 2024:
- Surfaces serving 14-17 audience (`audience_type =
  MINOR_TARGETED` with `age_band = 14-17`) get aggressive
  bot-mgmt sensitivity (per §D-7).
- "Addictive design" prohibition in KOSA prohibits engagement-
  optimization patterns; bot-mgmt may not be tuned to favor
  engagement metrics over safety.

Per the KOSA / MINOR-USER-2024 pack per ADR-0251 + ADR-0292.

#### §D-8.5. EU Digital Services Act (DSA) interactions

Per the EU Digital Services Act (Regulation (EU) 2022/2065):
- Very Large Online Platforms (VLOPs) + Very Large Online Search
  Engines (VLOSEs) must publish content moderation transparency
  reports.
- Abuse-defence decisions are within scope of content moderation
  transparency: oyatie publishes per-quarter aggregate counts of
  bot-blocks, spoof-detections, scrape-blocks.
- Affected users have right-to-appeal: per `appeal@oyatie.com`
  surface.

Per the DSA pack per ADR-0251.

#### §D-8.6. EU AI Act + Cedar abuse-defence

Per the EU AI Act (Regulation (EU) 2024/1689):
- Bot-management ML model is an "AI system" within scope of EU AI Act.
- Limited-risk classification (Article 50): transparency
  obligations apply (data subjects can request explanation of
  bot-classification decision).
- High-risk classification does NOT apply (bot-mgmt is not
  determining access to essential services, employment,
  education, etc.).

Per the EU-AI-ACT pack per ADR-0251 + ADR-0144.

#### §D-8.7. KR-CSAP + PIPL + Japan APPI

Per regional packs:
- KR-CSAP (Korean Cyber Security Assurance Program):
  bot-mgmt + scrape-defence are within the cybersecurity-control
  scope; certification audit references this ADR.
- CN-PIPL (China Personal Information Protection Law):
  cross-border data transfer rules limit forwarding bot-mgmt
  fingerprint data outside CN; the CN-PIPL pack ensures
  CN-data-residency for the bot-mgmt feature pipeline.
- Japan APPI (Act on Protection of Personal Information):
  legitimate-interest basis for bot-mgmt documented; explicit
  consent for profiling-class processing.

Per the per-pack registry per ADR-0251.

#### §D-8.8. PCI-DSS + HIPAA for HIGH_RISK tenants

Per PCI-DSS (Payment Card Industry Data Security Standard) 4.0:
- Requirement 6.4.1: anti-automation controls for payment-card
  surfaces. The abuse-defence baseline satisfies this; per-action
  HIGH_RISK tuning per §D-7 applies.
- Requirement 8.6: limit failed login attempts. Per §D-1.6
  stolen-credential check + per-IP rate-limiting per §D-1.1.

Per HIPAA Security Rule (45 CFR § 164.308 + § 164.312):
- §164.308(a)(5)(ii)(D): protection from malicious software (anti-
  bot covers automated attacks).
- §164.312(b): audit controls. Per ADR-0263 + audit-event-classes
  in §D-6.3.

Per the HIPAA + PCI packs per ADR-0251.

## §E. Implementation footprint

### §E.1. New crate

**`oya-shared-abuse-defence`** at `crates/oya-shared-abuse-defence/`.

**Crate structure (per ADR-0131 flat layout):**
```
crates/oya-shared-abuse-defence/
  Cargo.toml
  src/
    lib.rs                          # crate root, public API
    bot_score.rs                    # bot-score header parsing + traits
    fingerprint.rs                  # JA4 + JA4+ + HTTP-frame fingerprinting
    quota.rs                        # per-action quota gate trait
    cedar_evaluator_hook.rs         # Cedar evaluator integration per ADR-0246
    audit_emit.rs                   # audit-event-class emission helpers per ADR-0263
    rate_limit.rs                   # rate-limit bucket trait + impl
    challenge.rs                    # CAPTCHA challenge composition stack
    device_attestation.rs           # App Attest + Play Integrity + WebAuthn verifiers
    stolen_credential.rs            # HIBP k-anon + internal stuffing detector
    scrape_pattern.rs               # pattern-anomaly LSTM + heuristic detectors
    watermark.rs                    # zero-width + DCT + audio watermark library
    content_rewrite.rs              # per-session CSS class randomiser + HTML mutator
    honeypot.rs                     # honeypot route + canary token primitives
  tests/
    bot_score_parse.rs
    fingerprint_extraction.rs
    cedar_evaluator_hook.rs
    rate_limit_window.rs
    challenge_composition.rs
    watermark_encode_decode.rs
    scrape_pattern_lstm.rs
  fuzz/
    fuzz_targets/
      bot_score_header_fuzz.rs
      ja4_fingerprint_fuzz.rs
      webhook_hmac_fuzz.rs
```

**Public API (excerpt):**
```rust
// crates/oya-shared-abuse-defence/src/lib.rs

pub trait AbuseDefenceEvaluator: Send + Sync {
    fn evaluate(&self, context: AbuseDefenceContext) -> AbuseDefenceDecision;
}

pub struct AbuseDefenceContext {
    pub bot_score: Option<u8>,
    pub ja4_fingerprint: Option<JA4Fingerprint>,
    pub ja4h_fingerprint: Option<JA4HFingerprint>,
    pub device_attestation: Option<DeviceAttestation>,
    pub session_token_audience: Option<AudienceBinding>,
    pub spiffe_svid: Option<SpiffeId>,
    pub tenant_audience_type: AudienceType,
    pub request_route_class: RouteClass,
    pub request_method: HttpMethod,
    pub request_path: String,
    pub scrape_pattern_detected: Option<ScrapePatternId>,
    pub principal_id: PrincipalId,
    pub tenant_id: TenantId,
    pub request_id: RequestId,
}

pub enum AbuseDefenceDecision {
    Permit,
    Challenge(ChallengeType),
    Forbid(ForbidReason),
}

pub enum AudienceType {
    FriendlyCrawlerPartner,
    B2cConsumer,
    B2bTenant,
    HighRisk,
    MinorTargeted,
    InternalDevTools,
}

pub trait CedarEvaluatorHook {
    fn evaluate_abuse_defence(
        &self,
        ctx: &AbuseDefenceContext,
        cedar_fragment: &CedarFragment,
    ) -> CedarDecision;
}
```

**Cargo.toml dependencies:**
- `cedar-policy` v4.2 LTS per ADR-0243.
- `oya-shared-audit-chain` per ADR-0028 + ADR-0263.
- `oya-shared-cedar-eval` per ADR-0246 library-first.
- `oya-shared-spiffe` per ADR-0295.
- `tokio` 1.x async runtime.
- `tracing` for observability per ADR-0263.

### §E.2. New µservice extensions

For every internet-facing µservice, the following files MUST be
present:

#### §E.2.1. `policy/abuse-defence.cedar`

Per-µservice Cedar fragment per §D-4 above. Each µservice's
fragment contains the §D-4 baseline + µservice-specific extensions
(e.g., `microservices/intelligence/` adds a fragment to refuse
prompt-injection attempts that look like scrape patterns).

#### §E.2.2. `iac/<env>-edge-waf.yaml`

Per-µservice + per-env WAF configuration:

```yaml
# microservices/<ms>/iac/prod-edge-waf.yaml
# Cloudflare zone configuration for production
# Schema: per the Cloudflare Terraform Provider zone-settings schema

zone_id: "<cloudflare-zone-id>"
zone_name: "<msname>.oyatie.com"

bot_management:
  enabled: true
  fight_mode: "super-bot-fight-mode"
  auto_update_model: true
  ai_bots_protection: "block"  # Anthropic ClaudeBot, OpenAI GPTBot, etc.
  crawler_protection: "allow_listed"  # Google + Bing + DuckDuckGo allow-listed
  use_supreme_bot_management_rules: true

waf:
  enabled: true
  rulesets:
    - "cloudflare_managed_ruleset"
    - "cloudflare_owasp_core_ruleset"
    - "coraza_owasp_crs_4.0"  # OSS fallback
  custom_rules:
    - description: "Block honeypot route accesses"
      expression: '(http.request.uri.path matches "^/v1/admin/_unstable_dashboard")'
      action: "block"
    - description: "Tighten auth-path rate limit"
      expression: '(http.request.uri.path matches "^/v1/auth/")'
      action: "challenge"
      rate_limit:
        threshold: 10
        period: "1m"

ddos_protection:
  enabled: true
  sensitivity: "high"
  ruleset_action: "block"
  cost_protection: "enabled"

tls:
  min_version: "1.3"
  ciphers:
    - "TLS_AES_128_GCM_SHA256"
    - "TLS_AES_256_GCM_SHA384"
    - "TLS_CHACHA20_POLY1305_SHA256"
  ech: "enabled"
  pqc_hybrid: "enabled"

hsts:
  enabled: true
  max_age: 63072000
  include_subdomains: true
  preload: true

rate_limit:
  default:
    threshold: 1000
    period: "1m"
  per_route:
    "auth":
      threshold: 10
      period: "1m"
    "write":
      threshold: 100
      period: "1m"
    "read":
      threshold: 1000
      period: "1m"
    "admin":
      threshold: 5
      period: "1m"

forwarded_headers:
  bot_score: "X-Oya-Bot-Score"
  ja4: "X-Oya-JA4"
  ja4h: "X-Oya-JA4H"
  device_attestation: "X-Oya-Device-Attestation"
  challenge_solved: "X-Oya-Challenge-Solved"
  scrape_pattern: "X-Oya-Scrape-Pattern"
  source_ip: "X-Oya-Source-IP"
  source_asn: "X-Oya-Source-ASN"
  source_country: "X-Oya-Source-Country"
```

#### §E.2.3. Per-µservice dashboards

`microservices/<ms>/dashboards/abuse-defence.json` per ADR-0263
emission contract. Flag for follow-up authoring; per-µservice
dashboard inherits from the template at
`microservices/observability/dashboards/abuse-defence.json`.

#### §E.2.4. `ARCHITECTURE.md §abuse-defence` section

Each internet-facing µservice's `ARCHITECTURE.md` MUST include
a `§abuse-defence` section answering the 24-row taxonomy per
documentation-rigor.md §3.2.3. Sub-sections:

- §abuse-defence.anti-bot — which 8 controls wired (and per-route
  config).
- §abuse-defence.anti-spoof — which 8 controls wired.
- §abuse-defence.anti-scrape — which 8 controls wired.
- §abuse-defence.cedar-fragment — link to `policy/abuse-defence.cedar`.
- §abuse-defence.audience-type — which audience_type values served.

### §E.3. New runbooks (flag-only — authored by Wave-3 ops agents)

The following runbooks MUST be authored under `docs/runbooks/`
per `documentation-rigor.md` §2 runbook rigor (≥250 lines each;
Trigger / Pre-checks / Procedure / Verification / Rollback / Post-
incident / References). This ADR flags them for authoring; the
content is authored by Wave-3 ops agents.

- `docs/runbooks/abuse-defence-bot-storm.md` — incident response
  for sudden volumetric bot attack (≥10× baseline rate). Trigger:
  Cloudflare DDoS alarm + per-µservice `oya_abuse_defence_request_blocked_count`
  spike > 10×. Procedure: tighten bot-mgmt thresholds; engage
  Cloudflare Magic Transit if available; emergency rate-limit caps
  applied substrate-wide; engage council-security on-call.
- `docs/runbooks/abuse-defence-credential-stuffing.md` — incident
  response for credential-stuffing detection across multiple
  tenants. Trigger: `AbuseDefenceCredentialStuffing` audit class
  spike across multiple tenant_ids in <1h. Procedure: bulk-force
  affected accounts to step-up auth; emit tenant-level alert;
  engage HIBP + internal stuffing detector tuning; consider
  emergency-disable for affected route classes.
- `docs/runbooks/abuse-defence-scrape-pattern-detected.md` —
  response for sustained scrape pattern at planetary scale.
  Trigger: `AbuseDefenceScrapeBlocked` spike. Procedure: increase
  content-mutation aggressiveness; enable JS proof-of-work;
  consider blocking ASN-level; engage tenant on legal-channel
  registration if scrape targets specific tenant.

### §E.4. New CI lanes

Per `documentation-rigor.md` §3.2.3 + this ADR §B-§D.

- **`oya-governance-anti-bot-coverage`** — for every internet-
  facing µservice: verify `ARCHITECTURE.md §abuse-defence.anti-bot`
  declares 8 controls; verify `iac/<env>-edge-waf.yaml` includes
  the bot_management block; verify `policy/abuse-defence.cedar`
  contains the bot-score forbid; verify the µservice imports
  `oya-shared-abuse-defence`.
- **`oya-governance-anti-spoof-coverage`** — for every internet-
  facing µservice: verify DKIM/SPF/DMARC published per ADR-0273
  (if mail-emitting); verify TLS strict per ADR-0253; verify
  step-up auth class declarations per ADR-0188; verify SPIFFE SVID
  validation per ADR-0295; verify webhook HMAC verifier.
- **`oya-governance-anti-scrape-coverage`** — for every internet-
  facing µservice: verify rate-limit declarations; verify robots.txt
  authority; verify watermarking for high-value content (or explicit
  N/A declaration); verify paid API tier declaration; verify
  abuse-report + DMCA + GDPR Article 14 surfaces.
- **`oya-governance-abuse-defence`** — aggregate lane; rolls
  up the three above into a single advisory/BLOCKER verdict.
- **`oya-governance-abuse-defence-cedar-fragment`** — verify
  every internet-facing µservice has `policy/abuse-defence.cedar`
  + the fragment passes Cedar v4.2 LTS validation + the fragment
  is signed per `docs/standards/fips-hsm-substrate-root-signing.md`
  Tier 1.
- **`oya-governance-abuse-defence-audit-event-class-registered`**
  — verify the 18 audit-event-classes in §D-6.3 are present in
  ADR-0263 central registry.

All lanes are **advisory until 2026-08-15** to allow per-µservice
rollout; **BLOCKER from 2026-08-16**. PRs touching an internet-
facing µservice cannot merge after the BLOCKER date without all
lanes green.

### §E.5. Vendor selection rationale

#### §E.5.1. Primary edge: Cloudflare Bot Management

**Rationale:**
- The keystone bundle 2026-05-20's Cloudflare posture per ADR-0253
  §D-2 chose Cloudflare as the Year 1-2 edge layer. Bot Management
  is part of that posture.
- Cloudflare Bot Management is the canonical hyperscaler-grade
  bot mitigation; ~50 features; planetary-scale training corpus;
  documented at developers.cloudflare.com.
- ~300 POPs globally; planetary low-latency bot-score forwarding.
- Pricing per request ≪ per-µservice in-house cost (Cloudflare
  amortizes across thousands of customers).

**Limitations:**
- Per-tenant tuning requires zone-level config; oyatie's tenant
  control surface mediates per-tenant.
- Vendor lock-in to Cloudflare; mitigated by Pingora migration
  Year 3+ per ADR-0253.

#### §E.5.2. Failover: Akamai Bot Manager

**Rationale:**
- Cloudflare's ToS includes per-customer outage SLA; rare
  Cloudflare-wide outage (the November 2024 Cloudflare incident
  was the most recent material example) requires fallback path.
- Akamai Bot Manager runs at ~4,000 POPs (higher density than
  Cloudflare); LSTM behavioural model since 2023.
- Per ADR-0253 dual-CDN posture, oyatie maintains failover
  contracts with Akamai for high-traffic surfaces.

**Failover trigger:**
- Cloudflare-wide outage > 5 min;
- Per-zone DDoS escalation beyond Cloudflare's capacity;
- Cloudflare per-customer cost-protection breach.

#### §E.5.3. Challenge composition: hCaptcha + Cloudflare Turnstile + Cloudflare Challenge

**Rationale:**
- Turnstile: invisible challenge; ~95% legitimate-pass rate;
  GA 2022. Primary challenge for bot-score 30-60.
- hCaptcha: WCAG 2.1 AA tested; accessibility-compliant fallback.
  Primary for bot-score 60-90 when Turnstile rejected.
- Cloudflare Challenge: JS proof-of-work; runtime cost on headless-
  browser scrapers; secondary path.

**No Google reCAPTCHA dependency:**
- Per ADR-0211 in-house preference + ADR-0240 sovereign-cloud
  per-regional-pack, we avoid Google services in the critical
  path. reCAPTCHA is a Google service; not chosen.

#### §E.5.4. Credential-stuffing detection: HIBP + in-house

**Rationale:**
- HIBP is the canonical breach corpus; cited by NIST SP 800-63B.
- K-anonymity API preserves user privacy.
- In-house detector handles the per-tenant + cross-tenant patterns
  HIBP cannot see.

#### §E.5.5. Workload identity: SPIFFE/SPIRE per ADR-0295

Per ADR-0295 bootstrap CI + SPIFFE + kill-switch.

#### §E.5.6. Year 3+ migration to Pingora-native bot defence

Per ADR-0253 §D-2 Year 3+ migration:
- Pingora-native bot scoring (Cloudflare's open-source Rust proxy
  with Cloudflare-trained ML model; released 2024).
- In-house ML training on oyatie-wide labeled corpus.
- ~30 POPs initial → ~100 POPs Year 5 → ~300 POPs Year 7.

## §F. Migration

### §F.1. Per-µservice rollout sequenced by audience exposure

The 46 µservices have different audience-type exposure; rollout
is sequenced accordingly.

**Phase 1: B2C consumer surfaces (2026-05-21 to 2026-06-30).** B2C
surfaces face highest scrape + bot pressure. Per the corpus
snapshot in `documentation-rigor.md` §1, the B2C-internet-facing
µservices include: `mail`, `drive`, `calendar`, `messenger`,
`social`, `notes`, `shorts`, `news`, `marketplace` (consumer
side), `intelligence` (consumer surface), `app-store-consumer`,
`workflow-studio` (consumer mode).

For each: author `ARCHITECTURE.md §abuse-defence` + `policy/
abuse-defence.cedar` + `iac/prod-edge-waf.yaml` + dashboard
panels + per-µservice runbooks. Foundry-fitness lanes
verify (advisory).

**Phase 2: B2B tenant surfaces (2026-07-01 to 2026-07-31).**
B2B surfaces face moderate bot pressure (less scrape, more
credential-stuffing on enterprise SSO). B2B µservices:
`tenancy`, `identity`, `governance`, `audit-chain` (per-tenant
view), `policy-engine` (tenant-policy admin), `workflow-studio`
(B2B mode), `marketplace` (B2B catalog), `connector`, `feature-flags`,
`ops-dashboard-control-center`.

**Phase 3: Internal-platform surfaces (2026-08-01 to 2026-08-15).**
Substrate-substrate calls already SPIFFE-pinned per ADR-0253 +
ADR-0295; this phase adds the abuse-defence surface declarations
for completeness. µservices: `cell`, `cloud-iac`, `cloud-secrets`,
`observability`, `dev-tools-cell`, `foundry` (after dissolution
per ADR-0247).

**Phase 4: BLOCKER promotion (2026-08-16).**
All four CI lanes (`oya-governance-anti-bot-coverage`,
`oya-governance-anti-spoof-coverage`,
`oya-governance-anti-scrape-coverage`,
`oya-governance-abuse-defence`) promote from advisory to
BLOCKER. Per-µservice grace period 30d after lane lights
advisory (per the keystone bundle promotion-gate convention).

### §F.2. Per-µservice migration playbook

For each µservice:

1. Author `ARCHITECTURE.md §abuse-defence` section per §E.2.4
   above. Include audience_type declaration + 8-row anti-bot
   table + 8-row anti-spoof table + 8-row anti-scrape table.
2. Author `policy/abuse-defence.cedar` per §D-4. Run multispectrum
   review v2.4.0 facets F1, F2, F5, F6, F7, A1, A4, A6.
3. Author `iac/<env>-edge-waf.yaml` per §E.2.2.
4. Update `manifest.json` with `abuse_defence: { ... }` block
   declaring controls.
5. Add per-µservice dashboard `microservices/<ms>/dashboards/abuse-
   defence.json`.
6. Flag runbooks for authoring (per §E.3).
7. Run advisory CI lanes; fix findings.
8. Soak for 7d in dev-tools-cell-staging; verify SLO floor.
9. Promote to dev-tools-cell-prod.

### §F.3. Per-cell rollout pattern

Per ADR-0248 + ADR-0294 ≥60s soak:

1. Cedar fragment authored + signed.
2. Hot-load to dev-tools-cell-staging; observe ≥60s.
3. Hot-load to one Tier-2 control plane cell; observe ≥60s.
4. Hot-load to one Tier-3 data plane cell; observe ≥60s.
5. Hot-load to all Tier-2 cells; observe ≥5min.
6. Hot-load to all Tier-3 cells in waves of 10% per 5min.
7. SLO breach at any stage → automatic rollback via
   `oyatie.foundry.rollback-controller`.

### §F.4. What is NOT migrated

- Existing rate-limiting per-µservice that already exists (e.g.,
  api-gateway's own rate-limit) is NOT removed; it is augmented
  by the substrate-level rate-limiting.
- Per-µservice custom WAF rules NOT removed.
- The migration adds the substrate baseline; it does NOT
  replace per-µservice abuse defences.

### §F.5. Rollback path

Per ADR-0294 ≥60s soak + Cedar fragment rollback:

- Cedar fragment rollback: `ActivateFragmentVersion(v_prev)`.
- IaC rollback: revert `iac/<env>-edge-waf.yaml`; redeploy.
- Per-µservice opt-out: temporarily set
  `abuse_defence.opt_out: true` in `manifest.json` (advisory phase
  only; not permitted post-BLOCKER).

## §G. References

### §G.1. Hyperscaler precedents

- **Cloudflare Bot Management** — developers.cloudflare.com/bots/
  + Cloudflare blog "How Cloudflare's Bot Management algorithm
  reliably identifies bots" (2022, 2024 update).
- **Cloudflare DDoS reports** — radar.cloudflare.com (quarterly
  DDoS landscape reports; Q4 2024 published 3.5T mitigated DDoS
  requests).
- **Cloudflare Turnstile** — Cloudflare blog "Turnstile: Cloudflare's
  CAPTCHA alternative" (2022 announce; 2024 GA blog).
- **Cloudflare Pingora** — Cloudflare blog "How we built Pingora,
  the proxy that connects Cloudflare to the Internet" (2022);
  github.com/cloudflare/pingora (open-sourced 2024).
- **AWS Shield + AWS WAF Bot Control** — docs.aws.amazon.com/waf/
  latest/developerguide/aws-managed-rule-groups-bot.html (2021
  GA; 2024 Common + Target inspection levels).
- **AWS DDoS report** — AWS Shield Advanced 2.3 Tbps attack
  disclosure (October 2020 + November 2020 published incidents).
- **Google Cloud Armor Adaptive Protection** — cloud.google.com/
  armor/docs/adaptive-protection-overview (2021 GA).
- **Google reCAPTCHA Enterprise v3** — cloud.google.com/
  recaptcha-enterprise (2018+).
- **Akamai Bot Manager** — akamai.com/products/bot-manager (2014
  inception; 2023 LSTM update).
- **Akamai State of the Internet** — akamai.com/state-of-the-internet
  (Q2 2024 + Q4 2024 reports).
- **Apple App Attest** — developer.apple.com/documentation/
  devicecheck/establishing-your-app-s-integrity (iOS 14, 2020+).
- **Google Play Integrity API** — developer.android.com/google/play/
  integrity (2022 replacement for SafetyNet).
- **WebAuthn Origin-binding** — W3C WebAuthn Recommendation (2021)
  + FIDO Alliance specifications.

### §G.2. Standards + RFCs

- **TLS JA4 fingerprint specification** — FoxIO's JA4 spec at
  github.com/FoxIO-LLC/ja4 (2023 release; 2024 JA4+ updates).
- **HIBP API specification** — haveibeenpwned.com/API/v3 + the
  k-anonymity API at api.pwnedpasswords.com (2018+).
- **BIMI Working Group** — bimigroup.org + draft-ietf-dmarc-bimi-04
  (Brand Indicators for Message Identification).
- **robots.txt RFC 9309** — datatracker.ietf.org/doc/rfc9309
  (September 2022; formalises the 1994 informal standard).
- **DKIM RFC 6376** — datatracker.ietf.org/doc/rfc6376.
- **SPF RFC 7208** — datatracker.ietf.org/doc/rfc7208.
- **DMARC RFC 7489** — datatracker.ietf.org/doc/rfc7489.
- **ARC RFC 8617** — datatracker.ietf.org/doc/rfc8617.
- **DoH RFC 8484** — datatracker.ietf.org/doc/rfc8484.
- **OAuth 2.0 DPoP RFC 9449** — datatracker.ietf.org/doc/rfc9449.
- **Token Binding RFC 8473** — datatracker.ietf.org/doc/rfc8473.
- **Coraza WAF** — coraza.io (OSS WAF library; Apache 2.0
  license; OWASP CRS-compatible).

### §G.3. Legal + compliance

- **GDPR Article 14 + Article 21** — Regulation (EU) 2016/679
  Articles 14 (transparency) and 21 (right-to-object).
- **CCPA + CPRA** — CA Civil Code § 1798 et seq.; California
  Privacy Rights Act (2023+).
- **COPPA** — Children's Online Privacy Protection Act of 1998
  (15 U.S.C. § 6501-6506).
- **KOSA** — Kids Online Safety Act of 2024 (US S.1409).
- **DMCA designated agent** — US Copyright Office DMCA Designated
  Agent Directory (per 17 U.S.C. § 512(c)(2)).
- **EU Digital Services Act** — Regulation (EU) 2022/2065.
- **EU AI Act** — Regulation (EU) 2024/1689.
- **PCI-DSS 4.0** — Payment Card Industry Data Security Standard
  v4.0 (2024).
- **HIPAA Security Rule** — 45 CFR § 164.308 + § 164.312.

### §G.4. Bug bounty + abuse-reporting precedents

- **HackerOne** — hackerone.com (canonical bug-bounty platform).
- **Bugcrowd** — bugcrowd.com (canonical bug-bounty platform).
- **Thinkst Canary** — canarytokens.org (canonical canary token
  primitive).

### §G.5. Internal portfolio ADRs

- ADR-0028 — audit-chain Merkle-sealed.
- ADR-0044 — service mesh + mTLS.
- ADR-0105 — 13-layer canonical enum.
- ADR-0145 — inter-µservice communication reform.
- ADR-0148 — service mesh canonical: Cilium ambient.
- ADR-0188 — passkey-webauthn-as-canonical-auth.
- ADR-0211 — in-house tech-stack preference.
- ADR-0212 — buildability doctrine.
- ADR-0242 — oyatie-is-a-tenant doctrine.
- ADR-0243 — Cedar as universal gate.
- ADR-0244 — tenant as universal scoping primitive.
- ADR-0245 — substrate vs product layering.
- ADR-0246 — policy-engine substrate promotion (+ library-first
  amendment).
- ADR-0247 — self-hosting / self-modification doctrine.
- ADR-0248 — Amazon-shape cellular architecture.
- ADR-0250 — build-ahead-of-certification doctrine.
- ADR-0251 — compliance-pack cell certification levels.
- ADR-0253 — network topology — Anycast + edge POPs + Cilium
  ambient + ECH + PQC.
- ADR-0258 — API versioning + SemVer policy.
- ADR-0263 — observability emission contract.
- ADR-0272 — cookie consent per-purpose.
- ADR-0273 — per-tenant DKIM/SPF/DMARC email deliverability.
- ADR-0276 — backup portability + GDPR Article 20.
- ADR-0292 — minor user doctrine (COPPA + KOSA + EU AADC).
- ADR-0293 — Foundry meta-trust-root.
- ADR-0295 — bootstrap CI SPIFFE + kill-switch.
- ADR-0296 — library-first credential sidecar.

### §G.6. Standards docs

- `docs/standards/documentation-rigor.md` — §3.2.3 codifies the
  24-row abuse-defence taxonomy this ADR binds.
- `docs/standards/fips-hsm-substrate-root-signing.md` — Tier 1
  intermediate key for Cedar fragment signing.
- `docs/standards/cedar-policy-discipline.md` — Cedar v4.2 LTS
  authoring conventions.
- `docs/standards/step-up-auth-classes.md` — Class A..F step-up
  taxonomy.
- `docs/standards/doc-style.md` — Diátaxis quadrants + frontmatter
  shape.

### §G.7. Auto-memory feedback (related)

- `feedback_quality_performance_scalability_bar` — hyperscaler-grade
  bar driving substrate-first abuse-defence.
- `feedback_clean_architecture_requirements` — 12-layer enum +
  shared substrate layer 5.
- `feedback_no_silent_regression` — no per-µservice abuse defence
  drift.
- `feedback_autonomous_implementation_artifacts` — intern-buildable
  baseline from one substrate.
- `feedback_canonical_base_localization` — per-pack abuse-defence
  overlays (GDPR, COPPA, KOSA, KR-CSAP).
- `feedback_oyatie_is_a_tenant_doctrine` — oyatie's own surfaces
  subject to the baseline.
- `feedback_cedar_as_universal_gate` — abuse-defence composes as
  Cedar fragment.
- `feedback_amazon_shape_cellular_architecture` — per-cell-tier
  variants per §D-5.
- `feedback_compliance_pack_primitive` — pack overlays per §D-8.
- `feedback_naming_justification` — front-matter naming-justifications
  block.

## §H. Change log

- 2026-05-20: Initial publication. Bundled with the keystone bundle
  2026-05-20 foundational doctrine; closes the gap identified in
  `docs/standards/documentation-rigor.md` §3.2.3. Authored as
  the promotion-gate-fix-abuse-defence-baseline keystone.
