---
doc_class: Policy
template_id: TPL-POLICY
microservice: community
status: Accepted
classification: INTERNAL_ONLY
policy_class: data-residency
date: 2026-05-17
owner_team: council-privacy + ops-security + axis-community
related_adrs: [ADR-0056, ADR-0105, ADR-0117, ADR-0135, ADR-0131]
related_artifacts:
  - microservices/community/policy/community-isolation.md
  - microservices/community/dpia.md
  - microservices/community/compliance.md
doc_status: published
---

# Data-residency policy: community µservice

## Purpose

Bind every community surface object to the tenant's `jurisdiction_code` per ADR-0117 and the per-pack regulatory regime. Define retention, cross-border transfer, and deletion semantics.

## Binding

Each tenant declares `jurisdiction_code` at onboarding (ISO 3166-1 alpha-2). Every Postgres / Elasticsearch / Valkey / S3 cluster is regional; the tenant's cluster is selected from a region map at creation and immutable for the tenant's life.

## Retention Matrix

| Data class | Default | pack-kr | pack-eu | pack-us-healthcare | pack-jp | pack-sg | pack-au |
|---|---|---|---|---|---|---|---|
| Announcement post | 7 y | 5 y (PIPA min) | 6 y | 6 y (HIPAA) | 5 y | 5 y | 7 y |
| Q&A post + answer | Indefinite (with revisions) | Indefinite | Indefinite | 6 y | Indefinite | Indefinite | Indefinite |
| KB article + revisions | Indefinite | Indefinite | Indefinite | 6 y | Indefinite | Indefinite | Indefinite |
| KB attachment | Indefinite | Indefinite | Indefinite | 6 y | Indefinite | Indefinite | Indefinite |
| Vote record | 5 y | 3 y | 5 y | 6 y | 5 y | 5 y | 5 y |
| Moderation action | 7 y | 5 y | 6 y | 6 y | 5 y | 5 y | 7 y |
| Flag (raised, not yet actioned) | 90 d | 90 d | 90 d | 6 y if PHI-related | 90 d | 90 d | 90 d |
| Audit log | 7 y | 5 y | 6 y | 6 y | 5 y | 5 y | 7 y |
| IP / user-agent (abuse detection) | 30 d → aggregate | 30 d | 30 d | 30 d | 30 d | 30 d | 30 d |
| Search index | matches source | matches source | matches source | matches source | matches source | matches source | matches source |
| Hot-feed Valkey cache | 7 d | 7 d | 7 d | 7 d | 7 d | 7 d | 7 d |

## Cross-Border Transfer

- **pack-kr**: cross-border transfer requires PIPA Art. 28 explicit consent or contractual basis. Default = no cross-border.
- **pack-eu**: SCC 2021/914 + supplementary measures per EDPB Schrems II guidance. Adequacy decision (UK, JP, KR) applies where available.
- **pack-us-healthcare**: BAA with downstream sub-processor; no transfer outside US unless tenant opts in.
- **pack-jp**: APPI Art. 24 disclosure + consent.
- **pack-sg**: PDPA §26 protections + consent.
- **pack-au**: APP 8 cross-border disclosure protections.
- **pack-in**: DPDPA 2023 §16 restrictions; no transfer to denied jurisdictions.
- **pack-br**: LGPD Art. 33 transfer mechanisms (SCC equivalent).
- **pack-ae / pack-ksa**: PDPL local-only by default; cross-border via authority approval.

## Deletion Semantics

DSR Right-to-Erasure cascade (per `incident-response.md` "DSR Cascade"):

1. Receive DSR request at tenant-admin dashboard.
2. Tenant-admin approves (two-eyes for member > 30 d account age).
3. `post-store` tombstones member's posts (body redacted; metadata sealed in audit-chain).
4. `kb-article-store` redacts member-authored KB articles + attachments; revisions retained as tombstone.
5. `voting-engine` anonymises member's votes (member_id → DSR_REDACTED).
6. `moderation-queue` retains action records (legal hold) but redacts target body in linked snapshot.
7. `search-index` reindex invalidates redacted documents; verify with sample query.
8. `audit-chain` seals a `DsrErasureCompleted` event with completion sha256.
9. Tenant-admin receives completion attestation.

## Pack Overlays

Pack-specific overlays in `iac/kustomize/overlays/pack-<jurisdiction>/kustomization.yaml` adjust:
- Retention TTL cron schedules.
- Per-region S3 bucket reference.
- Postgres backup retention (matches retention matrix).
- Per-pack regulator-notification webhook (KR PIPC, EU DPA, US OCR).

## Verification

- Daily CI gate runs `data-residency-check`:
  - Asserts every Postgres / Elasticsearch / Valkey / S3 cluster's region matches tenant `jurisdiction_code`.
  - Asserts retention TTL cron is configured per matrix.
  - Asserts cross-border replication is disabled unless tenant opt-in flag set.
- Quarterly drill: simulate DSR cascade; verify all 9 steps complete within 30 days.

## Breach Response

Cross-border leak is a P0; per-pack regulator notification window applies (e.g., KR PIPA 24 h, GDPR 72 h, HIPAA 60 d).
