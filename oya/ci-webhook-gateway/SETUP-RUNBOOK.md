# SETUP-RUNBOOK — CI Webhook Gateway

This runbook lists **exactly what a human must provision** to make PRs against
`dev` gated by REAL automated checks and to retire the manual admin-relax-merge
seam. Every step that needs human credentials/authority is flagged
**[HUMAN-AUTH]**. Nothing here is auto-applied by the gateway; you run it.

Substrate: git + Jenkins + self-hosted Forgejo (ADR-0363). Design: ADR-0374.

---

## 0. Prerequisites (already standing per `infra/forge/README.md`)

- Self-hosted Forgejo running in namespace `oya-forge` (repo `oya-admin/oyatie`).
- Jenkins running in namespace `oya-ci-jenkins` with the `oyaCiLane` shared
  library (`infra/ci/jenkins/shared-library/vars/oyaCiLane.groovy`).
- The `forgejo-ci-token` Jenkins credential (so Jenkins can POST commit
  statuses) — created per `infra/forge/jenkins-forgejo-token.secret.template.yaml`.

If those are not up, stand them up first; this gateway is the *trigger* that
sits in front of them.

---

## 1. Generate + store the webhook HMAC secret  **[HUMAN-AUTH]**

The gateway verifies `HMAC-SHA256(secret, raw_body)` fail-closed. The SAME secret
must live in two places: Forgejo (which signs) and OpenBao (which the gateway
reads from). They MUST match exactly.

1. Generate a high-entropy secret (32 bytes hex):

   ```
   openssl rand -hex 32
   ```

2. **[HUMAN-AUTH]** Store it in OpenBao at the canonical path (the gateway reads
   it from here via External Secrets Operator → env `OYA_FORGEJO_WEBHOOK_SECRET`):

   ```
   bao kv put secret/oya/ci/forgejo-webhook-secret value='<SECRET_FROM_STEP_1>'
   ```

   The canonical reference is `sref://openbao/oya/ci/forgejo-webhook-secret`
   (ADR-0043 SecretReference contract). Do NOT commit the secret; gitleaks (run
   in the Jenkins lane) blocks it.

3. Confirm the ExternalSecret that projects it into the gateway namespace
   (`oya-ci`) exists and syncs (see §4 deploy).

---

## 2. Register the Forgejo webhook  **[HUMAN-AUTH]**

**[HUMAN-AUTH]** In Forgejo, as a repo admin of `oya-admin/oyatie`:

- Repo → **Settings → Webhooks → Add Webhook → Forgejo (Gitea-compatible)**.
- **Target URL**: the gateway's reachable address, e.g.
  `https://ci-webhook-gateway.<your-domain>/webhook/forgejo`
  (in-cluster: `http://ci-webhook-gateway.oya-ci.svc.cluster.local:8099/webhook/forgejo`).
  Note: Forgejo's `webhook.ALLOWED_HOST_LIST` must permit the gateway host —
  `infra/forge/forgejo.yaml` currently allows `*.cluster.local,oya-ci-jenkins`;
  add the gateway host if you front it with an external URL.
- **HTTP Method**: POST. **Content type**: `application/json`.
- **Secret**: paste the SAME secret from §1.
- **Trigger events**: choose **Custom events → Pull Request** (opened,
  reopened, synchronized). The gateway ignores everything else safely.
- **Active**: checked. Save, then **Test Delivery** — you should get a `200`
  `ignored` (ping) or, on a real PR, a `202` `dispatched`.

The closed router table (`src/event.rs`) only acts on `pull_request` against the
gated branch; a `ping` returns 200, an unknown event returns 422.

---

## 3. Enable commit signing so `required_signatures` is REAL  **[HUMAN-AUTH]**

`infra/branch-protection/dev.json` sets `required_signatures: true`. Today the
relax-merge hack sidesteps this. To satisfy it with REAL signed commits
(Ed25519 per the ADR-0039 signed-commits discipline):

### 3a. Each committer (human + agent identity) generates a signing key **[HUMAN-AUTH]**

```
ssh-keygen -t ed25519 -C "agent-or-human@oyatie" -f ~/.ssh/oya_signing_ed25519
```

Configure git to sign every commit with it:

```
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/oya_signing_ed25519.pub
git config --global commit.gpgsign true
git config --global tag.gpgsign true
```

### 3b. **[HUMAN-AUTH]** Register the PUBLIC key in Forgejo

