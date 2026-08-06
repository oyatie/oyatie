---
id: ADR-0295
status: Accepted
date: 2026-05-20
owners:
  - council-architecture
  - council-security
  - council-privacy
  - council-legal
  - ops-sre-reliability
  - ops-compliance
  - axis-identity
  - axis-policy-engine
  - axis-audit-chain
  - axis-cell
supersedes: []
amends: []
requires_amendment_to:
  - ADR-0247-self-hosting-self-modification-doctrine.md (§D-5 Stage 1 gains SPIFFE-pinned runner identity, sigstore cosign-attested artifact requirement, ≤8h bootstrap budget, T+8h Cedar kill-switch fragment; §D-5 Stage 1.10 → Stage 2.0 transition gains out-of-band council-security manual hash verification; §D-2 sub-scope `oyatie.foundry.bootstrap-runner` added)
  - ADR-0243-cedar-as-universal-gate.md (§D-5 chain of trust gains bootstrap-runner intermediate identity; baseline scope gains `bootstrap-trust-roots-kill-switch.cedar` fragment)
superseded_by: []
related:
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0049-cross-region-replication-and-residency.md
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
  - ADR-0280-substrate-of-substrate-dependency-doctrine.md
  - ADR-0293-governance-meta-trust-root.md
  - ADR-0294-cedar-fragment-soak-anomaly-rollback.md
related_specs:
  - /specs/platform-architecture.json
  - /specs/microservices/policy-engine.json
  - /specs/microservices/identity.json
  - /specs/bootstrap-tier-model.json
  - /specs/bootstrap-runner-identity-protocol.json
  - /specs/bootstrap-kill-switch-fragment.json
related_memory:
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_cedar_as_universal_gate
  - feedback_no_silent_regression
  - feedback_self_modification_doctrine
  - feedback_quality_performance_scalability_bar
  - feedback_autonomous_implementation_artifacts
  - feedback_build_ahead_of_certification
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-foundational-doctrine
keystone_position: promotion-gate-fix-3-of-4
authority_for_existence: docs/architecture/keystone-bundle-2026-05-20-synthesis.md §5.3
closes_findings:
  - F5-247-02 (Bootstrap CI external-runner highest-value compromise window, CRITICAL)
  - TMG-02 (Compromised-CI-runner threat model absent)
  - ASC-04 (Tier-0 external dependencies under nation-state legal process; partial — bootstrap aspect)
naming_justifications:
  - name: oyatie.foundry.bootstrap-runner
    bnf_v4_1: tenant=`oyatie` (reserved-namespace) · sub_scope=`foundry.bootstrap-runner` (kebab-case, hyphenated, no underscores) · arity=3
    layer_enum_adr_0105: `trust-anchor`
    rationale: Stage-1 external CI runners impersonate this principal via SPIFFE workload identity; the principal has narrow ephemeral authority bounded by the bootstrap window; classified as `trust-anchor` because compromise during the bootstrap window propagates into the platform's lifetime trust chain
  - name: oyatie.foundry.bootstrap-ca
    bnf_v4_1: tenant=`oyatie` · sub_scope=`foundry.bootstrap-ca` · arity=3
    layer_enum_adr_0105: `trust-anchor`
    rationale: One-shot offline-rooted CA whose private key is destroyed after Stage-1 completes; issues SPIFFE certificates to bootstrap-runner principals; distinct from `oyatie.foundry.meta-trust-root` because its scope is the bootstrap window only
  - name: oyatie.foundry.bootstrap-kill-switch-publisher
    bnf_v4_1: tenant=`oyatie` · sub_scope=`foundry.bootstrap-kill-switch-publisher` · arity=3
    layer_enum_adr_0105: `automation`
    rationale: Per-region automation that publishes the kill-switch Cedar fragment at T+8h regardless of Stage-2 readiness; bounded authority; classified as `automation` because it does not hold trust-chain key material
  - name: oya-shared-bootstrap-runner-identity
    bnf_v4_1: shared-domain crate per `feedback_glossary_shared_not_platform`; kebab-case
    layer_enum_adr_0105: `shared`
    rationale: Shared crate exposing SPIFFE workload identity primitives, cosign attestation primitives, and the kill-switch fragment construction protocol
  - name: oya-check-bootstrap-spiffe-identity
    bnf_v4_1: gate-name convention `oya-check-<predicate>` per ADR-0212
    layer_enum_adr_0105: `gate`
    rationale: CI lane verifying every Stage-1 artifact carries a SPIFFE-issued + cosign-attested provenance
enforcement_status: advisory-until-bootstrap-runner-ca-ceremony-runs
enforced_by:
  - oya gate validate bootstrap-spiffe-identity
  - oya gate validate bootstrap-runner-ca-ceremony-evidence
  - oya gate validate bootstrap-artifact-cosign-attestation
  - oya gate validate bootstrap-budget-ceiling
  - oya gate validate bootstrap-kill-switch-fragment-present
  - oya gate validate bootstrap-runner-revocation-coverage
---

# ADR-0295: Bootstrap CI SPIFFE Identity + T+8h Kill-Switch

## Status

Proposed — 2026-05-20.

Promotion-gate fix **3 of 4** for the keystone bundle 2026-05-20
(`docs/architecture/keystone-bundle-2026-05-20-synthesis.md` §5.3).
This ADR closes F5-Security finding **F5-247-02** (CRITICAL,
bootstrap CI external-runner highest-value compromise window),
threat-model gap **TMG-02** (compromised-CI-runner threat model
absent), and the bootstrap aspect of **ASC-04** (Tier-0 external
dependencies). ADR-0247 cannot promote from `Proposed` to
`Accepted` until this ADR's mechanics are implemented and the
one-shot bootstrap CA ceremony runs.

Enforcement is `advisory-until-bootstrap-runner-ca-ceremony-runs`.
The CI lanes that enforce this ADR become BLOCKER once:

1. The one-shot offline-rooted bootstrap CA ceremony has executed
   per §D-2 and its destruction certificate is recorded.
2. The first SPIFFE-pinned bootstrap runner has issued a
   cosign-attested artifact verified end-to-end against the
   bootstrap CA's published root.
3. The kill-switch Cedar fragment `baseline/bootstrap-trust-roots-
   kill-switch.cedar` is published, soaked, and activated per
   ADR-0294 in `dev-tools-cell-staging`.
4. A T+8h kill-switch rehearsal has executed end-to-end: a
   simulated Stage-1 runner remains active past T+8h; the kill-
   switch fragment publishes; subsequent attempts to consume
   Stage-1-rooted artifacts are denied; the audit trail is
   verifiable.
5. `oya-check-bootstrap-spiffe-identity` lane scans the bootstrap
   replay log and reports zero non-SPIFFE-attested artifacts.

## Date

2026-05-20.

## Context

### What F5-247-02 actually says

F5-Security's r1 verdict (CRITICAL) reads:

> ADR-0247 §D-5 Stage 1 describes external CI (GitHub Actions /
> CircleCI / temporary self-hosted runner) deploying cosign-verified
> images. The chain of trust depends on cosign-signed-by-org-root
> verification. But: (a) the external CI runner itself can be
> compromised by GitHub Actions supply chain (tj-actions/changed-
> files 2025-03 incident, supply-chain CVE-2024-27302, etc.); (b)
> the cosign verification logic runs IN the external CI; (c) if
> external CI is compromised, attacker substitutes signed-but-
> malicious image and the bootstrap retroactively trusts it; (d)
> the 'bootstrap-replay log' (§D-5 Stage 0.5) is itself signed only
> inside the external CI runner.

