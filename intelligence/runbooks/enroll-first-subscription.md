# Enroll first Anthropic OAuth subscription

This runbook provisions the oyatie-dogfood tenant's first Anthropic OAuth subscription
into the cloud-intelligence gateway. Every step that requires human credentials is
flagged **[human-auth]**.

Prereqs: Talos cluster with ArgoCD + ESO + owned cloud-secrets/cloud-kms adapters +
cloud-intelligence deployed and running (see `SETUP-RUNBOOK.md` — "Production deploy on Talos" section).

---

## Step 0 — Build + publish the cloud-intelligence container image

Run the build-and-push script from the repo root:

```sh
./scripts/build/build-and-push-cloud-intelligence.sh
# Outputs: digest: sha256:<...>
# Copy the digest into intelligence/iac/k8s/helm/values.yaml
```

Pin the digest in Helm values and push to dev so ArgoCD can deploy:

```sh
# Edit intelligence/iac/k8s/helm/values.yaml
# Set:  digest: "sha256:<actual-digest-from-above>"
git commit -am "chore(cloud-intelligence): pin v0.1.0 image digest to sha256:<...>" && git push origin dev
# ArgoCD picks up the change within ~30s
```

Verify ArgoCD transitions from "Missing" to "Healthy":

```sh
kubectl -n argocd get application cloud-intelligence -o jsonpath='{.status.health.status}'
# expect: Healthy
```

---

## Step 1 — Obtain an Anthropic OAuth refresh token  **[human-auth]**

Visit the Anthropic OAuth authorization endpoint in a browser:

```
https://claude.ai/oauth/authorize
  ?client_id=9d1c250a-e61b-44d9-88ed-5944d1962f5e
  &response_type=code
  &redirect_uri=https://cloud-intelligence.oyatie.com/anthropic/oauth/callback
  &scope=openid%20profile%20email
  &state=<random-csrf-token>
```

Complete the OAuth flow on claude.ai. The authorization server will redirect to the
callback URI with a `code` parameter. Exchange it for tokens:

```sh
curl -s -X POST https://claude.ai/oauth/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=authorization_code" \
  -d "code=<code-from-redirect>" \
  -d "client_id=9d1c250a-e61b-44d9-88ed-5944d1962f5e" \
  -d "redirect_uri=https://cloud-intelligence.oyatie.com/anthropic/oauth/callback"
```

Copy the `refresh_token` from the response. Keep it in a password manager — it is the
long-lived credential.

---

## Step 2 — Register the refresh-token handle  **[human-auth]**

Store the refresh token in owned cloud-secrets/cloud-kms and register only an
opaque handle with cloud-intelligence:

- `secret_handle`: `secret-ref://cloud-intelligence/oyatie-dogfood/anthropic/seat-dogfood-1`
- `seat_id`: `seat-dogfood-1`
- `secret_provider_token`: short-lived token for the secret-provider adapter
- provider API keys: stored behind their own `secret-ref://` or `kms-ref://`
  handles; never projected as raw env values

The ESO ExternalSecret `cloud-intelligence-secrets` will sync within the configured
`refreshInterval` (default: 5 minutes). Verify:

```sh
kubectl -n cloud-intelligence get externalsecret cloud-intelligence-secrets \
  -o jsonpath='{.status.conditions[0]}'
# expect: {"reason":"SecretSynced","status":"True","type":"Ready"}

kubectl -n cloud-intelligence get secret cloud-intelligence-secrets
# expect: NAME                          TYPE     DATA   AGE
#         cloud-intelligence-secrets   Opaque   4      <age>
```

---

## Step 3 — Verify the gateway picks up the seat  **[human-auth]**

After the secret syncs, restart the deployment so the new secret is mounted:

```sh
kubectl -n cloud-intelligence rollout restart deploy/cloud-intelligence
kubectl -n cloud-intelligence rollout status deploy/cloud-intelligence --timeout=5m
```

Confirm the seat is registered in the logs:

```sh
kubectl -n cloud-intelligence logs deploy/cloud-intelligence | grep seat-dogfood-1
# expect a log line containing seat_id=seat-dogfood-1 and event=seat_registered (or similar)
```

---

## Step 4 — Smoke-test the proxy  **[human-auth]**

Point Claude Code at the gateway using an ingress proxy key resolved from the
owned secret-provider handle:

```sh
# Retrieve an ingress proxy key
PROXY_KEY=$(kubectl -n cloud-intelligence exec deploy/cloud-intelligence -- \
  printenv OYA_CLOUD_INTEL_SECRET_PROVIDER_TOKEN 2>/dev/null || \
  echo "<resolve secret-ref://cloud-intelligence/ingress-proxy-keys>")

# Health check
curl -sS \
  -H "Authorization: Bearer ${PROXY_KEY}" \
  https://cloud-intelligence.oyatie.com/anthropic/v1/health
# expect: HTTP 200 {"status":"ok"}

# Route a real request through the gateway
ANTHROPIC_BASE_URL=https://cloud-intelligence.oyatie.com/anthropic \
  claude --version
# expect: version printed without TLS or auth error
```

---

## Step 5 — Label agent namespaces that should reach the gateway

Any namespace whose pods must reach the gateway needs the Cilium egress label:

```sh
kubectl label namespace <your-agent-namespace> oya.gateway-client=true
```

Without this label, the L4 network policy denies the connection.

---

## Rotation

90-day cadence: re-run steps 1–2 with fresh tokens; ESO syncs within 5 minutes; restart
the deployment to pick up the new secret:

```sh
kubectl -n cloud-intelligence rollout restart deploy/cloud-intelligence
```
