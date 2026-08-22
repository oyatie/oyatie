---
ip_id: cloud-iac/IP-velero-pgbackrest-restic-bootstrap
authored: 2026-05-18
slice_owner: axis-cloud-iac
related_adrs: [ADR-0152, ADR-0162, ADR-0165, ADR-0173-vendor-lock-in-avoidance-and-stack-ownership, ADR-0196, ADR-0197]
ip_status: planned
---

# IP — Backup substrate bootstrap (Velero + pgBackRest + Restic)

## Why this slice

ADR-0197 mandates the three-pronged backup substrate, each prong landing
in a SeaweedFS bucket and encrypted with age before write. The bootstrap
slice deploys all three prongs to `dev` with the canonical retention
schedules + sigil schedules + restore-drill cadence.

## Acceptance criteria

1. Velero 1.18 chart at `microservices/cloud-iac/iac/helm/velero/`
   deployed; backup schedules render per
   `values.yaml#configuration.schedules`.
2. pgBackRest 2.58 sidecar chart deployed alongside the canonical
   per-µservice Postgres pattern; WAL archive lands in
   `backup-postgres-shared-<env>`.
3. Restic 0.18 daemonset deployed on bare-metal nodes; filesystem
   archive lands in `backup-filesystem-shared-<env>`.
4. age recipient public keys per `(tenant, prong)` provisioned via
   OpenBao operator.
5. First backup of each prong completes; first restore-drill in
   `dr-staging` succeeds within the workload-class RTO target per
   ADR-0197 D-4.
6. `check-backup-retention-discipline` reports no blocking
   findings.

## File-level work plan

1. Helm charts (DONE this batch).
2. age key generation + OpenBao seeding (FOLLOW-UP).
3. ArgoCD ApplicationSet entries (FOLLOW-UP).
4. Chaos-substrate restore-drill scheduler wiring (FOLLOW-UP per
   ADR-0165).

## Risks

- pgBackRest maintainer transition; pgxbackup adapter swap path
  documented in ADR-0197 §In-house roadmap.
- Velero CNCF Sandbox path; in-house orchestrator named as the Phase 2
  replacement.

## References

- ADR-0197 — backup substrate canonical (this slice's authority).
- `docs/standards/backup-canonical.md`.
- `shared-backup-kernel` (this batch).
