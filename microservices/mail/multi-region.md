---
doc_class: MultiRegionPlan
title: Multi-Region + DR Plan
microservice: mail
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-mail + council-privacy
deciders: ops-sre-reliability, axis-mail, council-architecture, council-privacy
related_adrs: [ADR-0117, ADR-0135, ADR-0131]
related_artifacts:
  - microservices/mail/capacity-model.md
  - microservices/mail/cost-budget.md
  - microservices/mail/policy/data-residency.md
  - microservices/mail/failure-modes.md
  - microservices/mail/incident-response.md
review_cadence: annually + on every pack DR-pair activation
doc_status: published
---

# Multi-Region + DR Plan (mail µservice)

## Purpose

Define the multi-region topology of mail µservice, the DR-pair strategy per pack, RPO/RTO targets, and the failover runbook anchor. Cross-pack-replication forbidden by `policy/data-residency.md`; this document covers intra-pack DR + intra-region HA.

## Multi-Region Topology

Per-pack region structure:

| Pack | Primary region | DR pair region | Topology |
|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | (single-region; no DR pair at M03 launch) | active-standby (planned upgrade subsequent-to-M03-completion) |
| pack-eu | OCI eu-frankfurt-1 | OCI eu-amsterdam-1 | active-warm-standby |
| pack-us | OCI us-ashburn-1 | OCI us-phoenix-1 | active-warm-standby |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | OCI us-phoenix-1 (HIPAA-eligible) | active-warm-standby |
| pack-jp | OCI ap-tokyo-1 | (single-region at launch) | active-standby (planned upgrade) |
| pack-sg | OCI ap-singapore-1 | (single-region) | active-standby |
| pack-au | OCI ap-sydney-1 | OCI ap-melbourne-1 | active-warm-standby |
| pack-in | OCI ap-hyderabad-1 | OCI ap-mumbai-1 | active-warm-standby |
| pack-br | OCI sa-saopaulo-1 | OCI sa-vinhedo-1 | active-warm-standby |
| pack-ae | OCI me-abudhabi-1 | OCI me-dubai-1 | active-warm-standby |
| pack-ksa | OCI me-jeddah-1 | OCI me-riyadh-1 | active-warm-standby |

## Replication Strategy

### Within-pack replication (allowed; load-bearing for DR)

- **Postgres**: primary in primary region; sync replica in primary region (HA); async replica in DR-pair region (DR).
- **S3 MIME blobs**: bucket replication within-pack (primary + DR-pair region copies with within-pack replication policy).
- **Tantivy search index**: per-tenant index lives in primary region; on DR failover, rebuild from primary's mailbox-store snapshot.
- **KMS DEK**: per-pack KMS replicated within-pack (primary + DR-pair); single KMS region keyring.
- **Per-tenant DKIM key**: OpenBao multi-region cluster within-pack.
- **Audit-chain seals**: replicated within-pack.

### Cross-pack replication (FORBIDDEN by default)

Per `policy/data-residency.md`: no cross-pack replication without tenant SCC. Cross-border DR (e.g., pack-eu → pack-us) is NOT a failover option.

## RPO / RTO Targets

| Metric | Target | Implementation |
|---|---|---|
| RPO (Postgres) | ≤ 5 min | Sync WAL replication within-pack; async WAL ship to DR region |
| RPO (S3 MIME blobs) | ≤ 15 min | S3 cross-AZ replication within-pack |
| RPO (KMS DEK) | ≤ 5 min | KMS replication |
| RPO (Tantivy search index) | ≤ 1h (rebuild on DR) | Rebuild from mailbox-store; out-of-band |
| RTO (Postfix SMTP frontend HA failover) | ≤ 5 min | Cross-AZ HPA |
| RTO (Postgres HA failover within-pack) | ≤ 5 min | Patroni / pg_auto_failover |
| RTO (Postgres DR failover cross-AZ within DR pair) | ≤ 15 min | Promote DR-region replica; update DNS |
| RTO (full pack DR failover) | ≤ 30 min | Failover runbook §"Pack DR" |
| RTO (Tantivy rebuild on DR) | ≤ 1h | Out-of-band rebuild from mailbox-store |

