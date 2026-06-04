# OpenBao + External Secrets Operator (ESO) CI-secrets runbook

This runbook documents the live OpenBao <- ESO wiring that delivers CI secrets
to the `ci-controller` and `ci-webhook-gateway` microservices, and how to
reproduce it from the committed IaC.

> All secret material in this document is `PLACEHOLDER`. Never paste a real
> token, password, or HMAC secret into this file or any commit.

> Legacy adapter-secret compatibility note (2026-06-04): the exact external
> SCM adapter secret names below remain only because the current
> `ci-controller` / `ci-webhook-gateway` manifests still consume them. They are
> not SCM/CI authority and are not a pattern for new work. Retire or rename them
> only through the `retired_external_scm_adapter_retirement` lane after the Rust
> GitHub/native-SCM adapters are wired.

## Committed IaC (source of truth)

| File | Captures |
| --- | --- |
| `infra/eso/clustersecretstore-openbao.yaml` | `ClusterSecretStore/openbao-cluster-store` (ESO vault provider -> OpenBao) |
| `infra/eso/clusterrolebinding-external-secrets-auth-delegator.yaml` | `ClusterRoleBinding/external-secrets-auth-delegator` (`system:auth-delegator`) |
| `oya/ci-controller/iac/k8s/helm/templates/externalsecret.yaml` | `ExternalSecret` pulling `forgejo-ci-token` |
| `oya/ci-webhook-gateway/iac/k8s/helm/templates/externalsecret.yaml` | `ExternalSecret` pulling `forgejo-webhook-secret` |

## Topology at a glance

```
external-secrets ns                         oya-kms ns
+-----------------------+                    +--------------------------+
| ESO controller        |  client JWT auth   | OpenBao (Vault-compatible)|
| SA: external-secrets  | -----------------> | Service: openbao :8200    |
| (presents OWN SA JWT) |                    | auth/kubernetes (mount)   |
+-----------------------+                    | KV v2 engine: secret/     |
          |                                  +--------------------------+
          | OpenBao validates the client JWT          ^
          | by calling TokenReview ------------------>| kube-apiserver
          | (needs system:auth-delegator)             | (TokenReview API)
          v
  ExternalSecrets -> native k8s Secrets in each microservice namespace
```

## Authoritative names, paths, and fields

| Thing | Value |
| --- | --- |
| ClusterSecretStore name | `openbao-cluster-store` |
| ClusterSecretStore kind | `ClusterSecretStore` |
| OpenBao server (in-cluster) | `http://openbao.oya-kms.svc:8200` |
| KV mount / engine version | `secret` / v2 |
| OpenBao kubernetes auth mount | `kubernetes` |
| OpenBao role | `eso-oya-ci` |
| ESO client identity (ServiceAccount) | `external-secrets` in namespace `external-secrets` |
| ci-controller KV path | `secret/oya/ci/forgejo-ci-token`, field `token` |
| ci-webhook-gateway KV path | `secret/oya/ci/forgejo-webhook-secret`, field `value` |

## 1. OpenBao kubernetes auth configuration

ESO authenticates to OpenBao using the **client-JWT TokenReview** path. ESO
presents its own `external-secrets` ServiceAccount token as the client JWT, and
OpenBao validates that token by calling the cluster's TokenReview API. This is
enabled by `disable_local_ca_jwt=true`, which tells OpenBao NOT to use its own
pod's locally-mounted CA/token to reach the Kubernetes API.

