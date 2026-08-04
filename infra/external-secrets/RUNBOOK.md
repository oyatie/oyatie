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

The base `infra/kms/openbao.k8s.yaml` exposes only live plaintext `8200/8201`.
Run these stages in order; never put a CA private key, JWT, OpenBao token, server
private key, or issued leaf in git or captured output.

1. **Preflight.** Confirm the base Deployment is Available and its Service has
   only `8200/8201`. Do not apply the empty public-CA scaffold directly. Have
   the trusted bootstrap generate and apply populated ConfigMaps from the
   offline root's **public certificate only**. Bootstrap Secret
   `oya-kms/openbao-server-tls` with keys `tls.crt` and `tls.key`. Confirm the
   populated ConfigMap `openbao-offline-root-ca` exists in both
   `external-secrets` and `arc-runners`, the certificate covers
   `openbao.oya-kms.svc` and the Secret/ConfigMaps exist without printing data.
2. Apply `infra/kms/openbao-ci-identity.k8s.yaml`, then use the authenticated,
   no-echo **initial** bootstrap below. Its readback covers OpenBao state and
   the create-only KV record only; it does not touch the not-yet-deployed
   NativeLink ExternalSecret or Deployment. There is no bootstrap controller in this slice.
   Do **not** apply the TLS migration manifest directly: Argo owns the same
   Deployment, Service, and NetworkPolicy with prune/self-heal enabled.
3. After the bootstrap readback passes, promote TLS through a reviewed PR against
   `dev` that changes the `openbao` Application include in
   `infra/gitops/values.yaml` from `openbao.k8s.yaml` to
   `openbao-tls-migration.k8s.yaml` and adds an exact-path Application for
   `infra/kms/openbao-ci-identity.k8s.yaml` with `cascadeDelete: true`. Merge only
   after `oya-ci-required` and review; then require both Argo Applications to be
   Synced and Healthy. Record the Namespace and `openbao-data` PVC UIDs before
   and after reconciliation; both must remain identical.
4. **Wait/readback.** Wait for Deployment `oya-kms/openbao` rollout completion;
   verify the mounted config and Secret references, Service ports `8200..8203`,
   and successful authenticated TLS health on `8202`. Verify plaintext `8200`
   still answers during this dual-listener phase.
5. Only after step 4 succeeds, use a second reviewed PR to add exact-path GitOps
   Applications for
   `infra/external-secrets/clustersecretstore-openbao-oya-tls-migration.yaml` and
   `infra/nativelink/nativelink-cas.k8s.yaml`, both with `cascadeDelete: true`;
   do not deploy either by raw apply. That value adds Argo's foreground resources
   finalizer so a later reviewed Application removal cannot orphan live objects.
   Wait until all three `*-tls-migration` stores report Ready, restart ESO, wait
   for readiness again, and prove each existing consumer prefix refreshes. Once
   both new Applications are Synced and Healthy, run the post-deployment
   NativeLink projection/rollout procedure below.
6. A later reviewed cutover may point canonical stores to `8202`; remove
   `8200/8201` only after consumer readback.

Before `warm_reads_licensed` is true, run the cache canary manually on `dev` with
`prelicense_probe=true`, but only after setting repository variable
`OYA_CAS_IDENTITY_PROOF_ENABLED=true`. The next trusted `dev` push seeds the CAS
through the isolated writer PKI; the explicit canary then reads those entries
through the reader PKI without changing the license. Leave the variable absent
until every prerequisite below exists. Scheduled runs remain fail-closed while
unlicensed.

**Rollback is ordered.** First set `OYA_CAS_IDENTITY_PROOF_ENABLED=false`, keep
`warm_reads_licensed=false`, and wait for all cache identity jobs/clients to
quiesce. Revert the second promotion commit and wait for its two
`cascadeDelete: true` Applications plus NativeLink/migration-store resources to
disappear. Then revert the first promotion commit, wait for base OpenBao to be
Synced, Healthy, and Available, and prove the Namespace and `openbao-data` PVC
UIDs match the pre-promotion receipt. Finally verify the three canonical stores
are Ready on `8200`. Never raw-apply the base manifest against Argo self-heal.
Preserve the bootstrap TLS Secret for diagnosis/forward repair; do not print or
export it.

