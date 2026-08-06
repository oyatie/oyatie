---
id: ADR-0312
status: Accepted
date: 2026-05-20
owners:
  - council-architecture
  - council-product
  - council-privacy
  - council-security
  - council-legal
  - ops-compliance
  - ops-sre-reliability
  - ops-security
  - ops-trust-and-safety
  - axis-policy-engine
  - axis-audit-chain
  - axis-tenancy
  - axis-identity
  - axis-judicial-review
supersedes: []
amends: []
superseded_by: []
related:
  - ADR-0028-audit-chain-merkle-sealed.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0131-per-microservice-flat-layout.md
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
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0263-observability-emission-contract.md
  - ADR-0272-cookie-consent-per-purpose.md
  - ADR-0276-backup-portability-gdpr-art-20.md
  - ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md
  - ADR-0294-cedar-fragment-soak.md
  - ADR-0295-bootstrap-ci-spiffe-kill-switch.md
  - ADR-0296-library-first-credential-sidecar.md
  - ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md
  - ADR-0299-account-recovery-resilience.md
  - ADR-0300-whistleblower-press-freedom-anonymity.md
  - ADR-0304-cross-jurisdiction-data-conflict-resolution.md
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md
related_specs:
  - /specs/platform-architecture.json
  - /specs/microservices/identity.json
  - /specs/microservices/policy-engine.json
  - /specs/microservices/audit-chain.json
  - /specs/microservices/governance.json
  - /specs/warrant-intake-schema.json
  - /specs/cedar-fragment-schema.json
  - /specs/compliance-pack-schema.json
  - /specs/transparency-report-schema.json
related_memory:
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_cedar_as_universal_gate
  - feedback_tenant_as_universal_scoping_primitive
  - feedback_substrate_vs_product_layering
  - feedback_naming_justification
  - feedback_quality_performance_scalability_bar
  - feedback_clean_architecture_requirements
  - feedback_no_silent_regression
  - feedback_autonomous_implementation_artifacts
  - feedback_canonical_base_localization
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-foundational-doctrine
keystone_position: court-warrant-scoped-piercing
purpose: >
  Codify the court-warrant scoped-piercing doctrine surfaced by the
  Wave-3-E ecosystem journey catalog (j129). When a court warrant
  pierces the personal-tenant boundary established by ADR-0311, the
  pierce MUST be (a) scope-bounded by judicial review, (b) materialized
  as a time-limited tenant-id-scoped Cedar CrossTenantGrant with
  explicit action and resource sets, (c) cited to the originating
  judicial authority + jurisdiction + statutory anchor, (d) audited
  via Merkle-sealed chain-of-custody per ADR-0028, (e) reviewed by
  the per-pack ombudsman per documentation-rigor.md §3.2.5 row 19,
  (f) reported in the transparency-report surface (warrant-canary)
  when not gag-ordered, (g) resolved via higher-restriction-wins per
  ADR-0304 in cross-jurisdiction conflicts, (h) cross-linked with
  reporter-privilege exceptions per ADR-0300. Bulk warrants are
  refused; over-scope warrants are refused. The bar is: legitimate
  judicial process succeeds with minimum necessary scope; illegitimate
  or over-broad warrants are refused; transparency reports remain
  publishable.
enforcement_status: advisory-until-2026-09-15-blocker-thereafter
enforced_by:
  - cloud-ci/Rust gate packet warrant-scope-enforced
  - cloud-ci/Rust gate packet warrant-judicial-review-recorded
  - cloud-ci/Rust gate packet warrant-jurisdiction-cited
  - cloud-ci/Rust gate packet warrant-cedar-grant-shape
  - cloud-ci/Rust gate packet warrant-audit-chain-merkle-seal
  - cloud-ci/Rust gate packet warrant-ombudsman-attestation
  - cloud-ci/Rust gate packet warrant-canary-transparency-report
  - cloud-ci/Rust gate packet warrant-reporter-privilege-exception
naming_justifications:
  - name: oya-shared-warrant-handler
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.warrant-handler
    justification: >
      Per ADR-0105 13-layer canonical enum row 5 (shared-substrate),
      the crate that exposes the warrant-intake API, the
      judicial-review attestor, the scope-bound CrossTenantGrant
      issuer, the Merkle-sealed chain-of-custody emitter, the
      ombudsman attestation requester, the transparency-report
      surface, the per-jurisdiction warrant-scope resolver, and
      the bulk/over-scope refusal classifier belongs at the
      shared layer. Naming `oya-shared-warrant-handler` keeps
      the single-concern flat layout per ADR-0131 and avoids
      any "suite" packaging per ADR-0132. Drop-in companion to
      `oya-shared-dual-tenant-boundary` (ADR-0311),
      `oya-shared-whistleblower-channel` (ADR-0300),
      `oya-shared-anonymity-substrate` (ADR-0300),
      `oya-shared-abuse-defence` (ADR-0297).
  - name: microservices/governance/warrant-intake/
    layer: layer_8_microservice (subdirectory extension of governance µservice)
    bnf_segments: microservices.governance.warrant-intake
    justification: >
      Per ADR-0131 per-microservice flat layout, the warrant-intake
      surface is a sub-feature of the governance µservice rather
      than a standalone µservice (governance owns the regulator-
      facing surface set per the substrate-vs-product split in
      ADR-0245). Naming `microservices/governance/warrant-intake/`
      follows the canonical subdirectory pattern used by other
      governance sub-features (e.g., `microservices/governance/
      catalog-records/`). Single-concern; not a "suite" per
      ADR-0132.
  - name: oya-governance-warrant-scope-enforced
    layer: N/A (foundry-fitness aggregate CI lane)
    bnf_segments: oya.foundry-fitness.warrant-scope-enforced
    justification: >
      Aggregate CI fitness lane per ADR-0212 buildability doctrine;
      rolls up the child lanes verifying every warrant intake
      produces a scope-bounded Cedar grant, a judicial-review
      attestation, a Merkle-sealed audit-chain entry, an
      ombudsman attestation, and a transparency-report record
      (when not gag-ordered).
  - name: oya-governance-warrant-judicial-review
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.warrant-judicial-review
    justification: >
      Per-µservice child lane verifying every warrant carries an
      ombudsman attestation per documentation-rigor.md §3.2.5
      row 19 and a judicial-authority cite (court name, judge
      name, docket number, jurisdiction). Lane is BLOCKER for
      any warrant marked SEV2+.
  - name: oya-governance-warrant-canary-transparency
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.warrant-canary-transparency
    justification: >
      Per-µservice child lane verifying every non-gag-ordered
      warrant emits a transparency-report record per the
      industry-standard cadence (Google + Apple + Microsoft +
      Twitter/X publish quarterly; DSA Art. 24 requires annual).
  - name: oya-governance-warrant-bulk-refusal
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.warrant-bulk-refusal
    justification: >
      Per-µservice child lane verifying the warrant-handler
      refuses bulk warrants (more than N principals or more than
      M resources without specific identifier-by-identifier
      enumeration). Threshold N=10 + M=100 per §D-9.
  - name: WarrantReceived
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Warrant.Received
    justification: >
      Audit-event-class emitted on receipt of any court warrant
      at the warrant-intake surface; registered in ADR-0263
      central registry. Sealed under the per-pack ombudsman's
      audit stream + the affected tenant's audit stream
      (when not gag-ordered).
  - name: WarrantGrantIssued
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Warrant.GrantIssued
    justification: >
      Audit-event-class emitted when the warrant-handler issues
      a scope-bounded Cedar CrossTenantGrant (grant_kind =
      "court_warrant_scoped") per the warrant's authorized
      scope; registered in ADR-0263.
  - name: WarrantRefused
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Warrant.Refused
    justification: >
      Audit-event-class emitted when the warrant-handler refuses
      a warrant (bulk, over-scope, lacking judicial-authority
      cite, lacking ombudsman attestation, lacking statutory
      anchor); registered in ADR-0263.
  - name: WarrantTransparencyDisclosed
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Warrant.TransparencyDisclosed
    justification: >
      Audit-event-class emitted when a warrant is disclosed in
      the transparency report (warrant-canary surface) per the
      regulator cadence; registered in ADR-0263.
  - name: WarrantGagOrderExpired
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: Warrant.GagOrderExpired
    justification: >
      Audit-event-class emitted when a gag-order's expiry passes
      and the previously-suppressed warrant becomes eligible for
      transparency-report disclosure.
  - name: policy/court-warrant-scoped-piercing.cedar
    layer: N/A (canonical Cedar fragment filename)
    bnf_segments: policy.court-warrant-scoped-piercing
    justification: >
      Canonical filename for the per-µservice court-warrant
      Cedar fragment under the µservice's `policy/` directory
      per ADR-0246 + ADR-0243 fragment-lifecycle conventions;
      single-concern naming keeps the policy directory's
      contract-by-name invariant.
  - name: X-Oya-Warrant-Id
    layer: N/A (HTTP request header naming)
    bnf_segments: X-Oya.Warrant-Id
    justification: >
      Custom HTTP request header carrying the warrant identifier
      for any request that operates under a warrant-scoped Cedar
      grant; consumed by the audit-chain µservice for the per-
      request linkage between the request and the originating
      warrant. Namespace prefix `X-Oya-` reserves the platform's
      header surface.
  - name: court_warrant_scoped
    layer: N/A (Tenancy::CrossTenantGrant.grant_kind enum value per ADR-0244)
    bnf_segments: grant_kind.court_warrant_scoped
    justification: >
      Tenancy::CrossTenantGrant.grant_kind enum extension per
      ADR-0244 §D-4 + ADR-0311 §D-8. Identifies a cross-tenant
      grant issued by the warrant-handler under judicial review;
      MUST be the only mechanism by which a work-tenant principal
      reaches a personal-tenant resource without self-access.