## DR Failover Procedure

### Trigger

Pack primary region degraded:
- Multi-AZ outage in primary region (Sev-1).
- Postgres failure with replica also affected.
- Sustained ingest failure > 30 min.

### Failover Steps (target: ≤ 30 min)

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-1; engage ops-sre-reliability + axis-mail on-call | immediate |
| 2 | Verify DR-pair region health (cross-region monitoring) | ≤ 2 min |
| 3 | Promote Postgres DR-region replica → primary; verify replication lag | ≤ 5 min |
| 4 | Re-point Helm/Istio config to DR-pair endpoints | ≤ 5 min |
| 5 | Switch DNS for SMTP MX records to DR-region SMTP edge LB | ≤ 5 min (TTL-respecting) |
| 6 | Switch DNS for IMAP/JMAP/REST endpoints | ≤ 5 min |
| 7 | Restart `oya-mail-*` worker pods in DR region with new endpoint config | ≤ 5 min |
| 8 | Trigger Tantivy rebuild from DR-region mailbox-store (out-of-band; runs in background) | starts at minute 25; completes in ~1h |
| 9 | Update tenant status page; notify affected tenants | ≤ 30 min |
| 10 | Update audit-chain with DR failover event | ≤ 30 min |

### Recovery to Primary

After primary region restored:
1. Validate primary region health (sustained 1h).
2. Replicate DR-region Postgres back to primary (this is now the secondary).
3. Re-point DNS + Helm config back to primary.
4. Tantivy rebuild on primary.
5. Audit-chain emission.

## Per-Pack Specifics

### pack-kr (single-region at M03 launch)

No DR pair at launch. Single-region risk acknowledged in tenant DPA. M04 adds DR pair (ap-chuncheon-1 or analogous).

Mitigation for M03:
- Multi-AZ within ap-seoul-1 (3 AZs).
- Postgres HA across AZs.
- S3 cross-AZ replication.
- Acknowledge RTO ≤ 4h in tenant DPA for region-wide outage (vs ≤ 30 min for AZ).

### pack-us-healthcare (HIPAA DR pair, both HIPAA-eligible)

us-ashburn-1 ↔ us-phoenix-1 — both HIPAA-eligible per Oracle attestation. Failover allowed without separate BAA addendum.

Special: 6y retention extends to DR region. S3 cross-region replication with at-rest encryption preserved.

### pack-eu (Schrems II + EDPB)

eu-frankfurt-1 ↔ eu-amsterdam-1 — both EU-resident. Failover does NOT cross EU boundary; no SCC required.

Cross-border BCDR (e.g., to pack-us for cost) NOT permitted per `policy/data-residency.md`.

### pack-jp / pack-sg (single-region)

Same as pack-kr; single-region at launch; DR pair planned for M04-onward.

### pack-au / pack-in / pack-br / pack-ae / pack-ksa (DR pair active)

DR-pair active per topology above. Tested in quarterly drill.

## Chaos Drills

Quarterly DR drill per pack:
1. Inject primary-region AZ failure (chaos engineering).
2. Verify auto-failover within RPO.
3. Validate tenant data continuity.
4. Document findings; iterate runbook.

Annual full pack DR drill: full failover to DR region; validate ≤ 30 min RTO.

## Verification

- `cargo run -p oya-dev-cli -- gate validate multi-region-conformance --microservice mail` — exit 0; replication configs match this spec.
- Quarterly chaos drill: `runbooks/dr-failover.md`.
- Annual full DR exercise: documented in `evidence/dr-drills/<pack>-<year>.json`.

## References

- ADR-0117: Cloud-native infrastructure (residency).
- ADR-0135: Connect dissolution.
- ADR-0131: Per-microservice flat layout.
- `microservices/mail/capacity-model.md`.
- `microservices/mail/cost-budget.md`.
- `microservices/mail/policy/data-residency.md`.
- `microservices/mail/failure-modes.md`.
- `microservices/mail/incident-response.md`.
- `microservices/mail/runbooks/dr-failover.md` (cross-ref).
- Postgres HA + DR — `patroni.readthedocs.io/`.
- OCI region documentation.
