---
doc_class: Runbook
title: Redaction overlay corruption (un-redact attack or accidental compensation)
microservice: recordings
severity: "Sev-1 — confidentiality breach if redacted content becomes visible"
status: Accepted
owner_team: ops-security + ops-compliance + axis-recordings + council-privacy
date: 2026-05-17
related_artifacts:
  - microservices/recordings/decisions/ADR-RECORDINGS-0003-redaction-and-pii-policy.md
  - microservices/recordings/policy/cedar/legal-hold.cedar
doc_status: published
---

# Runbook: Redaction overlay corruption

## Purpose

Recover from a corrupted redaction overlay (whether attacker-induced or
accidental). Overlay rows are insert-only with audit-chain seal per
ADR-RECORDINGS-0003; a "corruption" means either a row was rewritten
in-place (must not be possible by design) or a compensating-overlay row
was added without proper authority.

## Symptoms

- Previously-redacted content visible at playback or in transcript.
- Audit-chain mismatch on overlay-row content_hash.
- Customer reports unauthorized content visibility.

## Diagnosis

1. Pull the overlay history for the affected `recording_id`:
   ```bash
   oya recordings redaction history --recording-id <id>
   ```
2. Verify every overlay row's Merkle commitment against the audit-chain.
3. Identify the offending row: insertion-time, principal, paired-approver
   (if any), reason.

## Procedure

| Step | Action | Owner | Time |
|---|---|---|---|
| 1 | Page ops-security + ops-compliance + axis-recordings | on-call | immediate |
| 2 | Engage emergency legal-hold on the recording per `runbooks/legal-hold-court-order-receipt.md` | ops-compliance | ≤ 5 min |
| 3 | Take playback session offline for the recording (revoke share-links + invalidate CDN cache + refuse `start_playback_session`) | axis-recordings | ≤ 5 min |
| 4 | Compute the **expected** overlay state from audit-chain by replaying the seal log | server | ≤ 5 min |
| 5 | Add a compensating overlay row that re-applies the redaction span; signed by ops-security SPIFFE + paired-approver | server | ≤ 5 min |
| 6 | Audit-chain seal of the corrective overlay; emit `RedactionRestored` event | server | ≤ 1 min |
| 7 | Search index re-emit for the recording | server | ≤ 5 min |
| 8 | CDN purge for the recording's playback URLs | server | ≤ 5 min |
| 9 | Customer notification + breach notification per pack | council-privacy | ≤ 24h |
| 10 | Forensic-image of the row that was corrupted; preserve under chain-of-custody per ISO 27037:2012 | ops-security | ≤ 1h |

## Verification

```bash
oya recordings redaction verify --recording-id <id>
# Expected output:
# All overlay rows verified against audit-chain.
# Search index reflects current redaction state.
# CDN cache purged.
```

## Postmortem Triggers

- Any confirmed un-redact-attack.
- Any accidental compensation-overlay without paired approver.
- Any audit-chain mismatch on overlay rows.

## Preventive Controls (per ADR-RECORDINGS-0003)

- Overlay rows are insert-only with audit-chain seal; CI lane
  `recordings-redaction-overlay-immutability` asserts no UPDATE statements
  exist in the redaction crate's SQL surface.
- Un-redact (compensating overlay) requires Cedar PERMIT + paired approver
  per `policy/cedar/legal-hold.cedar`.
- Per-recording audit-chain seal recomputed daily; mismatch triggers Sev-1.

## References

- ADR-RECORDINGS-0003.
- `runbooks/legal-hold-court-order-receipt.md`.
- ISO 27037:2012 §5.4.
- GDPR Art. 25 (data-protection by design).
- HIPAA Safe Harbor §164.514.
- NIST SP 800-86.