The exploit path:

1. Adversary observes the announced bootstrap window (≤ 8 hours
   per ADR-0247 §D-5).
2. Adversary compromises the external CI runner during that window
   via one of:
   - A supply-chain CVE in a GitHub Actions / CircleCI dependency
     (tj-actions/changed-files compromise of 2025-03 actually
     happened; CVE-2024-27302 in the Azure DevOps runner; the
     SolarWinds-class events of 2020-2025 demonstrate the pattern
     is recurring).
   - An OS-image compromise on a temporary self-hosted runner.
   - Cloud-provider control-plane compromise (single bad credential
     on a service account; insider attack at the cloud vendor).
3. Adversary substitutes one or more container images in the
   bootstrap sequence with a backdoored variant. The cosign
   verification step runs INSIDE the compromised CI runner and
   reports success.
4. The platform retroactively trusts the backdoored images as
   part of its own chain of trust. The bootstrap-replay log
   (Stage 0.5) signs the actions inside the compromised CI and
   propagates them to the audit-chain (Stage 2.6).
5. Post-Stage-3 the bootstrap cell self-retires (Stage 3.3) and
   the platform considers itself bootstrapped. The compromised
   substrate has audited itself as legitimate.

F5 ranks this CRITICAL because:

1. **The compromise persists.** Post-bootstrap, the platform's
   self-modification capability runs on top of the compromised
   substrate. Every subsequent deployment trusts artifacts
   transitively rooted in the compromised Stage-1.
2. **Detection is hard.** The audit trail itself is signed by
   keys whose generation occurred inside the compromised CI
   runner; there is no out-of-band ground-truth comparison.
3. **The bootstrap window is announced.** Per ADR-0247 §D-5, the
   window is publicly documented for transparency. This is the
   correct posture for trust-building but it also tells the
   adversary exactly when to attack.

### What TMG-02 + ASC-04 add

TMG-02 (threat-model gap, absent compromised-CI-runner model)
formalizes the observation that ADR-0247 §D-5 implicitly trusts
the external CI runner. The threat model must be made explicit
before BLOCKER promotion.

ASC-04 (attack-surface concern, Tier-0 external dependencies under
nation-state legal process) observes that GitHub / Cloudflare DNS /
ECR are US-jurisdiction services. A US legal process can compel
them to assist in attribution exposure or to inject artifacts. The
bootstrap aspect of ASC-04 — that the bootstrap window depends on
US-jurisdiction services — is closed by this ADR; the broader
sovereign-mode bootstrap profile remains open work tracked under
ASC-04 separately.

### Why SPIFFE + one-shot CA + cosign + kill-switch

Five alternatives were considered:

| Alternative | Why rejected |
|---|---|
| **A. Trust GitHub Actions / CircleCI as-is.** Accept the compromise risk. | Rejected by F5 verdict; mismatches `feedback_quality_performance_scalability_bar` hyperscaler-grade posture. |
| **B. Use an air-gapped self-hosted runner managed entirely by founding team.** | Eliminates external-vendor CI compromise but reintroduces single-point-of-failure (the runner's host machine); also slow to provision in a regulated environment and difficult to attest. |
| **C. SPIFFE workload identity from a permanent CA.** | A permanent CA introduces a permanent trust root that must be protected forever; over-engineered for a ≤8h window. |
| **D. SPIFFE workload identity from a one-shot offline-rooted CA + sigstore cosign attestations + T+8h kill-switch.** | Selected. The CA's private key is destroyed after Stage-1 completes; the SPIFFE identities issued by the CA become useless past T+8h; cosign attestations provide a publicly-verifiable supply-chain provenance per sigstore's transparency log. |
| **E. Multiple-independent-runner N-of-M agreement on artifact hashes.** | Considered as additional defense layer; folded into §D-4 as RECOMMENDED hardening for the highest-stakes artifacts (substrate images touching policy-engine + audit-chain + identity) but not mandated for all Stage-1 artifacts because the engineering complexity is high. |

The selected resolution combines three named hyperscaler patterns:

- **SPIFFE / SPIRE workload identity (CNCF graduated).** The SPIFFE
  Verifiable Identity Document (SVID) provides cryptographic
  workload identity. Used by Netflix internally (Project Bouncer),
  by Bloomberg, by Pinterest, and is the basis of Istio's
  authentication primitive.
- **Sigstore cosign + Rekor transparency log + Fulcio short-lived
  certificates (Linux Foundation, 2021+).** Cosign provides
  cryptographic attestation of artifacts; Rekor is the public
  transparency log; Fulcio issues short-lived (10-minute) certs
  tied to OIDC identity. Used by Kubernetes (SLSA Level 3 by
  2023), by Node.js, by Python (PEP 740), and by every major
  package registry.
- **Cloudflare time-limited service certificate pattern.**
  Cloudflare's "Origin CA" (2017+) and "Roughtime" (2018+)
  establish patterns for time-bounded trust roots that
  auto-expire.

### Why one-shot offline-rooted CA specifically

The bootstrap CA differs from `oyatie.foundry.meta-trust-root`
(ADR-0293) in three structural ways:

| Aspect | Meta-trust-root (ADR-0293) | Bootstrap CA (this ADR) |
|---|---|---|
| **Lifetime** | 1-year rotation cadence, indefinite continuation | One-shot; key destroyed at end of Stage-1 (≤ 8h) |
| **Use case** | Witness signatures for self-modification, ongoing | Issue SPIFFE certificates to bootstrap runners, time-bounded |
| **Shamir threshold** | 5-of-9 across ≥ 3 jurisdictions | 3-of-5 within ≥ 2 jurisdictions (lower because the key destructs ≤ 8h later) |
| **Storage** | Tier-0 HSM, offline, kept across rotations | Tier-0 HSM, offline, **physically destroyed** after Stage-1 |
| **Audit** | Annual ceremony + every signature | Single ceremony + every certificate issued; the audit closes when the CA destructs |

The one-shot property means: even if an adversary later obtains
the CA's private key (e.g., by reconstructing it from leaked
Shamir shares), the bootstrap window has elapsed and no further
SPIFFE certificates can be honored. The blast radius is bounded
by the ≤ 8h window.

### Why the T+8h kill-switch

Per the synthesis doc: "≤8h bootstrap budget with a hard kill-
switch Cedar fragment that disables Stage-1 trust roots at T+8h
regardless of Stage-2 readiness."

The kill-switch is a Cedar fragment that activates at T+8h and
forbids any action whose authorization derives from a Stage-1
SPIFFE certificate. Three failure modes the kill-switch closes:

1. **Stage-1 runner is still active past T+8h.** A bug, an
   operational delay, or an adversary forcing Stage-1 to drag
   means the bootstrap window does not close at the announced
   time. The kill-switch closes it unilaterally.
2. **Stage-2 is not ready at T+8h.** The platform's self-hosted
   CI is supposed to take over at Stage 2.0; if it is not ready,
   the operations team faces a choice: extend Stage-1 (high risk)
   or accept the kill-switch (rollback to pre-bootstrap). The
   kill-switch forces the safer choice.
3. **Adversary has compromised Stage-1 but the team has not
   detected it.** Even without detection, the T+8h hard cutoff
   ensures no Stage-1-rooted artifacts can be honored past the
   window. Stage-2's takeover requires fresh artifacts signed
   under post-bootstrap trust roots; the compromised Stage-1
   artifacts cannot persist.