---

# ADR-0312: Court-Warrant Scoped Piercing

## Status

Proposed — 2026-05-20.

Bundled with the keystone-bundle 2026-05-20 foundational doctrine
synthesis as the **court-warrant-scoped-piercing** ADR, the companion
to ADR-0311 (dual-tenant identity boundary). Surfaced by the Wave-3-E
ecosystem journey catalog j129 (`court-warrant-pierces-personal-tenant-with-judicial-oversight`)
and cross-linked with j130 (bribery attempt via personal Messenger
audit-only via ombudsman) + j131 (cross-jurisdiction audit
discrepancy).

Enforcement is `advisory-until-2026-09-15-blocker-thereafter`. The
doctrine accepts in text immediately; the CI lanes that enforce it
promote to BLOCKER on 2026-09-15 to align with ADR-0311's rollout
window. Until 2026-09-15, validators emit findings without failing
CI; post-2026-09-15, the lanes block merge.

## Date

2026-05-20.

## §A. Context

### §A.1. The warrant problem

ADR-0311 establishes a hard default-deny boundary at the
personal-tenant edge: no employer-tenant Cedar permit can read an
employee's personal-tenant surfaces. The boundary is load-bearing —
without it, the consumer-trust premise of the platform collapses.

But the boundary MUST yield to legitimate judicial process. A court
that has jurisdiction over a person + has probable cause + issues a
warrant with proper judicial review CAN compel disclosure of
specific personal-tenant artifacts in service of the rule of law.

The risk profile is binary:

- **Failure to honor a legitimate warrant** would constitute
  contempt of court, expose the platform's officers to criminal
  liability, and undermine the rule-of-law cooperation that EVERY
  hyperscaler provides under proper legal process.
- **Honoring an illegitimate / over-broad warrant** would shatter
  consumer trust, expose every personal-tenant surface to government
  fishing expeditions, and violate the constitutional protections
  that warrants are supposed to scope (US 4th Amendment, EU Charter
  Art. 7-8, KR Constitution Art. 16-18).

The platform's response must thread the needle: legitimate warrants
succeed at minimum necessary scope; illegitimate or over-broad
warrants are refused; the user is notified when not gag-ordered;
the transparency surface remains publishable.

### §A.2. Precedent — how hyperscalers handle warrants

Every named hyperscaler operates a published warrant-handling
process. The pattern is:

- **Microsoft v. United States (2018) "Microsoft Ireland".** US
  government sought email content stored in Microsoft's Dublin
  datacenter via a SCA §2703 warrant. Microsoft challenged on the
  ground that SCA warrants did not have extraterritorial reach. The
  Supreme Court initially granted certiorari but mooted the case
  after Congress passed the CLOUD Act in March 2018. The CLOUD Act
  established the framework where US warrants reach data held by
  US-incorporated providers regardless of physical location, AND
  permits providers to challenge warrants conflicting with foreign
  law via comity. Per Microsoft's Brad Smith's 2018 statement,
  Microsoft committed to publishing transparency reports + notifying
  users when not gag-ordered + challenging over-broad warrants. The
  oyatie warrant-handler adopts these three commitments verbatim.
- **In re Search Warrant Issued to Twitter, Inc. (2014)** (Manhattan
  Criminal Court People v. Harris). Twitter sought to quash a
  subpoena for a protester's tweets, arguing First Amendment
  protection. The court held tweets were not protected; Twitter
  complied but published the case as part of its transparency
  report. The case established the precedent that providers MAY
  challenge warrants and MUST publish what they cannot challenge.
- **US CLOUD Act (2018)** — Clarifying Lawful Overseas Use of Data
  Act. 18 USC §2713 (provider must produce data "regardless of
  whether such communication, record, or other information is located
  within or outside of the United States"). 18 USC §2523 (executive
  agreements to lower-friction lawful access to foreign data).
  Comity-based provider challenges permitted under 18 USC §2703(h).
- **EU GDPR Art. 48** — Transfers or disclosures not authorised by
  Union law. "Any judgment of a court or tribunal and any decision
  of an administrative authority of a third country requiring a
  controller or processor to transfer or disclose personal data
  may only be recognised or enforceable in any manner if based on
  an international agreement, such as a mutual legal assistance
  treaty, in force between the requesting third country and the
  Union or a Member State, without prejudice to other grounds for
  transfer pursuant to this Chapter." This frames the cross-
  jurisdiction-conflict resolution path (per ADR-0304).
- **KR Telecommunications Business Act (전기통신사업법) Art. 83(3)
  (구 통신비밀보호법 협력)** — provides framework under which Korean
  prosecutors and police may demand provider data; the Constitutional
  Court of Korea decision 2018Hun-Ma1059 (2022) held that bulk
  request without specific facts violates constitutional privacy
  rights. Korean providers now publish transparency reports.
- **EU Digital Services Act Art. 9 + Art. 10 (orders to act against
  illegal content / orders to provide information)** — defines the
  shape of cross-border orders + requires transparency-report
  disclosures per Art. 15 + Art. 24.
- **Apple Transparency Report** (semi-annual since 2013) publishes
  per-country government-request counts including device requests,
  financial-identifier requests, account requests, account
  preservation requests, and emergency requests. The shape is the
  industry standard. Source: apple.com/legal/transparency/.
- **Google Transparency Report** (semi-annual since 2010) publishes
  per-country government-request counts split by request type
  (subpoena, court order, search warrant, emergency disclosure
  request, preservation request, NSL). Source:
  transparencyreport.google.com.
- **Microsoft Digital Defense Report** (annual since 2021) + Law
  Enforcement Requests Report (semi-annual since 2013). Source:
  microsoft.com/en-us/corporate-responsibility/lerr.
- **Twitter/X Transparency Report** (semi-annual since 2012; per
  Elon Musk takeover 2022 the cadence changed but still publishes).
- **Cloudflare Transparency Report** (semi-annual since 2013).
  Cloudflare additionally publishes "warrant canaries" per the
  earliest 2014 Cloudflare blog "An update on the Cloudflare
  transparency report" — a structured statement that no national-
  security-letter has been received in the past period; the canary
  removal signals receipt.

The pattern from these precedents is unambiguous:

1. **Receive warrants via a published surface** (intake API + legal-
   process address).
2. **Apply scope-narrowing review** (the legal team challenges over-
   broad warrants; provider's counsel + counsel for affected user
   when feasible).
3. **Materialize the warrant as a time-limited authorization** (in
   oyatie's substrate: a Cedar CrossTenantGrant with
   `grant_kind = "court_warrant_scoped"`).
4. **Audit every action under the warrant** (Merkle-sealed chain-
   of-custody per ADR-0028).
5. **Notify the affected user when not gag-ordered** (consistent with
   18 USC §2705(b) delayed-notice rules and analogous foreign
   provisions).
6. **Disclose the warrant in the transparency report** per the per-
   pack regulator cadence (DSA Art. 24 annual; industry semi-annual).
7. **Refuse bulk warrants** (the 4th Amendment + KR Const. Ct.
   2018Hun-Ma1059 + EU Charter Art. 7-8 + Snowden-era precedent
   reject bulk fishing-expedition warrants).

ADR-0312 codifies this pattern as a substrate primitive.

### §A.3. Why this is its own ADR

