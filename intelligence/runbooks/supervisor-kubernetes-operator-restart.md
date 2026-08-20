---
doc_class: Runbook
title: Kubernetes Operator restart (controller crashloop / Postgres / Valkey / admission webhook / signing key)
microservice: foundry-supervisor
severity: "Sev-2 (HA failover) / Sev-1 (all replicas crashlooping > 10 min)"
status: Accepted
owner_team: axis-foundry-control-plane + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-04, FM-05, FM-06, FM-10, FM-14)
doc_status: published
---

# Runbook: Kubernetes Operator restart

Covers operator crashloop (FM-06), Postgres master loss (FM-05), Valkey failover (FM-04), admission webhook outage (FM-10), signing-key rotation failure (FM-14).

## Operator crashloop (FM-06)

### Trigger

`kubernetes_operator_alive == 0` for ≥ 2 min, OR `controller_runtime_reconcile_errors_total` rate climbs.

### Steps

| Step | Action | Time |
|---|---|---|
| 1 | Verify lease-leadership election re-ran: `kubectl get lease -n foundry-supervisor` | ≤ 1 min |
| 2 | Verify standby controller took over: `kubectl logs -n foundry-supervisor -l app=foundry-supervisor-operator -c controller --tail=50` | ≤ 2 min |
| 3 | If all replicas crashlooping (Sev-1): identify offending CRD parse error or OpenBao token-renewal failure; rollback last Helm release | ≤ 10 min |
| 4 | Verify reconcile-rate returns to baseline | ≤ 5 min |
| 5 | Postmortem within 5 business days | – |

## Postgres master loss (FM-05)

### Trigger

Patroni `postgres_master_unreachable`, OR PgBouncer connection failures spike.

### Steps

| Step | Action | Time |
|---|---|---|
| 1 | Verify Patroni promoted replica: `patronictl list -c /etc/patroni.yml` | ≤ 30 s |
| 2 | Verify PgBouncer rerouted (connection failures clear) | ≤ 1 min |
| 3 | Audit promotion: `patronictl history` | ≤ 2 min |
| 4 | Forensic: master loss cause (OOM / AZ failure / kernel panic / compromise) | varies |
| 5 | If compromise-suspected: engage ops-security + escalate to Sev-1 | – |
| 6 | New replica provisioned to replace promoted master: `patronictl reinit <old-master>` | ≤ 30 min |

## Valkey failover (FM-04)

### Trigger

`redis_cluster_replica_promoted_total > 0`, OR `oya_supervisor_kill_switch_engage_latency_p99` brief spike.

### Steps

| Step | Action |
|---|---|
| 1 | Verify cluster mode is functioning: `redis-cli --cluster check <one-of-the-nodes>:6379` |
| 2 | If cluster degraded (multiple replicas down): scale shards: `kubectl scale statefulset redis-cluster --replicas=<N>` |
| 3 | Wait for AOF replay + cluster heal |
| 4 | Verify kill-switch state propagation back within SLO (`oya_supervisor_kill_switch_engage_latency_p99 <= 1 s`) |
| 5 | If divergence with CRDs: trigger CRD reconcile (see `fleet-state-recovery.md`) |

## Admission webhook outage (FM-10)

### Trigger

`kubernetes_admission_webhook_unreachable`, OR new CRDs being rejected.

### Steps

| Step | Action |
|---|---|
| 1 | Verify webhook pods healthy: `kubectl get pods -n foundry-supervisor -l app=admission-webhook` |
| 2 | Verify cert validity: `kubectl describe certificate -n foundry-supervisor admission-webhook-cert` |
| 3 | If cert expired: trigger cert-manager renewal: `kubectl annotate certificate admission-webhook-cert cert-manager.io/force-renewal=true` |
| 4 | If webhook pod crashloop: rollback last Helm release affecting webhook |
| 5 | Verify CRD admissions succeed |

## Signing-key rotation failure (FM-14)

### Trigger

`oya_supervisor_signing_key_age_days > 90`, OR `oya_supervisor_event_signature_invalid_total > 0`.

### Steps

| Step | Action |
|---|---|
| 1 | Force-reload signing key: `cargo run -p oya-dev-cli -- supervisor reload-signing-key --source openbao` |
| 2 | Verify new key materialized: `kubectl exec -n foundry-supervisor supervisor-rest-0 -- /usr/local/bin/foundry-supervisor signing-key info` |
| 3 | Test signature: emit a synthetic event; verify audit-chain validates |
| 4 | If old key compromise-suspected: engage ops-security; rotate immediately; revoke old key in OpenBao |
| 5 | Audit-chain seal records the rotation event |

## General verification (any restart)

- Lease-leadership stable (one active controller).
- Reconcile lag back within p99 ≤ 1 s.
- REST + Worker availability back within p99 ≤ 200 ms.
- Two-channel corroboration (Mimir + OnCall) working.
- Per-changeset evidence updated.

## References

- `failure-modes.md` FM-04, FM-05, FM-06, FM-10, FM-14.
- Patroni — `patroni.readthedocs.io`.
- Valkey Cluster operations — `redis.io/docs/management/scaling/`.
- cert-manager — `cert-manager.io/docs/`.
- OpenBao — `openbao.org`.
- Kubernetes Operator pattern — `kubernetes.io/docs/concepts/extend-kubernetes/operator/`.
