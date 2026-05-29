---
doc_class: IncidentResponse
template_id: TPL-INCIDENT-RESPONSE
microservice: recordings
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + ops-security + ops-compliance + axis-recordings
related_adrs: [ADR-0139, ADR-RECORDINGS-0002]
doc_status: published
---

# Incident Response: recordings µservice

## Severity Classification

| Sev | Trigger | Page | RTO | RPO |
|---|---|---|---|---|
| Sev-1 | Legal-hold breach OR retention-purge against held recording OR cross-pack data leak | axis-recordings + ops-security + ops-compliance + council-privacy | ≤ 15 min | ≤ 1 min |
| Sev-1 | Playback unavailability > 99 % tenant base for > 5 min | axis-recordings + ops-sre | ≤ 15 min | ≤ 5 min |
| Sev-2 | Transcript pipeline degraded (queue > 60 min) | axis-recordings | ≤ 30 min | — |
| Sev-2 | eDiscovery export Merkle seal verification failure | axis-recordings + ops-compliance | ≤ 30 min | — |
| Sev-3 | Watermark rotation lag | axis-recordings | ≤ 4h | — |
| Sev-3 | Search index degraded | axis-recordings | ≤ 4h | — |

## Roles

| Role | Responsibility |
|---|---|
| Incident Commander | drives the response; declares Sev; closes incident |
| ops-sre-reliability on-call | substrate (Postgres, S3, Valkey, CDN, Meilisearch) |
| axis-recordings on-call | recordings µservice code + ingest contract |
| ops-security on-call | confidentiality + integrity incidents (FM-07, FM-11, FM-16, FM-18, FM-20) |
| ops-compliance on-call | legal-hold + retention + ediscovery (FM-08, FM-09, FM-10, FM-19) |
| council-privacy | privacy breach notification (GDPR Art. 33, KR PIPA, HIPAA) |

## Communication

- Sev-1: status page (red); customer notification (email + in-app) within 60
  min; council-privacy notified for breach-class incidents.
- Sev-2: status page (yellow); customer notification within 4h if customer-
  impacting.
- Sev-3: internal-only; no status page.

## Privacy Breach Notification (GDPR Art. 33)

If incident → unauthorized disclosure of personal data:
- 72h notification to relevant DPA (per pack).
- Customer notification (Art. 34) if high-risk to data subjects.
- KR PIPA Art. 34 — 72h notification + 5d successor-IP.
- HIPAA HITECH — 60d notification + 60d for > 500 affected.
- DPDPA 2023 — 72h notification + customer notification.

## Forensic Procedure (ISO 27037:2012 + NIST SP 800-86)

For Sev-1 confidentiality / integrity incidents:

1. **Identification**: confirm scope (which tenants? which recordings? which
   transcripts?).
2. **Collection**: snapshot Postgres + S3 + audit-chain at incident time;
   store under chain-of-custody.
3. **Acquisition**: forensic-image of affected nodes; Ed25519-sign image
   hashes.
4. **Preservation**: legal-hold-engagement on all affected recordings;
   refuse normal retention purges until cleared.
5. **Analysis**: ops-security + ops-compliance review.
6. **Reporting**: post-mortem within 5 business days; report to council-
   privacy + DPA where required.

## Post-Mortem

- Blameless template; published within 5 business days for Sev-1; 10
  business days for Sev-2.
- Actions tracked as IPs in `microservices/recordings/`.
- DPIA + threat-model refreshed if root-cause exposes a new threat.

## References

- ADR-0139.
- `failure-modes.md`.
- `runbooks/*.md`.
- ISO 27037:2012; NIST SP 800-86; GDPR Art. 33/34; KR PIPA Arts. 34/34-2;
  HIPAA HITECH 13402; DPDPA 2023.
