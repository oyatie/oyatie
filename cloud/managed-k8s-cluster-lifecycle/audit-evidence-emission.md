# Managed K8s Cluster Lifecycle — Audit Evidence Emission

## Current evidence ceiling

Cluster-lifecycle currently documents the deterministic create-admission foundation:
gateway tenant principal validation, quota-before-provisioning, fail-closed
admission, and control-plane-host invocation only after quota allow. It does not
claim sealed audit-chain emission, operation-ledger persistence, production
readiness, public SLA evidence, billing readiness, or measured SLO compliance.

## Dependency boundary

Quota decisions are delegated to `managed-k8s-tenant-quota` through the
`QuotaDecisionPort` dependency. Audit evidence for quota-service decision or quota
administration events belongs to that service's authority; cluster-lifecycle may
reference the dependency result only as an admission input.

## Follow-on target

A future cluster-lifecycle operation-ledger/audit build may emit lifecycle-scoped
events such as create-admission requested, quota denied, quota unavailable,
provisioning invoked, and provisioning failed. Until that build exists, these are
design targets only and must not be described as live audit-chain behavior.
