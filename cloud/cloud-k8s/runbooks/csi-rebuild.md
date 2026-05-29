---
doc_class: Runbook
title: CSI driver rebuild (block-volume / object / file)
microservice: cloud-k8s
severity: "Sev-1 (backend outage) / Sev-2 (controller pod failure)"
status: Accepted
owner_team: ops-sre-reliability + axis-cloud + cloud-iac
date: 2026-05-17
related_artifacts:
  - microservices/cloud-k8s/failure-modes.md (FM-10, FM-15)
doc_status: published
---

# Runbook: CSI driver rebuild

## Trigger

- CSI controller pod crashloop (FM-10)
- CSI node-plugin DaemonSet stuck on a node
- Backend storage outage (FM-15: OCI Block / Object / File service unavailable in pack region)
- PVC bind failures

## Severity

Sev-1 (backend outage; stateful workloads cannot bind new PVs) / Sev-2 (controller failure; existing mounts continue).

## CSI controller pod failure (FM-10)

| Step | Action | Time |
|---|---|---|
| 1 | Identify failed backend: block / object / file? `kubectl -n kube-system get pods -l app=csi-<backend>-controller` | ≤ 2 min |
| 2 | Capture logs: `kubectl -n kube-system logs <csi-controller-pod> --previous > /tmp/csi-fail.log` | ≤ 2 min |
| 3 | Verify backend reachability from pod: `kubectl exec <pod> -- curl <backend-endpoint>/health` | ≤ 5 min |
| 4 | Check CSI driver version compatibility with current k8s version | ≤ 5 min |
| 5 | Restart controller deployment: `kubectl -n kube-system rollout restart deployment csi-<backend>-controller` | ≤ 5 min |
| 6 | Verify HA: ≥ 2/2 replicas Ready | ≤ 5 min |
| 7 | Verify provisioning resumes: test PVC create + bind → cleanup | ≤ 10 min |

## CSI node-plugin DaemonSet stuck on a node

| Step | Action | Time |
|---|---|---|
| 1 | Identify stuck node: `kubectl -n kube-system get pods -l app=csi-<backend>-node -o wide | grep -v Running` | ≤ 2 min |
| 2 | Capture logs: `kubectl -n kube-system logs <node-plugin-pod>` | ≤ 2 min |
| 3 | If kernel module missing (rare; typically post-kernel-upgrade): SSH to node; verify module; `modprobe <needed>` | ≤ 10 min |
| 4 | If hostPath permissions: verify `/var/lib/kubelet/plugins/<csi>/` writable | ≤ 5 min |
| 5 | Restart pod: `kubectl -n kube-system delete pod <node-plugin-pod>` | ≤ 2 min |
| 6 | Verify Ready | ≤ 5 min |

## Backend outage (FM-15)

| Step | Action | Time |
|---|---|---|
| 1 | Verify backend service status (OCI status page) | ≤ 5 min |
| 2 | If pack has DR pair: assess DR failover trigger per `multi-region.md` § "DR Failover" | – |
| 3 | If no DR pair: graceful degradation — existing mounts continue; new PVCs queue with `Pending` status; tenants notified | – |
| 4 | Monitor backend recovery; cordon backend-pinned node-plugins until backend confirms recovery | varies |
| 5 | Verify post-recovery: queued PVCs eventually bind | ≤ 1h after recovery |
| 6 | Audit: any data-integrity concerns post-outage? Backend-side checksum verification | varies |

## VolumeSnapshot recovery (data restore from CSI snapshot)

| Step | Action | Time |
|---|---|---|
| 1 | Identify VolumeSnapshot: `kubectl -n <ns> get volumesnapshot` | ≤ 2 min |
| 2 | Create PVC referencing snapshot's source: `dataSource: kind: VolumeSnapshot, name: <snap>` | ≤ 2 min |
| 3 | Wait for binding: `kubectl -n <ns> get pvc <name> -w` | ≤ 15 min (depends on backend snapshot-to-volume restore time) |
| 4 | Mount in pod; verify data integrity (workload µservice's responsibility) | – |

## Verification

- `kubectl -n kube-system get pods -l 'app in (csi-block-controller,csi-object-controller,csi-file-controller)'` all Ready
- `kubectl get storageclass`: all backends listed
- Test: create PVC of each class; verify bind ≤ 30s p99
- `csi_controller_publish_volume_errors_total` rate at baseline

## References

- `microservices/cloud-k8s/failure-modes.md` FM-10, FM-15.
- `microservices/cloud-k8s/multi-region.md` (DR failover for backend outage).
- Kubernetes CSI — `kubernetes-csi.github.io/docs/`.
- OCI Block Volume CSI — `docs.oracle.com/en-us/iaas/Content/ContEng/Tasks/contengcreatingpersistentvolumeclaim.htm`.
