---
doc_class: PolicySpec
title: Seal Integrity + HSM Signing Policy
microservice: audit-chain
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-security + axis-audit-chain + cloud-secrets
deciders: council-architecture, ops-security, axis-audit-chain, council-privacy
related_adrs: [ADR-0028, ADR-0003, ADR-0117, ADR-0131, ADR-0140 (retired per ADR-0145)]
related_artifacts:
  - microservices/audit-chain/threat-model.md (T-T-01..T-T-06, T-S-02, T-E-02)
  - microservices/audit-chain/dpia.md (R-08)
  - microservices/audit-chain/policy/tenant-scope.cedar
  - microservices/audit-chain/runbooks/hsm-key-rotation.md
review_cadence: quarterly + on every HSM/cryptography library upgrade + after every key rotation
doc_status: published
---

# Seal Integrity + HSM Signing Policy (audit-chain µservice)

## Purpose

Define the load-bearing integrity invariants of the audit-chain's Merkle-tree sealing + HSM-backed Ed25519 signing. This document is the authoritative reference for SOC 2 examiners (CC6.6 / CC7.1 / CC8.1), ISO 27001 auditors (A.5.17 / A.8.5 / A.8.7 / A.8.24), GDPR Art. 32 reviewers, eIDAS AdES reviewers, KR 전자문서법 reviewers, and HIPAA §164.312(c)(1) reviewers asking *"how does oyatie's audit-chain achieve non-repudiation and tamper-evidence?"*

This document inherits Bominal ADR-0028 (Merkle + Ed25519 model) and ADR-0003 (emission contract) 1:1 and overlays the oyatie-specific HSM operational + key-rotation envelope.

## Merkle-Tree Invariants

### Invariant SI-01: Tree shape per RFC 6962

The Merkle tree per `(pack, tenant_partition, period)` is a RFC-6962-shaped SHA-256 binary tree with:
- Leaf hash = `sha256(0x00 || leaf_data)` where `leaf_data = canonical_serialize(EventEnvelope)`.
- Internal node = `sha256(0x01 || left || right)`.
- Odd-leaf handling: duplicate the last leaf to make pairs.

The 0x00 / 0x01 prefix bytes prevent second-preimage attacks per RFC 6962 §2.1.

CI lane: `oya-check-audit-chain-merkle-shape` validates the implementation against RFC 6962 reference vectors.

### Invariant SI-02: Period boundaries

Each `(pack, tenant_partition)` advances through periods (default 1s; pack-overrideable up to 60s for trial-tier tenants).

A period is **sealed** when:
1. The period's wall-clock has elapsed.
2. emission-rest has reported all events for the period are durable in WAL + S3.
3. sealing-worker has computed the Merkle root.
4. HSM has signed the root.
5. SealRecord is durable in Postgres + S3 WORM.
6. Root is published to Mimir + GitHub-pinned manifest.

Until step 6 completes, the period is "unsealed"; events in unsealed periods are emit-and-receipt-only — they have an `event_id` and durable storage but not yet a published proof. Tenants can query unsealed events; the response carries `sealed: false`. Per Bominal ADR-0028 §"Eventual consistency contract".

### Invariant SI-03: Root chaining

Per Bominal ADR-0028 §"Chain-of-roots": each period's signed root carries a reference (`prior_root_hash`) to the previous period's signed root for the same `(pack, tenant_partition)`. The genesis root for each `(pack, tenant_partition)` is a deterministic constant (`sha256("oyatie-audit-chain-genesis|<pack>|<tenant_partition>|<epoch_id>")`).

This means:
- Tampering any one root requires tampering every subsequent root (cascade).
- Verifiers can walk the chain from any sealed period back to genesis.
- Replay attacks (substitution of an old signed root) are detected by the chain hash mismatch.

CI lane: `oya-check-audit-chain-root-chaining` validates the chain in the published manifests.

### Invariant SI-04: Three-channel root publication

