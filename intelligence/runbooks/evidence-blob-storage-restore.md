---
doc_class: Runbook
title: Blob storage restore — payload-blob corruption or accidental redaction
microservice: foundry-evidence
severity: Sev-1 (always; payload-blob is load-bearing for any pack containing the blob hash)
status: Accepted
owner_team: ops-sre-reliability + axis-foundry-evidence + axis-audit-chain
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-09)
  - microservices/intelligence/policy/evidence-pack-integrity.md (EPI-03)
  - microservices/audit-chain/runbooks/audit-export.md (substrate)
doc_status: published
---

# Runbook: Blob storage restore

## Purpose

Recovery procedure when an evidence-pack payload blob (raw prompt/output text content-addressed in the audit-chain WORM substrate) is either:
- corrupted at rest (FM-09a: detected when read-side hash verify fails),
- accidentally redacted out of cycle (FM-09b: DSR cascade redacted a blob that should not have been redacted),
- or unreadable (FM-09c: substrate access path broken).

Note: WORM Object Lock (Compliance mode) on the substrate makes raw blob mutation extremely unlikely. If detected, this is treated as a fundamental control failure and escalates to ExecSponsor.

## Trigger

- `oya_foundry_evidence_payload_blob_hash_verify_failure_total` > 0 (single occurrence pages Sev-1).
- DSR cascade replay finds an unexpected redaction.
- Substrate WORM read returns NotFound for a blob that the pack index says exists.

## Severity

**Sev-1 always.**

## Procedure

### Phase 1: Halt + Engage (≤ 5 min)

1. Declare Sev-1; open `#inc-<id>`.
2. Engage IC: ops-sre-reliability primary + axis-foundry-evidence + axis-audit-chain SME + Cryptography SME + council-privacy + ExecSponsor for awareness.
3. Halt any in-flight regulator-export workflow that may reference the affected blob:
   ```
   oya foundry-evidence regulator-export pause-all --pack <pack>
   ```

### Phase 2: Forensic capture (≤ 30 min)

1. Capture canonical metadata from Postgres:
   ```
   psql ... -c "SELECT pack_id, invocation_id, payload_sha, audit_event_id, blob_storage_uri, redaction_state FROM evidence_pack WHERE pack_id=<id>;"
   ```
2. Capture audit-chain inclusion proof for the pack:
   ```
   oya audit-chain proof get --event-id <audit_event_id>
   ```
3. Capture S3 object metadata (Object Lock retention + legal hold + last-modified):
   ```
   aws s3api head-object --bucket <bucket> --key <key>
   ```
4. Capture substrate audit-emitted history for the blob:
   ```
   oya audit-chain query --payload-sha <sha> --include-redactions
   ```

### Phase 3: Diagnose (≤ 1 h)

| Suspect | Indicators | Action |
|---|---|---|
| FM-09a corruption at rest | Object Lock intact; ETag changed unexpectedly | Treat as fundamental control failure → escalate to ExecSponsor; engage cloud-secrets + Oracle Object Storage |
| FM-09b accidental redaction | substrate `RetentionApplied` event found with `mode=redact_payload` outside expected DSR cascade | Review DSR cascade entry-point in `tenancy`; if no DSR exists for this subject, redaction is unauthorised → forensic |
| FM-09c access path broken | S3 returns 403 or 5xx; lock + object both present | Check substrate IAM; rotate SPIFFE if revoked; verify cross-pack ACL not changed |

### Phase 4: Recovery (varies)

#### FM-09a recovery (corruption with WORM intact — should be unreachable)

1. WORM Object Lock prevents legitimate mutation; corruption with lock intact means either:
   - Storage backend integrity failure (rare; engage Oracle).
   - Lock was bypassed (engage Oracle + ExecSponsor immediately; this is a Sev-0-class control failure).
2. The audit-chain WORM has no second copy of raw payload by design (WORM is the canonical store); the recovered hash MUST match the pack's `payload_sha` for the pack to retain integrity.
3. If hash mismatch is permanent, the pack is **integrity-compromised**:
   - Tag the pack `integrity_compromised=true` (via retention-cascade RPC; 2-person rule).
   - Audit-emit `foundry.evidence.pack.integrity_compromised.v1` with `payload_sha_claimed`, `payload_sha_observed`, `forensic_incident_id`.
   - All downstream regulator exports that include this pack: trigger reissue runbook.
   - The audit-chain Merkle proof of the pack itself (the hash-of-pack at sealing time) remains valid; the integrity event documents that the underlying payload referenced by that hash is no longer producible.

#### FM-09b recovery (out-of-cycle redaction)

1. If redaction was authored by an unauthorised principal → forensic; treat as breach.
2. If redaction was authored by an authorised principal but on wrong key → procedural error; capture root cause; no blob restore possible (WORM-redacted payload is unrecoverable by design); pack is now in `redacted` state per substrate; document.
3. If the redaction was correct but DSR cascade lost track of it → backfill the DSR record in `tenancy`.

#### FM-09c recovery (access path)

1. Restore IAM / SPIFFE / network access.
2. Re-verify read path: `oya foundry-evidence pack read --pack-id <id> --include-payload`.
3. Confirm payload hash matches `payload_sha`.

### Phase 5: Tenant + regulator comms

1. If the pack was returned to tenant in the last 90 days or is part of an active regulator engagement:
   - council-privacy + legal-counsel determine notification scope.
   - tenancy DPA-bound notification.
   - Regulator-engagement notification if applicable; reissue any related bundle.
2. Postmortem within 5 business days for Sev-1.

## Halt conditions

- WORM Object Lock is no longer in Compliance mode → escalate to ExecSponsor; halt all pack assembly until lock restored.
- Audit-chain inclusion proof for the pack is invalid → join audit-chain Sev-1 (Merkle integrity event).
- Multiple blobs simultaneously corrupted → suspected systemic compromise; quarantine pack; engage breach-response.

## Verification (post-recovery)

- All affected pack hashes verified end-to-end (Postgres index ↔ payload blob ↔ audit-chain inclusion proof).
- Any `integrity_compromised=true` packs documented in `evidence/incidents/foundry-evidence/<inc-id>/`.
- Postmortem published.
- If applicable, CI regression test added (e.g., blob-hash drill).

## References

- `microservices/intelligence/policy/evidence-pack-integrity.md` EPI-03.
- `microservices/intelligence/failure-modes.md` FM-09.
- `microservices/audit-chain/policy/seal-integrity.md`.
- ADR-0028 (audit-chain Merkle/Ed25519).