### Why now (2026-05-20)

Three forcing functions:

1. **F5-247-02 is one of the keystone bundle's four CRITICAL
   findings.** ADR-0247 cannot promote to `Accepted` until it is
   closed. The synthesis doc (§5.3) names this ADR as the
   resolution.
2. **The bootstrap CA ceremony has lead-time.** FIPS 140-3 L3 HSM
   procurement, 5-shard ceremony coordination, sigstore Fulcio
   trust-root publication: minimum 6-week lead time before the
   ceremony can run. The bundle merging now lets ceremony
   preparation begin in parallel with implementation.
3. **The bootstrap window is the platform's most-asymmetric
   defensive surface.** A single 8-hour window during which the
   chain of trust is bootstrapping is the single highest-value
   adversary target in the platform's lifecycle. Spending
   disproportionate engineering effort on this window is correct.

## Decision

The keystone establishes seven decision sub-sections, D-1 through
D-7.

### D-1. SPIFFE workload identity for every Stage-1 runner

Every Stage-1 external CI runner — whether GitHub Actions,
CircleCI, a temporary self-hosted runner, or a multi-cloud bake-
in-place provisioner — receives a SPIFFE Verifiable Identity
Document (SVID) issued by the one-shot bootstrap CA. The SVID
binds the runner's identity to a cryptographic key pair and a
short-lived (≤ 8h) validity window.

#### D-1.1. SPIFFE ID format

```
spiffe://oyatie.foundry.bootstrap-ca/runner/<runner-class>/<runner-id>
```

Where:

| Component | Value space | Example |
|---|---|---|
| Trust domain | `oyatie.foundry.bootstrap-ca` (fixed) | (constant) |
| `runner-class` | One of: `github-actions`, `circleci`, `self-hosted`, `aws-codebuild`, `gcp-cloud-build`, `azure-pipelines` | `github-actions` |
| `runner-id` | UUID generated by the bootstrap CA at issuance time | `8c4a2b1e-7d3f-4f2a-9c5b-1e8f3a4c5d6e` |

#### D-1.2. SVID contents

| Field | Value |
|---|---|
| **Subject** | The SPIFFE ID above |
| **Issuer** | `CN=oyatie.foundry.bootstrap-ca,O=oyatie,C=<juris>` |
| **NotBefore** | Bootstrap start time T+0 |
| **NotAfter** | Bootstrap start time T+8h MAXIMUM (may be shorter for specific runner classes) |
| **Public Key** | Ed25519 (preferred) OR ECDSA P-384 |
| **Extension `oyatie-bootstrap-stage`** | `stage-1.0`, `stage-1.1`, ..., `stage-1.10` — the specific Stage-1 step the runner is provisioned for |
| **Extension `oyatie-allowed-artifacts`** | List of artifact name prefixes the runner is allowed to sign (e.g., `cloud-secrets`, `identity`, `policy-engine` — restricted per runner role) |
| **Extension `oyatie-revocation-uri`** | URL to the bootstrap CA's CRL endpoint; runner MUST re-fetch every 60s |
| **Extension `oyatie-bootstrap-ca-attestation-hash`** | SHA3-256 of the CA's ceremony attestation document |

#### D-1.3. SVID delivery

The SVID is delivered to the runner via the SPIFFE Workload API
(SPIRE Agent pattern). The runner does NOT generate its own key
pair offline — the bootstrap CA generates the keypair inside its
own HSM, then transfers it to the runner over a TLS-mutual-
authenticated channel (TLS 1.3 + post-quantum-hybrid X25519+ML-KEM-
768 if available; pure X25519 fallback).

The runner stores the private key in its own ephemeral storage
(in-memory only; not persisted to disk). At runner termination,
the private key is zeroized.

### D-2. One-shot offline-rooted bootstrap CA — ceremony

The bootstrap CA's private key is generated and stored under a
ceremony procedure modeled on the §D-2 of ADR-0293 with three
differences:

1. **Reduced Shamir threshold** (3-of-5 vs 5-of-9): the CA's
   destruction at end of Stage-1 bounds the blast radius
   sufficient to permit lower threshold.
2. **Reduced jurisdiction count** (≥ 2 vs ≥ 3): same rationale.
3. **Explicit destruction ceremony** at end of Stage-1.

#### D-2.1. Ceremony participants

| Role | Count |
|---|---|
| Cryptographic Officers | 5 (≥ 2 jurisdictions; ≥ 1 council-security; ≥ 1 council-architecture) |
| Internal Witnesses | 2 (council-legal + ops-compliance) |
| External Witnesses | 1 (QTSP representative; same as ADR-0293) |
| Ceremony Master | 1 (council-security designate) |
| Notary Public | 1 |
| Recording Operator | 1 |

Total: 11 attestable participants. In-person attendance required.

#### D-2.2. Hardware specification

Same as ADR-0293 §D-2.2 — FIPS 140-3 L3 HSM (Thales Luna or
Entrust nShield), tamper-evident smartcards, dual independent 4K
camera recording, air-gap workstation.

The HSM USED for the bootstrap CA may be the SAME physical HSM as
the meta-trust-root key but its key slot is partitioned and the
slot is zeroized at end of Stage-1. Re-using the HSM hardware
reduces cost while maintaining cryptographic isolation.

#### D-2.3. Ceremony attestation chain

| Artifact | Storage |
|---|---|
| `BootstrapCaGenerationCeremonyAttestation.pdf` | audit-chain (Merkle-sealed-per-event), AND sigstore Rekor transparency log, AND sealed paper original |
| Public key + fingerprint | sigstore Fulcio trust-root publication + DNSSEC-signed DNS TXT record `_bootstrap-ca.<oyatie-root-domain>` + GitHub gist + (planned) ADR-0280 substrate DAG attachment |
| HSM serial + certificate chain | audit-chain + paper original |
| Video recording | Tamper-evident container; opened only on destruction ceremony |
| Ceremony participant attestations | Each signed by participant + Notary apostille |

The CA's public key is published to multiple independent venues
specifically so that an adversary cannot suppress its discovery.
A SPIFFE SVID verifier MUST find consistent CA public key in at
least 3 of the 5 venues; mismatch triggers verification refusal.

#### D-2.4. Destruction ceremony

At end of Stage-1 (target: ≤ 8h after generation; hard ceiling:
T+10h), a destruction ceremony executes:

| Step | Action |
|---|---|
| 1 | All 5 shard holders attend in person (full quorum); ceremony master + notary + recording operator present |
| 2 | HSM is unlocked using full-quorum shard reconstruction |
| 3 | The CA's private key slot is **zeroized** per FIPS 140-3 §4.7.5 Purge procedure |
| 4 | HSM emits a destruction attestation signed by its internal vendor key (Thales / Entrust) |
| 5 | All 5 smartcards are physically destroyed: cut by Notary into ≥ 4 pieces each; pieces distributed across ≥ 3 jurisdictions; destruction recorded on video |
| 6 | `BootstrapCaDestructionCeremonyAttestation.pdf` is signed by all 11 participants; recorded in audit-chain + sigstore Rekor |
| 7 | The CA's CRL endpoint is updated to mark all outstanding SPIFFE SVIDs as `revoked` with reason `bootstrap-window-closed` |
| 8 | The bootstrap CA's row in `bootstrap_ca_ceremonies` table is updated with `destroyed_at = now()` |

After the destruction ceremony, the bootstrap CA exists only as a
public key plus an attestation chain. No further SVIDs can be
issued. Existing SVIDs are revoked at the CRL endpoint AND
explicitly blocked by the kill-switch fragment (§D-5).

