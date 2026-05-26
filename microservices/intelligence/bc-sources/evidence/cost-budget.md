---
doc_class: CostBudget
microservice: foundry-evidence
status: Accepted
date: 2026-05-17
owner_team: axis-foundry-evidence + ops-finops
related_artifacts:
  - microservices/intelligence-evidence/PRD.md
  - microservices/intelligence-evidence/capacity-model.md
  - microservices/audit-chain/cost-budget.md  (substrate; this doc references)
doc_status: published
---

# foundry-evidence — cost budget

## Target cost

| Item | Cost target | Rationale |
|---|---|---|
| Per-invocation pack assembly | ≤ $0.0001 fully-loaded | Aggressively below the per-invocation cost of foundry-runtime so evidence is never the gating cost |
| Per-bundle regulator-export | ≤ $0.10 per 10k-pack bundle | One-off; amortised over engagement |
| Per-query evidence-query | ≤ $0.000005 | Postgres B-tree lookup + Cedar evaluation + audit-of-audits emit |
| Per-pack storage (hot, 90d) | ≤ $0.00005 per pack-day | Postgres + WORM-hot (substrate); per-pack-day amortised |
| Per-pack storage (warm, 1y) | ≤ $0.00001 per pack-day | S3 IA |
| Per-pack storage (cold, multi-year) | ≤ $0.000002 per pack-day | S3 Glacier Deep Archive |

## Cost drivers — pack assembly

| Driver | Estimated cost per pack | Source |
|---|---|---|
| Postgres INSERT (HA + read-replicated) | $0.00001 | per OCI Database for PostgreSQL pricing; per-row insert amortised |
| audit-chain bridge emit (delegated cost) | $0.00001 | substrate; see `microservices/audit-chain/cost-budget.md` "emit" line item |
| Pack-builder CPU + memory | $0.00006 | per-pack pod-cycle amortised at 20 k packs/s |
| Cedar evaluation | $0.0000005 | per-policy evaluation; negligible |
| Workflow event consumption (4 inbound topics) | $0.00002 | per-pack share of event-bus cost |
| **Total per-pack** | **≤ $0.0001** | |

## Cost drivers — regulator-export

| Driver | Estimated cost per 10k-pack bundle | Source |
|---|---|---|
| Postgres range scan | $0.005 | per OCI pricing; B-tree range scan |
| Framework-profile field selection | $0.001 | CPU + memory |
| audit-chain pack-set Merkle-bundling | $0.02 | substrate-delegated |
| Ed25519 bundle signature | $0.001 | HSM via audit-chain bridge |
| S3 export-bucket upload | $0.05 | egress + storage; 10k-pack bundle ~50 MB |
| Workflow notification emits | $0.02 | regulator_export.{requested,completed,reissued} events |
| **Total per-bundle** | **≤ $0.10** | |

## Cost drivers — evidence-query

| Driver | Per-query cost |
|---|---|
| Postgres B-tree lookup | $0.000001 |
| Cedar evaluation | $0.0000005 |
| audit-of-audits emit | $0.000003 |
| Pre-signed URL for plaintext (when Cedar-permitted) | $0.0000005 |
| **Total per-query** | **≤ $0.000005** |

## Headroom + budget envelope (M01 launch)

| Pack | Forecast packs/day at M01 | Daily ingest cost | Daily storage cost (hot) | Monthly envelope |
|---|---|---|---|---|
| pack-kr | 100 M | $10 | $500 | ≤ $15,300 |
| pack-eu | 200 M | $20 | $1,000 | ≤ $30,600 |
| pack-us | 300 M | $30 | $1,500 | ≤ $45,900 |
| pack-us-healthcare | 50 M | $5 | $250 | ≤ $7,650 |
| Other packs (combined) | 100 M | $10 | $500 | ≤ $15,300 |
| **Total M01** | **750 M** | **$75** | **$3,750** | **≤ $114,750** |

## Cost guardrails

- **Per-tenant cost cap**: emitted as Mimir metric `oya_foundry_evidence_per_tenant_cost_estimate_usd_per_day`; alert at $50/day per tenant (Sev-3); $500/day (Sev-2).
- **Bundle cost cap**: regulator-export refuses bundles forecast > $5 without 2-person rule + council-privacy chair sign-off.
- **Archive cascade**: forced if hot-tier blob storage exceeds 90d retention budget for any pack; runs before exceeding ceiling.

## Cost gates (CI-enforced)

- `oya gate validate cost-budget --microservice foundry-evidence` checks declared per-call costs against drill-measured costs.
- `hyperscaler-maturity-claims` lane refuses any claim that "≤ $0.0001/pack" is achieved if drill cost exceeds the target.

## ADR-0133 honest gap

If at scale-up the per-pack cost exceeds $0.0001, the cost-budget MUST be revised + an ADR filed before the next milestone. Cost is treated as a contract, not aspiration.

## References

- `microservices/intelligence-evidence/capacity-model.md`.
- `microservices/audit-chain/cost-budget.md` (substrate-delegated lines).
- OCI Database / Object Storage / Cloud-HSM pricing (referenced at the date of this document).
