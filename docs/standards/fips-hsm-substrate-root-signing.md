---
purpose: "Canonical FIPS 140-3 / HSM tier binding for substrate root-signing operations. Defines the four-tier HSM model, approved vendor roster, ceremony procedure, Cedar gate, sovereign-cell variants, and CI lane. Closes F7-004 (P0) from keystone-bundle-2026-05-20 supply-chain review."
doc_class: Standard
shape: Reference
length_cap: 800
authority_tier: 1
status: Accepted
date: 2026-05-20
canonical_authority:
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0247-self-hosting-self-modification-doctrine.md
  - docs/decisions/ADR-0708-platform-foundations-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - ADR-NNNN-foundry-meta-trust-root
planned_enforcement_ref: oya-governance-fips-hsm-substrate-root
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/standards/cedar-policy-discipline.md
  - docs/runbooks/cedar-hsm-root-key-ceremony.md
  - docs/STANDARDS-AND-TEMPLATES.md
related_adrs:
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0254-deployment-model-spectrum.md
  - ADR-NNNN-foundry-meta-trust-root
related_memories:
  - feedback_cedar_as_universal_gate
  - feedback_compliance_pack_primitive
  - feedback_byok_everywhere_credentials
  - feedback_amazon_shape_cellular_architecture
  - feedback_self_modification_doctrine
change_log:
  - "2026-05-20: Initial publication. Closes F7-004 P0 from keystone-bundle-2026-05-20 supply-chain review (F7 Supply Chain). Authored as part of Slice 4 (F7/F6 fix subagent)."
---

# FIPS / HSM Substrate Root-Signing Standard

## Doctrinal Authority

This standard is **authoritative** for all substrate root-signing operations. It overlays and
supersedes any generic key-management guidance in `docs/standards/`. On conflict with any ADR
cited in the frontmatter, the ADR wins for its specific scope; this standard wins for the
cross-cutting tiering and vendor-selection decisions.

The standard closes F7-004 (P0) from the keystone-bundle-2026-05-20 supply-chain review:
> "ADR-0243 says 'tier-0 HSM' generically, ADR-0247 §D-4 Stage 0.4 says 'Ed25519 + cosign'
> but does not name the HSM. The keystone does not reconcile FIPS 140-2 L2 (FedRAMP Moderate)
> vs L3 (FedRAMP High) vs NSA Type 1 (IL6) HSM ceremony differences."

This standard makes that reconciliation explicit and machine-checkable.

---

## §1. Scope — Which Operations Require FIPS/HSM Rooting

The following substrate root-signing operations MUST be rooted in an HSM that meets the tier
requirement specified in §2:

| Operation | ADR reference | Required tier |
|---|---|---|
| Cosign org root key generation and use | ADR-0247 §D-4 Stage 0.4 | Tier 0 |
| Rekor transparency log signing key | ADR-0247 §D-4; ADR-NNNN-foundry-meta-trust-root | Tier 0 (self-hosted Rekor) or Sigstore PGI (external) |
| Fulcio root CA private key | ADR-0247 §D-5 Stage 2 (self-hosted Sigstore) | Tier 0 |
| Meta-trust-root key (`oyatie.foundry.meta-trust-root`) | ADR-NNNN-foundry-meta-trust-root; ADR-0247 §D-8 | Tier 0 (offline ceremony key); Tier 1 (networked signing use) |
| Cedar genesis fragment signing (bootstrap) | ADR-0243 §D-5 | Tier 0 |
| Org-baseline intermediate key (Cedar + cosign) | ADR-0243 §D-5; ADR-0247 §D-3 | Tier 1 |
| Per-pack-owner intermediate keys (compliance-pack signing) | ADR-0251 §D-1 | Tier 1 |
| Per-jurisdiction overlay keys | ADR-0243 §D-5 | Tier 1 (or Tier 2 for non-regulated jurisdictions) |
| Per-tenant CMEK root key (encryption-BYOK) | ADR-0251 §D-10 | Tier 2 |
| Per-tenant provider-BYOK credential envelope | ADR-0255 §D-4 | Tier 2 |
| Per-call ephemeral signing tokens (Fulcio-issued OIDC certs) | Sigstore keyless flow | Tier 3 (no FIPS requirement) |
| SPIFFE SVID issuance (short-lived) | ADR-0253 §D-3 SPIRE | Tier 3 (no FIPS requirement) |

**Out of scope:** Application-layer secrets (database passwords, API tokens), JWT signing
keys for user sessions, Kafka SASL credentials. Those are managed via OpenBao (HashiCorp
Vault protocol) per the substrate secret-management standard, not HSM.

---

## §2. Tiered Model

The HSM tier model is a four-level hierarchy from most restrictive (Tier 0, offline ceremony)
to least (Tier 3, per-call ephemeral). Each tier has a FIPS requirement floor and an
operational model.

### Tier 0 — Offline Ceremony Key (FIPS 140-3 Level 4 in HSM Safe)

**Purpose:** Root of the org signing chain. Used only during the annual (or emergency) root
key ceremony. Never online; never connected to any network.

