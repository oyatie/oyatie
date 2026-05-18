---
doc_class: CostBudget
title: Cost Budget + FinOps Posture (audit-chain)
microservice: audit-chain
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-finops + axis-audit-chain + ops-sre-reliability
deciders: ops-finops, axis-audit-chain, ops-sre-reliability, council-architecture
related_adrs: [ADR-0117, ADR-0028, ADR-0131]
related_artifacts:
  - microservices/audit-chain/capacity-model.md
  - microservices/audit-chain/multi-region.md
review_cadence: monthly + on every capacity-model revision
doc_status: published
---

# Cost Budget + FinOps Posture (audit-chain µservice)

## Purpose

Track audit-chain monthly cloud cost across compute + storage + KMS + HSM + network. Numbers cite OCI public pricing (2026-05-17). HSM line items dominate the security floor; S3 WORM dominates the storage floor; per-pack overhead is real (one HSM partition + one Postgres + one bucket per pack).

## Cost Categories

| Category | What | OCI pricing reference |
|---|---|---|
| Compute (OKE) | emission-rest + verification-rest + query-rest + sealing-worker + retention-cascade-worker | `oracle.com/cloud/compute/pricing/` |
| Object storage (S3-compatible) | Raw event blobs + Merkle-tree blobs + signed-root blobs (WORM-Compliance-locked) | `oracle.com/cloud/storage/object-storage/pricing/` |
| Block storage (PV) | Postgres data volumes | `oracle.com/cloud/storage/block-volume/pricing/` |
| Postgres (managed or self-hosted) | per-pack HA primary + replica | `oracle.com/database/pricing/` |
| **OCI Cloud-HSM (per pack partition)** | Per-pack dedicated HSM partition (Ed25519 + KMIP/PKCS#11) | `oracle.com/security/key-management/pricing/` (HSM tier) |
| KMS | Per-pack KMS keyring for SSE-KMS on S3 + Postgres | as above |
| Network egress | Auditor export bundle delivery + cross-channel publication | `oracle.com/cloud/networking/pricing/` |
| Load balancer | Per-pack Istio + public ingress for verification endpoint | as above |
| Mimir (root-publication channel) | Cost attributed to `observability` µservice; audit-chain pays for marginal `oya_audit_chain_*` series | `observability/cost-budget.md` cross-reference |

## Per-Component Monthly Cost (XS tier, pack-kr region, M01 launch)

Per `capacity-model.md` §"Worked example: XS tier (20 tenants pack-kr)".

| Component | Replicas × instance-type | Monthly compute | Monthly storage | Monthly total |
|---|---|---|---|---|
| emission-rest | 4 × VM.Standard.E4 2-core | $145 | – | $145 |
| sealing-worker (leader + warm replica per shard; 4 shards) | 8 × VM.Standard.E4 4-core | $580 | – | $580 |
| verification-rest | 3 × VM.Standard.E4 2-core | $108 | – | $108 |
| query-rest | 3 × VM.Standard.E4 2-core | $108 | – | $108 |
| retention-cascade-worker | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| Postgres HA primary | 1 × VM.Standard.E4 8-core | $290 | $200 PV (1 TB) | $490 |
| Postgres replica | 1 × VM.Standard.E4 8-core | $290 | $200 PV | $490 |
| **OCI Cloud-HSM partition** | 1 dedicated partition | $1500 (HSM dedicated tier baseline) | – | $1500 |
| S3 WORM raw blobs | – | – | $260 hot (10 TB Mimir-equivalent) + $300 cold (120 TB Object-Lock-Compliance) | $560 |
| KMS keyring (SSE + audit-chain key encryption) | – | $15 | – | $15 |
| Load balancer (per-pack) | – | $20 | – | $20 |
| Mimir marginal series (root publication) | shared | $5 (marginal) | – | $5 |
| **XS tier total per pack region** | | **~$3133** | **~$960** | **~$4093 / month** |

Verify-at-deploy: OCI pricing changes; reconfirm at deploy. Buffer 20% for actual-vs-forecast on first deploy.

## HSM Cost Note

OCI Cloud-HSM dedicated tier is **the dominant fixed cost** per pack (~$1500/mo). Cannot be amortized cross-pack (per `data-residency.md` chain locality). Implications:
- Conditional packs (everything except pack-kr at M01) carry $1500/mo HSM cost from first-tenant onward.
- Multi-region DR pairs need two partitions: 2× $1500 = $3000/mo per DR-pair pack.
- Total HSM cost at all 11 packs activated with DR pairs where applicable: ~$22500/mo (11 primary + 7 DR-pair = 18 HSM partitions).

This is the load-bearing cost trade-off vs Bominal ADR-0028 inheritance: a non-HSM software-key option would save $1500/mo per partition but violates the eIDAS AdES + KR 전자문서법 posture for relevant packs. Decision: HSM is mandatory; cost is the price of compliance.

## Per-Scale-Tier Forecast

| Scale tier | N_tenants | Monthly cost per pack region | Notes |
|---|---|---|---|
| XS (M01 launch; 20 tenants) | 20 | ~$4100 | pack-kr only; HSM dominates |
| S (~100 tenants; 3 active packs) | 100 | ~$15k (per pack ~$5k) | 3 packs × ($5k compute+storage + $1.5k HSM) |
| M (~1000 tenants; 5 active packs) | 1000 | ~$80k | per-pack ~$16k; storage growth |
| L (~10000 tenants; 11 packs + DR pairs) | 10000 | ~$650k | 18 HSM partitions + scaled compute + storage |

## Per-Pack Multipliers

- **DR pair packs**: 1.0× primary + 0.6× warm-standby (compute) + 2 HSM partitions.
- **HIPAA pack-us-healthcare**: 1.5× base (6y retention vs 2-3y baseline) + dedicated HIPAA-eligible region.
- **KR-FSS regulated tenants in pack-kr**: 1.3× base (5y vs 3y default retention).
- **SAMA-finance pack-ksa**: 2.0× base (10y retention; dominant for SAMA-regulated tenants).
- **Single-region packs (kr, jp, sg)**: 1.0× base; no DR multiplier; single HSM partition.

## Budget + Alert Thresholds

| Metric | Threshold | Action |
|---|---|---|
| Monthly cost (per pack region) | within 90% of forecast | normal |
| 90% < cost < 110% | yellow | FinOps + ops-sre review |
| 110% < cost < 130% | orange | FinOps + leadership review; check autoscale + retention conformance |
| cost > 130% | red; budget breach incident | engage ops-finops + axis-audit-chain |
| HSM ops/s utilisation | within 80% of partition baseline | normal |
| HSM ops/s > 80% | yellow; consider partition upsize | provider engagement |
| Per-tenant cost (highest spender) | within 5× median tenant | normal |
| Per-tenant cost > 10× median | yellow; review tenant emission discipline | tenant comms |

## FinOps SLI

| SLI | Target | Burn-rate alert |
|---|---|---|
| Monthly cost / N_tenants (unit-economic) | within 5% of forecast | 6× burn over 6h |
| Storage growth / day (per pack) | within forecast | 14.4× burn over 1h (catches runaway emission rate) |
| HSM utilisation / partition | < 70% of published throughput | informational |
| Spot-vs-on-demand ratio | ≥ 60% spot for stateless components (verification-rest, query-rest, emission-rest) | informational |

## Cost-Optimisation Levers

| Lever | Estimated saving | Trade-off |
|---|---|---|
| Increase Merkle batch size (period 1s → 5s) | 5–10% compute (fewer signing calls) | Higher emit-to-seal lag for low-volume tenants |
| Aggressive S3 archive tier transition (30d → 14d) | 10% storage | Slower historical queries |
| Spot fleet for stateless components | 30–40% compute (emission/verification/query) | Spot eviction recovery via HA |
| OCI committed-use 1y/3y discounts | 20–40% compute + HSM partition | Vendor lock-in window |
| Per-tenant emission rate enforcement | 5–10% (caps runaway emitters) | Tenant disruption if too aggressive |
| Multi-tenant Postgres sharing (vs partition-per-pack) | NO — forbidden by residency model | Inapplicable |
| Software-key signing instead of HSM | $1500/mo per pack | Violates eIDAS AdES + KR 전자문서법 — REJECTED |

## Verification

- `cargo run -p oya-dev-cli -- gate validate cost-budget --microservice audit-chain` — exit 0; current spend within 110%.
- Monthly FinOps review.
- Quarterly capacity-model + cost-budget refresh.

## References

- `microservices/audit-chain/capacity-model.md`.
- `microservices/audit-chain/multi-region.md`.
- `microservices/audit-chain/policy/data-residency.md` (per-pack retention multipliers).
- OCI pricing — `oracle.com/cloud/pricing/`.
- OCI Cloud-HSM pricing — `oracle.com/security/key-management/pricing/`.
- FinOps Foundation framework — `finops.org`.