### D-3. Cosign attestation for every Stage-1 artifact

Every artifact produced during Stage-1 — container images, Helm
chart releases, Cedar fragment bundles, configuration packages —
carries a sigstore cosign attestation in the following profile:

#### D-3.1. Cosign attestation profile

```
predicate-type: https://oyatie.dev/attestations/bootstrap-stage-1/v1
subject:
  - name: <artifact-name>
    digest:
      sha256: <hex-digest>
attestation:
  builder:
    id: spiffe://oyatie.foundry.bootstrap-ca/runner/<runner-class>/<runner-id>
  build_type: oyatie-bootstrap-stage-<step-number>
  invocation:
    config_source:
      uri: git+https://<git-host>/<repo>@<git-sha>
      entry_point: <bootstrap-step-script-path>
    parameters:
      bootstrap_start_at: <timestamp>
      bootstrap_step: <stage>.<step>
      svid_serial: <SVID-serial-number>
  metadata:
    build_started_on: <timestamp>
    build_finished_on: <timestamp>
    reproducible: true | false
    completeness:
      parameters: true
      environment: true
      materials: true
  materials:
    - uri: <input-artifact-uri>
      digest:
        sha256: <hex-digest>
```

The profile is **SLSA Level 3 compliant** per the SLSA v1.0
specification: every artifact has a verifiable builder identity
(the SPIFFE SVID), a reproducible build description, a hermetic
materials list, and a tamper-evident audit log entry.

#### D-3.2. Rekor transparency log

Every attestation is published to the sigstore Rekor transparency
log AND to a private oyatie-operated Rekor instance run on Tier-2
control-plane cells AFTER Stage-2 is ready (per ADR-0247 §D-5
Stage 2.2 onwards). The private instance permits post-Stage-3
audit even if the public sigstore Rekor is unavailable or
compromised.

#### D-3.3. Verification at consumption

Any platform component that consumes a Stage-1 artifact MUST:

1. Fetch the artifact's cosign attestation.
2. Verify the attestation's signature against the SPIFFE SVID
   embedded in the attestation.
3. Verify the SPIFFE SVID against the bootstrap CA's public key
   from at least 3 of the 5 publication venues (§D-2.3).
4. Verify the SVID has not been revoked via the CRL endpoint OR
   the kill-switch fragment (§D-5).
5. Verify the attestation's `predicate-type` is the expected
   bootstrap-stage-1 profile.
6. Verify the artifact's reproducible build by re-building from
   the same source + materials (RECOMMENDED for highest-stakes
   artifacts; OPTIONAL for routine ones).

Any step's failure causes consumption refusal with a structured
audit emission.

### D-4. Multiple-runner N-of-M agreement (RECOMMENDED hardening)

For the highest-stakes Stage-1 artifacts — substrate images
touching `microservices/policy-engine/`, `microservices/audit-
chain/`, `microservices/identity/`, `microservices/cloud-secrets/`,
or the bootstrap CA's own ceremony tooling — operators MAY require
that the artifact be produced by 2-of-3 independent runners
across distinct vendors:

| Runner pair | Trust independence basis |
|---|---|
| GitHub Actions + CircleCI | Different cloud + different organisation |
| GitHub Actions + self-hosted on third-party hardware | Different vendor + different infrastructure |
| AWS CodeBuild + GCP Cloud Build | Different cloud-provider trust root |
| GitHub Actions + Azure Pipelines + temporary self-hosted | 2-of-3 N-of-M is the highest level RECOMMENDED |

The artifact's digest from all participating runners is compared;
mismatch triggers a SEV-1 alert and consumption refusal.

This is RECOMMENDED hardening, not required. The required floor
is single-runner SPIFFE + cosign per §D-1 + §D-3.

### D-5. T+8h kill-switch Cedar fragment

The kill-switch is a Cedar fragment that activates at T+8h and
forbids any action authorised by a Stage-1 SPIFFE certificate.

#### D-5.1. Fragment shape

```cedar
// microservices/policy-engine/fragments/baseline/bootstrap-trust-roots-kill-switch.cedar
// SCOPE: baseline
// SIGNED BY: org-root-key (the deepest anchor; not an intermediate)
// SOAK_DURATION: 60 (per ADR-0294 minimum)
// ACTIVATE_AT: <bootstrap-start-time> + 8 hours
// SUNSET_AT: null  (permanent; the kill-switch never sunsets — it is
//                   the post-bootstrap default)

// FORBID any action whose authorisation chain references a Stage-1
// SPIFFE SVID
forbid (
  principal,
  action,
  resource
)
when {
  context.authorisation_chain.contains_spiffe_identity_from(
    trust_domain == "oyatie.foundry.bootstrap-ca"
  )
};

// FORBID any artifact consumption whose cosign attestation references a
// Stage-1 builder identity
forbid (
  principal,
  action == Substrate::Action::DeploySubstrateVersion,
  resource is SubstrateArtifact
)
when {
  resource.cosign_attestation.builder_id starts_with "spiffe://oyatie.foundry.bootstrap-ca/"
};

// FORBID any policy-engine fragment whose signing chain references the
// bootstrap CA
forbid (
  principal,
  action == Cedar::Action::ActivateFragment,
  resource is CedarFragment
)
when {
  resource.signing_chain.includes_ca("oyatie.foundry.bootstrap-ca")
};

// FORBID any audit-chain row whose signing key chains to the bootstrap
// CA (for retroactive purpose: the audit row can still be read, but new
// rows whose authorisation chains include Stage-1 SVIDs are denied)
forbid (
  principal,
  action == AuditChain::Action::EmitRow,
  resource
)
when {
  context.row_signing_chain.includes_ca("oyatie.foundry.bootstrap-ca")
};
```

#### D-5.2. Publication mechanism

The kill-switch fragment is published by `oyatie.foundry.bootstrap-
kill-switch-publisher` automation. The automation:

1. **Pre-publishes** the fragment at T+0 (bootstrap start) into the
   cedar-fragment-registry in lifecycle stage `Scheduled`. The
   `activate_at` is bootstrap-start + 8h.
2. The fragment enters the per-ADR-0294 `Soaking` stage at T+8h
   minus 60s and transitions to `Activated` at T+8h.
3. The fragment is signed at T+0 by the org root key (the deepest
   anchor) so that even if all intermediate keys are compromised
   during Stage-1, the kill-switch remains trusted.

#### D-5.3. Override path

Authorized override of the kill-switch requires:

1. ≥ 3 human approvers including council-security + council-
   architecture + council-legal.
2. A fresh meta-trust-root witness signature (per ADR-0293).
3. Co-signature by the org root key intermediate.

Override is permitted only if Stage-2 readiness is verified by
out-of-band council-security manual hash verification (per §D-7).
Override is recorded in audit-chain with the explicit
`BootstrapKillSwitchOverride` evidence type.

The override path is intentionally heavy because the kill-switch
exists exactly for the case where Stage-1 may be compromised; an
override could be requested by an adversary who has compromised
Stage-1 to extend the window.

### D-6. Bootstrap budget ceiling — ≤8h with hard cutoff

The bootstrap budget is bounded:

| Time | Constraint |
|---|---|
| T+0 | Stage 0.5 bootstrap-replay log initialised; bootstrap CA ceremony begins |
| T+1h | Stage 0 + CA ceremony must complete; if not, SEV-1 emitted and council convenes |
| T+1h to T+6h | Stage 1 executes (Stage 1.0 through 1.10 per ADR-0247 §D-5) |
| T+6h | Stage 2.0 must begin: bootstrap cell's `oyatie.foundry.ci-build-and-test` produces its first artifact |
| T+8h | **HARD CUTOFF**: kill-switch fragment activates regardless of Stage-2 readiness |
| T+8h to T+10h | Destruction ceremony for bootstrap CA executes |

Operations MAY extend Stage-1 only via the §D-5.3 override path.
An attempted extension without the override is denied by the
kill-switch fragment.

### D-7. Out-of-band Stage-1 → Stage-2 manual hash verification

Per F5-247-02 recommendation (a): "Mandate an out-of-band human
verification step at Stage 1.10 / Stage 2.0 transition: a council-
security council member, on a physically separate device, fetches
the deployed image hashes via direct HSM-stored API key and
compares against the cosign attestation Rekor log."

The procedure:

1. At Stage 1.10, the bootstrap-replay log enumerates every
   deployed artifact's digest + cosign attestation reference.
2. Two designated council-security members EACH (independently)
   on a separate device:
   - Fetch each artifact's digest from the container registry
     via a council-security-only API token stored offline.
   - Verify each cosign attestation against:
     - Public sigstore Rekor transparency log.
     - Bootstrap CA's published public key from ≥ 3 of the 5
       publication venues per §D-2.3.
     - SLSA Level 3 reproducible-build verification by re-
       building from the documented source + materials.
3. Both council-security members independently sign a
   `BootstrapStage1To2HandoffAttestation.pdf` confirming the
   verification.
4. The signed attestation is registered in audit-chain (Merkle-
   sealed-per-event).
5. Only after the attestation is registered does Stage 2.0 begin.

If any verification step fails:

1. Stage 2.0 does NOT begin.
2. The kill-switch fragment activates (the bootstrap is aborted).
3. Bootstrap restart requires a new bootstrap CA ceremony and a
   new bootstrap window.

The two-member independence is the load-bearing property: a single
compromised council-security member cannot wave through a
verification.

## Consequences

### Positive

1. **The F5-247-02 exploit window is closed.** A compromised
   external CI runner can no longer substitute artifacts because
   every artifact's authenticity is bound to a SPIFFE SVID that
   the adversary cannot forge without compromising the offline
   bootstrap CA's private key — which is held in a multi-
   jurisdiction Shamir quorum and destroyed at end of Stage-1.
2. **The blast radius of any Stage-1 compromise is bounded by
   T+8h.** Past T+8h, no Stage-1-rooted artifact can be
   consumed; the platform's post-bootstrap trust chain re-
   establishes from fresh roots.
3. **Provenance is publicly auditable via sigstore Rekor.** Any
   third party can verify the bootstrap artifacts' provenance
   without trusting oyatie's own audit chain.
4. **Out-of-band verification at Stage-1 → Stage-2 transition
   provides a human ground-truth checkpoint.** Two independent
   council-security members are the human anchor that the
   automated chain depends on.
5. **The mechanism composes with ADR-0293 and ADR-0294.**
   Bootstrap CA's destruction ceremony is independent of the
   meta-trust-root ceremony; the kill-switch fragment soaks
   through the ADR-0294 lifecycle; the kill-switch's emergency
   override path requires both meta-trust-root witness AND
   council-security approval.

### Negative

1. **Bootstrap operational complexity increases significantly.**
   Adding SPIFFE issuance, cosign attestation per artifact, Rekor
   transparency publication, and N-of-M agreement (where used)
   roughly doubles the number of moving parts in Stage-1.
2. **Ceremony cost increases.** Bootstrap CA ceremony adds an
   estimated $40K-60K per bootstrap (the ceremony cost is
   amortized over a single bootstrap event but recurs if the
   platform ever re-bootstraps).
3. **The ≤8h hard ceiling may be tight.** Real-world bootstraps
   could exceed 8h due to operational delays. The kill-switch's
   absence of soft extension (override is heavy) is a deliberate
   design choice — the trade-off is operational rigidity vs
   security.
4. **SPIFFE / SPIRE adds a substrate dependency that doesn't
   self-host until later.** The SPIFFE Workload API runs on the
   bootstrap CA's infrastructure during Stage-1; the post-
   bootstrap SPIFFE substrate is provisioned as part of Stage-2
   (per `microservices/identity/` self-hosting).
5. **Public publication of the bootstrap CA's public key creates
   a static target.** The 5-venue publication (DNSSEC, GitHub
   gist, sigstore Fulcio, ADR-0280 DAG, audit-chain) is the
   mitigation — an adversary must suppress 3 of 5 simultaneously
   to defeat consistency-check verification.

### Neutral

1. **The mechanism is invisible to customer tenants.** Customer
   tenants never interact with bootstrap artifacts directly; their
   first interaction with the platform is post-Stage-3 when the
   platform has retired Stage-1 trust roots entirely.
2. **The bootstrap CA's HSM hardware MAY be re-used for the meta-
   trust-root key.** The two key slots are partitioned; the
   bootstrap CA's slot is zeroized at destruction.
3. **The kill-switch fragment is permanent post-activation.** It
   never sunsets; it is the post-bootstrap default. New cells
   that come online post-Stage-3 inherit it through ADR-0243 §D-2
   baseline propagation.

## Detailed Mechanics

### D-1 expanded — SPIRE Agent topology in Stage-1

```
┌──────────────────────────────────────────────────────────────┐
│                Stage-1 External Environment                   │
│  (GitHub Actions / CircleCI / Self-hosted runner)            │
│                                                                │
│   ┌─────────────────────────────────────────────────────┐   │
│   │           Stage-1 CI Runner Container                │   │
│   │                                                       │   │
│   │   ┌─────────────────────────────────────────────┐   │   │
│   │   │   SPIRE Agent (vendored, ephemeral)         │   │   │
│   │   │                                              │   │   │
│   │   │   - Workload API: Unix domain socket         │   │   │
│   │   │   - Fetches SVID from bootstrap CA           │   │   │
│   │   │   - Renews every 60s (or 8h max lifetime)    │   │   │
│   │   │   - Zeroizes on container termination        │   │   │
│   │   └─────────────────────────────────────────────┘   │   │
│   │                       ▲                                │   │
│   │                       │ mTLS                          │   │
│   │                       │ (X25519 + ML-KEM-768 hybrid)  │   │
│   │                       │                                │   │
│   └───────────────────────┼────────────────────────────────┘   │
└────────────────────────────┼────────────────────────────────────┘
                             │
                             │ TLS 1.3 + PQ hybrid
                             │
┌────────────────────────────▼────────────────────────────────┐
│         Bootstrap CA (offline HSM-rooted)                    │
│                                                                │
│   ┌─────────────────────────────────────────────────────┐   │
│   │   SPIRE Server (vendored; runs in HSM-adjacent      │   │
│   │   admin workstation; ephemeral)                      │   │
│   │                                                       │   │
│   │   - Issues SVIDs against the bootstrap CA private    │   │
│   │     key in the offline HSM                           │   │
│   │   - CRL endpoint published on bootstrap CA's domain  │   │
│   │   - Logs every SVID issuance to audit-chain via      │   │
│   │     the bootstrap-replay log mechanism               │   │
│   └─────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
```

### D-3 expanded — cosign attestation Rust trait