```bash
# Run against the OpenBao server (e.g. `kubectl -n oya-kms exec` into the
# OpenBao pod, or with BAO_ADDR + a privileged token from a bastion).

# 1a. Enable the kubernetes auth method at mount path "kubernetes".
bao auth enable -path=kubernetes kubernetes

# 1b. Configure it for the client-JWT TokenReview path.
#     disable_local_ca_jwt=true => OpenBao validates the CLIENT JWT that ESO
#     presents via TokenReview (it does NOT use its own pod token/CA).
bao write auth/kubernetes/config \
  kubernetes_host="https://kubernetes.default.svc.cluster.local" \
  disable_local_ca_jwt=true

# 1c. Policy that allows reading the CI secrets under secret/oya/ci/*.
#     KV v2 read paths live under <mount>/data/<path>.
bao policy write oya-ci-read - <<'EOF'
path "secret/data/oya/ci/*" {
  capabilities = ["read"]
}
path "secret/metadata/oya/ci/*" {
  capabilities = ["read", "list"]
}
EOF

# 1d. Role binding the ESO ServiceAccount to that policy.
#     The role name "eso-oya-ci" must match the ClusterSecretStore
#     auth.kubernetes.role field.
bao write auth/kubernetes/role/eso-oya-ci \
  bound_service_account_names=external-secrets \
  bound_service_account_namespaces=external-secrets \
  policies=oya-ci-read \
  ttl=1h
```

## 2. The auth-delegator grant (so TokenReview is allowed)

Because `disable_local_ca_jwt=true` makes OpenBao validate the ESO client JWT
via TokenReview, the `external-secrets` controller ServiceAccount must be
permitted to create TokenReviews. That permission comes from the built-in
`system:auth-delegator` ClusterRole.

Captured in `infra/eso/clusterrolebinding-external-secrets-auth-delegator.yaml`:

```bash
kubectl apply -f infra/eso/clusterrolebinding-external-secrets-auth-delegator.yaml

# Equivalent imperative form (for reference only; prefer the committed manifest):
#   kubectl create clusterrolebinding external-secrets-auth-delegator \
#     --clusterrole=system:auth-delegator \
#     --serviceaccount=external-secrets:external-secrets
```

If this binding is missing, OpenBao's TokenReview call is rejected and every
ExternalSecret referencing `openbao-cluster-store` fails to sync with an auth
error.

## 3. Apply the ClusterSecretStore

```bash
kubectl apply -f infra/eso/clustersecretstore-openbao.yaml
```

This creates `ClusterSecretStore/openbao-cluster-store` pointing at
`http://openbao.oya-kms.svc:8200`, KV mount `secret` (v2),
kubernetes auth mount `kubernetes`, role `eso-oya-ci`, and
`serviceAccountRef` -> `external-secrets/external-secrets`.

## 4. Write the CI secrets into OpenBao (KV v2)

The two CI secrets live under `secret/oya/ci/`. Use `PLACEHOLDER` and replace
it out-of-band with the real value; do not commit real material.

```bash
# ci-controller: Forgejo CI token. Field name MUST be "token" — the
# ci-controller ExternalSecret reads remoteRef.property: token.
bao kv put secret/oya/ci/forgejo-ci-token token=PLACEHOLDER

# ci-webhook-gateway: Forgejo webhook HMAC secret. Field name MUST be "value"
# — the ci-webhook-gateway ExternalSecret reads remoteRef.property: value.
bao kv put secret/oya/ci/forgejo-webhook-secret value=PLACEHOLDER
```

## 5. How the microservices consume them

Both ExternalSecrets set `secretStoreRef: { name: openbao-cluster-store, kind:
ClusterSecretStore }` and create a native Kubernetes Secret in their own
namespace.

- **ci-controller** (`oya/ci-controller/iac/k8s/helm/templates/externalsecret.yaml`):
  - `remoteRef.key: secret/oya/ci/forgejo-ci-token`, `property: token`
  - materialized `secretKey: token` into Secret `forgejo-ci-token`
  - consumed as env `FORGEJO_CI_TOKEN` (controller) and mounted into gate-runner
    Pods as a `SecretKeyRef`.

- **ci-webhook-gateway** (`oya/ci-webhook-gateway/iac/k8s/helm/templates/externalsecret.yaml`):
  - `remoteRef.key: secret/oya/ci/forgejo-webhook-secret`, `property: value`
  - materialized `secretKey: forgejo_webhook_secret` into Secret
    `ci-webhook-gateway-secrets`
  - consumed as env `OYA_FORGEJO_WEBHOOK_SECRET` (the only runtime secret the
    binary reads, per `src/config.rs`).

