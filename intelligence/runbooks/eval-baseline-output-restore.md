---
doc_class: Runbook
title: Baseline-Output Restore (S3 + KMS recovery)
microservice: foundry-eval
severity: "Sev-2 (object loss or signature breach) / Sev-1 (mass baseline loss)"
status: Accepted
owner_team: axis-foundry + ops-security + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-05 baseline-output integrity)
  - microservices/intelligence/threat-model.md (T-T-02)
  - microservices/intelligence/policy/two-person-admin-ops.md
doc_status: published
---

# Runbook: Baseline-Output Restore

## Trigger

ONE of:

1. **Per-object Cosign verify failure** on read; quarantined; ops paged.
2. **Block-validator monthly job** flags SHA mismatch on baseline-output object.
3. **S3 bucket outage** in pack region; baseline-output reads return 5xx.
4. **Mass baseline loss** (rare): e.g., bucket-level event affecting multiple objects.

## Severity

- Single object signature breach: **Sev-2** (potential tampering; T-T-02).
- Single object unreachable (transient): **Sev-3**.
- Mass baseline loss: **Sev-1**.

## Pre-checks

1. Confirm scope: how many objects affected? `aws s3api list-objects --bucket oya-intelligence-eval-baselines-<pack> --prefix baselines/<tenant>/<cap>/` to enumerate.
2. Confirm DR-pair availability for affected pack (if DR pack).
3. Confirm KMS keyring availability + per-subject DEK availability.

## Steps

| Step | Action | Time budget |
|---|---|---|
| 1 | Open `#inc-<id>`; assign IC; declare severity | ≤ 5 min |
| 2 | Pre-checks above | ≤ 5 min |
| 3 | For per-object signature breach: (a) quarantine the affected object (rename to `quarantine/`); (b) trigger 2-person rule per `policy/two-person-admin-ops.md` to investigate tampering; (c) restore from DR-pair if available + signature valid there | ≤ 30 min |
| 4 | For per-object unreachable (transient): retry; if persists ≥ 10 min, restore from DR-pair | ≤ 15 min |
| 5 | For S3 bucket outage: failover to DR-pair (pack-eu / pack-us / pack-au / pack-in / pack-br have DR); for single-region packs, await provider recovery | per provider |
| 6 | For mass baseline loss: declare Sev-1; engage council-architecture + ops-security; restore from off-cluster backup (S3 cross-region replicated copy if SCC-approved; otherwise from per-pack archive) | ≤ 4 h |
| 7 | After restore: re-verify Cosign signature on each restored object | ≤ 1 h |
| 8 | After verify: re-run affected publish-gate evaluations to confirm baselines functional | ≤ 30 min |
| 9 | If tampering confirmed: engage ops-security incident response; begin breach-notification chain (GDPR Art. 33 if PII-class; KR PIPA equivalent) | per timeline |
| 10 | Postmortem within 5 business days for Sev-2; within 10 days for Sev-1 | — |

## Per-Subject DEK Recovery

If the per-subject DEK is unavailable (KMS unreachable, DEK deleted in error):
- Replay against affected subject is structurally unrecoverable (this is by design per ADR-0024).
- Verify the DEK deletion was authorised (DSR cascade emission audited).
- If unauthorised: declare Sev-1 + ops-security investigation; rotate KMS access credentials.

## Verification

After completion:
- All affected objects readable + Cosign verified + Rekor inclusion proof valid.
- Affected publish-gate evaluations re-run successfully.
- `BaselineOutputRestored{object_id, restored_from, verified_at}` event in audit-chain.

## References

- ADR-0024 §"Resolved 1" (per-subject-keyed cryptographic shredding).
- `microservices/intelligence/failure-modes.md` FM-05.
- `microservices/intelligence/threat-model.md` T-T-02.
- `microservices/intelligence/policy/two-person-admin-ops.md`.