```rust
// crates/oya-shared-bootstrap-runner-identity/src/cosign_attestation.rs

pub trait BootstrapAttestation {
    /// Verify that the artifact's cosign attestation chains to the
    /// bootstrap CA's public key from ≥ 3 of 5 publication venues.
    fn verify_provenance(&self) -> Result<ProvenanceVerified, AttestationError>;

    /// Verify the artifact's reproducible build by re-running the
    /// recorded build invocation against the recorded materials.
    fn verify_reproducible_build(&self) -> Result<ReproducibilityVerified, AttestationError>;

    /// Verify the SVID's CRL status; refuses if revoked.
    fn verify_svid_not_revoked(&self, crl_endpoint: &Url) -> Result<NotRevoked, AttestationError>;

    /// Verify the kill-switch fragment's predicate does not currently
    /// forbid consumption of this artifact.
    fn verify_kill_switch_does_not_apply(
        &self,
        cedar_evaluator: &CedarEvaluator,
    ) -> Result<KillSwitchInactive, AttestationError>;
}

pub struct ProvenanceVerified {
    pub bootstrap_ca_public_key_fingerprint: [u8; 32],
    pub publication_venues_verified: Vec<PublicationVenue>,
    pub svid_serial: u64,
    pub svid_lifetime: (SystemTime, SystemTime),
    pub builder_runner_class: RunnerClass,
    pub rekor_log_entry_uuid: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublicationVenue {
    DnssecTxt,
    GitHubGist,
    SigstoreFulcio,
    Adr0280SubstrateDag,
    AuditChain,
}

#[derive(Debug, thiserror::Error)]
pub enum AttestationError {
    #[error("bootstrap CA public key inconsistent across publication venues: {0:?}")]
    PublicationVenueInconsistent(Vec<PublicationVenue>),

    #[error("SVID expired: not_after={not_after:?}, now={now:?}")]
    SvidExpired { not_after: SystemTime, now: SystemTime },

    #[error("SVID revoked: serial={serial}, reason={reason}")]
    SvidRevoked { serial: u64, reason: String },

    #[error("kill-switch fragment forbids consumption of this artifact")]
    KillSwitchActive,

    #[error("reproducible build mismatch: expected={expected:?}, actual={actual:?}")]
    ReproducibleBuildMismatch { expected: [u8; 32], actual: [u8; 32] },

    #[error("cosign signature verification failed")]
    SignatureInvalid,
}
```

### D-5 expanded — Postgres schema for bootstrap CA + SVID + kill-switch

```sql
-- microservices/cedar-fragment-registry/migrations/0048_bootstrap_runner_identity.sql

CREATE TABLE bootstrap_ca_ceremonies (
    attestation_hash        BYTEA PRIMARY KEY,
    ceremony_type           TEXT NOT NULL CHECK (ceremony_type IN (
        'generation', 'destruction'
    )),
    ceremony_date           TIMESTAMPTZ NOT NULL,
    ceremony_facility       TEXT NOT NULL,
    notary_apostille_id     TEXT NOT NULL,
    qtsp_attestation_id     TEXT NOT NULL,
    hsm_brand_model         TEXT NOT NULL,
    hsm_serial_number       TEXT NOT NULL,
    fips_certificate_number TEXT NOT NULL,
    shard_count_n           SMALLINT NOT NULL CHECK (shard_count_n = 5),
    shard_threshold_m       SMALLINT NOT NULL CHECK (shard_threshold_m = 3),
    jurisdictions           TEXT[] NOT NULL CHECK (cardinality(jurisdictions) >= 2),
    public_key              BYTEA,         -- NULL after destruction
    public_key_fingerprint  BYTEA UNIQUE,  -- NULL after destruction
    publication_venues      JSONB NOT NULL,  -- map of venue -> URL/locator
    rekor_log_entry_uuid    UUID NOT NULL,
    valid_from              TIMESTAMPTZ NOT NULL,
    valid_until             TIMESTAMPTZ NOT NULL,
    destroyed_at            TIMESTAMPTZ,
    destruction_ceremony_ref BYTEA REFERENCES bootstrap_ca_ceremonies(attestation_hash),
    CONSTRAINT bootstrap_ca_lifetime_max_10_hours
        CHECK (valid_until <= valid_from + INTERVAL '10 hours')
);

CREATE TABLE bootstrap_svids (
    svid_serial             BIGSERIAL PRIMARY KEY,
    svid_uri                TEXT NOT NULL UNIQUE,
    runner_class            TEXT NOT NULL CHECK (runner_class IN (
        'github-actions', 'circleci', 'self-hosted',
        'aws-codebuild', 'gcp-cloud-build', 'azure-pipelines'
    )),
    runner_id               UUID NOT NULL,
    bootstrap_ca_ref        BYTEA NOT NULL REFERENCES bootstrap_ca_ceremonies(attestation_hash),
    public_key              BYTEA NOT NULL,
    issued_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
    not_before              TIMESTAMPTZ NOT NULL,
    not_after               TIMESTAMPTZ NOT NULL,
    allowed_artifacts       TEXT[] NOT NULL,
    revoked_at              TIMESTAMPTZ,
    revocation_reason       TEXT,
    UNIQUE (runner_class, runner_id),
    CONSTRAINT svid_lifetime_max_8_hours
        CHECK (not_after <= not_before + INTERVAL '8 hours')
);

CREATE TABLE bootstrap_artifact_attestations (
    attestation_uuid        UUID PRIMARY KEY,
    artifact_name           TEXT NOT NULL,
    artifact_digest_sha256  BYTEA NOT NULL,
    svid_serial             BIGINT NOT NULL REFERENCES bootstrap_svids(svid_serial),
    cosign_signature        BYTEA NOT NULL,
    rekor_log_entry_uuid    UUID NOT NULL,
    build_started_on        TIMESTAMPTZ NOT NULL,
    build_finished_on       TIMESTAMPTZ NOT NULL,
    reproducible            BOOLEAN NOT NULL,
    materials               JSONB NOT NULL,
    n_of_m_quorum_count     SMALLINT,  -- non-null only when N-of-M used
    n_of_m_runners          BIGINT[],
    UNIQUE (artifact_name, artifact_digest_sha256)
);

CREATE TABLE bootstrap_kill_switch_state (
    bootstrap_session_id    UUID PRIMARY KEY,
    bootstrap_started_at    TIMESTAMPTZ NOT NULL,
    kill_switch_activate_at TIMESTAMPTZ NOT NULL,
    kill_switch_activated   BOOLEAN NOT NULL DEFAULT FALSE,
    kill_switch_activated_at TIMESTAMPTZ,
    override_attestation_id UUID,
    override_approvers      TEXT[],
    audit_emission_hash     BYTEA NOT NULL,
    CONSTRAINT kill_switch_at_or_before_t_plus_8h
        CHECK (kill_switch_activate_at <= bootstrap_started_at + INTERVAL '8 hours')
);

CREATE TABLE bootstrap_stage1_to_stage2_handoffs (
    handoff_id              UUID PRIMARY KEY,
    bootstrap_session_id    UUID NOT NULL REFERENCES bootstrap_kill_switch_state(bootstrap_session_id),
    handoff_attestation_pdf_hash BYTEA NOT NULL,
    council_security_signer_1_pubkey BYTEA NOT NULL,
    council_security_signer_1_signature BYTEA NOT NULL,
    council_security_signer_2_pubkey BYTEA NOT NULL,
    council_security_signer_2_signature BYTEA NOT NULL,
    verified_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    artifacts_verified_count INTEGER NOT NULL,
    artifacts_verified_digests BYTEA[] NOT NULL,
    audit_emission_hash     BYTEA NOT NULL,
    CONSTRAINT signers_distinct
        CHECK (council_security_signer_1_pubkey != council_security_signer_2_pubkey)
);
```

