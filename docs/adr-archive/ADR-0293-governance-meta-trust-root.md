---
id: ADR-0293
status: Rejected
date: 2026-05-20
owners:
  - council-architecture
  - council-security
  - council-privacy
  - council-legal
  - ops-compliance
  - ops-sre-reliability
  - axis-identity
  - axis-policy-engine
  - axis-audit-chain
supersedes: []
amends: []
requires_amendment_to:
  - ADR-0247-self-hosting-self-modification-doctrine.md (§D-8 predicate `is_automated_with_baseline_signed_workflow` no longer evaluates to true on workflow-publisher signature alone; must also carry meta-trust-root witness signature; §D-4 step 0.4 Shamir parameters change from M=3,N=5 to M=5,N=9 across ≥3 jurisdictions for the meta-trust-root key and any other trust-chain anchor; §D-2 sub-scope `oyatie.foundry.meta-trust-root-attestor` added)
  - ADR-0243-cedar-as-universal-gate.md (§D-5 bootstrap chain of trust gains a separately-rooted meta-trust-root anchor; §D-5 Shamir parameters change from M=3,N=5 to M=5,N=9 across ≥3 jurisdictions)
  - ADR-0246-policy-engine-substrate-promotion.md (Cedar fragments touching the trust-chain MUST carry an `attested_by_meta_trust_root: true` annotation; the proposed advisory enforcement check is `oya-check-meta-trust-root-attestation`)
superseded_by: []
related:
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0144-eu-ai-act-graduated-risk-tier-model.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0250-build-ahead-of-certification-doctrine.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0255-intelligence-as-two-layer-ai-substrate.md
  - ADR-0280-substrate-of-substrate-dependency-doctrine.md
related_specs:
  - /specs/platform-architecture.json
  - /specs/microservices/policy-engine.json
  - /specs/microservices/audit-chain.json
  - /specs/microservices/identity.json
  - /specs/bootstrap-tier-model.json
  - /specs/self-modification-cedar-fragment-schema.json
  - /specs/meta-trust-root-key-ceremony.json
related_memory:
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_cedar_as_universal_gate
  - feedback_no_silent_regression
  - feedback_clean_architecture_requirements
  - feedback_quality_performance_scalability_bar
  - feedback_autonomous_implementation_artifacts
  - feedback_self_modification_doctrine
  - feedback_byok_everywhere_credentials
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-foundational-doctrine
keystone_position: promotion-gate-fix-1-of-4
authority_for_existence: docs/architecture/keystone-bundle-2026-05-20-synthesis.md §5.1 + §5.5
closes_findings:
  - F5-247-01 (self-modification meta-trust circular predicate, CRITICAL)
  - F5-243-02 (Shamir M-of-N too narrow, HIGH, expansion arm)
  - M1-KB-F4 (Shamir M-of-N too narrow)
naming_justifications:
  - name: oyatie.foundry.meta-trust-root
    bnf_v4_1: tenant=`oyatie` (reserved-namespace) · sub_scope=`foundry.meta-trust-root` (kebab segments, leading char alpha, no underscores per ADR-0244 §D-2 regex) · arity=3 (canonical max per BNF v4.1)
    layer_enum_adr_0105: `trust-anchor` (one of the 13 canonical layers; trust-anchor sits below `substrate` in the canonical enum and is the seat of any key whose compromise breaks the platform's chain of trust)
    rationale: Separate sub-scope under oyatie root tenant; preserves ADR-0242 reserved-namespace doctrine; explicit `meta-trust-root` keyword signals its role as the witness-signer for self-modification gates; distinct from `oyatie.foundry.workflow-publisher` to break the F5-247-01 circular-predicate exploit
  - name: oyatie.foundry.meta-trust-root-attestor
    bnf_v4_1: tenant=`oyatie` · sub_scope=`foundry.meta-trust-root-attestor` · arity=3
    layer_enum_adr_0105: `trust-anchor`
    rationale: Day-to-day automation principal that requests witness signatures from offline meta-trust-root key holders; does NOT hold key material itself; mirrors AWS KMS GenerateDataKey caller separation
  - name: oya-shared-meta-trust-root
    bnf_v4_1: shared-domain crate per `feedback_glossary_shared_not_platform`; kebab-case; canonical-base + localization-overlay shape per `feedback_canonical_base_localization`
    layer_enum_adr_0105: `shared`
    rationale: Shared crate exposing witness-verification primitives + Cedar `is_attested_by_meta_trust_root` predicate
  - name: oya-check-meta-trust-root-attestation
    bnf_v4_1: gate-name convention `oya-check-<predicate>` per ADR-0212 buildability doctrine
    layer_enum_adr_0105: `gate`
    rationale: CI lane verifying every trust-chain Cedar fragment carries a meta-trust-root witness signature; blocks promotion if absent
enforcement_status: advisory-until-meta-trust-root-ceremony-runs
enforced_by:
  - oya gate validate meta-trust-root-attestation
  - oya gate validate meta-trust-root-key-ceremony-evidence
  - oya gate validate meta-trust-root-shamir-distribution
  - oya gate validate meta-trust-root-rotation-cadence
  - oya gate validate self-modification-witness-present
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


> **Disposition light-edit (2026-08-06):** Keep Rejected: Foundry meta-trust-root self-mod — Foundry-framed; needs rewrite under intelligence before Accept

# ADR-0293: Foundry Meta-Trust-Root for Self-Modification Witness

## Status

Proposed — 2026-05-20.

Promotion-gate fix **1 of 4** for the keystone bundle 2026-05-20
(`docs/architecture/keystone-bundle-2026-05-20-synthesis.md` §5.1 +
§5.5). This ADR closes F5-Security findings **F5-247-01** (CRITICAL,
self-modification circular meta-trust predicate) and the
Shamir-expansion arm of **F5-243-02** (HIGH) + **M1-KB-F4**. The
keystone ADRs ADR-0247, ADR-0243, and ADR-0246 cannot promote from
`Proposed` to `Accepted` until this ADR's mechanics are implemented
and the meta-trust-root key ceremony runs.

Enforcement is `advisory-until-meta-trust-root-ceremony-runs`. The
CI lanes that enforce this ADR become BLOCKER once:

1. The meta-trust-root key ceremony has executed per §D-2 with all
   nine shard holders' presence attested by council-security + a
   notary outside the founding team.
2. The first meta-trust-root witness signature is verifiable by every
   per-cell policy-engine evaluator against the canonical chain-of-
   trust document.
3. The ADR-0247 §D-8 Cedar fragment is republished with the
   `is_attested_by_meta_trust_root` predicate in place of the
   previously-vulnerable `is_automated_with_baseline_signed_workflow`
   alone.
4. The kill-switch rehearsal in §D-7 has executed end-to-end at least
   once in `dev-tools-cell-staging`.
5. `oya-check-meta-trust-root-attestation` lane scans every Cedar
   fragment in `microservices/policy-engine/fragments/baseline/`
   and every workflow in `oyatie.foundry.*` and reports zero
   missing-witness findings.

## Date

2026-05-20.

## Context

### What F5-247-01 actually says

F5-Security's r1 verdict (CRITICAL) reads:

> The Cedar fragment at ADR-0247 §D-8 permits self-modification actions
> when `principal.is_human_approval_present(min_approvers: 2) ||
> principal.is_automated_with_baseline_signed_workflow`. The second
> predicate is the autonomous-masterplan path. But: what makes a
> workflow 'baseline-signed'? Per §D-2 + §D-3, workflows are versioned
> objects in Workflow Engine; their signatures live in the workflow
> library repo. The workflow library repo is itself self-modifiable
> by `oyatie.foundry.workflow-publisher`. Therefore: if
> `oyatie.foundry.workflow-publisher` is compromised (single
> principal), it can publish a new workflow version that performs
> arbitrary self-modification with NO human in the loop, because the
> signed-workflow predicate evaluates true for any version signed by
> the publisher's intermediate key.

The exploit path:

1. Adversary compromises one principal (`oyatie.foundry.workflow-
   publisher`) via a single vulnerable Cargo dependency exploited at
   build time, a single supply-chain CVE, or a single phishing event
   against the council member who holds the publisher's intermediate
   key.
2. Adversary publishes a new version of any baseline workflow (for
   example `oyatie.foundry.adr-drafter`) that, in the course of its
   normal execution, ALSO invokes
   `Substrate::Action::DeploySubstrateVersion` with a backdoored image
   reference.
3. The Cedar predicate `is_automated_with_baseline_signed_workflow`
   evaluates true because the workflow IS signed by the workflow-
   publisher key. The two-human-approver branch never fires.
4. Substrate deploys the backdoored image. The platform's audit chain
   shows the deployment as legitimate because every step carried a
   valid signature from a valid intermediate key.

This is a **single-event total compromise**. F5 ranks it CRITICAL
because the platform's autonomous-masterplan goal (per
`feedback_autonomous_implementation_artifacts`) requires that
self-modification proceed under automation, and the predicate as
authored binds "automation legitimacy" to a key that participates in
day-to-day publication operations — exactly the operational key most
likely to be compromised.

