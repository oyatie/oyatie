---
doc_class: Runbook
title: eDiscovery export bundle (channel + thread + attachment + audit-chain)
microservice: messenger
severity: "Sev-3 (planned) — every export is an audit-bearing operation"
status: Accepted
owner_team: ops-compliance + axis-messenger + ops-security
date: 2026-05-17
related_artifacts:
  - microservices/messenger/PRD.md (FR-11 eDiscovery)
  - microservices/messenger/compliance.md (SEC 17a-4 + FINRA 4511; HIPAA)
  - microservices/messenger/policy/auditor-scope.cedar
  - microservices/messenger/policy/channel-scope.cedar
doc_status: published
---

# Runbook: eDiscovery export

## Purpose

Produce a tamper-evident export bundle covering a tenant's discoverable
content (channel messages + thread replies + attachments + member-change
audit + retention-policy history + audit-chain seal) for delivery to a
regulator, court, or internal counsel under engagement letter.

## Preconditions

- Tenant has issued eDiscovery hold via POST /holds (per
  `contracts/openapi/messenger.yaml`); hold has identifier `hold_id`.
- Requester is a tenant compliance-officer principal per Cedar
  `Action::"export_for_ediscovery"` PERMIT.
- Two-person rule satisfied: requester + ops-compliance approver.
- For pack-us-healthcare: requesting counsel has signed BAA on file.
- For pack-us-financial: SEC 17a-4 retention floor honoured.

## Procedure

| Step | Action | Owner | Time |
|---|---|---|---|
| 1 | Tenant compliance-officer files export ticket via tenant ops portal | tenant | — |
| 2 | ops-compliance validates the engagement letter / court order / regulatory request | ops-compliance | ≤ 2h |
| 3 | Compliance-officer invokes `oya messenger ediscovery-export --hold-id <id> --requester <p> --paired-approver <p>` | ops-compliance | ≤ 5 min |
| 4 | Cedar evaluator validates four-eyes pair + hold scope | server | ≤ 1 s |
| 5 | Export worker streams matching rows → tar.gz with: messages.jsonl, threads.jsonl, attachments/, audit-chain-seal.json, retention-history.jsonl, manifest.json | worker | ≤ 1h per 1M messages |
| 6 | Manifest signed by export-worker SPIFFE identity (Ed25519) | worker | ≤ 1 s |
| 7 | Bundle uploaded to short-lived S3 prefix `oya-ediscovery-export-<pack>/<hold_id>/`; signed URL TTL 24h | worker | ≤ 5 min |
| 8 | Audit-chain seal: `EDiscoveryExportExecuted` event emitted | server | ≤ 1 s |
| 9 | Requester notified with signed URL + checksum | server | ≤ 5 min |
| 10 | Counsel/regulator downloads; verifies signature + checksum | external | — |

## Bundle Layout

```
hold-<id>.tar.gz
├── manifest.json              # signed manifest: includes Merkle root over all files
├── messages.jsonl             # one row per message; includes context_kind, hold_id, content_hash
├── threads.jsonl              # one row per thread + parent links
├── attachments/
│   ├── <attachment_id_1>      # original blob; preserved encrypted-at-rest
│   ├── <attachment_id_1>.meta.json  # scan-status, digest, mime, original-channel
│   └── ...
├── audit-chain-seal.json      # Ed25519 seals for every row + the manifest itself
└── retention-history.jsonl    # the retention-policy timeline applicable to the hold scope
```

## Pack-Specific Bundle Variants

| Pack | Variation |
|---|---|
| pack-us-financial | SEC 17a-4(f) — bundle includes WORM-attestation proof + 36mo retention manifest |
| pack-us-healthcare | HIPAA — body decrypted ONLY if counsel BAA-covered; otherwise body redacted per `policy/redaction-phi.md` |
| pack-eu | GDPR Art. 30 record-of-processing append; export emits ROP entry |
| pack-kr | KR 전자문서법 (Electronic Document Act) compliance markers |

## Verification

- Bundle Merkle root verified against audit-chain seal independently.
- Per-message content_hash matches Postgres row's content_hash (no
  silent rewrite during export).
- Per-attachment digest_sha256 matches S3 object hash.

## Failure Modes

| Failure | Recovery |
|---|---|
| Export worker crash mid-stream | restart from last sealed batch; bundle resumable |
| S3 upload fails | retry with backoff; alert if exceeds 1h |
| Cedar deny on four-eyes | refuse export; audit-chain logs attempted-export with deny reason |
| Hold has been closed before export completes | refuse + alert; legal review required |

## Postmortem Triggers

- Any export touching Personal-context resources (should be impossible — DCI invariant).
- Any export exceeding 24h beyond approval window.
- Any bundle that fails signature verification on counsel-side download.

## References

- SEC Rule 17a-4(f).
- FINRA Rule 4511.
- HIPAA 45 CFR §164.524 (right of access) + §164.526 (amendment).
- GDPR Art. 30 (records of processing).
- KR 전자문서법.
- `microservices/messenger/PRD.md` FR-11.
- `microservices/messenger/compliance.md`.
- `microservices/messenger/policy/auditor-scope.cedar`.
