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
| `clustersecretstore-openbao-oya.yaml` | `ClusterSecretStore openbao-oya`, `ClusterSecretStore openbao-cloud-k8s-csi`, `ClusterSecretStore openbao-cloud-iam-svid-operator` | ESO `vault` providers pointed at OpenBao with separate OpenBao roles for CI, cloud-k8s CSI, and cloud-iam SVID-operator prefixes |
| `clusterrolebinding-external-secrets-auth-delegator.yaml` | `ClusterRoleBinding` | grants the ESO SA `system:auth-delegator` for TokenReview |
| `externalsecret-github-ci-token.yaml` | `ExternalSecret github-ci-token` | projects the GitHub CI token into `oya-ci` |

## TLS migration (8200/8201 -> 8202/8203)

The live listener remains HTTP API `8200` / cluster `8201` during bootstrap.
`infra/kms/openbao.k8s.yaml` declares the parallel TLS API `8202` / cluster
`8203` config and exact GitHub OIDC/PKI roles without activating them early.

1. Install the offline root's **public certificate only** as ConfigMap
   `external-secrets/openbao-offline-root-ca` key `ca.crt`, and install the
   separately protected server TLS Secret in `oya-kms`. Never put the CA private
   key, JWT, OpenBao token, or issued leaf in git or captured output.
2. Apply the dual-listener config, then verify both legacy and TLS health.
3. Verify all three `*-tls-migration` ClusterSecretStores are Ready, restart ESO,
   and prove they remain Ready and can refresh their existing prefixes.
4. In a later reviewed change, point the three canonical stores at HTTPS 8202.
   Only after consumer readback may HTTP 8200/8201 be removed; TLS 8202/8203 is
   the permanent pair.

The OIDC role payloads in `openbao-ci-identity-contract` bind audience
`oya-openbao`, immutable repository/owner IDs, private visibility,
self-hosted runners, exact workflow refs, and exact event/ref claims. JWTs are
bounded to five minutes; issued client leaves are bounded to three hours.

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

### 3. Create the read policies

Grants read on CI secrets under `secret/data/oya/ci/*`, CSI substrate secrets
under `secret/data/cloud-k8s/csi/*`, and the cloud-iam SVID-operator join token
under `secret/data/cloud-iam/pdp-svid-operator/*` (KV v2 prefixes reads with `data/`).

```sh
bao policy write oya-ci-read - <<'EOF'
path "secret/data/oya/ci/*" {
  capabilities = ["read"]
}
EOF

bao policy write cloud-k8s-csi-read - <<'EOF'
path "secret/data/cloud-k8s/csi/*" {
  capabilities = ["read"]
}
EOF

bao policy write cloud-iam-svid-operator-read - <<'EOF'
path "secret/data/cloud-iam/pdp-svid-operator/*" {
  capabilities = ["read"]
}
EOF
```

### 4. Create the roles

Binds each role to the ESO ServiceAccount and attaches only the matching read
policy. These are the `role` values the ClusterSecretStores reference.

```sh
bao write auth/kubernetes/role/eso-oya-ci \
    bound_service_account_names=external-secrets \
    bound_service_account_namespaces=external-secrets \
    policies=oya-ci-read \
    ttl=1h

bao write auth/kubernetes/role/eso-cloud-k8s-csi \
    bound_service_account_names=external-secrets \
    bound_service_account_namespaces=external-secrets \
    policies=cloud-k8s-csi-read \
    ttl=1h

bao write auth/kubernetes/role/eso-cloud-iam-svid-operator \
    bound_service_account_names=external-secrets \
    bound_service_account_namespaces=external-secrets \
    policies=cloud-iam-svid-operator-read \
    ttl=1h
```

### 5. Seed the governed secrets

Store the actual GitHub CI commit-status token (mint via
`github admin user generate-access-token --username oya-admin --token-name jenkins-ci --scopes write:repository --raw`).
This is the source of truth the `github-ci-token` ExternalSecret pulls from.

```sh
bao kv put secret/oya/ci/github-ci-token token="<GITHUB_CI_TOKEN>"

# CSI substrate credentials consumed by cloud/cloud-k8s/iac/kustomize/base/openbao-secret-references.yaml
bao kv put secret/cloud-k8s/csi/block-volume \
    endpoint="<BLOCK_VOLUME_ENDPOINT>" \
    tenant_id="<BLOCK_VOLUME_TENANT_ID>" \
    key_id="<BLOCK_VOLUME_KEY_ID>"

bao kv put secret/cloud-k8s/csi/object \
    s3_endpoint="<OBJECT_S3_ENDPOINT>" \
    access_key_id="<OBJECT_ACCESS_KEY_ID>" \
    secret_access_key="<OBJECT_SECRET_ACCESS_KEY>"

bao kv put secret/cloud-k8s/csi/file \
    endpoint="<FILE_ENDPOINT>" \
    export_root="<FILE_EXPORT_ROOT>" \
    key_id="<FILE_KEY_ID>"

# cloud-iam SVID operator join token consumed by cloud/cloud-iam/iac/k8s/helm/templates/svid-operator-join-token-externalsecret.yaml
bao kv put secret/cloud-iam/pdp-svid-operator/join-token \
    join-token="<SVID_OPERATOR_JOIN_TOKEN>"
```

> Never commit real token or credential values. They live only in OpenBao
> (barrier-encrypted at rest per `infra/kms/openbao.k8s.yaml`) and are projected
> into their target namespaces by ESO.

## Verify

```sh
# ClusterSecretStore should report Valid / Ready
kubectl get clustersecretstore openbao-oya -o jsonpath='{.status.conditions}'
kubectl get clustersecretstore openbao-cloud-k8s-csi -o jsonpath='{.status.conditions}'
kubectl get clustersecretstore openbao-cloud-iam-svid-operator -o jsonpath='{.status.conditions}'

# ExternalSecret should report SecretSynced, and the target Secret should exist
kubectl -n oya-ci get externalsecret github-ci-token
kubectl -n oya-ci get secret github-ci-token

# cloud-k8s CSI ExternalSecrets should sync through the dedicated store/role
kubectl -n cloud-k8s-system get externalsecret cloud-k8s-csi-block-volume-credentials
kubectl -n cloud-k8s-system get externalsecret cloud-k8s-csi-object-credentials
kubectl -n cloud-k8s-system get externalsecret cloud-k8s-csi-file-credentials

# cloud-iam SVID operator join-token ExternalSecret should sync through its dedicated store/role
kubectl -n cloud-iam get externalsecret oya-cloud-iam-pdp-svid-operator-join-token
```

If the ClusterSecretStore is not Ready with a `permission denied` / `403` on
login, the usual cause is the `disable_local_ca_jwt` / auth-delegator invariant
above being broken (flag false, or the ClusterRoleBinding missing).
