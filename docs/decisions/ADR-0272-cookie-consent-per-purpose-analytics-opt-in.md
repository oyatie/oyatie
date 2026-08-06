---
id: ADR-0272
status: Accepted
date: 2026-05-20
owners:
  - council-privacy
  - council-legal
  - council-architecture
  - council-security
  - council-frontend
  - ops-compliance
  - ops-sre-reliability
  - axis-consent-ledger
  - axis-audit-chain
  - axis-policy-engine
  - axis-tenancy
  - axis-identity
  - axis-analytics
  - axis-i18n-localization
supersedes: []
amends:
  - ADR-0099-data-class-registry.md (extends data-class taxonomy with consent-purpose classes)
  - ADR-0251-compliance-pack-cell-certification-levels.md (registers `cookie-consent` as required evidence in EU-GDPR, EU-ePrivacy, KR-PIPA, KSA-PDPL, US-CCPA-CPRA, and BR-LGPD packs)
superseded_by: []
related:
  - ADR-0010-regional-pack-architecture.md
  - ADR-0049-cross-region-replication-and-residency.md
  - ADR-0064-canonical-base-and-localization-packs.md
  - ADR-0099-data-class-registry.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0140-cedar-policy-enforcement.md
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
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0255-intelligence-as-two-layer-ai-substrate.md
related_specs:
  - /specs/microservices/consent-ledger.json
  - /specs/microservices/audit-chain.json
  - /specs/microservices/policy-engine.json
  - /specs/microservices/analytics-aggregator.json
  - /specs/microservices/tenancy.json
  - /specs/microservices/governance.json
  - /specs/microservices/i18n-localization.json
  - /specs/cookie-consent-schema.json
  - /specs/consent-purpose-taxonomy.json
  - /specs/cmp-wcag-accessibility.json
  - /specs/gpc-signal-handling.json
  - /specs/compliance-pack-schema.json
related_memory:
  - feedback_canonical_base_localization
  - feedback_no_silent_regression
  - feedback_doc_coverage_enforced
  - feedback_quality_performance_scalability_bar
  - feedback_automate_everything
  - feedback_clean_architecture_requirements
  - feedback_workflow_objectgraph_adapter_layer
  - feedback_autonomous_implementation_artifacts
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-privacy-surface-keystone
keystone_position: 1-of-1
purpose: >
  Establish Tier-1 cookie consent and per-purpose analytics opt-in as
  the canonical privacy surface for every oyatie web/mobile property.
  Mandates GDPR + ePrivacy + KR-PIPA + KSA-PDPL + US-CCPA/CPRA
  + BR-LGPD compliance simultaneously through a per-jurisdiction
  overlay that resolves to the strictest applicable rule. Pre-checked
  opt-ins are forbidden. Cookie-less analytics is the default. The
  CMP (Consent Management Platform) is a substrate-layer concern
  with first-party-only cookies, no third-party trackers,
  Global Privacy Control honored, IAB TCF v2.2 reserved for any
  future ad-tech surface (currently scoped out), and per-tenant
  policy overlays so B2B tenants set their workforce policy.
enforcement_status: blocker-on-keystone-merge
enforced_by:
  - cloud-ci/Rust gate packet cookie-consent-schema
  - cloud-ci/Rust gate packet consent-purpose-taxonomy
  - cloud-ci/Rust gate packet cmp-wcag-2-2-aa
  - cloud-ci/Rust gate packet gpc-signal-honored
  - cloud-ci/Rust gate packet no-pre-checked-opt-ins
  - cloud-ci/Rust gate packet cookie-less-analytics-default
  - cloud-ci/Rust gate packet first-party-cookie-only
  - cloud-ci/Rust gate packet per-jurisdiction-overlay-strictest-wins
  - cloud-ci/Rust gate packet consent-revocation-one-click
  - cloud-ci/Rust gate packet consent-audit-chain-coverage
  - cloud-ci/Rust gate packet tenant-policy-override-surface
  - cloud-ci/Rust gate packet dark-pattern-lint
---

> **Disposition light-edit (2026-08-06):** Cookie consent / purpose analytics — privacy substrate

# ADR-0272: Cookie Consent + Per-Purpose Analytics Opt-In

## Status

Proposed — 2026-05-20.

Tier-1 privacy lockdown. This ADR is a BLOCKER class change: until the
CMP substrate ships and the twelve enumerated `cloud-ci/Rust gate packet` lanes
turn green, no oyatie web or mobile property may serve traffic to an
end-user from a jurisdiction in scope (EU/EEA, UK, KR, KSA, US-CA,
US-CO, US-CT, US-VA, US-UT, US-TX, US-IA, US-MT, US-IN, US-OR, US-DE,
US-NJ, US-FL (limited), US-WA (limited), BR, CH, IS, NO, LI, plus
sector-specific overlays where applicable).

The doctrine is accepted in text now; CI lanes mature over the next
two release windows. The ADR captures the design contract today so
downstream IPs (implementation plans) can begin scaffolding the
`consent-ledger`, `cmp-runtime`, `gpc-signal-handler`, and
`analytics-aggregator` microservices in lock-step with the legal
review.

Bundled as the first member of the
`2026-05-20-privacy-surface-keystone` bundle. Future privacy ADRs
(per-purpose DSAR routing, automated decision-making notice, ePrivacy
Regulation 2024 ratification overlay) will reference this ADR as the
substrate they extend.

## Context

oyatie is the open-source parallel to Bominal: a multi-product,
multi-tenant SaaS substrate hosting Workflow Studio, Ontology, Cloud
products, and a marketplace. Every product surface that touches an
end-user web/mobile session faces an immediate convergence of
overlapping privacy regimes:

1. **EU GDPR (Regulation 2016/679)** — requires lawful basis for any
   processing of personal data; for cookies that are NOT strictly
   necessary, the lawful basis is overwhelmingly consent (Article 6(1)(a))
   because legitimate interest (Article 6(1)(f)) is constrained by
   recital 47 and EDPB guidance 8/2020 to a narrow band of operational
   diagnostics. Consent must be freely given, specific, informed,
   unambiguous, and given by a clear affirmative action (Article 4(11)).
   Withdrawing consent must be as easy as giving it (Article 7(3)).
2. **ePrivacy Directive 2002/58/EC (as amended 2009/136/EC)** — the
   "cookie law". Article 5(3) requires prior informed consent before any
   information is stored on or accessed from a user's terminal
   equipment, regardless of whether that information is personal data
   under GDPR. The two exemptions are (a) strictly necessary for an
   information society service explicitly requested by the user, and
   (b) sole purpose of carrying out transmission. Analytics, A/B
   testing, personalization, and marketing all fall outside both
   exemptions.
3. **ePrivacy Regulation (2024 draft, expected ratification 2026-2027)**
   — replaces the directive with a directly-applicable regulation;
   tightens the strictly-necessary exemption further; explicitly
   recognises "audience measurement" as a narrow legitimate-interest
   carve-out IF AND ONLY IF it is first-party, aggregate, non-tracking,
   and per-website. This is the legal hook for D-2 below.
4. **Korean PIPA (Personal Information Protection Act, 2011, amended
   2020 and 2023)** — Article 15 requires consent for collection of
   personal information. Article 22 requires that consent for
   different purposes be obtained separately. Article 22-2 (2023
   amendment) introduces mandatory granular consent UI and forbids
   bundling. The 2023 amendment also formalizes the "right to be
   informed of automated decisions" and the right to refuse them.
   PIPC enforcement actions in 2024-2025 have explicitly cited
   pre-checked boxes and "accept all" dark patterns as violations
   carrying fines up to 3% of global turnover.
5. **KSA PDPL (Personal Data Protection Law, Royal Decree M/19 of
   2021, in force 2023-09-14, executive regulations 2023-09-07)** —
   Article 5 makes consent the default lawful basis. Article 6
   imposes explicit-consent and granularity requirements. Article
   29 of the executive regulations requires that consent be
   "specific, informed, and free", recorded, and revocable. The
   National Data Management Office (NDMO) interpretive bulletins
   2024 and 2025 align KSA closely with GDPR in substance while
   adding Sharia-compatible processing constraints (e.g. processing
   for religious classification requires heightened consent).
6. **US CCPA + CPRA (effective 2023-01-01, regulations finalized
   2023-03-29, updated 2025)** — does not require consent for most
   processing but DOES require an opt-out from "sale" and
   "sharing" (where sharing now explicitly includes cross-context
   behavioural advertising). CPRA Section 1798.135(b)(1) requires
   a "Do Not Sell or Share My Personal Information" link OR
   recognition of an opt-out preference signal. The California
   Privacy Protection Agency clarified 2024-04 that Global Privacy
   Control (GPC) IS such a preference signal and MUST be honored.
7. **Other US state laws** (Colorado CPA, Connecticut CTDPA,
   Virginia VCDPA, Utah UCPA, Texas TDPSA, Iowa ICDPA, Montana
   CDPA, Indiana INCDPA, Oregon OCPA, Delaware DPDPA, New Jersey
   NJDPA, Tennessee TIPA, Minnesota MNCDPA, Maryland MODPA, etc.) —
   converging on opt-out-with-universal-signal model. All requested
   in scope.
8. **Brazilian LGPD (Lei 13.709/2018)** — Article 8 requires consent
   for non-essential processing; Article 9 requires the consent
   record include the specific purpose. ANPD enforcement aligns
   substantively with GDPR.
9. **UK GDPR + PECR (Privacy and Electronic Communications
   Regulations 2003 as amended 2019)** — post-Brexit divergence is
   minimal; treat as GDPR-equivalent overlay with UK ICO guidance
   on cookies (2023 update) as the authoritative interpretation.
