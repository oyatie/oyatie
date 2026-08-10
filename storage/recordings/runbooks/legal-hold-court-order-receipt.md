---
doc_class: Runbook
title: Legal-hold engagement on receipt of a court order (load-bearing 100 %)
microservice: recordings
severity: "Sev-1 (load-bearing 100 % invariant) — any latency breach or correctness breach is Sev-1"
status: Accepted
owner_team: ops-compliance + axis-recordings + ops-security + council-privacy
date: 2026-05-17
related_artifacts:
  - microservices/recordings/PRD.md (FR-08, AC-08)
  - microservices/recordings/compliance.md (SEC 17a-4 + FINRA + HIPAA + KR 전자문서법)
  - storage/recordings/policy/cedar/legal-hold.cedar
  - microservices/recordings/decisions/ADR-RECORDINGS-0002-retention-and-legal-hold-policy.md
  - storage/observability/slos/legal-hold-engagement-latency.openslo.yaml
  - storage/observability/slos/legal-hold-chain-of-custody-correctness.openslo.yaml
doc_status: published
---

# Runbook: Legal-hold engagement on receipt of a court order

## Purpose

Engage a legal hold within the **load-bearing 100 % correctness invariant**:
once the court order is received and validated, every in-scope recording is
held within p99 ≤ 1s. After engagement, no retention purge, no KMS-shred, no
delete may execute against the held recording until release. Conforms to
FRCP Rule 26(f)/34 + Sedona Conference + ISO 27037:2012.

## Preconditions

- Engagement-requester is a tenant compliance-officer per Cedar
  `Action::"engage_legal_hold"` PERMIT.
- Paired four-eyes approver present (a second compliance-officer).
- Court-order reference (court ID + jurisdiction + case-no + order date)
  provided.
- For pack-us-healthcare: requesting counsel has signed BAA on file.
- For pack-us-financial: order conforms to SEC 17a-4 + FINRA 4511
  requirements.
- For pack-kr: order conforms to 전자문서법 Art. 5 evidentiary requirements.

## Procedure

| Step | Action | Owner | Time |
|---|---|---|---|
| 1 | Court order arrives at tenant; counsel + ops-compliance verify authenticity (jurisdiction, signature, scope) | ops-compliance | ≤ 4h |
| 2 | Compliance-officer + paired-approver invoke `oya recordings legal-hold engage --scope <hold-scope-expr> --court-order-ref <ref> --paired-approver <p>` via the recordings REST API | ops-compliance | ≤ 5 min |
| 3 | Cedar evaluator validates four-eyes pair + scope per `policy/cedar/legal-hold.cedar` | server | ≤ 100 ms |
| 4 | Legal-hold kernel takes a Postgres advisory-lock on the scope; emits `LegalHoldEngaged` event with engaging-principal + paired-approver + scope + court-order-ref | server | ≤ 200 ms |
| 5 | Audit-chain seal computed; event sealed Ed25519 + Merkle root commits | server | ≤ 300 ms |
| 6 | Retention worker reads the lock on its next poll cycle (cycle = 100ms); any in-flight purge against the scope aborts; `RetentionWorkerAborted` event emitted | server | ≤ 100 ms |
| 7 | KMS-shred worker similarly aborts in-flight shred against scope; `KmsShredAborted` event emitted | server | ≤ 100 ms |
| 8 | Verification: query recording metadata for scope; assert `legal_hold_engaged == true` for every in-scope row | server | ≤ 100 ms |
| 9 | Audit-chain final seal; engagement complete | server | total p99 ≤ 1s |
| 10 | Counsel notified with engagement audit-receipt; receipt signed by recordings-rest SPIFFE Ed25519 | server | ≤ 5 min |

## Verification

