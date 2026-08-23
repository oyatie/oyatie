---
contract: backup-canonical
authored: 2026-05-18
canonical_authority: ADR-0197
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/microservices/manifest-schema.json
related_adrs:
  - ADR-0152
  - ADR-0162
  - ADR-0165
  - ADR-0173-vendor-lock-in-avoidance-and-stack-ownership
  - ADR-0240-sovereign-cloud-per-regional-pack
  - ADR-0241-dr-business-continuity-portfolio-policy
  - ADR-0184
  - ADR-0186
  - ADR-0196
  - ADR-0197
  - ADR-0199
status: canonical-base
authorities_cited:
  - Stripe Engineering — PostgreSQL PITR + restore drill cadence (public availability retrospectives 2024-2025)
  - Pinterest Engineering — Velero + pgBackRest + Restic with chaos-engineering restore drills
  - Crunchy Data / Crunchy Bridge — pgBackRest production posture at multi-tenant SaaS scale
  - CNCF Velero project documentation — https://velero.io/
  - pgBackRest user guide — https://pgbackrest.org/user-guide.html
  - age encryption (Filippo Valsorda) — https://github.com/FiloSottile/age
---

# Backup canonical policy (RPO / RTO + restore drill cadence)

## Why this policy exists

ADR-0180 establishes the disaster-recovery portfolio. ADR-0197 mandates the
three-pronged backup substrate. This standards doc consolidates the
operational contract every µservice must honor:

1. **Which prong** backs up which surface.
2. **RPO / RTO** per workload class.
3. **Retention policy** per regulatory pack.
4. **Restore drill cadence + evidence** emit.

## Prong assignment (per surface)

| Surface                        | Prong              | Tool             | Storage target                    |
|--------------------------------|--------------------|------------------|-----------------------------------|
| Kubernetes manifests + CRDs    | K8s state          | Velero 1.18      | `velero-backup-shared-<env>` |
| Persistent volumes             | K8s state          | Velero + kopia   | (same bucket as Velero)           |
| Postgres database state        | Postgres PITR      | pgBackRest 2.58  | `backup-postgres-shared-<env>`|
| Postgres WAL archive           | Postgres PITR      | pgBackRest 2.58  | (same bucket)                     |
| Non-K8s host state (bastions)  | Filesystem         | Restic 0.18      | `backup-filesystem-shared-<env>` |
| Audit-chain archival           | Object replication | SeaweedFS GEO    | `audit-chain-archive-shared-<env>` |

## RPO / RTO per workload class (per ADR-0152)

| Class      | RPO       | RTO    | Backup cadence                                      |
|------------|-----------|--------|-----------------------------------------------------|
| app        | ≤ 15 min  | ≤ 1 h  | Velero hourly + daily; pgBackRest WAL (continuous)  |
| batch      | ≤ 1 h     | ≤ 4 h  | Velero daily; pgBackRest WAL                        |
| gpu        | ≤ 1 h     | ≤ 4 h  | Velero daily (model weights ephemeral)              |
| regulatory | ≤ 5 min   | ≤ 30 m | Velero hourly + daily; pgBackRest WAL; cross-AZ rep |

## Retention floor per regulatory pack

The baseline retention is **7 d hourly + 30 d daily + 12 m weekly + 7 y
annual**. Regulatory packs raise the floor — never lower it.

| Pack             | Floor                                  | Authority                       |
|------------------|----------------------------------------|---------------------------------|
| generic          | 7 y annual                             | baseline                        |
| kr               | 5 y annual                             | KR Personal Information Protection Act |
| eu               | 7 y annual                             | EU CSRD + financial retention   |
| us-healthcare    | 6 y annual                             | HIPAA                           |
| us-financial     | 7 y annual                             | SOX                             |
| us-public-sector | 7 y annual + 2 cross-region replicas   | FedRAMP                         |

The check `check-backup-retention-discipline` validates that the
declared retention on every µservice meets-or-exceeds its regulatory pack
floor.

## Encryption discipline

All backup payloads are age-encrypted before write per ADR-0197 D-3:

- One age key pair per `(tenant, prong)`.
- Public keys live in Helm values; private keys live in OpenBao under
  `secret/cloud-iac/<prong>/age-key-<tenant>`.
- Quarterly rotation per ADR-0043 secrets-rotation policy.
- The bucket-level KMS layer is additive: age ciphertext is what lands in
  the bucket.

## Restore drill cadence

- **Quarterly** restore drills are mandatory, run via the chaos-engineering
  substrate (ADR-0165).
- Drill targets one µservice per quarter on a rotation through the fleet.
- Procedure:
  1. Restore the target's Postgres primary to `dr-staging` from the
     most-recent pgBackRest full + applicable WAL up to `T-now - 5 min`.
  2. Restore associated PVs to `dr-staging` via Velero.
  3. Stand up the µservice's Deployments via Helm against the restored
     namespace.
  4. Run the µservice's smoke-test set.
  5. Measure observed RPO + RTO; compare to the workload-class targets.
- Evidence:
  - Drill outcome emitted to the audit chain as
    `class: BackupRestoreDrill`, signed by the chaos-substrate key.
  - Failed drill is a SEV-2 incident; the µservice owner has 7 days to
    remediate or the µservice is downgraded one promotion tier.

## CI gates this policy is enforced by

| Gate                                       | Lane mode          | Behavior                                  |
|--------------------------------------------|--------------------|-------------------------------------------|
| `check-backup-retention-discipline`    | advisory           | scans manifests; reports retention < pack floor |
| `check-tenant-cost-labels-coverage`    | advisory           | scans helm output; backup buckets must carry the tenant label block |

## Open questions parked here for follow-up

- The exact cadence + scoring rubric for "downgrade one promotion tier"
  on drill failure lives in `docs/standards/promotion-policy.md` (ADR-0181)
  and is referenced here, not duplicated.
- Cross-region replication for non-regulatory tiers is opt-in per pack;
  the decision when to make it default-on fleet-wide is parked for a
  future ADR (target: M03 horizon).

## References

- ADR-0197 — backup substrate (this doc's canonical authority).
- ADR-0196 — object storage canonical (the target bucket substrate).
- ADR-0152 — RPO/RTO tier mapping.
- ADR-0165 — chaos-engineering substrate (drill runner).
- ADR-0180 — DR + business continuity portfolio policy.