10. **Swiss revFADP (in force 2023-09-01)** — adequacy-aligned with
    GDPR but with narrower data-subject rights and slightly different
    consent recordkeeping requirements; in scope.

Across this matrix, the inescapable structural conclusions are:

- The strictest applicable rule MUST win at runtime, because every
  alternative (selecting per-jurisdiction subsets) creates a discovery
  attack surface that lets regulators trivially identify violations.
- Pre-checked opt-ins are universally forbidden (GDPR recital 32, PIPA
  Article 22, KSA PDPL executive regulations Article 29, PIPC dark
  pattern guidance 2024).
- "Accept all" as a default action is a dark pattern under at least
  CNIL guidance (2021), datatilsynet guidance (2022), PIPC guidance
  (2024), and the EDPB Dark Patterns Guidelines 03/2022.
- Cookie-less analytics that meet the audience-measurement carve-out
  in ePR draft Article 8 (first-party, aggregate, non-tracking,
  per-website, no cross-site profiling) are an attainable default
  that minimises consent friction without legal risk.
- Consent must be machine-verifiable; "we asked nicely" is not
  evidence. Consent records must be immutable, signed, and
  reproducible.

The current state of oyatie has NO cookie consent surface, NO consent
ledger, NO CMP substrate, NO per-purpose analytics opt-in mechanism,
and NO GPC handler. We are pre-traffic, which is the cheapest possible
moment to install Tier-1 lockdown. Delaying past first end-user
traffic creates an asymmetric compliance debt: every analytics event,
every A/B test impression, every personalization signal collected
without proper consent becomes a discrete regulatory liability that
must later be retroactively cured or purged.

This ADR locks in the design contract before that traffic begins.

## Decisions

### D-1: Per-purpose granular consent — five canonical purposes

oyatie's CMP exposes exactly FIVE consent purposes, in this strict
canonical order:

1. **necessary** — strictly necessary cookies (session token, CSRF
   token, load-balancer affinity, language preference for current
   session, accessibility-mode preference, consent-state cookie
   itself). No consent prompt required. ePrivacy Article 5(3)
   strictly-necessary exemption.
2. **preference** — non-essential settings that persist across
   sessions: theme (light/dark/system), per-product layout pinning,
   default landing tab, currency display preference, timezone
   override. Consent required. Default = OFF.
3. **statistics** — first-party aggregate audience measurement.
   ONLY granted IF cookie-less-by-default (see D-2) cannot satisfy
   the statistical objective, OR if the tenant has explicitly
   pinned a richer analytics overlay. Default = OFF.
4. **marketing** — cookies and pixels for advertising, retargeting,
   campaign attribution, cross-property attribution. Default = OFF.
   Note: oyatie's first-party product surface currently has no
   marketing pixels by design (D-6 + D-8); this purpose is reserved
   for any future ad-tech surface.
5. **personalization** — content recommendation, model fine-tuning
   signals, behavioural product UX adjustments. Default = OFF.
   This is the purpose under which any AI-driven personalization
   loops collect signal; ties to ADR-0255 (two-layer AI substrate)
   for the data residency boundary.

These five and ONLY these five are exposed. Sub-purposes are
collapsed into the parent. Adding a sixth purpose requires a new
ADR plus pack-registry pin so regulators can verify the taxonomy
hasn't drifted silently. This closed enum design follows the same
discipline as ADR-0099 (data class registry) and ADR-0105 (thirteen
layer enum).

Multiple toggles map cleanly to PIPA Article 22's mandatory
separation requirement, GDPR recital 32's specificity requirement,
KSA PDPL Article 6's purpose-specific requirement, and LGPD
Article 9's specific-purpose-record requirement. The closed enum
keeps the audit surface stable.

### D-2: Cookie-less analytics by default

oyatie's first-party product analytics layer is cookie-less by default.
It uses:

- Daily-rotated anonymous identifiers derived from a salted hash of
  (IP-prefix + User-Agent-class + acceptance-language-class), where
  the salt rotates every 24 hours and is never persisted past 25
  hours. This produces a same-day session identifier that cannot
  link across days, cannot identify the user, and cannot be cross-
  referenced with any other dataset.
- Aggregate per-day rollups (count of sessions per page, count of
  unique-per-day visitors, percentile response times) that are
  computed inside the daily rotation window and persisted only as
  aggregates.
- No persistent client-side storage (no cookies, no localStorage,
  no sessionStorage beyond strictly-necessary), no fingerprinting
  signals (no Canvas API, no AudioContext, no WebGL, no font
  enumeration, no device-pixel-ratio precision capture).
- No cross-site or cross-property linking.

This pattern satisfies the audience-measurement carve-out in the
ePrivacy Regulation draft Article 8(1)(c) and the corresponding
CNIL guidance on audience measurement (March 2022) plus the ICO's
August 2024 guidance on analytics cookies. PIPC Korea aligns with
this approach in 2024 guidance distinguishing personal information
from anonymous statistics.

Because cookie-less default analytics does NOT require consent under
any of the in-scope regimes (it is either non-personal data or
covered by a narrow audience-measurement carve-out), it does NOT
appear in the CMP prompt. The user is informed via the privacy
notice but not asked to consent because consent is not the lawful
basis.

If a tenant pins a richer analytics overlay (e.g. an enterprise
tenant electing Adobe Analytics, GA4, Amplitude, or PostHog
client-side for their workforce-facing surface) that pack imports
the "statistics" purpose and consent IS required from that
tenant's end-users. Per-tenant policy override (D-11) governs
this surface.

### D-3: Granular UI — separate toggle per purpose; no "accept all" default

The CMP modal MUST render exactly four toggles (necessary is shown
informational-only, always-on, not interactive) in this layout:

```
[ Cookie & data preferences                            X ]

  We need your consent to set cookies and process data for
  the purposes below. You can change these any time from
  the privacy menu in your account settings.

  [ Necessary       ON  (always)  ]   info
  [ Preference      OFF           ]   info
  [ Statistics      OFF           ]   info
  [ Marketing       OFF           ]   info
  [ Personalization OFF           ]   info

  ( Save my choices )   ( Reject non-essential )   ( Accept selected )
```

- Default state of every non-necessary toggle = OFF.
- The "Save my choices" button reflects whatever the user has set.
- The "Reject non-essential" button is visually equal in prominence
  to any "accept" affordance — same colour weight, same size, same
  font, same disability/hover state. (CNIL guidance 2021,
  datatilsynet 2022, PIPC 2024.)
- There is NO "Accept all" button by default. The pattern of a
  prominent "Accept all" with a deprioritised "Reject all" is a
  dark pattern under EDPB Dark Patterns Guidelines 03/2022 and is
  banned by CNIL enforcement decisions (Google 150M EUR fine 2022,
  Meta 60M EUR fine 2022). oyatie does not ship that pattern at all.
- An "Accept selected" button confirms the currently toggled state.
- An "Accept all" button MAY be tenant-enabled per D-11 ONLY if
  the tenant has signed acknowledgement that they accept the legal
  risk and ONLY if the tenant's jurisdiction profile does not
  forbid it. The default tenant policy is to not enable it.

WCAG 2.2 AA conformance (D-9) is mandatory.

### D-4: Consent persistence in tenant audit chain

Every consent action is persisted as an immutable signed record in
the tenant's audit chain (ADR-0246 audit-chain substrate). The
record schema is:

```json
{
  "schema": "consent-record/v1",
  "tenant_id": "<tenant>",
  "actor": {
    "kind": "end-user",
    "subject_id": "<pseudonymous-stable-id>",
    "jurisdiction": "<ISO-3166-2-or-country>",
    "preferred_locale": "<BCP-47>"
  },
  "consent": {
    "necessary": true,
    "preference": false,
    "statistics": false,
    "marketing": false,
    "personalization": false
  },
  "ui": {
    "cmp_version": "<semver>",
    "presented_locale": "<BCP-47>",
    "presented_text_hash": "<sha-256>",
    "interaction_kind": "click|gpc-signal|api-revocation|tenant-policy-default"
  },
  "context": {
    "source_ip_prefix": "<truncated-prefix>",
    "user_agent_class": "<class>",
    "page_uri": "<scrubbed-path-only>",
    "applied_jurisdiction_overlay": "EU-GDPR|EU-ePrivacy|KR-PIPA|KSA-PDPL|US-CCPA|US-CPRA|BR-LGPD|UK-GDPR|CH-revFADP|...",
    "tenant_policy_overlay_hash": "<sha-256>"
  },
  "issued_at": "<RFC3339-UTC>",
  "expires_at": "<RFC3339-UTC>",
  "signature": "<ed25519-by-tenant-signing-key>",
  "prior_record_ref": "<cas-hash-of-prior-record-or-null>"
}
```

Records form an append-only chain per `subject_id` so the full
history is reconstructible. The audit chain itself is replicated
per ADR-0049 residency rules; the consent record never leaves the
data-residency boundary of the subject's jurisdiction.

The `expires_at` defaults to twelve months from `issued_at` to
satisfy CNIL guidance that consent re-confirmation be sought at
least annually for non-essential cookies. KR-PIPA does not mandate
a fixed re-confirmation interval but the CMP defaults align across
jurisdictions for consistency.

### D-5: Consent revocation — one-click, immediately effective

A standing user-accessible affordance lets the subject revoke any
purpose at any time. Revocation paths:

- **CMP-modal re-open** — every page footer carries a "Cookie & data
  preferences" link that reopens the same modal as initial
  presentation. Required by GDPR Article 7(3) ("withdrawing consent
  shall be as easy as giving it") and PIPA Article 22(4).
- **In-app settings → Privacy → Cookie & data preferences** — for
  signed-in subjects.
- **API endpoint** `DELETE /consent/{subject_id}` (subject-authenticated)
  — for headless or scripted revocation.
- **GPC signal mid-session** (D-10) — re-evaluated on every request;
  emergence of the signal after consent equals revocation of
  consented opt-in purposes for which GPC is interpretively binding
  (marketing + statistics under CCPA/CPRA; treated as advisory but
  honored for GDPR/PIPA jurisdictions).

Revocation MUST be immediate in effect: the consent-ledger entry is
written first, then a fan-out invalidates every cached purpose
decision within the next request cycle. The implementation must
not rely on TTL expiry of cached consent state; the consent-ledger
publishes an invalidation event on the substrate event bus
(per ADR-0145) and every consumer drops cached state on receipt.

Already-collected data tagged with revoked purposes is purged or
re-anonymised per the data-class registry policy (ADR-0099).

### D-6: First-party-only cookies; no third-party trackers

oyatie does NOT load third-party scripts, third-party cookies,
third-party pixels, third-party fonts, third-party CDN-hosted
analytics, third-party CAPTCHA (or, where present, uses
self-hosted alternatives), or third-party social embeds unless:

1. The asset is necessary for an explicitly-requested information
   society service (e.g. embedded payment iframe from a regulated
   PSP, but ONLY upon user navigation to a checkout flow), AND
2. The user has consented to the specific purpose the third party
   serves, AND
3. The third party is contractually bound by a DPA (Data Processing
   Addendum / Article 28 GDPR / equivalent KSA/KR/US/BR clause), AND
4. The third party appears in the tenant's published privacy notice
   under the corresponding purpose, AND
5. The third party is registered in the substrate's vendor registry
   with current sub-processor status, transfer mechanism (SCCs,
   adequacy decision, BCRs), and data class registry mapping.

The default product surface ships zero third-party tags. Fonts are
self-hosted. CAPTCHA is self-hosted (Cap.js or equivalent
in-house). Analytics are first-party (D-2). Maps, where present,
are tile-server first-party. Video embeds, where present, are
self-hosted or tenant-pinned.

This is the most operationally consequential decision in this ADR
because it forecloses entire categories of common SaaS UX shortcuts
(Intercom, Hotjar, Pendant, Google Maps, YouTube embeds with
classic player, Facebook share buttons). The substrate compensates
by providing self-hosted equivalents under in-house ADR-0211.

### D-7: Per-jurisdiction overlay — strictest wins

The CMP runtime computes the applicable jurisdiction overlay at
request time, using:

1. The subject's declared residency (if signed in and recorded).
2. The subject's GPC signal (US states).
3. The subject's IP-geolocation (best-effort, never persisted).
4. The tenant's pinned override (D-11) for B2B workforce surfaces.
5. The strictest applicable rule WINS across the overlays produced
   in (1)-(4).

The overlay precedence rules are:

- If GPC is set → CCPA/CPRA opt-out is recorded regardless of
  geolocation (CPPA 2024 ruling: extraterritorial honoring is
  permissible; oyatie chooses to honor extraterritorially for
  consistency).
- If GDPR/ePrivacy applies → strict consent required for all
  non-essential; "Reject" path MUST be present; no pre-ticked;
  expiry ≤ 12 months.
- If KR-PIPA applies → separate-purpose toggles; no bundling;
  automated-decision notice; consent record evidentiary
  preservation ≥ 5 years post-revocation.
- If KSA-PDPL applies → explicit consent in Arabic with locale
  parity; Sharia-compatible-processing notice for any religious
  data class; controller registration display.
- If BR-LGPD applies → controller + DPO + specific-purpose record;
  data subject rights notice in Portuguese with locale parity.

"Strictest wins" resolution algorithm:

```
function resolveOverlay(overlays):
    # purpose default = OFF if any overlay says OFF default
    # text-presentation requirements = union of all overlay requirements
    # consent-record fields = union of all overlay-required fields
    # expiry = min(all overlay expiries)
    # revocation requirements = union of all overlay requirements
    # display-language requirements = union (parity required)
```

The resolved overlay is materialised as a hash and pinned in the
consent record (`context.tenant_policy_overlay_hash` +
`context.applied_jurisdiction_overlay`) so the determination is
reproducible during regulator audit.

### D-8: IAB TCF v2.2 reserved for future ad-tech surfaces

The Interactive Advertising Bureau Transparency and Consent
Framework v2.2 (effective 2023-11-20, schema updates 2024) is the
de-facto interop standard for cross-vendor ad-tech consent
signalling. oyatie's current product surface has no ad-tech
component (per D-6) and therefore TCF integration is OUT OF SCOPE
for the initial CMP substrate.

However, the CMP runtime is designed with a clean integration seam
so that if a future product surface emerges that genuinely requires
ad-tech (e.g. a marketplace-listing sponsored-placement surface
under ADR-0249), the substrate can add TCF v2.2 (or whichever
version is current at that time) as a layered emitter without
re-architecting:

- The consent ledger schema includes optional fields for
  `tcf_string` and `gpp_string` (Global Privacy Platform) that
  remain empty until ad-tech surfaces emerge.
- The substrate event bus admits a `consent.tcf.v2_2.emitted`
  topic reserved for future emission.
- The vendor registry schema includes optional fields for IAB
  Vendor ID and IAB Stack ID.

This is a forward-compatibility seam, not a current commitment.
The decision is to NOT ship TCF v2.2 today and to NOT require any
product surface to depend on TCF semantics; if and when ad-tech
emerges, a follow-up ADR will activate the seam with full
regulator review.

### D-9: Cookie banner design — WCAG 2.2 AA accessible; not a dark pattern

The CMP UI MUST conform to WCAG 2.2 Level AA, specifically:

- 1.4.3 Contrast (minimum) — 4.5:1 for body text, 3:1 for large
  text and UI components.
- 1.4.11 Non-text Contrast — UI components 3:1.
- 1.4.12 Text Spacing — accommodate user-overridden spacing.
- 1.4.13 Content on Hover or Focus — info tooltips dismissable,
  hoverable, persistent.
- 2.1.1 Keyboard — every toggle reachable and operable.
- 2.4.3 Focus Order — logical and matches visual order.
- 2.4.7 Focus Visible — visible focus indicator on every control.
- 2.4.11 Focus Not Obscured (Minimum) — focused control fully
  visible.
- 2.5.7 Dragging Movements — no drag-only patterns; all toggles
  click/tap operable.
- 2.5.8 Target Size (Minimum) — 24x24 CSS px minimum touch target.
- 3.2.6 Consistent Help — privacy help link in consistent location
  across pages.
- 3.3.7 Redundant Entry — consent state not re-prompted within
  session.
- 4.1.2 Name, Role, Value — every toggle exposes accessible name,
  role (switch), and current value to assistive tech.

The CMP is also screen-reader audited (NVDA, VoiceOver, JAWS,
TalkBack) every release.

The CMP is dark-pattern-linted at build time. The lint rules
include:

- No button styled to look disabled or de-emphasised when its
  meaning is "reject".
- No "Accept all" prominence above "Reject all" or "Save my
  choices".
- No copy framing rejection as a loss ("you'll miss out on...").
- No copy framing acceptance as agreement to "improve the service"
  without specificity.
- No nudging language that implies the user must accept.
- No mandatory acceptance to dismiss the modal; "Reject
  non-essential" must always dismiss the modal.
- No re-prompting within the consent's validity period unless the
  taxonomy expands (D-1).
- No geofencing that hides the prompt in some jurisdictions and
  shows it in others (consistent presentation, varying outcomes
  via D-7 overlay).

This lint is enforced by `cloud-ci/Rust gate packet dark-pattern-lint` and
is BLOCKER class.

### D-10: Global Privacy Control (GPC) signal honored per CCPA

Per California Privacy Protection Agency clarification (April
2024), GPC signals are an enforceable opt-out preference signal
under CCPA Section 1798.135(b)(1). oyatie's CMP runtime:

- Reads the `Sec-GPC: 1` HTTP request header on every request.
- Reads the `navigator.globalPrivacyControl === true` browser
  property at CMP-load time.
- When detected, treats the subject as having opted out of "sale"
  and "sharing" under CCPA/CPRA and as having declined the
  `marketing` and `personalization` purposes under the canonical
  taxonomy.
- For non-US jurisdictions, GPC is honored advisorily (treated as
  a signal of refusal for `marketing` and `personalization` but
  still presents the modal so the subject can affirmatively
  consent if desired — this is conservative because GDPR/PIPA do
  not yet legally bind controllers to GPC, but CNIL's 2024 update
  signals movement that direction).
- A GPC-driven decision is recorded in the consent ledger with
  `ui.interaction_kind = "gpc-signal"`.
- A GPC-driven decision can be overridden by an affirmative
  in-CMP click ONLY in non-US jurisdictions; in US jurisdictions
  where GPC is binding, the override is forbidden and the
  marketing/personalization toggles are locked OFF.

The Global Privacy Platform (GPP) string format (IAB tech-lab spec
1.1, 2024) is supported as a forward-compatible emission target;
the substrate emits GPP when a TCF-equivalent surface is active
(D-8) but does not require GPP today.

### D-11: Per-tenant policy override

oyatie is a multi-tenant substrate (ADR-0244). B2B tenants are
themselves controllers for their workforce-facing surface. The
substrate exposes a per-tenant policy override surface that allows
a tenant administrator to:

- Pin a stricter overlay than the user's jurisdiction overlay
  (e.g. a German tenant pinning EU-GDPR + ePrivacy as the
  minimum even for workforce travelling outside the EU).
- Pre-set their workforce's allowed purposes (e.g. require
  `preference` and `statistics` OFF by default but ALLOW
  workforce to consent in-app).
