---
doc_class: Runbook
title: etcd quorum recovery
microservice: cloud-k8s
severity: "Sev-1 (etcd quorum loss is always Sev-1)"
status: Accepted
owner_team: ops-sre-reliability + axis-cloud + ops-security
date: 2026-05-17
related_artifacts:
  - k8s/failure-modes.md (FM-02)
  - k8s/runbooks/control-plane-restore.md
doc_status: published
---

# Runbook: etcd quorum recovery

## Trigger

- etcd `has_leader == 0` for ≥ 1 min OR
- 2/3 etcd members down (subsequent-to-M04-completion HA) OR
- Single etcd corrupted at M01

## Severity

Sev-1 — cluster mutations frozen; reads degraded.

## Recovery — HA (subsequent-to-M04-completion)

| Step | Action | Time |
|---|---|---|
| 1 | Verify failure scope: `etcdctl --endpoints=<all> endpoint status --cluster` | ≤ 2 min |
| 2 | Determine cause: network partition (most common) vs hardware failure | ≤ 5 min |
| 3a (network) | Wait up to 5 min for partition to heal; etcd auto-reconverges | ≤ 5 min |
| 3b (permanent) | Identify the surviving member; verify it has the freshest data: `etcdctl endpoint hashkv --cluster` | ≤ 5 min |
| 4 | Restore lost members: provision new control-plane nodes; `etcdctl member add <new-name> --peer-urls=https://<new-ip>:2380` | ≤ 15 min |
| 5 | Re-bootstrap lost etcd peers from surviving member's snapshot | ≤ 10 min |
| 6 | Verify quorum restored: `etcdctl endpoint health --cluster` all healthy | ≤ 5 min |
| 7 | Smoke test: write + read via kube-apiserver | ≤ 5 min |

Total RTO: ≤ 30 min (HA case).

## Recovery — M01 single etcd

| Step | Action | Time |
|---|---|---|
| 1 | Identify most-recent signed snapshot in per-pack object storage | ≤ 5 min |
| 2 | Verify Ed25519 signature; copy to control-plane node | ≤ 5 min |
| 3 | Stop kube-apiserver (it depends on etcd; will crashloop until restored): `kubectl -n kube-system delete pod kube-apiserver-<cp-node>` (or systemctl stop kubelet temporarily) | ≤ 2 min |
| 4 | Restore: `etcdctl snapshot restore /tmp/snap.db --data-dir /var/lib/etcd-restore` | ≤ 5 min |
| 5 | Replace `/var/lib/etcd` with restored data-dir; restart kubelet | ≤ 5 min |
| 6 | Verify etcd healthy: `etcdctl --endpoints=https://127.0.0.1:2379 endpoint status` | ≤ 2 min |
| 7 | kube-apiserver auto-restarts via kubelet; verify Ready | ≤ 5 min |
| 8 | Smoke test: write + read | ≤ 5 min |

Total RTO: ≤ 30 min.

## Data-loss assessment (RPO)

Snapshot cadence is 5 min; data loss is bounded by time-since-last-snapshot. If incident occurred within the 5-min snapshot window: last 5 min of cluster mutations are lost (pods scheduled, NetworkPolicy applied, etc.). Workload state itself is in pod / PV / external storage — only cluster-mutation history is lost.

## Audit-chain reconciliation

After restore: replay audit-chain records emitted between snapshot-time and restore-time. Mark replayed records with `replayed=true` label. Council-privacy review if any tenant-impactful mutation was rolled back.

## Verification

- `etcdctl endpoint health --cluster` all healthy
- `etcdctl endpoint status --cluster --write-out=table` shows consistent revisions
- kube-apiserver writes succeed end-to-end
- audit-chain integrity restored

## References

- `k8s/failure-modes.md` FM-02.
- `k8s/runbooks/control-plane-restore.md`.
- etcd disaster recovery — `etcd.io/docs/v3.5/op-guide/recovery/`.