The structural defect is **circularity of trust**: the predicate that
proves the workflow is trustworthy is signed by the same authority
that publishes the workflow. There is no independent witness.

### What F5-243-02 + M1-KB-F4 add

F5-243-02 (HIGH) and M1-KB-F4 jointly observe that the org root key
Shamir parameters (M=3, N=5 per ADR-0247 §D-4 step 0.4) are too narrow
to survive nation-state-grade adversaries:

> Nation-state coercion of three named individuals (or compromise of
> three home-office safes) recovers the org root key.

The fix is twofold:

1. Raise Shamir defaults to ≥5-of-9 across ≥3 jurisdictions for every
   trust-chain anchor (org root, meta-trust-root introduced here,
   compliance-pack publisher root).
2. Add a duress-revocation path: any one shard-holder can unilaterally
   trigger a platform freeze.

This ADR closes the meta-trust-root + duress-revocation aspects of
F5-243-02. The compliance-pack publisher root + org root expansion are
also folded in via the §D-2 / §D-4 amendment list.

### Why "meta-trust-root" specifically (vs. additional human approver)

Three alternatives were considered before settling on
`oyatie.foundry.meta-trust-root`:

| Alternative | Why rejected |
|---|---|
| **A. Require ≥3 human approvers always.** Hard-code human approval into the autonomous-masterplan path. | Defeats the autonomous-masterplan goal (per `feedback_autonomous_implementation_artifacts`). The user explicitly rejects half-finished implementations that bypass automation. |
| **B. Require workflow signature by a second council member's intermediate key.** | Reduces the single-key compromise to a two-key compromise — better but not safer-by-shape. Both keys are still day-to-day-operational; both can still be compromised through ordinary supply-chain or phishing means. |
| **C. Introduce `oyatie.foundry.meta-trust-root` as a separately-rooted, offline-HSM-resident principal.** | Selected. The witness key never participates in day-to-day publication; it is invoked only at workflow-version-promotion time and at meta-permit-fragment publication time; compromise requires the much harder offline-HSM-ceremony attack. |

The selected resolution matches the PKI separation-of-duties pattern
canonical across industry root CAs (Mozilla CA Certificate Policy
§4.6, Let's Encrypt root ceremony 2015 + 2024 rollover, AWS Private
CA hierarchy, GCP Certificate Authority Service offline-root pattern,
DigiCert offline-root + issuing-CA topology). In every case the root
key is offline, used only at intermediate-CA-issuance time, and
witnessed by an externally-attested ceremony.

### Why offline HSM + multi-jurisdiction Shamir

Five named precedents informed the §D-2 ceremony shape:

- **Internet Root Key Ceremony (DNSSEC IANA).** ICANN's KSK ceremonies
  (held every three months in El Segundo and Culpeper since 2010)
  combine an offline HSM (a Thales nShield XC), seven
  Cryptographic Officers from seven distinct countries (each holding
  a smartcard component of the operator card), and a separate
  Trusted Community Representative quorum. The full procedure is
  publicly documented in IANA-DNSSEC-Practice-Statement §5.2. M-of-N
  thresholds are 3-of-7 for operator cards and 5-of-7 for the safe
  combination — multi-jurisdiction by design.
- **Mozilla CA Certificate Policy §6.2.7.** Mandates offline storage
  for root keys, m-of-n procedural controls for activation, and
  formal ceremony witnesses including at least one independent
  auditor.
- **AWS KMS Custom Key Store + AWS Nitro Enclave attestation
  ceremonies.** AWS publishes its FIPS 140-3 L3 HSM provisioning
  ceremony pattern at re:Invent 2023 "How AWS Builds Trust" session
  SEC403; the pattern is multi-region key shard distribution with
  ceremony video-recording for post-hoc verification.
- **Azure Key Vault Managed HSM + Microsoft Defense ATO ceremony.**
  Microsoft Sovereign Cloud key-ceremony documentation (2024)
  specifies geographic distribution of key shards across countries
  outside the FVEY block for sovereign customer roots.
- **Google Cloud KMS Asymmetric Signing + Google Trust Services
  WIDGE ceremony.** Google documents an analogous offline-root + HSM
  ceremony for Google Trust Services root CA, including the 2023
  GTS-R3 + GTS-R4 root rollover.

oyatie's meta-trust-root ceremony adopts the **5-of-9 across
≥3 jurisdictions** parameter because:

- 5-of-9 has higher resilience against shard-loss than 3-of-5 (any
  four shards may be lost or unavailable without rendering the key
  unrecoverable) while keeping ceremony-attendance feasible.
- 3-jurisdiction distribution makes single-state legal-process
  compromise non-trivial: any one jurisdiction's legal process can
  reach at most ⌈9/3⌉ = 3 shards, less than the 5-shard threshold.
- The combinatorial brute-force surface against 5-of-9 holders is
  C(9,5) = 126 candidate sets, each requiring physical-coercion or
  HUMINT access to five geographically-separated individuals; the
  expected adversary cost is approximately 100× that of 3-of-5.

### Boundary with org root + with workflow-publisher

oyatie's chain of trust after this ADR has three distinct anchors:

```
┌──────────────────────────────────────────────────────────┐
│  org-root-key                                             │
│  • Tier-0 HSM (offline)                                   │
│  • 5-of-9 Shamir across ≥3 jurisdictions                  │
│  • Used ONLY at intermediate-key rotation                 │
│  • Signs: meta-trust-root-key, workflow-publisher-key,    │
│           pack-publisher-key, compliance-anchor-key       │
│  • Rotation cadence: 5 years                              │
└──────────────────────────────────────────────────────────┘
        │                              │
        │ certifies                    │ certifies
        ▼                              ▼
┌────────────────────────────┐  ┌─────────────────────────┐
│ meta-trust-root-key        │  │ workflow-publisher-key  │
│ • Tier-0 HSM (offline)     │  │ • Tier-1 HSM (online)   │
│ • 5-of-9 Shamir / ≥3 juris │  │ • Used in day-to-day    │
│ • Used ONLY for witness    │  │   workflow publication  │
│   signatures on self-      │  │ • Rotation cadence:     │
│   modification ceremonies  │  │   90 days               │
│ • Rotation cadence: 1 year │  │ • Compromise = bounded  │
│ • Compromise = MUST trigger│  │   blast radius because  │
│   org-root re-attestation  │  │   self-modification     │
│   ceremony within 7 days   │  │   requires ALSO a       │
│                            │  │   meta-trust-root       │
│                            │  │   witness signature     │
└────────────────────────────┘  └─────────────────────────┘
```

The structural property is: **no single key — including the
workflow-publisher key — can independently authorize self-
modification.** Self-modification always requires either
≥2 human-approvers OR ≥1 meta-trust-root witness signature in
addition to the workflow-publisher signature.

The meta-trust-root key is **not** the org root key. The org root is
the deepest anchor and rotates rarely; the meta-trust-root is one
intermediate role beneath it and rotates yearly. The separation
allows the meta-trust-root to be re-ceremonied under compromise
without an org-root ceremony.

### Why now (2026-05-20)

Three forcing functions:

1. **F5-247-01 is the keystone bundle's only CRITICAL self-
   modification finding.** ADR-0247 cannot promote to `Accepted`
   until it is closed. The synthesis doc (§5.1) names this ADR as
   the resolution.