ADR-0311 establishes the boundary. ADR-0312 codifies the ONLY
legitimate pierce path through that boundary. The two ADRs are
mutually-reinforcing companions: without ADR-0312, the boundary in
ADR-0311 becomes either too rigid (no warrant succeeds, contempt of
court) or too leaky (every warrant succeeds, no scope review). The
critical-path matrix in documentation-rigor.md §3.2.5 row 19
explicitly calls for an ombudsman-attestation invariant for any
SEV2+ scoping pierce; this ADR codifies the warrant intake +
review + materialization + audit + transparency cycle that satisfies
row 19.

### §A.4. The j129 worked example

j129 (`court-warrant-pierces-personal-tenant-with-judicial-oversight`)
is the canonical journey. Sequence:

1. **T-0d.** Inspector Diana Reyes's personal Mail (under her
   personal tenant `b2c-<hash>`) contains evidence of a bribery
   attempt directed at her in her capacity as GAO auditor. The
   bribery attempt is itself a federal felony (18 USC §201).
2. **T-3d.** A federal judge (issued in Diana's district) signs a
   search warrant for the specific Mail thread between Diana and
   the bribery offeror. The warrant cites 18 USC §201 + the SCA
   §2703(d). It identifies the specific Mail thread, not Diana's
   broader Mailbox. The warrant has a 30-day execution window.
3. **T-3d+1h.** The warrant is delivered to oyatie's legal process
   address (per the published transparency-report contact list).
4. **T-3d+2h.** The warrant-intake µservice (per §E.2) parses the
   warrant. The intake worker:
   - Validates the warrant's judicial-authority cite (court name +
     judge name + docket).
   - Validates the statutory anchor (18 USC §201 + SCA §2703(d)).
   - Validates the scope is bounded (specific Mail thread, not
     Mailbox-wide).
   - Refuses to process if any validation fails.
5. **T-3d+4h.** The ombudsman is notified for attestation per
   documentation-rigor.md §3.2.5 row 19. The ombudsman reviews:
   - Is the scope minimum necessary?
   - Is the statutory anchor authentic?
   - Is the affected user notification appropriate (gag-ordered
     or not)?
6. **T-3d+8h.** Ombudsman attests. The warrant-handler issues a
   Cedar CrossTenantGrant with `grant_kind = "court_warrant_scoped"`,
   scoped to:
   - `from_tenant`: "tenant-doj-investigations" (the DOJ's tenant
     under the federal-LE pack).
   - `to_tenant`: Diana's personal tenant.
   - `actions_permitted`: `{ "ReadInScope" }` (read only).
   - `resources_permitted`: `{ "mail:thread:<specific-thread-id>" }`.
   - `expires_at`: T-3d + 30 days.
   - `evidence_uri`: the warrant document sealed in audit-chain.
7. **T-3d+9h.** The grant is loaded into the Cedar evaluator per
   ADR-0294 soak.
8. **T-3d+24h.** DOJ investigator reads the specific Mail thread
   under the grant. Every read emits `WarrantGrantUsed` audit event.
9. **T-3d+30d.** Grant expires. No further reads possible.
10. **T-3d+45d.** Diana is notified of the warrant (when not gag-
    ordered, per 18 USC §2705(b) the notice may be delayed up to
    180 days renewable). Diana sees the warrant detail in her
    personal-tenant transparency dashboard.
11. **T-3d+90d.** The warrant is disclosed in oyatie's quarterly
    transparency report (aggregated; specific identity withheld
    until Diana opts in).

The journey exercises every §D mechanic.

### §A.5. Failure modes — what goes wrong without this ADR

1. **Bulk warrant succeeds.** A warrant requesting "all Messenger
   threads of all employees of company X for 2026" is accepted
   without scope review. (Failure mode F-CWP-001.)
2. **Over-scope warrant succeeds.** A warrant requesting "Diana's
   Mailbox" succeeds where the underlying probable cause covers
   only a specific thread. (Failure mode F-CWP-002.)
3. **Judicial-authority absent.** A document styled as a "warrant"
   but lacking judge signature + docket is processed. (Failure mode
   F-CWP-003.)
4. **Cross-jurisdiction conflict ignored.** US warrant for EU-
   resident's data processed without higher-restriction-wins per
   ADR-0304. (Failure mode F-CWP-004.)
5. **No ombudsman review.** Warrant is processed without per-pack
   ombudsman attestation. (Failure mode F-CWP-005.)
6. **Audit-chain seal omitted.** Warrant-derived reads are not
   Merkle-sealed; chain-of-custody breaks. (Failure mode F-CWP-006.)
7. **Transparency-report omitted.** Non-gag-ordered warrant
   processed without transparency disclosure. (Failure mode
   F-CWP-007.)
8. **User notification omitted when due.** Gag-order expires but
   user is never notified. (Failure mode F-CWP-008.)
9. **Grant outlives scope.** Warrant grant remains active past
   warrant expiry. (Failure mode F-CWP-009.)
10. **Reporter-privilege exception ignored.** Warrant for a journal-
    ist's source list processed despite reporter-privilege per
    ADR-0300. (Failure mode F-CWP-010.)

All ten failure modes are addressed by the §D mechanics below.

### §A.6. Capacity context

The expected warrant volume at scale:

- Hyperscaler benchmarks: Google Transparency Report H2 2024 ~163,000
  user-data requests; Apple H2 2024 ~31,000; Microsoft H2 2024
  ~80,000; Twitter/X 2024 (post-takeover) ~50,000. Mid-range
  hyperscaler scale ~50-150k warrants/year.
- oyatie's projected scale at GA: ≤100 warrants/year (cell-fanned out
  to per-pack ombudsman cohort); ~1 per business day. The substrate
  is designed to handle 10× peak without architectural change.

## §B. Decision

The platform adopts the **Court-Warrant Scoped Piercing** doctrine.
The following decisions are locked:

### §B.1. Warrant is the ONLY pierce path

The only legitimate cross-tenant grant that pierces the personal-
tenant boundary (per ADR-0311) is a `grant_kind = "court_warrant_scoped"`
CrossTenantGrant issued by the warrant-handler under §B.2 review.
No other code path may emit a grant crossing the personal-tenant
boundary. CI lane `oya-governance-personal-tenant-cedar-deny`
verifies the invariant.

### §B.2. Every warrant SEV2+ requires ombudsman review

Per documentation-rigor.md §3.2.5 row 19, every warrant flagged
SEV2+ (any cross-tenant pierce, any consumer-user pierce, any
journalist/whistleblower pierce, any minor-user pierce) requires
per-pack ombudsman attestation BEFORE the Cedar grant is issued.

### §B.3. Per-jurisdiction scope is computed at intake

The warrant-handler computes the per-jurisdiction scope envelope at
intake:

- US warrant — 4th Amendment scope (particularity requirement).
- EU warrant — GDPR Art. 48 scope (recognized via MLAT or executive
  agreement only); native EU/EEA member-state warrants per local
  CCPI implementations.
- KR warrant — KR Telecommunications Business Act Art. 83 +
  Constitutional Court 2018Hun-Ma1059 specificity floor.
- JP warrant — Criminal Procedure Code Art. 218 specificity floor.
- CN warrant — only via sovereign-cloud pack overlay (CN-PIPL +
  Cybersecurity Law); data does not leave CN cells.
- Other jurisdictions per per-pack overlay.

### §B.4. Cross-jurisdiction conflict resolves per ADR-0304

When a US warrant targets data resident in EU cells (or vice versa),
the higher-restriction jurisdiction wins per ADR-0304. The warrant-
handler refuses or requires MLAT-mediation as appropriate.

### §B.5. Audit-chain is Merkle-sealed

Every warrant intake, every ombudsman attestation, every Cedar grant
issuance, every read under the grant, every grant expiry, every
transparency disclosure emits an audit event under
`microservices/audit-chain/` Merkle-sealed per ADR-0028.

### §B.6. User notification when not gag-ordered

When the warrant lacks a gag-order, OR the gag-order has expired,
the affected user MUST be notified via:

- A personal-tenant Mail to their primary address.
- A persistent banner in their personal-tenant transparency dashboard.
- A push notification to their personal-device shell (per ADR-0299
  passkey-bound device list).

Notification carries: warrant identifier, issuing court, statutory
anchor, scope summary, actions taken, expiry date, ombudsman contact.

### §B.7. Transparency-report disclosure

Per pack-regulator cadence (US: industry semi-annual; EU DSA Art. 24:
annual; KR: quarterly post-2024 amendments), the platform publishes
a transparency report with:

- Aggregate warrant counts per jurisdiction per category.
- Refusal counts (bulk, over-scope, lacking-anchor).
- Average review SLA.
- Warrant-canary statement (no NSL received in period).

