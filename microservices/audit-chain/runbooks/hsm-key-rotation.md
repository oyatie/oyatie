---
doc_class: Runbook
title: HSM key rotation
microservice: audit-chain
severity: "Sev-2 normal-rotation; Sev-1 emergency revocation"
status: Accepted
owner_team: ops-security + axis-audit-chain + cloud-secrets
date: 2026-05-17
related_artifacts:
  - microservices/audit-chain/policy/seal-integrity.md §"SI-10..SI-12"
  - microservices/audit-chain/threat-model.md T-T-05, T-E-02
  - microservices/audit-chain/failure-modes.md FM-01, FM-13
doc_status: published
---

# Runbook: HSM key rotation

## Purpose

Scheduled 90-day rotation of pack HSM Ed25519 signing keys + emergency revocation procedure when compromise is suspected. Per Bominal ADR-0028 §"Chain-of-trust on rotation" + `policy/seal-integrity.md` §"SI-10..SI-12".

## Cadence

- **Scheduled rotation**: every 90 days per pack.
- **Overlap window**: 24 hours from new-key activation to old-key retirement.
- **Emergency revocation**: any time compromise suspected.

## Roles

- Requester: ops-security or axis-audit-chain operator
- Approver: ops-security director (or council-architecture chair for emergency)
- HSM Operator: cloud-secrets µservice on-call
- Witness: a third operator (recorded in chain) for emergency revocation

## Scheduled Rotation — Pre-checks

1. Verify no pending sealing operations queued against the current key beyond the next 24h window: `oya_audit_chain_unsealed_buffer_depth_seconds < 60` for ≥ 1h.
2. Verify HSM partition health: `oya_audit_chain_hsm_avail{pack=<pack>} == 1` for ≥ 24h.
3. Verify scheduled rotation calendar entry exists (every quarterly drill is scheduled in advance).
4. Verify approver availability + 2-person rule readiness.

## Scheduled Rotation — Procedure

