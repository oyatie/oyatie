# Managed K8s Cluster Lifecycle — Failure Modes

## Quota dependency unavailable
- **Impact**: `QuotaDecisionPort` returns an unavailable or persistence-style
  dependency error; cluster-lifecycle treats this as a hard admission deny and does
  not call control-plane-host.
- **Recovery**: restore the `managed-k8s-tenant-quota` dependency or retry with
  backoff. Do not bypass quota-before-provisioning.

## Quota denied or not found
- **Impact**: Quota dependency returns deny or not-found; cluster create admission
  fails closed.
- **Recovery**: remediate quota state through the tenant-quota service owner path,
  then retry cluster create.

## Missing or mismatched tenant principal
- **Impact**: Gateway tenant principal is absent, malformed, or inconsistent with
  the request; cluster-lifecycle rejects the request before quota or provisioning
  dependency calls.
- **Recovery**: fix gateway/authentication context and request shape; do not accept
  caller-supplied tenant override values.

## Control-plane-host failure after quota allow
- **Impact**: Quota admission allowed, but `managed-k8s-control-plane-host` returns
  a provisioning failure. Cluster-lifecycle surfaces the failure and does not claim
  live provider reconciliation or rollback behavior in the current foundation.
- **Recovery**: inspect control-plane-host reachability and follow its owner
  runbooks. Operation-ledger retry/rollback semantics are follow-on scope.