### §B.8. Bulk and over-scope warrants are refused

The warrant-handler refuses warrants that:

- Request more than N=10 distinct principals without per-principal
  enumeration of specific facts.
- Request more than M=100 distinct resources without per-resource
  enumeration of specific facts.
- Lack a statutory anchor.
- Lack a judicial-authority signature.
- Have expired.
- Conflict with higher-restriction jurisdiction (per ADR-0304).
- Target reporter-privilege-protected sources (per ADR-0300 +
  shield-law overlays).

Refusal emits `WarrantRefused` audit event and a non-gag-ordered
response back to the issuing authority citing the refusal ground.

### §B.9. Reporter-privilege exception per ADR-0300

When a warrant targets a journalist's source list (per ADR-0300
whistleblower-press-freedom doctrine), the per-pack shield-law
overlay determines the response: in US-federal contexts the
Department of Justice 28 CFR §50.10 framework limits warrants
against journalists; in shield-law states (NY, CA, etc.) state law
constrains. The warrant-handler refuses or requires DOJ-policy
attestation per the overlay.

### §B.10. Chain-of-custody to the per-pack ombudsman

The per-pack ombudsman receives chain-of-custody for every warrant
+ every grant + every action under the grant. The ombudsman may
revoke a grant at any time (per ADR-0294 fragment-soak emergency
rollback) if the scope was misjudged.

## §C. Consequences

### §C.1. Maintainability

- New crate `oya-shared-warrant-handler` introduces the substrate
  primitive.
- New µservice extension `microservices/governance/warrant-intake/`
  hosts the intake API + state machine. (Flagged for follow-up IP;
  not scaffolded in this ADR.)
- New Cedar fragment `policy/court-warrant-scoped-piercing.cedar`
  added to every personal-tenant-bearing µservice.
- New audit event classes registered in ADR-0263 registry.
- Each new jurisdiction adds an overlay file; per-overlay maintenance
  scales linearly with jurisdiction count.

### §C.2. Observability

- Audit event classes: `WarrantReceived`, `WarrantGrantIssued`,
  `WarrantRefused`, `WarrantGrantUsed`, `WarrantGrantExpired`,
  `WarrantTransparencyDisclosed`, `WarrantGagOrderExpired`,
  `WarrantOmbudsmanAttested`.
- Metrics: `oya_warrant_intake_total{jurisdiction,status}`,
  `oya_warrant_review_latency_seconds`, `oya_warrant_grant_active`,
  `oya_warrant_refusal_total{reason}`,
  `oya_warrant_transparency_disclosed_total{jurisdiction,quarter}`.
- Trace span shape: every warrant-bearing request carries
  `oya.warrant.id`, `oya.warrant.jurisdiction`,
  `oya.warrant.statutory_anchor`, `oya.warrant.expires_at`.
- Dashboards: `dashboards/warrant-handler-grafana.json` rolls up
  per-jurisdiction intake rate, refusal rate, review SLA.

### §C.3. Scalability

- Expected volume ≤100 warrants/year; substrate sized for 10× peak.
- Per-warrant processing P95 ≤4 hours human review + ≤500 µs
  Cedar evaluator load.
- The warrant intake µservice is stateless except for the warrant-
  document blob storage (in OpenBao + audit-chain Merkle log).
- Horizontal scale via cell mesh per ADR-0248.

### §C.4. Performance

- Warrant intake API P95 ≤200 ms (form submission + initial
  validation).
- Ombudsman review P95 ≤4 business hours.
- Cedar grant issuance P95 ≤500 ms post-attestation.
- Per-read overhead under warrant grant P95 ≤200 µs (Cedar
  evaluation cost is identical to ADR-0311 boundary check).

### §C.5. Optimization

- Per-warrant cost: dominated by human ombudsman review (~4 hours
  legal-staff time). Substrate cost is negligible (~1 minute
  compute per warrant lifecycle).
- Caching strategy: warrant Cedar grants cached per ADR-0246
  library-first dispatch; TTL bounded by warrant expiry.

### §C.6. Code quality

- The shared substrate crate `oya-shared-warrant-handler` MUST pass:
  - `cargo test` line coverage ≥85%, branch coverage ≥75%.
  - `cargo clippy -- -D warnings`.
  - `cargo fmt --check`.
  - `cargo deny check`.
  - `proptest` on the scope-bounded-grant invariant (grant.actions
    ⊆ warrant.scope; grant.resources ⊆ warrant.scope;
    grant.expires_at ≤ warrant.execution_window_end).
  - `libFuzzer` on the warrant-document parser.
  - Mutation testing via `cargo-mutants` ≥80% kill rate.
- ABI: `#[non_exhaustive]` on public enums; SemVer 1.x.y per
  ADR-0258.

### §C.7. Cross-µservice impact

- `governance` µservice adds the `warrant-intake/` subdirectory
  extension with intake API + state machine.
- `policy-engine` loads the warrant-cedar-fragment per ADR-0294.
- `audit-chain` registers the new event classes.
- `identity` connects warrant grant evaluation to its session-token
  enforcement path.
- All personal-tenant-bearing µservices load the
  `policy/court-warrant-scoped-piercing.cedar` fragment.

## §D. Detailed mechanics

### §D-1. Warrant intake workflow — Cedar permit grants temporary scope-bounded read for warrant scope only

The warrant intake state machine:

```
[RECEIVED]
    -> validate_judicial_authority -> [VALIDATED] | [REFUSED]
[VALIDATED]
    -> validate_statutory_anchor -> [ANCHORED] | [REFUSED]
[ANCHORED]
    -> validate_scope_bounded -> [SCOPED] | [REFUSED]
[SCOPED]
    -> check_cross_jurisdiction_conflict -> [JURISDICTION_OK] |
       [REFUSED] | [MLAT_REQUIRED]
[JURISDICTION_OK]
    -> check_reporter_privilege -> [PRIVILEGE_OK] | [REFUSED]
[PRIVILEGE_OK]
    -> ombudsman_review -> [ATTESTED] | [REFUSED]
[ATTESTED]
    -> issue_cedar_grant -> [GRANT_ACTIVE]
[GRANT_ACTIVE]
    -> wait_for_expiry_or_revoke -> [EXPIRED] | [REVOKED]
[EXPIRED] | [REVOKED]
    -> notify_user_if_no_gag_or_expired -> [NOTIFIED]
[NOTIFIED]
    -> transparency_report_in_next_period -> [DISCLOSED]
[DISCLOSED]
    -> seal_in_warrant_archive -> [SEALED]
```

Postgres state machine table:

```sql
-- microservices/governance/warrant-intake/migrations/0001_warrant_state.sql
-- Per ADR-0312 §D-1.

CREATE TYPE warrant_state AS ENUM (
    'RECEIVED',
    'VALIDATED',
    'ANCHORED',
    'SCOPED',
    'JURISDICTION_OK',
    'MLAT_REQUIRED',
    'PRIVILEGE_OK',
    'ATTESTED',
    'GRANT_ACTIVE',
    'EXPIRED',
    'REVOKED',
    'NOTIFIED',
    'DISCLOSED',
    'SEALED',
    'REFUSED'
);

CREATE TYPE warrant_refusal_reason AS ENUM (
    'BULK_OVER_PRINCIPAL_LIMIT',
    'BULK_OVER_RESOURCE_LIMIT',
    'LACKING_JUDICIAL_AUTHORITY',
    'LACKING_STATUTORY_ANCHOR',
    'OVER_SCOPE',
    'CROSS_JURISDICTION_HIGHER_RESTRICTION',
    'REPORTER_PRIVILEGE_PROTECTED',
    'OMBUDSMAN_REJECTED',
    'WARRANT_EXPIRED_BEFORE_ISSUE',
    'OTHER'
);

CREATE TABLE warrants (
    warrant_id            TEXT        PRIMARY KEY
                                      CHECK (warrant_id ~ '^warrant-[a-z0-9-]{1,128}$'),

    issuing_court         TEXT        NOT NULL,
    judge_name            TEXT        NOT NULL,
    docket_number         TEXT        NOT NULL,
    jurisdiction_code     TEXT        NOT NULL
                                      CHECK (jurisdiction_code ~ '^[A-Z]{2}(-[A-Z]{1,3})?$'),

    statutory_anchor      TEXT        NOT NULL,
    statutory_anchor_uri  TEXT,

    warrant_document_uri  TEXT        NOT NULL,
    warrant_document_hash TEXT        NOT NULL,

    received_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    received_via          TEXT        NOT NULL
                                      CHECK (received_via IN
                                          ('legal_process_address', 'mlat_channel',
                                           'sovereign_overlay_channel',
                                           'emergency_disclosure_request')),

    execution_window_start TIMESTAMPTZ NOT NULL,
    execution_window_end   TIMESTAMPTZ NOT NULL,

    gag_order_active      BOOLEAN     NOT NULL DEFAULT FALSE,
    gag_order_expires_at  TIMESTAMPTZ,

    target_principals     TEXT[]      NOT NULL,
    target_resources      TEXT[]      NOT NULL,
    actions_authorized    TEXT[]      NOT NULL,

    state                 warrant_state NOT NULL DEFAULT 'RECEIVED',
    refusal_reason        warrant_refusal_reason,
    refusal_detail        TEXT,

    ombudsman_principal   TEXT,
    ombudsman_attested_at TIMESTAMPTZ,
    ombudsman_attestation_signature TEXT,
    ombudsman_attestation_evidence_uri TEXT,

    cedar_grant_id        TEXT,
    cedar_grant_issued_at TIMESTAMPTZ,
    cedar_grant_expires_at TIMESTAMPTZ,
    cedar_grant_revoked_at TIMESTAMPTZ,

    user_notified_at      TIMESTAMPTZ,
    transparency_period   TEXT,
    transparency_disclosed_at TIMESTAMPTZ,

    -- Merkle seal under audit-chain
    audit_chain_seal_uri  TEXT        NOT NULL,

    -- Audit trail
    created_by            TEXT        NOT NULL,
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    revision              BIGINT      NOT NULL DEFAULT 1
);

CREATE INDEX idx_warrants_state ON warrants (state);
CREATE INDEX idx_warrants_jurisdiction ON warrants (jurisdiction_code);
CREATE INDEX idx_warrants_target_principals ON warrants USING GIN (target_principals);
CREATE INDEX idx_warrants_execution_window ON warrants (execution_window_start, execution_window_end);

SELECT create_distributed_table('warrants', 'warrant_id');
```