- Forbid certain purposes outright (e.g. healthcare tenants
  forbidding `marketing` and `personalization` substrate-wide).
- Enable "Accept all" surfaces only if their jurisdiction profile
  permits it and they sign acknowledgement (D-3).
- Designate per-jurisdiction sub-policies for multinational
  workforces.

Tenant policy overrides are themselves recorded in the audit
chain (per-tenant signed governance records). The override never
LOOSENS a regulator-required protection; the substrate enforces
the strictest-wins semantic across tenant + jurisdiction overlays.

This is the same overlay pattern as ADR-0064 (canonical base +
localization packs) applied to consent surfaces, and it composes
cleanly with ADR-0218 (tenant granular control surface).

### D-12: Audit trail per consent action

Every consent action — first acceptance, modification, revocation,
expiry-driven re-prompt, GPC-driven decision, tenant-policy update,
substrate-version migration of the consent record — is recorded as
a discrete signed entry in the tenant's audit chain.

The audit trail satisfies:

- GDPR Article 7(1) — controller must demonstrate consent was
  obtained.
- PIPA Article 22(5) — controller must retain evidence of consent.
- KSA PDPL executive regulations Article 29 — consent records must
  be retrievable for at least the duration of processing plus the
  applicable statute-of-limitations period.
- LGPD Article 8(2) — burden of proof of consent rests on the
  controller.
- US state laws — opt-out records must be retained for at least
  24 months (CPRA implementing regulations).

Audit records are retained for the greater of (a) regulator-
required period, (b) statute-of-limitations + 1 year, or (c)
the tenant's pinned retention overlay. The audit chain itself
inherits replication/residency from ADR-0049 (cross-region
replication and residency).

The audit trail is queryable by the data subject for their own
records (Article 15 GDPR access right + PIPA Article 35 right of
access + KSA PDPL Article 14 right of access + LGPD Article 18
right of access + CCPA right to know).

## Alternatives Considered

### Alt-1: Single global "Accept all / Reject all" surface (rejected)

The alternative is the industry-default "two-button" consent banner:
`[ Reject all ] [ Accept all ]`. It is the simplest to implement
and the most common pattern across SaaS competitors.

Rejected because:

- "Accept all" cannot be bundled with rejection-of-bundled-purposes
  under GDPR Article 7(2) (granularity) and PIPA Article 22-2
  (separation requirement).
- "Accept all" defaults — even when phrased neutrally — produce
  ~95% acceptance under behavioural economics literature (Acquisti
  et al. 2017, Utz et al. 2019), which is treated by CNIL and
  EDPB as evidence the consent was not freely given.
- The pattern has produced multiple eight-figure fines (Google
  150M EUR, Meta 60M EUR, Amazon 35M EUR via CNIL).
- The pattern violates KR-PIPA dark-pattern guidance 2024.
- The pattern conflicts with the autonomous-decision principle in
  feedback_autonomous_decision_principles (long-term right >
  short-term cost): the apparent cost is converted into a
  liability + regulatory tail.

### Alt-2: Third-party CMP vendor (e.g. OneTrust, Cookiebot, Usercentrics) (rejected)

The alternative is to license a commercial CMP and embed its
script on every oyatie property.

Rejected because:

- It violates D-6 (no third-party scripts).
- It violates ADR-0211 (in-house tech stack preference).
- It violates the substrate-vs-product layering doctrine
  (ADR-0245): consent is a substrate-layer concern that must be
  owned by oyatie's own audit chain.
- Vendor CMPs frequently inject their own analytics and consent-
  state tracking that compose into a meta-consent problem.
- Vendor lock-in risk: the consent ledger schema must remain
  oyatie's IP and oyatie's authoritative record.
- Per-jurisdiction overlay logic (D-7) is sensitive enough that
  it must be co-designed with oyatie's legal and privacy
  councils, not outsourced.
- Vendor pricing scales adversely with multi-tenant deployments
  where each tenant has separate jurisdiction profiles.

### Alt-3: Consent-required-for-everything (rejected)

The alternative is to treat every cookie and every analytic event
as requiring consent, including the cookie-less aggregate
analytics.

Rejected because:

- The ePR draft Article 8(1)(c) audience-measurement carve-out is
  explicitly designed to permit cookie-less aggregate first-party
  analytics without consent. Demanding consent for them creates
  user friction without legal benefit.
- It would degrade product UX (constant consent prompts) and
  reduce baseline operational visibility (we'd lose aggregate
  per-page metrics for users who declined).
- The PIPC and CNIL guidance both distinguish anonymous
  aggregate measurement from personal-data processing; treating
  them identically violates the principle of data minimisation
  inverted (we'd be over-collecting consent records that
  themselves contain personal data).

### Alt-4: Consent-only-for-EU (rejected)

The alternative is to geofence the consent prompt — present it to
EU/UK users only, default-on for non-EU.

Rejected because:

- Violates D-7 (strictest wins).
- Creates a discovery attack surface for regulators outside the
  EU (PIPC, CPPA, NDMO, ANPD) who would treat geofenced absence
  as evidence of intentional non-compliance.
- Creates dual code paths that diverge in subtle ways and are
  hard to keep in sync.
- Violates feedback_no_silent_regression — the consent contract
  must be uniform.
- Misreads the regulatory trajectory: KR-PIPA, KSA-PDPL, BR-LGPD,
  and US state laws are converging on GDPR-substance, not
  diverging. Geofenced compliance is a fixed cost that has to be
  unwound later.

### Alt-5: Defer until first end-user traffic (rejected)

The alternative is to ship product surfaces without a CMP and add
one "when needed".

Rejected because:

- Every analytic event collected without proper consent becomes a
  retroactive liability. The cheap moment is pre-traffic.
- The autonomous-implementation goal (feedback_autonomous_-
  implementation_artifacts) requires the substrate to be
  complete without further user intervention; CMP is part of
  that completeness.
- The keystone-bundle pattern (per ADR-0251) is designed exactly
  to avoid this kind of deferral.

### Alt-6: Outsource jurisdiction overlay to a CMP-as-a-service API (rejected)

The alternative is to use a hosted API like Iubenda or
TrustArc for jurisdiction overlay logic while owning the rest.

Rejected on the same axes as Alt-2 plus: the per-jurisdiction
overlay is precisely the substrate's IP — its proper resolution
embodies oyatie's legal/privacy doctrine and must be reviewable
end-to-end.

### Alt-7: Full IAB TCF v2.2 integration today (rejected — see D-8)

The alternative is to ship TCF v2.2 emission today as a
forward-compatibility hedge.

Rejected because TCF semantics are designed for ad-tech vendor
interop and impose specific consent-string formats that bias the
UI toward ad-tech patterns. Without an ad-tech surface, TCF
emission has no consumer and only adds complexity. The seam in
D-8 keeps the door open without forcing the pattern.

### Alt-8: Pop-up modal vs full-page interstitial (rejected — modal chosen)

The alternative is a full-page interstitial that blocks all
product UI until consent is decided.

Rejected because the interstitial pattern, while compliant, is
the worst UX choice and is criticised by CNIL 2024 guidance as
coercive even when not technically a dark pattern. A modal
overlay with a clear dismissal-via-reject affordance is
compliant AND less coercive.

### Alt-9: Server-side-rendered consent vs client-side-rendered (chosen: SSR-first with CSR enhancement)

The alternative is pure client-side rendering of the CMP.

Chosen pattern: SSR-first so the modal is present even with JS
disabled (accessibility), with CSR enhancement for interactivity.
This satisfies WCAG 2.2 (no-JS fallback) and reduces a class of
race conditions where the CMP loads after analytics scripts.

### Alt-10: Consent stored in cookie vs server-side ledger (chosen: ledger-of-record + thin cookie)

The alternative is to persist consent only in a client-side
cookie.

Chosen pattern: the consent ledger is the record of truth, and a
thin client-side cookie (`consent_state_ref`) holds only the
ledger reference (opaque ID + checksum). This makes the ledger
queryable, auditable, and revocable centrally; the cookie is just
a fast path for the runtime.

## Consequences

### Positive

- **Regulator-defensible by construction.** Every consent decision
  is reproducible, signed, and auditable.
- **Privacy-by-default product UX.** Cookie-less analytics keeps
  product visibility high without requiring consent for the most
  common operational metric.
- **No third-party trust burden.** D-6 eliminates an entire
  category of supply-chain risk.
- **Tenant-friendly.** D-11 lets B2B tenants pin policies
  appropriate to their sector.
- **Forward-compatible.** D-8 reserves the TCF / GPP seam without
  paying the complexity tax today.
- **Substrate-coherent.** The CMP is a substrate-layer concern
  per ADR-0245 and ADR-0246, composing cleanly with
  audit-chain (ADR-0246), policy-engine (ADR-0140 + ADR-0150),
  tenancy (ADR-0244), and i18n.
- **Pre-traffic install.** Zero retroactive remediation cost.

### Negative

- **Engineering scope.** New microservices required:
  `consent-ledger`, `cmp-runtime`, `gpc-signal-handler`,
  `analytics-aggregator` (cookie-less), `dark-pattern-lint`,
  `cmp-i18n-overlay`. Estimated initial scope ~4-6 implementation
  plans across the privacy axis.
- **No "Accept all" default reduces opt-in rates.** This is by
  design but may reduce signal for any future
  personalization/marketing surface.
- **Self-hosting fonts/CAPTCHA/maps costs ops effort.** D-6
  forecloses common shortcuts and substrate has to provide the
  in-house equivalent (ADR-0211).
- **Per-jurisdiction overlay logic is non-trivial.** D-7's
  strictest-wins resolver is one of the more complex pieces of
  business logic in the substrate; needs careful test coverage.
