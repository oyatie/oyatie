---
doc_class: Runbook
title: Signature verification failure (tamper detection)
microservice: audit-chain
severity: Sev-1
status: Accepted
owner_team: ops-security + axis-audit-chain + Cryptography SME
date: 2026-05-17
related_artifacts:
  - microservices/audit-chain/failure-modes.md (FM-02, FM-10)
  - microservices/audit-chain/policy/seal-integrity.md
  - microservices/audit-chain/incident-response.md
  - microservices/audit-chain/runbooks/hsm-key-rotation.md
doc_status: published
---

# Runbook: Signature verification failure

## Purpose

Recovery for two related Sev-1 conditions:
- FM-02: HSM returns a signature that fails local-verification (potential HSM compromise).
- FM-10: Verification-failed alert spikes (potential tampering of chain artifacts).

## Trigger

- FM-02: sealing-worker `oya_audit_chain_hsm_signing_mismatch_total > 0`.
- FM-10: `oya_audit_chain_verification_failed_total` rate > 0.1/s sustained 5min OR concentrated failures on a specific period range OR pattern of `reason=signature_invalid`.

## Severity

**Sev-1 always.** Treat as tamper-suspect until evidence proves otherwise.

## FM-02 — HSM signing mismatch — Procedure

### Phase 1: Halt + Quarantine (≤ 5 min)

1. Declare Sev-1; open `#inc-<id>`.
2. Engage IC (ops-security) + Cryptography SME + ops-sre-reliability + axis-audit-chain.
3. HALT sealing-worker for affected pack: `kubectl scale deployment sealing-worker-<pack> --replicas=0`.
4. Quarantine HSM partition: mark in OpenBao as `quarantine=true`; no further signing operations until cleared.
5. Capture forensic state: dump current sealing-worker state, HSM-side OCI audit log, partition state.

### Phase 2: Diagnose

| Suspect cause | Indicator | Action |
|---|---|---|
| HSM hardware fault | OCI status page shows HSM-tier degraded | Engage Oracle; consider failover to DR-pair partition |
| Cryptography library bug | Recent `ring` / `ed25519-dalek` / `sha2` version bump | Pin to last-known-good version; engage axis-audit-chain |
| Compromised partition | OCI audit log shows unauthorized PKCS#11 sessions | Treat as breach; emergency revocation (`runbooks/hsm-key-rotation.md` Emergency Revocation) |
| Sealing-worker bug | Local-verify code path bug producing false-positive mismatch | Engage axis-audit-chain; verify by running canonical test vectors |

### Phase 3: Recover

Depending on cause:
- **Hardware fault**: failover to DR-pair partition or wait for Oracle recovery; restart sealing-worker.
- **Library bug**: pin to known-good version; redeploy; restart sealing-worker.
- **Compromised partition**: emergency key revocation per `runbooks/hsm-key-rotation.md`. New partition + new key + new chain epoch.
- **Worker bug**: fix; re-deploy; restart.

### Phase 4: Validate

- Sample sealing call against a synthetic root produces a signature that locally-verifies.
- Three consecutive sealing cycles succeed without mismatch.
- Resume normal sealing.

### Phase 5: Notify

Per `incident-response.md` Sev-1 template.

## FM-10 — Verification-failed spike — Procedure

### Phase 1: Classify failures

Query `oya_audit_chain_verification_failed_total` by `reason` label:

| reason | Likely cause |
|---|---|
| `signature_invalid` | tamper-suspect; same investigation as FM-02 |
| `proof_invalid` | tamper-suspect OR client-side error (verifier passing wrong proof); investigate caller pattern |
| `root_chain_invalid` | chain-link broken; possible cross-channel divergence (`runbooks/merkle-seal-recovery.md`) |
| `key_epoch_mismatch` | client using wrong public key OR KeyResolver published incorrect epoch; check publication state |
| `event_not_found` | client probing for events that don't exist OR retention-cascade redacted them |
| `payload_redacted` | DSR-cascade redaction; legitimate; not tamper |

### Phase 2: Diagnose

**If reason = `signature_invalid` or `proof_invalid` and verification rate from internal verifiers (not external probing) is elevated**:
- Treat as tamper-suspect.
- Cross-check: pick affected `event_id`s; query Postgres SealRecord; compute proof manually; compare to claimed proof.
- If reference computation succeeds where verification reports failure: KeyResolver or root-publication state inconsistent.
- If reference computation fails: actual chain tampering — escalate to FM-02 procedure.

**If reason = `signature_invalid` and pattern indicates anonymous probing (high external rate)**:
- Likely scan; WAF rate-limit kicks in; not tamper.
- Log + investigate origin.

**If reason = `event_not_found` after retention-cascade**:
- Legitimate; verifier should query the redaction record instead.
- Update verification SDK documentation.

### Phase 3: Recover

Same as Phase 3 of FM-02 if tamper-confirmed.
Routine config or KeyResolver issues: fix publication; re-publish; verify three-channel match.

### Phase 4: Tenant notification

If tamper-confirmed: Sev-1 chain template per `incident-response.md`.
If config / KeyResolver issue: Sev-2 template (operational; no integrity breach).

## Verification (post-recovery)

- `oya_audit_chain_verification_failed_total` rate returns to baseline.
- `oya_audit_chain_hsm_signing_mismatch_total == 0` for ≥ 1h.
- Three-channel root match rate at 1.0.
- Sample anonymous + tenant + auditor verify calls succeed.
- Postmortem within 5 business days.

## References

- `microservices/audit-chain/failure-modes.md` FM-02 + FM-10.
- `microservices/audit-chain/policy/seal-integrity.md`.
- `microservices/audit-chain/runbooks/hsm-key-rotation.md`.
- `microservices/audit-chain/runbooks/merkle-seal-recovery.md`.
- `microservices/audit-chain/incident-response.md`.
- RFC 8032 (Ed25519 spec).