## 6. Verification

```bash
# 6a. ClusterSecretStore should report a Valid status (Ready=True).
kubectl get clustersecretstore openbao-cluster-store \
  -o jsonpath='{.status.conditions[?(@.type=="Ready")].reason}{"\n"}'
# Expect: ValidStore (and STATUS column = Valid in the wide view below)
kubectl get clustersecretstore openbao-cluster-store -o wide

# 6b. List all ExternalSecrets across namespaces and confirm SyncedReady=True.
kubectl get externalsecret -A

# 6c. Confirm the auth-delegator binding exists and targets the ESO SA.
kubectl get clusterrolebinding external-secrets-auth-delegator -o yaml | \
  grep -A3 subjects

# 6d. Confirm the synced native Secrets exist in each namespace.
kubectl get secret forgejo-ci-token -n <ci-controller-namespace>
kubectl get secret ci-webhook-gateway-secrets -n <ci-webhook-gateway-namespace>
```

A healthy state: `ClusterSecretStore/openbao-cluster-store` STATUS `Valid`, and
each `ExternalSecret` STATUS `SecretSynced` / Ready `True`.

### Troubleshooting

- ExternalSecret stuck with an auth/permission-denied error from OpenBao:
  verify the `external-secrets-auth-delegator` ClusterRoleBinding (step 2) and
  that the OpenBao role `bound_service_account_names`/`...namespaces` match the
  `external-secrets` SA in the `external-secrets` namespace.
- Store `Invalid`: confirm OpenBao is reachable at
  `http://openbao.oya-kms.svc:8200` and the kubernetes auth
  mount path is `kubernetes`.
- Empty/missing key on sync: confirm the KV writes in step 4 used the exact
  field names `token` (ci-token) and `value` (webhook-secret).

## 7. Store-name reconciliation: legacy `openbao-oya` → canonical `openbao-cluster-store`

**Drift (observed 2026-05-31).** The live cluster currently has a single
hand-applied ClusterSecretStore named **`openbao-oya`** (`Valid`), while every
committed manifest — both helm `ExternalSecret` templates
(`oya/ci-controller`, `oya/ci-webhook-gateway`), `oya/feature-flags`, and the
two files in `infra/eso/` — references the canonical name
**`openbao-cluster-store`**, which does **not yet exist live**. So the committed
helm charts are not yet deployable as-is (their ExternalSecrets would point at a
missing store). `infra/eso/clustersecretstore-openbao.yaml` captures the *real*
working config (server `http://openbao.oya-kms.svc:8200`, role `eso-oya-ci`)
under the canonical name.

**Convergence steps (require cluster-admin / RBAC authorization — not yet
applied).** Performing these is a Permission-Grant + shared-resource change; run
only with explicit authorization:

```bash
# 1. Create the canonical store (+ the auth-delegator binding if absent).
kubectl apply -f infra/eso/clustersecretstore-openbao.yaml \
               -f infra/eso/clusterrolebinding-external-secrets-auth-delegator.yaml
kubectl get clustersecretstore openbao-cluster-store -o wide   # expect STATUS Valid

# 2. Repoint each ExternalSecret from openbao-oya to openbao-cluster-store
#    (same OpenBao, same KV paths -> secret content is unchanged; ESO re-syncs).
#    CI lane (oya-ci): forgejo-ci-token, forgejo-webhook-secret-eso.
#    NOTE: other consumers also use openbao-oya (e.g. observability/grafana-admin);
#    migrate ALL of them before deleting openbao-oya.
kubectl -n oya-ci patch externalsecret forgejo-ci-token --type=merge \
  -p '{"spec":{"secretStoreRef":{"name":"openbao-cluster-store"}}}'
# ... repeat for every openbao-oya consumer, verifying SecretSynced=True each ...

# 3. Once NOTHING references openbao-oya, retire it.
kubectl delete clustersecretstore openbao-oya
```

Until step 1 runs, the live CI secrets continue to flow through the legacy
`openbao-oya` store (the gate loop is unaffected).