- **GPC honoring in non-US jurisdictions is conservative.** May
  reduce signal but is the right call under
  feedback_autonomous_decision_principles (long-term right >
  short-term cost).

### Neutral

- **Vendor registry overhead.** The vendor registry is required
  by D-6 even though the default product surface has zero
  vendors; required for any future opt-in by a tenant or
  product surface.
- **Locale parity.** D-7 and D-9 require parity across at least
  EN, KO, AR (RTL), DE, FR, NL, IT, ES, PT-BR, ZH, JA for the
  CMP modal. The i18n substrate already underwrites this but
  CMP is high-stakes for translation quality.
- **Annual consent re-confirmation.** Twelve-month expiry
  produces a re-prompt cohort each anniversary; this is
  expected and is signal of compliance, not friction.

## Implementation Surface

This ADR pre-stages the following implementation work. None of it
is in scope for the ADR itself; each item becomes one or more
implementation plans (IPs) cited in downstream PRs.

### Microservices (per ADR-0131 per-microservice flat layout)

1. `microservices/consent-ledger/` — append-only signed consent
   ledger; ed25519 per-tenant signing; per-jurisdiction residency
   pinning per ADR-0049; emits `consent.granted`,
   `consent.modified`, `consent.revoked`, `consent.expired`
   events to the substrate bus per ADR-0145.
2. `microservices/cmp-runtime/` — server-rendered + client-enhanced
   CMP modal; SSR + CSR hybrid; embeds the resolved overlay
   verdict from `consent-overlay-resolver`; renders WCAG-2.2-AA
   compliant UI.
3. `microservices/consent-overlay-resolver/` — pure-function
   resolver implementing D-7 strictest-wins semantic; deterministic;
   property-test-covered.
4. `microservices/gpc-signal-handler/` — request-time GPC signal
   detection + response; per-request idempotent.
5. `microservices/analytics-aggregator/` — cookie-less aggregate
   analytics emission per D-2; 24-hour salt rotation; aggregate
   rollup persistence; no per-session persistence.
6. `microservices/vendor-registry/` — substrate-level vendor
   registry per D-6; vendors registered with DPA reference,
   transfer mechanism, data-class mapping, IAB Vendor ID (where
   applicable), GPP support flag.
7. `microservices/dark-pattern-lint/` — build-time + CI-time lint
   per D-3 and D-9; lints CMP templates, copy, and visual
   regression artifacts.
8. `microservices/cmp-i18n-overlay/` — locale-specific text bundles
   per D-9; parity-tested across the supported locale set; ties
   to ADR-0064 localization packs.
9. `microservices/consent-dsar-bridge/` — bridges consent ledger
   to data-subject-access-request fulfilment per
   GDPR/PIPA/PDPL/LGPD/CCPA access rights.

### Specs (per /specs convention)

1. `/specs/cookie-consent-schema.json` — JSON Schema for the
   consent record.
2. `/specs/consent-purpose-taxonomy.json` — the closed enum of
   five purposes per D-1.
3. `/specs/cmp-wcag-accessibility.json` — the WCAG checklist per
   D-9.
4. `/specs/gpc-signal-handling.json` — GPC + GPP handling
   specification per D-10.
5. `/specs/jurisdiction-overlay-matrix.json` — the per-
   jurisdiction overlay matrix per D-7.
6. `/specs/dark-pattern-lint-rules.json` — the dark-pattern
   lint rules per D-3 and D-9.
7. `/specs/tenant-policy-override-schema.json` — per D-11.
8. `/specs/consent-audit-trail-schema.json` — per D-12.
9. `/specs/vendor-registry-schema.json` — per D-6.
10. `/specs/cookie-less-analytics-schema.json` — per D-2.

### Cedar policy fragments (per ADR-0150)

1. `pack/eu-gdpr/cedar/consent.cedar` — GDPR consent
   admission rules.
2. `pack/eu-eprivacy/cedar/cookies.cedar` — ePrivacy cookie
   rules.
3. `pack/kr-pipa/cedar/consent.cedar` — PIPA per-purpose
   separation rules.
4. `pack/ksa-pdpl/cedar/consent.cedar` — PDPL explicit-consent
   rules.
5. `pack/us-ccpa-cpra/cedar/opt-out.cedar` — CCPA + CPRA
   opt-out rules + GPC enforcement.
6. `pack/br-lgpd/cedar/consent.cedar` — LGPD specific-purpose
   record rules.
7. `pack/uk-gdpr/cedar/consent.cedar` — UK GDPR + PECR rules.
8. `pack/ch-revfadp/cedar/consent.cedar` — Swiss revFADP rules.

### CI lanes (per `cloud-ci/Rust gate packet`)

The twelve enforcement gates enumerated in the frontmatter become
twelve concrete CI lanes:

1. `lean-pr-cmp-consent-schema` — validates every consent-record
   emission against `/specs/cookie-consent-schema.json`.
2. `lean-pr-cmp-purpose-taxonomy-pin` — validates that no purpose
   outside the closed enum appears in code.
3. `lean-pr-cmp-wcag-2-2-aa` — Axe-core + custom WCAG checks
   against the CMP-runtime snapshot.
4. `lean-pr-cmp-gpc-signal-honored` — fuzz test that GPC signals
   under all in-scope jurisdictions produce expected ledger
   entries.
5. `lean-pr-cmp-no-pre-checked-opt-ins` — visual + semantic check
   that no purpose toggle defaults to ON.
6. `lean-pr-cmp-cookie-less-analytics-default` — verifies the
   analytics-aggregator emits no per-session persistence.
7. `lean-pr-cmp-first-party-cookie-only` — scans every property's
   network manifest for third-party requests; allow-list per
   tenant via vendor-registry only.
8. `lean-pr-cmp-per-jurisdiction-strictest-wins` — property tests
   the overlay resolver.
9. `lean-pr-cmp-consent-revocation-one-click` — UI test that
   revocation completes within one user action.
10. `lean-pr-cmp-consent-audit-chain-coverage` — every consent
    event has a corresponding audit-chain entry.
11. `lean-pr-cmp-tenant-policy-override-surface` — per-tenant
    overrides write to audit-chain.
12. `lean-pr-cmp-dark-pattern-lint` — runs the dark-pattern-lint
    rule set against the CMP-runtime templates and visual
    regression baseline.

### Documentation surface (per feedback_doc_coverage_enforced)

Every CMP-related microservice ships a full doc set (overview,
runbook, threat model, data-class registry mapping, DSAR
playbook, locale parity report, accessibility conformance report).
CI lane `lean-a5-doc-coverage` enforces.

### Frontend surface

The CMP-runtime renders into every product property's shell layout
via a slot that the frontend platform (Workflow Studio, Ontology,
Cloud Console, Marketplace) reserves. The slot is mandatory and
verified by CI lane `lean-pr-product-shell-cmp-slot-present`.

### Tenant onboarding

Tenant onboarding gains a "Privacy & Consent" step that surfaces
D-11 tenant-policy overrides + the BAA/DPA acknowledgement. The
defaults are the strictest jurisdiction overlay for the tenant's
incorporation jurisdiction; the tenant may loosen only within
allowed bounds.

### Data-class registry extensions (amends ADR-0099)

The data-class registry gains five new classes:

- `consent.record` — the consent record itself.
- `consent.audit` — audit-chain entries about consent.
- `consent.vendor-registry` — vendor registry entries.
- `consent.tenant-policy` — per-tenant override records.
- `consent.dsar-bridge` — DSAR fulfilment records.

Each class has residency, retention, and replication rules per
ADR-0049.

### Compliance-pack registrations (amends ADR-0251)

The following compliance packs declare `cookie-consent` as
required evidence:

- `pack/eu-gdpr/`
- `pack/eu-eprivacy/`
- `pack/uk-gdpr/`
- `pack/ch-revfadp/`
- `pack/kr-pipa/`
- `pack/ksa-pdpl/`
- `pack/us-ccpa-cpra/`
- `pack/br-lgpd/`
- `pack/jp-appi/` (forward-compat; APPI does not strictly require
  cookie consent but conformance is harmless and forward-
  compatible)
- `pack/sg-pdpa/` (similar to APPI)
- `pack/au-privacy-act/` (similar)

## Verification

### Regulator-compliant CMP audit

The verification surface is structured as a regulator-style audit
because that is the form in which regulator engagement will
arrive. The audit covers:

#### A. Schema audit

- The consent-record schema validates per
  `/specs/cookie-consent-schema.json`.
- All five purposes appear exactly once per record.
- All required jurisdiction overlay fields populated.
- Signature verifies under the tenant signing key.
- Prior-record chain reconstructs from genesis.

#### B. UI audit

- Every product property surfaces the CMP modal on first visit.
- All five purpose toggles render; "necessary" is always-on and
  informational; the other four default OFF.
- "Reject non-essential" affordance present with equal visual
  weight as any "Accept" affordance.
- No "Accept all" default.
- WCAG 2.2 AA Axe-core scan = 0 errors, 0 warnings of the
  rule set in `/specs/cmp-wcag-accessibility.json`.
- Screen reader walkthrough (NVDA + VoiceOver + JAWS +
  TalkBack) audit passes.
- RTL rendering (Arabic) is correct.
- Locale parity across all supported locales.

#### C. Behavioural audit

- Submitting with no toggles changed → consent record reflects
  necessary=true, all others=false.
- Toggling preference + statistics → corresponding ledger
  entry.
- Revoking marketing later → ledger entry; cache invalidation
  fan-out completes within 1 request-cycle.
- GPC signal set → marketing + personalization locked off
  (US jurisdictions); marketing + personalization signalled
  refused but overrideable (non-US).
- Per-tenant override OFF for marketing → marketing toggle
  hidden or locked off across tenant's surface.
