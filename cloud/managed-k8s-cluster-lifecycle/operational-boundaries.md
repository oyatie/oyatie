# Managed K8s Cluster Lifecycle — Operational Boundaries

## Capacity model

- Current scope is the dogfood/design deterministic create-admission foundation.
- The lifecycle hot path is bounded to request validation, one quota-decision port
  call, and one control-plane-host port call only after quota allow.
- Cluster-lifecycle owns no quota store, quota RBAC administration, provider
  reconciler, operation ledger, billing adapter, or measured SLO pipeline in this
  wave.

## Incident response

- On quota dependency failure: fail closed; do not invoke control-plane-host.
- On quota deny or not-found: return an admission failure and require quota-service
  remediation through the `managed-k8s-tenant-quota` owner path.
- On malformed or mismatched tenant principal: reject before any dependency call.
- On control-plane-host failure after quota allow: surface provisioning failure;
  do not claim live provider rollback or reconciliation until follow-on operation
  ledger support exists.

## Multi-region and production claims

- No production multi-region replication, public SLA, billing readiness, DPIA
  completion, or measured SLO compliance is claimed by this service-local document.
- Future placement, operation-ledger, and observability work must add explicit
  evidence before upgrading the claim ceiling.
