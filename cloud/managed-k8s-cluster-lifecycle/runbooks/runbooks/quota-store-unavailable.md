# Runbook: Quota Dependency Unavailable During Cluster Create

## Symptom
The `managed-k8s-tenant-quota` dependency cannot return a quota decision for a
cluster create request.

## Impact
Cluster-lifecycle cannot provision new clusters (fail-closed by design).

## Steps
1. Check the `managed-k8s-tenant-quota` dependency health through its owner runbooks.
2. Review cluster-lifecycle logs for quota-decision dependency-unavailable or
   persistence-failure entries without treating tenant-quota internals as
   cluster-lifecycle authority.
3. Retry cluster create only after tenant-quota can return an explicit allow.
4. Verify cluster-lifecycle still fails closed while the dependency is unavailable.

## Prevention
Future quota dependency persistence and retry work belongs to the tenant-quota
service; cluster-lifecycle must continue to treat dependency failures as admission
denies.