- Expiry at 12 months → re-prompt + new ledger entry.

#### D. Audit-chain audit

- Every consent action produces exactly one audit entry.
- Audit entries are cryptographically chained per subject.
- Audit entries are queryable by subject + by tenant admin
  (within tenant policy).
- Residency: audit entries do not leave the subject's
  jurisdiction.
- Retention: audit entries persist per the retention overlay.

#### E. Vendor-registry audit

- Default product surface has zero registered vendors.
- Any tenant-added vendor has a DPA reference, transfer
  mechanism, data-class mapping, and applicable IAB Vendor ID
  (where ad-tech).

#### F. Dark-pattern audit

- The CMP-runtime templates pass `dark-pattern-lint` with zero
  findings.
- Visual regression baseline shows equal prominence on the
  reject affordance.
- Copy review shows no nudging language.

#### G. Resolver audit

- Property tests cover at least 10,000 randomized
  (jurisdiction × tenant-overlay × GPC × user-toggle) tuples
  and verify the resolver picks the strictest outcome.
- Differential testing against the published overlay matrix
  passes 100%.

#### H. Integration audit

- Cedar policy fragments per pack admit/deny the expected
  scenarios.
- The substrate event bus emits the expected events.
- The cookie-less analytics-aggregator never persists per-session
  state past 25 hours.
- The first-party-cookie scanner reports zero third-party
  requests across the default product surface.

#### I. Disaster scenarios

- Consent-ledger unavailable → CMP fails closed (treats user as
  having declined all non-essential); never grants implicit
  consent during outage.
- Cedar policy-engine unavailable → policy-engine fails closed
  per ADR-0140 default-deny.
- CMP-runtime template corruption → product property fails to
  serve traffic; gate prevents serving without a CMP slot.

#### J. Pre-launch checklist

- All twelve CI lanes green.
- Legal sign-off recorded as a signed audit entry.
- Privacy council sign-off recorded as a signed audit entry.
- Multispectrum-review v2.4.0 facets F1-F11 + M1+M2 + A1-A7
  evidence at evidence/debate/ADR-0272/.
- Reviewer-agent APPROVE plus CI green per
  feedback_self_merge_via_contract_path.

## References

### Regulatory primary sources

1. **Regulation (EU) 2016/679** (General Data Protection Regulation,
   "GDPR"). Articles 4(11), 6, 7, 13, 14, recitals 32, 42, 43, 47.
   https://eur-lex.europa.eu/eli/reg/2016/679/oj
2. **Directive 2002/58/EC** (ePrivacy Directive, as amended by
   Directive 2009/136/EC). Article 5(3).
   https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX%3A32002L0058
3. **ePrivacy Regulation 2024 draft** (Proposal for a Regulation
   of the European Parliament and of the Council concerning the
   respect for private life and the protection of personal data
   in electronic communications). Articles 5, 8.
4. **California Consumer Privacy Act (CCPA), Civil Code §1798.100
   et seq.**, as amended by the California Privacy Rights Act
   (CPRA, Proposition 24, effective 2023-01-01). Section
   1798.135(b)(1) on opt-out preference signals.
5. **California Privacy Protection Agency, Regulations updating
   the CCPA**, finalised 2023-03-29, updated 2024 (universal
   opt-out preference signal clarification, April 2024).
6. **Korean Personal Information Protection Act (PIPA, Act No.
   10465 of 2011, amended Act No. 16930 of 2020 and Act No.
   19234 of 2023)**. Articles 15, 22, 22-2, 35.
7. **Personal Information Protection Commission of Korea (PIPC)
   dark-pattern guidance, 2024**.
8. **Kingdom of Saudi Arabia Personal Data Protection Law (PDPL,
   Royal Decree M/19 of 1443H/2021)**. Implementing regulations
   2023-09-07. Articles 5, 6, 14, 29.
9. **National Data Management Office (NDMO) and SDAIA
   interpretive bulletins on PDPL, 2024 and 2025**.
10. **Lei Geral de Proteção de Dados Pessoais (LGPD, Lei
    13.709/2018, Brazil)**. Articles 8, 9, 18.
11. **UK General Data Protection Regulation** + **Privacy and
    Electronic Communications Regulations 2003 (PECR, as amended
    2019)**. ICO cookie guidance 2023 update.
12. **Swiss revised Federal Act on Data Protection (revFADP)**,
    in force 2023-09-01.
13. **Japan Act on the Protection of Personal Information
    (APPI, Act No. 57 of 2003, last amended 2022)**.
14. **Singapore Personal Data Protection Act (PDPA, 2012, as
    amended 2020)**.
15. **Australia Privacy Act 1988** (as amended by the Privacy
    Legislation Amendment 2024).

### Regulator guidance

16. **EDPB Guidelines 05/2020 on consent under Regulation
    2016/679**, version 1.1.
17. **EDPB Guidelines 03/2022 on Dark Patterns in social media
    platform interfaces**, version 2.0.
18. **EDPB Guidelines 8/2020 on the targeting of social media
    users**.
19. **CNIL, Délibération n° 2020-091 du 17 septembre 2020**
    (cookie guidelines).
20. **CNIL, Sanction Google LLC + Google Ireland Limited**,
    January 2022 (150M EUR fine).
21. **CNIL, Sanction Meta Platforms Ireland Limited**, January
    2022 (60M EUR fine).
22. **CNIL, Sanction Amazon Europe Core**, December 2020 (35M
    EUR fine).
23. **datatilsynet (Norway), guidance on cookie banners, 2022**.
24. **ICO (UK), Guidance on the use of cookies and similar
    technologies**, August 2023.
25. **PIPC (Korea), dark-pattern enforcement notices, 2024**.
26. **CPPA (California), regulations updating the CCPA**, March
    2023 + April 2024 GPC clarification.

### Industry standards

27. **IAB Tech Lab, Transparency & Consent Framework (TCF)
    v2.2**, effective 2023-11-20.
28. **IAB Tech Lab, Global Privacy Platform (GPP)**, version
    1.1 (2024).
29. **W3C, Tracking Preference Expression (DNT)** — historical
    context; DNT is deprecated.
30. **GPC.org, Global Privacy Control specification**.
31. **W3C Web Content Accessibility Guidelines (WCAG) 2.2**,
    Recommendation 2023-10-05.
32. **W3C ARIA 1.2** for the switch role.

### Academic and behavioural literature

33. **Acquisti, Brandimarte, Loewenstein (2017)**, *Privacy and
    Human Behavior in the Age of Information*. Science.
34. **Utz, Degeling, Fahl, Schaub, Holz (2019)**, *(Un)informed
    Consent: Studying GDPR Consent Notices in the Field*. ACM
    CCS.
35. **Nouwens et al. (2020)**, *Dark Patterns after the GDPR*.
    ACM CHI.
36. **Sanchez-Rola et al. (2019)**, *Can I Opt Out Yet? GDPR
    and the Global Illusion of Cookie Control*. AsiaCCS.

### Oyatie internal references

37. ADR-0049 — cross-region replication and residency.
38. ADR-0064 — canonical base and localization packs.
39. ADR-0099 — data-class registry.
40. ADR-0105 — thirteen-layer canonical enum.
41. ADR-0140 — Cedar policy enforcement.
42. ADR-0145 — inter-microservice communication reform.
43. ADR-0150 — Cedar policy engine.
44. ADR-0183 — policy-engine separation (Cedar app authz +
    Kyverno admission).
45. ADR-0211 — in-house tech stack preference.
46. ADR-0218 — tenant granular control surface.
47. ADR-0240 — sovereign cloud per regional pack.
48. ADR-0242 — oyatie is a tenant doctrine.
49. ADR-0243 — Cedar as universal gate.
50. ADR-0244 — tenant as universal scoping primitive.
51. ADR-0245 — substrate vs product layering.
52. ADR-0246 — policy-engine substrate promotion.
53. ADR-0251 — compliance pack + cell certification levels.
54. ADR-0255 — intelligence as two-layer AI substrate.
55. /specs/master-plan-sequencing.json
56. /specs/markdown-retirement-policy.json
57. /specs/root-hub-pointers.json

## Appendix A — Pattern Attribution

The patterns in this ADR are not inventions of oyatie. They are
distilled from the converging best practices of the regulator
community, civil-society research, and the privacy engineering
discipline. Attribution where due:

### A.1 The "strictest wins" overlay pattern

The strictest-wins overlay (D-7) is the natural multinational
implementation of the GDPR's "highest standard applies" principle
combined with US state laws' "we don't preempt stricter state
rules" clauses. It is documented in:

- IAPP, *Cross-Border Privacy Strategy* (2023).
- FPF, *Multi-State Privacy Comparison Chart* (2024).
- The patterns are mirrored in international tax law (Pillar
  Two minimum tax) and AML (FATF risk-based approach), so they
  are battle-tested in adjacent regimes.

### A.2 The closed-enum-of-purposes pattern

The five canonical purposes (D-1) follow the IAB Europe
"Standard Purposes" taxonomy in spirit (which has 11) but
collapsed to the smallest mutually exclusive set that meets
PIPA Article 22's separation requirement. The closed-enum
discipline mirrors ADR-0099 (data-class registry) and ADR-0105
(thirteen-layer enum) — same engineering principle: a small
fixed taxonomy is more auditable than an open one.

### A.3 The cookie-less analytics pattern

D-2's cookie-less analytics design draws on:

- **Plausible Analytics** (open-source, Estonia-based) — the
  canonical small example of first-party aggregate analytics
  without cookies. Plausible's salt-rotation design directly
  inspires D-2's 24-hour salt rotation.
- **Matomo / PIWIK PRO** — similar self-hosted analytics
  patterns.
- **Cloudflare Web Analytics** — server-side aggregate
  analytics with no client tags.