### §D-2. Judicial-review oversight — every warrant SEV2+ requires ombudsman review per documentation-rigor.md §3.2.5 row 19

Per-pack ombudsman is a designated principal under the
`oyatie.ombudsman.<pack>` sub-scope (per ADR-0244 §D-2). Ombudsman
attestation flow:

1. **Warrant moves to PRIVILEGE_OK state.** The warrant-handler
   posts a review request to the ombudsman's queue
   (`oyatie.ombudsman.<pack>.review-queue`).
2. **Ombudsman reviews via the governance shell.** UI surfaces the
   warrant document, the parsed scope, the cross-jurisdiction
   analysis, the reporter-privilege analysis.
3. **Ombudsman decides.** Three choices:
   - ATTEST — sign the attestation; warrant proceeds to ATTESTED
     state.
   - REJECT — sign the rejection; warrant proceeds to REFUSED state
     with `ombudsman_rejected` reason.
   - DEFER — request additional information from the issuing
     authority; warrant stays in PRIVILEGE_OK pending response.
4. **Attestation signature** is a passkey-bound (per ADR-0299)
   signature over `(warrant_id, warrant_document_hash, scope_summary,
   attested_at)`. The signature is sealed in audit-chain per
   ADR-0028.
5. **SLA.** Ombudsman attestation SLA is 4 business hours P95;
   24 business hours P99. The ombudsman cohort is per-pack
   (US-LE-pack ombudsman, EU-DSA-pack ombudsman, KR-PIPA-pack
   ombudsman, etc.) so jurisdictional review can be specialized.

For SEV2+ warrants (any cross-tenant pierce, any consumer pierce,
any journalist/whistleblower pierce, any minor pierce), TWO
ombudsmen sign — the per-pack ombudsman + the platform-tier
ombudsman. Both signatures sealed under audit-chain.

### §D-3. Per-jurisdiction warrant scope

| Jurisdiction | Scope authority | Refusal threshold | Cross-border? |
|---|---|---|---|
| US federal | 4th Amendment particularity; SCA §2703(d) specific-and-articulable; ECPA §2511(2)(a)(ii) law-enforcement carve-out | bulk N=10 / M=100 | CLOUD Act applies |
| US state | varies — most states track federal | bulk N=10 / M=100 | MLAT for foreign data |
| EU member state | GDPR Art. 48 + Member-state CCPI implementations | bulk N=5 / M=50 (stricter than US) | Art. 48 requires MLAT for non-EU |
| UK | DPA 2018 + UK GDPR Art. 48 equivalent | bulk N=5 / M=50 | Mutual recognition with EU partial |
| KR | Telecom Business Act Art. 83 + Const. Ct. 2018Hun-Ma1059 specificity | bulk N=5 / M=50 | KR sovereign-cloud pack applies |
| JP | Criminal Procedure Code Art. 218 specificity | bulk N=10 / M=100 | JP-APPI overlay |
| AU | Telecommunications (Interception and Access) Act 1979 + Surveillance Devices Act (per state) | bulk N=10 / M=100 | Five Eyes MLAT |
| CA | Charter §8 + Criminal Code 487 | bulk N=10 / M=100 | MLAT |
| SG | Criminal Procedure Code 2010 §39-40 | bulk N=10 / M=100 | PDPA overlay |
| IN | CrPC §91 + IT Act §69 | bulk N=10 / M=100 | DPDP overlay |
| BR | LGPD + CPP Art. 240-241 | bulk N=10 / M=100 | LGPD Art. 33 |
| CN | Cybersecurity Law + PIPL + Sovereign-overlay (data does NOT leave CN cell) | bulk N=10 / M=100 | NO data export |

Per-jurisdiction overlays in
`specs/warrant-jurisdiction-scope-overlays.json` (new spec file).

### §D-4. Warrant-canary surface — published transparency-report disclosures

The transparency-report surface lives at
`microservices/governance/transparency-report/`. It publishes:

- **Quarterly transparency report** (`/v1/transparency/quarterly`):
  - Aggregate warrant intake by jurisdiction × category.
  - Aggregate refusals by reason.
  - Aggregate grants issued.
  - Average ombudsman-review SLA.
- **Warrant canary** (`/v1/transparency/canary`):
  - Structured statement: "As of <date>, no national-security letter
    has been received in the most recent reporting period." The
    statement is signed by the platform's transparency officer.
    Removal of this statement is the canary signal.
- **Annual DSA Art. 24 report** (`/v1/transparency/annual-dsa`)
  for EU jurisdictions per DSA Art. 24 requirements.