Every sealed root is published to **three independent channels**:
1. **Postgres + S3 WORM** (canonical source-of-truth; oyatie-controlled; immutable per Object Lock Compliance mode).
2. **Mimir `oya_audit_chain_root_*` series** (`oya-self` + `oya-aggregate` reserved tenants; tamper-evident operational channel; observability µservice's SLO model gates Mimir tampering).
3. **GitHub-pinned `evidence/audit-chain-roots/<pack>/<epoch>.json`** (git history is the third-party-verifiable channel; signed-commit + branch-protection enforced).

Tampering one without the others is **deterministically detected** by the cross-channel verification rule (`oya:audit_chain_root_cross_channel_match:rate`). The rule alerts at any mismatch.

Per Bominal ADR-0028 §"External transparency": three-channel publication is the substitute for an external "transparency log" service that we cannot rely on at deployment time.

### Invariant SI-05: Genesis recording

The genesis root for every `(pack, tenant_partition)` is recorded at tenant onboarding in a special unrelinquishable record. This record is published to all three channels with a marker `genesis: true`. Verifiers can walk backward from any sealed root to genesis and confirm the chain is rooted.

## HSM Signing Policy

### Invariant SI-06: Per-pack HSM partition

Each pack has its own OCI Cloud-HSM partition. The partition holds the active + retiring signing-keys for that pack's chains. No cross-pack HSM access; pack-pinning extends to the HSM layer.

### Invariant SI-07: Ed25519 algorithm

Per Bominal ADR-0028: signing algorithm is Ed25519 (RFC 8032). Choice rationale:
- Small signatures (64 bytes); efficient verification.
- Deterministic (no entropy dependency at signing time; reduces side-channel surface).
- Standardised; mature OSS implementations (`ed25519-dalek` v2+).
- HSM-native support across OCI / AWS / GCP / Azure cloud-HSM offerings.
- eIDAS Art. 26 AdES-compatible.

Key-strength: 256-bit (Ed25519 native).

### Invariant SI-08: Private key never leaves HSM

The private key is generated inside the HSM partition; signing is performed by remote PKCS#11 / KMIP calls. The key never appears in oyatie-process memory.

Verification: only public keys are exported; public keys are published in all three channels per SI-04 + SI-10.

### Invariant SI-09: Signing call authenticated by SPIFFE

Each PKCS#11 session is authenticated with a short-lived (≤ 24h) certificate issued by OpenBao; the certificate carries the sealing-worker SPIFFE identity. OCI Cloud-HSM IAM matches the SPIFFE identity to the partition's signing-key access policy.

CI lane: `oya-check-audit-chain-hsm-iam-conformance` validates the partition's IAM declaration matches the expected SPIFFE-bound identity.

### Invariant SI-10: Key rotation cadence and overlap

Per ISO 27001 A.5.17 + Bominal ADR-0028:
- Active signing key rotated every 90 days.
- 24-hour overlap window before old key is retired: both keys can sign during overlap; both keys' signatures are accepted by verification during their respective active epoch.
- Retired keys are NOT deleted; they remain in HSM partition in retired-state. Verification of pre-rotation events uses the period's active key (lookup via KeyResolver).
- Key-generation + rotation are themselves audit-emitted as `KeyRotated` events; the rotation event is signed by BOTH the outgoing and incoming key (chain-of-trust at rotation boundary).

Per Bominal ADR-0028 §"No retroactive re-sign": old roots are NEVER re-signed by the new key; the original signature stands. KeyResolver maps `(pack, tenant_partition, period_id) → public_key` so verification knows which public key to use.

### Invariant SI-11: Key-rotation transparency

Every key rotation is published to:
1. SealRecord in Postgres + S3 (record-of-rotation event).
2. Mimir `oya_audit_chain_key_rotation_total` + a special `oya_audit_chain_active_signing_key_fingerprint` info-metric series.
3. GitHub-pinned `evidence/audit-chain-keys/<pack>/<epoch>.json` carrying the new public key + retirement-effective-period of the old key.

Tenants + auditors verify against the published public key for the relevant period.

CI lane: `oya-check-audit-chain-key-publication` validates all three channels reflect the same rotation event.

### Invariant SI-12: 2-person rule for key lifecycle operations

- Key generation: 2-person OpenBao JIT elevation (requester + ops-security approver).
- Key rotation: 2-person.
- Key retirement (post-overlap): 2-person.
- Key destruction (post-retention-window — typically pack retention + 1y): 2-person.
- Emergency key revocation (compromise suspected): 2-person + ExecSponsor.

Every 2-person operation is itself audit-chained.

## Verification Invariants

### Invariant SI-13: KeyResolver authority

For verification, the KeyResolver port (declared in `oya-audit-chain-verification-kernel`) maps `(pack, tenant_partition, period_id) → public_key`. The mapping is derived from the published GitHub-pinned + S3-mirrored `evidence/audit-chain-keys/` manifest.

If a SealRecord references a key not in the resolver's mapping (e.g., a key claimed to be active at a period when that key wasn't active per the rotation log), verification returns `verified: false` with `reason: key_epoch_mismatch`.