### D-6 expanded — bootstrap budget telemetry

The bootstrap timeline is tracked in real-time via the `bootstrap_
kill_switch_state` table and emitted to a dashboard accessible
to:

- All 11 ceremony participants (with read-only access).
- All council members (with read-only access).
- ops-sre-reliability on-call (with read-only access).

Telemetry panels:
- Time elapsed since T+0
- Time remaining until T+8h hard cutoff
- Stage 1.X progress (per-step completion timestamps)
- Stage 2.0 readiness indicator
- Per-step artifact count + cosign attestation status
- Out-of-band Stage-1→Stage-2 handoff status (pending / verified
  by 1 of 2 / verified by 2 of 2)

### D-7 expanded — council-security verification protocol

Both council-security members independently execute the following
script (a vendored tool, not the in-flight platform code) on
physically separate devices:

```bash
# Pseudo-shell for the verification protocol
# (real implementation: oya-bootstrap-verify CLI tool, vendored
# alongside the bootstrap-replay log)

oya-bootstrap-verify \
    --replay-log <bootstrap-replay-log.signed> \
    --ca-attestation <bootstrap-ca-attestation.pdf> \
    --rekor-endpoint https://rekor.sigstore.dev \
    --dnssec-txt-record _bootstrap-ca.oyatie.dev \
    --github-gist <gist-url> \
    --sigstore-fulcio-trust-root <url> \
    --output handoff-attestation.unsigned.json

# The tool:
# 1. Parses the bootstrap-replay log.
# 2. For each Stage-1 artifact:
#    a. Fetches the artifact's cosign attestation.
#    b. Verifies SVID against bootstrap CA public key from ≥ 3 of 5
#       venues.
#    c. Confirms SVID not revoked.
#    d. Re-runs reproducible build (where applicable).
#    e. Confirms Rekor log entry.
# 3. Generates an unsigned handoff attestation containing the
#    verification outcomes for each artifact.

# Council-security member then signs the unsigned attestation with
# their HSM-stored offline signing key:
oya-bootstrap-verify sign \
    --attestation handoff-attestation.unsigned.json \
    --signer-key council-security-1.hsm.key \
    --output handoff-attestation.signed.member1.json
```

The two signed attestations are submitted to the
`bootstrap_stage1_to_stage2_handoffs` table. The platform's
Stage-2 entry waits for BOTH to land before proceeding.

## Implementation Footprint

### Microservice scope

| Microservice | Change | Effort |
|---|---|---|
| `microservices/policy-engine/` | Add kill-switch fragment publishing; add SPIFFE verification predicate; add bootstrap-CA chain verification | ≈ 3 weeks |
| `microservices/cedar-fragment-registry/` | Add migrations 0048; add bootstrap-svids + bootstrap-artifact-attestations CRUD | ≈ 2 weeks |
| `microservices/audit-chain/` | Add bootstrap-stage-1 evidence stream + handoff attestation stream | ≈ 1 week |
| `microservices/identity/` | Register the new principals; add CRL endpoint serving for the bootstrap CA | ≈ 2 weeks |
| `microservices/cloud-secrets/` | OpenBao policy attaching SPIFFE SVID to short-lived bootstrap tokens (overlap with ADR-0296) | ≈ 1 week |
| `crates/oya-shared-bootstrap-runner-identity/` (new) | Shared crate exposing SPIFFE primitives + cosign attestation primitives + kill-switch fragment construction | ≈ 4 weeks |
| `tools/oya-bootstrap-verify/` (new vendored CLI) | Standalone tool for council-security manual verification; vendored separately so it does not depend on in-flight platform code | ≈ 2 weeks |

Total: ≈ 15 weeks engineering effort, parallelizable across crews.
Calendar time ≈ 5 weeks.

### Hardware + ceremony scope

| Item | Quantity | Lead time |
|---|---|---|
| FIPS 140-3 L3 HSM (may share with ADR-0293 meta-trust-root) | 1 (shared) | 6-8 weeks |
| Tamper-evident smartcards | 10 (5 primary + 5 backup) | 4 weeks |
| Ceremony facility (single use for the bootstrap ceremony) | 1 | 4 weeks |
| Recording rig + storage | 1 | 2 weeks |
| Council-security offline signing kits (for §D-7 handoff signing) | 2 (one per council-security member) | 4 weeks |

### CI lane scope

| CI lane | Behavior |
|---|---|
| `oya-check-bootstrap-spiffe-identity` | Scans the bootstrap-replay log for any artifact lacking SPIFFE-attested provenance; emits findings |
| `oya-check-bootstrap-runner-ca-ceremony-evidence` | Verifies the canonical bootstrap CA's generation + destruction ceremony attestations are recorded in audit-chain |
| `oya-check-bootstrap-artifact-cosign-attestation` | Verifies every recorded Stage-1 artifact has a corresponding cosign attestation row + Rekor log entry |
| `oya-check-bootstrap-budget-ceiling` | Verifies the bootstrap-replay log's elapsed time from T+0 to Stage 1.10 ≤ 8h |
| `oya-check-bootstrap-kill-switch-fragment-present` | Verifies the `baseline/bootstrap-trust-roots-kill-switch.cedar` fragment is present + signed by org root key + in lifecycle stage at-or-past `Activated` |
| `oya-check-bootstrap-runner-revocation-coverage` | Verifies every Stage-1 SVID has either a `revoked_at` timestamp OR a `not_after` already-elapsed timestamp; no SVID exists in indefinite-validity state |

## Migration

### Stage 0 — Hardware + ceremony scheduling (T+0 to T+6w)

| Step | Action |
|---|---|
| 0.1 | Council-security recruits 5 Cryptographic Officers (≥ 2 jurisdictions) |
| 0.2 | HSM coordinated with ADR-0293 ceremony (shared hardware) |
| 0.3 | Sigstore Fulcio publishes the bootstrap CA's public key as a recognized trust root |
| 0.4 | DNSSEC TXT records published for `_bootstrap-ca.oyatie.dev` and equivalent jurisdictions |
| 0.5 | Tooling scaffolded: oya-bootstrap-verify CLI, SPIRE Agent vendor pinning, cosign integration |

### Stage 1 — Code implementation in parallel (T+0 to T+5w)

| Step | Action |
|---|---|
| 1.1 | `oya-shared-bootstrap-runner-identity` crate scaffolded |
| 1.2 | Migrations 0048 applied to dev + staging |
| 1.3 | Cedar kill-switch fragment authored + signed by org root key (offline) |
| 1.4 | SPIRE Agent + Server vendored into the bootstrap-replay log toolchain |
| 1.5 | Integration tests: SVID issuance, cosign attestation, kill-switch activation, council-security handoff verification |

### Stage 2 — Rehearsal in `dev-tools-cell-staging` (T+5w to T+8w)

| Step | Action |
|---|---|
| 2.1 | Bootstrap rehearsal: a controlled bootstrap-replay run in staging with all Stage-1 artifacts SPIFFE-attested |
| 2.2 | Out-of-band handoff verification rehearsal: council-security members independently verify the rehearsal artifacts |
| 2.3 | Kill-switch rehearsal: simulate T+8h hard cutoff; verify post-cutoff artifacts are denied |
| 2.4 | N-of-M agreement rehearsal: verify 2-of-3 runner agreement for the highest-stakes artifacts |
| 2.5 | Improvements from rehearsal folded into the production ceremony procedure |

