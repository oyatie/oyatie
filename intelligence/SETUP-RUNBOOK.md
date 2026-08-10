# SETUP-RUNBOOK — cloud-intelligence live deployment

This runbook lists **exactly what an operator must do** to take the cloud-intelligence from "deployed manifests in dev" to "actively serving parallel agent traffic." Every step that needs human credentials/authority is flagged **[HUMAN-AUTH]**.

Prereqs: a Talos cluster with ArgoCD + ESO + owned cloud-secrets/cloud-kms adapters + the in-cluster registry up (per the bring-up README).

## 1. Register cloud-secrets / cloud-kms handles  **[HUMAN-AUTH]**

The gateway consumes handles and realm tokens only. Register these values in the
owned cloud-secrets/cloud-kms substrate; any concrete backing store is a
transient adapter and is not a cloud-intelligence contract:

- `secret_provider_token` — short-lived token for the secret-provider adapter.
- `admin_bearer_token` — operator token for admin endpoints.
- `initial_seats` / `tenant_provider_pools` — opaque `secret-ref://` or
  `kms-ref://` handles; never raw provider credentials.
- `ingress_proxy_keys` — keys agents present to the ingress realm.

Then verify the ExternalSecret syncs:

```sh
kubectl -n oya-cloud-intelligence get externalsecret cloud-intelligence-secrets -o yaml | grep -E "status|lastSyncedAt"
kubectl -n oya-cloud-intelligence get secret cloud-intelligence-secrets   # exists after sync
```

## 2. Build the image  **[HUMAN-AUTH]**

```sh
kubectl apply -f intelligence/k8s/buildkit-build.yaml
kubectl -n oya-ci wait --for=condition=complete job/cloud-intelligence-image-build --timeout=15m
kubectl -n oya-registry exec deploy/registry -- ls /var/lib/registry/docker/registry/v2/repositories/cloud-intelligence/_manifests/tags
```

## 3. Sync ArgoCD

```sh
argocd app sync cloud-intelligence
kubectl -n oya-cloud-intelligence wait deploy/cloud-intelligence --for=condition=available --timeout=5m
kubectl -n oya-cloud-intelligence get pods   # 3 running pods
```

## 4. Verify

```sh
kubectl -n oya-cloud-intelligence port-forward svc/cloud-intelligence 8080:8080 &
curl -sS -H "Authorization: Bearer <one of INGRESS_PROXY_KEYS>" \
  http://localhost:8080/healthz
# expect: {"status":"ok"}
```

## 5. Enable agent namespaces

For each namespace whose pods should reach the gateway:

```sh
kubectl label namespace <my-agent-ns> oya.gateway-client=true
```

Without this label, Cilium L3/L4 denies the connection.

## 6. Wire agents

Per `README.md` "Live fanout" section: set `ANTHROPIC_BASE_URL` / `OPENAI_BASE_URL` / `GEMINI_BASE_URL` + the corresponding `*_API_KEY` env in each agent's launch environment.

## Rotation

90-day rotation: re-run step 1 with new values; ESO refreshes within 5 min; Deployment pods pick up via env (restart pods to force-refresh: `kubectl -n oya-cloud-intelligence rollout restart deploy/cloud-intelligence`).

---

## Production deploy on Talos

This section covers deploying `cloud-intelligence` (the canonical production name of the
gateway binary) on the founder's Talos cluster via ArgoCD GitOps.

### Helm chart

The chart lives at `microservices/cloud-intelligence/iac/k8s/helm/`. It packages:

- `Chart.yaml` — chart named `cloud-intelligence`, version `0.1.0`.
- `values.yaml` — image pinned by digest, `kata-cloud-hypervisor` runtime class (Tier-2
  isolation), service account `oya-cloud-intelligence`, ClusterIP on port 8080.
- `templates/deployment.yaml` — injects `OYA_CLOUD_INTEL_LISTEN_ADDR`,
  `OYA_CLOUD_INTEL_TENANT_ID`, and `OYA_CLOUD_INTEL_SECRET_PROVIDER_TOKEN`
  (from the ESO-managed secret).
- `templates/externalsecret.yaml` — ESO `SecretStore` + `ExternalSecret`
  projecting owned cloud-secrets/cloud-kms handles.
- `templates/networkpolicy.yaml` — cell-boundary policy: inbound from `istio-ingress`
  only; egress to the secret-provider adapter (`cloud-secrets:8200`), upstream AI
  providers (`:443`), DNS, and the OTel collector.
- `templates/httproute.yaml` — Gateway API `HTTPRoute` matching `/anthropic/*`,
  `/openai/*`, `/gemini/*` to the `cloud-intelligence` Service.

To lint and render locally:

```sh
helm lint microservices/cloud-intelligence/iac/k8s/helm

helm template microservices/cloud-intelligence/iac/k8s/helm \
  --set image.digest= \
  > /tmp/helm-rendered.yaml
```

### ArgoCD ApplicationSet

`microservices/cloud-iac/iac/oyatie-cloud-provider/argocd/apps/cloud-intelligence-applicationset.yaml`
generates one ArgoCD Application per environment (dev / staging / prod), syncing the
Helm chart with environment-specific value overrides. Sync policy: automated + selfHeal
+ prune. Namespace `cloud-intelligence` is created and labelled automatically.

Apply (once the cluster is reachable):

```sh
kubectl apply -f \
  microservices/cloud-iac/iac/oyatie-cloud-provider/argocd/apps/cloud-intelligence-applicationset.yaml

argocd app list | grep cloud-intelligence
# expect: cloud-intelligence-dev   Synced  Healthy
```

### Enroll the first subscription

See `intelligence/runbooks/enroll-first-subscription.md` for the
step-by-step OAuth enrollment flow, secret-provider handle registration, ESO sync
verification, and smoke-test commands.

### What still requires founder action before first traffic

1. **Publish the container image** — build `bin/cloud-intelligence` and push a
   cosign-signed image to `registry.oyatie.dev/oya-cloud-intelligence:0.1.0` with the
   production digest. Update `image.digest` in `values.yaml` (or via the ApplicationSet
   parameter) to the real digest.
2. **Secret-provider adapter access** — bind the `cloud-intelligence-service-role`
   to the owned cloud-secrets/cloud-kms policy that grants read access to
   cloud-intelligence launch handles only.
3. **Istio Gateway** — confirm `oyatie-ingress-gateway` in namespace `istio-ingress`
   exists and has a TLS listener on `cloud-intelligence.oyatie.com`.
4. **OAuth callback registration** — register
   `https://cloud-intelligence.oyatie.com/anthropic/oauth/callback` as an allowed redirect URI
   with Anthropic if using Authorization Code flow.
5. **Cosign signature** — sign the published image with the project cosign key so the
   Kyverno `verify-image-signatures` policy passes on admission.
