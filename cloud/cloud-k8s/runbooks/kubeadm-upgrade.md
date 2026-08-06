---
doc_class: Runbook
title: kubeadm minor-version upgrade (N → N+1)
microservice: cloud-k8s
severity: "Sev-3 planned / Sev-1 if rollback needed"
status: Accepted
owner_team: axis-cloud + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/cloud-k8s/failure-modes.md (FM-11)
  - microservices/cloud-k8s/policy/cluster-isolation.md
  - docs/decisions/ADR-0709-general-live-apex.md
doc_status: published
---

# Runbook: kubeadm minor-version upgrade

## Trigger

- Upstream Kubernetes minor release (every 4 months per CNCF cadence)
- N-2 support window approaching expiry
- Security patch requiring minor version

## Severity

Sev-3 planned (off-hours, drained workloads). Sev-1 if rollback needed (FM-11).

## Pre-upgrade preparation (T-7 days)

| Step | Action |
|---|---|
| 1 | Review upstream release notes for breaking changes (`kubernetes.io/blog/.../kubernetes-N.NN-release/`) |
| 2 | Validate target version supported by containerd 2.3.0 + Istio 1.29.2 + Cilium 1.16 + Cosign + Kyverno |
| 3 | Update `docs/standards/cloud-k8s-stack.md` with target version pin |
| 4 | Dry-run on DR-pair cluster (if pack has one) or staging cluster |
| 5 | Capture pre-upgrade etcd snapshot; verify integrity |
| 6 | Schedule maintenance window; notify tenants per `incident-response.md` |

## Upgrade procedure (control plane first)

| Step | Action | Time |
|---|---|---|
| 1 | Capture immediate pre-upgrade snapshot: `etcdctl snapshot save /var/lib/etcd/pre-upgrade-N.db` | ≤ 5 min |
| 2 | Drain control-plane node(s) (HA only): `kubectl drain <cp-node> --ignore-daemonsets` | ≤ 10 min |
| 3 | Upgrade kubeadm binary: `apt-mark unhold kubeadm && apt-get install kubeadm=N+1.* -y && apt-mark hold kubeadm` | ≤ 5 min |
| 4 | Plan: `kubeadm upgrade plan` — verify proposed plan matches expectation | ≤ 5 min |
| 5 | Apply: `kubeadm upgrade apply v<N+1>.<minor>.<patch>` | ≤ 15 min |
| 6 | Upgrade kubelet + kubectl: `apt-get install kubelet=N+1.* kubectl=N+1.* -y` | ≤ 5 min |
| 7 | Restart kubelet: `systemctl daemon-reload && systemctl restart kubelet` | ≤ 5 min |
| 8 | Uncordon CP node: `kubectl uncordon <cp-node>` | ≤ 2 min |
| 9 | Verify: `kubectl version` shows new server version; `kubectl get nodes` CP node Ready | ≤ 5 min |
| 10 | Repeat for each HA CP replica (rolling) | ≤ 30 min each |

## Worker node upgrade (rolling, after CP)

For each worker node:

| Step | Action | Time |
|---|---|---|
| 1 | Drain: `kubectl drain <worker> --ignore-daemonsets --delete-emptydir-data` | ≤ 10 min |
| 2 | SSH; upgrade kubeadm binary | ≤ 5 min |
| 3 | `kubeadm upgrade node` | ≤ 10 min |
| 4 | Upgrade kubelet + kubectl; restart kubelet | ≤ 5 min |
| 5 | Uncordon: `kubectl uncordon <worker>` | ≤ 2 min |
| 6 | Wait for sidecar re-injection + workloads ready: `kubectl get pods -o wide --field-selector spec.nodeName=<worker>` | ≤ 10 min |

Total: ≤ 90 min cluster-wide (depends on worker count + PDB).

## Post-upgrade verification

- `kubectl version` shows new server version
- `kubectl get nodes` all Ready + new kubelet version
- `kubectl get componentstatuses` (deprecated but informative)
- Component sanity:
  - `kubectl -n kube-system get pods` — no CrashLoopBackOff
  - `kubectl -n istio-system get pods` — istiod + sidecars Ready
  - `kubectl -n cilium-system get pods` — Cilium agent Ready everywhere
  - `kubectl -n cosign-system get pods` — Kyverno Ready
- API compatibility: deprecated API check via `kubent` (kube-no-trouble); refuse to declare upgrade complete if any tenant workload uses deprecated API in next minor version
- Workload SLO: `oya:current_verdict:by_microservice_env` shows no regression vs pre-upgrade baseline over a 1h window

## Rollback procedure (FM-11)

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-1; engage ops-sre-reliability + axis-cloud | immediate |
| 2 | Stop further worker upgrades immediately | ≤ 1 min |
| 3 | If control-plane already upgraded: `kubeadm upgrade rollback` (custom procedure; restore etcd from pre-upgrade snapshot per `runbooks/etcd-quorum-recovery.md`) | ≤ 30 min |
| 4 | Downgrade kubeadm + kubelet + kubectl on upgraded nodes | ≤ 5 min per node |
| 5 | Re-apply pre-upgrade etcd snapshot to restore control-plane state | ≤ 15 min |
| 6 | Verify cluster operational on N (prior version) | ≤ 15 min |
| 7 | Investigate failure root cause before re-attempt | varies |

Total rollback RTO: ≤ 90 min cluster-wide.

## Audit-chain

Every upgrade emits a `KubeadmUpgraded` event with:
- `from_version`, `to_version`
- `executor` (SPIFFE identity)
- `pre_snapshot_sha`, `post_snapshot_sha`
- Outcome (`success | rolled_back`)
- Ed25519 signature

## References

- `microservices/cloud-k8s/failure-modes.md` FM-11.
- `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`.
- ADR-0121.
- kubeadm upgrade — `kubernetes.io/docs/tasks/administer-cluster/kubeadm/kubeadm-upgrade/`.
- Kubernetes deprecation policy — `kubernetes.io/docs/reference/using-api/deprecation-policy/`.
- kubent — `github.com/doitintl/kube-no-trouble`.
