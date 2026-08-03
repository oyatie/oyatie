# Runbook: Cluster create fail-closed

1. Confirm tenant id and cluster name are non-empty and tier is `hosted` or `dedicated`.
2. For `quota_denied`, inspect the dependency-owned quota decision and requested dimensions.
3. For `quota_unavailable`, restore the `managed-k8s-tenant-quota` dependency; do not bypass quota.
4. For `provisioning_failed`, inspect control-plane-host management-cluster reachability.

Invariant: quota failures are admission failures, never a reason to manually call provisioning.