### Stage 3 — Production bootstrap ceremony (T+8w to T+10w)

| Step | Action |
|---|---|
| 3.1 | Bootstrap CA generation ceremony executed per §D-2 |
| 3.2 | Stage-1 bootstrap window opens; runners issued SVIDs; artifacts produced + cosign-attested |
| 3.3 | Stage 1.10 → Stage 2.0 handoff verified by 2 council-security members |
| 3.4 | Stage 2.0 onwards proceeds per ADR-0247 §D-5 with self-hosted CI |
| 3.5 | Bootstrap CA destruction ceremony at T+8h to T+10h |
| 3.6 | Kill-switch fragment activates; bootstrap-replay log finalized + ingested into audit-chain per ADR-0247 §D-5 Stage 2.6 |

### Stage 4 — Advisory → BLOCKER (T+10w+)

The six CI lanes flip from advisory to BLOCKER:
1. `oya-check-bootstrap-spiffe-identity`
2. `oya-check-bootstrap-runner-ca-ceremony-evidence`
3. `oya-check-bootstrap-artifact-cosign-attestation`
4. `oya-check-bootstrap-budget-ceiling`
5. `oya-check-bootstrap-kill-switch-fragment-present`
6. `oya-check-bootstrap-runner-revocation-coverage`

The bundle's promotion gate for ADR-0247 (bootstrap aspect of
F5-247-02) closes.

## References

### Primary

- `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` §5.3
- `evidence/debate/keystone-bundle-2026-05-20-F5-security-r1.json`
  — F5-247-02 (CRITICAL).

### Related ADRs

- ADR-0247 (Self-Hosting / Self-Modification Doctrine) — §D-5
  Stage 1 trust framing is amended by this ADR.
- ADR-0243 (Cedar as Universal Gate) — chain of trust gains the
  bootstrap-runner intermediate identity; baseline scope gains
  the kill-switch fragment.
- ADR-0293 (Foundry Meta-Trust-Root) — ceremony shape inherited;
  HSM hardware may be shared.
- ADR-0294 (Cedar Fragment Soak + Anomaly-Rollback) — kill-switch
  fragment soaks through this lifecycle.
- ADR-0250 (Build Ahead of Certification) — cosign / SLSA / Rekor
  pattern aligns with the build-ahead-of-certification posture.
- ADR-0280 (Substrate-of-Substrate Dependency) — bootstrap CA's
  public-key publication includes the substrate DAG node.

### Industry references

- **SPIFFE / SPIRE specification (CNCF graduated, 2022).**
  Workload identity primitive; SVID document format.
- **Sigstore project (Linux Foundation, 2021+).** Cosign + Rekor
  + Fulcio architecture; SLSA Level 3 alignment.
- **SLSA v1.0 specification (OpenSSF, 2023).** Provenance +
  reproducible build + isolation requirements.
- **CNCF Software Supply Chain Best Practices (2021).** N-of-M
  build agreement pattern.
- **Cloudflare Origin CA + Roughtime (2017+).** Time-bounded trust
  root precedent.
- **tj-actions/changed-files supply chain compromise (2025-03)
  post-mortem.** Concrete example of GitHub Actions supply chain
  attack; motivates the SPIFFE-pinned identity requirement.
- **CVE-2024-27302 Azure DevOps runner.** Concrete example of
  runner-level compromise; motivates the one-shot CA pattern.
- **SolarWinds Orion supply chain compromise (2020).** Concrete
  example of long-dwell compromise that lateralized through
  CI/CD.
- **Codecov bash uploader compromise (2021).** Concrete example
  of build-time injection.
- **Netflix Project Bouncer (2021-2024 blog posts).** SPIFFE
  internal-platform precedent.
- **Bloomberg + Pinterest SPIFFE deployments (2022-2023).**
  Production SPIFFE patterns.
- **Istio mutual TLS + SPIFFE identity (Istio 1.x+).** Service
  mesh SPIFFE precedent.

### Cryptographic + standards references

- **NIST SP 800-204D "Strategies for the Integration of Software
  Supply Chain Security in DevSecOps CI/CD Pipelines."** (2024)
- **FIPS 140-3 §4.7.5 Purge procedure.** Used for bootstrap CA
  destruction.
- **RFC 9162 — Certificate Transparency Version 2.0.** Transparency
  log precedent inherited by Rekor.
- **RFC 8809 — Registries for SPIFFE.** Trust-domain registration.
- **NIST IR 8397 "Guidelines on Minimum Standards for Developer
  Verification of Software."** (2021)
- **OpenSSF Scorecard (2021+).** Public scoring of supply-chain
  security practices.

### Slice cross-references

- **Slice 1 (runbooks):**
  `docs/runbooks/bootstrap-ca-key-ceremony.md`,
  `docs/runbooks/bootstrap-ca-destruction-ceremony.md`,
  `docs/runbooks/bootstrap-stage-1-to-stage-2-handoff.md`,
  `docs/runbooks/bootstrap-kill-switch-rehearsal.md`,
  `docs/runbooks/bootstrap-emergency-extension.md` are required
  by this ADR's CI lanes; their authoring is in Slice 1 scope.
- **Slice 3 (ADR-0246 amendment):** The
  `oya-check-bootstrap-spiffe-identity` CI lane is added to
  ADR-0246's fragment-validation lane catalogue; the actual
  amendment to ADR-0246 is in Slice 3 scope.
- **Slice 4 (naming justifications):** The five new names
  (`oyatie.foundry.bootstrap-runner`,
  `oyatie.foundry.bootstrap-ca`,
  `oyatie.foundry.bootstrap-kill-switch-publisher`,
  `oya-shared-bootstrap-runner-identity`,
  `oya-check-bootstrap-spiffe-identity`) are justified in this
  ADR's front matter.

### Specifications

- `/specs/bootstrap-runner-identity-protocol.json` (new) —
  canonical machine-readable record of the SPIFFE issuance
  protocol, CRL refresh cadence, SVID lifetime constraints.
- `/specs/bootstrap-kill-switch-fragment.json` (new) — canonical
  record of the kill-switch fragment's Cedar source, signing
  chain, and publication venues.
- `/specs/bootstrap-tier-model.json` — extended to add the
  bootstrap CA as a Tier-0 component alongside the org root key.

### Memory references

- `feedback_oyatie_is_a_tenant_doctrine` — reserved-namespace
  under which the bootstrap principals are registered.
- `feedback_cedar_as_universal_gate` — kill-switch is a Cedar
  fragment.
- `feedback_self_modification_doctrine` — bootstrap is the
  prerequisite for self-modification; closing the bootstrap
  exploit window is prerequisite for ADR-0247 promotion.
- `feedback_build_ahead_of_certification` — cosign / SLSA Level 3
  alignment matches the build-ahead-of-certification posture.
- `feedback_no_silent_regression` — ADR-0247 §D-5 is amended
  through a documented + CI-enforced path, not silently changed.
- `feedback_autonomous_implementation_artifacts` — the bootstrap
  budget ceiling + handoff verification + kill-switch combine to
  make autonomous re-bootstrap feasible without unbounded human-
  intervention windows.
- `feedback_naming_justification` — the five new names carry
  inline justification.
- `feedback_quality_performance_scalability_bar` — SPIFFE +
  sigstore + SLSA Level 3 alignment matches hyperscaler-grade
  supply-chain posture.

---

**End of ADR-0295.**