Forgejo → **Settings → SSH / GPG Keys → Add Key → Signing Keys** — paste
`~/.ssh/oya_signing_ed25519.pub`. Forgejo then marks commits signed by that key
as **Verified**, which is what `required_signatures` consults.

### 3c. Confirm

Push a signed commit to a branch and open a test PR; Forgejo should show the
commit as **Verified**. With every commit verified, branch protection's
signed-commit requirement is met **without** disabling `enforce_admins`.

> The gateway removes the *required-checks* half of the relax-merge hack;
> commit signing removes the *signed-commit* half. After both, `dev` merges on
> real signed-green, no admin override.

---

## 4. Deploy the gateway  **[HUMAN-AUTH]**

The gateway is one small stateless pod in namespace `oya-ci`. Provide:

- `OYA_FORGEJO_WEBHOOK_SECRET` — from the ExternalSecret projecting
  `sref://openbao/oya/ci/forgejo-webhook-secret` (§1).
- `OYA_JENKINS_DISPATCH_URL` — the Jenkins endpoint that kicks `oyaCiLane`,
  e.g. the Generic Webhook Trigger or build-token URL:
  `http://jenkins.oya-ci-jenkins.svc.cluster.local:8080/generic-webhook-trigger/invoke?token=<JOB_TOKEN>`.
  **[HUMAN-AUTH]** Create the Jenkins job token (Jenkins → job → Configure →
  Build Triggers → Generic Webhook Trigger / Trigger builds remotely → token).
- `OYA_GATEWAY_TARGET_BRANCH` — defaults to `dev` (override only if gating a
  different branch).
- `OYA_GATEWAY_BIND_ADDR` — defaults to `0.0.0.0:8099`.

Build + run locally to smoke-test:

```
cd microservices/ci-webhook-gateway
cargo build
OYA_FORGEJO_WEBHOOK_SECRET='<secret>' \
OYA_JENKINS_DISPATCH_URL='http://localhost:8080/generic-webhook-trigger/invoke?token=t' \
cargo run
# then: curl localhost:8099/healthz  -> {"status":"ok"}
```

Container/Helm packaging follows the per-microservice `iac/k8s/helm/` convention
(ADR-0349); the deployment must mount the ExternalSecret as the env var above
and expose port 8099 behind a Service (and an ingress if Forgejo posts from
outside the cluster).

> **Liveness/readiness**: probe `GET /healthz`.
> **Fail-closed proof**: if `OYA_FORGEJO_WEBHOOK_SECRET` is unset the gateway
> logs a WARN and rejects every delivery with 503 — it will not accept unsigned
> traffic. Provision the secret before going live.

---

## 5. Rotate the webhook secret  **[HUMAN-AUTH]**

Per the ADR-0112 90-day rotation guidance (carried forward):

1. Generate a new secret (`openssl rand -hex 32`).
2. **[HUMAN-AUTH]** Update OpenBao (`bao kv put …`) AND the Forgejo webhook
   Secret field — **in the same change window** (they must match; a mismatch
   causes 401s until re-synced).
3. Restart/roll the gateway pod so it re-reads the env (or rely on the
   ExternalSecret refresh + pod restart policy).

---

## 6. Cutover checklist (retire the relax-merge seam)  **[HUMAN-AUTH]**

- [ ] Webhook secret in OpenBao + Forgejo (match).
- [ ] Forgejo `pull_request` webhook registered + Test Delivery green.
- [ ] Gateway deployed; `/healthz` green; `OYA_JENKINS_DISPATCH_URL` set.
- [ ] A real test PR against `dev` → gateway 202 → Jenkins runs the
      `oya-ci-gate` job (ADR-0380 D3; declared in JCasC per PR #242) → a single
      `oya-ci-gate` status context posted to Forgejo (`pending` → `success` /
      `failure`) + `oya-pr-review` from the reviewer. (The older
      14-status-context design from ADR-0359 colima-era is superseded by the
      single-overarching-context Talos design — see ADR-0380 amendment (5) and
      `infra/ci/jenkins/Jenkinsfile-oya-ci-gate`.)
- [ ] Every committer's Ed25519 signing key registered in Forgejo (commits show
      Verified).
- [ ] Forgejo **auto-merge** ("merge when checks succeed") enabled on the repo.
- [ ] Confirm a green PR auto-merges **without** toggling `enforce_admins`.
- [ ] Stop using the admin-merge path (the seam is now retired).