The OIDC role payloads in `openbao-ci-identity-contract` bind audience
`oya-openbao`, immutable repository/owner IDs, private visibility,
self-hosted runners, exact `sub`/`workflow_ref`, and exact event/ref claims. JWTs are
bounded to five minutes; issued client leaves are bounded to three hours.

### Authenticated CI identity bootstrap

Prerequisites are an unsealed OpenBao, an authenticated operator session in
`BAO_TOKEN`, the public OpenBao HTTPS CA in `OYA_OPENBAO_CA_CERT`, and a
ceremony-issued NativeLink server leaf, key, and independent public CA in
`OYA_NATIVELINK_SERVER_CERT`, `OYA_NATIVELINK_SERVER_KEY`, and
`OYA_NATIVELINK_SERVER_CA_CERT`. Run from the repository root with `bao`,
`kubectl`, and OpenSSL; the commands redirect PKI material to a mode-0700
temporary directory and print no token, private key, certificate, or ConfigMap
body. The KV-v2 `cas=0` write is create-only, so an existing, soft-deleted, or
concurrently created NativeLink TLS record fails closed instead of being overwritten
([OpenBao `kv put`](https://openbao.org/docs/commands/kv/put/)).

```sh
set -eu
umask 077
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
: "${BAO_TOKEN:?BAO_TOKEN is required}"
: "${OYA_OPENBAO_CA_CERT:?OYA_OPENBAO_CA_CERT is required}"
: "${OYA_NATIVELINK_SERVER_CERT:?OYA_NATIVELINK_SERVER_CERT is required}"
: "${OYA_NATIVELINK_SERVER_KEY:?OYA_NATIVELINK_SERVER_KEY is required}"
: "${OYA_NATIVELINK_SERVER_CA_CERT:?OYA_NATIVELINK_SERVER_CA_CERT is required}"
for path in "$OYA_OPENBAO_CA_CERT" "$OYA_NATIVELINK_SERVER_CERT" \
  "$OYA_NATIVELINK_SERVER_KEY" "$OYA_NATIVELINK_SERVER_CA_CERT"; do
  [ -s "$path" ] || { echo "required PKI input is absent or empty" >&2; exit 1; }
done
ca_begin_count="$(grep -c '^-----BEGIN ' "$OYA_NATIVELINK_SERVER_CA_CERT" || true)"
ca_end_count="$(grep -c '^-----END ' "$OYA_NATIVELINK_SERVER_CA_CERT" || true)"
if [ "$ca_begin_count" -ne 1 ] || [ "$ca_end_count" -ne 1 ] || \
  ! grep -qx -- '-----BEGIN CERTIFICATE-----' "$OYA_NATIVELINK_SERVER_CA_CERT" || \
  ! grep -qx -- '-----END CERTIFICATE-----' "$OYA_NATIVELINK_SERVER_CA_CERT"; then
  echo "NativeLink server CA must contain exactly one PEM certificate" >&2
  exit 1
fi
openssl verify -purpose sslserver -CAfile "$OYA_NATIVELINK_SERVER_CA_CERT" \
  "$OYA_NATIVELINK_SERVER_CERT" >/dev/null
openssl x509 -in "$OYA_NATIVELINK_SERVER_CERT" -noout \
  -ext subjectAltName >"$tmp/server-sans.txt"
tr ',' '\n' <"$tmp/server-sans.txt" | \
  sed 's/^[[:space:]]*//; /^$/d; /X509v3 Subject Alternative Name:/d' | \
  LC_ALL=C sort -u >"$tmp/server-sans.actual"
cat >"$tmp/server-sans.expected" <<'EOF'
DNS:nativelink-cas-reader.oya-ci.svc.cluster.local
DNS:nativelink-cas-writer.oya-ci.svc.cluster.local
EOF
cmp -s "$tmp/server-sans.actual" "$tmp/server-sans.expected" || {
  echo "NativeLink server certificate SANs are not the exact two service DNS names" >&2
  exit 1
}
openssl x509 -in "$OYA_NATIVELINK_SERVER_CERT" -noout \
  -ext extendedKeyUsage >"$tmp/server-eku.txt"
grep -Fq "TLS Web Server Authentication" "$tmp/server-eku.txt" || {
  echo "NativeLink server certificate lacks the server-auth EKU" >&2
  exit 1
}
if grep -Fq "TLS Web Client Authentication" "$tmp/server-eku.txt"; then
  echo "NativeLink server certificate must not carry the client-auth EKU" >&2
  exit 1
fi
openssl x509 -in "$OYA_NATIVELINK_SERVER_CERT" -pubkey -noout \
  >"$tmp/server-cert.pub"
openssl pkey -in "$OYA_NATIVELINK_SERVER_KEY" -pubout \
  >"$tmp/server-key.pub"
cmp -s "$tmp/server-cert.pub" "$tmp/server-key.pub" || {
  echo "NativeLink server certificate and key do not match" >&2
  exit 1
}
kubectl apply -f infra/kms/openbao-ci-identity.k8s.yaml >/dev/null
for namespace in external-secrets arc-runners; do
  kubectl -n "$namespace" create configmap openbao-offline-root-ca \
    --from-file=ca.crt="$OYA_OPENBAO_CA_CERT" \
    --dry-run=client -o yaml | kubectl apply -f - >/dev/null
done
kubectl -n oya-kms get configmap openbao-ci-identity-contract \
  -o jsonpath='{.data.jwt-config\.json}' >"$tmp/jwt-config.json"
bao auth enable -path=jwt jwt >/dev/null 2>&1 || bao auth list -format=json | grep -q '"jwt/"'
bao write auth/jwt/config @"$tmp/jwt-config.json" >/dev/null

for identity in writer reader; do
  mount="pki_cas_${identity}"
  role="cas-${identity}"
  if bao secrets list -format=json | grep -q "\"${mount}/\""; then
    bao read -field=certificate "$mount/cert/ca" >"$tmp/${identity}-client-ca.crt"
  else
    bao secrets enable -path="$mount" pki >/dev/null
    bao secrets tune -max-lease-ttl=8760h "$mount" >/dev/null
    bao write -field=certificate "$mount/root/generate/internal" \
      common_name="Oyatie CAS ${identity} client root" ttl=8760h >"$tmp/${identity}-client-ca.crt"
  fi
  kubectl -n oya-kms get configmap openbao-ci-identity-contract \
    -o "jsonpath={.data.pki-cas-${identity}\.json}" >"$tmp/pki-role.json"
  bao write "$mount/roles/$role" @"$tmp/pki-role.json" >/dev/null
  kubectl -n oya-kms get configmap openbao-ci-identity-contract \
    -o "jsonpath={.data.ci-cas-${identity}\.hcl}" >"$tmp/policy.hcl"
  bao policy write "ci-cas-${identity}" "$tmp/policy.hcl" >/dev/null
done

for binding in github-cas-writer-dev-push github-cas-reader-integrity-canary; do
  kubectl -n oya-kms get configmap openbao-ci-identity-contract \
    -o "jsonpath={.data.${binding}\.json}" >"$tmp/jwt-role.json"
  bao write "auth/jwt/role/$binding" @"$tmp/jwt-role.json" >/dev/null
done

# Create the four-field record atomically. KV-v2 cas=0 is a server-side
# create-only precondition, so it closes the get/put race and refuses existing
# or soft-deleted keys as well as concurrent creators.
bao kv put -mount=secret -cas=0 oya/ci/nativelink-cas-tls \
  server-cert=@"$OYA_NATIVELINK_SERVER_CERT" \
  server-key=@"$OYA_NATIVELINK_SERVER_KEY" \
  writer-client-ca=@"$tmp/writer-client-ca.crt" \
  reader-client-ca=@"$tmp/reader-client-ca.crt" >/dev/null || {
  echo "NativeLink TLS record is not new; refusing overwrite" >&2
  exit 1
}
```

Read back names and metadata only: both PKI mounts must exist with different CA
serials, both JWT roles must report five-minute maximum TTLs, both policies must
name only their matching mount, and the NativeLink KV-v2 record must be at
version one.

### Retryable NativeLink public-CA projection

Run this only after the create-only write above succeeded. It is a separate,
retryable stage so a rejected CAS write cannot mutate runner trust. A retry first
proves that the stored server certificate is the ceremony certificate and that
the supplied public CA validates it; a different or deleted record fails before
Kubernetes is changed.

```sh
set -eu
umask 077
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
: "${BAO_TOKEN:?BAO_TOKEN is required}"
: "${OYA_NATIVELINK_SERVER_CERT:?OYA_NATIVELINK_SERVER_CERT is required}"
: "${OYA_NATIVELINK_SERVER_CA_CERT:?OYA_NATIVELINK_SERVER_CA_CERT is required}"
ca_begin_count="$(grep -c '^-----BEGIN ' "$OYA_NATIVELINK_SERVER_CA_CERT" || true)"
ca_end_count="$(grep -c '^-----END ' "$OYA_NATIVELINK_SERVER_CA_CERT" || true)"
if [ "$ca_begin_count" -ne 1 ] || [ "$ca_end_count" -ne 1 ] || \
  ! grep -qx -- '-----BEGIN CERTIFICATE-----' "$OYA_NATIVELINK_SERVER_CA_CERT" || \
  ! grep -qx -- '-----END CERTIFICATE-----' "$OYA_NATIVELINK_SERVER_CA_CERT"; then
  echo "NativeLink server CA must contain exactly one PEM certificate" >&2
  exit 1
fi
bao kv get -mount=secret -field=server-cert \
  oya/ci/nativelink-cas-tls >"$tmp/stored-server.crt"
openssl x509 -in "$tmp/stored-server.crt" -outform DER \
  >"$tmp/stored-server.der"
openssl x509 -in "$OYA_NATIVELINK_SERVER_CERT" -outform DER \
  >"$tmp/ceremony-server.der"
cmp -s "$tmp/stored-server.der" "$tmp/ceremony-server.der" || {
  echo "stored NativeLink server certificate differs from the ceremony" >&2
  exit 1
}
openssl verify -purpose sslserver -CAfile "$OYA_NATIVELINK_SERVER_CA_CERT" \
  "$tmp/stored-server.crt" >/dev/null
kubectl -n arc-runners create configmap nativelink-server-ca \
  --from-file=ca.crt="$OYA_NATIVELINK_SERVER_CA_CERT" \
  --dry-run=client -o yaml | kubectl apply -f - >/dev/null
kubectl -n arc-runners get configmap nativelink-server-ca \
  -o jsonpath='{.data.ca\.crt}' >"$tmp/projected-ca.crt"
cmp -s "$tmp/projected-ca.crt" "$OYA_NATIVELINK_SERVER_CA_CERT" || {
  echo "projected NativeLink server CA differs from the ceremony" >&2
  exit 1
}
```

The public CA ConfigMap must now exist without printing its data.

### Post-deployment NativeLink projection and rollout

Run this procedure only after step 5 has reconciled the NativeLink ExternalSecret
and Deployment. It fails immediately when either object is absent; the initial
bootstrap above never calls it.

```sh
set -eu
# NativeLink v1.6.2 reads its TLS acceptors/client roots at process start. Confirm
# ESO has projected the initial Secret without printing data, then restart
# NativeLink and wait for the replacement pod to become Available. Identity proof
# MUST NOT begin before this rollout completes.
kubectl -n oya-ci get externalsecret nativelink-cas-tls >/dev/null
kubectl -n oya-ci get deployment nativelink-cas >/dev/null
kubectl -n oya-ci wait --for=condition=Ready externalsecret/nativelink-cas-tls \
  --timeout=5m >/dev/null
refresh_time="$(kubectl -n oya-ci get externalsecret nativelink-cas-tls \
  -o jsonpath='{.status.refreshTime}')"
secret_rv="$(kubectl -n oya-ci get secret nativelink-cas-tls \
  -o jsonpath='{.metadata.resourceVersion}')"
[ -n "$refresh_time" ] && [ -n "$secret_rv" ] || {
  echo "NativeLink TLS Secret projection metadata is incomplete" >&2
  exit 1
}
kubectl -n oya-ci rollout restart deployment/nativelink-cas >/dev/null
kubectl -n oya-ci rollout status deployment/nativelink-cas --timeout=10m >/dev/null
kubectl -n oya-ci wait --for=condition=Available deployment/nativelink-cas \
  --timeout=10m >/dev/null
```

Read back names and metadata only: the NativeLink ExternalSecret must be Ready
with a present refresh time and Secret resourceVersion, and the deliberate
NativeLink rollout above must be Available. Do not require a resourceVersion
change for this initial same-data projection: ESO may correctly leave an
unchanged Secret untouched. A later rotation must capture both values before
changing the OpenBao record, force-sync ESO, and require both values to advance
before restarting NativeLink.

`openssl s_client` is diagnostic only: a generic failure can be DNS, outage,
timeout, or server-CA failure and does **not** prove reader-to-writer isolation.
Activation remains blocked by issue #1551 until a fresh runner records a typed
mTLS rejection of the reader leaf on `:50051` plus a positive writer control on
`:50051`; the reader must also succeed on `:50052`. A successful
reader-to-writer handshake is a hard stop: unset the variable and rotate both
client roots before proceeding.

### DARK ARC-to-OpenBao network proof

`infra/arc/cas-network-proof.k8s.yaml` is deliberately unregistered in GitOps
and its Job is suspended. It reuses the exact general ARC policy labels and
public OpenBao CA, but carries no Kubernetes token, OpenBao token, Secret read,
or client key. Activation remains off until a later reviewed change supplies a
trusted Hubble Relay observation path; this repository does not add an unpinned
Hubble client image merely to make the declaration look executable.

Before an authorized activation, prove all of these conditions in one bounded
receipt or fail closed:

1. Talos declares `cluster.network.cni.name=none`, Cilium is Ready on every node,
   and the live Cilium ConfigMap reports `enable-policy=default`, never `never`.
   If Flannel is live, stop and rebuild/reprovision the disposable cell from the
   canonical Talos+Cilium declarations. Do not claim or attempt an in-place
   Flannel migration.
2. Service `oya-kms/openbao` has an EndpointSlice with at least one endpoint
   whose `conditions.ready` and `conditions.serving` are both `true`, whose
   `conditions.terminating` is `false` or absent, and whose ports include `8200`
   and `8202`. A connection timeout without this readback is not network-policy
   proof.
3. Unsuspend exactly Job `arc-runners/cas-network-proof-probe`. Its Pod must have
   `automountServiceAccountToken:false` and the exact three labels from the
   general ARC scale set. Preserve its start time, Pod name, logs, and finish
   time; do not exec into it.
4. The same Pod log must contain both `plaintext_8200=connection_failed` and
   `tls_8202_health=success`. The first value is a neutral observation, never a
   denial verdict: DNS, routing, outage, or timeout can all produce it.
5. Within that Pod's recorded time window, the exact raw Hubble JSON flow record
   must have both `verdict == "DROPPED"` and
   `drop_reason_desc == "POLICY_DENIED"`, correlated to the exact source Pod and
   destination TCP port `8200`; its destination IP must equal an address from
   the healthy OpenBao EndpointSlice validated in step 2. An aggregate, either
   field alone, a different destination IP, a flow from another Pod, a different
   port, or a record outside the time window is not evidence.

This DARK slice adds no verifier ServiceAccount, RBAC, `pods/log`, exec, proxy,
or port-forward authority. The current repository has no reviewed,
digest-pinned Hubble CLI image, so a trusted operator must provide an existing
read-only connection to `hubble-relay` before activation. Delete the completed
Job after the receipt is captured. This network proof does not license CAS warm
reads or remote execution.

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
