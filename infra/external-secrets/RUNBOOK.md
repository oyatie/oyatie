# ESO -> OpenBao runbook (oya-kms)

Captures the **OpenBao-side** setup that pairs with the Kubernetes manifests in
this directory. The k8s objects (ClusterSecretStore / ClusterRoleBinding /
ExternalSecret) are declarative and GitOps-reconciled; the OpenBao
configuration below is applied **inside** OpenBao (its own state/storage) and is
not expressible as a k8s manifest, so it lives here as a reproducible runbook.

These steps were originally applied live-only against the running cluster. This
runbook makes a fresh cluster reproducible: bring up OpenBao
(`infra/kms/openbao.k8s.yaml`) + the External Secrets Operator
(`infra/gitops/values.yaml` -> `external-secrets`), apply the manifests in this
directory, then run the OpenBao steps below.

## Components (this directory)

| File | Kind | Purpose |
|------|------|---------|
| `clustersecretstore-openbao-oya.yaml` | `ClusterSecretStore openbao-oya` | ESO `vault` provider pointed at OpenBao, kubernetes auth |
| `clusterrolebinding-external-secrets-auth-delegator.yaml` | `ClusterRoleBinding` | grants the ESO SA `system:auth-delegator` for TokenReview |
| `externalsecret-github-ci-token.yaml` | `ExternalSecret github-ci-token` | projects the GitHub CI token into `oya-ci` |

## The auth-delegator / `disable_local_ca_jwt` invariant (read this first)

The whole binding hinges on **one** OpenBao config flag:

```
auth/kubernetes/config  disable_local_ca_jwt=true
```

- With `disable_local_ca_jwt=true`, OpenBao performs the Kubernetes
  **TokenReview using the CLIENT JWT** — the `external-secrets/external-secrets`
  ServiceAccount token that ESO presents on each login.
- That client SA is granted `system:auth-delegator` by
  `clusterrolebinding-external-secrets-auth-delegator.yaml`, so the TokenReview
  is authorized.
- This **avoids** having to grant `system:auth-delegator` to **OpenBao's own**
  ServiceAccount. OpenBao runs as a plain Deployment SA in `oya-kms` and is not
  wired for delegated TokenReview; leaving the flag at its default (`false`)
  makes OpenBao try the TokenReview with ITS local SA token, which 403s and
  breaks every ExternalSecret bound to this store.

If you ever flip `disable_local_ca_jwt` back to `false`, you MUST instead grant
`system:auth-delegator` to OpenBao's ServiceAccount in `oya-kms`.

## OpenBao-side setup

Run these against OpenBao (e.g. `kubectl -n oya-kms exec` into the pod, or via a
port-forward with `BAO_ADDR=http://127.0.0.1:8200` and an authenticated token).
`bao` and `vault` CLIs are interchangeable against OpenBao.

### 1. Enable the Kubernetes auth method

```sh
bao auth enable kubernetes        # mounts at auth/kubernetes (matches ClusterSecretStore mountPath)
```

### 2. Configure the Kubernetes auth method

`kubernetes_host` is the in-cluster API server (the default ClusterIP of the
`kubernetes` service in `default`). `kubernetes_ca_cert` is the cluster CA that
signs the API server cert — read it from the in-cluster SA mount or the
kube-root-ca ConfigMap.

```sh
bao write auth/kubernetes/config \
    kubernetes_host="https://10.96.0.1:443" \
    kubernetes_ca_cert=@/var/run/secrets/kubernetes.io/serviceaccount/ca.crt \
    disable_local_ca_jwt=true \
    disable_iss_validation=true
```

- `disable_local_ca_jwt=true` — see the invariant above (CLIENT JWT performs the
  TokenReview; ESO SA holds `system:auth-delegator`).
- `disable_iss_validation=true` — do not validate the JWT `iss` claim; tolerates
  projected-token / bound-SA-token issuer differences across cluster setups.
- The cluster CA can also be supplied inline:
  `kubernetes_ca_cert="$(kubectl -n kube-system get cm kube-root-ca.crt -o jsonpath='{.data.ca\.crt}')"`.

### 3. Create the read policy `oya-ci-read`

Grants read on every CI secret under `secret/data/oya/ci/*` (KV v2 prefixes
reads with `data/`).

```sh
bao policy write oya-ci-read - <<'EOF'
path "secret/data/oya/ci/*" {
  capabilities = ["read"]
}
EOF
```

### 4. Create the role `eso-oya-ci`

Binds the role to the ESO ServiceAccount and attaches the read policy. This is
the `role` the ClusterSecretStore references.

```sh
bao write auth/kubernetes/role/eso-oya-ci \
    bound_service_account_names=external-secrets \
    bound_service_account_namespaces=external-secrets \
    policies=oya-ci-read \
    ttl=1h
```

### 5. Seed the GitHub CI token

Store the actual GitHub CI commit-status token (mint via
`github admin user generate-access-token --username oya-admin --token-name jenkins-ci --scopes write:repository --raw`).
This is the source of truth the `github-ci-token` ExternalSecret pulls from.

```sh
bao kv put secret/oya/ci/github-ci-token token="<GITHUB_CI_TOKEN>"
```

> Never commit the real token. It lives only in OpenBao (barrier-encrypted at
> rest per `infra/kms/openbao.k8s.yaml`) and is projected into `oya-ci` by ESO.

## Verify

```sh
# ClusterSecretStore should report Valid / Ready
kubectl get clustersecretstore openbao-oya -o jsonpath='{.status.conditions}'

# ExternalSecret should report SecretSynced, and the target Secret should exist
kubectl -n oya-ci get externalsecret github-ci-token
kubectl -n oya-ci get secret github-ci-token
```

If the ClusterSecretStore is not Ready with a `permission denied` / `403` on
login, the usual cause is the `disable_local_ca_jwt` / auth-delegator invariant
above being broken (flag false, or the ClusterRoleBinding missing).