### Invariant SI-14: Inclusion-proof verification correctness

Per Bominal ADR-0028 §"Inclusion proof shape":
```
verify(event_envelope, claimed_root, claimed_proof, claimed_signature, claimed_period_id, claimed_pack, claimed_tenant_partition):
  1. resolve public_key via KeyResolver
  2. verify Ed25519(claimed_signature, claimed_root, public_key) → must return true
  3. compute leaf_hash from event_envelope (RFC-6962-shaped)
  4. walk Merkle proof: starting from leaf_hash, hash each level with proof siblings → must reach claimed_root
  5. verify chain: claimed_root.prior_root_hash chains to a published root for (pack, partition, period_id - 1)
  6. all 5 checks pass ⇒ verified: true
  any check fails ⇒ verified: false + structured reason
```

Property test: 10k random EventEnvelope + 10k random mutations; every mutation must classify `verified: false`. CI gate.

### Invariant SI-15: Verification is pure-function

verify() reads only the published roots + public keys; never mutates state. This means:
- Anyone can verify (tenants, auditors, public).
- Verification can be performed offline given the published artefacts.
- Reference implementation lives at `microservices/audit-chain/src/crates/oya-audit-chain-verification-sdk/`; future open-source decision per `sdk-plan.md`.

## Failure Modes

### FM-SI-01: HSM partition unreachable

Behaviour: sealing-worker enters degraded mode; emission continues; events accumulate in unsealed buffer. SLI `audit_chain_unsealed_buffer_depth_seconds` alarms at > 60s.

Recovery: HSM partition restored OR failover to DR-pair pack (where applicable). On restore, sealing-worker batch-seals accumulated events.

Tenant impact: emit() succeeds with `sealed: false`; sealed status follows once HSM restored. Tenant-facing dashboard surfaces this.

### FM-SI-02: HSM signing call returns mismatched signature

Behaviour: sealing-worker verifies every signature it receives against the same public key + root locally before storing the SealRecord. Mismatch → Sev-1 alarm; HSM partition quarantined; ops-security paged.

Recovery: Diagnose HSM partition health; if compromise suspected, emergency key revocation + rotate to new partition.

### FM-SI-03: Cross-channel root divergence

Behaviour: `oya:audit_chain_root_cross_channel_match:rate < 1.0` alarm. Possible causes: Mimir tampering; GitHub manifest tampering; S3 tampering; sealing-worker bug.

Recovery: Halt sealing; investigate which channel is divergent; declare Sev-1; engage ops-security.

### FM-SI-04: Key rotation overlap window expired without key retirement

Behaviour: scheduled-job alarm: "Pack <X> key <K> overlap expired without retire".

Recovery: Manually verify retirement is safe (no pending sealing operations against the old key); execute retirement via 2-person OpenBao JIT.

### FM-SI-05: Genesis record mismatch

Behaviour: At sealing-worker startup, the worker reads the genesis record from all three channels; mismatch ⇒ Sev-1 halt.

Recovery: Genesis records are designed to be unrelinquishable; mismatch indicates fundamental tampering. Engage ExecSponsor + ops-security.

## Audit Trail

Every key-lifecycle + sealing-correctness event is itself audit-chained per Bominal ADR-0028:

