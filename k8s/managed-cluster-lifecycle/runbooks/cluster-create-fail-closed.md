# Runbook: Cluster create fail-closed

1. Confirm tenant id and cluster name are non-empty and tier is `hosted` or `dedicated`.
2. For `quota_denied`, inspect quota and usage records.
3. For `quota_unavailable`, restore/seed quota; do not bypass quota.
4. For `provisioning_failed`, inspect control-plane-host management-cluster reachability.

Invariant: quota failures are admission failures, never a reason to manually call provisioning.
