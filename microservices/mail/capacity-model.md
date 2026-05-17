---
doc_class: CapacityModel
title: Capacity Sizing Model
microservice: mail
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-mail + ops-deliverability
deciders: ops-sre-reliability, axis-mail, council-architecture
related_adrs: [ADR-0117, ADR-0126, ADR-0131]
related_artifacts:
  - microservices/mail/cost-budget.md
  - microservices/mail/multi-region.md
  - microservices/mail/policy/data-residency.md
review_cadence: quarterly + on every component-replica-set change
doc_status: published
---

# Capacity Sizing Model (mail µservice)

## Purpose

Sizing formulae + reference-architecture baselines for every Layer-A component (Postfix SMTP, Dovecot/Rust-IMAP, Rspamd, OpenDKIM, Postgres, S3, Tantivy) and Layer-B component (oya-mail-*). Drives `cost-budget.md` + `multi-region.md`.

## Inputs

| Input | Variable | Source |
|---|---|---|
| Active tenants | `N_tenants` | OpenBao tenant-resolver |
| Active mailboxes | `N_mailboxes` | Per-tenant `tenant_scope` × avg-mailboxes-per-tenant |
| Inbound mail rate (per sec, peak) | `I_in_per_sec` | Per-tenant baseline × `N_tenants` |
| Outbound submission rate (per sec, peak) | `O_out_per_sec` | analogous |
| Average mail size (KB) | `S_avg_kb` | typically 50 KB (median); 200 KB (with attachments) |
| Concurrent IMAP sessions | `C_imap` | per-user-active × `N_mailboxes` × 0.3 |
| Search queries/sec | `Q_search_per_sec` | per-active-user × users-online × 0.1 |
| Legal hold engagement rate | `H_per_day` | per-tenant rare; ~0.01/tenant/day |
| eDiscovery export rate | `E_per_month` | per-regulated-tenant ~0.1/month |

## Mailbox Storage Sizing

```
mailbox_storage_per_user = avg_messages_per_user × S_avg_kb
                         × (1 + attachment_overhead_factor 1.2)
                         ÷ deduplication_efficiency (0.7 for content-addressable storage)

avg_messages_per_user = retention_years × 365 × 50 mail/day inbox average  (production-tier baseline)

total_mailbox_storage_TB = N_mailboxes × mailbox_storage_per_user
```

### Reference baselines

| Tier | N_mailboxes | mailbox_storage_per_user | total_mailbox_storage |
|---|---|---|---|
| XS (M03 launch; 20 tenants × 250 mailboxes avg) | 5000 | ~7 GB (7y × 50 mail/day × 50KB) | 35 TB |
| S (100 tenants × 500 mailboxes) | 50,000 | ~7 GB | 350 TB |
| M (1000 tenants × 500 mailboxes) | 500,000 | ~7 GB | 3.5 PB |
| L (10,000 tenants × 500 mailboxes) | 5,000,000 | ~7 GB | 35 PB |

Storage tier policy per `policy/data-residency.md`:
- 0–30d: standard (hot) — Postgres metadata + S3 standard for active blobs
- 30d–6mo: infrequent-access (warm) — S3 IA
- 6mo+: archive (cold) — S3 archive
- HIPAA pack: extend to 6y minimum; KR-FSS: 5y minimum

## SMTP Frontend Sizing

### Inbound SMTP (Postfix)

```
postfix_inbound_replicas = ceil(I_in_per_sec / 50) × replication_factor (3) × 1.3 buffer
```

Per Postfix benchmarks (`postfix.org/STRESS_README.html`): single Postfix instance handles ~50 mail/sec sustained.

### Outbound SMTP

```
postfix_outbound_replicas = ceil(O_out_per_sec / 50) × replication_factor (3) × 1.3 buffer

# Outbound also needs deliverability queue capacity:
deliverability_queue_capacity = O_out_per_sec × max_retry_window_seconds (5 days = 432000)
                              × avg_retry_count (3)
```

### Rspamd (spam/phishing classifier)

```
rspamd_replicas = ceil(I_in_per_sec / 100) × 1.2 buffer
```

### Reference baselines

| Tier | I_in_per_sec | O_out_per_sec | Postfix-inbound | Postfix-outbound | Rspamd | OpenDKIM |
|---|---|---|---|---|---|---|
| XS | 5 | 5 | 4 | 6 | 3 | 4 |
| S | 50 | 50 | 12 | 16 | 6 | 8 |
| M | 500 | 500 | 100 | 130 | 25 | 30 |
| L | 5000 | 5000 | 800 | 1000 | 200 | 200 |

## Postgres Mailbox-Store Sizing

```
postgres_mailbox_store_replicas = primary 1 + sync_replicas 2 (HA) × buffer 1.2

postgres_storage_GB = N_mailboxes × avg_metadata_per_mailbox_MB (typically 5 MB metadata + WAL)
                    + retention_ledger_GB (small; ~1 GB per 10M events)
                    + legal_hold_lifecycle_GB (small; ~10 MB per active hold)

# Citus distributed table activation threshold:
when postgres_size_TB > 5 TB OR ingest_rate_qps > 5000:
    activate Citus distributed table by tenant_id (shard count = ceil(N_tenants / 100))
```

### Reference baselines

| Tier | Postgres replicas | Postgres storage | Citus activation? |
|---|---|---|---|
| XS | primary + 2 sync (3 total) | 50 GB | NO |
| S | primary + 2 sync + 4 read-replicas | 500 GB | NO |
| M | Citus coordinator + 50 workers | 5 TB | YES (50 shards) |
| L | Citus coordinator + 500 workers | 50 TB | YES (500 shards) |

