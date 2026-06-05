---
doc_class: CostBudget
title: Cost Budget + FinOps Posture
microservice: mail
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-finops + axis-mail + ops-sre-reliability
deciders: ops-finops, axis-mail, ops-sre-reliability, council-architecture
related_adrs: [ADR-0117, ADR-0135, ADR-0131]
related_artifacts:
  - microservices/mail/capacity-model.md
  - microservices/mail/multi-region.md
review_cadence: monthly + on every capacity-model revision
doc_status: published
---

# Cost Budget + FinOps Posture (mail µservice)

## Purpose

Track mail µservice monthly cloud cost across infrastructure (compute + storage + network), per Layer-A + Layer-B component, per pack region; surface budget breach via `oya-check-cost-budget` LEAN lane. Numbers cite OCI public pricing (2026-05-17); verify-at-deploy markers called out where vendor pricing may have moved.

## Cost Categories

| Category | What | OCI pricing reference |
|---|---|---|
| Compute (OKE node) | Postfix SMTP + Dovecot/Rust-IMAP + Rspamd + Tantivy + worker pods + REST sidecars | `oracle.com/cloud/compute/pricing/` |
| Postgres (HA Citus distributed when needed) | Mailbox metadata (per-tenant RLS); retention ledger; legal-hold lifecycle | `oracle.com/database/postgresql-pricing/` |
| Object storage (S3-compatible) | MIME blobs (CAS); eDiscovery export bundles; cold-tier retention archive | `oracle.com/cloud/storage/object-storage/pricing/` |
| Block storage (PV) | SMTP queue spool; Tantivy index local cache; Postfix mail queue | `oracle.com/cloud/storage/block-volume/pricing/` |
| Network egress | Outbound SMTP delivery to Internet recipient MX; eDiscovery downloads; cross-region replication (rare) | `oracle.com/cloud/networking/pricing/` |
| KMS | Per-tenant DEK envelope + per-tenant DKIM keys + per-user personal-pillar DEK | `oracle.com/security/key-management/pricing/` |
| Load balancer | Per-pack Istio gateway + SMTP edge LB | `oracle.com/cloud/networking/load-balancing/pricing/` |
| Per-tenant SMTP IP allocation (BYOIP or OCI-managed) | Per-tenant pool sizes 1-8 IPs in M03 launch | `oracle.com/cloud/networking/byoip-pricing/` |
| External-provider feedback-loop subscriptions | M³AAWG / Gmail FBL / Outlook JMRP / Yahoo CFL | varies (mostly free) |

## Per-Component Monthly Cost (XS tier, single pack-kr region, M03 launch)

Per `capacity-model.md` §"Worked example: oyatie XS tier (M03 launch; 20 tenants pack-kr-only)".

| Component | Replicas × instance-type | Monthly compute | Monthly storage | Monthly total |
|---|---|---|---|---|
| inbound-smtp (Postfix receive) | 4 × VM.Standard.E4 2-core | $144 | $50 PV (queue spool) | $194 |
| outbound-smtp (Postfix submit + delivery) | 6 × VM.Standard.E4 2-core | $216 | $80 PV (queue spool) | $296 |
| imap-frontend (Dovecot or Rust-IMAP) | 4 × VM.Standard.E4 2-core | $144 | – | $144 |
| Rspamd (spam/phishing) | 3 × VM.Standard.E4 2-core | $108 | $20 PV (Bayes DB) | $128 |
| OpenDKIM (sign/verify) | 4 × VM.Standard.E4 1-core | $72 | – | $72 |
| Postgres mailbox-store HA (primary + 2 replicas) | 3 × VM.Standard.E4 4-core | $435 | $400 PV (mailbox metadata + WAL) | $835 |
| Postgres retention-ledger | 2 × VM.Standard.E4 2-core | $72 | $50 PV | $122 |
| S3 MIME blobs | – | – | $500 hot (20 TB) + $300 cold (120 TB archive) | $800 |
| Tantivy search index (per-tenant sharded) | 4 × VM.Standard.E4 4-core | $290 | $200 PV (index cache) | $490 |
| oya-mail-mailbox-store-app | 4 × VM.Standard.E4 2-core | $144 | – | $144 |
| oya-mail-mailbox-store-worker (retention sweep) | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| oya-mail-legal-hold-app + worker | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| oya-mail-search-index-worker (rebuild + index) | 2 × VM.Standard.E4 4-core | $145 | – | $145 |
| oya-mail-inbound-smtp-app | 4 × VM.Standard.E4 2-core | $144 | – | $144 |
| oya-mail-outbound-smtp-app (reputation + bounce) | 4 × VM.Standard.E4 2-core | $144 | – | $144 |
| oya-mail-imap-frontend-app | 4 × VM.Standard.E4 2-core | $144 | – | $144 |
| KMS keyring (per-pack) | – | $20 | – | $20 |
| Load balancer + SMTP edge LB | – | $80 | – | $80 |
| Per-tenant SMTP IP allocation (20 tenants × 2 IPs avg) | 40 IPs | $200 | – | $200 |
| **XS tier total per pack region** | | **~$2640** | **~$1600** | **~$4240 / month** |

