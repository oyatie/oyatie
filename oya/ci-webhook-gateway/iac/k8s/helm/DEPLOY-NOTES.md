# ci-webhook-gateway — Deploy Notes

This PR authors the ArgoCD ApplicationSet + Helm chart manifests that deploy the
ci-webhook-gateway microservice (ADR-0374, ADR-0387). The chart is static — no
live-cluster mutation is performed by this PR.

## What is deployed (when a founder syncs)

- **Namespace**: `oya-ci` (dev), `oya-ci-staging` (staging), `oya-ci-prod` (prod)
- **Helm chart**: `microservices/ci-webhook-gateway/iac/k8s/helm`
- **ArgoCD ApplicationSet**: `microservices/cloud-iac/iac/oyatie-cloud-provider/argocd/apps/ci-webhook-gateway-applicationset.yaml`
- **Resources**: Deployment (1 replica), Service (ClusterIP :8099), ServiceAccount,
  ExternalSecret (ESO → OpenBao), NetworkPolicy, HTTPRoute

## Env vars wired (from `src/config.rs`)

| Env var | Source | Notes |
|---|---|---|
| `OYA_GATEWAY_BIND_ADDR` | `values.yaml` → `"0.0.0.0:8099"` | Matches `DEFAULT_BIND_ADDR` in config.rs |
| `OYA_GATEWAY_TARGET_BRANCH` | hardcoded `"dev"` in deployment.yaml | PRs targeting `dev` are gated |
| `OYA_JENKINS_DISPATCH_URL` | `values.yaml` → `jenkins.dispatchUrl` | Full http:// URL for oyaCiLane kick |
| `OYA_GITHUB_WEBHOOK_SECRET` | ExternalSecret `github_webhook_secret` key | HMAC-SHA256 secret; gateway fails closed if unset |

## FOUNDER-GATED checklist — steps this PR does NOT perform

These steps require live-cluster authority and are blocked on OpenBao init:

1. **OpenBao init** (`bao operator init`) — OpenBao is NOT yet initialized.
   The ExternalSecret cannot resolve until this is done.

2. **Write the HMAC webhook secret to OpenBao**:
   ```
   openssl rand -hex 32   # generate secret
   bao kv put secret/oya/ci/github-webhook-secret value='<SECRET>'
   ```
   This unblocks the ExternalSecret → `ci-webhook-gateway-secrets` → `OYA_GITHUB_WEBHOOK_SECRET`.

3. **Update `jenkins.dispatchUrl`** in `values.yaml` with the live
   generic-webhook-trigger or build-token endpoint once Jenkins is confirmed up.
   Current placeholder: `http://oya-jenkins.oya-ci-jenkins.svc.cluster.local:8080/generic-webhook-trigger/invoke`

4. **Set a real cosign image digest** — the current `image.digest: ""` in `values.yaml`
   (and the `imageDigest: ""` in the ApplicationSet) means the Helm guard
   (`image.cosign.required=true`) will block deployment until a real
   `sha256:<64-hex>` digest is injected by release automation (ADR-0181).

5. **Register the GitHub webhook** (SETUP-RUNBOOK.md §2) — founder action on
   the live GitHub instance (`oya-admin/oyatie` repo → Settings → Webhooks).
   Target URL: `http://ci-webhook-gateway.oya-ci.svc.cluster.local:8099/webhook/github`
   (or the external ingress URL). Use the SAME secret written to OpenBao in step 2.

6. **`argocd app sync ci-webhook-gateway-dev`** (or enable auto-sync) — founder
   action. The ApplicationSet has `automated.prune=true` + `selfHeal=true` but
   ArgoCD itself must be running and the Application must be admitted.

7. **Jenkins `github-ci-token` credential** (SETUP-RUNBOOK.md §5) — Jenkins must
   have the GitHub API token in `oya-ci-jenkins` before it can post commit
   statuses back to GitHub. See `runbooks/provision-secrets.md` for steps.

## No live-cluster mutation in this PR

This PR only authors static YAML manifests. It does not:
- Apply any manifest (`kubectl apply`, `argocd sync`)
- Register any GitHub webhook
- Provision any secret in OpenBao or Kubernetes
- Touch any live cluster resource