- **Goatcounter** — minimal self-hosted analytics.
- **Fathom Analytics** — also salt-rotation based.

oyatie's implementation is in-house per ADR-0211 but takes the
salt-rotation and aggregate-rollup patterns from this lineage.

### A.4 The dark-pattern lint pattern

D-9's dark-pattern lint is inspired by:

- **EDPB Guidelines 03/2022** (the substantive rule set).
- **CNIL's "Cookies and Other Trackers" cheatsheet** (2024).
- **Mathur et al. (2019)**, *Dark Patterns at Scale*, which
  produced the original taxonomy of dark patterns at e-commerce
  sites — the same taxonomy is now applied to consent surfaces.
- **deceptive.design** (Brignull) — the canonical taxonomy of
  dark patterns generally.

The lint rule set distills the visual + copy + behavioural rules
into machine-checkable assertions.

### A.5 The GPC-honoring pattern

D-10 mirrors the CCPA / CPRA + CPPA April 2024 guidance plus
the broader GPC.org specification. The "honor advisorily in
non-US jurisdictions" stance is a conservative extension that
preempts CNIL's likely future guidance (signalled at the 2024
Global Privacy Assembly).

### A.6 The append-only ledger pattern

D-4 and D-12's audit ledger pattern is the same as ADR-0246
audit-chain substrate. The pattern lineage:

- Git-style content-addressed append-only chains.
- Certificate Transparency (RFC 6962).
- Sigstore Rekor transparency log.
- Hyperledger Fabric ordering services.

Each ADR-using-this-pattern entry composes cleanly with
ADR-0246.

### A.7 The per-tenant policy override pattern

D-11 mirrors ADR-0064 (canonical base + localization packs)
plus ADR-0218 (tenant granular control surface). The pattern
generalises across substrate: tenants get a strictly-bounded
override surface against a canonical baseline.

### A.8 Acknowledged influences

- **Apple's App Tracking Transparency framework** (iOS 14.5+)
  — shifted the industry default from opt-out to opt-in for
  cross-app tracking. Influences D-1's marketing/personalization
  defaults.
- **Mozilla's Enhanced Tracking Protection** — influences D-6's
  first-party-only stance.
- **Brave Browser's defaults** — influences the conservative
  stance throughout.
- **DuckDuckGo's privacy-by-default product UX** — influences
  the modal copy direction (specific, not nudging).
- **the GDPR Hall of Shame / consent banner ranking sites
  (Tarte au Citron, etc.)** — provide negative exemplars used
  to derive the dark-pattern lint rule set.

The pattern attribution is itself an audit obligation: regulator
audits frequently ask "where did this design come from", and
documented lineage is a credibility signal.

## Appendix B — Worked Example: Berlin User Consent Flow

This worked example traces a single end-user consent decision
end-to-end as it flows through the substrate. It is intentionally
verbose because it serves both as documentation for downstream
implementers and as a regulator-readable narrative.

### B.1 Scenario

- **Subject**: an unsigned-in user in Berlin, Germany.
- **Browser**: Firefox 131 on macOS, GPC signal NOT set.
- **Browser language**: `Accept-Language: de-DE, de;q=0.9, en;q=0.8`.
- **Tenant**: `acme-eu` — a German B2B tenant with default
  jurisdiction overlay = EU-GDPR + EU-ePrivacy + DE-BDSG.
- **Product surface**: Workflow Studio marketing site landing
  page.

### B.2 Request lifecycle

#### Step B.2.1 — Initial request

User hits `https://workflow.acme-eu.oyatie.example/`.

Edge layer (per ADR-0253 network topology) terminates TLS,
forwards to the regional ingress in `eu-central-1` (Frankfurt)
per ADR-0049 data-residency policy.

#### Step B.2.2 — CMP runtime resolves overlay

The `cmp-runtime` microservice receives the request. It calls
`consent-overlay-resolver` with:

```json
{
  "subject_jurisdiction_hint_ip": "DE",
  "subject_jurisdiction_hint_accept_language": "de-DE",
  "subject_gpc_header": null,
  "tenant_id": "acme-eu",
  "tenant_pinned_overlay": ["EU-GDPR", "EU-ePrivacy", "DE-BDSG"]
}
```

The resolver computes the union overlay:

```json
{
  "applied_jurisdiction_overlay": "EU-GDPR+EU-ePrivacy+DE-BDSG",
  "required_purposes_default_off": ["preference", "statistics", "marketing", "personalization"],
  "required_separation": true,
  "pre_checked_forbidden": true,
  "reject_affordance_required": true,
  "max_expiry_days": 365,
  "presentation_language_required": ["de"],
  "audit_chain_retention_years": 6,
  "gpc_binding": "advisory"
}
```

#### Step B.2.3 — SSR render

The CMP-runtime SSR-renders the modal into the page shell:

- Modal in German (`de-DE`) text bundle.
- Four interactive toggles (preference, statistics, marketing,
  personalization), all default OFF.
- Information-only "necessary" indicator, always ON.
- Three buttons: "Auswahl speichern" (Save my choices),
  "Nicht-essenzielle ablehnen" (Reject non-essential),
  "Auswahl akzeptieren" (Accept selected).
- No "Alle akzeptieren" button (tenant has not enabled it).
- Footer link "Cookie- und Datenpräferenzen" persistently
  accessible.
- WCAG 2.2 AA conformance verified at build time.

#### Step B.2.4 — User interaction

The user toggles "preference" to ON and "statistics" to ON,
then clicks "Auswahl akzeptieren".

The CMP client posts to `POST /consent/`:

```json
{
  "purposes": {
    "necessary": true,
    "preference": true,
    "statistics": true,
    "marketing": false,
    "personalization": false
  },
  "ui": {
    "cmp_version": "1.0.0",
    "presented_locale": "de-DE",
    "presented_text_hash": "sha256:8c4a...",
    "interaction_kind": "click"
  }
}
```

#### Step B.2.5 — Consent-ledger writes record

The `consent-ledger` microservice:

1. Mints a subject pseudonymous stable ID (24-hour rotated
   per D-2's cookie-less pattern, but for consent we mint a
   distinct stable opaque ID that lives in the consent cookie).
2. Constructs the consent record per the schema in D-4.
3. Signs with the tenant's ed25519 key.
4. Writes to the append-only chain, residency=DE per ADR-0049.
5. Emits `consent.granted` event on the substrate bus per
   ADR-0145.
6. Sets a thin client cookie:
   `consent_state_ref=<opaque-ledger-ref>; Domain=workflow.acme-eu.oyatie.example; Path=/; Secure; HttpOnly; SameSite=Strict; Max-Age=31536000`.

The full ledger entry:

```json
{
  "schema": "consent-record/v1",
  "tenant_id": "acme-eu",
  "actor": {
    "kind": "end-user",
    "subject_id": "ps:opaque:b7e1a3f2c9d8e4...",
    "jurisdiction": "DE",
    "preferred_locale": "de-DE"
  },
  "consent": {
    "necessary": true,
    "preference": true,
    "statistics": true,
    "marketing": false,
    "personalization": false
  },
  "ui": {
    "cmp_version": "1.0.0",
    "presented_locale": "de-DE",
    "presented_text_hash": "sha256:8c4a...",
    "interaction_kind": "click"
  },
  "context": {
    "source_ip_prefix": "2a02:8108::/32",
    "user_agent_class": "firefox-131-macos",
    "page_uri": "/",
    "applied_jurisdiction_overlay": "EU-GDPR+EU-ePrivacy+DE-BDSG",
    "tenant_policy_overlay_hash": "sha256:1a4b..."
  },
  "issued_at": "2026-05-20T14:23:11Z",
  "expires_at": "2027-05-20T14:23:11Z",
  "signature": "ed25519:...",
  "prior_record_ref": null
}
```

#### Step B.2.6 — Downstream consumers respond to the event

- **analytics-aggregator** — already running in cookie-less mode
  (D-2); the `statistics` opt-in is informational because
  cookie-less analytics doesn't require it; the aggregator
  optionally enriches the per-day aggregate with a
  "consented-statistics" cohort marker that lets the tenant
  see a more granular per-cohort breakdown.
- **preference-store** — opt-in for `preference` activates
  cross-session theme persistence, layout-pinning,
  default-landing-tab, currency-display preferences.
- **marketing-pixel-emitter** — opt-out (marketing=false)
  keeps the (zero by default) emission set empty.
- **personalization-engine** — opt-out (personalization=false)
  keeps the user out of any model-fine-tuning signal
  collection.

Each downstream consumer logs the receipt of the
`consent.granted` event with their own audit trail entry per
ADR-0246.

#### Step B.2.7 — User browses

The user navigates the site. Analytics aggregate per-day
counters increment in cookie-less mode. The user's theme
preference (set during interaction) persists across sessions.
No marketing pixel fires. No personalization signal is
collected.

#### Step B.2.8 — User revokes statistics

Three weeks later, the user clicks the footer link "Cookie- und
Datenpräferenzen" and reopens the modal. Statistics is shown ON
(the persisted state). The user toggles statistics OFF and
clicks "Auswahl speichern".

The CMP client posts:

```json
{
  "purposes": {
    "necessary": true,
    "preference": true,
    "statistics": false,
    "marketing": false,
    "personalization": false
  },
  "ui": {
    "cmp_version": "1.0.0",
    "presented_locale": "de-DE",
    "presented_text_hash": "sha256:8c4a...",
    "interaction_kind": "click"
  }
}
```

The `consent-ledger` writes a new record, chained via
`prior_record_ref` to the first record, with `interaction_kind=
"click"` and an updated `issued_at`. A `consent.modified` event
fires. The analytics-aggregator drops the user's "consented-
statistics" cohort marker on receipt and resumes treating the
user as a cookie-less anonymous aggregate.