**FIPS requirement:** FIPS 140-3 Level 4. Level 4 is the highest level of physical security
defined by FIPS 140-3; it requires active tamper response (zeroisation on physical intrusion)
and environmental failure protection. This is the requirement for NSA Type 1 analogue at the
highest assurance root.

**HSM requirement:** The Tier 0 HSM MUST be one of:
- **Thales Luna Network HSM 7** (FIPS 140-3 L3+; for non-IL6 Tier 0 use where L4 is not
  required by the certification level) — acceptable for FedRAMP High and below
- **Entrust nShield Connect+ HSM** (FIPS 140-3 L3+) — same scope as Thales Luna
- **SafeNet Luna PCIe HSM** (FIPS 140-2 L3; legacy acceptable for ceremony backup only until
  replaced at next ceremony)
- For IL6 / NSA Type 1 paths: a **NSA-evaluated / TSEC-approved Type 1 cryptographic module**
  is required. The specific module is determined by the IL6 ceremony delegation model (see §6).

**Key material:** The Tier 0 key material is split via Shamir Secret Sharing (SSS) into shares
that are stored on individual Tier 1 HSMs (YubiHSM 2 FIPS for ceremony backup portability;
see §3). The Tier 0 HSM itself is kept in a dual-locked safe in a physically hardened ceremony
facility. See §4 for the ceremony procedure.

**Algorithm:** Ed25519 for non-IL6 paths (FIPS 186-5 includes Ed25519 since 2023); ECDSA
P-384 for FedRAMP High and below where explicit NIST P-curve requirement exists; ECDSA P-384
or algorithm per NSA Suite B for IL6 (NSA Type 1 module may not support Ed25519 — confirm per
specific approved module). Where dual-algorithm support is required (Ed25519 for cosign keyless
flow; ECDSA P-384 for FIPS-compliant paths), the org root key has TWO public-key certificates:
one Ed25519 (cosign primary chain) and one ECDSA P-384 (FIPS-compliant chain).

**Rotation cadence:** Annual ceremony (see §4). Emergency rotation triggered by key compromise
or shareholder compromise (see §4.5).

### Tier 1 — Substrate Root Signing Key (FIPS 140-3 Level 3, Networked HSM)

**Purpose:** The operational signing key for substrate-level signing operations: Cedar
fragment publication, cosign attestation of `.oab` bundles, compliance pack signing. This
key is online and used by automated pipeline operations.

**FIPS requirement:** FIPS 140-3 Level 3 minimum. Level 3 requires physical tamper evidence,
role-based authentication, and prohibits export of unencrypted key material. This meets the
requirement for FedRAMP High workloads.

**HSM vendors approved for Tier 1:**

