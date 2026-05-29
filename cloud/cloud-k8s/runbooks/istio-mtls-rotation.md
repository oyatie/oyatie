---
doc_class: Runbook
title: Istio mTLS rotation + istiod recovery
microservice: cloud-k8s
severity: "Sev-2 (data plane survives; new config blocked)"
status: Accepted
owner_team: ops-sre-reliability + axis-cloud + ops-security
date: 2026-05-17
related_artifacts:
  - microservices/cloud-k8s/failure-modes.md (FM-07)
  - microservices/cloud-k8s/policy/cluster-isolation.md
doc_status: published
---

# Runbook: Istio mTLS rotation + istiod recovery

## Trigger

- Istio root CA rotation (annual cadence)
- Istio intermediate CA rotation (per pack)
- istiod outage (FM-07)
- Canary control-plane upgrade rollback

## Severity

Sev-2 (data-plane survives; new config blocked). Sev-1 if root CA rotation fails and certs expire.

## Root CA rotation (planned; annual)

| Step | Action | Time |
|---|---|---|
| 1 | Pre-rotation: verify all sidecars have ≥ 30d on existing root CA cert | ≤ 5 min |
| 2 | Generate new root CA in OpenBao (HSM-backed where available); sign new intermediate | ≤ 15 min |
| 3 | Update Istio `cacerts` Secret in istio-system with NEW + OLD root CA concatenated | ≤ 5 min |
| 4 | Restart istiod: `kubectl -n istio-system rollout restart deployment istiod` | ≤ 5 min |
| 5 | Sidecars learn both root CAs via xDS; existing mTLS connections continue working | ≤ 5 min |
| 6 | Wait 24h for sidecars to refresh workload certs (default cert lifetime) | 24h |
| 7 | Remove OLD root CA from cacerts; restart istiod again | ≤ 5 min |
| 8 | Verify all sidecars now using new root CA: `istioctl x check-inject` + `istioctl proxy-status` | ≤ 10 min |
| 9 | Audit-chain emit `IstioPolicyChanged` event (CA rotation is a control-plane change) | (automatic) |

Total: ≤ 24h rotation cycle.

## istiod outage recovery (FM-07)

| Step | Action | Time |
|---|---|---|
| 1 | Identify failed pods: `kubectl -n istio-system get pods -l app=istiod` | ≤ 2 min |
| 2 | Verify data plane unaffected: pick a sample pod; `kubectl exec <pod> -c istio-proxy -- curl http://localhost:15000/clusters` returns last-known config | ≤ 5 min |
| 3 | Capture logs pre-restart: `kubectl -n istio-system logs <failed-pod> --previous > /tmp/istiod-fail.log` | ≤ 2 min |
| 4 | Restart deployment: `kubectl -n istio-system rollout restart deployment istiod` | ≤ 5 min |
| 5 | Verify HA: ≥ 3/3 replicas Ready | ≤ 5 min |
| 6 | Verify xDS publish resuming: `istio_pilot_xds_pushes_total` rate ≥ baseline | ≤ 5 min |
| 7 | If persistent failure: rollback to prior Istio revision (canary pattern) — see below | ≤ 30 min |

## Canary control-plane rollback

| Step | Action | Time |
|---|---|---|
| 1 | Identify current + prior revisions: `istioctl x revision list` | ≤ 1 min |
| 2 | Validate prior revision still healthy: `istioctl x precheck --revision <prior>` | ≤ 5 min |
| 3 | Tag prior revision as default: `kubectl label namespace <tenant-ns> istio.io/rev=<prior>` (for affected namespaces) | ≤ 5 min |
| 4 | Restart workload pods to pick up prior-revision sidecars: `kubectl -n <ns> rollout restart deployment <name>` | ≤ 10 min |
| 5 | Verify mesh health: `istioctl analyze` clean | ≤ 5 min |
| 6 | Decommission failed revision: `istioctl uninstall --revision <failed>` | ≤ 5 min |

## Verification

- `istioctl proxy-status`: all proxies SYNCED across CDS / LDS / RDS / EDS
- `istioctl analyze`: no warnings
- mTLS strict still enforced: `istioctl x authz check <pod> <other-pod>` shows mTLS
- Sample mTLS handshake from a pod to a cross-namespace service succeeds with mutual cert validation

## References

- `microservices/cloud-k8s/failure-modes.md` FM-07.
- Istio operations — `istio.io/latest/docs/ops/`.
- Istio CA rotation — `istio.io/latest/docs/tasks/security/cert-management/plugin-ca-cert/`.
- Istio canary upgrade — `istio.io/latest/docs/setup/upgrade/canary/`.
