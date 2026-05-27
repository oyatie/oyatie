# IP-001: Cluster Lifecycle Foundation

Crates:
- `oya-managed-k8s-cluster-lifecycle-kernel`: pure request values and validation.
- `oya-managed-k8s-cluster-lifecycle-api`: quota-before-provisioning orchestration.
- `oya-managed-k8s-cluster-lifecycle-app`: minimal HTTP composition root using deterministic in-memory adapters.

Acceptance:
- Validate tenant id, cluster name, tier, and requested resources before port calls.
- Invoke quota before provisioning.
- Do not provision on malformed request, quota denial, quota not-found, or quota failure.
- Map downstream provisioning failures after quota allow.
