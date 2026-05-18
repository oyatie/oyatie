---
doc_id: finops-portal/multi-region-strategy
authored: 2026-05-18
status: ready
authority: ADR-0064 canonical-base + ADR-0008 data-use-boundary
classification: internal
---

# Multi-region strategy — finops-portal

Per ADR-0064 (canonical-base + localization) and ADR-0008
(data-use-boundary), `finops-portal` ships as a canonical global
base with per-pack overlays. Each pack maps to one or more residency
regions.

## Region map

| Pack            | Primary region | Secondary region | Notes                                  |
|-----------------|----------------|------------------|----------------------------------------|
| generic         | us-east-1      | us-west-2        | Default; dev/staging fan-out           |
| kr              | kr-1 (Seoul)   | kr-2 (Busan)     | PIPA residency; data stays in KR       |
| eu              | eu-central-1   | eu-west-1        | GDPR; data stays in EEA                |
| us-healthcare   | us-east-1      | us-west-2        | HIPAA; encrypted-hipaa storage class   |
| us-financial    | us-east-1      | us-east-2        | SOX                                    |
| us-public-sector| us-gov-east-1  | us-gov-west-1    | FedRAMP                                 |

## Residency invariants

1. **Tenant data never crosses regions**. A tenant in KR sees only
   KR-resident replicas of finops-portal; KR ledger entries live
   only in KR postgres.
2. **Cedar policies enforce double-guard**:
   `principal.residency_region == resource.residency_region`.
3. **Helm overlays pin** `nodeSelector` + image registry to the
   region's local mirror.
4. **Audit-chain emits** to the per-region audit-chain instance;
   never cross-region.

## Cross-region failover (DR)

For the **generic** pack, failover is automatic across
us-east-1 ↔ us-west-2 via global load balancer + active-active
read replicas.

For the **kr / eu / us-healthcare / us-financial / us-public-sector**
packs, failover is **intra-region only** (primary → secondary
within the pack region). Cross-pack failover is **forbidden** by
both Cedar + NetworkPolicy.

## RPO / RTO per pack

| Pack            | RPO         | RTO       | Notes                                  |
|-----------------|-------------|-----------|----------------------------------------|
| generic         | 15 min      | 1 h       | per ADR-0152 `app` class                |
| kr              | 5 min       | 30 min    | tightened — FSS expectation             |
| eu              | 15 min      | 1 h       | GDPR-compliant; encrypted backups       |
| us-healthcare   | 5 min       | 15 min    | HIPAA — clinical-grade availability     |
| us-financial    | 5 min       | 15 min    | SOX — control-period coverage           |
| us-public-sector| 15 min      | 1 h       | FedRAMP moderate                         |

## Promotion sequence (rollout per pack)

A new finops-portal release rolls through packs in this order:

1. **generic-dev** → smoke tests pass.
2. **generic-staging** → canary 10 % → 50 % → 100 % over 24 h.
3. **eu-staging** → canary; observation window 48 h.
4. **kr-staging** → canary; observation window 48 h.
5. **us-healthcare-staging** → canary; observation window 72 h
   (longer per HIPAA change-control).
6. **us-financial / us-public-sector** sequenced per
   `runbooks/finops-portal-deploy-rollback.md`.
7. **Production**: same order; held by SLO-gated promotion per
   ADR-0130 (no promote unless prior-stage SLOs green for 24h).

## Image registry strategy

- `ghcr.io/oyatie/finops-portal` is the canonical source.
- Per-region mirrors auto-replicate on tag publish:
  - `kr-registry.oya.internal`
  - `eu-registry.oya.internal`
  - `us-hc-registry.oya.internal`
- Pull policy: `IfNotPresent`; tags are immutable.

## Cell topology

Within a region, finops-portal pods are distributed across at least
3 availability zones / cells per ADR-0152. The cell-cost-attribution
label propagates through OpenCost and surfaces in the fleet-rollup
dashboard.

## Cross-pack regulator reads (forbidden)

A regulator principal authorized in pack X cannot read evidence
from pack Y. Enforced by:

1. Cedar policy `regulator-evidence-emit.cedar` double-guard.
2. NetworkPolicy egress allow-list per pack.
3. Audit-chain class `CrossPackReadAttemptDenied` emits if
   attempted; SEV-3.

## References

- ADR-0064 canonical-base + localization.
- ADR-0008 data-use-boundary.
- ADR-0130 SLO-gated promotion.
- ADR-0152 RPO/RTO classes.
- `capacity-model.md`.
- `compliance-matrix.md`.