#### Step B.2.9 — DSAR

Six months later, the user files a Subject Access Request via
the German `acme-eu` privacy portal.

The `consent-dsar-bridge` queries the consent-ledger by
`subject_id` and produces the full chain of consent records as
part of the SAR response, in the format required by
GDPR Article 15 and DE-BDSG implementation rules.

#### Step B.2.10 — Tenant audit

The Bundesbeauftragte für den Datenschutz und die
Informationsfreiheit (BfDI) opens a routine audit of `acme-eu`.
The tenant's privacy officer queries the consent-ledger for
random sample records and produces:

- The signed consent record for each sampled subject.
- The complete chain of modifications/revocations.
- The applied jurisdiction overlay hash.
- The presented text hash + the underlying text bundle.
- The CMP version active at issuance.
- The WCAG conformance report for that CMP version.

The auditor independently verifies the chain integrity by
recomputing signatures. The audit closes without findings.

### B.3 Counterfactual variations

The worked example branches:

#### B.3.1 — Variation: GPC signal set

Same scenario but the user's Firefox has GPC enabled. The CMP
runtime detects `Sec-GPC: 1` and renders the modal with
marketing + personalization signalled as refused (toggles
locked OFF). The `gpc-binding: advisory` resolver outcome
allows the user to override; if they leave the toggles OFF
the record reflects refusal with `interaction_kind=
"gpc-signal"`. If a US-resident user with GPC enabled visits
the same surface (e.g. via VPN), GPC is binding under CCPA
and the toggles cannot be overridden.

#### B.3.2 — Variation: Tenant pins marketing OFF substrate-wide

Same scenario but `acme-eu` has pinned `marketing=forbidden`
in the tenant policy. The marketing toggle is HIDDEN from the
modal (not just OFF) because the tenant's policy makes the
purpose unavailable. The consent record reflects
`marketing=false` with `tenant_policy_overlay_hash` pointing
to a different overlay.

#### B.3.3 — Variation: Korean user

Same scenario but the user is in Seoul. The resolver applies
KR-PIPA overlay. The modal renders in Korean
(`ko-KR` locale bundle). PIPA Article 22-2 separation
requirements are satisfied by the per-purpose toggle design.
The audit-chain retention is set to 5 years post-revocation
per PIPA enforcement guidance.

#### B.3.4 — Variation: Saudi user

The user is in Riyadh. The resolver applies KSA-PDPL overlay.
The modal renders in Arabic (`ar-SA` locale bundle, RTL).
PDPL controller registration display appears in the modal
footer. Sharia-compatible-processing notice appears for the
personalization purpose because personalization may produce
inferences about religious classification.

#### B.3.5 — Variation: Healthcare tenant

The tenant is `acme-health` — a healthcare tenant whose
compliance pack pins `marketing=forbidden` AND
`personalization=forbidden` substrate-wide. The modal renders
two toggles only (preference + statistics). The consent record
reflects the constrained taxonomy.

### B.4 What the worked example proves

The example demonstrates:

1. **End-to-end traceability.** A single end-user decision flows
   from browser interaction → CMP runtime → consent-ledger →
   substrate event bus → downstream consumers → audit chain →
   DSAR fulfilment → regulator audit. Every step is signed and
   replayable.
2. **Per-jurisdiction overlay.** The same product surface adapts
   to DE, KR, KSA, healthcare-tenant contexts without
   per-context code paths.
3. **No dark patterns.** No "Accept all" button by default. No
   pre-ticked boxes. The reject affordance has equal prominence.
4. **One-click revocation.** Step B.2.8 demonstrates the
   revocation path is a single user action, satisfying
   GDPR Article 7(3) and PIPA Article 22(4).
5. **GPC honoring.** Variation B.3.1 demonstrates GPC is honored
   per CCPA/CPRA and advisorily elsewhere.
6. **Tenant policy override.** Variations B.3.2 and B.3.5
   demonstrate the per-tenant policy surface.
7. **Cookie-less analytics.** Throughout the example, the
   default cookie-less analytics emits per-day aggregates
   without persistent per-session storage.
8. **First-party only.** No third-party domains are contacted
   in any variation of the example.
9. **Audit-chain coverage.** Every consent action — issuance,
   modification, revocation — produces a signed audit entry.

## Open Items

The following items are flagged for follow-up but do NOT block
this ADR's acceptance:

- **ePR ratification overlay.** The ePrivacy Regulation (draft
  2024) is expected to ratify in 2026-2027. When that happens
  a follow-up ADR will pin the ratified overlay and update D-2's
  audience-measurement carve-out reference from draft to
  ratified text.
- **TCF v2.3 / v3 emergence.** IAB Tech Lab is signalling
  TCF v3 development. The D-8 seam is version-agnostic; a
  follow-up ADR will pin the version at activation time.
- **GPC standardisation.** GPC is currently a W3C
  Tracking Protection Working Group draft. If/when it becomes
  a formal W3C Recommendation a follow-up ADR will pin the
  normative reference.
- **Per-jurisdiction overlay matrix expansion.** The matrix
  starts with the in-scope jurisdictions; new jurisdictions
  add via dedicated IPs that extend the matrix without
  re-architecting.
- **Mobile app consent UI.** This ADR focuses on web; a
  parallel ADR will address React Native / iOS / Android
  consent surfaces in the mobile shells. The substrate
  consent-ledger is shared.
- **Browser extension surfaces.** If oyatie ever ships a
  browser extension product surface, a follow-up ADR will
  address chrome.permissions + WebExtension consent norms.
- **Connected-TV / set-top / kiosk surfaces.** Not in scope;
  flag for future.
- **Automated decision-making notices under GDPR Article 22,
  PIPA Article 36-2, KSA PDPL Article 16, LGPD Article 20.**
  Adjacent to consent and warrants its own ADR (likely
  ADR-0273 in the privacy-surface keystone bundle).
- **Children's privacy (COPPA / GDPR-K / KR YouthPIPA / etc.).**
  Adjacent and warrants its own ADR.
- **Sensitive-data consent (Article 9 GDPR, equivalent
  PIPA/PDPL/LGPD).** Adjacent and warrants its own ADR.

## Uncertainties

This ADR carries the following uncertainties that the implementing
IPs (implementation plans) must resolve before each lane goes
green:

1. **Exact text of consent prompts.** The presented text in
   each locale is subject to legal council review per
   jurisdiction. Initial drafts exist; final ratified text per
   locale is an IP deliverable.
2. **Tenant-policy override UI.** The exact UX of the tenant
   admin's policy-override surface is to be designed in
   collaboration with the tenant-onboarding IP.
3. **Vendor-registry initial population.** The default product
   surface has zero vendors; tenant-added vendors require a
   per-vendor onboarding workflow that is itself a substrate
   surface still in design.
4. **Cookie-less analytics granularity.** The trade-off between
   aggregation granularity and re-identification risk is a
   tunable parameter; initial settings are conservative,
   refined via differential-privacy analysis in a follow-up
   IP.
5. **Per-locale parity QA.** The CMP must pass parity QA across
   the supported locales. Locale parity QA tooling exists in
   the i18n substrate but the CMP-specific assertions are an
   IP deliverable.
6. **WCAG 2.2 AA evidence artifact format.** Axe-core baseline
   exists; the regulator-readable conformance report format
   is to be finalised with privacy council review.
7. **Dark-pattern lint rule expansion.** The initial rule set
   captures EDPB + CNIL + PIPC patterns; coverage of newer
   patterns (e.g. confirm-shaming variations) will expand as
   the rule set matures.
8. **GPC advisory enforcement in non-US.** D-10's advisory
   stance is conservative; CNIL signals from 2024 Global
   Privacy Assembly suggest binding treatment is coming.
   Follow-up ADR will pin if/when binding.
9. **Per-tenant DPA template.** The substrate ships a default
   DPA template; tenants may pin custom DPAs. The template
   library is an IP deliverable.
10. **Consent record retention floor.** Default is the greater
    of regulator-required period + 1 year, but the exact
    floor per jurisdiction is a legal-review deliverable per
    jurisdiction.
11. **Cross-pack composition.** When multiple compliance packs
    are installed (EU-GDPR + KR-PIPA + KSA-PDPL on a single
    tenant), the strictest-wins resolution interacts with
    ADR-0251 cross-pack traffic Cedar gating. The interaction
    is well-defined in principle but the implementation IP
    must produce the cross-pack-coherence test matrix.
12. **Browser fingerprinting boundary.** D-2 forbids
    fingerprinting signals, but the exact set of forbidden
    signals evolves with browser API surface; a follow-up IP
    maintains the forbidden-signal registry.

## Authority Chain

This ADR proceeds under the standard oyatie authority chain:

- **Authors**: council-privacy, council-legal,
  council-architecture, council-security, council-frontend,
  ops-compliance, ops-sre-reliability.
- **Reviewers** (multispectrum v2.4.0): F1-F9 + M1 + M2 + F10
  (frontend UX) + F11 (i18n) + A1-A7 (own-policy-adherence
  family). Evidence at `evidence/debate/ADR-0272/`.
- **Approvers**: council-architecture + council-privacy +
  council-legal (joint sign-off required per
  feedback_self_merge_via_contract_path).
- **CI lanes**: the twelve enumerated `cloud-ci/Rust gate packet`
  lanes must turn green before promotion past `dev`.
- **Foundry pipeline**: this ADR enters via the
  Foundry pipeline per docs/AGENTS.md operating contract and
  ADR-0116; admission gate at `dev`, merge queue per ADR-0111,
  completion gate at reviewer-agent APPROVE plus CI green.

---

*End of ADR-0272*
