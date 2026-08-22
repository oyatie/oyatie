---
doc_class: Runbook
title: Control-plane restore (kube-apiserver / kube-controller-manager / kube-scheduler / api-proxy)
microservice: cloud-k8s
severity: "Sev-1 (control-plane outage is always Sev-1)"
status: Accepted
owner_team: ops-sre-reliability + axis-cloud
date: 2026-05-17
related_artifacts:
  - microservices/cloud-k8s/failure-modes.md (FM-01, FM-03, FM-14)
  - microservices/cloud-k8s/multi-region.md
  - microservices/cloud-k8s/capacity-model.md
doc_status: published
---

# Runbook: Control-plane restore

## Trigger

Any of:
- kube-apiserver outage (FM-01)
- etcd at-rest encryption key rotation failure (FM-03)
- kubernetes-api-proxy outage (FM-14)
- Control-plane node-network partition (FM-04)

## Severity

**Sev-1 always** per ADR-0121: control-plane unavailability blocks every workload µservice's cluster mutation. M01 single-control-plane = full freeze; subsequent-to-M04-completion HA = degraded but writable through surviving replica.

## Pre-flight verification

```bash
# Verify pack + cluster identity
kubectl --context <pack>-cluster cluster-info
kubectl --context <pack>-cluster get nodes -l node-role.kubernetes.io/control-plane=
```

## kube-apiserver pod outage (FM-01)

| Step | Action | Time |
|---|---|---|
| 1 | Identify failed replica: `kubectl -n kube-system get pods -l component=kube-apiserver` | ≤ 2 min |
| 2 | Capture logs pre-restart: `kubectl -n kube-system logs <pod> --previous > /tmp/kube-apiserver-fail.log` | ≤ 2 min |
| 3 | Verify cause: OOM (memory usage spiked)? Liveness probe failure (etcd unreachable)? Cert expired? | ≤ 5 min |
| 4a (OOM) | Increase memory request via Helm values + apply | ≤ 5 min |
| 4b (etcd) | Check etcd health: `etcdctl --endpoints=<etcd-endpoints> endpoint health` | ≤ 5 min |
| 4c (cert) | Renew via `kubeadm certs renew <component>` + restart pod | ≤ 5 min |
| 5 | Verify recovery: `kubectl get --raw /readyz` returns 200 | ≤ 5 min |
| 6 | Validate api-proxy is forwarding cleanly | ≤ 5 min |
| 7 | If M01 single-CP and recovery fails: initiate cluster-restore from etcd snapshot (below) | ≤ 30 min |

## kubernetes-api-proxy outage (FM-14)

| Step | Action | Time |
|---|---|---|
| 1 | Identify failed replicas: `kubectl -n cloud-k8s-system get pods -l app=kubernetes-api-proxy` | ≤ 2 min |
| 2 | Verify upstream kube-apiserver reachable (api-proxy logs show 5xx from upstream?) | ≤ 2 min |
| 3 | Verify Cedar evaluator: `kubectl -n cloud-k8s-system logs <api-proxy-pod> | grep cedar_eval_error` | ≤ 5 min |
| 4 | Verify OpenBao token-renewal: api-proxy ServiceAccount token still valid? | ≤ 5 min |
| 5 | Restart failed replicas: `kubectl -n cloud-k8s-system rollout restart deployment kubernetes-api-proxy` | ≤ 5 min |
| 6 | Verify HA: at least 3/3 replicas Ready | ≤ 5 min |
| 7 | Verify recovery via probe: `curl https://k8s-api-<pack>.oyatie.dev/health` returns 200 | ≤ 2 min |

## Encryption key rotation rollback (FM-03)

| Step | Action | Time |
|---|---|---|
| 1 | Engage ops-security; declare Sev-1 + `#inc-sec-<id>` | immediate |
| 2 | Pause rotation: `kubeadm-rotate-config pause` (custom CLI from `cluster-bootstrap-app`) | ≤ 2 min |
| 3 | Verify both keys present in `--encryption-provider-config`: old key (for reads) + new key (for writes) | ≤ 5 min |
| 4 | If new key write succeeded but old key removed: cluster cannot read recent data. Restore from snapshot (below). Engage council-privacy. | – |
| 5 | If both keys present: re-attempt rotation when KMS recovers | ≤ 30 min |
| 6 | Audit: kubectl create encryption-test secret + read + verify both encryption paths work | ≤ 10 min |

