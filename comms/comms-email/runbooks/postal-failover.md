# Runbook — Postal failover

> ADR anchor: ADR-0201, IP-002, IP-013.
> Severity: SEV-1 for sovereign-tier clusters (no fallback);
> SEV-2 for cloud-hosted clusters with healthy SES.

## When to use

- Postal Helm release degraded (MariaDB primary failure,
  RabbitMQ broker offline, worker fleet < quorum).
- Postal regional outage in a sovereign-tier cluster.

## Prereqs

- Postal Helm chart deployed per
  `comms/comms-email/iac/helm/postal/`.
- OpenBao reachable for DKIM keys.

## Procedure

### Cloud-hosted (SES fallback available)

1. Confirm Postal degradation.
2. Page on-call (SEV-2).
3. IP-013 auto-fails-over to SES for affected tenants.
4. Audit-chain entry `routing.failover` emitted.
5. Recover Postal:
   - MariaDB primary: promote replica.
   - RabbitMQ: restart broker.
   - Worker fleet: scale-out HPA.
6. Validate Postal healthy; flip tenants back.

### Sovereign-tier (no fallback)

1. Page on-call (SEV-1).
2. The substrate **rejects new sends** for affected tenants
   (sovereign packs cannot fail over outside).
3. Recovery path is the same as above; SLA = 15 min RTO per
   ADR-0180.
4. Customer-comms per ADR-0180 SLA.

## Validation

- Postal Helm release status `Ready`.
- Send rate resumes; backlog drains within 60 min.
- Webhook ingest catches up; DLQ depth normalizes.

## Rollback

- Postal is the canonical sovereign-tier provider; there is no
  rollback path. Recovery is the path forward.

## Anti-patterns

- Sovereign-tier failover to SES (forbidden).
- Manual MariaDB writes during outage (causes inconsistency).

## References

- IP-002 Postal adapter implementation.
- IP-013 multi-region routing.
- ADR-0180 DR / BC.
- `multi-region.md`.