2. **The autonomous-masterplan goal needs the witness path defined
   before the first autonomous self-modification cycle runs.** Per
   ADR-0247 §D-5 Stage 4 ("dev-tools-cell-prod hosts the dev-tools
   workflow library; self-modification cycles execute autonomously
   under Cedar gates"), the witness mechanism must be live before
   that stage.
3. **Multi-jurisdiction Shamir requires lead time.** Shard-holder
   recruitment, jurisdictional vetting, HSM procurement (FIPS 140-3
   L3 certified), and ceremony scheduling have a minimum 8-week
   lead-time per industry precedent. The bundle merging now lets
   the implementation work begin in parallel with promotion-gate
   closure.

## Decision

The keystone establishes seven decision sub-sections, D-1 through
D-7.

### D-1. `oyatie.foundry.meta-trust-root` principal — definition

A new principal `oyatie.foundry.meta-trust-root` is introduced under
the `oyatie` tenant per ADR-0242 §D-2. It is a sub-scope of the
`oyatie.foundry.*` family but **architecturally distinct** from all
other `oyatie.foundry.*` principals in three ways:

1. **No automated workflow may impersonate the meta-trust-root
   principal.** Its private key material lives in an offline HSM and
   never enters any online process. Daily automation calls into the
   `oyatie.foundry.meta-trust-root-attestor` companion principal,
   which **requests** witness signatures from the offline holders
   via an out-of-band ceremony — it does not sign anything itself.
2. **The principal has exactly one action authority.** Cedar permits
   for `oyatie.foundry.meta-trust-root` are restricted to:
   - `MetaTrustRoot::Action::IssueWitnessSignature`
   - `MetaTrustRoot::Action::RevokeWitnessSignature`
   - `MetaTrustRoot::Action::RotateMetaTrustRootKey`
   - `MetaTrustRoot::Action::TriggerDuressRevocation`
   Any other action attempted under this principal is default-denied
   (matches ADR-0243 §D-3 default-deny posture).
3. **All actions emit to the dedicated audit stream
   `audit.meta-trust-root` with Merkle-sealed-per-event evidence
   level** (matches the F5 ASC-05 recommendation for high-stakes
   substrate operations).

The principal is **not** an `oyatie.foundry.*` workflow-execution
principal. The `oyatie.foundry.workflow-publisher` and
`oyatie.foundry.adr-drafter` principals remain operational; the
meta-trust-root principal sits beside them as an out-of-band witness
authority, not as a workflow executor.

### D-2. Key ceremony — 5-of-9 Shamir, ≥3 jurisdictions, FIPS 140-3 L3

The meta-trust-root key is generated and stored under the following
ceremony procedure. The procedure is materially identical for both
the initial ceremony and each yearly rotation.

#### D-2.1. Ceremony participants

| Role | Count | Eligibility |
|---|---|---|
| Cryptographic Officers (CO) | 9 | One per shard; ≥3 must be domiciled in jurisdictions outside FVEY (Five-Eyes); ≥1 from EU; ≥1 from APAC; ≥1 from Latin America or Africa |
| Internal Witnesses | 3 | One each from council-security, council-architecture, council-legal |
| External Witnesses | 2 | One Qualified Trust Service Provider (QTSP) representative + one independent auditor (no oyatie equity) |
| Ceremony Master | 1 | council-security designate, rotates yearly, does NOT hold a shard |
| Notary Public | 1 | Jurisdiction-licensed notary; not an oyatie employee |
| Recording Operator | 1 | Operates the immutable-video-recording rig; not an oyatie employee |

Total: 17 attestable participants. The ceremony must be conducted
**in person** at a designated facility; remote participation is not
permitted (matches ICANN KSK ceremony posture).

#### D-2.2. Hardware specification

| Component | Specification |
|---|---|
| Primary HSM | Thales Luna Network HSM 7 (FIPS 140-3 Level 3 certified; certificate #4543) OR Entrust nShield XC (FIPS 140-3 Level 3 certified; certificate #4458). Brand+model is fixed for any given key lifetime; rotation may select either. |
| Backup HSM | Second HSM of the same brand/model installed in a geographically-separate facility (≥1000km) and synchronized via Shamir reconstruction, not key replication |
| Shard storage media | Tamper-evident smartcards (e.g., Gemalto IDPrime 940 with FIPS 140-3 L2 certification); each shard holder receives one smartcard sealed in a tamper-evident bag |
| Ceremony recording | Dual independent 4K cameras with offline storage; audio captured separately; recordings sealed in tamper-evident container at ceremony close |
| Air-gap workstation | Dell Precision workstation with no wireless radios (physically removed at procurement); fresh OS install per ceremony; destroyed at ceremony close per NIST 800-88 Purge |

The HSM brand + model + FIPS certificate number is recorded in the
`/specs/meta-trust-root-key-ceremony.json` artifact at ceremony close
and audited annually for vendor advisory updates.

#### D-2.3. Shard distribution

After key generation inside the HSM, the key material is exported as
9 Shamir secret shares (Adi Shamir's 1979 scheme; threshold
M=5, N=9). Each share is sealed onto one smartcard in a tamper-
evident bag.

Distribution constraints:

- **Geographic.** Shards 1-3 distribute to North America (one each
  to three distinct US states OR to US/Canada/Mexico); shards 4-6
  distribute to EU (three distinct EU member states); shards 7-9
  distribute to one APAC jurisdiction, one African jurisdiction or
  Latin American jurisdiction, and one remaining EU member state OR
  Oceania (Australia / NZ).
- **No two shards held by individuals with first-degree familial
  relations** (spouse, parent, child, sibling).
- **No two shards held by individuals in the same employer outside
  oyatie** (matches Mozilla CA root-key ceremony rule).
- **At least one shard held by a council-architecture member; at
  least one by a council-security member; at least one by a
  council-legal member;** the remaining six are independent
  Cryptographic Officers vetted by council-security.

The shard holders sign a formal `MetaTrustRootShardHolderAgreement`
that obligates them to:

1. Store the shard in a tamper-evident container in a personally-
   controlled safe.
2. Attend any duly-called ceremony within ≤72 hours of notification.
3. Trigger the duress-revocation path (per §D-7) if coerced or if
   their shard's tamper-evident seal is broken without authorisation.
4. Notify the Ceremony Master immediately upon any change of legal
   residence that would cross jurisdiction.

#### D-2.4. Ceremony attestation chain

At ceremony close, the following artifacts are produced and signed:

| Artifact | Signed by | Stored in |
|---|---|---|
| `MetaTrustRootKeyGenerationCeremonyAttestation.pdf` | All 17 participants + Notary apostille | audit-chain (Merkle-sealed-per-event) + `/specs/meta-trust-root-key-ceremony.json` + sealed paper original in tamper-evident container held by Notary |
| Public key + key fingerprint | Burned to two write-once optical media + posted to RFC 9162 Certificate Transparency log | Two geographically-separate immutable storages + public CT log |
| HSM serial numbers + certificate chain | HSM vendor's attestation key + Notary apostille | audit-chain + paper original |
| Video recording of full ceremony | Dual-camera recording operator + Notary | Tamper-evident container; opened only on subsequent ceremony's audit phase |
| Air-gap workstation destruction certificate | Notary + Ceremony Master + Recording Operator | audit-chain + paper original |

The ceremony attestation chain is **the audit substrate for the
meta-trust-root key for its entire lifetime.** Any subsequent
witness signature issued by the key references its ceremony
attestation hash.

### D-3. Witness-signature protocol — how a meta-trust-root signature happens

Day-to-day automation calls into the
`oyatie.foundry.meta-trust-root-attestor` principal, which is an
online automation but holds **no signing key material**. The attestor
mediates between the substrate and the offline shard-holders.

#### D-3.1. Sequence diagram

```
1. oyatie.foundry.workflow-publisher prepares a workflow-version
   promotion candidate. The candidate is signed by the publisher's
   intermediate key (per ADR-0247 §D-8 baseline-signed-workflow
   pathway).

2. workflow-publisher calls
   meta-trust-root-attestor.request_witness_signature(
       artifact_hash, artifact_metadata, intended_action, justification
   ).

3. meta-trust-root-attestor:
   3.1. Verifies workflow-publisher's signature using ADR-0243 §D-5
        chain-of-trust.
   3.2. Verifies that intended_action is in the meta-trust-root
        action allowlist (substrate-deploy, fragment-publish over
        baseline/, workflow-promote over baseline workflows).
   3.3. Verifies that the artifact's multispectrum-review evidence
        is complete per ADR-0247 §D-2 (PR-review verdict + eval
        verdict + security verdict + naming verdict).
   3.4. Constructs a CeremonyRequest payload containing:
        - artifact_hash + cosign attestation
        - intended_action + Cedar resource ID
        - justification text (≤ 2000 chars)
        - workflow-publisher signature
        - timestamp + nonce
        - SHA3-256 of all above.

4. meta-trust-root-attestor PUBLISHES the CeremonyRequest to the
   `meta-trust-root-ceremony-queue` Kafka topic with
   exactly-once-effective delivery and Merkle-sealed-per-event audit
   emission.

5. ≥5 of 9 shard holders (within their ≤72h response window):
   5.1. Independently fetch the CeremonyRequest via their personal
        attestor-client (an air-gapped device).
   5.2. Manually verify the artifact_hash matches a hash they
        independently retrieved from the workflow library + cosign
        registry (out-of-band).
   5.3. If satisfied, attend a synchronous ceremony (in-person or
        secure-video-bridge with hardware key-attestation; see
        §D-3.4 for the secure-video-bridge profile) at which they
        each load their smartcard into the ceremony HSM in turn.

6. Once ≥5 smartcards are loaded, the HSM internally:
   6.1. Reconstructs the meta-trust-root key.
   6.2. Verifies the CeremonyRequest's outer signature chain again.
   6.3. Signs the artifact_hash with the meta-trust-root key under
        the Ed25519+cosign attestation profile.
   6.4. Emits the WitnessSignature object containing:
        - artifact_hash
        - intended_action
        - signed_at (HSM-internal RTC; not NTP)
        - ceremony_attestation_hash (the §D-2.4 hash, fixed for the
          life of the key)
        - signer_pubkey_fingerprint
        - signature (Ed25519, 64 bytes)
        - validity_window: signed_at .. signed_at + 24h

7. The HSM emits the WitnessSignature back to the ceremony-queue
   topic. meta-trust-root-attestor receives it.

8. meta-trust-root-attestor returns the WitnessSignature to
   workflow-publisher.

9. workflow-publisher embeds the WitnessSignature into the workflow-
   version promotion record + the audit-chain emission.

10. policy-engine, when evaluating the subsequent self-modification
    action, checks:
    - principal.is_attested_by_meta_trust_root  ← NEW PREDICATE
    - the WitnessSignature is within its 24h validity window
    - the WitnessSignature's intended_action matches the requested
      action
    - the ceremony_attestation_hash matches the canonical
      meta-trust-root key's attestation.
```

#### D-3.2. Witness-signature scope

A single witness signature authorises a single (artifact_hash,
intended_action) pair within a 24-hour validity window. The
signature cannot be re-used for a different artifact or a different
action. Re-use detection is enforced by policy-engine via the
`(artifact_hash, intended_action, signature)` triple being recorded
in audit-chain and refused on second presentation.

The 24-hour window is sized to allow standard CI propagation +
canary-deploy timing while limiting replay opportunity. Witness
signatures cannot be issued more than 24 hours in advance.

#### D-3.3. Ceremony cadence + latency budget

| Cadence class | Use case | Median latency budget | Hard ceiling |
|---|---|---|---|
| Routine | Baseline workflow version promotion; baseline Cedar fragment publication | 4h | 72h |
| Expedited | Substrate-deploy of a security-critical fix; meta-permit fragment update | 1h | 8h |
| Emergency | Kill-switch fragment activation (per ADR-0295 forthcoming) | 30 min | 2h |

The routine cadence is the default and the expected operational
mode. Expedited and emergency cadences are exception modes; the
attestor admits at most 3 expedited and 1 emergency request per
calendar week before declining further requests and emitting a
SEV-2 alert (matches §D-7 rate-limit guard).

#### D-3.4. Secure-video-bridge ceremony profile

Routine ceremonies may use a secure-video-bridge instead of in-
person attendance, **only if** all of the following hold:

- Every shard holder uses a hardware-attested device (Apple Secure
  Enclave / Android StrongBox / TPM-backed Windows Hello key) with
  remote-attestation proof to the ceremony HSM.
- The bridge runs over MLS-encrypted multi-party transport (per
  messenger-e2e-encryption-mls.md), not a third-party
  video-conference vendor.
- The Notary and Recording Operator are physically present in the
  ceremony facility (≥2 of the 17 participants are always in-person
  to maintain the chain of physical custody).
- Each remote holder physically inserts the smartcard into a
  vendor-attested smartcard reader connected to their hardware-
  attested device; the smartcard's reader-attestation chain
  evaluates against the ceremony HSM's verifier.

Expedited and emergency ceremonies always permit secure-video-bridge
because their hard ceilings (8h and 2h) do not accommodate in-person
travel for geographically-distributed holders.

### D-4. Cedar fragment amendment — replacing the circular predicate

The amended ADR-0247 §D-8 Cedar fragment becomes:

```cedar
// microservices/policy-engine/fragments/baseline/oyatie-self-modification-permits.cedar
// SCOPE: baseline
// SIGNED BY: org-baseline-key (intermediate, chained to org root per ADR-0243 §D-5)
// ATTESTED BY: oyatie.foundry.meta-trust-root (per ADR-0293)
// VERSION: v2 (replaces v1; v1 carried the circular predicate; supersedes_at: <publication-time>)

permit (
  principal in Tenant::"oyatie".sub_scopes("foundry"),
  action in [
    Workflow::Action::PublishWorkflowVersion,
    Workflow::Action::ActivateWorkflowVersion,
    Cedar::Action::PublishFragment,
    Cedar::Action::ActivateFragment,
    Substrate::Action::DeploySubstrateVersion,
    Substrate::Action::RollbackSubstrateVersion
  ],
  resource
)
when {
  // Either ≥2 human approvers (per ADR-0247 D-8 original safety branch)
  principal.is_human_approval_present(min_approvers: 2)
  ||
  // OR baseline-signed-workflow PLUS independent meta-trust-root witness
  (
    principal.is_automated_with_baseline_signed_workflow
    && principal.is_attested_by_meta_trust_root
    && context.meta_trust_root_witness.is_valid
    && context.meta_trust_root_witness.artifact_hash == resource.artifact_hash
    && context.meta_trust_root_witness.intended_action == action
    && context.meta_trust_root_witness.signed_at + duration("24h") > context.now
  )
};

// Meta-permit: modifying THIS fragment requires the strongest gate
forbid (
  principal,
  action == Cedar::Action::PublishFragment,
  resource is CedarFragment
)
when {
  resource.fragment_id == "baseline/oyatie-self-modification-permits.cedar"
}
unless {
  principal.is_human_approval_present(min_approvers: 3)
  && principal.is_council_security_approver_present
  && principal.is_council_architecture_approver_present
  && principal.is_signed_with_org_root_key_intermediate
  && principal.is_attested_by_meta_trust_root_at_ceremony_grade
};
```

Key changes from v1 to v2:

1. The autonomous path no longer evaluates true on
   `is_automated_with_baseline_signed_workflow` alone — it now
   requires `is_attested_by_meta_trust_root` as a conjunction. The
   single-key-compromise exploit is closed.
2. The witness signature's `artifact_hash`, `intended_action`, and
   `signed_at` are bound to the evaluation context (not merely
   verified as "present") — preventing signature replay across
   different artifacts.
3. The meta-permit clause requires ceremony-grade attestation for
   modifications of the self-modification permits fragment itself.
   Ceremony-grade attestation is a `WitnessSignature` issued at a
   ceremony with **all 9** Shamir shard holders present (not the
   5-of-9 threshold), recorded as `ceremony_grade: full-quorum`.

### D-5. Server-side attribute resolution for meta-trust-root context

Per F5-243-03 (HIGH) — Cedar context attribute injection across trust
boundary — the policy-engine MUST resolve the
`context.meta_trust_root_witness` attributes **server-side**, not
trust them from the caller's EvaluationRequest.

Concretely: when the caller (`oyatie.foundry.workflow-publisher`)
submits an EvaluationRequest, it carries only the
`witness_signature_id` (a Uuid). policy-engine looks up the witness
record from `microservices/cedar-fragment-registry/witness-signatures`
table (which is itself signed and Merkle-sealed). The caller cannot
falsify the witness signature attributes because the caller never
provides them.

```sql
-- microservices/cedar-fragment-registry/migrations/0042_witness_signatures.sql
CREATE TABLE witness_signatures (
    witness_signature_id    UUID PRIMARY KEY,
    artifact_hash           BYTEA NOT NULL,
    intended_action         TEXT NOT NULL CHECK (intended_action LIKE '%::Action::%'),
    cedar_resource_id       TEXT,
    signed_at               TIMESTAMPTZ NOT NULL,
    validity_window_end     TIMESTAMPTZ NOT NULL,
    ceremony_attestation_hash BYTEA NOT NULL REFERENCES meta_trust_root_ceremonies(attestation_hash),
    ceremony_grade          TEXT NOT NULL CHECK (ceremony_grade IN ('routine', 'expedited', 'emergency', 'full-quorum')),
    signer_pubkey_fingerprint BYTEA NOT NULL,
    signature               BYTEA NOT NULL,  -- Ed25519, 64 bytes
    consumed_at             TIMESTAMPTZ,     -- non-null once redeemed; redemption is single-shot per witness
    audit_emission_hash     BYTEA NOT NULL,  -- foreign key into audit-chain Merkle leaf
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    tenant_id               TEXT NOT NULL DEFAULT 'oyatie'
        CHECK (tenant_id = 'oyatie'),         -- only oyatie can hold witness signatures
    CONSTRAINT validity_window_positive CHECK (validity_window_end > signed_at),
    CONSTRAINT validity_window_bounded  CHECK (validity_window_end <= signed_at + INTERVAL '24 hours')
);

-- Single-shot redemption: once consumed_at is set, the row cannot be
-- presented again for evaluation.
CREATE UNIQUE INDEX witness_signatures_consumed_at_idx
    ON witness_signatures (witness_signature_id, consumed_at);

-- Detect replay attempts (same artifact_hash + intended_action signed
-- after a prior signature has already been consumed)
CREATE INDEX witness_signatures_artifact_action_idx
    ON witness_signatures (artifact_hash, intended_action, signed_at DESC);

CREATE TABLE meta_trust_root_ceremonies (
    attestation_hash        BYTEA PRIMARY KEY,
    ceremony_type           TEXT NOT NULL CHECK (ceremony_type IN (
        'initial-generation', 'yearly-rotation', 'duress-revocation',
        'shard-rotation', 'full-quorum-meta-permit-update'
    )),
    ceremony_date           DATE NOT NULL,
    ceremony_facility       TEXT NOT NULL,
    notary_apostille_id     TEXT NOT NULL,
    qtsp_attestation_id     TEXT NOT NULL,
    hsm_brand_model         TEXT NOT NULL,
    hsm_serial_numbers      TEXT[] NOT NULL,
    fips_certificate_number TEXT NOT NULL,
    shard_count_n           SMALLINT NOT NULL CHECK (shard_count_n = 9),
    shard_threshold_m       SMALLINT NOT NULL CHECK (shard_threshold_m = 5),
    jurisdictions           TEXT[] NOT NULL CHECK (cardinality(jurisdictions) >= 3),
    public_key              BYTEA NOT NULL,
    public_key_fingerprint  BYTEA NOT NULL UNIQUE,
    ceremony_video_seal     BYTEA NOT NULL,  -- SHA3-256 of sealed-recording bag
    superseded_by           BYTEA REFERENCES meta_trust_root_ceremonies(attestation_hash),
    valid_from              TIMESTAMPTZ NOT NULL,
    valid_until             TIMESTAMPTZ NOT NULL,
    CONSTRAINT key_lifetime_max_1_year CHECK (valid_until <= valid_from + INTERVAL '1 year' + INTERVAL '14 days')
);

CREATE TABLE meta_trust_root_shard_holders (
    holder_id               UUID PRIMARY KEY,
    attestation_hash        BYTEA NOT NULL REFERENCES meta_trust_root_ceremonies(attestation_hash),
    shard_index             SMALLINT NOT NULL CHECK (shard_index BETWEEN 1 AND 9),
    jurisdiction_iso_3166   TEXT NOT NULL,
    council_role            TEXT,             -- nullable; non-null only for the 3 council members
    holder_agreement_hash   BYTEA NOT NULL,   -- SHA3-256 of signed agreement document
    smartcard_serial        TEXT NOT NULL,
    smartcard_fips_cert     TEXT NOT NULL,
    holder_status           TEXT NOT NULL CHECK (holder_status IN (
        'active', 'rotated-out', 'duress-revoked', 'compromise-suspected', 'unavailable'
    )),
    last_response_at        TIMESTAMPTZ,
    UNIQUE (attestation_hash, shard_index),
    UNIQUE (attestation_hash, smartcard_serial)
);

CREATE TABLE meta_trust_root_duress_revocations (
    revocation_id           UUID PRIMARY KEY,
    triggered_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    triggered_by_holder_id  UUID NOT NULL REFERENCES meta_trust_root_shard_holders(holder_id),
    attestation_hash        BYTEA NOT NULL REFERENCES meta_trust_root_ceremonies(attestation_hash),
    duress_reason           TEXT NOT NULL CHECK (duress_reason IN (
        'coercion-imminent', 'shard-seal-broken', 'jurisdiction-change',
        'sentinel-non-response', 'discretionary'
    )),
    freeze_window_end       TIMESTAMPTZ NOT NULL,  -- triggered_at + 24h default
    resolved_at             TIMESTAMPTZ,
    resolution_ceremony_id  BYTEA REFERENCES meta_trust_root_ceremonies(attestation_hash),
    audit_emission_hash     BYTEA NOT NULL
);
```

### D-6. Rotation cadence + attestation chain

The meta-trust-root key rotates on three triggers:

1. **Scheduled annual rotation.** The key is rotated every 12 months
   ±14 days. The yearly rotation ceremony follows the §D-2 procedure
   identically; the new key's attestation chain references the prior
   key's `attestation_hash` via `superseded_by`.
2. **Shard-holder turnover.** Any time a shard holder's status
   transitions to `rotated-out`, `compromise-suspected`, or
   `unavailable`, a shard-rotation ceremony is required within 30
   days. Shard rotation regenerates the entire 9-shard set against
   the same underlying key (using a fresh polynomial); the
   underlying key is unchanged.
3. **Duress revocation** (per §D-7). A duress revocation forces an
   immediate key rotation within 72 hours.

The attestation chain across rotations forms a linked list:

```
ceremony_initial → ceremony_y1_rotation → ceremony_y2_rotation → ...
       (Jan 2027)        (Jan 2028)            (Jan 2029)
```

Each `WitnessSignature` row references its specific
`ceremony_attestation_hash`, so witness signatures issued before a
rotation remain verifiable forever — they prove witness occurrence
under the then-current key, and the policy-engine's verification
chain walks `superseded_by` to confirm continuity.

### D-7. Duress revocation + kill-switch

Any single shard holder MAY unilaterally trigger a duress revocation
without requiring consensus. The duress-revocation primitive is
modelled on the Mozilla CA Compromise Reporting flow §4.9 and on
Signal Protocol's "panic-PIN" pattern.

#### D-7.1. Triggering

A shard holder triggers duress revocation by sending a signed
revocation request from their personal attestor-client. The request
carries:

- `revocation_id` (client-generated UUID)
- `triggered_by_holder_id`
- `duress_reason` (one of the enum values in the SQL above)
- A signature using the holder's smartcard private-key-fragment
  (the same private key that signs ceremony attendance)
- Optionally, a free-form text (≤ 500 chars) for context

The signature is verifiable by the policy-engine against the
holder's `holder_id` row + smartcard's public attestation. The
threshold to **trigger** is 1-of-9 (any single holder); the
threshold to **resolve** is 5-of-9 (the standard ceremony threshold).

#### D-7.2. Effect

Upon receipt of a valid duress revocation request, the platform
enters **trust-chain freeze**:

1. policy-engine immediately publishes a `kill-switch` Cedar fragment
   to the baseline scope that **forbids** all actions in the
   meta-trust-root action allowlist regardless of any witness
   signature. The fragment's effect lasts for 24 hours (default) or
   until a resolution ceremony (per §D-6).
2. All in-flight WitnessSignatures' `validity_window_end` is
   truncated to `triggered_at`. Any further attempt to consume them
   is denied.
3. `oyatie.foundry.workflow-publisher`, `oyatie.foundry.adr-drafter`,
   `oyatie.foundry.substrate-upgrader`, and other principals in the
   `oyatie.foundry.*` family whose actions are gated by the
   meta-trust-root predicate are effectively frozen for the duration.
4. Manual operations remain available via the 2-human-approver
   branch of §D-4 — engineers can still respond to incidents under
   council-security + council-architecture approval.
5. SEV-1 alert paged to council-security; SEV-2 alert paged to
   council-architecture, council-privacy, council-legal,
   ops-sre-reliability. The incident enters the incident-response
   workflow per `docs/runbooks/meta-trust-root-duress-recovery.md`
   (cross-references Slice 1 output).

#### D-7.3. Resolution

The freeze resolves through one of three paths:

1. **False-positive resolution.** If the triggering holder later
   declares the trigger was in error (e.g., a tamper-evident seal
   was inspected and intact), council-security may declare false-
   positive within 24 hours. The kill-switch fragment is revoked;
   normal operations resume. The incident is recorded but no key
   rotation occurs.
2. **Confirmed compromise resolution.** If the compromise is
   confirmed, the resolution ceremony is convened immediately. The
   existing key is destroyed (HSM zeroize ceremony per FIPS 140-3
   §4.7.5); a new key is generated under a fresh §D-2 ceremony; all
   nine shard holders attend (full-quorum requirement for the
   ceremony following a confirmed compromise). The old key's
   `valid_until` is set to the destruction timestamp; the new key's
   `valid_from` follows.
3. **Indeterminate resolution.** If 72 hours elapse without
   declaration of false-positive or confirmed-compromise, the
   freeze is escalated to platform-wide self-modification freeze.
   `oyatie.foundry.*` principals' self-modification actions remain
   denied. The platform continues to serve customer traffic; only
   self-modification is paused. An out-of-band council escalation
   to all council chairs is triggered with mandatory 24h response.

#### D-7.4. Rate-limit guard

To prevent denial-of-service via spurious duress triggers:

- Each shard holder may trigger duress revocation at most twice per
  rolling 365-day window; the third trigger requires co-attestation
  by council-security.
- The platform-wide rate ceiling is 4 duress triggers per rolling
  90-day window before council-architecture must convene an
  emergency review.
- A duress trigger that follows within 7 days of a confirmed
  false-positive from the same holder is rate-limited (the trigger
  is recorded but the kill-switch fragment is published in
  shadow-mode rather than enforcing-mode until council-security
  ratifies).

## Consequences

### Positive

1. **Single-key compromise of `oyatie.foundry.workflow-publisher`
   no longer compromises self-modification.** The exploit path
   identified by F5-247-01 is closed; the predicate cited as
   circular is replaced by a conjunction that requires independent
   witness.
2. **Nation-state coercion against operational keys is bounded.**
   Even compromise of all four `council-*` intermediate keys would
   require ALSO the multi-jurisdiction 5-of-9 ceremony to mount a
   self-modification attack.
3. **Duress revocation provides immediate response capability.** Any
   one shard holder can freeze self-modification with one signed
   request — matching the panic-PIN posture that hardware-token
   industry standards now widely adopt.
4. **The Shamir parameter expansion to 5-of-9 across ≥3 jurisdictions
   matches industry root-CA precedent.** ICANN, DigiCert, Let's
   Encrypt, AWS, GCP, Azure, Mozilla CA, and Sectigo all maintain
   geographically-distributed quorum for offline root keys.
5. **The witness mechanism is composable with the rest of the
   keystone bundle.** ADR-0247 §D-8 amendment, ADR-0243 §D-5
   amendment, and ADR-0246's fragment validation lane all reference
   this ADR's witness primitive identically.

### Negative

1. **Operational latency.** Routine self-modification cycles now
   incur a ≤4h median (≤72h ceiling) ceremony latency that did not
   exist in the v1 ADR-0247 §D-8 fragment. The autonomous-masterplan
   throughput is correspondingly capped at approximately
   24h/4h = 6 self-modification ceremonies per day per workflow class
   for routine cadence; this is acceptable for autonomous masterplan
   work whose unit-of-progress is "one PR per workflow per few hours"
   rather than "many self-modifications per minute."
2. **Ceremony cost.** Recruiting and maintaining 9 multi-
   jurisdiction shard holders, plus 3 internal witnesses, 2 external
   witnesses, ceremony facility, notary, recording operator, and
   FIPS 140-3 L3 HSM hardware procurement, costs an estimated
   $120K-180K initial setup + $40K-60K per yearly rotation
   (per industry benchmarks from comparable root-CA operators).
3. **Holder availability risk.** The ≤72h response window assumes
   shard holders are reachable. The shard-holder agreement
   obligates response, but jurisdictional travel restrictions
   (pandemics, sanctions) could in principle prevent quorum. The
   shard-rotation primitive (§D-6) handles single-holder
   unavailability; multi-holder unavailability requires expedited
   shard rotation (added to docs/runbooks/meta-trust-root-shard-
   rotation.md, Slice 1 output).
4. **Smartcard supply-chain risk.** The smartcards (Gemalto IDPrime
   940) are themselves a procurement chain that could be compromised.
   The mitigation is: smartcards are procured from at least two
   distinct vendors with independent supply chains (e.g., half
   Gemalto, half Yubico), and shard holders verify their smartcard's
   FIPS certificate authenticity at receipt.
5. **The witness mechanism does not protect against the org-root
   compromise.** A compromised org root key could re-issue both the
   meta-trust-root certification AND the workflow-publisher
   certification — defeating the conjunction. The org-root remains
   the deepest trust anchor; its protection is the §D-2 ceremony
   procedure inherited at the org-root scope plus its 5-year
   rotation cadence.

### Neutral

1. **The witness mechanism is invisible to customer tenants.**
   Customer-tenant principals (`tenant-<id>.*`) never interact with
   the meta-trust-root key; the witness primitive is scoped to
   `oyatie.foundry.*` self-modification only.
2. **The mechanism composes cleanly with ADR-0294's fragment soak
   window.** Even when a meta-trust-root witness signature
   authorises a fragment publication, the fragment still enters
   the §ADR-0294 soak + anomaly-rollback window before broad
   enforcement. The two ADRs are independent and stack.
3. **The mechanism composes cleanly with ADR-0295's bootstrap
   kill-switch.** Bootstrap-stage operations may invoke witness
   signatures or human approval; the bootstrap kill-switch's
   T+8h cutoff overrides both, providing a final fallback.

## Detailed Mechanics

### D-1 expanded — principal definition under BNF v4.1

The principal name `oyatie.foundry.meta-trust-root` conforms to:

```
principal      ::= tenant "." sub-scope
tenant         ::= "oyatie"                          // reserved-namespace per ADR-0242
sub-scope      ::= sub-scope-segment ( "." sub-scope-segment )*
sub-scope-segment ::= alpha-lower ( alpha-lower | digit | "-" )*
                      // BNF v4.1: leading char alpha-lower; no underscores;
                      // hyphens permitted intra-segment
```

Tokenization:
- `oyatie` — reserved-namespace tenant
- `foundry` — first sub-scope segment (existing per ADR-0242 §D-2)
- `meta-trust-root` — second sub-scope segment (new; kebab-case;
  three words separated by hyphens; no underscores)

Layer-enum classification (ADR-0105 13-layer enum):
- Layer: `trust-anchor`
- Justification: the principal's key signs witness attestations that
  gate the platform's self-modification capability. Compromise of
  the key breaks the platform's chain of trust for autonomous
  self-modification. This is the precise role of the `trust-anchor`
  layer per ADR-0105.

The companion `oyatie.foundry.meta-trust-root-attestor` principal:
- Layer: `automation`
- Justification: the attestor is a day-to-day automation surface
  that mediates between offline shard holders and the substrate; it
  does not itself hold key material; it does not itself decide; it
  is plumbing. The `automation` layer is the correct fit.

### D-3 expanded — sequence diagram with failure modes

```
Happy path:
  workflow-publisher → request_witness_signature → meta-trust-root-attestor
  meta-trust-root-attestor → publish CeremonyRequest → ceremony-queue (Kafka)
  Shard holders (≥5 of 9) → fetch CeremonyRequest → independent-hash-verify
  Shard holders → attend ceremony (in-person or secure-video-bridge)
  HSM → reconstruct key → sign artifact → emit WitnessSignature
  meta-trust-root-attestor → fetch WitnessSignature → return to workflow-publisher
  workflow-publisher → embed witness → emit Cedar evaluation request
  policy-engine → server-side resolve witness → emit Permit
  workflow-publisher → execute action

Failure mode 1: shard holders insufficient quorum within ≤72h ceiling
  meta-trust-root-attestor → emit TimeoutNotification → workflow-publisher
  workflow-publisher → cancel artifact promotion → log incident
  Audit row: `WitnessTimeoutEvidence(artifact_hash, holders_responded[])`

Failure mode 2: independent-hash-verify fails for a shard holder
  Shard holder → emit MismatchNotification → meta-trust-root-attestor
  meta-trust-root-attestor → escalate to council-security → publish
    CeremonyAbortNotice
  workflow-publisher → cancel artifact promotion
  Triggers an out-of-band investigation of the workflow-publisher's
    artifact-signing chain (this is the F5-247-01 exploit signature)

Failure mode 3: HSM verification of CeremonyRequest signature fails
  HSM emits AttestationFailureEvidence (HSM-internal)
  Audit row emitted; ceremony aborts
  Triggers investigation: either CeremonyRequest was tampered or the
    workflow-publisher's key has been compromised

Failure mode 4: WitnessSignature replay attempt
  policy-engine attempts to redeem witness whose consumed_at is non-null
  policy-engine emits ForbidDecision with reason "witness-replay"
  Audit row: `WitnessReplayDetected(witness_signature_id, attempted_at)`
  Triggers investigation of the workflow-publisher's witness-tracking
    state

Failure mode 5: witness validity window expired
  policy-engine evaluates witness signed_at + 24h < context.now
  policy-engine emits ForbidDecision with reason "witness-expired"
  workflow-publisher must request a fresh witness signature

Failure mode 6: duress revocation in progress
  policy-engine sees active duress kill-switch fragment
  All meta-trust-root-gated actions denied regardless of witness
  workflow-publisher receives WitnessFrozenError; queues request for
    post-resolution retry
```

### D-7 expanded — duress revocation as a Cedar primitive

The duress-revocation fragment is itself a Cedar fragment published
to the baseline scope:

```cedar
// microservices/policy-engine/fragments/baseline/meta-trust-root-duress-freeze.cedar
// SCOPE: baseline
// SIGNED BY: kill-switch-publisher-key (a key dedicated to one-shot kill-switch fragment activation)
// LIFETIME: triggered_at .. triggered_at + 24h (default)
// ACTIVATION: triggered by a single MetaTrustRootDuressRevocation row landing in cedar-fragment-registry

forbid (
  principal in Tenant::"oyatie".sub_scopes("foundry"),
  action in [
    MetaTrustRoot::Action::IssueWitnessSignature,
    Workflow::Action::PublishWorkflowVersion,
    Workflow::Action::ActivateWorkflowVersion,
    Cedar::Action::PublishFragment,
    Cedar::Action::ActivateFragment,
    Substrate::Action::DeploySubstrateVersion,
    Substrate::Action::RollbackSubstrateVersion
  ],
  resource
)
when {
  context.meta_trust_root_freeze.is_active
  && context.meta_trust_root_freeze.triggered_at + duration("24h") > context.now
};

// Carve-out: human emergency-override remains via the 2-human-approver
// branch of the self-modification fragment §D-4. The duress freeze does
// not apply to that branch — engineers can still respond to incidents.
```

### D-7 expanded — kill-switch publication flow

```
Shard holder triggers duress
  → revocation request signed by holder's smartcard fragment key
  → meta-trust-root-attestor verifies signature
  → meta-trust-root-attestor writes MetaTrustRootDuressRevocation row
  → kill-switch-publisher (a watcher on the duress-revocation table)
    publishes meta-trust-root-duress-freeze.cedar to baseline scope
  → policy-engine cell evaluators hot-reload the fragment per
    ADR-0243 §D-10 + ADR-0294 soak override (emergency cadence
    bypasses the 60s soak per ADR-0294 §D-3)
  → All cells enforce the freeze within 5 seconds

Resolution path triggers fragment revocation
  → resolution ceremony emits MetaTrustRootResolutionEvidence row
  → kill-switch-publisher revokes the freeze fragment
  → policy-engine cell evaluators hot-remove the fragment
  → Normal operations resume
```

## Implementation Footprint

### Microservice scope

The implementation footprint spans five existing substrates and one
new crate:

| Microservice | Change | Effort |
|---|---|---|
| `microservices/policy-engine/` | Add server-side witness resolution; add `is_attested_by_meta_trust_root` Cedar predicate; add witness-signature SQL tables; add duress-freeze hot-reload handler | ≈ 4 weeks |
| `microservices/audit-chain/` | Add `audit.meta-trust-root` stream with Merkle-sealed-per-event evidence level | ≈ 1 week |
| `microservices/cedar-fragment-registry/` | Add witness signature storage; add duress revocation storage; add ceremony attestation storage | ≈ 2 weeks |
| `microservices/identity/` | Register `oyatie.foundry.meta-trust-root` and `oyatie.foundry.meta-trust-root-attestor` principals | ≈ 1 week |
| `microservices/workflow-engine/` | Add witness-signature request + embed primitive; add witness-validity check in workflow version-promotion gates | ≈ 2 weeks |
| `crates/oya-shared-meta-trust-root/` (new) | Shared crate exposing witness primitives + Cedar predicate; canonical-base + localization-overlay shape | ≈ 3 weeks |

Total: ≈ 13 weeks of engineering effort. Parallelizable across five
crews; calendar time ≈ 4 weeks.

### Hardware + ceremony scope

| Item | Quantity | Lead time |
|---|---|---|
| FIPS 140-3 L3 HSMs (primary + backup) | 2 | 6-8 weeks procurement |
| Tamper-evident smartcards | 18 (9 primary + 9 backup) | 4 weeks procurement |
| Ceremony facility (initial + yearly) | 1 per ceremony | 4 weeks scheduling |
| Notary services | 1 per ceremony | 2 weeks scheduling |
| QTSP attestation | 1 per ceremony | 4-6 weeks contracting |
| Recording rig + sealed-recording storage | 1 per ceremony | 2 weeks |

Total ceremony lead time from this ADR's promotion: 8-12 weeks
calendar minimum. The keystone-bundle promotion gates allow the
ceremony work to proceed in parallel with code implementation.

### CI lane scope

| CI lane | Behavior |
|---|---|
| `oya-check-meta-trust-root-attestation` | Scans Cedar fragments under `baseline/`, `policy-engine/fragments/`, and the `oyatie.foundry.*` workflow library for any trust-chain-touching content; verifies each has either a current meta-trust-root witness or a 2-human-approver attestation; emits findings for missing |
| `oya-check-meta-trust-root-key-ceremony-evidence` | Verifies that the canonical meta-trust-root key in `meta_trust_root_ceremonies` table has a current (not-superseded, not-expired) row; verifies the ceremony attestation hash matches a known-good ledger |
| `oya-check-meta-trust-root-shamir-distribution` | Verifies that the active key's `meta_trust_root_shard_holders` rows have ≥3 distinct `jurisdiction_iso_3166` values; emits BLOCKER finding if not |
| `oya-check-meta-trust-root-rotation-cadence` | Verifies the active key's `valid_from` is within the last 12 + 1 month grace period; emits WARNING at 11 months; BLOCKER at 12 months + 14 days |
| `oya-check-self-modification-witness-present` | Scans recent audit-chain rows for `Workflow::Action::PublishWorkflowVersion` etc. under the autonomous-masterplan path; verifies each carries a `witness_signature_id`; emits findings for absent |

## Migration

### Stage 0 — Procurement + recruitment (T+0 to T+8w)

| Step | Action | Owner |
|---|---|---|
| 0.1 | Council-security drafts the shard-holder eligibility criteria + recruits 9 candidate Cryptographic Officers; council-architecture reviews and accepts | council-security |
| 0.2 | HSM procurement initiated; Thales OR Entrust selected based on lead time + jurisdictional certification | council-security + ops-sre-reliability |
| 0.3 | Smartcards procured from two distinct vendors (Gemalto + Yubico) | council-security |
| 0.4 | QTSP contracting (e.g., GlobalSign, DigiCert root-services); notary identified | council-legal |
| 0.5 | Ceremony facility booked (typically Tier-2 cloud-region adjacent secure room or a hardened co-location) | ops-sre-reliability |

### Stage 1 — Code implementation in parallel (T+0 to T+4w)

| Step | Action | Owner |
|---|---|---|
| 1.1 | `oya-shared-meta-trust-root` crate scaffolded; witness primitives + Cedar predicate authored | axis-identity |
| 1.2 | `microservices/policy-engine` migration `0042_witness_signatures.sql` and `0043_meta_trust_root_ceremonies.sql` authored | axis-policy-engine |
| 1.3 | `microservices/cedar-fragment-registry` witness-signature CRUD authored | axis-policy-engine |
| 1.4 | `microservices/audit-chain` `audit.meta-trust-root` stream provisioned | axis-audit-chain |
| 1.5 | `microservices/workflow-engine` witness-request + embed primitive authored | axis-workflow-engine |
| 1.6 | `microservices/identity` registers the new principals | axis-identity |
| 1.7 | Integration tests in `microservices/policy-engine/tests/witness_e2e.rs` cover happy path + 6 failure modes from §D-3 | axis-policy-engine |

### Stage 2 — Ceremony rehearsal (T+8w to T+10w)

| Step | Action | Owner |
|---|---|---|
| 2.1 | Test-mode ceremony executed in `dev-tools-cell-staging` with a non-production HSM and test smartcards | council-security |
| 2.2 | Witness signature emitted by the test ceremony; verified end-to-end through staging policy-engine | council-security + axis-policy-engine |
| 2.3 | Duress-revocation rehearsal executed: a test shard holder triggers duress; freeze fragment publishes; staging policy-engine denies a test self-modification action; resolution ceremony resolves the freeze | council-security |
| 2.4 | Rehearsal findings + improvements folded into the production ceremony procedure | council-security |

### Stage 3 — Production ceremony (T+10w to T+12w)

| Step | Action | Owner |
|---|---|---|
| 3.1 | Production ceremony executed per §D-2 with all 17 participants | council-security + ceremony master |
| 3.2 | Production meta-trust-root public key + attestation chain ingested into `meta_trust_root_ceremonies` table | axis-policy-engine |
| 3.3 | First production witness signature issued for the canonical
`oyatie.foundry.adr-drafter` v2 workflow promotion candidate | axis-workflow-engine + meta-trust-root attestor |
| 3.4 | ADR-0247 §D-8 fragment v2 published to baseline scope (the §D-4 fragment above) | axis-policy-engine + council-security |
| 3.5 | All five CI lanes flip from advisory to BLOCKER | ops-sre-reliability |

### Stage 4 — Autonomous masterplan resumption (T+12w+)

Once the production ceremony has succeeded and CI lanes are green,
the autonomous-masterplan workflows resume; every self-modification
cycle henceforth includes a meta-trust-root witness signature in its
audit emission.

The ADR-0247 §D-8 fragment v1 (carrying the circular predicate) is
**revoked**, not merely superseded — its `revoked_at` is set to the
publication time of v2, and any attempted re-evaluation against v1
in any cell evaluator emits a `ForbidDecision` with reason
`fragment-revoked-by-supersession`.

## References

### Primary

- `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` §5.1 +
  §5.5 — authority for this ADR's existence and the resolution
  of F5-247-01 + F5-243-02 Shamir-expansion arm.
- `evidence/debate/keystone-bundle-2026-05-20-F5-security-r1.json` —
  the F5 verdict containing F5-247-01 (CRITICAL) and F5-243-02
  (HIGH).

### Related ADRs

- ADR-0247 (Self-Hosting / Self-Modification Doctrine) — §D-8
  Cedar fragment v1 carried the circular predicate; this ADR
  amends it.
- ADR-0243 (Cedar as Universal Gate) — §D-5 chain of trust gains
  the meta-trust-root anchor.
- ADR-0246 (Policy Engine Substrate Promotion) — fragment-validation
  lane gains the `oya-check-meta-trust-root-attestation` lane.
- ADR-0242 (`oyatie` is a tenant doctrine) — defines the reserved-
  namespace under which the new principals are registered.
- ADR-0105 (Thirteen-layer canonical enum) — `trust-anchor` layer
  classifies the meta-trust-root principal.
- ADR-0244 (Tenant as universal scoping primitive) — defines the
  scoping primitive under which the new principals live.

### Industry references

- **ICANN KSK ceremony documentation.** IANA-DNSSEC-Practice-
  Statement §5.2; quarterly ceremony procedures; 7-of-7 operator
  card threshold + 5-of-7 safe combination.
- **Mozilla CA Certificate Policy §6.2.7.** Offline root key
  storage, m-of-n procedural controls, formal ceremony witnesses.
- **AWS re:Invent SEC403 (2023) "How AWS Builds Trust" and AWS KMS
  Custom Key Store documentation.** Multi-region key shard
  distribution; ceremony video-recording; FIPS 140-3 L3 HSM tier.
- **GCP Certificate Authority Service offline-root + issuing-CA
  topology.** GCS Cloud KMS asymmetric signing precedent.
- **Azure Key Vault Managed HSM + Microsoft Sovereign Cloud key
  ceremony documentation (2024).** Geographic distribution across
  non-FVEY jurisdictions.
- **Let's Encrypt root ceremony 2015 + 2024 rollover.** Public
  cryptographic ceremony with audit recording; precedent for
  rotation cadence.
- **DigiCert root ceremony documentation.** Annual rotation cadence
  precedent; multi-jurisdiction shard distribution.
- **Stripe internal key ceremony documentation (Brandur Leach
  blog posts 2019-2022).** PCI-compliant key management; multi-
  region quorum.
- **Cloudflare Geo Key Manager (Cloudflare blog, 2017) +
  Distributed Keyless SSL.** Multi-jurisdiction key holding with
  cryptographic threshold protocols.

### Cryptographic references

- Shamir, A. (1979). "How to Share a Secret." Communications of the
  ACM, 22(11): 612-613.
- FIPS 140-3 (NIST, 2019) — Security Requirements for Cryptographic
  Modules; Level 3 + Level 4 procedural controls.
- FIPS 203 (NIST, 2024) — Module-Lattice-Based Key-Encapsulation
  Mechanism Standard (ML-KEM); future PQ ceremony substitution
  candidate.
- RFC 9162 — Certificate Transparency Version 2.0 (public-log
  precedent for ceremony attestation publication).
- NIST 800-88 Rev. 1 — Guidelines for Media Sanitization; Purge
  procedure used for air-gap workstation destruction.
- Edward Felten et al. (2020). "Practical Threshold Signatures."
  Springer LNCS; threshold-signature literature underpinning the
  Shamir → Ed25519 reconstruction flow.

### Slice cross-references

- **Slice 1 (runbooks):**
  `docs/runbooks/meta-trust-root-key-ceremony.md`,
  `docs/runbooks/meta-trust-root-duress-recovery.md`,
  `docs/runbooks/meta-trust-root-shard-rotation.md`,
  `docs/runbooks/meta-trust-root-yearly-rotation.md` are required
  by this ADR's CI lanes; their authoring is in Slice 1 scope.
- **Slice 3 (ADR-0246 amendment):** The
  `oya-check-meta-trust-root-attestation` CI lane is added to
  ADR-0246's fragment-validation lane catalogue; the actual
  amendment to ADR-0246 is in Slice 3 scope.
- **Slice 4 (naming justifications):** Per
  `feedback_naming_justification`, the four new names
  (`oyatie.foundry.meta-trust-root`, `oyatie.foundry.meta-trust-
  root-attestor`, `oya-shared-meta-trust-root`,
  `oya-check-meta-trust-root-attestation`) are justified in the
  front-matter `naming_justifications:` block of this ADR.

### Specifications

- `/specs/meta-trust-root-key-ceremony.json` (new; authored as part
  of this ADR's implementation) — canonical machine-readable record
  of the ceremony procedure, participant roles, and attestation
  chain.
- `/specs/bootstrap-tier-model.json` — updated to add the
  meta-trust-root key as a Tier-0 component alongside the org root
  key.
- `/specs/self-modification-cedar-fragment-schema.json` — updated
  to add the `attested_by_meta_trust_root` annotation field on
  trust-chain Cedar fragments.

### Memory references

- `feedback_oyatie_is_a_tenant_doctrine` — reserved-namespace under
  which the new principals are registered.
- `feedback_cedar_as_universal_gate` — Cedar evaluation is the
  enforcement primitive; the `is_attested_by_meta_trust_root`
  predicate is a new Cedar predicate per this ADR.
- `feedback_self_modification_doctrine` — original self-modification
  doctrine that ADR-0247 establishes; this ADR's amendments close
  the F5-247-01 finding against it.
- `feedback_no_silent_regression` — the v1 → v2 fragment transition
  is documented, ADR'd, and CI-enforced; the v1 fragment is
  explicitly revoked rather than allowed to silently coexist.
- `feedback_autonomous_implementation_artifacts` — the witness
  mechanism is designed to preserve autonomous masterplan execution
  while adding the independent-witness invariant.
- `feedback_naming_justification` — the four new names carry
  inline justification in this ADR's front matter.
- `feedback_byok_everywhere_credentials` — the meta-trust-root key
  is held only by oyatie's own ceremony participants; the substrate
  itself owns zero copies; matches key-custody-BYOK posture.

---

**End of ADR-0293.**