Transparency report schema (new spec
`specs/transparency-report-schema.json`):

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://specs.oyatie.local/transparency-report-schema.json",
  "title": "Transparency report record (per ADR-0312 §D-4)",
  "_meta": {
    "purpose": "Public transparency-report disclosure of warrant activity",
    "binding_adr": "ADR-0312",
    "industry_citations": [
      "Apple Transparency Report (semi-annual since 2013)",
      "Google Transparency Report (semi-annual since 2010)",
      "Microsoft Law Enforcement Requests Report (semi-annual since 2013)",
      "EU DSA Art. 24 (annual)",
      "Cloudflare Transparency Report (semi-annual since 2013)"
    ]
  },
  "type": "object",
  "required": ["report_period", "report_jurisdiction", "warrant_intake_count",
               "warrant_refusal_count", "warrant_grant_issued_count",
               "average_review_sla_hours", "warrant_canary_statement",
               "canary_signed_by", "canary_signature"],
  "properties": {
    "report_period": {
      "type": "string",
      "pattern": "^[0-9]{4}-(Q[1-4]|H[1-2]|annual)$",
      "examples": ["2026-Q1", "2026-H1", "2026-annual"]
    },
    "report_jurisdiction": {
      "type": "string",
      "pattern": "^[A-Z]{2}(-[A-Z]{1,3})?$|^global$"
    },
    "warrant_intake_count": {"type": "integer", "minimum": 0},
    "warrant_refusal_count_by_reason": {
      "type": "object",
      "additionalProperties": {"type": "integer", "minimum": 0}
    },
    "warrant_grant_issued_count": {"type": "integer", "minimum": 0},
    "warrant_grant_categories": {
      "type": "object",
      "description": "Breakdown by category (read/preserve/intercept/emergency)",
      "additionalProperties": {"type": "integer", "minimum": 0}
    },
    "users_affected_count": {"type": "integer", "minimum": 0},
    "average_review_sla_hours": {"type": "number", "minimum": 0},
    "warrant_canary_statement": {
      "type": "string",
      "description": "Structured canary statement; absence signals NSL receipt"
    },
    "canary_signed_by": {
      "type": "string",
      "description": "Principal identifier of the transparency officer"
    },
    "canary_signature": {
      "type": "string",
      "description": "Passkey-bound signature over the canonical canary text"
    },
    "next_report_period": {"type": "string"}
  }
}
```

### §D-5. Cross-jurisdiction conflict (j129 worked example) — warrant from US for EU-resident's data; higher-restriction wins per ADR-0304

Cross-jurisdiction conflicts resolve via the higher-restriction-wins
rule of ADR-0304. Worked example sequence:

1. **US warrant arrives** at oyatie's legal process address. Target
   is Diana Reyes's personal Mailbox. Diana is US-resident.
2. **Jurisdiction check** — Diana's `tenant.jurisdiction_code = 'US'`;
   her data resides in US cells. No EU restriction applies. US
   warrant proceeds per US scope rules.
3. **Counter-example** — US warrant arrives for the personal
   Mailbox of an EU-resident user. Their
   `tenant.jurisdiction_code = 'DE'`; data resides in EU-Sovereign
   cell (per ADR-0240). The cross-jurisdiction-conflict resolver
   computes:
   - US warrant scope vs GDPR Art. 48 + BDSG §26 + EU CCPI =
     GDPR/BDSG/CCPI is higher-restriction.
   - Resolution: REFUSE the US warrant; respond citing GDPR Art.
     48 + require MLAT-mediation via the US-EU MLAT-2003 +
     Umbrella Agreement.
4. **The MLAT path** — US prosecutor must route the request through
   DOJ Office of International Affairs → German Federal Ministry
   of Justice → German court → German judicial process. The
   resulting MLAT request would be a German court's domestic
   warrant, which the warrant-handler processes per the DE-jurisdiction
   overlay.
5. **Audit event emitted**: `WarrantRefused` with
   `refusal_reason = 'CROSS_JURISDICTION_HIGHER_RESTRICTION'` +
   `MLAT_REQUIRED` next-step.

This implements the Microsoft Ireland (2018) precedent — providers
challenge over-reaching warrants and require MLAT for foreign data.

### §D-6. Notification to affected user (per ADR-0299/0300 doctrine — when not gag-ordered)

Per 18 USC §2705(b) the US permits delayed notice up to 180 days
renewable; many warrants carry gag-orders. Notification flow:

1. **At intake**, the warrant-handler parses gag-order presence +
   expiry.
2. **If no gag-order**, notify user within 24 hours of warrant
   intake.
3. **If gag-order**, suppress notification. Schedule a
   gag-order-expiry check job for `gag_order_expires_at`.
4. **At gag-order expiry**, the substrate emits
   `WarrantGagOrderExpired` audit event; notification dispatch
   proceeds within 24 hours.
5. **Notification content**:
   - Warrant identifier (e.g., `warrant-2026-q2-0042`).
   - Issuing court (e.g., `US District Court for the District of
     Columbia`).
   - Judge name.
   - Docket number.
   - Statutory anchor (e.g., `18 USC §201; SCA §2703(d)`).
   - Scope summary (e.g., `Read a single Mail thread between
     <user> and offeror`).
   - Actions taken (e.g., `1 read access provided to
     DOJ-Investigations`).
   - Expiry date.
   - Ombudsman contact for inquiries.
   - Right to challenge the warrant via legal counsel.
6. **Notification delivery**:
   - Personal-tenant Mail to primary address.
   - Persistent banner in personal-tenant transparency dashboard.
   - Push notification to all registered personal-tenant devices
     (per ADR-0299 device list).
7. **The user's right to challenge** — every notification includes
   a templated legal-challenge form; the user may file a
   challenge via the per-pack ombudsman's challenge intake.

### §D-7. Per-pack regulator-cadence — when LEA disclosure must be reported

Regulator-cadence table per pack:

| Pack | Regulator | Cadence | Schema |
|---|---|---|---|
| US-federal-LE | US DOJ (industry convention) | semi-annual | matches Apple/Google/Microsoft format |
| EU-DSA | European Commission | annual per DSA Art. 24 + Art. 15 | EC-template |
| EU-GDPR-member-state | per-member DPA | annual | per-DPA template |
| KR-PIPA | KCC (Korea Communications Commission) | quarterly | KCC-template |
| JP-APPI | PPC (Personal Information Protection Commission) | annual | PPC-template |
| AU-ACMA | ACMA | annual | ACMA-template |
| CA-PIPEDA | OPC (Office of the Privacy Commissioner) | annual | OPC-template |
| SG-PDPA | PDPC (Personal Data Protection Commission) | annual | PDPC-template |
| IN-DPDP | DPB (Data Protection Board) | annual | DPB-template |
| BR-LGPD | ANPD (Autoridade Nacional de Proteção de Dados) | annual | ANPD-template |
| CN-PIPL | CAC (Cyberspace Administration of China) | annual + on-demand | CAC-template; sovereign-overlay |

The transparency-report µservice formats each pack's report per its
template. Reports are published at
`https://transparency.oyatie.local/<pack>/<period>` + signed by the
transparency officer.

### §D-8. Chain-of-custody — Merkle-sealed evidence chain per ADR-0028

Every event under the warrant lifecycle is sealed in the audit-chain
Merkle log per ADR-0028. The chain shape:

```
[warrant-document-blob (sealed in OpenBao)]
   ↓ hash → warrant_document_hash
[WarrantReceived event] → audit-chain Merkle leaf
   ↓ next leaf
[WarrantOmbudsmanAttested event] → audit-chain Merkle leaf
   ↓ next leaf
[WarrantGrantIssued event] → audit-chain Merkle leaf
   ↓ next leaf
[WarrantGrantUsed event × N] → audit-chain Merkle leaves
   ↓ next leaf
[WarrantGrantExpired event] → audit-chain Merkle leaf
   ↓ next leaf
[WarrantUserNotified event] → audit-chain Merkle leaf
   ↓ next leaf
[WarrantTransparencyDisclosed event] → audit-chain Merkle leaf
```

The Merkle root is sealed under both the issuing tenant's audit
stream and the platform-ombudsman's audit stream. Tampering with
any leaf invalidates the seal. The audit chain is queryable by
warrant_id; the chain-of-custody is auditable end-to-end.

### §D-9. Forbidden patterns — bulk warrant rejection; over-scope warrant rejection

Forbidden warrant patterns:

| # | Pattern | Refusal reason | Threshold |
|---:|---|---|---|
| 1 | Bulk principals without per-principal facts | BULK_OVER_PRINCIPAL_LIMIT | N>10 (or N>5 per EU pack) |
| 2 | Bulk resources without per-resource facts | BULK_OVER_RESOURCE_LIMIT | M>100 (or M>50 per EU pack) |
| 3 | "All Mail of tenant X" without specific thread/sender/subject | OVER_SCOPE | scope undelineated |
| 4 | Warrant without judge signature | LACKING_JUDICIAL_AUTHORITY | absent signature |
| 5 | Warrant without docket number | LACKING_JUDICIAL_AUTHORITY | absent docket |
| 6 | Warrant without statutory anchor | LACKING_STATUTORY_ANCHOR | absent statute cite |
| 7 | Warrant with expired execution window | WARRANT_EXPIRED_BEFORE_ISSUE | now() > execution_window_end |
| 8 | US warrant for EU-resident data without MLAT | CROSS_JURISDICTION_HIGHER_RESTRICTION | jurisdiction conflict |
| 9 | Warrant for journalist-source list | REPORTER_PRIVILEGE_PROTECTED | per ADR-0300 + shield law |
| 10 | "Future-data" warrant (intercept all future communications) | OVER_SCOPE | wiretap-Act paths required |
| 11 | Warrant for whistleblower-channel submissions | REPORTER_PRIVILEGE_PROTECTED | per ADR-0300 |
| 12 | Warrant for minor's personal-tenant data without parent/guardian + special judicial | OVER_SCOPE | per ADR-0292 |

Refusal procedure: emit `WarrantRefused` audit event; respond to
issuing authority via the legal-process channel with a refusal
explanation citing the reason + the legal basis; preserve the
warrant document in audit-chain for transparency-report disclosure.

### §D-10. Reporter-privilege exception (j06 cross-link per ADR-0300)

When a warrant targets:

- A journalist's source list,
- A whistleblower's submission chain,
- A pseudonymity-class principal under the anonymity-substrate per
  ADR-0300,

The reporter-privilege overlay activates:

1. **US federal** — DOJ 28 CFR §50.10 requires Attorney General
   approval; subpoena to journalist requires high-level review.
   The warrant-handler refuses and notifies the issuing authority
   that DOJ-policy attestation is required.
2. **US state shield laws** — NY Civil Rights §79-h; CA Evidence
   Code §1070; 49 other states have varying shield laws. Per-pack
   overlay encodes each.