## S3 MIME Blob Storage Sizing

```
s3_blob_storage_TB = total_mailbox_storage_TB - postgres_metadata_TB  # blob is the bulk
                   × 1 (dedup already factored)
                   × per_pack_replication_factor (1.0 single-region; 1.2 with cross-region for DR pair)
```

S3 standard pricing at OCI ~$0.0255/GB/month; archive at ~$0.0025/GB/month.

## Tantivy Search Index Sizing

```
tantivy_index_GB = total_mailbox_storage_TB × 0.05  # encrypted-token index is ~5% of raw mail size
                 × N_tenants_factor (per-tenant sharded)

tantivy_query_replicas = ceil(Q_search_per_sec / 100) × 1.3
```

### Reference baselines

| Tier | Tantivy storage | Tantivy query replicas |
|---|---|---|
| XS | 2 GB | 4 |
| S | 20 GB | 12 |
| M | 200 GB | 50 |
| L | 2 TB | 500 |

## IMAP/JMAP/REST Frontend Sizing

```
imap_frontend_replicas = ceil(C_imap / 5000) × 1.5 buffer  # 5k concurrent sessions per pod

jmap_frontend_replicas = ceil(C_jmap_per_sec / 100) × 1.3 buffer
rest_frontend_replicas = ceil(C_rest_per_sec / 200) × 1.3 buffer
```

## Layer-B Sizing (oya-mail-*)

```
mailbox_store_app_replicas = max(4, ceil(I_in_per_sec × 2 / 100)) × 1.5 HA buffer
mailbox_store_worker_replicas = 2 (HA; nightly retention sweep)
legal_hold_app_replicas = 2 (HA min)
legal_hold_worker_replicas = max(2, ceil(active_holds / 100))
search_index_worker_replicas = max(2, ceil(N_mailboxes / 100000))
```

For M03 XS launch (5000 mailboxes), worker_replicas = 2 (HA min suffices).

## Headroom + Burst

- **Pre-warmed pool**: 2 standby pods per critical surface (inbound-smtp, outbound-smtp, imap-frontend, mailbox-store-app).
- **HPA**: scales on CPU > 70% OR queue depth thresholds.
- **VPA**: vertical scaling for non-critical components (retention sweep worker, search rebuild).
- **Burst capacity**: peak mail rate can be 3× baseline; HPA pre-scale on calendar (e.g., Monday morning typically 2× Sunday).

## Worked Example: oyatie XS tier (M03 launch; 20 tenants pack-kr-only)

```
N_tenants = 20
N_mailboxes = 5000 (avg 250 per tenant)
I_in_per_sec = 5 (peak)
O_out_per_sec = 5 (peak)
S_avg_kb = 50 KB median (200 KB with attachments)
C_imap = 1500 concurrent sessions
Q_search_per_sec = 25

Storage:
  total_mailbox_storage = 5000 × 7 GB = 35 TB
  s3_hot (30d) = 35 TB × 30/2555 (7y) = 0.4 TB hot
  s3_warm (30-180d) = 35 TB × 150/2555 = 2 TB warm
  s3_cold (180d+) = 35 TB × 2375/2555 = 32 TB cold
  Postgres metadata = 5000 × 5 MB = 25 GB metadata + 5 GB WAL + ledger ≈ 50 GB
  Tantivy = 35 TB × 0.05 = 1.8 TB encrypted-token index

Replicas:
  postfix_inbound = ceil(5/50) × 3 × 1.3 ≈ 4
  postfix_outbound = 4 (with reputation queue capacity for 5 day retry × 5/s × 3 retries = 6.5M slots)
  rspamd = 3
  imap_frontend = ceil(1500/5000) × 1.5 ≈ 1 (rounded to 2 HA)
  postgres = primary + 2 sync (3 total)
  tantivy_query = ceil(25/100) × 1.3 ≈ 1 (rounded to 4 HA shards)
  mailbox_store_app = max(4, ceil(5×2/100) × 1.5) = 4
  legal_hold_app = 2 (HA min)

Total observability storage (XS, M03 launch):
  ~50 TB / pack region all-tiers
  ~$1500/month per pack region (mix of hot+warm+archive)
```

Cost projections per scale tier in `cost-budget.md`.

## Per-Pack Region Multipliers

- **DR pair packs** (pack-eu, pack-us, pack-au, pack-in, pack-br, pack-ae, pack-ksa): 1.0× primary + 0.6× warm-standby.
- **HIPAA pack** (pack-us-healthcare): 1.4× base (6y retention extension; HIPAA-eligible region exclusivity).
- **KR-FSS** tenants in pack-kr: 1.2× base (5y retention; KMS-in-KR).

## Verification

- `cargo run -p oya-dev-cli -- gate validate capacity-conformance --microservice mail` — exit 0; replica counts ≥ formula minimums.
- Quarterly capacity review: actual vs forecast; recalibrate `avg_messages_per_user`.
- Annual reference-architecture refresh: re-verify Postfix + Tantivy benchmarks.

## References

- Postfix sizing — `postfix.org/STRESS_README.html`.
- Dovecot scaling — `doc.dovecot.org/admin_manual/scaling/`.
- Rspamd performance — `rspamd.com/doc/configuration/performance.html`.
- Tantivy benchmarks — `github.com/quickwit-oss/tantivy`.
- Postgres + Citus — `citusdata.com/docs/`.
- OCI object-storage pricing — `oracle.com/cloud/storage/pricing/`.
- `microservices/mail/cost-budget.md`.
- `microservices/mail/multi-region.md`.
- `microservices/mail/policy/data-residency.md`.
