---
doc_class: CostBudget
microservice: cloud-secrets
status: Accepted
date: 2026-05-17
owner_team: ops-finance + axis-cloud-secrets
related_adrs: [ADR-0117, ADR-0131]
review_cadence: quarterly + on every pack activation
doc_status: published
---

# Cost Budget: cloud-secrets µservice

## Purpose

Bound the steady-state monthly cost of the cloud-secrets substrate per pack. Drive scale-out triggers, vendor-selection decisions, and per-tenant unit-economics modelling.

## Cost Drivers

| Driver | Unit | Cost surface |
|---|---|---|
| OpenBao compute | 5-node Raft cluster per pack; each node = OCI VM.Standard.E4.Flex 4 oCPU + 32 GiB | OCI compute |
| Postgres backend (HA Patroni) | 3-node cluster per pack; each = OCI VM.Standard.E4.Flex 4 oCPU + 64 GiB + block storage | OCI compute + storage |
| OCI Cloud-HSM partition | per-pack partition (1) + HA replica (1) | OCI HSM-as-a-Service |
| Thales Luna HSM (pack-kr) | partition on dedicated appliance | Thales managed-service or CapEx |
| Object storage (backups + audit) | per-pack bucket; ~100 GiB/month growth | OCI Object Storage |
| Network egress (cross-AZ replication) | intra-pack Raft + intra-pack Postgres sync | OCI VCN egress |
| HSM signing operations | per-op cost on managed-HSM tiers | OCI Cloud-HSM per-op |
| Audit-chain bridge traffic | per-event to audit-chain µservice | OCI VCN intra-cluster (free) |
| OpenBao license | OpenBao is OSS Apache-2.0 — $0 | n/a |
| cert-manager + SPIRE | OSS — $0 | n/a |
| Helm operator | OSS — $0 | n/a |

## Steady-State Monthly Budget (per pack, USD)

| Component | pack-kr (M01 launch) | pack-eu (DR-pair) | pack-us | pack-us-healthcare | pack-jp | pack-sg | pack-au (DR-pair) | pack-in (DR-pair) | pack-br (DR-pair) | pack-ae (DR-pair) | pack-ksa (DR-pair) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| OpenBao compute (5×E4.Flex 4oCPU/32G) | $1,600 | $3,200 | $1,600 | $1,600 | $1,600 | $1,600 | $3,200 | $3,200 | $3,200 | $3,200 | $3,200 |
| Postgres HA (3×E4.Flex 4oCPU/64G + 500GB block) | $1,200 | $2,400 | $1,200 | $1,200 | $1,200 | $1,200 | $2,400 | $2,400 | $2,400 | $2,400 | $2,400 |
| OCI Cloud-HSM partition (1 + HA replica) | n/a (Luna) | $2,800 | $1,400 | $1,400 | $1,400 | $1,400 | $2,800 | $2,800 | $2,800 | $2,800 | $2,800 |
| Thales Luna HSM (pack-kr) | $4,500 | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a |
| Object Storage (backups + audit; 1TB) | $25 | $50 | $25 | $25 | $25 | $25 | $50 | $50 | $50 | $50 | $50 |
| Network egress intra-pack | $150 | $300 | $150 | $150 | $150 | $150 | $300 | $300 | $300 | $300 | $300 |
| HSM signing ops (estimated 1M ops/month at $0.001) | $1,000 | $1,000 | $1,000 | $1,000 | $1,000 | $1,000 | $1,000 | $1,000 | $1,000 | $1,000 | $1,000 |
| **Pack subtotal (USD/month)** | **$8,475** | **$9,750** | **$5,375** | **$5,375** | **$5,375** | **$5,375** | **$9,750** | **$9,750** | **$9,750** | **$9,750** | **$9,750** |

**M01 launch monthly cost (pack-kr only):** $8,475 USD/month.

**All-packs activated steady-state estimate:** ~$96,475 USD/month before tenant-driven growth.