| Phase | Step | Time |
|---|---|---|
| 1 | Requester invokes JIT-elevation via OpenBao: `openbao auth-jit elevate --scope audit-chain-hsm-rotate --pack <pack> --justification "<reason>"` | ≤ 5 min |
| 2 | Approver receives notification; reviews; approves: `openbao auth-jit approve <request-id>` | ≤ 30 min |
| 3 | HSM Operator + Requester (2-person) generate new key in the pack's HSM partition via OCI Cloud-HSM CLI: `oci kms key create --partition <pack-hsm-partition> --algorithm Ed25519` | ≤ 5 min |
| 4 | Verify the new key's public-key extracted; sign-once test against a synthetic root | ≤ 2 min |
| 5 | Invoke rotation: `cargo run -p oya-dev-cli -- audit-chain key-rotate --pack <pack> --new-key-fp <fp>`. The CLI: <br>  a. signs a `KeyRotated` event with BOTH the outgoing and incoming key (chain-of-trust per Bominal ADR-0028 §"Chain-of-trust on rotation"); <br>  b. updates the KeyResolver to register the new key as active from `now()`; <br>  c. publishes the new public key to S3 + Mimir + GitHub-pinned manifest; <br>  d. emits `KeyRotated` event to AsyncAPI bus; <br>  e. records the rotation in the chain (sealed by the new key — establishes new key's validity); <br>  f. begins 24h overlap window. | ≤ 5 min |
| 6 | Verify three-channel publication of the new public key: `cargo run -p oya-dev-cli -- audit-chain key-verify-publication --pack <pack> --epoch <new-epoch-id>` | ≤ 2 min |
| 7 | Monitor sealing during overlap window: confirm new key is being used for new periods; old key handles only retroactive sealing within the overlap window if needed | 24h |
| 8 | At end of overlap (24h after Phase 5): retire the old key. Invoke: `cargo run -p oya-dev-cli -- audit-chain key-retire --pack <pack> --key-fp <old-fp>`. CLI: <br>  a. verifies no pending operations against old key; <br>  b. marks old key as retired in HSM; <br>  c. retains key in HSM partition (not destroyed — needed for verification of past events); <br>  d. emits `KeyRotated` event with `retired_at` populated. | ≤ 5 min |
| 9 | Verify retirement: `oya_audit_chain_active_signing_key_fingerprint{pack=<pack>} == <new-fp>`; old fingerprint marked retired in KeyResolver | ≤ 5 min |
| 10 | Update calendar entry for next rotation (90 days from now) | ≤ 5 min |

## Verification (post-rotation)

- `oya_audit_chain_active_signing_key_fingerprint{pack=<pack>} == <new-fp>` for ≥ 5 min.
- Three-channel publication of new key visible: S3 + Mimir + GitHub manifest match.
- Sample sealing call returns signature verifying with new public key.
- Verification of a pre-rotation event still succeeds with old public key (via KeyResolver).
- Verification of a post-rotation event succeeds with new public key.
- `KeyRotated` event in chain audit-trail (verifiable).

## Emergency Revocation — Pre-checks

1. Compromise indicator confirmed: typically `oya_audit_chain_hsm_signing_mismatch_total > 0` OR OCI HSM-side audit anomaly OR ops-security investigation finding.
2. ExecSponsor engaged (Sev-1).
3. Cryptography SME available.

## Emergency Revocation — Procedure

| Phase | Step | Time |
|---|---|---|
| 1 | Declare Sev-1; engage ExecSponsor + ops-security director + Cryptography SME | ≤ 5 min |
| 2 | HALT sealing-worker for affected partition: `kubectl scale deployment sealing-worker-<pack> --replicas=0` | ≤ 2 min |
| 3 | 3-person rule (requester + approver + witness) generates new key in a fresh HSM partition (NOT the compromised partition): `oci kms partition create ... && oci kms key create ...` | ≤ 30 min |
| 4 | Invoke emergency revocation: `cargo run -p oya-dev-cli -- audit-chain key-revoke-emergency --pack PACK --compromised-key-fp FP --new-key-fp NEW_FP --new-partition NEW_PARTITION --justification "INC-YYYYMMDD-NNN per incident-response.md Sev-1 record"`. CLI: <br>  a. emits `KeyRevoked` event signed by the new key (cannot trust the compromised key); <br>  b. updates KeyResolver to mark compromised key as revoked-from `INCIDENT_TIME` (any events sealed by it after this time are invalid); <br>  c. publishes revocation record to all three channels; <br>  d. enables the new key as active. | ≤ 10 min |
| 5 | Resume sealing-worker against new partition: `kubectl scale deployment sealing-worker-<pack> --replicas=8 ...` | ≤ 5 min |
| 6 | Verify three-channel publication of revocation + new key | ≤ 5 min |
| 7 | Forensic investigation: preserve compromised partition state for Oracle forensic team; engage incident-response chain per `incident-response.md` | ongoing |
| 8 | Tenant notification: status page + email per `incident-response.md` Sev-1 template | ≤ 30 min |
| 9 | Regulatory notification (if applicable per pack): GDPR 72h, HIPAA per BAA, KR PIPC 72h | per `compliance.md` |

## Verification (post-revocation)

- `oya_audit_chain_active_signing_key_fingerprint{pack=<pack>} == <new-fp>`.
- New partition + new key in HSM.
- Compromised key marked revoked-from `<incident-time>` in KeyResolver.
- Events sealed by compromised key BEFORE incident-time remain verifiable (legitimate).
- Events sealed by compromised key AFTER incident-time return `verified=false` with `reason=key_epoch_mismatch` (correctly).
- Postmortem published within 5 business days.

## References

- Bominal ADR-0028 §"Chain-of-trust on rotation".
- `microservices/audit-chain/policy/seal-integrity.md` §"SI-10..SI-12".
- `microservices/audit-chain/threat-model.md` T-T-05 + T-E-02.
- `microservices/audit-chain/failure-modes.md` FM-01 + FM-13.
- OCI Cloud-HSM CLI docs.
- ISO 27001 A.5.17 (cryptographic key management).
- NIST SP 800-57 (key management).