| Event | Emitter | Fields | Retention |
|---|---|---|---|
| `SealMinted` | sealing-worker | `pack, tenant_partition, period_id, root_hash, signature, signer_public_key_fp, signed_at` | indefinite |
| `KeyRotated` | sealing-worker + ops-security | `pack, prior_key_fp, new_key_fp, overlap_start_at, overlap_end_at, retired_at?` | indefinite |
| `VerificationFailed` | verification-rest | `event_id, claimed_proof, failure_reason, requested_at` | ≥ pack retention |
| `HsmSigningError` | sealing-worker | `pack, hsm_partition_id, error_class, attempted_at` | ≥ 1y |
| `CrossChannelDivergence` | continuous-validator | `pack, period_id, divergent_channel, detected_at` | ≥ pack retention |
| `GenesisRecordMismatch` | sealing-worker (boot) | `pack, tenant_partition, mismatched_channels` | indefinite |
| `KeyRevoked` | ops-security | `pack, key_fp, reason, revoked_at` | indefinite |

## Per-Pack Overlay

### pack-kr (KR-ISMS-P + KR PIPA + 전자문서법)

- KR 전자문서법 Art. 5 (electronic-document integrity): satisfied by Ed25519 + Merkle + three-channel publication.
- KR 전자문서법 Art. 6 (electronic-document storage): satisfied by S3 WORM + per-pack retention.
- KR 전자문서법 Art. 7 (electronic-document verification): satisfied by KeyResolver + verification SDK.
- KR PIPA Art. 29-2 (encryption): AES-256 SSE + TLS 1.3 + Ed25519 chain integrity.
- KR-ISMS-P §2.6 (암호화): same.
- KR-ISMS-P §2.12 (위반관리): Sev-1 escalation per `incident-response.md`.

### pack-us-healthcare (HIPAA)

- §164.312(b) audit controls — this entire µservice is the implementation.
- §164.312(c)(1) integrity — Ed25519 + Merkle + WORM.
- §164.316(b)(2) 6y retention — retention-cascade enforces.

### pack-eu (GDPR + EDPB + eIDAS)

- Art. 32(1)(a) pseudonymisation — subject_hash + per-pack salt.
- Art. 32(1)(b) confidentiality + integrity — chain integrity per this doc.
- Art. 32(1)(c) availability — HA emission + DR-pair sealing.
- Art. 32(1)(d) regular testing — quarterly rotation drill + annual pen-test.
- eIDAS 910/2014 Art. 26 AdES — Ed25519-HSM satisfies; declared in EU-pack legal binding documentation.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/audit-chain-seal-integrity-overlay.md`.

## Verification

- `buck2 build //:quality-lane-registry-authority-check # lane=audit-chain-merkle-shape` — exit 0.
- `buck2 build //:quality-lane-registry-authority-check # lane=audit-chain-root-chaining` — exit 0.
- `buck2 build //:quality-lane-registry-authority-check # lane=audit-chain-hsm-iam-conformance` — exit 0.
- `buck2 build //:quality-lane-registry-authority-check # lane=audit-chain-key-publication` — exit 0.
- `buck2 build //:quality-lane-registry-authority-check # lane=audit-chain-cross-channel-match` — exit 0.
- Quarterly key-rotation drill: rotate pack-kr key; verify pre- and post-rotation events both verifiable.
- Annual pen-test against the verification correctness boundary.

## References

- Bominal ADR-0028 (Audit chain).
- Bominal ADR-0003 (Emission contract).
- ADR-0117 (cloud-native infra).
- ADR-0140 (Cedar policy).
- `microservices/audit-chain/threat-model.md` T-T-* + T-S-02 + T-E-02.
- `microservices/audit-chain/runbooks/hsm-key-rotation.md`.
- RFC 6962 (Certificate Transparency Merkle-tree).
- RFC 8032 (Ed25519).
- eIDAS 910/2014 + EU 2015/1502 implementing.
- KR 전자문서법 — `law.go.kr`.
- HIPAA 45 CFR §164.312.
- OCI Cloud-HSM docs.
