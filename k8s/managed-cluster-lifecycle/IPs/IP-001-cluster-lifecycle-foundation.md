# IP-001: Cluster Lifecycle Foundation

Crates:
- `managed-k8s-cluster-lifecycle-kernel`: pure request values and validation.
- `managed-k8s-cluster-lifecycle-api`: quota-before-provisioning orchestration.
- `managed-k8s-cluster-lifecycle-app`: minimal HTTP composition root using deterministic in-memory adapters.

Acceptance:
- Validate tenant id, cluster name, tier, and requested resources before port calls.
- Invoke quota before provisioning.
- Do not provision on malformed request, quota denial, quota not-found, or quota failure.
- Map downstream provisioning failures after quota allow.
