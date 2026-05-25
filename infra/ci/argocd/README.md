# ArgoCD — CD half of the local CI/CD substrate

Realizes the CD side of `specs/ci-farm-substrate-canonical.json` / ADR-0349:
ArgoCD drives GitOps deploys and (with Argo Rollouts) progressive delivery
— canary → bake-time SLO observation → metric-gated rollback.

## Install (local k3s)

```bash
kubectl create namespace argocd
kubectl apply -n argocd -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml
kubectl -n argocd rollout status deploy/argocd-server
```

Access:
```bash
kubectl -n argocd port-forward svc/argocd-server 8081:443
# user: admin   password:
kubectl -n argocd get secret argocd-initial-admin-secret -o jsonpath='{.data.password}' | base64 -d; echo
```

## Local-vs-production deltas

| Element | Production | Local profile |
|---|---|---|
| progressive delivery | Argo Rollouts canary + automated metric-gated rollback (ADR-0349) | Argo Rollouts installed; canary + AnalysisTemplate wired (`rollouts-demo.yaml`); metric provider is a job placeholder (prod: Prometheus burn-rate) |
| image trust | cosign-verified images (ADR-0181) admission-gated | not enforced locally |
| HA | redundant controllers/repo-servers | single replicas (upstream `install.yaml`) |
| exposure | ingress + SSO (dex/OIDC) | `port-forward` + initial admin secret |

## Honest status

- **Claimed when green:** ArgoCD control plane runs on local k8s (server +
  application/applicationset controllers + repo-server + redis + dex).
- **Demonstrated:** Argo Rollouts canary stepped `25% → analysis gate → 50% →
  75% → promote` on an image bump, with an `AnalysisRun` gating promotion
  (`rollouts-demo.yaml`; evidence in `evidence/ci/`).
- **NOT claimed:** no ArgoCD git `Application` is synced (needs an in-cluster git
  remote — same seam as the build lane), the analysis metric is a job placeholder
  (not a real Prometheus SLO query), and no metric-triggered rollback or deploy
  SLO is measured. Per spec `non_claims`, those stay unproven pending wiring.