## Cluster-restore from etcd snapshot

When the control plane cannot recover in-place. Cross-references `runbooks/etcd-quorum-recovery.md`.

| Step | Action | Time |
|---|---|---|
| 1 | Identify most-recent valid signed snapshot: list `snap-<ts>.db` in per-pack object storage | ≤ 5 min |
| 2 | Verify Ed25519 signature on snapshot | ≤ 2 min |
| 3 | On a fresh control-plane node (or surviving M01 node): `kubeadm reset` (preserve etcd backup directory) | ≤ 5 min |
| 4 | Place snapshot at `/var/lib/etcd-backup/snap.db` | ≤ 1 min |
| 5 | Initialize etcd from snapshot: `etcdctl snapshot restore /var/lib/etcd-backup/snap.db --data-dir /var/lib/etcd` | ≤ 5 min |
| 6 | Bring up new control-plane: `kubeadm init --config /etc/kubernetes/kubeadm.config --upload-certs` (with etcd-pre-restored flag) | ≤ 15 min |
| 7 | Verify all kube-system pods Ready | ≤ 10 min |
| 8 | Re-join worker nodes if they were drained | ≤ 5 min per worker |
| 9 | Smoke test: schedule a test pod; verify it reaches Ready | ≤ 5 min |

Total cluster-restore RTO: ≤ 30 min from snapshot decision to operational cluster.

## API proxy recovery (FM-14)

| Step | Action | Time |
|---|---|---|
| 1 | Verify HPA: `kubectl -n cloud-k8s-system get hpa kubernetes-api-proxy` shows desired replicas | ≤ 2 min |
| 2 | If desired > current: pod-eviction-storm. Cordon affected nodes; wait for re-schedule. | ≤ 10 min |
| 3 | If desired = current but unhealthy: investigate per `kubectl -n cloud-k8s-system describe pod <api-proxy-pod>` | ≤ 5 min |
| 4 | Cedar fragment failure? Verify `/policy/*.cedar` integrity via SHA: `sha256sum policy/*.cedar | diff -q git-recorded-hashes.txt -` | ≤ 5 min |
| 5 | Restart via `rollout restart`; verify all replicas Ready | ≤ 5 min |

## Verification (after recovery)

- `kubectl get componentstatuses` (deprecated but still informative)
- `kubectl get --raw /healthz`, `/readyz`, `/livez` all return 200
- `kubectl get nodes` all Ready
- `kubectl get pods --all-namespaces` no CrashLoopBackOff in kube-system / cloud-k8s-system
- Test: create + delete a namespace via `kubernetes-api-proxy` (verifies end-to-end path)
- `kubernetes_api_proxy_request_duration_seconds{quantile="0.99"} < 100ms` for ≥ 30 min
- audit-chain: verify post-restore events sealed (audit-chain integrity check)

## Post-incident updates

- Postmortem ≤ 5 business days.
- If FM-01 (pod outage): adjust memory request / liveness threshold per `capacity-model.md`.
- If FM-03 (encryption key): root-cause KMS outage; harden retry logic in `cluster-bootstrap-worker`.
- If FM-14 (api-proxy): check Cedar evaluator perf; consider in-process variant per M04 plan.

## References

- `microservices/cloud-k8s/failure-modes.md`.
- `microservices/cloud-k8s/multi-region.md` §"DR Failover" (if region-level escalation needed).
- `microservices/cloud-k8s/capacity-model.md`.
- `microservices/cloud-k8s/incident-response.md`.
- Kubernetes high availability — `kubernetes.io/docs/tasks/administer-cluster/highly-available-control-plane/`.
- etcd recovery — `etcd.io/docs/v3.5/op-guide/recovery/`.
- kubeadm troubleshooting — `kubernetes.io/docs/setup/production-environment/tools/kubeadm/troubleshooting-kubeadm/`.