Verify-at-deploy: OCI pricing changes; reconfirm against `oracle.com/cloud/pricing/`. Buffer 15% for rate increases + 20% for actual-vs-forecast.

## Per-Scale-Tier Forecast

| Scale tier | N_tenants | Active mailboxes | Mail/day | Monthly cost per pack region | Notes |
|---|---|---|---|---|---|
| XS (M03 launch; 20 tenants) | 20 | 5k | 100k | ~$4500 | pack-kr only |
| S (~100 tenants) | 100 | 50k | 1M | ~$18k | pack-kr + pack-eu + pack-us |
| M (~1000 tenants) | 1000 | 500k | 10M | ~$90k | 5 active packs |
| L (~10000 tenants) | 10000 | 5M | 100M | ~$800k | all 11 packs + multi-region per pack |

## Per-Pack Multipliers

- **DR pair packs** (pack-eu, pack-us, pack-au, etc.): 1.0× primary + 0.6× warm-standby.
- **HIPAA pack** (pack-us-healthcare): 1.4× base (HIPAA-eligible region + 6y retention multiplier).
- **KR-FSS-regulated tenants** in pack-kr: 1.2× base (KR-resident KMS + 5y retention floor).
- **Heavy-volume tenants** (>10M mail/month): linear scaling per `capacity-model.md`.

## Per-Tenant Unit Economics

| Tenant profile | Mailboxes | Mail/day | Monthly cost | Notes |
|---|---|---|---|---|
| Trial / SMB (1-50 mailboxes) | 25 | 200 | ~$3 | shared IP pool; trial scope |
| Mid-market (51-500 mailboxes) | 250 | 2k | ~$30 | small dedicated IP pool (2 IPs) |
| Enterprise (501-5000) | 2500 | 20k | ~$300 | dedicated IP pool (4-8 IPs); deliverability concierge |
| Large enterprise (5001+) | 10000+ | 100k+ | ~$1.5k+ | large dedicated IP pool + reputation team |

## Budget + Alert Thresholds

| Metric | Threshold | Action |
|---|---|---|
| Monthly cost (per pack region) | within 90% of forecast | normal |
| 90% < cost < 110% | yellow alert | FinOps + ops-sre-reliability review |
| 110% < cost < 130% | orange alert | FinOps + leadership; review autoscale + capacity-model |
| cost > 130% | red alert; budget breach | engage ops-finops + axis-mail; consider per-tenant tightening |
| Per-tenant cost projection (highest spender) | within 5× median | normal |
| Per-tenant cost > 10× median | yellow; engage tenant on cardinality discipline | tenant dashboard surfaces self-overage |
| eDiscovery export storage (cold-tier) | within 110% of forecast | normal |
| Single eDiscovery export > 100 GB | review with ops-legal | not a cost overrun but data discovery scope concern |

## FinOps SLI

| SLI | Target | Burn-rate alert |
|---|---|---|
| Monthly cost / N_active_mailboxes (unit-economic) | within 5% of forecast | 6× burn over 6h |
| Storage growth / day (avg over last 7d) | within forecast | 14.4× burn over 1h (catches runaway-attachment-volume) |
| Spot-vs-on-demand ratio | ≥ 70% spot for stateless SMTP frontends | informational |
| Per-tenant SMTP delivery cost / message | within forecast | 6× burn over 6h (catches spam wave) |

## Cost-Optimisation Levers

| Lever | Estimated saving | Trade-off |
|---|---|---|
| Mail-content compression (Brotli on MIME body) | 20-30% S3 cost | CPU cost; decompress on read |
| Aggressive deduplication of attachments (CAS) | 10-30% S3 cost | already in design; further dedupe yields diminishing |
| Cold-tier archive earlier (90d → 30d threshold) | 5-10% storage | Slower historical access |
| Spot-instance fleet for stateless frontends | 30-50% compute | Spot eviction recovery via HA |
| OCI committed-use discounts (1y / 3y) | 20-40% compute | Vendor lock-in window |
| Per-tenant cardinality budget on search index | 5-15% Tantivy compute | Tenant disruption if too aggressive |
| Per-tenant SMTP IP pool consolidation (low-volume tenants share pool) | 30-50% IP costs | Reputation cross-pollination risk; only for trial tier |
| KMS DEK envelope batch operations | 5-10% KMS cost | Slight latency increase |

## Verification

- `buck2 build //:quality-lane-registry-authority-check # lane=cost-budget --microservice mail` — exit 0; current spend within 110%.
- Monthly FinOps review: actual vs forecast.
- Quarterly: capacity-model + cost-budget refresh.

## References

- `microservices/mail/capacity-model.md`.
- `microservices/mail/multi-region.md`.
- `microservices/mail/policy/data-residency.md` (per-pack retention multipliers).
- OCI pricing — `oracle.com/cloud/pricing/`.
- Postfix sizing guide — `postfix.org/STRESS_README.html`.
- Tantivy benchmarks — `github.com/quickwit-oss/tantivy`.
- FinOps Foundation framework — `finops.org`.
