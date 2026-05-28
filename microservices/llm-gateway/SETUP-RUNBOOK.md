# SETUP-RUNBOOK — llm-gateway live deployment

This runbook lists **exactly what an operator must do** to take the llm-gateway from "deployed manifests in dev" to "actively serving parallel agent traffic." Every step that needs human credentials/authority is flagged **[HUMAN-AUTH]**.

Prereqs: a Talos cluster with ArgoCD + OpenBao + ESO + the in-cluster registry up (per the bring-up README).

## 1. Mint OpenBao secrets  **[HUMAN-AUTH]**

Three secret paths the gateway consumes:

```sh
# (a) the BAO_TOKEN itself — a Vault token with read on oya/llm-gateway/*
bao kv put secret/oya/llm-gateway/bao-token \
  value="<BAO_TOKEN scope-limited to read on secret/oya/llm-gateway/*>"

# (b) ADMIN_TOKEN — operator token for /admin endpoints (rotate, key-add)
bao kv put secret/oya/llm-gateway/admin-token \
  value="$(openssl rand -hex 32)"

# (c) INGRESS_PROXY_KEYS — comma-separated keys agents present as their API key
bao kv put secret/oya/llm-gateway/ingress-proxy-keys \
  value="$(openssl rand -hex 32),$(openssl rand -hex 32)"

# (d) per-provider API keys (one or more per provider)
bao kv put secret/oya/llm-gateway/openai/keys \
  value="sk-...,sk-...,sk-..."
bao kv put secret/oya/llm-gateway/anthropic/keys \
  value="sk-ant-...,sk-ant-..."
bao kv put secret/oya/llm-gateway/gemini/keys \
  value="AIza..."
```

Then verify the ExternalSecret syncs:

```sh
kubectl -n oya-llm-gateway get externalsecret llm-gateway-secrets -o yaml | grep -E "status|lastSyncedAt"
kubectl -n oya-llm-gateway get secret llm-gateway-secrets   # exists after sync
```

## 2. Build the image  **[HUMAN-AUTH]**

```sh
kubectl apply -f microservices/llm-gateway/k8s/buildkit-build.yaml
kubectl -n oya-ci wait --for=condition=complete job/llm-gateway-image-build --timeout=15m
kubectl -n oya-registry exec deploy/registry -- ls /var/lib/registry/docker/registry/v2/repositories/llm-gateway/_manifests/tags
```

## 3. Sync ArgoCD

```sh
argocd app sync llm-gateway
kubectl -n oya-llm-gateway wait deploy/llm-gateway --for=condition=available --timeout=5m
kubectl -n oya-llm-gateway get pods   # 3 running pods
```

## 4. Verify

```sh
kubectl -n oya-llm-gateway port-forward svc/llm-gateway 8080:8080 &
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

90-day rotation: re-run step 1 with new values; ESO refreshes within 5 min; Deployment pods pick up via env (restart pods to force-refresh: `kubectl -n oya-llm-gateway rollout restart deploy/llm-gateway`).