> NOTE — the reviewer gate (Intelligence-service CI stage, ADR-0367 D2) and the
> merge-queue (ADR-0111) are NOT yet built; the gateway returns a typed 501 for
> them and they are tracked in `registry/placeholder-debt/adr-follow-ups.yaml`
> (`adr-0374-reviewer-gate-dispatch`, `adr-0374-merge-queue-admit`). Until the
> reviewer stage exists, `oya-pr-review` is posted by the current reviewer agent
> as it is today; the gateway does not regress that.

---

## 7. ADR-0380 D4 — mint and project the `forgejo-ci-token`  **[HUMAN-AUTH]**

The `oya-ci-gate` Jenkinsfile (ADR-0380 D3; `infra/ci/jenkins/Jenkinsfile-oya-ci-gate`)
posts commit-status via `httpRequest` against the Forgejo statuses API, authed
by a Jenkins credential `forgejo-ci-token` (declared in JCasC `oya-ci-credentials`
configScript, PR #242). The credential's value is materialized from the
controller pod's `FORGEJO_CI_TOKEN` env, which is projected from a Kubernetes
Secret `forgejo-ci-token` in `oya-ci-jenkins`. **That Secret does not exist
yet** — D4 mints it.

### 7a. Mint the token in-pod  **[HUMAN-AUTH]**

```sh
# Exec into the Forgejo pod and mint a per-CI access token. `oya-admin` is the
# repo-admin identity already used by the existing `gateway-build-git` credential.
kubectl -n oya-forge exec -it deploy/forgejo -- \
  forgejo admin user generate-access-token \
    --username oya-admin \
    --scopes write:repository \
    --token-name oya-ci-gate-status-poster
```

Forgejo prints the token to stdout — **once**. Capture it; it is not retrievable
later (Forgejo only stores a hash). If lost, regenerate.

### 7b. Project it as a Kubernetes Secret  **[HUMAN-AUTH]**

```sh
kubectl -n oya-ci-jenkins create secret generic forgejo-ci-token \
  --from-literal=token='<TOKEN_FROM_STEP_7a>'
```

Once the Secret exists, the Jenkins controller pod must restart so the
`containerEnv` projection (PR #242, `infra/ci/jenkins/values-local.yaml`) re-
binds `FORGEJO_CI_TOKEN` to the new Secret value. JCasC then re-materializes the
`forgejo-ci-token` Jenkins credential at boot:

```sh
kubectl -n oya-ci-jenkins rollout restart statefulset/oya-jenkins
kubectl -n oya-ci-jenkins rollout status  statefulset/oya-jenkins --timeout=300s
```

### 7c. Verify  **[HUMAN-AUTH]**

```sh
# In the Jenkins UI: Manage Jenkins → Credentials → System → Global. Confirm
# `forgejo-ci-token` shows as a String credential (not the literal placeholder).
# Then: trigger a build manually:
JENKINS_URL=http://oya-jenkins.oya-ci-jenkins.svc.cluster.local:8080
curl -fsS "$JENKINS_URL/generic-webhook-trigger/invoke?token=oya-ci-gate" \
  -H 'Content-Type: application/json' \
  -d '{"pull_request":{"number":1,"head":{"sha":"<TEST_SHA>","ref":"refs/heads/test"}},"repository":{"name":"oyatie","owner":{"login":"oya-admin"}}}'
# The Jenkinsfile's `Post pending status` stage should succeed (not fail at
# withCredentials). Confirm the test status appears on the SHA in Forgejo.
```

If the `Post pending status` stage fails with `CredentialNotFoundException`,
the Secret projection or JCasC binding didn't take effect — verify the Secret
exists, that the controller pod's `env` shows `FORGEJO_CI_TOKEN`, and that
the credential is materialized in Jenkins UI.

---

## 8. ADR-0380 D5 — cutover (retiring the admin-merge seam)  **[HUMAN-AUTH]**

After §1–§7 are green AND a real test PR completes one full cycle:
gateway → Jenkins → `oya-ci-gate` posts `success` → reviewer-agent posts
`oya-pr-review` → Forgejo auto-merges on green.

### 8a. Verify the loop end-to-end (before cutting over)

- [ ] Open a trivial test PR against `dev` (e.g., comment-only change in a
      docs file).
- [ ] Forgejo webhook delivery shows 202 in the Forgejo UI.
- [ ] The CI webhook gateway pod logs `signature: verified` (HMAC §1 match).
- [ ] Jenkins shows `oya-ci-gate` build #N triggered with the PR_NUMBER /
      PR_SHA / REPO_OWNER / REPO_NAME variables.
- [ ] The build runs `cargo build -p oya-dev-cli --release` then
      `./target/release/oya gate run-all`.
- [ ] On a green gate, Forgejo shows the `oya-ci-gate` status as **success**
      on the PR's SHA.
- [ ] Forgejo auto-merge (enabled in §6) merges the PR on green without
      `--admin` override.

### 8b. Flip the dev-branch protection back to strict  **[HUMAN-AUTH]**

The current relax (per memory `oya-dev-branch-protection-merge`) tolerates a
no-CI-status state on `dev` by allowing admin-merge. Once §8a passes:

- Re-enable any temporarily-relaxed required status checks on `dev` that the
  oya-ci-gate context now satisfies (`infra/branch-protection/dev.json` is the
  source of truth; add `oya-ci-gate` to `required_status_checks.contexts`).
- Stop using the admin-merge path. Document this transition in a follow-up
  commit that updates the `oya-dev-branch-protection-merge` memory from
  "ACTIVE seam" to "RETIRED on <date>".

### 8c. Update the memory record  **[HUMAN-AUTH]**

```sh
# Update ~/.claude/projects/-Users-jasonlee-Developer-source/memory/oya-dev-branch-protection-merge.md
# Add a section header "Retired: YYYY-MM-DD" with the date of the green-merge
# verification PR. Keep the prior content for audit; do not delete.
```

The memory's `description:` field can be updated to reflect post-cutover
state (e.g., "RETIRED YYYY-MM-DD — dev merges now gate on `oya-ci-gate` green
status; admin-merge no longer used"). Memory index in `MEMORY.md` may also
need a description tweak.

> Until §8a and §8b are done, the relax-merge memory remains ACTIVE — this
> runbook section documents the planned transition, not its completion.

---

## 9. Production deploy on Talos  **[HUMAN-AUTH]**

The ci-webhook-gateway ships as a Helm chart managed by ArgoCD on the Talos
local substrate (ADR-0387). Once the gateway is running and Forgejo webhooks
flow through it, the admin-merge bridge described in §8 can be permanently
retired.

### 9a. Chart location

```
microservices/ci-webhook-gateway/iac/k8s/helm/
```

The chart follows the per-microservice flat layout (ADR-0131) with templates
for Deployment, Service, ServiceAccount, ExternalSecret (ESO → OpenBao),
NetworkPolicy, and HTTPRoute (Istio Gateway). Service port is **8081**.

### 9b. ArgoCD ApplicationSet

```
microservices/cloud-iac/iac/oyatie-cloud-provider/argocd/apps/ci-webhook-gateway-applicationset.yaml
```

This ApplicationSet generates Applications for `dev`, `staging`, and `prod`
environments. Sync policy is `automated + selfHeal + prune`. Destination
namespace is `ci-webhook-gateway` (ArgoCD creates it with the correct Istio
ambient-mode labels).

ArgoCD syncs the chart from `dev` branch for the dev environment and `main`
for staging/prod. Image digests are passed as Helm parameters; cosign
verification is required (`image.cosign.required: true`, ADR-0181).

### 9c. Provision secrets before first sync

Run **`microservices/ci-webhook-gateway/runbooks/provision-secrets.md`** in
full before allowing ArgoCD to sync. The ExternalSecret will fail to sync
(and the pod will fail to start) if `secret/oya/ci-webhook-gateway` does not
exist in OpenBao with the three required keys:
`forgejo_ed25519_pub`, `jenkins_api_token`, `github_token`.

### 9d. Verify ArgoCD sync

```sh
# Watch the ApplicationSet generate Applications
kubectl -n oya-cd-argocd get applications -l app.kubernetes.io/name=ci-webhook-gateway

# Check the dev Application health
kubectl -n oya-cd-argocd get application ci-webhook-gateway-dev -o jsonpath='{.status.health.status}'

# Check the ESO secret sync
kubectl -n ci-webhook-gateway get externalsecret ci-webhook-gateway-secrets

# Check the pod is running
kubectl -n ci-webhook-gateway get pods -l app.kubernetes.io/name=ci-webhook-gateway

# Smoke-test the health endpoint
kubectl -n ci-webhook-gateway exec -it deploy/ci-webhook-gateway -- \
  curl -fsS http://localhost:8081/healthz
```

Expected output from `/healthz`: `{"status":"ok"}`.

### 9e. Retire the admin-merge seam

Once §9d passes and a real test PR completes one full cycle (§8a), proceed
with §8b and §8c to flip dev branch-protection back to strict and stop using
the admin-merge path.