```bash
oya recordings legal-hold show --hold-id <id>
# Expected output:
# {
#   "hold_id": "...",
#   "tenant_id": "...",
#   "scope_expression": "...",
#   "engaged_at": "<ts>",
#   "court_order_ref": "...",
#   "engaged_by_principal": "...",
#   "paired_approver": "...",
#   "audit_chain_seal_ref": "...",
#   "matched_recordings_count": N,
#   "engagement_latency_ms": <100..1000>
# }

oya recordings legal-hold verify-coverage --hold-id <id>
# Expected output:
# All N matched recordings show legal_hold_engaged=true
# No retention purge has executed against scope since engaged_at
# No KMS-shred has executed against scope since engaged_at
```

## Bundle Layout (chain-of-custody seal artifact)

```
hold-<hold_id>-engagement-seal.json
{
  "hold_id": "...",
  "tenant_id": "...",
  "scope_expression": "...",
  "engaged_at": "<RFC 3339>",
  "court_order_ref": {
    "court_id": "...",
    "jurisdiction": "...",
    "case_no": "...",
    "order_date": "<RFC 3339>"
  },
  "engaged_by_principal": "...",
  "paired_approver": "...",
  "matched_recordings": [
    { "recording_id": "...", "content_hash": "sha256:...", "pre_hold_retention_state": "..." },
    ...
  ],
  "audit_chain_merkle_root": "sha256:...",
  "spiffe_signature": "ed25519:..."
}
```

## Pack-Specific Variants

| Pack | Variation |
|---|---|
| pack-us-financial | SEC 17a-4(f) — engagement automatically extends retention to non-erasable WORM if not already |
| pack-us-healthcare | HIPAA — body decrypted ONLY for BAA-covered counsel during subsequent eDiscovery |
| pack-eu | GDPR Art. 30 — ROP entry appended for the hold |
| pack-kr | 전자문서법 Art. 5 — integrity attestation included in seal |
| pack-au | TIA Act — order verification additionally checked against state Surveillance Devices Act |

## Failure Modes

| Failure | Recovery |
|---|---|
| Cedar deny on four-eyes | refuse + log `LegalHoldEngagementRefused`; engagement-requester must obtain paired-approver |
| Postgres advisory-lock contention (rare; another hold in progress on same scope) | retry with exponential backoff; alert if > 5 retries |
| Audit-chain seal fails | abort engagement + alert ops-security; engagement is **not** considered complete until seal succeeds |
| p99 engagement latency > 1s | **Sev-1**; page on-call axis-recordings + council-privacy; investigate Postgres lock contention + audit-chain latency |
| Retention worker fails to abort in-flight purge against scope | **Sev-1 (load-bearing breach)**; emergency rollback per `runbooks/retention-policy-rollback.md`; root-cause investigation |

## Release Procedure

When the court order is dissolved or the litigation closes:

| Step | Action | Owner |
|---|---|---|
| 1 | Counsel issues release authorization | ops-compliance + counsel |
| 2 | Compliance-officer + paired-approver invoke `oya recordings legal-hold release --hold-id <id> --paired-approver <p> --reason <text>` | ops-compliance |
| 3 | Cedar validates pair + reason; `LegalHoldReleased` event emitted | server |
| 4 | Retention worker re-evaluates the scope; any rows whose pack retention floor has been reached become purge-eligible | server |
| 5 | Audit-chain final seal | server |

## Postmortem Triggers

- Any p99 engagement latency > 1s.
- Any retention purge executed against a held recording (load-bearing breach
  — must never happen).
- Any KMS-shred executed against a held recording (load-bearing breach).
- Any audit-chain seal failure.

## References

- FRCP Rule 26(f), Rule 34.
- Sedona Conference Commentary on Legal Holds.
- ISO 27037:2012 §5.4 (preservation).
- SEC Rule 17a-4(f).
- FINRA Rule 4511.
- MiFID II Art. 16(7).
- HIPAA 45 CFR §164.524, §164.526.
- KR 전자문서법 Art. 5.
- GDPR Art. 30 (records of processing).
- ADR-RECORDINGS-0002.
- `policy/cedar/legal-hold.cedar`.
- `slos/legal-hold-engagement-latency.openslo.yaml`.
- `slos/legal-hold-chain-of-custody-correctness.openslo.yaml`.