| Vendor / Product | FIPS Level | Substrate roles | SPIFFE/SPIRE attestation | Rotation cadence |
|---|---|---|---|---|
| **AWS CloudHSM** (FIPS 140-3 L3, validated 2023) | 140-3 L3 | Cedar fragment signing; cosign `.oab` attestation; compliance pack signing in AWS-hosted cells | AWS IAM instance role → SPIFFE SVID via SPIRE AWS plugin (`x509pop` attestor); SPIRE issues SVID to CloudHSM-backed signing service | Annual key rotation; HSM cluster: no rotation needed (AWS-managed hardware) |
| **Azure Dedicated HSM** (Thales Luna 7, FIPS 140-2 L3 → moving to 140-3 L3 per Microsoft roadmap 2026) | 140-2 L3 (current); 140-3 L3 (2026 target) | Cedar fragment signing; cosign attestation in Azure-hosted cells | Azure Managed Identity → SPIFFE SVID via SPIRE Azure plugin (`azure_msi` attestor) | Annual key rotation |
| **GCP Cloud HSM** (FIPS 140-2 L3, validated) | 140-2 L3 | Cedar fragment signing; cosign attestation in GCP-hosted cells | GCP service account → SPIFFE SVID via SPIRE GCP plugin (`gcp_iit` attestor) | Annual key rotation |
| **Entrust nShield Connect+** (FIPS 140-3 L3+) | 140-3 L3+ | Substrate root in on-prem cells; FedRAMP High on-prem substrate; highest assurance non-IL6 | SPIRE TPM attestor (`tpm_devid` attestor + nShield PKCS#11 binding) | Annual key rotation |
| **Thales Luna Network HSM 7** (FIPS 140-3 L3) | 140-3 L3 | Substrate root in on-prem cells; FedRAMP High on-prem substrate; KR-CSAP (requires KR-resident hardware per §6) | SPIRE TPM attestor with Thales Luna PKCS#11 binding | Annual key rotation |
| **YubiHSM 2 FIPS** (FIPS 140-2 L3) | 140-2 L3 | **Ceremony backup only** (Tier 0 Shamir share carriers; NOT for Tier 1 production signing) | N/A — ceremony use only, not SPIRE-attested | Per-ceremony; YubiHSM 2 replaces at next annual ceremony if share count changes |

**IL5 path (DoD-CIO-approved cryptography):** AWS GovCloud CloudHSM (FIPS 140-3 L3) in an
IL5-authorized region is the approved Tier 1 HSM for IL5 workloads. All signing operations
for IL5 compliance pack fragments MUST use the GovCloud CloudHSM cluster.

**IL6 path (NSA Type 1):** IL6 requires a separate cert chain rooted in NSA-approved
hardware. See §6.3 for the IL6 ceremony delegation model. The Ed25519 org root key chain
is NOT IL6-acceptable; a separate ECDSA P-384 or algorithm per the specific NSA Type 1
module's capabilities is required.

**SPIFFE/SPIRE attestation chain:** Every Tier 1 HSM-backed signing service MUST have a
SPIFFE SVID issued by the cell's SPIRE server. The attestation plugin must bind the workload
identity to the HSM hardware attestation certificate (for CloudHSM: instance role + HSM
cluster certificate; for Thales/Entrust: TPM devID). The SPIRE-issued SVID is presented
as the `principal` in the Cedar evaluation request for root-signing operations, enabling
Cedar policy enforcement (§5).

### Tier 2 — Per-Tenant encryption-key BYOK / KMS and provider-credential BYOK envelopes (FIPS 140-2 Level 2 Minimum)

**Purpose:** Per-tenant encryption-BYOK (CMEK) and per-tenant provider-BYOK credential
envelope. Not in the substrate trust chain; isolated per-tenant.

**FIPS requirement:** FIPS 140-2 Level 2 minimum. Level 2 adds tamper evidence to Level 1's
algorithm requirements; this meets FedRAMP Moderate requirements.

**Acceptable implementations:**
- AWS KMS (FIPS 140-2 L2; optional CloudHSM-backed L3 upgrade)
- GCP Cloud KMS (FIPS 140-2 L2; HSM-backed keys use L3)
- Azure Key Vault (FIPS 140-2 L2; Premium tier uses L3)
- Tenant-supplied HSM (must present FIPS 140-2 L2 or higher certificate; validated during
  BYO-cloud onboarding)

**No ceremony required.** Tier 2 keys are provisioned automatically per-tenant during
onboarding via the tenancy substrate. Rotation is tenant-configurable (default: annual
rotation enforced by Cedar policy; tenant may shorten but not extend beyond the pack
requirement — e.g., HIPAA packs enforce ≤365-day rotation).

### Tier 3 — Per-Call Ephemeral Tokens (No FIPS Requirement)

**Purpose:** Fulcio-issued ephemeral OIDC certificates for cosign keyless flow; SPIFFE SVIDs
for workload identity; short-lived JWT tokens for service-to-service auth.

**FIPS requirement:** None. These are short-lived (≤10 minute SVID TTL; ≤5 minute Fulcio
cert TTL) and do not persist beyond a single call or session. The FIPS trust chain is
established by the issuer (Fulcio CA rooted in Tier 0/1; SPIRE root CA rooted in Tier 1),
not by the ephemeral token itself.

**Note on air-gap deployments:** Tier 3 keyless cosign flow (Fulcio + Rekor) requires OIDC
and Rekor network reachability. For air-gapped cells (ADR-0254 §D-1.5), cosign signing mode
MUST use Tier 1 KMS-backed signing (long-lived intermediate key derived from Tier 1 HSM root)
rather than the keyless Tier 3 flow. The `.oab` bundle trust-bundle (sigstore-tuf-root.json,
fulcio-root-ca.pem, rekor-pubkey.pem) embeds the offline verification material for the
receiving air-gap cell.

---

## §3. Approved HSM Vendors — Consolidated Reference

| Vendor / Product | FIPS Level | Tier(s) | Substrate Roles | Notes |
|---|---|---|---|---|
| Thales Luna Network HSM 7 | FIPS 140-3 L3 | 0 (non-IL6 ceremony), 1 (on-prem Tier 1) | All substrate signing; KR-CSAP jurisdictional requirement | KR-CSAP MUST use KR-resident Thales Luna cluster (see §6.1) |
| Entrust nShield Connect+ | FIPS 140-3 L3+ | 0 (non-IL6 ceremony), 1 (on-prem Tier 1) | All substrate signing; EU-sovereign preferred vendor | EU-sovereign cells MUST use EU-resident nShield cluster (see §6.2) |
| AWS CloudHSM | FIPS 140-3 L3 | 1 (cloud Tier 1) | Cedar/cosign/.oab signing in AWS cells; IL5 GovCloud | IL5: GovCloud us-gov-east-1 / us-gov-west-1 only |
| Azure Dedicated HSM (Thales Luna 7) | FIPS 140-2 L3 (→ 140-3 L3 in 2026) | 1 (cloud Tier 1) | Cedar/cosign/.oab signing in Azure cells | Moving to FIPS 140-3 L3 per 2026 Microsoft roadmap; track in ADR-0250 §D-8 milestone |
| GCP Cloud HSM | FIPS 140-2 L3 | 1 (cloud Tier 1) | Cedar/cosign/.oab signing in GCP cells | |
| YubiHSM 2 FIPS | FIPS 140-2 L3 | 0 (ceremony backup only) | Shamir share carriers for offline root key ceremony | NOT for production Tier 1 signing |
| NSA Type 1 module (cleared vendor pending) | NSA-evaluated (supercedes FIPS levels for IL6) | 0 (IL6 ceremony), 1 (IL6 Tier 1) | IL6-only cert chain; separate from main org root key chain | IL6 delegation model pending; see §6.3 |

---

## §4. Ceremony Procedure

The annual root key ceremony establishes or renews the Tier 0 offline ceremony key, derives
the Tier 1 substrate root signing key, and issues intermediate key certificates.

### §4.1. Participants and Quorum

**Shamir Secret Sharing:** The Tier 0 HSM key is split into **N=9 shares** with a threshold
of **M=5** (5-of-9 quorum required). Shares are distributed across ≥3 jurisdictions:

| Share set | Jurisdiction | Holder role | Count |
|---|---|---|---|
| Set A | Primary jurisdiction (HQ country) | council-security principal (2 persons) | 2 shares |
| Set B | Secondary jurisdiction (second country) | ops-compliance principal (2 persons) | 2 shares |
| Set C | Tertiary jurisdiction (third country) | External auditor (1 person) | 1 share |
| Set D | Quaternary jurisdiction (fourth country or HSM safe) | council-security backup (2 persons) | 2 shares |
| Set E | Quinary jurisdiction (fifth country or notary-sealed safe) | External legal counsel (2 persons) | 2 shares |

Any 5 of the 9 shareholders can reconstitute the key. The ≥3-jurisdiction requirement ensures
that no single country's legal process can compel access to a quorum of shares.

**Witness attestations:** At least 2 external witnesses (external auditor firm + external legal
counsel) MUST be present. Witnesses attest to:
1. Correct number of shareholders present.
2. No coercion evident.
3. HSM devices inspected for tamper evidence.
4. Ceremony recording started before key material handling.

### §4.2. Ceremony Recording Requirements

The ceremony MUST be recorded (video + audio) from before hardware inspection through final
key material sealing. The recording:
- MUST show all HSM devices before and after use.
- MUST show all shareholder acknowledgement statements.
- MUST NOT show key material on screen or in audio.
- MUST be stored in dual-encrypted form: one copy per-jurisdiction safe, one copy in
  the audit-chain's Tier 0 evidence store.

### §4.3. Ceremony Steps

1. **Pre-ceremony audit.** ops-compliance verifies all HSM devices are in tamper-evident
   packaging. Any broken tamper seal triggers ceremony abort; replacement hardware ordered.
2. **Recording start.** Video/audio recording initiated.
3. **HSM initialization.** Tier 0 HSM initialized in air-gap facility. No network devices
   present in the room (phones in Faraday bag).
4. **Key generation.** Ed25519 (non-IL6) or ECDSA P-384 (IL6/FIPS-strict) key pair generated
   on the Tier 0 HSM. Public key written to `ceremony-output/org-root-pub.pem`.
5. **Share generation.** Shamir shares generated (5-of-9). Each share written to one
   YubiHSM 2 FIPS device held by the corresponding shareholder.
6. **Intermediate key derivation.** Tier 1 substrate root signing key derived:
   - Ed25519 intermediate key generated on the target Tier 1 HSM (CloudHSM, Entrust, etc.).
   - Intermediate key certificate signed by the Tier 0 key on the air-gap HSM.
   - Certificate written to `ceremony-output/org-baseline-intermediate-cert.pem`.
   - Certificate format: X.509 v3 with custom OIDs: `oyatie-signing-scope` (OID assigned during root-signing profile registration;
     placeholder `1.3.6.1.4.1.99999.1.1` until assigned) and `oyatie-key-tier` (OID assigned during root-signing profile registration).
   - Certificate validity: 1 year + 14 days overlap for rotation.
7. **Meta-trust-root derivation** (per ADR-NNNN-foundry-meta-trust-root). A separate offline
   key for `oyatie.foundry.meta-trust-root` is generated and its intermediate key certificate
   issued. This key lives on a SEPARATE Tier 0 HSM safe, not the org root key HSM.
8. **Post-ceremony verification.** Verify round-trip: sign a test message with the Tier 1
   intermediate key; verify against the Tier 0 root public key. Assert success.
9. **Shareholder acknowledgement.** Each shareholder signs a paper receipt acknowledging
   custody of their share.
10. **HSM sealing.** Tier 0 HSM placed in tamper-evident packaging and returned to safe.
11. **Recording stop + sealing.** Recording stopped and stored per §4.2.
12. **Witness attestation signing.** Witnesses sign the ceremony report (`ceremony-output/
    ceremony-attestation.pdf`).

### §4.4. Post-Ceremony Verification

Within 24 hours of the ceremony, ops-compliance MUST verify:

```bash
# 1. Verify the intermediate key certificate against the org root public key
openssl verify \
  -CAfile ceremony-output/org-root-pub.pem \
  ceremony-output/org-baseline-intermediate-cert.pem
# Expected: org-baseline-intermediate-cert.pem: OK

# 2. Verify a cosign signature produced by the Tier 1 signing service
cosign verify \
  --key ceremony-output/org-baseline-intermediate-cert.pem \
  <test-image-digest>
# Expected: Verified OK

# 3. Verify the Rekor inclusion proof for the test cosign attestation
cosign verify \
  --key ceremony-output/org-baseline-intermediate-cert.pem \
  --rekor-url https://rekor.sigstore.dev \
  <test-image-digest> | jq '.[] | .optional.Bundle.Payload.logIndex'
# Expected: a valid logIndex > 0

# 4. Verify Cedar genesis fragment signature
oya gate verify cedar-genesis-fragment \
  --root-cert ceremony-output/org-root-pub.pem \
  --fragment microservices/policy-engine/fragments/bootstrap/genesis.cedar
# Expected: signature valid; certificate chain valid
```

### §4.5. Emergency Rotation

Emergency rotation is triggered by:
- Confirmed or suspected Tier 0 or Tier 1 private key compromise.
- Shareholder count falls below M=5 available (e.g., 5+ shareholders simultaneously
  unavailable or deceased).
- HSM hardware compromise (tamper seal broken; HSM returned from unauthorized custody).

Emergency rotation procedure:
1. Immediately rotate the **Tier 1 intermediate key** (does not require quorum; Tier 1 HSM
   admin can reissue from the existing Tier 0 certificate within its validity period).
2. Convene emergency ceremony within 48 hours for Tier 0 compromise.
3. Emergency key (`oyatie.foundry.meta-trust-root` emergency-key) used only for the duration
   of the emergency; sunset ≤ 4 hours per ADR-0243 Appendix B.
4. Audit-chain evidence emitted for all emergency key usage.

### §4.6. Drill Requirement

The ceremony runbook (`docs/runbooks/cedar-hsm-root-key-ceremony.md`) MUST be drilled at
least **twice in separate quarters** before the first real key ceremony. Drill uses test HSMs
(non-production key material) and simulates all steps including shareholder quorum assembly
and share reconstitution.

---

## §5. Cedar Policy — Root-Signing Gate

Every request to perform a Tier 0 or Tier 1 root-signing operation MUST be evaluated by
Cedar (per ADR-0243) against the following policy. This is the canonical Cedar fragment for
the root-signing gate; it lives at
`microservices/policy-engine/fragments/substrate/root-signing-gate.cedar`.

```cedar
// root-signing-gate.cedar — Tier 0/1 substrate root-signing operations gate
// Binding ADR: ADR-0243, docs/standards/fips-hsm-substrate-root-signing.md
// Version: 1.0.0
// Fragment tier: substrate (signed by org-baseline-key; Tier 1 HSM)

namespace OyatieSubstrate;

// --- Principal types ---
// oyatie.foundry.meta-trust-root — offline ceremony process principal
// oyatie.council-security.<member> — council-security principal
// oyatie.ops-compliance.<member> — ops-compliance principal
// oyatie.foundry.ci-signing-service — automated CI signing service (Tier 1)

// Tier 0 operations: Offline ceremony only. Requires human quorum.
// No automated system may exercise Tier 0 operations.
permit(
  principal in OyatieSubstrate::Group::"ceremony-quorum",
  action in [
    OyatieSubstrate::Action::"GenerateTier0Key",
    OyatieSubstrate::Action::"IssueTier1IntermediateCert",
    OyatieSubstrate::Action::"ReconstituteShares",
    OyatieSubstrate::Action::"IssueMetaTrustRootCert"
  ],
  resource is OyatieSubstrate::HsmOperation
)
when {
  // Hard MFA requirement: every ceremony participant must have MFA active
  context.mfa_verified == true &&
  // HSM quorum: at least M=5 shareholders physically present
  // (attested by ceremony controller hardware; HSM rejects below quorum)
  context.hsm_quorum_count >= 5 &&
  // Jurisdiction diversity: shareholders from ≥3 distinct jurisdictions
  context.shareholder_jurisdiction_count >= 3 &&
  // Ceremony recording active (attestation from recording device)
  context.ceremony_recording_active == true &&
  // No network devices in ceremony room (Faraday attestation)
  context.faraday_attestation == true
};

// Tier 1 operations: Automated CI signing service only. Requires MFA + HSM attestation.
// Human principals may NOT directly invoke Tier 1 signing (must go through the CI service).
permit(
  principal == OyatieSubstrate::Principal::"oyatie.foundry.ci-signing-service",
  action in [
    OyatieSubstrate::Action::"SignCedarFragment",
    OyatieSubstrate::Action::"SignArtifactBundle",
    OyatieSubstrate::Action::"SignCompliancePack",
    OyatieSubstrate::Action::"CosignAttestation"
  ],
  resource is OyatieSubstrate::HsmOperation
)
when {
  // SPIFFE SVID from Tier 1 HSM-attested signing service
  context.spiffe_svid_valid == true &&
  // SVID trust domain matches cell's SPIRE root
  context.spiffe_trust_domain == "spiffe://oyatie.internal" &&
  // HSM attestation certificate valid (CloudHSM cluster cert / Entrust PKCS#11 cert)
  context.hsm_attestation_valid == true &&
  // Signing request approved by at least 2 human reviewers
  // for new fragment publications; existing approved fragments do not require re-review
  (context.fragment_action == "re-sign" ||
   context.human_approver_count >= 2) &&
  // Not during Tier 0 ceremony (prevent accidental Tier 1 signing during ceremony)
  context.ceremony_mode_active == false
};

// Tier 2 operations: Tenant KMS operations. Tenant admin only.
permit(
  principal in OyatieSubstrate::Group::"tenant-admins",
  action in [
    OyatieSubstrate::Action::"RotateTenantCmekKey",
    OyatieSubstrate::Action::"ProvisionTenantKmsPartition",
    OyatieSubstrate::Action::"RevokeTenantCmekKey"
  ],
  resource is OyatieSubstrate::TenantKmsOperation
)
when {
  context.mfa_verified == true &&
  context.tenant_id == resource.tenant_id &&
  // Tenant must own the operation's resource
  context.principal_tenant_id == resource.tenant_id
};

// Deny everything else (fail-closed default)
forbid(
  principal,
  action in [
    OyatieSubstrate::Action::"GenerateTier0Key",
    OyatieSubstrate::Action::"IssueTier1IntermediateCert",
    OyatieSubstrate::Action::"ReconstituteShares",
    OyatieSubstrate::Action::"IssueMetaTrustRootCert",
    OyatieSubstrate::Action::"SignCedarFragment",
    OyatieSubstrate::Action::"SignArtifactBundle",
    OyatieSubstrate::Action::"SignCompliancePack",
    OyatieSubstrate::Action::"CosignAttestation"
  ],
  resource
)
unless {
  principal in OyatieSubstrate::Group::"ceremony-quorum" ||
  principal == OyatieSubstrate::Principal::"oyatie.foundry.ci-signing-service" ||
  principal in OyatieSubstrate::Group::"tenant-admins"
};
```

**Hard requirements from this Cedar policy:**

1. **Tier 0 operations REQUIRE:** MFA verified + HSM quorum ≥5 + ≥3 jurisdictions present +
   ceremony recording active + Faraday attestation (no network devices). No automated system
   can satisfy these conditions — Tier 0 is humans-only.
2. **Tier 1 operations REQUIRE:** SPIFFE SVID (machine identity) + HSM attestation + ≥2
   human approvers for new fragments (existing fragments allow re-sign without re-review).
3. **Tier 2 operations REQUIRE:** MFA + tenant ownership assertion.
4. **All other principals are denied by the forbid clause** (fail-closed).

---

## §6. Sovereign-Cell Variants

Different jurisdiction packs impose additional HSM locality and algorithm constraints beyond
the default tier model above.

### §6.1. KR-CSAP (Korea Cloud Security Assurance Program)

**Requirement:** All Tier 0 and Tier 1 signing operations for KR-resident cells MUST be
performed by KR-resident HSM clusters.

- Tier 0 ceremony for KR-CSAP: MUST use a Thales Luna Network HSM 7 physically located in
  South Korea (Seoul or Busan datacenter). The ceremony MUST be performed in Korea with
  Korean counsel as one of the ≥3 jurisdictions.
- Tier 1 substrate signing for KR-CSAP cells: MUST use a KR-resident CloudHSM equivalent
  or Thales Luna cluster. Naver Cloud HSM (FIPS 140-2 L3) is approved for Tier 1 in KR-CSAP
  cells pending FIPS 140-3 validation.
- The org root key chain for KR-CSAP cells MUST use a KR-jurisdiction intermediate key
  (subordinate CA cert issued from the global Tier 0 root; restricted to `oyatie-jurisdiction:
  KR` OID in the cert extension).
- Personal information (PI) signing for PIPA compliance: ECDSA P-256 or P-384 on the
  KR-resident Tier 1 HSM; never on the global Tier 1 HSM.

### §6.2. EU-Sovereign (GDPR + GAIA-X-relevant)

**Requirement:** All Tier 0 and Tier 1 signing operations for EU-sovereign cells MUST be
performed by EU-resident HSM clusters.

- Tier 0 ceremony for EU-sovereign: Entrust nShield Connect+ physically located in the EU
  (Netherlands, Germany, or Ireland). EU Data Protection Officer MUST be one of the ≥3
  jurisdiction witnesses.
- Tier 1 substrate signing for EU-sovereign cells: MUST use AWS CloudHSM in eu-west-1 /
  eu-central-1, or Azure Dedicated HSM in West Europe / Germany West Central, or GCP Cloud
  HSM in europe-west4. The EU-sovereign Tier 1 key material MUST NOT traverse non-EU network
  paths (enforced by HSM cluster VPC policy).
- The org root key chain for EU-sovereign cells uses an EU-jurisdiction intermediate key
  (cert extension `oyatie-jurisdiction: EU`). Data subject requests processed under GDPR
  Article 20 (data portability) must be signed by the EU-jurisdiction key chain only.

### §6.3. IL6 (NSA Type 1)

**Requirement:** IL6 certification requires a completely separate cert chain rooted in
NSA-approved Type 1 hardware. The oyatie Ed25519 org root key chain is NOT IL6-acceptable.

- The IL6 ceremony delegation model is not fully defined in this document because oyatie does
  not yet have IL6 clearance (target: 2029-2030 per ADR-0250 §D-8).
- **Placeholder requirement:** An IL6-capable cleared MSSP (e.g., Iron Bank / Platform One /
  other DISA-authorized partner) will perform the IL6 root key ceremony under the appropriate
  SCIF requirements with cleared personnel.
- Until the IL6 delegation model is defined in a follow-up ADR, no oyatie cell claims IL6
  certification and no IL6 compliance pack is issued.
- This placeholder is tracked as an open item in ADR-0250 §D-8 IL5/IL6 launch milestone
  (2029-2030).

### §6.4. CN-PIPL (China Personal Information Protection Law)

**Requirement:** CN-PIPL cells MUST use CN-resident HSMs approved by the Cyberspace
Administration of China (CAC). FIPS 140-3 L3 is not automatically recognized; the equivalent
Chinese standard is GM/T 0051-2016 (CFCA).

- Tier 1 for CN-PIPL cells: CFCA-approved HSM cluster (vendor pending per CN-specific procurement;
  Westone / BJCA are candidate vendors). The CN-PIPL Tier 1 key uses SM2 (Chinese elliptic
  curve) rather than Ed25519 for PIPL-regulated operations.
- The org root key chain for CN-PIPL cells uses a CN-jurisdiction intermediate key.
- CN-PIPL is planned for Year 3+ (ADR-0250 §D-8); vendor selection pending.

### §6.5. AU-IRAP (Australia Information Security Registered Assessors Program)

**Requirement:** AU-IRAP PROTECTED and OFFICIAL: Sensitive cells MUST use AU-resident HSMs.

- Tier 1: AWS CloudHSM in ap-southeast-2 (Sydney) is the approved AU-IRAP Tier 1 HSM.
- The org root key chain for AU-IRAP cells uses an AU-jurisdiction intermediate key
  (cert extension `oyatie-jurisdiction: AU`).
- AU-IRAP ceremony participant MUST include an IRAP assessor or an ASD-approved auditor.

---

## §7. CI Lane

**Lane name:** `oya-governance-fips-hsm-substrate-root`

**Status:** Advisory until 2026-07-15; BLOCKER thereafter (matching the overall doc-rigor
enforcement timeline per `docs/standards/documentation-rigor.md` §8).

**What the lane checks:**

1. Every substrate root-signing operation invocation (Tier 0/1) in CI logs carries a
   `hsm_tier: [0|1]` tag and a `fips_level: [140-3-L3|140-3-L4|140-2-L3]` tag in the
   audit-chain emission.
2. Every `cosign sign` or `cosign attest` invocation in any CI pipeline is routed through
   the Tier 1 signing service (not via a raw key file or environment variable). Static
   analysis: grep for `cosign sign --key file://` in any Makefile / GitHub Actions YAML;
   FAIL if found.
3. The Cedar fragment `root-signing-gate.cedar` is present and parseable in the policy-engine
   fragment registry.
4. The intermediate key certificate in `ceremony-output/org-baseline-intermediate-cert.pem`
   has not expired (validity check; fail 30 days before expiry to give ceremony scheduling
   time).
5. For KR-CSAP/EU-sovereign cells: HSM cluster region tag in cell manifest matches the
   required jurisdiction region.

**Lane configuration:**

```yaml
# iac/ci-lanes/fips-hsm-substrate-root.yaml
lane: oya-governance-fips-hsm-substrate-root
enforcement_status: advisory  # → BLOCKER 2026-07-15
checks:
  - id: audit-hsm-tier-tag
    description: Every root-signing audit event carries hsm_tier + fips_level tags
    query: audit_chain_query("event_class:HsmOperation AND NOT hsm_tier:*")
    expected: count == 0
  - id: no-raw-key-cosign
    description: No CI pipeline uses cosign with a raw key file
    grep_pattern: 'cosign sign --key file://'
    search_paths: [".github/workflows/", "Makefile", "scripts/"]
    expected: no_matches
  - id: root-signing-cedar-fragment-present
    description: root-signing-gate.cedar present and parseable
    path: microservices/policy-engine/fragments/substrate/root-signing-gate.cedar
    expected: exists_and_valid_cedar
  - id: intermediate-cert-not-expired
    description: Tier 1 intermediate cert not expired or within 30-day warning window
    cert_path: ceremony-output/org-baseline-intermediate-cert.pem
    warn_days_before_expiry: 30
    fail_days_before_expiry: 0
  - id: sovereign-cell-hsm-region-match
    description: KR-CSAP/EU-sovereign/AU-IRAP cell manifests use correct HSM region
    policy: sovereign_cell_hsm_region_policy
    expected: all_cells_compliant
```

---

## §8. Verification — What an Intern Should Do to Confirm Correct Setup

An intern with zero prior oyatie knowledge can verify the FIPS/HSM setup is correctly
configured by following these steps in order:

### Step 1: Verify the Tier 1 signing service is running and HSM-attested

```bash
# Check the signing service pod has a valid SPIFFE SVID
kubectl -n policy-engine get pod -l app=cedar-signing-service \
  -o jsonpath='{.items[0].metadata.annotations.spiffe-svid-expiry}'
# Expected: a timestamp in the future

# Verify the SPIFFE SVID matches the expected trust domain
oya gate validate spiffe-svid \
  --workload cedar-signing-service \
  --expected-trust-domain spiffe://oyatie.internal
# Expected: PASS

# Verify HSM attestation (CloudHSM example)
aws cloudhsm describe-clusters --region us-east-1 \
  --output json | jq '.Clusters[].State'
# Expected: "ACTIVE"
```

### Step 2: Verify the org root certificate chain

```bash
# Verify the Tier 1 intermediate cert chains to the org root
openssl verify \
  -CAfile ceremony-output/org-root-pub.pem \
  ceremony-output/org-baseline-intermediate-cert.pem
# Expected: org-baseline-intermediate-cert.pem: OK

# Check certificate expiry (must be > 30 days away)
openssl x509 -in ceremony-output/org-baseline-intermediate-cert.pem \
  -noout -dates
# Expected: notAfter > today + 30 days
```

### Step 3: Verify the Cedar root-signing gate fragment is active

```bash
# Check the fragment is in the policy-engine registry
oya policy get-fragment \
  --fragment-id substrate/root-signing-gate \
  --status
# Expected: status: ACTIVE; tier: substrate

# Verify the fragment's signature against the org root cert
oya gate verify cedar-fragment-signature \
  --fragment-id substrate/root-signing-gate \
  --cert ceremony-output/org-baseline-intermediate-cert.pem
# Expected: PASS
```

### Step 4: Verify the CI lane passes

```bash
# Run the FIPS/HSM CI lane in advisory mode
oya gate run fips-hsm-substrate-root --advisory
# Expected: All checks PASS; any warnings shown but not blocking
```

### Step 5: Verify a sovereign-cell HSM region (KR-CSAP example)

```bash
# For a KR-CSAP cell, verify the HSM cluster is KR-resident
oya gate validate sovereign-cell-hsm-region \
  --cell kr-csap-seoul-1 \
  --expected-region ap-northeast-2
# Expected: PASS — HSM cluster in ap-northeast-2 (Seoul)
```

If any step fails, follow the remediation in `docs/runbooks/cedar-hsm-root-key-ceremony.md`
or escalate to `ops-compliance` + `council-security`.

---

## §9. Anti-Patterns

| Anti-pattern | Why it fails | Fix |
|---|---|---|
| Using `cosign sign --key file://key.pem` in CI | Private key material in file system; not HSM-protected; FIPS non-compliant; CI lane will FAIL | Route all cosign signing through the Tier 1 signing service (`oya signing-service sign`) which holds the key in the HSM |
| Single-jurisdiction Shamir shares | Single government can compel all shares via one legal process | Distribute shares across ≥3 jurisdictions; minimum per §4.1 |
| Using the same org root key for all certification levels | IL6 requires NSA Type 1 module; Ed25519 org root is not IL6-acceptable | Maintain separate cert chains per certification level for IL5/IL6 (§6.3) |
| Air-gap cell using keyless cosign (Fulcio + Rekor) | Air-gap has no OIDC/Rekor network path; cosign verify fails without Rekor reachability | Use KMS-backed cosign (Tier 1 long-lived key) for air-gap; embed trust-bundle in `.oab` |
| Tier 2 (tenant KMS) key used for substrate signing | Tier 2 is tenant-scoped; substrate signing requires the global org trust chain | Substrate signing MUST use Tier 1 only; Cedar gate enforces this at policy level |
| Skipping ceremony drills | First real ceremony with untrained shareholders increases shareholder error risk | ≥2 drills required per §4.6 before first real ceremony |
| Sovereign cell using non-jurisdictional HSM | KR-CSAP, EU-sovereign, AU-IRAP require jurisdictionally resident HSMs | CI lane `oya-governance-fips-hsm-substrate-root` check `sovereign-cell-hsm-region-match` enforces this |

---

## §10. Cross-References

- `ADR-0243-cedar-as-universal-gate.md` — Cedar genesis fragment signing; Cedar gate on
  fragment publication; §D-5 signing chain bootstrap
- `ADR-0247-self-hosting-self-modification-doctrine.md` — org root key in Tier 0 HSM; Shamir
  M-of-N (now 5-of-9 per this standard); meta-trust-root key
- `ADR-0251-compliance-pack-cell-certification-levels.md` — FIPS level requirements per
  certification level (FedRAMP Moderate L2; FedRAMP High L3; IL5 DoD-CIO; IL6 NSA Type 1)
- `ADR-0254-deployment-model-spectrum.md` — air-gap bundle signing mode; KMS-backed vs
  keyless cosign decision per deployment model
- `ADR-NNNN-foundry-meta-trust-root` — meta-trust-root key (sibling ADR authored by Slice 2;
  Tier 0/1 HSM requirement for the 2-human-approval gate)
- `docs/runbooks/cedar-hsm-root-key-ceremony.md` — operational runbook for the annual
  ceremony; must be updated to reference this standard as the authority for FIPS levels
- `docs/performance-budgets/cedar-hot-path-1ms-p99.md` — Cedar performance does not depend
  on HSM tier (HSM operations are not on the hot path; only at ceremony/fragment-publish time)
- `docs/standards/documentation-rigor.md` — this standard meets the Standard doc class bar
  (≥250 lines, §1–§10, RFC-2119 language, enforcement lane, anti-patterns table)