3. **EU** — EU Whistleblower Directive 2019/1937 + EU Charter Art.
   11 (freedom of expression). Refuse.
4. **KR** — Anti-Corruption and Bribery Prohibition Act §17 + KR
   Press Freedom doctrine. Refuse.

Refusal emits `WarrantRefused` with
`refusal_reason = 'REPORTER_PRIVILEGE_PROTECTED'` and cites the
shield-law overlay.

## §E. Implementation footprint

### §E.1. New crate `oya-shared-warrant-handler`

Layer: shared-substrate (layer 5, per ADR-0105). Single-concern flat
layout per ADR-0131.

```
crates/oya-shared-warrant-handler/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs
│   ├── intake.rs                # warrant intake + parsing
│   ├── judicial_authority.rs    # judge-signature + docket validator
│   ├── statutory_anchor.rs      # per-jurisdiction statute resolver
│   ├── scope_validator.rs       # bulk/over-scope refusal classifier
│   ├── cross_jurisdiction.rs    # higher-restriction-wins per ADR-0304
│   ├── reporter_privilege.rs    # ADR-0300 + shield-law overlay
│   ├── ombudsman.rs             # ombudsman attestation requester
│   ├── grant_issuer.rs          # Cedar grant issuance
│   ├── notification.rs          # user notification when not gag-ordered
│   ├── transparency.rs          # transparency-report record emitter
│   ├── chain_of_custody.rs      # ADR-0028 Merkle-seal emitter
│   ├── audit_emitter.rs         # Warrant* event emitter
│   └── types.rs                 # WarrantState + WarrantRefusalReason enums
├── tests/
│   ├── intake_proptest.rs
│   ├── scope_validator_proptest.rs
│   ├── cross_jurisdiction_test.rs
│   ├── reporter_privilege_test.rs
│   ├── ombudsman_attestation_e2e.rs
│   ├── grant_issuer_integration.rs
│   ├── notification_e2e.rs
│   ├── transparency_e2e.rs
│   └── chain_of_custody_test.rs
├── benches/
│   └── warrant_intake_bench.rs
└── fuzz/
    └── warrant_document_fuzz.rs
```

Public surface (extract):

```rust
// crates/oya-shared-warrant-handler/src/lib.rs

#[non_exhaustive]
pub enum WarrantState {
    Received, Validated, Anchored, Scoped, JurisdictionOk,
    MlatRequired, PrivilegeOk, Attested, GrantActive,
    Expired, Revoked, Notified, Disclosed, Sealed, Refused,
}

#[non_exhaustive]
pub enum RefusalReason {
    BulkOverPrincipalLimit, BulkOverResourceLimit,
    LackingJudicialAuthority, LackingStatutoryAnchor, OverScope,
    CrossJurisdictionHigherRestriction, ReporterPrivilegeProtected,
    OmbudsmanRejected, WarrantExpiredBeforeIssue, Other,
}

pub struct WarrantIntake<'a> {
    pub warrant_id: &'a str,
    pub document_uri: &'a str,
    pub jurisdiction_code: &'a str,
    pub statutory_anchor: &'a str,
    pub issuing_court: &'a str,
    pub judge_name: &'a str,
    pub docket_number: &'a str,
    pub execution_window_start: i64,
    pub execution_window_end: i64,
    pub target_principals: &'a [&'a str],
    pub target_resources: &'a [&'a str],
    pub actions_authorized: &'a [&'a str],
    pub gag_order_active: bool,
    pub gag_order_expires_at: Option<i64>,
}

pub trait WarrantHandler {
    fn intake(&self, w: &WarrantIntake<'_>) -> Result<WarrantState, RefusalReason>;
    fn request_ombudsman_attestation(&self, warrant_id: &str) -> Result<AttestationReceipt, WarrantError>;
    fn issue_cedar_grant(&self, warrant_id: &str) -> Result<GrantReceipt, WarrantError>;
    fn notify_user_if_eligible(&self, warrant_id: &str) -> Result<NotificationReceipt, WarrantError>;
    fn record_transparency(&self, warrant_id: &str, period: &str) -> Result<TransparencyRecord, WarrantError>;
}
```

### §E.2. New µservice extension `microservices/governance/warrant-intake/`

The warrant-intake state machine + intake API + ombudsman dashboard
live under the governance µservice as a sub-feature:

```
microservices/governance/warrant-intake/
├── ARCHITECTURE.md
├── README.md
├── manifest.json
├── contracts/
│   ├── openapi-v1.yaml          # /v1/warrant/intake + /v1/warrant/{id}
│   └── asyncapi-v1.yaml         # WarrantReceived + WarrantGrantIssued + ...
├── policy/
│   └── warrant-intake.cedar     # Cedar gates for intake API
├── runbooks/
│   ├── warrant-emergency-revocation.md
│   ├── warrant-ombudsman-cohort-failover.md
│   └── warrant-cross-jurisdiction-mlat.md
├── migrations/
│   └── 0001_warrant_state.sql
├── dashboards/
│   └── warrant-intake-grafana.json
└── IP-NN-warrant-intake-bootstrap.md
```

Per the ADR constraints, the scaffold is NOT created in this ADR;
it appears in the Wave-3-F authoring queue.

### §E.3. New CI lanes

- `oya-governance-warrant-scope-enforced` — aggregate lane.
- `oya-governance-warrant-judicial-review` — per-µservice; checks
  ombudsman attestation present for SEV2+ warrants.
- `oya-governance-warrant-canary-transparency` — per-µservice;
  checks non-gag-ordered warrants surface in transparency report.
- `oya-governance-warrant-bulk-refusal` — per-µservice; checks bulk
  refusal threshold enforced.

### §E.4. New specs

- `specs/warrant-intake-schema.json` — the warrant intake payload
  shape.
- `specs/warrant-jurisdiction-scope-overlays.json` — per-jurisdiction
  scope thresholds.
- `specs/transparency-report-schema.json` — the transparency-report
  record shape.
- `specs/warrant-state-machine.json` — the warrant state machine.

### §E.5. New per-µservice files

Every personal-tenant-bearing µservice adds:

- `policy/court-warrant-scoped-piercing.cedar` — Cedar fragment.
- `ARCHITECTURE.md §warrant-grant-acceptance` — declares the
  µservice's behavior when serving requests under a warrant grant.

## §F. Migration

### §F.1. Sequencing

**Wave 1 (substrate; t=0 to t+14d):**
- `governance` — add warrant-intake/ subdirectory + state-machine
  migration.
- `policy-engine` — load
  `policy/court-warrant-scoped-piercing.cedar` baseline.
- `audit-chain` — register `Warrant*` event classes per ADR-0263.
- `identity` — wire warrant-grant evaluation to session-token
  enforcement.

**Wave 2 (consumer µservices; t+14d to t+30d):**
- `messenger`, `mail`, `drive`, `calendar`, `workflow-engine`,
  `workflow-studio`, `notes`, `payments`, `marketplace`,
  `community`, `meet` — all personal-tenant-bearing µservices load
  the Cedar fragment.

**Wave 3 (transparency-report µservice; t+30d to t+45d):**
- `microservices/governance/transparency-report/` — publish quarterly
  + warrant-canary surfaces.

**Wave 4 (per-pack overlays; t+45d to t+90d):**
- US-federal-LE-pack overlay loaded.
- EU-DSA-pack overlay loaded.
- KR-PIPA-pack overlay loaded.
- JP-APPI-pack overlay loaded.
- Other packs loaded sequentially per per-pack ombudsman onboarding.

### §F.2. Per-wave rollback

1. **Disable warrant-intake feature flag** — substrate falls back to
   "no warrant handling" (warrants queued for manual review only).
2. **Cedar fragment rollback** per ADR-0294 emergency rollback.
3. **Postgres rollback** is non-destructive (column drops + table
   drops preserve data via OpenBao backup).
4. **Audit event class rollback** — events remain in registry; emit
   pause feature-flag.

### §F.3. Multi-region awareness

- Each cell-tier runs its own warrant-intake instance (per ADR-0248).
- Cross-cell warrant references via the audit-chain Merkle log.
- Sovereign-cloud overlays per ADR-0240 apply: CN-cell warrants are
  handled by the CN-sovereign warrant-intake instance and never
  cross out of the CN cell.
- DR pair failover (per ADR-0241): warrant state machine replicated
  active-passive; DR failover preserves in-flight warrants.

### §F.4. Sunset

The legacy ad-hoc warrant-handling path (if any pre-ADR-0312
implementations existed) is sunset on 2026-09-15.

