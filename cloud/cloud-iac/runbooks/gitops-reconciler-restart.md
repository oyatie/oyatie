---
doc_class: Runbook
title: GitOps reconciler restart (ArgoCD / Flux)
microservice: cloud-iac
severity: "Sev-2 (degraded) / Sev-1 (persistent > 1h)"
status: Accepted
owner_team: ops-sre-reliability + axis-cloud-iac
date: 2026-05-17
related_artifacts:
  - microservices/cloud-iac/failure-modes.md (FM-05, FM-14)
  - microservices/cloud-iac/incident-response.md
doc_status: published
---

# Runbook: GitOps reconciler restart (ArgoCD / Flux)

## Trigger

ONE of:
1. **ArgoCD outage** (FM-05): application-controller crashloop; etcd unavailable; reconciler pod-eviction storm.
2. **Admission webhook outage** (FM-14): ArgoCD admission webhook unhealthy; new Application resources rejected.
3. **Flux outage**: source-controller / kustomize-controller / helm-controller unhealthy.

## Severity

- Transient (auto-resolves < 15min): Sev-3.
- Persistent ≥ 15min OR HA replicas all down: Sev-2.
- Persistent > 1h: Sev-1.

## Pre-checks

1. Verify ArgoCD pods: `kubectl -n argocd get pods`.
2. Check ArgoCD UI health: `curl -s https://argocd-<pack>.oyatie.dev/healthz`.
3. Check Valkey Sentinel cluster (counterpart-fact: Argo CD labels the backing component `app=argocd-redis`): `kubectl -n argocd get pods -l app=argocd-redis`.
4. Check Flux pods (if active): `kubectl -n flux-system get pods`.
5. Check admission webhook: `kubectl get validatingwebhookconfigurations argocd-applicationset-controller-webhook`.

## Recovery Path A — ArgoCD application-controller crashloop

| Step | Action |
|---|---|
| 1 | Inspect logs: `kubectl -n argocd logs <app-controller-pod> --previous` |
| 2 | Common causes: Valkey Sentinel split-brain; OOM; etcd contention |
| 3 | If Valkey split-brain: `kubectl -n argocd delete pod -l app=argocd-redis` (counterpart-fact: Argo CD label retained; sentinels re-elect) |
| 4 | If OOM: increase pod memory limits; vertical-scale |
| 5 | Restart app-controller: `kubectl -n argocd rollout restart deployment/argocd-application-controller` |
| 6 | Verify recovery: `argocd_app_info` cardinality returns to baseline |

## Recovery Path B — ArgoCD admission webhook outage (FM-14)

| Step | Action |
|---|---|
| 1 | Inspect webhook pod: `kubectl -n argocd get pods -l app=argocd-applicationset-controller` |
| 2 | If pod is crashloop: check cert validity (`kubectl get secret -n argocd argocd-applicationset-controller-webhook-cert -o yaml` then verify `tls.crt` expiration) |
| 3 | If cert expired: trigger cert-manager renewal: `kubectl annotate certificate -n argocd argocd-applicationset-controller-webhook cert-manager.io/issue-temporary-certificate=true --overwrite` |
| 4 | Restart webhook pod |
| 5 | If persistent and blocking deploys: temporarily set `failurePolicy: Ignore` (with audit-chain seal). REQUIRES ops-security approval |
| 6 | Verify new Application resources accepted: `argocd app create test-app --dry-run` returns success |

## Recovery Path C — Flux reconciler outage

| Step | Action |
|---|---|
| 1 | Inspect Flux pods: `kubectl -n flux-system get pods`; identify unhealthy controllers |
| 2 | Common causes: source-controller cannot reach git; kustomize-controller OOM; helm-controller fetch failure |
| 3 | If git unreachable: verify network policy + DNS resolution |
| 4 | Restart unhealthy controllers: `kubectl -n flux-system rollout restart deployment/<controller>` |
| 5 | Verify recovery: `flux get reconcilers --all-namespaces` shows all green |

## Recovery Path D — DR failover (Sev-1; persistent > 1h)

| Step | Action |
|---|---|
| 1 | Declare Sev-1; engage axis-cloud-iac + ops-sre-reliability + ExecSponsor |
| 2 | If pack has DR pair: initiate failover per `multi-region.md` §"DR Failover" |
| 3 | If pack is single-region (pack-kr / pack-jp / pack-sg): graceful degradation; manual apply via CLI; engage OCI on region recovery |
| 4 | Notify tenants per `incident-response.md` template |

## Verification

After recovery:
- ArgoCD UI returns 200; `argocd_app_info` cardinality matches expected (one entry per Application).
- New Application resources are accepted (test-app dry-run succeeds).
- Existing Applications continue to reconcile within reconcile_interval (default 180s).
- Flux controllers (if active) all green.
- Self-SLO returns to green: `https://grafana-<pack>.oyatie.dev/d/cloud-iac-reconciler/health`.

## Post-incident updates

- Postmortem within 5 business days (Sev-2+).
- If Valkey split-brain: review sentinel topology + capacity; consider increasing sentinel quorum.
- If cert expiration: harden cert-manager renewal schedule; alert at 14d / 7d / 1d before expiry.
- If etcd contention: cloud-k8s µservice action item.

## References

- `microservices/cloud-iac/failure-modes.md` FM-05 + FM-14.
- `microservices/cloud-iac/incident-response.md`.
- `microservices/cloud-iac/multi-region.md` §"DR Failover".
- ArgoCD HA + admission webhook — `argo-cd.readthedocs.io/en/stable/operator-manual/high_availability/`.
- Flux operations — `fluxcd.io/docs/operations/`.
- cert-manager renewal — `cert-manager.io/docs/usage/certificate/`.
