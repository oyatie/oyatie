---
id: ADR-0250
status: Superseded
planning_impact: true
date: 2026-05-20
owners:
  - council-architecture
  - council-product
  - council-privacy
  - council-security
  - council-legal
  - ops-compliance
  - ops-sre-reliability
  - ops-dr-capacity
  - axis-payments
  - axis-healthcare
  - axis-government
  - axis-education
  - axis-marketplace
  - axis-financial-services
  - axis-defense
  - axis-pharma
supersedes: []
amends: []
superseded_by: [ADR-0709]
related:
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0010-regional-pack-architecture.md
  - ADR-0049-cross-region-replication-and-residency.md
  - ADR-0064-canonical-base-localization.md
  - ADR-0099-data-class-registry.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0144-eu-ai-act-graduated-risk-tier-model.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0218-tenant-granular-control-surface.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0241-dr-business-continuity-portfolio-policy.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0249-multi-category-marketplace-doctrine.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0252-time-coordination-distributed-consistency.md
  - ADR-0253-network-topology-edge-service-mesh.md
  - ADR-0254-deployment-model-spectrum.md
  - ADR-0255-intelligence-as-two-layer-ai-substrate.md
related_specs:
  - /specs/platform-architecture.json
  - /specs/capability-certification-matrix.json
  - /specs/capability-launch-runbooks.json
  - /specs/capability-launch-roadmap.json
  - /specs/cedar-fragment-schema.json
  - /specs/microservices/policy-engine.json
  - /specs/tenant-model.json
related_memory:
  - feedback_build_ahead_of_certification_doctrine
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_quality_performance_scalability_bar
  - feedback_autonomous_implementation_artifacts
  - feedback_canonical_base_localization
  - feedback_no_silent_regression
  - feedback_automate_everything
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-foundational-doctrine
keystone_position: 9-of-14
purpose: >
  Establish that every certification-gated capability in the portfolio
  (payments, healthcare, government, education, pharma, defense,
  marketplace physical-goods, marketplace services, marketplace c2c,
  financial-services data) is architected and built day-one and
  launched per-market only after regulatory clearance lands. Build
  precedes certification; certifications drop on working systems
  rather than triggering build-from-zero. Replicates the Apple Pay
  per-country rollout pattern, the Stripe geographic expansion shape,
  and the AWS regional service-availability pattern. Eliminates the
  "wait for certification, then start coding" failure mode that
  produces months of post-cert delay and a permanent build vs market
  race condition.
enforcement_status: advisory-until-three-state-lifecycle-lands
enforced_by:
  - oya gate validate capability-three-state-coherence
  - oya gate validate built-but-unlaunched-cedar-gate
  - oya gate validate capability-launch-runbook-completeness
  - oya gate validate certification-evidence-retention
  - oya gate validate anti-bypass-built-only-tenant
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


> **Disposition light-edit (2026-08-06):** Build-ahead-of-certification remains hyperscaler posture

# ADR-0250: Build-Ahead-of-Certification Doctrine

## Status

Proposed — 2026-05-20.

Bundled with the 14-ADR foundational keystone set (ADR-0242 through
ADR-0255 inclusive) landing as a single multispectrum-reviewed PR.
This is keystone #9 of 14. Partial acceptance is rejected: the
build-ahead doctrine is mutually-reinforcing with ADR-0242
(oyatie-is-a-tenant), ADR-0243 (Cedar as universal gate), ADR-0240
(sovereign-cloud-per-regional-pack), ADR-0241 (DR + BC portfolio),
ADR-0144 (EU AI Act graduated tiers), ADR-0249 (data-residency
enforcement), and ADR-0251 (compliance pack + cell certification
levels). Without the keystone bundle, the three-state lifecycle
declared here does not have the Cedar fragments, tenancy substrate,
or compliance-pack abstraction to back it.

Enforcement is `advisory-until-three-state-lifecycle-lands`: the
doctrine is accepted in text now, but the CI lanes that enforce
it move to BLOCKER status only after:

1. `microservices/tenancy/` admits the `tenant.eligible_capabilities[]`
   column and the `tenant.compliance_packs[]` column per ADR-0244 +
   ADR-0251.
2. `microservices/policy-engine/` carries a published
   `capability-X.permitted` Cedar fragment for each capability
   declared in §D-4.
3. `microservices/cloud-iac/ARCHITECTURE.md#cell-provisioning` admits the `cell.certifications[]` metadata
   per ADR-0248 + ADR-0251 with regulator-attested certificate
   entries.
4. The capability-launch-runbook template at
   `docs/standards/capability-launch-runbook-template.md` is
   authored.
5. The capability-launch-roadmap spec at
   `/specs/capability-launch-roadmap.json` enumerates every
   capability in §D-4 with date, market, certification chain.

Until those five bootstrap items land, the validators emit findings
without failing CI. Post-bootstrap, the lanes promote to BLOCKER and
any merge that violates a build-ahead invariant is rejected.

## Date

2026-05-20.

## Context

### The certification-versus-build race condition

Every major regulated software market is gated by a certification
artifact (PCI DSS Level 1, HIPAA + BAA, FedRAMP Moderate/High, IL5/
IL6, KR CSAP, EU PSD2, etc.) issued by an auditor or regulator. The
certification process typically requires:

- 6-12 months of operational evidence (logs, change-control records,
  drill receipts, incident post-mortems) before the formal audit
  window opens.
- A 3-6 month audit window with auditor interviews, evidence
  walkthroughs, gap-remediation cycles.
- A 1-3 month regulator-review window before issuance.
- Per-market renewal cadence (PCI DSS annually; SOC 2 Type II
  annually; FedRAMP continuous monitoring + 3-year reauthorization;
  EU PSD2 ongoing supervision; KR CSAP biennial).

The total elapsed time from "we want to certify this capability in
market M" to "we have the certificate" is **typically 12-24 months**,
with several certifications (FedRAMP High, IL5/IL6, KR CSAP for
financial-services data) routinely taking 24-36 months from kickoff
to issuance.

Two failure modes follow from naively sequencing build behind
certification:

**Failure mode 1 — Build-from-zero on cert grant.** The team defers
build until the certification clears. When the certificate arrives,
the team starts coding. Time-to-market = 12-24 months (cert) + 6-18
months (build) = 18-42 months. Competitors who built ahead arrive
12-24 months earlier; the market is lost before launch.

**Failure mode 2 — Build-during-cert with scope drift.** The team
starts building when audit window opens. Auditor findings force code
changes; code changes invalidate prior evidence; re-audit cycle is
triggered. Net effect: certification is delayed, build is delayed,
launch is delayed, and the team learns (often in a costly retro) that
the build should have been complete before the audit window opened.

### What every named hyperscaler actually does

The hyperscaler-pattern reference set demonstrates a uniform shape:
**build the capability globally; launch the capability per-market
post-certification.**

- **Apple Pay (launched 2014-10-20 in the US).** By the time Apple
  Pay launched in the US, Apple had:
  - Built the full Apple Pay infrastructure (Secure Element, tokenisation,
    payment-network integration with Visa, Mastercard, American Express,
    Discover).
  - Completed PCI DSS Level 1 certification.
  - Negotiated with US issuing banks (announced 6 launch partners,
    grew to 500+ within 12 months).
  - Built per-country launch readiness for UK, Canada, Australia,
    China before any of those markets shipped.

  Apple Pay launched in the UK on 2015-07-14 — eight months after
  US launch. The UK build was complete on day one; the UK launch
  waited for Apple's per-bank partnerships + UK Visa/Mastercard
  routing + UK Treasury sign-off. Same shape for Canada (2015-11-17),
  Australia (2015-11-19), China (2016-02-18, via UnionPay), and the
  ~80 markets that followed through 2024.

  Apple did not rebuild Apple Pay for each market. The architecture
  was built once with multi-market support; launch flipped a per-
  market gate.

- **Stripe geographic expansion (2011-present).** Stripe's expansion
  blog series (stripe.com/blog) documents the same shape:
  - US launch 2011-09.
  - Stripe.com blog "Stripe expands to..." posts from 2012-2025
    describe the launch sequence: build infrastructure (multi-currency,
    multi-acquirer, per-country compliance overlays) → secure
    regulatory clearance (e-money authorisation under EU PSD2; KR
    결제대행업 license; JP Funds Settlement Act registration; SG
    MAS PS Act license) → flip the per-country switch.
  - Stripe launched in 50+ countries by 2024. The Stripe Treasury
    product, Stripe Atlas, Stripe Climate, Stripe Tax — each followed
    the same shape.
  - Critical: Stripe did NOT defer build until certification landed.
    Stripe's PSD2 e-money licence took ~18 months end-to-end; the
    Stripe EU stack was built and ready when the licence dropped.

- **AWS regional service expansion (2006-present).** When AWS opens
  a new region, the launch is sequenced:
  - 18-24 months prior: building data centres + electrical
    infrastructure + network backbone.
  - 12-18 months prior: regional regulatory + sovereignty agreements.
  - 6-12 months prior: regional service deployment (control plane
    + data plane replicas of all in-scope services).
  - 0-6 months prior: regional certifications (FedRAMP, IRAP,
    K-ISMS-P, etc.) finalize.
  - Region opens.

  By the time AWS opens a region, every in-scope service is already
  deployed and operationally proven. The launch flipped a routing
  switch.

  AWS GovCloud (US) opened 2011-08; AWS Top Secret region opened
  2014-06 (with then-pending IL5 + IL6 work that completed in 2017);
  AWS GovCloud (US-East) opened 2018-12 — each followed the
  build-ahead pattern.

- **Microsoft Azure Government, Azure for Healthcare, Azure for
  Financial Services.** All follow the same shape: regulated
  capabilities are built in the global Azure stack, gated on
  per-tenant compliance attestations, with regional sovereignty
  + certification handled by separate region deployments (Azure US
  Gov, Azure China via 21Vianet, Azure Germany historically). The
  per-Vertical-Industry overlays (Healthcare APIs, Financial Services
  Cloud) shipped as feature gates flipped per-tenant post-attestation.

- **Salesforce Health Cloud, Financial Services Cloud, Government
  Cloud.** Same shape: capability built on the standard Salesforce
  multi-tenant substrate; per-vertical regulatory wrapper (HIPAA BAA,
  SOX-aligned controls, FedRAMP-aligned controls) layered above;
  capability available to tenants who execute the appropriate BAA
  or sign-up flow.

- **Palantir Foundry (commercial vs government).** Palantir Foundry
  ships one codebase. Government deployments (DoD, IC, FedRAMP High
  + IL5 + IL6 customers) run on certified Palantir Apollo deployment
  pipelines into accredited environments; commercial deployments
  run on the same codebase into commercial environments. Build is
  done once; certification + deployment is per-environment.

- **Cloudflare for Government, Cloudflare One, Cloudflare Workers.**
  Cloudflare's FedRAMP Moderate authorisation (granted 2022) and
  IL5 pending authorisation (in process since 2024) gate access to
  the existing Cloudflare platform, not a separately-built one. The
  cf.gov deployment runs the same Pingora codebase with additional
  isolation, attestation, and supply-chain controls.

The pattern is uniform: **mature regulated-market platforms build
once, certify per-market, launch per-market.**

### What "build-ahead-of-certification" actually means for oyatie

The doctrine establishes:

1. **All certification-gated capabilities are architected day-one.**
   Architecture lives in ADRs + specs + Cedar fragments + manifest
   declarations.
2. **All certification-gated capabilities are built day-one.** Code
   is written, integration-tested, ops-ready, instrumented, drill-
   passed.
3. **Capabilities are launched per-market post-certification.** A
   capability LAUNCHED in market M means tenants whose home cell is
   certified for M's required certs can USE the capability for
   that market.
4. **Building is not launching.** A capability in the BUILT state is
   exercised in sandbox tenants (`oyatie.dev.*`, `oyatie.preview.*`),
   subjected to integration tests, drilled in DR scenarios, observed
   in observability dashboards — but no customer-tenant production
   traffic flows through it.