Note: Costs scale roughly linearly with active tenants once tenants exceed the per-pack OpenBao 1k-tenant baseline (Raft cluster scale-out: add second 5-node cluster per pack, doubling OpenBao + Postgres line items).

## Per-Tenant Unit Economics

| Tenant tier | Estimated tenant-attributed cost (USD/month) | Notes |
|---|---:|---|
| `sandbox` | $0.10 | shared sandbox tenant `tenant:cisandbox*`; capacity-only |
| `trial` (≤ 90d) | $0.50 | shared OpenBao namespace overhead |
| `production-small` | $2.00 | per-tenant namespace + estimated 10k secrets + 100k resolves/month |
| `production-medium` | $8.00 | 100k secrets + 1M resolves + cascade-rotation overhead |
| `production-large` | $35.00 | 1M secrets + 10M resolves + encryption-key BYOK HSM ops (ADR-0251 §D-10) |
| `production-regulated` (KR-FSS / HIPAA / KSA NCA) | $120.00 | dedicated HSM partition + extended retention + KEK ceremony amortisation |

Per-tenant cost includes per-pack overhead amortisation; unit cost falls as tenants per pack grow.

## Scale-Out Triggers (re-budget when crossed)

| Trigger | Action | Cost delta |
|---|---|---|
| OpenBao read qps > 70% of cluster capacity sustained 10min | Add 5-node read-replica cluster per pack | +$1,600/month |
| OpenBao write qps > 70% of leader capacity | Re-shard via additional namespace partition | architectural decision |
| HSM signing op queue > 200ms p99 | Add HSM partition | +$1,400/month (OCI) or +$4,500/month (Luna) |
| Postgres CPU > 70% sustained 1h | Scale up node size (E4.Flex 4 → 8 oCPU) | +$400/month/node |
| Audit emission backlog > 1s | Scale audit-emitter worker (HPA) + audit-chain capacity | $50/month/replica |
| Per-pack tenant count > 1000 | Activate secondary OpenBao cluster | pack subtotal × 2 |

## Cost-Avoidance Levers

| Lever | Annualised savings | Risk |
|---|---:|---|
| Use OCI Cloud-HSM for non-FSS pack-kr tenants instead of Luna | $30,000/yr | KR-FSS tenants still on Luna; mixed-vendor complexity |
| Move audit cold-storage to OCI Archive Tier | $5,000/yr | Restore SLA is hours, not seconds; acceptable for audit cold |
| Defer pack-ae / pack-ksa activation until first paying tenant | $234,000/yr (scheduled-for-distinct-tracked-work) | Sales blockers if reactive activation lag > 90 days |
| Single-region (no DR-pair) for non-regulated packs | $4,200/pack/month | Violates 99.99% SLO target if region-fail |

## Cost Anti-Patterns (forbidden)

| Anti-pattern | Why forbidden |
|---|---|
| Cross-pack replication "to save HSM cost" | Violates residency contract (data-residency.md) |
| Shared HSM partition across tenants in regulated packs | Violates per-tenant DEK isolation; regulatory breach |
| Disabling Postgres backups "to save storage" | Violates RPO ≤1s for audit-emission backlog recovery |
| Single-node OpenBao "for dev parity" | No HA = production SLO unreachable; refuse in IaC review |

## Verification

```bash
cargo run -p oya-dev-cli -- gate validate cost-budget-conformance --microservice cloud-secrets
# Cross-checks Helm replica counts + node sizes against this budget.
```

Quarterly: ops-finance + axis-cloud-secrets joint review; reconciliation against OCI billing.

## References

- ADR-0117 (Cloud-native infrastructure)
- ADR-0131 (Cloud split)
- `microservices/cloud-secrets/capacity-model.md`
- `microservices/cloud-secrets/multi-region.md`
- OCI public pricing (canonical source of unit costs)
- Thales Luna HSM list price (CapEx-equivalent monthly amortisation)