### §F.5. Versioning

- `oya-shared-warrant-handler` ships at 1.0.0; `#[non_exhaustive]`
  on enums; SemVer per ADR-0258.
- Warrant intake API at `/v1/warrant/intake`; future versions per
  ADR-0258.

## §G. References

### §G.1. Court precedents

- **Microsoft v. United States (2018) "Microsoft Ireland"** — 584
  U.S. ___ (2018) (per curiam) — mooted by CLOUD Act.
- **In re Search Warrant Issued to Twitter, Inc. (2014)** — Manhattan
  Criminal Court People v. Harris.
- **United States v. Warshak (2010)** — 631 F.3d 266 (6th Cir.
  2010) — established 4th Amendment protection for stored email.
- **Carpenter v. United States (2018)** — 585 U.S. ___ (2018) —
  4th Amendment protection for cell-site location records.
- **Constitutional Court of Korea 2018Hun-Ma1059 (2022)** — bulk
  prosecutorial demands without specific facts unconstitutional.
- **CJEU Schrems II (2020)** — Case C-311/18 — invalidated EU-US
  Privacy Shield; bears on cross-jurisdiction transfers.
- **Smyth v. Pillsbury (1996)** — 914 F. Supp. 97 (E.D. Pa. 1996) —
  employer monitoring of work email (referenced via ADR-0311 §A.1).

### §G.2. Statutory anchors

- 18 USC §201 (bribery of public officials).
- 18 USC §2511 (ECPA Title III).
- 18 USC §2701-2712 (Stored Communications Act).
- 18 USC §2703 (SCA disclosure orders).
- 18 USC §2705(b) (delayed notice).
- 18 USC §2713 (CLOUD Act extraterritorial).
- 18 USC §2523 (CLOUD Act executive agreements).
- 28 CFR §50.10 (DOJ policy re: news media).
- GDPR Art. 48 (transfers not authorised by Union law).
- DSA Art. 9 (orders to act against illegal content).
- DSA Art. 10 (orders to provide information).
- DSA Art. 15 (transparency reporting).
- DSA Art. 24 (additional online platform transparency).
- KR Telecommunications Business Act Art. 83.
- KR Anti-Corruption and Bribery Prohibition Act §17.
- JP Criminal Procedure Code Art. 218.
- AU Telecommunications (Interception and Access) Act 1979.
- CA Charter §8 + Criminal Code §487.
- CN Cybersecurity Law + PIPL.
- BR LGPD Art. 33.
- IN IT Act §69 + DPDP Act 2023.
- EU Whistleblower Directive 2019/1937.

### §G.3. Hyperscaler precedents

- Apple Transparency Report — apple.com/legal/transparency/.
- Google Transparency Report — transparencyreport.google.com.
- Microsoft Law Enforcement Requests Report —
  microsoft.com/en-us/corporate-responsibility/lerr.
- Twitter/X Transparency Report — transparency.twitter.com.
- Cloudflare Transparency Report — cloudflare.com/transparency.
- Reddit Transparency Report — reddit.com/policies/transparency-report.
- GitHub Transparency Report — gh.io/transparency.
- Stripe Transparency Report — stripe.com/files/legal/stripe-transparency-report.pdf.

### §G.4. Internal references

- documentation-rigor.md §1.1, §1.2, §2 ADR-row, §3.2.5 row 19
  (ombudsman attestation invariant).
- docs/user-journeys/CATALOG-j126-j150-ecosystem.md (j129 worked
  example).
- ADR-0028 (audit-chain Merkle-seal).
- ADR-0243 (Cedar default-deny gate).
- ADR-0244 (tenant scoping — CrossTenantGrant entity-type).
- ADR-0246 + amendment (library-first Cedar dispatch).
- ADR-0247 (self-modification doctrine — internal-tenant
  principals).
- ADR-0263 (audit-event emission contract).
- ADR-0272 (consent-management).
- ADR-0292 (minor-user doctrine — minor pierce extra-review).
- ADR-0294 (Cedar fragment soak — emergency revocation).
- ADR-0299 (account-recovery — passkey identity).
- ADR-0300 (whistleblower-press-freedom — reporter-privilege
  exception).
- ADR-0304 (cross-jurisdiction conflict resolution — higher-
  restriction-wins).
- ADR-0311 (dual-tenant identity — the boundary this ADR pierces).

### §G.5. Catalog cross-link

This ADR was surfaced by `docs/user-journeys/CATALOG-j126-j150-ecosystem.md`,
specifically j129 (`court-warrant-pierces-personal-tenant-with-judicial-oversight`)
+ j130 (cross-link to ADR-0300) + j131 (cross-link to ADR-0304).

### §G.6. Industry guidance + standards

- **NIST SP 800-53 Rev. 5** — AC-21 Information Sharing controls
  + AU-12 Audit Generation controls bear on warrant chain-of-custody
  shape.
- **NIST SP 800-92** — Guide to Computer Security Log Management;
  informs the Merkle-seal cadence of §D-8.
- **ISO/IEC 27037:2012** — Guidelines for identification, collection,
  acquisition and preservation of digital evidence; informs the
  chain-of-custody shape.
- **ISO/IEC 27050-1:2019** — Electronic discovery overview and
  concepts; bears on warrant-derived discovery requests.
- **The Berkman Klein Center "Don't Panic" Report (2016)** on
  transparency disclosures; bears on warrant-canary shape.
- **Stanford Internet Observatory + EFF "Who Has Your Back"** annual
  reports — measure provider warrant-handling rigor; oyatie's
  warrant-handler is engineered to score top-tier on every published
  metric (user notification, transparency report, refusal of
  bulk warrants, judicial-process bar, public policy advocacy).
- **EFF "Who Has Your Back" 2024** — used as a calibration anchor
  for the §B.6 + §B.7 + §B.8 + §D-4 commitments.
- **Access Now Transparency Report Tracker (2024)** — informs the
  per-jurisdiction regulator-cadence in §D-7.
- **CDT (Center for Democracy and Technology) "Government Hacking
  Toolkit"** — informs the §D-9 forbidden-patterns list (especially
  rows 10-12 on future-data warrants + whistleblower channels +
  minor users).
- **R Street Institute "Lawful Access to Encrypted Data" (2024)** —
  informs the encryption-boundary interaction with warrant scope
  (warrant cannot compel decryption of E2EE content where the
  platform lacks keys; per ADR-0251 §D-10 encryption-BYOK).

### §G.7. Cross-doctrine integration

The warrant-handler integrates with:

- **ADR-0028 audit-chain Merkle-seal** for chain-of-custody.
- **ADR-0046 incident-response framework** for emergency-disclosure
  request paths.
- **ADR-0099 data-class registry** — warrant grant resources MUST
  reference data classes via the registry, never ad-hoc.
- **ADR-0140 Cedar policy enforcement** for grant evaluation.
- **ADR-0188 passkey WebAuthn canonical-auth** for ombudsman
  attestation signature.
- **ADR-0212 buildability doctrine** for CI lane shape.
- **ADR-0247 self-modification doctrine** — warrants directed at
  oyatie's internal Foundry sub-tenants follow the same scope-
  bounded grant model (no carve-outs).
- **ADR-0276 backup portability** — warrant scope must not pierce
  the personal-tenant portable-export bundle without explicit
  warrant clause covering exports.
- **ADR-0292 minor-user doctrine** — warrants targeting minor
  users (<13 COPPA tier, 13-17 KOSA tier) require parent/guardian
  notification + state-AG-coordination in addition to ombudsman
  attestation.
- **ADR-0295 bootstrap CI SPIFFE kill-switch** — warrant-handler
  itself runs under SPIFFE workload identity; bootstrap-trust path
  protected.

### §G.8. Catalog journey reachability

The six-hops invariant per documentation-rigor.md §3.1 holds from
this ADR:

- ADR-0312 → ADR-0311 (1 hop) → ADR-0244 (2 hops) → tenant-model
  spec (3 hops). Reachable in ≤3 hops.
- ADR-0312 → ADR-0028 (1 hop) → audit-chain spec (2 hops). Reachable
  in ≤2 hops.
- ADR-0312 → ADR-0300 (1 hop) → whistleblower-channel spec (2 hops).
  Reachable in ≤2 hops.
- ADR-0312 → ADR-0304 (1 hop) → cross-jurisdiction-resolver spec
  (2 hops). Reachable in ≤2 hops.

## §H. Change log

- 2026-05-20: Initial draft (this document). Surfaced by Wave-3-E
  ecosystem catalog j129. Keystone-bundle 2026-05-20. Companion to
  ADR-0311.

— End of ADR-0312 —