5. **Three states are distinct.** ARCHITECTED (in spec + ADR) → BUILT
   (code + tests + ops-grade) → LAUNCHED-IN-MARKET (Cedar-permitted
   for that market's tenants). The three-state lifecycle is declared
   in the platform spec, exposed in tenant config, and Cedar-
   evaluated at every invocation.

### Why now (2026-05-20)

Three forcing functions:

- **The masterplan timeline assumes parallel certification + build.**
  `feedback_autonomous_implementation_artifacts` says "Implement the
  masterplan runs without user intervention" — including the parts
  of the masterplan that touch certification-gated capabilities. If
  we sequence build behind certification, the masterplan stalls.
- **The keystone bundle (ADR-0242 through ADR-0255) establishes the
  enforcement primitives.** Cedar fragments per capability +
  tenant.eligible_capabilities + cell.certifications + compliance
  packs are the artifacts that make build-ahead enforceable. Without
  the keystone bundle, build-ahead is unenforced wishful thinking.
- **The marketplace + payments + financial-services substrate work
  (ADR-0252 + ADR-0254) lands as part of this same keystone bundle.**
  Without an explicit build-ahead doctrine, those ADRs would read
  ambiguously: "build the substrate" — but is the substrate launched
  or just built? The answer is "built day-one; launched per-market
  per the roadmap in §D-8."

### What this is NOT

This ADR is NOT:

- A demand that every market launch on day one. The launch sequence
  in §D-8 is multi-year.
- A demand that the build be 100% bug-free before launch. Built means
  ops-grade per §D-2 quality bar; some defects will be discovered
  in market; the post-launch SLO + incident response per ADR-0241
  applies.
- A demand to skip certification. Certifications are mandatory before
  launch in any market that requires them.
- A demand to ignore regulator pre-engagement. Some certifications
  (FedRAMP, KR CSAP, EU PSD2) benefit from regulator pre-engagement
  during build; that pre-engagement is encouraged. What's forbidden
  is **deferring build until pre-engagement concludes.**
- A statement about which markets to enter. Market selection is a
  product/strategy decision; this ADR only governs the build-and-
  launch sequencing within a chosen market.

## Decision

### D-1. Three states for certification-gated capabilities

Every capability listed in §D-4 progresses through three distinct
states. The state is declared at the capability level (per the
`/specs/capability-certification-matrix.json` spec) and tracked per
market (per the `/specs/capability-launch-roadmap.json` spec).

| State | Meaning | Artifacts required to enter | Exit gate |
|---|---|---|---|
| **ARCHITECTED** | Capability has a design, ADR, spec, Cedar fragment skeleton, manifest declaration. No production code yet. | Approved ADR; `/specs/microservices/<ms>/<capability>.json` exists; Cedar fragment `policy-engine/fragments/baseline/<capability>-permits.cedar` exists in skeleton; multispectrum review v2.4.0 verdict APPROVE on the ADR. | All §D-2 quality-bar checks pass for the capability's code. |
| **BUILT** | Capability is fully implemented; ops-grade; tested in sandbox; drill-passed; observed. NOT available to customer tenants. | All §D-2 quality-bar artifacts; sandbox-tenant end-to-end traffic; DR drill receipt; threat model APPROVE; security review APPROVE. | Per-market launch gates (§D-3) cleared for the target market. |
| **LAUNCHED-IN-MARKET** | Capability is Cedar-permitted for tenants whose home cell carries the required certification + whose compliance packs include the required pack + whose KYB/KYC status satisfies the market's requirements. | All §D-3 launch gates cleared for that market; Cedar fragment `capability-X.permitted` is published + signed; capability shows on tenant admin console for eligible tenants. | (Steady state.) Or DEGRADED/DARK per §D-12 rollback. |

A capability may be LAUNCHED in market A while still BUILT (not
launched) in market B. A capability may simultaneously be DEGRADED
in market C while LAUNCHED-IN-MARKET in market D. State is
(capability, market)-tuple-scoped.

### D-2. Build-state quality bar

A capability transitions from ARCHITECTED to BUILT only after the
following artifacts exist + are verifiable in CI:

1. **Full implementation in code.** Every action the capability
   exposes is implemented; no stubs, no `todo!()`, no
   `unimplemented!()` paths. The `oya gate validate no-stubs-in-
   capability` lane scans the capability's code root.
2. **Integration tests.** End-to-end test coverage of the capability's
   API surface against in-test sandbox tenants. The
   `oya gate validate capability-integration-coverage` lane requires
   ≥ 90% action coverage with named-test attestations.
3. **CI green.** All standard CI lanes (multispectrum review v2.4.0,
   lean-a* lanes, foundry fitness lanes) pass.
4. **Threat model APPROVE.** A threat model document exists at
   `docs/security/threat-models/<capability>.md`; security council
   has signed.
5. **Runbooks authored.** Incident-response runbook, capacity
   runbook, DR-failover runbook, rollback runbook all exist at
   `docs/runbooks/<capability>-*.md`.
6. **Capacity model verified.** A capacity model document exists at
   `docs/capacity/<capability>.md`; load test results at
   `evidence/load-tests/<capability>/` show the µservice meets the
   projected peak load with headroom per ADR-0128 invariant.
7. **DR drills passed.** Per ADR-0241, the capability's owning
   µservice has a `dr_tier` declared + at least one successful drill
   receipt emitted to the audit chain (audit class `DrDrillReceipt`)
   within the trailing drill-cadence window.
8. **Security review APPROVE.** Security council signs the capability
   for production readiness. Signed verdict at
   `evidence/security-review/<capability>-verdict.json`.
9. **SLO instrumented.** OpenSLO document at
   `microservices/<ms>/slos/<capability>.openslo.yaml`; metrics
   emit per ADR-0130 + ADR-0131; SLO burn-rate dashboard exists.
10. **Observability dashboard exists.** Per-capability dashboard at
    `microservices/observability/dashboards/<capability>.md`; the
    dashboard is publicly visible to oyatie engineers.
11. **Cedar fragments published.** The capability's baseline Cedar
    fragments are reviewed, signed, and published (per ADR-0243
    fragment lifecycle).
12. **Audit-chain emission verified.** Every capability action emits
    to the audit chain (audit class `<capability>Action`); sampling
    a sandbox-tenant invocation returns the audit row within SLO.

The transition ARCHITECTED → BUILT is gated by the `oya gate validate
capability-built-quality-bar` lane evaluating these 12 criteria.
Promotion is a versioned event emitted to the audit chain (class
`CapabilityBuiltPromotion`).

**Built ≠ public.** A capability in the BUILT state is not visible
to customer tenants. The Cedar gate (§D-6) refuses every action
attempt by a customer principal because the capability has not been
LAUNCHED for that tenant's market.

### D-3. Launch gates per-market

A capability transitions from BUILT to LAUNCHED-IN-MARKET only after
**every** per-market launch gate is satisfied:

| Gate | Required artifact | Owner |
|---|---|---|
| **Certification** | Regulator-issued certificate for the capability's required-cert set in the target market | ops-compliance |
| **Regulator approval** | Per ADR-0240 sovereign-pack overlay, the regulator has issued written approval to operate the capability in the target market | ops-compliance + council-legal |
| **Legal review** | Per the `oyatie.legal` sub-scope (ADR-0242 §D-2), legal council has signed the launch | oyatie.legal |
| **Tenant onboarding flow** | Per-market KYB / KYC / sanctioned-party-screening flow exists; works against the market's local requirements; tested with sandbox identity-verification cases per ADR-0253 | axis-identity + council-product |
| **Market-specific Cedar pack** | Compliance pack per ADR-0251 is published + signed; pack's Cedar fragments are loaded in the target-market cells | ops-compliance + axis-policy-engine |
| **Operational readiness** | On-call rotation staffed for the target market's timezone; observability dashboards localized for relevant currency / language; support documentation in market-native language | ops-sre-reliability |
| **Marketing approval** | Per ADR-0144 EU AI Act Article 50 disclosure + per-market consumer-protection requirements, marketing material reviewed by legal | council-product + oyatie.legal |
| **Insurance + bonding** | For payments + marketplace + financial-services, market-specific insurance + bonding (e.g., MTL surety bonds in US states, EU e-money safeguarding) is in place | ops-compliance + oyatie.finance |
| **Tax registration** | Per-market tax nexus established; tax-engine routes per market's tax overlay (e.g., marketplace facilitator tax in 50 US states; VAT MOSS in EU; KR 부가가치세 in KR) | oyatie.finance |
| **Audit-chain regulator-readiness** | Per-market regulator-evidence cadence configured; first quarterly packet emitted | ops-compliance |

The transition BUILT → LAUNCHED-IN-MARKET is gated by the `oya gate
validate capability-launch-gate-completeness` lane evaluating every
gate for the (capability, market) tuple. Promotion is a versioned
event emitted to the audit chain (class
`CapabilityLaunchedInMarketPromotion`) carrying the gate-evidence
manifest.

### D-4. Capability + certification matrix

The following capability + certification matrix is canonical. Each
row enumerates the certifications required for launch in each
relevant market. The matrix is the source-of-truth backing the
`/specs/capability-certification-matrix.json` spec.

| Capability | Market | Required certifications | Renewal cadence | Cost-estimate band |
|---|---|---|---|---|
| **Payments — Card present + card-not-present + ACH + wire + bank-debit + open-banking** | US | PCI DSS Level 1 (annual); SOC 2 Type II (annual); per-state Money Transmitter License × 50 (initial 9-24 months per state; biennial renewal); FinCEN MSB registration (initial; biennial); OFAC sanctions-screening compliance (continuous); CFPB Regulation E + Z compliance (continuous); state UDAAP compliance (continuous); 1099-K reporting infrastructure (annual) | PCI annual; SOC 2 annual; MTL biennial per state; FinCEN biennial | $5-15M initial across 50 states; $1.5-3M/yr renewals |
| Payments | EU | EU PSD2 e-money authorisation (member-state-issued; ~18 months end-to-end; passportable across EEA after); PCI DSS Level 1; SOC 2 Type II; GDPR Article 32 + 35 attestation; PSR (Payment Services Regulations) UK if UK in scope; EBA Strong Customer Authentication compliance | Authorisation continuous; PCI/SOC 2 annual | $2-4M initial; $0.5-1M/yr |
| Payments | KR | 결제대행업 (Payment Gateway business) registration with FSS (~6-12 months); PCI DSS Level 1; SOC 2 Type II; ISMS-P; CSAP if any government tenant uses payments; KR e-금융감독규정 compliance; 전자금융거래법 compliance | Annual FSS reporting; PCI annual; ISMS-P annual; CSAP biennial | $0.8-2M initial; $0.3-0.6M/yr |
| Payments | JP | Funds Settlement Act registration with FSA (~12 months); PCI DSS Level 1; SOC 2 Type II; APPI compliance; J-LIS partnership for high-volume merchants; J-CSIP membership preferred | Annual FSA reporting; PCI/SOC 2 annual | $0.8-2M initial; $0.3-0.6M/yr |
| Payments | SG | MAS Payment Services Act licence (Major Payment Institution; ~12-18 months); PCI DSS Level 1; SOC 2 Type II; PDPA compliance; MAS Technology Risk Management (TRM) framework compliance | MAS supervision continuous; PCI/SOC 2 annual | $0.5-1.2M initial; $0.2-0.4M/yr |
| Payments | UK (post-Brexit) | FCA e-money institution / payment institution authorisation (~12-18 months); PCI DSS Level 1; SOC 2 Type II; UK GDPR; PSR 2017 compliance; SCA compliance | FCA supervision continuous; PCI/SOC 2 annual | $0.6-1.5M initial; $0.2-0.5M/yr |
| Payments | AU | AUSTRAC AML/CTF reporting entity registration; ASIC AFS Licence (if relevant); PCI DSS Level 1; SOC 2 Type II; Privacy Act 1988 compliance; CDR compliance for open-banking | AUSTRAC continuous; PCI/SOC 2 annual | $0.4-1M initial; $0.15-0.3M/yr |
| **Healthcare — PHI processing + EHR integration + telehealth + Rx + de-identification** | US | HIPAA Privacy Rule + Security Rule + Breach Notification Rule compliance (no formal "cert" — assertion + ongoing audit); SOC 2 Type II (HITRUST-mapped); HITRUST r2 Validated Assessment (every 24 months); BAA infrastructure (signed BAAs with every business associate); state-specific breach notification laws (all 50 states + DC + PR); CCPA + CMIA in CA; TX HB 300 in TX; NY SHIELD in NY | HITRUST every 24 months; SOC 2 annual; BAAs perpetual with annual review | $1-2.5M initial; $0.5-1M/yr |
| Healthcare | EU | GDPR Article 9 (special categories of personal data) compliance with DPIA per processing activity; EU Medical Device Regulation (MDR) if any device classification applies; Member-state-specific health-data laws (Germany BDSG; France Code de la santé publique; Italy GDPR + Health Authority); EHDS (European Health Data Space) compliance as it ratifies (2026-2028) | DPIA per processing activity; member-state reviews continuous | $0.5-1.5M initial; $0.3-0.6M/yr |
| Healthcare | KR | 의료법 (Medical Service Act) compliance; 개인정보보호법 (PIPA) Article 23 sensitive-info compliance; HIRA (Health Insurance Review and Assessment) integration if Rx; KISA-led certification scheme; CSAP if any public-hospital tenant | KISA continuous; CSAP biennial | $0.3-0.8M initial; $0.15-0.3M/yr |
| Healthcare | JP | APPI (Act on the Protection of Personal Information) Article 17 special-category compliance; Health-care-related laws under MHLW guidance; APPI cross-border-transfer rules; J-DPF (Data Free Flow with Trust) eligibility | MHLW continuous; APPI annual review | $0.2-0.6M initial; $0.1-0.2M/yr |
| **Government — sovereign-cloud tenants (US federal + state; EU member-states; KR public sector; international defense)** | US (federal) | FedRAMP Moderate (initial 12-18 months); FedRAMP High (additional 12-18 months); FIPS 140-3 validated cryptographic modules; FISMA continuous monitoring; ATO (Authority To Operate) per agency; NIST 800-53 control compliance; CJIS compliance if law-enforcement; ITAR + EAR controls; SP-CMMI / DFARS if defense-adjacent | FedRAMP continuous; ATO per-agency renewal 3-yr; CJIS annual | $3-8M initial Moderate→High; $1-2M/yr continuous monitoring |
| Government | US (state) | StateRAMP authorisation (state-equivalent of FedRAMP; ~12-18 months); per-state ATO + compliance; state-specific (e.g., CA SOC 2; TX TXRAMP; AZ StateRAMP) | StateRAMP annual; per-state continuous | $1-3M initial; $0.5-1M/yr |
| Government | US (defense) | IL5 (Impact Level 5; up to controlled-unclassified-information); IL6 (up to SECRET when classified-environment-adjacent); ITAR registration; DFARS 252.204-7012 cybersecurity compliance; CMMC Level 3+ (Cybersecurity Maturity Model Certification); SCIF (Sensitive Compartmented Information Facility) adjacency for IL6+ workloads | IL5 continuous; IL6 reauthorization 3-yr; CMMC every 3 yr | $5-15M initial IL5; $10-30M for IL6; $2-5M/yr |
| Government | KR | CSAP (Cloud Security Assurance Program) Level 2 (high+) for government tenants; KR-NIS clearance for sensitive workloads; KR 정보통신망법 compliance; KR 개인정보보호법 government overlay; integration with 정부24 / GPKI for citizen-facing services | CSAP biennial; KR-NIS continuous; legal-compliance continuous | $0.8-2M initial; $0.4-0.8M/yr |
| Government | EU | NIS2 (Network and Information Security Directive 2) compliance; GAIA-X certification (federated trust infrastructure); per-member-state sovereignty (e.g., France SecNumCloud Qualified; Germany BSI C5 Type 2); EUCS (Cloud Services certification scheme) when ratified | NIS2 continuous; GAIA-X annual; member-state varies | $1.5-4M initial; $0.6-1.5M/yr |
| **Education — student PII + parental consent + minors + grade-recording** | US | FERPA (Family Educational Rights and Privacy Act) compliance; COPPA (Children's Online Privacy Protection Act) compliance for users under 13; state-specific (CA SOPIPA; CO HB 1132; NY Ed Law 2-d; CT SB 949; multiple others as of 2026); annual security audit per per-state-requirements; iKeepSafe certification optional but recommended | FERPA continuous; COPPA continuous; state laws vary | $0.3-0.8M initial; $0.2-0.5M/yr |
| Education | EU | GDPR Article 8 (children's consent) compliance with member-state-specific age threshold (13-16 depending on member state); per-member-state school-data laws (e.g., Italy GDPR-Schools guidance; Germany Datenschutzkonferenz school positions) | DPIA per processing activity; member-state continuous | $0.2-0.5M initial; $0.15-0.3M/yr |
| Education | KR | 개인정보보호법 Article 22(2) child-data compliance; Ministry of Education curriculum-data integration requirements; KISA continuous certification | KISA annual; MOE per-implementation review | $0.15-0.4M initial; $0.1-0.2M/yr |
| **Pharma — clinical-trial data + GxP + 21 CFR Part 11 + Annex 11** | US | FDA 21 CFR Part 11 (Electronic Records and Electronic Signatures) compliance; GxP (GLP, GCP, GMP, GPvP) compliance; FDA SaMD (Software as Medical Device) clearance if applicable; HITRUST r2 + HIPAA + state laws (if clinical-trial-data spans patient data); SOC 2 Type II | 21 CFR Part 11 continuous; FDA SaMD per-clearance; HITRUST 24 months | $0.5-1.5M initial; $0.3-0.6M/yr |
| Pharma | EU | EU EMA validation per EMA's GAMP 5 (Good Automated Manufacturing Practice) framework; Annex 11 compliance; Clinical Trials Regulation (CTR) 536/2014 compliance; ENGENA pseudonymisation framework | GAMP 5 continuous; EMA per-trial validation | $0.4-1.2M initial; $0.2-0.5M/yr |
| Pharma | JP | PMDA (Pharmaceuticals and Medical Devices Agency) compliance; J-GxP equivalents; APPI Article 17 special-category compliance | PMDA continuous | $0.2-0.6M initial; $0.15-0.3M/yr |
| **Defense — classified-adjacent + ITAR + export-controlled** | US | ITAR (International Traffic in Arms Regulations) registration with US State Department DDTC; EAR (Export Administration Regulations) compliance with US BIS; DFARS 252.204-7012 cybersecurity controls; SP-CMMC Level 3+ (Cybersecurity Maturity Model Certification); FedRAMP High + IL5 + IL6 (per Gov-defense overlap); CDS (Cross Domain Solution) accreditation if classified-spanning workflows; NDAA Section 889 supply-chain attestations; UCNI (Unclassified Controlled Nuclear Information) handling for nuclear-adjacent customers | ITAR continuous (with annual self-cert); CMMC every 3 yr; FedRAMP/IL5/IL6 per their cadence | $5-15M initial; $3-8M/yr |
| Defense | UK | UK MOD's List X security clearance (facility-level); UK Security Service liaison; UK Official-Sensitive handling; ISO 27001 + ISO 9001; UK Defence Standard 05-138 compliance | MOD continuous; ISO annual | $1-3M initial; $0.5-1M/yr |
| Defense | EU (NATO) | NATO STANAG 4774 compliance; per-member-state defence-clearance; NATO C&I (Communications and Information) Agency certification for NATO-network adjacency | NATO continuous; member-state varies | $1.5-4M initial; $0.7-1.5M/yr |
| **Marketplace — physical goods** | US | Marketplace facilitator tax compliance × 50 US states (each state has its own marketplace-facilitator-tax-act since 2018-2021; nexus thresholds vary; ~9-18 months per state to operationally cover); FTC Section 5 (unfair or deceptive acts) compliance; CPSC product safety compliance; FDA registration if food/drug/cosmetics; state-specific consumer-protection laws (CA Prop 65 chemicals; NY UDAP; multiple others); state weights-and-measures if anything sold by weight; Made-In labelling under FTC origin rules; environmental laws (CA SB 343, CA AB 1276 etc.) | Marketplace facilitator continuous per state; FTC continuous; product-safety continuous | $1.5-3M initial 50-state rollout; $0.8-1.5M/yr |
| Marketplace physical | EU | VAT MOSS (or post-Brexit OSS for non-UK EU, plus separate UK VAT) registration; EU UCPD (Unfair Commercial Practices Directive) compliance; CE marking for relevant product categories; EU Toy Safety Directive 2009/48/EC; REACH (chemicals) compliance; WEEE (e-waste) compliance; EPR (Extended Producer Responsibility) per member state; Digital Services Act (DSA) compliance if marketplace-facilitator; GPSR (General Product Safety Regulation) compliance from 2024-12 | VAT continuous; DSA continuous; per-product-category continuous | $0.8-1.8M initial; $0.4-0.9M/yr |
| Marketplace physical | KR | 부가가치세 (VAT) registration; 전자상거래 등에서의 소비자보호에 관한 법률 (E-Commerce Consumer Protection Act, ECA) compliance; KC (Korea Certification) marks for relevant product categories; 전기용품 및 생활용품 안전관리법 (Electrical Appliances and Consumer Products Safety Control Act); product-liability laws | VAT continuous; KC per-product-category | $0.3-0.8M initial; $0.15-0.4M/yr |
| Marketplace physical | CN (if in scope) | CCC (China Compulsory Certification) for relevant product categories; ICP licence for content-facing surfaces; PIPL (Personal Information Protection Law) compliance; cross-border-data-transfer security assessment | CCC per-product-category; PIPL continuous | $0.5-1.5M initial; $0.3-0.7M/yr |
| **Marketplace — services (gig + consulting + skilled trades)** | US | State-specific service-platform laws (CA AB 5 / Prop 22 worker classification; CO HB 21-1115; IL HB 5189; NY SB 02137A; multiple others 2020-2026); identity verification + background checks at scale (per FCRA Section 604 if used in employment decisions); escrow regulations per state for service marketplaces; per-state contractor licensing-board integration; state UDAP compliance | Per-state continuous; FCRA continuous; per-trade-licensing continuous | $0.5-1.5M initial; $0.3-0.7M/yr |
| Marketplace services | EU | EU Platform Workers Directive 2024/2831 (transposition 2026-12-02); GDPR Article 22 + 35 for algorithmic-management; per-member-state worker-classification; per-trade-licensing per member state | Platform Workers continuous; per-member-state varies | $0.3-0.8M initial; $0.15-0.4M/yr |
| Marketplace services | KR | 부가가치세 + 종합소득세 reporting integration; identity-verification per 정보통신망법; 근로기준법 if employment-classification triggers | Per-tax-year continuous | $0.15-0.4M initial; $0.1-0.2M/yr |
| **Marketplace — c2c (peer-to-peer consumer-to-consumer)** | US | Identity verification at lower-friction-than-merchant threshold but sufficient for FTC + state UDAP; minor protection (COPPA + state-specific); local-pickup logistics + safety guidance; sales-tax-collection threshold per state | Per-state continuous; COPPA continuous | $0.3-0.7M initial; $0.15-0.3M/yr |
| Marketplace c2c | EU | DSA compliance; GDPR Article 8 (children); local-pickup safety; member-state second-hand-goods laws | DSA continuous | $0.2-0.5M initial; $0.1-0.2M/yr |
| Marketplace c2c | KR | E-Commerce Consumer Protection Act compliance; identity verification per 정보통신망법; minor protection per 정보통신망 이용촉진 및 정보보호 등에 관한 법률 | Per-tax-year continuous | $0.1-0.3M initial; $0.08-0.15M/yr |
| **Financial-services data (data plane for banks, brokers, insurers, asset managers)** | US | SOC 1 Type II (for SOX-touching workloads); SOC 2 Type II; ISO 27001:2022 + ISO 27017 + ISO 27018; FFIEC IT Examination Handbook compliance (if banking-tenant); SEC Regulation S-P + S-ID; FINRA Rule 4530 + 3110 (if broker-dealer-tenant); state insurance-information-security laws (NY DFS Part 500; multiple others); NAIC Insurance Data Security Model Law | SOC annual; ISO annual; FFIEC continuous | $0.6-1.5M initial; $0.3-0.6M/yr |
| Financial-services data | EU | EU MiCA (Markets in Crypto-Assets Regulation) if crypto in scope; EU DORA (Digital Operational Resilience Act) 2024-12-17 compliance; per-member-state financial-supervision (BaFin Germany; AMF France; FINMA Switzerland-equivalent); EBA outsourcing guidelines; PSD2/PSD3; CRD VI / CRR III if banking-tenant | DORA continuous; MiCA continuous if scope; per-member continuous | $0.5-1.2M initial; $0.3-0.6M/yr |
| Financial-services data | KR | KR-FSS (Financial Supervisory Service) cloud-outsourcing approval per 금융회사 핵심업무 위탁 관련 가이드라인; ISMS-P; CSAP for fintech tenants; 전자금융거래법 + 신용정보의 이용 및 보호에 관한 법률 (Credit Information Use and Protection Act) | KR-FSS continuous; ISMS-P annual; CSAP biennial | $0.4-1M initial; $0.2-0.4M/yr |
| Financial-services data | UK | FCA outsourcing rules (SYSC 8 / SS2/21); UK DORA-equivalent (Operational Resilience Policy 2021-onwards); SOC 2 Type II; ISO 27001 | FCA continuous; SOC/ISO annual | $0.3-0.7M initial; $0.15-0.3M/yr |

**Cost-estimate bands** are 2026-currency USD, covering initial
certification + first-year staffing + first-year audit fees. They
DO NOT include long-tail per-incident response, expansion to new
sub-markets within a market, or product-team build cost (that's
captured in the build-state quality bar §D-2 work).

The matrix is canonical and lives in
`/specs/capability-certification-matrix.json`. Modifications to
the matrix require a follow-up ADR (because changes affect roadmap
sequencing + capacity planning + sales commitments).

### D-5. Three-state lifecycle declared in tenant model

The three-state lifecycle is enforced by data in the tenant model
(per ADR-0244):

```yaml
# tenant table (microservices/tenancy/)
tenant_id: "tenant-acme-corp"
home_cell: "cell-us-east-2-a"
jurisdiction:
  primary: "US-DE"
  data_residency_allowed: ["US"]
compliance_packs:                         # ADR-0251 packs the tenant has adopted
  - pack_id: "pci-dss-l1-2026"
    version: "1.2"
    accepted_at: "2026-04-15T10:30:00Z"
  - pack_id: "soc2-t2-2026"
    version: "1.0"
    accepted_at: "2026-03-01T14:00:00Z"
eligible_capabilities:                    # Derived, not directly settable
  - capability_id: "payments-card-cnp"
    state: "LAUNCHED-IN-MARKET"
    market: "US"
    enabled_at: "2026-08-01T00:00:00Z"
  - capability_id: "marketplace-services"
    state: "BUILT"                        # Not yet launched in US
    market: "US"
identity_verification:
  level: "merchant-tier-3"                # Per ADR-0253
  verified_at: "2026-04-10T09:00:00Z"
kyb_status:
  status: "VERIFIED"
  jurisdiction: "US-DE"
  documents_signed:
    - "platform-services-agreement-v2026.1"
    - "payments-merchant-agreement-v2026.1"
```

```yaml
# cell registry record (cloud-iac OpenTofu state)
cell_id: "cell-us-east-2-a"
provider: "aws"
region: "us-east-2"
certifications:                           # ADR-0248 cell certifications
  - cert_id: "soc2-type-ii"
    issuer: "external-auditor-deloitte"
    issued_at: "2026-01-15"
    expires_at: "2027-01-15"
    evidence_pack_url: "cosign://...sha256:..."
  - cert_id: "pci-dss-l1"
    issuer: "external-qsa-coalfire"
    issued_at: "2025-12-01"
    expires_at: "2026-12-01"
    evidence_pack_url: "cosign://...sha256:..."
  - cert_id: "fedramp-moderate"
    issuer: "fedramp-pmo"
    issued_at: "2026-03-15"
    expires_at: "2029-03-15"
    evidence_pack_url: "cosign://...sha256:..."
```

The relationship is:

```
tenant.eligible_capabilities[capability_X].state == LAUNCHED-IN-MARKET
  ⇔
   capability_X is in state BUILT (globally)
   AND
   home_cell(tenant).certifications ⊇ required_certs(capability_X, market(tenant))
   AND
   tenant.compliance_packs ⊇ required_packs(capability_X, market(tenant))
   AND
   tenant.identity_verification.level ⊇ required_identity_level(capability_X)
   AND
   tenant.kyb_status == VERIFIED for jurisdiction(market(tenant))
```

The `eligible_capabilities` field is **derived**, not settable. It is
computed by the tenancy substrate's reconciler on every change to
the constituent inputs (tenant compliance packs, cell certifications,
capability state). The reconciler emits to the audit chain (class
`TenantCapabilityEligibilityChanged`).

### D-6. Cedar gate composition

Every capability action is gated by a Cedar fragment composed per
the universal-gate doctrine (ADR-0243). The canonical fragment shape:

```cedar
// microservices/policy-engine/fragments/baseline/payments-card-cnp-permits.cedar
// SCOPE: baseline
// FRAGMENT_ID: payments-card-cnp-permits
// VERSION: 3
// APPLIES_TO_ACTIONS: [Payments::Action::Charge, Payments::Action::Refund,
//                     Payments::Action::Capture, Payments::Action::Void]
// APPLIES_TO_RESOURCES: [Payments::PaymentIntent]
// SIGNED_BY: org-baseline-key
// EFFECTIVE_AT: 2026-08-01T00:00:00Z

permit (
  principal,
  action in [Payments::Action::Charge, Payments::Action::Refund,
             Payments::Action::Capture, Payments::Action::Void],
  resource is Payments::PaymentIntent
)
when {
  // 1. The capability must be in BUILT state globally
  context.capability_state["payments-card-cnp"] == "BUILT"

  // 2. The capability must be LAUNCHED in the market the action targets
  && context.capability_market_state["payments-card-cnp"][resource.market] == "LAUNCHED-IN-MARKET"

  // 3. The tenant's home cell must carry the required certifications
  && context.cell_certifications.contains_all([
       "pci-dss-l1",
       "soc2-t2"
     ])

  // 4. Plus market-specific certifications
  && (resource.market != "US"
      || context.cell_certifications.contains("us-mtl-" ++ resource.us_state))
  && (resource.market != "EU"
      || context.cell_certifications.contains("eu-psd2"))
  && (resource.market != "KR"
      || context.cell_certifications.contains("kr-결제대행업"))
  && (resource.market != "JP"
      || context.cell_certifications.contains("jp-funds-settlement-act"))
  && (resource.market != "SG"
      || context.cell_certifications.contains("sg-mas-ps-act"))

  // 5. Tenant must have adopted the required compliance pack
  && principal.tenant.compliance_packs.contains_all([
       "pci-dss-l1-2026",
       "soc2-t2-2026"
     ])

  // 6. Plus market-specific pack
  && (resource.market != "US"
      || principal.tenant.compliance_packs.contains("payments-us-2026"))
  && (resource.market != "EU"
      || principal.tenant.compliance_packs.contains("payments-eu-2026"))
  && (resource.market != "KR"
      || principal.tenant.compliance_packs.contains("payments-kr-2026"))

  // 7. Tenant identity verification at merchant tier or above
  && principal.tenant.identity_verification.level
       in ["merchant-tier-2", "merchant-tier-3", "merchant-tier-4"]

  // 8. KYB status verified for the resource's market jurisdiction
  && principal.tenant.kyb_status == "VERIFIED"
  && principal.tenant.kyb_jurisdiction == jurisdiction_for_market(resource.market)
};

// Companion default-deny for safety (per ADR-0243 D-3)
forbid (
  principal,
  action in [Payments::Action::Charge, Payments::Action::Refund,
             Payments::Action::Capture, Payments::Action::Void],
  resource is Payments::PaymentIntent
)
unless {
  context.capability_state["payments-card-cnp"] == "BUILT"
};
```

Each capability authors:

- A `<capability>-permits.cedar` fragment at
  `microservices/policy-engine/fragments/baseline/`.
- Per-market overlay fragments at
  `microservices/policy-engine/fragments/overlay/<jurisdiction>/`
  encoding market-specific obligations.
- A compliance-pack fragment at
  `microservices/policy-engine/fragments/pack/<pack-id>/` referenced
  by tenants who have adopted the pack (per ADR-0251).

The per-capability Cedar fragment set is reviewed by multispectrum
review v2.4.0 (per ADR-0243 D-8) before publication. The fragments
are signed by the appropriate signing keys (per ADR-0243 D-2 + D-5).

### D-7. Per-capability launch-runbook template

Every capability in §D-4 requires a launch runbook authored before
its first market launch. The runbook template lives at
`docs/standards/capability-launch-runbook-template.md` and produces,
per (capability, market) tuple, a runbook at
`docs/runbooks/launch/<capability>-<market>.md` containing:

1. **Capability + market identification.** Capability ID, market ID,
   target launch date, launch decision body (council-product +
   council-architecture sign-off).
2. **Certification chain.** List every certification with: regulator,
   issued-at date, expires-at date, evidence-pack URL, renewal
   responsible team.
3. **Regulator letter.** Signed regulator approval letter or
   equivalent attestation, archived at cosign-attested storage.
4. **Compliance pack reference.** Pack ID + version + activation
   date for every pack required by the capability in the target
   market.
5. **Cedar fragment reference.** Fragment IDs + versions for every
   fragment that gates the capability in the target market.
6. **Tenant onboarding flow update.** Identification of the per-
   market onboarding wizard updates; tested with sandbox-tenant KYB
   + KYC + identity-verification flows.
7. **Observability dashboard reference.** Capability-and-market-
   filtered dashboard URL; expected SLO + burn-rate alerts.
8. **On-call rotation.** Team(s) on rotation for the capability +
   market combination; escalation paths; secondary on-call;
   timezone coverage matrix.
9. **Support documentation.** Customer-facing help center articles
   in market-native language; internal support runbooks for
   common issues.
10. **Marketing material approval.** Signed marketing-asset list from
    legal review (per ADR-0144 + market-specific UDAAP/UCPD/etc.).
11. **Legal sign-off.** Signed launch authorization from
    `oyatie.legal` sub-scope (per ADR-0242 §D-2).
12. **Rollback procedure.** Per ADR-0241 + §D-12, the launched-to-
    DEGRADED-to-DARK procedure if certification lapses or
    incident-triggered rollback is needed.
13. **First-customer pilot plan.** Phased rollout: ≤ 10 pilot
    tenants for week 1; expand to 100 tenants week 2-4; full
    availability month 2; with metrics gates between each phase.

The runbook is reviewed by multispectrum review v2.4.0 with facets
F1 (correctness), F5 (security), F9 (operational readiness), A4
(architecture-adherence), and M1 (compliance) at minimum.

### D-8. Roadmap timeline

The portfolio-level capability launch roadmap is canonical and lives
at `/specs/capability-launch-roadmap.json`. The timeline is multi-
year and milestone-anchored. All dates are 2026-relative to
masterplan kickoff.

| Year | Capabilities BUILT (substrate complete) | Capabilities LAUNCHED-IN-MARKET |
|---|---|---|
| **Year 0-1** (2026-2027) | Payments substrate; healthcare de-identification substrate; breach-notification substrate; encryption substrate (encryption-BYOK); consent-management substrate; identity-verification substrate (ADR-0253); tax-engine substrate; marketplace physical-goods substrate (ADR-0252); marketplace services substrate; marketplace c2c substrate; financial-services-data substrate (ADR-0254); audit-chain substrate; cell certifications substrate (ADR-0251 + ADR-0248) | (None launched; oyatie sandbox tenants only) |
| **Year 1.5** (~2027-Q3) | Pharma 21 CFR Part 11 substrate; education FERPA substrate; healthcare HIPAA tier-1 substrate | Payments Phase 1 — US priority states (CA, NY, TX, FL, IL, WA, MA, NJ, CO; ~10 states with MTL secured) with PCI L1 + SOC 2 T2 certifications complete |
| **Year 2** (~2028-Q1) | Government FedRAMP Moderate substrate; defense IL5 substrate | Payments US — full 50 states (remaining 40 MTLs); Payments EU — PSD2 e-money authorisation passportable across EEA (member-state issuer pending: likely IE or LU based on regulator velocity); Marketplace subscriptions — SaaS-only first wave (US + EU); Healthcare tier-1 (limited B2B; pilot tenants only) |
| **Year 2.5** (~2028-Q3) | Government StateRAMP substrate; defense ITAR substrate | Marketplace services — US + EU + KR; Healthcare expanded — full HIPAA scope US + GDPR Article 9 EU; Payments — UK + JP + SG additions |
| **Year 3** (~2029-Q1) | Pharma EU EMA validation substrate; defense IL6 substrate | Marketplace physical goods — US (50-state marketplace facilitator); FedRAMP Moderate (US federal launch); Education K-12 US; Education EU |
| **Year 3.5** (~2029-Q3) | Defense classified-adjacent substrate; nuclear UCNI substrate (optional, only if customer demand) | Marketplace c2c — US + EU + KR; FedRAMP High; KR-FSS financial-services data; Healthcare KR + JP |
| **Year 4** (~2030-Q1) | (Substrate work tapering; product-feature work accelerating) | IL5 (defense); IL6 (defense if pursued); ITAR-controlled defense capability; Pharma — US 21 CFR Part 11 + EU EMA; Marketplace physical goods EU + KR + CN (if in scope); Financial-services data EU MiCA + DORA |

The roadmap is reviewed annually by council-product + council-
architecture + ops-compliance and updated. Changes to roadmap that
affect customer-facing commitments require a follow-up ADR.

### D-9. Anti-bypass rule

Built-but-unlaunched capabilities CANNOT be enabled for tenants
outside their certification gate via tenant configuration override,
admin override, allowlist, feature-flag override, or any other
bypass mechanism. The Cedar policy refuses every such attempt; the
CI lane `oya-check-anti-bypass-built-only-tenant` enforces.

Specifically:

- A capability in state BUILT (not LAUNCHED-IN-MARKET) only permits
  invocation by principals in the `oyatie.dev.*` or `oyatie.preview.*`
  or `oyatie.ci.*` sub-scopes (per ADR-0242 §D-2 ephemeral tenants).
  No customer tenant can invoke a built-but-unlaunched capability.
- A capability LAUNCHED in market A does not permit invocation for
  tenants whose home cell is not certified for market A.
- The Cedar evaluator at every gate evaluates the
  `context.capability_market_state` + `context.cell_certifications`
  + `context.tenant_compliance_packs` triple; missing any required
  element → `Forbid`.
- A purported tenant config that claims to enable a built-but-
  unlaunched capability is rejected by the tenancy substrate's
  admission gate; the rejection emits to the audit chain
  (class `BuiltButUnlaunchedBypassAttempt`).
- A tenant admin attempting to manually override + the override
  Cedar fragment refusing → SEV-2 alert to council-security +
  ops-compliance.

The anti-bypass invariant is enforced both at policy time (Cedar)
and at admission time (Kyverno on tenant-config CRD per ADR-0183).
Defense in depth.

### D-10. Pre-launch testing

Built-but-unlaunched capabilities are exercised by:

- **`oyatie.dev.<engineer-id>` sandbox tenants.** Engineers exercise
  the capability against the real platform with isolated data.
- **`oyatie.preview.<pr-number>` ephemeral tenants.** Each PR
  exercises the capability against a per-PR cell snapshot.
- **`oyatie.ci.<run-id>` ephemeral tenants.** CI integration tests
  exercise the capability in sandboxed test cells with synthetic data.
- **Multispectrum-review v2.4.0 facets exercise the capability** via
  facet-specific test invocations (e.g., F5 security drills emit
  attack-pattern simulations; F6 performance drills emit load
  profiles).
- **DR drills exercise the capability's failover paths.**
- **Audit-chain replay exercises the capability's per-action
  emission.**

End-to-end flows are verified; SLO instrumentation is proven; DR
behaviour is observed; the capability is "ops-confidence" ready
before its first market launch.

Per-tenant pilot launches (the ≤ 10 pilot tenants of D-7's
runbook) are the bridge between BUILT and LAUNCHED-IN-MARKET-AT-
SCALE: the capability is LAUNCHED-IN-MARKET for those tenants only,
with telemetry monitored intensively, before broader rollout.

### D-11. Certification evidence retention

Per-certification audit-chain receipts + cosign-attested compliance
evidence packages are retained on a regulator-extractable timeline.

For each certification of a capability + market:

- **Initial certification evidence pack.** A versioned bundle
  containing: every artifact submitted to the regulator/auditor; the
  regulator's response (acceptance letter, conditional acceptance,
  rejection); each remediation cycle; the final issued certificate.
  Stored at cosign-attested immutable storage; URL referenced in the
  cell's `certifications[]` row.
- **Continuous monitoring evidence.** Per-month log of: control
  effectiveness samples, drill receipts, incident reports, change-
  control records, vulnerability scan summaries. Cosign-attested;
  aggregated quarterly per ADR-0241 D-8.
- **Renewal evidence.** Pre-renewal preparation pack (gap analysis,
  remediation, fresh evidence); auditor walkthrough; renewal
  issuance. Same chain as initial.
- **Lapse + recovery evidence.** If a certification lapses, the
  audit-chain row captures: lapse-detection event; recovery plan;
  remediation completion; recertification (or capability sunset).

Retention is the longest of:

- The regulator's stated minimum (PCI DSS = 1 year of evidence per
  cycle; HIPAA = 6 years; FedRAMP = 3 years per ATO; SOC 2 = 7
  years for SOX-touching workloads).
- Customer contract minimums (some enterprise customers require 10+
  year retention).
- Litigation hold (per FRCP 37(e); supersedes other timelines).

Evidence packs are signed by the cell's per-certification signing
key + cosign-attested + Merkle-anchored to the audit chain. Tamper
detection is automatic.

### D-12. Rollback model — LAUNCHED → DEGRADED → DARK

If a certification lapses (revocation, expiration, regulator
suspension) or a SEV-1 incident requires it, the capability
transitions through:

```
LAUNCHED-IN-MARKET
  ↓ (cert lapse or SEV-1 detected)
DEGRADED
  ↓ (grace period expires or unresolved)
DARK
```

**DEGRADED state semantics:**

- Existing tenants retain access; new tenant onboarding is paused.
- Existing tenants are notified via tenant admin console + email
  + (per the regulator's notification requirement) postal mail.
- Cedar fragment is updated: the `Permit` for new-tenant-onboarding
  becomes `Forbid`; the `Permit` for existing-tenant-usage remains
  but a `context.is_grace_period == true` annotation triggers a
  customer-facing degradation banner.
- Grace period varies per regulator: PCI DSS = 60 days; HIPAA =
  immediate for new PHI but 60 days for existing; FedRAMP =
  per-ATO terms; EU PSD2 = per-NCA (national competent authority)
  decision; KR-FSS = per-FSS notice.
- Per-tenant migration plans drafted: where can the tenant move?
  What capability sunsets when? What data export is offered?

**DARK state semantics:**

- The capability is forbidden for all tenants (existing + new) in
  the affected market.
- Cedar fragment for the capability is sunset; the Cedar evaluator
  returns `Forbid` on every invocation.
- Affected data is preserved (subject to data-residency rules per
  ADR-0049 + ADR-0240) for tenant export + regulator inspection.
- Per the contractual SLA, tenant gets a credit + an alternative
  capability path (if available).
- The capability transitions back to BUILT (globally) until
  recertification permits a new LAUNCHED-IN-MARKET attempt.

Rollback is a Cedar fragment publication event (per ADR-0243 fragment
hot-reload + emergency-permit/forbid pattern). The transition emits
to the audit chain (class `CapabilityMarketStateTransition`)
carrying: trigger (cert-lapse, sev-1, regulator-order, scheduled-
sunset), grace-period-duration, affected-tenant-count, recovery-
plan-url.

The `oya gate validate capability-rollback-readiness` lane verifies
that every LAUNCHED-IN-MARKET capability has a rollback runbook
authored and tested via tabletop exercise within the trailing 12
months.

## Alternatives considered

### Alt-1. Build only after certification (waterfall)

Defer build of certification-gated capabilities until each
certification clears. Build starts on cert-grant day.

**Pros:**

- Minimal up-front engineering cost (no built-but-unlaunched
  inventory).
- Sequential clarity (one capability at a time per market).
- Regulator engagement happens against an empty-stack target — some
  regulators view this favourably ("we're going to build this exactly
  to your spec").

**Cons:**

- **Time-to-market loss.** Cert + build sequentially = 18-42 months
  per capability per market. Competitors who build ahead arrive
  earlier; markets are lost before launch.
- **Build-from-zero risk under cert pressure.** When the cert
  arrives, the engineering team starts coding against the spec — but
  the spec was written months ago; ambiguities and edge cases
  emerge. Re-cert may be required if implementation diverges.
- **Capacity-planning impossibility.** Cannot pre-stage engineering
  capacity for a 18-month build that hasn't started.
- **No pilot tenants until launch.** Pilot tenant feedback cannot
  shape the substrate; bugs surface at market launch under full
  load.
- **Contradicts every named hyperscaler reference.** Apple, Stripe,
  AWS, Microsoft, Salesforce, Palantir, Cloudflare all build first.
- **Foundry pipeline + autonomous-masterplan-execution is gated.**
  Per `feedback_autonomous_implementation_artifacts`, the masterplan
  is supposed to be autonomously implementable. If build is gated
  on cert, the masterplan stalls for years per capability.

**Rejected** because the cons are unbounded time-to-market loss
across every regulated capability and the doctrine contradicts every
named industry reference.

### Alt-2. Build at certification-process kickoff (parallel start)

Start build the day the certification process kicks off. Build +
cert run in parallel.

**Pros:**

- Reduces time-to-market vs Alt-1 by overlapping build + cert.
- Some regulator engagement is concurrent with engineering, which
  surfaces requirements earlier.
- Pilot tenants can engage before cert grants.

**Cons:**

- **Cert-driven scope thrash.** Regulator findings during cert force
  late-stage code changes; the build is sufficiently far along that
  changes are expensive. Re-audit may be required.
- **Insufficient slack for ops drill cadence.** Per ADR-0241, T1
  capabilities need quarterly drills + cumulative drill-success
  evidence; a parallel-build can't accumulate the evidence quickly
  enough for cert evaluators who want 6-12 months of operational
  history.
- **Still slower than build-ahead.** Build-ahead amortizes build
  across all markets simultaneously; parallel-build starts a fresh
  build per market.
- **Pilot tenants are constrained to a single market.** Build-ahead
  allows pilot tenants in every market simultaneously (in sandbox
  + via dev tenancy); parallel-build can't.

**Rejected** because the parallel-start still produces re-audit cycles
+ insufficient operational-history slack + slower than build-ahead.

### Alt-3. Partial build (skeletons only)

Build skeleton/scaffold of each capability day-one (the µservice
exists, the API surface is declared, but the action handlers return
`Unimplemented`). Implement on-demand per market.

**Pros:**

- Low up-front cost (just the skeleton).
- Some pre-cert artifacts exist (API surface, manifest, threat
  model template).
- Each market's implementation can be customized to that market's
  requirements.

**Cons:**

- **Skeleton ≠ ops-grade.** Skeletons can't pass the §D-2 quality
  bar; skeletons can't be drilled in DR; skeletons can't emit useful
  observability.
- **Per-market customization re-implements common machinery N times.**
  Tax engine for US is re-implemented for EU is re-implemented for
  KR — but each implementation should share the canonical-base per
  ADR-0064. Partial build implies per-market re-implementation.
- **Pilot tenants can't use skeletons.** Pilot feedback delayed
  until per-market implementation.
- **Regulators see skeletons as "not yet operational"** — and may
  require the operational completion before issuing the certificate.
  Back to Alt-1's problem.
- **Foundry pipeline + masterplan execution can't be driven against
  skeletons.** The autonomous workflow needs operational substrate
  to operate against.

**Rejected** because skeleton-only doesn't satisfy the operational
readiness bar that the §D-2 build-state quality requires and
contradicts the canonical-base-localization pattern per ADR-0064.

### Alt-4. Launch globally simultaneously (no per-market sequencing)

Build day-one and launch globally on day-one across all markets.

**Pros:**

- Maximum revenue capture upon launch.
- Marketing simplicity (one launch event).
- No "we're launched in US but not EU" customer confusion.

**Cons:**

- **Certification per-market doesn't align.** Even if everything's
  built, certifications land per-market on different timelines.
  Launching globally means launching some markets without
  certifications, which is illegal in those markets.
- **KYB/KYC + tax + identity-verification flows are per-market.**
  Launching globally without per-market onboarding is unworkable.
- **Per-market operational on-call requires per-market staffing.**
  Cannot launch in a market without timezone-coverage.
- **Per-market regulator pre-approval is required for some
  certifications** (e.g., FedRAMP requires regulator pre-engagement;
  KR CSAP requires KISA pre-engagement). Global simultaneity is
  incompatible with this sequenced pre-engagement.
- **Apple Pay's per-country launch shape was deliberate; ditto
  Stripe + AWS regions.** All sequenced per-market for the
  certification + operational + KYB reasons above.

**Rejected** because per-market sequencing is mandatory; "global on
day-one" is incompatible with how certifications work.

### Alt-5. Build-ahead, per-market launch (CHOSEN)

The selected alternative, fully specified in §Decision.

**Pros:**

- **Matches every named industry reference** (Apple Pay, Stripe,
  AWS, Microsoft, Salesforce, Palantir, Cloudflare).
- **Minimizes time-to-market per market.** Build cost is amortized
  across markets; launch is gated only on per-market certification +
  KYB readiness.
- **Enables pilot tenants in every market simultaneously** via
  sandbox tenants (`oyatie.dev.*`, `oyatie.preview.*`); pilot
  feedback shapes the substrate before market launch.
- **Enforces three-state lifecycle via Cedar gates + tenancy
  substrate + cell certifications** — drift is structurally prevented
  by composition of the keystone bundle.
- **Aligns with autonomous-masterplan-execution doctrine.** The
  masterplan can be implemented autonomously because build is not
  blocked on external certification timelines.
- **Per-market scope-thrash minimized.** Once a capability is BUILT
  to the §D-2 quality bar, market-specific work is overlay (Cedar
  fragments + compliance pack + market-specific onboarding flow)
  rather than re-implementation.
- **Failure-mode-resistant.** The three-state lifecycle prevents the
  failure modes of Alt-1 (build-from-zero-on-cert) and Alt-2 (cert-
  driven scope thrash) and Alt-3 (skeleton not ops-grade) and Alt-4
  (illegal global launch).

**Cons:**

- **Pre-revenue capital cost.** Building before launching = paying
  engineering cost without revenue offset. Mitigated by phased
  roadmap (§D-8) so build is paced; revenue starts earliest at Year
  1.5 (US payments).
- **Built-but-unlaunched inventory must be kept fresh.** A capability
  built in Year 1 but launched in Year 4 must stay current with
  three years of substrate evolution, security updates, dependency
  upgrades. Mitigated by treating built-but-unlaunched capabilities
  with the same DR + security + maintenance discipline as launched
  ones (per ADR-0241 + ADR-0247).
- **Operational + observability complexity.** Per-cell + per-
  capability + per-market state matrix is large. Mitigated by the
  three-state lifecycle being machine-readable (tenancy + cell
  + capability state tables) + dashboards that filter cleanly.

**Accepted** as the foundational keystone for build-launch
sequencing.

## Consequences

### Positive

1. **Time-to-market minimized per market.** Once cert + KYB + Cedar
   pack lands for market M, launch is days-not-months.
2. **Build is not on the certification critical path.** Cert lapses
   or delays don't paralyze engineering.
3. **Pilot-tenant feedback shapes the substrate before market
   launch.** Sandbox tenants exercise every capability throughout
   build; bugs surface in sandbox, not in market.
4. **Hyperscaler-shape achieved.** Matches Apple Pay, Stripe, AWS,
   Microsoft, Salesforce, Palantir, Cloudflare.
5. **Autonomous masterplan execution unblocked.** Foundry workflows
   can build capabilities continuously; per-market launch gates are
   workflow-evaluable.
6. **Per-capability + per-market state is observable.** Dashboards
   show capability state per market; cert expiration alerts fire 90
   days before lapse.
7. **DR + BC + sustainability discipline applies uniformly.** Built-
   but-unlaunched capabilities follow the same ADR-0241 drill cadence
   as launched capabilities.
8. **Cedar-gate composition enforces invariants.** Anti-bypass +
   cell-certification + tenant-compliance-pack triple-check at every
   action; defense-in-depth.
9. **Three-state lifecycle prevents drift.** "Are we ready to
   launch?" is answered by reading the eligibility-state machine,
   not by a human reading tea leaves.
10. **Regulator engagement is calmer.** Regulators see an
    operationally-mature substrate when they begin certification
    review; less remediation; faster issuance.

### Negative — pre-revenue capital cost

1. **Build cost precedes revenue.** All §D-4 capabilities cost
   engineering capacity through Year 0-2 before any revenue lands.
   Mitigated by:
   - Phased roadmap (§D-8) sequences build to align with capacity.
   - Substrate is shared across capabilities (per ADR-0064 canonical-
     base-localization) — payments substrate serves Phase 1 US payments
     AND Phase 4 EU payments AND Phase 5 JP payments; cost is paid
     once.
   - Internal sandbox tenant use generates "free" pilot data + bug
     surfacing, reducing post-launch SEV-1 risk.
   - Pre-revenue period is bounded; Year 1.5 first revenue from US
     payments.
2. **Maintenance cost of built-but-unlaunched inventory.** Per ADR-
   0241 + ADR-0247, every capability in BUILT state remains drill-
   passed + security-updated. This is ongoing cost. Mitigated by
   automation per `feedback_automate_everything`.
3. **Engineering attention is divided across many concurrent
   capabilities.** Mitigated by µservice-level ownership (per ADR-
   0131 flat layout) — each capability has one owning team,
   limiting context-switching.

### Operational

1. **New CI lanes (advisory until three-state-lifecycle lands;
   BLOCKER post-bootstrap):**
   - `oya-check-capability-three-state-coherence` — verifies every
     capability has ARCHITECTED + BUILT state declared; per-market
     LAUNCHED state coherent.
   - `oya-check-built-but-unlaunched-cedar-gate` — verifies Cedar
     fragments refuse customer-tenant invocation of built-but-
     unlaunched capabilities.
   - `oya-check-capability-launch-runbook-completeness` — verifies
     every LAUNCHED-IN-MARKET (capability, market) tuple has a
     completed runbook.
   - `oya-check-certification-evidence-retention` — verifies cosign-
     attested evidence packs exist for every certification in
     `cell.certifications[]`.
   - `oya-check-anti-bypass-built-only-tenant` — verifies no tenant
     config grants access to a built-but-unlaunched capability.
   - `oya-check-capability-rollback-readiness` — verifies every
     LAUNCHED-IN-MARKET capability has a rollback runbook + tabletop
     exercise within trailing 12 months.
2. **New µservice surfaces / extensions:**
   - `microservices/tenancy/` adds `tenant.eligible_capabilities[]`
     derived column + reconciler.
   - `microservices/cloud-iac/ARCHITECTURE.md#cell-provisioning` adds `cell.certifications[]` metadata +
     evidence-pack reference.
   - `microservices/policy-engine/` adds per-capability + per-market
     Cedar fragments.
   - `microservices/observability/` adds per-capability + per-market
     dashboards.
   - `microservices/audit-chain/` adds new audit classes:
     `CapabilityBuiltPromotion`, `CapabilityLaunchedInMarketPromotion`,
     `CapabilityMarketStateTransition`,
     `TenantCapabilityEligibilityChanged`,
     `BuiltButUnlaunchedBypassAttempt`.
3. **New specs:**
   - `/specs/capability-certification-matrix.json` (canonical from §D-4)
   - `/specs/capability-launch-roadmap.json` (canonical from §D-8)
   - `/specs/capability-launch-runbooks.json` (per-capability +
     per-market runbook index)
4. **Tooling:**
   - `oya capability state get <capability> --market <market>`
     CLI reports current state.
   - `oya capability launch propose --capability <c> --market <m>`
     workflow drives the runbook completion + multispectrum review.
   - `oya capability rollback initiate --capability <c> --market <m>
     --reason <r>` workflow drives the LAUNCHED → DEGRADED → DARK
     transition.
5. **Observability:**
   - Per-capability + per-market state dashboard.
   - Cert-expiration-90-days-out alerts (SEV-3 ramping to SEV-2 at
     30 days).
   - Built-but-unlaunched maintenance status (drill-currency + sec-
     update-currency + dependency-freshness per capability).

### Sustainability

- Per ADR-0174 cost-tag, each capability + market combination
  carries a sustainability metric (compute, storage, network egress
  attribution). Built-but-unlaunched capabilities still consume
  sustainability budget (drills + sandbox tenant use + maintenance
  CI); mitigated by the substrate-sharing efficiency per ADR-0064.
- The roadmap (§D-8) intentionally sequences high-cost capabilities
  (defense IL5/6) to later years where sustainability budget can
  absorb the dedicated SCIF + classified-adjacent compute overhead.

### Compliance

- **GDPR Article 25 (Data Protection by Design and by Default).**
  Build-ahead doctrine satisfies the "by design" requirement;
  capabilities are designed with privacy + protection before any
  market launch.
- **EU AI Act Article 9 (Risk management system).** Per ADR-0144,
  high-risk capabilities are subject to risk management before
  market placement; build-ahead aligns by construction.
- **HIPAA Security Rule §164.308(a)(1)(ii)(A) (Risk Analysis).**
  Risk analysis is performed during build state; documented in
  threat models per §D-2.
- **PCI DSS Requirement 6.2 (Develop applications securely).**
  Secure development through build state.
- **FedRAMP Continuous Monitoring.** Built-but-unlaunched substrate
  is in continuous monitoring before market launch; smooths the
  cert kickoff.
- **SOC 2 CC8.1 (Change Management).** Change management is in
  place throughout build state; SOC 2 audit can sample.
- **ISO 22301:2019 (Business Continuity Management Systems).**
  Built-but-unlaunched capabilities have DR drills per ADR-0241;
  business continuity is end-to-end before launch.

## Implementation surface

The following artifacts are required for this keystone to be
considered implemented:

| Artifact | Status |
|---|---|
| `/specs/capability-certification-matrix.json` | NEW — canonical capability + cert matrix per §D-4 |
| `/specs/capability-launch-roadmap.json` | NEW — multi-year per-market launch sequence per §D-8 |
| `/specs/capability-launch-runbooks.json` | NEW — per-capability + per-market runbook index |
| `/specs/platform-architecture.json` (this keystone's `platform.capability_lifecycle` section) | NEW — three-state lifecycle definition |
| `microservices/tenancy/src/eligible_capabilities_reconciler.rs` | NEW — derived-column reconciler |
| `microservices/tenancy/migrations/000X_add_eligible_capabilities.sql` | NEW |
| `microservices/cloud-iac/ARCHITECTURE.md#cell-provisioning` | UPDATE — cell certification metadata owner after ADR-0333 |
| `microservices/observability/ARCHITECTURE.md#cell-health` | UPDATE — certification health evidence owner after ADR-0333 |
| `microservices/policy-engine/fragments/baseline/payments-card-cnp-permits.cedar` | NEW |
| `microservices/policy-engine/fragments/baseline/payments-ach-permits.cedar` | NEW |
| `microservices/policy-engine/fragments/baseline/healthcare-phi-permits.cedar` | NEW |
| `microservices/policy-engine/fragments/baseline/government-fedramp-permits.cedar` | NEW |
| `microservices/policy-engine/fragments/baseline/education-ferpa-permits.cedar` | NEW |
| `microservices/policy-engine/fragments/baseline/pharma-21-cfr-11-permits.cedar` | NEW |
| `microservices/policy-engine/fragments/baseline/defense-itar-permits.cedar` | NEW |
| `microservices/policy-engine/fragments/baseline/marketplace-physical-permits.cedar` | NEW |
| `microservices/policy-engine/fragments/baseline/marketplace-services-permits.cedar` | NEW |
| `microservices/policy-engine/fragments/baseline/marketplace-c2c-permits.cedar` | NEW |
| `microservices/policy-engine/fragments/baseline/financial-services-data-permits.cedar` | NEW |
| `microservices/policy-engine/fragments/overlay/us/payments-mtl-overlay.cedar` | NEW |
| `microservices/policy-engine/fragments/overlay/eu/payments-psd2-overlay.cedar` | NEW |
| `microservices/policy-engine/fragments/overlay/kr/payments-결제대행업-overlay.cedar` | NEW |
| `microservices/policy-engine/fragments/overlay/<market>/<capability>-overlay.cedar` | NEW (matrix per §D-4) |
| `microservices/audit-chain/src/capability_lifecycle_events.rs` | NEW — new audit-event classes |
| `microservices/observability/dashboards/capability-launch-status.md` | NEW — per-capability + per-market state dashboard |
| `microservices/observability/dashboards/certification-expiration.md` | NEW — 90-day cert-expiration alerts |
| `microservices/observability/dashboards/built-but-unlaunched-maintenance.md` | NEW — drill + sec-update currency |
| `tools/oya-check-capability-three-state-coherence/` | NEW |
| `tools/oya-check-built-but-unlaunched-cedar-gate/` | NEW |
| `tools/oya-check-capability-launch-runbook-completeness/` | NEW |
| `tools/oya-check-certification-evidence-retention/` | NEW |
| `tools/oya-check-anti-bypass-built-only-tenant/` | NEW |
| `tools/oya-check-capability-rollback-readiness/` | NEW |
| `docs/standards/capability-launch-runbook-template.md` | NEW — runbook template |
| `docs/standards/capability-build-state-quality-bar.md` | NEW — §D-2 quality-bar checklist |
| `docs/runbooks/launch/<capability>-<market>.md` | NEW per (capability, market) tuple — populated as each launch approaches |
| `docs/runbooks/capability-rollback-procedure.md` | NEW — generic rollback template |
| `docs/runbooks/certification-evidence-pack-curation.md` | NEW — evidence retention runbook |
| `cli/oya/src/capability.rs` | NEW — capability state CLI |

## Verification

- [ ] `/specs/capability-certification-matrix.json` enumerates every capability + market + certification + cost band per §D-4.
- [ ] `/specs/capability-launch-roadmap.json` enumerates per-year per-market launch sequence per §D-8.
- [ ] `microservices/tenancy/` has `eligible_capabilities[]` derived column; reconciler emits `TenantCapabilityEligibilityChanged` on input change.
- [ ] `microservices/cloud-iac/ARCHITECTURE.md#cell-provisioning` has `certifications[]` metadata; populated with at least the cell's currently-held certifications.
- [ ] `microservices/policy-engine/fragments/baseline/` has a `<capability>-permits.cedar` for every capability in §D-4.
- [ ] `oya gate validate capability-three-state-coherence` exits 0 on a bootstrapped instance.
- [ ] `oya gate validate built-but-unlaunched-cedar-gate` exits 0 — built-but-unlaunched capabilities Cedar-refuse customer-tenant invocation.
- [ ] `oya gate validate capability-launch-runbook-completeness` exits 0 — every LAUNCHED-IN-MARKET (capability, market) has a runbook at `docs/runbooks/launch/<capability>-<market>.md`.
- [ ] `oya gate validate certification-evidence-retention` exits 0 — every entry in `cell.certifications[]` has a cosign-attested evidence pack.
- [ ] `oya gate validate anti-bypass-built-only-tenant` exits 0 — no tenant config grants access to built-but-unlaunched capability.
- [ ] `oya gate validate capability-rollback-readiness` exits 0 — every LAUNCHED-IN-MARKET capability has rollback runbook + recent tabletop exercise.
- [ ] Per-capability + per-market dashboard exists; renders the eligibility matrix.
- [ ] Cert-expiration alerts fire at 90 days, ramp to SEV-2 at 30 days, escalate to SEV-1 at lapse.
- [ ] Sandbox-tenant invocation of every built-but-unlaunched capability succeeds end-to-end (per `oyatie.dev.*` + `oyatie.preview.*` test scope).
- [ ] Audit-chain emits `CapabilityBuiltPromotion` on first build-state promotion of each capability.
- [ ] Audit-chain emits `CapabilityLaunchedInMarketPromotion` on first launch-state promotion of each (capability, market) tuple.
- [ ] Rollback drill exercised at least once per capability per year via tabletop; audit-chain row exists.

## References

### Industry sources

- **Apple Pay country launch history.** Apple Newsroom press
  releases 2014-2024 — every market launch documented with date,
  certification chain, and partner-bank list. Apple Pay's per-
  country launch shape is the canonical reference for build-ahead-
  per-market-launch.
- **Stripe Engineering geographic-expansion blog series**
  (stripe.com/blog "Now available in...", "Stripe expands to..."
  from 2012-2025). Stripe's country launches documented with
  certification + regulator + onboarding-flow narrative.
- **AWS regional service launch pattern**, AWS What's New blog
  (aws.amazon.com/about-aws/whats-new/) + AWS re:Invent keynotes
  2014-2024. Regional builds precede regional launches; certifications
  follow per-region.
- **AWS GovCloud (US) launch history** (2011-08; 2018-12 East;
  evolving 2019-2024 with FedRAMP High + IL4 + IL5 capability
  additions). aws.amazon.com/govcloud-us.
- **Microsoft Azure Government** + Azure for Healthcare APIs +
  Azure for Financial Services regulatory roll-out documentation.
- **Salesforce Health Cloud, Financial Services Cloud, Government
  Cloud** product-page documentation.
- **Palantir Apollo + Palantir Foundry deployment model** (palantir.com/
  platforms/foundry + palantir.com/platforms/apollo).
- **Cloudflare for Government** authorization process documentation
  (cloudflare.com/government).

### Regulatory sources — payments

- **PCI Security Standards Council** — Payment Card Industry Data
  Security Standard v4.0 (2024-03-31 transition deadline; v4.0.1
  errata 2024-06). pcisecuritystandards.org.
- **PCI DSS Level 1 — assessment process + Reports on Compliance
  (RoC).** Annual QSA-led assessment.
- **NIST SP 800-66 Rev. 2 (HIPAA Security Rule implementation
  guide).** Per-control evidence requirements.
- **FinCEN Money Services Business (MSB) registration process.**
  fincen.gov/money-services-business-msb-registration.
- **NMLS (Nationwide Multistate Licensing System).** State-by-state
  money transmitter license requirements 2024 edition.
- **CSBS (Conference of State Bank Supervisors) Multistate MSB
  Licensing Agreement.** csbs.org/msb.
- **EU PSD2 (Directive 2015/2366) + EBA SCA RTS** + EU PSD3 proposal
  (under negotiation 2024-2026).
- **EBA Guidelines on Outsourcing Arrangements** (EBA/GL/2019/02).
- **KR 전자금융거래법 (Electronic Financial Transactions Act).**
  KR FSS supervision framework.
- **JP Funds Settlement Act** (資金決済法; amended 2024).
- **SG MAS Payment Services Act 2019.** mas.gov.sg.
- **UK PSR 2017 + FCA PERG 15 (Perimeter Guidance manual).**

### Regulatory sources — healthcare

- **HIPAA Privacy Rule (45 CFR §164.502-§164.534).**
- **HIPAA Security Rule (45 CFR §164.302-§164.318).**
- **HIPAA Breach Notification Rule (45 CFR §164.400-§164.414).**
- **HHS Office for Civil Rights HIPAA enforcement.**
- **HITRUST CSF v11.x + r2 Validated Assessment.** hitrustalliance.net.
- **GDPR Article 9 (Special categories of personal data).**
- **EU EHDS (European Health Data Space) Regulation** (in force
  2025-03-12; phased application 2026-2029).
- **KR 의료법 (Medical Service Act).**
- **JP APPI Article 17.**

### Regulatory sources — government + defense

- **FedRAMP PMO authorization process** (fedramp.gov).
- **NIST SP 800-53 Rev. 5 (Security and Privacy Controls).**
- **CMMC Program (Cybersecurity Maturity Model Certification) 2.0**
  (cyber.mil/cmmc).
- **DFARS 252.204-7012 (Safeguarding Covered Defense Information).**
- **ITAR (22 CFR §120-§130).** US State Department DDTC.
- **EAR (15 CFR §730-§774).** US BIS.
- **NIST SP 800-171 (Protecting CUI in Nonfederal Systems).**
- **NIST SP 800-172 (Enhanced Security Requirements).**
- **DoD IL5 + IL6 Authorization.** Defense Information Systems
  Agency (DISA) Cloud Computing Security Requirements Guide v1
  Release 4.
- **StateRAMP** (stateramp.org).
- **KR CSAP (클라우드 보안 인증제) v3.1.**
- **EU NIS2 Directive 2022/2555.**
- **GAIA-X Trust Framework v2024.**
- **UK MOD List X.**
- **NATO STANAG 4774.**

### Regulatory sources — education + pharma

- **FERPA (20 USC §1232g).** ed.gov.
- **COPPA (15 USC §6501-§6506).** ftc.gov/coppa.
- **CA SOPIPA (CA Bus & Prof Code §22584).**
- **CO HB 21-1132 (Student Privacy).**
- **FDA 21 CFR Part 11 (Electronic Records; Electronic
  Signatures).**
- **FDA Software as Medical Device (SaMD) guidance.**
- **EU EMA GAMP 5 Guide.**
- **EU CTR (Clinical Trials Regulation 536/2014).**

### Regulatory sources — marketplace + financial-services

- **State marketplace facilitator tax laws** (every US state has
  enacted such a law since 2018-2021; e.g., CA AB 147 effective
  2019-04-01; NY S6615 effective 2019-06-01). NCSL multistate
  tracker.
- **EU VAT MOSS / OSS** (Regulation (EU) 2021/1147 + Member State
  implementations).
- **EU Digital Services Act 2022/2065** (in force 2023-11-16; for
  marketplaces from 2024-02-17).
- **EU GPSR (General Product Safety Regulation) 2023/988** (applies
  from 2024-12-13).
- **KR 전자상거래법.**
- **CN CCC (China Compulsory Certification) Implementation Rules.**
- **EU Platform Workers Directive 2024/2831** (transposition by
  2026-12-02).
- **EU MiCA (Regulation 2023/1114).**
- **EU DORA (Regulation 2022/2554).** Applies from 2025-01-17.
- **NY DFS 23 NYCRR Part 500** (Cybersecurity).
- **NAIC Insurance Data Security Model Law (#668-1).**
- **KR-FSS 금융회사 핵심업무 위탁 관련 가이드라인.**
- **FCA SYSC 8 + SS2/21 Outsourcing.**

### Continuity + audit + risk standards

- **ISO 22301:2019 — Security and resilience — Business continuity
  management systems.**
- **ISO 27001:2022 + ISO 27017 + ISO 27018.**
- **SOC 2 Type II Trust Service Criteria** (AICPA TSC 2017 + 2022
  update).
- **SOC 1 Type II** (for SOX-touching workloads).
- **FFIEC IT Examination Handbook.**

### Internal portfolio ADRs

- **ADR-0009 — Cell architecture per-tenant per-region.** Cells
  carry certifications.
- **ADR-0010 — Regional pack architecture.** Per-pack compliance
  overlays.
- **ADR-0049 — Cross-region replication + residency.** Built-but-
  unlaunched data residency unchanged.
- **ADR-0064 — Canonical-base + localization.** Capability built
  once; per-market overlays via packs.
- **ADR-0099 — Data class registry.** Capability data classes
  declared at build time.
- **ADR-0105 — Thirteen-layer canonical enum.** Layering unchanged.
- **ADR-0128 — Hyperscaler architecture invariants.** Build-ahead
  is an invariant.
- **ADR-0144 — EU AI Act graduated-risk tier model.** Tier
  evaluation is per-action; built capabilities carry tier metadata.
- **ADR-0150 — Cedar policy engine.** Build-ahead capabilities
  gated by Cedar fragments.
- **ADR-0183 — Cedar app authz + Kyverno admission.** Both gate
  capability invocation.
- **ADR-0211 — In-house Rust-primary tech stack.** Capability
  substrate is Rust.
- **ADR-0212 — Buildability doctrine.** Every capability is
  buildable to §D-2 quality bar.
- **ADR-0218 — Tenant granular control surface.** Tenant admin
  console reflects eligibility state.
- **ADR-0240 — Sovereign cloud per regional pack.** Per-pack
  overlay binds capability data classes.
- **ADR-0241 — DR + business-continuity portfolio policy.**
  Capabilities carry DR tier; built-but-unlaunched capabilities
  drill.
- **ADR-0242 — `oyatie`-is-a-tenant doctrine.** Sandbox + preview
  tenants exercise built-but-unlaunched capabilities.
- **ADR-0243 — Cedar as universal gate.** Capability launch gates
  are Cedar fragments.
- **ADR-0244 — Tenant as universal scoping primitive.**
  Eligibility derived from tenant + cell + capability state.
- **ADR-0245 — Substrate vs Product layering.** Capabilities live
  in their layer.
- **ADR-0246 — Policy-engine substrate promotion.** Per-capability
  Cedar fragments live in policy-engine.
- **ADR-0247 — Self-hosting + self-modification doctrine.** Foundry
  workflows build capabilities; capabilities never bypass.
- **ADR-0248 — Amazon-shape cellular architecture.** Cell-level
  certification scope.
- **ADR-0249 — Data residency enforcement mechanics.** Per-market
  data residency built-in.
- **ADR-0251 — Compliance pack + cell certification levels.**
  Compliance packs are Cedar fragment bundles per market.
- **ADR-0252 — Marketplace physical/services/c2c substrate
  split.** Three marketplace capabilities.
- **ADR-0253 — Tenant identity-verification tiers.** Per-market
  identity-verification thresholds.
- **ADR-0254 — Financial-services substrate architecture.**
  Financial-services-data is a separate capability.
- **ADR-0255 — Intelligence substrate rewrite.** Intelligence
  substrate is built-ahead.

### Auto-memory feedback

- `feedback_build_ahead_of_certification_doctrine` — NEW; captures
  this keystone for future agent context.
- `feedback_oyatie_is_a_tenant_doctrine` — applies; sandbox tenants
  exercise built-but-unlaunched capabilities.
- `feedback_quality_performance_scalability_bar` — reinforced;
  build-state quality bar is hyperscaler-grade.
- `feedback_autonomous_implementation_artifacts` — reinforced;
  build is not gated on external certification timelines.
- `feedback_canonical_base_localization` — reinforced; canonical
  build + per-market overlay.
- `feedback_no_silent_regression` — reinforced; three-state
  transitions are observable + auditable.
- `feedback_automate_everything` — reinforced; capability state
  is machine-evaluable.

---

## Appendix A: Hyperscaler-pattern attribution matrix

Per the audit pattern established in the keystone bundle (ADR-0242
Appendix A et al.), every architectural decision in this ADR is
attributed to a named hyperscaler pattern + source + anti-pattern
avoided.

| Decision section | Hyperscaler pattern (named) | Source citation | Anti-pattern avoided |
|---|---|---|---|
| D-1 (three-state lifecycle) | "Architected-Built-Launched Tri-State" | AWS service-availability progression (preview → public → cert-authorized); Apple Pay country progression; Microsoft Azure Vertical Industry Solutions launch shape | "Build-on-cert-grant" — single-state model with binary launched/not-launched and no built-but-unlaunched intermediate |
| D-2 (build quality bar) | "Operationally-Ready-Before-Launch" | AWS Builder's Library "Ten things we wish we'd known sooner"; Apple Pay "we launched only when we were ready" Vogels-equivalent quote; Stripe Engineering blog 2014-2020 | "Demo-quality launch" — launching with insufficient ops-readiness |
| D-3 (per-market launch gates) | "Per-Market Launch Gate Matrix" | Apple Pay per-country launch playbook; Stripe per-country onboarding sequence; AWS per-region service launch sequencing | "Global simultaneous launch" — incompatible with per-market certification |
| D-4 (capability + certification matrix) | "Certification Catalog as Canonical Source" | AWS Service Authorisation Reference; Microsoft Trust Center compliance offerings catalog; Salesforce Trust + Compliance | "Ad-hoc per-market certification lookup" — discovery via emails + tribal knowledge |
| D-5 (three-state lifecycle in tenant model) | "Eligibility-as-Derived-State" | AWS Organization OU eligibility; GCP IAM eligibility from inherited permissions; Apple App Store entitlement-as-derived | "Imperative eligibility checks" — per-µservice if/else against tenant flags |
| D-6 (Cedar gate composition) | "Layered Policy Composition" | AWS Verified Permissions cell-policy composition; AWS Cedar policy union semantics; ADR-0243 D-4 layered overlay | "Single monolithic policy" — non-composable per-tenant or per-market policy |
| D-7 (launch runbook template) | "Pre-Launch Runbook Discipline" | AWS Operational Readiness Review (ORR) checklist; Google SRE Workbook ch. 18 Production Readiness Review (PRR); Microsoft Operations Manual | "Tribal launch knowledge" — launch executes from informal team memory |
| D-8 (roadmap timeline) | "Multi-Year Capability Roadmap" | Apple Pay 2014-2024 country launch progression; Stripe 2011-2025 product + country progression; AWS 2006-2024 service + region progression | "Quarterly product roadmap" — incompatible with 18-36-month certification timelines |
| D-9 (anti-bypass) | "No-Bypass Defense in Depth" | AWS SCP + IAM + permission boundary triple-check; NIST SP 800-207 Zero Trust no-bypass; ADR-0242 D-3 no-internal-bypass | "Admin override loophole" — bypass paths acquired during incidents, retained afterward |
| D-10 (pre-launch testing) | "Sandbox-Tenant Pilot" | Vercel preview deploys; Stripe test mode; Heroku review apps; ADR-0242 D-8 ephemeral tenants | "Production-only test" — live tenant exposure to built-but-unlaunched capability |
| D-11 (evidence retention) | "Audit-Chain-Anchored Compliance Evidence" | AWS Audit Manager evidence packs; Sigstore Rekor immutability; Sedona Conference legal-hold supersession | "Evidence siloed per certification" — non-uniform retention; lost evidence under audit |
| D-12 (rollback model) | "Graceful Capability Sunset" | Salesforce End of Life roadmap for legacy products; AWS deprecation policy (12-month notice); Stripe API version sunset cadence | "Hard kill on cert lapse" — abrupt customer service interruption |

---

## Appendix B: Worked example — Year 1.5 US payments launch in California

To illustrate the build-ahead-of-certification doctrine concretely,
here is the launch sequence for the first market launch of the
payments capability: California, USA, Year 1.5 (target 2027-Q3).

### Capability identification

- Capability: `payments-card-cnp` (card-not-present payment
  processing for online merchants).
- Market: US-CA (California).
- Target launch date: 2027-07-15 (Year 1.5 + 1 month buffer).

### Build-state arrival (Year 1)

By 2027-01 (Year 1.0), the capability has reached BUILT state:

- Full implementation of card-not-present processing flows: tokenization,
  card-vault, network routing (Visa, Mastercard, Amex, Discover, Diners,
  UnionPay), authorization, capture, refund, void, dispute, chargeback,
  3DS2 SCA, network-token-service integration, network tokenization,
  card-on-file scenarios, recurring billing, soft-descriptor management.
- Integration tests cover 200+ flows against test networks (Visa
  TestSpec, Mastercard MIP, Amex AET, Discover Discovery).
- CI green across all standard lanes.
- Threat model approved (council-security signed).
- Runbooks authored:
  - `docs/runbooks/payments-incident-response.md`.
  - `docs/runbooks/payments-capacity-management.md`.
  - `docs/runbooks/payments-dr-failover.md`.
  - `docs/runbooks/payments-rollback.md`.
- Capacity model verified for projected peak load (100k QPS authorization,
  10k QPS capture, 1k QPS refund) with 3× headroom.
- DR drills passed: T1 (< 5 min RTO; 0 RPO) cross-region failover
  exercised quarterly throughout Year 1.
- Security review APPROVE for production readiness.
- SLO instrumented: 99.99% authorization-success availability, p99
  < 200ms, p999 < 800ms. OpenSLO at
  `microservices/payments/slos/card-cnp.openslo.yaml`.
- Observability dashboard at
  `microservices/observability/dashboards/payments-card-cnp.md` with
  per-network + per-currency + per-cell + per-data-class views.
- Cedar fragments published:
  - `baseline/payments-card-cnp-permits.cedar:v3` (signed by
    org-baseline-key).
  - `baseline/payments-card-cnp-forbid-default.cedar:v3`.
- Audit-chain emissions verified: `PaymentsAuthorize`,
  `PaymentsCapture`, `PaymentsRefund`, `PaymentsVoid`,
  `PaymentsDispute`, `PaymentsChargeback`, `PaymentsTokenize` —
  every action emits.

At this point, `oyatie.dev.*` and `oyatie.preview.*` tenants have
been exercising the capability for 6 months; bug surface is well-
known + tracked; dependencies are current.

The audit-chain event `CapabilityBuiltPromotion` emits on 2027-01-31
with the gate-evidence manifest signed by council-architecture.

### Certification chain (Year 1 - Year 1.5)

Through 2026-Q4 + 2027-Q1, the certification chain has been
in progress in parallel:

- **PCI DSS Level 1 audit.** Coalfire (QSA) engagement Q3 2026.
  Audit window 2026-10 through 2027-02. Report on Compliance issued
  2027-03-15. Certificate valid 2027-03-15 to 2028-03-14.
- **SOC 2 Type II audit.** Deloitte (auditor) engagement Q2 2026.
  Audit window covering 2026-04-01 to 2027-03-31 (12 months observation
  per AICPA TSC). Report issued 2027-05-15. Valid through 2028-05-14.
- **California Money Transmitter License (CA DFPI).** Pre-application
  consultation Q2 2026. Application filed 2026-08-15. Background
  checks + financial review 2026-09 through 2027-03. License issued
  2027-04-15. Initial validity 2027-04-15 to 2029-04-14 (biennial).
  $250k surety bond posted via Surety One; Trustee account configured.
- **FinCEN MSB registration.** Filed 2026-08-15. Registration confirmed
  2026-09-01. Biennial renewal 2028-08-15.
- **OFAC sanctions-screening compliance.** Continuous; Surrogate Daily
  Briefing list ingested via the OFAC TXT feed; ChainAnalysis screening
  for any crypto-adjacent activity; per-transaction screening in the
  authorization path; weekly false-positive review.
- **CFPB Regulation E + Z compliance.** Continuous; legal-counsel-
  reviewed disclosures; per-transaction email-receipt with mandated
  disclosure; complaint-handling workflow integrated.
- **California UDAAP compliance.** Continuous; CA Civil Code §1798
  compliance for any incidental data collection; CCPA + CPRA
  compliance for any incidental data subject rights.
- **1099-K reporting infrastructure.** Built into the payments
  substrate; annual generation for Form 1099-K filings as
  applicable to platform-facilitator status.

By 2027-05-31, every cert in the chain is in hand. The cell
`cell-us-west-2-a` (target home cell for CA tenants) has its
`certifications[]` updated:

```yaml
cell_id: "cell-us-west-2-a"
certifications:
  - cert_id: "pci-dss-l1"
    issuer: "qsa-coalfire"
    issued_at: "2027-03-15"
    expires_at: "2028-03-14"
    evidence_pack_url: "cosign://...sha256:<coalfire-roc-2027>"
  - cert_id: "soc2-type-ii"
    issuer: "auditor-deloitte"
    issued_at: "2027-05-15"
    expires_at: "2028-05-14"
    evidence_pack_url: "cosign://...sha256:<deloitte-soc2-2027>"
  - cert_id: "us-mtl-ca"
    issuer: "ca-dfpi"
    issued_at: "2027-04-15"
    expires_at: "2029-04-14"
    evidence_pack_url: "cosign://...sha256:<ca-dfpi-mtl-2027>"
  - cert_id: "fincen-msb"
    issuer: "fincen"
    issued_at: "2026-09-01"
    expires_at: "2028-08-15"
    evidence_pack_url: "cosign://...sha256:<fincen-msb-2026>"
```

### Launch gates (Year 1.5)

Through 2027-06, the launch-gate work completes:

- **Regulator approval.** CA DFPI MTL issuance letter signed +
  archived (the cert itself is the approval).
- **Legal review.** `oyatie.legal` sub-scope reviews + signs launch
  authorization. Output at
  `docs/runbooks/launch/payments-card-cnp-us-ca.md` carries the
  signature.
- **Tenant onboarding flow.** Per-market wizard updated for CA;
  KYB flow uses CA SOS (Secretary of State) business-record lookup;
  KYC flow uses Persona + Plaid for identity-verification at
  merchant-tier-3 (per ADR-0253); sanctioned-party screening
  via World-Check + OFAC. Tested with 50 sandbox-tenant onboarding
  cases (positive + negative + adversarial).
- **Market-specific Cedar pack.** Compliance pack
  `payments-us-ca-2027` published, version 1.0. Includes:
  - `pack/payments-us-ca-2027/cedar-fragments/ca-udaap.cedar`.
  - `pack/payments-us-ca-2027/cedar-fragments/ca-ccpa.cedar`.
  - `pack/payments-us-ca-2027/cedar-fragments/ca-1099-k.cedar`.
  - `pack/payments-us-ca-2027/cedar-fragments/ca-mtl-bond-monitoring.cedar`.
  - `pack/payments-us-ca-2027/configuration/regulatory-evidence-cadence.yaml`.
- **Operational readiness.** On-call rotation staffed: payments-
  oncall-us-west team (4 engineers) for Pacific Time; payments-
  oncall-us-east (4 engineers) for Eastern Time coverage during
  US business hours. Dashboard localized to USD; support docs
  in English; help center articles published.
- **Marketing approval.** "Stripe-class payments now available for
  California merchants" press materials reviewed by `oyatie.legal`;
  CCPA + Cal. Civil Code §1798 disclosures cleared; ADA + Cal.
  AB-434 accessibility checked.
- **Insurance + bonding.** $250k CA MTL surety bond active via
  Surety One; CGL + cyber-insurance policies extended to cover
  CA-resident payments operations.
- **Tax registration.** CA franchise tax registration; marketplace
  facilitator status established with CDTFA (California Department
  of Tax and Fee Administration).
- **Audit-chain regulator-readiness.** Quarterly evidence packet
  (per ADR-0241 D-8) format approved by CA DFPI compliance staff;
  first packet to be emitted 2027-09-30.

The runbook at `docs/runbooks/launch/payments-card-cnp-us-ca.md`
is fully populated, multispectrum-reviewed, and signed.

### Pilot rollout (2027-07-15 through 2027-08-15)

Week 1-2 (2027-07-15 to 2027-07-29):

- 10 pilot CA-resident merchant tenants enabled; each tenant
  individually onboarded with hands-on support from payments-
  customer-success.
- Per-tenant `eligible_capabilities[].payments-card-cnp.state` set
  to `LAUNCHED-IN-MARKET` with `market: "US-CA"`.
- Cedar evaluator on `cell-us-west-2-a` evaluates: capability state
  BUILT (yes), market state LAUNCHED-IN-MARKET for US-CA (yes), cell
  certifications include `pci-dss-l1` + `soc2-type-ii` + `us-mtl-ca`
  + `fincen-msb` (yes), tenant compliance packs include `pci-dss-l1-2026`
  + `soc2-t2-2026` + `payments-us-ca-2027` (yes), tenant identity
  verification at merchant-tier-3 (yes), tenant KYB status VERIFIED
  for US-CA (yes). Decision: Permit.
- Daily metrics review with payments-product team; bug-triage in real
  time; intensive monitoring.

Week 3-4 (2027-07-30 to 2027-08-12):

- Expand to 100 pilot merchants.
- Continuous monitoring; SEV-3 alerts route to payments-oncall;
  any SEV-2+ triggers rollback consideration.

Month 2+ (2027-08-13 onwards):

- Full CA availability; new CA merchant signups self-serve via
  tenant onboarding wizard.
- Quarterly regulator evidence packet emits to CA DFPI on 2027-09-30.

### Post-launch evolution

- 2027-10: NY MTL approval expected; Phase 1 expansion adds NY.
- 2027-12: TX + FL approvals expected; further expansion.
- 2028-Q1: Phase 1 complete (10 US states); Phase 2 begins (remaining
  40 states); PSD2 EU work concurrent.
- 2028-Q3: Phase 2 complete; UK + JP + SG begin.
- 2029: Multi-market payments fully launched (US 50 + EU EEA + UK +
  JP + SG + KR pending).

### What this worked example demonstrates

- **Build precedes certification by months, not years.** The capability
  was BUILT 2027-01; first market launch 2027-07. Six-month gap is
  used productively: certification, regulator engagement, pilot
  rollout, operational readiness.
- **Per-market work is overlay, not re-implementation.** CA-specific
  work is the compliance pack `payments-us-ca-2027` + the runbook +
  the onboarding-wizard variant — NOT a re-implementation of the
  payments substrate.
- **Three-state lifecycle is mechanically observable.** Every state
  transition emits an audit-chain event; dashboards render the
  state matrix; CI lanes verify coherence.
- **Cedar gate composition enforces every invariant.** The capability
  + market + cell-cert + tenant-pack + tenant-identity + tenant-KYB
  composite is evaluated at every authorization; defense-in-depth.
- **Time-to-market for additional markets is months, not years.**
  After CA launch, NY adds in 3 months because the substrate is
  shared + only the NY-specific work (MTL, NY DFS) is incremental.

---

## Naming justification

Every name introduced or ratified by this ADR is validated against BNF v4.1
(`oya-<microservice>[-<bc-tokens>]-<layer>`) and the ADR-0105 13-value canonical
layer enum.

| Name | Layer (ADR-0105) | BNF v4.1 segments | Justification |
|------|-----------------|-------------------|---------------|
| `oya-check-capability-three-state-coherence` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `capability-three-state-coherence` | Fitness-check; verifies every capability declares exactly one of BUILT_ONLY / CERTIFIED_DARK / LAUNCHED; `oya-check-*` flat namespace |
| `oya-check-built-but-unlaunched-cedar-gate` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `built-but-unlaunched-cedar-gate` | Fitness-check; verifies Cedar gate present for every BUILT_ONLY capability; `oya-check-*` flat namespace |
| `oya-check-capability-launch-runbook-completeness` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `capability-launch-runbook-completeness` | Fitness-check; verifies launch runbook exists and is populated before LAUNCHED transition; `oya-check-*` flat namespace |
| `oya-check-certification-evidence-retention` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `certification-evidence-retention` | Fitness-check; verifies cosign-signed certification evidence retained per §D-11; `oya-check-*` flat namespace |
| `oya-check-anti-bypass-built-only-tenant` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `anti-bypass-built-only-tenant` | Fitness-check; verifies no tenant can activate a BUILT_ONLY capability without certification approval; `oya-check-*` flat namespace |
| `oya-check-capability-rollback-readiness` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `capability-rollback-readiness` | Fitness-check; verifies every LAUNCHED capability has DEGRADED→DARK rollback path per §D-12; `oya-check-*` flat namespace |
| `BUILT_ONLY` | n/a (enum variant) | n/a | Capability state: built and tested, not certifiable yet; Cedar gate blocks public activation |
| `CERTIFIED_DARK` | n/a (enum variant) | n/a | Capability state: certified, dark-launched; pending per-market launch gate approval |
| `LAUNCHED` | n/a (enum variant) | n/a | Capability state: certified and live in one or more markets |

---

*End of ADR-0250.*
