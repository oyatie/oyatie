---
doc_class: Runbook
title: eDiscovery export bundle (recording + transcript + redaction overlay + audit-chain seal)
microservice: recordings
severity: "Sev-3 (planned) — every export is an audit-bearing operation"
status: Accepted
owner_team: ops-compliance + axis-recordings + ops-security
date: 2026-05-17
related_artifacts:
  - microservices/recordings/PRD.md (FR-10 eDiscovery)
  - microservices/recordings/compliance.md (FRCP + SEC 17a-4 + FINRA + HIPAA + KR 전자문서법)
  - microservices/recordings/policy/cedar/legal-hold.cedar
  - microservices/recordings/decisions/ADR-RECORDINGS-0002-retention-and-legal-hold-policy.md
doc_status: published
---

# Runbook: eDiscovery export

## Purpose

Produce a tamper-evident export bundle covering a tenant's discoverable
recording content for delivery to a regulator, court, or internal counsel
under engagement letter. Conforms to FRCP Rule 26(f)/34, Sedona Conference,
ISO 27037:2012, SEC 17a-4(f) WORM-attestation, FINRA Rule 4511, MiFID II
Art. 16(7), HIPAA, KR 전자문서법.

## Preconditions

- Tenant has issued legal hold per `runbooks/legal-hold-court-order-receipt.md`;
  hold has identifier `hold_id`.
- Requester is a tenant compliance-officer per Cedar
  `Action::"export_for_ediscovery"` PERMIT.
- Paired four-eyes approver satisfied.
- For pack-us-healthcare: requesting counsel has signed BAA on file.
- For pack-us-financial: SEC 17a-4 retention floor honoured.
- For pack-eu: GDPR Art. 30 ROP entry will be appended.

## Procedure

| Step | Action | Owner | Time |
|---|---|---|---|
| 1 | Compliance-officer files export ticket via tenant ops portal | tenant | — |
| 2 | ops-compliance validates the engagement letter / court order / regulatory request | ops-compliance | ≤ 2h |
| 3 | Compliance-officer invokes `oya recordings ediscovery export --hold-id <id> --requester <p> --paired-approver <p>` | ops-compliance | ≤ 5 min |
| 4 | Cedar evaluator validates four-eyes pair + hold scope per `policy/cedar/legal-hold.cedar` | server | ≤ 1 s |
| 5 | Export worker streams matching rows → tar.gz bundle (see Bundle Layout below); transcode via ffmpeg 7.x in gVisor if media format conversion needed | worker | ≤ 1h per 100 hours of media |
| 6 | Manifest signed by export-worker SPIFFE identity (Ed25519) | worker | ≤ 1 s |
| 7 | Merkle root computed over the full bundle | worker | ≤ 1 min |
| 8 | Bundle uploaded to short-lived S3 prefix `oya-recordings-ediscovery-<pack>/<hold_id>/`; signed URL TTL 24h | worker | ≤ 5 min |
| 9 | Audit-chain seal: `EDiscoveryExportExecuted` event emitted | server | ≤ 1 s |
| 10 | Requester notified with signed URL + checksum + Merkle root | server | ≤ 5 min |
| 11 | Counsel/regulator downloads; verifies signature + checksum + Merkle root per ISO 27037:2012 | external | — |

## Bundle Layout

```
hold-<id>.tar.gz
├── manifest.json                 # signed manifest: includes Merkle root
├── recordings/
│   ├── <recording_id_1>/
│   │   ├── source.<ext>          # original blob; preserved encrypted-at-rest unless tenant + counsel BAA permits decryption
│   │   ├── source.meta.json      # ingest metadata + scan-status + content_hash
│   │   ├── transcript.json       # speaker-diarised transcript JSON
│   │   ├── transcript.vtt        # WebVTT rendition
│   │   ├── transcript.pdf        # Pandoc-rendered PDF
│   │   ├── redaction-overlay.jsonl  # one row per overlay span; redactions applied at playback
│   │   ├── chapters.json
│   │   ├── summary.md
│   │   └── seal.json             # per-recording Ed25519 seal
│   └── ...
├── retention-history.jsonl       # the retention-policy timeline for the hold scope
├── legal-hold-history.jsonl      # the hold engagement + release history
├── audit-chain-seal.json         # Ed25519 seals for every row + the manifest itself
└── README.md                     # bundle map + verification instructions
```

## Pack-Specific Bundle Variants

| Pack | Variation |
|---|---|
| pack-us-financial | SEC 17a-4(f) — bundle includes WORM-attestation proof + 36mo retention manifest + MiFID II Art. 16(7) annexation if applicable |
| pack-us-healthcare | HIPAA — body decrypted ONLY if counsel BAA-covered; otherwise body remains encrypted; transcript redacted per `policy/redaction-phi.md` (if added) |
| pack-eu | GDPR Art. 30 record-of-processing append; ROP entry emitted |
| pack-kr | KR 전자문서법 Art. 5 integrity attestation included; 통신비밀보호법 consent record included |
| pack-au | TIA Act + Surveillance Devices Act order verification annexed |

## Verification

- Bundle Merkle root verified against audit-chain seal independently.
- Per-recording content_hash matches Postgres row's content_hash (no
  silent rewrite during export).
- Per-segment digest_sha256 matches S3 object hash.
- Counsel verifies Ed25519 signature against published recordings-rest
  public key.

## Failure Modes

| Failure | Recovery |
|---|---|
| Export worker crash mid-stream | restart from last sealed batch; bundle resumable |
| S3 upload fails | retry with backoff; alert if exceeds 1h |
| Cedar deny on four-eyes | refuse export; audit-chain logs attempted-export with deny reason |
| Hold has been released before export completes | refuse + alert; re-engagement required per `runbooks/legal-hold-court-order-receipt.md` |
| Merkle root mismatch (audit-chain disagrees) | block export; investigate audit-chain divergence; engage ops-security |

## Postmortem Triggers

- Any export touching out-of-scope recordings (hold scope leak).
- Any export exceeding 24h beyond approval window.
- Any bundle that fails signature verification on counsel-side download.

## References

- FRCP Rule 26(f), Rule 34.
- Sedona Conference Commentary.
- ISO 27037:2012.
- SEC Rule 17a-4(f).
- FINRA Rule 4511.
- MiFID II Art. 16(7).
- HIPAA 45 CFR §164.524, §164.526.
- KR 전자문서법, KR 통신비밀보호법.
- GDPR Art. 30.
- ADR-RECORDINGS-0002.
- `runbooks/legal-hold-court-order-receipt.md`.
- `policy/cedar/legal-hold.cedar`.
